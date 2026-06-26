//! Query-time traversal and channel propagation (design §5.2 contract, §5.5
//! I5/I6/I7). Pure reads over the RESOLVED per-overlay adjacency views — never
//! over the composed order `U` or `order_keys` (those degrade *order*, not
//! *reachability*, I7). Cycle-safe over degraded `Reject` views by a visited set
//! (F12/F47). `build()`/`Graph` orchestrate; this module owns the mechanism, a
//! sibling to `resolve` (build-time) — the query-time half of the same seam.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    Channel, ChannelDiagReason, ChannelDiagnostic, ChannelSpec, ChannelValue, Combinator,
    Direction, InIndex, NodeId, OutIndex, OverlayId, Reach, ValueKind,
};

/// A breadth-first walk from `start`: the discovery `order` (`start` first, each
/// node once), each node's min-hop `depth` from `start` (`start` at 0), and whether
/// an `max_depth` cap suppressed a successor that was still unvisited (`truncated`).
struct BfsWalk {
    order: Vec<NodeId>,
    depths: BTreeMap<NodeId, usize>,
    truncated: bool,
}

/// Breadth-first discovery from `start` over the `neighbours` relation: each
/// reachable node yielded exactly once, `start` first. A FIFO frontier and a
/// visited set make it deterministic (given `neighbours` in adjacency-key order)
/// and cycle-safe over a degraded `Reject` view — the visited set bounds re-entry
/// (F12/F47). The single locus of that invariant, shared by `reachable` and
/// `spine_path`. `neighbours` may yield a `Vec` (many successors) or an `Option`
/// (≤1 successor) — `IntoIterator` covers both with no wrapper allocation.
///
/// The FIFO frontier yields nodes in non-decreasing depth, so a node's first visit
/// is its min-hop distance (SL-138 D6). `max_depth: Some(k)` bounds the walk to
/// depth `k`: a node at depth `k` is kept, but its successors (depth `k+1`) are not
/// enqueued. `truncated` is set when such a suppressed successor was still unvisited
/// — by BFS ordering it is genuinely deeper than `k` (never a shallower node reached
/// another way, F5). `None` is unbounded — byte-identical to the pre-SL-138 walk.
///
/// Deliberately NOT the basis of `cone_on_overlay`: the cone records each node's
/// full pred-set as map values and must terminate at degraded-SCC entries
/// (record, don't expand) — neither fits discovery order (SL-140 design D2).
fn walk_bfs<I>(start: NodeId, max_depth: Option<usize>, neighbours: impl Fn(NodeId) -> I) -> BfsWalk
where
    I: IntoIterator<Item = NodeId>,
{
    let mut order = vec![start];
    let mut depths: BTreeMap<NodeId, usize> = BTreeMap::new();
    depths.insert(start, 0);
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    visited.insert(start);
    let mut frontier: VecDeque<(NodeId, usize)> = VecDeque::new();
    frontier.push_back((start, 0));
    let mut truncated = false;
    while let Some((node, depth)) = frontier.pop_front() {
        let at_cap = matches!(max_depth, Some(cap) if depth >= cap);
        for next in neighbours(node) {
            if at_cap {
                // A successor beyond the cap: flag truncation only if it is not
                // already reached within the cap via another (shallower) path.
                if !visited.contains(&next) {
                    truncated = true;
                }
                continue;
            }
            // visited carries `start`, so a cycle back to it never re-adds it.
            if visited.insert(next) {
                depths.insert(next, depth + 1);
                order.push(next);
                frontier.push_back((next, depth + 1));
            }
        }
    }
    BfsWalk {
        order,
        depths,
        truncated,
    }
}

/// The strict reachable set of `start` on `overlay` in `direction` (I6/F8):
/// `start` itself is excluded even when cyclically reachable. `Along` walks
/// out-edges, `Against` walks in-edges, `None` yields ∅ (F25). A foreign overlay
/// or node yields ∅ (F14). Cycle-safe via `walk_bfs`'s visited set (F12).
///
/// Re-expressed over `reachable_bounded(.., None)` (SL-138): the unbounded depth
/// map's keys, `start` already removed, are exactly the strict reachable set.
pub(crate) fn reachable(
    out: &OutIndex,
    incoming: &InIndex,
    overlay: OverlayId,
    start: NodeId,
    direction: Direction,
) -> BTreeSet<NodeId> {
    reachable_bounded(out, incoming, overlay, start, direction, None)
        .depths
        .into_keys()
        .collect()
}

/// The depth-tagged reachable set of `start` on `overlay` in `direction`, bounded by
/// `max_depth` (SL-138 §5). `Reach::depths` maps each strictly-reachable node to its
/// min-hop distance from `start` (`start` excluded, preserving I6/F8 strictness);
/// `Reach::truncated` is true iff the cap suppressed a successor genuinely deeper than
/// it (F5). `max_depth: None` is unbounded — the closure `reachable` returns. `Along`
/// walks out-edges, `Against` in-edges, `None` yields an empty `Reach`; a foreign
/// overlay or node likewise (F14/F25). Cycle-safe via `walk_bfs`'s visited set.
pub(crate) fn reachable_bounded(
    out: &OutIndex,
    incoming: &InIndex,
    overlay: OverlayId,
    start: NodeId,
    direction: Direction,
    max_depth: Option<usize>,
) -> Reach {
    let mut walk = walk_bfs(start, max_depth, |node| {
        neighbours(out, incoming, overlay, node, direction)
    });
    // Strictness: `start` is the only depth-0 entry and is never re-emitted, so
    // dropping it yields exactly `{reachable} \ {start}`.
    walk.depths.remove(&start);
    Reach {
        depths: walk.depths,
        truncated: walk.truncated,
    }
}

/// The spine chain of `node`: follow the single kept parent (pass-1 arity left
/// ≤1 incoming per node on an `AtMostOne` overlay) up the `incoming` view to a
/// root or a cycle re-entry, returned **root → … → node** (ancestor-first). The
/// caller has already gated on `AtMostOne`. Cycle-safe: `walk_bfs`'s visited set
/// stops a surviving `Reject` cycle at re-entry (the chain ends there).
///
/// `single_parent` yields ≤1 successor, so the discovery order degenerates to the
/// linear chain `node → … → root`; `reverse` makes it ancestor-first.
pub(crate) fn spine_path(incoming: &InIndex, overlay: OverlayId, node: NodeId) -> Vec<NodeId> {
    // Unbounded walk; the spine consumes discovery order only — depth is irrelevant.
    let mut chain = walk_bfs(node, None, |cur| single_parent(incoming, overlay, cur)).order;
    chain.reverse();
    chain
}

/// The predecessor cone of `node`, per overlay: the predecessor sub-DAG as a
/// `node ↦ {immediate in-cone predecessors}` adjacency map (design §5.4, A-5,
/// D2/F13). For each overlay present in the `incoming` view, BFS UP the in-edges
/// from `node` with a **global** visited set — each cone node is enqueued at most
/// once, so the walk is O(V+E): no path enumeration, no clone, no `2^layers`
/// blow-up on diamonds. Policy reconstructs any chain/spine/witness by walking the
/// returned adjacency.
///
/// Termination (F47): the in-edges are followed up to **roots** (no predecessors)
/// and **degraded-SCC entries**. A root is recorded as a key with an empty
/// pred-set. A node inside a degraded post-arity SCC on the overlay
/// (`degraded_sccs[overlay]`) is recorded as an endpoint — a key with an empty
/// pred-set, never expanded — even though it stays in its successor's pred-set. So
/// the cone is finite and deterministic even on a cyclic `Reject` view. If `node`
/// itself is inside a degraded SCC its cone is exactly `{node: {}}` (the old
/// `[[node]]`).
pub(crate) fn predecessor_cone(
    incoming: &InIndex,
    degraded_sccs: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    node: NodeId,
) -> BTreeMap<OverlayId, BTreeMap<NodeId, BTreeSet<NodeId>>> {
    incoming
        .keys()
        .map(|&overlay| {
            let cone = cone_on_overlay(incoming, degraded_sccs, overlay, node);
            (overlay, cone)
        })
        .collect()
}

/// The predecessor cone of `node` on one overlay (node ↦ immediate in-cone
/// predecessors). A node inside a degraded SCC yields `{node: {}}`; otherwise a
/// single BFS up the in-edges with a global visited set records each reached node
/// once, terminating at roots (empty pred-set) and degraded-SCC entries (recorded
/// as empty-pred endpoints, never enqueued).
///
/// Deliberately NOT a `walk_bfs` caller (SL-140 design D2): the cone records each
/// node's full pred-set as map *values* and must block expansion at degraded-SCC
/// entries (record, don't expand) — neither is expressible through the
/// discovery-order primitive without a heavier visitor+predicate abstraction that
/// would obscure the SCC-endpoint logic. The reuse seam is the shared neighbour
/// helpers (`predecessors`, `in_degraded_scc`), not the loop.
fn cone_on_overlay(
    incoming: &InIndex,
    degraded_sccs: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    overlay: OverlayId,
    node: NodeId,
) -> BTreeMap<NodeId, BTreeSet<NodeId>> {
    let mut cone: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    // A node that is itself a degraded-SCC member is the endpoint: {node: {}}.
    if in_degraded_scc(degraded_sccs, overlay, node) {
        cone.insert(node, BTreeSet::new());
        return cone;
    }
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    visited.insert(node);
    let mut frontier: VecDeque<NodeId> = VecDeque::new();
    frontier.push_back(node);
    while let Some(cur) = frontier.pop_front() {
        let pset: BTreeSet<NodeId> = predecessors(incoming, overlay, cur).into_iter().collect();
        for &p in &pset {
            if in_degraded_scc(degraded_sccs, overlay, p) {
                // SCC entry — record as an empty-pred endpoint, never expand it.
                cone.entry(p).or_default();
            } else if visited.insert(p) {
                frontier.push_back(p);
            }
            // Either way, `p` stays in `cur`'s pred-set below.
        }
        cone.insert(cur, pset);
    }
    cone
}

/// Whether `node` is a member of any degraded post-arity SCC on `overlay`.
fn in_degraded_scc(
    degraded_sccs: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    overlay: OverlayId,
    node: NodeId,
) -> bool {
    degraded_sccs
        .get(&overlay)
        .is_some_and(|sccs| sccs.iter().any(|scc| scc.contains(&node)))
}

/// The predecessors of `node` on `overlay` (in-edge sources), in adjacency-key
/// order — a deterministic, possibly multi-element list (`AtMostOne` leaves ≤1).
fn predecessors(incoming: &InIndex, overlay: OverlayId, node: NodeId) -> Vec<NodeId> {
    incoming
        .get(&overlay)
        .and_then(|by_dst| by_dst.get(&node))
        .map(|set| set.iter().map(|e| e.src).collect())
        .unwrap_or_default()
}

/// The kept parent of `node` on `overlay`, or `None` at a root. Post pass-1
/// arity an `AtMostOne` overlay has ≤1 in-edge per node; the first (and only) is
/// it.
fn single_parent(incoming: &InIndex, overlay: OverlayId, node: NodeId) -> Option<NodeId> {
    incoming
        .get(&overlay)
        .and_then(|by_dst| by_dst.get(&node))
        .and_then(|set| set.iter().next().map(|e| e.src))
}

/// The direct successors of `node` under `direction`: out-edge `dst`s for
/// `Along`, in-edge `src`s for `Against`, nothing for `None`. A foreign overlay
/// or node yields nothing (the index lookups return `None`).
fn neighbours(
    out: &OutIndex,
    incoming: &InIndex,
    overlay: OverlayId,
    node: NodeId,
    direction: Direction,
) -> Vec<NodeId> {
    match direction {
        Direction::Along => out
            .get(&overlay)
            .and_then(|by_src| by_src.get(&node))
            .map(|set| set.iter().map(|e| e.dst).collect())
            .unwrap_or_default(),
        Direction::Against => incoming
            .get(&overlay)
            .and_then(|by_dst| by_dst.get(&node))
            .map(|set| set.iter().map(|e| e.src).collect())
            .unwrap_or_default(),
        Direction::None => Vec::new(),
    }
}

// ── channel propagation (design §5.2 contract, F15/F34) ───────────────────────

/// Evaluate `spec` over `seeds` (design §5.2). Per node `n` (over `0..node_count`)
/// the fold set is `{n} ∪ reachable(n)` for the idempotent combinators
/// (`Any`/`All`/`Max`) and STRICT `reachable(n)` for `CountDistinct` (F34) — and
/// `reachable(_, None) = ∅` makes `Direction::None` fall out of the same
/// definition (own-seed for idempotent, always-absent for `CountDistinct`, F35).
/// The seed contract (F16/F41/F45) is enforced once up front: a foreign-`NodeId`
/// seed → `UnknownSeedNode`; a variant-mismatched seed → `SeedVariantMismatch`;
/// `UnknownSeedNode` wins (a foreign node is never variant-checked), ≤1 diagnostic
/// per node, sorted by `NodeId` (the seed `BTreeMap` is already in id order).
pub(crate) fn evaluate(
    out: &OutIndex,
    incoming: &InIndex,
    degraded_sccs: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    node_count: u32,
    spec: ChannelSpec,
    seeds: &BTreeMap<NodeId, ChannelValue>,
) -> Channel {
    let combinator = spec.combinator();
    let domain = seed_domain(combinator);
    let (effective, diagnostics) = vet_seeds(seeds, node_count, domain);
    let view = Resolved {
        out,
        incoming,
        overlay: spec.overlay(),
        direction: spec.direction(),
    };

    // RSK-004: one condensation fold per call, not a per-node reachable BFS.
    // (1) SCC partition of the DIRECTION-RESOLVED neighbour view (G1/C1): None ⇒
    //     all singletons (neighbours = ∅, never group the stored {b,c}); Evict ⇒
    //     no stored SCCs ⇒ singletons; Reject Along/Against ⇒ the stored
    //     degraded_sccs grouped (SCCs survive transpose), the rest singletons.
    let partition = scc_partition(degraded_sccs, view.overlay, view.direction, node_count);
    // (2) Condensation DAG + reverse-topo (sinks first), built from the SAME
    //     direction-resolved neighbour view — `out` for Along, `incoming` for
    //     Against (A-2); inter-SCC edges only (self/intra-SCC dropped, C3).
    let reverse_topo = condensation_reverse_topo(&view, &partition);
    // (3) Per-combinator fold up the reverse-topo order; each node emits the same
    //     (value, contributors) fold_node would, with no reachable-set materialised.
    let (values, contributors) =
        fold_condensation(&view, combinator, &effective, &partition, &reverse_topo);

    Channel {
        values,
        contributors,
        diagnostics,
    }
}

/// The direction-resolved neighbour view a single `evaluate` call walks: a
/// `(overlay, direction)` lens over the two adjacency indices. Bundling the four
/// fields keeps the condensation helpers' signatures small and guarantees the
/// partition, the condensation edges, and the fold all read the SAME view (A-2).
struct Resolved<'a> {
    out: &'a OutIndex,
    incoming: &'a InIndex,
    overlay: OverlayId,
    direction: Direction,
}

impl Resolved<'_> {
    /// The direct neighbours of `node` under this view (`out.dst` for `Along`,
    /// `incoming.src` for `Against`, ∅ for `None`).
    fn neighbours(&self, node: NodeId) -> Vec<NodeId> {
        neighbours(self.out, self.incoming, self.overlay, node, self.direction)
    }
}

/// A total partition of `0..node_count` into SCCs of the **direction-resolved**
/// neighbour view (G1/C1). `scc_of[ord]` is node `NodeId(ord)`'s SCC id;
/// `members[scc_id]` lists that SCC's nodes in `NodeId` order.
///
/// `Direction::None` ⇒ neighbours are ∅, so every node is a singleton (the stored
/// `{b,c}` is NEVER grouped). An `Evict` overlay has no entry in `degraded_sccs`,
/// also yielding singletons. A `Reject` overlay under `Along`/`Against` reuses the
/// stored cyclic SCCs (mutual reachability survives transpose), every other node a
/// singleton — a total partition either way.
struct Partition {
    scc_of: Vec<usize>,
    members: Vec<Vec<NodeId>>,
}

/// A `NodeId`'s ordinal as a `usize` index into the per-node partition vectors —
/// `NodeId` wraps a `u32`, so `usize::from` is unavailable; `try_from` saturates
/// (a `u32` always fits `usize` on supported targets).
fn ord_index(node: NodeId) -> usize {
    usize::try_from(node.0).unwrap_or(usize::MAX)
}

fn scc_partition(
    degraded_sccs: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    overlay: OverlayId,
    direction: Direction,
    node_count: u32,
) -> Partition {
    let count = usize::try_from(node_count).unwrap_or(usize::MAX);
    let mut scc_of: Vec<usize> = vec![usize::MAX; count];
    let mut members: Vec<Vec<NodeId>> = Vec::new();

    // Under None the partition dissolves to all singletons regardless of the
    // stored SCCs (C1). Otherwise group the stored cyclic SCCs for this overlay
    // (absent for Evict / acyclic overlays).
    if !matches!(direction, Direction::None) {
        if let Some(sccs) = degraded_sccs.get(&overlay) {
            for scc in sccs {
                if scc.len() < 2 {
                    continue; // a singleton SCC is handled by the trivial pass
                }
                let id = members.len();
                let mut group: Vec<NodeId> = Vec::with_capacity(scc.len());
                for &node in scc {
                    if let Some(slot) = scc_of.get_mut(ord_index(node)) {
                        *slot = id;
                    }
                    group.push(node);
                }
                members.push(group);
            }
        }
    }

    // Every node not claimed by a grouped SCC is its own singleton.
    for ord in 0..node_count {
        let idx = usize::try_from(ord).unwrap_or(usize::MAX);
        if scc_of.get(idx).copied() == Some(usize::MAX) {
            let id = members.len();
            if let Some(slot) = scc_of.get_mut(idx) {
                *slot = id;
            }
            members.push(vec![NodeId(ord)]);
        }
    }

    Partition { scc_of, members }
}

/// Reverse-topological order (sinks first) of the condensation DAG, built from the
/// **same direction-resolved neighbour view** `evaluate` walks (`out` for `Along`,
/// `incoming` for `Against`; A-2). Inter-SCC edges are the member neighbours
/// quotiented by SCC id — a neighbour in the same SCC (self/intra, C3) is dropped,
/// so the quotient is a genuine DAG and reverse-topo is well-defined. Explicit
/// stack, no recursion; O(V+E).
fn condensation_reverse_topo(view: &Resolved<'_>, partition: &Partition) -> Vec<usize> {
    let scc_count = partition.members.len();
    // Distinct successor SCC ids per SCC (BTreeSet ⇒ deterministic, deduped).
    let mut succ: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); scc_count];
    for (id, group) in partition.members.iter().enumerate() {
        for &node in group {
            for nb in view.neighbours(node) {
                if let Some(&nb_id) = partition.scc_of.get(ord_index(nb)) {
                    if nb_id != id {
                        if let Some(set) = succ.get_mut(id) {
                            set.insert(nb_id);
                        }
                    }
                }
            }
        }
    }

    // Iterative post-order DFS: an SCC is emitted only after all its successors,
    // yielding sinks-first (reverse-topo). `state`: 0 unseen, 1 on-stack, 2 done.
    let mut order: Vec<usize> = Vec::with_capacity(scc_count);
    let mut state: Vec<u8> = vec![0; scc_count];
    for root in 0..scc_count {
        if state.get(root).copied() != Some(0) {
            continue;
        }
        let mut stack: Vec<usize> = vec![root];
        while let Some(&id) = stack.last() {
            match state.get(id).copied() {
                Some(0) => {
                    if let Some(slot) = state.get_mut(id) {
                        *slot = 1;
                    }
                    if let Some(children) = succ.get(id) {
                        for &child in children {
                            if state.get(child).copied() == Some(0) {
                                stack.push(child);
                            }
                        }
                    }
                }
                Some(1) => {
                    if let Some(slot) = state.get_mut(id) {
                        *slot = 2;
                    }
                    order.push(id);
                    stack.pop();
                }
                _ => {
                    stack.pop();
                }
            }
        }
    }
    order
}

/// A flag-combinator accumulator: the present-true and present-false seed nodes
/// over an SCC's reachable cone, unioned up the condensation. `present` =
/// `trues ∪ falses` (any present `Flag` seed, F45).
#[derive(Clone, Default)]
struct FlagWitnesses {
    trues: BTreeSet<NodeId>,
    falses: BTreeSet<NodeId>,
}

/// Fold the condensation per combinator, emitting for each node the SAME
/// `(value, contributors)` `fold_node` produces — without materialising any
/// reachable set. Idempotent combinators (`Any`/`All`/`Max`) fold `{n} ∪ reach`
/// and the whole SCC shares one result; `CountDistinct` folds STRICT `reach` and
/// each member restricts to `\ {n}` off the shared SCC witness set (C2/F34).
fn fold_condensation(
    view: &Resolved<'_>,
    combinator: Combinator,
    effective: &BTreeMap<NodeId, ChannelValue>,
    partition: &Partition,
    reverse_topo: &[usize],
) -> (
    BTreeMap<NodeId, ChannelValue>,
    BTreeMap<NodeId, BTreeSet<NodeId>>,
) {
    match combinator {
        Combinator::Max => fold_max_condensation(view, effective, partition, reverse_topo),
        Combinator::Any | Combinator::All | Combinator::CountDistinct => {
            fold_flags_condensation(view, combinator, effective, partition, reverse_topo)
        }
    }
}

/// The distinct successor-SCC ids of `scc_id` under the direction-resolved view —
/// the same quotient `condensation_reverse_topo` builds, recomputed locally so the
/// fold needs no second adjacency allocation.
fn successor_sccs(view: &Resolved<'_>, partition: &Partition, scc_id: usize) -> BTreeSet<usize> {
    let mut succ: BTreeSet<usize> = BTreeSet::new();
    if let Some(group) = partition.members.get(scc_id) {
        for &node in group {
            for nb in view.neighbours(node) {
                if let Some(&nb_id) = partition.scc_of.get(ord_index(nb)) {
                    if nb_id != scc_id {
                        succ.insert(nb_id);
                    }
                }
            }
        }
    }
    succ
}

/// `Max` fold up the condensation: each SCC's `(value, argmax)` is the max over its
/// own member seeds (own seed included — `{n} ∪ reach`) and its successor SCCs'
/// results, min-`NodeId` tiebreak. Mutual reachability ⇒ the whole SCC shares one
/// `(value, argmax)`. Fully O(V+E) — a singleton argmax, no set materialised.
fn fold_max_condensation(
    view: &Resolved<'_>,
    effective: &BTreeMap<NodeId, ChannelValue>,
    partition: &Partition,
    reverse_topo: &[usize],
) -> (
    BTreeMap<NodeId, ChannelValue>,
    BTreeMap<NodeId, BTreeSet<NodeId>>,
) {
    let mut scc_best: Vec<Option<(i64, NodeId)>> = vec![None; partition.members.len()];
    for &scc_id in reverse_topo {
        let mut best: Option<(i64, NodeId)> = None;
        // Own member seeds ({n} ∪ reach includes every SCC member's own seed).
        if let Some(group) = partition.members.get(scc_id) {
            for &node in group {
                if let Some(ChannelValue::Scalar(value)) = effective.get(&node).copied() {
                    best = supersede(best, value, node);
                }
            }
        }
        // Already-folded successor SCC results (sinks-first ⇒ ready).
        for succ_id in successor_sccs(view, partition, scc_id) {
            if let Some(Some((value, argmax))) = scc_best.get(succ_id).copied() {
                best = supersede(best, value, argmax);
            }
        }
        if let Some(slot) = scc_best.get_mut(scc_id) {
            *slot = best;
        }
    }

    let mut values: BTreeMap<NodeId, ChannelValue> = BTreeMap::new();
    let mut contributors: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for (scc_id, group) in partition.members.iter().enumerate() {
        if let Some(Some((value, argmax))) = scc_best.get(scc_id).copied() {
            for &node in group {
                values.insert(node, ChannelValue::Scalar(value));
                contributors.insert(node, BTreeSet::from([argmax]));
            }
        }
    }
    (values, contributors)
}

/// The `fold_max` supersede rule (F21): a strictly greater value wins; an equal
/// value wins only on a strictly smaller `NodeId` (min-id argmax tiebreak).
fn supersede(best: Option<(i64, NodeId)>, value: i64, node: NodeId) -> Option<(i64, NodeId)> {
    let wins = match best {
        None => true,
        Some((best_value, best_node)) => {
            value > best_value || (value == best_value && node < best_node)
        }
    };
    if wins { Some((value, node)) } else { best }
}

/// `Any`/`All`/`CountDistinct` fold up the condensation. Each SCC accumulates the
/// present-true / present-false seed nodes over its reachable cone (the union of
/// member seeds and successor-SCC witnesses). The idempotent combinators include
/// the own seed (`{n} ∪ reach`) and share the SCC result; `CountDistinct` is
/// STRICT — the shared set is the pre-subtraction SCC witnesses and each member
/// restricts to `\ {n}` (C2/F34).
fn fold_flags_condensation(
    view: &Resolved<'_>,
    combinator: Combinator,
    effective: &BTreeMap<NodeId, ChannelValue>,
    partition: &Partition,
    reverse_topo: &[usize],
) -> (
    BTreeMap<NodeId, ChannelValue>,
    BTreeMap<NodeId, BTreeSet<NodeId>>,
) {
    let mut scc_wit: Vec<FlagWitnesses> = vec![FlagWitnesses::default(); partition.members.len()];
    for &scc_id in reverse_topo {
        let mut wit = FlagWitnesses::default();
        if let Some(group) = partition.members.get(scc_id) {
            for &node in group {
                match effective.get(&node).copied() {
                    Some(ChannelValue::Flag(true)) => {
                        wit.trues.insert(node);
                    }
                    Some(ChannelValue::Flag(false)) => {
                        wit.falses.insert(node);
                    }
                    _ => {}
                }
            }
        }
        for succ_id in successor_sccs(view, partition, scc_id) {
            if let Some(succ_wit) = scc_wit.get(succ_id) {
                wit.trues.extend(succ_wit.trues.iter().copied());
                wit.falses.extend(succ_wit.falses.iter().copied());
            }
        }
        if let Some(slot) = scc_wit.get_mut(scc_id) {
            *slot = wit;
        }
    }

    let mut values: BTreeMap<NodeId, ChannelValue> = BTreeMap::new();
    let mut contributors: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for (scc_id, group) in partition.members.iter().enumerate() {
        let Some(wit) = scc_wit.get(scc_id) else {
            continue;
        };
        for &node in group {
            if let Some((value, contrib)) = member_value(combinator, node, wit) {
                values.insert(node, value);
                if !contrib.is_empty() {
                    contributors.insert(node, contrib);
                }
            }
        }
    }
    (values, contributors)
}

/// One node's `(value, contributors)` derived from its SCC's flag witnesses,
/// matching `fold_any`/`fold_all`/`fold_count` exactly. The idempotent combinators
/// fold `{n} ∪ reach` (the SCC set, own seed included); `CountDistinct` is STRICT,
/// so it removes `n` from both the presence test and the witness set (F8/F34).
fn member_value(
    combinator: Combinator,
    node: NodeId,
    wit: &FlagWitnesses,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    match combinator {
        Combinator::Any => {
            if wit.trues.is_empty() && wit.falses.is_empty() {
                return None; // no present Flag seed in {n} ∪ reach
            }
            Some((ChannelValue::Flag(!wit.trues.is_empty()), wit.trues.clone()))
        }
        Combinator::All => {
            if wit.trues.is_empty() && wit.falses.is_empty() {
                return None;
            }
            if wit.falses.is_empty() {
                Some((ChannelValue::Flag(true), wit.trues.clone()))
            } else {
                Some((ChannelValue::Flag(false), wit.falses.clone()))
            }
        }
        Combinator::CountDistinct => {
            // STRICT: exclude `node` from its own fold set (reach excludes n, F8).
            let present = wit
                .trues
                .iter()
                .chain(wit.falses.iter())
                .any(|m| *m != node);
            if !present {
                return None;
            }
            let counted: BTreeSet<NodeId> =
                wit.trues.iter().copied().filter(|m| *m != node).collect();
            Some((ChannelValue::Count(strict_count(&counted)), counted))
        }
        Combinator::Max => None, // Max never routes here (handled by fold_max_condensation)
    }
}

/// `BTreeSet` cardinality as the `u32` count, saturating like `fold_count`.
fn strict_count(counted: &BTreeSet<NodeId>) -> u32 {
    u32::try_from(counted.len()).unwrap_or(u32::MAX)
}

/// Split the seed map into the **effective** seeds (known node, in-domain variant)
/// and the diagnostics for the rest. `UnknownSeedNode` is checked first, so it
/// wins over a co-located variant mismatch (F41).
fn vet_seeds(
    seeds: &BTreeMap<NodeId, ChannelValue>,
    node_count: u32,
    domain: ValueKind,
) -> (BTreeMap<NodeId, ChannelValue>, Vec<ChannelDiagnostic>) {
    let mut effective: BTreeMap<NodeId, ChannelValue> = BTreeMap::new();
    let mut diagnostics: Vec<ChannelDiagnostic> = Vec::new();
    for (&node, &value) in seeds {
        if node.0 >= node_count {
            diagnostics.push(ChannelDiagnostic {
                node,
                reason: ChannelDiagReason::UnknownSeedNode,
            });
        } else if value_kind(value) != domain {
            diagnostics.push(ChannelDiagnostic {
                node,
                reason: ChannelDiagReason::SeedVariantMismatch {
                    expected: domain,
                    actual: value_kind(value),
                },
            });
        } else {
            effective.insert(node, value);
        }
    }
    (effective, diagnostics)
}

/// The seed/output `ValueKind` a combinator consumes (`Any`/`All`/`CountDistinct`
/// take `Flag`, `Max` takes `Scalar`).
fn seed_domain(combinator: Combinator) -> ValueKind {
    match combinator {
        Combinator::Any | Combinator::All | Combinator::CountDistinct => ValueKind::Flag,
        Combinator::Max => ValueKind::Scalar,
    }
}

/// The discriminant of a [`ChannelValue`].
fn value_kind(value: ChannelValue) -> ValueKind {
    match value {
        ChannelValue::Flag(_) => ValueKind::Flag,
        ChannelValue::Scalar(_) => ValueKind::Scalar,
        ChannelValue::Count(_) => ValueKind::Count,
    }
}
