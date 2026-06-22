// SPDX-License-Identifier: GPL-3.0-only
//! Pure channel synthesis over a [`PriorityGraph`] (SL-047 design §5.2) — the
//! derived per-node work-priority signals: eligibility, direct blockers, the
//! actionable boolean (D12), the blocking-others set, the score read (+ its base /
//! leverage / optionality breakdown), the composed order key, and the dep-cycle
//! degrade diagnostic.
//!
//! Pure: no clock, RNG, or disk — every signal is DERIVED per query from the graph
//! (ADR-004: nothing stores a reverse field; `blocked_by`/`blocking` are computed each
//! call; `score`/`leverage`/`optionality` read the post-pass maps). Determinism:
//! `BTreeSet`/`Vec` over the graph's ordered
//! adjacency — permutation-invariant (REQ-077).
//!
//! `actionable = eligible && !blocked` (D12). `blocked_by` is the DIRECT-blocker test
//! ONLY — `in_edges` on the dep overlay kept to non-terminal predecessors. The direct
//! test SUFFICES for the boolean (I1): a non-terminal predecessor that is itself
//! blocked is still a direct non-terminal blocker, so transitivity adds nothing here.
//! Transitive reachability is for `blockers --transitive`/`explain` (PHASE-03), not
//! this layer.
//!
//! Consumed by the priority CLI surface (SL-047 PHASE-03 — `surface` builds the view
//! rows from these channels), so the PHASE-02 self-clearing `not(test)` `dead_code`
//! suppression has retired itself, as designed (`mem.pattern.lint.
//! dead-code-expect-vs-cfg-test`).

use std::collections::BTreeSet;

use crate::backlog_order::OverrideReason;
use crate::relation_graph::EntityKey;

use super::graph::PriorityGraph;
use super::partition::{StatusClass, status_class};

/// The [`StatusClass`] of a node (its `kind` + authored status). A node with no
/// attrs entry (never minted) is treated as [`StatusClass::Unrecognised`] — a
/// defensive default that can only arise on a caller bug.
fn class_of(g: &PriorityGraph, node: EntityKey) -> StatusClass {
    match g.attrs.get(&node) {
        Some(attr) => status_class(attr.kind, attr.status.as_deref()),
        None => StatusClass::Unrecognised,
    }
}

/// Whether a node is **eligible** — its status class is [`StatusClass::Workable`]
/// (design §5.2). Eligibility is status-only; blocking is a separate axis.
pub(crate) fn eligible(g: &PriorityGraph, node: EntityKey) -> bool {
    class_of(g, node) == StatusClass::Workable
}

/// Whether a backlog node is **promoted** (its `resolution == Promoted`) — the F1 /
/// REQ-075 AC2 separate exclusion reason. A node with no attrs entry is not promoted.
/// Surfaced HERE (not in `status_class`) because `promoted` is a node-attr concern,
/// not a status class.
pub(crate) fn promoted(g: &PriorityGraph, node: EntityKey) -> bool {
    g.attrs.get(&node).is_some_and(|attr| attr.promoted)
}

/// The node's **direct blockers** — its `dep`-overlay predecessors (the prereqs it
/// `needs`, B→A flip) whose status class is NOT [`StatusClass::Terminal`] (design
/// §5.2). A terminal prereq is satisfied and does not block. Sorted, deduped — a
/// `BTreeSet` collected to `Vec` for a deterministic, permutation-invariant result.
pub(crate) fn blocked_by(g: &PriorityGraph, node: EntityKey) -> Vec<EntityKey> {
    let Some(n) = g.projection.resolve(node) else {
        return Vec::new();
    };
    g.graph
        .in_edges(g.dep_overlay, n)
        .filter_map(|(pred, _)| g.projection.key_of(pred))
        .filter(|pred| class_of(g, *pred) != StatusClass::Terminal)
        .collect::<BTreeSet<EntityKey>>()
        .into_iter()
        .collect()
}

/// Whether a node is **blocked** — it has at least one non-terminal direct blocker.
pub(crate) fn blocked(g: &PriorityGraph, node: EntityKey) -> bool {
    !blocked_by(g, node).is_empty()
}

/// Whether a node is **actionable** (the D12 synthesis) — `eligible && !blocked`.
/// The direct-blocker test suffices for this boolean (I1): no transitive closure.
pub(crate) fn actionable(g: &PriorityGraph, node: EntityKey) -> bool {
    eligible(g, node) && !blocked(g, node)
}

/// The nodes this node is **blocking** — its `dep`-overlay successors (the items that
/// `need` it, B→A flip means this node is the predecessor). Sorted, deduped.
pub(crate) fn blocking(g: &PriorityGraph, node: EntityKey) -> Vec<EntityKey> {
    let Some(n) = g.projection.resolve(node) else {
        return Vec::new();
    };
    g.graph
        .out_edges(g.dep_overlay, n)
        .filter_map(|(succ, _)| g.projection.key_of(succ))
        .collect::<BTreeSet<EntityKey>>()
        .into_iter()
        .collect()
}

/// The node's **transitive blockers** — every non-terminal node reachable from it
/// along the `dep` overlay AGAINST direction (its prereqs, their prereqs, …), the
/// `blockers --transitive`/`explain` chain (REQ-073). `reachable` walks the resolved
/// dep adjacency (`Against` = predecessors via `in_edges`), excludes the node itself
/// (cordage I6), and is total/terminating on a degraded cyclic view. Non-terminal
/// filtered (a satisfied prereq is not a blocker) and sorted/deduped — a `BTreeSet`
/// for a deterministic, permutation-invariant result. The DIRECT [`blocked_by`] is a
/// subset; this is its transitive closure.
pub(crate) fn blocked_by_transitive(g: &PriorityGraph, node: EntityKey) -> Vec<EntityKey> {
    let Some(n) = g.projection.resolve(node) else {
        return Vec::new();
    };
    g.graph
        .reachable(g.dep_overlay, n, cordage::Direction::Against)
        .into_iter()
        .filter_map(|pred| g.projection.key_of(pred))
        .filter(|pred| class_of(g, *pred) != StatusClass::Terminal)
        .collect::<BTreeSet<EntityKey>>()
        .into_iter()
        .collect()
}

/// The node's **transitive dependents** — every node reachable from it ALONG the
/// `dep` overlay (the items that need it, their dependents, …), the `blockers
/// --transitive`/`explain` chain (REQ-073). Mirrors [`blocked_by_transitive`] in the
/// opposite direction; the DIRECT [`blocking`] is a subset. No terminal filter — a
/// dependent's own status is its concern; this is the structural cone.
pub(crate) fn blocking_transitive(g: &PriorityGraph, node: EntityKey) -> Vec<EntityKey> {
    let Some(n) = g.projection.resolve(node) else {
        return Vec::new();
    };
    g.graph
        .reachable(g.dep_overlay, n, cordage::Direction::Along)
        .into_iter()
        .filter_map(|succ| g.projection.key_of(succ))
        .collect::<BTreeSet<EntityKey>>()
        .into_iter()
        .collect()
}

/// The **evicted `after` (seq) edges** touching `node` — the soft sequence
/// preferences cordage dropped to linearize, surfaced by `explain` (design §5.4).
/// Each is `(from, to, reason)` in `EntityKey` terms, mapping cordage's
/// [`cordage::EvictReason`] onto the shared [`OverrideReason`] vocabulary (the
/// `backlog_order::overrides` precedent — same projection, one reason vocabulary).
/// Filtered to the seq overlay and to evictions where `node` is an endpoint; sorted
/// by `(from, to)` for determinism. An `ArityViolation` cannot arise (the dep/seq
/// overlays are `Unbounded`), so it is skipped defensively.
pub(crate) fn evicted_seq_edges(
    g: &PriorityGraph,
    node: EntityKey,
) -> Vec<(EntityKey, EntityKey, OverrideReason)> {
    let Some(n) = g.projection.resolve(node) else {
        return Vec::new();
    };
    let mut out: Vec<(EntityKey, EntityKey, OverrideReason)> = g
        .graph
        .provenance()
        .evictions()
        .iter()
        .filter(|e| e.overlay() == g.seq_overlay)
        .filter(|e| e.edge().src() == n || e.edge().dst() == n)
        .filter_map(|e| {
            let from = g.projection.key_of(e.edge().src())?;
            let to = g.projection.key_of(e.edge().dst())?;
            let reason = match e.reason() {
                cordage::EvictReason::IntraOverlayCycle => OverrideReason::SoftCycleEvicted,
                cordage::EvictReason::UnionCycleVsLayer => OverrideReason::Contradicted,
                cordage::EvictReason::ArityViolation => return None,
            };
            Some((from, to, reason))
        })
        .collect();
    // OverrideReason is not Ord; the (from, to) endpoint pair is the deterministic
    // sort key (a given pair carries one eviction reason).
    out.sort_by_key(|a| (a.0, a.1));
    out
}

/// The node's **score** — the final priority signal (`base + leverage + optionality`),
/// computed by the consequence post-pass and stored on the graph (`g.score`, default
/// `0.0` when absent). The display-time sort key for `survey`/`next` (SL-133 §5.4).
pub(crate) fn score(g: &PriorityGraph, node: EntityKey) -> f64 {
    g.score.get(&node).copied().unwrap_or(0.0)
}

/// The node's **base** score total — `value_dim + risk_dim` from its own facets (the
/// mint tiebreaker and the `explain` breakdown; SL-133 §5.1). Default `0.0`.
pub(crate) fn base(g: &PriorityGraph, node: EntityKey) -> f64 {
    g.attrs.get(&node).map_or(0.0, |a| a.base_score.total())
}

/// The node's base **value dimension** (SL-133 §5.1) — for the `explain` breakdown.
pub(crate) fn value_dim(g: &PriorityGraph, node: EntityKey) -> f64 {
    g.attrs.get(&node).map_or(0.0, |a| a.base_score.value_dim)
}

/// The node's base **risk dimension** (SL-133 §5.1) — for the `explain` breakdown.
pub(crate) fn risk_dim(g: &PriorityGraph, node: EntityKey) -> f64 {
    g.attrs.get(&node).map_or(0.0, |a| a.base_score.risk_dim)
}

/// The node's recursive **needs-leverage** (`g.leverage`, default `0.0`; SL-133 §5.4).
pub(crate) fn leverage(g: &PriorityGraph, node: EntityKey) -> f64 {
    g.leverage.get(&node).copied().unwrap_or(0.0)
}

/// The node's one-hop **ref-optionality** (`g.optionality`, default `0.0`; SL-133 §5.4).
pub(crate) fn optionality(g: &PriorityGraph, node: EntityKey) -> f64 {
    g.optionality.get(&node).copied().unwrap_or(0.0)
}

/// The diagnosed **dep cycles** (REQ-076 / F2) — each a component of `EntityKey`s
/// caught in a provenance cycle on the dep overlay. cordage's `Reject` policy
/// preserves the cyclic edges and still yields a total `ordered()`; the affected
/// component degrades to the base/`NodeId` fallback rather than emitting a
/// false topological order. Mirrors `backlog_order::dep_cycles()`.
pub(crate) fn dep_cycles(g: &PriorityGraph) -> Vec<BTreeSet<EntityKey>> {
    g.graph
        .provenance()
        .cycles()
        .iter()
        .filter(|cycle| cycle.overlay() == g.dep_overlay)
        .map(|cycle| g.projection.remap_set(cycle.nodes()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    use crate::priority::graph::build;

    // -- Fixture seeding (small, intention-revealing corpora over `build`) -----

    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn key(prefix: &'static str, id: u32) -> EntityKey {
        EntityKey { prefix, id }
    }

    /// Test-local remap of cordage's total `ordered()` to `EntityKey` (the former
    /// `order_key` channel). `next` no longer consumes a level-then-`NodeId` order
    /// (RV-132 F-3), so this remap is now only test scaffolding for the cycle-degrade
    /// and permutation-invariance assertions.
    fn order_key(g: &PriorityGraph) -> Vec<EntityKey> {
        g.graph
            .ordered()
            .iter()
            .filter_map(|node| g.projection.key_of(*node))
            .collect()
    }

    /// Seed a slice with a given lifecycle status and relations (SL-048 migrated
    /// shape — tier-1 axes become `[[relation]]` rows, typed leftovers a table).
    fn seed_slice(root: &Path, id: u32, status: &str, axes: &[(&str, &[&str])]) {
        let rels = crate::relation::rels_block(&crate::slice::SLICE_KIND, axes);
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s\"\ntitle = \"S\"\nstatus = \"{status}\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed a backlog issue with status, resolution, and relations (migrated shape).
    fn seed_issue(root: &Path, id: u32, status: &str, resolution: &str, axes: &[(&str, &[&str])]) {
        let rels = crate::relation::rels_block(&crate::backlog::ISSUE_KIND, axes);
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"{status}\"\n\
                 resolution = \"{resolution}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 {rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    /// Seed a risk backlog item (a second backlog kind for prereqs).
    fn seed_risk(root: &Path, id: u32, status: &str, axes: &[(&str, &[&str])]) {
        let rels = crate::relation::rels_block(&crate::backlog::RISK_KIND, axes);
        write(
            root,
            &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"k\"\ntitle = \"K\"\nkind = \"risk\"\nstatus = \"{status}\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 {rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.md"),
            "k\n",
        );
    }

    /// Seed a requirement (an edge target with a top-level status).
    fn seed_requirement(root: &Path, id: u32) {
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
            &format!("id = {id}\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n"),
        );
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
            "r\n",
        );
    }

    /// Seed a status-less reconciliation record.
    fn seed_rec(root: &Path, id: u32, owning_slice: &str) {
        write(
            root,
            &format!(".doctrine/rec/{id:03}/rec-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"r\"\ntitle = \"R\"\n\
                 [rec]\nmove = \"accept\"\nowning_slice = \"{owning_slice}\"\n"
            ),
        );
    }

    // -- VT-2: D12 synthesis (eligible / blocked / actionable) ----------------

    #[test]
    fn workable_unblocked_is_actionable() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 open (workable) with no prereqs → actionable.
        seed_issue(root, 1, "open", "", &[]);
        let g = build(root).unwrap();
        let n = key("ISS", 1);
        assert!(eligible(&g, n), "open issue is eligible");
        assert!(!blocked(&g, n), "no prereqs → not blocked");
        assert!(actionable(&g, n), "workable + unblocked → actionable");
    }

    #[test]
    fn workable_blocked_is_eligible_but_not_actionable() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs RSK-001; RSK-001 is open (non-terminal) → blocks.
        seed_issue(root, 1, "open", "", &[("needs", &["RSK-001"])]);
        seed_risk(root, 1, "open", &[]);
        let g = build(root).unwrap();
        let n = key("ISS", 1);
        assert!(eligible(&g, n), "open issue is eligible");
        assert_eq!(blocked_by(&g, n), vec![key("RSK", 1)], "RSK-001 blocks");
        assert!(blocked(&g, n));
        assert!(!actionable(&g, n), "eligible but blocked → not actionable");
    }

    #[test]
    fn terminal_prereq_does_not_block() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs RSK-001; RSK-001 is closed (terminal) → satisfied, no block.
        seed_issue(root, 1, "open", "", &[("needs", &["RSK-001"])]);
        seed_risk(root, 1, "closed", &[]);
        let g = build(root).unwrap();
        let n = key("ISS", 1);
        assert!(
            blocked_by(&g, n).is_empty(),
            "a terminal prereq is satisfied, not a blocker"
        );
        assert!(actionable(&g, n), "satisfied prereq → actionable");
    }

    #[test]
    fn terminal_node_is_not_eligible() {
        let dir = tmp();
        let root = dir.path();
        // A done slice (terminal) and a closed issue (terminal) are not eligible.
        seed_slice(root, 1, "done", &[]);
        seed_issue(root, 1, "closed", "", &[]);
        let g = build(root).unwrap();
        assert!(!eligible(&g, key("SL", 1)), "done slice not eligible");
        assert!(!actionable(&g, key("SL", 1)));
        assert!(!eligible(&g, key("ISS", 1)), "closed issue not eligible");
    }

    #[test]
    fn audit_and_reconcile_slices_are_workable() {
        let dir = tmp();
        let root = dir.path();
        // VT-2 boundary: audit / reconcile slice statuses are WORKABLE.
        seed_slice(root, 1, "audit", &[]);
        seed_slice(root, 2, "reconcile", &[]);
        let g = build(root).unwrap();
        assert!(eligible(&g, key("SL", 1)), "audit slice is workable");
        assert!(actionable(&g, key("SL", 1)));
        assert!(eligible(&g, key("SL", 2)), "reconcile slice is workable");
        assert!(actionable(&g, key("SL", 2)));
    }

    // -- VT-3: conservative / status-less / promoted exclusion ----------------

    #[test]
    fn rec_status_less_is_not_eligible() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, "proposed", &[]);
        seed_rec(root, 1, "SL-001");
        let g = build(root).unwrap();
        // REC None → Terminal class → not eligible, not actionable.
        assert!(!eligible(&g, key("REC", 1)), "status-less REC not eligible");
        assert!(!actionable(&g, key("REC", 1)));
    }

    #[test]
    fn unrecognised_slice_status_is_not_eligible() {
        let dir = tmp();
        let root = dir.path();
        // A slice with a status outside the table (stringly status tolerated on disk)
        // rides Unrecognised → not eligible (the conservative default).
        seed_slice(root, 1, "frobnicate", &[]);
        let g = build(root).unwrap();
        assert!(
            !eligible(&g, key("SL", 1)),
            "unrecognised status → not eligible"
        );
    }

    #[test]
    fn promoted_backlog_node_surfaces_its_own_reason() {
        let dir = tmp();
        let root = dir.path();
        // EX-3: a promoted (resolution=promoted) backlog node is excluded regardless
        // of status class — surfaced as its OWN reason (F1), distinct from terminal.
        seed_issue(root, 1, "resolved", "promoted", &[]);
        seed_issue(root, 2, "open", "", &[]);
        let g = build(root).unwrap();
        assert!(
            promoted(&g, key("ISS", 1)),
            "resolution=promoted ⇒ promoted"
        );
        assert!(
            !promoted(&g, key("ISS", 2)),
            "plain open issue not promoted"
        );
    }

    // -- blocking (the inverse direction) -------------------------------------

    #[test]
    fn blocking_lists_dependents() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs RSK-001 → RSK-001 is blocking ISS-001.
        seed_issue(root, 1, "open", "", &[("needs", &["RSK-001"])]);
        seed_risk(root, 1, "open", &[]);
        let g = build(root).unwrap();
        assert_eq!(
            blocking(&g, key("RSK", 1)),
            vec![key("ISS", 1)],
            "RSK-001 blocks ISS-001"
        );
        assert!(
            blocking(&g, key("ISS", 1)).is_empty(),
            "ISS-001 blocks nothing"
        );
    }

    #[test]
    fn score_reads_the_post_pass_value() {
        let dir = tmp();
        let root = dir.path();
        // No facets ⇒ base/leverage/optionality all 0 ⇒ score reads 0.0 (the floor),
        // and the readers default to 0.0 for an unseen key (the SL-133 score contract:
        // `channels::score` reads `g.score`, never the old u32 tally).
        seed_slice(root, 1, "proposed", &[("requirements", &["REQ-005"])]);
        seed_requirement(root, 5);
        let g = build(root).unwrap();
        assert_eq!(score(&g, key("REQ", 5)), 0.0, "no facets → score floor 0");
        assert_eq!(base(&g, key("REQ", 5)), 0.0);
        assert_eq!(leverage(&g, key("REQ", 5)), 0.0);
        assert_eq!(optionality(&g, key("REQ", 5)), 0.0);
    }

    // -- VT-4: cycle degrade ---------------------------------------------------

    #[test]
    fn dep_cycle_named_fallback_order_no_false_topo() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs ISS-002, ISS-002 needs ISS-001 → a dep cycle. cordage Reject
        // preserves the edges and still yields ordered(); dep_cycles names the
        // component; ordering elsewhere is unaffected.
        seed_issue(root, 1, "open", "", &[("needs", &["ISS-002"])]);
        seed_issue(root, 2, "open", "", &[("needs", &["ISS-001"])]);
        // A separate, acyclic slice — its order is unaffected.
        seed_slice(root, 9, "proposed", &[]);
        let g = build(root).unwrap();

        let cycles = dep_cycles(&g);
        assert_eq!(cycles.len(), 1, "exactly one dep cycle");
        let component = &cycles[0];
        assert!(component.contains(&key("ISS", 1)));
        assert!(component.contains(&key("ISS", 2)));

        // ordered() still yields a TOTAL order over every node (no panic, no drop) —
        // the cyclic component degrades to the fallback, never a false topo.
        let order = order_key(&g);
        assert!(order.contains(&key("ISS", 1)));
        assert!(order.contains(&key("ISS", 2)));
        assert!(order.contains(&key("SL", 9)), "acyclic node still ordered");
        assert_eq!(
            order.len(),
            3,
            "every node appears exactly once in the order"
        );
    }

    #[test]
    fn no_cycle_means_no_diagnostic() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", &[("needs", &["RSK-001"])]);
        seed_risk(root, 1, "open", &[]);
        let g = build(root).unwrap();
        assert!(dep_cycles(&g).is_empty(), "acyclic corpus → no cycles");
    }

    // -- VT-5: determinism (permutation invariance) ---------------------------

    #[test]
    fn channels_are_permutation_invariant() {
        // Build the SAME corpus authored in two different on-disk orders and assert
        // every channel output is identical (BTree-keyed, no clock/RNG; REQ-077).
        let build_corpus = |authoring: u8| {
            let dir = tmp();
            let root = dir.path().to_path_buf();
            let pieces: [&dyn Fn(&Path); 5] = [
                &|r: &Path| seed_issue(r, 1, "open", "", &[("needs", &["RSK-001"])]),
                // `after` carries a per-edge payload (a typed tier-2 axis, NOT migrated
                // to `[[relation]]`), so it is seeded directly as a `[relationships]`
                // table rather than through the simple-list `axes` seam.
                &|r: &Path| {
                    write(
                        r,
                        ".doctrine/backlog/issue/002/backlog-002.toml",
                        "id = 2\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
                         resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                         [relationships]\nafter = [{ to = \"ISS-001\", rank = 0 }]\n",
                    );
                    write(r, ".doctrine/backlog/issue/002/backlog-002.md", "b\n");
                },
                &|r: &Path| seed_risk(r, 1, "open", &[]),
                &|r: &Path| seed_slice(r, 5, "design", &[("requirements", &["REQ-007"])]),
                &|r: &Path| seed_requirement(r, 7),
            ];
            // Two distinct seeding orders over the same logical corpus.
            if authoring == 0 {
                for p in &pieces {
                    p(&root);
                }
            } else {
                for p in pieces.iter().rev() {
                    p(&root);
                }
            }
            (dir, build(&root).unwrap())
        };

        let (_d0, g0) = build_corpus(0);
        let (_d1, g1) = build_corpus(1);

        let nodes = [
            key("ISS", 1),
            key("ISS", 2),
            key("RSK", 1),
            key("SL", 5),
            key("REQ", 7),
        ];
        for n in nodes {
            assert_eq!(eligible(&g0, n), eligible(&g1, n), "eligible {n:?}");
            assert_eq!(actionable(&g0, n), actionable(&g1, n), "actionable {n:?}");
            assert_eq!(blocked_by(&g0, n), blocked_by(&g1, n), "blocked_by {n:?}");
            assert_eq!(blocking(&g0, n), blocking(&g1, n), "blocking {n:?}");
            assert_eq!(score(&g0, n), score(&g1, n), "score {n:?}");
        }
        assert_eq!(order_key(&g0), order_key(&g1), "order_key invariant");
        assert_eq!(dep_cycles(&g0), dep_cycles(&g1), "dep_cycles invariant");
    }
}
