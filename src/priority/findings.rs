// SPDX-License-Identifier: GPL-3.0-only
//! The interestingness FINDINGS catalogue (SL-194 PHASE-01) — pure detectors over a
//! built [`PriorityGraph`] that surface the *aggregate / relational* structure a flat
//! `next`/`survey` row list conspicuously cannot show: forks, joins, gating fan-out,
//! value inversions, order displacement, score plateaus, and provenance (evicted soft
//! edges + degraded dep cycles).
//!
//! Engine layer, PURE (ADR-001): no clock, RNG, or disk. Every detector is a
//! `fn(&PriorityGraph, …) -> Vec<Finding>` deriving its signal from the honest substrate
//! (the [`super::channels`] accessors + the [`super::order`] frontier primitives) — the
//! impure shell ([`super::surface::findings`]) owns the scan + build. Edge access rides
//! `channels` (the cordage `Graph` has no `edge_count`/`node_count` — accessors iterate
//! overlay edges per node), so the detectors match `survey`/`explain` eligibility and
//! terminal handling exactly rather than re-deriving edge logic.
//!
//! Direction (the trap): the dep overlay stores `needs` prereq→dependent (the B→A flip).
//! `Fork` / `GatingFanOut` read `channels::blocking` (OUT — what settling the hub
//! unblocks); `Join` reads `channels::blocked_by` (IN — its prerequisites).
//!
//! Findings are represented as an enum + accessor `impl`, mirroring the [`ReasonKind`]
//! render-source-of-truth idiom next door (`view.rs`): each variant carries its own
//! structured payload, the renderer only formats. Provenance findings REUSE the existing
//! `ReasonKind::{EvictedEdge,CycleDegraded}` variants (DRY, and the render fragment is
//! shared with `explain`).
//!
//! The β-family (`OrderInstability` / `ArmResequencing`) is PHASE-02 — contested
//! orderings surfaced by rebuilding the graph at the two β endpoints (`skew` 0 / 1 over
//! the SAME scan). [`detect`] runs them ONLY when the shell supplies a
//! `Some(&BetaEndpoints)`; with `None` (no non-terminal interval estimate) the β-family
//! stays silent (starved-until-estimates, R1).

use std::collections::{BTreeMap, BTreeSet};

use crate::backlog_order::OverrideReason;
use crate::comparison;
use crate::relation_graph::EntityKey;

use super::channels;
use super::config::PriorityConfig;
use super::graph::PriorityGraph;
use super::order;
use super::partition::{StatusClass, status_class};
use super::view::ReasonKind;

// ── Thresholds (STD-001: one named-const block, no magic literals) ──────────
//
// The six judgment knobs. The three ε knobs are SEEDED defaults (design D5 / R3),
// calibrated from the first live-corpus output — the probe itself is the instrument.

/// A fork/gating hub must open at least this many non-terminal arms to be interesting.
pub(crate) const FORK_MIN_ARMS: usize = 2;
/// A join must gather at least this many non-terminal prerequisites.
pub(crate) const JOIN_MIN_PREREQS: usize = 2;
/// A gating-class fan-out must block at least this many non-terminal dependents.
pub(crate) const GATING_MIN_BLOCKS: usize = 2;
/// Adjacent scores within this epsilon are a near-tie (a plateau step). SEEDED.
pub(crate) const PLATEAU_EPS: f64 = 0.01;
/// A value inversion fires only when `base(blocked) − base(blocker)` exceeds this
/// positive gap — filters sub-noise inversions. SEEDED.
pub(crate) const INVERSION_MIN_GAP: f64 = 0.5;
/// A node must move at least this many positions between the constraint-bearing survey
/// order and the constraint-free score order to count as displaced. SEEDED.
pub(crate) const DISPLACEMENT_MIN_DELTA: usize = 3;
/// The β sweep endpoints (design D4 / SL-172): `skew = BETA_LO` → cost = lower
/// (optimistic), `skew = BETA_HI` → cost = upper (pessimistic). The `{0, 1}` sample is
/// probe-grade — a finer flip-β grid is the deferred refinement (R5).
pub(crate) const BETA_LO: f64 = 0.0;
pub(crate) const BETA_HI: f64 = 1.0;

// ── The catalogue ───────────────────────────────────────────────────────────

/// One interestingness finding (design §The catalogue). Each variant carries its own
/// structured payload (canonical `KIND-NNN` refs, never opaque cordage ids); the render
/// layer formats it. `Provenance` wraps the existing [`ReasonKind`] provenance variants.
///
/// PHASE-01 subset — the β-family (`OrderInstability` / `ArmResequencing`) is PHASE-02.
///
/// NOT `Eq` — `ValueInversion.gap` / `Plateau.span` are `f64`; `PartialEq` carries the
/// test assertions (no `Finding` is a map key).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Finding {
    /// A hub whose settling unblocks ≥ [`FORK_MIN_ARMS`] non-terminal dependents
    /// (a branch point). Excludes gating-class hubs (those are `GatingFanOut`).
    Fork { hub: String, arms: Vec<String> },
    /// A node gated by ≥ [`JOIN_MIN_PREREQS`] non-terminal prerequisites (a merge point).
    Join { node: String, prereqs: Vec<String> },
    /// A GATING-class record (ADR-017 — unsettled knowledge, never workable) blocking
    /// ≥ [`GATING_MIN_BLOCKS`] non-terminal dependents. Takes precedence over `Fork`.
    GatingFanOut { record: String, blocks: Vec<String> },
    /// A low-worth blocker gating a high-worth dependent — `base(blocked) − base(blocker)`
    /// exceeds [`INVERSION_MIN_GAP`]. Compares intrinsic `base` (NOT `score`, which folds
    /// the gated dependent's leverage back into the blocker — F-ext-2).
    ValueInversion {
        blocker: String,
        blocked: String,
        gap: f64,
    },
    /// A node whose position in the constraint-bearing survey order (`act_rank → score →
    /// id`) differs from its position in the constraint-free score order by ≥
    /// [`DISPLACEMENT_MIN_DELTA`] — the constraints doing real work.
    Displacement {
        node: String,
        score_rank: usize,
        constrained_rank: usize,
        delta: usize,
    },
    /// A maximal adjacent near-tie run in the `next` frontier order (within
    /// [`PLATEAU_EPS`] step-to-step) — a segment the flat list cannot distinguish.
    Plateau { members: Vec<String>, span: f64 },
    /// A pair ADJACENT in the base frontier order whose relative order INVERTS between
    /// the β=0 and β=1 frontier orders (endpoint-contested — R5). `high`/`low` are the
    /// pair in base-order (higher-priority first); `moved` is the positions the pair
    /// shifted across the sweep (the magnitude, surfaced via [`Finding::magnitude`], not
    /// a payload key).
    OrderInstability {
        high: String,
        low: String,
        moved: usize,
    },
    /// A base [`Finding::Fork`] whose ARM order (the arms frontier-ordered among
    /// themselves) differs between the β=0 and β=1 endpoints. `moved` is the count of
    /// arms whose position changed (the magnitude).
    ArmResequencing {
        hub: String,
        order_lo: Vec<String>,
        order_hi: Vec<String>,
        moved: usize,
    },
    /// A provenance finding — an evicted soft (`after`) edge, or a degraded dep cycle
    /// SCC. Reuses [`ReasonKind::EvictedEdge`] / [`ReasonKind::CycleDegraded`] (DRY;
    /// shares the `explain` render fragment).
    Provenance(ReasonKind),
    /// SL-213 PHASE-06 (design §4 S4, comparison C3) — a preference cycle
    /// quarantined every row on it. `classes` are the cyclic equality classes
    /// (entity ids); `rows` the quarantined row uids. Exit (C7/D6): supersede
    /// one row (`--supersedes <uid>`) or tombstone one to break the cycle —
    /// re-asking alone cannot clear it (R3 concurrency).
    PreferenceCycle {
        domain: ComparisonDomain,
        classes: Vec<String>,
        rows: Vec<String>,
    },
    /// SL-213 PHASE-06 (comparison C2/C4) — retained rows contradict a pair
    /// of authored anchors. `anchors` names both conflicting entities WITH
    /// their anchor values (authored `[value]` facets in the value domain;
    /// β-resolved `authored_est_cost` points in the estimate domain — SL-219
    /// D1); `rows` the quarantined row uids. Exit (C7/D6): supersede a
    /// conflicting row, tombstone one, or edit an anchor — the est-domain
    /// render says "revise the estimate" (the likeliest defect is a stale
    /// estimate, D1).
    AnchorConflict {
        domain: ComparisonDomain,
        anchors: Vec<(String, f64)>,
        rows: Vec<String>,
    },
    /// SL-213 PHASE-06 (comparison P7/P8) — entities placed by the gauge
    /// convention despite anchors existing in OTHER components (no directed
    /// order path connects them to any anchor). Under per-component placement
    /// (SL-216) this is every member of an anchor-free component, not just
    /// its P7 sinks. The hint: compare against an anchored item to place it.
    AnchorGaugeDisconnect {
        domain: ComparisonDomain,
        entities: Vec<String>,
    },
    /// SL-213 PHASE-06 (comparison R2) — a supersession cycle deactivated
    /// every participating row (`ResolutionStatus::Malformed`). `rows` are
    /// the participant uids. Exit (C7/D6): tombstone one row to break the
    /// cycle.
    MalformedSupersession {
        domain: ComparisonDomain,
        rows: Vec<String>,
    },
    /// SL-220 D6 (RV-278 F-4) — an authored `[value]` facet survives on the
    /// entity: unmigrated evidence debt, fired on facet PRESENCE, never on
    /// rung-5 consumption — a facet shadowed by projection (a compared
    /// facet-bearing item resolves at rung 2) is still debt. Exit: run
    /// `scripts/migrate_value_facets.py`, or re-assert via
    /// `value set`/`value pin` and `value clear` the residue.
    UnmigratedFacet {
        domain: ComparisonDomain,
        entity: String,
        value: f64,
    },
    /// SL-220 PHASE-06 (design §2/§6, D3/D14) — a SAME-tier magnitude
    /// disagreement on one item's value claim: the winning tier's multiset
    /// mean stands as the point value; `rows` claims span `[low, high]`.
    /// Anchored tiers (Pin/Human) carry the "contested" framing + the reprobe
    /// disclosure; agent/migrated conflicts route to comparison, never the
    /// human queue (D14). Sourced from the claim pass
    /// ([`comparison::ClaimFinding::Conflict`]); domain-tagged Value.
    ClaimConflict {
        domain: ComparisonDomain,
        item: String,
        tier: comparison::ClaimTier,
        low: f64,
        high: f64,
        rows: u32,
    },
}

/// Which per-domain comparison system produced a comparison finding (SL-219
/// D9) — SET AT CONSTRUCTION by the shell that knows the producing system
/// ([`comparison_findings`] runs the detectors once per system); compile
/// payloads stay domain-agnostic. Lives here beside [`Finding`] (not in
/// `comparison`): finding provenance is a findings-layer concern, and the
/// closed two-variant set mirrors the two compiled systems, not the open
/// wire-domain vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComparisonDomain {
    Value,
    Estimate,
}

impl ComparisonDomain {
    /// The wire-domain token (STD-001: the `wire.rs` constants, no parallel
    /// strings) — the render tag and the JSON `domain` value.
    pub(crate) fn token(self) -> &'static str {
        match self {
            ComparisonDomain::Value => comparison::DOMAIN_VALUE,
            ComparisonDomain::Estimate => comparison::DOMAIN_ESTIMATE,
        }
    }
}

/// `f64` magnitude of a count, cast losslessly via `u32` (avoids the denied `as`
/// conversion; saturates for absurd counts that cannot arise in practice).
fn count_magnitude(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

impl Finding {
    /// The finding's magnitude — ranks findings WITHIN their kind group (F-ext-5: kind
    /// grouping outranks magnitude by design; this is NOT a cross-kind rank).
    pub(crate) fn magnitude(&self) -> f64 {
        match self {
            Finding::Fork { arms, .. } => count_magnitude(arms.len()),
            Finding::Join { prereqs, .. } => count_magnitude(prereqs.len()),
            Finding::GatingFanOut { blocks, .. } => count_magnitude(blocks.len()),
            Finding::ValueInversion { gap, .. } => *gap,
            Finding::Displacement { delta, .. } => count_magnitude(*delta),
            Finding::Plateau { members, .. } => count_magnitude(members.len()),
            // Both β-family variants rank within their kind by positions moved.
            Finding::OrderInstability { moved, .. } | Finding::ArmResequencing { moved, .. } => {
                count_magnitude(*moved)
            }
            Finding::Provenance(ReasonKind::CycleDegraded { nodes }) => {
                count_magnitude(nodes.len())
            }
            // An evicted edge — the two endpoints; ranks below any multi-node cycle.
            Finding::Provenance(_) => 2.0,
            Finding::PreferenceCycle { rows, .. } | Finding::MalformedSupersession { rows, .. } => {
                count_magnitude(rows.len())
            }
            Finding::AnchorConflict { rows, .. } => count_magnitude(rows.len()),
            Finding::AnchorGaugeDisconnect { entities, .. } => count_magnitude(entities.len()),
            // Presence-based debt: one facet each — kind grouping carries the
            // signal; within the group the stable sort keeps id order.
            Finding::UnmigratedFacet { .. } => 1.0,
            Finding::ClaimConflict { rows, .. } => f64::from(*rows),
        }
    }

    /// The finding's kind label — the group header AND the json `kind` tag. Kind grouping
    /// is the primary output sort key (F-ext-5).
    pub(crate) fn kind_label(&self) -> &'static str {
        match self {
            Finding::Fork { .. } => "forks",
            Finding::Join { .. } => "joins",
            Finding::GatingFanOut { .. } => "gating fan-out",
            Finding::ValueInversion { .. } => "value inversions",
            Finding::Displacement { .. } => "displacements",
            Finding::Plateau { .. } => "plateaus",
            Finding::OrderInstability { .. } => "order instability",
            Finding::ArmResequencing { .. } => "arm resequencing",
            Finding::Provenance(_) => "provenance",
            Finding::PreferenceCycle { .. } => "preference cycles",
            Finding::AnchorConflict { .. } => "anchor conflicts",
            Finding::AnchorGaugeDisconnect { .. } => "anchor/gauge disconnects",
            Finding::MalformedSupersession { .. } => "malformed supersessions",
            Finding::UnmigratedFacet { .. } => "unmigrated facets",
            Finding::ClaimConflict { .. } => "claim conflicts",
        }
    }
}

/// The pre-built β endpoint graphs (skew 0 / skew 1 over the SAME scan) the β-family
/// detectors read (design §Purity boundary). The impure shell
/// ([`super::surface::beta_endpoints`]) owns the two builds; the detectors here derive
/// frontier orders from these raw graphs via the pure [`super::order`] primitives.
pub(crate) struct BetaEndpoints {
    /// The graph rebuilt at `estimate.skew = BETA_LO` (optimistic cost).
    pub(crate) lo: PriorityGraph,
    /// The graph rebuilt at `estimate.skew = BETA_HI` (pessimistic cost).
    pub(crate) hi: PriorityGraph,
}

// ── Status-class helper (engine-pure read of the partition table) ────────────

/// The [`StatusClass`] of a node — a pure read of the single-source partition table
/// (`super::partition::status_class`) over the node's captured `NodeAttr`. Mirrors the
/// private `channels::class_of`; re-derived here (not imported) so `findings.rs` stays
/// within its own module seam while still routing through the ONE classification source.
fn class_of(g: &PriorityGraph, key: EntityKey) -> StatusClass {
    match g.attrs.get(&key) {
        Some(a) => status_class(a.kind, a.status.as_deref()),
        None => StatusClass::Unrecognised,
    }
}

/// Canonical refs for a slice of keys (sorted-by-key order preserved).
fn refs(keys: &[EntityKey]) -> Vec<String> {
    keys.iter().map(|k| k.canonical()).collect()
}

// ── Detectors (pure fns over the graph) ──────────────────────────────────────

/// The structural basis of a fork — every non-gating hub with ≥ [`FORK_MIN_ARMS`]
/// non-terminal arms, as `(hub, arm keys)`. Shared by the [`Finding::Fork`] detector and
/// the β-family [`arm_resequencing`] (DRY): both need the same hub + arm set, differing
/// only in how they present it. Reads `channels::blocking` (OUT); filters arms to
/// non-terminal (blocking does NOT — F5); excludes gating-class hubs so a gating fan-out
/// reports ONCE, as `GatingFanOut` (F6 precedence).
fn fork_arms(g: &PriorityGraph) -> Vec<(EntityKey, Vec<EntityKey>)> {
    let mut out = Vec::new();
    for &hub in g.attrs.keys() {
        if class_of(g, hub) == StatusClass::Gating {
            continue; // gating hubs → GatingFanOut only
        }
        let arms: Vec<EntityKey> = channels::blocking(g, hub)
            .into_iter()
            .filter(|a| class_of(g, *a) != StatusClass::Terminal)
            .collect();
        if arms.len() >= FORK_MIN_ARMS {
            out.push((hub, arms));
        }
    }
    out
}

/// **Fork** — a non-gating hub whose settling unblocks ≥ [`FORK_MIN_ARMS`] non-terminal
/// dependents. Presents [`fork_arms`] as findings.
fn forks(g: &PriorityGraph) -> Vec<Finding> {
    fork_arms(g)
        .into_iter()
        .map(|(hub, arms)| Finding::Fork {
            hub: hub.canonical(),
            arms: refs(&arms),
        })
        .collect()
}

/// **Join** — a node gated by ≥ [`JOIN_MIN_PREREQS`] non-terminal prerequisites. Reads
/// `channels::blocked_by` (IN — already non-terminal-filtered, sorted, deduped).
fn joins(g: &PriorityGraph) -> Vec<Finding> {
    let mut out = Vec::new();
    for &node in g.attrs.keys() {
        let prereqs = channels::blocked_by(g, node);
        if prereqs.len() >= JOIN_MIN_PREREQS {
            out.push(Finding::Join {
                node: node.canonical(),
                prereqs: refs(&prereqs),
            });
        }
    }
    out
}

/// **`GatingFanOut`** — a GATING-class record (ADR-017) blocking ≥ [`GATING_MIN_BLOCKS`]
/// non-terminal dependents. Reads `channels::blocking` (OUT) from a gating hub; arms
/// filtered non-terminal (F5). Precedence: `forks` excludes gating hubs, so no node is
/// both (F6).
fn gating_fan_out(g: &PriorityGraph) -> Vec<Finding> {
    let mut out = Vec::new();
    for &hub in g.attrs.keys() {
        if class_of(g, hub) != StatusClass::Gating {
            continue;
        }
        let blocks: Vec<EntityKey> = channels::blocking(g, hub)
            .into_iter()
            .filter(|b| class_of(g, *b) != StatusClass::Terminal)
            .collect();
        if blocks.len() >= GATING_MIN_BLOCKS {
            out.push(Finding::GatingFanOut {
                record: hub.canonical(),
                blocks: refs(&blocks),
            });
        }
    }
    out
}

/// **`ValueInversion`** — a low-worth blocker gating a high-worth dependent:
/// `base(blocked) − base(blocker)` exceeds [`INVERSION_MIN_GAP`]. Compares intrinsic
/// `base` (value+risk, pre-consequence), NOT `score` — `score(blocker)` folds `leverage` from
/// the very dependent it gates, so a score test is dead (F-ext-2). Iterates each node's
/// non-terminal `blocked_by` prerequisites (the active dep edges).
fn value_inversions(g: &PriorityGraph) -> Vec<Finding> {
    let mut out = Vec::new();
    for &blocked in g.attrs.keys() {
        let blocked_base = channels::base(g, blocked);
        for blocker in channels::blocked_by(g, blocked) {
            let gap = blocked_base - channels::base(g, blocker);
            if gap > INVERSION_MIN_GAP {
                out.push(Finding::ValueInversion {
                    blocker: blocker.canonical(),
                    blocked: blocked.canonical(),
                    gap,
                });
            }
        }
    }
    out
}

/// **Displacement** — a node whose position in the constraint-bearing survey order
/// differs from its position in the constraint-free score order by ≥
/// [`DISPLACEMENT_MIN_DELTA`]. Both orders are DIRECT sorts the detector computes over
/// `channels::{blocked, score}` (survey-order = `act_rank → score desc → id`;
/// score-order = `score desc → id`) — NOT `order.rs` (survey-order is not a frontier
/// order) and NOT the shell's `survey_for_map` (keeps the engine from importing the
/// shell — plan.md "Order sources"). Node set = the survey default (eligible, not
/// promoted) so blocked-but-eligible items participate (the displacement signal).
fn displacements(g: &PriorityGraph) -> Vec<Finding> {
    let nodes: Vec<EntityKey> = g
        .attrs
        .keys()
        .copied()
        .filter(|&k| channels::eligible(g, k) && !channels::promoted(g, k))
        .collect();

    // Constrained (survey) order: Actionable(0) before Blocked(1), then score desc, id asc.
    let mut constrained = nodes.clone();
    constrained.sort_by(|a, b| {
        let ra = u8::from(channels::blocked(g, *a));
        let rb = u8::from(channels::blocked(g, *b));
        ra.cmp(&rb)
            .then(channels::score(g, *b).total_cmp(&channels::score(g, *a)))
            .then(a.cmp(b))
    });

    // Constraint-free score order: score desc, id asc.
    let mut by_score = nodes.clone();
    by_score.sort_by(|a, b| {
        channels::score(g, *b)
            .total_cmp(&channels::score(g, *a))
            .then(a.cmp(b))
    });

    let cpos: BTreeMap<EntityKey, usize> = constrained
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, i))
        .collect();
    let spos: BTreeMap<EntityKey, usize> =
        by_score.iter().enumerate().map(|(i, &k)| (k, i)).collect();

    let mut out = Vec::new();
    for &k in &nodes {
        let (Some(&c), Some(&s)) = (cpos.get(&k), spos.get(&k)) else {
            continue;
        };
        let delta = c.abs_diff(s);
        if delta >= DISPLACEMENT_MIN_DELTA {
            out.push(Finding::Displacement {
                node: k.canonical(),
                score_rank: s,
                constrained_rank: c,
                delta,
            });
        }
    }
    out
}

/// The `next` frontier order of a graph — the actionable, non-promoted set in the
/// score-aware induced-frontier order (`surface::next`'s exact basis), via the pure
/// [`order`] primitives. Shared by [`plateaus`] and the β-family detectors so the base /
/// lo / hi orderings are all derived through ONE recipe (DRY, purity preserved — no disk).
fn frontier_order_of(g: &PriorityGraph) -> Vec<EntityKey> {
    let actionable_set: BTreeSet<EntityKey> = g
        .attrs
        .keys()
        .copied()
        .filter(|&k| channels::actionable(g, k) && !channels::promoted(g, k))
        .collect();
    let actionable: Vec<EntityKey> = actionable_set.iter().copied().collect();
    let preds = order::surviving_seq_predecessors(g, &actionable_set);
    order::frontier_order(&actionable, &|k| channels::score(g, k), &preds)
}

/// **Plateau** — a maximal adjacent near-tie run in the `next` frontier order (each
/// step-to-step score difference within [`PLATEAU_EPS`]). Uses [`frontier_order_of`]
/// (byte-identical to `surface::next`'s order basis).
fn plateaus(g: &PriorityGraph) -> Vec<Finding> {
    let ordering = frontier_order_of(g);

    // Pair each node with its score ONCE, then group into maximal adjacent near-tie runs
    // (each step-to-step gap within PLATEAU_EPS). No indexing — a single forward scan.
    let scored: Vec<(EntityKey, f64)> = ordering
        .iter()
        .map(|&k| (k, channels::score(g, k)))
        .collect();

    let mut out = Vec::new();
    let mut run: Vec<(EntityKey, f64)> = Vec::new();
    for &(k, s) in &scored {
        match run.last() {
            Some(&(_, prev)) if (prev - s).abs() < PLATEAU_EPS => run.push((k, s)),
            _ => {
                flush_plateau(&mut out, &run);
                run.clear();
                run.push((k, s));
            }
        }
    }
    flush_plateau(&mut out, &run);
    out
}

/// Emit a `Plateau` finding for a completed near-tie run of ≥2 members (a lone node is
/// no plateau). Span = |first score − last score| over the run. No panics (uses
/// `first`/`last`).
fn flush_plateau(out: &mut Vec<Finding>, run: &[(EntityKey, f64)]) {
    // A plateau needs a second member — a lone node is no run (structural, not a knob).
    if run.get(1).is_none() {
        return;
    }
    let members: Vec<EntityKey> = run.iter().map(|&(k, _)| k).collect();
    let first = run.first().map_or(0.0, |&(_, s)| s);
    let last = run.last().map_or(0.0, |&(_, s)| s);
    out.push(Finding::Plateau {
        members: refs(&members),
        span: (first - last).abs(),
    });
}

/// **Provenance** — evicted soft (`after`) edges + degraded dep-cycle SCCs. The
/// `channels::evicted_seq_edges` accessor is NODE-LOCAL (returns each eviction for BOTH
/// endpoints), so evictions are deduped GLOBALLY over `(from, to)` (the reason is
/// functionally determined by the pair — F-ext-3) → exactly one finding per evicted
/// edge. `channels::dep_cycles` SCCs are already component-deduped.
fn provenance(g: &PriorityGraph) -> Vec<Finding> {
    let mut out = Vec::new();

    // Global dedupe over the endpoint pair (a given pair carries one eviction reason).
    let mut edges: BTreeMap<(EntityKey, EntityKey), OverrideReason> = BTreeMap::new();
    for &k in g.attrs.keys() {
        for (from, to, reason) in channels::evicted_seq_edges(g, k) {
            edges.entry((from, to)).or_insert(reason);
        }
    }
    for ((from, to), reason) in edges {
        out.push(Finding::Provenance(ReasonKind::EvictedEdge {
            from: from.canonical(),
            to: to.canonical(),
            reason,
        }));
    }

    for component in channels::dep_cycles(g) {
        let nodes: Vec<String> = component.into_iter().map(EntityKey::canonical).collect();
        out.push(Finding::Provenance(ReasonKind::CycleDegraded { nodes }));
    }

    out
}

// ── SL-213 PHASE-06 comparison-domain detectors ─────────────────────────────
//
// These four read the comparison-tier PIPELINE directly (§ design D12), not the
// `PriorityGraph`/`channels` substrate the detectors above share — the comparison
// tier sits BELOW `priority` (ADR-001), so its own artifacts (the compiled
// `ConstraintSet`, the resolve-tier finding streams) are the honest source, not a
// re-derivation over the graph.

/// **`PreferenceCycle`** (comparison C3) — every `QuarantineReason::PreferenceCycle`
/// entry, grouped by its cyclic class set (one finding per distinct SCC — several
/// rows on the same cycle share one finding, C3's `classes` payload is identical
/// across them).
fn preference_cycle_findings(
    cs: &comparison::ConstraintSet,
    domain: ComparisonDomain,
) -> Vec<Finding> {
    let mut groups: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    for (uid, reason) in &cs.quarantined {
        if let comparison::QuarantineReason::PreferenceCycle { classes } = reason {
            groups
                .entry(classes.clone())
                .or_default()
                .insert(uid.clone());
        }
    }
    groups
        .into_iter()
        .map(|(classes, rows)| Finding::PreferenceCycle {
            domain,
            classes,
            rows: rows.into_iter().collect(),
        })
        .collect()
}

/// **`AnchorConflict`** (comparison C2/C4) — one finding per DISTINCT conflicting
/// anchor pair, gathering every row quarantined for that pair (a row serving
/// several conflicts appears in several findings, C4's "quarantined once, finding
/// lists every pair" — here inverted to "one finding per pair, listing every row").
fn anchor_conflict_findings(
    cs: &comparison::ConstraintSet,
    domain: ComparisonDomain,
) -> Vec<Finding> {
    let mut by_pair: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for (uid, reason) in &cs.quarantined {
        if let comparison::QuarantineReason::AnchorConflict { pairs } = reason {
            for pair in pairs {
                by_pair.entry(pair.clone()).or_default().insert(uid.clone());
            }
        }
    }
    by_pair
        .into_iter()
        .map(|((x, y), rows)| {
            let ax = cs.anchors.get(&x).copied().unwrap_or(f64::NAN);
            let ay = cs.anchors.get(&y).copied().unwrap_or(f64::NAN);
            Finding::AnchorConflict {
                domain,
                anchors: vec![(x, ax), (y, ay)],
                rows: rows.into_iter().collect(),
            }
        })
        .collect()
}

/// **`AnchorGaugeDisconnect`** (comparison P7/P8) — entities placed by the
/// gauge convention despite anchors existing in other components: under
/// per-component placement (SL-216) that is every member of each anchor-free
/// component. When `cs.anchors` is empty the corpus is pure-gauge by
/// construction (no component has an anchor) — that is the base state, not a
/// disconnect, so the detector stays silent (it would otherwise flag every
/// entity in a comparison-free-of-anchors ledger).
fn anchor_gauge_disconnect_findings(
    cs: &comparison::ConstraintSet,
    projection: &comparison::Projection,
    domain: ComparisonDomain,
) -> Vec<Finding> {
    if cs.anchors.is_empty() {
        return Vec::new();
    }
    let entities: Vec<String> = projection
        .iter()
        .filter(|(_, (_, prov))| matches!(prov, comparison::ValueProvenance::Gauge))
        .map(|(e, _)| e.clone())
        .collect();
    if entities.is_empty() {
        return Vec::new();
    }
    vec![Finding::AnchorGaugeDisconnect { domain, entities }]
}

/// **`MalformedSupersession`** (comparison R2) — one finding per supersession
/// cycle the resolve tier already grouped (`Resolution::malformed`). The
/// resolve pass is SHARED across domains (SL-219 §1), so the domain is joined
/// back from the cycle's own rows — the identity key carries `domain`, so a
/// supersession cycle never spans domains (every participant shares one).
fn malformed_supersession_findings(pipeline: &comparison::Pipeline) -> Vec<Finding> {
    let domain_of_uid = |uid: &str| -> ComparisonDomain {
        match pipeline.rows.iter().find(|r| r.uid == uid) {
            Some(r) if r.domain == comparison::DOMAIN_ESTIMATE => ComparisonDomain::Estimate,
            _ => ComparisonDomain::Value,
        }
    };
    pipeline
        .malformed
        .iter()
        .map(|m| Finding::MalformedSupersession {
            domain: m
                .cycle
                .first()
                .map_or(ComparisonDomain::Value, |uid| domain_of_uid(uid)),
            rows: m.cycle.clone(),
        })
        .collect()
}

/// Run the four SL-213 PHASE-06 comparison-domain detectors over the loaded
/// pipeline (design §4 S4) — once per domain system (SL-219 D9: the shell that
/// knows which system produced a finding stamps its [`ComparisonDomain`] at
/// construction). The caller ([`super::surface::findings`]) merges these into
/// the graph-domain catalogue and re-sorts.
pub(crate) fn comparison_findings(pipeline: &comparison::Pipeline) -> Vec<Finding> {
    let mut out = Vec::new();
    for (system, domain) in [
        (&pipeline.value, ComparisonDomain::Value),
        (&pipeline.estimate, ComparisonDomain::Estimate),
    ] {
        out.extend(preference_cycle_findings(&system.constraint_set, domain));
        out.extend(anchor_conflict_findings(&system.constraint_set, domain));
        out.extend(anchor_gauge_disconnect_findings(
            &system.constraint_set,
            &system.projection,
            domain,
        ));
    }
    out.extend(malformed_supersession_findings(pipeline));
    out.extend(claim_conflict_findings(pipeline));
    out
}

/// **`ClaimConflict`** (SL-220 PHASE-06, design §2/§6) — lift the value claim
/// pass's same-tier conflicts ([`comparison::ClaimFinding::Conflict`], fired at
/// EVERY tier, surfaced-never-silent) into the findings surface, domain-tagged
/// Value. The render distinguishes the anchored "contested" framing from the
/// agent/migrated "calibrate via comparison" guidance (D14); the pass already
/// carries the tier, interval, and row count.
fn claim_conflict_findings(pipeline: &comparison::Pipeline) -> Vec<Finding> {
    pipeline
        .value_claims
        .findings
        .iter()
        .map(|f| {
            let comparison::ClaimFinding::Conflict {
                item,
                tier,
                low,
                high,
                rows,
                ..
            } = f;
            Finding::ClaimConflict {
                domain: ComparisonDomain::Value,
                item: item.clone(),
                tier: *tier,
                low: *low,
                high: *high,
                rows: *rows,
            }
        })
        .collect()
}

/// **`UnmigratedFacet`** (SL-220 D6, RV-278 F-4) — every entity still
/// authoring a `[value]` facet, fired on PRESENCE: the ladder's rung-5
/// consumption is irrelevant (a compared facet-bearing item resolves at
/// rung 2 and its facet is STILL debt — the flip's permanent semantics,
/// design §3). Graph-derived (the facet rides `NodeAttr.facets`), so it
/// lives with the graph detectors, not `comparison_findings`; domain-tagged
/// Value at construction (SL-219 D9 — Phase 2's estimate migration reuses
/// the variant with its own tag).
fn unmigrated_facets(g: &PriorityGraph) -> Vec<Finding> {
    g.attrs
        .iter()
        .filter_map(|(key, attr)| {
            attr.facets
                .value
                .as_ref()
                .map(|v| Finding::UnmigratedFacet {
                    domain: ComparisonDomain::Value,
                    entity: key.canonical(),
                    value: v.value,
                })
        })
        .collect()
}

// ── β-family detectors (PHASE-02 — over the pre-built BetaEndpoints) ──────────

/// A `key → index` position map for a frontier order (the sort position each node holds).
fn positions(order: &[EntityKey]) -> BTreeMap<EntityKey, usize> {
    order.iter().enumerate().map(|(i, &k)| (k, i)).collect()
}

/// **`OrderInstability`** — a pair ADJACENT in the base frontier order whose relative
/// order INVERTS between the β=0 (lo) and β=1 (hi) frontier orders. O(N) adjacent-only
/// (not the O(N²) all-pairs flood — sound for detection: any total-order change forces
/// ≥1 inverted adjacent base pair, F4). ENDPOINT-contested only — interior flips sharing
/// a sign at both endpoints are a known false-negative (R5 / F-ext-1). Silent when the
/// order is endpoint-stable.
fn order_instability(base: &PriorityGraph, betas: &BetaEndpoints) -> Vec<Finding> {
    let base_order = frontier_order_of(base);
    let lo = positions(&frontier_order_of(&betas.lo));
    let hi = positions(&frontier_order_of(&betas.hi));

    let mut out = Vec::new();
    for pair in base_order.windows(2) {
        let [a, b] = *pair else { continue };
        // Both endpoints must carry both nodes (the actionable set is β-invariant, so
        // this holds in practice; skip defensively if a rebuild ever diverges).
        let (Some(&la), Some(&lb), Some(&ha), Some(&hb)) =
            (lo.get(&a), lo.get(&b), hi.get(&a), hi.get(&b))
        else {
            continue;
        };
        // `a` precedes `b` in base; an inversion is the two endpoints disagreeing on that.
        if (la < lb) != (ha < hb) {
            let moved = la.abs_diff(ha).max(lb.abs_diff(hb));
            out.push(Finding::OrderInstability {
                high: a.canonical(),
                low: b.canonical(),
                moved,
            });
        }
    }
    out
}

/// The frontier order of a fork's arms among THEMSELVES — the arm set treated as its own
/// frontier (score-aware, honouring any inter-arm surviving seq edges). β-sensitive via
/// `channels::score`, so it flips lo↔hi exactly when the endpoints disagree on arm order.
fn arm_order(g: &PriorityGraph, arms: &[EntityKey]) -> Vec<EntityKey> {
    let set: BTreeSet<EntityKey> = arms.iter().copied().collect();
    let preds = order::surviving_seq_predecessors(g, &set);
    order::frontier_order(arms, &|k| channels::score(g, k), &preds)
}

/// **`ArmResequencing`** — a base [`fork_arms`] fork whose arm order differs between the
/// β=0 and β=1 endpoints. Arms come from the BASE fork; their order is recomputed at each
/// endpoint via [`arm_order`]. Silent when the arm order is endpoint-stable. Direction /
/// terminal handling is `fork_arms`' (shared with the Fork detector).
fn arm_resequencing(base: &PriorityGraph, betas: &BetaEndpoints) -> Vec<Finding> {
    let mut out = Vec::new();
    for (hub, arms) in fork_arms(base) {
        let lo = arm_order(&betas.lo, &arms);
        let hi = arm_order(&betas.hi, &arms);
        if lo != hi {
            let moved = lo.iter().zip(hi.iter()).filter(|(a, b)| a != b).count();
            out.push(Finding::ArmResequencing {
                hub: hub.canonical(),
                order_lo: refs(&lo),
                order_hi: refs(&hi),
                moved,
            });
        }
    }
    out
}

/// Run every PHASE-01 (today's-data) detector over the built graph and return the
/// findings sorted `(kind_label asc, magnitude desc)` — kind grouping is the primary key
/// (F-ext-5), magnitude ranks within a kind section. PURE + graph-only (design
/// §The purity boundary): derives every order internally; the shell owns disk.
///
/// The β-family (`OrderInstability` / `ArmResequencing`) runs ONLY when `betas` is
/// `Some` — the shell supplies the pre-built endpoint graphs iff a non-terminal interval
/// estimate exists; with `None` the family stays silent (starved-until-estimates, R1).
/// `_cfg` is retained for signature parity (the shell reads it to build `betas`; the
/// pure detectors need only the graphs).
pub(crate) fn detect(
    base: &PriorityGraph,
    _cfg: &PriorityConfig,
    betas: Option<&BetaEndpoints>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(forks(base));
    findings.extend(joins(base));
    findings.extend(gating_fan_out(base));
    findings.extend(value_inversions(base));
    findings.extend(displacements(base));
    findings.extend(plateaus(base));
    findings.extend(provenance(base));
    findings.extend(unmigrated_facets(base));
    if let Some(betas) = betas {
        findings.extend(order_instability(base, betas));
        findings.extend(arm_resequencing(base, betas));
    }

    findings.sort_by(|a, b| {
        a.kind_label()
            .cmp(b.kind_label())
            .then(b.magnitude().total_cmp(&a.magnitude()))
    });
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    use crate::priority::config;
    use crate::priority::graph;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    /// Seed a backlog issue with a status + raw `[relationships]` body (needs/after
    /// lines authored verbatim).
    fn seed_issue(root: &Path, id: u32, status: &str, rels: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I{id}\"\nkind = \"issue\"\nstatus = \"{status}\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\n{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    /// Seed a valued issue (explicit `[value]` over a fixed estimate) with relations.
    fn seed_valued(root: &Path, id: u32, value: f64, rels: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I{id}\"\nkind = \"issue\"\nstatus = \"open\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [estimate]\nlower = 0.0\nupper = 10.0\n[value]\nvalue = {value}\n\
                 [relationships]\n{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    /// Seed a question record with the given status (open → Gating class).
    fn seed_question(root: &Path, id: u32, status: &str) {
        use crate::test_support::SCHEMA_KNOWLEDGE;
        write(
            root,
            &format!(".doctrine/knowledge/question/{id:03}/record-{id:03}.toml"),
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\nid = {id}\nslug = \"q{id}\"\n\
                 title = \"Q{id}\"\nrecord_kind = \"question\"\nstatus = \"{status}\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n"
            ),
        );
        write(
            root,
            &format!(".doctrine/knowledge/question/{id:03}/record-{id:03}.md"),
            "q\n",
        );
    }

    fn detect_root(root: &Path) -> Vec<Finding> {
        let g = graph::build(root).unwrap();
        detect(&g, &config::load(root), None)
    }

    /// Seed an issue with an explicit `[value]` over an ARBITRARY estimate interval
    /// (`lo`/`hi`) — the β sweep knob: a wide interval makes `est_cost` swing hard with
    /// `skew`, so the item's score (∝ value/est_cost) is β-sensitive.
    fn seed_interval(root: &Path, id: u32, value: f64, lo: f64, hi: f64, rels: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I{id}\"\nkind = \"issue\"\nstatus = \"open\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [estimate]\nlower = {lo}\nupper = {hi}\n[value]\nvalue = {value}\n\
                 [relationships]\n{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    /// Build the base graph + the β endpoint sweep (skew 0 / 1 over the same scan) and run
    /// the full detector set — the β-family's positive-path harness (mirrors the impure
    /// shell `surface::beta_endpoints`, which is not reachable from the engine layer).
    fn detect_beta(root: &Path) -> Vec<Finding> {
        use crate::catalog::scan::ScanMode;
        let scanned =
            crate::relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = config::load(root);
        // Mechanical call-site update (SL-213 PHASE-05, SL-219 PHASE-04,
        // SL-220 PHASE-05): build_from_with_cfg gained projected-values,
        // cost-feed, and claims params; no comparisons dir in this fixture,
        // so the empty inputs are byte-identical to the pre-comparison-tier
        // behaviour.
        let no_projection = crate::comparison::Projection::new();
        let no_feed = crate::comparison::CostFeed::new();
        let no_claims = crate::comparison::ClaimResolution::default();
        let base =
            graph::build_from_with_cfg(&scanned, root, &cfg, &no_projection, &no_feed, &no_claims)
                .unwrap();
        let mut lo_cfg = cfg.clone();
        lo_cfg.estimate.skew = BETA_LO;
        let mut hi_cfg = cfg.clone();
        hi_cfg.estimate.skew = BETA_HI;
        let lo = graph::build_from_with_cfg(
            &scanned,
            root,
            &lo_cfg,
            &no_projection,
            &no_feed,
            &no_claims,
        )
        .unwrap();
        let hi = graph::build_from_with_cfg(
            &scanned,
            root,
            &hi_cfg,
            &no_projection,
            &no_feed,
            &no_claims,
        )
        .unwrap();
        let betas = BetaEndpoints { lo, hi };
        detect(&base, &cfg, Some(&betas))
    }

    fn has_fork(fs: &[Finding], hub: &str) -> bool {
        fs.iter()
            .any(|f| matches!(f, Finding::Fork { hub: h, .. } if h == hub))
    }
    fn has_join(fs: &[Finding], node: &str) -> bool {
        fs.iter()
            .any(|f| matches!(f, Finding::Join { node: n, .. } if n == node))
    }
    fn has_gating(fs: &[Finding], record: &str) -> bool {
        fs.iter()
            .any(|f| matches!(f, Finding::GatingFanOut { record: r, .. } if r == record))
    }

    // ── VT-3: structural detectors — positive + negative, direction/precedence ──

    #[test]
    fn fork_fires_on_two_nonterminal_arms_and_is_silent_on_one() {
        let dir = tmp();
        let root = dir.path();
        // ISS-002 and ISS-003 both need ISS-001 → ISS-001 forks (blocking OUT = {2,3}).
        seed_issue(root, 1, "open", "");
        seed_issue(root, 2, "open", "needs = [\"ISS-001\"]\n");
        seed_issue(root, 3, "open", "needs = [\"ISS-001\"]\n");
        // ISS-010 has a single dependent → NOT a fork.
        seed_issue(root, 10, "open", "");
        seed_issue(root, 11, "open", "needs = [\"ISS-010\"]\n");

        let fs = detect_root(root);
        assert!(has_fork(&fs, "ISS-001"), "ISS-001 forks 2 arms: {fs:?}");
        assert!(
            !has_fork(&fs, "ISS-010"),
            "single dependent is not a fork: {fs:?}"
        );
        // Direction: the ARMS are the dependents, and the hub is the prereq.
        let fork = fs
            .iter()
            .find(|f| matches!(f, Finding::Fork { hub, .. } if hub == "ISS-001"))
            .unwrap();
        if let Finding::Fork { arms, .. } = fork {
            assert_eq!(arms, &vec!["ISS-002".to_string(), "ISS-003".to_string()]);
        }
    }

    #[test]
    fn fork_filters_terminal_arms() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 blocks ISS-002 (open) and ISS-003 (closed/terminal). Only one
        // non-terminal arm survives → NOT a fork.
        seed_issue(root, 1, "open", "");
        seed_issue(root, 2, "open", "needs = [\"ISS-001\"]\n");
        seed_issue(root, 3, "closed", "needs = [\"ISS-001\"]\n");
        let fs = detect_root(root);
        assert!(
            !has_fork(&fs, "ISS-001"),
            "a terminal arm does not count toward the fork: {fs:?}"
        );
    }

    #[test]
    fn join_fires_on_two_prereqs_and_is_silent_on_one() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs ISS-002 and ISS-003 (both non-terminal) → a join.
        seed_issue(root, 1, "open", "needs = [\"ISS-002\", \"ISS-003\"]\n");
        seed_issue(root, 2, "open", "");
        seed_issue(root, 3, "open", "");
        // ISS-010 needs a single prereq → not a join.
        seed_issue(root, 10, "open", "needs = [\"ISS-011\"]\n");
        seed_issue(root, 11, "open", "");
        let fs = detect_root(root);
        assert!(has_join(&fs, "ISS-001"), "ISS-001 joins 2 prereqs: {fs:?}");
        assert!(
            !has_join(&fs, "ISS-010"),
            "single prereq is not a join: {fs:?}"
        );
    }

    #[test]
    fn gating_fan_out_fires_on_gating_hub_and_excludes_fork_double_report() {
        let dir = tmp();
        let root = dir.path();
        // QUE-001 (open → Gating) blocks ISS-002 and ISS-003 (both need it).
        seed_question(root, 1, "open");
        seed_issue(root, 2, "open", "needs = [\"QUE-001\"]\n");
        seed_issue(root, 3, "open", "needs = [\"QUE-001\"]\n");
        let fs = detect_root(root);
        assert!(has_gating(&fs, "QUE-001"), "gating hub fans out: {fs:?}");
        // Precedence: the gating hub is NOT also reported as a Fork.
        assert!(
            !has_fork(&fs, "QUE-001"),
            "a gating fan-out is reported ONCE (not a Fork): {fs:?}"
        );
    }

    #[test]
    fn gating_fan_out_silent_when_hub_is_workable() {
        let dir = tmp();
        let root = dir.path();
        // A workable hub with a fan-out is a Fork, not a GatingFanOut.
        seed_issue(root, 1, "open", "");
        seed_issue(root, 2, "open", "needs = [\"ISS-001\"]\n");
        seed_issue(root, 3, "open", "needs = [\"ISS-001\"]\n");
        let fs = detect_root(root);
        assert!(has_fork(&fs, "ISS-001"));
        assert!(
            !has_gating(&fs, "ISS-001"),
            "a workable hub is never a GatingFanOut: {fs:?}"
        );
    }

    #[test]
    fn plateau_fires_on_near_ties_and_is_silent_on_spread() {
        let dir = tmp();
        let root = dir.path();
        // Three bare open issues → identical default score → one plateau of 3.
        seed_issue(root, 1, "open", "");
        seed_issue(root, 2, "open", "");
        seed_issue(root, 3, "open", "");
        let fs = detect_root(root);
        let plateau = fs.iter().find(|f| matches!(f, Finding::Plateau { .. }));
        assert!(plateau.is_some(), "equal scores form a plateau: {fs:?}");
        if let Some(Finding::Plateau { members, .. }) = plateau {
            assert_eq!(members.len(), 3, "all three tie: {members:?}");
        }

        // A spread corpus: distinct values well beyond PLATEAU_EPS → no plateau.
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_valued(root2, 1, 5.0, "");
        seed_valued(root2, 2, 50.0, "");
        seed_valued(root2, 3, 200.0, "");
        let fs2 = detect_root(root2);
        assert!(
            !fs2.iter().any(|f| matches!(f, Finding::Plateau { .. })),
            "well-separated scores form no plateau: {fs2:?}"
        );
    }

    #[test]
    fn displacement_fires_when_a_high_score_item_is_blocked() {
        let dir = tmp();
        let root = dir.path();
        // A high-value item blocked by an open risk sinks in the survey (constrained)
        // order (Blocked ranks after every Actionable) while topping the pure-score
        // order → a large positional delta. Several low-value actionable fillers widen
        // the gap past DISPLACEMENT_MIN_DELTA.
        seed_valued(root, 100, 500.0, "needs = [\"ISS-200\"]\n"); // high score, blocked
        seed_issue(root, 200, "open", ""); // the blocker (actionable, modest score)
        for id in 1..=6 {
            seed_valued(root, id, 0.1, ""); // low-score actionable fillers
        }
        let fs = detect_root(root);
        let displaced = fs
            .iter()
            .any(|f| matches!(f, Finding::Displacement { node, .. } if node == "ISS-100"));
        assert!(
            displaced,
            "the blocked high-score item is displaced: {fs:?}"
        );

        // Negative: an all-actionable corpus with matching orders → no displacement.
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_valued(root2, 1, 5.0, "");
        seed_valued(root2, 2, 50.0, "");
        seed_valued(root2, 3, 200.0, "");
        let fs2 = detect_root(root2);
        assert!(
            !fs2.iter()
                .any(|f| matches!(f, Finding::Displacement { .. })),
            "actionable-only, order-agreeing corpus has no displacement: {fs2:?}"
        );
    }

    // ── VT-4: ValueInversion compares base, NOT score (leverage inflation) ──────

    #[test]
    fn value_inversion_fires_on_base_despite_leverage_inflated_score() {
        let dir = tmp();
        let root = dir.path();
        // ISS-002 (value 500 → base 500/6.5 ≈ 76.9) NEEDS ISS-001 (value 1 → base
        // 1/6.5 ≈ 0.15). base(blocked) − base(blocker) ≈ 76.8 > gap → inversion.
        seed_valued(root, 1, 1.0, ""); // the low-base blocker
        seed_valued(root, 2, 500.0, "needs = [\"ISS-001\"]\n"); // high-base dependent

        let g = graph::build(root).unwrap();
        let fs = detect(&g, &config::load(root), None);
        let inversion = fs.iter().any(|f| {
            matches!(
                f,
                Finding::ValueInversion { blocker, blocked, .. }
                    if blocker == "ISS-001" && blocked == "ISS-002"
            )
        });
        assert!(
            inversion,
            "low-base blocker of high-base dependent flagged: {fs:?}"
        );

        // Prove the leverage-inflation hazard the base test sidesteps: ISS-001's SCORE
        // is lifted well above its intrinsic base by the very dependent it gates.
        let k = |id| EntityKey { prefix: "ISS", id };
        assert!(
            channels::score(&g, k(1)) > channels::base(&g, k(1)) + 1.0,
            "score(blocker) is inflated by the gated dependent's leverage"
        );
    }

    #[test]
    fn value_inversion_silent_when_blocker_outweighs_dependent() {
        let dir = tmp();
        let root = dir.path();
        // A high-base blocker gating a low-base dependent is the CORRECT order — no
        // inversion (base(blocked) − base(blocker) is negative).
        seed_valued(root, 1, 500.0, ""); // high-base blocker
        seed_valued(root, 2, 1.0, "needs = [\"ISS-001\"]\n"); // low-base dependent
        let fs = detect_root(root);
        assert!(
            !fs.iter()
                .any(|f| matches!(f, Finding::ValueInversion { .. })),
            "a high-worth blocker gating low-worth work is not an inversion: {fs:?}"
        );
    }

    // ── VT-5: a single evicted seq edge → exactly ONE Provenance finding ────────

    #[test]
    fn one_evicted_seq_edge_yields_exactly_one_provenance_finding() {
        let dir = tmp();
        let root = dir.path();
        // A seq 2-cycle: ISS-001 after ISS-002 AND ISS-002 after ISS-001 → cordage
        // evicts one edge to linearize. The node-local accessor reports the eviction
        // for BOTH endpoints; the detector must dedupe to ONE finding.
        seed_valued(root, 1, 10.0, "after = [{ to = \"ISS-002\", rank = 0 }]\n");
        seed_valued(root, 2, 100.0, "after = [{ to = \"ISS-001\", rank = 0 }]\n");

        let g = graph::build(root).unwrap();
        // Precondition: an eviction actually occurred (both endpoints report it).
        let total_local: usize = g
            .attrs
            .keys()
            .map(|&k| channels::evicted_seq_edges(&g, k).len())
            .sum();
        assert!(
            total_local >= 2,
            "both endpoints report the eviction: {total_local}"
        );

        let fs = detect(&g, &config::load(root), None);
        let evicted: Vec<&Finding> = fs
            .iter()
            .filter(|f| matches!(f, Finding::Provenance(ReasonKind::EvictedEdge { .. })))
            .collect();
        assert_eq!(
            evicted.len(),
            1,
            "the node-local double-report is globally deduped to ONE finding: {fs:?}"
        );
    }

    // ── detect ordering: kind grouping outranks magnitude (F-ext-5) ─────────────

    #[test]
    fn detect_sorts_by_kind_then_magnitude_desc() {
        let dir = tmp();
        let root = dir.path();
        // Two forks of different arm counts → same kind section, larger first.
        seed_issue(root, 1, "open", "");
        seed_issue(root, 2, "open", "needs = [\"ISS-001\"]\n");
        seed_issue(root, 3, "open", "needs = [\"ISS-001\"]\n");
        seed_issue(root, 4, "open", "needs = [\"ISS-001\"]\n"); // ISS-001 → 3 arms
        seed_issue(root, 10, "open", "");
        seed_issue(root, 11, "open", "needs = [\"ISS-010\"]\n");
        seed_issue(root, 12, "open", "needs = [\"ISS-010\"]\n"); // ISS-010 → 2 arms

        let fs = detect_root(root);
        let fork_hubs: Vec<&String> = fs
            .iter()
            .filter_map(|f| match f {
                Finding::Fork { hub, .. } => Some(hub),
                _ => None,
            })
            .collect();
        assert_eq!(
            fork_hubs,
            vec![&"ISS-001".to_string(), &"ISS-010".to_string()],
            "larger fork (3 arms) precedes smaller (2 arms) within the kind: {fs:?}"
        );
    }

    // ── VT-2: β-family — OrderInstability + ArmResequencing (positive + negative) ──

    fn order_instability_pairs(fs: &[Finding]) -> Vec<(String, String)> {
        fs.iter()
            .filter_map(|f| match f {
                Finding::OrderInstability { high, low, .. } => Some((high.clone(), low.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn order_instability_fires_on_endpoint_flip_and_is_silent_when_stable() {
        // Two actionable leaves whose relative score order FLIPS across the β sweep:
        //   ISS-001 value 10, estimate [1,100] — cheap at β0 (cost 1 → 10), dear at β1
        //     (cost 100 → 0.1).
        //   ISS-002 value 5, estimate [5,6] — nearly β-invariant (β0 → 1, β1 → 0.83).
        // β0 order: [ISS-001, ISS-002]; β1 order: [ISS-002, ISS-001] → an inversion.
        let dir = tmp();
        let root = dir.path();
        seed_interval(root, 1, 10.0, 1.0, 100.0, "");
        seed_interval(root, 2, 5.0, 5.0, 6.0, "");

        let fs = detect_beta(root);
        let pairs = order_instability_pairs(&fs);
        assert_eq!(
            pairs.len(),
            1,
            "exactly one contested adjacent pair: {fs:?}"
        );
        let members: BTreeSet<&str> = [pairs[0].0.as_str(), pairs[0].1.as_str()]
            .into_iter()
            .collect();
        assert_eq!(
            members,
            BTreeSet::from(["ISS-001", "ISS-002"]),
            "the flipping pair is the two leaves: {fs:?}"
        );
        // magnitude is the positions-moved (≥1 for a real inversion).
        let mag = fs
            .iter()
            .find(|f| matches!(f, Finding::OrderInstability { .. }))
            .unwrap()
            .magnitude();
        assert!(mag >= 1.0, "positions moved ≥ 1: {mag}");

        // Negative: a corpus whose order is endpoint-STABLE (ISS-001 dominates at both
        // endpoints) → no OrderInstability.
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_interval(root2, 1, 100.0, 1.0, 2.0, "");
        seed_interval(root2, 2, 1.0, 1.0, 2.0, "");
        let fs2 = detect_beta(root2);
        assert!(
            !fs2.iter()
                .any(|f| matches!(f, Finding::OrderInstability { .. })),
            "endpoint-stable order emits no OrderInstability: {fs2:?}"
        );
    }

    #[test]
    fn arm_resequencing_fires_when_fork_arm_order_flips_lo_hi() {
        // ISS-001 is a fork hub; its two arms (both need it) reorder across the sweep:
        //   ISS-002 value 10, estimate [1,100] (β-sensitive), ISS-003 value 5, [5,6].
        // β0 arm order: [ISS-002, ISS-003]; β1: [ISS-003, ISS-002] → resequenced.
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "");
        seed_interval(root, 2, 10.0, 1.0, 100.0, "needs = [\"ISS-001\"]\n");
        seed_interval(root, 3, 5.0, 5.0, 6.0, "needs = [\"ISS-001\"]\n");

        let fs = detect_beta(root);
        let reseq = fs
            .iter()
            .find(|f| matches!(f, Finding::ArmResequencing { hub, .. } if hub == "ISS-001"));
        assert!(reseq.is_some(), "the fork's arms resequence lo↔hi: {fs:?}");
        if let Some(Finding::ArmResequencing {
            order_lo, order_hi, ..
        }) = reseq
        {
            assert_ne!(order_lo, order_hi, "the two arm orders differ: {fs:?}");
            assert_eq!(
                order_lo.iter().collect::<BTreeSet<_>>(),
                order_hi.iter().collect::<BTreeSet<_>>(),
                "same arm set, different order"
            );
        }

        // Negative: arms with identical (β-invariant) estimates keep their order → no
        // ArmResequencing.
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_issue(root2, 1, "open", "");
        seed_interval(root2, 2, 10.0, 1.0, 2.0, "needs = [\"ISS-001\"]\n");
        seed_interval(root2, 3, 1.0, 1.0, 2.0, "needs = [\"ISS-001\"]\n");
        let fs2 = detect_beta(root2);
        assert!(
            !fs2.iter()
                .any(|f| matches!(f, Finding::ArmResequencing { .. })),
            "endpoint-stable arm order emits no ArmResequencing: {fs2:?}"
        );
    }

    #[test]
    fn beta_family_silent_without_betas() {
        // With `None` betas (no interval estimate available), the β-family emits nothing
        // even over a corpus that WOULD flip — the starved-until-estimates behaviour (R1).
        let dir = tmp();
        let root = dir.path();
        seed_interval(root, 1, 10.0, 1.0, 100.0, "");
        seed_interval(root, 2, 5.0, 5.0, 6.0, "");
        let fs = detect_root(root); // passes None
        assert!(
            !fs.iter().any(|f| matches!(
                f,
                Finding::OrderInstability { .. } | Finding::ArmResequencing { .. }
            )),
            "None betas ⇒ β-family silent: {fs:?}"
        );
    }

    // ── SL-219 VT-4: findings carry their producing system's domain (D9) ──────

    use crate::comparison::Judgement;

    /// One judgement row for the domain-discriminator fixtures below.
    fn domain_judgement(uid: &str, domain: &str, frame: &str, a: &str, b: &str) -> Judgement {
        use crate::comparison::{RaterKind, Response, RowForm};
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: a.to_string(),
            b: Some(b.to_string()),
            response: Some(Response::PreferA),
            domain: domain.to_string(),
            frame: frame.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-14".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    /// The in-memory pipeline over one session (the VT-4 harness). Value
    /// anchors are claim-derived inside the pipeline (SL-220 PHASE-05) —
    /// fixtures mint them as anchor ROWS via [`value_anchor_row`].
    fn pipeline_of(
        judgements: Vec<Judgement>,
        est_anchors: &crate::comparison::AnchorMap,
    ) -> comparison::Pipeline {
        use crate::comparison::{
            COMPARISON_SCHEMA, COMPARISON_VERSION, ComparisonSession, SessionHeader, StatusMap,
            VALUE_PROJECTION_PARAMS, pipeline_from_sessions,
        };
        let sessions = vec![ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: COMPARISON_VERSION,
            session: SessionHeader {
                uid: "sess-1".to_string(),
                date: "2026-07-14".to_string(),
                audience: None,
            },
            judgements,
            tombstones: Vec::new(),
        }];
        pipeline_from_sessions(
            &sessions,
            &StatusMap::new(),
            est_anchors,
            &VALUE_PROJECTION_PARAMS,
            &VALUE_PROJECTION_PARAMS,
        )
        .unwrap()
    }

    /// A live human value-anchor row (SL-220 §1) — the post-flip way a
    /// fixture pins a value anchor (a resolved claim feeds `anchor_map()`).
    fn value_anchor_row(uid: &str, item: &str, magnitude: f64) -> Judgement {
        use crate::comparison::{DOMAIN_VALUE, FRAME_VALUE_ANCHOR, RowForm};
        let mut j = domain_judgement(uid, DOMAIN_VALUE, FRAME_VALUE_ANCHOR, item, "unused");
        j.b = None;
        j.response = None;
        j.form = RowForm::Anchor;
        j.magnitude = Some(magnitude);
        j
    }

    /// D9: each comparison finding carries the [`ComparisonDomain`] of the
    /// system that produced it, SET AT CONSTRUCTION — and the domain reaches
    /// both the human render (est wording per D1; value bytes unchanged) and
    /// the JSON payload (`domain` key — render/JSON parity).
    #[test]
    fn comparison_findings_carry_domain_in_render_and_json() {
        use crate::comparison::{
            AnchorMap, DOMAIN_ESTIMATE, DOMAIN_VALUE, FRAME_EQUAL_EFFORT, FRAME_MORE_WORK,
        };
        // Each domain gets its own C4 anchor conflict: the row prefers the
        // LOWER-anchored side, contradicting the anchors of its own system.
        // Value anchors are minted as claim ROWS (SL-220 PHASE-05 — the
        // resolved Pin/Human claims feed compile via `anchor_map()`).
        let est_anchors: AnchorMap = [("ISS-003".to_string(), 1.0), ("ISS-004".to_string(), 5.0)]
            .into_iter()
            .collect();
        let pipeline = pipeline_of(
            vec![
                domain_judgement("v1", DOMAIN_VALUE, FRAME_EQUAL_EFFORT, "ISS-001", "ISS-002"),
                domain_judgement("e1", DOMAIN_ESTIMATE, FRAME_MORE_WORK, "ISS-003", "ISS-004"),
                value_anchor_row("va1", "ISS-001", 1.0),
                value_anchor_row("va2", "ISS-002", 2.0),
            ],
            &est_anchors,
        );

        let fs = comparison_findings(&pipeline);
        let domain_of = |rows: &[&str]| {
            fs.iter()
                .find_map(|f| match f {
                    Finding::AnchorConflict {
                        domain, rows: r, ..
                    } if r == rows => Some(*domain),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("conflict over {rows:?} found: {fs:?}"))
        };
        assert_eq!(domain_of(&["v1"]), ComparisonDomain::Value);
        assert_eq!(domain_of(&["e1"]), ComparisonDomain::Estimate);

        // Human render: the est line carries the domain tag + the D1 stale-
        // estimate wording; the value line keeps the pre-SL-219 bytes.
        let human = crate::priority::render::findings_human(&fs);
        assert!(
            human.contains("[estimate] anchors ISS-003=1.0 vs ISS-004=5.0 conflict"),
            "est conflict domain-tagged: {human}"
        );
        assert!(
            human.contains("sizing evidence contradicts the β-resolved costs")
                && human.contains("revise the estimate or supersede the row"),
            "D1 wording on the est conflict: {human}"
        );
        assert!(
            human.contains(
                "  anchors ISS-001=1.0 vs ISS-002=2.0 conflict — quarantines {v1}; exit: \
                 supersede a conflicting row, tombstone one, or edit an anchor\n"
            ),
            "value wording unchanged and untagged: {human}"
        );

        // JSON parity: every comparison finding carries its `domain` token.
        let json = crate::priority::render::findings_json(&fs).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let domains: Vec<&str> = value["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["kind"] == "anchor conflicts")
            .map(|f| f["domain"].as_str().expect("domain is a string"))
            .collect();
        assert_eq!(domains, ["value", "estimate"], "JSON domain per finding");
    }

    /// The shared-resolution finding (`MalformedSupersession`) joins its domain
    /// back from the cycle's own rows — an est-domain supersession cycle is
    /// tagged `estimate` even though resolve runs once over all rows.
    #[test]
    fn malformed_supersession_joins_domain_from_its_rows() {
        use crate::comparison::{AnchorMap, DOMAIN_ESTIMATE, FRAME_MORE_WORK};
        let mut a = domain_judgement(
            "mal-a",
            DOMAIN_ESTIMATE,
            FRAME_MORE_WORK,
            "ISS-001",
            "ISS-002",
        );
        let mut b = domain_judgement(
            "mal-b",
            DOMAIN_ESTIMATE,
            FRAME_MORE_WORK,
            "ISS-002",
            "ISS-001",
        );
        a.supersedes = Some("mal-b".to_string());
        b.supersedes = Some("mal-a".to_string());
        b.seq = 1;
        let pipeline = pipeline_of(vec![a, b], &AnchorMap::new());
        let fs = comparison_findings(&pipeline);
        assert!(
            fs.iter().any(|f| matches!(
                f,
                Finding::MalformedSupersession {
                    domain: ComparisonDomain::Estimate,
                    ..
                }
            )),
            "the est cycle carries the estimate domain: {fs:?}"
        );
    }

    #[test]
    fn anchor_gauge_disconnect_lists_whole_island() {
        // Mixed corpus (the s8 shape): anchored chain x>y (x=4.0) plus an
        // anchor-free island z>w, through the REAL compile+project seam. The
        // P7 hint ("compare against an anchored item") applies island-wide
        // (SL-216 D1): every member of the island is gauge-placed, so the
        // finding lists z AND w — not only the floorless sink.
        use crate::comparison::{
            AnchorMap, DOMAIN_VALUE, FRAME_EQUAL_EFFORT, Judgement, QuarantinePolicy, RaterKind,
            Response, RowForm, compile, project,
        };
        let judgement = |uid: &str, winner: &str, loser: &str| Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: winner.to_string(),
            b: Some(loser.to_string()),
            response: Some(Response::PreferA),
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-12".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        };
        let rows = [judgement("j0", "x", "y"), judgement("j1", "z", "w")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let anchors: AnchorMap = [("x".to_string(), 4.0)].into_iter().collect();
        let cs = compile(&refs, &anchors, QuarantinePolicy::Symmetric);
        let projection = project(&cs, &crate::comparison::VALUE_PROJECTION_PARAMS);

        let fs = anchor_gauge_disconnect_findings(&cs, &projection, ComparisonDomain::Value);
        assert_eq!(fs.len(), 1, "one disconnect finding: {fs:?}");
        let Finding::AnchorGaugeDisconnect { entities, .. } = &fs[0] else {
            panic!("wrong finding kind: {fs:?}");
        };
        assert_eq!(
            entities,
            &vec!["w".to_string(), "z".to_string()],
            "the finding must list the WHOLE island, not only the P7 sink"
        );
    }
}
