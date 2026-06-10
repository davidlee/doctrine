//! VT-1 — the build-input rejection suite (design §5.2 F14/F22/F38): each
//! malformed-input class yields a `BuildError` naming the offence; an empty
//! `OrderSpec` is valid. Foreign ids are minted from a sibling builder (ids are
//! opaque; tests never fabricate them).

use cordage::{
    Arity, BuildError, CyclePolicy, Direction, EdgeAttrs, GraphBuilder, OrderLayer, OrderSpec,
    OverlayConfig,
};

fn cfg() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

/// A `NodeId` with ordinal 6, foreign to any builder holding ≤6 nodes.
fn foreign_node() -> cordage::NodeId {
    let mut sib = GraphBuilder::new();
    let mut last = sib.node();
    for _ in 0..6 {
        last = sib.node();
    }
    last
}

#[test]
fn edge_with_unknown_node_is_rejected() {
    let foreign = foreign_node();
    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    let n0 = b.node();
    b.edge(ov, n0, foreign, EdgeAttrs::new(0, 0));
    assert!(matches!(b.build(), Err(BuildError::UnknownNode(_))));
}

#[test]
fn edge_with_unknown_overlay_is_rejected() {
    let mut sib = GraphBuilder::new();
    let foreign_ov = sib.overlay(cfg());
    let mut b = GraphBuilder::new(); // zero overlays
    let n0 = b.node();
    let n1 = b.node();
    b.edge(foreign_ov, n0, n1, EdgeAttrs::new(0, 0));
    assert!(matches!(b.build(), Err(BuildError::UnknownOverlay(_))));
}

#[test]
fn order_layer_with_unknown_overlay_is_rejected() {
    let mut sib = GraphBuilder::new();
    let foreign_ov = sib.overlay(cfg());
    let mut b = GraphBuilder::new(); // zero overlays
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(
        foreign_ov,
        Direction::Along,
    )]));
    assert!(matches!(b.build(), Err(BuildError::UnknownOverlay(_))));
}

#[test]
fn overlay_repeated_in_order_spec_is_rejected() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    b.order_spec(OrderSpec::new(vec![
        OrderLayer::new(ov, Direction::Along),
        OrderLayer::new(ov, Direction::Against),
    ]));
    assert!(matches!(
        b.build(),
        Err(BuildError::OverlayRepeatedInOrderSpec(_))
    ));
}

#[test]
fn direction_none_layer_is_rejected() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(cfg());
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(ov, Direction::None)]));
    assert!(matches!(b.build(), Err(BuildError::DirectionNoneLayer(_))));
}

#[test]
fn overlay_cap_exceeded_is_rejected() {
    let mut b = GraphBuilder::new();
    // u16 holds 65_536 ids (0..=65_535); the 65_537th allocation overflows.
    for _ in 0..=65_536_u32 {
        b.overlay(cfg());
    }
    assert!(matches!(b.build(), Err(BuildError::OverlayCapExceeded)));
}

#[test]
fn empty_order_spec_is_valid() {
    let mut b = GraphBuilder::new();
    b.order_spec(OrderSpec::new(vec![]));
    assert!(b.build().is_ok());
}

#[test]
fn well_formed_order_spec_builds() {
    let mut b = GraphBuilder::new();
    let dep = b.overlay(cfg());
    let seq = b.overlay(cfg());
    b.order_spec(OrderSpec::new(vec![
        OrderLayer::new(dep, Direction::Against),
        OrderLayer::new(seq, Direction::Along),
    ]));
    assert!(b.build().is_ok());
}
