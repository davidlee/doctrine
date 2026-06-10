//! VT-1..7 (channel facets) — `evaluate`'s per-combinator-class fold-set
//! contract (F34): idempotent combinators fold present seeds over `{n} ∪
//! reachable`; `CountDistinct` counts `Flag(true)` seeds over STRICT reachable.
//! Plus the seed contract (F16/F41/F45), per-combinator `Direction::None` (F35),
//! contributors (F21/F43), and `OrderSpec`-invariance of values (I7/F18).
//! Black-box: opaque builder ids, overlay-neutral vocabulary, test-supplied age.

use cordage::{
    Arity, ChannelDiagReason, ChannelSpec, ChannelValue, Combinator, CyclePolicy, Direction,
    EdgeAttrs, GraphBuilder, NodeId, OrderLayer, OrderSpec, OverlayConfig, ValueKind,
};
use std::collections::BTreeMap;

fn unbounded() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

fn seeds<const N: usize>(entries: [(NodeId, ChannelValue); N]) -> BTreeMap<NodeId, ChannelValue> {
    entries.into_iter().collect()
}

fn nodes<const N: usize>(ids: [NodeId; N]) -> std::collections::BTreeSet<NodeId> {
    ids.into_iter().collect()
}

// ── VT-1 channel half: DD1 rollup (F5) ───────────────────────────────────────

#[test]
fn vt1_against_aggregates_from_both_parents_and_counts_diamond_once() {
    // Diamond: g → p1 → n, g → p2 → n (Unbounded). Against from n reaches both
    // parents and the shared grandparent (counted ONCE — R3).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let g = b.node();
    let p1 = b.node();
    let p2 = b.node();
    let n = b.node();
    b.edge(ov, g, p1, EdgeAttrs::new(0, 0));
    b.edge(ov, g, p2, EdgeAttrs::new(0, 0));
    b.edge(ov, p1, n, EdgeAttrs::new(0, 0));
    b.edge(ov, p2, n, EdgeAttrs::new(0, 0));
    let graph = b.build().expect("valid");

    // All Against: both parents true → n is true, contributors = both parents.
    let all = graph.evaluate(
        ChannelSpec::new(ov, Combinator::All, Direction::Against),
        &seeds([
            (p1, ChannelValue::Flag(true)),
            (p2, ChannelValue::Flag(true)),
        ]),
    );
    assert_eq!(all.value(n), Some(ChannelValue::Flag(true)));
    assert_eq!(*all.contributors(n), nodes([p1, p2]));

    // CountDistinct Against from a single seeded grandparent reachable two ways →
    // counted once.
    let cd = graph.evaluate(
        ChannelSpec::new(ov, Combinator::CountDistinct, Direction::Against),
        &seeds([(g, ChannelValue::Flag(true))]),
    );
    assert_eq!(cd.value(n), Some(ChannelValue::Count(1)));
    assert_eq!(*cd.contributors(n), nodes([g]));
}

// ── VT-2: CountDistinct set (F34/F45) ─────────────────────────────────────────

#[test]
fn vt2_countdistinct_strict_differs_per_scc_member() {
    // a ↔ b on a Reject overlay (cycle survives, diagnosed). Seed a only.
    // STRICT reachable: a→{b} (b unseeded → a absent); b→{a} (a seeded → Count 1).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let na = b.node();
    let nb = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, na, EdgeAttrs::new(0, 0));
    let graph = b.build().expect("valid");

    let cd = graph.evaluate(
        ChannelSpec::new(ov, Combinator::CountDistinct, Direction::Along),
        &seeds([(na, ChannelValue::Flag(true))]),
    );
    assert_eq!(
        cd.value(na),
        None,
        "a's strict reach {{b}} has no seed → absent"
    );
    assert_eq!(
        cd.value(nb),
        Some(ChannelValue::Count(1)),
        "b reaches seeded a"
    );
}

#[test]
fn vt2_all_false_fold_set_is_count_zero_not_absent() {
    // x → y, y seeded Flag(false). x's strict reach {y} has a PRESENT seed (false)
    // → Count(0), real data, distinct from absence.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let x = b.node();
    let y = b.node();
    b.edge(ov, x, y, EdgeAttrs::new(0, 0));
    let graph = b.build().expect("valid");

    let cd = graph.evaluate(
        ChannelSpec::new(ov, Combinator::CountDistinct, Direction::Along),
        &seeds([(y, ChannelValue::Flag(false))]),
    );
    assert_eq!(
        cd.value(x),
        Some(ChannelValue::Count(0)),
        "present-false ≠ absent"
    );
    assert_eq!(cd.value(y), None, "y strict reach is empty → absent");
}

// ── VT-3: None per combinator (F35) ───────────────────────────────────────────

#[test]
fn vt3_direction_none_per_combinator() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let n = b.node();
    let graph = b.build().expect("valid");

    // Max + None → own present seed.
    let mx = graph.evaluate(
        ChannelSpec::new(ov, Combinator::Max, Direction::None),
        &seeds([(n, ChannelValue::Scalar(7))]),
    );
    assert_eq!(mx.value(n), Some(ChannelValue::Scalar(7)));

    // CountDistinct + None → always absent (strict fold set is empty).
    let cd = graph.evaluate(
        ChannelSpec::new(ov, Combinator::CountDistinct, Direction::None),
        &seeds([(n, ChannelValue::Flag(true))]),
    );
    assert_eq!(cd.value(n), None);
}

// ── VT-4: foreign seed (F41) ──────────────────────────────────────────────────

#[test]
fn vt4_unknown_seed_node_wins_over_variant_mismatch() {
    // Foreign node minted from a sibling builder; seeded with a variant that is
    // ALSO wrong for the combinator (Any wants Flag, given Scalar). The diagnostic
    // must be UnknownSeedNode, not SeedVariantMismatch — and the seed is ignored.
    let mut sib = GraphBuilder::new();
    let mut foreign = sib.node();
    for _ in 0..8 {
        foreign = sib.node();
    }

    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let n = b.node();
    let graph = b.build().expect("valid");

    let ch = graph.evaluate(
        ChannelSpec::new(ov, Combinator::Any, Direction::Along),
        &seeds([
            (n, ChannelValue::Flag(true)),
            (foreign, ChannelValue::Scalar(5)),
        ]),
    );
    assert_eq!(
        ch.value(n),
        Some(ChannelValue::Flag(true)),
        "valid seed still folds"
    );
    assert_eq!(ch.diagnostics().len(), 1, "exactly one diagnostic");
    assert_eq!(ch.diagnostics()[0].node(), foreign);
    assert_eq!(
        ch.diagnostics()[0].reason(),
        ChannelDiagReason::UnknownSeedNode
    );
}

// ── VT-5: seed contract (F16) ─────────────────────────────────────────────────

#[test]
fn vt5_scalar_min_is_data_not_absence_and_no_identity_escapes() {
    // Max over a/b/c. a seeded Scalar(i64::MIN) (a legitimate extreme), b unseeded,
    // c seeded Flag(true) (variant mismatch for Max). Expect: a emits i64::MIN; b
    // absent (no fabricated identity); c diagnosed + absent.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let a = b.node();
    let bn = b.node();
    let c = b.node();
    let graph = b.build().expect("valid");

    let ch = graph.evaluate(
        ChannelSpec::new(ov, Combinator::Max, Direction::None),
        &seeds([
            (a, ChannelValue::Scalar(i64::MIN)),
            (c, ChannelValue::Flag(true)),
        ]),
    );
    assert_eq!(
        ch.value(a),
        Some(ChannelValue::Scalar(i64::MIN)),
        "extreme is real data"
    );
    assert_eq!(
        ch.value(bn),
        None,
        "unseeded → absent, never a fabricated identity"
    );
    assert_eq!(ch.value(c), None, "variant mismatch → treated absent");
    assert_eq!(ch.values().len(), 1, "only the one real value escapes");
    assert_eq!(ch.diagnostics().len(), 1);
    assert_eq!(
        ch.diagnostics()[0].reason(),
        ChannelDiagReason::SeedVariantMismatch {
            expected: ValueKind::Scalar,
            actual: ValueKind::Flag
        }
    );
}

// ── VT-7: ties (F21) ──────────────────────────────────────────────────────────

#[test]
fn vt7_equal_max_seeds_resolve_to_min_nodeid_argmax() {
    // t → p, t → q; p and q both seeded Scalar(10). Max from t = 10, argmax is the
    // min-NodeId maximal node (p, allocated first).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let t = b.node();
    let p = b.node();
    let q = b.node();
    b.edge(ov, t, p, EdgeAttrs::new(0, 0));
    b.edge(ov, t, q, EdgeAttrs::new(0, 0));
    let graph = b.build().expect("valid");

    let mx = graph.evaluate(
        ChannelSpec::new(ov, Combinator::Max, Direction::Along),
        &seeds([(p, ChannelValue::Scalar(10)), (q, ChannelValue::Scalar(10))]),
    );
    assert_eq!(mx.value(t), Some(ChannelValue::Scalar(10)));
    assert_eq!(
        *mx.contributors(t),
        nodes([p]),
        "min-NodeId argmax tie-break"
    );
}

// ── VT-8: REQ-080 seam ────────────────────────────────────────────────────────

#[test]
fn vt8_a_fresh_channel_needs_no_core_change() {
    // Two DISTINCT channel meanings over the SAME graph, composed purely by
    // choosing (combinator, direction) — no new enum variant, no core edit. One:
    // a backward "is any ancestor flagged?" reachability channel (Any/Against).
    // Two: a forward "max priority among me and my dependents" (Max/Along). Both
    // are fresh channels expressed through the existing curated combinator set.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(unbounded());
    let root = b.node();
    let mid = b.node();
    let leaf = b.node();
    b.edge(ov, root, mid, EdgeAttrs::new(0, 0));
    b.edge(ov, mid, leaf, EdgeAttrs::new(0, 0));
    let graph = b.build().expect("valid");

    // Fresh channel A — "flagged ancestor?" seeded at the root, read at the leaf.
    let flagged = graph.evaluate(
        ChannelSpec::new(ov, Combinator::Any, Direction::Against),
        &seeds([(root, ChannelValue::Flag(true))]),
    );
    assert_eq!(flagged.value(leaf), Some(ChannelValue::Flag(true)));

    // Fresh channel B — "max priority over self+dependents" seeded at the leaf,
    // read at the root. Different domain (Scalar), different direction, same core.
    let priority = graph.evaluate(
        ChannelSpec::new(ov, Combinator::Max, Direction::Along),
        &seeds([(leaf, ChannelValue::Scalar(9))]),
    );
    assert_eq!(priority.value(root), Some(ChannelValue::Scalar(9)));
}

// ── VT-6: eviction scope (I7/F18) ─────────────────────────────────────────────

#[test]
fn vt6_channel_invariant_under_orderspec_eviction() {
    // Build the SAME graph twice — once with no OrderSpec, once with an OrderSpec
    // whose layers force a cross-layer (pass-3) eviction on `U`. Channel values
    // read the per-overlay adjacency, untouched by U eviction → byte-identical.
    // The channel spec is captured from the builder (ids are opaque) — ov0/seed
    // ordinals are identical across the two builds.
    fn build(with_spec: bool) -> (cordage::Graph, ChannelSpec, BTreeMap<NodeId, ChannelValue>) {
        let mut b = GraphBuilder::new();
        let ov0 = b.overlay(unbounded());
        let ov1 = b.overlay(unbounded());
        let a = b.node();
        let bn = b.node();
        // Layer-0 edge a→b; layer-1 edge b→a — together a cross-layer cycle that
        // pass-3 resolves by evicting from U (not from the overlay edge sets).
        b.edge(ov0, a, bn, EdgeAttrs::new(0, 0));
        b.edge(ov1, bn, a, EdgeAttrs::new(0, 0));
        if with_spec {
            b.order_spec(OrderSpec::new(vec![
                OrderLayer::new(ov0, Direction::Along),
                OrderLayer::new(ov1, Direction::Along),
            ]));
        }
        let spec = ChannelSpec::new(ov0, Combinator::Any, Direction::Along);
        let s = seeds([
            (a, ChannelValue::Flag(true)),
            (bn, ChannelValue::Flag(false)),
        ]);
        (b.build().expect("valid"), spec, s)
    }

    let (no_spec, spec, s) = build(false);
    let (with_spec, _, _) = build(true);
    // Sanity: the spec build actually evicted something from U.
    assert!(
        !with_spec.provenance().evictions().is_empty(),
        "the OrderSpec build must exercise a U eviction"
    );

    // Channel over ov0 must be identical regardless of the OrderSpec.
    assert_eq!(no_spec.evaluate(spec, &s), with_spec.evaluate(spec, &s));
}
