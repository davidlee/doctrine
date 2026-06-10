//! VT-2/VT-3 — adjacency views: foreign-id queries are empty and non-panicking;
//! duplicate identical edges dedupe by set semantics; iteration follows the
//! explicit adjacency keys (out by `(dst, rank, age)`, in by `(src, rank, age)`).

use cordage::{Arity, CyclePolicy, EdgeAttrs, GraphBuilder, OverlayConfig};

fn cfg() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

#[test]
fn foreign_node_and_overlay_queries_are_empty() {
    // Sibling graph donates a high-ordinal node + an overlay foreign to `g`.
    let mut sib = GraphBuilder::new();
    let foreign_ov = sib.overlay(cfg());
    let mut foreign_node = sib.node();
    for _ in 0..4 {
        foreign_node = sib.node();
    }

    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    let n = b.node();
    let g = b.build().expect("valid");

    // Foreign node on a known overlay.
    assert!(g.out_edges(ov, foreign_node).next().is_none());
    assert!(g.in_edges(ov, foreign_node).next().is_none());
    // Foreign overlay on a known node.
    assert!(g.out_edges(foreign_ov, n).next().is_none());
    assert!(g.in_edges(foreign_ov, n).next().is_none());
    // Known node with no edges.
    assert!(g.out_edges(ov, n).next().is_none());
}

#[test]
fn duplicate_identical_edge_dedupes() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    let s = b.node();
    let d = b.node();
    b.edge(ov, s, d, EdgeAttrs::new(0, 0));
    b.edge(ov, s, d, EdgeAttrs::new(0, 0)); // identical
    let g = b.build().expect("valid");

    let outs: Vec<_> = g.out_edges(ov, s).collect();
    assert_eq!(outs.len(), 1, "identical edges collapse to one");
    assert_eq!(outs[0].0, d);
}

#[test]
fn parallel_edges_differing_in_attrs_are_distinct() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    let s = b.node();
    let d = b.node();
    b.edge(ov, s, d, EdgeAttrs::new(1, 0));
    b.edge(ov, s, d, EdgeAttrs::new(0, 0)); // same endpoints, lower rank
    let g = b.build().expect("valid");

    let ranks: Vec<i32> = g.out_edges(ov, s).map(|(_, a)| a.rank()).collect();
    // Same dst → ordered by rank: 0 then 1.
    assert_eq!(ranks, vec![0, 1]);
}

#[test]
fn out_edges_ordered_by_dst() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    let s = b.node();
    let d0 = b.node();
    let d1 = b.node();
    let d2 = b.node();
    // Insert out of dst order.
    b.edge(ov, s, d2, EdgeAttrs::new(0, 0));
    b.edge(ov, s, d0, EdgeAttrs::new(0, 0));
    b.edge(ov, s, d1, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let dsts: Vec<_> = g.out_edges(ov, s).map(|(n, _)| n).collect();
    assert_eq!(dsts, vec![d0, d1, d2]);
}

#[test]
fn in_edges_ordered_by_src() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    let s0 = b.node();
    let s1 = b.node();
    let s2 = b.node();
    let d = b.node();
    // Insert out of src order.
    b.edge(ov, s2, d, EdgeAttrs::new(0, 0));
    b.edge(ov, s0, d, EdgeAttrs::new(0, 0));
    b.edge(ov, s1, d, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let srcs: Vec<_> = g.in_edges(ov, d).map(|(n, _)| n).collect();
    assert_eq!(srcs, vec![s0, s1, s2]);
}
