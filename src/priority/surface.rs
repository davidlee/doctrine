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
    Actionability, ActionabilityBlock, BlockersView, Explanation, NextRow, ReasonKind, SurveyRow,
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

/// `survey [--all]` (design §5.4) — the eligible set in importance order (D10).
///
/// Set: every `eligible` node, MINUS `promoted` backlog items (excluded as their own
/// reason, F1), UNLESS `all` reveals the full picture. With `all`, terminal +
/// promoted nodes are included too (the complete view). Sort (the empty
/// authored-priority slot collapses to): `actionability(Actionable first) →
/// consequence desc → canonical-id asc`.
pub(crate) fn survey(root: &Path, all: bool) -> anyhow::Result<Vec<SurveyRow>> {
    let g = graph::build(root)?;

    let mut keys: Vec<EntityKey> = g
        .attrs
        .keys()
        .copied()
        .filter(|&k| {
            if all {
                return true;
            }
            // Default: eligible, and not a promoted backlog item (its own exclusion).
            channels::eligible(&g, k) && !channels::promoted(&g, k)
        })
        .collect();

    // Importance order (D10), authored-priority slot empty → actionability → cons → id.
    keys.sort_by(|&a, &b| {
        let aa = actionability(&g, a);
        let ab = actionability(&g, b);
        // Actionable before Blocked.
        let act = act_rank(aa).cmp(&act_rank(ab));
        let cons = channels::consequence(&g, b).cmp(&channels::consequence(&g, a));
        act.then(cons).then_with(|| a.cmp(&b))
    });

    let rows = keys
        .into_iter()
        .map(|k| {
            let act = actionability(&g, k);
            let blockers = refs(&channels::blocked_by(&g, k));
            let mut reasons = vec![eligibility_reason(&g, k)];
            if !blockers.is_empty() {
                reasons.push(ReasonKind::BlockedBy {
                    items: blockers.clone(),
                });
            }
            reasons.push(ReasonKind::Consequence {
                inbound: channels::consequence(&g, k),
            });
            SurveyRow {
                id: k.canonical(),
                title: title_of(&g, k),
                kind: kind_of(&g, k),
                status: status_of(&g, k),
                act,
                consequence: channels::consequence(&g, k),
                blockers,
                reasons,
            }
        })
        .collect();
    Ok(rows)
}

/// Sort rank for [`Actionability`] — Actionable (0) before Blocked (1).
fn act_rank(a: Actionability) -> u8 {
    match a {
        Actionability::Actionable => 0,
        Actionability::Blocked => 1,
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

    // Order contributors: the dep-topology level (the count of transitive prereqs as
    // a proxy for depth) + the seq rank of any after edge constraining it. The
    // authoritative composed level is cordage-internal; the transitive-prereq count
    // is the agent-legible depth proxy (design §5.4 — contributors, not the raw key).
    let dep_level =
        u32::try_from(channels::blocked_by_transitive(&g, key).len()).unwrap_or(u32::MAX);
    let order_contrib = ReasonKind::OrderContrib {
        dep_level,
        seq_rank: None,
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
        order_contrib,
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
