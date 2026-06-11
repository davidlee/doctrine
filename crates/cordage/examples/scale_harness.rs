//! `scale_harness` — SL-038 PHASE-01 measurement harness for the four confirmed
//! cordage scale cliffs, all reachable inside the ~tens-of-thousands target:
//!
//! - RSK-003 overflow: `deep_chain` build SIGABRTs (Tarjan / `level_of` recurse
//!   to chain depth).
//! - RSK-002 explain: `explain` enumerates 2^layers predecessor paths.
//! - RSK-003 quadratic: `dense_evict` drives the eviction-to-fixpoint pass
//!   O(E·(V+E)).
//! - RSK-004 evaluate: `evaluate` runs one `reachable` BFS per node → O(V²) over
//!   the sparse deep-chain spine.
//!
//! std-only, public-API-only, zero-dep. Each invocation runs ONE cliff and emits
//! CSV `cliff,param,metric,value` lines to stdout. Disposable evidence tool, not a
//! shipped surface — the durable reds live in `tests/scale_cliffs.rs` (PHASE-02).
//!
//! NOTE (opaque ids): `NodeId`/`OverlayId` have no public constructor — a seed
//! node must be captured from `GraphBuilder::node`, never written `NodeId(0)`.
//! `deep_chain` therefore returns its head node for the evaluate seed.

use std::collections::BTreeMap;
use std::error::Error;
use std::io::Write;
use std::time::Instant;

use cordage::{
    Arity, ChannelSpec, ChannelValue, Combinator, CyclePolicy, Direction, EdgeAttrs, Graph,
    GraphBuilder, NodeId, OrderLayer, OrderSpec, OverlayConfig, OverlayId,
};

const USAGE: &str = "usage: scale_harness --cliff overflow|explain|quadratic|evaluate \
                     [--n N] [--layers L]";

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

fn millis(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn run(cliff: &str, n: Option<u32>, layers: Option<u32>, out: &mut impl Write) -> Built<()> {
    match cliff {
        "overflow" => {
            let n = n.ok_or("overflow needs --n")?;
            let start = Instant::now();
            // Build may SIGABRT (rc 134) here — that abort IS the cliff signal; if
            // it survives, the build time is printed instead.
            let (_g, _ov, _head) = deep_chain(n)?;
            writeln!(out, "overflow,{n},build_ms,{:.1}", millis(start))?;
        }
        "explain" => {
            let layers = layers.ok_or("explain needs --layers")?;
            let (g, _src, sink, ov) = diamond(layers)?;
            let start = Instant::now();
            let ex = g.explain(sink);
            // Cone node count (predecessor sub-DAG) — LINEAR in layers post-RSK-002,
            // not the old 2^layers path count.
            let cone_nodes = ex.predecessors().get(&ov).map_or(0, BTreeMap::len);
            let ms = millis(start);
            writeln!(out, "explain,{layers},cone_nodes,{cone_nodes}")?;
            writeln!(out, "explain,{layers},explain_ms,{ms:.1}")?;
        }
        "quadratic" => {
            let n = n.ok_or("quadratic needs --n")?;
            let start = Instant::now();
            let g = dense_evict(n, n)?;
            let ms = millis(start);
            writeln!(
                out,
                "quadratic,{n},evicted,{}",
                g.provenance().evictions().len()
            )?;
            writeln!(out, "quadratic,{n},build_ms,{ms:.1}")?;
        }
        "evaluate" => {
            let n = n.ok_or("evaluate needs --n")?;
            let (g, ov, head) = deep_chain(n)?;
            let mut seeds = BTreeMap::new();
            seeds.insert(head, ChannelValue::Flag(true));
            let spec = ChannelSpec::new(ov, Combinator::Any, Direction::Along);
            let start = Instant::now();
            let ch = g.evaluate(spec, &seeds);
            let ms = millis(start);
            writeln!(out, "evaluate,{n},values,{}", ch.values().len())?;
            writeln!(out, "evaluate,{n},evaluate_ms,{ms:.1}")?;
        }
        other => return Err(format!("unknown cliff: {other}\n{USAGE}").into()),
    }
    Ok(())
}

fn main() -> Built<()> {
    let mut cliff: Option<String> = None;
    let mut n: Option<u32> = None;
    let mut layers: Option<u32> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cliff" => cliff = Some(args.next().ok_or("--cliff needs a value")?),
            "--n" => n = Some(args.next().ok_or("--n needs a value")?.parse()?),
            "--layers" => layers = Some(args.next().ok_or("--layers needs a value")?.parse()?),
            other => return Err(format!("unknown argument: {other}\n{USAGE}").into()),
        }
    }
    let cliff = cliff.ok_or_else(|| format!("--cliff required\n{USAGE}"))?;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    run(&cliff, n, layers, &mut out)
}
