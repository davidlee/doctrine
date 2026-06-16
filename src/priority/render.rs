// SPDX-License-Identifier: GPL-3.0-only
//! The priority RENDER layer (SL-047 §5.4) — human table + `--json`, produced FROM
//! the [`super::view`] structured reasons (REQ-072 AC3), never recomputed.
//!
//! Rides `crate::listing` (`Format`, the SL-045/SL-046 read-surface `--json`
//! precedent) and mirrors `relation_graph::render_human`/`render_json`: house style
//! is `Vec<String>` parts each carrying their own newline, joined by `concat` (avoids
//! the `push_str(&format!)` lint); `--json` is built manually with `serde_json::json!`
//! and stamps the [`PRIORITY_POLICY_VERSION`] (D6 / REQ-094). NO trailing newline on
//! either surface — the black-box golden contract (`write!`, not `writeln!`).

use crate::listing::RenderOpts;

use super::view::{ActionabilityBlock, BlockersView, Explanation, NextRow, ReasonKind, SurveyRow};

/// The priority policy version stamped into every `--json` envelope (D6 / REQ-094).
/// A consumer keys behaviour off this; bump it whenever the policy (partition,
/// channel synthesis, or order composition) changes its observable verdicts.
pub(crate) const PRIORITY_POLICY_VERSION: &str = "priority.v2";

/// Render `survey` for human reading — one row per eligible node in importance order.
/// Columns: id, kind, status, BLOCKED badge (or blank), consequence, direct blocker.
/// Rides `listing::render_table` (the shared list layout). A blocked row shows its
/// badge + first direct blocker (the rest live in `blockers`/`explain` — direct-only
/// here, D11).
pub(crate) fn survey_human(rows: &[SurveyRow], opts: RenderOpts) -> String {
    if rows.is_empty() {
        return "(no eligible work)\n".to_string();
    }
    let mut grid: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);
    grid.push(
        ["id", "kind", "status", "", "cons", "blocker", "title"]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    );
    for r in rows {
        grid.push(vec![
            r.id.clone(),
            r.kind.clone(),
            r.status.clone(),
            r.act.badge().to_string(),
            r.consequence.to_string(),
            r.blockers.first().cloned().unwrap_or_default(),
            r.title.clone(),
        ]);
    }
    crate::listing::render_table(&grid, opts.term_width)
}

/// Render `next` for human reading — actionable-only, in `order_key` order. Columns:
/// id, kind, status, blocking-count, title. Advisory.
pub(crate) fn next_human(rows: &[NextRow], opts: RenderOpts) -> String {
    if rows.is_empty() {
        return "(nothing actionable)\n".to_string();
    }
    let mut grid: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);
    grid.push(
        ["id", "kind", "status", "unblocks", "title"]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    );
    for r in rows {
        grid.push(vec![
            r.id.clone(),
            r.kind.clone(),
            r.status.clone(),
            r.blocking.len().to_string(),
            r.title.clone(),
        ]);
    }
    crate::listing::render_table(&grid, opts.term_width)
}

/// Render `blockers` for human reading — the blocked-by and blocking lists (direct or
/// transitive). Each section omitted when empty; an all-empty result renders a clean
/// note. The `transitive` flag annotates the header (display depth, never reorders).
pub(crate) fn blockers_human(view: &BlockersView) -> String {
    let depth = if view.transitive {
        "transitive"
    } else {
        "direct"
    };
    let mut parts: Vec<String> = vec![format!("{} — blockers ({depth})\n", view.id)];
    if !view.blocked_by.is_empty() {
        parts.push("\nblocked by:\n".to_string());
        for b in &view.blocked_by {
            parts.push(format!("  {b}\n"));
        }
    }
    if !view.blocking.is_empty() {
        parts.push("\nblocking:\n".to_string());
        for b in &view.blocking {
            parts.push(format!("  {b}\n"));
        }
    }
    if view.blocked_by.is_empty() && view.blocking.is_empty() {
        parts.push("\n(no blockers, blocks nothing)\n".to_string());
    }
    parts.concat()
}

/// Render one structured reason as a human line (the render source of truth — every
/// human reason line comes from here). Used by `explain`.
fn reason_line(reason: &ReasonKind) -> String {
    match reason {
        ReasonKind::Eligibility { status, class } => {
            let s = status.as_deref().unwrap_or("—");
            format!("  eligibility: {s} → {class:?}\n")
        }
        ReasonKind::BlockedBy { items } => format!("  blocked by: {}\n", items.join(", ")),
        ReasonKind::Blocking { items } => format!("  blocking: {}\n", items.join(", ")),
        ReasonKind::Consequence { inbound } => format!("  consequence: {inbound}\n"),
        ReasonKind::EvictedEdge { from, to, reason } => {
            format!("  evicted seq edge: {from} → {to} ({reason:?})\n")
        }
        ReasonKind::CycleDegraded { nodes } => {
            format!("  dep cycle (order degraded): {}\n", nodes.join(", "))
        }
    }
}

/// Render `explain` for human reading — every structured reason in a fixed section
/// order: eligibility, blocker chain, evicted edges, consequence.
pub(crate) fn explain_human(ex: &Explanation) -> String {
    let mut parts: Vec<String> = vec![format!("{} — explain\n", ex.id)];
    parts.push(reason_line(&ex.eligibility));
    for r in &ex.blocker_chain {
        parts.push(reason_line(r));
    }
    for r in &ex.evictions {
        parts.push(reason_line(r));
    }
    parts.push(reason_line(&ex.consequence));
    parts.concat()
}

/// Render the `inspect` actionability block for human reading — the trailing block
/// appended below the relation view (SL-046 D1). A leading blank line separates it
/// from the relation portion above.
pub(crate) fn actionability_block_human(block: &ActionabilityBlock) -> String {
    let mut parts: Vec<String> = vec!["\nactionability:\n".to_string()];
    parts.push(format!("  eligible: {}\n", block.eligible));
    parts.push(format!("  actionable: {}\n", block.actionable));
    parts.push(format!("  consequence: {}\n", block.consequence));
    if !block.blockers.is_empty() {
        parts.push(format!("  blocked by: {}\n", block.blockers.join(", ")));
    }
    if !block.blocking.is_empty() {
        parts.push(format!("  blocking: {}\n", block.blocking.join(", ")));
    }
    parts.concat()
}

// ---------------------------------------------------------------------------
// --json — built manually, stamps PRIORITY_POLICY_VERSION (D6 / REQ-094).
// ---------------------------------------------------------------------------

/// One structured reason as JSON (the faithful `ReasonKind` shape — a `kind`
/// discriminant + its payload).
fn reason_json(reason: &ReasonKind) -> serde_json::Value {
    match reason {
        ReasonKind::Eligibility { status, class } => serde_json::json!({
            "kind": "eligibility",
            "status": status,
            "class": format!("{class:?}"),
        }),
        ReasonKind::BlockedBy { items } => {
            serde_json::json!({ "kind": "blocked_by", "items": items })
        }
        ReasonKind::Blocking { items } => {
            serde_json::json!({ "kind": "blocking", "items": items })
        }
        ReasonKind::Consequence { inbound } => {
            serde_json::json!({ "kind": "consequence", "inbound": inbound })
        }
        ReasonKind::EvictedEdge { from, to, reason } => serde_json::json!({
            "kind": "evicted_edge",
            "from": from,
            "to": to,
            "reason": format!("{reason:?}"),
        }),
        ReasonKind::CycleDegraded { nodes } => {
            serde_json::json!({ "kind": "cycle_degraded", "nodes": nodes })
        }
    }
}

/// Stamp the policy version onto a value's envelope and serialize (pretty, no
/// trailing newline — the golden contract).
fn finish(value: &serde_json::Value) -> anyhow::Result<String> {
    serde_json::to_string_pretty(value)
        .map_err(|e| anyhow::anyhow!("failed to serialize priority JSON: {e}"))
}

/// `survey --json` — every row's full surface (id/title/kind/status/actionability/
/// consequence/blockers/reasons) under a policy-versioned envelope.
pub(crate) fn survey_json(rows: &[SurveyRow]) -> anyhow::Result<String> {
    let rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "title": r.title,
                "kind": r.kind,
                "status": r.status,
                "actionability": r.act.token(),
                "consequence": r.consequence,
                "blockers": r.blockers,
                "reasons": r.reasons.iter().map(reason_json).collect::<Vec<_>>(),
            })
        })
        .collect();
    finish(&serde_json::json!({
        "kind": "survey",
        "policy_version": PRIORITY_POLICY_VERSION,
        "rows": rows,
    }))
}

/// `next --json` — actionable rows in `order_key` order, full surface.
pub(crate) fn next_json(rows: &[NextRow]) -> anyhow::Result<String> {
    let rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "title": r.title,
                "kind": r.kind,
                "status": r.status,
                "actionability": r.act.token(),
                "blocking": r.blocking,
                "reasons": r.reasons.iter().map(reason_json).collect::<Vec<_>>(),
            })
        })
        .collect();
    finish(&serde_json::json!({
        "kind": "next",
        "policy_version": PRIORITY_POLICY_VERSION,
        "rows": rows,
    }))
}

/// `blockers --json` — the blocked-by + blocking lists, with the display-depth flag.
pub(crate) fn blockers_json(view: &BlockersView) -> anyhow::Result<String> {
    finish(&serde_json::json!({
        "kind": "blockers",
        "policy_version": PRIORITY_POLICY_VERSION,
        "id": view.id,
        "transitive": view.transitive,
        "blocked_by": view.blocked_by,
        "blocking": view.blocking,
    }))
}

/// `explain --json` — every structured reason faithfully serialized.
pub(crate) fn explain_json(ex: &Explanation) -> anyhow::Result<String> {
    finish(&serde_json::json!({
        "kind": "explain",
        "policy_version": PRIORITY_POLICY_VERSION,
        "id": ex.id,
        "eligibility": reason_json(&ex.eligibility),
        "blocker_chain": ex.blocker_chain.iter().map(reason_json).collect::<Vec<_>>(),
        "evictions": ex.evictions.iter().map(reason_json).collect::<Vec<_>>(),
        "consequence": reason_json(&ex.consequence),
    }))
}

/// The actionability block as a JSON value (NOT a standalone envelope) — embedded
/// under the `inspect --json` relation view at the command layer (SL-046 D1).
pub(crate) fn actionability_block_value(block: &ActionabilityBlock) -> serde_json::Value {
    serde_json::json!({
        "eligible": block.eligible,
        "actionable": block.actionable,
        "blockers": block.blockers,
        "blocking": block.blocking,
        "consequence": block.consequence,
    })
}
