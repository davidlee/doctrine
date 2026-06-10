//! Query-time traversal and channel propagation (design §5.2 contract, §5.5
//! I5/I6/I7). Pure reads over the RESOLVED per-overlay adjacency views — never
//! over the composed order `U` or `order_keys` (those degrade *order*, not
//! *reachability*, I7). Cycle-safe over degraded `Reject` views by a visited set
//! (F12/F47). `build()`/`Graph` orchestrate; this module owns the mechanism, a
//! sibling to `resolve` (build-time) — the query-time half of the same seam.

use std::collections::{BTreeSet, VecDeque};

use crate::{Direction, InIndex, NodeId, OutIndex, OverlayId};

/// The strict reachable set of `start` on `overlay` in `direction` (I6/F8):
/// `start` itself is excluded even when cyclically reachable. `Along` walks
/// out-edges, `Against` walks in-edges, `None` yields ∅ (F25). A foreign overlay
/// or node yields ∅ (F14). Cycle-safe: a visited set bounds the BFS over a
/// degraded `Reject` view (F12).
pub(crate) fn reachable(
    out: &OutIndex,
    incoming: &InIndex,
    overlay: OverlayId,
    start: NodeId,
    direction: Direction,
) -> BTreeSet<NodeId> {
    let mut reached: BTreeSet<NodeId> = BTreeSet::new();
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    visited.insert(start);
    let mut frontier: VecDeque<NodeId> = VecDeque::new();
    frontier.push_back(start);
    while let Some(node) = frontier.pop_front() {
        for next in neighbours(out, incoming, overlay, node, direction) {
            // visited carries `start`, so a cycle back to it never re-adds it —
            // strictness holds without a separate guard.
            if visited.insert(next) {
                reached.insert(next);
                frontier.push_back(next);
            }
        }
    }
    reached
}

/// The spine chain of `node`: follow the single kept parent (pass-1 arity left
/// ≤1 incoming per node on an `AtMostOne` overlay) up the `incoming` view to a
/// root or a cycle re-entry, returned **root → … → node** (ancestor-first). The
/// caller has already gated on `AtMostOne`. Cycle-safe: a visited set stops a
/// surviving `Reject` cycle at re-entry (the chain ends there).
pub(crate) fn spine_path(incoming: &InIndex, overlay: OverlayId, node: NodeId) -> Vec<NodeId> {
    let mut chain = vec![node];
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    visited.insert(node);
    let mut cur = node;
    while let Some(parent) = single_parent(incoming, overlay, cur) {
        if !visited.insert(parent) {
            break; // re-entry into a surviving cycle — stop, chain ends here
        }
        chain.push(parent);
        cur = parent;
    }
    chain.reverse();
    chain
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
