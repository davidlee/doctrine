// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::query` — SL-217 design §1: the [`ConstraintSet`] query API
//! (RFC-019 Phase C; the predicates SL-213 D12 anticipated).
//!
//! Pure leaf (ADR-001): imports `compile` + `wire` only. No clock, disk, rng,
//! or git. No priority imports — pair weights and pools arrive as inputs.
//!
//! Three predicates over the compiled constraint system:
//!
//! - [`determined`] — is a pair's `value_dim` order fixed under EVERY feasible
//!   assignment? Closed-form over the pair joint region (design D8/D9): the
//!   per-class C6 interval boxes intersected with the coupling term from
//!   condensed-DAG reachability. Returns a [`SignRange`], never pretending an
//!   open-set infimum is attained.
//! - [`hypothetical_outcome`] / [`hypothetical_yield`] — apply a candidate
//!   answer (or anchor-review outcome), recompile via [`compile`] (pure,
//!   evidence-sized — no second propagation engine), and report the signed
//!   determinacy delta over the relevant pairs (design D10: negative deltas
//!   are real).
//! - [`indeterminate_pairs`] — enumerate the open pairs within a candidate
//!   pool (≤ K(K−1)/2 for planning depth K, never corpus-quadratic).
//!
//! The D8 marginal-exactness lemma (proved in design §1) is what licenses the
//! closed form; it is vocabulary-scoped to strict order edges + point anchors.
//! Ratio rows or band constraints void it — the phase admitting them must
//! revisit [`determined`]. The backing property suite (test-only backtracking
//! extension oracle) lives in this file's tests.

use std::collections::{BTreeMap, BTreeSet};

use super::compile::{
    AnchorMap, Bound, ClassId, ConstraintSet, QuarantinePolicy, RowUid, ValueBounds, compile,
};
use super::wire::{DOMAIN_VALUE, FRAME_EQUAL_EFFORT, Judgement, RaterKind, Response, RowForm};

/// Forward transitive closure over the retained class DAG, built once per
/// refresh and shared across all pair checks and hypotheticals (design §1).
/// Post-C3 the retained graph is acyclic, so plain DFS per source suffices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reachability {
    fwd: BTreeMap<ClassId, BTreeSet<ClassId>>,
}

impl Reachability {
    /// Build the closure from a compiled set's retained edges.
    pub(crate) fn build(cs: &ConstraintSet) -> Self {
        let mut adj: BTreeMap<ClassId, BTreeSet<ClassId>> = BTreeMap::new();
        for (winner, loser) in cs.edges.keys() {
            adj.entry(winner.clone()).or_default().insert(loser.clone());
        }
        let mut fwd: BTreeMap<ClassId, BTreeSet<ClassId>> = BTreeMap::new();
        for src in adj.keys() {
            let mut seen: BTreeSet<ClassId> = BTreeSet::new();
            let mut stack: Vec<&ClassId> = vec![src];
            while let Some(node) = stack.pop() {
                if let Some(next) = adj.get(node) {
                    for m in next {
                        if seen.insert(m.clone()) {
                            stack.push(m);
                        }
                    }
                }
            }
            fwd.insert(src.clone(), seen);
        }
        Reachability { fwd }
    }

    /// Does `from`'s class strictly dominate `to`'s (`from ⇝ to`)?
    pub(crate) fn reaches(&self, from: &ClassId, to: &ClassId) -> bool {
        self.fwd.get(from).is_some_and(|set| set.contains(to))
    }
}

/// One side of a candidate pair, as the queue assembler sees it: the class,
/// its effective weight in the pair objective (design D6: `m_i · c_other`,
/// both factors ≥ 0 with cost > 0), its C6 interval, and its anchor if any.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PairSide {
    pub class: ClassId,
    pub eff_weight: f64,
    pub bounds: ValueBounds,
    pub anchor: Option<f64>,
}

/// The sign range of the pair objective `f = w_A·v_A − w_B·v_B` over the pair
/// joint region (design D9). `Mixed` means the `value_dim` order is genuinely
/// open; anything else is determined. Open-set infima are never reported as
/// attained — the range rule works on the closure extrema and stays honest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignRange {
    NegativeOnly,
    ZeroOnly,
    PositiveOnly,
    Mixed,
}

impl SignRange {
    /// Determined ⇔ not `Mixed` (a structural tie IS a determination — D6).
    pub(crate) fn is_determined(self) -> bool {
        !matches!(self, SignRange::Mixed)
    }
}

/// `w == 0` without a `float_cmp` footgun (`abs` normalises `-0.0`).
fn is_zero(w: f64) -> bool {
    w.abs().total_cmp(&0.0).is_eq()
}

/// The closed closure `[lo, hi]` of one side's feasible interval; `None` is
/// unbounded. An anchored class is a point (attained); open C6 bounds are
/// used by value — the range rule accounts for their non-attainment.
#[derive(Debug, Clone, Copy)]
struct Interval {
    lo: Option<f64>,
    hi: Option<f64>,
}

fn side_interval(side: &PairSide) -> Interval {
    if let Some(v) = side.anchor {
        return Interval {
            lo: Some(v),
            hi: Some(v),
        };
    }
    let take = |b: Bound| match b {
        Bound::Unbounded => None,
        Bound::Open(v) | Bound::Closed(v) => Some(v),
    };
    Interval {
        lo: take(side.bounds.lower),
        hi: take(side.bounds.upper),
    }
}

/// The coupling term of the pair joint region (design D8).
#[derive(Debug, Clone, Copy)]
enum Coupling {
    /// `v_A > v_B` — A's class reaches B's in the retained DAG.
    AGreater,
    /// `v_A < v_B`.
    BGreater,
    /// Order-incomparable: the box alone is the region.
    Free,
}

/// Is the pair's `value_dim` order determined (design D5/D9)? Evaluates the
/// sign range of `f(v_A, v_B) = w_A·v_A − w_B·v_B` over the pair joint region
/// via the pinned extremum algorithm: degenerate rows first, then closure
/// vertex enumeration (box corners, coupling-boundary∩box-edge intersections,
/// directional limits per unbounded side), then the open-interval range rule.
pub(crate) fn determined(reach: &Reachability, a: &PairSide, b: &PairSide) -> SignRange {
    let (wa, wb) = (a.eff_weight, b.eff_weight);
    // Degenerate: both weights zero — structurally tied (D6).
    if is_zero(wa) && is_zero(wb) {
        return SignRange::ZeroOnly;
    }
    let ia = side_interval(a);
    let ib = side_interval(b);
    // Degenerate: same class — substitute v = v_A = v_B, 1-D sign read.
    if a.class == b.class {
        return same_class_range(wa - wb, ia);
    }
    let coupling = if reach.reaches(&a.class, &b.class) {
        Coupling::AGreater
    } else if reach.reaches(&b.class, &a.class) {
        Coupling::BGreater
    } else {
        Coupling::Free
    };
    joint_range(wa, wb, ia, ib, coupling)
}

/// Sign range of `g(v) = c·v` over one class interval (same-class pairs).
fn same_class_range(c: f64, i: Interval) -> SignRange {
    if is_zero(c) {
        return SignRange::ZeroOnly;
    }
    if let (Some(lo), Some(hi)) = (i.lo, i.hi)
        && lo.total_cmp(&hi).is_eq()
    {
        // Point interval (anchored class): the value is attained.
        return sign_of(c * lo);
    }
    let mut vals: Vec<f64> = Vec::new();
    let mut sup_unbounded = false;
    let mut inf_unbounded = false;
    match i.lo {
        Some(lo) => vals.push(c * lo),
        None => {
            if c > 0.0 {
                inf_unbounded = true;
            } else {
                sup_unbounded = true;
            }
        }
    }
    match i.hi {
        Some(hi) => vals.push(c * hi),
        None => {
            if c > 0.0 {
                sup_unbounded = true;
            } else {
                inf_unbounded = true;
            }
        }
    }
    range_verdict(&vals, sup_unbounded, inf_unbounded)
}

fn sign_of(v: f64) -> SignRange {
    if v > 0.0 {
        SignRange::PositiveOnly
    } else if v < 0.0 {
        SignRange::NegativeOnly
    } else {
        SignRange::ZeroOnly
    }
}

/// Closure-vertex enumeration + open-interval range rule (design D9) for a
/// two-class pair. The coupling boundary `v_A = v_B` can define the infimum
/// where no box corner does (the web-review golden) — its clipped segment
/// endpoints are vertices too, and a fully unbounded segment contributes an
/// interior sample (the segment value is constant when `w_A = w_B`, which no
/// recession ray reports).
fn joint_range(wa: f64, wb: f64, ia: Interval, ib: Interval, coupling: Coupling) -> SignRange {
    let f = |x: f64, y: f64| wa * x - wb * y;
    let point_ok = |x: f64, y: f64| match coupling {
        Coupling::AGreater => x >= y,
        Coupling::BGreater => x <= y,
        Coupling::Free => true,
    };
    let mut vals: Vec<f64> = Vec::new();

    // Box corners, filtered by the coupling closure. A side with NO finite
    // endpoint contributes a representative in-region coordinate instead
    // (0.0 — always inside a fully unbounded interval): its growth is the
    // rays' job, but the OTHER side's endpoint values must still be sampled
    // (the w_other = 0 case has no ray to report them).
    let finite_or_rep = |i: Interval| -> Vec<f64> {
        let fin: Vec<f64> = [i.lo, i.hi].into_iter().flatten().collect();
        if fin.is_empty() { vec![0.0] } else { fin }
    };
    for &x in &finite_or_rep(ia) {
        for &y in &finite_or_rep(ib) {
            if point_ok(x, y) {
                vals.push(f(x, y));
            }
        }
    }

    // Coupling boundary ∩ box: the segment v_A = v_B = t, t ∈ [t0, t1].
    if !matches!(coupling, Coupling::Free) {
        let t0 = match (ia.lo, ib.lo) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let t1 = match (ia.hi, ib.hi) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let nonempty = match (t0, t1) {
            (Some(a), Some(b)) => a <= b,
            _ => true,
        };
        if nonempty {
            if let Some(t) = t0 {
                vals.push(f(t, t));
            }
            if let Some(t) = t1 {
                vals.push(f(t, t));
            }
            if t0.is_none() && t1.is_none() {
                // Fully unbounded segment: constant contribution when
                // w_A = w_B; growth along it rides the (±1, ±1) rays.
                vals.push(f(0.0, 0.0));
            }
        }
    }

    // Directional limits: recession rays of the closure region, filtered by
    // the coupling cone (d_A ≥ d_B for AGreater, ≤ for BGreater).
    let ray_ok = |dx: f64, dy: f64| match coupling {
        Coupling::AGreater => dx >= dy,
        Coupling::BGreater => dx <= dy,
        Coupling::Free => true,
    };
    let mut rays: Vec<(f64, f64)> = Vec::new();
    if ia.hi.is_none() {
        rays.push((1.0, 0.0));
    }
    if ia.lo.is_none() {
        rays.push((-1.0, 0.0));
    }
    if ib.hi.is_none() {
        rays.push((0.0, 1.0));
    }
    if ib.lo.is_none() {
        rays.push((0.0, -1.0));
    }
    if ia.hi.is_none() && ib.hi.is_none() {
        rays.push((1.0, 1.0));
    }
    if ia.lo.is_none() && ib.lo.is_none() {
        rays.push((-1.0, -1.0));
    }
    let mut sup_unbounded = false;
    let mut inf_unbounded = false;
    for (dx, dy) in rays {
        if !ray_ok(dx, dy) {
            continue;
        }
        let growth = wa * dx - wb * dy;
        if growth > 0.0 {
            sup_unbounded = true;
        } else if growth < 0.0 {
            inf_unbounded = true;
        }
    }

    range_verdict(&vals, sup_unbounded, inf_unbounded)
}

/// The open-interval range rule (design D9). Over the closure extrema
/// `(inf, sup)`: a constant `f` reads its sign directly (the point/segment IS
/// attained); otherwise the feasible set is the relative interior and the
/// range is the OPEN interval, so `Mixed ⇔ inf < 0 < sup`, `PositiveOnly ⇔
/// inf ≥ 0`, `NegativeOnly ⇔ sup ≤ 0` — no attainment bookkeeping.
fn range_verdict(vertex_values: &[f64], sup_unbounded: bool, inf_unbounded: bool) -> SignRange {
    let mut inf = f64::INFINITY;
    let mut sup = f64::NEG_INFINITY;
    for &v in vertex_values {
        inf = inf.min(v);
        sup = sup.max(v);
    }
    if inf_unbounded {
        inf = f64::NEG_INFINITY;
    }
    if sup_unbounded {
        sup = f64::INFINITY;
    }
    // An empty enumeration means an empty pair region, which a satisfiable
    // ConstraintSet cannot produce for evidenced classes (C5 + D8).
    debug_assert!(
        inf <= sup,
        "empty pair joint region from a satisfiable ConstraintSet"
    );
    if inf.total_cmp(&sup).is_eq() {
        return sign_of(inf);
    }
    if inf < 0.0 && sup > 0.0 {
        SignRange::Mixed
    } else if inf >= 0.0 {
        SignRange::PositiveOnly
    } else {
        SignRange::NegativeOnly
    }
}

/// Sentinel session identity for synthetic hypothetical rows (design §2, the
/// R3 trap): the hypothetical path augments the post-resolve ACTIVE set, so
/// resolution never sees these rows — the sentinel additionally guarantees a
/// synthetic uid can never collide with (or supersede) a real row's.
pub(crate) const SYNTHETIC_SESSION_UID: &str = "synthetic-hypothetical";
/// Sentinel date for synthetic rows (the pure layer has no clock).
pub(crate) const SYNTHETIC_ROW_DATE: &str = "0000-01-01";

/// Build the synthetic order-bearing row for a hypothetical answer (D11).
pub(crate) fn synthetic_answer_row(a: &str, b: &str, response: Response) -> Judgement {
    Judgement {
        uid: format!("{SYNTHETIC_SESSION_UID}:{a}:{b}"),
        seq: 0,
        a: a.to_string(),
        b: Some(b.to_string()),
        response: Some(response),
        domain: DOMAIN_VALUE.to_string(),
        frame: FRAME_EQUAL_EFFORT.to_string(),
        form: RowForm::Order,
        magnitude: None,
        est_lower: None,
        est_upper: None,
        supersedes: None,
        lens: None,
        rater: RaterKind::Agent,
        by: None,
        note: None,
        date: Some(SYNTHETIC_ROW_DATE.to_string()),
        observed_at: None,
        basis: None,
        admission: None,
    }
}

/// A candidate answer applied to the evidence, for recompile-and-count
/// (design D10/D12). No `Clone`/`Eq`: contains a [`Judgement`].
#[derive(Debug)]
pub(crate) enum Hypothetical<'a> {
    /// A synthetic order-bearing judgement row (comparison candidates).
    /// Boxed: a `Judgement` dwarfs the other variants.
    Answer(Box<Judgement>),
    /// Anchor-review: a revise modelled optimistically as removal (D12).
    AnchorRemoved(&'a str),
    /// Anchor-review: uphold — the cited conflicting rows drop (D12).
    RowsRetired(&'a BTreeSet<RowUid>),
}

/// The determinacy delta of one hypothetical over the relevant pairs. Pairs
/// are keyed by their BASELINE class ids. D13 needs the newly-determined
/// PAIRS (rank-decay impact), not just a count — hence the outcome struct
/// with [`hypothetical_yield`] as the thin signed-count wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct HypotheticalOutcome {
    pub newly_determined: Vec<(ClassId, ClassId)>,
    pub no_longer_determined: Vec<(ClassId, ClassId)>,
}

impl HypotheticalOutcome {
    /// The signed yield (D10): newly determined minus no longer determined.
    pub(crate) fn yield_delta(&self) -> i64 {
        let gained = i64::try_from(self.newly_determined.len()).unwrap_or(i64::MAX);
        let lost = i64::try_from(self.no_longer_determined.len()).unwrap_or(i64::MAX);
        gained - lost
    }
}

/// Apply one hypothetical, recompile via [`compile`], rebuild reachability,
/// and diff determinacy over `relevant`. The baseline reachability is passed
/// in (the caller holds it, memoised per refresh — §2's cost bound counts one
/// recompile per hypothetical, none for the baseline). A side whose
/// representative entity loses all evidence under the hypothetical counts as
/// not determined.
pub(crate) fn hypothetical_outcome(
    baseline_reach: &Reachability,
    active: &[&Judgement],
    anchors: &AnchorMap,
    hypo: &Hypothetical<'_>,
    relevant: &[(PairSide, PairSide)],
) -> HypotheticalOutcome {
    let mut anchors2: AnchorMap = anchors.clone();
    let active2: Vec<&Judgement> = match hypo {
        Hypothetical::Answer(row) => {
            let mut v = active.to_vec();
            v.push(row.as_ref());
            v
        }
        Hypothetical::AnchorRemoved(entity) => {
            anchors2.remove(*entity);
            active.to_vec()
        }
        Hypothetical::RowsRetired(uids) => active
            .iter()
            .copied()
            .filter(|j| !uids.contains(&j.uid))
            .collect(),
    };
    let cs2 = compile(&active2, &anchors2, QuarantinePolicy::Symmetric);
    let reach2 = Reachability::build(&cs2);

    // Remap a baseline side into the hypothetical set: a ClassId doubles as
    // its smallest member's entity id, so it survives class reshapes.
    let remap = |side: &PairSide| -> Option<PairSide> {
        let class = cs2.classes.get(&side.class)?.clone();
        let bounds = *cs2.bounds.get(&class)?;
        let anchor = cs2.anchors.get(&class).copied();
        Some(PairSide {
            class,
            eff_weight: side.eff_weight,
            bounds,
            anchor,
        })
    };

    let mut out = HypotheticalOutcome::default();
    for (a, b) in relevant {
        let before = determined(baseline_reach, a, b).is_determined();
        let after = match (remap(a), remap(b)) {
            (Some(a2), Some(b2)) => determined(&reach2, &a2, &b2).is_determined(),
            _ => false,
        };
        match (before, after) {
            (false, true) => out
                .newly_determined
                .push((a.class.clone(), b.class.clone())),
            (true, false) => out
                .no_longer_determined
                .push((a.class.clone(), b.class.clone())),
            _ => {}
        }
    }
    out
}

/// The signed determinacy delta of one hypothetical (design D10). Negative
/// is real: a contradicting hypothetical quarantines structure on recompile.
///
/// A thin wrapper over [`hypothetical_outcome`]: `elicit` consumes the outcome
/// directly (D13 impact needs the pair sets, not a bare count), so this stays
/// production-unused and carries a self-clearing per-item gate
/// (mem.pattern.lint.dead-code-staged-ahead-cfg-test) until a count-only caller
/// lands.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "signed-count convenience wrapper; elicit uses hypothetical_outcome directly (SL-217 PHASE-02)"
    )
)]
pub(crate) fn hypothetical_yield(
    baseline_reach: &Reachability,
    active: &[&Judgement],
    anchors: &AnchorMap,
    hypo: &Hypothetical<'_>,
    relevant: &[(PairSide, PairSide)],
) -> i64 {
    hypothetical_outcome(baseline_reach, active, anchors, hypo, relevant).yield_delta()
}

/// Enumerate the indeterminate pairs within a candidate pool (design §1):
/// all pairs over the pool (≤ K(K−1)/2), filtered by [`determined`].
///
/// `elicit` carries per-PAIR effective weights (D6: `m_self · c_other`), so it
/// pairs its pool by hand rather than through this fixed-weight helper; the
/// predicate stays for a same-weight caller and keeps its own battery green.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "fixed-weight pool helper; elicit pairs with per-pair weights (SL-217 PHASE-02)"
    )
)]
pub(crate) fn indeterminate_pairs(
    reach: &Reachability,
    pool: &[PairSide],
) -> Vec<(ClassId, ClassId)> {
    let mut out: Vec<(ClassId, ClassId)> = Vec::new();
    for (i, a) in pool.iter().enumerate() {
        for b in pool.iter().skip(i + 1) {
            if !determined(reach, a, b).is_determined() {
                out.push((a.class.clone(), b.class.clone()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comparison::wire::{DOMAIN_VALUE, FRAME_EQUAL_EFFORT};

    // ---- fixtures (project.rs idiom) ---------------------------------------

    fn row(uid: &str, a: &str, b: &str, response: Response) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: a.to_string(),
            b: Some(b.to_string()),
            response: Some(response),
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            est_lower: None,
            est_upper: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-12".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    fn win(uid: &str, winner: &str, loser: &str) -> Judgement {
        row(uid, winner, loser, Response::PreferA)
    }

    fn eq_row(uid: &str, a: &str, b: &str) -> Judgement {
        row(uid, a, b, Response::Equal)
    }

    fn amap(anchors: &[(&str, f64)]) -> AnchorMap {
        anchors.iter().map(|&(e, v)| (e.to_string(), v)).collect()
    }

    fn compiled(rows: &[Judgement], anchors: &[(&str, f64)]) -> (ConstraintSet, Reachability) {
        let refs: Vec<&Judgement> = rows.iter().collect();
        let cs = compile(&refs, &amap(anchors), QuarantinePolicy::Symmetric);
        let reach = Reachability::build(&cs);
        (cs, reach)
    }

    /// A pair side for `entity`'s class, straight off the compiled set.
    fn side(cs: &ConstraintSet, entity: &str, w: f64) -> PairSide {
        let class = cs.classes[entity].clone();
        PairSide {
            class: class.clone(),
            eff_weight: w,
            bounds: cs.bounds[&class],
            anchor: cs.anchors.get(&class).copied(),
        }
    }

    // ---- VT-1: determinacy battery (design §5.1) ----------------------------

    #[test]
    fn chain_ordered_equal_weights_determined() {
        let rows = [win("j0", "A", "B")];
        let (cs, reach) = compiled(&rows, &[]);
        let verdict = determined(&reach, &side(&cs, "A", 1.0), &side(&cs, "B", 1.0));
        assert_eq!(verdict, SignRange::PositiveOnly);
    }

    #[test]
    fn chain_ordered_cheaper_winner_determined_positive_intervals() {
        // A > B > C, C anchored 0 ⇒ boxes (0, ∞); cheaper winner ⇒ w_A > w_B.
        let rows = [win("j0", "A", "B"), win("j1", "B", "C")];
        let (cs, reach) = compiled(&rows, &[("C", 0.0)]);
        let verdict = determined(&reach, &side(&cs, "A", 2.0), &side(&cs, "B", 1.0));
        assert_eq!(verdict, SignRange::PositiveOnly);
    }

    #[test]
    fn chain_ordered_costlier_winner_indeterminate() {
        // Same graph, costlier winner (w_A < w_B): f = v_A − 2·v_B flips sign.
        let rows = [win("j0", "A", "B"), win("j1", "B", "C")];
        let (cs, reach) = compiled(&rows, &[("C", 0.0)]);
        let verdict = determined(&reach, &side(&cs, "A", 1.0), &side(&cs, "B", 2.0));
        assert_eq!(verdict, SignRange::Mixed);
    }

    #[test]
    fn costlier_winner_determined_under_anchor_squeeze() {
        // Both anchored: point evaluation — the squeeze closes it.
        let rows = [win("j0", "A", "B")];
        let (cs, reach) = compiled(&rows, &[("A", 10.0), ("B", 1.0)]);
        let verdict = determined(&reach, &side(&cs, "A", 1.0), &side(&cs, "B", 2.0));
        assert_eq!(verdict, SignRange::PositiveOnly);
        // And a squeeze the other way: f = 1·10 − 20·1 < 0.
        let verdict = determined(&reach, &side(&cs, "A", 1.0), &side(&cs, "B", 20.0));
        assert_eq!(verdict, SignRange::NegativeOnly);
    }

    #[test]
    fn box_overlap_but_chain_coupled_determined() {
        // T(10) > A > B > Z(0) plus A > B: boxes of A and B both (0, 10) —
        // overlapping — yet the coupling term determines the pair. The box
        // is not the oracle.
        let rows = [
            win("j0", "T", "A"),
            win("j1", "A", "B"),
            win("j2", "B", "Z"),
        ];
        let (cs, reach) = compiled(&rows, &[("T", 10.0), ("Z", 0.0)]);
        let a = side(&cs, "A", 1.0);
        let b = side(&cs, "B", 1.0);
        assert_eq!(a.bounds, b.bounds, "boxes genuinely overlap");
        assert_eq!(determined(&reach, &a, &b), SignRange::PositiveOnly);
    }

    #[test]
    fn same_class_differing_weights_spanning_zero_indeterminate() {
        // Class {A, B} with interval (−5, 5): g = (w_A − w_B)·v flips with
        // the value's sign (negative-domain golden, D9 — no positivity).
        let rows = [
            eq_row("j0", "A", "B"),
            win("j1", "T", "A"),
            win("j2", "A", "L"),
        ];
        let (cs, reach) = compiled(&rows, &[("T", 5.0), ("L", -5.0)]);
        let verdict = determined(&reach, &side(&cs, "A", 2.0), &side(&cs, "B", 1.0));
        assert_eq!(verdict, SignRange::Mixed);
    }

    #[test]
    fn same_class_equal_weights_zero_only() {
        let rows = [eq_row("j0", "A", "B")];
        let (cs, reach) = compiled(&rows, &[]);
        let verdict = determined(&reach, &side(&cs, "A", 1.5), &side(&cs, "B", 1.5));
        assert_eq!(verdict, SignRange::ZeroOnly);
    }

    #[test]
    fn same_class_anchored_point_reads_sign() {
        let rows = [eq_row("j0", "A", "B")];
        let (cs, reach) = compiled(&rows, &[("A", 3.0)]);
        // g = (2 − 1)·3 > 0.
        let verdict = determined(&reach, &side(&cs, "A", 2.0), &side(&cs, "B", 1.0));
        assert_eq!(verdict, SignRange::PositiveOnly);
    }

    #[test]
    fn both_weights_zero_zero_only() {
        let rows = [win("j0", "A", "B")];
        let (cs, reach) = compiled(&rows, &[]);
        let verdict = determined(&reach, &side(&cs, "A", 0.0), &side(&cs, "B", 0.0));
        assert_eq!(verdict, SignRange::ZeroOnly);
    }

    #[test]
    fn unbounded_side_differing_weights_mixed() {
        // Order-incomparable pair, all sides unbounded: limits dominate.
        let rows = [win("j0", "A", "B"), win("j1", "C", "D")];
        let (cs, reach) = compiled(&rows, &[]);
        let verdict = determined(&reach, &side(&cs, "A", 1.0), &side(&cs, "C", 2.0));
        assert_eq!(verdict, SignRange::Mixed);
    }

    #[test]
    fn one_zero_weight_over_negative_spanning_interval_mixed() {
        // RV-269 F-4 shape: w_B = 0 ⇒ f = w_A·v_A over A's interval (−5, 5).
        let rows = [
            win("j0", "T", "A"),
            win("j1", "A", "L"),
            win("j2", "C", "D"),
        ];
        let (cs, reach) = compiled(&rows, &[("T", 5.0), ("L", -5.0)]);
        let verdict = determined(&reach, &side(&cs, "A", 1.0), &side(&cs, "C", 0.0));
        assert_eq!(verdict, SignRange::Mixed);
        // Positive interval instead: determined despite the zero weight.
        let (cs2, reach2) = compiled(&rows, &[("T", 5.0), ("L", 1.0)]);
        let verdict = determined(&reach2, &side(&cs2, "A", 1.0), &side(&cs2, "C", 0.0));
        assert_eq!(verdict, SignRange::PositiveOnly);
    }

    #[test]
    fn coupling_boundary_infimum_golden() {
        // Web-review golden: the infimum of f lives ON v_A = v_B, attained at
        // no box corner. U(10) > A > B > L(0), A > L2(4), M(6) > B:
        // boxes A (4, 10), B (0, 6); the boundary clip [4, 6] is interior to
        // both boxes' edges. w_A = 0.7, w_B = 1.0: every admissible corner is
        // positive (2.8, 7, 1) but the boundary dips negative (−1.2, −1.8) ⇒
        // Mixed. Corner-only enumeration would wrongly report PositiveOnly.
        let rows = [
            win("j0", "U", "A"),
            win("j1", "A", "L2"),
            win("j2", "A", "B"),
            win("j3", "M", "B"),
            win("j4", "B", "L"),
        ];
        let (cs, reach) = compiled(&rows, &[("U", 10.0), ("L2", 4.0), ("M", 6.0), ("L", 0.0)]);
        let a = side(&cs, "A", 0.7);
        let b = side(&cs, "B", 1.0);
        assert_eq!(a.bounds.lower, Bound::Open(4.0));
        assert_eq!(a.bounds.upper, Bound::Open(10.0));
        assert_eq!(b.bounds.lower, Bound::Open(0.0));
        assert_eq!(b.bounds.upper, Bound::Open(6.0));
        assert_eq!(determined(&reach, &a, &b), SignRange::Mixed);
    }

    #[test]
    fn indeterminate_pairs_filters_pool() {
        let rows = [
            win("j0", "A", "B"),
            row("j1", "C", "A", Response::Incomparable),
        ];
        let (cs, reach) = compiled(&rows, &[]);
        let pool = [
            side(&cs, "A", 1.0),
            side(&cs, "B", 1.0),
            side(&cs, "C", 1.0),
        ];
        let open = indeterminate_pairs(&reach, &pool);
        // (A,B) determined by the chain; (A,C) and (B,C) open.
        assert_eq!(open.len(), 2);
        assert!(open.contains(&(cs.classes["A"].clone(), cs.classes["C"].clone())));
        assert!(open.contains(&(cs.classes["B"].clone(), cs.classes["C"].clone())));
    }

    // ---- VT-2: D8 property suite — backtracking extension oracle ------------

    /// Naive backtracking extension oracle (test-only; no LP/SMT): does an
    /// assignment of ALL classes exist that meets every anchor exactly,
    /// every retained strict edge, and the two pinned values? Dumb by
    /// design — a finite candidate grid (fine enough for every order type)
    /// with DFS + pruning. Shares no code with the production path.
    fn oracle_extends(cs: &ConstraintSet, pin_a: (&ClassId, f64), pin_b: (&ClassId, f64)) -> bool {
        let mut pinned: BTreeMap<ClassId, f64> = cs.anchors.clone();
        for (class, v) in [pin_a, pin_b] {
            if let Some(&existing) = pinned.get(class) {
                if existing.total_cmp(&v).is_ne() {
                    return false;
                }
            }
            pinned.insert(class.clone(), v);
        }
        let all: BTreeSet<ClassId> = cs.classes.values().cloned().collect();
        let free: Vec<ClassId> = all
            .iter()
            .filter(|c| !pinned.contains_key(*c))
            .cloned()
            .collect();

        // Candidate grid: n interior points per gap between consecutive pins
        // (n = class count, enough for any chain inside one gap), plus n
        // below the min and n above the max.
        let mut pins: Vec<f64> = pinned.values().copied().collect();
        pins.sort_by(f64::total_cmp);
        pins.dedup_by(|a, b| a.total_cmp(b).is_eq());
        let n = all.len();
        let mut grid: Vec<f64> = Vec::new();
        if let (Some(&first), Some(&last)) = (pins.first(), pins.last()) {
            for i in 1..=n {
                grid.push(first - i as f64);
                grid.push(last + i as f64);
            }
            for w in pins.windows(2) {
                let (lo, hi) = (w[0], w[1]);
                for i in 1..=n {
                    grid.push(lo + (hi - lo) * (i as f64) / ((n + 1) as f64));
                }
            }
        }
        grid.extend(pins.iter().copied());

        fn consistent(
            edges: &super::super::compile::EdgeMap,
            vals: &BTreeMap<ClassId, f64>,
        ) -> bool {
            edges.keys().all(|(w, l)| match (vals.get(w), vals.get(l)) {
                (Some(wv), Some(lv)) => wv > lv,
                _ => true,
            })
        }

        fn bt(
            free: &[ClassId],
            vals: &mut BTreeMap<ClassId, f64>,
            grid: &[f64],
            edges: &super::super::compile::EdgeMap,
        ) -> bool {
            let Some((head, rest)) = free.split_first() else {
                return consistent(edges, vals);
            };
            for &v in grid {
                vals.insert(head.clone(), v);
                if consistent(edges, vals) && bt(rest, vals, grid, edges) {
                    return true;
                }
            }
            vals.remove(head);
            false
        }

        let mut vals = pinned.clone();
        // Pinned values themselves must not already violate an edge.
        if !consistent(&cs.edges, &vals) {
            return false;
        }
        bt(&free, &mut vals, &grid, &cs.edges)
    }

    /// Sample points from one side's claimed interval: strictly-interior
    /// candidates plus (for open bounds) the boundary value itself, tagged
    /// with whether the claim says it is feasible.
    fn interval_samples(b: &ValueBounds, anchor: Option<f64>) -> Vec<(f64, bool)> {
        if let Some(v) = anchor {
            return vec![(v, true), (v + 1.0, false), (v - 1.0, false)];
        }
        let mut out: Vec<(f64, bool)> = Vec::new();
        match (b.lower, b.upper) {
            (Bound::Open(l), Bound::Open(u)) => {
                out.push((l + (u - l) * 0.25, true));
                out.push(((l + u) / 2.0, true));
                out.push((l + (u - l) * 0.75, true));
                out.push((l, false)); // open endpoint: exactly l is infeasible
                out.push((u, false));
                out.push((l - 1.0, false));
                out.push((u + 1.0, false));
            }
            (Bound::Open(l), Bound::Unbounded) => {
                out.push((l + 0.5, true));
                out.push((l + 10.0, true));
                out.push((l, false));
                out.push((l - 1.0, false));
            }
            (Bound::Unbounded, Bound::Open(u)) => {
                out.push((u - 0.5, true));
                out.push((u - 10.0, true));
                out.push((u, false));
                out.push((u + 1.0, false));
            }
            (Bound::Unbounded, Bound::Unbounded) => {
                out.push((-7.0, true));
                out.push((0.0, true));
                out.push((7.0, true));
            }
            // Closed bounds only arise via anchors — handled above.
            _ => {}
        }
        out
    }

    /// The D8 lemma, checked by oracle: over a grid of generated anchored
    /// DAGs (chains, diamonds, shared anchored/unanchored intermediates,
    /// equality merges, open + unbounded sides), a pair point is oracle-
    /// extendable IFF it lies in the claimed joint region box ∩ coupling.
    #[test]
    fn extension_oracle_confirms_joint_region_exactness() {
        struct Scenario {
            name: &'static str,
            rows: Vec<Judgement>,
            anchors: Vec<(&'static str, f64)>,
            pair: (&'static str, &'static str),
        }
        let scenarios = vec![
            Scenario {
                name: "bare chain, unbounded sides",
                rows: vec![win("j0", "A", "B")],
                anchors: vec![],
                pair: ("A", "B"),
            },
            Scenario {
                name: "chain with bottom anchor (open + unbounded)",
                rows: vec![win("j0", "A", "B"), win("j1", "B", "C")],
                anchors: vec![("C", 0.0)],
                pair: ("A", "B"),
            },
            Scenario {
                name: "shared ANCHORED intermediate A ⇝ Z ⇝ B (reviewer case)",
                rows: vec![win("j0", "A", "Z"), win("j1", "Z", "B")],
                anchors: vec![("Z", 5.0)],
                pair: ("A", "B"),
            },
            Scenario {
                name: "shared UNANCHORED intermediate, outer anchors",
                rows: vec![
                    win("j0", "T", "A"),
                    win("j1", "A", "Z"),
                    win("j2", "Z", "B"),
                    win("j3", "B", "L"),
                ],
                anchors: vec![("T", 10.0), ("L", 0.0)],
                pair: ("A", "B"),
            },
            Scenario {
                name: "diamond: multiple anchored ancestors and descendants",
                rows: vec![
                    win("j0", "T1", "A"),
                    win("j1", "T2", "A"),
                    win("j2", "B", "L1"),
                    win("j3", "B", "L2"),
                    win("j4", "A", "B"),
                ],
                anchors: vec![("T1", 8.0), ("T2", 9.0), ("L1", 1.0), ("L2", 2.0)],
                pair: ("A", "B"),
            },
            Scenario {
                name: "equality merge inside the chain",
                rows: vec![
                    eq_row("j0", "A", "A2"),
                    win("j1", "A2", "B"),
                    win("j2", "B", "L"),
                ],
                anchors: vec![("L", 0.0)],
                pair: ("A", "B"),
            },
            Scenario {
                name: "order-incomparable pair, one side anchored chain",
                rows: vec![win("j0", "A", "B"), win("j1", "C", "D")],
                anchors: vec![("B", 3.0)],
                pair: ("A", "C"),
            },
            Scenario {
                name: "negative domain (no positivity assumption)",
                rows: vec![win("j0", "A", "B"), win("j1", "B", "C")],
                anchors: vec![("A", -1.0), ("C", -9.0)],
                pair: ("A", "B"),
            },
            Scenario {
                name: "coupling-boundary golden graph",
                rows: vec![
                    win("j0", "U", "A"),
                    win("j1", "A", "L2"),
                    win("j2", "A", "B"),
                    win("j3", "M", "B"),
                    win("j4", "B", "L"),
                ],
                anchors: vec![("U", 10.0), ("L2", 4.0), ("M", 6.0), ("L", 0.0)],
                pair: ("A", "B"),
            },
        ];

        for s in &scenarios {
            let (cs, reach) = compiled(&s.rows, &s.anchors);
            assert!(
                cs.quarantined.is_empty(),
                "{}: scenario must compile clean",
                s.name
            );
            let (ea, eb) = s.pair;
            let ca = cs.classes[ea].clone();
            let cb = cs.classes[eb].clone();
            let ba = cs.bounds[&ca];
            let bb = cs.bounds[&cb];
            let aa = cs.anchors.get(&ca).copied();
            let ab = cs.anchors.get(&cb).copied();
            let a_greater = reach.reaches(&ca, &cb);
            let b_greater = reach.reaches(&cb, &ca);

            for &(x, x_in) in &interval_samples(&ba, aa) {
                for &(y, y_in) in &interval_samples(&bb, ab) {
                    let coupling_ok = if a_greater {
                        x > y
                    } else if b_greater {
                        x < y
                    } else {
                        true
                    };
                    let claimed = x_in && y_in && coupling_ok;
                    let extends = oracle_extends(&cs, (&ca, x), (&cb, y));
                    assert_eq!(
                        extends, claimed,
                        "{}: point ({x}, {y}) — oracle {extends}, claimed {claimed}",
                        s.name
                    );
                }
            }
        }
    }

    /// The oracle itself, hand-verified on a micro-case (guards against a
    /// false-green oracle): A > B with A anchored 2, B anchored 5 would be
    /// infeasible — but compile would quarantine it, so pin at the oracle
    /// level instead: extending (A=1, B=3) against A > B must fail.
    #[test]
    fn extension_oracle_micro_case_rejects_violation() {
        let rows = [win("j0", "A", "B")];
        let (cs, _) = compiled(&rows, &[]);
        let ca = cs.classes["A"].clone();
        let cb = cs.classes["B"].clone();
        assert!(oracle_extends(&cs, (&ca, 3.0), (&cb, 1.0)));
        assert!(!oracle_extends(&cs, (&ca, 1.0), (&cb, 3.0)));
        assert!(!oracle_extends(&cs, (&ca, 2.0), (&cb, 2.0)), "strict edge");
    }

    // ---- VT-3: yield battery (design §5.2) ----------------------------------

    /// Hand-computed guaranteed yield: A > B chain, C incomparable-adjacent.
    /// prefer-a on (A, C) determines 1 pair; prefer-b determines 2 (chain
    /// closure); equal merges and determines 2. Min over order-bearing = 1.
    #[test]
    fn hypothetical_yield_min_over_order_bearing_answers() {
        let rows = [
            win("j0", "A", "B"),
            row("j1", "C", "A", Response::Incomparable),
        ];
        let (cs, reach) = compiled(&rows, &[]);
        let refs: Vec<&Judgement> = rows.iter().collect();
        let anchors = amap(&[]);
        let relevant = [
            (side(&cs, "A", 1.0), side(&cs, "C", 1.0)),
            (side(&cs, "B", 1.0), side(&cs, "C", 1.0)),
        ];
        let yield_of = |response: Response| {
            let hypo = Hypothetical::Answer(Box::new(synthetic_answer_row("A", "C", response)));
            hypothetical_yield(&reach, &refs, &anchors, &hypo, &relevant)
        };
        assert_eq!(yield_of(Response::PreferA), 1); // A > C: (B, C) still open
        assert_eq!(yield_of(Response::PreferB), 2); // C > A > B: both close
        assert_eq!(yield_of(Response::Equal), 2); // C ≈ A: both close
        let order_bearing_min = [
            yield_of(Response::PreferA),
            yield_of(Response::PreferB),
            yield_of(Response::Equal),
        ]
        .into_iter()
        .min();
        assert_eq!(order_bearing_min, Some(1));
    }

    #[test]
    fn incomparable_hypothetical_yields_zero() {
        let rows = [
            win("j0", "A", "B"),
            row("j1", "C", "A", Response::Incomparable),
        ];
        let (cs, reach) = compiled(&rows, &[]);
        let refs: Vec<&Judgement> = rows.iter().collect();
        let relevant = [(side(&cs, "A", 1.0), side(&cs, "C", 1.0))];
        let hypo = Hypothetical::Answer(Box::new(synthetic_answer_row(
            "A",
            "C",
            Response::Incomparable,
        )));
        assert_eq!(
            hypothetical_yield(&reach, &refs, &amap(&[]), &hypo, &relevant),
            0
        );
    }

    /// D10: a hypothetical contradicting a chain quarantines BOTH rows (C3
    /// two-cycle) — the previously determined pair reopens; delta is −1.
    #[test]
    fn contradicting_hypothetical_negative_delta() {
        let rows = [win("j0", "A", "B")];
        let (cs, reach) = compiled(&rows, &[]);
        let refs: Vec<&Judgement> = rows.iter().collect();
        let relevant = [(side(&cs, "A", 1.0), side(&cs, "B", 1.0))];
        let hypo =
            Hypothetical::Answer(Box::new(synthetic_answer_row("B", "A", Response::PreferA)));
        let outcome = hypothetical_outcome(&reach, &refs, &amap(&[]), &hypo, &relevant);
        assert_eq!(outcome.newly_determined.len(), 0);
        assert_eq!(outcome.no_longer_determined.len(), 1);
        assert_eq!(outcome.yield_delta(), -1);
    }

    /// Web review: a hypothetical `equal` between differently-anchored
    /// classes triggers C2 quarantine on recompile, and any REAL equal row
    /// riding the violating path is quarantined with it — accounted as a
    /// negative delta. The anchor sits on the NON-representative member E:
    /// the C2 split strips class-rep A of its hoisted anchor, reopening the
    /// watched (A, C) pair.
    #[test]
    fn equal_between_differently_anchored_quarantines_and_goes_negative() {
        // Class {A, E} anchored 5 (hoisted from E), C anchored 3.
        let rows = [eq_row("j0", "A", "E"), win("j1", "C", "D")];
        let anchors = [("E", 5.0), ("C", 3.0)];
        let (cs, reach) = compiled(&rows, &anchors);
        let refs: Vec<&Judgement> = rows.iter().collect();
        let a = side(&cs, "A", 1.0);
        let c = side(&cs, "C", 1.0);
        assert_eq!(determined(&reach, &a, &c), SignRange::PositiveOnly);
        let relevant = [(a, c)];
        let hypo = Hypothetical::Answer(Box::new(synthetic_answer_row("A", "C", Response::Equal)));
        let outcome = hypothetical_outcome(&reach, &refs, &amap(&anchors), &hypo, &relevant);
        assert_eq!(outcome.yield_delta(), -1, "C2 quarantine reopens (A, C)");
    }

    /// The R3 trap pin: synthetic rows carry the sentinel identity, never a
    /// real row's uid, and augmenting a pair that already has real evidence
    /// leaves every real row's compile status untouched.
    #[test]
    fn synthetic_session_identity_never_touches_real_rows() {
        let real = win("j0", "A", "B");
        let synthetic = synthetic_answer_row("A", "B", Response::PreferA);
        assert!(synthetic.uid.starts_with(SYNTHETIC_SESSION_UID));
        assert_ne!(synthetic.uid, real.uid);
        assert_eq!(synthetic.supersedes, None);

        let rows = [real];
        let (cs, reach) = compiled(&rows, &[]);
        let refs: Vec<&Judgement> = rows.iter().collect();
        let relevant = [(side(&cs, "A", 1.0), side(&cs, "B", 1.0))];
        let hypo = Hypothetical::Answer(Box::new(synthetic));
        let outcome = hypothetical_outcome(&reach, &refs, &amap(&[]), &hypo, &relevant);
        // A duplicate confirmation changes nothing: no quarantine, no delta.
        assert_eq!(outcome.yield_delta(), 0);
    }

    /// AnchorRemoved reactivates the closure a stale anchor sterilised: with
    /// the A(1) > B(5) conflict quarantining j0, pair (A, C) is open; drop
    /// B's anchor and the chain A > B > C re-forms — determined, +1.
    #[test]
    fn anchor_removed_reactivates_closure_rows() {
        let rows = [win("j0", "A", "B"), win("j1", "B", "C")];
        let anchors = [("A", 1.0), ("B", 5.0)];
        let (cs, reach) = compiled(&rows, &anchors);
        assert!(!cs.quarantined.is_empty(), "conflict must quarantine");
        let refs: Vec<&Judgement> = rows.iter().collect();
        let relevant = [(side(&cs, "A", 1.0), side(&cs, "C", 1.0))];
        assert_eq!(
            determined(&reach, &relevant[0].0, &relevant[0].1),
            SignRange::Mixed,
            "baseline pair open while the chain is sterilised"
        );
        let hypo = Hypothetical::AnchorRemoved("B");
        assert_eq!(
            hypothetical_yield(&reach, &refs, &amap(&anchors), &hypo, &relevant),
            1
        );
    }

    /// RowsRetired: retiring a load-bearing row costs its determinations —
    /// the side's evidence disappears and the pair counts as reopened.
    #[test]
    fn rows_retired_counts_lost_determinations() {
        let rows = [win("j0", "A", "B")];
        let (cs, reach) = compiled(&rows, &[]);
        let refs: Vec<&Judgement> = rows.iter().collect();
        let relevant = [(side(&cs, "A", 1.0), side(&cs, "B", 1.0))];
        let retired: BTreeSet<RowUid> = ["j0".to_string()].into_iter().collect();
        let hypo = Hypothetical::RowsRetired(&retired);
        assert_eq!(
            hypothetical_yield(&reach, &refs, &amap(&[]), &hypo, &relevant),
            -1
        );
    }
}
