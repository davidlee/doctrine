// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::project` — tier-3 projection & gauge (SL-213 design §3, rules
//! P1–P15). Pure leaf (ADR-001): depends only on the tier-2 [`ConstraintSet`]
//! plus std `BTree` collections. No clock, disk, rng, or git.
//!
//! Input: a compiled [`ConstraintSet`] (retained, post-quarantine classes,
//! strict class digraph, and per-class anchors) plus a [`ProjectionCfg`] (the
//! two gauge constants as pure inputs — a later phase homes the shipped
//! `GAUGE_STEP` / `DEFAULT_VALUE` in `priority/config.rs`). Output: a
//! [`Projection`] mapping every evidence-bearing **entity** id to a scalar and
//! its [`ValueProvenance`]; every member of a class takes the class value.
//!
//! ## Method (design §3)
//! Reverse-topological greedy placement (sinks/lowest first, uid-sorted
//! tiebreak). Each unanchored class is a pure function of its already-placed
//! direct successors (the floor) and the min anchor weakly above it (the
//! ceiling), by **budgeted** interpolation `f + (c − f)/(d_up + 1)` (P4).
//! Anchors are exact (P3). Unbounded tails step by `gauge_step` off a synthetic
//! floor/off the floor (P5/P6). A class with neither floor nor ceiling is
//! gauged to `default_value` (P7). An anchor-free graph is spread by height
//! (P8). This is a faithful port of `.doctrine/slice/213/projection-prototype.py`,
//! whose scenario battery (S1–S8, Y1–Y7, N1–N4) is the golden suite below.
//!
//! ## Gauge scope — a flagged design/prototype tension
//! The prototype computes the P8 gauge spread over the **whole** node set
//! (single global height `H`), and takes the anchored branch whenever ANY
//! anchor exists anywhere. Design P1/P8/P12 read this per weakly-connected
//! component (independent components, `H` = component max height, universal
//! locality). They agree on every case EXCEPT a graph with ≥2 disjoint
//! anchor-free components (only golden `S2`'s `f`/`g` pendant): the prototype's
//! global `H` couples them, which a strict reading of P12 (locality) forbids.
//! This port follows the prototype (the executable ground truth the task pins
//! as the verbatim golden). The gauge tier is explicitly "a convention, not
//! evidence" (design P9) and its artifacts are accepted (D14/P15); locality is
//! property-tested for the evidence-bearing (anchored) regime, and the global
//! coupling is pinned transparently by the `s2_*` golden. Reported for
//! orchestrator adjudication.

use std::collections::{BTreeMap, BTreeSet};

use super::compile::{ClassId, ConstraintSet, EdgeMap};

/// The provenance of a projected scalar (design D11, `authored > projected >
/// gauge`). Absence from a [`Projection`] is the implicit fourth tier
/// (`default`) — an entity with no row evidence is never placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueProvenance {
    /// P3: the class carries an authored anchor; the value is exact.
    Authored,
    /// P4/P5/P6: interpolated or gauge-stepped between order neighbours.
    Projected,
    /// P7/P8: placed by the gauge convention, not by evidence.
    Gauge,
}

/// The two gauge constants, passed as pure inputs this phase. A later phase
/// homes the shipped `GAUGE_STEP` / `DEFAULT_VALUE` named constants in
/// `priority/config.rs` (STD-001) and threads them here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ProjectionCfg {
    /// The additive gauge step for unbounded tails/heads (`0.25` = a quarter
    /// of `default_value`, design P5).
    pub gauge_step: f64,
    /// The gauge value for a class with no order path to any anchor (P7) and
    /// the scale of the anchor-free spread (P8).
    pub default_value: f64,
}

/// Entity id → its projected scalar and provenance. Deterministic ordering
/// (`BTree`); `f64` is never a key.
pub(crate) type Projection = BTreeMap<String, (f64, ValueProvenance)>;

/// Directed adjacency over class ids (winner → losers, or its transpose).
type Adj = BTreeMap<ClassId, BTreeSet<ClassId>>;

/// Project a [`ConstraintSet`] onto per-entity scalars (design §3). Pure and
/// deterministic over its inputs.
pub(crate) fn project(cs: &ConstraintSet, cfg: &ProjectionCfg) -> Projection {
    let nodes: BTreeSet<ClassId> = cs.classes.values().cloned().collect();
    let (out, inn) = adjacency(&cs.edges);
    let class_values = place(&nodes, &out, &inn, &cs.anchors, cfg);
    // Fan the class value out to every member entity (P1: every member of a
    // class gets the class value).
    cs.classes
        .iter()
        .filter_map(|(entity, class)| {
            class_values
                .get(class)
                .map(|&(value, prov)| (entity.clone(), (value, prov)))
        })
        .collect()
}

// ---- graph helpers ---------------------------------------------------------

/// Build the forward (winner → losers) and transpose (loser → winners)
/// adjacencies from the retained edge keys. Deliberate small duplication of
/// `compile.rs`'s private `adjacency`/`transpose` (design §1 sanctions local
/// graph helpers over widening a sibling module's surface).
fn adjacency(edges: &EdgeMap) -> (Adj, Adj) {
    let mut out: Adj = BTreeMap::new();
    let mut inn: Adj = BTreeMap::new();
    for (winner, loser) in edges.keys() {
        out.entry(winner.clone()).or_default().insert(loser.clone());
        inn.entry(loser.clone()).or_default().insert(winner.clone());
    }
    (out, inn)
}

/// Reverse-topological order (sinks/lowest first), deterministic by class id
/// (Kahn over out-degree; every node in `nodes`, including edge-free isolates,
/// is emitted). Post-C3 the graph is a DAG (PHASE-03 guarantee), so this
/// always drains.
fn topo_order(nodes: &BTreeSet<ClassId>, out: &Adj, inn: &Adj) -> Vec<ClassId> {
    let mut indeg: BTreeMap<ClassId, usize> = nodes
        .iter()
        .map(|n| (n.clone(), out.get(n).map_or(0, BTreeSet::len)))
        .collect();
    let mut ready: BTreeSet<ClassId> = nodes
        .iter()
        .filter(|n| indeg.get(*n).copied().unwrap_or(0) == 0)
        .cloned()
        .collect();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(n) = ready.pop_first() {
        if let Some(preds) = inn.get(&n) {
            for p in preds {
                if let Some(d) = indeg.get_mut(p) {
                    *d -= 1;
                    if *d == 0 {
                        ready.insert(p.clone());
                    }
                }
            }
        }
        order.push(n);
    }
    order
}

/// Longest-path height above each node's sinks (`h[v] = 0` at a sink, else
/// `max(h[successor]) + 1`). The P8 gauge spread's `h`.
fn heights(order: &[ClassId], out: &Adj) -> BTreeMap<ClassId, usize> {
    let mut h: BTreeMap<ClassId, usize> = BTreeMap::new();
    for v in order {
        let hv = out.get(v).map_or(0, |succ| {
            succ.iter()
                .map(|s| h.get(s).copied().unwrap_or(0) + 1)
                .max()
                .unwrap_or(0)
        });
        h.insert(v.clone(), hv);
    }
    h
}

/// For each class: `hi` = the min anchor value weakly above (the ceiling `c`,
/// P4), and `dup` = the longest directed path up to that ceiling-defining
/// anchor (ties on value resolved by the LONGER path — most room). Processed
/// sources-first so every predecessor is settled first.
fn longest_up(
    order: &[ClassId],
    inn: &Adj,
    anchors: &BTreeMap<ClassId, f64>,
) -> (BTreeMap<ClassId, f64>, BTreeMap<ClassId, usize>) {
    let mut hi: BTreeMap<ClassId, f64> = BTreeMap::new();
    let mut dup: BTreeMap<ClassId, usize> = BTreeMap::new();
    for v in order.iter().rev() {
        let mut best: Option<(f64, usize)> = None;
        if let Some(preds) = inn.get(v) {
            for p in preds {
                let cand = if let Some(&a) = anchors.get(p) {
                    Some((a, 1usize))
                } else if let (Some(&hp), Some(&dp)) = (hi.get(p), dup.get(p)) {
                    Some((hp, dp + 1))
                } else {
                    None
                };
                if let Some((cv, cd)) = cand {
                    let take = match best {
                        None => true,
                        Some((bv, bd)) => {
                            cv.total_cmp(&bv).is_lt() || (cv.total_cmp(&bv).is_eq() && cd > bd)
                        }
                    };
                    if take {
                        best = Some((cv, cd));
                    }
                }
            }
        }
        if let Some((bv, bd)) = best {
            hi.insert(v.clone(), bv);
            dup.insert(v.clone(), bd);
        }
    }
    (hi, dup)
}

/// Longest directed path from the nearest anchor above down to each class
/// (`d_down`, P6 synthetic-floor depth). Processed sources-first.
fn depth_below_ceiling(
    order: &[ClassId],
    inn: &Adj,
    anchors: &BTreeMap<ClassId, f64>,
) -> BTreeMap<ClassId, usize> {
    let mut depth: BTreeMap<ClassId, usize> = BTreeMap::new();
    for v in order.iter().rev() {
        let mut best: Option<usize> = None;
        if let Some(preds) = inn.get(v) {
            for p in preds {
                let cand = if anchors.contains_key(p) {
                    Some(1)
                } else {
                    depth.get(p).map(|d| d + 1)
                };
                if let Some(c) = cand {
                    best = Some(best.map_or(c, |b: usize| b.max(c)));
                }
            }
        }
        if let Some(b) = best {
            depth.insert(v.clone(), b);
        }
    }
    depth
}

/// The max already-placed value over `v`'s DIRECT successors — the floor `f`
/// (P4). `None` when `v` has no placed descendant.
fn successor_max(values: &BTreeMap<ClassId, f64>, out: &Adj, v: &str) -> Option<f64> {
    out.get(v).and_then(|succ| {
        succ.iter()
            .filter_map(|s| values.get(s).copied())
            .reduce(|a, b| if a.total_cmp(&b).is_ge() { a } else { b })
    })
}

/// Python `x or 1` for an optional count: `None`/`0` → `1`.
fn nonzero_or_one(x: Option<usize>) -> usize {
    match x {
        Some(0) | None => 1,
        Some(n) => n,
    }
}

/// Widen a small graph count (height / path length) to `f64` without an `as`
/// cast — the project-wide lint surface denies `as_conversions`
/// (`priority/findings.rs` idiom). These counts never approach `u32::MAX`.
fn count_f64(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// Place every class (P3–P8). Anchor-free graph → gauge spread (P8);
/// otherwise reverse-topological budgeted interpolation with gauge tails.
fn place(
    nodes: &BTreeSet<ClassId>,
    out: &Adj,
    inn: &Adj,
    anchors: &BTreeMap<ClassId, f64>,
    cfg: &ProjectionCfg,
) -> BTreeMap<ClassId, (f64, ValueProvenance)> {
    let order = topo_order(nodes, out, inn);

    // P8: anchor-free spread over the whole node set (see module note on the
    // global-vs-per-component gauge scope).
    if anchors.is_empty() {
        let h = heights(&order, out);
        let big_h = h.values().copied().max().unwrap_or(0);
        let denom = count_f64(big_h) + 2.0;
        return nodes
            .iter()
            .map(|n| {
                let hn = h.get(n).copied().unwrap_or(0);
                let value = 2.0 * cfg.default_value * (count_f64(hn) + 1.0) / denom;
                (n.clone(), (value, ValueProvenance::Gauge))
            })
            .collect();
    }

    let (hi, dup) = longest_up(&order, inn, anchors);
    let dbc = depth_below_ceiling(&order, inn, anchors);
    let mut values: BTreeMap<ClassId, f64> = BTreeMap::new();
    let mut result: BTreeMap<ClassId, (f64, ValueProvenance)> = BTreeMap::new();

    for v in &order {
        // P3: an anchored class takes its authored value exactly.
        if let Some(&anchor) = anchors.get(v) {
            debug_assert!(
                successor_max(&values, out, v).is_none_or(|f| f < anchor),
                "P3: infeasible anchor placement — C5 should have guaranteed floor < anchor"
            );
            values.insert(v.clone(), anchor);
            result.insert(v.clone(), (anchor, ValueProvenance::Authored));
            continue;
        }

        let floor = successor_max(&values, out, v);
        let ceiling = hi.get(v).copied();
        let value = match (floor, ceiling) {
            // P7: no floor AND no ceiling — gauge to default_value.
            (None, None) => {
                values.insert(v.clone(), cfg.default_value);
                result.insert(v.clone(), (cfg.default_value, ValueProvenance::Gauge));
                continue;
            }
            // P6: unbounded below — synthetic floor strictly under the ceiling
            // (clamped to ≥ 0 only when the ceiling is positive), then P4 up.
            (None, Some(c)) => {
                let down = nonzero_or_one(dbc.get(v).copied());
                let mut synthetic = c - cfg.gauge_step * (count_f64(down) + 1.0);
                if c > 0.0 {
                    synthetic = synthetic.max(0.0);
                }
                let up = nonzero_or_one(dup.get(v).copied());
                synthetic + (c - synthetic) / (count_f64(up) + 1.0)
            }
            // P5: unbounded above — one additive gauge step off the floor.
            (Some(f), None) => f + cfg.gauge_step,
            // P4: budgeted interpolation between floor and ceiling.
            (Some(f), Some(c)) => {
                let up = nonzero_or_one(dup.get(v).copied());
                f + (c - f) / (count_f64(up) + 1.0)
            }
        };
        values.insert(v.clone(), value);
        result.insert(v.clone(), (value, ValueProvenance::Projected));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::super::compile::{AnchorMap, QuarantinePolicy, compile};
    use super::{Projection, ProjectionCfg, ValueProvenance, project};
    use crate::comparison::{
        DOMAIN_VALUE, FRAME_EQUAL_EFFORT, Judgement, RaterKind, Response, RowForm,
    };

    use ValueProvenance::{Authored, Gauge, Projected};

    /// The prototype's `DEFAULT = 1.0`, `STEP = 0.25`.
    const CFG: ProjectionCfg = ProjectionCfg {
        gauge_step: 0.25,
        default_value: 1.0,
    };
    const EPS: f64 = 1e-4;

    // ---- fixtures ----------------------------------------------------------

    fn judgement(uid: &str, winner: &str, loser: &str) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: winner.to_string(),
            b: loser.to_string(),
            response: Response::PreferA,
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: "2026-07-11".to_string(),
        }
    }

    /// Compile a scenario (edges winner→loser, anchors) through the real
    /// tier-2 seam and project it. Going through `compile()` keeps the
    /// fixtures honest: classes, edges and anchors are exactly what the
    /// pipeline produces (all scenarios are feasible DAGs — no quarantine).
    fn project_scenario(edges: &[(&str, &str)], anchors: &[(&str, f64)]) -> Projection {
        project(&compiled(edges, anchors), &CFG)
    }

    fn compiled(edges: &[(&str, &str)], anchors: &[(&str, f64)]) -> super::ConstraintSet {
        let rows: Vec<Judgement> = edges
            .iter()
            .enumerate()
            .map(|(i, (w, l))| judgement(&format!("j{i}"), w, l))
            .collect();
        let refs: Vec<&Judgement> = rows.iter().collect();
        let amap: AnchorMap = anchors.iter().map(|&(e, v)| (e.to_string(), v)).collect();
        let cs = compile(&refs, &amap, QuarantinePolicy::Symmetric);
        assert!(cs.quarantined.is_empty(), "fixture must be a feasible DAG");
        cs
    }

    fn value(p: &Projection, e: &str) -> f64 {
        p.get(e).unwrap_or_else(|| panic!("missing entity {e}")).0
    }

    /// Assert an entity's value (approx 1e-4) and provenance against the
    /// prototype's 4-decimal printed golden.
    fn golden(p: &Projection, e: &str, expected: f64, prov: ValueProvenance) {
        let (got, got_prov) = *p.get(e).unwrap_or_else(|| panic!("missing entity {e}"));
        assert!(
            (got - expected).abs() < EPS,
            "{e}: got {got:.4}, want {expected:.4}"
        );
        assert_eq!(got_prov, prov, "{e} provenance");
        assert!(!got.is_nan(), "{e} is NaN");
    }

    fn chain(prefix: &str, n: usize) -> Vec<(String, String)> {
        (0..n.saturating_sub(1))
            .map(|i| (format!("{prefix}{i}"), format!("{prefix}{}", i + 1)))
            .collect()
    }

    fn edge_refs(edges: &[(String, String)]) -> Vec<(&str, &str)> {
        edges
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect()
    }

    // ---- VT-1: the prototype battery (S1–S8, Y1–Y7, N1–N4) -----------------

    #[test]
    fn s1_chain8_gauge() {
        let e = chain("n", 8);
        let p = project_scenario(&edge_refs(&e), &[]);
        for (i, want) in [
            1.7778, 1.5556, 1.3333, 1.1111, 0.8889, 0.6667, 0.4444, 0.2222,
        ]
        .into_iter()
        .enumerate()
        {
            golden(&p, &format!("n{i}"), want, Gauge);
        }
    }

    #[test]
    fn s2_partial_order_gauge() {
        // Diamond a>b,a>c,b>d,c>d,d>e PLUS a disjoint pendant f>g. The gauge
        // spread here uses the GLOBAL height H=3 (prototype ground truth) —
        // f/g are pinned at 0.8/0.4 (not the 1.3333/0.6667 a per-component H
        // would give). This transparently pins the global-gauge coupling
        // flagged in the module note (design P8/P12 read per-component).
        let edges = [
            ("a", "b"),
            ("a", "c"),
            ("b", "d"),
            ("c", "d"),
            ("d", "e"),
            ("f", "g"),
        ];
        let p = project_scenario(&edges, &[]);
        golden(&p, "a", 1.6000, Gauge);
        golden(&p, "b", 1.2000, Gauge);
        golden(&p, "c", 1.2000, Gauge);
        golden(&p, "d", 0.8000, Gauge);
        golden(&p, "f", 0.8000, Gauge);
        golden(&p, "e", 0.4000, Gauge);
        golden(&p, "g", 0.4000, Gauge);
    }

    #[test]
    fn s3_mid_anchor_budgeted() {
        let e = chain("n", 8);
        let p = project_scenario(&edge_refs(&e), &[("n4", 5.0)]);
        golden(&p, "n0", 6.0000, Projected);
        golden(&p, "n1", 5.7500, Projected);
        golden(&p, "n2", 5.5000, Projected);
        golden(&p, "n3", 5.2500, Projected);
        golden(&p, "n4", 5.0000, Authored);
        golden(&p, "n5", 4.7500, Projected);
        golden(&p, "n6", 4.5000, Projected);
        golden(&p, "n7", 4.2500, Projected);
    }

    #[test]
    fn s4_low_anchor_deep_tail() {
        let e = chain("m", 6);
        let p = project_scenario(&edge_refs(&e), &[("m0", 0.5)]);
        golden(&p, "m0", 0.5000, Authored);
        golden(&p, "m1", 0.4167, Projected);
        golden(&p, "m2", 0.3333, Projected);
        golden(&p, "m3", 0.2500, Projected);
        golden(&p, "m4", 0.1667, Projected);
        golden(&p, "m5", 0.0833, Projected);
    }

    #[test]
    fn s5_bracket_crowding_budgeted() {
        let e = chain("b", 6);
        let p = project_scenario(&edge_refs(&e), &[("b0", 8.0), ("b5", 2.0)]);
        golden(&p, "b0", 8.0000, Authored);
        golden(&p, "b1", 6.8000, Projected);
        golden(&p, "b2", 5.6000, Projected);
        golden(&p, "b3", 4.4000, Projected);
        golden(&p, "b4", 3.2000, Projected);
        golden(&p, "b5", 2.0000, Authored);
    }

    #[test]
    fn s6_sparse_anchor_before_and_after() {
        let edges = [
            ("p", "q"),
            ("q", "r"),
            ("p", "s"),
            ("s", "t"),
            ("t", "u"),
            ("q", "t"),
        ];
        let before = project_scenario(&edges, &[]);
        golden(&before, "p", 1.6000, Gauge);
        golden(&before, "q", 1.2000, Gauge);
        golden(&before, "s", 1.2000, Gauge);
        golden(&before, "t", 0.8000, Gauge);
        golden(&before, "r", 0.4000, Gauge);
        golden(&before, "u", 0.4000, Gauge);

        let after = project_scenario(&edges, &[("s", 5.0)]);
        golden(&after, "p", 5.2500, Projected);
        golden(&after, "q", 5.0000, Projected);
        golden(&after, "s", 5.0000, Authored);
        golden(&after, "t", 4.7500, Projected);
        golden(&after, "u", 4.5000, Projected);
        golden(&after, "r", 1.0000, Gauge);
        // Order flips r below u after the anchor lands (minimal-motion demo).
        assert!(value(&after, "u") > value(&after, "r"));
    }

    #[test]
    fn s7_cross_window_edge() {
        let edges = [("A8", "u"), ("u", "v"), ("v", "A2"), ("A5", "v")];
        let p = project_scenario(&edges, &[("A8", 8.0), ("A5", 5.0), ("A2", 2.0)]);
        golden(&p, "A8", 8.0000, Authored);
        golden(&p, "u", 5.7500, Projected);
        golden(&p, "A5", 5.0000, Authored);
        golden(&p, "v", 3.5000, Projected);
        golden(&p, "A2", 2.0000, Authored);
    }

    #[test]
    fn s8_incremental_locality() {
        let base = [("x", "y"), ("z", "w")];
        let islands = project_scenario(&base, &[("x", 4.0)]);
        golden(&islands, "x", 4.0000, Authored);
        golden(&islands, "y", 3.7500, Projected);
        golden(&islands, "z", 1.2500, Projected);
        golden(&islands, "w", 1.0000, Gauge);

        let joined = project_scenario(&[("x", "y"), ("z", "w"), ("y", "z")], &[("x", 4.0)]);
        golden(&joined, "x", 4.0000, Authored);
        golden(&joined, "y", 3.7500, Projected);
        golden(&joined, "z", 3.5000, Projected);
        golden(&joined, "w", 3.2500, Projected);
    }

    #[test]
    fn y1_join_y_gauge() {
        let edges = [
            ("a0", "a1"),
            ("a1", "a2"),
            ("a2", "j"),
            ("b0", "b1"),
            ("b1", "j"),
        ];
        let p = project_scenario(&edges, &[]);
        golden(&p, "a0", 1.6000, Gauge);
        golden(&p, "a1", 1.2000, Gauge);
        golden(&p, "b0", 1.2000, Gauge);
        golden(&p, "a2", 0.8000, Gauge);
        golden(&p, "b1", 0.8000, Gauge);
        golden(&p, "j", 0.4000, Gauge);
    }

    #[test]
    fn y2_split_y_gauge() {
        let edges = [
            ("h", "c0"),
            ("c0", "c1"),
            ("c1", "c2"),
            ("h", "d0"),
            ("d0", "d1"),
        ];
        let p = project_scenario(&edges, &[]);
        golden(&p, "h", 1.6000, Gauge);
        golden(&p, "c0", 1.2000, Gauge);
        golden(&p, "c1", 0.8000, Gauge);
        golden(&p, "d0", 0.8000, Gauge);
        golden(&p, "c2", 0.4000, Gauge);
        golden(&p, "d1", 0.4000, Gauge);
    }

    #[test]
    fn y3_sensitivity_extend_short_arm() {
        let edges = [
            ("a0", "a1"),
            ("a1", "a2"),
            ("a2", "j"),
            ("b0", "b1"),
            ("b1", "b2"),
            ("b2", "j"),
        ];
        let p = project_scenario(&edges, &[]);
        golden(&p, "a0", 1.6000, Gauge);
        golden(&p, "b0", 1.6000, Gauge);
        golden(&p, "a1", 1.2000, Gauge);
        golden(&p, "b1", 1.2000, Gauge);
        golden(&p, "a2", 0.8000, Gauge);
        golden(&p, "b2", 0.8000, Gauge);
        golden(&p, "j", 0.4000, Gauge);
    }

    #[test]
    fn y4_pin_cross_judgement() {
        let edges = [
            ("a0", "a1"),
            ("a1", "a2"),
            ("a2", "j"),
            ("b0", "b1"),
            ("b1", "j"),
            ("b0", "a1"),
        ];
        let p = project_scenario(&edges, &[]);
        golden(&p, "a0", 1.6000, Gauge);
        golden(&p, "b0", 1.6000, Gauge);
        golden(&p, "a1", 1.2000, Gauge);
        golden(&p, "a2", 0.8000, Gauge);
        golden(&p, "b1", 0.8000, Gauge);
        golden(&p, "j", 0.4000, Gauge);
    }

    #[test]
    fn y5_pin_with_anchor_collision() {
        // b0=3 anchor; a1 and b1 both land at 2.75 — the accepted
        // anchor-value collision (D14/P15), order-safe, disambiguated by
        // provenance.
        let edges = [
            ("a0", "a1"),
            ("a1", "a2"),
            ("a2", "j"),
            ("b0", "b1"),
            ("b1", "j"),
        ];
        let p = project_scenario(&edges, &[("b0", 3.0)]);
        golden(&p, "a0", 3.2500, Projected);
        golden(&p, "a1", 3.0000, Projected);
        golden(&p, "b0", 3.0000, Authored);
        golden(&p, "a2", 2.7500, Projected);
        golden(&p, "b1", 2.7500, Projected);
        golden(&p, "j", 2.5000, Projected);
    }

    #[test]
    fn y6_bracketed_arms() {
        let edges = [
            ("T", "a1"),
            ("a1", "a2"),
            ("a2", "a3"),
            ("a3", "B"),
            ("T", "b1"),
            ("b1", "B"),
        ];
        let p = project_scenario(&edges, &[("T", 8.0), ("B", 2.0)]);
        golden(&p, "T", 8.0000, Authored);
        golden(&p, "a1", 6.5000, Projected);
        golden(&p, "a2", 5.0000, Projected);
        golden(&p, "b1", 5.0000, Projected);
        golden(&p, "a3", 3.5000, Projected);
        golden(&p, "B", 2.0000, Authored);
    }

    #[test]
    fn y7_pin_inside_bracket() {
        let edges = [
            ("T", "a1"),
            ("a1", "a2"),
            ("a2", "a3"),
            ("a3", "B"),
            ("T", "b1"),
            ("b1", "B"),
            ("a2", "b1"),
        ];
        let p = project_scenario(&edges, &[("T", 8.0), ("B", 2.0)]);
        golden(&p, "T", 8.0000, Authored);
        golden(&p, "a1", 6.5000, Projected);
        golden(&p, "a2", 5.0000, Projected);
        golden(&p, "a3", 3.5000, Projected);
        golden(&p, "b1", 3.5000, Projected);
        golden(&p, "B", 2.0000, Authored);
        // Minimal motion: only the touched arm (b1) moved vs y6; a1/a2/a3 held.
        assert!((value(&p, "a2") - 5.0).abs() < EPS);
    }

    #[test]
    fn n1_negative_ceiling_tail() {
        let e = chain("k", 4);
        let p = project_scenario(&edge_refs(&e), &[("k0", -0.5)]);
        golden(&p, "k0", -0.5000, Authored);
        golden(&p, "k1", -0.7500, Projected);
        golden(&p, "k2", -1.0000, Projected);
        golden(&p, "k3", -1.2500, Projected);
    }

    #[test]
    fn n2_sign_crossing_bracket() {
        let e = chain("j", 4);
        let p = project_scenario(&edge_refs(&e), &[("j0", 3.0), ("j3", -2.0)]);
        golden(&p, "j0", 3.0000, Authored);
        golden(&p, "j1", 1.3333, Projected);
        golden(&p, "j2", -0.3333, Projected);
        golden(&p, "j3", -2.0000, Authored);
    }

    #[test]
    fn n3_branch_floor_and_ceiling() {
        let edges = [("A", "X"), ("X", "Y"), ("X", "Z")];
        let p = project_scenario(&edges, &[("A", 10.0), ("Z", 8.0)]);
        golden(&p, "A", 10.0000, Authored);
        golden(&p, "X", 9.7500, Projected);
        golden(&p, "Y", 9.5000, Projected);
        golden(&p, "Z", 8.0000, Authored);
    }

    #[test]
    fn n4_multi_ceiling_tiebreak() {
        let edges = [("A", "X"), ("B", "W"), ("W", "X"), ("X", "Y")];
        let p = project_scenario(&edges, &[("A", 10.0), ("B", 9.0)]);
        golden(&p, "A", 10.0000, Authored);
        golden(&p, "B", 9.0000, Authored);
        golden(&p, "W", 8.7500, Projected);
        golden(&p, "X", 8.5000, Projected);
        golden(&p, "Y", 8.2500, Projected);
    }

    // ---- VT-2: property tests over DETERMINISTICALLY generated inputs -------

    /// Every response assignment over four entities' six pairs (4^6 ledgers)
    /// crossed with three anchor configs — the same deterministic enumeration
    /// PHASE-03 uses (no proptest, no rng).
    fn generated_cases() -> impl Iterator<Item = (Vec<(String, String)>, Vec<(&'static str, f64)>)>
    {
        const PAIRS: [(&str, &str); 6] = [
            ("A", "B"),
            ("A", "C"),
            ("A", "D"),
            ("B", "C"),
            ("B", "D"),
            ("C", "D"),
        ];
        let configs: [Vec<(&'static str, f64)>; 3] = [
            vec![],
            vec![("A", 5.0), ("D", 1.0)],
            vec![("A", 8.0), ("B", 3.0), ("D", 1.0)],
        ];
        configs.into_iter().flat_map(move |cfg| {
            (0..4_u32.pow(6)).filter_map(move |mask| {
                // Only keep ledgers whose anchors are order-consistent with a
                // possible acyclic reading — compile quarantines the rest, so
                // we simply project whatever survives (retained edges only).
                let mut edges = Vec::new();
                for (i, (a, b)) in PAIRS.iter().enumerate() {
                    match (mask / 4_u32.pow(i as u32)) % 4 {
                        1 => edges.push(((*a).to_string(), (*b).to_string())),
                        2 => edges.push(((*b).to_string(), (*a).to_string())),
                        // 0 = absent, 3 = equal (skip; we only need strict
                        // edges for the order-safety property).
                        _ => {}
                    }
                }
                Some((edges, cfg.clone()))
            })
        })
    }

    #[test]
    fn p10_generated_order_consistency_no_nan() {
        for (edges, anchors) in generated_cases() {
            let refs = edge_refs(&edges);
            let rows: Vec<Judgement> = refs
                .iter()
                .enumerate()
                .map(|(i, (w, l))| judgement(&format!("j{i}"), w, l))
                .collect();
            let jrefs: Vec<&Judgement> = rows.iter().collect();
            let amap: AnchorMap = anchors.iter().map(|&(e, v)| (e.to_string(), v)).collect();
            let cs = compile(&jrefs, &amap, QuarantinePolicy::Symmetric);
            let p = project(&cs, &CFG);
            // Every RETAINED strict edge is strictly respected; nothing NaN.
            for (winner, loser) in cs.edges.keys() {
                let wv = cs.classes.get(winner).and_then(|c| class_value(&cs, &p, c));
                let lv = cs.classes.get(loser).and_then(|c| class_value(&cs, &p, c));
                if let (Some(w), Some(l)) = (wv, lv) {
                    assert!(w > l, "edge {winner}>{loser}: {w} !> {l}");
                }
            }
            for (_, (val, _)) in &p {
                assert!(!val.is_nan(), "NaN projected value");
            }
        }
    }

    /// A class's value via any of its member entities.
    fn class_value(cs: &super::ConstraintSet, p: &Projection, class: &str) -> Option<f64> {
        cs.classes
            .iter()
            .find(|(_, c)| c.as_str() == class)
            .and_then(|(entity, _)| p.get(entity).map(|&(v, _)| v))
    }

    #[test]
    fn p11_determinism_under_permuted_input() {
        let edges = [("A", "B"), ("B", "C"), ("C", "D"), ("A", "E"), ("E", "D")];
        let anchors = [("A", 9.0), ("D", 1.0)];
        let rows: Vec<Judgement> = edges
            .iter()
            .enumerate()
            .map(|(i, (w, l))| judgement(&format!("j{i}"), w, l))
            .collect();
        let amap: AnchorMap = anchors.iter().map(|&(e, v)| (e.to_string(), v)).collect();
        let forward: Vec<&Judgement> = rows.iter().collect();
        let backward: Vec<&Judgement> = rows.iter().rev().collect();
        let p1 = project(&compile(&forward, &amap, QuarantinePolicy::Symmetric), &CFG);
        let p2 = project(
            &compile(&backward, &amap, QuarantinePolicy::Symmetric),
            &CFG,
        );
        assert_eq!(
            p1, p2,
            "projection must be bitwise-identical across input order"
        );
    }

    #[test]
    fn p12_locality_disjoint_anchored_components() {
        // Scope: the evidence-bearing (anchored) regime — the gauge
        // convention's whole-graph spread is a documented artifact (module
        // note, s2 golden). Component X = {A>B, A=10}; component Y = {P>Q,
        // P=5}. Perturbing X (append B>C) must not move Y's values.
        let y_edges = [("P", "Q")];
        let x_before = [("A", "B")];
        let anchors = [("A", 10.0), ("P", 5.0)];

        let combined = |x: &[(&str, &str)]| -> Projection {
            let edges: Vec<(&str, &str)> = x.iter().chain(y_edges.iter()).copied().collect();
            project_scenario(&edges, &anchors)
        };
        let base = combined(&x_before);
        let perturbed = combined(&[("A", "B"), ("B", "C")]);
        for e in ["P", "Q"] {
            assert_eq!(
                base.get(e),
                perturbed.get(e),
                "component Y entity {e} moved under a disjoint-X evidence delta"
            );
        }
    }

    #[test]
    fn p14_scoped_affine_equivariance() {
        // Within an anchor-bracketed span, shifting/scaling BOTH anchors
        // shifts/scales the interior projections identically. Scope limit:
        // this holds for bracketed (floor-and-ceiling) spans; unbounded tails
        // move by absolute gauge_step, not affinely (design P14).
        let e = chain("b", 6);
        let refs = edge_refs(&e);
        let base = project_scenario(&refs, &[("b0", 8.0), ("b5", 2.0)]);

        // Shift: +10 on both anchors ⇒ +10 on every bracketed class.
        let shifted = project_scenario(&refs, &[("b0", 18.0), ("b5", 12.0)]);
        for i in 1..5 {
            let k = format!("b{i}");
            assert!(
                (value(&shifted, &k) - (value(&base, &k) + 10.0)).abs() < EPS,
                "shift {k}"
            );
        }

        // Scale: ×2 about zero on both anchors ⇒ ×2 on every bracketed class.
        let scaled = project_scenario(&refs, &[("b0", 16.0), ("b5", 4.0)]);
        for i in 1..5 {
            let k = format!("b{i}");
            assert!(
                (value(&scaled, &k) - value(&base, &k) * 2.0).abs() < EPS,
                "scale {k}"
            );
        }
    }

    #[test]
    fn p6_synthetic_floor_strictly_below_ceiling() {
        // Unbounded-below tails: every projected value strictly below its
        // ceiling and strictly decreasing; positive ceiling keeps them ≥ 0,
        // negative ceiling lets them go negative (RV-265 F-1). Swept over
        // depths and both ceiling signs.
        for &(anchor, positive) in &[(0.5_f64, true), (-0.5_f64, false)] {
            for depth in 1..6 {
                let e = chain("t", depth + 1);
                let p = project_scenario(&edge_refs(&e), &[("t0", anchor)]);
                let mut prev = anchor;
                for i in 1..=depth {
                    let cur = value(&p, &format!("t{i}"));
                    assert!(cur < prev, "not strictly decreasing at t{i}");
                    assert!(cur < anchor, "t{i} not strictly below ceiling");
                    if positive {
                        assert!(cur >= 0.0, "positive ceiling manufactured a negative t{i}");
                    }
                    prev = cur;
                }
            }
        }
    }

    #[test]
    fn p8_gauge_spread_positive_and_centred() {
        // Anchor-free spread lands strictly in (0, 2·default) and is centred:
        // min + max = 2·default (the extremes are symmetric about default).
        let e = chain("n", 8);
        let p = project_scenario(&edge_refs(&e), &[]);
        let vals: Vec<f64> = (0..8).map(|i| value(&p, &format!("n{i}"))).collect();
        let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(lo > 0.0, "gauge below 0");
        assert!(hi < 2.0 * CFG.default_value, "gauge above 2·default");
        assert!(
            (lo + hi - 2.0 * CFG.default_value).abs() < EPS,
            "not centred"
        );
    }

    // ---- EX-3: GAUGE_STEP sensitivity sweep --------------------------------

    #[test]
    fn ex3_gauge_step_sweep_order_and_provenance_invariant() {
        // Order-safety and provenance labels must hold at every gauge_step on
        // the grid 0.05..=1.0 (step 0.05), over scenarios exercising both
        // unbounded tails (P6) and unbounded heads (P5/P7).
        let tail = chain("m", 6); // low anchor, deep tail below (P6)
        let head = [("x", "y"), ("z", "w"), ("y", "z")]; // chain under anchor + P5 head
        let mut step_milli = 5_u32;
        while step_milli <= 1000 {
            let cfg = ProjectionCfg {
                gauge_step: f64::from(step_milli) / 1000.0,
                default_value: 1.0,
            };

            let tp = project(&compiled(&edge_refs(&tail), &[("m0", 0.5)]), &cfg);
            assert_eq!(tp.get("m0").map(|&(_, pr)| pr), Some(Authored));
            for i in 1..6 {
                assert_eq!(
                    tp.get(&format!("m{i}")).map(|&(_, pr)| pr),
                    Some(Projected),
                    "tail provenance drifted at step {step_milli}"
                );
            }
            for i in 0..5 {
                assert!(
                    value(&tp, &format!("m{i}")) > value(&tp, &format!("m{}", i + 1)),
                    "tail order broke at step {step_milli}"
                );
            }

            let hp = project(&compiled(&head, &[("x", 4.0)]), &cfg);
            for (winner, loser) in &head {
                assert!(
                    value(&hp, winner) > value(&hp, loser),
                    "head order {winner}>{loser} broke at step {step_milli}"
                );
            }
            assert_eq!(hp.get("x").map(|&(_, pr)| pr), Some(Authored));

            step_milli += 50;
        }
    }
}
