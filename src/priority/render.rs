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

use super::findings::{ComparisonDomain, Finding};
use super::view::{
    ActionabilityBlock, BlockersView, ContestedClaim, EdgeVerb, Explanation, NextRow, NextView,
    ReasonKind, SurveyRow, TensionCauseView, TensionGradeView,
};
use crate::comparison::ClaimTier;

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

/// Render the estimate column cell from the resolved cost-source reason
/// (SL-222 PHASE-09). Falls back to [`listing::ABSENT_CELL`] when no cost
/// source is available.
fn estimate_cell(r: &NextRow) -> String {
    match &r.cost_source {
        Some(reason) => estimate_cell_from_reason(reason),
        None => listing::ABSENT_CELL.to_string(),
    }
}

/// Extract the lower/upper bounds from a cost-source reason for the
/// estimate column cell.
fn estimate_cell_from_reason(reason: &ReasonKind) -> String {
    let (lo, hi) = match reason {
        ReasonKind::CostPin { lower, upper, .. } | ReasonKind::CostClaim { lower, upper, .. } => {
            (*lower, *upper)
        }
        ReasonKind::CostProjected {
            lower: Some(l),
            upper: Some(u),
            ..
        } => (*l, *u),
        ReasonKind::CostProjected { est_cost, .. }
        | ReasonKind::CostClassAnchor { est_cost, .. }
        | ReasonKind::CostBareAnchor { est_cost, .. }
        | ReasonKind::CostGauge { est_cost, .. } => (*est_cost, *est_cost),
        ReasonKind::CostUnmigratedFacet => (0.0, 0.0),
        _ => return listing::ABSENT_CELL.to_string(),
    };
    format!("{:>4} - {:>4}", format_bound(lo), format_bound(hi))
}

/// Marker suffix on a value cell showing the effective *default* value (a
/// value-bearing kind that authored no `[value]`), distinguishing it from an
/// authored value of the same magnitude (IMP-211).
const DEFAULT_VALUE_MARKER: &str = "*";
/// SL-220 PHASE-06 per-rung value-cell source markers (design §6) — the compact
/// glyph the value column appends to signal provenance. Implementation-owned,
/// distinct, pinned by the row goldens. A human claim (rung 1, canonical
/// evidence) renders BARE, mirroring the retired authored-facet convention.
const VALUE_MARKER_PIN: &str = "!";
const VALUE_MARKER_HUMAN: &str = "";
const VALUE_MARKER_AGENT: &str = "~";
const VALUE_MARKER_MIGRATED: &str = "°";
const VALUE_MARKER_UNMIGRATED: &str = "?";
const VALUE_MARKER_PROJECTED: &str = "≈";
const VALUE_MARKER_GAUGE: &str = "^";

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
    match &r.value_source {
        Some(reason) => {
            let (value, marker) = value_cell_parts(reason);
            format!("{}{marker}", format_bound(value))
        }
        None if crate::kinds::is_value_bearing(&r.kind) => {
            format!("{}{DEFAULT_VALUE_MARKER}", format_bound(DEFAULT_VALUE))
        }
        None => listing::ABSENT_CELL.to_string(),
    }
}

/// Extract the value-cell magnitude + per-rung source marker from a resolved
/// value-source reason (SL-220 PHASE-06, design §6). The reason always comes
/// from [`super::surface::value_source_reason`], so only the value-source arms
/// are reachable; the fallthrough is a defensive floor.
fn value_cell_parts(reason: &ReasonKind) -> (f64, &'static str) {
    match reason {
        ReasonKind::ValuePin { value, .. } => (*value, VALUE_MARKER_PIN),
        ReasonKind::ValueClaim { value, tier, .. } => (
            *value,
            match tier {
                ClaimTier::Pin => VALUE_MARKER_PIN,
                ClaimTier::Human => VALUE_MARKER_HUMAN,
                ClaimTier::Agent => VALUE_MARKER_AGENT,
                ClaimTier::Migrated => VALUE_MARKER_MIGRATED,
            },
        ),
        ReasonKind::ValueUnmigratedFacet => (DEFAULT_VALUE, VALUE_MARKER_UNMIGRATED),
        ReasonKind::ValueProjected { value, .. } => (*value, VALUE_MARKER_PROJECTED),
        ReasonKind::ValueGauge { value, .. } => (*value, VALUE_MARKER_GAUGE),
        _ => (DEFAULT_VALUE, DEFAULT_VALUE_MARKER),
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
        ReasonKind::ValuePin { .. }
        | ReasonKind::ValueClaim { .. }
        | ReasonKind::ValueUnmigratedFacet
        | ReasonKind::ValueProjected { .. }
        | ReasonKind::ValueGauge { .. } => {
            format!("  {}\n", value_source_fragment(reason).unwrap_or_default())
        }
        ReasonKind::CostPin { .. }
        | ReasonKind::CostClaim { .. }
        | ReasonKind::CostClassAnchor { .. }
        | ReasonKind::CostUnmigratedFacet
        | ReasonKind::CostProjected { .. }
        | ReasonKind::CostBareAnchor { .. }
        | ReasonKind::CostGauge { .. } => {
            format!("  {}\n", cost_source_fragment(reason).unwrap_or_default())
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
        // SL-220 PHASE-06 claim shapes (design §6) — attribution parenthetical,
        // the contested-interval variant (anchored-tier "contested … resolve by
        // superseding row" vs agent/migrated "… calibrate via comparison", D14),
        // and the anchor-conflict citation suffix.
        ReasonKind::ValuePin {
            value,
            conflict,
            by,
            date,
            basis,
            contested,
        } => Some(claim_fragment(
            *value,
            ClaimTier::Pin,
            by.as_deref(),
            date.as_deref(),
            basis.as_deref(),
            contested.as_ref(),
            conflict,
        )),
        ReasonKind::ValueClaim {
            value,
            tier,
            conflict,
            by,
            date,
            contested,
        } => Some(claim_fragment(
            *value,
            *tier,
            by.as_deref(),
            date.as_deref(),
            None,
            contested.as_ref(),
            conflict,
        )),
        ReasonKind::ValueUnmigratedFacet => Some(
            "value — unmigrated [value] facet — run scripts/migrate_value_facets.py".to_string(),
        ),
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

/// The SL-220 PHASE-06 pin/claim value-source fragment (design §6) — the SINGLE
/// template for every ledgered-claim rung, shared by `explain` (human) and, via
/// the reason, the elicit + `show` surfaces. Three cases:
///
/// - **contested** (same-tier magnitude disagreement): the interval + row-count
///   line. Anchored tiers (Pin/Human) carry the "contested" framing and the
///   "resolve by superseding row" reprobe disclosure; agent/migrated tiers drop
///   "contested" and read "calibrate via comparison" instead (D14).
/// - **singleton**: `— {tier word}{attribution}`, with the agent prior's
///   "below projection" disclosure (rung 3 — projection would have won had any
///   evidence existed).
///
/// The anchor-conflict citation (a cross-class order violation, distinct from
/// the same-tier contest) is appended in every case.
fn claim_fragment(
    value: f64,
    tier: ClaimTier,
    by: Option<&str>,
    date: Option<&str>,
    basis: Option<&str>,
    contested: Option<&ContestedClaim>,
    conflict: &[String],
) -> String {
    let anchored = matches!(tier, ClaimTier::Pin | ClaimTier::Human);
    let tier_word = match tier {
        ClaimTier::Pin => "pin",
        ClaimTier::Human => "human claim",
        ClaimTier::Agent => "agent claim",
        ClaimTier::Migrated => "migrated claim",
    };
    let body = if let Some(c) = contested {
        // The "contested" label is anchored-tiers-only; agent/migrated keep the
        // bare tier word and route to comparison, not the human reprobe queue.
        let label = match tier {
            ClaimTier::Pin => "contested pin",
            ClaimTier::Human => "contested human claim",
            _ => tier_word,
        };
        let guidance = if anchored {
            "resolve by superseding row"
        } else {
            "calibrate via comparison"
        };
        format!(
            "value {value:.1} — {label} · {} claims, interval ({:.1} ‥ {:.1}), mean — {guidance}",
            c.rows, c.low, c.high,
        )
    } else {
        let attr = attribution_paren(tier, by, date, basis);
        // Only the agent prior (rung 3) discloses "below projection" — a pin or
        // human claim anchors (rung 1); a migrated prior renders bare (design §6).
        let prior_suffix = if matches!(tier, ClaimTier::Agent) {
            " · below projection — no projection evidence exists"
        } else {
            ""
        };
        format!("value {value:.1} — {tier_word}{attr}{prior_suffix}")
    };
    format!("{body}{}", anchor_conflict_suffix(conflict))
}

/// The attribution parenthetical (design §6). Migrated rows read
/// `(unattributed · observed <date>)` — `by` is typically absent, and the
/// timestamp is the migration `observed_at`. Every other tier reads
/// `(by, date[, basis N])`, dropping any absent part; a fully-absent
/// attribution renders nothing.
fn attribution_paren(
    tier: ClaimTier,
    by: Option<&str>,
    date: Option<&str>,
    basis: Option<&str>,
) -> String {
    if matches!(tier, ClaimTier::Migrated) {
        let who = by.unwrap_or("unattributed");
        return match date {
            Some(d) => format!(" ({who} · observed {d})"),
            None => format!(" ({who})"),
        };
    }
    let mut parts: Vec<String> = Vec::new();
    if let Some(b) = by {
        parts.push(b.to_string());
    }
    if let Some(d) = date {
        parts.push(d.to_string());
    }
    if let Some(b) = basis {
        parts.push(format!("basis {b}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(", "))
    }
}

/// The cross-class anchor-conflict citation suffix (an `AnchorConflict` finding
/// reference) — distinct from the same-tier contest. Empty when no citation.
fn anchor_conflict_suffix(conflict: &[String]) -> String {
    if conflict.is_empty() {
        String::new()
    } else {
        format!(" (see anchor-conflict finding: {})", conflict.join(", "))
    }
}

/// The SL-220 PHASE-06 entity-`show` value line (design §6) — the SINGLE pure
/// renderer that dissolved the former nine-fold `format_value_normal`
/// duplication. Consumes the RESOLVED value-source reason (never the raw
/// facet), so a record's captured human claim renders with its provenance and
/// a `scoring-inert` annotation (D7). `inert_kind` is `Some(kind)` for a
/// non-value-bearing kind; `None` (evidence absent) omits the line entirely.
/// The unit is retained alongside the provenance parenthetical.
pub(crate) fn show_value_render(
    reason: &ReasonKind,
    unit: &str,
    inert_kind: Option<&str>,
) -> Option<String> {
    let provenance = show_provenance(reason)?;
    let (value, _) = value_cell_parts(reason);
    let base = format!("value: {value:.1} {unit} ({provenance})");
    Some(match inert_kind {
        Some(kind) => format!("{base} — scoring-inert ({kind} kind)"),
        None => base,
    })
}

/// The SL-222 PHASE-07 `show` cost line (design §6 remainder) — the SINGLE
/// renderer for the entity `show`'s estimate provenance line, analogous to
/// [`show_value_render`]. Consumes the RESOLVED cost-source reason (never the
/// raw facet). `inert_kind` annotates a record-kind entity whose captured
/// estimate claim is scoring-inert (D7). Absent evidence ⇒ `None` (line
/// omitted).
pub(crate) fn show_cost_render(
    reason: &ReasonKind,
    unit: &str,
    inert_kind: Option<&str>,
) -> Option<String> {
    let provenance = show_cost_provenance(reason)?;
    // Extract range (lower–upper) from the reason; single-value variants use
    // est_cost for both sides.
    let (lo, hi): (f64, f64) = match reason {
        ReasonKind::CostPin { lower, upper, .. } | ReasonKind::CostClaim { lower, upper, .. } => {
            (*lower, *upper)
        }
        ReasonKind::CostUnmigratedFacet => (0.0, 0.0),
        ReasonKind::CostProjected {
            lower: Some(l),
            upper: Some(u),
            ..
        } => (*l, *u),
        ReasonKind::CostProjected { est_cost, .. }
        | ReasonKind::CostClassAnchor { est_cost, .. }
        | ReasonKind::CostBareAnchor { est_cost, .. }
        | ReasonKind::CostGauge { est_cost, .. } => (*est_cost, *est_cost),
        _ => return None,
    };
    let base = format!("estimate: {lo:.1}–{hi:.1} {unit} ({provenance})");
    Some(match inert_kind {
        Some(kind) => format!("{base} — scoring-inert ({kind} kind)"),
        None => base,
    })
}

/// The compact provenance phrase for the `show` estimate line (design §6 remainder),
/// analogous to [`show_provenance`] — tier word + singleton attribution. `None`
/// for a non-cost-source reason.
pub(crate) fn show_cost_provenance(reason: &ReasonKind) -> Option<String> {
    match reason {
        ReasonKind::CostPin {
            by,
            date,
            contested,
            ..
        } => {
            let head = if contested.is_some() {
                "contested pin"
            } else {
                "pin"
            };
            Some(join_provenance(head, by.as_deref(), date.as_deref(), false))
        }
        ReasonKind::CostClaim { tier, by, date, .. } => {
            let word = match tier {
                ClaimTier::Pin => "pin",
                ClaimTier::Human => "human claim",
                ClaimTier::Agent => "agent claim",
                ClaimTier::Migrated => "migrated claim",
            };
            let migrated = matches!(tier, ClaimTier::Migrated);
            let head = word.to_string();
            Some(join_provenance(
                &head,
                by.as_deref(),
                date.as_deref(),
                migrated,
            ))
        }
        ReasonKind::CostClassAnchor { .. } => Some("class anchor".to_string()),
        ReasonKind::CostUnmigratedFacet => Some("unmigrated [estimate] facet".to_string()),
        ReasonKind::CostProjected { .. } => Some("projected".to_string()),
        ReasonKind::CostGauge { .. } => Some("gauge".to_string()),
        ReasonKind::CostBareAnchor { .. } => Some("bare anchor".to_string()),
        _ => None,
    }
}

/// The compact provenance phrase for the `show` value line (design §6) — tier
/// word + singleton attribution (`, by, date`; migrated reads
/// `, unattributed, observed <date>`), with a leading "contested" for an
/// anchored-tier same-tier conflict. `None` for a non-value-source reason.
fn show_provenance(reason: &ReasonKind) -> Option<String> {
    match reason {
        ReasonKind::ValuePin {
            by,
            date,
            contested,
            ..
        } => {
            let head = if contested.is_some() {
                "contested pin"
            } else {
                "pin"
            };
            Some(join_provenance(head, by.as_deref(), date.as_deref(), false))
        }
        ReasonKind::ValueClaim {
            tier,
            by,
            date,
            contested,
            ..
        } => {
            let word = match tier {
                ClaimTier::Pin => "pin",
                ClaimTier::Human => "human claim",
                ClaimTier::Agent => "agent claim",
                ClaimTier::Migrated => "migrated claim",
            };
            let anchored = matches!(tier, ClaimTier::Pin | ClaimTier::Human);
            let head = if contested.is_some() && anchored {
                format!("contested {word}")
            } else {
                word.to_string()
            };
            let migrated = matches!(tier, ClaimTier::Migrated);
            Some(join_provenance(
                &head,
                by.as_deref(),
                date.as_deref(),
                migrated,
            ))
        }
        ReasonKind::ValueUnmigratedFacet => Some("unmigrated [value] facet".to_string()),
        ReasonKind::ValueProjected { .. } => Some("projected".to_string()),
        ReasonKind::ValueGauge { .. } => Some("gauge".to_string()),
        _ => None,
    }
}

/// Append the singleton attribution to a provenance head phrase (design §6).
/// A migrated row with no `by` reads `unattributed` and frames its date as
/// `observed`; every other tier drops an absent part.
fn join_provenance(head: &str, by: Option<&str>, date: Option<&str>, migrated: bool) -> String {
    let mut s = head.to_string();
    match (by, migrated) {
        (Some(b), _) => {
            s.push_str(", ");
            s.push_str(b);
        }
        (None, true) => s.push_str(", unattributed"),
        (None, false) => {}
    }
    if let Some(d) = date {
        s.push_str(if migrated { ", observed " } else { ", " });
        s.push_str(d);
    }
    s
}

/// One C6 display bound as text — `"unbounded"` for `None`.
fn bound_fragment(bound: Option<f64>) -> String {
    match bound {
        Some(v) => format!("{v:.1}"),
        None => "unbounded".to_string(),
    }
}

/// The SL-219 PHASE-06 cost-source fragment (design §5, the three shapes' +
/// gauge flag's literal templates) — the SINGLE source shared by `reason_line`
/// (human `explain`) and the `--json` surface. Bare text framed by the caller;
/// the gauge case carries an INTERNAL newline (`\n  `) so the caller's single
/// `  {frag}\n` wrap renders TWO indented lines. `None` for any non-cost-source
/// reason.
pub(crate) fn cost_source_fragment(reason: &ReasonKind) -> Option<String> {
    match reason {
        ReasonKind::CostPin {
            est_cost,
            lower,
            upper,
            beta,
            by,
            date,
            basis,
            contested,
        } => {
            let base =
                format!("est_cost {est_cost:.1} — pin [{lower:.1} ‥ {upper:.1}] · β {beta:.2}");
            let suffix = if let Some(c) = contested {
                format!(
                    " — contested · {} claims, cost interval ({:.1} ‥ {:.1}), mean range [{:.1} ‥ {:.1}] — resolve by superseding row",
                    c.rows, c.low, c.high, c.low, c.high,
                )
            } else {
                let mut parts = Vec::new();
                if let Some(b) = by {
                    parts.push(b.clone());
                }
                if let Some(d) = date {
                    parts.push(d.clone());
                }
                if let Some(b) = basis {
                    parts.push(format!("basis {b}"));
                }
                if parts.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", parts.join(", "))
                }
            };
            Some(format!("{base}{suffix}"))
        }
        ReasonKind::CostClaim {
            est_cost,
            lower,
            upper,
            beta,
            tier,
            by,
            date,
            conflict,
        } => {
            let tier_label = match tier {
                ClaimTier::Human => "human claim",
                ClaimTier::Agent => "agent claim",
                ClaimTier::Migrated => "migrated claim",
                ClaimTier::Pin => "claim",
            };
            let conflict_rider = if conflict.is_empty() {
                String::new()
            } else {
                let cls = conflict.join(" vs ");
                format!(" · anchor conflict with {cls} — resolve by superseding row")
            };
            let attribution = {
                let mut parts = Vec::new();
                let by_str = by.as_deref().unwrap_or("unattributed");
                parts.push(by_str.to_string());
                if let Some(d) = date {
                    let prep = match tier {
                        ClaimTier::Migrated => format!("observed {d}"),
                        _ => d.clone(),
                    };
                    parts.push(prep);
                }
                if parts.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", parts.join(", "))
                }
            };
            Some(format!(
                "est_cost {est_cost:.1} — {tier_label} [{lower:.1} ‥ {upper:.1}] · β {beta:.2}{attribution}{conflict_rider}"
            ))
        }
        ReasonKind::CostClassAnchor { est_cost } => Some(format!(
            "est_cost {est_cost:.1} — anchored (via class anchor)"
        )),
        ReasonKind::CostUnmigratedFacet => Some(
            "est_cost — unmigrated [estimate] facet — run scripts/migrate_estimate_facets.py"
                .to_string(),
        ),
        ReasonKind::CostProjected {
            est_cost,
            lower,
            upper,
            human,
            agent,
        } => Some(format!(
            "est_cost {est_cost:.1} — projected · bounds ({} ‥ {}) · from {} constraining sizing \
             judgements ({human} human, {agent} agent)",
            bound_fragment(*lower),
            bound_fragment(*upper),
            human + agent,
        )),
        ReasonKind::CostBareAnchor {
            est_cost,
            max_estimate,
            margin,
        } => Some(bare_anchor_fragment(*est_cost, *max_estimate, *margin)),
        ReasonKind::CostGauge {
            est_cost,
            max_estimate,
            margin,
            judgements,
        } => Some(format!(
            "{}\n  sizing: gauge · ordered by {judgements} judgements, no estimated item in \
             component — estimate any member to calibrate",
            bare_anchor_fragment(*est_cost, *max_estimate, *margin),
        )),
        _ => None,
    }
}

/// The bare-anchor cost-source line (design §5 shape 3), reused verbatim as the
/// first line of the gauge flag case (D2 honesty — the divisor scoring actually
/// used). `max_estimate` `None` is the empty-corpus fallback (no authored upper
/// to cite; `est_cost` is then the `1.0` default).
fn bare_anchor_fragment(est_cost: f64, max_estimate: Option<f64>, margin: f64) -> String {
    match max_estimate {
        Some(me) => {
            format!(
                "est_cost {est_cost:.1} — bare anchor (max estimate {me:.1} + margin {margin:.1})"
            )
        }
        None => format!(
            "est_cost {est_cost:.1} — bare anchor (no estimate in corpus; default {est_cost:.1})"
        ),
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
    // SL-219 PHASE-06 (design §5): the cost-source block, beside value-source.
    if let Some(r) = &ex.cost_source {
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
        Finding::PreferenceCycle {
            domain,
            classes,
            rows,
        } => format!(
            "  {}cycle among {{{}}} — quarantines {{{}}} ({} rows); exit: supersede one row \
             (`--supersedes <uid>`) or tombstone one to break the cycle\n",
            domain_tag(*domain),
            classes.join(", "),
            rows.join(", "),
            rows.len()
        ),
        Finding::AnchorConflict {
            domain,
            anchors,
            rows,
        } => {
            let anchor_text = anchors
                .iter()
                .map(|(e, v)| format!("{e}={v:.1}"))
                .collect::<Vec<_>>()
                .join(" vs ");
            match domain {
                // The value wording is pre-SL-219 golden text — unchanged.
                ComparisonDomain::Value => format!(
                    "  anchors {anchor_text} conflict — quarantines {{{}}}; exit: supersede a \
                     conflicting row, tombstone one, or edit an anchor\n",
                    rows.join(", ")
                ),
                // SL-219 D1: the likeliest defect is a stale estimate.
                ComparisonDomain::Estimate => format!(
                    "  [estimate] anchors {anchor_text} conflict — sizing evidence contradicts \
                     the β-resolved costs; quarantines {{{}}}; exit: revise the estimate or \
                     supersede the row\n",
                    rows.join(", ")
                ),
            }
        }
        Finding::AnchorGaugeDisconnect { domain, entities } => format!(
            "  {}{{{}}} placed by gauge convention — no order path to any anchor; compare against \
             an anchored item to place it\n",
            domain_tag(*domain),
            entities.join(", ")
        ),
        Finding::MalformedSupersession { domain, rows } => format!(
            "  {}supersession cycle among {{{}}} — all deactivated; exit: tombstone one row to \
             break the cycle\n",
            domain_tag(*domain),
            rows.join(", ")
        ),
        Finding::UnmigratedFacet { domain, entity } => format!(
            "  {}{entity} authors an unmigrated `[{}]` facet — facet no longer read; exit: \
             run scripts/migrate_{}_facets.py (stdlib-only, any corpus root) \
             or re-assert via `{} set --rater human`\n",
            domain_tag(*domain),
            domain.token(),
            domain.token(),
            domain.token(),
        ),
        Finding::ClaimConflict {
            domain,
            item,
            tier,
            low,
            high,
            rows,
        } => {
            // Anchored tiers wear the "contested" framing + reprobe disclosure;
            // agent/migrated conflicts route to comparison instead (D14).
            let (label, exit) = match tier {
                ClaimTier::Pin => ("contested pin", "resolve by superseding row"),
                ClaimTier::Human => ("contested human claim", "resolve by superseding row"),
                ClaimTier::Agent => ("agent claim conflict", "calibrate via comparison"),
                ClaimTier::Migrated => ("migrated claim conflict", "calibrate via comparison"),
            };
            format!(
                "  {}{item} {label} — {rows} claims span interval ({low:.1} ‥ {high:.1}), mean \
                 stands; exit: {exit}\n",
                domain_tag(*domain),
            )
        }
    }
}

/// The est-domain line tag (SL-219 D9 domain-tagged render). Value-domain
/// lines stay UNTAGGED — they are pre-SL-219 golden text (extend, don't
/// replace); only the estimate system's findings announce their domain.
fn domain_tag(domain: ComparisonDomain) -> &'static str {
    match domain {
        ComparisonDomain::Value => "",
        ComparisonDomain::Estimate => "[estimate] ",
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
        // SL-219 D9: comparison findings carry their producing system's
        // `domain` token (JSON parity with the domain-tagged human render).
        Finding::PreferenceCycle {
            domain,
            classes,
            rows,
        } => {
            serde_json::json!({ "domain": domain.token(), "classes": classes, "rows": rows })
        }
        Finding::AnchorConflict {
            domain,
            anchors,
            rows,
        } => {
            let anchors: Vec<serde_json::Value> = anchors
                .iter()
                .map(|(e, v)| serde_json::json!({ "entity": e, "value": v }))
                .collect();
            serde_json::json!({ "domain": domain.token(), "anchors": anchors, "rows": rows })
        }
        Finding::AnchorGaugeDisconnect { domain, entities } => {
            serde_json::json!({ "domain": domain.token(), "entities": entities })
        }
        Finding::MalformedSupersession { domain, rows } => {
            serde_json::json!({ "domain": domain.token(), "rows": rows })
        }
        Finding::UnmigratedFacet { domain, entity } => {
            serde_json::json!({ "domain": domain.token(), "entity": entity })
        }
        Finding::ClaimConflict {
            domain,
            item,
            tier,
            low,
            high,
            rows,
        } => {
            let tier_token = match tier {
                ClaimTier::Pin => "pin",
                ClaimTier::Human => "human-claim",
                ClaimTier::Agent => "agent-claim",
                ClaimTier::Migrated => "migrated-claim",
            };
            serde_json::json!({
                "domain": domain.token(),
                "item": item,
                "tier": tier_token,
                "low": low,
                "high": high,
                "rows": rows,
            })
        }
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
        // SL-220 D11: the new value-source shapes carry the pinned provenance
        // token AS the kind (view.rs `value_source_token`, the single source);
        // `value_projected`/`value_gauge` below stay byte-stable.
        ReasonKind::ValuePin {
            value, conflict, ..
        }
        | ReasonKind::ValueClaim {
            value, conflict, ..
        } => {
            // Attribution/contested are render-only (design §6, human surfaces);
            // the JSON stays the pinned token + value + anchor-conflict citation.
            serde_json::json!({
                "kind": reason.value_source_token(),
                "value": value,
                "conflict": conflict,
            })
        }
        ReasonKind::ValueUnmigratedFacet => serde_json::json!({
            "kind": reason.value_source_token(),
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
        ReasonKind::CostPin {
            est_cost,
            lower,
            upper,
            beta,
            by,
            date,
            basis,
            contested,
        } => {
            serde_json::json!({
                "kind": reason.cost_source_token(),
                "est_cost": est_cost,
                "lower": lower,
                "upper": upper,
                "beta": beta,
                "by": by,
                "date": date,
                "basis": basis,
                "contested": contested.as_ref().map(|c| serde_json::json!({
                    "low": c.low,
                    "high": c.high,
                    "rows": c.rows,
                })),
            })
        }
        ReasonKind::CostClaim {
            est_cost,
            lower,
            upper,
            beta,
            tier,
            by,
            date,
            conflict,
        } => {
            serde_json::json!({
                "kind": reason.cost_source_token(),
                "est_cost": est_cost,
                "lower": lower,
                "upper": upper,
                "beta": beta,
                "tier": format!("{tier:?}"),
                "by": by,
                "date": date,
                "conflict": conflict,
            })
        }
        ReasonKind::CostClassAnchor { est_cost } => {
            serde_json::json!({
                "kind": reason.cost_source_token(),
                "est_cost": est_cost,
            })
        }
        ReasonKind::CostUnmigratedFacet => {
            serde_json::json!({
                "kind": reason.cost_source_token(),
            })
        }
        ReasonKind::CostProjected {
            est_cost,
            lower,
            upper,
            human,
            agent,
        } => serde_json::json!({
            "kind": "cost_projected",
            "est_cost": est_cost,
            "lower": lower,
            "upper": upper,
            "human": human,
            "agent": agent,
        }),
        ReasonKind::CostBareAnchor {
            est_cost,
            max_estimate,
            margin,
        } => serde_json::json!({
            "kind": "cost_bare_anchor",
            "est_cost": est_cost,
            "max_estimate": max_estimate,
            "margin": margin,
        }),
        ReasonKind::CostGauge {
            est_cost,
            max_estimate,
            margin,
            judgements,
        } => serde_json::json!({
            "kind": "cost_gauge",
            "est_cost": est_cost,
            "max_estimate": max_estimate,
            "margin": margin,
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
        "cost_source": ex.cost_source.as_ref().map(reason_json),
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
    use crate::listing::ABSENT_CELL;
    use crate::priority::view::Actionability;

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
            cost_source: None,
            value_source: None,
            tags: vec![],
        }
    }

    /// A resolved human-claim value source (renders bare — the canonical rung-1
    /// evidence, the retired authored-facet convention).
    fn human_value(val: f64) -> ReasonKind {
        ReasonKind::ValueClaim {
            value: val,
            tier: ClaimTier::Human,
            conflict: vec![],
            by: None,
            date: None,
            contested: None,
        }
    }

    /// Build a NextRow with facets.
    fn faceted_row(id: &str, _lo: f64, _hi: f64, val: f64, tags: &[&str]) -> NextRow {
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
            cost_source: None,
            value_source: Some(human_value(val)),
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
        let mut row = faceted_row("ISS-001", 3.2, 4.8, 5.0, &[]);
        row.cost_source = Some(ReasonKind::CostClaim {
            est_cost: 4.0,
            lower: 3.2,
            upper: 4.8,
            beta: 0.65,
            tier: crate::comparison::ClaimTier::Human,
            by: None,
            date: None,
            conflict: Vec::new(),
        });
        let out = next_human(&nv(&[row]), RenderOpts::default(), None, 20, 0, false).unwrap();
        assert!(out.contains(" 3.2 -  4.8"), "fractional estimate: {out}");
    }

    #[test]
    fn vt4_format_bound_estimate_integral() {
        let mut row = faceted_row("ISS-001", 3.0, 8.0, 5.0, &[]);
        row.cost_source = Some(ReasonKind::CostClaim {
            est_cost: 6.25,
            lower: 3.0,
            upper: 8.0,
            beta: 0.65,
            tier: crate::comparison::ClaimTier::Human,
            by: None,
            date: None,
            conflict: Vec::new(),
        });
        let out = next_human(&nv(&[row]), RenderOpts::default(), None, 20, 0, false).unwrap();
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
        let mut r = bare_row("ISS-001"); // kind ISS is value-bearing, no value source
        assert_eq!(
            value_cell(&r),
            format!("{}{DEFAULT_VALUE_MARKER}", format_bound(DEFAULT_VALUE))
        );
        // A resolved human claim (rung 1) renders bare — the canonical evidence.
        r.value_source = Some(human_value(2.0));
        assert_eq!(value_cell(&r), format_bound(2.0));
    }

    /// SL-220 PHASE-06: each ladder rung marks its cell distinctly (design §6).
    #[test]
    fn value_cell_marks_each_source_rung() {
        let mut r = bare_row("ISS-001");
        r.value_source = Some(ReasonKind::ValuePin {
            value: 6.5,
            conflict: vec![],
            by: None,
            date: None,
            basis: None,
            contested: None,
        });
        assert_eq!(
            value_cell(&r),
            format!("{}{VALUE_MARKER_PIN}", format_bound(6.5))
        );
        r.value_source = Some(ReasonKind::ValueClaim {
            value: 3.0,
            tier: ClaimTier::Agent,
            conflict: vec![],
            by: None,
            date: None,
            contested: None,
        });
        assert_eq!(
            value_cell(&r),
            format!("{}{VALUE_MARKER_AGENT}", format_bound(3.0))
        );
        r.value_source = Some(ReasonKind::ValueProjected {
            value: 4.2,
            lower: Some(3.0),
            upper: Some(5.0),
            human: 2,
            agent: 1,
        });
        assert_eq!(
            value_cell(&r),
            format!("{}{VALUE_MARKER_PROJECTED}", format_bound(4.2))
        );
    }

    #[test]
    fn value_cell_is_absent_for_valueless_kind_without_facet() {
        // A valueless kind (governance/REV/records) contributes no value to the
        // score, so an absent value stays ABSENT_CELL — the default marker would
        // misrepresent it.
        let mut r = bare_row("REV-001");
        r.kind = "REV".to_string();
        r.value_source = None;
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

    /// SL-220 PHASE-06: the `ClaimConflict` finding renders (contested framing
    /// for an anchored tier) and its JSON carries the domain-tagged payload —
    /// render + JSON parity (design §6).
    #[test]
    fn claim_conflict_finding_renders_and_json_parity() {
        let human = Finding::ClaimConflict {
            domain: ComparisonDomain::Value,
            item: "SL-118".to_string(),
            tier: ClaimTier::Human,
            low: 4.0,
            high: 8.0,
            rows: 2,
        };
        let line = finding_line(&human);
        assert!(
            line.contains("SL-118 contested human claim")
                && line.contains("2 claims span interval (4.0 ‥ 8.0)")
                && line.contains("resolve by superseding row"),
            "human contested render: {line}"
        );
        let v = finding_json(&human);
        assert_eq!(v["kind"], "claim conflicts");
        assert_eq!(v["domain"], "value");
        assert_eq!(v["tier"], "human-claim");
        assert_eq!(v["low"], 4.0);
        assert_eq!(v["rows"], 2);
        assert_eq!(v["magnitude"], 2.0);

        // An agent-tier conflict routes to comparison, never the human queue.
        let agent = Finding::ClaimConflict {
            domain: ComparisonDomain::Value,
            item: "SL-120".to_string(),
            tier: ClaimTier::Agent,
            low: 1.0,
            high: 3.0,
            rows: 2,
        };
        let agent_line = finding_line(&agent);
        assert!(
            agent_line.contains("agent claim conflict")
                && agent_line.contains("calibrate via comparison")
                && !agent_line.contains("contested"),
            "agent conflict routes to comparison (D14): {agent_line}"
        );
    }

    /// SL-222 PHASE-07: estimate-domain claim conflict render carries the
    /// [estimate] tag prefix (D9 parity with value domain).
    #[test]
    fn claim_conflict_finding_estimate_domain_parity() {
        let est_conflict = Finding::ClaimConflict {
            domain: ComparisonDomain::Estimate,
            item: "SL-100".to_string(),
            tier: ClaimTier::Human,
            low: 4.0,
            high: 8.0,
            rows: 2,
        };
        let line = finding_line(&est_conflict);
        assert!(
            line.contains("[estimate] SL-100 contested human claim"),
            "estimate domain line has [estimate] tag: {line}"
        );
        assert!(
            line.contains("resolve by superseding row"),
            "anchored tier has contested framing: {line}"
        );

        let v = finding_json(&est_conflict);
        assert_eq!(v["domain"], "estimate", "JSON domain is estimate");
        assert_eq!(v["item"], "SL-100");
        assert_eq!(v["tier"], "human-claim");
        assert_eq!(v["magnitude"], 2.0);
    }

    /// SL-222 PHASE-09: estimate-domain UnmigratedFacet render carries the
    /// [estimate] tag prefix; the finding is magnitude-free.
    #[test]
    fn unmigrated_facet_finding_estimate_domain_magnitude_free() {
        let est_facet = Finding::UnmigratedFacet {
            domain: ComparisonDomain::Estimate,
            entity: "ISS-007".to_string(),
        };
        let line = finding_line(&est_facet);
        assert!(line.contains("[estimate]"), "estimate tagged: {line}");
        assert!(
            line.contains("facet no longer read"),
            "unread message: {line}"
        );

        let v = finding_json(&est_facet);
        assert_eq!(v["domain"], "estimate");
        assert_eq!(v["entity"], "ISS-007");
        assert!(v.get("value").is_none(), "PHAE-09: magnitude omitted");
    }

    /// SL-222 PHASE-09: value-domain UnmigratedFacet has NO [estimate] tag
    /// (untagged); the finding is magnitude-free.
    #[test]
    fn unmigrated_facet_finding_value_domain_magnitude_free() {
        let val_facet = Finding::UnmigratedFacet {
            domain: ComparisonDomain::Value,
            entity: "SL-100".to_string(),
        };
        let line = finding_line(&val_facet);
        assert!(
            !line.contains("[estimate]"),
            "value domain unmigrated facet is untagged: {line}"
        );

        let v = finding_json(&val_facet);
        assert_eq!(v["domain"], "value");
        assert!(v.get("value").is_none(), "PHASE-09: magnitude omitted");
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

    // ── SL-218 PHASE-03: the four design §3 wording samples (VT-E) ────────────
    // The render source of truth (REQ-072 AC3) — `reason_line` over a
    // `ReasonKind::Tension` must produce each sample byte-exact. Pinned here as
    // pure unit tests (the `needs`/agent-proposed determinacy states are awkward
    // to reach through a black-box corpus; the e2e goldens cover the reachable
    // integration paths). Constraints: `value_dim` named (SL-217 D5), no
    // stability/membership language (D15), grade + counts always present (D6).

    fn structure(edge_from: &str, verb: EdgeVerb, grade: TensionGradeView) -> ReasonKind {
        ReasonKind::Tension {
            preferred: "SL-014".to_string(),
            surfaced: "SL-009".to_string(),
            cause: TensionCauseView::Structure {
                edge_from: edge_from.to_string(),
                verb,
            },
            grade,
        }
    }

    #[test]
    fn tension_sample_structure_determined() {
        let r = structure(
            "SL-009",
            EdgeVerb::After,
            TensionGradeView::Determined { human: 3, agent: 0 },
        );
        assert_eq!(
            reason_line(&r),
            "  tension: SL-014 ranks above SL-009 on value_dim (determined — 3 human \
             judgements); SL-009 surfaces first — `after SL-009` sequence survives.\n"
        );
    }

    #[test]
    fn tension_sample_structure_projected() {
        let r = structure("SL-009", EdgeVerb::Needs, TensionGradeView::Projected);
        assert_eq!(
            reason_line(&r),
            "  tension: SL-014 ranks above SL-009 on value_dim (projected order — no \
             determining evidence); SL-009 surfaces first — `needs SL-009` holds.\n"
        );
    }

    #[test]
    fn tension_sample_structure_agent_proposed() {
        let r = structure(
            "SL-009",
            EdgeVerb::After,
            TensionGradeView::AgentProposed { agent: 4 },
        );
        assert_eq!(
            reason_line(&r),
            "  tension: SL-014 ranks above SL-009 on value_dim (agent-proposed — 4 agent \
             judgements, unconfirmed); SL-009 surfaces first — `after SL-009` sequence \
             survives.\n"
        );
    }

    #[test]
    fn tension_sample_composition() {
        let r = ReasonKind::Tension {
            preferred: "SL-014".to_string(),
            surfaced: "SL-009".to_string(),
            cause: TensionCauseView::Composition {
                risk_dim: 0.8,
                leverage: 2.1,
                optionality: 0.0,
            },
            grade: TensionGradeView::Determined { human: 2, agent: 1 },
        };
        assert_eq!(
            reason_line(&r),
            "  SL-009 surfaces above SL-014 on full score (leverage +2.1, risk +0.8); on \
             value_dim alone SL-014 ranks higher (determined — 2 human + 1 agent).\n"
        );
    }

    #[test]
    fn tension_json_schema_structure_and_composition() {
        // The design §3 JSON schema: {preferred, surfaced, cause, edge?|deltas?, grade, counts?}.
        let s = tension_json(&structure(
            "SL-009",
            EdgeVerb::After,
            TensionGradeView::Determined { human: 3, agent: 0 },
        ))
        .unwrap();
        assert_eq!(s["preferred"], "SL-014");
        assert_eq!(s["surfaced"], "SL-009");
        assert_eq!(s["cause"], "structure");
        assert_eq!(s["edge"]["from"], "SL-009");
        assert_eq!(s["edge"]["verb"], "after");
        assert_eq!(s["grade"], "determined");
        assert_eq!(s["counts"]["human"], 3);
        assert_eq!(s["counts"]["agent"], 0);
        assert!(s.get("deltas").is_none(), "structure carries no deltas");

        let c = tension_json(&ReasonKind::Tension {
            preferred: "SL-014".to_string(),
            surfaced: "SL-009".to_string(),
            cause: TensionCauseView::Composition {
                risk_dim: 0.8,
                leverage: 2.1,
                optionality: 0.0,
            },
            grade: TensionGradeView::AgentProposed { agent: 4 },
        })
        .unwrap();
        assert_eq!(c["cause"], "composition");
        assert_eq!(c["deltas"]["leverage"], 2.1);
        assert_eq!(c["grade"], "agent_proposed");
        assert_eq!(c["counts"]["agent"], 4);
        assert!(c.get("edge").is_none(), "composition carries no edge");
    }

    #[test]
    fn tension_projected_json_omits_counts() {
        let p = tension_json(&structure(
            "SL-009",
            EdgeVerb::After,
            TensionGradeView::Projected,
        ))
        .unwrap();
        assert_eq!(p["grade"], "projected");
        assert!(p.get("counts").is_none(), "projected has no counts");
    }

    #[test]
    fn zero_weight_disclosure_line_singular_and_plural() {
        // F-6 scoped disclosure (SL-217 D6 wording); singular for count == 1.
        assert_eq!(
            reason_line(&ReasonKind::ZeroWeightExcluded { count: 3 }),
            "  3 pairs value-insensitive, zero weight\n"
        );
        assert_eq!(
            reason_line(&ReasonKind::ZeroWeightExcluded { count: 1 }),
            "  1 pair value-insensitive, zero weight\n"
        );
    }

    // ── §8.8 cost-source render fragments ────────────────────────────────────

    /// Helper: create a CostPin ReasonKind.
    fn cost_pin(
        est_cost: f64,
        contested: bool,
        by: Option<&str>,
        date: Option<&str>,
    ) -> ReasonKind {
        ReasonKind::CostPin {
            est_cost,
            lower: est_cost * 0.8,
            upper: est_cost * 1.2,
            beta: 0.65,
            by: by.map(String::from),
            date: date.map(String::from),
            basis: None,
            contested: contested.then(|| ContestedClaim {
                low: est_cost * 0.7,
                high: est_cost * 1.3,
                rows: 2,
            }),
        }
    }

    /// Helper: create a CostClaim ReasonKind.
    fn cost_claim(
        est_cost: f64,
        tier: ClaimTier,
        by: Option<&str>,
        date: Option<&str>,
    ) -> ReasonKind {
        ReasonKind::CostClaim {
            est_cost,
            lower: est_cost * 0.8,
            upper: est_cost * 1.2,
            beta: 0.65,
            tier,
            by: by.map(String::from),
            date: date.map(String::from),
            conflict: Vec::new(),
        }
    }

    #[test]
    fn cost_source_fragment_pin_singleton() {
        let r = cost_pin(6.0, false, Some("david"), Some("2026-07-17"));
        let frag = cost_source_fragment(&r).unwrap();
        assert_eq!(
            frag,
            "est_cost 6.0 — pin [4.8 ‥ 7.2] · β 0.65 (david, 2026-07-17)"
        );
    }

    #[test]
    fn cost_source_fragment_pin_no_attribution() {
        let r = cost_pin(2.5, false, None, None);
        let frag = cost_source_fragment(&r).unwrap();
        assert_eq!(frag, "est_cost 2.5 — pin [2.0 ‥ 3.0] · β 0.65");
    }

    #[test]
    fn cost_source_fragment_pin_contested() {
        let r = cost_pin(6.0, true, Some("david"), Some("2026-07-17"));
        let frag = cost_source_fragment(&r).unwrap();
        assert!(
            frag.contains("contested") && frag.contains("resolve by superseding row"),
            "contested cost pin: {frag}"
        );
    }

    #[test]
    fn cost_source_fragment_human_claim() {
        let r = cost_claim(4.0, ClaimTier::Human, Some("ada"), Some("2026-07-16"));
        let frag = cost_source_fragment(&r).unwrap();
        assert_eq!(
            frag,
            "est_cost 4.0 — human claim [3.2 ‥ 4.8] · β 0.65 (ada, 2026-07-16)"
        );
    }

    #[test]
    fn cost_source_fragment_agent_claim() {
        let r = cost_claim(3.0, ClaimTier::Agent, Some("bot"), None);
        let frag = cost_source_fragment(&r).unwrap();
        assert_eq!(
            frag,
            "est_cost 3.0 — agent claim [2.4 ‥ 3.6] · β 0.65 (bot)"
        );
    }

    #[test]
    fn cost_source_fragment_migrated_claim() {
        let r = cost_claim(5.0, ClaimTier::Migrated, None, Some("2026-06-01"));
        let frag = cost_source_fragment(&r).unwrap();
        assert_eq!(
            frag,
            "est_cost 5.0 — migrated claim [4.0 ‥ 6.0] · β 0.65 (unattributed, observed 2026-06-01)"
        );
    }

    #[test]
    fn cost_source_fragment_class_anchor() {
        let r = ReasonKind::CostClassAnchor { est_cost: 3.5 };
        let frag = cost_source_fragment(&r).unwrap();
        assert_eq!(frag, "est_cost 3.5 — anchored (via class anchor)");
    }

    #[test]
    fn cost_source_fragment_unmigrated_facet() {
        let r = ReasonKind::CostUnmigratedFacet;
        let frag = cost_source_fragment(&r).unwrap();
        assert!(
            frag.contains("unmigrated [estimate] facet"),
            "unmigrated facet: {frag}"
        );
        assert!(
            frag.contains("scripts/migrate_estimate_facets.py"),
            "migration hint: {frag}"
        );
    }

    #[test]
    fn cost_source_fragment_projected() {
        let r = ReasonKind::CostProjected {
            est_cost: 5.5,
            lower: Some(3.0),
            upper: Some(8.0),
            human: 2,
            agent: 1,
        };
        let frag = cost_source_fragment(&r).unwrap();
        assert!(
            frag.contains("est_cost 5.5 — projected · bounds (3.0 ‥ 8.0) · from 3 constraining sizing judgements (2 human, 1 agent)"),
            "projected: {frag}"
        );
    }

    #[test]
    fn cost_source_fragment_bare_anchor_with_max() {
        let r = ReasonKind::CostBareAnchor {
            est_cost: 11.0,
            max_estimate: Some(10.0),
            margin: 1.0,
        };
        let frag = cost_source_fragment(&r).unwrap();
        assert_eq!(
            frag,
            "est_cost 11.0 — bare anchor (max estimate 10.0 + margin 1.0)"
        );
    }

    #[test]
    fn cost_source_fragment_bare_anchor_default() {
        let r = ReasonKind::CostBareAnchor {
            est_cost: 1.0,
            max_estimate: None,
            margin: 0.0,
        };
        let frag = cost_source_fragment(&r).unwrap();
        assert!(
            frag.contains("no estimate in corpus; default 1.0"),
            "bare anchor default: {frag}"
        );
    }

    #[test]
    fn cost_source_fragment_gauge() {
        let r = ReasonKind::CostGauge {
            est_cost: 11.0,
            max_estimate: Some(10.0),
            margin: 1.0,
            judgements: 3,
        };
        let frag = cost_source_fragment(&r).unwrap();
        assert!(
            frag.contains("gauge · ordered by 3 judgements"),
            "gauge: {frag}"
        );
    }

    #[test]
    fn cost_source_fragment_none_for_value_reason() {
        let r = ReasonKind::ValuePin {
            value: 1.0,
            conflict: vec![],
            by: None,
            date: None,
            basis: None,
            contested: None,
        };
        assert!(
            cost_source_fragment(&r).is_none(),
            "value reason yields None"
        );
    }

    // ── show_cost_render tests (§8.8 show-line helper) ───────────────────

    #[test]
    fn show_cost_render_pin_with_attribution() {
        let r = cost_pin(6.0, false, Some("david"), Some("2026-07-17"));
        let line = show_cost_render(&r, "espresso_shots", None).unwrap();
        assert_eq!(
            line,
            "estimate: 4.8–7.2 espresso_shots (pin, david, 2026-07-17)"
        );
    }

    #[test]
    fn show_cost_render_human_claim() {
        let r = cost_claim(4.0, ClaimTier::Human, Some("ada"), Some("2026-07-16"));
        let line = show_cost_render(&r, "espresso_shots", None).unwrap();
        assert_eq!(
            line,
            "estimate: 3.2–4.8 espresso_shots (human claim, ada, 2026-07-16)"
        );
    }

    #[test]
    fn show_cost_render_class_anchor() {
        let r = ReasonKind::CostClassAnchor { est_cost: 3.5 };
        let line = show_cost_render(&r, "hours", None).unwrap();
        assert_eq!(line, "estimate: 3.5–3.5 hours (class anchor)");
    }

    #[test]
    fn show_cost_render_projected() {
        let r = ReasonKind::CostProjected {
            est_cost: 5.5,
            lower: Some(3.0),
            upper: Some(8.0),
            human: 2,
            agent: 1,
        };
        let line = show_cost_render(&r, "espresso_shots", None).unwrap();
        assert_eq!(line, "estimate: 3.0–8.0 espresso_shots (projected)");
    }

    #[test]
    fn show_cost_render_bare_anchor() {
        let r = ReasonKind::CostBareAnchor {
            est_cost: 11.0,
            max_estimate: Some(10.0),
            margin: 1.0,
        };
        let line = show_cost_render(&r, "espresso_shots", None).unwrap();
        assert_eq!(line, "estimate: 11.0–11.0 espresso_shots (bare anchor)");
    }

    #[test]
    fn show_cost_render_gauge() {
        let r = ReasonKind::CostGauge {
            est_cost: 11.0,
            max_estimate: Some(10.0),
            margin: 1.0,
            judgements: 3,
        };
        let line = show_cost_render(&r, "espresso_shots", None).unwrap();
        assert_eq!(line, "estimate: 11.0–11.0 espresso_shots (gauge)");
    }

    #[test]
    fn show_cost_render_scoring_inert() {
        let r = cost_claim(4.0, ClaimTier::Human, Some("ada"), Some("2026-07-16"));
        let line = show_cost_render(&r, "espresso_shots", Some("REC")).unwrap();
        assert_eq!(
            line,
            "estimate: 3.2–4.8 espresso_shots (human claim, ada, 2026-07-16) — scoring-inert (REC kind)"
        );
    }

    #[test]
    fn show_cost_render_none_for_absent_evidence() {
        // A non-cost-source reason yields None (line omitted).
        let r = ReasonKind::Score {
            base: 1.0,
            value_dim: 1.0,
            risk_dim: 0.0,
            leverage: 1.0,
            optionality: 0.0,
            total: 2.0,
        };
        assert!(show_cost_render(&r, "espresso_shots", None).is_none());
    }

    #[test]
    fn show_cost_render_none_for_no_cost_source_reason() {
        // Even a value-source reason returns None (not a cost source).
        let r = ReasonKind::ValuePin {
            value: 1.0,
            conflict: vec![],
            by: None,
            date: None,
            basis: None,
            contested: None,
        };
        assert!(show_cost_render(&r, "espresso_shots", None).is_none());
    }

    // ── show_cost_provenance tests ──────────────────────────────────────

    #[test]
    fn show_cost_provenance_unmigrated_facet() {
        let r = ReasonKind::CostUnmigratedFacet;
        assert_eq!(
            show_cost_provenance(&r).unwrap(),
            "unmigrated [estimate] facet"
        );
    }

    #[test]
    fn show_cost_provenance_class_anchor() {
        let r = ReasonKind::CostClassAnchor { est_cost: 3.5 };
        assert_eq!(show_cost_provenance(&r).unwrap(), "class anchor");
    }

    #[test]
    fn show_cost_provenance_none_for_value_reason() {
        let r = ReasonKind::ValueProjected {
            value: 5.5,
            lower: Some(3.0),
            upper: Some(8.0),
            human: 2,
            agent: 1,
        };
        assert!(show_cost_provenance(&r).is_none());
    }
}
