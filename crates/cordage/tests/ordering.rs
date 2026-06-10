//! PHASE-03 — build passes 3–4: cross-layer order composition (`U`), longest-path
//! levels, and cycle taint. Black-box, vocabulary-free: overlays are opaque
//! tokens, ids are minted from sibling builders, directions are `Along`/`Against`.
//! The observable surface is `ordered()`, `order_key()`, and `provenance()`.

use cordage::{
    Arity, CyclePolicy, Direction, EdgeAttrs, EvictReason, GraphBuilder, Level, NodeId, OrderLayer,
    OrderSpec, OverlayConfig,
};

fn reject_unbounded() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

fn reject_at_most_one() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::AtMostOne)
}

fn evict_unbounded() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded)
}

fn attrs() -> EdgeAttrs {
    EdgeAttrs::new(0, 0)
}

/// The `(overlay-ordinal-free) (src, dst, reason)` of each eviction, in
/// provenance order — the assertion shape for union conflicts.
fn evictions(g: &cordage::Graph) -> Vec<(NodeId, NodeId, EvictReason)> {
    g.provenance()
        .evictions()
        .iter()
        .map(|e| (e.edge().src(), e.edge().dst(), e.reason()))
        .collect()
}

// ── VT-1: union (F2) — a later layer that contradicts an earlier one yields ──
// EvictedEdge{UnionCycleVsLayer}; the earlier layer's order is preserved.

#[test]
fn union_later_layer_contradiction_is_evicted_earlier_order_preserved() {
    let mut b = GraphBuilder::new();
    let o0 = b.overlay(reject_unbounded());
    let o1 = b.overlay(reject_unbounded());
    let o2 = b.overlay(reject_unbounded());
    let a = b.node();
    let c = b.node();
    b.edge(o0, a, c, attrs()); // layer 0: a → c (authoritative)
    b.edge(o2, c, a, attrs()); // layer 2: c → a (contradicts layer 0)
    b.order_spec(OrderSpec::new(vec![
        OrderLayer::new(o0, Direction::Along),
        OrderLayer::new(o1, Direction::Along),
        OrderLayer::new(o2, Direction::Along),
    ]));
    let g = b.build().expect("valid");

    // The layer-2 edge is the one evicted; layer-0 a → c survives.
    assert_eq!(evictions(&g), vec![(c, a, EvictReason::UnionCycleVsLayer)]);
    assert_eq!(g.ordered(), vec![a, c]);
}

// ── VT-2: union composite (F10) — layer-k edges individually consistent with ──
// the prior closure can jointly close a cycle; batch-insert + SCC eviction
// removes exactly one (the total-key min) and leaves U acyclic.

#[test]
fn union_composite_cycle_evicts_one_total_key_min() {
    let mut b = GraphBuilder::new();
    let o0 = b.overlay(reject_unbounded());
    let o1 = b.overlay(reject_unbounded());
    let a = b.node();
    let c = b.node();
    let d = b.node();
    b.edge(o0, a, c, attrs()); // layer 0: a → c
    b.edge(o1, c, d, attrs()); // layer 1: c → d  ┐ jointly close a → c → d → a
    b.edge(o1, d, a, attrs()); // layer 1: d → a  ┘
    b.order_spec(OrderSpec::new(vec![
        OrderLayer::new(o0, Direction::Along),
        OrderLayer::new(o1, Direction::Along),
    ]));
    let g = b.build().expect("valid");

    // Exactly one layer-1 edge evicted; F17 key (rank,age,src,dst)-min is c → d
    // (src c < src d on the (0,0) tie).
    assert_eq!(evictions(&g), vec![(c, d, EvictReason::UnionCycleVsLayer)]);
    // Surviving U: a → c, d → a → acyclic. levels d=0, a=1, c=2.
    assert_eq!(g.ordered(), vec![d, a, c]);
}

// ── VT-3: refinement (F11) — a surviving later-layer edge refines an earlier ──
// incomparability; it is never violated.

#[test]
fn refinement_surviving_later_edge_orders_incomparable_node() {
    let mut b = GraphBuilder::new();
    let o0 = b.overlay(reject_unbounded());
    let o1 = b.overlay(reject_unbounded());
    let a = b.node();
    let c = b.node();
    let d = b.node();
    b.edge(o0, a, c, attrs()); // layer 0: a → c (d incomparable)
    b.edge(o1, c, d, attrs()); // layer 1: c → d (refines d's position)
    b.order_spec(OrderSpec::new(vec![
        OrderLayer::new(o0, Direction::Along),
        OrderLayer::new(o1, Direction::Along),
    ]));
    let g = b.build().expect("valid");

    assert!(g.provenance().evictions().is_empty());
    assert_eq!(g.ordered(), vec![a, c, d]);
}

// ── VT-4: degraded taint (F12) — a reject-SCC plus a clean successor: the ─────
// successor is Degraded, the order is total, no overflow.

#[test]
fn degraded_scc_taints_clean_successor() {
    let mut b = GraphBuilder::new();
    let o0 = b.overlay(reject_unbounded());
    let a = b.node();
    let c = b.node();
    let d = b.node();
    b.edge(o0, a, c, attrs()); // a ↔ c reject cycle
    b.edge(o0, c, a, attrs());
    b.edge(o0, c, d, attrs()); // boundary edge c → d
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(o0, Direction::Along)]));
    let g = b.build().expect("valid");

    // a, c degraded at level 0; d degraded at level 1 (its position depends on
    // the cyclic part). U holds only the boundary edge c → d.
    assert_eq!(g.order_key(a).expect("a").level(), Level::Degraded(0));
    assert_eq!(g.order_key(c).expect("c").level(), Level::Degraded(0));
    assert_eq!(g.order_key(d).expect("d").level(), Level::Degraded(1));
    assert_eq!(g.ordered(), vec![a, c, d]);
}

// ── VT-5: degrade scope (F31) — a cycle in a Reject overlay NOT in the spec ───
// is diagnosed but does not degrade the order it is not part of.

#[test]
fn cycle_outside_order_spec_does_not_degrade() {
    let mut b = GraphBuilder::new();
    let ordered_ov = b.overlay(reject_unbounded());
    let cyclic_ov = b.overlay(reject_unbounded());
    let a = b.node();
    let c = b.node();
    let x = b.node();
    let y = b.node();
    b.edge(ordered_ov, a, c, attrs()); // the ordered overlay: a → c (clean)
    b.edge(cyclic_ov, x, y, attrs()); // a cycle on an overlay NOT in the spec
    b.edge(cyclic_ov, y, x, attrs());
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(
        ordered_ov,
        Direction::Along,
    )]));
    let g = b.build().expect("valid");

    // The cycle is diagnosed (PHASE-02) but nothing is Degraded.
    assert!(!g.provenance().cycles().is_empty());
    for n in [a, c, x, y] {
        assert!(
            matches!(g.order_key(n).expect("node").level(), Level::Finite(_)),
            "node not degraded"
        );
    }
    // Clean spec order: a before c.
    assert_eq!(g.order_key(a).expect("a").level(), Level::Finite(0));
    assert_eq!(g.order_key(c).expect("c").level(), Level::Finite(1));
}

// ── VT-6: taint crossing (F32) — intra-SCC edges are absent from U (so the ────
// two degraded nodes share level 0), but taint crosses the boundary edge.

#[test]
fn intra_scc_edges_absent_from_u_taint_crosses_boundary() {
    let mut b = GraphBuilder::new();
    let o0 = b.overlay(reject_unbounded());
    let a = b.node();
    let c = b.node();
    let d = b.node();
    b.edge(o0, a, c, attrs()); // a ↔ c reject cycle (in the spec)
    b.edge(o0, c, a, attrs());
    b.edge(o0, c, d, attrs()); // boundary: c → d
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(o0, Direction::Along)]));
    let g = b.build().expect("valid");

    // If a → c were in U, c would be level 1. It is level 0 — proof the intra-SCC
    // edge was withheld (asserted via ordering, not internal access).
    assert_eq!(g.order_key(a).expect("a").level(), Level::Degraded(0));
    assert_eq!(g.order_key(c).expect("c").level(), Level::Degraded(0));
    // Taint still crosses the boundary edge to d.
    assert_eq!(g.order_key(d).expect("d").level(), Level::Degraded(1));
}

// ── VT-7: suffix order (F33) — a surviving clean-layer edge between two ───────
// degraded nodes is respected: the suffix orders by U level, not NodeId.

#[test]
fn degraded_suffix_respects_surviving_clean_edge() {
    let mut b = GraphBuilder::new();
    let cyclic_ov = b.overlay(reject_unbounded());
    let clean_ov = b.overlay(evict_unbounded());
    let a = b.node(); // NodeId-lower
    let c = b.node(); // NodeId-higher
    b.edge(cyclic_ov, a, c, attrs()); // a ↔ c reject cycle (layer 0)
    b.edge(cyclic_ov, c, a, attrs());
    b.edge(clean_ov, c, a, attrs()); // layer 1: surviving clean edge c → a
    b.order_spec(OrderSpec::new(vec![
        OrderLayer::new(cyclic_ov, Direction::Along),
        OrderLayer::new(clean_ov, Direction::Along),
    ]));
    let g = b.build().expect("valid");

    // Both degraded; the surviving c → a makes c level 0, a level 1 — so the
    // suffix orders c before a though NodeId(a) < NodeId(c).
    assert_eq!(g.order_key(c).expect("c").level(), Level::Degraded(0));
    assert_eq!(g.order_key(a).expect("a").level(), Level::Degraded(1));
    assert_eq!(g.ordered(), vec![c, a]);
}

// ── VT-8: arity × reject, ordering half (F30/F46) — pass-1 breaks the authored ─
// cycle; the post-arity view is acyclic so nothing degrades and every surviving
// resolved edge is respected.

#[test]
fn arity_broken_cycle_orders_cleanly() {
    let mut b = GraphBuilder::new();
    let o0 = b.overlay(reject_at_most_one());
    let a = b.node();
    let c = b.node();
    let d = b.node();
    b.edge(o0, a, c, EdgeAttrs::new(1, 0)); // a → c rank 1 ┐ arity contest on c
    b.edge(o0, d, c, EdgeAttrs::new(2, 0)); // d → c rank 2 ┘ keeps d → c
    b.edge(o0, c, a, attrs()); // c → a
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(o0, Direction::Along)]));
    let g = b.build().expect("valid");

    // Authored cycle a → c → a is diagnosed (PHASE-02) ...
    assert!(!g.provenance().cycles().is_empty());
    // ... but the post-arity view d → c → a is acyclic: nothing Degraded.
    for n in [a, c, d] {
        assert!(
            matches!(g.order_key(n).expect("node").level(), Level::Finite(_)),
            "node not degraded"
        );
    }
    // Order respects every surviving resolved edge: d → c → a.
    assert_eq!(g.ordered(), vec![d, c, a]);
}

// ── VT-9: determinism (REQ-077), ordering half — identical inputs give ───────
// identical order keys and provenance across builds.

#[test]
fn determinism_identical_order_keys_and_provenance() {
    fn build() -> cordage::Graph {
        let mut b = GraphBuilder::new();
        let x = b.overlay(reject_unbounded());
        let y = b.overlay(reject_unbounded());
        let a = b.node();
        let c = b.node();
        b.edge(x, a, c, attrs()); // earlier layer: a → c
        b.edge(y, c, a, attrs()); // later layer: c → a (loses)
        b.order_spec(OrderSpec::new(vec![
            OrderLayer::new(x, Direction::Along),
            OrderLayer::new(y, Direction::Along),
        ]));
        b.build().expect("valid")
    }

    let g1 = build();
    let g2 = build();
    assert_eq!(g1.provenance(), g2.provenance());
    assert_eq!(g1.ordered(), g2.ordered());
    // Compare keys node-by-node over the shared id space.
    for n in g1.ordered() {
        assert_eq!(g1.order_key(n), g2.order_key(n));
    }
}
