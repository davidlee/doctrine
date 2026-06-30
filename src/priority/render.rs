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

use crate::estimate::display::format_bound;
use crate::listing::{self, Column, ColumnPaint, RenderOpts, TITLE_EVEN, TITLE_ODD, status_hue};
use owo_colors::{
    AnsiColors::{Cyan, Red},
    DynColors,
};

use super::view::{ActionabilityBlock, BlockersView, Explanation, NextRow, ReasonKind, SurveyRow};

/// The priority policy version stamped into every `--json` envelope (D6 / REQ-094).
/// A consumer keys behaviour off this; bump it whenever the policy (partition,
/// channel synthesis, or order composition) changes its observable verdicts.
pub(crate) const PRIORITY_POLICY_VERSION: &str = "priority.v3";

// ---------------------------------------------------------------------------
// Column definitions for priority human tables (SL-079 PHASE-02)
// ---------------------------------------------------------------------------

const SURVEY_COLS: [Column<SurveyRow>; 6] = [
    Column {
        name: "id",
        header: "id",
        cell: |r| r.id.clone(),
        paint: ColumnPaint::ByValue(|r| {
            if matches!(r.act, super::view::Actionability::Blocked) {
                Some(DynColors::Ansi(Red))
            } else {
                Some(DynColors::Ansi(Cyan))
            }
        }),
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
const SURVEY_DEFAULT: &[&str] = &["id", "kind", "status", "score", "blocker", "title"];

const NEXT_COLS: [Column<NextRow>; 8] = [
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
        name: "estimate",
        header: "estimate",
        cell: |r| estimate_cell(r),
        paint: ColumnPaint::None,
    },
    Column {
        name: "value",
        header: "value",
        cell: |r| value_cell(r),
        paint: ColumnPaint::None,
    },
    Column {
        name: "tags",
        header: "tags",
        cell: |r| {
            if r.tags.is_empty() {
                listing::ABSENT_CELL.to_string()
            } else {
                r.tags.join(", ")
            }
        },
        paint: ColumnPaint::PerToken {
            split: |r| r.tags.clone(),
            render: listing::paint_tag,
        },
    },
    Column {
        name: "title",
        header: "title",
        cell: |r| r.title.clone(),
        paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]),
    },
];

const NEXT_DEFAULT: &[&str] = &["id", "status", "score", "estimate", "value", "title"];

// ---------------------------------------------------------------------------
// Facet cell formatters — pure fn(&NextRow) -> String (SL-171 PHASE-01, D4)
// ---------------------------------------------------------------------------

/// Render the estimate column cell: `{format_bound(lo)} - {format_bound(hi)}`,
/// each side right-padded to 4 chars (accommodates 1.0–99.9).
/// Unset estimates render [`listing::ABSENT_CELL`].
fn estimate_cell(r: &NextRow) -> String {
    match &r.estimate {
        Some(e) => format!(
            "{:>4} - {:>4}",
            format_bound(e.lower),
            format_bound(e.upper)
        ),
        None => listing::ABSENT_CELL.to_string(),
    }
}

/// Render the value column cell: `format_bound(v.value)`, or
/// [`listing::ABSENT_CELL`] when no value is authored.
fn value_cell(r: &NextRow) -> String {
    match &r.value {
        Some(v) => format_bound(v.value),
        None => listing::ABSENT_CELL.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Render functions
// ---------------------------------------------------------------------------

/// Render `survey` for human reading — one row per eligible node in importance order.
/// Columns: id, kind, status, score, direct blocker, title. Blocked rows render
/// their `id` cell in red (replacing the old BLOCKED badge column). Rides
/// `listing::render_columns` (the shared list layout + colour seam). Pagination via
/// `limit`/`offset` mirrors `next_human` (IMP-218).
pub(crate) fn survey_human(
    rows: &[SurveyRow],
    opts: RenderOpts,
    limit: usize,
    offset: usize,
) -> String {
    if rows.is_empty() {
        return "(no eligible work)\n".to_string();
    }
    let (visible, footer) = paginated(rows, limit, offset);
    let sel: Vec<&Column<SurveyRow>> = SURVEY_COLS.iter().collect();
    let mut out = listing::render_columns(visible, &sel, opts);
    if let Some(f) = footer {
        out.push_str(&f);
    }
    out
}

/// Slice rows into `(visible_page, optional_footer)` per limit/offset.
/// Footer is `None` when uncapped (`limit == 0`) or all rows fit. Single source
/// for the slice math + `limit == 0` guard — used by both `next_human` and
/// `survey_human` (IMP-218 DRY extraction).
fn paginated<T>(rows: &[T], limit: usize, offset: usize) -> (&[T], Option<String>) {
    let total = rows.len();
    let start = offset.min(total);
    let end = if limit == 0 {
        total
    } else {
        (start + limit).min(total)
    };
    let visible = rows.get(start..end).unwrap_or(&[]);
    let shown = visible.len();
    let footer = if limit != 0 && shown < total {
        Some(listing::format_truncation_notice(
            shown, total, offset, limit,
        ))
    } else {
        None
    };
    (visible, footer)
}

/// Render `next` for human reading — actionable-only, in the score-aware
/// induced-frontier order (SL-133 §5.4). Columns: projected via
/// `select_columns` + `default_with_tags`, with facet cells rendered compact
/// and unitless (SL-171 PHASE-01, D4).
///
/// `limit`/`offset` paginate the visible slice AFTER the sort order (SL-171 PHASE-02).
/// `limit == 0` is uncapped — all rows from `offset` onward, no footer.
/// `--json` path does not reach here (the caller bypasses pagination).
pub(crate) fn next_human(
    rows: &[NextRow],
    opts: RenderOpts,
    columns: Option<&[String]>,
    limit: usize,
    offset: usize,
) -> anyhow::Result<String> {
    if rows.is_empty() {
        return Ok("(nothing actionable)\n".to_string());
    }
    let (visible, footer) = paginated(rows, limit, offset);

    // D7 (SL-171 PHASE-02): any_tagged computed over the VISIBLE (post-slice) page.
    let any_tagged = visible.iter().any(|r| !r.tags.is_empty());
    let effective = listing::default_with_tags(NEXT_DEFAULT, any_tagged);
    let sel = listing::select_columns(&NEXT_COLS, &effective, columns)?;

    let mut out = listing::render_columns(visible, &sel, opts);
    if let Some(f) = footer {
        out.push_str(&f);
    }
    Ok(out)
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

// ---------------------------------------------------------------------------
// Tests — SL-171 PHASE-01 verification (VT-1 through VT-5)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::estimate::EstimateFacet;
    use crate::listing::ABSENT_CELL;
    use crate::priority::view::Actionability;
    use crate::value::ValueFacet;

    /// Build a bare NextRow with no facets (estimate/value/tags absent).
    fn bare_row(id: &str) -> NextRow {
        NextRow {
            id: id.to_string(),
            title: "Title".to_string(),
            kind: "ISS".to_string(),
            status: "open".to_string(),
            act: Actionability::Actionable,
            score: 0.0,
            reasons: vec![],
            blockers: vec![],
            blocking: vec![],
            estimate: None,
            value: None,
            tags: vec![],
        }
    }

    /// Build a NextRow with facets.
    fn faceted_row(id: &str, lo: f64, hi: f64, val: f64, tags: &[&str]) -> NextRow {
        NextRow {
            id: id.to_string(),
            title: "Title".to_string(),
            kind: "ISS".to_string(),
            status: "open".to_string(),
            act: Actionability::Actionable,
            score: val / 6.5,
            reasons: vec![],
            blockers: vec![],
            blocking: vec![],
            estimate: Some(EstimateFacet {
                lower: lo,
                upper: hi,
            }),
            value: Some(ValueFacet { value: val }),
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
        }
    }

    /// Helper — render next_human and return the header line (first line of output).
    fn header(out: &str) -> &str {
        out.lines().next().unwrap_or("")
    }

    // ── VT-1: --columns projection ──────────────────────────────────────

    #[test]
    fn vt1_columns_id_score_emits_exact_headers() {
        let rows = vec![bare_row("ISS-001")];
        let out = next_human(
            &rows,
            RenderOpts::default(),
            Some(&["id".to_string(), "score".to_string()]),
            20,
            0,
        )
        .unwrap();
        assert!(header(&out).contains("id"), "header has id: {out}");
        assert!(header(&out).contains("score"), "header has score: {out}");
        assert!(!header(&out).contains("kind"), "header lacks kind: {out}");
    }

    #[test]
    fn vt1_columns_bogus_errors_with_available_set() {
        let rows = vec![bare_row("ISS-001")];
        let err = next_human(
            &rows,
            RenderOpts::default(),
            Some(&["bogus".to_string()]),
            20,
            0,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("unknown column `bogus`"), "got: {err}");
        assert!(err.contains("available:"), "got: {err}");
    }

    // ── VT-2: default headers, kind/unblocks absent ──────────────────────

    #[test]
    fn vt2_default_headers_no_kind_no_unblocks() {
        let rows = vec![bare_row("ISS-001")];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        let h = header(&out);
        assert!(h.contains("id"), "header has id: {h}");
        assert!(h.contains("status"), "header has status: {h}");
        assert!(h.contains("score"), "header has score: {h}");
        assert!(h.contains("estimate"), "header has estimate: {h}");
        assert!(h.contains("value"), "header has value: {h}");
        assert!(h.contains("title"), "header has title: {h}");
        assert!(!h.contains("kind"), "kind absent from default: {h}");
        assert!(!h.contains("unblocks"), "unblocks absent from default: {h}");
    }

    #[test]
    fn vt2_columns_unblocks_errors_no_such_column() {
        let rows = vec![bare_row("ISS-001")];
        let err = next_human(
            &rows,
            RenderOpts::default(),
            Some(&["unblocks".to_string()]),
            20,
            0,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("unknown column `unblocks`"), "got: {err}");
    }

    // ── VT-3: tags conditional via default_with_tags ────────────────────

    #[test]
    fn vt3_tags_column_appears_when_any_row_tagged() {
        let rows = vec![
            bare_row("ISS-001"),
            faceted_row("ISS-002", 0.0, 10.0, 5.0, &["cli:command"]),
        ];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        assert!(
            header(&out).contains("tags"),
            "tags column appears when any row tagged: {out}"
        );
    }

    #[test]
    fn vt3_tags_column_hidden_when_none_tagged() {
        let rows = vec![bare_row("ISS-001"), bare_row("ISS-002")];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        assert!(
            !header(&out).contains("tags"),
            "tags column hidden when none tagged: {out}"
        );
    }

    #[test]
    fn vt3_columns_tags_forces_column_even_all_empty() {
        let rows = vec![bare_row("ISS-001")];
        let out = next_human(
            &rows,
            RenderOpts::default(),
            Some(&["id".to_string(), "tags".to_string()]),
            20,
            0,
        )
        .unwrap();
        assert!(
            header(&out).contains("tags"),
            "--columns tags forces column: {out}"
        );
    }

    // ── VT-4: format_bound cells ─────────────────────────────────────────

    #[test]
    fn vt4_format_bound_estimate_fractional() {
        let rows = vec![faceted_row("ISS-001", 3.2, 4.8, 5.0, &[])];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        assert!(out.contains(" 3.2 -  4.8"), "fractional estimate: {out}");
    }

    #[test]
    fn vt4_format_bound_estimate_integral() {
        let rows = vec![faceted_row("ISS-001", 3.0, 8.0, 5.0, &[])];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        assert!(
            out.contains(" 3.0 -  8.0"),
            "integral estimate shows .0: {out}"
        );
    }

    #[test]
    fn vt4_format_bound_value_integral() {
        let rows = vec![faceted_row("ISS-001", 3.0, 8.0, 5.0, &[])];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        // value 5.0 → "5.0" via format_bound (always one decimal)
        assert!(
            out.contains(" 5.0 ") || out.contains("│5.0│"),
            "integral value 5.0: {out}"
        );
    }

    #[test]
    fn vt4_format_bound_value_fractional() {
        let rows = vec![faceted_row("ISS-001", 3.0, 8.0, 5.5, &[])];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        assert!(out.contains("5.5"), "fractional value 5.5: {out}");
    }

    #[test]
    fn vt4_absent_cell_for_bare_row() {
        let rows = vec![bare_row("ISS-001")];
        let out = next_human(&rows, RenderOpts::default(), None, 20, 0).unwrap();
        assert!(out.contains(ABSENT_CELL), "bare row has ABSENT_CELL: {out}");
    }

    // ── PHASE-02 pagination (next_human limit/offset slice + footer) ─────
    // The CLI page→offset resolution + --page validation are covered at the
    // black-box level in tests/e2e_priority_golden.rs; these pin the pure
    // next_human slice + footer-guard + D7 visible-slice gate (SL-171 PHASE-02).

    /// Five bare rows for pagination slicing.
    fn five_rows() -> Vec<NextRow> {
        (1..=5).map(|n| bare_row(&format!("ISS-00{n}"))).collect()
    }

    // ── VT-1: footer wording + offset→page math ─────────────────────────

    #[test]
    fn vt_pagination_limit_shows_footer() {
        let rows = five_rows();
        let out = next_human(&rows, RenderOpts::default(), None, 2, 0).unwrap();
        assert!(out.contains("ISS-001"), "page 1 row 1: {out}");
        assert!(out.contains("ISS-002"), "page 1 row 2: {out}");
        assert!(!out.contains("ISS-003"), "row 3 clipped: {out}");
        // footer: shown=2 of total=5; offset 0 / page_size 2 → next page 2.
        assert!(out.contains("2 of 5"), "footer count: {out}");
        assert!(out.contains("--page 2"), "footer next-page: {out}");
    }

    #[test]
    fn vt_pagination_offset_slices_and_advances_page() {
        let rows = five_rows();
        let out = next_human(&rows, RenderOpts::default(), None, 2, 2).unwrap();
        assert!(out.contains("ISS-003"), "offset page row 3: {out}");
        assert!(out.contains("ISS-004"), "offset page row 4: {out}");
        assert!(!out.contains("ISS-001"), "row 1 skipped: {out}");
        // offset 2 / page_size 2 → next page (2/2)+2 = 3.
        assert!(out.contains("--page 3"), "footer advances page: {out}");
    }

    // ── VT-2: --limit 0 (uncapped) — all rows, no footer, no panic ───────

    #[test]
    fn vt_pagination_limit_zero_uncapped_no_footer() {
        let rows = five_rows();
        let out = next_human(&rows, RenderOpts::default(), None, 0, 0).unwrap();
        for n in 1..=5 {
            assert!(out.contains(&format!("ISS-00{n}")), "row {n} shown: {out}");
        }
        assert!(!out.contains(" of 5"), "no footer when uncapped: {out}");
    }

    #[test]
    fn vt_pagination_limit_zero_with_offset_no_panic_no_footer() {
        // F1 guard: limit==0 with offset>0 must not divide by zero in the footer.
        let rows = five_rows();
        let out = next_human(&rows, RenderOpts::default(), None, 0, 2).unwrap();
        assert!(!out.contains("ISS-001"), "offset honoured: {out}");
        assert!(out.contains("ISS-003"), "rows[2..] shown: {out}");
        assert!(
            !out.contains(" of 5"),
            "no footer with --limit 0 --offset N: {out}"
        );
    }

    // ── VT-4: offset beyond total → empty body + offset-branch footer ────

    #[test]
    fn vt_pagination_offset_exceeds_total() {
        let rows = five_rows();
        let out = next_human(&rows, RenderOpts::default(), None, 2, 10).unwrap();
        assert!(
            out.contains("no results at this offset"),
            "offset-branch footer: {out}"
        );
        assert!(out.contains("0 of 5"), "shown=0 of total: {out}");
    }

    // ── VT-5: D7 — any_tagged computed over the VISIBLE (post-slice) page ─

    #[test]
    fn vt_d7_tags_gate_is_per_visible_page() {
        // Tagged row lands only on page 2 (offset 2). Page 1 must show NO tags
        // column; page 2 must show it.
        let rows = vec![
            bare_row("ISS-001"),
            bare_row("ISS-002"),
            faceted_row("ISS-003", 0.0, 1.0, 1.0, &["cli:command"]),
        ];
        let page1 = next_human(&rows, RenderOpts::default(), None, 2, 0).unwrap();
        assert!(
            !page1.lines().next().unwrap_or("").contains("tags"),
            "page 1 (no tagged row) hides tags column: {page1}"
        );
        let page2 = next_human(&rows, RenderOpts::default(), None, 2, 2).unwrap();
        assert!(
            page2.lines().next().unwrap_or("").contains("tags"),
            "page 2 (tagged row) shows tags column: {page2}"
        );
    }

    #[test]
    fn vt_d7_columns_tags_overrides_visible_gate() {
        // Explicit --columns tags forces the column even on a page with no tagged row.
        let rows = vec![bare_row("ISS-001"), bare_row("ISS-002")];
        let out = next_human(
            &rows,
            RenderOpts::default(),
            Some(&["id".to_string(), "tags".to_string()]),
            2,
            0,
        )
        .unwrap();
        assert!(
            out.lines().next().unwrap_or("").contains("tags"),
            "explicit --columns tags overrides the visible-page gate: {out}"
        );
    }
}
