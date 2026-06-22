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
//! Importance order (survey, SL-133 §5.4): `actionability(Actionable > Blocked) →
//! score desc (total_cmp) → canonical-id asc`. `next` runs its OWN score-aware
//! induced-frontier (Kahn) sort over the SURVIVING seq edges (`seq_overlay` − evictions),
//! filtered to actionable nodes — NOT cordage `order_key` (it ranks Level before
//! `NodeId`, demoting score-promoted successors; RV-132 F-3).

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

/// The score-breakdown reason for a node (SL-133 §5.4) — `base` (+ its `value_dim` /
/// `risk_dim` split), the recursive `leverage`, the one-hop `optionality`, and the
/// `total`. Built ONCE here so the human + `--json` renders cannot drift.
fn score_reason(g: &PriorityGraph, key: EntityKey) -> ReasonKind {
    ReasonKind::Score {
        base: channels::base(g, key),
        value_dim: channels::value_dim(g, key),
        risk_dim: channels::risk_dim(g, key),
        leverage: channels::leverage(g, key),
        optionality: channels::optionality(g, key),
        total: channels::score(g, key),
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
    score: f64,
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
            score: channels::score(g, k),
            blockers: refs(&channels::blocked_by(g, k)),
        })
        .collect();

    // Importance order (SL-133 §5.4): actionability → score DESC (total_cmp) → id ASC.
    // The comparator does ZERO graph work — it compares only pre-computed scalars.
    rows.sort_by(|a, b| {
        // Actionable before Blocked.
        let act = act_rank(a.act).cmp(&act_rank(b.act));
        let score = b.score.total_cmp(&a.score); // score DESC
        act.then(score).then_with(|| a.key.cmp(&b.key))
    });

    rows.into_iter()
        .map(|d| {
            let mut reasons = vec![eligibility_reason(g, d.key)];
            if !d.blockers.is_empty() {
                reasons.push(ReasonKind::BlockedBy {
                    items: d.blockers.clone(),
                });
            }
            reasons.push(score_reason(g, d.key));
            SurveyRow {
                id: d.key.canonical(),
                title: title_of(g, d.key),
                kind: kind_of(g, d.key),
                status: status_of(g, d.key),
                act: d.act,
                score: d.score,
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

    /// Work-entity kinds — the only entities with dep/seq edges that constitute
    /// the actionability graph (SL-089 D2). SPEC, REQ, ADR, etc. are governance
    /// entities and are excluded from the actionability view.
    const WORK_PREFIXES: &[&str] = &["SL", "ISS", "IMP", "CHR", "RSK", "IDE"];

    // 1. Build canonical rows (eligible set + ordering).
    let rows: Vec<_> = survey_for_map(g, all)
        .into_iter()
        .filter(|r| {
            // Only work entities appear in the actionability graph (SL-089 D2).
            WORK_PREFIXES.contains(&r.kind.as_str())
        })
        .collect();

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
                score: r.score,
                rank,
                blockers: r.blockers,
            })
        })
        .collect();

    ActionabilityView {
        kind: "actionability_graph".into(),
        policy_version: "priority.v3".into(),
        nodes,
        edges,
    }
}

/// The **surviving** seq predecessors of each actionable node (SL-133 §5.4 / F-3) —
/// the `seq_overlay` `in_edges` MINUS the edges cordage EVICTED to linearize an `Evict`
/// cycle, restricted to edges whose BOTH endpoints are in `actionable`. The induced
/// precedence relation `next`'s frontier sort honours; an evicted (broken) seq edge
/// does NOT re-impose precedence.
///
/// Empirical finding (this cordage build): `in_edges(seq_overlay, ·)` ALREADY excludes
/// the evicted edge — for an `Evict` 2-cycle, `provenance().evictions()` reports both
/// directed entries but `in_edges` yields only the one surviving edge. The explicit
/// subtraction via [`channels::evicted_seq_edges`] is therefore DEFENSIVE here (a no-op
/// in the common path), kept to honour the design §5.4 contract ("read surviving edges,
/// not raw `seq_overlay`") and stay correct if cordage's enumeration ever changes. VT-7's
/// evicted-seq case proves the broken edge does not re-impose precedence either way.
fn surviving_seq_predecessors(
    g: &PriorityGraph,
    actionable: &std::collections::BTreeSet<EntityKey>,
) -> std::collections::BTreeMap<EntityKey, std::collections::BTreeSet<EntityKey>> {
    let mut preds: std::collections::BTreeMap<EntityKey, std::collections::BTreeSet<EntityKey>> =
        std::collections::BTreeMap::new();
    for &k in actionable {
        // The evicted (from, to) pairs touching `k` — subtract these from the raw
        // enumeration so a broken seq edge never re-imposes an order.
        let evicted: std::collections::BTreeSet<(EntityKey, EntityKey)> =
            channels::evicted_seq_edges(g, k)
                .into_iter()
                .map(|(from, to, _reason)| (from, to))
                .collect();
        let mut set = std::collections::BTreeSet::new();
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

/// Pure induced-frontier (Kahn-style) sort of the actionable set (SL-133 §5.4 / F-3).
/// Precedence is `preds` (the SURVIVING actionable seq edges); among nodes whose
/// surviving predecessors are all emitted, the next pick is the max by
/// `(score desc via total_cmp, id asc)`. NOT cordage `order_key` (its `(Level, NodeId)`
/// ranks Level before `NodeId`, demoting score-promoted successors; RV-132 F-3).
///
/// Total + terminating: every node is emitted exactly once; a residual seq cycle (none
/// expected — the seq overlay is `Evict`-linearized) would still drain via the same
/// `(score, id)` pick once the ready set empties, so the loop always makes progress.
fn frontier_order(
    actionable: &[EntityKey],
    score: &dyn Fn(EntityKey) -> f64,
    preds: &std::collections::BTreeMap<EntityKey, std::collections::BTreeSet<EntityKey>>,
) -> Vec<EntityKey> {
    let mut emitted: std::collections::BTreeSet<EntityKey> = std::collections::BTreeSet::new();
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
            score(*a)
                .total_cmp(&score(*b))
                .then_with(|| b.cmp(a))
        }) else {
            break;
        };
        emitted.insert(pick);
        out.push(pick);
    }
    out
}

/// `next` (design §5.4 / SL-133) — the ACTIONABLE nodes only, in a score-aware
/// induced-frontier order over the SURVIVING seq edges (`seq_overlay` − evictions). The
/// workable-but-BLOCKED items are ABSENT (the divergence feature). Advisory; mutates
/// nothing. NOT cordage `order_key` (it ranks Level before `NodeId`; RV-132 F-3).
pub(crate) fn next(root: &Path) -> anyhow::Result<Vec<NextRow>> {
    let g = graph::build(root)?;
    // The actionable, non-promoted set (a promoted item is excluded by its own reason,
    // F1 / REQ-075 AC2 — the same exclusion `survey` applies).
    let actionable_set: std::collections::BTreeSet<EntityKey> = g
        .attrs
        .keys()
        .copied()
        .filter(|&k| channels::actionable(&g, k) && !channels::promoted(&g, k))
        .collect();
    let actionable: Vec<EntityKey> = actionable_set.iter().copied().collect();
    let preds = surviving_seq_predecessors(&g, &actionable_set);
    let order = frontier_order(&actionable, &|k| channels::score(&g, k), &preds);
    let rows = order
        .into_iter()
        .map(|k| {
            let blocking = refs(&channels::blocking(&g, k));
            let mut reasons = vec![eligibility_reason(&g, k)];
            if !blocking.is_empty() {
                reasons.push(ReasonKind::Blocking {
                    items: blocking.clone(),
                });
            }
            reasons.push(score_reason(&g, k));
            NextRow {
                id: k.canonical(),
                title: title_of(&g, k),
                kind: kind_of(&g, k),
                status: status_of(&g, k),
                act: Actionability::Actionable,
                score: channels::score(&g, k),
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
/// reason, the transitive blocker chain, the evicted seq edges, and the score
/// breakdown. Each a structured reason.
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
    let score = score_reason(&g, key);

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
        score,
    })
}

/// The `inspect` actionability block over a PRE-SCANNED entity slice (design §5.4 /
/// SL-046 D1 + the SL-050 F2 shared-scan seam) — the eligible / actionable flags, the
/// direct blockers + blocking, and the score for one entity. Composed at the
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
        score: channels::score(&g, key),
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

        assert!(
            view.edges
                .iter()
                .any(|e| e.source == "ISS-003" && e.target == "ISS-002" && e.kind == "needs")
        );
        assert!(
            view.edges
                .iter()
                .any(|e| e.source == "ISS-002" && e.target == "ISS-001" && e.kind == "needs")
        );
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
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "b\n");

        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);

        assert!(
            view.edges
                .iter()
                .any(|e| e.source == "ISS-002" && e.target == "ISS-001" && e.kind == "after")
        );
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

        assert_eq!(
            from_survey, from_for_map,
            "survey_for_map must match survey output exactly"
        );
    }

    // ── SL-133 dedicated helpers + ordering proofs (VT-5 / VT-7 / VA-1) ───

    /// Seed an open backlog issue with an explicit `[value]` over a fixed estimate mid
    /// of 5.0 (lower 0, upper 10), plus optional `needs`/`after` relationship lines.
    /// `value` of `v` ⇒ base = value_coeff(1.0) · v · 1.0 / 5.0 = v/5.
    fn seed_valued(root: &Path, id: u32, value: f64, rel_lines: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I{id}\"\nkind = \"issue\"\nstatus = \"open\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [estimate]\nlower = 0.0\nupper = 10.0\n[value]\nvalue = {value}\n\
                 [relationships]\n{rel_lines}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    fn next_ids(root: &Path) -> Vec<String> {
        next(root).unwrap().into_iter().map(|r| r.id).collect()
    }

    fn survey_ids(root: &Path) -> Vec<String> {
        survey(root, false).unwrap().into_iter().map(|r| r.id).collect()
    }

    /// VT-5 (the point of the slice): a blocker gating ONE high-value slice outranks a
    /// blocker gating FIVE ideas — the OLD inbound-count would rank them opposite.
    /// RSK-001 is the prereq of one valued ISS-001 (value 100 → base 20, so RSK-001's
    /// leverage = 0.5·20 = 10). RSK-002 is the prereq of five zero-value ideas (leverage
    /// 0). In survey both blockers' DEPENDENTS are blocked, but the blockers themselves
    /// are actionable and ordered by score: RSK-001 (10) before RSK-002 (0).
    #[test]
    fn vt5_blocker_of_one_high_value_outranks_blocker_of_five_ideas() {
        let dir = tmp();
        let root = dir.path();
        // RSK-001 gates one high-value issue.
        seed_issue(root, 1, "open", "", &[]); // placeholder so RSK keys are distinct kinds
        write(
            root,
            ".doctrine/backlog/risk/001/backlog-001.toml",
            "id = 1\nslug = \"k\"\ntitle = \"K1\"\nkind = \"risk\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
        );
        write(root, ".doctrine/backlog/risk/001/backlog-001.md", "k\n");
        write(
            root,
            ".doctrine/backlog/risk/002/backlog-002.toml",
            "id = 2\nslug = \"k\"\ntitle = \"K2\"\nkind = \"risk\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
        );
        write(root, ".doctrine/backlog/risk/002/backlog-002.md", "k\n");
        // One high-value issue needs RSK-001 (value 100 → base 20).
        seed_valued(root, 10, 100.0, "needs = [\"RSK-001\"]\n");
        // Five zero-value ideas each need RSK-002.
        for id in 20..25 {
            seed_valued(root, id, 0.0, "needs = [\"RSK-002\"]\n");
        }

        let g = build(root).unwrap();
        let rsk1 = EntityKey { prefix: "RSK", id: 1 };
        let rsk2 = EntityKey { prefix: "RSK", id: 2 };
        // RSK-001's leverage = 0.5 · (base(ISS-010) + 0) = 0.5 · 20 = 10.
        assert!(
            (channels::score(&g, rsk1) - 10.0).abs() < 1e-9,
            "RSK-001 leverages the one high-value dependent: got {}",
            channels::score(&g, rsk1)
        );
        // RSK-002 gates five zero-value ideas → leverage 0.
        assert!(
            channels::score(&g, rsk2).abs() < 1e-9,
            "RSK-002 gates only zero-value ideas → score 0"
        );
        // survey orders RSK-001 (score 10) BEFORE RSK-002 (score 0) — the old
        // inbound-count (5 vs 1) would have ranked RSK-002 first.
        let ids = survey_ids(root);
        let p1 = ids.iter().position(|x| x == "RSK-001").unwrap();
        let p2 = ids.iter().position(|x| x == "RSK-002").unwrap();
        assert!(p1 < p2, "RSK-001 outranks RSK-002 by score (not inbound count): {ids:?}");
    }

    /// VT-5 (recursive-leverage proof): a DEEP blocker gating a cheap chore that gates a
    /// valuable cone outranks a SHALLOW blocker fronting one modest item. The recursive
    /// DP propagates the cone's value back through the chain.
    #[test]
    fn vt5_deep_blocker_of_valuable_cone_outranks_shallow_blocker_of_modest_item() {
        let dir = tmp();
        let root = dir.path();
        // Deep chain: ISS-001 (deep blocker) ← ISS-002 (cheap chore) ← ISS-003 (valuable).
        // needs: ISS-002 needs ISS-001; ISS-003 needs ISS-002.
        seed_valued(root, 1, 0.0, ""); // deep blocker, no own value
        seed_valued(root, 2, 0.0, "needs = [\"ISS-001\"]\n"); // cheap chore
        seed_valued(root, 3, 200.0, "needs = [\"ISS-002\"]\n"); // valuable cone (base 40)
        // Shallow blocker: ISS-010 fronting one modest ISS-011 (value 10 → base 2).
        seed_valued(root, 10, 0.0, ""); // shallow blocker
        seed_valued(root, 11, 10.0, "needs = [\"ISS-010\"]\n");

        let g = build(root).unwrap();
        let k = |id| EntityKey { prefix: "ISS", id };
        // leverage(ISS-002) = 0.5·(base(ISS-003)+0) = 0.5·40 = 20.
        // leverage(ISS-001) = 0.5·(base(ISS-002)+leverage(ISS-002)) = 0.5·(0+20) = 10.
        let deep = channels::score(&g, k(1));
        // leverage(ISS-010) = 0.5·(base(ISS-011)+0) = 0.5·2 = 1.
        let shallow = channels::score(&g, k(10));
        assert!((deep - 10.0).abs() < 1e-9, "deep blocker recursive leverage = 10: got {deep}");
        assert!((shallow - 1.0).abs() < 1e-9, "shallow blocker leverage = 1: got {shallow}");
        let ids = survey_ids(root);
        let pd = ids.iter().position(|x| x == "ISS-001").unwrap();
        let ps = ids.iter().position(|x| x == "ISS-010").unwrap();
        assert!(pd < ps, "deep blocker of a valuable cone outranks the shallow one: {ids:?}");
    }

    /// VT-7 (a): a Y-fixture — two seq-INCOMPARABLE ready arms order by score. ISS-002
    /// and ISS-003 both follow ISS-001 (after), but have no seq edge between each other,
    /// so the order between them is the score tiebreak.
    #[test]
    fn vt7_y_fixture_incomparable_arms_order_by_score() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 is the shared upstream; ISS-002 (value 50, base 10) and ISS-003
        // (value 100, base 20) both `after` ISS-001 — incomparable to each other.
        seed_valued(root, 1, 0.0, "");
        seed_valued(root, 2, 50.0, "after = [{ to = \"ISS-001\", rank = 0 }]\n");
        seed_valued(root, 3, 100.0, "after = [{ to = \"ISS-001\", rank = 0 }]\n");

        let ids = next_ids(root);
        // ISS-001 leads (predecessor of both); among the two arms, higher score first.
        let p1 = ids.iter().position(|x| x == "ISS-001").unwrap();
        let p2 = ids.iter().position(|x| x == "ISS-002").unwrap();
        let p3 = ids.iter().position(|x| x == "ISS-003").unwrap();
        assert!(p1 < p2 && p1 < p3, "shared upstream leads: {ids:?}");
        assert!(p3 < p2, "higher-score arm ISS-003 before ISS-002: {ids:?}");
    }

    /// VT-7 (b): a same-chain seq pair keeps STRUCTURAL order regardless of score. A
    /// lower-score predecessor still precedes its higher-score successor on one chain.
    #[test]
    fn vt7_same_chain_seq_keeps_structural_order_over_score() {
        let dir = tmp();
        let root = dir.path();
        // ISS-002 `after` ISS-001 — a single chain. ISS-001 is LOW score (base 2),
        // ISS-002 is HIGH score (base 20). Structure overrides score on the chain.
        seed_valued(root, 1, 10.0, "");
        seed_valued(root, 2, 100.0, "after = [{ to = \"ISS-001\", rank = 0 }]\n");

        let ids = next_ids(root);
        let p1 = ids.iter().position(|x| x == "ISS-001").unwrap();
        let p2 = ids.iter().position(|x| x == "ISS-002").unwrap();
        assert!(
            p1 < p2,
            "low-score predecessor ISS-001 precedes high-score ISS-002 (structural): {ids:?}"
        );
    }

    /// VT-7 (c): an EVICTED (cyclic) seq edge does NOT re-impose precedence — the sort
    /// reads SURVIVING edges, not raw `seq_overlay` (F-3). A seq cycle ISS-001 ↔ ISS-002
    /// is `Evict`-linearized; the broken edge must not force an order, so the higher-score
    /// node leads despite a raw seq edge pointing at it.
    #[test]
    fn vt7_evicted_seq_edge_does_not_reimpose_precedence() {
        let dir = tmp();
        let root = dir.path();
        // A 2-cycle on the seq overlay: ISS-001 after ISS-002 AND ISS-002 after ISS-001.
        // cordage Evict drops one edge to linearize. ISS-002 has the higher score (base
        // 20 vs 2), so once the evicted edge is subtracted it leads on score.
        seed_valued(root, 1, 10.0, "after = [{ to = \"ISS-002\", rank = 0 }]\n");
        seed_valued(root, 2, 100.0, "after = [{ to = \"ISS-001\", rank = 0 }]\n");

        let g = build(root).unwrap();
        // Prove an eviction actually occurred on the seq overlay (the precondition).
        let evicted_total: usize = g
            .attrs
            .keys()
            .map(|&k| channels::evicted_seq_edges(&g, k).len())
            .sum();
        assert!(evicted_total > 0, "the seq 2-cycle must produce an eviction");

        let ids = next_ids(root);
        let p1 = ids.iter().position(|x| x == "ISS-001").unwrap();
        let p2 = ids.iter().position(|x| x == "ISS-002").unwrap();
        // The SURVIVING precedence (one edge) plus the score tiebreak determine order.
        // Whichever edge survived, the result must be a clean total order with no
        // contradiction; the higher-score ISS-002 must NOT be demoted by the evicted
        // edge — if only the evicted edge pointed predecessor→ISS-002, it is ignored.
        // We assert the score-promoted node is not pinned last by a broken edge: ISS-002
        // leads unless the SURVIVING edge genuinely orders it after ISS-001.
        let surviving_pred_of_2 = {
            let preds = surviving_seq_predecessors(
                &g,
                &g.attrs
                    .keys()
                    .copied()
                    .filter(|&k| channels::actionable(&g, k) && !channels::promoted(&g, k))
                    .collect(),
            );
            preds
                .get(&EntityKey { prefix: "ISS", id: 2 })
                .map(|s| s.contains(&EntityKey { prefix: "ISS", id: 1 }))
                .unwrap_or(false)
        };
        if surviving_pred_of_2 {
            assert!(p1 < p2, "surviving edge orders ISS-001 before ISS-002: {ids:?}");
        } else {
            assert!(
                p2 < p1,
                "evicted edge does NOT re-impose precedence; higher-score ISS-002 leads: {ids:?}"
            );
        }
    }

    /// VA-1: the `explain` Score reason exposes the full breakdown and the human render
    /// reads it correctly.
    #[test]
    fn va1_explain_exposes_full_score_breakdown() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 value 50 over mid 5 → base 10; one dependent ISS-002 (value 100, base
        // 20) needs it → leverage(ISS-001) = 0.5·20 = 10. No referencers → optionality 0.
        seed_valued(root, 1, 50.0, "");
        seed_valued(root, 2, 100.0, "needs = [\"ISS-001\"]\n");

        let ex = explain(root, "ISS-001").unwrap();
        match ex.score {
            ReasonKind::Score {
                base,
                value_dim,
                risk_dim,
                leverage,
                optionality,
                total,
            } => {
                assert!((base - 10.0).abs() < 1e-9, "base 10");
                assert!((value_dim - 10.0).abs() < 1e-9, "value_dim 10");
                assert!(risk_dim.abs() < 1e-9, "risk_dim 0");
                assert!((leverage - 10.0).abs() < 1e-9, "leverage 10");
                assert!(optionality.abs() < 1e-9, "optionality 0");
                assert!((total - 20.0).abs() < 1e-9, "total = base + leverage = 20");
            }
            other => panic!("explain score must be a Score reason, got {other:?}"),
        }
        // Human render reads the breakdown line correctly.
        let human = crate::priority::render::explain_human(&ex);
        assert!(
            human.contains(
                "score: 20.0 (base 10.0 [value 10.0, risk 0.0], leverage 10.0, optionality 0.0)"
            ),
            "human explain renders the full breakdown: {human}"
        );
    }
}
