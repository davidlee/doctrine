// SPDX-License-Identifier: GPL-3.0-only
//! The priority VIEW layer (SL-047 §5.4) — the structured reasons that are the
//! render SOURCE OF TRUTH (REQ-072 AC3).
//!
//! These types carry the COMPUTED classification of each surfaced node: its
//! actionability, its multi-dimensional score, its direct blockers, and a
//! `Vec<ReasonKind>` of the structured reasons behind the verdict. The human table
//! and the `--json`
//! output ([`super::render`]) are produced *from* these types — never recomputed in
//! the renderer. A reason is built ONCE, here (or in the surface shell that fills
//! these rows from the pure [`super::channels`] signals), so the two render targets
//! cannot drift.
//!
//! Pure data: no clock, RNG, or disk. The surface shell ([`super::surface`]) reads
//! the graph + titles and fills these rows; the renderer only formats them.

use serde::Serialize;

use super::partition::StatusClass;
use crate::backlog_order::OverrideReason;
use crate::comparison::ClaimTier;

/// One structured reason behind a node's classification (design §5.4). The render
/// SOURCE OF TRUTH — every human line and `--json` reason field is produced from a
/// `ReasonKind`, never recomputed (REQ-072 AC3). Refs are canonical `KIND-NNN`
/// strings (the opaque cordage ids never escape — re-mapped in the surface shell).
///
/// NOT `Eq` — the `Score` arm carries `f64` dimensions (SL-133); `PartialEq` suffices
/// for the golden/equivalence assertions (no `ReasonKind` is a map key).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReasonKind {
    /// The node's eligibility verdict: its authored status + the class it landed in
    /// (`Workable` ⇒ eligible; `Terminal`/`Unrecognised` ⇒ not). `status` is `None`
    /// for the status-less REC kind.
    Eligibility {
        status: Option<String>,
        class: StatusClass,
    },
    /// The node is blocked by these (non-terminal) prerequisites (direct, or the
    /// transitive chain for `explain`/`--transitive`).
    BlockedBy { items: Vec<String> },
    /// The node is blocking these dependents (direct, or transitive).
    Blocking { items: Vec<String> },
    /// The node's multi-dimensional **score** breakdown (SL-133 §5.4) — `base`
    /// (`value_dim + risk_dim`) plus the recursive `leverage` and the one-hop
    /// `optionality`, summing to `total`. THIS field order is pinned (EX-1 / VA-1).
    Score {
        base: f64,
        value_dim: f64,
        risk_dim: f64,
        leverage: f64,
        optionality: f64,
        total: f64,
    },
    /// A soft `after` edge cordage evicted to linearize — the honest record
    /// (`from → to`, with the cordage reason re-expressed in the shared vocabulary).
    EvictedEdge {
        from: String,
        to: String,
        reason: OverrideReason,
    },
    /// The node sits in a diagnosed dep cycle — its order degraded to the fallback
    /// rather than a false topological order (REQ-076 / F2).
    CycleDegraded { nodes: Vec<String> },
    /// SL-220 PHASE-05 value-source shape (design §3 rung 1, D11) — the value
    /// is an operator PIN (an anchored claim, `admission = pin`). `conflict`
    /// names the other class(es) this claim's compiled anchor was found to
    /// violate order against (an `AnchorConflict` citation) — empty when
    /// none. Replaces the retired `ValueAuthored` (no code path emits an
    /// `authored` value source post-flip).
    ValuePin {
        value: f64,
        conflict: Vec<String>,
        /// SL-220 PHASE-06 render attribution (design §6): a pin renders
        /// `pin (by, date, basis N)` when singleton. `None` fields drop from
        /// the parenthetical; `contested` supersedes attribution entirely.
        by: Option<String>,
        date: Option<String>,
        basis: Option<String>,
        /// Present ⇔ the winning (Pin) tier disagreed on magnitude — the
        /// "contested pin" render (interval + row count), never attribution.
        contested: Option<ContestedClaim>,
    },
    /// SL-220 PHASE-05 value-source shape — a resolved ledgered claim:
    /// rung 1 (`tier = Human`, anchoring compile) or rungs 3–4
    /// (`tier = Agent | Migrated`, the below-projection priors — their
    /// `conflict` citation is empty by construction: priors never anchor
    /// compile, D4). SL-220 PHASE-06 threads render attribution: `by`/`date`
    /// (the migrated tier reads `date` as its `observed_at` timestamp — the
    /// render frames it as `observed`, and a `None` `by` renders
    /// `unattributed`).
    ValueClaim {
        value: f64,
        tier: ClaimTier,
        conflict: Vec<String>,
        by: Option<String>,
        date: Option<String>,
        contested: Option<ContestedClaim>,
    },
    /// SL-222 PHASE-09 (deletion) — a `[value]` top-level key survives on the
    /// entity's TOML. The raw facet read model has been deleted; this is a
    /// magnitude-free presence tripwire. The exit is the migration script or
    /// re-assertion via `value set --rater human`.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "SL-222 PHASE-09: used in tests")
    )]
    ValueUnmigratedFacet,
    /// SL-213 PHASE-06 value-source shape 2 — projected: budgeted
    /// interpolation between order neighbours. `lower`/`upper` are the C6
    /// display bounds (`None` = unbounded that side); `human`/`agent` are the
    /// constraining-judgement rater split (the T7 disclosure; `NoConstraint`
    /// rows excluded).
    ValueProjected {
        value: f64,
        lower: Option<f64>,
        upper: Option<f64>,
        human: u32,
        agent: u32,
    },
    /// SL-213 PHASE-06 value-source shape 3 — gauge: placed by the P7/P8
    /// convention, not evidence. `judgements` is the constraining-judgement
    /// count (human + agent) that ordered it.
    ValueGauge { value: f64, judgements: u32 },
    /// SL-222 PHASE-06 cost-source shape 1 — the `est_cost` came from a
    /// PIN-tier anchored claim (the operator pin, D2). Carries the claim's
    /// bounds `(lower, upper, β)`, attribution, and the `basis` ref.
    /// `contested` is present when the winning Pin tier internally disagreed
    /// on magnitude.
    CostPin {
        est_cost: f64,
        lower: f64,
        upper: f64,
        beta: f64,
        by: Option<String>,
        date: Option<String>,
        basis: Option<String>,
        contested: Option<ContestedClaim>,
    },
    /// SL-222 PHASE-06 cost-source shape 2 — a resolved ledgered claim
    /// (Human/Agent/Migrated). Rung 1 (Human, anchoring compile) or rungs
    /// 3–4 (Agent/Migrated, the below-projection priors). The `conflict`
    /// citation is non-empty for anchored-tier (Pin/Human) cross-class
    /// anchor-conflicts; priors never anchor compile so they carry empty.
    /// `tier` discriminates Human (anchored) from Agent/Migrated (priors).
    /// Migrated claims carry `date` as their `observed_at` timestamp.
    CostClaim {
        est_cost: f64,
        lower: f64,
        upper: f64,
        beta: f64,
        tier: ClaimTier,
        by: Option<String>,
        date: Option<String>,
        conflict: Vec<String>,
    },
    /// SL-222 PHASE-06 cost-source shape 3 — the entity inherited its
    /// cost from the class anchor by an `equal` sizing merge (provenance
    /// Authored in the est projection). The item is facet-less; its cost
    /// was set by the class-anchored member.
    CostClassAnchor { est_cost: f64 },
    /// SL-222 PHASE-09 (deletion) — a `[estimate]` top-level key survives on
    /// the entity's TOML. The raw facet read model has been deleted; this is a
    /// magnitude-free presence tripwire. The exit is the migration script or
    /// re-assertion via `estimate set --rater human`.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "SL-222 PHASE-09: used in tests")
    )]
    CostUnmigratedFacet,
    /// SL-222 PHASE-06 cost-source shape 5 — projected: the deterministic
    /// point projection in an anchored est component. `lower`/`upper` are the
    /// C6 display bounds (`None` = unbounded that side); `human`/`agent` are
    /// the constraining sizing-judgement rater split (the T7 disclosure;
    /// `NoConstraint` rows excluded — S3 precedent).
    CostProjected {
        est_cost: f64,
        lower: Option<f64>,
        upper: Option<f64>,
        human: u32,
        agent: u32,
    },
    /// SL-222 PHASE-06 cost-source shape 6 — the bare anchor `max_upper +
    /// margin` (D7): the divisor a bare item with no sizing evidence takes.
    /// `max_estimate` is `None` in the empty-corpus fallback (`est_cost` is
    /// then the `1.0` default, no authored upper to cite).
    CostBareAnchor {
        est_cost: f64,
        max_estimate: Option<f64>,
        margin: f64,
    },
    /// SL-222 PHASE-06 gauge flag (D2 honesty) — the item sits in a gauge est
    /// component (ordered by `judgements`, no anchor in the component), so
    /// scoring used the BARE ANCHOR (`est_cost`/`max_estimate`/`margin`, the
    /// shape-3 fields); the render NEVER implies gauge fed the divisor. Emits
    /// TWO lines: the bare-anchor cost-source + the `sizing: gauge …`
    /// disclosure.
    CostGauge {
        est_cost: f64,
        max_estimate: Option<f64>,
        margin: f64,
        judgements: u32,
    },
    /// SL-213 PHASE-06 (design §4 S4) — the inert `priority`-domain
    /// disclosure: `count` prefer-first judgements recorded corpus-wide.
    /// NOT a finding (nothing is wrong) — `explain`-only, corpus-global (not
    /// entity-scoped).
    PriorityDomainDisclosure { count: usize },
    /// SL-218 D2 — the `[priority.compare] demote_agent_evidence` knob is on:
    /// determinacy verdicts are read over the human-rows-only system. Knob
    /// state, not entity evidence — present only when on, so knob-off
    /// surfaces stay byte-identical (INV-1).
    AgentEvidenceDemoted,
    /// SL-218 PHASE-03 (design §3) — one rendered frontier tension: `preferred`
    /// outranks `surfaced` on `value_dim` yet `surfaced` surfaces first. Canonical
    /// ids (the surface shell maps `tension::Tension`'s `EntityKey`s here). The
    /// render SOURCE OF TRUTH for every tension line — `reason_line` (human) and
    /// `reason_json` format from this, never recompute (REQ-072 AC3).
    Tension {
        preferred: String,
        surfaced: String,
        cause: TensionCauseView,
        grade: TensionGradeView,
    },
    /// SL-218 PHASE-03 (design §2 / F-6) — the page-scoped m=0 disclosure: `count`
    /// value inversions were excluded because a member is value-insensitive
    /// (`m = 0`). Present only when `count > 0` (SL-217 D6 scoped-disclosure).
    ZeroWeightExcluded { count: usize },
}

impl ReasonKind {
    /// The `value_source` provenance token (SL-220 D11) — the SINGLE source
    /// for every surface that names where a value came from (explain JSON,
    /// the elicit participant value block; PHASE-06's row markers and `show`
    /// line consume the same map). A **disclosed breaking token-set change**:
    /// `authored` is REMOVED (no code path emits it post-flip — the variant
    /// is retired); `pin` / `human-claim` / `agent-claim` / `migrated-claim` /
    /// `unmigrated-facet` are ADDED; `projected` / `gauge` are byte-stable,
    /// and the scoring floor stays a render *marker*, never a citable source
    /// (no `default` token minted here — byte-stable by absence). Pinned by
    /// the post-flip vocabulary golden below.
    pub(crate) fn value_source_token(&self) -> Option<&'static str> {
        match self {
            ReasonKind::ValuePin { .. } => Some("pin"),
            ReasonKind::ValueClaim { tier, .. } => Some(match tier {
                // Total over the tier enum; construction routes Pin through
                // `ValuePin`, so this arm is a belt, not a second source.
                ClaimTier::Pin => "pin",
                ClaimTier::Human => "human-claim",
                ClaimTier::Agent => "agent-claim",
                ClaimTier::Migrated => "migrated-claim",
            }),
            ReasonKind::ValueUnmigratedFacet => Some("unmigrated-facet"),
            ReasonKind::ValueProjected { .. } => Some("projected"),
            ReasonKind::ValueGauge { .. } => Some("gauge"),
            _ => None,
        }
    }

    /// The `cost_source` provenance token (SL-222 PHASE-06, design §5) — the
    /// SINGLE source for every surface that names where a cost came from.
    /// A **disclosed breaking token-set change**: `authored` is removed;
    /// `pin` / `human-claim` / `agent-claim` / `migrated-claim` /
    /// `class-anchor` / `unmigrated-facet` are added; `projected` / `gauge` /
    /// `bare-anchor` are byte-stable. Pinned by the post-flip vocabulary
    /// golden below.
    pub(crate) fn cost_source_token(&self) -> Option<&'static str> {
        match self {
            ReasonKind::CostPin { .. } => Some("pin"),
            ReasonKind::CostClaim { tier, .. } => Some(match tier {
                ClaimTier::Pin => "pin",
                ClaimTier::Human => "human-claim",
                ClaimTier::Agent => "agent-claim",
                ClaimTier::Migrated => "migrated-claim",
            }),
            ReasonKind::CostClassAnchor { .. } => Some("class-anchor"),
            ReasonKind::CostUnmigratedFacet => Some("unmigrated-facet"),
            ReasonKind::CostProjected { .. } => Some("projected"),
            ReasonKind::CostGauge { .. } => Some("gauge"),
            ReasonKind::CostBareAnchor { .. } => Some("bare-anchor"),
            _ => None,
        }
    }
}

/// SL-220 PHASE-06 render carrier for a same-tier CLAIM conflict (design §6,
/// the "contested" variant). DISTINCT from the anchor-conflict citation
/// (`conflict: Vec<String>`, a cross-class order violation): this is the
/// disagreement WITHIN the winning tier's rows. `rows` is the active
/// winning-tier row count ("N claims"); `low`/`high` bound the rendered
/// interval. Sourced from `ResolvedClaim.{conflict, rows}`.
///
/// NOT `Eq` — `low`/`high` are `f64`; `PartialEq` carries the golden assertions.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContestedClaim {
    pub(crate) low: f64,
    pub(crate) high: f64,
    pub(crate) rows: u32,
}

/// The render-facing cause of a [`ReasonKind::Tension`] (design §3). Structure
/// cites the surviving edge (its keyword + tail drive the callout); Composition
/// cites the full-score component deltas (surfaced − preferred).
///
/// NOT `Eq` — Composition carries `f64` deltas; `PartialEq` suffices.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TensionCauseView {
    /// A surviving seq/dep edge forces the inversion. `edge_from` is the cited
    /// constraint's tail (the surfaced member — the first forward hop leaves it).
    Structure { edge_from: String, verb: EdgeVerb },
    /// No structural path — full-score dimensions lift the surfaced member.
    Composition {
        risk_dim: f64,
        leverage: f64,
        optionality: f64,
    },
}

/// Which precedence overlay a Structure edge rode in on — drives the callout
/// keyword (`after` vs `needs`) and tail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeVerb {
    /// A `seq` (`after`) edge — tail "sequence survives".
    After,
    /// A `dep` (`needs`) edge — tail "holds".
    Needs,
}

/// The render-facing evidence grade of a [`ReasonKind::Tension`] (design D6).
/// Counts come from the system that produced the verdict (RV-271 F-2/F-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TensionGradeView {
    /// The verdict system determines the order — that system's constraining
    /// rater split.
    Determined { human: u32, agent: u32 },
    /// Knob-on: the full system determines, the human system does not — an
    /// unconfirmed agent proposal. `agent` is the full system's agent split.
    AgentProposed { agent: u32 },
    /// No determining evidence in any consulted system.
    Projected,
}

/// Whether an eligible node is ready to start now, or held by a blocker (design
/// §5.4). EVERY survey row is eligible; the variant splits actionable from blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Actionability {
    /// Eligible AND unblocked — ready to start.
    Actionable,
    /// Eligible but held by at least one non-terminal direct blocker.
    Blocked,
}

impl Actionability {
    /// The JSON token for the actionability axis.
    pub(crate) fn token(self) -> &'static str {
        match self {
            Actionability::Actionable => "actionable",
            Actionability::Blocked => "blocked",
        }
    }
}

/// One `survey` row (design §5.4) — an eligible node with its importance signals and
/// structured reasons. The set is all eligible nodes (terminal excluded unless
/// `--all`); both [`Actionability`] variants appear (the divergence feature — a
/// blocked-but-workable item still leads importance order, D10).
///
/// NOT `Eq` — `score` is `f64` (SL-133); `PartialEq` carries the golden assertions.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SurveyRow {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) act: Actionability,
    /// The node's multi-dimensional score (SL-133) — the display sort key.
    pub(crate) score: f64,
    /// Direct blockers (canonical refs) — empty for an actionable row.
    pub(crate) blockers: Vec<String>,
    pub(crate) reasons: Vec<ReasonKind>,
}

/// One `next` row (design §5.4) — an ACTIONABLE node only (blocked items are absent,
/// the divergence feature). Ordered by the score-aware induced-frontier sort over the
/// surviving seq edges (SL-133 §5.4). Carries its blocking set (what it unblocks) for
/// the advisory display; blockers is empty by construction.
///
/// NOT `Eq` — `score` is `f64` (SL-133); `PartialEq` carries the golden assertions.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NextRow {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) act: Actionability,
    /// The node's multi-dimensional score (SL-133) — the frontier ready-set priority.
    pub(crate) score: f64,
    pub(crate) reasons: Vec<ReasonKind>,
    pub(crate) blockers: Vec<String>,
    pub(crate) blocking: Vec<String>,
    /// SL-222 PHASE-09: the cost-source reason from the comparison ladder
    /// (the `explain` cost block). `None` for a non-value-bearing kind or an
    /// est-inactive corpus. Mirrors `value_source`.
    pub(crate) cost_source: Option<ReasonKind>,
    /// SL-220 PHASE-06 (design §6) — the RESOLVED value-source reason from the
    /// comparison ladder (the same input `explain` consumes via
    /// [`super::surface::value_source_reason`]), NOT the raw authored facet. The
    /// value cell renders its magnitude + a per-rung source marker. `None` for a
    /// value-bearing kind with no evidence (the scoring floor — the cell shows
    /// the marked default) or a valueless kind (absent cell). The facet reader it
    /// replaces died at PHASE-06 (EX-3 grep-gate).
    pub(crate) value_source: Option<ReasonKind>,
    /// Authored tags (SL-171 PHASE-01) — empty when no tags authored.
    pub(crate) tags: Vec<String>,
}

/// The `next` command's full render model (SL-218 PHASE-03) — the actionable
/// rows plus the frontier's graded tensions and the page's m=0 scoped
/// disclosure. The tension list is the FULL frontier (design §3: JSON uncapped);
/// the renderer page-filters to the visible rows and caps the human callout
/// block. `tensions` are all [`ReasonKind::Tension`]; `zero_weight` is
/// `Some(ReasonKind::ZeroWeightExcluded)` only when exclusions occurred.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NextView {
    pub(crate) rows: Vec<NextRow>,
    pub(crate) tensions: Vec<ReasonKind>,
    pub(crate) zero_weight: Option<ReasonKind>,
}

/// The `blockers <ID>` result (design §5.4 / REQ-073) — the node's direct (or
/// `--transitive`) blocked-by set and blocking set, in canonical refs. Display depth
/// (`transitive`) is a presentation flag carried for the renderer; it NEVER reorders
/// (both lists are canonical-id sorted regardless).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockersView {
    pub(crate) id: String,
    pub(crate) transitive: bool,
    pub(crate) blocked_by: Vec<String>,
    pub(crate) blocking: Vec<String>,
}

/// The `inspect` actionability block (design §5.4 / SL-046 D1) — appended below the
/// relation view at the command layer. Carries the eligible/actionable flags, the
/// direct blockers + blocking, and the score; rendered as a trailing block.
///
/// NOT `Eq` — `score` is `f64` (SL-133); `PartialEq` carries the golden assertions.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActionabilityBlock {
    pub(crate) eligible: bool,
    pub(crate) actionable: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) blocking: Vec<String>,
    pub(crate) score: f64,
}

/// The `explain <ID>` result (design §5.4 / D11) — always walked to root: the
/// eligibility reason, the transitive blocker chain, the evicted seq edges, and the
/// score breakdown. Each field is a structured reason (or a list of them) so the
/// renderer only formats.
///
/// NOT `Eq` — the `score` reason carries `f64` (SL-133); `PartialEq` suffices.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Explanation {
    pub(crate) id: String,
    pub(crate) eligibility: ReasonKind,
    pub(crate) blocker_chain: Vec<ReasonKind>,
    pub(crate) evictions: Vec<ReasonKind>,
    pub(crate) score: ReasonKind,
    /// SL-213 PHASE-06 (design §4 S3) — the comparison-tier value-source
    /// block. `None` for a non-value-bearing kind.
    pub(crate) value_source: Option<ReasonKind>,
    /// SL-219 PHASE-06 (design §5) — the comparison-tier cost-source block,
    /// beside the value-source block. `None` when the corpus carries no
    /// est-domain projection (byte-identical to pre-SL-219 explain, S3
    /// precedent) or the kind is not value-bearing (no divisor consumed).
    pub(crate) cost_source: Option<ReasonKind>,
    /// SL-213 PHASE-06 (design §4 S4) — the inert priority-domain
    /// disclosure, corpus-global. `None` when no `priority`-domain rows
    /// exist.
    pub(crate) priority_disclosure: Option<ReasonKind>,
    /// SL-218 D2 — the agent-demotion disclosure. `None` when the knob is
    /// off (surfaces byte-identical to shipped behaviour).
    pub(crate) agent_demotion: Option<ReasonKind>,
    /// SL-218 PHASE-03 — the frontier tensions involving this id (design §3),
    /// both classes, already converted to the render arm and filtered to this id
    /// (`surfaced == id || preferred == id`). Empty when the id has none.
    pub(crate) tensions: Vec<ReasonKind>,
    /// SL-218 PHASE-03 (design §2 considered-set) — is this id on the current
    /// frontier at all? `false` ⇒ render the "not on the current frontier"
    /// disclosure instead of an (empty) tensions section.
    pub(crate) on_frontier: bool,
}

// ── SL-089 actionability-graph view types ──────────────────────────────────

/// One node in the actionability graph — the render source of truth for the
/// web UI. Carries the server-computed rank (topological layer over the dep
/// overlay) so the frontend never computes ordering.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActionabilityNode {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    /// `"actionable"` | `"blocked"` | `"terminal"`.
    pub(crate) actionability: String,
    /// The node's multi-dimensional score (SL-133) — replaces the old consequence tally.
    pub(crate) score: f64,
    /// Topological layer: 0 = no non-terminal blockers.
    pub(crate) rank: u32,
    /// Direct non-terminal blockers (canonical refs).
    pub(crate) blockers: Vec<String>,
}

/// One edge in the actionability graph.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActionabilityEdge {
    /// Canonical ref of the prerequisite.
    pub(crate) source: String,
    /// Canonical ref of the dependent.
    pub(crate) target: String,
    /// `"needs"` (hard block) | `"after"` (soft sequence).
    pub(crate) kind: String,
}

/// The full actionability graph for the web UI.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActionabilityView {
    pub(crate) kind: String,
    pub(crate) policy_version: String,
    pub(crate) nodes: Vec<ActionabilityNode>,
    pub(crate) edges: Vec<ActionabilityEdge>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EX-3 / VT-3: the FULL post-flip `value_source` token vocabulary,
    /// pinned as a set (SL-220 D11 — a disclosed breaking change, not an
    /// incremental diff). One shape per ladder rung: `pin` (rung 1, pinned),
    /// `human-claim` (rung 1), `projected`/`gauge` (rung 2, byte-stable),
    /// `agent-claim` (rung 3), `migrated-claim` (rung 4), `unmigrated-facet`
    /// (rung 5). The default floor (rung 6) mints NO token — byte-stable by
    /// absence (the scoring floor is a render marker, not a citable source).
    #[test]
    fn value_source_token_vocabulary_golden_post_flip() {
        let shapes: Vec<(ReasonKind, &str)> = vec![
            (
                ReasonKind::ValuePin {
                    value: 6.5,
                    conflict: vec![],
                    by: None,
                    date: None,
                    basis: None,
                    contested: None,
                },
                "pin",
            ),
            (
                ReasonKind::ValueClaim {
                    value: 6.2,
                    tier: ClaimTier::Human,
                    conflict: vec![],
                    by: None,
                    date: None,
                    contested: None,
                },
                "human-claim",
            ),
            (
                ReasonKind::ValueProjected {
                    value: 4.0,
                    lower: Some(3.2),
                    upper: Some(9.1),
                    human: 7,
                    agent: 2,
                },
                "projected",
            ),
            (
                ReasonKind::ValueGauge {
                    value: 1.0,
                    judgements: 3,
                },
                "gauge",
            ),
            (
                ReasonKind::ValueClaim {
                    value: 3.0,
                    tier: ClaimTier::Agent,
                    conflict: vec![],
                    by: None,
                    date: None,
                    contested: None,
                },
                "agent-claim",
            ),
            (
                ReasonKind::ValueClaim {
                    value: 3.0,
                    tier: ClaimTier::Migrated,
                    conflict: vec![],
                    by: None,
                    date: None,
                    contested: None,
                },
                "migrated-claim",
            ),
            (ReasonKind::ValueUnmigratedFacet, "unmigrated-facet"),
        ];
        let tokens: Vec<&str> = shapes
            .iter()
            .map(|(reason, _)| reason.value_source_token().expect("a value source"))
            .collect();
        let expected: Vec<&str> = shapes.iter().map(|&(_, token)| token).collect();
        assert_eq!(tokens, expected, "per-shape token map");

        // The COMPLETE vocabulary, as a set: `authored` is gone.
        let vocabulary: std::collections::BTreeSet<&str> = tokens.into_iter().collect();
        let pinned: std::collections::BTreeSet<&str> = [
            "pin",
            "human-claim",
            "agent-claim",
            "migrated-claim",
            "unmigrated-facet",
            "projected",
            "gauge",
        ]
        .into_iter()
        .collect();
        assert_eq!(vocabulary, pinned, "full post-flip token set (D11)");
        assert!(!vocabulary.contains("authored"), "authored removed (D11)");

        // A non-value-source reason mints no token.
        assert_eq!(
            ReasonKind::BlockedBy { items: vec![] }.value_source_token(),
            None
        );
    }

    /// D11 belt: a `ValueClaim` accidentally carrying the Pin tier still
    /// reads `pin` — the token map is total, never a second tier source.
    #[test]
    fn value_claim_pin_tier_reads_pin_token() {
        let reason = ReasonKind::ValueClaim {
            value: 1.0,
            tier: ClaimTier::Pin,
            conflict: vec![],
            by: None,
            date: None,
            contested: None,
        };
        assert_eq!(reason.value_source_token(), Some("pin"));
    }

    /// EX-3 / VT-3: the FULL post-flip `cost_source` token vocabulary,
    /// pinned as a set (SL-222 PHASE-06 — a disclosed breaking change).
    /// One shape per ladder rung: `pin` (rung 1), `human-claim` (rung 1),
    /// `class-anchor` (rung 2), `projected` (rung 3, byte-stable),
    /// `gauge` (rung 3, byte-stable), `agent-claim` (rung 4),
    /// `migrated-claim` (rung 5), `unmigrated-facet` (rung 6),
    /// `bare-anchor` (rung 7, byte-stable).
    #[test]
    fn cost_source_token_vocabulary_golden_post_flip() {
        let shapes: Vec<(ReasonKind, &str)> = vec![
            (
                ReasonKind::CostPin {
                    est_cost: 5.9,
                    lower: 2.0,
                    upper: 8.0,
                    beta: 0.65,
                    by: Some("david".into()),
                    date: Some("2026-07-17".into()),
                    basis: Some("ASM-014".into()),
                    contested: None,
                },
                "pin",
            ),
            (
                ReasonKind::CostClaim {
                    est_cost: 5.9,
                    lower: 2.0,
                    upper: 8.0,
                    beta: 0.65,
                    tier: ClaimTier::Human,
                    by: Some("david".into()),
                    date: Some("2026-07-17".into()),
                    conflict: vec![],
                },
                "human-claim",
            ),
            (
                ReasonKind::CostClaim {
                    est_cost: 2.7,
                    lower: 1.0,
                    upper: 4.0,
                    beta: 0.65,
                    tier: ClaimTier::Agent,
                    by: Some("claude".into()),
                    date: Some("2026-07-12".into()),
                    conflict: vec![],
                },
                "agent-claim",
            ),
            (
                ReasonKind::CostClaim {
                    est_cost: 2.1,
                    lower: 1.0,
                    upper: 3.0,
                    beta: 0.65,
                    tier: ClaimTier::Migrated,
                    by: None,
                    date: Some("2026-07-17".into()),
                    conflict: vec![],
                },
                "migrated-claim",
            ),
            (
                ReasonKind::CostClassAnchor { est_cost: 4.0 },
                "class-anchor",
            ),
            (ReasonKind::CostUnmigratedFacet, "unmigrated-facet"),
            (
                ReasonKind::CostProjected {
                    est_cost: 3.4,
                    lower: Some(2.0),
                    upper: Some(5.65),
                    human: 3,
                    agent: 1,
                },
                "projected",
            ),
            (
                ReasonKind::CostGauge {
                    est_cost: 11.0,
                    max_estimate: Some(10.0),
                    margin: 1.0,
                    judgements: 3,
                },
                "gauge",
            ),
            (
                ReasonKind::CostBareAnchor {
                    est_cost: 11.0,
                    max_estimate: Some(10.0),
                    margin: 1.0,
                },
                "bare-anchor",
            ),
        ];
        let tokens: Vec<&str> = shapes
            .iter()
            .map(|(reason, _)| reason.cost_source_token().expect("a cost source"))
            .collect();
        let expected: Vec<&str> = shapes.iter().map(|&(_, token)| token).collect();
        assert_eq!(tokens, expected, "per-shape token map");

        // The COMPLETE vocabulary, as a set: `authored` is gone.
        let vocabulary: std::collections::BTreeSet<&str> = tokens.into_iter().collect();
        let pinned: std::collections::BTreeSet<&str> = [
            "pin",
            "human-claim",
            "agent-claim",
            "migrated-claim",
            "class-anchor",
            "unmigrated-facet",
            "projected",
            "gauge",
            "bare-anchor",
        ]
        .into_iter()
        .collect();
        assert_eq!(vocabulary, pinned, "full post-flip cost_source token set");
        assert!(
            !vocabulary.contains("authored"),
            "authored removed from cost_source"
        );

        // A non-cost-source reason mints no token.
        assert_eq!(
            ReasonKind::BlockedBy { items: vec![] }.cost_source_token(),
            None
        );
    }
}
