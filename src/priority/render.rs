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

use crate::listing::{self, Column, ColumnPaint, RenderOpts, TITLE_EVEN, TITLE_ODD, status_hue};
use owo_colors::{AnsiColors::Cyan, DynColors};

use super::view::{ActionabilityBlock, BlockersView, Explanation, NextRow, ReasonKind, SurveyRow};

/// The priority policy version stamped into every `--json` envelope (D6 / REQ-094).
/// A consumer keys behaviour off this; bump it whenever the policy (partition,
/// channel synthesis, or order composition) changes its observable verdicts.
pub(crate) const PRIORITY_POLICY_VERSION: &str = "priority.v3";

// ---------------------------------------------------------------------------
// Column definitions for priority human tables (SL-079 PHASE-02)
// ---------------------------------------------------------------------------

const SURVEY_COLS: [Column<SurveyRow>; 7] = [
    Column {
        name: "id",
        header: "id",
        cell: |r| r.id.clone(),
        paint: ColumnPaint::Fixed(DynColors::Ansi(Cyan)),
    },
    Column {
        name: "kind",
        header: "kind",
        cell: |r| r.kind.clone(),
        paint: ColumnPaint::None,
    },
    Column {
        name: "status",
        header: "status",
        cell: |r| r.status.clone(),
        paint: ColumnPaint::ByValue(|r| status_hue(&r.status)),
    },
    Column {
        name: "act",
        header: "",
        cell: |r| r.act.badge().to_string(),
        paint: ColumnPaint::ByValue(|r| status_hue(r.act.token())),
    },
    Column {
        name: "score",
        header: "score",
        cell: |r| format!("{:.1}", r.score),
        paint: ColumnPaint::None,
    },
    Column {
        name: "blocker",
        header: "blocker",
        cell: |r| r.blockers.first().cloned().unwrap_or_default(),
        paint: ColumnPaint::None,
    },
    Column {
        name: "title",
        header: "title",
        cell: |r| r.title.clone(),
        paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]),
    },
];

#[expect(
    dead_code,
    reason = "declared for IMP-038 validation parity; not used by render_columns (priority has no --columns surface)"
)]
const SURVEY_DEFAULT: &[&str] = &["id", "kind", "status", "act", "score", "blocker", "title"];

const NEXT_COLS: [Column<NextRow>; 6] = [
    Column {
        name: "id",
        header: "id",
        cell: |r| r.id.clone(),
        paint: ColumnPaint::Fixed(DynColors::Ansi(Cyan)),
    },
    Column {
        name: "kind",
        header: "kind",
        cell: |r| r.kind.clone(),
        paint: ColumnPaint::None,
    },
    Column {
        name: "status",
        header: "status",
        cell: |r| r.status.clone(),
        paint: ColumnPaint::ByValue(|r| status_hue(&r.status)),
    },
    Column {
        name: "score",
        header: "score",
        cell: |r| format!("{:.1}", r.score),
        paint: ColumnPaint::None,
    },
    Column {
        name: "unblocks",
        header: "unblocks",
        cell: |r| r.blocking.len().to_string(),
        paint: ColumnPaint::None,
    },
    Column {
        name: "title",
        header: "title",
        cell: |r| r.title.clone(),
        paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]),
    },
];

#[expect(
    dead_code,
    reason = "declared for IMP-038 validation parity; not used by render_columns (priority has no --columns surface)"
)]
const NEXT_DEFAULT: &[&str] = &["id", "kind", "status", "score", "unblocks", "title"];

// ---------------------------------------------------------------------------
// Render functions
// ---------------------------------------------------------------------------

/// Render `survey` for human reading — one row per eligible node in importance order.
/// Columns: id, kind, status, BLOCKED badge (or blank), score, direct blocker.
/// Rides `listing::render_columns` (the shared list layout + colour seam). A blocked
/// row shows its badge + first direct blocker (the rest live in `blockers`/`explain` —
/// direct-only here, D11).
pub(crate) fn survey_human(rows: &[SurveyRow], opts: RenderOpts) -> String {
    if rows.is_empty() {
        return "(no eligible work)\n".to_string();
    }
    let sel: Vec<&Column<SurveyRow>> = SURVEY_COLS.iter().collect();
    listing::render_columns(rows, &sel, opts)
}

/// Render `next` for human reading — actionable-only, in the score-aware
/// induced-frontier order (SL-133 §5.4). Columns: id, kind, status, score,
/// blocking-count, title. Advisory.
pub(crate) fn next_human(rows: &[NextRow], opts: RenderOpts) -> String {
    if rows.is_empty() {
        return "(nothing actionable)\n".to_string();
    }
    let sel: Vec<&Column<NextRow>> = NEXT_COLS.iter().collect();
    listing::render_columns(rows, &sel, opts)
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
        ReasonKind::Score {
            base,
            value_dim,
            risk_dim,
            leverage,
            optionality,
            total,
        } => format!(
            "  score: {total:.1} (base {base:.1} [value {value_dim:.1}, risk {risk_dim:.1}], \
             leverage {leverage:.1}, optionality {optionality:.1})\n"
        ),
        ReasonKind::EvictedEdge { from, to, reason } => {
            format!("  evicted seq edge: {from} → {to} ({reason:?})\n")
        }
        ReasonKind::CycleDegraded { nodes } => {
            format!("  dep cycle (order degraded): {}\n", nodes.join(", "))
        }
    }
}

/// Render `explain` for human reading — every structured reason in a fixed section
/// order: eligibility, blocker chain, evicted edges, score.
pub(crate) fn explain_human(ex: &Explanation) -> String {
    let mut parts: Vec<String> = vec![format!("{} — explain\n", ex.id)];
    parts.push(reason_line(&ex.eligibility));
    for r in &ex.blocker_chain {
        parts.push(reason_line(r));
    }
    for r in &ex.evictions {
        parts.push(reason_line(r));
    }
    parts.push(reason_line(&ex.score));
    parts.concat()
}

/// Render the `inspect` actionability block for human reading — the trailing block
/// appended below the relation view (SL-046 D1). A leading blank line separates it
/// from the relation portion above.
pub(crate) fn actionability_block_human(block: &ActionabilityBlock) -> String {
    let mut parts: Vec<String> = vec!["\nactionability:\n".to_string()];
    parts.push(format!("  eligible: {}\n", block.eligible));
    parts.push(format!("  actionable: {}\n", block.actionable));
    parts.push(format!("  score: {:.1}\n", block.score));
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
        ReasonKind::Score {
            base,
            value_dim,
            risk_dim,
            leverage,
            optionality,
            total,
        } => serde_json::json!({
            "kind": "score",
            "base": base,
            "value_dim": value_dim,
            "risk_dim": risk_dim,
            "leverage": leverage,
            "optionality": optionality,
            "total": total,
        }),
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
/// score/blockers/reasons) under a policy-versioned envelope.
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
                "score": r.score,
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

/// `next --json` — actionable rows in the score-aware frontier order, full surface.
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
                "score": r.score,
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
        "score": reason_json(&ex.score),
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
        "score": block.score,
    })
}
