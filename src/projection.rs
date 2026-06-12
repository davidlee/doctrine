// SPDX-License-Identifier: GPL-3.0-only
//! A generic id↔node bimap over a cordage [`GraphBuilder`] (design §5.2, D3).
//!
//! The pure projection leaf below the ordering adapter (`backlog_order`): it owns
//! the mapping between a domain key `K` and an opaque cordage [`NodeId`], so the
//! engine half speaks only `K` and never re-derives the correspondence. It rides
//! [`GraphBuilder`] only — no clock, disk, or rng — and is **below** `backlog_order`
//! in the layering (ADR-001): it never imports the engine.
//!
//! [`Projection::intern`] is mint-or-get and mints in **caller call-order** — it
//! imposes no order of its own. [`NodeId`] allocation order is behaviour-relevant for
//! the consumer's tier-4 tie-break, so the leaf must preserve the order the caller
//! interns in (`backlog_order` pre-interns in its own sorted order, C4).
use cordage::{GraphBuilder, NodeId};
use std::collections::{BTreeMap, BTreeSet};

/// A bidirectional `key ↔ NodeId` map. `K` is `Copy + Ord` so both directions are
/// ordered `BTreeMap`s (deterministic, no `HashMap` — repo ban / REQ-077).
#[derive(Debug)]
pub(crate) struct Projection<K: Copy + Ord> {
    by_key: BTreeMap<K, NodeId>,
    by_node: BTreeMap<NodeId, K>,
}

impl<K: Copy + Ord> Projection<K> {
    /// An empty projection.
    pub(crate) fn new() -> Self {
        Self {
            by_key: BTreeMap::new(),
            by_node: BTreeMap::new(),
        }
    }

    /// Mint-or-get: return the `NodeId` already bound to `key`, or allocate a fresh
    /// one from `builder` and record both directions. Mints in **caller call-order**
    /// — a fresh key calls `builder.node()` at the point of the call, so `NodeId`
    /// allocation follows the sequence the caller interns in (never an internal
    /// re-ordering).
    pub(crate) fn intern(&mut self, builder: &mut GraphBuilder, key: K) -> NodeId {
        if let Some(&node) = self.by_key.get(&key) {
            return node;
        }
        let node = builder.node();
        self.by_key.insert(key, node);
        self.by_node.insert(node, key);
        node
    }

    /// Get-only: the `NodeId` bound to `key`, or `None` if `key` was never interned.
    /// Never mints — the edge-emission pass resolves endpoints through this so it can
    /// never allocate a node out of the pre-intern order.
    pub(crate) fn resolve(&self, key: K) -> Option<NodeId> {
        self.by_key.get(&key).copied()
    }

    /// Reverse lookup: the `key` bound to `node`, or `None` if `node` is foreign.
    pub(crate) fn key_of(&self, node: NodeId) -> Option<K> {
        self.by_node.get(&node).copied()
    }

    /// Remap a node-set to its key-set (`key_of` each, silently skipping any node
    /// foreign to this projection).
    pub(crate) fn remap_set(&self, nodes: &BTreeSet<NodeId>) -> BTreeSet<K> {
        nodes.iter().filter_map(|node| self.key_of(*node)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // intern is mint-or-get: the same key twice yields the same NodeId, minting once.
    #[test]
    fn intern_is_idempotent_per_key() {
        let mut builder = GraphBuilder::new();
        let mut proj: Projection<u32> = Projection::new();
        let first = proj.intern(&mut builder, 7);
        let second = proj.intern(&mut builder, 7);
        assert_eq!(first, second, "same key reuses the same NodeId");
    }

    // NodeIds follow caller call-order, NOT key order. Interning keys in a
    // non-ascending sequence must allocate NodeIds in that same sequence — the
    // leaf imposes no order of its own (the tier-4 tie-break depends on this).
    #[test]
    fn intern_mints_in_caller_order_not_key_order() {
        let mut builder = GraphBuilder::new();
        let mut proj: Projection<u32> = Projection::new();
        // Intern out of key order: 5, then 2, then 9.
        let n5 = proj.intern(&mut builder, 5);
        let n2 = proj.intern(&mut builder, 2);
        let n9 = proj.intern(&mut builder, 9);
        // The NodeIds reflect call order: n5 allocated first, n2 second, n9 third.
        // We prove order via the reverse map round-trip in call sequence.
        assert_eq!(proj.key_of(n5), Some(5));
        assert_eq!(proj.key_of(n2), Some(2));
        assert_eq!(proj.key_of(n9), Some(9));
        // And distinct: three calls, three distinct nodes.
        assert_ne!(n5, n2);
        assert_ne!(n2, n9);
        assert_ne!(n5, n9);
        // Monotonic allocation: the first-interned key got the lowest NodeId, the
        // last the highest — i.e. ascending by call order, independent of key value.
        assert!(n5 < n2, "first-interned key allocates the lowest NodeId");
        assert!(n2 < n9, "allocation is monotonic in call order");
    }

    // resolve is get-only: an unminted key resolves to None.
    #[test]
    fn resolve_returns_none_for_an_unminted_key() {
        let mut builder = GraphBuilder::new();
        let mut proj: Projection<u32> = Projection::new();
        proj.intern(&mut builder, 1);
        assert_eq!(proj.resolve(1), proj.by_key.get(&1).copied());
        assert_eq!(proj.resolve(2), None, "unminted key resolves to None");
    }

    // key_of round-trips a minted node and returns None for an unrecorded one.
    #[test]
    fn key_of_round_trips_and_rejects_unknown_nodes() {
        let mut builder = GraphBuilder::new();
        let mut proj: Projection<u32> = Projection::new();
        let node = proj.intern(&mut builder, 42);
        assert_eq!(proj.key_of(node), Some(42));
        // A node allocated from the same builder but never interned into this
        // projection has no key (the builder mints it; the projection never recorded
        // the reverse binding).
        let unrecorded = builder.node();
        assert_eq!(proj.key_of(unrecorded), None, "unrecorded node has no key");
    }

    // remap_set round-trips a node-set into its key-set.
    #[test]
    fn remap_set_maps_nodes_to_keys() {
        let mut builder = GraphBuilder::new();
        let mut proj: Projection<u32> = Projection::new();
        let na = proj.intern(&mut builder, 3);
        let nb = proj.intern(&mut builder, 8);
        let nodes: BTreeSet<NodeId> = [na, nb].into_iter().collect();
        let keys = proj.remap_set(&nodes);
        assert_eq!(keys, BTreeSet::from([3, 8]));
    }
}
