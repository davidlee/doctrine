//! Build-time resolution passes (design §5.4 passes 1–2): arity enforcement and
//! per-overlay cycle resolution. Pure — consumes the authored inputs, never
//! mutates them; cycles, evictions, and degraded marks are returned as data,
//! never errors. `build()` orchestrates; this module owns the mechanism.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Arity, CycleDiagnostic, CyclePolicy, EdgeAttrs, EdgeRef, EvictReason, EvictedEdge, NodeId,
    OverlayConfig, OverlayId, Provenance, RawEdge,
};

/// An edge inside a single overlay's working set. Ordered by the F17 **eviction
/// key** `(rank, age, src, dst)` — so `BTreeSet` iteration yields edges
/// weakest-first and `.min()` / `.max()` select by that key directly, never by
/// adjacency-set order (F37). `BTreeSet` membership also dedupes identical edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Edge {
    src: NodeId,
    dst: NodeId,
    rank: i32,
    age: u64,
}

impl Edge {
    fn to_ref(self) -> EdgeRef {
        EdgeRef {
            src: self.src,
            dst: self.dst,
            attrs: EdgeAttrs::new(self.rank, self.age),
        }
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        // F17 eviction key: (rank asc, age asc, src, dst). Total — determinism is
        // core-internal, independent of the `age` semantic contract (A1).
        (self.rank, self.age, self.src, self.dst)
            .cmp(&(other.rank, other.age, other.src, other.dst))
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The outcome of resolution: the resolved per-overlay edge list (post-arity for
/// all overlays, additionally post-cycle-eviction for `Evict` overlays), the
/// `Provenance` of what was diagnosed/evicted, and the degraded post-arity SCCs
/// of `Reject` overlays (F46) — keyed for PHASE-03 pass-3/4 consumption.
pub(crate) struct Resolution {
    pub edges: Vec<RawEdge>,
    pub provenance: Provenance,
    pub degraded_sccs: BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
}

/// Run the resolution passes over the authored edges. `configs` is indexed by
/// overlay ordinal (the builder's allocation order).
pub(crate) fn resolve(authored: &[RawEdge], configs: &[OverlayConfig]) -> Resolution {
    let authored_by_overlay = group_by_overlay(authored);
    let mut working = authored_by_overlay.clone();
    let mut evictions: Vec<EvictedEdge> = Vec::new();
    let mut cycles: Vec<CycleDiagnostic> = Vec::new();
    let mut degraded_sccs: BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>> = BTreeMap::new();

    pass1_arity(&mut working, configs, &mut evictions);
    pass2_cycles(
        &authored_by_overlay,
        &mut working,
        configs,
        &mut cycles,
        &mut evictions,
        &mut degraded_sccs,
    );

    let edges = flatten(&working);
    let mut provenance = Provenance { cycles, evictions };
    sort_provenance(&mut provenance);
    Resolution {
        edges,
        provenance,
        degraded_sccs,
    }
}

/// Group authored edges into per-overlay working sets (deduped by set semantics).
fn group_by_overlay(authored: &[RawEdge]) -> BTreeMap<OverlayId, BTreeSet<Edge>> {
    let mut by_overlay: BTreeMap<OverlayId, BTreeSet<Edge>> = BTreeMap::new();
    for raw in authored {
        by_overlay.entry(raw.overlay).or_default().insert(Edge {
            src: raw.src,
            dst: raw.dst,
            rank: raw.attrs.rank(),
            age: raw.attrs.age(),
        });
    }
    by_overlay
}

/// Config for an overlay by ordinal — `None` if the ordinal is out of range
/// (cannot happen post-validation, but keeps the read total).
fn config_of(configs: &[OverlayConfig], overlay: OverlayId) -> Option<OverlayConfig> {
    configs.get(usize::from(overlay.0)).copied()
}

// ── pass 1: arity enforcement (F7/F19/F36) ───────────────────────────────────
// On every `AtMostOne` overlay (regardless of cycle policy — F27), a node with
// >1 incoming edge keeps the `(rank, age, src, dst)`-MAX parent (D4: higher rank
// wins the keep); the rest are evicted as `ArityViolation`.

fn pass1_arity(
    working: &mut BTreeMap<OverlayId, BTreeSet<Edge>>,
    configs: &[OverlayConfig],
    evictions: &mut Vec<EvictedEdge>,
) {
    for (&overlay, edges) in working.iter_mut() {
        match config_of(configs, overlay) {
            Some(cfg) if matches!(cfg.arity(), Arity::AtMostOne) => {}
            _ => continue,
        }
        // Group incoming edges by destination.
        let mut incoming: BTreeMap<NodeId, Vec<Edge>> = BTreeMap::new();
        for &e in edges.iter() {
            incoming.entry(e.dst).or_default().push(e);
        }
        for (_dst, parents) in incoming {
            if parents.len() <= 1 {
                continue;
            }
            let Some(&keep) = parents.iter().max() else {
                continue;
            };
            for e in parents {
                if e != keep {
                    evictions.push(EvictedEdge {
                        overlay,
                        edge: e.to_ref(),
                        reason: EvictReason::ArityViolation,
                    });
                    edges.remove(&e);
                }
            }
        }
    }
}

// ── pass 2: per-overlay cycle resolution (D5/REQ-092) ────────────────────────
// THE TRAP (F30/F46): two cycle concepts, two SCC computations.
//   • Reject — the AUTHORED pre-arity SCC drives the `CycleDiagnostic` (the
//     authoring error is always surfaced, even when pass 1 already broke the
//     cycle); the POST-ARITY SCC drives the degraded marks PHASE-03 consumes.
//     No mutation — the cycle stays in the traversal view (REQ-076).
//   • Evict — iteratively drop the globally-minimal participating edge by the
//     F17 key until the post-arity set is acyclic.

fn pass2_cycles(
    authored_by_overlay: &BTreeMap<OverlayId, BTreeSet<Edge>>,
    working: &mut BTreeMap<OverlayId, BTreeSet<Edge>>,
    configs: &[OverlayConfig],
    cycles: &mut Vec<CycleDiagnostic>,
    evictions: &mut Vec<EvictedEdge>,
    degraded_sccs: &mut BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
) {
    for (&overlay, authored_edges) in authored_by_overlay {
        let Some(cfg) = config_of(configs, overlay) else {
            continue;
        };
        match cfg.cycle_policy() {
            CyclePolicy::Reject => {
                // Diagnostic from the AUTHORED pre-arity set.
                for component in cyclic_components(authored_edges) {
                    cycles.push(make_diagnostic(overlay, &component, authored_edges));
                }
                // Degraded marks from the POST-ARITY (resolved) set — F46.
                if let Some(post_arity) = working.get(&overlay) {
                    let degraded = cyclic_components(post_arity);
                    if !degraded.is_empty() {
                        degraded_sccs.insert(overlay, degraded);
                    }
                }
                // Resolved set unchanged — the cycle is preserved for traversal.
            }
            CyclePolicy::Evict => {
                if let Some(edges) = working.get_mut(&overlay) {
                    pass2_evict(edges, overlay, evictions);
                }
            }
        }
    }
}

/// Evict the globally-minimal participating edge by the F17 key, to fixpoint.
/// Each iteration removes one edge → terminates in ≤ |E| steps.
fn pass2_evict(edges: &mut BTreeSet<Edge>, overlay: OverlayId, evictions: &mut Vec<EvictedEdge>) {
    loop {
        let components = cyclic_components(edges);
        if components.is_empty() {
            break;
        }
        // `Edge` orders by the F17 key, so `.min()` selects the eviction-key
        // minimum directly — never adjacency-set order (F37).
        let victim = edges
            .iter()
            .filter(|e| participates(e, &components))
            .min()
            .copied();
        let Some(victim) = victim else {
            break;
        };
        edges.remove(&victim);
        evictions.push(EvictedEdge {
            overlay,
            edge: victim.to_ref(),
            reason: EvictReason::IntraOverlayCycle,
        });
    }
}

/// An edge participates in a cycle iff both endpoints lie in one cyclic component.
fn participates(edge: &Edge, components: &[BTreeSet<NodeId>]) -> bool {
    components
        .iter()
        .any(|c| c.contains(&edge.src) && c.contains(&edge.dst))
}

/// Build a `CycleDiagnostic` for one cyclic component: its nodes and the edges
/// with both endpoints inside it (sorted for deterministic reporting).
fn make_diagnostic(
    overlay: OverlayId,
    component: &BTreeSet<NodeId>,
    edges: &BTreeSet<Edge>,
) -> CycleDiagnostic {
    let mut refs: Vec<EdgeRef> = edges
        .iter()
        .filter(|e| component.contains(&e.src) && component.contains(&e.dst))
        .map(|e| e.to_ref())
        .collect();
    refs.sort();
    CycleDiagnostic {
        overlay,
        nodes: component.clone(),
        edges: refs,
    }
}

// ── strongly-connected components (Tarjan) ───────────────────────────────────
// Deterministic: state is keyed by `NodeId` in `BTree` maps and adjacency is
// walked in `BTreeSet` order, so discovery order is fixed (no ordinal Vec
// indexing — sidesteps the repo `indexing-slicing`/`as`-cast bans). A self-loop
// is a single-node SCC; it counts as cyclic only when the node carries an `n→n`
// edge (F20).

/// The cyclic components of an edge set: SCCs of size > 1, plus single nodes
/// carrying a self-loop.
fn cyclic_components(edges: &BTreeSet<Edge>) -> Vec<BTreeSet<NodeId>> {
    let nodes = node_set(edges);
    let adjacency = adjacency_of(edges);
    let self_loops: BTreeSet<NodeId> = edges
        .iter()
        .filter(|e| e.src == e.dst)
        .map(|e| e.src)
        .collect();
    let mut components = Tarjan::run(&adjacency, &nodes);
    components.retain(|c| c.len() > 1 || c.iter().any(|n| self_loops.contains(n)));
    components
}

fn node_set(edges: &BTreeSet<Edge>) -> BTreeSet<NodeId> {
    let mut nodes = BTreeSet::new();
    for e in edges {
        nodes.insert(e.src);
        nodes.insert(e.dst);
    }
    nodes
}

fn adjacency_of(edges: &BTreeSet<Edge>) -> BTreeMap<NodeId, BTreeSet<NodeId>> {
    let mut adjacency: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for e in edges {
        adjacency.entry(e.src).or_default().insert(e.dst);
    }
    adjacency
}

struct Tarjan<'a> {
    adjacency: &'a BTreeMap<NodeId, BTreeSet<NodeId>>,
    next_index: u32,
    index: BTreeMap<NodeId, u32>,
    lowlink: BTreeMap<NodeId, u32>,
    on_stack: BTreeSet<NodeId>,
    stack: Vec<NodeId>,
    out: Vec<BTreeSet<NodeId>>,
}

impl<'a> Tarjan<'a> {
    fn run(
        adjacency: &'a BTreeMap<NodeId, BTreeSet<NodeId>>,
        nodes: &BTreeSet<NodeId>,
    ) -> Vec<BTreeSet<NodeId>> {
        let mut t = Tarjan {
            adjacency,
            next_index: 0,
            index: BTreeMap::new(),
            lowlink: BTreeMap::new(),
            on_stack: BTreeSet::new(),
            stack: Vec::new(),
            out: Vec::new(),
        };
        for &n in nodes {
            if !t.index.contains_key(&n) {
                t.strongconnect(n);
            }
        }
        t.out
    }

    fn strongconnect(&mut self, v: NodeId) {
        let idx = self.next_index;
        self.next_index = self.next_index.saturating_add(1);
        self.index.insert(v, idx);
        self.lowlink.insert(v, idx);
        self.stack.push(v);
        self.on_stack.insert(v);

        // Copy the `&'a` reference out so the successor borrow is tied to `'a`,
        // not to `&mut self` — lets us recurse without aliasing.
        let adjacency = self.adjacency;
        if let Some(successors) = adjacency.get(&v) {
            for &w in successors {
                if !self.index.contains_key(&w) {
                    self.strongconnect(w);
                    let low_w = self.lowlink.get(&w).copied().unwrap_or(idx);
                    let low_v = self.lowlink.get(&v).copied().unwrap_or(idx);
                    self.lowlink.insert(v, low_v.min(low_w));
                } else if self.on_stack.contains(&w) {
                    let index_w = self.index.get(&w).copied().unwrap_or(idx);
                    let low_v = self.lowlink.get(&v).copied().unwrap_or(idx);
                    self.lowlink.insert(v, low_v.min(index_w));
                }
            }
        }

        if self.lowlink.get(&v) == self.index.get(&v) {
            let mut component = BTreeSet::new();
            while let Some(w) = self.stack.pop() {
                self.on_stack.remove(&w);
                component.insert(w);
                if w == v {
                    break;
                }
            }
            self.out.push(component);
        }
    }
}

/// Flatten the resolved per-overlay sets back into a `RawEdge` list for indexing.
fn flatten(working: &BTreeMap<OverlayId, BTreeSet<Edge>>) -> Vec<RawEdge> {
    let mut out = Vec::new();
    for (&overlay, edges) in working {
        for &e in edges {
            out.push(RawEdge {
                overlay,
                src: e.src,
                dst: e.dst,
                attrs: EdgeAttrs::new(e.rank, e.age),
            });
        }
    }
    out
}

/// Sort provenance for deterministic reporting (F21): evictions by
/// `(overlay, edge)`, cycles by `(overlay, nodes)`. Distinct from the F17
/// selection key used during eviction (F37).
fn sort_provenance(provenance: &mut Provenance) {
    provenance.evictions.sort_by_key(|e| (e.overlay, e.edge));
    provenance
        .cycles
        .sort_by(|a, b| (a.overlay, &a.nodes).cmp(&(b.overlay, &b.nodes)));
}
