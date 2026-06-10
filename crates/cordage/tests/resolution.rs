//! PHASE-02 — build passes 1–2: arity enforcement and per-overlay cycle
//! resolution. Black-box, vocabulary-free: overlays are `a`/`b`, ids are opaque
//! tokens. The observable surface is the resolved adjacency views
//! (`out_edges`/`in_edges`) plus `provenance()`.

use cordage::{
    Arity, CyclePolicy, EdgeAttrs, EvictReason, GraphBuilder, NodeId, OverlayConfig, OverlayId,
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

/// The `(overlay, nodes, edges)` of each diagnosed cycle — the assertion shape.
/// `nodes` is sorted (a `BTreeSet`); `edges` is the sorted `(src, dst)` list.
fn cycles(g: &cordage::Graph) -> Vec<(OverlayId, Vec<NodeId>, Vec<(NodeId, NodeId)>)> {
    g.provenance()
        .cycles()
        .iter()
        .map(|c| {
            (
                c.overlay(),
                c.nodes().iter().copied().collect(),
                c.edges().iter().map(|e| (e.src(), e.dst())).collect(),
            )
        })
        .collect()
}

/// The `(overlay, src, dst, reason)` of an eviction — the assertion shape.
fn evictions(g: &cordage::Graph) -> Vec<(OverlayId, NodeId, NodeId, EvictReason)> {
    g.provenance()
        .evictions()
        .iter()
        .map(|e| (e.overlay(), e.edge().src(), e.edge().dst(), e.reason()))
        .collect()
}

// ── T1: scaffold — resolution is a no-op on clean inputs ─────────────────────

#[test]
fn empty_graph_has_empty_provenance() {
    let g = GraphBuilder::new().build().expect("valid");
    assert!(g.provenance().cycles().is_empty());
    assert!(g.provenance().evictions().is_empty());
}

#[test]
fn single_acyclic_overlay_resolves_to_authored_with_empty_provenance() {
    // a → b → c on an Unbounded/Reject overlay: no arity contest, no cycle.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_unbounded());
    let n0 = b.node();
    let n1 = b.node();
    let n2 = b.node();
    b.edge(ov, n0, n1, EdgeAttrs::new(0, 0));
    b.edge(ov, n1, n2, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert!(g.provenance().cycles().is_empty());
    assert!(g.provenance().evictions().is_empty());
    // Resolved view == authored.
    let outs: Vec<_> = g.out_edges(ov, n0).map(|(n, _)| n).collect();
    assert_eq!(outs, vec![n1]);
    let outs: Vec<_> = g.out_edges(ov, n1).map(|(n, _)| n).collect();
    assert_eq!(outs, vec![n2]);
}

// ── T3: pass 1 — arity enforcement (F7/F19/F36, VT-3) ────────────────────────

#[test]
fn arity_keeps_highest_rank_parent_evicts_rest() {
    // child c has two parents on an AtMostOne overlay; higher rank wins the keep.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_at_most_one());
    let p_low = b.node();
    let p_high = b.node();
    let c = b.node();
    b.edge(ov, p_low, c, EdgeAttrs::new(1, 0));
    b.edge(ov, p_high, c, EdgeAttrs::new(2, 0)); // higher rank → kept
    let g = b.build().expect("valid");

    // Resolved in-view keeps only the max-key parent.
    let parents: Vec<_> = g.in_edges(ov, c).map(|(n, _)| n).collect();
    assert_eq!(parents, vec![p_high]);
    // The loser is surfaced as an ArityViolation.
    assert_eq!(
        evictions(&g),
        vec![(ov, p_low, c, EvictReason::ArityViolation)]
    );
}

#[test]
fn arity_equal_rank_age_resolves_by_src() {
    // two parents tie on (rank, age); the F17 key breaks it by src — max src kept.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_at_most_one());
    let q0 = b.node();
    let q1 = b.node(); // higher src ordinal
    let c = b.node();
    b.edge(ov, q0, c, EdgeAttrs::new(5, 7));
    b.edge(ov, q1, c, EdgeAttrs::new(5, 7)); // same (rank, age) → src decides
    let g = b.build().expect("valid");

    let parents: Vec<_> = g.in_edges(ov, c).map(|(n, _)| n).collect();
    assert_eq!(parents, vec![q1], "higher src kept on (rank,age) tie");
    assert_eq!(
        evictions(&g),
        vec![(ov, q0, c, EvictReason::ArityViolation)]
    );
}

#[test]
fn arity_untouched_on_unbounded_overlay() {
    // Unbounded overlays carry multi-parent membership — no arity contest.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_unbounded());
    let p0 = b.node();
    let p1 = b.node();
    let c = b.node();
    b.edge(ov, p0, c, EdgeAttrs::new(1, 0));
    b.edge(ov, p1, c, EdgeAttrs::new(2, 0));
    let g = b.build().expect("valid");

    let parents: Vec<_> = g.in_edges(ov, c).map(|(n, _)| n).collect();
    assert_eq!(parents, vec![p0, p1], "both parents survive");
    assert!(g.provenance().evictions().is_empty());
}

// ── T4: pass 2 Reject — diagnose, never linearize (F30/F46, VT-1/4/5) ────────

#[test]
fn reject_cycle_is_diagnosed_not_mutated() {
    // a ↔ b on a Reject overlay: cycle named, edges preserved, build Ok (REQ-076).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_unbounded());
    let a = b.node();
    let bb = b.node();
    b.edge(ov, a, bb, EdgeAttrs::new(0, 0));
    b.edge(ov, bb, a, EdgeAttrs::new(0, 0));
    let g = b.build().expect("cycle is data, not an error");

    assert_eq!(cycles(&g), vec![(ov, vec![a, bb], vec![(a, bb), (bb, a)])]);
    assert!(
        g.provenance().evictions().is_empty(),
        "Reject never evicts cycles"
    );
    // Resolved set is the authored set — cycle intact for the traversal view.
    assert_eq!(
        g.out_edges(ov, a).map(|(n, _)| n).collect::<Vec<_>>(),
        vec![bb]
    );
    assert_eq!(
        g.out_edges(ov, bb).map(|(n, _)| n).collect::<Vec<_>>(),
        vec![a]
    );
}

#[test]
fn arity_breaks_authored_reject_cycle_diagnostic_still_emitted() {
    // F30 example: a→b (rank1), c→b (rank2), b→a on AtMostOne+Reject. Arity keeps
    // c→b, evicts a→b — the post-arity view (c→b→a) is acyclic, yet the AUTHORED
    // cycle {a,b} must still surface as a diagnostic (F46: authored SCC → diag).
    let mut bld = GraphBuilder::new();
    let ov = bld.overlay(reject_at_most_one());
    let a = bld.node();
    let bb = bld.node();
    let c = bld.node();
    bld.edge(ov, a, bb, EdgeAttrs::new(1, 0));
    bld.edge(ov, c, bb, EdgeAttrs::new(2, 0));
    bld.edge(ov, bb, a, EdgeAttrs::new(0, 0));
    let g = bld.build().expect("valid");

    // Authored cycle {a,b} diagnosed.
    assert_eq!(cycles(&g), vec![(ov, vec![a, bb], vec![(a, bb), (bb, a)])]);
    // Arity loser a→b surfaced alongside.
    assert_eq!(
        evictions(&g),
        vec![(ov, a, bb, EvictReason::ArityViolation)]
    );
    // Resolved (post-arity) view: c→b and b→a survive.
    assert_eq!(
        g.in_edges(ov, bb).map(|(n, _)| n).collect::<Vec<_>>(),
        vec![c]
    );
    assert_eq!(
        g.out_edges(ov, bb).map(|(n, _)| n).collect::<Vec<_>>(),
        vec![a]
    );
}

#[test]
fn self_loop_under_reject_is_diagnosed() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_unbounded());
    let n = b.node();
    b.edge(ov, n, n, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert_eq!(cycles(&g), vec![(ov, vec![n], vec![(n, n)])]);
    assert!(g.provenance().evictions().is_empty());
    assert_eq!(
        g.out_edges(ov, n).map(|(x, _)| x).collect::<Vec<_>>(),
        vec![n]
    );
}

// ── T5: pass 2 Evict — resolve to fixpoint by the F17 key (F17/F37, VT-2/5/6) ─

#[test]
fn evict_cycle_drops_min_key_edge_to_fixpoint() {
    // a ↔ b on an Evict overlay: a→b (rank0) is the eviction-key min → dropped.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(evict_unbounded());
    let a = b.node();
    let bb = b.node();
    b.edge(ov, a, bb, EdgeAttrs::new(0, 0));
    b.edge(ov, bb, a, EdgeAttrs::new(1, 0));
    let g = b.build().expect("valid");

    assert!(
        g.provenance().cycles().is_empty(),
        "Evict resolves, never diagnoses"
    );
    assert_eq!(
        evictions(&g),
        vec![(ov, a, bb, EvictReason::IntraOverlayCycle)]
    );
    assert!(g.out_edges(ov, a).next().is_none(), "min-key edge gone");
    assert_eq!(
        g.out_edges(ov, bb).map(|(n, _)| n).collect::<Vec<_>>(),
        vec![a]
    );
}

#[test]
fn evict_disjoint_cycles_each_lose_their_own_min() {
    // Two independent 2-cycles; each loses its own eviction-key-minimal edge.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(evict_unbounded());
    let a = b.node();
    let bb = b.node();
    let c = b.node();
    let d = b.node();
    b.edge(ov, a, bb, EdgeAttrs::new(0, 0)); // cycle 1 min
    b.edge(ov, bb, a, EdgeAttrs::new(5, 0));
    b.edge(ov, c, d, EdgeAttrs::new(2, 0)); // cycle 2 min
    b.edge(ov, d, c, EdgeAttrs::new(3, 0));
    let g = b.build().expect("valid");

    assert_eq!(
        evictions(&g),
        vec![
            (ov, a, bb, EvictReason::IntraOverlayCycle),
            (ov, c, d, EvictReason::IntraOverlayCycle),
        ]
    );
}

#[test]
fn self_loop_under_evict_is_dropped() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(evict_unbounded());
    let n = b.node();
    b.edge(ov, n, n, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert!(g.provenance().cycles().is_empty());
    assert_eq!(
        evictions(&g),
        vec![(ov, n, n, EvictReason::IntraOverlayCycle)]
    );
    assert!(g.out_edges(ov, n).next().is_none(), "self-loop dropped");
}

#[test]
fn evict_selects_by_eviction_key_not_adjacency_order() {
    // F37: a→b (rank1), b→a (rank9). Eviction-key min = a→b (rank1). A buggy impl
    // ordering by the adjacency key (dst, rank, age) would pick b→a (smaller dst).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(evict_unbounded());
    let a = b.node();
    let bb = b.node();
    b.edge(ov, a, bb, EdgeAttrs::new(1, 0));
    b.edge(ov, bb, a, EdgeAttrs::new(9, 0));
    let g = b.build().expect("valid");

    assert_eq!(
        evictions(&g),
        vec![(ov, a, bb, EvictReason::IntraOverlayCycle)],
        "the (rank,age,src,dst)-minimal edge is evicted, not the adjacency-min"
    );
    assert_eq!(
        g.out_edges(ov, bb).map(|(n, _)| n).collect::<Vec<_>>(),
        vec![a]
    );
}

// ── T6: determinism + multi-node SCCs ────────────────────────────────────────

#[test]
fn reject_multi_node_scc_with_tail_diagnoses_only_the_cycle() {
    // a→b→c→a (3-cycle) plus c→d (tail). The cyclic component is {a,b,c}; d is
    // reachable from it but not part of it.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_unbounded());
    let a = b.node();
    let bb = b.node();
    let c = b.node();
    let d = b.node();
    b.edge(ov, a, bb, EdgeAttrs::new(0, 0));
    b.edge(ov, bb, c, EdgeAttrs::new(0, 0));
    b.edge(ov, c, a, EdgeAttrs::new(0, 0));
    b.edge(ov, c, d, EdgeAttrs::new(0, 0)); // tail out of the SCC
    let g = b.build().expect("valid");

    assert_eq!(
        cycles(&g),
        vec![(ov, vec![a, bb, c], vec![(a, bb), (bb, c), (c, a)])]
    );
    assert!(g.provenance().evictions().is_empty());
}

/// Build a fixture (arity contest + an Evict 3-cycle) with edges inserted in a
/// caller-chosen order; returns the resolved provenance shape.
fn build_permuted(
    reverse: bool,
) -> (
    Vec<(OverlayId, NodeId, NodeId, EvictReason)>,
    Vec<(OverlayId, Vec<NodeId>, Vec<(NodeId, NodeId)>)>,
) {
    let mut b = GraphBuilder::new();
    let am = b.overlay(reject_at_most_one()); // arity overlay
    let ev = b.overlay(evict_unbounded()); // evict-cycle overlay
    let n0 = b.node();
    let n1 = b.node();
    let n2 = b.node();
    let n3 = b.node();

    // Edge insertions, listed in canonical order.
    type Ins = (OverlayId, NodeId, NodeId, EdgeAttrs);
    let mut ins: Vec<Ins> = vec![
        (am, n0, n2, EdgeAttrs::new(1, 0)), // arity loser
        (am, n1, n2, EdgeAttrs::new(2, 0)), // arity winner
        (ev, n0, n1, EdgeAttrs::new(3, 0)), // 3-cycle on ev
        (ev, n1, n3, EdgeAttrs::new(1, 0)), // eviction-key min
        (ev, n3, n0, EdgeAttrs::new(2, 0)),
    ];
    if reverse {
        ins.reverse();
    }
    for (ov, s, d, attrs) in ins {
        b.edge(ov, s, d, attrs);
    }
    let g = b.build().expect("valid");
    (evictions(&g), cycles(&g))
}

#[test]
fn provenance_is_insertion_order_independent() {
    // Same graph, edges inserted forwards vs reversed → byte-identical provenance
    // (partial REQ-077; full permutation property test is PHASE-05).
    assert_eq!(build_permuted(false), build_permuted(true));
}
