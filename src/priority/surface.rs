// SPDX-License-Identifier: GPL-3.0-only
//! The priority SURFACE shell (SL-047 §5.4) — the impure layer that builds the
//! [`super::view`] rows from a [`super::graph::PriorityGraph`] and the pure
//! [`super::channels`] signals.
//!
//! This is where disk meets pure policy: it calls [`super::graph::build`] (the one
//! scan), then composes the four operator surfaces (`survey`/`next`/`blockers`/
//! `explain`) plus the `inspect` actionability block by reading the pure channel
//! synthesis and the per-node titles captured in [`super::graph::NodeAttr`]. It
//! builds the structured reasons ONCE (the render source of truth, REQ-072 AC3) — the
//! renderer only formats them. Opaque cordage ids never escape (every ref is a
//! canonical `KIND-NNN` via `EntityKey::canonical`).
//!
//! Importance order (survey, D10): the v1 authored-priority slot is EMPTY (PRD-009
//! OQ-001 unbuilt), so the effective sort is `actionability(Actionable > Blocked) →
//! consequence desc → canonical-id asc`. `next` is ordered by the cordage-composed
//! `order_key` (D9), filtered to actionable nodes.

use std::path::Path;

use crate::relation_graph::{self, EntityKey};

use super::channels;
use super::graph::{self, NodeAttr, PriorityGraph};
use super::partition::{StatusClass, status_class};
use super::view::{
    Actionability, ActionabilityBlock, ActionabilityEdge, ActionabilityNode, ActionabilityView,
    BlockersView, Explanation, NextRow, ReasonKind, SurveyRow,
};

/// The per-node attrs entry, or the defensive `None` path (a caller bug — every
/// surfaced key comes from the same scan that filled `attrs`).
fn attr(g: &PriorityGraph, key: EntityKey) -> Option<&NodeAttr> {
    g.attrs.get(&key)
}

/// The kind column for a node — its canonical-id prefix (the kind discriminant; no
/// separate display name exists on `entity::Kind`).
fn kind_of(g: &PriorityGraph, key: EntityKey) -> String {
    attr(g, key).map_or_else(|| key.prefix.to_string(), |a| a.kind.prefix.to_string())
}

/// The title column for a node (captured in the scan), or its canonical ref when no
/// attrs entry exists (defensive).
fn title_of(g: &PriorityGraph, key: EntityKey) -> String {
    attr(g, key).map_or_else(|| key.canonical(), |a| a.title.clone())
}

/// The status display for a node — its authored status, or `—` for the status-less
/// REC kind.
fn status_of(g: &PriorityGraph, key: EntityKey) -> String {
    attr(g, key)
        .and_then(|a| a.status.clone())
        .unwrap_or_else(|| "—".to_string())
}

/// The node's [`StatusClass`] (kind + authored status), for the eligibility reason.
fn class_of(g: &PriorityGraph, key: EntityKey) -> StatusClass {
    match attr(g, key) {
        Some(a) => status_class(a.kind, a.status.as_deref()),
        None => StatusClass::Unrecognised,
    }
}

/// The eligibility reason for a node (status + class).
fn eligibility_reason(g: &PriorityGraph, key: EntityKey) -> ReasonKind {
    ReasonKind::Eligibility {
        status: attr(g, key).and_then(|a| a.status.clone()),
        class: class_of(g, key),
    }
}

/// Canonical refs for a slice of keys (sorted-by-key order preserved).
fn refs(keys: &[EntityKey]) -> Vec<String> {
    keys.iter().map(|k| k.canonical()).collect()
}

/// The actionability of an eligible node.
fn actionability(g: &PriorityGraph, key: EntityKey) -> Actionability {
    if channels::blocked(g, key) {
        Actionability::Blocked
    } else {
        Actionability::Actionable
    }
}

/// A survey node decorated ONCE with its sort + render signals, so the comparator
/// and the row map reuse them instead of re-walking the graph per comparison (the
/// decorate-sort-undecorate refactor, SL-050 F3).
struct SurveyDecorated {
    key: EntityKey,
    act: Actionability,
    consequence: u32,
    blockers: Vec<String>,
}

/// Sort rank for [`Actionability`] — Actionable (0) before Blocked (1).
fn act_rank(a: Actionability) -> u8 {
    match a {
        Actionability::Actionable => 0,
        Actionability::Blocked => 1,
    }
}

/// Pure survey over an already-built [`PriorityGraph`] (the body of [`survey`],
/// extracted for the web map server so it reuses a single build — SL-089 D2).
/// Zero behavioural divergence — byte-identical output (VT-7).
///
/// Filtering (when `all == false`):
///   1. [`channels::eligible`] — status-class gate ([`super::partition::StatusClass::Workable`]) only
///   2. `!`[`channels::promoted`] — exclude promoted-backlog items
///      These two filters exactly match the CLI `survey` default.
pub(crate) fn survey_for_map(g: &PriorityGraph, all: bool) -> Vec<SurveyRow> {
    // Decorate ONCE: materialise each surfaced node's sort/render signals so neither
    // the comparator nor the row map recomputes a graph walk per comparison (SL-050 F3).
    let mut rows: Vec<SurveyDecorated> = g
        .attrs
        .keys()
        .copied()
        .filter(|&k| {
            if all {
                return true;
            }
            // Default: eligible, and not a promoted backlog item (its own exclusion).
            channels::eligible(g, k) && !channels::promoted(g, k)
        })
        .map(|k| SurveyDecorated {
            key: k,
            act: actionability(g, k),
            consequence: channels::consequence(g, k),
            blockers: refs(&channels::blocked_by(g, k)),
        })
        .collect();

    // Importance order (D10), authored-priority slot empty → actionability → cons → id.
    // The comparator does ZERO graph work — it compares only pre-computed scalars.
    rows.sort_by(|a, b| {
        // Actionable before Blocked.
        let act = act_rank(a.act).cmp(&act_rank(b.act));
        let cons = b.consequence.cmp(&a.consequence); // consequence DESC
        act.then(cons).then_with(|| a.key.cmp(&b.key))
    });

    rows.into_iter()
        .map(|d| {
            let mut reasons = vec![eligibility_reason(g, d.key)];
            if !d.blockers.is_empty() {
                reasons.push(ReasonKind::BlockedBy {
                    items: d.blockers.clone(),
                });
            }
            reasons.push(ReasonKind::Consequence {
                inbound: d.consequence,
            });
            SurveyRow {
                id: d.key.canonical(),
                title: title_of(g, d.key),
                kind: kind_of(g, d.key),
                status: status_of(g, d.key),
                act: d.act,
                consequence: d.consequence,
                blockers: d.blockers,
                reasons,
            }
        })
        .collect()
}

/// `survey [--all]` (design §5.4) — the eligible set in importance order (D10).
///
/// Set: every `eligible` node, MINUS `promoted` backlog items (excluded as their own
/// reason, F1), UNLESS `all` reveals the full picture. With `all`, terminal +
/// promoted nodes are included too (the complete view). Sort (the empty
/// authored-priority slot collapses to): `actionability(Actionable first) →
/// consequence desc → canonical-id asc`.
pub(crate) fn survey(root: &Path, all: bool) -> anyhow::Result<Vec<SurveyRow>> {
    let g = graph::build(root)?;
    Ok(survey_for_map(&g, all))
}

/// Build the actionability graph view for the web UI from a [`PriorityGraph`]
/// (SL-089 D3). Pure over the graph — no disk, no clock.
///
/// Returns nodes with server-computed topological ranks over the dep overlay,
/// plus `needs` and `after` edges among work entities.
///
/// Node set (default, `all == false`): eligible AND !promoted — exactly the
/// [`survey_for_map`] filter. Every node carries its rank (topological layer
/// over the dep overlay: 0 = no non-terminal blockers).
///
/// Edges:
///   - `needs` edges: dep overlay, non-terminal source only → oriented
///     prerequisite→dependent (matching the B→A flip stored in the graph).
///   - `after` edges: seq overlay, oriented prerequisite→dependent.
///     Both source and target must be in the node set.
pub(crate) fn survey_view_for_map(g: &PriorityGraph, all: bool) -> ActionabilityView {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    // 1. Build canonical rows (eligible set + ordering).
    let rows = survey_for_map(g, all);

    // 2. EntityKey lookup: canonical ref ↔ key.
    let key_by_id: BTreeMap<String, EntityKey> = rows
        .iter()
        .filter_map(|r| parse_key(&r.id).ok().map(|k| (r.id.clone(), k)))
        .collect();
    let node_keys: BTreeSet<EntityKey> = key_by_id.values().copied().collect();

    // 3. Compute ranks via Kahn-style topological walk over the dep overlay.
    //    Indegree = number of non-terminal blockers (in the node set).
    let mut blockers_of: BTreeMap<EntityKey, Vec<EntityKey>> = BTreeMap::new();
    let mut dependents_of: BTreeMap<EntityKey, Vec<EntityKey>> = BTreeMap::new();
    let mut indeg: BTreeMap<EntityKey, usize> = BTreeMap::new();

    for &k in &node_keys {
        let blockers: Vec<EntityKey> = channels::blocked_by(g, k)
            .into_iter()
            .filter(|b| node_keys.contains(b))
            .collect();
        indeg.insert(k, blockers.len());
        for &b in &blockers {
            dependents_of.entry(b).or_default().push(k);
        }
        blockers_of.insert(k, blockers);
    }

    let mut ranks: BTreeMap<EntityKey, u32> = BTreeMap::new();

    // Kahn: seed with in-degree 0 nodes (no non-terminal blockers in set).
    let mut queue: VecDeque<EntityKey> = indeg
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(&k, _)| k)
        .collect();

    while let Some(k) = queue.pop_front() {
        // Rank = 1 + max(blocker ranks), or 0 if no blockers.
        let rank = blockers_of.get(&k).map_or(0, |bs| {
            bs.iter()
                .filter_map(|b| ranks.get(b))
                .max()
                .map_or(0, |r| r + 1)
        });
        ranks.insert(k, rank);

        // Decrement dependents; enqueue when their in-degree reaches 0.
        if let Some(deps) = dependents_of.get(&k) {
            for &dep in deps {
                if let Some(d) = indeg.get_mut(&dep) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    // Fallback: cyclic nodes (still indeg > 0).
    for &k in &node_keys {
        if !ranks.contains_key(&k) {
            let rank = blockers_of.get(&k).map_or(0, |bs| {
                bs.iter()
                    .filter_map(|b| ranks.get(b))
                    .max()
                    .map_or(0, |r| r + 1)
            });
            ranks.insert(k, rank);
        }
    }

    // 4. Extract needs edges (dep overlay, non-terminal src, both ends in node set).
    //    blocked_by already filters to non-terminal, so every edge source is
    //    non-terminal by construction.
    let mut edges: Vec<ActionabilityEdge> = Vec::new();
    for &k in &node_keys {
        for blocker in &channels::blocked_by(g, k) {
            if node_keys.contains(blocker) {
                edges.push(ActionabilityEdge {
                    source: blocker.canonical(),
                    target: k.canonical(),
                    kind: "needs".into(),
                });
            }
        }
    }

    // 5. Extract after edges (seq overlay, both ends in node set, oriented
    //    prerequisite→dependent).
    for &k in &node_keys {
        if let Some(n) = g.projection.resolve(k) {
            for (pred, _) in g.graph.in_edges(g.seq_overlay, n) {
                if let Some(pred_key) = g.projection.key_of(pred)
                    && node_keys.contains(&pred_key)
                {
                    edges.push(ActionabilityEdge {
                        source: pred_key.canonical(),
                        target: k.canonical(),
                        kind: "after".into(),
                    });
                }
            }
        }
    }

    // 6. Assemble nodes — reuse the pre-computed row data + rank.
    let nodes: Vec<ActionabilityNode> = rows
        .into_iter()
        .filter_map(|r| {
            let k = parse_key(&r.id).ok()?;
            let rank = ranks.get(&k).copied().unwrap_or(0);
            let actionability = match r.act {
                Actionability::Actionable => "actionable",
                Actionability::Blocked => "blocked",
            };
            Some(ActionabilityNode {
                id: r.id,
                title: r.title,
                kind: r.kind,
                status: r.status,
                actionability: actionability.into(),
                consequence: r.consequence,
                rank,
                blockers: r.blockers,
            })
        })
        .collect();

    ActionabilityView {
        kind: "actionability_graph".into(),
        policy_version: "priority.v2".into(),
        nodes,
        edges,
    }
}

/// `next` (design §5.4) — the ACTIONABLE nodes only, in cordage `order_key` order
/// (D9). Blocked items are ABSENT (the divergence feature). Advisory; mutates
/// nothing.
pub(crate) fn next(root: &Path) -> anyhow::Result<Vec<NextRow>> {
    let g = graph::build(root)?;
    let order = channels::order_key(&g);
    let rows = order
        .into_iter()
        // Actionable AND not a promoted backlog item: a promoted item is excluded by
        // its own reason (F1 / REQ-075 AC2), the same exclusion `survey` applies — it
        // is no longer work to start, so it never leads the advisory worklist.
        .filter(|&k| channels::actionable(&g, k) && !channels::promoted(&g, k))
        .map(|k| {
            let blocking = refs(&channels::blocking(&g, k));
            let mut reasons = vec![eligibility_reason(&g, k)];
            if !blocking.is_empty() {
                reasons.push(ReasonKind::Blocking {
                    items: blocking.clone(),
                });
            }
            NextRow {
                id: k.canonical(),
                title: title_of(&g, k),
                kind: kind_of(&g, k),
                status: status_of(&g, k),
                act: Actionability::Actionable,
                reasons,
                blockers: Vec::new(),
                blocking,
            }
        })
        .collect();
    Ok(rows)
}

/// Resolve the canonical ref `id` to an [`EntityKey`] — a clean error for an unknown
/// prefix / malformed ref (never a panic).
fn parse_key(id: &str) -> anyhow::Result<EntityKey> {
    let (kref, qid) = crate::integrity::parse_canonical_ref(id)?;
    Ok(EntityKey {
        prefix: kref.kind.prefix,
        id: qid,
    })
}

/// `blockers <ID> [--transitive]` (design §5.4 / REQ-073) — direct blocked-by +
/// blocking by default; `--transitive` walks both chains via `reachable`. Display
/// depth NEVER reorders (both lists canonical-id sorted).
pub(crate) fn blockers(root: &Path, id: &str, transitive: bool) -> anyhow::Result<BlockersView> {
    let key = parse_key(id)?;
    let g = graph::build(root)?;
    // Existence gate (SL-050 F6): a well-formed but never-minted id errors rather than
    // rendering a clean empty block indistinguishable from a real isolated node.
    relation_graph::require_minted(&g.projection, key)?;
    let (blocked_by, blocking) = if transitive {
        (
            channels::blocked_by_transitive(&g, key),
            channels::blocking_transitive(&g, key),
        )
    } else {
        (channels::blocked_by(&g, key), channels::blocking(&g, key))
    };
    Ok(BlockersView {
        id: key.canonical(),
        transitive,
        blocked_by: refs(&blocked_by),
        blocking: refs(&blocking),
    })
}

/// `explain <ID>` (design §5.4 / D11) — always walked to root: the eligibility
/// reason, the transitive blocker chain, the order-key contributors, the evicted seq
/// edges, and the consequence. Each a structured reason.
pub(crate) fn explain(root: &Path, id: &str) -> anyhow::Result<Explanation> {
    let key = parse_key(id)?;
    let g = graph::build(root)?;
    // Existence gate (SL-050 F6): a well-formed but never-minted id errors rather than
    // explaining a phantom node.
    relation_graph::require_minted(&g.projection, key)?;

    let eligibility = eligibility_reason(&g, key);

    let chain = channels::blocked_by_transitive(&g, key);
    let blocker_chain = if chain.is_empty() {
        Vec::new()
    } else {
        vec![ReasonKind::BlockedBy {
            items: refs(&chain),
        }]
    };

    let evictions = channels::evicted_seq_edges(&g, key)
        .into_iter()
        .map(|(from, to, reason)| ReasonKind::EvictedEdge {
            from: from.canonical(),
            to: to.canonical(),
            reason,
        })
        .collect();

    // Cycle degrade: if the node sits in a diagnosed dep cycle, surface it.
    let cycle = channels::dep_cycles(&g)
        .into_iter()
        .find(|c| c.contains(&key));
    let consequence = ReasonKind::Consequence {
        inbound: channels::consequence(&g, key),
    };

    let mut blocker_chain = blocker_chain;
    if let Some(component) = cycle {
        let nodes = component.into_iter().map(EntityKey::canonical).collect();
        blocker_chain.push(ReasonKind::CycleDegraded { nodes });
    }

    Ok(Explanation {
        id: key.canonical(),
        eligibility,
        blocker_chain,
        evictions,
        consequence,
    })
}

/// The `inspect` actionability block over a PRE-SCANNED entity slice (design §5.4 /
/// SL-046 D1 + the SL-050 F2 shared-scan seam) — the eligible / actionable flags, the
/// direct blockers + blocking, and the consequence for one entity. Composed at the
/// command layer below the relation view (`run_inspect` passes the single corpus scan
/// it already built). `root` is RETAINED for the per-backlog `dep_seq_for` reads inside
/// `graph::build_from`. A well-formed ref to a never-minted id is an ERROR (F6), not an
/// empty block.
pub(crate) fn actionability_block_from(
    scanned: &[relation_graph::ScannedEntity],
    root: &Path,
    id: &str,
) -> anyhow::Result<ActionabilityBlock> {
    let key = parse_key(id)?;
    let g = graph::build_from(scanned, root)?;
    // Existence gate (SL-050 F6): a well-formed but never-minted id errors rather than
    // rendering an all-empty block indistinguishable from a real isolated node.
    relation_graph::require_minted(&g.projection, key)?;
    Ok(ActionabilityBlock {
        eligible: channels::eligible(&g, key),
        actionable: channels::actionable(&g, key),
        blockers: refs(&channels::blocked_by(&g, key)),
        blocking: refs(&channels::blocking(&g, key)),
        consequence: channels::consequence(&g, key),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    use crate::priority::graph::build;

    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

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

    // ── VT-1: survey_rank_topological ─────────────────────────────────────

    #[test]
    fn survey_rank_topological_chain_a_to_b_to_c() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs ISS-002; ISS-002 needs ISS-003.
        seed_issue(root, 1, "open", "", &[("needs", &["ISS-002"])]);
        seed_issue(root, 2, "open", "", &[("needs", &["ISS-003"])]);
        seed_issue(root, 3, "open", "", &[]);

        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);

        // Find nodes by id.
        let n1 = view.nodes.iter().find(|n| n.id == "ISS-001").unwrap();
        let n2 = view.nodes.iter().find(|n| n.id == "ISS-002").unwrap();
        let n3 = view.nodes.iter().find(|n| n.id == "ISS-003").unwrap();

        assert_eq!(n3.rank, 0, "ISS-003 has no blockers → rank 0");
        assert_eq!(n2.rank, 1, "ISS-002 blocked by ISS-003 (rank 0) → rank 1");
        assert_eq!(n1.rank, 2, "ISS-001 blocked by ISS-002 (rank 1) → rank 2");
    }

    // ── VT-2: survey_needs_edges_present ──────────────────────────────────

    #[test]
    fn survey_needs_edges_present() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", &[("needs", &["ISS-002"])]);
        seed_issue(root, 2, "open", "", &[("needs", &["ISS-003"])]);
        seed_issue(root, 3, "open", "", &[]);

        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);

        assert!(view
            .edges
            .iter()
            .any(|e| e.source == "ISS-003" && e.target == "ISS-002" && e.kind == "needs"));
        assert!(view
            .edges
            .iter()
            .any(|e| e.source == "ISS-002" && e.target == "ISS-001" && e.kind == "needs"));
    }

    // ── VT-3: survey_after_edges_present ──────────────────────────────────

    #[test]
    fn survey_after_edges_present() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 has an after edge onto ISS-002.
        seed_issue(root, 2, "open", "", &[]);
        write(
            root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nafter = [{ to = \"ISS-002\", rank = 0 }]\n",
        );
        write(
            root,
            ".doctrine/backlog/issue/001/backlog-001.md",
            "b\n",
        );

        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);

        assert!(view
            .edges
            .iter()
            .any(|e| e.source == "ISS-002" && e.target == "ISS-001" && e.kind == "after"));
    }

    // ── VT-4: survey_empty_graph ──────────────────────────────────────────

    #[test]
    fn survey_empty_graph() {
        let dir = tmp();
        let root = dir.path();
        // Only a terminal (closed) issue — no eligible nodes.
        seed_issue(root, 1, "closed", "", &[]);

        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);

        assert!(view.nodes.is_empty());
        assert!(view.edges.is_empty());
    }

    // ── VT-5: survey_excludes_terminal ────────────────────────────────────

    #[test]
    fn survey_excludes_terminal() {
        let dir = tmp();
        let root = dir.path();
        // Two issues: one open (eligible), one closed (terminal).
        seed_issue(root, 1, "open", "", &[]);
        seed_issue(root, 2, "closed", "", &[]);

        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);

        assert_eq!(view.nodes.len(), 1, "only the eligible (open) node");
        assert_eq!(view.nodes[0].id, "ISS-001");
        assert!(view.nodes.iter().all(|n| n.id != "ISS-002"));
    }

    // ── VT-6: survey_terminal_blocker_no_edge ─────────────────────────────

    #[test]
    fn survey_terminal_blocker_no_edge() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 (open) needs ISS-002 (closed/terminal).
        // The terminal blocker is satisfied → no edge emitted.
        seed_issue(root, 1, "open", "", &[("needs", &["ISS-002"])]);
        seed_issue(root, 2, "closed", "", &[]);

        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);

        // ISS-001 appears (eligible), ISS-002 does not (terminal).
        assert_eq!(view.nodes.len(), 1);
        let n1 = &view.nodes[0];
        assert_eq!(n1.id, "ISS-001");
        // No edge from ISS-002 (it's terminal and not in the node set).
        assert!(view.edges.is_empty(), "terminal → eligible edge suppressed");
        // ISS-001 is actionable (its blocker is terminal/satisfied).
        assert_eq!(n1.actionability, "actionable");
        assert_eq!(n1.rank, 0);
    }

    // ── VT-7: survey_for_map matches survey byte-for-byte ─────────────────

    #[test]
    fn survey_for_map_matches_survey_byte_for_byte() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", &[("needs", &["ISS-003"])]);
        seed_issue(root, 2, "open", "", &[("needs", &["ISS-003"])]);
        seed_issue(root, 3, "open", "", &[]);

        let g = build(root).unwrap();
        let from_survey = survey(root, false).unwrap();
        let from_for_map = survey_for_map(&g, false);

        assert_eq!(from_survey, from_for_map, "survey_for_map must match survey output exactly");
    }

    // ── VT-8 (implicit): existing tests pass — verified by `cargo test` ───
}
