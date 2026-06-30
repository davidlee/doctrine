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
    /// Authored estimate facet (SL-171 PHASE-01) — `None` when no estimate authored.
    pub(crate) estimate: Option<crate::estimate::EstimateFacet>,
    /// Authored value facet (SL-171 PHASE-01) — `None` when no value authored.
    pub(crate) value: Option<crate::value::ValueFacet>,
    /// Authored tags (SL-171 PHASE-01) — empty when no tags authored.
    pub(crate) tags: Vec<String>,
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
