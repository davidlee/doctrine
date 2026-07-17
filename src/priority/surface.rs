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

use std::collections::{BTreeMap, BTreeSet};

use crate::catalog::scan::ScanMode;
use crate::comparison::{
    self, ConstraintSet, Judgement, Projection as ValueProjection, QuarantinePolicy, RaterCounts,
    Reachability, compile_human_only, constraining_counts_by_class, determined,
};
use crate::relation_graph::{self, EntityKey};

use super::channels;
use super::config::PriorityConfig;
use super::elicit::pair_side;
use super::graph::{self, NodeAttr, PriorityGraph};
// SL-194: the frontier-order primitives moved to the pure `order` module; `next`
// reuses them byte-identically (the detectors share the same implementation).
use super::order::{frontier_order, surviving_seq_predecessors};
use super::partition::{StatusClass, status_class};
use super::tension::{
    self, DetectInputs, EdgeKind, EvidenceGrade, PredEdge, Tension, TensionCause,
};
use super::view::{
    Actionability, ActionabilityBlock, ActionabilityEdge, ActionabilityNode, ActionabilityView,
    BlockersView, ContestedClaim, EdgeVerb, Explanation, NextRow, NextView, ReasonKind, SurveyRow,
    TensionCauseView, TensionGradeView,
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

/// `survey [--all] [--hide-blocked]` (design §5.4 / IMP-218) — the eligible set in
/// importance order (D10).
///
/// Set: every `eligible` node, MINUS `promoted` backlog items (excluded as their own
/// reason, F1), UNLESS `all` reveals the full picture. With `all`, terminal +
/// promoted nodes are included too (the complete view). `hide_blocked` drops blocked
/// rows from the result (default false — blocked rows included). Sort (the empty
/// authored-priority slot collapses to): `actionability(Actionable first) →
/// consequence desc → canonical-id asc`.
pub(crate) fn survey(root: &Path, all: bool, hide_blocked: bool) -> anyhow::Result<Vec<SurveyRow>> {
    let g = graph::build(root)?;
    let mut rows = survey_for_map(&g, all);
    if hide_blocked {
        rows.retain(|r| r.act == Actionability::Actionable);
    }
    Ok(rows)
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

    // Value-bearing kinds — the only entities with dep/seq edges that constitute
    // the actionability graph (SL-089 D2). SPEC, REQ, ADR, etc. are governance
    // entities and are excluded from the actionability view.
    // Uses `crate::kinds::is_value_bearing` — the single source for this set.

    // 1. Build canonical rows (eligible set + ordering).
    let rows: Vec<_> = survey_for_map(g, all)
        .into_iter()
        .filter(|r| {
            // Only work entities appear in the actionability graph (SL-089 D2).
            crate::kinds::is_value_bearing(r.kind.as_str())
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

/// `next` (design §5.4 / SL-133) — the ACTIONABLE nodes only, in a score-aware
/// induced-frontier order over the SURVIVING seq edges (`seq_overlay` − evictions). The
/// workable-but-BLOCKED items are ABSENT (the divergence feature). Advisory; mutates
/// nothing. NOT cordage `order_key` (it ranks Level before `NodeId`; RV-132 F-3).
pub(crate) fn next(root: &Path) -> anyhow::Result<NextView> {
    // Load via the comparison pipeline (like `explain`) — the tension grades need
    // the compiled determinacy view. The graph is byte-identical to the old
    // `graph::build` path (`build_from` loads the SAME `pipeline.value.projection`), so
    // row order/score are unchanged; only the additive tension surfaces are new.
    let scanned = relation_graph::scan_entities(root, &mut vec![], ScanMode::default())?;
    let cfg = super::config::load(root);
    let pipeline = graph::load_comparison_pipeline(root, &scanned, &cfg)?;
    let cost_feed = comparison::cost_feed(&pipeline.estimate.projection);
    let g = graph::build_from_with_cfg(
        &scanned,
        root,
        &cfg,
        &pipeline.value.projection,
        &cost_feed,
        &pipeline.value_claims,
    )?;
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
            // Project the estimate/tags facets from NodeAttr (SL-171 PHASE-01, D2);
            // the value cell re-paths to the RESOLVED ladder (SL-220 PHASE-06,
            // design §6) — the facet reader here died with the flip (EX-3).
            let (estimate, tags) = attr(&g, k).map_or((None, Vec::new()), |a| {
                (a.facets.estimate.clone(), a.facets.tags.clone())
            });
            let value_source = value_source_reason(&g, k, &pipeline);
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
                estimate,
                value_source,
                tags,
            }
        })
        .collect();

    // Tensions: the FULL frontier (design §3 — JSON carries the uncapped list);
    // the renderer page-filters to visible rows and caps the human callout block.
    let tensions = graded_tensions(&g, &pipeline, &cfg, usize::MAX)
        .iter()
        .map(tension_reason)
        .collect();
    let zero_weight = match frontier_zero_weight(&g, &cfg, usize::MAX) {
        0 => None,
        count => Some(ReasonKind::ZeroWeightExcluded { count }),
    };
    Ok(NextView {
        rows,
        tensions,
        zero_weight,
    })
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
///
/// SL-213 PHASE-06: also loads the comparison-tier pipeline over the SAME
/// scan (mirrors `findings`'s one-scan style — no shared-scan seam exists
/// across verbs; each surface fn already re-scans independently, e.g.
/// `blockers`/`actionability_block_from` above) to compose the value-source
/// block + the corpus-global inert priority-domain disclosure.
pub(crate) fn explain(root: &Path, id: &str) -> anyhow::Result<Explanation> {
    let key = parse_key(id)?;
    let scanned = relation_graph::scan_entities(root, &mut vec![], ScanMode::default())?;
    let cfg = super::config::load(root);
    let pipeline = graph::load_comparison_pipeline(root, &scanned, &cfg)?;
    let cost_feed = comparison::cost_feed(&pipeline.estimate.projection);
    let g = graph::build_from_with_cfg(
        &scanned,
        root,
        &cfg,
        &pipeline.value.projection,
        &cost_feed,
        &pipeline.value_claims,
    )?;
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

    let value_source = value_source_reason(&g, key, &pipeline);
    let cost_source = cost_source_reason(&g, key, &pipeline, &cfg);
    let priority_disclosure =
        (pipeline.priority_domain_count > 0).then_some(ReasonKind::PriorityDomainDisclosure {
            count: pipeline.priority_domain_count,
        });

    // Tensions: explain considers the WHOLE frontier (design §2 considered-set —
    // `page_k = usize::MAX`), then keeps only those involving this id, as surfaced
    // member OR off-page preferred counterparty (the displaced-item story explain
    // exists to tell). `on_frontier` gates the "not on the current frontier"
    // disclosure for non-actionable ids.
    let id_ref = key.canonical();
    let on_frontier = channels::actionable(&g, key) && !channels::promoted(&g, key);
    let tensions: Vec<ReasonKind> = graded_tensions(&g, &pipeline, &cfg, usize::MAX)
        .iter()
        .filter(|t| t.preferred == key || t.surfaced == key)
        .map(tension_reason)
        .collect();

    Ok(Explanation {
        id: id_ref,
        eligibility,
        blocker_chain,
        evictions,
        score,
        value_source,
        cost_source,
        priority_disclosure,
        agent_demotion: cfg
            .compare
            .demote_agent_evidence
            .then_some(ReasonKind::AgentEvidenceDemoted),
        tensions,
        on_frontier,
    })
}

/// The owned detection projections for the actionable frontier (design §2) —
/// built once off the graph, borrowed by [`DetectInputs`]. Shared by
/// [`graded_tensions`] and [`frontier_zero_weight`] so both scan the SAME basis.
struct FrontierProjections {
    order: Vec<EntityKey>,
    value_dim: BTreeMap<EntityKey, f64>,
    multiplier: BTreeMap<EntityKey, f64>,
    full_score: BTreeMap<EntityKey, f64>,
    risk_dim: BTreeMap<EntityKey, f64>,
    leverage: BTreeMap<EntityKey, f64>,
    optionality: BTreeMap<EntityKey, f64>,
    preds: BTreeMap<EntityKey, Vec<PredEdge>>,
}

impl FrontierProjections {
    fn inputs(&self, page_k: usize) -> DetectInputs<'_> {
        DetectInputs {
            delivery_order: &self.order,
            page_k,
            value_dim: &self.value_dim,
            multiplier: &self.multiplier,
            full_score: &self.full_score,
            risk_dim: &self.risk_dim,
            leverage: &self.leverage,
            optionality: &self.optionality,
            preds: &self.preds,
        }
    }
}

/// Build the frontier delivery order + per-node projection maps + the merged
/// surviving seq(`after`)+dep(`needs`) predecessor graph — the SAME basis
/// `next`/`explain` render (surviving-seq precedence, score tiebreak). Read once
/// off the graph, never re-derived.
fn frontier_projections(g: &PriorityGraph, cfg: &PriorityConfig) -> FrontierProjections {
    let actionable_set: BTreeSet<EntityKey> = g
        .attrs
        .keys()
        .copied()
        .filter(|&k| channels::actionable(g, k) && !channels::promoted(g, k))
        .collect();
    let actionable: Vec<EntityKey> = actionable_set.iter().copied().collect();
    let seq_preds = surviving_seq_predecessors(g, &actionable_set);
    let order = frontier_order(&actionable, &|k| channels::score(g, k), &seq_preds);

    let map = |f: &dyn Fn(EntityKey) -> f64| -> BTreeMap<EntityKey, f64> {
        actionable.iter().map(|&k| (k, f(k))).collect()
    };

    let mut preds: BTreeMap<EntityKey, Vec<PredEdge>> = BTreeMap::new();
    for &k in &actionable {
        let mut edges: Vec<PredEdge> = seq_preds
            .get(&k)
            .into_iter()
            .flatten()
            .map(|&pred| PredEdge {
                pred,
                kind: EdgeKind::Seq,
            })
            .collect();
        for pred in channels::blocked_by(g, k) {
            if actionable_set.contains(&pred) {
                edges.push(PredEdge {
                    pred,
                    kind: EdgeKind::Dep,
                });
            }
        }
        if !edges.is_empty() {
            preds.insert(k, edges);
        }
    }

    FrontierProjections {
        value_dim: map(&|k| channels::value_dim(g, k)),
        risk_dim: map(&|k| channels::risk_dim(g, k)),
        leverage: map(&|k| channels::leverage(g, k)),
        optionality: map(&|k| channels::optionality(g, k)),
        full_score: map(&|k| channels::score(g, k)),
        multiplier: map(&|k| g.item_costing(&k, cfg).map_or(0.0, |(m, _, _)| m)),
        order,
        preds,
    }
}

/// The F-6 m=0 scoped-disclosure count for the frontier page (design §2) — the
/// value inversions [`tension::detect`] dropped because a member is
/// value-insensitive. Same projection basis as [`graded_tensions`].
pub(crate) fn frontier_zero_weight(
    g: &PriorityGraph,
    cfg: &PriorityConfig,
    page_k: usize,
) -> usize {
    tension::zero_weight_excluded(&frontier_projections(g, cfg).inputs(page_k))
}

/// Convert a graded [`Tension`] into its render arm (design §3). The surface
/// shell canonicalizes the `EntityKey`s here (view.rs convention) so the renderer
/// only formats — never recomputes ids or wording (REQ-072 AC3).
pub(crate) fn tension_reason(t: &Tension) -> ReasonKind {
    let cause = match t.cause {
        TensionCause::Structure { edge } => TensionCauseView::Structure {
            edge_from: edge.from.canonical(),
            verb: match edge.kind {
                EdgeKind::Seq => EdgeVerb::After,
                EdgeKind::Dep => EdgeVerb::Needs,
            },
        },
        TensionCause::Composition { deltas } => TensionCauseView::Composition {
            risk_dim: deltas.risk_dim,
            leverage: deltas.leverage,
            optionality: deltas.optionality,
        },
    };
    let grade = match t.grade {
        EvidenceGrade::Determined { counts } => TensionGradeView::Determined {
            human: counts.human,
            agent: counts.agent,
        },
        EvidenceGrade::AgentProposed { counts } => TensionGradeView::AgentProposed {
            agent: counts.agent,
        },
        EvidenceGrade::Projected => TensionGradeView::Projected,
    };
    ReasonKind::Tension {
        preferred: t.preferred.canonical(),
        surfaced: t.surfaced.canonical(),
        cause,
        grade,
    }
}

/// Detect and grade the frontier's tensions (SL-218 PHASE-02, design §2). The
/// pure [`tension::detect`] scan over the frontier's `value_dim` vs delivery
/// orders, each inversion graded through the SHIPPED determinacy machinery (the
/// elicit pattern): the verdict system is the human-rows-only compile when the
/// knob is on, the full pipeline compile otherwise (SL-218 D1). Grade and elicit
/// queue therefore read the SAME predicate over the SAME system selection
/// (design F-1/F-7, one truth per question). `page_k` bounds the surfaced
/// member's delivery rank (`usize::MAX` = the full frontier, explain's view).
pub(crate) fn graded_tensions(
    g: &PriorityGraph,
    pipeline: &comparison::Pipeline,
    cfg: &PriorityConfig,
    page_k: usize,
) -> Vec<Tension> {
    let proj = frontier_projections(g, cfg);
    let detected = tension::detect(&proj.inputs(page_k));
    if detected.is_empty() {
        return Vec::new();
    }

    // Verdict systems (SL-218 D1): the full pipeline compile always; a fresh
    // human-rows-only compile when the knob is on. The knob-on verdict reads the
    // human system; the full system stays available for the AgentProposed fallback.
    let active: Vec<&Judgement> = pipeline.active_pairwise().iter().collect();
    let full_reach = Reachability::build(&pipeline.value.constraint_set);
    let knob_on = cfg.compare.demote_agent_evidence;
    let human = knob_on.then(|| {
        let cs = compile_human_only(
            &active,
            &pipeline.value.anchors,
            QuarantinePolicy::Symmetric,
        );
        let reach = Reachability::build(&cs);
        let counts = constraining_counts_by_class(&cs, &active);
        (cs, reach, counts)
    });
    let (verdict_cs, verdict_reach, verdict_counts) = match &human {
        Some((cs, reach, counts)) => (cs, reach, counts),
        None => (
            &pipeline.value.constraint_set,
            &full_reach,
            &pipeline.constraining_by_class,
        ),
    };

    detected
        .into_iter()
        .map(|t| {
            let grade = grade_pair(
                g,
                cfg,
                t.preferred,
                t.surfaced,
                (verdict_cs, verdict_reach, verdict_counts),
                (
                    &pipeline.value.constraint_set,
                    &full_reach,
                    &pipeline.constraining_by_class,
                ),
                knob_on,
            );
            t.with_grade(grade)
        })
        .collect()
}

/// The full compiled determinacy view of one system: its constraint set,
/// reachability, and per-class rater counts.
type SystemView<'a> = (
    &'a ConstraintSet,
    &'a Reachability,
    &'a BTreeMap<comparison::ClassId, RaterCounts>,
);

/// Grade one detected pair's `value_dim` ordering (design D6) via the SHIPPED
/// machinery: build each member's [`PairSide`](crate::comparison::PairSide) with
/// `eff_weight = m_self·c_other` through the shared [`pair_side`] resolver (the
/// elicit queue's own seam), evaluate [`determined`] over the verdict system,
/// then over the full system for the knob-on `AgentProposed` fallback. Counts come
/// from the system that issued the verdict (RV-271 F-2/F-3). A member with no
/// costing ⇒ `Projected` (no comparison class to grade).
fn grade_pair(
    g: &PriorityGraph,
    cfg: &PriorityConfig,
    preferred: EntityKey,
    surfaced: EntityKey,
    verdict: SystemView<'_>,
    full: SystemView<'_>,
    knob_on: bool,
) -> EvidenceGrade {
    let (Some((ma, ca, _)), Some((mb, cb, _))) = (
        g.item_costing(&preferred, cfg),
        g.item_costing(&surfaced, cfg),
    ) else {
        return EvidenceGrade::Projected;
    };
    let (pa, pb) = (preferred.canonical(), surfaced.canonical());
    // `eff_weight = m_self · c_other` (design D6), tracked for each member.
    let evaluate = |sys: SystemView<'_>| -> (bool, RaterCounts) {
        let (cs, reach, by_class) = sys;
        let sa = pair_side(cs, &pa, ma * cb);
        let sb = pair_side(cs, &pb, mb * ca);
        let det = determined(reach, &sa, &sb).is_determined();
        (det, pair_counts(by_class, &sa.class, &sb.class))
    };
    let (det_verdict, verdict_counts) = evaluate(verdict);
    // The full system is only consulted for the knob-on `AgentProposed` fallback;
    // knob-off it IS the verdict system (grade ignores these).
    let (det_full, full_counts) = if knob_on {
        evaluate(full)
    } else {
        (det_verdict, verdict_counts)
    };
    tension::grade(knob_on, det_verdict, verdict_counts, det_full, full_counts)
}

/// Sum the constraining rater counts of a pair's two classes (deduped when the
/// pair shares a class), from the producing system's per-class map.
fn pair_counts(
    by_class: &BTreeMap<comparison::ClassId, RaterCounts>,
    a: &str,
    b: &str,
) -> RaterCounts {
    let mut out = by_class.get(a).copied().unwrap_or_default();
    if a != b {
        let other = by_class.get(b).copied().unwrap_or_default();
        out.human += other.human;
        out.agent += other.agent;
    }
    out
}

/// The value-source block for one entity — the SL-220 §3 evidence ladder made
/// legible, mirroring `graph::effective_raw_value` rung-for-rung: anchored
/// claim (pin / human, rung 1) > projection (bounds + rater split / gauge,
/// rung 2) > agent / migrated prior (rungs 3–4) > unmigrated `[value]` facet
/// (rung 5, transitional). `None` for a non-value-bearing kind, and for the
/// default floor (rung 6 is a NUMERIC floor, not a value-SOURCE worth a
/// citation — citing it would regress every `explain` golden with no
/// comparison evidence; the S3 precedent).
///
/// The SINGLE precedence source (SL-217 PHASE-03): the elicit participant
/// value block maps this `ReasonKind` to `{provenance, point}` (via the D11
/// `value_source_token`) and reads the STRUCTURAL bounds via
/// [`class_bounds_structural`] separately (this variant has already flattened
/// them for the human explain contract).
///
/// A projection hit with `Authored` provenance is a class-mate's anchored
/// claim hoisted onto the item's class (the item's OWN claim would have won
/// at rung 1) — reported as projected, the rung that actually fed scoring;
/// the tier-attributed hoist render is PHASE-06 material.
/// Convert a resolved claim into its render reason (SL-220 PHASE-06, design
/// §6) — the SINGLE claim→`ReasonKind` mapping shared by the anchored (rung 1)
/// and prior (rungs 3–4) arms and by `show`'s pipeline-only resolver. Pin tier
/// routes to [`ReasonKind::ValuePin`] (carries `basis`); every other tier to
/// [`ReasonKind::ValueClaim`]. Attribution rides through only for a singleton
/// resolution (`claim.attribution` is `None` for corroboration/conflict). A
/// migrated claim reads its timestamp from `observed_at` (the render frames it
/// as `observed`); every other tier reads `date`.
pub(crate) fn claim_reason(claim: &comparison::ResolvedClaim, conflict: Vec<String>) -> ReasonKind {
    let contested = claim.conflict.as_ref().map(|c| ContestedClaim {
        low: c.low,
        high: c.high,
        rows: claim.rows,
    });
    let attr = claim.attribution.clone().unwrap_or_default();
    // Migrated rows carry `observed_at`, not `date`; every other tier the reverse.
    let date = if matches!(claim.tier, comparison::ClaimTier::Migrated) {
        attr.observed_at
    } else {
        attr.date
    };
    match claim.tier {
        comparison::ClaimTier::Pin => ReasonKind::ValuePin {
            value: claim.value,
            conflict,
            by: attr.by,
            date,
            basis: attr.basis,
            contested,
        },
        tier => ReasonKind::ValueClaim {
            value: claim.value,
            tier,
            conflict,
            by: attr.by,
            date,
            contested,
        },
    }
}

pub(crate) fn value_source_reason(
    g: &PriorityGraph,
    key: EntityKey,
    pipeline: &comparison::Pipeline,
) -> Option<ReasonKind> {
    let attrs = g.attrs.get(&key)?;
    if !crate::kinds::is_value_bearing(key.prefix) {
        return None;
    }
    // Rung 5 fallback is the entity's own authored `[value]` facet — the ONE
    // sanctioned ladder read of `EntityFacets.value` (EX-3 enumerated exception).
    let facet_value = attrs.facets.value.as_ref().map(|v| v.value);
    resolve_value_reason(pipeline, &key.canonical(), facet_value)
}

/// The kind-AGNOSTIC value ladder resolver (SL-220 PHASE-06, design §3/§6) —
/// resolves one canonical id's value-source `ReasonKind` from the comparison
/// pipeline alone, with the entity's `[value]` facet magnitude as the rung-5
/// fallback. The SINGLE ladder walk shared by the graph-gated
/// [`value_source_reason`] (rows/explain, value-bearing kinds only) and `show`'s
/// pipeline-only resolver (design §6 — `ClaimResolution` captures claims for
/// EVERY subject including non-scored kinds, D7, so a record's human claim
/// resolves here too). `None` ⇔ no evidence at any rung (the line is omitted —
/// never the scoring floor).
pub(crate) fn resolve_value_reason(
    pipeline: &comparison::Pipeline,
    canonical: &str,
    facet_value: Option<f64>,
) -> Option<ReasonKind> {
    let claims = &pipeline.value_claims;
    // Rung 1: an anchored claim (Pin/Human) — row-less included (scope R1).
    if let Some(claim) = claims.anchored.get(canonical) {
        let conflict = anchor_conflict_citation(&pipeline.value.constraint_set, canonical);
        return Some(claim_reason(claim, conflict));
    }
    // Rung 2: the comparison projection (anchors claim-derived post-flip).
    if let Some(&(value, provenance)) = pipeline.value.projection.get(canonical) {
        return Some(match provenance {
            comparison::ValueProvenance::Authored | comparison::ValueProvenance::Projected => {
                let (lower, upper) = class_bounds(&pipeline.value.constraint_set, canonical);
                let counts = class_rater_counts(pipeline, canonical);
                ReasonKind::ValueProjected {
                    value,
                    lower,
                    upper,
                    human: counts.human,
                    agent: counts.agent,
                }
            }
            comparison::ValueProvenance::Gauge => {
                let counts = class_rater_counts(pipeline, canonical);
                ReasonKind::ValueGauge {
                    value,
                    judgements: counts.total(),
                }
            }
        });
    }
    // Rungs 3–4: the below-projection priors (agent, then migrated — the
    // within-map contest is already resolved by the claims pass).
    if let Some(claim) = claims.priors.get(canonical) {
        // Priors never anchor compile (D4) — no anchor-conflict citation.
        return Some(claim_reason(claim, Vec::new()));
    }
    // Rung 5: the unmigrated authored facet (zero claim rows by construction here).
    facet_value.map(|value| ReasonKind::ValueUnmigratedFacet { value })
}

/// The impure `show` value-line helper (SL-220 PHASE-06, design §6) — the ONE
/// scan-threading seam every entity `show` shell calls, dissolving the former
/// nine-fold `format_value_normal` duplication. Loads the comparison pipeline,
/// walks the ladder for `canonical` (with the entity's own `[value]` facet as
/// the rung-5 fallback), and renders the resolved provenance line. `kind` is the
/// entity kind prefix — a non-value-bearing kind (record/governance/REV) still
/// renders its captured claim, annotated `scoring-inert` (D7). `Ok(None)` ⇔ no
/// evidence at any rung (the line is omitted — matching today's absent-facet
/// behaviour, never the `1.0` default).
pub(crate) fn show_value_line(
    root: &Path,
    canonical: &str,
    facet_value: Option<f64>,
    kind: &str,
    value_unit: &str,
) -> anyhow::Result<Option<String>> {
    let pipeline = graph::load_comparison_pipeline_for_root(root)?;
    Ok(value_line_from_pipeline(
        &pipeline,
        canonical,
        facet_value,
        kind,
        value_unit,
    ))
}

/// The pure body of [`show_value_line`] over an ALREADY-LOADED pipeline — for
/// callers that render many entities in one scan and must not reload the
/// pipeline per entity (`lazyspec`'s spec catalog, `retrieve`'s memory loop).
/// Same ladder walk, same render, same scoring-inert annotation.
pub(crate) fn value_line_from_pipeline(
    pipeline: &comparison::Pipeline,
    canonical: &str,
    facet_value: Option<f64>,
    kind: &str,
    value_unit: &str,
) -> Option<String> {
    let reason = resolve_value_reason(pipeline, canonical, facet_value)?;
    let inert = (!crate::kinds::is_value_bearing(kind)).then_some(kind);
    super::render::show_value_render(&reason, value_unit, inert)
}

/// The SL-219 PHASE-06 cost-source block for one entity (design §5): the
/// `est_cost` ladder made legible — authored (own `[estimate]` facet, the
/// operator pin) > projected (bounds + rater split) > bare anchor; the gauge
/// flag discloses that scoring used the bare anchor while sizing evidence
/// merely ordered a gauge component (D2 honesty — gauge never divides).
///
/// GATED on est engagement (`pipeline.estimate.projection` non-empty): a corpus
/// with zero est-domain rows shows NO cost block, so every pre-SL-219 `explain`
/// golden stays byte-identical (the value-source S3 precedent — a bare divisor
/// is a numeric floor, not a citable source until the est system is live). Only
/// value-bearing kinds consume a divisor, so records (est-admissible anchors)
/// carry no cost block.
pub(crate) fn cost_source_reason(
    g: &PriorityGraph,
    key: EntityKey,
    pipeline: &comparison::Pipeline,
    cfg: &PriorityConfig,
) -> Option<ReasonKind> {
    let attrs = g.attrs.get(&key)?;
    if !crate::kinds::is_value_bearing(key.prefix) {
        return None;
    }
    if pipeline.estimate.projection.is_empty() {
        return None;
    }
    let canonical = key.canonical();
    // Shape 1 (own authored `[estimate]` facet): the operator pin — the ladder's
    // authored branch, β-resolved via the ONE formula site.
    if let Some(e) = attrs.facets.estimate.as_ref() {
        let est_cost = graph::authored_est_cost((e.lower, e.upper), &cfg.estimate);
        return Some(ReasonKind::CostAuthored {
            est_cost,
            pin: Some((e.lower, e.upper, cfg.estimate.skew)),
        });
    }
    let margin = cfg.estimate.margin;
    let absent = g.cost_ctx.absent;
    let max_estimate = max_authored_upper(g);
    // Shapes 2 / gauge (est projection engagement).
    if let Some(&(cost, provenance)) = pipeline.estimate.projection.get(&canonical) {
        return Some(match provenance {
            // A facet-less member hoisted onto an anchored class by an `equal`
            // merge (design §4 — provenance Authored, cost inherited).
            comparison::ValueProvenance::Authored => ReasonKind::CostAuthored {
                est_cost: cost,
                pin: None,
            },
            comparison::ValueProvenance::Projected => {
                let (lower, upper) = class_bounds(&pipeline.estimate.constraint_set, &canonical);
                let counts = est_class_rater_counts(pipeline, &canonical);
                ReasonKind::CostProjected {
                    est_cost: cost,
                    lower,
                    upper,
                    human: counts.human,
                    agent: counts.agent,
                }
            }
            // Gauge: scoring used the BARE ANCHOR (the feed excludes gauge, so
            // the ladder falls to `ctx.absent`); the render never implies gauge
            // fed the divisor (D2). `judgements` ordered it, nothing more.
            comparison::ValueProvenance::Gauge => {
                let counts = est_class_rater_counts(pipeline, &canonical);
                ReasonKind::CostGauge {
                    est_cost: absent,
                    max_estimate,
                    margin,
                    judgements: counts.total(),
                }
            }
        });
    }
    // Shape 3: est-admissible bare item, no projection engagement — the divisor
    // scoring actually used is the bare anchor (D7).
    Some(ReasonKind::CostBareAnchor {
        est_cost: absent,
        max_estimate,
        margin,
    })
}

/// The est system's constraining-judgement rater split for the est class
/// `canonical` belongs to (the cost-source "projected" shape's `(human,
/// agent)` disclosure) — the est-domain analog of [`class_rater_counts`], off
/// the est constraint set + the est split (`NoConstraint` rows excluded, S3).
fn est_class_rater_counts(pipeline: &comparison::Pipeline, canonical: &str) -> RaterCounts {
    pipeline
        .estimate
        .constraint_set
        .classes
        .get(canonical)
        .and_then(|class| pipeline.est_constraining_by_class.get(class))
        .copied()
        .unwrap_or_default()
}

/// The maximum non-terminal authored `upper` in the corpus — the bare anchor's
/// `max_upper` before `+ margin` (`ctx.absent = max_upper + margin`). `None`
/// in the empty-corpus fallback (no non-terminal authored estimate, `absent =
/// 1.0`). Mirrors `graph::bare_cost_anchor`'s fold so the cost-source render
/// can decompose `ctx.absent` into `(max_estimate, margin)` without widening
/// the pure `CostCtx` — a display-only read over the already-built graph.
fn max_authored_upper(g: &PriorityGraph) -> Option<f64> {
    g.attrs
        .values()
        .filter(|a| status_class(a.kind, a.status.as_deref()) != StatusClass::Terminal)
        .filter_map(|a| a.facets.estimate.as_ref().map(|e| e.upper))
        .max_by(f64::total_cmp)
}

/// The constraining-judgement rater split for the class `canonical` belongs
/// to, or a zero split when the entity carries no comparison evidence.
fn class_rater_counts(
    pipeline: &comparison::Pipeline,
    canonical: &str,
) -> crate::comparison::RaterCounts {
    pipeline
        .value
        .constraint_set
        .classes
        .get(canonical)
        .and_then(|class| pipeline.constraining_by_class.get(class))
        .copied()
        .unwrap_or_default()
}

/// The C6 display bounds for `canonical`'s class, as plain scalars (`None` =
/// unbounded that side) — the value-source "projected" shape's `bounds
/// (lower ‥ upper)` fragment.
fn class_bounds(cs: &comparison::ConstraintSet, canonical: &str) -> (Option<f64>, Option<f64>) {
    let Some(class) = cs.classes.get(canonical) else {
        return (None, None);
    };
    let Some(bounds) = cs.bounds.get(class) else {
        return (None, None);
    };
    (bound_value(bounds.lower), bound_value(bounds.upper))
}

fn bound_value(b: comparison::Bound) -> Option<f64> {
    match b {
        comparison::Bound::Unbounded => None,
        comparison::Bound::Open(v) | comparison::Bound::Closed(v) => Some(v),
    }
}

/// The C6 value interval for `canonical`'s class as the STRUCTURAL
/// [`comparison::ValueBounds`] (open/closed/unbounded retained) — the elicit
/// JSON surface's `value.bounds` source (SL-217 PHASE-03, design §3/D16). The
/// human `explain` path deliberately flattens via [`class_bounds`]; this keeps
/// the open/closed distinction the web review required (`[null, 2.8]` loses
/// it). `None` when the entity is not (yet) in a compiled class.
pub(crate) fn class_bounds_structural(
    cs: &comparison::ConstraintSet,
    canonical: &str,
) -> Option<comparison::ValueBounds> {
    let class = cs.classes.get(canonical)?;
    cs.bounds.get(class).copied()
}

/// Every OTHER class this `canonical`'s class was found to conflict with (an
/// `AnchorConflict` citation) — the value-source "authored" shape's optional
/// finding reference. Empty when the entity carries no comparison anchor
/// conflict.
fn anchor_conflict_citation(cs: &comparison::ConstraintSet, canonical: &str) -> Vec<String> {
    let Some(class) = cs.classes.get(canonical) else {
        return Vec::new();
    };
    let mut others = std::collections::BTreeSet::new();
    for reason in cs.quarantined.values() {
        if let comparison::QuarantineReason::AnchorConflict { pairs } = reason {
            for (x, y) in pairs {
                if x == class {
                    others.insert(y.clone());
                }
                if y == class {
                    others.insert(x.clone());
                }
            }
        }
    }
    others.into_iter().collect()
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

/// `findings` (SL-194) — the impure shell that owns ALL disk for the interestingness
/// catalogue: ONE `scan_entities`, ONE `config::load`, then the base build plus the β
/// endpoint sweep (`beta_endpoints`), before delegating to the PURE graph-only
/// `findings::detect` (design §The purity boundary). `beta_endpoints` returns `None` when
/// no non-terminal interval estimate exists, so the β-family findings simply do not fire.
pub(crate) fn findings(root: &Path) -> anyhow::Result<Vec<super::findings::Finding>> {
    let scanned = relation_graph::scan_entities(root, &mut vec![], ScanMode::default())?;
    let cfg = super::config::load(root);
    // SL-213 PHASE-05/06: ONE comparison-tier PIPELINE over the shared scan,
    // reused across the base build, the β endpoint sweep, AND the new
    // comparison-domain detectors (the pipeline is scan-derived, not
    // cfg-swept — a second load would just re-read the same ledger for the
    // same answer).
    let pipeline = graph::load_comparison_pipeline(root, &scanned, &cfg)?;
    let cost_feed = comparison::cost_feed(&pipeline.estimate.projection);
    let base = graph::build_from_with_cfg(
        &scanned,
        root,
        &cfg,
        &pipeline.value.projection,
        &cost_feed,
        &pipeline.value_claims,
    )?;
    let betas = beta_endpoints(
        &scanned,
        root,
        &cfg,
        &pipeline.value.projection,
        &cost_feed,
        &pipeline.value_claims,
    )?;
    let mut findings = super::findings::detect(&base, &cfg, betas.as_ref());
    findings.extend(super::findings::comparison_findings(&pipeline));
    findings.sort_by(|a, b| {
        a.kind_label()
            .cmp(b.kind_label())
            .then(b.magnitude().total_cmp(&a.magnitude()))
    });
    Ok(findings)
}

/// Whether the corpus carries a NON-terminal interval estimate (`lower < upper`) — the
/// precondition for the β sweep to say anything. A point estimate (`lower == upper`) is
/// β-invariant (`est_cost` is constant in `skew`), and terminal items are excluded from
/// the cost anchor, so neither can produce a contested ordering.
fn has_nonterminal_interval(scanned: &[relation_graph::ScannedEntity]) -> bool {
    scanned.iter().any(|e| {
        status_class(e.kind, e.status.as_deref()) != StatusClass::Terminal
            && e.estimate.as_ref().is_some_and(|est| est.lower < est.upper)
    })
}

/// The β endpoint sweep (SL-194 PHASE-02) — rebuild the graph at `skew = BETA_LO` and
/// `skew = BETA_HI` over the SAME `scanned` (design D4 / §Purity boundary). Returns
/// `Some(BetaEndpoints)` iff a non-terminal interval estimate exists; else `None` — no
/// wasted builds, and the β-family stays silent (starved-until-estimates, R1). The
/// three reads (base + lo + hi) share the one scan; dep/seq topology is re-read per build
/// under the quiescent-tree precondition (R4).
///
/// `projected` (SL-213 PHASE-05) is the ONE comparison-tier projection the caller
/// loaded over the shared scan — shared across both endpoint builds rather than
/// re-derived per build (the projection is scan-derived, not cfg-swept).
/// `cost_feed` (SL-219 PHASE-04) rides the same contract: derived once from the
/// caller's pipeline, shared across both endpoint builds.
/// `claims` (SL-220 PHASE-05) likewise: the one claim resolution off the
/// caller's pipeline, shared — claims are scan-derived, not cfg-swept.
pub(crate) fn beta_endpoints(
    scanned: &[relation_graph::ScannedEntity],
    root: &Path,
    cfg: &super::config::PriorityConfig,
    projected: &ValueProjection,
    cost_feed: &comparison::CostFeed,
    claims: &comparison::ClaimResolution,
) -> anyhow::Result<Option<super::findings::BetaEndpoints>> {
    if !has_nonterminal_interval(scanned) {
        return Ok(None);
    }
    let mut lo_cfg = cfg.clone();
    lo_cfg.estimate.skew = super::findings::BETA_LO;
    let mut hi_cfg = cfg.clone();
    hi_cfg.estimate.skew = super::findings::BETA_HI;
    let lo = graph::build_from_with_cfg(scanned, root, &lo_cfg, projected, cost_feed, claims)?;
    let hi = graph::build_from_with_cfg(scanned, root, &hi_cfg, projected, cost_feed, claims)?;
    Ok(Some(super::findings::BetaEndpoints { lo, hi }))
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
        let from_survey = survey(root, false, false).unwrap();
        let from_for_map = survey_for_map(&g, false);

        assert_eq!(
            from_survey, from_for_map,
            "survey_for_map must match survey output exactly"
        );
    }

    // ── SL-177: actionability view set-preserved after WORK_PREFIXES → is_value_bearing ──

    /// The actionability view node set is unchanged after promoting the local
    /// `WORK_PREFIXES` to `kinds::is_value_bearing` — the same six kind prefixes
    /// are admitted, same exclusion of governance/knowledge entities.
    #[test]
    fn actionability_view_set_preserved_after_value_bearing_promotion() {
        let dir = tmp();
        let root = dir.path();
        // The old WORK_PREFIXES = ["SL", "ISS", "IMP", "CHR", "RSK", "IDE"] —
        // exactly VALUE_BEARING. Seed one of each non-work kind to prove they're excluded.
        seed_issue(root, 1, "open", "", &[]);
        // Also seed a requirement (governance — excluded from actionability view).
        write(
            root,
            ".doctrine/requirement/005/requirement-005.toml",
            "id = 5\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n",
        );
        write(root, ".doctrine/requirement/005/requirement-005.md", "r\n");
        let g = build(root).unwrap();
        let view = survey_view_for_map(&g, false);
        // Only the work/value-bearing entity appears; the requirement does NOT.
        assert_eq!(view.nodes.len(), 1, "only the ISS appears, not REQ-005");
        assert_eq!(view.nodes[0].id, "ISS-001");
        // The set of kind prefixes in the view matches the old WORK_PREFIXES set.
        let kind_set: std::collections::BTreeSet<&str> =
            view.nodes.iter().map(|n| n.kind.as_str()).collect();
        let expected: std::collections::BTreeSet<&str> = ["ISS"].iter().copied().collect();
        assert_eq!(kind_set, expected, "only work/value-bearing kinds appear");
    }

    // ── SL-133 dedicated helpers + ordering proofs (VT-5 / VT-7 / VA-1) ───

    /// Seed an open backlog issue with an explicit `[value]` over a fixed estimate
    /// (lower 0, upper 10), plus optional `needs`/`after` relationship lines.
    /// `value` of `v` ⇒ base = value_coeff(1.0) · v · 1.0 / est_cost (6.5) = v/6.5.
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
        next(root).unwrap().rows.into_iter().map(|r| r.id).collect()
    }

    fn survey_ids(root: &Path) -> Vec<String> {
        survey(root, false, false)
            .unwrap()
            .into_iter()
            .map(|r| r.id)
            .collect()
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
        let rsk1 = EntityKey {
            prefix: "RSK",
            id: 1,
        };
        let rsk2 = EntityKey {
            prefix: "RSK",
            id: 2,
        };
        // RSK-001's leverage = 0.5 · base(ISS-010). base(ISS-010) = 100.0/6.5.
        //   leverage = 50.0/6.5 ≈ 7.692307692.
        //   SL-177 PHASE-02: RSK-001 value_dim = default 1.0 / absent(11.0) ≈ 0.090909.
        //   score = lev + value_dim = 50.0/6.5 + 1.0/11.0 ≈ 7.783216783.
        assert!(
            (channels::score(&g, rsk1) - (50.0 / 6.5 + 1.0 / 11.0)).abs() < 1e-9,
            "RSK-001 leverages the one high-value dependent: got {}",
            channels::score(&g, rsk1)
        );
        // RSK-002 gates five zero-value ideas → leverage 0.
        // SL-177 PHASE-02: value_dim = default 1.0 / absent(11.0) ≈ 0.090909.
        assert!(
            (channels::score(&g, rsk2) - 1.0 / 11.0).abs() < 1e-9,
            "RSK-002 gates only zero-value ideas → score = value_dim only: got {}",
            channels::score(&g, rsk2)
        );
        // survey orders RSK-001 (score 10) BEFORE RSK-002 (score 0) — the old
        // inbound-count (5 vs 1) would have ranked RSK-002 first.
        let ids = survey_ids(root);
        let p1 = ids.iter().position(|x| x == "RSK-001").unwrap();
        let p2 = ids.iter().position(|x| x == "RSK-002").unwrap();
        assert!(
            p1 < p2,
            "RSK-001 outranks RSK-002 by score (not inbound count): {ids:?}"
        );
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
        // leverage(ISS-002) = 0.5·(base(ISS-003)+0) = 0.5·(200/6.5) = 100/6.5
        //   = 1000/65 ≈ 15.384615385
        // leverage(ISS-001) = 0.5·(base(ISS-002)+leverage(ISS-002))
        //   = 0.5·(0 + 100/6.5) = 50/6.5 ≈ 7.692307692
        let deep = channels::score(&g, k(1));
        // leverage(ISS-010) = 0.5·(base(ISS-011)+0) = 0.5·(10/6.5) = 5/6.5 ≈ 0.769230769
        let shallow = channels::score(&g, k(10));
        assert!(
            (deep - 50.0 / 6.5).abs() < 1e-9,
            "deep blocker recursive leverage = 50/6.5: got {deep}"
        );
        assert!(
            (shallow - 5.0 / 6.5).abs() < 1e-9,
            "shallow blocker leverage = 5/6.5: got {shallow}"
        );
        let ids = survey_ids(root);
        let pd = ids.iter().position(|x| x == "ISS-001").unwrap();
        let ps = ids.iter().position(|x| x == "ISS-010").unwrap();
        assert!(
            pd < ps,
            "deep blocker of a valuable cone outranks the shallow one: {ids:?}"
        );
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
        assert!(
            evicted_total > 0,
            "the seq 2-cycle must produce an eviction"
        );

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
                .get(&EntityKey {
                    prefix: "ISS",
                    id: 2,
                })
                .map(|s| {
                    s.contains(&EntityKey {
                        prefix: "ISS",
                        id: 1,
                    })
                })
                .unwrap_or(false)
        };
        if surviving_pred_of_2 {
            assert!(
                p1 < p2,
                "surviving edge orders ISS-001 before ISS-002: {ids:?}"
            );
        } else {
            assert!(
                p2 < p1,
                "evicted edge does NOT re-impose precedence; higher-score ISS-002 leads: {ids:?}"
            );
        }
    }

    // ── SL-194 VT-1: beta_endpoints — Some over interval estimate, None otherwise ──

    #[test]
    fn beta_endpoints_some_over_interval_estimate_none_over_estimate_free() {
        // Interval-estimate corpus: seed_valued authors [estimate] lower 0 < upper 10 on
        // an OPEN (non-terminal) item → a β sweep has something to perturb.
        let dir = tmp();
        let root = dir.path();
        seed_valued(root, 1, 10.0, "");
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = super::super::config::load(root);
        assert!(
            beta_endpoints(
                &scanned,
                root,
                &cfg,
                &ValueProjection::new(),
                &Default::default(),
                &Default::default(),
            )
            .unwrap()
            .is_some(),
            "a non-terminal interval estimate yields Some"
        );

        // Estimate-free corpus: a bare open issue authors no [estimate] → None (no wasted
        // builds; the β-family stays silent).
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_issue(root2, 1, "open", "", &[]);
        let scanned2 =
            relation_graph::scan_entities(root2, &mut vec![], ScanMode::default()).unwrap();
        let cfg2 = super::super::config::load(root2);
        assert!(
            beta_endpoints(
                &scanned2,
                root2,
                &cfg2,
                &ValueProjection::new(),
                &Default::default(),
                &Default::default(),
            )
            .unwrap()
            .is_none(),
            "an estimate-free corpus yields None"
        );
    }

    /// A terminal item's interval must NOT trip the sweep — the precondition is a
    /// NON-terminal interval (terminals are excluded from the cost anchor).
    #[test]
    fn beta_endpoints_none_when_only_terminal_has_interval() {
        let dir = tmp();
        let root = dir.path();
        // A CLOSED (terminal) issue carrying an interval estimate, nothing else.
        write(
            root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"closed\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [estimate]\nlower = 0.0\nupper = 10.0\n",
        );
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "b\n");
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = super::super::config::load(root);
        assert!(
            beta_endpoints(
                &scanned,
                root,
                &cfg,
                &ValueProjection::new(),
                &Default::default(),
                &Default::default(),
            )
            .unwrap()
            .is_none(),
            "a terminal-only interval does not arm the sweep"
        );
    }

    /// VA-1: the `explain` Score reason exposes the full breakdown and the human render
    /// reads it correctly.
    #[test]
    fn va1_explain_exposes_full_score_breakdown() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 value 50 over est_cost 6.5 → base 50/6.5 ≈ 7.6923;
        // one dependent ISS-002 (value 100, base 100/6.5 ≈ 15.3846) needs it
        // → leverage(ISS-001) = 0.5·15.3846 ≈ 7.6923. No referencers → optionality 0.
        seed_valued(root, 1, 50.0, "");
        seed_valued(root, 2, 100.0, "needs = [\"ISS-001\"]\n");

        let ex = explain(root, "ISS-001").unwrap();
        // base = value_dim = 50/6.5 (no risk); leverage = 0.5 * 100/6.5 = 50/6.5
        let expected_base = 50.0 / 6.5;
        let expected_lev = 50.0 / 6.5;
        match ex.score {
            ReasonKind::Score {
                base,
                value_dim,
                risk_dim,
                leverage,
                optionality,
                total,
            } => {
                assert!((base - expected_base).abs() < 1e-9, "base = 50/6.5");
                assert!(
                    (value_dim - expected_base).abs() < 1e-9,
                    "value_dim = 50/6.5"
                );
                assert!(risk_dim.abs() < 1e-9, "risk_dim 0");
                assert!((leverage - expected_lev).abs() < 1e-9, "leverage = 50/6.5");
                assert!(optionality.abs() < 1e-9, "optionality 0");
                assert!(
                    (total - (expected_base + expected_lev)).abs() < 1e-9,
                    "total = base+lev"
                );
            }
            other => panic!("explain score must be a Score reason, got {other:?}"),
        }
        // Human render reads the breakdown line correctly.
        let expected_total = expected_base + expected_lev;
        let human = crate::priority::render::explain_human(&ex);
        assert!(
            human.contains(&format!(
                "score: {expected_total:.1} (base {expected_base:.1} [value {expected_base:.1}, risk 0.0], leverage {expected_lev:.1}, optionality 0.0)"
            )),
            "human explain renders the full breakdown: {human}"
        );
    }

    // ── VT-3: tension grading assembly (design VT-G) ──────────────────────────
    //
    // The pure grade vocabulary is unit-tested in `tension.rs` (VT-2); these
    // exercise the SURFACE assembly end-to-end over a real graph + comparison
    // pipeline: detection flags a value_dim-vs-delivery inversion, and
    // `grade_pair` threads `eff_weight = m·c_other` through the shipped
    // `pair_side`/`determined()` machinery so the grade tracks the value_dim
    // claim (never a raw v/c one — SL-217 D6's multiplier-folded objective is the
    // sole predicate).

    /// Hand-author a session-of-one: ISS-001 preferred over ISS-002 on value, by
    /// `rater` (the wire shape `compare record` mints).
    fn prefer_a_over_b(root: &Path, rater: &str) {
        write(
            root,
            ".doctrine/comparisons/2026-01-01-vt3.toml",
            &format!(
                "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
                 [session]\nuid = \"vt3-sess\"\ndate = \"2026-01-01\"\n\n\
                 [[judgement]]\nuid = \"vt3-row\"\nseq = 0\na = \"ISS-001\"\nb = \"ISS-002\"\n\
                 response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\n\
                 form = \"order\"\nrater = \"{rater}\"\ndate = \"2026-01-01\"\n"
            ),
        );
    }

    /// The explain-path assembly: scan → pipeline → graph → `graded_tensions`
    /// over the full frontier (mirrors `explain()`).
    fn assemble_tensions(root: &Path) -> Vec<Tension> {
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = crate::priority::config::load(root);
        let pipeline = graph::load_comparison_pipeline(root, &scanned, &cfg).unwrap();
        let g = graph::build_from_with_cfg(
            &scanned,
            root,
            &cfg,
            &pipeline.value.projection,
            &comparison::cost_feed(&pipeline.estimate.projection),
            &pipeline.value_claims,
        )
        .unwrap();
        graded_tensions(&g, &pipeline, &cfg, usize::MAX)
    }

    #[test]
    fn vt3_structural_tension_human_row_graded_determined() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 (high value) is sequenced AFTER ISS-002 (low value): structure
        // forces the higher-value item to surface behind the lower one, so
        // ISS-002 surfaces first while ISS-001 outranks it on value_dim — the
        // tension. A HUMAN prefer-a row determines that value_dim ordering.
        seed_valued(root, 1, 8.0, "after = [{ to = \"ISS-002\", rank = 0 }]\n");
        seed_valued(root, 2, 1.0, "");
        prefer_a_over_b(root, "human");

        let ts = assemble_tensions(root);
        assert_eq!(ts.len(), 1, "one structural tension: {ts:?}");
        assert_eq!(ts[0].surfaced.canonical(), "ISS-002");
        assert_eq!(ts[0].preferred.canonical(), "ISS-001");
        assert!(matches!(
            ts[0].cause,
            tension::TensionCause::Structure { .. }
        ));
        match ts[0].grade {
            EvidenceGrade::Determined { counts } => assert!(
                counts.human >= 1 && counts.agent == 0,
                "human-determined ⇒ human counts only (no agent rows cited): {counts:?}"
            ),
            other => panic!("expected Determined, got {other:?}"),
        }
    }

    #[test]
    fn vt3_agent_row_knob_on_graded_agent_proposed() {
        let dir = tmp();
        let root = dir.path();
        // Same corpus, but the deciding row is an AGENT and the demotion knob is
        // on: the human system has no rows to retire the question (indeterminate),
        // the full system determines it ⇒ AgentProposed with the full system's
        // (agent) counts, labelled unconfirmed (SL-218 D1 / RV-271 F-2).
        write(
            root,
            ".doctrine/doctrine.toml",
            "[priority.compare]\ndemote_agent_evidence = true\n",
        );
        seed_valued(root, 1, 8.0, "after = [{ to = \"ISS-002\", rank = 0 }]\n");
        seed_valued(root, 2, 1.0, "");
        prefer_a_over_b(root, "agent");

        let ts = assemble_tensions(root);
        assert_eq!(ts.len(), 1, "one structural tension: {ts:?}");
        match ts[0].grade {
            EvidenceGrade::AgentProposed { counts } => assert!(
                counts.agent >= 1,
                "agent-proposed carries the full-system agent counts: {counts:?}"
            ),
            other => {
                panic!("expected AgentProposed (human system indeterminate), got {other:?}")
            }
        }
    }

    // ── SL-219 PHASE-06 VT-1: cost-source block (design §5) ───────────────────
    //
    // The three shapes + the gauge flag line, exercised end-to-end over a real
    // scan → pipeline → graph, then rendered through the SINGLE
    // `render::cost_source_fragment` source. Rater split with `NoConstraint`
    // excluded from the counts (S3 precedent).

    /// Seed an open value-bearing issue with a verbatim facet tail (`[estimate]`
    /// / `[value]` tables, before the empty `[relationships]`).
    fn seed_cost(root: &Path, id: u32, facets: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I{id}\"\nkind = \"issue\"\nstatus = \"open\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 {facets}[relationships]\n"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    /// Hand-author an est-domain `more-work` session-of-one over the pair
    /// (`prefer-a` ⇒ `a` is the costlier item, D5). `resp` lets a caller mint an
    /// `incomparable` (→ `NoConstraint`) row to prove it is excluded from the
    /// rater split.
    fn capture_more_work(root: &Path, slot: &str, a: &str, b: &str, resp: &str, rater: &str) {
        write(
            root,
            &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
            &format!(
                "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
                 [session]\nuid = \"sess-{slot}\"\ndate = \"2026-01-01\"\n\n\
                 [[judgement]]\nuid = \"row-{slot}\"\nseq = 0\na = \"{a}\"\nb = \"{b}\"\n\
                 response = \"{resp}\"\ndomain = \"estimate\"\nframe = \"more-work\"\n\
                 form = \"order\"\nrater = \"{rater}\"\ndate = \"2026-01-01\"\n"
            ),
        );
    }

    /// The cost-source reason for `id`, over the same scan → pipeline → graph
    /// assembly `explain()` runs.
    fn cost_reason(root: &Path, id: &str) -> Option<ReasonKind> {
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = crate::priority::config::load(root);
        let pipeline = graph::load_comparison_pipeline(root, &scanned, &cfg).unwrap();
        let g = graph::build_from_with_cfg(
            &scanned,
            root,
            &cfg,
            &pipeline.value.projection,
            &comparison::cost_feed(&pipeline.estimate.projection),
            &pipeline.value_claims,
        )
        .unwrap();
        let key = parse_key(id).unwrap();
        cost_source_reason(&g, key, &pipeline, &cfg)
    }

    /// The rendered human fragment for `id`'s cost source (through the shared
    /// render source), or `""` when there is no block.
    fn cost_fragment(root: &Path, id: &str) -> String {
        cost_reason(root, id)
            .and_then(|r| crate::priority::render::cost_source_fragment(&r))
            .unwrap_or_default()
    }

    #[test]
    fn cost_source_none_without_est_engagement() {
        let dir = tmp();
        let root = dir.path();
        // An authored estimate + value, but ZERO est-domain rows ⇒ no cost block
        // (byte-identical to pre-SL-219 explain; the bare divisor is not a source).
        seed_cost(
            root,
            1,
            "[estimate]\nlower = 2.0\nupper = 8.0\n[value]\nvalue = 10.0\n",
        );
        assert!(
            cost_reason(root, "ISS-001").is_none(),
            "no est engagement ⇒ no cost-source block"
        );
    }

    #[test]
    fn cost_source_authored_shape_shows_pin_bounds_and_beta() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 authors its own [estimate] pin [2,8] → β-resolved 5.9; a
        // more-work row against a cheaper anchor makes the est system live.
        seed_cost(
            root,
            1,
            "[estimate]\nlower = 2.0\nupper = 8.0\n[value]\nvalue = 10.0\n",
        );
        seed_cost(
            root,
            2,
            "[estimate]\nlower = 1.0\nupper = 1.0\n[value]\nvalue = 10.0\n",
        );
        capture_more_work(root, "mw", "ISS-001", "ISS-002", "prefer-a", "human");

        match cost_reason(root, "ISS-001") {
            Some(ReasonKind::CostAuthored {
                est_cost,
                pin: Some((lower, upper, beta)),
            }) => {
                assert!((est_cost - 5.9).abs() < 1e-9, "β-resolved pin: {est_cost}");
                assert_eq!((lower, upper), (2.0, 8.0));
                assert!((beta - 0.65).abs() < 1e-9);
            }
            other => panic!("expected CostAuthored pin, got {other:?}"),
        }
        assert_eq!(
            cost_fragment(root, "ISS-001"),
            "est_cost 5.9 — authored [2.0 ‥ 8.0] · β 0.65"
        );
    }

    #[test]
    fn cost_source_projected_shape_rater_split_excludes_noconstraint() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 is bare; a more-work row (ISS-002 costlier) projects it below
        // the 8.0 anchor. An extra `incomparable` row touching ISS-001 is
        // NoConstraint — it must NOT inflate the rater split (S3).
        seed_cost(root, 1, "[value]\nvalue = 10.0\n");
        seed_cost(
            root,
            2,
            "[estimate]\nlower = 8.0\nupper = 8.0\n[value]\nvalue = 10.0\n",
        );
        capture_more_work(root, "mw", "ISS-002", "ISS-001", "prefer-a", "human");
        capture_more_work(root, "nc", "ISS-001", "ISS-002", "incomparable", "agent");

        match cost_reason(root, "ISS-001") {
            Some(ReasonKind::CostProjected {
                upper,
                human,
                agent,
                ..
            }) => {
                assert_eq!(upper, Some(8.0), "C6 upper bound at the anchor");
                assert_eq!((human, agent), (1, 0), "NoConstraint row excluded");
            }
            other => panic!("expected CostProjected, got {other:?}"),
        }
        assert!(
            cost_fragment(root, "ISS-001").contains(
                "projected · bounds (unbounded ‥ 8.0) · from 1 constraining sizing \
                           judgements (1 human, 0 agent)"
            ),
            "projected fragment: {}",
            cost_fragment(root, "ISS-001")
        );
    }

    #[test]
    fn cost_source_bare_anchor_shape_decomposes_max_and_margin() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 is bare and untouched by any est row; ISS-002/003 make the est
        // system live (max non-terminal upper = 10 ⇒ bare anchor 10 + 1 = 11).
        seed_cost(root, 1, "[value]\nvalue = 10.0\n");
        seed_cost(
            root,
            2,
            "[estimate]\nlower = 10.0\nupper = 10.0\n[value]\nvalue = 10.0\n",
        );
        seed_cost(
            root,
            3,
            "[estimate]\nlower = 1.0\nupper = 1.0\n[value]\nvalue = 10.0\n",
        );
        capture_more_work(root, "mw", "ISS-002", "ISS-003", "prefer-a", "human");

        match cost_reason(root, "ISS-001") {
            Some(ReasonKind::CostBareAnchor {
                est_cost,
                max_estimate,
                margin,
            }) => {
                assert!((est_cost - 11.0).abs() < 1e-9);
                assert_eq!(max_estimate, Some(10.0));
                assert!((margin - 1.0).abs() < 1e-9);
            }
            other => panic!("expected CostBareAnchor, got {other:?}"),
        }
        assert_eq!(
            cost_fragment(root, "ISS-001"),
            "est_cost 11.0 — bare anchor (max estimate 10.0 + margin 1.0)"
        );
    }

    #[test]
    fn cost_source_gauge_flag_shows_bare_anchor_never_the_divisor() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001/002 are both bare and mutually ordered → an anchor-free (gauge)
        // component. ISS-003 authors an estimate (no est row) so a bare anchor
        // exists (max 10 + margin 1 = 11). The gauge item's cost-source shows the
        // BARE ANCHOR (what scoring used) + a SEPARATE sizing-gauge line.
        seed_cost(root, 1, "[value]\nvalue = 10.0\n");
        seed_cost(root, 2, "[value]\nvalue = 10.0\n");
        seed_cost(
            root,
            3,
            "[estimate]\nlower = 10.0\nupper = 10.0\n[value]\nvalue = 10.0\n",
        );
        capture_more_work(root, "mw", "ISS-001", "ISS-002", "prefer-a", "human");

        match cost_reason(root, "ISS-001") {
            Some(ReasonKind::CostGauge {
                est_cost,
                max_estimate,
                margin,
                judgements,
            }) => {
                assert!(
                    (est_cost - 11.0).abs() < 1e-9,
                    "scoring used the bare anchor"
                );
                assert_eq!(max_estimate, Some(10.0));
                assert!((margin - 1.0).abs() < 1e-9);
                assert_eq!(judgements, 1);
            }
            other => panic!("expected CostGauge, got {other:?}"),
        }
        let frag = cost_fragment(root, "ISS-001");
        assert!(
            frag.contains("est_cost 11.0 — bare anchor (max estimate 10.0 + margin 1.0)"),
            "line 1 is the bare anchor (what scoring used): {frag}"
        );
        assert!(
            frag.contains(
                "sizing: gauge · ordered by 1 judgements, no estimated item in component — \
                 estimate any member to calibrate"
            ),
            "line 2 discloses gauge separately — never implies it fed the divisor: {frag}"
        );
    }
}
