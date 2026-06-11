//! SL-038 / SL-043 — scale gates for the confirmed cordage scale cliffs, all
//! reachable inside the ~tens-of-thousands target.
//!
//! SL-043 PHASE-01 fixed the build-time resolve.rs defects and PHASE-02 fixed the
//! query-time `evaluate` cliff, so the overflow, eviction-locality, and evaluate
//! gates here now assert the FIX (build succeeds / eviction linear / evaluate
//! near-linear), not the cliff:
//!
//! - RSK-003 overflow: `deep_chain(80k)` now BUILDS Ok — the iterative
//!   Tarjan/`level_of` no longer overflow the native stack (gate, not `#[ignore]`).
//! - SL-043 eviction locality: N independent small cycles evict in ~linear time,
//!   and the evicted SET is identical to the pre-fix global loop (set-identity).
//! - RSK-004 evaluate: the per-node `reachable` BFS is replaced by ONE
//!   condensation fold per call → near-linear over a deep_chain spine (gate, not
//!   `#[ignore]`).
//! - RSK-002 explain: the recursive `2^layers` chain enumeration is replaced by
//!   ONE cone builder → the predecessor sub-DAG is LINEAR in diamond depth (gate,
//!   not `#[ignore]`).
//!
//! Still `#[ignore]`d as deferred:
//! - EXC-2 dense_evict superlinearity: a single dense cycle's fixpoint stays
//!   superlinear — deferred residual, NOT fixable in scope (linearizing it would
//!   change the evicted set).
//!
//! std-only, public-API-only, zero-dep. Generators are duplicated inline (D4 —
//! `examples/` and `tests/` cannot import each other); the canonical copy lives in
//! `examples/scale_harness.rs`. Follows the existing-test convention
//! (`expect`/`unwrap`, short names) — `tests/` is not clippy-gated (design §8).

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::time::{Duration, Instant};

use cordage::{
    Arity, ChannelSpec, ChannelValue, Combinator, CyclePolicy, Direction, EdgeAttrs, EvictReason,
    Graph, GraphBuilder, NodeId, OrderLayer, OrderSpec, OverlayConfig, OverlayId,
};

type Built<T> = Result<T, Box<dyn Error>>;

/// `BuildError` does not implement `std::error::Error`, so `?` cannot widen it —
/// render it through `Debug` into the boxed error.
fn built(r: Result<Graph, cordage::BuildError>) -> Built<Graph> {
    r.map_err(|e| format!("build failed: {e:?}").into())
}

/// Linear spine `0→1→…→(n-1)` on one `AtMostOne` overlay carried by a single
/// `OrderLayer`, so the build runs `level_of` / Tarjan to recursion depth `n` (the
/// overflow cliff). Re-used at a sub-overflow `n` for the evaluate cliff: each
/// per-node `reachable` walks the remaining suffix, so the call is O(n²). Returns
/// the graph, the spine overlay, and the head node (the evaluate seed).
fn deep_chain(n: u32) -> Built<(Graph, OverlayId, NodeId)> {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::AtMostOne));
    if n == 0 {
        return Err("deep_chain requires n >= 1".into());
    }
    let head = b.node();
    let mut prev = head;
    for _ in 1..n {
        let node = b.node();
        b.edge(ov, prev, node, EdgeAttrs::new(0, 0));
        prev = node;
    }
    b.order_spec(OrderSpec::new(vec![OrderLayer::new(ov, Direction::Along)]));
    Ok((built(b.build())?, ov, head))
}

/// `layers` diamond stages on one `Unbounded` overlay; each stage splits to two
/// then rejoins, so source→sink has exactly `2^layers` predecessor paths
/// (acyclic). Returns the graph, source, sink, and overlay.
fn diamond(layers: u32) -> Built<(Graph, NodeId, NodeId, OverlayId)> {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
    let source = b.node();
    let mut src = source;
    for _ in 0..layers {
        let left = b.node();
        let right = b.node();
        let join = b.node();
        b.edge(ov, src, left, EdgeAttrs::new(0, 0));
        b.edge(ov, src, right, EdgeAttrs::new(0, 0));
        b.edge(ov, left, join, EdgeAttrs::new(0, 0));
        b.edge(ov, right, join, EdgeAttrs::new(0, 0));
        src = join;
    }
    Ok((built(b.build())?, source, src, ov))
}

/// One `Evict` overlay carrying a near-complete dense cycle over `nodes`: each
/// vertex points at the next `min(edges_per_node, nodes-1)` vertices (mod `nodes`,
/// self excluded), forcing the eviction-to-fixpoint pass to recompute SCCs per
/// evicted edge (the RSK-003 quadratic). `nodes==0` yields an empty graph.
fn dense_evict(nodes: u32, edges_per_node: u32) -> Built<Graph> {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded));
    let ids: Vec<NodeId> = (0..nodes).map(|_| b.node()).collect();
    let count = ids.len();
    let reach = usize::try_from(edges_per_node)?.min(count.saturating_sub(1));
    for (i, &src) in ids.iter().enumerate() {
        for off in 1..=reach {
            let dst = *ids
                .get((i + off) % count)
                .ok_or("dense_evict index range")?;
            b.edge(ov, src, dst, EdgeAttrs::new(0, 0));
        }
    }
    built(b.build())
}

/// Wall-clock the closure (debug-build timing — bounds budget for ~10× the release
/// probe numbers, `mem.pattern.testing.debug-vs-release-scale-timing`).
fn time<T>(f: impl FnOnce() -> T) -> Duration {
    let start = Instant::now();
    let _ = f();
    start.elapsed()
}

// ── 6.1 explain — cone is LINEAR, deterministic (RSK-002 retired) ─────────────

#[test]
fn explain_cone_is_linear_in_diamond_depth() {
    // The old recursive chain enumeration was 2^layers on a diamond. The cone
    // builder records each node once (global visited set), so the cone is the
    // predecessor sub-DAG: its node count is LINEAR in `layers`, far below the
    // 2^layers paths the enumeration produced. A fast gate (no #[ignore]).
    let layers = 18; // 2^18 = 262_144 — the old path explosion; the cone is ~55.
    let (g, _src, sink, ov) = diamond(layers).expect("diamond build");
    let ex = g.explain(sink);
    let cone_nodes = ex.predecessors().get(&ov).map_or(0, BTreeMap::len);

    // Every diamond node is an ancestor of the sink: 1 source + 3 per stage
    // (left, right, join). The cone keys exactly that set.
    let expected = 3 * usize::try_from(layers).expect("layers fits usize") + 1;
    assert_eq!(
        cone_nodes, expected,
        "cone keys every ancestor of the sink, once"
    );
    assert!(
        cone_nodes < (1usize << layers),
        "cone ({cone_nodes}) is far below 2^layers ({})",
        1usize << layers
    );
}

// ── 6.2 overflow — FIXED: deep_chain(80k) now builds (SL-043 PHASE-01) ────────
// Was a self-re-exec subprocess asserting the build SIGABRTs (rc 134). After the
// iterative Tarjan + iterative `level_of` rewrite, the build succeeds at 80k in
// the native stack. `deep_chain` is a clean acyclic chain on a Reject overlay
// carried by one OrderLayer, so the build runs BOTH overflow sites: Tarjan over
// the spine (cycle pass) AND `level_of`'s longest-path over the 80k-long preds
// chain (pass 4) — this single fixture exercises both independent cliffs.

#[test]
fn deep_chain_builds_inside_target_scale() {
    let (g, ov, head) = deep_chain(80_000).expect("deep_chain(80k) must build post-fix");
    // Sanity: the spine head exists in the resolved graph and the order is total
    // over all 80k nodes (level_of finalised every node — no overflow, no gap).
    assert!(
        g.out_edges(ov, head).next().is_some(),
        "head has a successor"
    );
    assert_eq!(g.ordered().len(), 80_000, "every node ordered");
}

// ── 6.3 eviction locality — FIXED: N small cycles evict linearly + set-identity ─
// SL-043 PHASE-01 localized pass2_evict to vertex-disjoint components. This gate
// asserts (a) the evicted SET is byte-identical to the pre-fix global loop on a
// small deterministic fixture, and (b) eviction over many independent small
// cycles is ~linear in N (loose debug bound).

/// N independent 2-cycles on one Evict overlay: cycle i is `x_i → y_i` (rank 0)
/// and `y_i → x_i` (rank 1). The F17-min participant of each cycle is the rank-0
/// `x_i → y_i` edge, so BOTH the pre-fix global loop and the localized loop evict
/// exactly `{ x_i → y_i }`. Returns the graph and the (src, dst) of each evicted
/// edge as the global loop would have produced them (the set-identity oracle).
fn many_small_cycles(n: u32) -> Built<(Graph, Vec<(NodeId, NodeId)>)> {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded));
    let mut expected_evicted: Vec<(NodeId, NodeId)> = Vec::new();
    for _ in 0..n {
        let x = b.node();
        let y = b.node();
        b.edge(ov, x, y, EdgeAttrs::new(0, 0)); // F17-min → evicted
        b.edge(ov, y, x, EdgeAttrs::new(1, 0)); // survives
        expected_evicted.push((x, y));
    }
    Ok((built(b.build())?, expected_evicted))
}

/// The `(src, dst)` of each `IntraOverlayCycle` eviction, as a set.
fn intra_cycle_evicted(g: &Graph) -> BTreeSet<(NodeId, NodeId)> {
    g.provenance()
        .evictions()
        .iter()
        .filter(|e| e.reason() == EvictReason::IntraOverlayCycle)
        .map(|e| (e.edge().src(), e.edge().dst()))
        .collect()
}

#[test]
fn many_small_cycles_evict_set_identical_to_global_loop() {
    // Small deterministic fixture: 4 disjoint 2-cycles → exactly the 4 rank-0
    // edges evicted, the same set the pre-fix global "drop global-min, re-Tarjan
    // all" loop produces (disjointness ⇒ identical set, design T3).
    let (g, expected) = many_small_cycles(4).expect("build 4 cycles");
    let got = intra_cycle_evicted(&g);
    let want: BTreeSet<(NodeId, NodeId)> = expected.into_iter().collect();
    assert_eq!(got, want, "localized eviction set == global-loop set");
}

#[test]
fn many_small_cycles_evict_in_linear_time() {
    // Eviction over N disjoint cycles is ~linear in N: doubling N should NOT
    // blow a coarse debug bound (the quadratic global loop would). Loose — a
    // gate against a regression to O(N²), not a tight perf assertion.
    let t = time(|| many_small_cycles(20_000).expect("build 20k cycles"));
    assert!(
        t < Duration::from_secs(60),
        "20k disjoint cycles evicted in {t:?} (linear-eviction gate)"
    );
}

// ── 6.4 dense_evict — EXC-2 deferred residual (NOT fixable in scope) ──────────

#[test]
#[ignore = "deferred residual (EXC-2): a single dense cycle's fixpoint stays \
            superlinear; linearizing it would change the evicted set"]
fn eviction_fixpoint_scales_superlinearly() {
    // PHASE-01 debug-pinned (50,100): 2.2s / 41s, ratio 18.5× — NOT (100,200),
    // which would blow the 120s bound in debug (mem.debug-vs-release-scale-timing).
    let t1 = time(|| dense_evict(50, 50).expect("dense_evict 50"));
    let t2 = time(|| dense_evict(100, 100).expect("dense_evict 100"));
    eprintln!(
        "eviction ratio {:.1}x for 4x edges",
        t2.as_secs_f64() / t1.as_secs_f64()
    );
    assert!(t2 < Duration::from_secs(120)); // sanity, not a tight gate
}

// ── 6.4 evaluate — FIXED: condensation fold scales ~linearly (RSK-004) ────────
// SL-043 PHASE-02 replaced the per-node `reachable` BFS in `evaluate` with ONE
// direction-resolved condensation fold per call (O(V+E)). The cliff is inverted:
// this is now a near-linear GATE, not an `#[ignore]`'d quadratic recording. The
// pre-fix per-node BFS over a deep_chain spine was O(V²) — at 20k nodes that is
// ~4·10^8 frontier steps, blowing any coarse debug bound; the fold finishes in
// milliseconds. Doubling N must stay well under the quadratic 4× (debug ~10×
// budget, mem.pattern.testing.debug-vs-release-scale-timing).

#[test]
fn evaluate_scales_near_linearly_in_node_count() {
    // The deep_chain spine is the worst case for the old BFS (each node walked the
    // whole remaining suffix). Seed the builder-returned head (opaque id — never
    // NodeId(0)); Any's seed domain is Flag, so Flag(true) is in-domain.
    fn eval_chain(n: u32) -> Duration {
        let (g, ov, head) = deep_chain(n).expect("deep_chain builds (post-PHASE-01)");
        let seeds = BTreeMap::from([(head, ChannelValue::Flag(true))]);
        time(|| {
            g.evaluate(
                ChannelSpec::new(ov, Combinator::Any, Direction::Along),
                &seeds,
            )
        })
    }

    // Absolute coarse gate: a 20k-spine evaluate that was O(V²) cannot finish here.
    let t_big = eval_chain(20_000);
    assert!(
        t_big < Duration::from_secs(30),
        "20k-spine evaluate took {t_big:?} — a quadratic per-node BFS would not finish (near-linear gate)"
    );

    // Ratio sanity: doubling N stays well under the quadratic 4× (loose for debug
    // noise on sub-second timings — a regression to O(V²) would clear 4×).
    let t1 = eval_chain(8_000);
    let t2 = eval_chain(16_000);
    let ratio = t2.as_secs_f64() / t1.as_secs_f64().max(1e-6);
    eprintln!("evaluate ratio {ratio:.1}x for 2x nodes (near-linear; quadratic ≈ 4x)");
    assert!(
        ratio < 3.0,
        "evaluate doubled to {ratio:.1}x for 2x nodes — near-quadratic regression (expect ~2x)"
    );
}
