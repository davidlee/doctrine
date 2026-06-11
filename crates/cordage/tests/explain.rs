//! VT-1 (explain on cycles, F47) + VT-2 (eviction endpoint filter, F26).
//!
//! `explain(n)` assembles, per overlay, the predecessor **cone** of `n` — the
//! predecessor sub-DAG as a `node ↦ {immediate in-cone predecessors}` adjacency
//! map — and the evictions with `n` as an endpoint. On a cyclic `Reject` view the
//! cone terminates at roots and degraded post-arity SCC entries: both are recorded
//! as empty-pred keys (endpoints, never walked through); a node inside an SCC gets
//! `{n: {}}`. The cycle itself is explained by `Provenance.cycles`, never by the
//! cone. Black-box: opaque ids minted by the builder, structural vocabulary only.

use std::collections::{BTreeMap, BTreeSet};

use cordage::{
    Arity, CyclePolicy, Direction, EdgeAttrs, EvictReason, GraphBuilder, NodeId, OrderLayer,
    OrderSpec, OverlayConfig,
};

/// A cone `node ↦ preds` built from `(node, [preds])` pairs — captured builder
/// ids only, never minted literals.
fn cone(entries: Vec<(NodeId, Vec<NodeId>)>) -> BTreeMap<NodeId, BTreeSet<NodeId>> {
    entries
        .into_iter()
        .map(|(n, preds)| (n, preds.into_iter().collect()))
        .collect()
}

fn reject() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

fn at_most_one() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::AtMostOne)
}

fn evict() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded)
}

// ── VT-1: explain on cycles (F47) ─────────────────────────────────────────────

#[test]
fn explain_chain_ends_at_a_degraded_scc_entry_not_walking_through_it() {
    // §9 explain-on-cycles row: Reject a ↔ b (a degraded post-arity SCC) plus
    // a → x. explain(x)'s chain must end AT a (the SCC entry), never walk into the
    // cycle. The order spec references the overlay so the SCC actually degrades.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nx = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, na, EdgeAttrs::new(0, 0)); // a ↔ b cycle
    b.edge(ov, na, nx, EdgeAttrs::new(0, 0));
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(ov, Direction::Along)]));
    let g = b.build().expect("valid");

    let ex = g.explain(nx);
    // x's cone records a as its predecessor; a is a degraded-SCC endpoint, so it is
    // an empty-pred key — the walk stops there and never continues into b.
    assert_eq!(
        ex.predecessors().get(&ov),
        Some(&cone(vec![(nx, vec![na]), (na, vec![])]))
    );
}

#[test]
fn explain_of_a_node_inside_a_degraded_scc_is_the_singleton_chain() {
    // explain(a) where a is itself a member of the degraded SCC a ↔ b → {a: {}}.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nx = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, na, EdgeAttrs::new(0, 0));
    b.edge(ov, na, nx, EdgeAttrs::new(0, 0));
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(ov, Direction::Along)]));
    let g = b.build().expect("valid");

    let ex = g.explain(na);
    assert_eq!(ex.predecessors().get(&ov), Some(&cone(vec![(na, vec![])])));
}

#[test]
fn explain_terminates_and_is_deterministic_with_the_cycle_in_provenance() {
    // The cycle is surfaced via Provenance.cycles, NOT via the cone. explain must
    // terminate (proven by returning) and be deterministic (recompute identical).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nx = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, na, EdgeAttrs::new(0, 0));
    b.edge(ov, na, nx, EdgeAttrs::new(0, 0));
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(ov, Direction::Along)]));
    let g = b.build().expect("valid");

    // The cycle lives in provenance, not in the cone.
    assert!(!g.provenance().cycles().is_empty());

    // Deterministic: a second explain of the same node is identical.
    assert_eq!(g.explain(nx).predecessors(), g.explain(nx).predecessors());
}

#[test]
fn explain_walks_a_multi_parent_dag_to_distinct_roots() {
    // root1 → n, root2 → n on a plain (acyclic) Reject overlay: n's cone records
    // BOTH roots as its predecessors, each itself an empty-pred root key. Distinct
    // from spine_path (single chain) — this is the multi-parent Unbounded cone.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let r1 = b.node();
    let r2 = b.node();
    let n = b.node();
    b.edge(ov, r1, n, EdgeAttrs::new(0, 0));
    b.edge(ov, r2, n, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    // n ↦ {r1, r2}; both roots are empty-pred keys — deterministic adjacency.
    assert_eq!(
        g.explain(n).predecessors().get(&ov),
        Some(&cone(vec![(n, vec![r1, r2]), (r1, vec![]), (r2, vec![])]))
    );
}

#[test]
fn explain_keys_every_overlay_a_root_node_is_singleton_on_each() {
    // A8: a node with no predecessors on an overlay is {n: {}} (present, not
    // absent). Two overlays, the node a root on both → both keyed, each {n: {}}.
    let mut b = GraphBuilder::new();
    let ov1 = b.overlay(reject());
    let ov2 = b.overlay(reject());
    let n = b.node();
    let m = b.node();
    b.edge(ov1, n, m, EdgeAttrs::new(0, 0));
    b.edge(ov2, n, m, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let ex = g.explain(n);
    assert_eq!(ex.predecessors().get(&ov1), Some(&cone(vec![(n, vec![])])));
    assert_eq!(ex.predecessors().get(&ov2), Some(&cone(vec![(n, vec![])])));
}

#[test]
fn explain_order_key_matches_graph_order_key() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let n = b.node();
    let m = b.node();
    b.edge(ov, n, m, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert_eq!(Some(g.explain(m).order_key()), g.order_key(m));
}

// ── VT-2: eviction endpoint filter (F26) ──────────────────────────────────────

#[test]
fn explain_evicted_filters_to_n_as_src_or_dst() {
    // Build a graph that evicts edges, some touching n and one not. explain(n)
    // surfaces exactly the evictions with n as src OR dst; unrelated absent.
    //
    // On an AtMostOne overlay, a node with two parents loses the weaker in-edge
    // (ArityViolation). We arrange two evictions touching n (as dst, and as src
    // via a downstream child with two parents where n is the loser) plus one
    // unrelated eviction between two other nodes.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(at_most_one());
    let p_lo = b.node(); // weak parent of n → evicted (n as dst)
    let p_hi = b.node(); // kept parent of n
    let n = b.node();
    let child = b.node(); // child with two parents: n (weak) + q (strong)
    let q = b.node();
    let u = b.node(); // unrelated pair u → w, w has a stronger parent
    let v = b.node();
    let w = b.node();

    b.edge(ov, p_lo, n, EdgeAttrs::new(0, 0)); // evicted: n as dst
    b.edge(ov, p_hi, n, EdgeAttrs::new(5, 0)); // kept
    b.edge(ov, n, child, EdgeAttrs::new(0, 0)); // evicted: n as src (loser)
    b.edge(ov, q, child, EdgeAttrs::new(5, 0)); // kept parent of child
    b.edge(ov, u, w, EdgeAttrs::new(0, 0)); // unrelated, evicted (u/w, not n)
    b.edge(ov, v, w, EdgeAttrs::new(5, 0)); // kept parent of w
    let g = b.build().expect("valid");

    // Sanity: three evictions total on this overlay.
    assert_eq!(g.provenance().evictions().len(), 3);

    let ex = g.explain(n);
    let evicted: Vec<(NodeId, NodeId)> = ex
        .evicted()
        .iter()
        .map(|e| (e.edge().src(), e.edge().dst()))
        .collect();

    // n as dst (p_lo → n) and n as src (n → child) both present; (u → w) absent.
    assert!(evicted.contains(&(p_lo, n)), "n-as-dst eviction present");
    assert!(evicted.contains(&(n, child)), "n-as-src eviction present");
    assert!(!evicted.contains(&(u, w)), "unrelated eviction absent");
    assert_eq!(evicted.len(), 2);

    // The kept reason is ArityViolation (sanity that we evicted via arity).
    assert!(
        ex.evicted()
            .iter()
            .all(|e| matches!(e.reason(), EvictReason::ArityViolation))
    );
}

#[test]
fn explain_evicted_is_empty_when_no_eviction_touches_n() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(evict());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    // a self-disjoint clean DAG — no evictions at all.
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert!(g.provenance().evictions().is_empty());
    assert!(g.explain(na).evicted().is_empty());
}
