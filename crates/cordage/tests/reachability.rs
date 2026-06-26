//! VT-9 — `reachable` is strict (excludes the start node, I6/F8), total and
//! cycle-safe over a degraded `Reject` view (visited-set termination, F12), and
//! `reachable(_, None) = ∅` (F25). Foreign ids yield ∅ (F14). Black-box: opaque
//! ids minted by the builder, no vocabulary.

use cordage::{Arity, CyclePolicy, Direction, EdgeAttrs, GraphBuilder, OverlayConfig};
use std::collections::{BTreeMap, BTreeSet};

fn reject() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

fn at_most_one() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::AtMostOne)
}

fn set<const N: usize>(ids: [cordage::NodeId; N]) -> BTreeSet<cordage::NodeId> {
    ids.into_iter().collect()
}

fn depths<const N: usize>(
    entries: [(cordage::NodeId, usize); N],
) -> BTreeMap<cordage::NodeId, usize> {
    entries.into_iter().collect()
}

#[test]
fn reachable_is_strict_and_transitive_along() {
    // a → b → c
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    // Strict: a excluded; transitive closure reaches b and c.
    assert_eq!(g.reachable(ov, na, Direction::Along), set([nb, nc]));
    assert_eq!(g.reachable(ov, nb, Direction::Along), set([nc]));
    assert_eq!(g.reachable(ov, nc, Direction::Along), set([]));
}

#[test]
fn reachable_against_walks_in_edges() {
    // a → b → c; Against from c reaches b and a.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert_eq!(g.reachable(ov, nc, Direction::Against), set([na, nb]));
    assert_eq!(g.reachable(ov, na, Direction::Against), set([]));
}

#[test]
fn reachable_none_is_empty() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert_eq!(g.reachable(ov, na, Direction::None), set([]));
}

#[test]
fn reachable_terminates_and_stays_strict_on_a_reject_cycle() {
    // a ↔ b on a Reject overlay: the traversal view stays cyclic (diagnosed, not
    // linearized). reachable must terminate and exclude the start even though it
    // is cyclically reachable.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, na, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");
    // Sanity: the cycle was diagnosed, not removed.
    assert!(!g.provenance().cycles().is_empty());

    assert_eq!(g.reachable(ov, na, Direction::Along), set([nb]));
    assert_eq!(g.reachable(ov, nb, Direction::Along), set([na]));
}

#[test]
fn spine_path_follows_the_single_kept_parent_root_first() {
    // AtMostOne overlay, chain root → m → n. spine_path returns the chain
    // oriented root → … → node (ancestor-first).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(at_most_one());
    let root = b.node();
    let m = b.node();
    let n = b.node();
    b.edge(ov, root, m, EdgeAttrs::new(0, 0));
    b.edge(ov, m, n, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert_eq!(g.spine_path(ov, n), Some(vec![root, m, n]));
    assert_eq!(g.spine_path(ov, root), Some(vec![root]));
}

#[test]
fn spine_path_is_none_on_an_unbounded_overlay() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject()); // Unbounded
    let m = b.node();
    let n = b.node();
    b.edge(ov, m, n, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    assert_eq!(g.spine_path(ov, n), None);
}

#[test]
fn spine_path_follows_the_arity_kept_parent() {
    // n has two parents on an AtMostOne overlay; pass-1 keeps the (rank,age,src,
    // dst)-MAX parent. spine_path must follow the kept one (p_hi), not the loser.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(at_most_one());
    let p_lo = b.node();
    let p_hi = b.node();
    let n = b.node();
    b.edge(ov, p_lo, n, EdgeAttrs::new(0, 0)); // lower rank → evicted
    b.edge(ov, p_hi, n, EdgeAttrs::new(5, 0)); // higher rank → kept
    let g = b.build().expect("valid");

    assert_eq!(g.spine_path(ov, n), Some(vec![p_hi, n]));
}

// ── reachable_bounded (SL-138 PHASE-01) ───────────────────────────────────────

#[test]
fn reachable_bounded_records_min_hop_depth() {
    // Diamond: a→b→d (d at 2 hops) and a→c→e→d (d at 3 hops). BFS first-visit wins,
    // so the shorter path sets depth[d] = 2 (VT-2).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    let nd = b.node();
    let ne = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nd, EdgeAttrs::new(0, 0));
    b.edge(ov, na, nc, EdgeAttrs::new(0, 0));
    b.edge(ov, nc, ne, EdgeAttrs::new(0, 0));
    b.edge(ov, ne, nd, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let r = g.reachable_bounded(ov, na, Direction::Along, None);
    assert_eq!(r.depths, depths([(nb, 1), (nc, 1), (nd, 2), (ne, 2)]));
    assert!(!r.truncated);
}

#[test]
fn reachable_bounded_cap_excludes_deeper_and_flags_truncation() {
    // Chain a→b→c→d. Some(2) keeps b,c and excludes d; c sits at the cap with an
    // unvisited successor (d) ⟹ truncated. Some(3) reaches the leaf, no truncation.
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    let nd = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    b.edge(ov, nc, nd, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let capped = g.reachable_bounded(ov, na, Direction::Along, Some(2));
    assert_eq!(capped.depths, depths([(nb, 1), (nc, 2)]));
    assert!(capped.truncated);

    let full = g.reachable_bounded(ov, na, Direction::Along, Some(3));
    assert_eq!(full.depths, depths([(nb, 1), (nc, 2), (nd, 3)]));
    assert!(!full.truncated);
}

#[test]
fn reachable_bounded_truncation_false_when_closure_ends_at_cap() {
    // Chain a→b→c, c a leaf. Some(2) reaches exactly to the cap with no unvisited
    // successor beyond it ⟹ NOT truncated (the BFS-ordering invariant, F5).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let r = g.reachable_bounded(ov, na, Direction::Along, Some(2));
    assert_eq!(r.depths, depths([(nb, 1), (nc, 2)]));
    assert!(!r.truncated);
}

#[test]
fn reachable_bounded_unbounded_matches_legacy_reachable() {
    // None == today's strict `reachable` set on a fixture (VT-1, behaviour-preservation).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let r = g.reachable_bounded(ov, na, Direction::Along, None);
    let keys: BTreeSet<cordage::NodeId> = r.depths.into_keys().collect();
    assert_eq!(keys, g.reachable(ov, na, Direction::Along));
}

#[test]
fn reachable_bounded_terminates_on_a_reject_cycle_under_cap() {
    // a ↔ b on a Reject overlay (cyclic, diagnosed). A capped walk must terminate
    // and stay strict — the visited set bounds re-entry to `start` (VT-4).
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, na, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");
    assert!(!g.provenance().cycles().is_empty());

    let r = g.reachable_bounded(ov, na, Direction::Along, Some(1));
    // b reached at depth 1; b's only successor is `start` (already visited) — no
    // unvisited successor beyond the cap, so not truncated and start stays excluded.
    assert_eq!(r.depths, depths([(nb, 1)]));
    assert!(!r.truncated);
}

#[test]
fn reachable_bounded_none_direction_is_empty() {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let na = b.node();
    let nb = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    let g = b.build().expect("valid");

    let r = g.reachable_bounded(ov, na, Direction::None, Some(5));
    assert!(r.depths.is_empty());
    assert!(!r.truncated);
}

#[test]
fn reachable_bounded_foreign_ids_are_empty() {
    let mut sib = GraphBuilder::new();
    let foreign_ov = sib.overlay(reject());
    let mut foreign_node = sib.node();
    for _ in 0..4 {
        foreign_node = sib.node();
    }

    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let n = b.node();
    let g = b.build().expect("valid");

    let by_overlay = g.reachable_bounded(foreign_ov, n, Direction::Along, None);
    assert!(by_overlay.depths.is_empty() && !by_overlay.truncated);
    let by_node = g.reachable_bounded(ov, foreign_node, Direction::Along, None);
    assert!(by_node.depths.is_empty() && !by_node.truncated);
}

#[test]
fn reachable_foreign_ids_are_empty() {
    let mut sib = GraphBuilder::new();
    let foreign_ov = sib.overlay(reject());
    let mut foreign_node = sib.node();
    for _ in 0..4 {
        foreign_node = sib.node();
    }

    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject());
    let n = b.node();
    let g = b.build().expect("valid");

    assert_eq!(g.reachable(foreign_ov, n, Direction::Along), set([]));
    assert_eq!(g.reachable(ov, foreign_node, Direction::Along), set([]));
}
