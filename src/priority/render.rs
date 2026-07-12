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

use super::graph::DEFAULT_VALUE;
use crate::estimate::display::format_bound;
use crate::listing::{self, Column, ColumnPaint, RenderOpts, TITLE_EVEN, TITLE_ODD, status_hue};
use owo_colors::{
    AnsiColors::{Cyan, Red},
    DynColors,
};

use super::findings::Finding;
use super::view::{
    ActionabilityBlock, BlockersView, EdgeVerb, Explanation, NextRow, NextView, ReasonKind,
    SurveyRow, TensionCauseView, TensionGradeView,
};

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

/// Marker suffix on a value cell showing the effective *default* value (a
/// value-bearing kind that authored no `[value]`), distinguishing it from an
/// authored value of the same magnitude (IMP-211).
const DEFAULT_VALUE_MARKER: &str = "*";

/// Max tension callouts rendered under a `next` page (SL-218 PHASE-03, design §3
/// / VT-6). A HUMAN-render bound only — `next --json` carries the full list
/// uncapped.
pub(crate) const TENSION_MAX_CALLOUTS: usize = 3;

/// Render the value column cell. An authored value renders bare
/// (`format_bound(v.value)`). With no authored value the cell must still reflect
/// what the score used (`graph::effective_raw_value`): a value-bearing kind is
/// scored as [`DEFAULT_VALUE`], so render that default with [`DEFAULT_VALUE_MARKER`]
/// — never [`listing::ABSENT_CELL`], which would contradict the ranking (IMP-211).
/// A genuinely valueless kind (records/governance/REV) has no value in the score
/// either, so it stays `ABSENT_CELL`.
fn value_cell(r: &NextRow) -> String {
    match &r.value {
        Some(v) => format_bound(v.value),
        None if crate::kinds::is_value_bearing(&r.kind) => {
            format!("{}{DEFAULT_VALUE_MARKER}", format_bound(DEFAULT_VALUE))
        }
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
    view: &NextView,
    opts: RenderOpts,
    columns: Option<&[String]>,
    limit: usize,
    offset: usize,
    verbose: bool,
) -> anyhow::Result<String> {
    if view.rows.is_empty() {
        return Ok("(nothing actionable)\n".to_string());
    }
    let (visible, footer) = paginated(&view.rows, limit, offset);

    // D7 (SL-171 PHASE-02): any_tagged computed over the VISIBLE (post-slice) page.
    let any_tagged = visible.iter().any(|r| !r.tags.is_empty());
    let effective = listing::default_with_tags(NEXT_DEFAULT, any_tagged);
    let sel = listing::select_columns(&NEXT_COLS, &effective, columns)?;

    let mut out = listing::render_columns(visible, &sel, opts);
    if let Some(f) = footer {
        out.push_str(&f);
    }
    out.push_str(&next_tension_block(view, visible, verbose));
    Ok(out)
}

/// The `next` tension callout block (design §3): structure callouts under the
/// visible page by default, composition added with `--verbose`, capped at
/// [`TENSION_MAX_CALLOUTS`]; then the F-6 m=0 scoped disclosure. Callouts attach
/// to their SURFACED row, so only tensions whose surfaced member is on the
/// visible page appear. Empty string when the page has nothing to say.
fn next_tension_block(view: &NextView, visible: &[NextRow], verbose: bool) -> String {
    let on_page: std::collections::BTreeSet<&str> = visible.iter().map(|r| r.id.as_str()).collect();
    let callouts: Vec<String> = view
        .tensions
        .iter()
        .filter(|t| tension_surfaced(t).is_some_and(|s| on_page.contains(s)))
        .filter(|t| verbose || tension_is_structure(t))
        .take(TENSION_MAX_CALLOUTS)
        .filter_map(tension_fragment)
        .collect();
    if callouts.is_empty() && view.zero_weight.is_none() {
        return String::new();
    }
    // House style: `Vec<String>` parts each carrying their own newline, joined by
    // `concat` (avoids the `push_str(&format!)` lint).
    let mut parts: Vec<String> = vec!["\ntensions:\n".to_string()];
    parts.extend(callouts.iter().map(|c| format!("  {c}\n")));
    if let Some(z) = &view.zero_weight {
        parts.push(reason_line(z));
    }
    parts.concat()
}

/// The surfaced-member id of a [`ReasonKind::Tension`] (the row a callout attaches
/// to); `None` for any other reason.
fn tension_surfaced(reason: &ReasonKind) -> Option<&str> {
    match reason {
        ReasonKind::Tension { surfaced, .. } => Some(surfaced),
        _ => None,
    }
}

/// Whether a tension is a Structure cause (`next`'s default class; Composition is
/// verbose-only, design D5).
fn tension_is_structure(reason: &ReasonKind) -> bool {
    matches!(
        reason,
        ReasonKind::Tension {
            cause: TensionCauseView::Structure { .. },
            ..
        }
    )
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
        ReasonKind::EvictedEdge { .. } | ReasonKind::CycleDegraded { .. } => {
            format!("  {}\n", provenance_fragment(reason).unwrap_or_default())
        }
        ReasonKind::ValueAuthored { .. }
        | ReasonKind::ValueProjected { .. }
        | ReasonKind::ValueGauge { .. } => {
            format!("  {}\n", value_source_fragment(reason).unwrap_or_default())
        }
        ReasonKind::PriorityDomainDisclosure { count } => format!(
            "  {count} prefer-first judgements recorded — not value-bearing; no consumer yet\n"
        ),
        ReasonKind::AgentEvidenceDemoted => format!("  {AGENT_DEMOTION_DISCLOSURE}\n"),
        ReasonKind::Tension { .. } => {
            format!("  {}\n", tension_fragment(reason).unwrap_or_default())
        }
        ReasonKind::ZeroWeightExcluded { count } => {
            format!("  {}\n", zero_weight_fragment(*count))
        }
    }
}

/// The SL-218 PHASE-03 tension line (design §3) — the SINGLE wording source
/// shared by `reason_line` (human `explain`), the `next` callout block, and the
/// `--json` surfaces map their structure off the SAME [`ReasonKind::Tension`].
/// Bare text (no indent / newline — the caller frames it); `None` for any
/// non-tension reason. Structure and Composition have distinct shapes (design's
/// four samples); the grade clause and counts are shared.
pub(crate) fn tension_fragment(reason: &ReasonKind) -> Option<String> {
    let ReasonKind::Tension {
        preferred,
        surfaced,
        cause,
        grade,
    } = reason
    else {
        return None;
    };
    let g = tension_grade_clause(grade);
    Some(match cause {
        TensionCauseView::Structure { edge_from, verb } => {
            let (kw, tail) = match verb {
                EdgeVerb::After => ("after", "sequence survives"),
                EdgeVerb::Needs => ("needs", "holds"),
            };
            format!(
                "tension: {preferred} ranks above {surfaced} on value_dim ({g}); \
                 {surfaced} surfaces first — `{kw} {edge_from}` {tail}."
            )
        }
        TensionCauseView::Composition {
            risk_dim,
            leverage,
            optionality,
        } => {
            let deltas = tension_deltas_clause(*leverage, *risk_dim, *optionality);
            format!(
                "{surfaced} surfaces above {preferred} on full score ({deltas}); \
                 on value_dim alone {preferred} ranks higher ({g})."
            )
        }
    })
}

/// The grade clause inside a tension callout's parentheses (design §3 samples).
fn tension_grade_clause(grade: &TensionGradeView) -> String {
    match grade {
        TensionGradeView::Determined { human, agent } => {
            format!("determined — {}", tension_counts_clause(*human, *agent))
        }
        TensionGradeView::AgentProposed { agent } => {
            format!("agent-proposed — {agent} agent judgements, unconfirmed")
        }
        TensionGradeView::Projected => "projected order — no determining evidence".to_string(),
    }
}

/// The rater-count clause (T7 disclosure): mixed rows read `H human + A agent`;
/// a single class reads `N human judgements` / `N agent judgements`.
fn tension_counts_clause(human: u32, agent: u32) -> String {
    match (human, agent) {
        (0, 0) => "no constraining judgements".to_string(),
        (h, 0) => format!("{h} human judgements"),
        (0, a) => format!("{a} agent judgements"),
        (h, a) => format!("{h} human + {a} agent"),
    }
}

/// The Composition component-delta clause (surfaced − preferred): nonzero
/// dimensions only, in `leverage, risk, optionality` order, sign-prefixed.
fn tension_deltas_clause(leverage: f64, risk_dim: f64, optionality: f64) -> String {
    [
        ("leverage", leverage),
        ("risk", risk_dim),
        ("optionality", optionality),
    ]
    .into_iter()
    .filter(|&(_, v)| v.total_cmp(&0.0) != std::cmp::Ordering::Equal)
    .map(|(label, v)| format!("{label} {v:+.1}"))
    .collect::<Vec<_>>()
    .join(", ")
}

/// The m=0 scoped-disclosure line (design §2 / F-6, SL-217 D6 wording).
fn zero_weight_fragment(count: usize) -> String {
    let noun = if count == 1 { "pair" } else { "pairs" };
    format!("{count} {noun} value-insensitive, zero weight")
}

/// The design-schema JSON for a [`ReasonKind::Tension`] (design §3:
/// `{preferred, surfaced, cause, edge?, deltas?, grade, counts?}`) — the SINGLE
/// structure source shared by `next --json`'s `tensions` array and `explain
/// --json`. `None` for any non-tension reason.
pub(crate) fn tension_json(reason: &ReasonKind) -> Option<serde_json::Value> {
    let ReasonKind::Tension {
        preferred,
        surfaced,
        cause,
        grade,
    } = reason
    else {
        return None;
    };
    let mut obj = serde_json::json!({
        "preferred": preferred,
        "surfaced": surfaced,
    });
    let map = obj.as_object_mut()?;
    match cause {
        TensionCauseView::Structure { edge_from, verb } => {
            map.insert("cause".into(), serde_json::json!("structure"));
            let verb = match verb {
                EdgeVerb::After => "after",
                EdgeVerb::Needs => "needs",
            };
            map.insert(
                "edge".into(),
                serde_json::json!({ "from": edge_from, "verb": verb }),
            );
        }
        TensionCauseView::Composition {
            risk_dim,
            leverage,
            optionality,
        } => {
            map.insert("cause".into(), serde_json::json!("composition"));
            map.insert(
                "deltas".into(),
                serde_json::json!({
                    "risk_dim": risk_dim,
                    "leverage": leverage,
                    "optionality": optionality,
                }),
            );
        }
    }
    let (grade_tok, counts) = match grade {
        TensionGradeView::Determined { human, agent } => ("determined", Some((*human, *agent))),
        TensionGradeView::AgentProposed { agent } => ("agent_proposed", Some((0, *agent))),
        TensionGradeView::Projected => ("projected", None),
    };
    map.insert("grade".into(), serde_json::json!(grade_tok));
    if let Some((human, agent)) = counts {
        map.insert(
            "counts".into(),
            serde_json::json!({ "human": human, "agent": agent }),
        );
    }
    Some(obj)
}

/// SL-218 D2 disclosure — the SINGLE wording source for the knob-on line on
/// `compare elicit` and `explain` (design §1: shared fragment, never
/// per-surface prose).
pub(crate) const AGENT_DEMOTION_DISCLOSURE: &str =
    "agent evidence demoted: agent judgements propose orderings but do not retire questions";

/// The SL-213 PHASE-06 value-source fragment (design §4 S3, the three shapes'
/// literal templates) — the SINGLE source shared by `reason_line` (human), the
/// SL-217 elicit human render (participant value line), and the `--json`
/// surfaces. Bare text (no indent, no newline — the caller frames it); `None`
/// for any non-value-source reason.
pub(crate) fn value_source_fragment(reason: &ReasonKind) -> Option<String> {
    match reason {
        ReasonKind::ValueAuthored { value, conflict } => {
            let suffix = if conflict.is_empty() {
                String::new()
            } else {
                format!(" (see anchor-conflict finding: {})", conflict.join(", "))
            };
            Some(format!("value {value:.1} — authored{suffix}"))
        }
        ReasonKind::ValueProjected {
            value,
            lower,
            upper,
            human,
            agent,
        } => Some(format!(
            "value {value:.1} — projected · bounds ({} ‥ {}) · from {} constraining judgements \
             ({human} human, {agent} agent)",
            bound_fragment(*lower),
            bound_fragment(*upper),
            human + agent,
        )),
        ReasonKind::ValueGauge { value, judgements } => Some(format!(
            "value {value:.1} — gauge · ordered by {judgements} judgements, no anchor in \
             component · set a value on any member to calibrate"
        )),
        _ => None,
    }
}

/// One C6 display bound as text — `"unbounded"` for `None`.
fn bound_fragment(bound: Option<f64>) -> String {
    match bound {
        Some(v) => format!("{v:.1}"),
        None => "unbounded".to_string(),
    }
}

/// The human fragment for the two PROVENANCE reason variants — an evicted soft (`after`)
/// edge, or a degraded dep cycle. The SINGLE source shared by `explain`'s [`reason_line`]
/// and the `findings` `Provenance` render (SL-194 R2: reuse the *fragment*, NOT
/// `explain()` — its foreign-node path is the ISS-003 hazard). Returns the bare text (no
/// indent, no newline — the caller frames it); `None` for any non-provenance reason.
fn provenance_fragment(reason: &ReasonKind) -> Option<String> {
    match reason {
        ReasonKind::EvictedEdge { from, to, reason } => {
            Some(format!("evicted seq edge: {from} → {to} ({reason:?})"))
        }
        ReasonKind::CycleDegraded { nodes } => {
            Some(format!("dep cycle (order degraded): {}", nodes.join(", ")))
        }
        _ => None,
    }
}

/// Render `explain` for human reading — every structured reason in a fixed section
/// order: eligibility, blocker chain, evicted edges, score, SL-213 PHASE-06's
/// value-source block, then the inert priority-domain disclosure (last —
/// corpus-global, not entity-scoped, design §4 S4).
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
    // SL-218 PHASE-03 (design §3): the tensions section, after score. Both classes
    // for a frontier id; the "not on the current frontier" disclosure otherwise.
    parts.push(explain_tension_section(ex));
    if let Some(r) = &ex.value_source {
        parts.push(reason_line(r));
    }
    if let Some(r) = &ex.priority_disclosure {
        parts.push(reason_line(r));
    }
    if let Some(r) = &ex.agent_demotion {
        parts.push(reason_line(r));
    }
    parts.concat()
}

/// The `explain` tensions section (design §3): every tension involving the id
/// (both classes, already filtered in the surface shell) as reason lines; or the
/// "not on the current frontier" disclosure when the id is not actionable. Empty
/// for a frontier id with no tensions (nothing to say).
fn explain_tension_section(ex: &Explanation) -> String {
    if !ex.on_frontier {
        return "  not on the current frontier — no tension analysis\n".to_string();
    }
    ex.tensions.iter().map(reason_line).collect()
}

// ---------------------------------------------------------------------------
// findings (SL-194 PHASE-01) — human grouped by kind + --json.
// ---------------------------------------------------------------------------

/// One `findings` line (the indented body under its kind header) — the render source of
/// truth for the human catalogue. Provenance REUSES the shared [`provenance_fragment`]
/// (R2), never re-formatting. Every other variant formats its own structured payload.
fn finding_line(f: &Finding) -> String {
    match f {
        Finding::Fork { hub, arms } => {
            format!(
                "  {hub}  settles → {{{}}}   ({} arms)\n",
                arms.join(", "),
                arms.len()
            )
        }
        Finding::Join { node, prereqs } => format!(
            "  {node}  needs → {{{}}}   ({} prereqs)\n",
            prereqs.join(", "),
            prereqs.len()
        ),
        Finding::GatingFanOut { record, blocks } => format!(
            "  {record}  gates → {{{}}}   ({} blocks)\n",
            blocks.join(", "),
            blocks.len()
        ),
        Finding::ValueInversion {
            blocker,
            blocked,
            gap,
        } => format!("  {blocker} gates {blocked}   Δ{gap:.1}\n"),
        Finding::Displacement {
            node,
            score_rank,
            constrained_rank,
            delta,
        } => format!("  {node}  score #{score_rank} vs survey #{constrained_rank}   Δ{delta}\n"),
        Finding::Plateau { members, span } => {
            let body = match members.as_slice() {
                [] => String::new(),
                [only] => only.clone(),
                [first, .., last] => format!("{first} … {last}"),
            };
            format!("  {{{body}}}  ({}, span {span:.2})\n", members.len())
        }
        Finding::OrderInstability { high, low, .. } => {
            format!("  {high} ↔ {low}   (flips β0↔β1)\n")
        }
        Finding::ArmResequencing {
            hub,
            order_lo,
            order_hi,
            ..
        } => format!(
            "  {hub}  arms {{{}}} → {{{}}}   (β0↔β1)\n",
            order_lo.join(", "),
            order_hi.join(", ")
        ),
        // Provenance reuses the shared explain fragment (R2 — no re-format).
        Finding::Provenance(reason) => {
            format!("  {}\n", provenance_fragment(reason).unwrap_or_default())
        }
        Finding::PreferenceCycle { classes, rows } => format!(
            "  cycle among {{{}}} — quarantines {{{}}} ({} rows); exit: supersede one row \
             (`--supersedes <uid>`) or tombstone one to break the cycle\n",
            classes.join(", "),
            rows.join(", "),
            rows.len()
        ),
        Finding::AnchorConflict { anchors, rows } => {
            let anchor_text = anchors
                .iter()
                .map(|(e, v)| format!("{e}={v:.1}"))
                .collect::<Vec<_>>()
                .join(" vs ");
            format!(
                "  anchors {anchor_text} conflict — quarantines {{{}}}; exit: supersede a \
                 conflicting row, tombstone one, or edit an anchor\n",
                rows.join(", ")
            )
        }
        Finding::AnchorGaugeDisconnect { entities } => format!(
            "  {{{}}} placed by gauge convention — no order path to any anchor; compare against \
             an anchored item to place it\n",
            entities.join(", ")
        ),
        Finding::MalformedSupersession { rows } => format!(
            "  supersession cycle among {{{}}} — all deactivated; exit: tombstone one row to \
             break the cycle\n",
            rows.join(", ")
        ),
    }
}

/// Render `findings` for human reading — findings GROUPED by `kind_label` (a header per
/// group), one line per finding within. `detect` already sorts `(kind_label, magnitude
/// desc)`, so magnitude ranks WITHIN a kind section and kind grouping outranks magnitude
/// (F-ext-5). An empty catalogue renders a clean note.
pub(crate) fn findings_human(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "(no findings)\n".to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    let mut current: Option<&str> = None;
    for f in findings {
        let label = f.kind_label();
        if current != Some(label) {
            parts.push(format!("{label}\n"));
            current = Some(label);
        }
        parts.push(finding_line(f));
    }
    parts.concat()
}

/// One finding as JSON — `{kind, …payload, magnitude}` (the `kind` tag IS the
/// `kind_label`, the group header's single source). Provenance nests the shared
/// [`reason_json`] under `detail` (source-of-truth reuse — no re-format).
fn finding_json(f: &Finding) -> serde_json::Value {
    let mut value = match f {
        Finding::Fork { hub, arms } => serde_json::json!({ "hub": hub, "arms": arms }),
        Finding::Join { node, prereqs } => serde_json::json!({ "node": node, "prereqs": prereqs }),
        Finding::GatingFanOut { record, blocks } => {
            serde_json::json!({ "record": record, "blocks": blocks })
        }
        Finding::ValueInversion {
            blocker,
            blocked,
            gap,
        } => serde_json::json!({ "blocker": blocker, "blocked": blocked, "gap": gap }),
        Finding::Displacement {
            node,
            score_rank,
            constrained_rank,
            delta,
        } => serde_json::json!({
            "node": node,
            "score_rank": score_rank,
            "constrained_rank": constrained_rank,
            "delta": delta,
        }),
        Finding::Plateau { members, span } => {
            serde_json::json!({ "members": members, "span": span })
        }
        // `moved` is surfaced via the top-level `magnitude`, NOT as a payload key
        // (design payload = {high, low}).
        Finding::OrderInstability { high, low, .. } => {
            serde_json::json!({ "high": high, "low": low })
        }
        Finding::ArmResequencing {
            hub,
            order_lo,
            order_hi,
            ..
        } => serde_json::json!({ "hub": hub, "order_lo": order_lo, "order_hi": order_hi }),
        Finding::Provenance(reason) => serde_json::json!({ "detail": reason_json(reason) }),
        Finding::PreferenceCycle { classes, rows } => {
            serde_json::json!({ "classes": classes, "rows": rows })
        }
        Finding::AnchorConflict { anchors, rows } => {
            let anchors: Vec<serde_json::Value> = anchors
                .iter()
                .map(|(e, v)| serde_json::json!({ "entity": e, "value": v }))
                .collect();
            serde_json::json!({ "anchors": anchors, "rows": rows })
        }
        Finding::AnchorGaugeDisconnect { entities } => serde_json::json!({ "entities": entities }),
        Finding::MalformedSupersession { rows } => serde_json::json!({ "rows": rows }),
    };
    if let Some(obj) = value.as_object_mut() {
        obj.insert("kind".to_string(), serde_json::json!(f.kind_label()));
        obj.insert("magnitude".to_string(), serde_json::json!(f.magnitude()));
    }
    value
}

/// `findings --json` — every finding as `{kind, …payload, magnitude}` under the
/// policy-versioned envelope.
pub(crate) fn findings_json(findings: &[Finding]) -> anyhow::Result<String> {
    let items: Vec<serde_json::Value> = findings.iter().map(finding_json).collect();
    finish(&serde_json::json!({
        "kind": "findings",
        "policy_version": PRIORITY_POLICY_VERSION,
        "findings": items,
    }))
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
        ReasonKind::ValueAuthored { value, conflict } => serde_json::json!({
            "kind": "value_authored",
            "value": value,
            "conflict": conflict,
        }),
        ReasonKind::ValueProjected {
            value,
            lower,
            upper,
            human,
            agent,
        } => serde_json::json!({
            "kind": "value_projected",
            "value": value,
            "lower": lower,
            "upper": upper,
            "human": human,
            "agent": agent,
        }),
        ReasonKind::ValueGauge { value, judgements } => serde_json::json!({
            "kind": "value_gauge",
            "value": value,
            "judgements": judgements,
        }),
        ReasonKind::PriorityDomainDisclosure { count } => serde_json::json!({
            "kind": "priority_domain_disclosure",
            "count": count,
        }),
        ReasonKind::AgentEvidenceDemoted => serde_json::json!({
            "kind": "agent_evidence_demoted",
            "text": AGENT_DEMOTION_DISCLOSURE,
        }),
        ReasonKind::Tension { .. } => {
            let mut v = tension_json(reason).unwrap_or_else(|| serde_json::json!({}));
            if let Some(obj) = v.as_object_mut() {
                obj.insert("kind".into(), serde_json::json!("tension"));
            }
            v
        }
        ReasonKind::ZeroWeightExcluded { count } => serde_json::json!({
            "kind": "zero_weight_excluded",
            "count": count,
        }),
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
pub(crate) fn next_json(view: &NextView) -> anyhow::Result<String> {
    let rows: Vec<serde_json::Value> = view
        .rows
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
    // SL-218 PHASE-03: the FULL structured tension list, uncapped (the
    // `TENSION_MAX_CALLOUTS` cap is a human-render bound only, design §3). Both
    // classes; the page-scoped m=0 disclosure rides alongside as `zero_weight`.
    let tensions: Vec<serde_json::Value> = view.tensions.iter().filter_map(tension_json).collect();
    finish(&serde_json::json!({
        "kind": "next",
        "policy_version": PRIORITY_POLICY_VERSION,
        "rows": rows,
        "tensions": tensions,
        "zero_weight": view.zero_weight.as_ref().map(reason_json),
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

/// `explain --json` — every structured reason faithfully serialized. SL-213
/// PHASE-06's `value_source`/`priority_disclosure` are `null` when absent —
/// the same structural fields as human, `--json` carries them, never a
/// re-derivation (design §4 S3).
pub(crate) fn explain_json(ex: &Explanation) -> anyhow::Result<String> {
    let mut envelope = serde_json::json!({
        "kind": "explain",
        "policy_version": PRIORITY_POLICY_VERSION,
        "id": ex.id,
        "eligibility": reason_json(&ex.eligibility),
        "blocker_chain": ex.blocker_chain.iter().map(reason_json).collect::<Vec<_>>(),
        "evictions": ex.evictions.iter().map(reason_json).collect::<Vec<_>>(),
        "score": reason_json(&ex.score),
        "value_source": ex.value_source.as_ref().map(reason_json),
        "priority_disclosure": ex.priority_disclosure.as_ref().map(reason_json),
        // SL-218 PHASE-03: tensions involving this id (both classes, filtered in
        // the surface shell) + frontier participation (design §2 considered-set).
        "on_frontier": ex.on_frontier,
        "tensions": ex.tensions.iter().filter_map(tension_json).collect::<Vec<_>>(),
    });
    // SL-218 D2: ADDITIVE key, knob-on only — knob-off bytes stay identical
    // to shipped output (INV-1), unlike the always-present nullable fields.
    if let Some(r) = &ex.agent_demotion
        && let Some(obj) = envelope.as_object_mut()
    {
        obj.insert("agent_demotion".to_string(), reason_json(r));
    }
    finish(&envelope)
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
    /// Wrap rows in a tension-free `NextView` for the table/pagination tests
    /// (SL-218 PHASE-03 — tension rendering is covered by the e2e goldens).
    fn nv(rows: &[NextRow]) -> NextView {
        NextView {
            rows: rows.to_vec(),
            tensions: Vec::new(),
            zero_weight: None,
        }
    }

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
            &nv(&rows),
            RenderOpts::default(),
            Some(&["id".to_string(), "score".to_string()]),
            20,
            0,
            false,
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
            &nv(&rows),
            RenderOpts::default(),
            Some(&["bogus".to_string()]),
            20,
            0,
            false,
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
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
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
            &nv(&rows),
            RenderOpts::default(),
            Some(&["unblocks".to_string()]),
            20,
            0,
            false,
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
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
        assert!(
            header(&out).contains("tags"),
            "tags column appears when any row tagged: {out}"
        );
    }

    #[test]
    fn vt3_tags_column_hidden_when_none_tagged() {
        let rows = vec![bare_row("ISS-001"), bare_row("ISS-002")];
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
        assert!(
            !header(&out).contains("tags"),
            "tags column hidden when none tagged: {out}"
        );
    }

    #[test]
    fn vt3_columns_tags_forces_column_even_all_empty() {
        let rows = vec![bare_row("ISS-001")];
        let out = next_human(
            &nv(&rows),
            RenderOpts::default(),
            Some(&["id".to_string(), "tags".to_string()]),
            20,
            0,
            false,
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
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
        assert!(out.contains(" 3.2 -  4.8"), "fractional estimate: {out}");
    }

    #[test]
    fn vt4_format_bound_estimate_integral() {
        let rows = vec![faceted_row("ISS-001", 3.0, 8.0, 5.0, &[])];
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
        assert!(
            out.contains(" 3.0 -  8.0"),
            "integral estimate shows .0: {out}"
        );
    }

    #[test]
    fn vt4_format_bound_value_integral() {
        let rows = vec![faceted_row("ISS-001", 3.0, 8.0, 5.0, &[])];
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
        // value 5.0 → "5.0" via format_bound (always one decimal)
        assert!(
            out.contains(" 5.0 ") || out.contains("│5.0│"),
            "integral value 5.0: {out}"
        );
    }

    #[test]
    fn vt4_format_bound_value_fractional() {
        let rows = vec![faceted_row("ISS-001", 3.0, 8.0, 5.5, &[])];
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
        assert!(out.contains("5.5"), "fractional value 5.5: {out}");
    }

    #[test]
    fn vt4_absent_cell_for_bare_row() {
        let rows = vec![bare_row("ISS-001")];
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 20, 0, false).unwrap();
        assert!(out.contains(ABSENT_CELL), "bare row has ABSENT_CELL: {out}");
    }

    #[test]
    fn value_cell_shows_marked_default_for_value_bearing_kind_without_facet() {
        // IMP-211: a value-bearing kind with no authored [value] is SCORED as
        // graph::DEFAULT_VALUE (effective_raw_value), so the value cell must show
        // that default (marked), never ABSENT_CELL — else the displayed value
        // contradicts the score driving the ranking.
        let mut r = bare_row("ISS-001"); // kind ISS is value-bearing, value None
        assert_eq!(
            value_cell(&r),
            format!("{}{DEFAULT_VALUE_MARKER}", format_bound(DEFAULT_VALUE))
        );
        // An authored value still renders bare — no marker.
        r.value = Some(ValueFacet { value: 2.0 });
        assert_eq!(value_cell(&r), format_bound(2.0));
    }

    #[test]
    fn value_cell_is_absent_for_valueless_kind_without_facet() {
        // A valueless kind (governance/REV/records) contributes no value to the
        // score, so an absent value stays ABSENT_CELL — the default marker would
        // misrepresent it.
        let mut r = bare_row("REV-001");
        r.kind = "REV".to_string();
        r.value = None;
        assert_eq!(value_cell(&r), ABSENT_CELL);
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
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 2, 0, false).unwrap();
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
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 2, 2, false).unwrap();
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
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 0, 0, false).unwrap();
        for n in 1..=5 {
            assert!(out.contains(&format!("ISS-00{n}")), "row {n} shown: {out}");
        }
        assert!(!out.contains(" of 5"), "no footer when uncapped: {out}");
    }

    #[test]
    fn vt_pagination_limit_zero_with_offset_no_panic_no_footer() {
        // F1 guard: limit==0 with offset>0 must not divide by zero in the footer.
        let rows = five_rows();
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 0, 2, false).unwrap();
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
        let out = next_human(&nv(&rows), RenderOpts::default(), None, 2, 10, false).unwrap();
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
        let page1 = next_human(&nv(&rows), RenderOpts::default(), None, 2, 0, false).unwrap();
        assert!(
            !page1.lines().next().unwrap_or("").contains("tags"),
            "page 1 (no tagged row) hides tags column: {page1}"
        );
        let page2 = next_human(&nv(&rows), RenderOpts::default(), None, 2, 2, false).unwrap();
        assert!(
            page2.lines().next().unwrap_or("").contains("tags"),
            "page 2 (tagged row) shows tags column: {page2}"
        );
    }

    // ── SL-194 VT-6: findings_human golden + findings_json shape ────────────

    use super::super::view::ReasonKind;
    use crate::backlog_order::OverrideReason;

    #[test]
    fn vt6_findings_human_groups_by_kind_and_reuses_provenance_fragment() {
        let findings = vec![
            Finding::Fork {
                hub: "ISS-001".to_string(),
                arms: vec!["ISS-002".to_string(), "ISS-003".to_string()],
            },
            Finding::ValueInversion {
                blocker: "ISS-004".to_string(),
                blocked: "ISS-005".to_string(),
                gap: 16.3,
            },
            Finding::Provenance(ReasonKind::EvictedEdge {
                from: "ISS-006".to_string(),
                to: "ISS-007".to_string(),
                reason: OverrideReason::SoftCycleEvicted,
            }),
        ];
        let out = findings_human(&findings);
        // Kind headers present, one per group.
        assert!(out.contains("forks\n"), "forks header: {out}");
        assert!(
            out.contains("value inversions\n"),
            "value inversions header: {out}"
        );
        assert!(out.contains("provenance\n"), "provenance header: {out}");
        // Fork line carries the hub → arms and the count.
        assert!(
            out.contains("ISS-001  settles → {ISS-002, ISS-003}   (2 arms)"),
            "fork line: {out}"
        );
        // Inversion line carries the Δ.
        assert!(
            out.contains("ISS-004 gates ISS-005   Δ16.3"),
            "inversion line: {out}"
        );
        // Provenance REUSES the explain fragment verbatim (R2) — same text `reason_line`
        // would produce for the wrapped ReasonKind.
        let via_reason = reason_line(&ReasonKind::EvictedEdge {
            from: "ISS-006".to_string(),
            to: "ISS-007".to_string(),
            reason: OverrideReason::SoftCycleEvicted,
        });
        assert!(
            out.contains(via_reason.trim_end()),
            "reuses explain fragment: {out}"
        );
    }

    #[test]
    fn vt6_findings_human_empty_is_clean_note() {
        assert_eq!(findings_human(&[]), "(no findings)\n");
    }

    #[test]
    fn vt6_findings_json_shape_kind_payload_magnitude() {
        let findings = vec![Finding::Fork {
            hub: "ISS-001".to_string(),
            arms: vec!["ISS-002".to_string(), "ISS-003".to_string()],
        }];
        let out = findings_json(&findings).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["kind"], "findings");
        assert_eq!(v["policy_version"], PRIORITY_POLICY_VERSION);
        let f0 = &v["findings"][0];
        assert_eq!(f0["kind"], "forks", "json kind tag == kind_label");
        assert_eq!(f0["hub"], "ISS-001");
        assert_eq!(f0["arms"][0], "ISS-002");
        assert_eq!(f0["magnitude"], 2.0, "magnitude = arm count");
    }

    #[test]
    fn vt6_findings_json_provenance_nests_reason() {
        let findings = vec![Finding::Provenance(ReasonKind::CycleDegraded {
            nodes: vec!["ISS-001".to_string(), "ISS-002".to_string()],
        })];
        let out = findings_json(&findings).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let f0 = &v["findings"][0];
        assert_eq!(f0["kind"], "provenance");
        // The nested detail is the shared reason_json (source-of-truth reuse).
        assert_eq!(f0["detail"]["kind"], "cycle_degraded");
        assert_eq!(f0["detail"]["nodes"][1], "ISS-002");
        assert_eq!(f0["magnitude"], 2.0);
    }

    // ── SL-194 VT-3: β-family render extension (human line + json payload) ──────

    #[test]
    fn vt3_beta_family_renders_one_line_each_human() {
        let findings = vec![
            Finding::OrderInstability {
                high: "IMP-054".to_string(),
                low: "IMP-071".to_string(),
                moved: 3,
            },
            Finding::ArmResequencing {
                hub: "QUE-003".to_string(),
                order_lo: vec!["IMP-054".to_string(), "IMP-071".to_string()],
                order_hi: vec!["IMP-071".to_string(), "IMP-054".to_string()],
                moved: 2,
            },
        ];
        let out = findings_human(&findings);
        // Kind headers present, one per group.
        assert!(
            out.contains("order instability\n"),
            "order instability header: {out}"
        );
        assert!(
            out.contains("arm resequencing\n"),
            "arm resequencing header: {out}"
        );
        // OrderInstability: the contested pair, flip annotation.
        assert!(
            out.contains("IMP-054 ↔ IMP-071   (flips β0↔β1)"),
            "order-instability line: {out}"
        );
        // ArmResequencing: hub + both arm orders.
        assert!(
            out.contains("QUE-003  arms {IMP-054, IMP-071} → {IMP-071, IMP-054}   (β0↔β1)"),
            "arm-resequencing line: {out}"
        );
    }

    #[test]
    fn vt3_beta_family_json_payload_and_magnitude() {
        // OrderInstability: payload {high, low} + kind + magnitude (== moved); `moved`
        // itself is NOT a payload key.
        let oi = Finding::OrderInstability {
            high: "IMP-054".to_string(),
            low: "IMP-071".to_string(),
            moved: 4,
        };
        let out = findings_json(std::slice::from_ref(&oi)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let f0 = &v["findings"][0];
        assert_eq!(f0["kind"], "order instability");
        assert_eq!(f0["high"], "IMP-054");
        assert_eq!(f0["low"], "IMP-071");
        assert_eq!(f0["magnitude"], 4.0, "magnitude = positions moved");
        assert!(
            f0.get("moved").is_none(),
            "moved is surfaced via magnitude only"
        );

        // ArmResequencing: payload {hub, order_lo, order_hi} + kind + magnitude.
        let ar = Finding::ArmResequencing {
            hub: "QUE-003".to_string(),
            order_lo: vec!["A-1".to_string(), "A-2".to_string()],
            order_hi: vec!["A-2".to_string(), "A-1".to_string()],
            moved: 2,
        };
        let out2 = findings_json(std::slice::from_ref(&ar)).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&out2).unwrap();
        let g0 = &v2["findings"][0];
        assert_eq!(g0["kind"], "arm resequencing");
        assert_eq!(g0["hub"], "QUE-003");
        assert_eq!(g0["order_lo"][0], "A-1");
        assert_eq!(g0["order_hi"][0], "A-2");
        assert_eq!(g0["magnitude"], 2.0, "magnitude = arms moved");
    }

    /// A findings list WITHOUT any β variant (the estimate-free / `None`-betas case)
    /// renders no β-family section — the human surface stays silent, mirroring `detect`.
    #[test]
    fn vt3_no_beta_variants_render_no_beta_section() {
        let findings = vec![Finding::Fork {
            hub: "ISS-001".to_string(),
            arms: vec!["ISS-002".to_string(), "ISS-003".to_string()],
        }];
        let out = findings_human(&findings);
        assert!(!out.contains("order instability"), "no OI section: {out}");
        assert!(!out.contains("arm resequencing"), "no AR section: {out}");
    }

    #[test]
    fn vt_d7_columns_tags_overrides_visible_gate() {
        // Explicit --columns tags forces the column even on a page with no tagged row.
        let rows = vec![bare_row("ISS-001"), bare_row("ISS-002")];
        let out = next_human(
            &nv(&rows),
            RenderOpts::default(),
            Some(&["id".to_string(), "tags".to_string()]),
            2,
            0,
            false,
        )
        .unwrap();
        assert!(
            out.lines().next().unwrap_or("").contains("tags"),
            "explicit --columns tags overrides the visible-page gate: {out}"
        );
    }
}
