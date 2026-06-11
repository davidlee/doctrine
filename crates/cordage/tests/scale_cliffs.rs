//! SL-038 — durable red / characterization tests for the four confirmed cordage
//! scale cliffs, all reachable inside the ~tens-of-thousands target. Each red is
//! `#[ignore]`d (off the default gate) and encodes a cliff so the eventual fix
//! flips it:
//!
//! - RSK-002 explain: exact `2^layers` predecessor-path count (deterministic).
//! - RSK-003 overflow: `deep_chain` build SIGABRTs (rc 134) — asserted in a child
//!   process via self-re-exec (a stack overflow is uncatchable in-process).
//! - RSK-003 quadratic: `dense_evict` eviction-to-fixpoint pass O(E·(V+E)).
//! - RSK-004 evaluate: `evaluate` runs one `reachable` BFS per node → O(V²) over
//!   the sparse deep-chain spine (analytical-only — first measured here).
//!
//! std-only, public-API-only, zero-dep. Generators are duplicated inline (D4 —
//! `examples/` and `tests/` cannot import each other); the canonical copy lives in
//! `examples/scale_harness.rs`. Follows the existing-test convention
//! (`expect`/`unwrap`, short names) — `tests/` is not clippy-gated (design §8).

use std::collections::BTreeMap;
use std::error::Error;
use std::time::{Duration, Instant};

use cordage::{
    Arity, ChannelSpec, ChannelValue, Combinator, CyclePolicy, Direction, EdgeAttrs, Graph,
    GraphBuilder, NodeId, OrderLayer, OrderSpec, OverlayConfig, OverlayId,
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

// ── 6.1 explain — exact, deterministic (RSK-002) ─────────────────────────────

#[test]
#[ignore = "exponential; demonstrates RSK-002, not a gate run by default"]
fn explain_path_count_is_exponential_in_diamond_depth() {
    let layers = 18; // 2^18 = 262_144 paths; test process stays ~100MB
    let (g, _src, sink, ov) = diamond(layers).expect("diamond build");
    let ex = g.explain(sink);
    let n = ex.paths().get(&ov).map_or(0, Vec::len);
    assert_eq!(n, 1usize << layers); // exact: proves 2^layers growth
}

// ── 6.2 overflow — self-re-exec subprocess, signal-asserted (RSK-003 primary) ─

#[test]
#[ignore = "re-execs itself to crash a child; demonstrates RSK-003"]
fn deep_chain_overflows_inside_target_scale() {
    if std::env::var_os("CORDAGE_OVERFLOW_CHILD").is_some() {
        let _ = deep_chain(80_000); // CHILD: build aborts (rc 134); tuple unused
        return;
    }
    let exe = std::env::current_exe().expect("test bin path");
    let status = std::process::Command::new(exe)
        .args([
            "--exact",
            "deep_chain_overflows_inside_target_scale",
            "--ignored",
        ])
        .env("CORDAGE_OVERFLOW_CHILD", "1")
        .status()
        .expect("spawn child");
    assert!(!status.success()); // signal / rc-134 — the cliff, in-target
}

// ── 6.3 quadratic — measured, recorded, coarse bound (RSK-003 secondary) ──────

#[test]
#[ignore = "slow; records the eviction-fixpoint quadratic for RSK-003"]
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

// ── 6.4 evaluate — measured, recorded, coarse bound (RSK-004, first here) ─────

#[test]
#[ignore = "slow; records the evaluate() per-node-BFS quadratic for RSK-004"]
fn evaluate_scales_quadratically_in_node_count() {
    // Sub-overflow pair (2000,4000): build MUST succeed so query-time cost is
    // isolated. Seed the head NodeId the builder returned (opaque ids — never
    // NodeId(0)). Any's seed domain is ValueKind::Flag → Flag(true) is in-domain.
    let (g1, ov1, h1) = deep_chain(2_000).expect("deep_chain 2000");
    let (g2, ov2, h2) = deep_chain(4_000).expect("deep_chain 4000");
    let s1 = BTreeMap::from([(h1, ChannelValue::Flag(true))]);
    let s2 = BTreeMap::from([(h2, ChannelValue::Flag(true))]);
    let t1 = time(|| {
        g1.evaluate(
            ChannelSpec::new(ov1, Combinator::Any, Direction::Along),
            &s1,
        )
    });
    let t2 = time(|| {
        g2.evaluate(
            ChannelSpec::new(ov2, Combinator::Any, Direction::Along),
            &s2,
        )
    });
    eprintln!(
        "evaluate ratio {:.1}x for 2x nodes",
        t2.as_secs_f64() / t1.as_secs_f64()
    );
    assert!(t2 < Duration::from_secs(120)); // sanity, not a tight gate
}
