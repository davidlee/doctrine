// SPDX-License-Identifier: GPL-3.0-only
//! `review show`/`list`/`status` — the read-only projections (SL-268 PHASE-02 T5).

use super::prime::{CacheVerdict, cache_staleness, read_cache};
use super::turn::{
    Baton, LockGuard, read_baton, reconcile_baton_fields, resolve_review_root, write_baton,
};
use super::{
    Await, Column, Context, Facet, FindingRow, Format, ListArgs, Path, PathBuf, REVIEW_DIR,
    REVIEW_KIND, REVIEW_STATUSES, ReviewDoc, ReviewOutput, ReviewStatus, ReviewWarning, Serialize,
    Severity, Target, UnreadableReview, Vocab, canonical_id, counters, derived_status,
    finding_states_of, fs, listing, parse_ref, read_authored, read_review, scan_reviews,
    warnings_of,
};
use crate::review_ledger::{
    Depth, FindingView, Outstanding, OutstandingCounts, Projection, ReviewView, TurnRow,
};
use crate::tomlfmt::toml_string;

// ---------------------------------------------------------------------------
// Render (pure — user free-text spliced via `toml_string`, design §4/§5)
// ---------------------------------------------------------------------------

/// Render a single `[[finding]]` TOML block (design §5), from the authored
/// row (SL-278 PHASE-02b T11 — the `--json`/MCP typed [`Finding`] this once
/// read is gone; the raw strings on [`FindingRow`] carry the same values).
/// Every user free-text field (`title`, `detail`, `disposition`, `response`)
/// is emitted through `toml_string` so a `"`, `\`, newline, or `]` can
/// neither break the document nor inject a key
/// (mem.pattern.render.toml-splice-escape-user-values). The id/status/
/// severity fields are closed vocabularies, rendered bare.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by the review drift canaries / tests (SL-268 D4 split)"
    )
)]
pub(crate) fn render_finding(finding: &FindingRow) -> String {
    let mut out = String::new();
    out.push_str("[[finding]]\n");
    push_line(&mut out, "id", &toml_string(&finding.id));
    push_line(&mut out, "status", &toml_string(&finding.status));
    push_line(&mut out, "severity", &toml_string(&finding.severity));
    push_line(&mut out, "title", &toml_string(&finding.title));
    push_line(&mut out, "detail", &toml_string(&finding.detail));
    if let Some(disposition) = &finding.disposition {
        push_line(&mut out, "disposition", &toml_string(disposition));
    }
    if let Some(response) = &finding.response {
        push_line(&mut out, "response", &toml_string(response));
    }
    out
}

/// Emit a single `key = value` line (no `push`/`format!` of literals — repo
/// clippy bans the noisy forms; this is the sanctioned string-assembly shape).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by the review drift canaries / tests (SL-268 D4 split)"
    )
)]
fn push_line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(value);
    out.push('\n');
}

/// The `reviews`-edge line shown in `show` and as the `target` column: the
/// outbound edge to the subject, with an optional `@phase` scope (ADR-004).
/// Delegates to the engine-tier [`Target::label`] (SL-278 PHASE-01 A1) — no
/// second copy.
pub(super) fn edge_label(doc: &ReviewDoc) -> String {
    doc.target.label()
}

/// `doctrine review show <RV-NNN>` — read the RV and build its [`ReviewView`]
/// ONCE under `projection` (SL-278 design sec-2), then render the readable
/// table (`Table`) or the one JSON envelope (`Json`), both off the same view
/// (design sec-3/sec-4, B10). The status is DERIVED (review never asks the
/// shared reader for a stored status, D-C8). The baton is read, never locked
/// or written: it supplies only the legacy round counters.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    projection: &Projection,
    format: Format,
) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let review_root = root.join(REVIEW_DIR);
    let id = parse_ref(reference)?;
    let doc = read_review(&review_root, id)?;
    let body = read_brief(&review_root, id)?;
    let baton = read_baton(&root, id)?.unwrap_or_default();
    let view = ReviewView::build(&doc, &body, (baton.rounds, baton.contests), projection)?;
    let formatted = match format {
        Format::Table => {
            let cfg = crate::dtoml::load_doctrine_toml(&root)?;
            let estimation_unit = crate::estimate::resolve_unit(&cfg.estimation);
            let value_unit = crate::value::resolve_unit(&cfg.value);
            let (lower_pct, upper_pct) = crate::estimate::resolve_confidence(&cfg.estimation)?;
            let value_line = crate::priority::surface::show_value_line(
                &root,
                &canonical_id(id),
                REVIEW_KIND.prefix,
                &value_unit,
            )?;
            let estimate_line = crate::priority::surface::show_estimate_line(
                &root,
                &canonical_id(id),
                REVIEW_KIND.prefix,
                &estimation_unit,
            )?;
            format_show(
                &view,
                &estimation_unit,
                value_line.as_deref(),
                estimate_line.as_deref(),
                lower_pct,
                upper_pct,
            )
        }
        // SL-278 PHASE-02b (design sec-4, B10): the one JSON shape, straight off
        // the view — no second, hand-mirrored `ShowJson` row.
        Format::Json => serde_json::to_string_pretty(&serde_json::json!({
            "kind": "review",
            "review": &view,
        }))
        .context("failed to serialize review show JSON")?,
    };
    Ok(ReviewOutput::Showed { view, formatted })
}

/// Read the `review-NNN.md` brief body (the prose companion).
pub(super) fn read_brief(review_root: &Path, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = review_root.join(&name).join(format!("review-{name}.md"));
    fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

/// The text-channel rendering of a set of warnings: one line each.
fn warning_lines(warnings: &[ReviewWarning]) -> String {
    warnings.iter().map(ReviewWarning::line).collect()
}

/// The status-line suffix for the pass-level concluded marker — shared by
/// `show` and `status`. It rides the line only when set: silence is the far
/// commoner state, and a `concluded=no` on every unconcluded pass would be
/// noise on the line a reader scans for the derived state.
fn concluded_marker(concluded: bool) -> &'static str {
    if concluded { " · concluded" } else { "" }
}

/// The em-dash rendered in the index's `disposition` column when a finding
/// carries none (it is set only by `dispose`).
pub(super) const NO_DISPOSITION: &str = "—";

/// A severity as a table cell (SL-278 DEC-367; the census reuses it): the two
/// gating-weight tokens in capitals, through the known variant — so an
/// out-of-vocabulary lookalike is never capitalised — and anything else
/// verbatim. JSON always carries the lowercase token.
pub(super) fn severity_cell(severity: &Vocab<Severity>) -> String {
    match severity.known() {
        Some(Severity::Blocker) => "BLOCKER".to_owned(),
        Some(Severity::Major) => "MAJOR".to_owned(),
        _ => severity.as_str().to_owned(),
    }
}

/// Render the finding index (RFC-032 D5, SL-278 sec-3): one row per finding —
/// `id │ severity │ status │ disposition │ [route │] title`, the `route` column
/// only when a shown finding has one. Rendered through the shared
/// `listing::render_table` (not bespoke formatting) so colour/width policy and
/// the census's column machinery stay in one place. No findings render
/// nothing (the count line stands alone).
pub(super) fn render_finding_index(findings: &[FindingView]) -> String {
    if findings.is_empty() {
        return String::new();
    }
    let routed = findings.iter().any(|f| f.route.is_some());
    let mut header = vec!["id", "severity", "status", "disposition"];
    if routed {
        header.push("route");
    }
    header.push("title");
    let mut grid: Vec<Vec<String>> = Vec::with_capacity(findings.len() + 1);
    grid.push(header.into_iter().map(str::to_owned).collect());
    grid.extend(findings.iter().map(|f| {
        let mut row = vec![
            f.id.clone(),
            severity_cell(&f.severity),
            f.status.as_str().to_owned(),
            f.disposition
                .clone()
                .unwrap_or_else(|| NO_DISPOSITION.to_owned()),
        ];
        if routed {
            row.push(f.route.clone().unwrap_or_default());
        }
        row.push(f.title.clone());
        row
    }));
    listing::render_table(&grid, None)
}

/// `text` with every line prefixed by `indent` and newline-terminated; a blank
/// line stays empty (no trailing indent). Indentation is the only change made
/// to prose — nothing is wrapped or cut (DEC-366).
fn indented(text: &str, indent: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                "\n".to_owned()
            } else {
                [indent, line, "\n"].concat()
            }
        })
        .collect()
}

/// One finding at full depth (SL-278 sec-3): the header line, the title, then
/// `detail`/`response`/`turns` — each only when set — with prose verbatim at
/// four spaces and a turn's note/response at six.
fn render_finding_block(finding: &FindingView) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut head = vec![
        finding.id.clone(),
        severity_cell(&finding.severity),
        finding.status.as_str().to_owned(),
    ];
    if let Some(disposition) = &finding.disposition {
        head.push(format!("disposition {disposition}"));
    }
    if let Some(route) = &finding.route {
        head.push(format!("route {route}"));
    }
    parts.push(format!("{}\n  {}\n\n", head.join(" · "), finding.title));
    if let Some(detail) = &finding.detail {
        parts.push(format!("  detail:\n{}", indented(detail, "    ")));
    }
    if let Some(response) = &finding.response {
        parts.push(format!("  response:\n{}", indented(response, "    ")));
    }
    if !finding.turns.is_empty() {
        parts.push("  turns:\n".to_owned());
        parts.extend(finding.turns.iter().map(render_turn));
    }
    parts.concat()
}

/// One journal turn of a full block: `act · role[ · disposition][ → route]` at
/// four spaces, then its note and response verbatim at six.
fn render_turn(turn: &TurnRow) -> String {
    let mut head = vec![turn.act.as_str(), turn.role.as_str()];
    if let Some(disposition) = &turn.disposition {
        head.push(disposition);
    }
    let route = turn
        .route
        .as_ref()
        .map_or_else(String::new, |route| format!(" → {route}"));
    let mut parts = vec![format!("    {}{route}\n", head.join(" · "))];
    parts.extend(
        [&turn.note, &turn.response]
            .into_iter()
            .flatten()
            .map(|prose| indented(prose, "      ")),
    );
    parts.concat()
}

/// `outstanding: …` — the non-terminal findings grouped by the party they
/// await, responder first, each group its non-zero severities in vocabulary
/// order with a verb agreeing with the group total; `none` when nothing is
/// owed (SL-278 sec-3).
fn outstanding_line(outstanding: &Outstanding) -> String {
    let groups: Vec<String> = [
        (Await::Responder, &outstanding.responder),
        (Await::Raiser, &outstanding.raiser),
    ]
    .into_iter()
    .filter_map(|(party, counts)| outstanding_group(party, counts))
    .collect();
    if groups.is_empty() {
        "outstanding: none\n".to_owned()
    } else {
        format!("outstanding: {}\n", groups.join(" · "))
    }
}

/// One party's group of the outstanding line, or `None` when it owes nothing.
fn outstanding_group(party: Await, counts: &OutstandingCounts) -> Option<String> {
    let owed: Vec<(Severity, usize)> = counts
        .by_severity()
        .into_iter()
        .filter(|&(_, n)| n > 0)
        .collect();
    if owed.is_empty() {
        return None;
    }
    let cells: Vec<String> = owed
        .iter()
        .map(|&(severity, n)| format!("{n} {}", severity_cell(&Vocab::Known(severity))))
        .collect();
    let total: usize = owed.iter().map(|&(_, n)| n).sum();
    let verb = if total > 1 { "await" } else { "awaits" };
    Some(format!("{} {verb} {}", cells.join(", "), party.as_str()))
}

/// The hint that stands in for the brief body (SL-278 sec-3): its size and
/// the two invocations that reach detail, naming the review's own id.
fn brief_hint(canonical: &str, body_lines: usize) -> String {
    let size = match body_lines {
        0 => "empty".to_owned(),
        1 => "1 line".to_owned(),
        n => format!("{n} lines"),
    };
    format!(
        "brief: {size} — doctrine review show {canonical} --full · -f F-n for chosen findings\n"
    )
}

/// Render the `Table` show from the view (SL-278 sec-3): identity, the derived
/// status line, the `reviews` edge, the parties, what is outstanding, the
/// count, then the index (or, at full depth, one block per finding), the
/// warnings, tags, estimate and value — and last the brief hint, or the body
/// after a blank line when the view carries it. House style — `Vec<String>`
/// joined by `concat` (avoids the `push_str(&format!)` lint).
pub(super) fn format_show(
    view: &ReviewView,
    _estimation_unit: &str,
    value_line: Option<&str>,
    estimate_line: Option<&str>,
    _lower_pct: f64,
    _upper_pct: f64,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", view.canonical, view.title));
    parts.push(format!(
        "{} · {} · await={} · rounds {}{}\n",
        view.facet,
        view.status.as_str(),
        view.awaiting.as_str(),
        view.rounds,
        concluded_marker(view.concluded)
    ));
    parts.push(format!("{} ──reviews──▶ {}\n", view.canonical, view.target));
    parts.push(format!(
        "raiser {} · responder {}\n",
        view.raiser, view.responder
    ));
    parts.push(outstanding_line(&view.outstanding));
    let shown = view.findings.len();
    parts.push(if shown == view.finding_count {
        format!("findings: {shown}\n")
    } else {
        format!("findings: {shown} of {}\n", view.finding_count)
    });
    parts.push(match view.depth {
        Depth::Index => render_finding_index(&view.findings),
        Depth::Full => view
            .findings
            .iter()
            .map(render_finding_block)
            .collect::<Vec<_>>()
            .join("\n"),
    });
    parts.push(warning_lines(&view.warnings));
    if !view.tags.is_empty() {
        parts.push(format!("tags: {}\n", view.tags.join(", ")));
    }
    // SL-222 PHASE-07: pipeline-resolved estimate line (facet fallback deleted PHASE-09).
    if let Some(line) = estimate_line {
        parts.push(format!("{line}\n"));
    }
    // SL-220 PHASE-06: the value line re-sources from the ladder (design §6).
    if let Some(line) = value_line {
        parts.push(format!("{line}\n"));
    }
    parts.push(match &view.body {
        Some(body) => format!("\n{body}"),
        None => brief_hint(&view.canonical, view.body_lines.unwrap_or(0)),
    });
    parts.concat()
}

/// Attach a non-empty array under `key` to a JSON object envelope (absent
/// when empty — the clean-ledger shape stays byte-identical). Generalised
/// (SL-278 PHASE-04a) so `warnings` and `unreadable` share one "serialize +
/// skip-if-empty" helper rather than each hand-rolling it.
pub(super) fn with_array<T: Serialize>(
    value: &mut serde_json::Value,
    key: &str,
    items: &[T],
) -> anyhow::Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    let items =
        serde_json::to_value(items).with_context(|| format!("failed to serialize review {key}"))?;
    if let Some(object) = value.as_object_mut() {
        object.insert(key.to_owned(), items);
    }
    Ok(())
}

/// The `review list` row tuple: the doc plus its derived status (computed once).
type ReviewRow = (ReviewDoc, ReviewStatus, Await);

pub(super) const REVIEW_COLUMNS: [Column<ReviewRow>; 6] = [
    Column {
        name: "id",
        header: "id",
        cell: |(d, _, _)| canonical_id(d.id),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    Column {
        name: "status",
        header: "status",
        cell: |(_, s, a)| {
            let mut cell = s.as_str().to_owned();
            cell.push_str(" (await ");
            cell.push_str(a.as_str());
            cell.push(')');
            cell
        },
        // ByValue reads the row's RAW derived status, NOT the emitted composite
        // `active (await …)` cell (F-4) — matching the cell text would drop colour.
        paint: listing::ColumnPaint::ByValue(|(_, s, _)| listing::status_hue(s.as_str())),
    },
    Column {
        name: "facet",
        header: "facet",
        cell: |(d, _, _)| d.review.facet.clone(),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "target",
        header: "target",
        cell: |(d, _, _)| edge_label(d),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "tags",
        header: "tags",
        cell: |(d, _, _)| d.tags.join(", "),
        paint: listing::ColumnPaint::PerToken {
            split: |(d, _, _)| d.tags.clone(),
            render: listing::paint_tag,
        },
    },
    Column {
        name: "title",
        header: "title",
        cell: |(d, _, _)| d.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
];

/// The default visible column set for `review list`.
pub(super) const REVIEW_DEFAULT: &[&str] = &["id", "status", "facet", "target", "title"];

/// A review's filterable projection (design §5 list axes). The derived status is
/// the filter status (review stores none); `canonical` is the regex domain.
fn key(d: &ReviewDoc) -> listing::FilterFields {
    let (status, _) = d.derived();
    listing::FilterFields {
        canonical: canonical_id(d.id),
        slug: d.slug.clone(),
        title: d.title.clone(),
        status: status.as_str().to_owned(),
        tags: d.tags.clone(),
    }
}

/// A review-scope predicate (design sec-5, DEC-365): an optional target ref
/// (narrowed by an optional phase scope, the `--target` spelling) and an
/// optional facet. The one place this match lives — `review list` (04a) and
/// the census (04b) both filter through [`ReviewScope::admits`] rather than
/// each growing its own copy.
pub(super) struct ReviewScope {
    target: Option<Target>,
    facet: Option<Facet>,
}

impl ReviewScope {
    /// Parse the `--target`/`--facet` pair. `target` accepts the
    /// `ref@PHASE-NN` spelling via [`Target::parse`]; `facet` is already a
    /// parsed [`Facet`] (clap/MCP validate the raw token before this point).
    pub(super) fn parse(target: Option<&str>, facet: Option<Facet>) -> anyhow::Result<Self> {
        let target = target.map(|want| Target::parse(want, None)).transpose()?;
        Ok(Self { target, facet })
    }

    /// Whether `doc` is in scope. A target ref admits any phase unless the
    /// scope's own `@PHASE-NN` narrows it (RFC-032 D5, SL-268 PHASE-07
    /// D-T3-3). A facet compares the ledger's raw `review.facet` string to
    /// the parsed facet's token — an out-of-vocabulary stored facet never
    /// matches (A2).
    pub(super) fn admits(&self, doc: &ReviewDoc) -> bool {
        let target_ok = self.target.as_ref().is_none_or(|wanted| {
            doc.target.reference == wanted.reference
                && wanted
                    .phase
                    .as_deref()
                    .is_none_or(|p| doc.target.phase.as_deref() == Some(p))
        });
        let facet_ok = self.facet.is_none_or(|f| doc.review.facet == f.as_str());
        target_ok && facet_ok
    }
}

/// What `list_rows` computes: the rendered projection, its JSON rows, the
/// closed-vocabulary warnings, and any unreadable RVs across the listed set
/// (all in id order) — a named struct rather than a growing tuple (R1),
/// generic over the row type so `review list` and the census share it.
pub(super) struct Listing<R> {
    pub(super) formatted: String,
    pub(super) rows: Vec<R>,
    pub(super) warnings: Vec<ReviewWarning>,
    pub(super) unreadable: Vec<UnreadableReview>,
}

/// `review list` rows as a string — the compute half of [`run_list`]. No hide-set
/// (an RV is either Active or Done; both are listed), sorted by id, each row
/// carrying its derived status. The listed (post-filter) RVs' vocabulary defects
/// are gathered once here, for the CLI's stderr and the MCP `warnings` field
/// alike; `--json` also carries them top-level when there are any. The scan is
/// tolerant (design sec-5 § Tolerant scan): an id that exists but does not read
/// is disclosed as `unreadable` rather than failing the whole list (STD-003);
/// unreadable RVs are never subject to `scope` (the scan can't know their
/// target/facet, A3) and are never capped (A3, mirroring `warnings`).
fn list_rows(
    root: &Path,
    mut args: ListArgs,
    target: Option<&str>,
    facet: Option<Facet>,
) -> anyhow::Result<Listing<ListRow>> {
    listing::validate_statuses(&args.status, REVIEW_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let review_root = root.join(REVIEW_DIR);
    let (docs, unreadable) = scan_reviews(&review_root)?;
    let scope = ReviewScope::parse(target, facet)?;
    let docs: Vec<ReviewDoc> = docs.into_iter().filter(|d| scope.admits(d)).collect();
    let mut docs = listing::retain(docs, &filter, |_| false, key);
    docs.sort_by_key(|d| d.id);
    let any_tagged = docs.iter().any(|d| !d.tags.is_empty());
    let warnings: Vec<ReviewWarning> = docs.iter().flat_map(warnings_of).collect();
    let rows: Vec<ReviewRow> = docs
        .into_iter()
        .map(|d| {
            let (status, awaited) = d.derived();
            (d, status, awaited)
        })
        .collect();
    let formatted = match format {
        Format::Table => {
            let effective_default = listing::default_with_tags(REVIEW_DEFAULT, any_tagged);
            let sel =
                listing::select_columns(&REVIEW_COLUMNS, &effective_default, columns.as_deref())?;
            listing::render_columns(&rows, &sel, render)
        }
        // The shared `listing::json_envelope` shape, plus `warnings`/`unreadable`
        // when any (absent when empty — the clean-ledger shape stays unchanged).
        Format::Json => {
            let mut value = serde_json::json!({ "kind": "review", "rows": json_rows(&rows) });
            with_array(&mut value, "warnings", &warnings)?;
            with_array(&mut value, "unreadable", &unreadable)?;
            serde_json::to_string_pretty(&value)
                .context("failed to serialize list JSON envelope")?
        }
    };
    Ok(Listing {
        formatted,
        rows: json_rows(&rows),
        warnings,
        unreadable,
    })
}

/// Faithful JSON rows for `list` — the prefixed id, derived status/await, facet,
/// target edge, and title.
#[derive(Debug, Serialize)]
pub(crate) struct ListRow {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) awaiting: String,
    pub(crate) facet: String,
    pub(crate) target: String,
    pub(crate) tags: Vec<String>,
    pub(crate) title: String,
}

fn json_rows(rows: &[ReviewRow]) -> Vec<ListRow> {
    rows.iter()
        .map(|(d, status, awaited)| ListRow {
            id: canonical_id(d.id),
            status: status.as_str().to_owned(),
            awaiting: awaited.as_str().to_owned(),
            facet: d.review.facet.clone(),
            target: edge_label(d),
            tags: d.tags.clone(),
            title: d.title.clone(),
        })
        .collect()
}

/// The CLI stderr line for one unreadable RV (EX-4, A4) — mirrors
/// [`ReviewWarning::line`]'s `warning: ` prefix convention.
pub(super) fn unreadable_line(u: &UnreadableReview) -> String {
    format!("warning: {} unreadable: {}\n", u.rv, u.reason)
}

/// `doctrine review list` — list reviews by id with derived status, facet, and
/// the `reviews`-edge target.
pub(crate) fn run_list(
    path: Option<PathBuf>,
    args: ListArgs,
    target: Option<&str>,
    facet: Option<Facet>,
) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let listing = list_rows(&root, args, target, facet)?;
    Ok(ReviewOutput::Listed {
        rows: listing.rows,
        total: None,
        warnings: listing.warnings,
        unreadable: listing.unreadable,
        formatted: listing.formatted,
    })
}

// ---------------------------------------------------------------------------
// status (Read class) + unlock (escape hatch)
// ---------------------------------------------------------------------------

/// `doctrine review status <RV-NNN>` — report the derived state and REBUILD the
/// baton (the cache == a fresh recompute, design §8/§Verification). Read-class for
/// authored conduct (no authored mutation), but it acquires the lock to serialize
/// the baton write against a concurrent verb. When a warm-cache is primed, it also
/// reports the cache staleness signal (`current`/`stale`, §9 — a signal, not a gate).
pub(crate) fn run_status(path: Option<PathBuf>, reference: &str) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    let _lock = LockGuard::acquire(&root, id)?;
    let (text, doc) = read_authored(&root, id)?;
    let hash = crate::git::sha256(text.as_bytes());
    let states = finding_states_of(&doc);
    let (status, awaited) = derived_status(&states, doc.review.concluded);
    let (awaiting, authored_hash) = reconcile_baton_fields(&states, doc.review.concluded, &hash);
    let prior = read_baton(&root, id)?.unwrap_or_default();
    let rebuilt = Baton {
        awaiting,
        authored_hash,
        ..prior
    };
    write_baton(&root, id, &rebuilt)?;
    // Base plus count off the ledger journal; the baton's legacy counters stand
    // in only for a ledger never journalled (SL-268 sec-2).
    let rounds = counters(&doc, (rebuilt.rounds, rebuilt.contests)).rounds;

    let mut formatted = format!(
        "{} — {} · await={} · findings {} · rounds {}{}\n",
        canonical_id(id),
        status.as_str(),
        awaited.as_str(),
        doc.finding.len(),
        rounds,
        concluded_marker(doc.review.concluded)
    );
    let warnings = warnings_of(&doc);
    formatted.push_str(&warning_lines(&warnings));

    let mut cache_primed = false;
    let mut stale_paths: Vec<String> = Vec::new();
    if let Some(cache) = read_cache(&root, id)? {
        cache_primed = true;
        match cache_staleness(&root, &cache)? {
            CacheVerdict::Current => {
                formatted.push_str("cache: current\n");
            }
            CacheVerdict::Stale(paths) => {
                let joined = paths.join(", ");
                stale_paths = paths;
                formatted.push_str("cache: stale (");
                formatted.push_str(&joined);
                formatted.push_str(")\n");
            }
        }
    }

    Ok(ReviewOutput::Status {
        canonical: canonical_id(id),
        status: status.as_str().to_owned(),
        awaiting: awaited.as_str().to_owned(),
        findings_count: doc.finding.len(),
        rounds: usize::try_from(rounds).unwrap_or(0),
        cache_primed,
        stale_paths,
        warnings,
        formatted,
    })
}
