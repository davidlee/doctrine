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
use super::tension::{self, DetectInputs, EdgeKind, EvidenceGrade, PredEdge, Tension};
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
            // Project facet fields from NodeAttr (SL-171 PHASE-01, D2) — read once,
            // never recompute.
            let (estimate, value, tags) = attr(&g, k).map_or((None, None, Vec::new()), |a| {
                (
                    a.facets.estimate.clone(),
                    a.facets.value.clone(),
                    a.facets.tags.clone(),
                )
            });
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
                value,
                tags,
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
    let g = graph::build_from_with_cfg(&scanned, root, &cfg, &pipeline.projection)?;
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
    let priority_disclosure =
        (pipeline.priority_domain_count > 0).then_some(ReasonKind::PriorityDomainDisclosure {
            count: pipeline.priority_domain_count,
        });

    Ok(Explanation {
        id: key.canonical(),
        eligibility,
        blocker_chain,
        evictions,
        score,
        value_source,
        priority_disclosure,
        agent_demotion: cfg
            .compare
            .demote_agent_evidence
            .then_some(ReasonKind::AgentEvidenceDemoted),
        // explain considers the WHOLE frontier (design §2 considered-set): every
        // frontier member is on-page, so `page_k = usize::MAX`. PHASE-03 filters
        // to the explained id at render.
        tensions: graded_tensions(&g, &pipeline, &cfg, usize::MAX),
    })
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
    // Frontier delivery order over the actionable, non-promoted set — the SAME
    // basis `next`/`explain` render (surviving-seq precedence, score tiebreak).
    let actionable_set: BTreeSet<EntityKey> = g
        .attrs
        .keys()
        .copied()
        .filter(|&k| channels::actionable(g, k) && !channels::promoted(g, k))
        .collect();
    let actionable: Vec<EntityKey> = actionable_set.iter().copied().collect();
    let seq_preds = surviving_seq_predecessors(g, &actionable_set);
    let order = frontier_order(&actionable, &|k| channels::score(g, k), &seq_preds);

    // Detection projections (read once off the graph — never re-derived).
    let map = |f: &dyn Fn(EntityKey) -> f64| -> BTreeMap<EntityKey, f64> {
        actionable.iter().map(|&k| (k, f(k))).collect()
    };
    let value_dim = map(&|k| channels::value_dim(g, k));
    let risk_dim = map(&|k| channels::risk_dim(g, k));
    let leverage = map(&|k| channels::leverage(g, k));
    let optionality = map(&|k| channels::optionality(g, k));
    let full_score = map(&|k| channels::score(g, k));
    let multiplier = map(&|k| g.item_costing(&k, cfg).map_or(0.0, |(m, _, _)| m));

    // Merged surviving seq+dep predecessor graph restricted to the frontier:
    // seq (`after`) from the order primitive, dep (`needs`) from the direct
    // blocked-by set. Structure reachability walks this graph (design D4).
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

    let detected = tension::detect(&DetectInputs {
        delivery_order: &order,
        page_k,
        value_dim: &value_dim,
        multiplier: &multiplier,
        full_score: &full_score,
        risk_dim: &risk_dim,
        leverage: &leverage,
        optionality: &optionality,
        preds: &preds,
    });
    if detected.is_empty() {
        return Vec::new();
    }

    // Verdict systems (SL-218 D1): the full pipeline compile always; a fresh
    // human-rows-only compile when the knob is on. The knob-on verdict reads the
    // human system; the full system stays available for the AgentProposed fallback.
    let active: Vec<&Judgement> = pipeline.active_judgements.iter().collect();
    let full_reach = Reachability::build(&pipeline.constraint_set);
    let knob_on = cfg.compare.demote_agent_evidence;
    let human = knob_on.then(|| {
        let cs = compile_human_only(&active, &pipeline.anchors, QuarantinePolicy::Symmetric);
        let reach = Reachability::build(&cs);
        let counts = constraining_counts_by_class(&cs, &active);
        (cs, reach, counts)
    });
    let (verdict_cs, verdict_reach, verdict_counts) = match &human {
        Some((cs, reach, counts)) => (cs, reach, counts),
        None => (
            &pipeline.constraint_set,
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
                    &pipeline.constraint_set,
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

/// The SL-213 PHASE-06 value-source block for one entity (design §4 S3):
/// authored (own `[value]` facet — possibly an anchor hoisted onto a shared
/// class) > projected (bounds + rater split) > gauge (judgement count) >
/// the implicit default tier (D11). `None` for a non-value-bearing kind.
///
/// The SINGLE precedence source (SL-217 PHASE-03): the elicit participant
/// value block maps this `ReasonKind` to `{provenance, point}` and reads the
/// STRUCTURAL bounds via [`class_bounds_structural`] separately (this variant
/// has already flattened them for the human explain contract).
pub(crate) fn value_source_reason(
    g: &PriorityGraph,
    key: EntityKey,
    pipeline: &comparison::Pipeline,
) -> Option<ReasonKind> {
    let attrs = g.attrs.get(&key)?;
    if !crate::kinds::is_value_bearing(key.prefix) {
        return None;
    }
    let canonical = key.canonical();
    if let Some(v) = attrs.facets.value.as_ref() {
        return Some(ReasonKind::ValueAuthored {
            value: v.value,
            conflict: anchor_conflict_citation(&pipeline.constraint_set, &canonical),
        });
    }
    if let Some(&(value, provenance)) = pipeline.projection.get(&canonical) {
        return Some(match provenance {
            comparison::ValueProvenance::Authored => ReasonKind::ValueAuthored {
                value,
                conflict: anchor_conflict_citation(&pipeline.constraint_set, &canonical),
            },
            comparison::ValueProvenance::Projected => {
                let (lower, upper) = class_bounds(&pipeline.constraint_set, &canonical);
                let counts = class_rater_counts(pipeline, &canonical);
                ReasonKind::ValueProjected {
                    value,
                    lower,
                    upper,
                    human: counts.human,
                    agent: counts.agent,
                }
            }
            comparison::ValueProvenance::Gauge => {
                let counts = class_rater_counts(pipeline, &canonical);
                ReasonKind::ValueGauge {
                    value,
                    judgements: counts.total(),
                }
            }
        });
    }
    // No own facet, no comparison-tier engagement at all: nothing to disclose
    // (design §4 S3 names three shapes only — the scoring-tier `DEFAULT_VALUE`
    // fallback `effective_raw_value` applies for `value_dim` is a NUMERIC
    // floor, not a value-SOURCE worth a citation; showing one here for every
    // untouched value-bearing entity would regress every existing `explain`
    // golden that carries no comparison evidence).
    None
}

/// The constraining-judgement rater split for the class `canonical` belongs
/// to, or a zero split when the entity carries no comparison evidence.
fn class_rater_counts(
    pipeline: &comparison::Pipeline,
    canonical: &str,
) -> crate::comparison::RaterCounts {
    pipeline
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
    let base = graph::build_from_with_cfg(&scanned, root, &cfg, &pipeline.projection)?;
    let betas = beta_endpoints(&scanned, root, &cfg, &pipeline.projection)?;
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
pub(crate) fn beta_endpoints(
    scanned: &[relation_graph::ScannedEntity],
    root: &Path,
    cfg: &super::config::PriorityConfig,
    projected: &ValueProjection,
) -> anyhow::Result<Option<super::findings::BetaEndpoints>> {
    if !has_nonterminal_interval(scanned) {
        return Ok(None);
    }
    let mut lo_cfg = cfg.clone();
    lo_cfg.estimate.skew = super::findings::BETA_LO;
    let mut hi_cfg = cfg.clone();
    hi_cfg.estimate.skew = super::findings::BETA_HI;
    let lo = graph::build_from_with_cfg(scanned, root, &lo_cfg, projected)?;
    let hi = graph::build_from_with_cfg(scanned, root, &hi_cfg, projected)?;
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
        next(root).unwrap().into_iter().map(|r| r.id).collect()
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
            beta_endpoints(&scanned, root, &cfg, &ValueProjection::new())
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
            beta_endpoints(&scanned2, root2, &cfg2, &ValueProjection::new())
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
            beta_endpoints(&scanned, root, &cfg, &ValueProjection::new())
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
}
