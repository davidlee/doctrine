//! Build-time resolution passes (design §5.4 passes 1–2): arity enforcement and
//! per-overlay cycle resolution. Pure — consumes the authored inputs, never
//! mutates them; cycles, evictions, and degraded marks are returned as data,
//! never errors. `build()` orchestrates; this module owns the mechanism.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Arity, CycleDiagnostic, CyclePolicy, Direction, EdgeAttrs, EdgeRef, EvictReason, EvictedEdge,
    Level, NodeId, OrderKey, OrderSpec, OutIndex, OverlayConfig, OverlayId, Provenance, RawEdge,
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

/// Evict the F17-minimal participating edge per cyclic component, to fixpoint.
/// The cyclic components are vertex-disjoint, so the global "drop the
/// globally-minimal participant, re-Tarjan ALL edges" loop and this localized
/// "process each component to its own fixpoint" loop evict the IDENTICAL set:
/// eviction in component A never changes whether an edge in disjoint component B
/// is cyclic. Computing the SCCs once up front (then re-Tarjan only the shrinking
/// sub-component) drops the cost from O(E·(V+E)) to near-linear in N components.
fn pass2_evict(edges: &mut BTreeSet<Edge>, overlay: OverlayId, evictions: &mut Vec<EvictedEdge>) {
    for component in cyclic_components(edges) {
        evict_component(edges, &component, overlay, evictions);
    }
}

/// Drive ONE cyclic component to acyclicity. `component` seeds the vertex set;
/// each step evicts the F17-min edge with both endpoints still inside the
/// shrinking sub-component, then re-Tarjans only that induced sub-edge-set. Each
/// step removes one edge → terminates in ≤ |induced edges| steps.
fn evict_component(
    edges: &mut BTreeSet<Edge>,
    component: &BTreeSet<NodeId>,
    overlay: OverlayId,
    evictions: &mut Vec<EvictedEdge>,
) {
    // The induced sub-edge-set of this component (both endpoints inside it).
    let mut sub: BTreeSet<Edge> = edges
        .iter()
        .filter(|e| component.contains(&e.src) && component.contains(&e.dst))
        .copied()
        .collect();
    loop {
        let cyclic = cyclic_components(&sub);
        if cyclic.is_empty() {
            break;
        }
        // `Edge` orders by the F17 key, so `.min()` selects the eviction-key
        // minimum directly — never adjacency-set order (F37).
        let victim = sub
            .iter()
            .filter(|e| participates(e, &cyclic))
            .min()
            .copied();
        let Some(victim) = victim else {
            break;
        };
        sub.remove(&victim);
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

    /// Iterative DFS (explicit `Vec` stack — depth tracks graph depth without
    /// recursing, so a deep chain no longer overflows the native stack). Each
    /// `Frame` carries a node and an index into its successor list; `successors`
    /// is the `BTreeSet` walked deterministically. A frame is processed in two
    /// interleaved roles: first visit assigns `index`/`lowlink` and pushes to the
    /// Tarjan stack; each successor step either descends (push a child frame) or
    /// folds the child/back-edge `lowlink` in; when successors are exhausted and
    /// `lowlink[v]==index[v]`, the SCC is popped. The `lowlink` fold-on-return is
    /// applied when control comes BACK to the parent frame (`returned`), mirroring
    /// the recursive `lowlink[v] = min(lowlink[v], lowlink[w])`.
    fn strongconnect(&mut self, root: NodeId) {
        // (node, successor cursor). `returned` carries the child whose return is
        // being folded back into the frame now on top of the stack.
        let mut frames: Vec<(NodeId, usize)> = vec![(root, 0)];
        let mut returned: Option<NodeId> = None;

        while let Some(&(v, cursor)) = frames.last() {
            if cursor == 0 && returned.is_none() {
                // First visit of `v`.
                let idx = self.next_index;
                self.next_index = self.next_index.saturating_add(1);
                self.index.insert(v, idx);
                self.lowlink.insert(v, idx);
                self.stack.push(v);
                self.on_stack.insert(v);
            }

            // Fold a just-returned child's lowlink into `v` (recursive
            // `lowlink[v] = min(lowlink[v], lowlink[w])`).
            if let Some(w) = returned.take() {
                let low_w = self.lowlink.get(&w).copied().unwrap_or(0);
                let low_v = self.lowlink.get(&v).copied().unwrap_or(0);
                self.lowlink.insert(v, low_v.min(low_w));
            }

            // Advance through successors until we either descend or exhaust them.
            let successors = self.adjacency.get(&v);
            let mut descended = false;
            let mut next_cursor = cursor;
            if let Some(succ) = successors {
                for &w in succ.iter().skip(cursor) {
                    next_cursor = next_cursor.saturating_add(1);
                    if !self.index.contains_key(&w) {
                        // Descend: park the parent cursor, push the child frame.
                        if let Some(last) = frames.last_mut() {
                            last.1 = next_cursor;
                        }
                        frames.push((w, 0));
                        descended = true;
                        break;
                    } else if self.on_stack.contains(&w) {
                        // Back/cross edge to an on-stack node: fold in index[w].
                        let index_w = self.index.get(&w).copied().unwrap_or(0);
                        let low_v = self.lowlink.get(&v).copied().unwrap_or(0);
                        self.lowlink.insert(v, low_v.min(index_w));
                    }
                }
            }
            if descended {
                continue;
            }
            if let Some(last) = frames.last_mut() {
                last.1 = next_cursor;
            }

            // Successors exhausted: close `v`. Pop its SCC if it is a root.
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
            frames.pop();
            returned = Some(v);
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
pub(crate) fn sort_provenance(provenance: &mut Provenance) {
    provenance.evictions.sort_by_key(|e| (e.overlay, e.edge));
    provenance
        .cycles
        .sort_by(|a, b| (a.overlay, &a.nodes).cmp(&(b.overlay, &b.nodes)));
}

// ── passes 3–4: cross-layer order composition (design §5.4) ──────────────────
// Pass 3 composes the order structure `U` — a DAG SEPARATE from the overlay edge
// sets (I7/F18). Layers are walked in precedence order; each layer's resolved
// edges are oriented and batch-inserted, then U cycles are broken by evicting the
// F17-minimal LAYER-k edge to fixpoint (F10) — earlier layers are never evicted
// against (I2). Pass 4 reads the acyclic U: a longest-path level total over all
// nodes (no sentinel, F12), tainted by descent from the spec-referenced degraded
// SCCs (F31). U eviction reuses `cyclic_components`/`participates`/`Edge` Ord —
// no second Tarjan, no second selection key.

/// The composed-order outcome: per-node keys (pass 4) and the cross-layer
/// `UnionCycleVsLayer` evictions (pass 3).
pub(crate) struct ComposeOutcome {
    pub order_keys: BTreeMap<NodeId, OrderKey>,
    pub evictions: Vec<EvictedEdge>,
}

/// Compose `U` and materialise order keys. `out` is the RESOLVED adjacency (post
/// passes 1–2); `degraded` the post-arity degraded SCCs (F46). Evictions touch
/// `U` only — the overlay edge sets are untouched (I7).
pub(crate) fn compose_order(
    out: &OutIndex,
    spec: &OrderSpec,
    degraded: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    node_count: u32,
) -> ComposeOutcome {
    let mut u: BTreeSet<Edge> = BTreeSet::new();
    let mut evictions: Vec<EvictedEdge> = Vec::new();

    for layer in &spec.layers {
        let overlay = layer.overlay;
        let scc = degraded.get(&overlay);
        let mut layer_k: BTreeSet<Edge> = BTreeSet::new();
        for edge in overlay_edges(out, overlay) {
            // Withhold an intra-SCC edge of a degraded overlay (F32/F46); a
            // boundary-crossing edge still enters U so taint can reach dependents.
            if let Some(components) = scc {
                if participates(&edge, components) {
                    continue;
                }
            }
            let oriented = orient(edge, layer.direction);
            u.insert(oriented);
            layer_k.insert(oriented);
        }
        evict_layer_cycles(&mut u, &mut layer_k, overlay, &mut evictions);
    }

    let order_keys = materialize_keys(&u, spec, degraded, node_count);
    ComposeOutcome {
        order_keys,
        evictions,
    }
}

/// The resolved edges of one overlay, lifted from the adjacency index into the
/// F17-ordered working `Edge` (so U eviction reuses `.min()`/`participates`).
fn overlay_edges(out: &OutIndex, overlay: OverlayId) -> BTreeSet<Edge> {
    let mut set: BTreeSet<Edge> = BTreeSet::new();
    if let Some(by_src) = out.get(&overlay) {
        for (&src, outs) in by_src {
            for oe in outs {
                set.insert(Edge {
                    src,
                    dst: oe.dst,
                    rank: oe.rank,
                    age: oe.age,
                });
            }
        }
    }
    set
}

/// Orient a resolved edge per the layer direction. `Against` reverses; `Along`
/// keeps. `None` is validated out before build, so it cannot reach here.
fn orient(edge: Edge, direction: Direction) -> Edge {
    match direction {
        Direction::Against => Edge {
            src: edge.dst,
            dst: edge.src,
            rank: edge.rank,
            age: edge.age,
        },
        Direction::Along | Direction::None => edge,
    }
}

/// Evict the F17-minimal LAYER-k edge in any U cycle, to fixpoint (F10). Only
/// `layer_k` is evictable, so earlier-layer authority holds (I2). The U cyclic
/// components are vertex-disjoint, so — as in `pass2_evict` — processing each
/// component to its own fixpoint evicts the identical set the global re-Tarjan
/// loop would, at near-linear cost.
fn evict_layer_cycles(
    u: &mut BTreeSet<Edge>,
    layer_k: &mut BTreeSet<Edge>,
    overlay: OverlayId,
    evictions: &mut Vec<EvictedEdge>,
) {
    for component in cyclic_components(u) {
        evict_layer_component(u, layer_k, &component, overlay, evictions);
    }
}

/// Drive ONE U cyclic component to acyclicity by evicting only its `layer_k`
/// edges. Relies on the G2 layer-k invariant: every U-cycle present at layer k
/// contains ≥1 `layer_k` edge (each prior layer is at fixpoint before layer k is
/// inserted, so `U` minus the new `layer_k` edges is acyclic). Hence while a
/// cyclic sub-component remains a `layer_k` victim always exists.
fn evict_layer_component(
    u: &mut BTreeSet<Edge>,
    layer_k: &mut BTreeSet<Edge>,
    component: &BTreeSet<NodeId>,
    overlay: OverlayId,
    evictions: &mut Vec<EvictedEdge>,
) {
    let mut sub: BTreeSet<Edge> = u
        .iter()
        .filter(|e| component.contains(&e.src) && component.contains(&e.dst))
        .copied()
        .collect();
    loop {
        let cyclic = cyclic_components(&sub);
        if cyclic.is_empty() {
            break;
        }
        let victim = sub
            .iter()
            .filter(|e| layer_k.contains(*e) && participates(e, &cyclic))
            .min()
            .copied();
        let Some(victim) = victim else {
            // G2 invariant violation: a cyclic sub-component with NO evictable
            // layer_k edge. The localized loop must not silently leave U cyclic
            // (the global loop would `break` here too, but on the WHOLE U — this
            // is the documented STOP seam). Panic loudly rather than work around.
            debug_assert!(
                false,
                "G2 violated: cyclic U sub-component with no layer_k victim (overlay {overlay:?})"
            );
            break;
        };
        sub.remove(&victim);
        u.remove(&victim);
        layer_k.remove(&victim);
        evictions.push(EvictedEdge {
            overlay,
            edge: victim.to_ref(),
            reason: EvictReason::UnionCycleVsLayer,
        });
    }
}

/// Pass 4: per-node [`OrderKey`] from the longest-path level over the acyclic `U`
/// (total over `0..node_count`, F12) and the taint set (F31/F32).
fn materialize_keys(
    u: &BTreeSet<Edge>,
    spec: &OrderSpec,
    degraded: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    node_count: u32,
) -> BTreeMap<NodeId, OrderKey> {
    let levels = longest_levels(u, node_count);
    let tainted = taint(u, spec, degraded);
    let mut keys: BTreeMap<NodeId, OrderKey> = BTreeMap::new();
    for ordinal in 0..node_count {
        let node = NodeId(ordinal);
        let depth = levels.get(&node).copied().unwrap_or(0);
        let level = if tainted.contains(&node) {
            Level::Degraded(depth)
        } else {
            Level::Finite(depth)
        };
        keys.insert(node, OrderKey { level, node });
    }
    keys
}

/// Longest-path level over the acyclic `U`: `0` with no predecessor, else
/// `1 + max(level(pred))`. Memoised; total over `0..node_count`.
fn longest_levels(u: &BTreeSet<Edge>, node_count: u32) -> BTreeMap<NodeId, u32> {
    let mut preds: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for e in u {
        preds.entry(e.dst).or_default().insert(e.src);
    }
    let mut cache: BTreeMap<NodeId, u32> = BTreeMap::new();
    for ordinal in 0..node_count {
        level_of(NodeId(ordinal), &preds, &mut cache);
    }
    cache
}

/// Memoised longest-path level of one node, computed with an explicit post-order
/// `Vec` stack (no recursion → a clean acyclic chain no longer overflows the
/// native stack). A node is finalised only after every predecessor is cached:
/// the first time it is seen it is re-pushed under all its uncached parents
/// (push-children-then-revisit); on the revisit all parents are cached, so
/// `level = 0` with no preds else `1 + max(level(parent))` — identical to the
/// recursive form. `U` is acyclic here (pass 3 broke every U cycle), so no
/// parent can be on the active-path twice.
fn level_of(
    node: NodeId,
    preds: &BTreeMap<NodeId, BTreeSet<NodeId>>,
    cache: &mut BTreeMap<NodeId, u32>,
) -> u32 {
    if let Some(&cached) = cache.get(&node) {
        return cached;
    }
    // `expanded` marks a node whose children have already been pushed: its next
    // pop is the finalise visit. Distinguishes first-sight from revisit.
    let mut stack: Vec<(NodeId, bool)> = vec![(node, false)];
    while let Some((cur, expanded)) = stack.pop() {
        if cache.contains_key(&cur) {
            continue;
        }
        match preds.get(&cur) {
            None => {
                cache.insert(cur, 0);
            }
            Some(parents) if expanded => {
                let mut best = 0;
                for &parent in parents {
                    let parent_level = cache.get(&parent).copied().unwrap_or(0);
                    best = best.max(parent_level.saturating_add(1));
                }
                cache.insert(cur, best);
            }
            Some(parents) => {
                // First sight: revisit `cur` after its uncached parents resolve.
                stack.push((cur, true));
                for &parent in parents {
                    if !cache.contains_key(&parent) {
                        stack.push((parent, false));
                    }
                }
            }
        }
    }
    cache.get(&node).copied().unwrap_or(0)
}

/// The taint set: seeds = degraded SCC members of spec-referenced overlays only
/// (F31), propagated forward over `U` to every descendant (F32). Empty when no
/// spec overlay is degraded.
fn taint(
    u: &BTreeSet<Edge>,
    spec: &OrderSpec,
    degraded: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
) -> BTreeSet<NodeId> {
    let mut tainted: BTreeSet<NodeId> = BTreeSet::new();
    let mut stack: Vec<NodeId> = Vec::new();
    for layer in &spec.layers {
        if let Some(components) = degraded.get(&layer.overlay) {
            for component in components {
                for &node in component {
                    stack.push(node);
                }
            }
        }
    }
    if stack.is_empty() {
        return tainted;
    }
    let mut succ: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for e in u {
        succ.entry(e.src).or_default().insert(e.dst);
    }
    while let Some(node) = stack.pop() {
        if !tainted.insert(node) {
            continue;
        }
        if let Some(children) = succ.get(&node) {
            for &child in children {
                if !tainted.contains(&child) {
                    stack.push(child);
                }
            }
        }
    }
    tainted
}
