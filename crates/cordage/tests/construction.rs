//! Build-success edge cases (design §5.5): empty graph, single node, empty
//! OrderSpec. Black-box — structural vocabulary only (overlays `a`/`b`), tests
//! never mint a `NodeId`/`OverlayId` directly (opaque, builder-allocated).

use cordage::{Arity, CyclePolicy, EdgeAttrs, GraphBuilder, OverlayConfig};

fn overlay_cfg() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

#[test]
fn empty_graph_builds() {
    let g = GraphBuilder::new().build();
    assert!(g.is_ok(), "empty graph is a valid build");
}

#[test]
fn single_node_no_edges_builds() {
    let mut b = GraphBuilder::new();
    let _n = b.node();
    assert!(b.build().is_ok());
}

#[test]
fn nodes_edges_and_overlay_build() {
    let mut b = GraphBuilder::new();
    let a = b.overlay(overlay_cfg());
    let x = b.node();
    let y = b.node();
    b.edge(a, x, y, EdgeAttrs::new(0, 0));
    assert!(b.build().is_ok());
}
