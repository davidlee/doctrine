// SPDX-License-Identifier: GPL-3.0-only
//! `priority::elicit` — SL-217 design §2: the elicitation queue assembler.
//!
//! Pure over its [`ElicitInputs`] (active comparison rows, anchors, the ranked
//! frontier, a per-item costing map, and the projection) plus a
//! [`DecisionContext`]. No clock, disk, rng, or git — the one impure load
//! (session scan + costing build from the priority graph) stays in the command
//! shell / PHASE-03. The assembler compiles the baseline [`ConstraintSet`]
//! internally (single source; compile is evidence-sized) and rides the
//! `comparison::query` predicates for every determinacy question.
//!
//! Three candidate sources feed one ranked queue:
//!
//! 1. **Comparison** — indeterminate pairs among the *constrained* top-K
//!    value-sensitive items, behind the capture-admissibility gate.
//! 2. **Median-probe** (D14) — one calibration probe per *un-constrained*
//!    top-K item against the projected median of its comparable set. Stateless.
//! 3. **Anchor-review** (D12) — one candidate per distinct suspect anchor in an
//!    `AnchorConflict` quarantine pair; admits on suspect EXISTENCE, not on
//!    yield, and is deliberately NOT K-gated.
//!
//! Ranking (D13): `score = guaranteed_yield × guaranteed_impact ×
//! confirm_boost`; `guaranteed_impact` = the min, over the answers attaining
//! the min yield, of the rank-decay-weighted count of that answer's
//! newly-determined pairs. Determinism: `BTree` everywhere, `total_cmp` on
//! scores, id-lexicographic tiebreak, no float in any key.
//!
//! Design-internal reconciliation flagged for audit (notes.md): the
//! `guaranteed_yield > 0` admission filter applies to the yield-motivated
//! sources (comparison, median-probe) only; anchor-review admits on existence
//! (D15 pins a live stale-anchor suspect as standing evidence-debt — a blanket
//! filter would gut it). A second in-phase reading: comparison enumerates
//! *constrained* pool pairs while median-probe owns the *un-constrained* items,
//! partitioning the two sources so a brand-new item yields one probe rather
//! than K flooding pairs (D14's calibration intent).

use std::collections::{BTreeMap, BTreeSet};

use crate::comparison::{
    AnchorMap, Bound, ClassId, ConstraintSet, Hypothetical, Judgement, PairSide, Projection,
    QuarantinePolicy, QuarantineReason, Reachability, Response, RowUid, ValueBounds,
    ValueProvenance, admissible_value_pair, compile, compile_human_only,
    constraining_counts_by_class, determined, human_rows, hypothetical_outcome,
    synthetic_answer_row,
};

// ── inputs ──────────────────────────────────────────────────────────────────

/// The pluggable decision-context seam (design D3). `Sequencing` ships alone;
/// `Scoping { budget }` is Phase E's slot. The relevant-pair predicate varies
/// by context; the yield machinery is context-blind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecisionContext {
    /// The sequencing context: probe the top-`depth` frontier band.
    Sequencing { depth: usize },
}

/// One frontier item, ranked best-first: its entity id and kind prefix. The
/// kind drives the capture-admissibility gate (design §2 source 1) — the same
/// `admissible_value_pair` rule `compare record` applies, no second rule set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrontierItem {
    pub id: String,
    pub kind: String,
}

/// Per-item costing, built by the PHASE-03 shell from the priority graph
/// (`m = coeff.value × kind_weight × tag_term`, design D6; `est_cost` per the
/// β-skewed model). `bare_estimate` flags a projected/gauge value with no
/// estimate facet (D17 mask).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ItemCosting {
    pub multiplier: f64,
    pub est_cost: f64,
    pub bare_estimate: bool,
}

/// The pure inputs to [`assemble`] (design §2). The shell fills these from the
/// composed pipeline; the assembler needs no disk. `rank_decay` /
/// `confirm_boost` are the `[priority.elicit]` numeric shapes (D13), passed as
/// pure config inputs.
#[derive(Debug, Clone)]
pub(crate) struct ElicitInputs<'a> {
    pub active: Vec<&'a Judgement>,
    pub anchors: AnchorMap,
    pub frontier: Vec<FrontierItem>,
    pub costing: BTreeMap<String, ItemCosting>,
    pub projection: Projection,
    pub rank_decay: f64,
    pub confirm_boost: f64,
    /// SL-218 D1: knob-on, every determinacy verdict is read over the
    /// human-rows-only system; the full system keeps bounds, projection,
    /// and the queue pool.
    pub demote_agent_evidence: bool,
}

// ── queue model ─────────────────────────────────────────────────────────────

/// The queue state (design D15, precedence pinned): any entries ⇒ `Candidates`;
/// no entries with every value-sensitive top-K pair determined ⇒ `Stable`; no
/// entries with an indeterminate pair (a zero-yield bridge) ⇒ `Stalled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueueState {
    Candidates,
    Stalled { depth: usize },
    Stable { depth: usize },
}

/// The candidate kind (design D16, kind-tagged entries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CandidateKind {
    Comparison,
    AnchorReview,
}

/// The answer space an entry's guaranteed yield ranges over (design D11) —
/// disclosed so a curator knows the numbers stay spine-comparable but the
/// semantics differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum YieldBasis {
    OrderBearingAnswers,
    CanonicalResolvingActions,
}

/// A structured reason (design D16, findings JSON-parity idiom).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reason {
    pub code: String,
    pub text: String,
}

/// A lean participant (design D16): id + annotations; no body summaries — the
/// render fetches entity context itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Participant {
    pub id: String,
    pub annotations: Vec<String>,
}

/// The suspect anchor an anchor-review entry is about.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnchorSubject {
    pub id: String,
    pub anchor: Option<f64>,
    pub conflict_pairs: Vec<(String, String)>,
    pub quarantined_rows: Vec<String>,
}

/// The ask block: the answer tokens, each token's disclosed yield, and (for
/// anchor-review) the conditional-yield note (design §3, RV-269 F-3).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AskSpec {
    pub answers: Vec<&'static str>,
    pub yield_by_answer: BTreeMap<String, i64>,
    pub yield_note: Option<String>,
}

/// The kind-specific payload (design §2/§3).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EntryPayload {
    Comparison {
        a: Participant,
        b: Participant,
        ask: AskSpec,
    },
    AnchorReview {
        subject: AnchorSubject,
        ask: AskSpec,
    },
}

/// One ranked queue entry (design §2 spine).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct QueueEntry {
    pub kind: CandidateKind,
    pub guaranteed_yield: i64,
    pub guaranteed_impact: f64,
    pub score: f64,
    pub yield_basis: YieldBasis,
    pub reasons: Vec<Reason>,
    pub payload: EntryPayload,
}

/// The assembled queue: its state, the ranked entries, and the D6 disclosure
/// count of top-K pairs dropped as value-insensitive (`m = 0`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ElicitQueue {
    pub state: QueueState,
    pub entries: Vec<QueueEntry>,
    pub excluded_value_insensitive: usize,
}

// ── reason / ask codes (STD-001) ────────────────────────────────────────────

const REASON_FRONTIER_PAIR: &str = "indeterminate-frontier-pair";
const REASON_MEDIAN_PROBE: &str = "median-probe";
const REASON_AGENT_ONLY: &str = "agent-only-calibration";
const REASON_STALE_ANCHOR: &str = "stale-anchor-suspect";

const ANSWER_PREFER_A: &str = "prefer-a";
const ANSWER_PREFER_B: &str = "prefer-b";
const ANSWER_EQUAL: &str = "equal";
const ANSWER_INCOMPARABLE: &str = "incomparable";
// Anchor-review answer tokens are `pub(crate)` — the elicit JSON surface keys
// each entry's `exits` action list by answer token (design §3), so the shell
// shares this ONE definition rather than restating the strings (STD-001).
pub(crate) const ANSWER_REVISE_ANCHOR: &str = "revise-anchor";
pub(crate) const ANSWER_UPHOLD_ANCHOR: &str = "uphold-anchor";

const MASK_ANNOTATION: &str = "projection masked by bare estimate";
const ANCHOR_YIELD_NOTE: &str = "revise-anchor yield assumes a RESOLVING revision (conflict \
    removed); a still-conflicting value yields nothing and re-surfaces this candidate next \
    refresh. uphold-anchor models retiring the COMPLETE cited closure — real yield may exceed it";

/// The unbounded interval, for a classless frontier item with zero rows
/// (design decision: `PairSide` over its own entity id, Free coupling).
const UNBOUNDED: ValueBounds = ValueBounds {
    lower: Bound::Unbounded,
    upper: Bound::Unbounded,
};

// ── pool items ──────────────────────────────────────────────────────────────

/// A top-K value-sensitive item, resolved against the compiled baseline.
/// Class and interval facts are NOT stored: a [`PairSide`] is resolved per
/// verdict system via [`side_in`] (SL-218 D1).
#[derive(Debug, Clone)]
struct PoolItem {
    id: String,
    kind: String,
    multiplier: f64,
    cost: f64,
    constrained: bool,
    agent_only: bool,
    bare: bool,
}

/// `w == 0` without a `float_cmp` footgun.
fn is_zero(w: f64) -> bool {
    w.abs().total_cmp(&0.0).is_eq()
}

/// The rank-decay weight of a newly-determined pair at better frontier rank `r`
/// (design D13): `w(r) = 1/(1 + decay·r)`, `r` 0-based.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "frontier ranks are tiny counts, far from f64 precision limits"
)]
fn rank_weight(r: usize, decay: f64) -> f64 {
    1.0 / (1.0 + decay * (r as f64))
}

/// Resolve a class's better (min) 0-based rank among the top-K frontier
/// members that name it; a class with no member in the band ranks at `depth`.
fn class_rank(rank_map: &BTreeMap<ClassId, usize>, class: &ClassId, depth: usize) -> usize {
    rank_map.get(class).copied().unwrap_or(depth)
}

/// The system that issues determinacy verdicts (SL-218 D1): the baseline full
/// system knob-off; the fresh human-rows-only system knob-on. Pool
/// composition, bounds display, projection, and confirm-boost stay on the
/// full system (INV-3) — only `determined()` and its hypothetical diffs move.
struct VerdictSystem<'s, 'a> {
    cs: &'s ConstraintSet,
    reach: &'s Reachability,
    rows: &'s [&'a Judgement],
}

/// Resolve an entity's [`PairSide`] in `cs` — its class, C6 interval, and anchor
/// — carrying the given pre-computed `eff_weight` (design D6: `m_self·c_other`).
/// An entity absent from that system falls back to a singleton class with
/// unbounded interval, so the pair reads indeterminate rather than panicking
/// (SL-218 D1). The SINGLE [`PairSide`] resolver — the elicit queue ([`side_in`])
/// and the PHASE-02 tension grader ([`super::surface`]) both go through it, so
/// grade and queue read one predicate over one system (design F-1/F-7).
pub(crate) fn pair_side(cs: &ConstraintSet, id: &str, eff_weight: f64) -> PairSide {
    let class = cs
        .classes
        .get(id)
        .cloned()
        .unwrap_or_else(|| id.to_string());
    let bounds = cs.bounds.get(&class).copied().unwrap_or(UNBOUNDED);
    let anchor = cs.anchors.get(&class).copied();
    PairSide {
        class,
        eff_weight,
        bounds,
        anchor,
    }
}

/// The pool item's [`PairSide`] against a partner whose cost is `cost_other`
/// (design D6: `eff_weight = m_self · c_other`, built per pair), resolved in
/// `cs` — the verdict system's constraint set.
fn side_in(cs: &ConstraintSet, item: &PoolItem, cost_other: f64) -> PairSide {
    pair_side(cs, &item.id, item.multiplier * cost_other)
}

/// One answer's evaluated determinacy outcome over the relevant pool pairs.
struct AnswerEval {
    yield_delta: i64,
    newly: Vec<(ClassId, ClassId)>,
}

/// The impact of one answer: `Σ w(r)` over its newly-determined pairs (design
/// D13), `r` the better frontier rank in each pair.
fn answer_impact(
    newly: &[(ClassId, ClassId)],
    rank_map: &BTreeMap<ClassId, usize>,
    depth: usize,
    decay: f64,
) -> f64 {
    newly
        .iter()
        .map(|(ca, cb)| {
            let r = class_rank(rank_map, ca, depth).min(class_rank(rank_map, cb, depth));
            rank_weight(r, decay)
        })
        .sum()
}

/// Reduce an answer set to `(guaranteed_yield, guaranteed_impact)`: the min
/// yield, then the min impact over the answers attaining it (design D13 —
/// worst-case in both count and placement, answer-token invariant).
fn reduce_answers(
    evals: &[AnswerEval],
    rank_map: &BTreeMap<ClassId, usize>,
    depth: usize,
    decay: f64,
) -> Option<(i64, f64)> {
    let gy = evals.iter().map(|e| e.yield_delta).min()?;
    let gi = evals
        .iter()
        .filter(|e| e.yield_delta == gy)
        .map(|e| answer_impact(&e.newly, rank_map, depth, decay))
        .reduce(|a, b| if a.total_cmp(&b).is_le() { a } else { b })?;
    Some((gy, gi))
}

// ── candidate assembly ──────────────────────────────────────────────────────

/// A ranked candidate before sort: the entry plus its lexicographic tiebreak
/// key (design D13: id-lexicographic tiebreak, `total_cmp` on scores).
struct Candidate {
    sort_key: String,
    entry: QueueEntry,
}

/// Assemble the elicitation queue (design §2). Pure over `inputs`; compiles the
/// baseline internally and rides the `comparison::query` predicates.
pub(crate) fn assemble(inputs: &ElicitInputs<'_>, ctx: DecisionContext) -> ElicitQueue {
    let DecisionContext::Sequencing { depth } = ctx;
    let cs = compile(&inputs.active, &inputs.anchors, QuarantinePolicy::Symmetric);
    let reach = Reachability::build(&cs);
    let counts = constraining_counts_by_class(&cs, &inputs.active);

    // The verdict system (SL-218 D1): knob-on, determinacy is read over a
    // fresh human-rows-only compile (its own C2–C4); knob-off it IS the
    // baseline — no second compile, shipped behaviour bit-for-bit.
    let human = inputs.demote_agent_evidence.then(|| {
        let vcs = compile_human_only(&inputs.active, &inputs.anchors, QuarantinePolicy::Symmetric);
        let vreach = Reachability::build(&vcs);
        (vcs, vreach, human_rows(&inputs.active))
    });
    let verdict = match &human {
        Some((vcs, vreach, vrows)) => VerdictSystem {
            cs: vcs,
            reach: vreach,
            rows: vrows,
        },
        None => VerdictSystem {
            cs: &cs,
            reach: &reach,
            rows: &inputs.active,
        },
    };

    // Top-K frontier band and its class → best-rank map (all members, not just
    // value-sensitive ones — rank is a frontier fact).
    let band: Vec<&FrontierItem> = inputs.frontier.iter().take(depth).collect();
    let rank_map = build_rank_map(&band, &cs);

    // Value-bearing top-K items, split into the value-sensitive pool (m > 0)
    // and the value-insensitive exclusions (m = 0, design D6).
    let mut pool: Vec<PoolItem> = Vec::new();
    let mut value_bearing = 0_usize;
    for item in &band {
        if admissible_value_pair(&item.kind, &item.kind).is_err() {
            continue; // not value-bearing (or a risk): outside the value pool
        }
        let Some(costing) = inputs.costing.get(&item.id) else {
            continue; // no costing: the shell could not price it — skip
        };
        value_bearing += 1;
        if is_zero(costing.multiplier) {
            continue; // value-insensitive: excluded from pool AND stability (D6)
        }
        pool.push(resolve_item(
            item,
            costing,
            &cs,
            &counts,
            &inputs.projection,
        ));
    }
    let n_pool = pool.len();
    let excluded_value_insensitive = pairs(value_bearing).saturating_sub(pairs(n_pool));

    // The relevant pair set for every yield: ALL pool pairs (flips counted both
    // directions by `hypothetical_outcome`). Keyed off the VERDICT system's
    // classes — determinacy diffs must live where the verdicts do (SL-218 D1).
    let relevant = relevant_pairs(verdict.cs, &pool);

    let mut candidates: Vec<Candidate> = Vec::new();
    comparison_candidates(
        inputs,
        &pool,
        &verdict,
        &relevant,
        &rank_map,
        depth,
        &mut candidates,
    );
    median_probe_candidates(
        inputs,
        &pool,
        &verdict,
        &relevant,
        &rank_map,
        depth,
        &mut candidates,
    );
    anchor_review_candidates(
        inputs,
        &cs,
        &verdict,
        &relevant,
        &rank_map,
        depth,
        &mut candidates,
    );

    // Rank: score desc via total_cmp, id-lexicographic tiebreak.
    candidates.sort_by(|a, b| {
        b.entry
            .score
            .total_cmp(&a.entry.score)
            .then_with(|| a.sort_key.cmp(&b.sort_key))
    });
    let entries: Vec<QueueEntry> = candidates.into_iter().map(|c| c.entry).collect();

    let state = if !entries.is_empty() {
        QueueState::Candidates
    } else if pool_has_indeterminate(&pool, &verdict) {
        QueueState::Stalled { depth }
    } else {
        QueueState::Stable { depth }
    };

    ElicitQueue {
        state,
        entries,
        excluded_value_insensitive,
    }
}

/// `n·(n−1)/2` — the pair count over `n` items.
#[expect(clippy::integer_division, reason = "exact: n·(n−1) is even")]
fn pairs(n: usize) -> usize {
    n.saturating_mul(n.saturating_sub(1)) / 2
}

/// Class → best (min) 0-based frontier rank among the top-K members naming it.
fn build_rank_map(band: &[&FrontierItem], cs: &ConstraintSet) -> BTreeMap<ClassId, usize> {
    let mut out: BTreeMap<ClassId, usize> = BTreeMap::new();
    for (r, item) in band.iter().enumerate() {
        let class = cs
            .classes
            .get(&item.id)
            .cloned()
            .unwrap_or_else(|| item.id.clone());
        out.entry(class)
            .and_modify(|best| *best = (*best).min(r))
            .or_insert(r);
    }
    out
}

/// Resolve a frontier item against the compiled baseline: its class, interval,
/// anchor, constrained/agent-only flags, and bare-estimate mask.
fn resolve_item(
    item: &FrontierItem,
    costing: &ItemCosting,
    cs: &ConstraintSet,
    counts: &BTreeMap<ClassId, crate::comparison::RaterCounts>,
    projection: &Projection,
) -> PoolItem {
    let class = cs
        .classes
        .get(&item.id)
        .cloned()
        .unwrap_or_else(|| item.id.clone());
    let anchor = cs.anchors.get(&class).copied();
    let class_counts = counts.get(&class).copied().unwrap_or_default();
    let constrained = class_counts.total() > 0 || anchor.is_some();
    let agent_only = class_counts.human == 0 && class_counts.agent >= 1;
    let bare = costing.bare_estimate && is_masked(projection.get(&item.id));
    PoolItem {
        id: item.id.clone(),
        kind: item.kind.clone(),
        multiplier: costing.multiplier,
        cost: costing.est_cost,
        constrained,
        agent_only,
        bare,
    }
}

/// The bare-estimate mask applies only over a projected/gauge value (design
/// D17): an authored anchor is exact, not masked.
fn is_masked(projected: Option<&(f64, ValueProvenance)>) -> bool {
    matches!(
        projected,
        Some((_, ValueProvenance::Projected | ValueProvenance::Gauge))
    )
}

/// Every pool pair as a `(PairSide, PairSide)` with per-pair effective
/// weights, resolved in the verdict system's constraint set.
fn relevant_pairs(cs: &ConstraintSet, pool: &[PoolItem]) -> Vec<(PairSide, PairSide)> {
    let mut out = Vec::new();
    for (i, a) in pool.iter().enumerate() {
        for b in pool.iter().skip(i + 1) {
            out.push((side_in(cs, a, b.cost), side_in(cs, b, a.cost)));
        }
    }
    out
}

/// Is any pool pair indeterminate (design D15 stall/stable discriminator)?
fn pool_has_indeterminate(pool: &[PoolItem], verdict: &VerdictSystem<'_, '_>) -> bool {
    for (i, a) in pool.iter().enumerate() {
        for b in pool.iter().skip(i + 1) {
            let (sa, sb) = (
                side_in(verdict.cs, a, b.cost),
                side_in(verdict.cs, b, a.cost),
            );
            if !determined(verdict.reach, &sa, &sb).is_determined() {
                return true;
            }
        }
    }
    false
}

/// Source 1: indeterminate pairs among the *constrained* pool items, behind the
/// capture-admissibility gate. Admission `guaranteed_yield > 0`.
fn comparison_candidates(
    inputs: &ElicitInputs<'_>,
    pool: &[PoolItem],
    verdict: &VerdictSystem<'_, '_>,
    relevant: &[(PairSide, PairSide)],
    rank_map: &BTreeMap<ClassId, usize>,
    depth: usize,
    out: &mut Vec<Candidate>,
) {
    for (i, a) in pool.iter().enumerate() {
        for b in pool.iter().skip(i + 1) {
            if !a.constrained || !b.constrained {
                continue; // un-constrained items are median-probe's job
            }
            if admissible_value_pair(&a.kind, &b.kind).is_err() {
                continue;
            }
            let (sa, sb) = (
                side_in(verdict.cs, a, b.cost),
                side_in(verdict.cs, b, a.cost),
            );
            if determined(verdict.reach, &sa, &sb).is_determined() {
                continue; // already fixed — nothing to ask
            }
            if let Some(entry) = build_comparison(
                inputs,
                verdict,
                a,
                b,
                relevant,
                rank_map,
                depth,
                REASON_FRONTIER_PAIR,
            ) {
                out.push(entry);
            }
        }
    }
}

/// Source 2: one median-probe per un-constrained top-K item, against the
/// projected-median comparable item (design D14). Stateless, heuristic.
fn median_probe_candidates(
    inputs: &ElicitInputs<'_>,
    pool: &[PoolItem],
    verdict: &VerdictSystem<'_, '_>,
    relevant: &[(PairSide, PairSide)],
    rank_map: &BTreeMap<ClassId, usize>,
    depth: usize,
    out: &mut Vec<Candidate>,
) {
    for u in pool.iter().filter(|p| !p.constrained) {
        let Some(target) = median_target(inputs, pool, u) else {
            continue;
        };
        if let Some(entry) = build_comparison(
            inputs,
            verdict,
            u,
            target,
            relevant,
            rank_map,
            depth,
            REASON_MEDIAN_PROBE,
        ) {
            out.push(entry);
        }
    }
}

/// The comparable item nearest the projected median of `u`'s comparable set —
/// the other pool items with a projection (design D14). Deterministic: ties
/// break to the smaller projected value, then the smaller id.
#[expect(
    clippy::integer_division,
    reason = "median index; integer halving is intended"
)]
fn median_target<'p>(
    inputs: &ElicitInputs<'_>,
    pool: &'p [PoolItem],
    u: &PoolItem,
) -> Option<&'p PoolItem> {
    let mut comparable: Vec<(&PoolItem, f64)> = pool
        .iter()
        .filter(|p| p.id != u.id)
        .filter(|p| admissible_value_pair(&u.kind, &p.kind).is_ok())
        .filter_map(|p| inputs.projection.get(&p.id).map(|&(v, _)| (p, v)))
        .collect();
    if comparable.is_empty() {
        return None;
    }
    comparable.sort_by(|(pa, va), (pb, vb)| va.total_cmp(vb).then_with(|| pa.id.cmp(&pb.id)));
    let mid = comparable.len() / 2;
    let median = comparable.get(mid).map_or(0.0, |&(_, v)| v);
    comparable
        .into_iter()
        .min_by(|(pa, va), (pb, vb)| {
            (va - median)
                .abs()
                .total_cmp(&(vb - median).abs())
                .then_with(|| pa.id.cmp(&pb.id))
        })
        .map(|(p, _)| p)
}

/// Build a comparison-kind candidate (sources 1 and 2 share this): synthetic
/// order-bearing answers, min-over-order-bearing yield, argmin-yield impact,
/// confirm-boost, admission `guaranteed_yield > 0`.
#[expect(
    clippy::too_many_arguments,
    reason = "yield inputs + impact-band context (rank_map, depth) fanned to a private helper"
)]
fn build_comparison(
    inputs: &ElicitInputs<'_>,
    verdict: &VerdictSystem<'_, '_>,
    a: &PoolItem,
    b: &PoolItem,
    relevant: &[(PairSide, PairSide)],
    rank_map: &BTreeMap<ClassId, usize>,
    depth: usize,
    reason_code: &str,
) -> Option<Candidate> {
    let order_bearing = [
        (ANSWER_PREFER_A, Response::PreferA),
        (ANSWER_PREFER_B, Response::PreferB),
        (ANSWER_EQUAL, Response::Equal),
    ];
    let mut evals: Vec<AnswerEval> = Vec::new();
    let mut yield_by_answer: BTreeMap<String, i64> = BTreeMap::new();
    for (token, response) in order_bearing {
        // The synthetic answer joins the VERDICT system's rows: fresh
        // testimony always constrains — post-filter, its rater is moot.
        let row = synthetic_answer_row(&a.id, &b.id, response);
        let outcome = hypothetical_outcome(
            verdict.reach,
            verdict.rows,
            &inputs.anchors,
            &Hypothetical::Answer(Box::new(row)),
            relevant,
        );
        yield_by_answer.insert(token.to_string(), outcome.yield_delta());
        evals.push(AnswerEval {
            yield_delta: outcome.yield_delta(),
            newly: outcome.newly_determined,
        });
    }
    // `incomparable` is structurally 0 — disclosed, excluded from the min (D11).
    yield_by_answer.insert(ANSWER_INCOMPARABLE.to_string(), 0);

    let (gy, gi) = reduce_answers(&evals, rank_map, depth, inputs.rank_decay)?;
    if gy <= 0 {
        return None; // admission: yield-motivated sources need positive yield
    }
    let boost = if a.agent_only && b.agent_only {
        inputs.confirm_boost
    } else {
        1.0
    };
    let score = i64_as_f64(gy) * gi * boost;

    let mut reasons = vec![Reason {
        code: reason_code.to_string(),
        text: comparison_reason_text(reason_code),
    }];
    if boost.total_cmp(&1.0).is_gt() {
        reasons.push(Reason {
            code: REASON_AGENT_ONLY.to_string(),
            text: "both items currently calibrated only by agent evidence".to_string(),
        });
    }

    let ask = AskSpec {
        answers: vec![
            ANSWER_PREFER_A,
            ANSWER_PREFER_B,
            ANSWER_EQUAL,
            ANSWER_INCOMPARABLE,
        ],
        yield_by_answer,
        yield_note: None,
    };
    let payload = EntryPayload::Comparison {
        a: participant(a),
        b: participant(b),
        ask,
    };
    let (lo, hi) = if a.id <= b.id {
        (&a.id, &b.id)
    } else {
        (&b.id, &a.id)
    };
    Some(Candidate {
        sort_key: format!("cmp:{lo}:{hi}"),
        entry: QueueEntry {
            kind: CandidateKind::Comparison,
            guaranteed_yield: gy,
            guaranteed_impact: gi,
            score,
            yield_basis: YieldBasis::OrderBearingAnswers,
            reasons,
            payload,
        },
    })
}

fn comparison_reason_text(code: &str) -> String {
    if code == REASON_MEDIAN_PROBE {
        "un-constrained item — calibrate against the projected median of its comparable set"
            .to_string()
    } else {
        "an indeterminate value_dim order between two top-K frontier items".to_string()
    }
}

fn participant(item: &PoolItem) -> Participant {
    let mut annotations = Vec::new();
    if item.bare {
        annotations.push(MASK_ANNOTATION.to_string());
    }
    Participant {
        id: item.id.clone(),
        annotations,
    }
}

/// Source 3: one candidate per distinct suspect anchor in an `AnchorConflict`
/// quarantine pair (design D12). Admits on EXISTENCE; yield = min over the two
/// resolving outcomes (revise-as-removal, uphold-as-rows-retired); NOT K-gated.
fn anchor_review_candidates(
    inputs: &ElicitInputs<'_>,
    cs: &ConstraintSet,
    verdict: &VerdictSystem<'_, '_>,
    relevant: &[(PairSide, PairSide)],
    rank_map: &BTreeMap<ClassId, usize>,
    depth: usize,
    out: &mut Vec<Candidate>,
) {
    // Suspects come from the FULL system's quarantine diagnostics (the pool
    // stays full-system, SL-218 D1); only the yield evaluation — a
    // determinacy diff — moves to the verdict system.
    for suspect in suspect_anchors(cs, &inputs.anchors) {
        let rows = rows_citing(cs, &suspect);
        let removed = hypothetical_outcome(
            verdict.reach,
            verdict.rows,
            &inputs.anchors,
            &Hypothetical::AnchorRemoved(&suspect),
            relevant,
        );
        let retired = hypothetical_outcome(
            verdict.reach,
            verdict.rows,
            &inputs.anchors,
            &Hypothetical::RowsRetired(&rows),
            relevant,
        );
        let revise_yield = removed.yield_delta();
        let uphold_yield = retired.yield_delta();
        let evals = [
            AnswerEval {
                yield_delta: revise_yield,
                newly: removed.newly_determined,
            },
            AnswerEval {
                yield_delta: uphold_yield,
                newly: retired.newly_determined,
            },
        ];
        let Some((gy, gi)) = reduce_answers(&evals, rank_map, depth, inputs.rank_decay) else {
            continue;
        };
        // Existence admission (D15): a suspect stays on the queue whatever its
        // yield — zero-yield suspects sink to the bottom, never vanish.
        let score = i64_as_f64(gy).max(0.0) * gi;

        let mut yield_by_answer = BTreeMap::new();
        yield_by_answer.insert(ANSWER_REVISE_ANCHOR.to_string(), revise_yield);
        yield_by_answer.insert(ANSWER_UPHOLD_ANCHOR.to_string(), uphold_yield);

        let subject = AnchorSubject {
            id: suspect.clone(),
            anchor: inputs.anchors.get(&suspect).copied(),
            conflict_pairs: conflict_pairs_for(cs, &suspect),
            quarantined_rows: rows.iter().cloned().collect(),
        };
        let ask = AskSpec {
            answers: vec![ANSWER_REVISE_ANCHOR, ANSWER_UPHOLD_ANCHOR],
            yield_by_answer,
            yield_note: Some(ANCHOR_YIELD_NOTE.to_string()),
        };
        out.push(Candidate {
            sort_key: format!("anc:{suspect}"),
            entry: QueueEntry {
                kind: CandidateKind::AnchorReview,
                guaranteed_yield: gy,
                guaranteed_impact: gi,
                score,
                yield_basis: YieldBasis::CanonicalResolvingActions,
                reasons: vec![Reason {
                    code: REASON_STALE_ANCHOR.to_string(),
                    text: format!("anchor on {suspect} sits on a quarantined conflict path"),
                }],
                payload: EntryPayload::AnchorReview { subject, ask },
            },
        });
    }
}

/// The distinct anchored entities named (directly, or via a class-id token) in
/// any `AnchorConflict` quarantine pair (design D12; notes: resolve class-id
/// tokens to their anchored member via the `AnchorMap`).
fn suspect_anchors(cs: &ConstraintSet, anchors: &AnchorMap) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for reason in cs.quarantined.values() {
        if let QuarantineReason::AnchorConflict { pairs } = reason {
            for (x, y) in pairs {
                for token in [x, y] {
                    if let Some(entity) = resolve_anchored(token, anchors, cs) {
                        out.insert(entity);
                    }
                }
            }
        }
    }
    out.into_iter().collect()
}

/// Resolve a conflict-pair token to the anchored entity it names: a direct
/// `AnchorMap` key (C2 member tokens), else the anchored member of the class it
/// names (C4 class-id tokens).
fn resolve_anchored(token: &str, anchors: &AnchorMap, cs: &ConstraintSet) -> Option<String> {
    if anchors.contains_key(token) {
        return Some(token.to_string());
    }
    cs.classes
        .iter()
        .find(|(entity, class)| class.as_str() == token && anchors.contains_key(*entity))
        .map(|(entity, _)| entity.clone())
}

/// Every row uid whose `AnchorConflict` quarantine entry cites `suspect`
/// (design D12: the complete cited closure, deliberately pessimistic).
fn rows_citing(cs: &ConstraintSet, suspect: &str) -> BTreeSet<RowUid> {
    let mut out = BTreeSet::new();
    for (uid, reason) in &cs.quarantined {
        if let QuarantineReason::AnchorConflict { pairs } = reason
            && pairs
                .iter()
                .any(|(x, y)| cites(x, suspect, cs) || cites(y, suspect, cs))
        {
            out.insert(uid.clone());
        }
    }
    out
}

/// Does conflict token `token` name `suspect` — directly, or as the class of
/// the suspect entity?
fn cites(token: &str, suspect: &str, cs: &ConstraintSet) -> bool {
    token == suspect || cs.classes.get(suspect).is_some_and(|c| c.as_str() == token)
}

/// The conflict pairs any quarantine entry citing `suspect` records (display).
fn conflict_pairs_for(cs: &ConstraintSet, suspect: &str) -> Vec<(String, String)> {
    let mut out: BTreeSet<(String, String)> = BTreeSet::new();
    for reason in cs.quarantined.values() {
        if let QuarantineReason::AnchorConflict { pairs } = reason {
            for (x, y) in pairs {
                if cites(x, suspect, cs) || cites(y, suspect, cs) {
                    out.insert((x.clone(), y.clone()));
                }
            }
        }
    }
    out.into_iter().collect()
}

/// `i64 → f64` for the score product (yields are tiny counts).
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "guaranteed yields are small determinacy counts, exact in f64"
)]
fn i64_as_f64(v: i64) -> f64 {
    v as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comparison::{DOMAIN_VALUE, FRAME_EQUAL_EFFORT, RaterKind, RowForm};

    // ---- fixtures --------------------------------------------------------------

    fn jrow(uid: &str, a: &str, b: &str, response: Response, rater: RaterKind) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: a.to_string(),
            b: b.to_string(),
            response,
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater,
            by: None,
            note: None,
            date: "2026-07-12".to_string(),
        }
    }

    fn win(uid: &str, w: &str, l: &str) -> Judgement {
        jrow(uid, w, l, Response::PreferA, RaterKind::Human)
    }
    fn win_agent(uid: &str, w: &str, l: &str) -> Judgement {
        jrow(uid, w, l, Response::PreferA, RaterKind::Agent)
    }

    /// Build inputs; every frontier item is kind `IMP` (value-bearing),
    /// `bare_estimate = false`, `rank_decay = 1.0`, `confirm_boost = 1.5`.
    fn mk<'a>(
        active: Vec<&'a Judgement>,
        anchors: &[(&str, f64)],
        frontier: &[&str],
        costing: &[(&str, f64, f64)],
        projection: &[(&str, f64, ValueProvenance)],
    ) -> ElicitInputs<'a> {
        ElicitInputs {
            active,
            anchors: anchors.iter().map(|&(e, v)| (e.to_string(), v)).collect(),
            frontier: frontier
                .iter()
                .map(|&id| FrontierItem {
                    id: id.to_string(),
                    kind: "IMP".to_string(),
                })
                .collect(),
            costing: costing
                .iter()
                .map(|&(id, m, c)| {
                    (
                        id.to_string(),
                        ItemCosting {
                            multiplier: m,
                            est_cost: c,
                            bare_estimate: false,
                        },
                    )
                })
                .collect(),
            projection: projection
                .iter()
                .map(|&(id, v, p)| (id.to_string(), (v, p)))
                .collect(),
            rank_decay: 1.0,
            confirm_boost: 1.5,
            demote_agent_evidence: false,
        }
    }

    fn seq(depth: usize) -> DecisionContext {
        DecisionContext::Sequencing { depth }
    }

    // ---- VT-4: state machine (QueueState / Stalled / Stable) --------------------

    #[test]
    fn indeterminate_constrained_pair_is_a_comparison_candidate() {
        // A>C, B>D: A, B both constrained, order-incomparable, unbounded ⇒ the
        // (A, B) value_dim order is open ⇒ one comparison candidate; state
        // Candidates.
        let rows = vec![win("j0", "A", "C"), win("j1", "B", "D")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[],
            &["A", "B"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0)],
            &[],
        );
        let q = assemble(&inputs, seq(2));
        assert_eq!(q.state, QueueState::Candidates);
        assert_eq!(q.entries.len(), 1);
        assert_eq!(q.entries[0].kind, CandidateKind::Comparison);
        assert!(q.entries[0].guaranteed_yield >= 1);
    }

    #[test]
    fn all_determined_pool_no_suspects_is_stable() {
        // A(5) and B(3) both anchored (consistent chain over C=0): the pair is
        // point-determined; no suspects, no un-constrained items ⇒ Stable.
        let rows = vec![win("j0", "A", "C"), win("j1", "B", "C")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[("A", 5.0), ("B", 3.0), ("C", 0.0)],
            &["A", "B"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0)],
            &[],
        );
        let q = assemble(&inputs, seq(2));
        assert_eq!(q.state, QueueState::Stable { depth: 2 });
        assert!(q.entries.is_empty());
    }

    #[test]
    fn zero_yield_bridge_drops_admission_and_stalls() {
        // T(5) > A,B > L(-5): A, B both span (-5, 5). With differing costs the
        // pair objective 2·v_A − v_B stays sign-mixed under EVERY order-bearing
        // answer (negative domain), so guaranteed yield is 0 ⇒ admission drops
        // the candidate ⇒ no entries, but the pair is still indeterminate ⇒
        // Stalled, never Stable (D15 zero-yield bridge; RV-269 admission).
        let rows = vec![
            win("j0", "T", "A"),
            win("j1", "A", "L"),
            win("j2", "T", "B"),
            win("j3", "B", "L"),
        ];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[("T", 5.0), ("L", -5.0)],
            &["A", "B"],
            &[("A", 1.0, 1.0), ("B", 1.0, 2.0)],
            &[],
        );
        let q = assemble(&inputs, seq(2));
        assert!(q.entries.is_empty(), "zero-yield candidate not admitted");
        assert_eq!(q.state, QueueState::Stalled { depth: 2 });
    }

    // ---- VT-3: ranking (confirm_boost / guaranteed_impact / argmin) -------------

    #[test]
    fn confirm_boost_agent_only_outranks_human_touched() {
        // The SAME structural pair: agent-only evidence earns the boost, human
        // evidence does not; score_agent = confirm_boost × score_human, and the
        // agent case discloses the agent-only-calibration reason.
        let agent_rows = vec![win_agent("j0", "A", "C"), win_agent("j1", "B", "D")];
        let human_rows = vec![win("j0", "A", "C"), win("j1", "B", "D")];
        let a_refs: Vec<&Judgement> = agent_rows.iter().collect();
        let h_refs: Vec<&Judgement> = human_rows.iter().collect();
        let cost = [("A", 1.0, 1.0), ("B", 1.0, 1.0)];
        let qa = assemble(&mk(a_refs.clone(), &[], &["A", "B"], &cost, &[]), seq(2));
        let qh = assemble(&mk(h_refs.clone(), &[], &["A", "B"], &cost, &[]), seq(2));
        let sa = qa.entries[0].score;
        let sh = qh.entries[0].score;
        assert!(sa > sh, "agent-only outranks human-touched");
        assert!(
            (sa - sh * 1.5).abs() < 1e-9,
            "score scales by confirm_boost"
        );
        assert!(
            qa.entries[0]
                .reasons
                .iter()
                .any(|r| r.code == "agent-only-calibration"),
            "agent case discloses the boost reason"
        );
        assert!(
            qh.entries[0]
                .reasons
                .iter()
                .all(|r| r.code != "agent-only-calibration"),
            "human case claims no boost"
        );
    }

    #[test]
    fn guaranteed_impact_is_min_over_argmin_yield_answers() {
        // RV-269 F-2: two answers tie on the min yield (1) but close pairs at
        // different frontier ranks — prefer-a a rank-0 pair (impact 1.0),
        // prefer-b a rank-1 pair (impact 0.5); guaranteed_impact is the WORSE
        // (min) of the argmin-yield answers = 0.5. A higher-yield answer is
        // excluded from the argmin set.
        let rank_map: BTreeMap<ClassId, usize> = [("A", 0usize), ("B", 1), ("C", 2)]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        let evals = vec![
            AnswerEval {
                yield_delta: 1,
                newly: vec![("A".to_string(), "B".to_string())],
            },
            AnswerEval {
                yield_delta: 1,
                newly: vec![("B".to_string(), "C".to_string())],
            },
            AnswerEval {
                yield_delta: 2,
                newly: vec![
                    ("A".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                ],
            },
        ];
        let (gy, gi) = reduce_answers(&evals, &rank_map, 3, 1.0).unwrap();
        assert_eq!(gy, 1);
        assert!((gi - 0.5).abs() < 1e-9, "min over argmin-yield answers");
    }

    #[test]
    fn rank_weight_decays_monotonically() {
        assert!((rank_weight(0, 1.0) - 1.0).abs() < 1e-9);
        assert!(rank_weight(0, 1.0) > rank_weight(1, 1.0));
        assert!(rank_weight(1, 1.0) > rank_weight(2, 1.0));
    }

    #[test]
    fn value_insensitive_zero_multiplier_excluded_and_counted() {
        // A, B, Z all value-bearing, constrained, pairwise indeterminate; Z has
        // m = 0 ⇒ excluded from the pool AND the stability obligation, and the
        // dropped pairs (A,Z),(B,Z) are disclosed. Only (A,B) survives.
        let rows = vec![
            win("j0", "A", "P"),
            win("j1", "B", "Q"),
            win("j2", "Z", "R"),
        ];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[],
            &["A", "B", "Z"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0), ("Z", 0.0, 1.0)],
            &[],
        );
        let q = assemble(&inputs, seq(3));
        assert_eq!(q.excluded_value_insensitive, 2, "pairs (A,Z),(B,Z) dropped");
        assert_eq!(q.entries.len(), 1);
        for e in &q.entries {
            if let EntryPayload::Comparison { a, b, .. } = &e.payload {
                assert!(a.id != "Z" && b.id != "Z", "Z never surfaces");
            }
        }
    }

    #[test]
    fn entries_sorted_by_score_then_id() {
        // Two independent stale-anchor suspects (A, C) both score 0 ⇒ the id
        // tiebreak orders `anc:A` before `anc:C`.
        let rows = vec![win("j0", "A", "B"), win("j1", "B", "C")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[("A", 1.0), ("C", 3.0)],
            &["A", "B", "C"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0), ("C", 1.0, 1.0)],
            &[],
        );
        let q = assemble(&inputs, seq(3));
        // Scores are non-increasing.
        for w in q.entries.windows(2) {
            assert!(w[0].score >= w[1].score - 1e-12);
        }
        let subjects: Vec<&str> = q
            .entries
            .iter()
            .filter_map(|e| match &e.payload {
                EntryPayload::AnchorReview { subject, .. } => Some(subject.id.as_str()),
                EntryPayload::Comparison { .. } => None,
            })
            .collect();
        assert_eq!(subjects, vec!["A", "C"], "id-lexicographic tiebreak");
    }

    // ---- VT-1: reprobe / anchor-review (AnchorReview / assemble / resolving) -----

    #[test]
    fn anchor_review_min_over_resolving_uphold_below_removal() {
        // A(1) > B > C(3): the stale A=1 anchor sterilises the A>B>C closure.
        // For suspect A: revise (AnchorRemoved) reactivates two pairs (+2);
        // uphold (RowsRetired of the complete cited closure) retires the ONLY
        // rows A and C have, so their anchors drop and the anchored (A,C) pair
        // reopens — a real −1 (negative deltas are honest, D10). The guaranteed
        // yield is the MIN over resolving answers = −1 (uphold < removal),
        // disclosed per answer with the conditional-yield note.
        let rows = vec![win("j0", "A", "B"), win("j1", "B", "C")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[("A", 1.0), ("C", 3.0)],
            &["A", "B", "C"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0), ("C", 1.0, 1.0)],
            &[],
        );
        let q = assemble(&inputs, seq(3));
        let subject_a = q
            .entries
            .iter()
            .find(|e| matches!(&e.payload, EntryPayload::AnchorReview { subject, .. } if subject.id == "A"))
            .expect("an anchor-review candidate for suspect A");
        assert_eq!(subject_a.kind, CandidateKind::AnchorReview);
        assert_eq!(subject_a.yield_basis, YieldBasis::CanonicalResolvingActions);
        assert_eq!(
            subject_a.guaranteed_yield, -1,
            "min over resolving = uphold"
        );
        if let EntryPayload::AnchorReview { ask, .. } = &subject_a.payload {
            assert_eq!(ask.yield_by_answer.get("revise-anchor"), Some(&2));
            assert_eq!(ask.yield_by_answer.get("uphold-anchor"), Some(&-1));
            assert!(ask.yield_note.is_some(), "conditional-yield disclosure");
        } else {
            panic!("expected anchor-review payload");
        }
    }

    #[test]
    fn anchor_review_not_k_gated_and_keeps_determined_pool_as_candidates() {
        // Top-K frontier [P, Q] is fully determined (both anchored, consistent).
        // A separate stale-anchor conflict on R(1) > S(3) — entities OUTSIDE the
        // top-K — still raises anchor-review candidates (not K-gated), and the
        // queue stays Candidates despite the determined pool (D15 precedence).
        let rows = vec![
            win("j0", "P", "Pl"),
            win("j1", "Q", "Ql"),
            win("j2", "R", "S"),
        ];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[("P", 5.0), ("Q", 3.0), ("R", 1.0), ("S", 3.0)],
            &["P", "Q"],
            &[("P", 1.0, 1.0), ("Q", 1.0, 1.0)],
            &[],
        );
        let q = assemble(&inputs, seq(2));
        assert_eq!(q.state, QueueState::Candidates, "suspect keeps Candidates");
        assert!(
            q.entries.iter().any(
                |e| matches!(&e.payload, EntryPayload::AnchorReview { subject, .. }
                    if subject.id == "R" || subject.id == "S")
            ),
            "suspect outside top-K still admits (not K-gated)"
        );
    }

    // ---- VT-2: median-probe (median_probe) --------------------------------------

    #[test]
    fn median_probe_surfaces_for_unconstrained_item() {
        // U has zero constraining rows and no anchor; W is constrained and
        // carries a projection. U yields ONE median-probe candidate against the
        // projected-median comparable (W), reason `median-probe` — not the full
        // fan of pairs.
        let rows = vec![win("j0", "W", "Z")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let inputs = mk(
            refs.clone(),
            &[("Z", 0.0)],
            &["U", "W"],
            &[("U", 1.0, 1.0), ("W", 1.0, 1.0)],
            &[
                ("U", 2.0, ValueProvenance::Projected),
                ("W", 3.0, ValueProvenance::Projected),
            ],
        );
        let q = assemble(&inputs, seq(2));
        let probe = q
            .entries
            .iter()
            .find(|e| e.reasons.iter().any(|r| r.code == "median-probe"))
            .expect("a median-probe candidate");
        assert_eq!(probe.kind, CandidateKind::Comparison);
        assert!(probe.guaranteed_yield > 0);
        if let EntryPayload::Comparison { a, b, .. } = &probe.payload {
            let ids = [a.id.as_str(), b.id.as_str()];
            assert!(ids.contains(&"U") && ids.contains(&"W"));
        } else {
            panic!("expected comparison payload");
        }
    }

    // ---- SL-218 PHASE-01 D7: demotion knob (design INV-2 / VT-B unit level) -----

    #[test]
    fn demote_reopens_agent_only_determined_pair() {
        // One agent row A>B determines the pair in the full system. Knob-on,
        // determinacy verdicts come from the human-rows-only system: the pair
        // reads indeterminate, re-enters the queue as a comparison candidate
        // (VT-F seed), and the queue state follows.
        let rows = vec![win_agent("g0", "A", "B")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let mut inputs = mk(
            refs.clone(),
            &[],
            &["A", "B"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0)],
            &[],
        );

        let off = assemble(&inputs, seq(2));
        assert_eq!(
            off.state,
            QueueState::Stable { depth: 2 },
            "knob-off: the agent row retires the pair"
        );
        assert!(off.entries.is_empty());

        inputs.demote_agent_evidence = true;
        let on = assemble(&inputs, seq(2));
        assert_eq!(
            on.state,
            QueueState::Candidates,
            "knob-on: agent evidence proposes, never retires (INV-2)"
        );
        assert_eq!(on.entries.len(), 1);
        assert_eq!(on.entries[0].kind, CandidateKind::Comparison);
        assert!(
            on.entries[0].guaranteed_yield >= 1,
            "a fresh answer closes the pair in the human system"
        );
    }

    #[test]
    fn anchored_pair_stays_determined_both_knob_states() {
        // Authored anchors are human authority (design D6: point constraints,
        // no special case). A human row names both entities so classes exist
        // in both systems; the anchored order reads determined either way.
        let rows = vec![jrow(
            "h0",
            "A",
            "B",
            Response::Incomparable,
            RaterKind::Human,
        )];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let mut inputs = mk(
            refs.clone(),
            &[("A", 2.0), ("B", 1.0)],
            &["A", "B"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0)],
            &[],
        );

        let off = assemble(&inputs, seq(2));
        assert_eq!(off.state, QueueState::Stable { depth: 2 });
        assert!(off.entries.is_empty());

        inputs.demote_agent_evidence = true;
        let on = assemble(&inputs, seq(2));
        assert_eq!(
            on.state,
            QueueState::Stable { depth: 2 },
            "knob-on: anchors still determine — human authority"
        );
        assert!(on.entries.is_empty());
    }

    #[test]
    fn human_determined_pair_survives_knob_on() {
        // Mixed evidence on the same pair: the human row alone determines it,
        // so demotion changes nothing.
        let rows = vec![win("h0", "A", "B"), win_agent("g0", "A", "B")];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let mut inputs = mk(
            refs.clone(),
            &[],
            &["A", "B"],
            &[("A", 1.0, 1.0), ("B", 1.0, 1.0)],
            &[],
        );
        inputs.demote_agent_evidence = true;
        let on = assemble(&inputs, seq(2));
        assert_eq!(on.state, QueueState::Stable { depth: 2 });
        assert!(on.entries.is_empty());
    }
}
