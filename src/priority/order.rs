// SPDX-License-Identifier: GPL-3.0-only
//! The priority ORDERING primitives (SL-194 PHASE-01) — the pure linear-order
//! synthesis extracted from [`super::surface`] so both `surface::next` and the
//! interestingness detectors ([`super::findings`]) share ONE implementation (design
//! §Architecture, "Ordering-primitive extraction"). No re-implementation, purity
//! preserved.
//!
//! Two primitives, byte-identical to their former `surface.rs` bodies:
//! - [`surviving_seq_predecessors`] — the induced precedence relation (`seq_overlay`
//!   minus cordage's evictions), restricted to a node set. Reads the graph but is a
//!   pure query (no clock/RNG/disk — it walks `in_edges` + `channels::evicted_seq_edges`).
//! - [`frontier_order`] — the score-aware induced-frontier (Kahn) sort over that
//!   precedence relation. Total, terminating, graph-free (takes a `score` closure).
//!
//! Extracting these behaviour-preserving keeps `next` byte-identical (the existing
//! `surface` suite passes unmodified — VA-1) while giving the detectors the same
//! `next` frontier order for the Plateau near-tie basis (and, in PHASE-02, the β
//! endpoint sweeps).

use std::collections::{BTreeMap, BTreeSet};

use crate::relation_graph::EntityKey;

use super::channels;
use super::graph::PriorityGraph;

/// The **surviving** seq predecessors of each node in `actionable` (SL-133 §5.4 / F-3) —
/// the `seq_overlay` `in_edges` MINUS the edges cordage EVICTED to linearize an `Evict`
/// cycle, restricted to edges whose BOTH endpoints are in `actionable`. The induced
/// precedence relation [`frontier_order`] honours; an evicted (broken) seq edge does NOT
/// re-impose precedence.
///
/// Empirical finding (this cordage build): `in_edges(seq_overlay, ·)` ALREADY excludes
/// the evicted edge — for an `Evict` 2-cycle, `provenance().evictions()` reports both
/// directed entries but `in_edges` yields only the one surviving edge. The explicit
/// subtraction via [`channels::evicted_seq_edges`] is therefore DEFENSIVE here (a no-op
/// in the common path), kept to honour the design §5.4 contract ("read surviving edges,
/// not raw `seq_overlay`") and stay correct if cordage's enumeration ever changes.
pub(crate) fn surviving_seq_predecessors(
    g: &PriorityGraph,
    actionable: &BTreeSet<EntityKey>,
) -> BTreeMap<EntityKey, BTreeSet<EntityKey>> {
    let mut preds: BTreeMap<EntityKey, BTreeSet<EntityKey>> = BTreeMap::new();
    for &k in actionable {
        // The evicted (from, to) pairs touching `k` — subtract these from the raw
        // enumeration so a broken seq edge never re-imposes an order.
        let evicted: BTreeSet<(EntityKey, EntityKey)> = channels::evicted_seq_edges(g, k)
            .into_iter()
            .map(|(from, to, _reason)| (from, to))
            .collect();
        let mut set = BTreeSet::new();
        if let Some(n) = g.projection.resolve(k) {
            for (pred, _) in g.graph.in_edges(g.seq_overlay, n) {
                if let Some(pk) = g.projection.key_of(pred)
                    && actionable.contains(&pk)
                    && !evicted.contains(&(pk, k))
                {
                    set.insert(pk);
                }
            }
        }
        preds.insert(k, set);
    }
    preds
}

/// Pure induced-frontier (Kahn-style) sort of the `actionable` set (SL-133 §5.4 / F-3).
/// Precedence is `preds` (the SURVIVING actionable seq edges); among nodes whose
/// surviving predecessors are all emitted, the next pick is the max by
/// `(score desc via total_cmp, id asc)`. NOT cordage `order_key` (its `(Level, NodeId)`
/// ranks Level before `NodeId`, demoting score-promoted successors; RV-132 F-3).
///
/// Total + terminating: every node is emitted exactly once; a residual seq cycle (none
/// expected — the seq overlay is `Evict`-linearized) would still drain via the same
/// `(score, id)` pick once the ready set empties, so the loop always makes progress.
pub(crate) fn frontier_order(
    actionable: &[EntityKey],
    score: &dyn Fn(EntityKey) -> f64,
    preds: &BTreeMap<EntityKey, BTreeSet<EntityKey>>,
) -> Vec<EntityKey> {
    let mut emitted: BTreeSet<EntityKey> = BTreeSet::new();
    let mut out: Vec<EntityKey> = Vec::with_capacity(actionable.len());
    while out.len() < actionable.len() {
        // Ready = un-emitted nodes whose surviving predecessors are all emitted.
        let ready: Vec<EntityKey> = actionable
            .iter()
            .copied()
            .filter(|k| !emitted.contains(k))
            .filter(|k| {
                preds
                    .get(k)
                    .is_none_or(|ps| ps.iter().all(|p| emitted.contains(p)))
            })
            .collect();
        // No ready node ⇒ a residual cycle among the un-emitted; fall back to every
        // un-emitted node so the loop still terminates (defensive — Evict precludes it).
        let candidates: Vec<EntityKey> = if ready.is_empty() {
            actionable
                .iter()
                .copied()
                .filter(|k| !emitted.contains(k))
                .collect()
        } else {
            ready
        };
        let Some(pick) = candidates.into_iter().max_by(|a, b| {
            // Max by score asc then id DESC ⇒ picks highest score, lowest id first.
            score(*a).total_cmp(&score(*b)).then_with(|| b.cmp(a))
        }) else {
            break;
        };
        emitted.insert(pick);
        out.push(pick);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: u32) -> EntityKey {
        EntityKey { prefix: "ISS", id }
    }

    // ── SL-194 VT-2: frontier_order fixture (pure, no graph) ──────────────────

    /// A Y-fixture proven purely: two seq-incomparable arms (2, 3) both follow the
    /// shared upstream (1). The upstream leads (it is every arm's predecessor); among
    /// the incomparable arms the higher-score one wins the tiebreak. This is the exact
    /// order basis `surface::next` produces — extracted here so the detectors reuse it.
    #[test]
    fn frontier_order_shared_upstream_leads_then_score_tiebreak() {
        let nodes = [key(1), key(2), key(3)];
        // 2 and 3 both `after` 1; no edge between 2 and 3.
        let mut preds: BTreeMap<EntityKey, BTreeSet<EntityKey>> = BTreeMap::new();
        preds.insert(key(2), BTreeSet::from([key(1)]));
        preds.insert(key(3), BTreeSet::from([key(1)]));
        // Score: arm 3 outscores arm 2; upstream 1 is lowest.
        let score = |k: EntityKey| -> f64 {
            match k.id {
                1 => 0.0,
                2 => 50.0,
                3 => 100.0,
                _ => 0.0,
            }
        };
        let order = frontier_order(&nodes, &score, &preds);
        let pos = |id: u32| order.iter().position(|k| k.id == id).unwrap();
        assert!(
            pos(1) < pos(2) && pos(1) < pos(3),
            "shared upstream leads: {order:?}"
        );
        assert!(pos(3) < pos(2), "higher-score arm 3 before 2: {order:?}");
    }

    /// A same-chain seq pair keeps STRUCTURAL order regardless of score: a low-score
    /// predecessor still precedes its high-score successor on one chain.
    #[test]
    fn frontier_order_chain_keeps_structure_over_score() {
        let nodes = [key(1), key(2)];
        let mut preds: BTreeMap<EntityKey, BTreeSet<EntityKey>> = BTreeMap::new();
        preds.insert(key(2), BTreeSet::from([key(1)]));
        let score = |k: EntityKey| -> f64 { if k.id == 1 { 2.0 } else { 100.0 } };
        let order = frontier_order(&nodes, &score, &preds);
        let pos = |id: u32| order.iter().position(|k| k.id == id).unwrap();
        assert!(
            pos(1) < pos(2),
            "low-score predecessor leads on the chain: {order:?}"
        );
    }

    /// `surviving_seq_predecessors` over a built corpus: a plain `after` chain yields
    /// the predecessor set, mirroring the surface shell's byte-identical behaviour.
    #[test]
    fn surviving_seq_predecessors_reads_after_chain() {
        use crate::priority::graph;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".doctrine/backlog/issue/001")).unwrap();
        std::fs::create_dir_all(root.join(".doctrine/backlog/issue/002")).unwrap();
        std::fs::write(
            root.join(".doctrine/backlog/issue/001/backlog-001.toml"),
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join(".doctrine/backlog/issue/001/backlog-001.md"),
            "b\n",
        )
        .unwrap();
        // ISS-002 after ISS-001.
        std::fs::write(
            root.join(".doctrine/backlog/issue/002/backlog-002.toml"),
            "id = 2\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nafter = [{ to = \"ISS-001\", rank = 0 }]\n",
        )
        .unwrap();
        std::fs::write(
            root.join(".doctrine/backlog/issue/002/backlog-002.md"),
            "b\n",
        )
        .unwrap();

        let g = graph::build(root).unwrap();
        let set: BTreeSet<EntityKey> = [key(1), key(2)].into_iter().collect();
        let preds = surviving_seq_predecessors(&g, &set);
        assert_eq!(
            preds.get(&key(2)),
            Some(&BTreeSet::from([key(1)])),
            "ISS-002's surviving seq predecessor is ISS-001"
        );
        assert!(
            preds.get(&key(1)).is_none_or(BTreeSet::is_empty),
            "ISS-001 has no seq predecessor"
        );
    }
}
