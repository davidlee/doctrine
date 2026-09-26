// SPDX-License-Identifier: GPL-3.0-only
//! `review show`/`list`/`status` — the read-only projections (SL-268 PHASE-02 T5).

use super::prime::{CacheVerdict, cache_staleness, read_cache};
use super::turn::{
    Baton, LockGuard, read_baton, reconcile_baton_fields, resolve_review_root, write_baton,
};
use super::{
    Await, Column, Context, FindingRow, FindingStatus, Format, ListArgs, Path, PathBuf, REVIEW_DIR,
    REVIEW_KIND, REVIEW_STATUSES, ReviewDoc, ReviewOutput, ReviewStatus, Serialize, Severity,
    Vocab, VocabDefect, canonical_id, counters, derived_status, finding_states_of, fs, listing,
    parse_ref, read_authored, read_review, read_reviews, vocabulary_defects,
};
use crate::tomlfmt::toml_string;

// ---------------------------------------------------------------------------
// Render (pure — user free-text spliced via `toml_string`, design §4/§5)
// ---------------------------------------------------------------------------

/// A finding as raised, for rendering. Raiser-owned fixed fields plus the
/// responder-owned mutable pair; `status` is transition-graph-owned. The
/// authored on-disk shape of a `[[finding]]` (design §5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Finding {
    pub(crate) id: String,
    pub(crate) status: Vocab<FindingStatus>,
    pub(crate) severity: Vocab<Severity>,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) disposition: Option<String>,
    pub(crate) response: Option<String>,
}

/// Render a single `[[finding]]` TOML block (design §5). Every user free-text
/// field (`title`, `detail`, `disposition`, `response`) is emitted through
/// `toml_string` so a `"`, `\`, newline, or `]` can neither break the document
/// nor inject a key (mem.pattern.render.toml-splice-escape-user-values). The
/// id/status/severity fields are closed vocabularies, rendered bare.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by the review drift canaries / tests (SL-268 D4 split)"
    )
)]
pub(crate) fn render_finding(finding: &Finding) -> String {
    let mut out = String::new();
    out.push_str("[[finding]]\n");
    push_line(&mut out, "id", &toml_string(&finding.id));
    push_line(&mut out, "status", &toml_string(finding.status.as_str()));
    push_line(
        &mut out,
        "severity",
        &toml_string(finding.severity.as_str()),
    );
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
fn edge_label(doc: &ReviewDoc) -> String {
    match &doc.target.phase {
        Some(p) => {
            let mut s = doc.target.reference.clone();
            s.push('@');
            s.push_str(p);
            s
        }
        None => doc.target.reference.clone(),
    }
}

/// `doctrine review show <RV-NNN>` — read the RV as data and render the readable
/// whole (`Table`) or the faithful toml-as-data + brief (`Json`). The status is
/// DERIVED here (review never asks the shared reader for a stored status, D-C8);
/// the `reviews` edge is rendered as `RV-NNN ──reviews──▶ <target>`.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let review_root = root.join(REVIEW_DIR);
    let id = parse_ref(reference)?;
    let doc = read_review(&review_root, id)?;
    let body = read_brief(&review_root, id)?;
    let view = ReviewView::of(&doc);
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
                &body,
                &estimation_unit,
                value_line.as_deref(),
                estimate_line.as_deref(),
                lower_pct,
                upper_pct,
            )
        }
        Format::Json => show_json(&doc, &body, &view.warnings)?,
    };
    let canonical = view.canonical.clone();
    let title = view.title.to_owned();
    let facet = view.facet.to_owned();
    let target = view.target.clone();
    let status = view.status.as_str().to_owned();
    let awaiting = view.awaiting.as_str().to_owned();
    let findings_count = view.findings.len();
    let findings = view.findings;
    let warnings = view.warnings;
    Ok(ReviewOutput::Showed {
        id,
        canonical,
        title,
        status,
        awaiting,
        facet,
        target,
        findings_count,
        findings,
        body,
        warnings,
        formatted,
    })
}

/// Read the `review-NNN.md` brief body (the prose companion).
pub(super) fn read_brief(review_root: &Path, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = review_root.join(&name).join(format!("review-{name}.md"));
    fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

/// The one read projection of a review ledger (RFC-032 D5, IMP-490). Everything
/// a render needs — the derived status/await, the header fields, and the typed
/// findings — computed ONCE from the authored doc. `show` renders it today; the
/// JSON projection and the corpus census (D5 remainder) will read the same
/// struct, so the ledger module can move (D4) without any renderer rewriting.
/// Pure: no I/O, no clock.
pub(super) struct ReviewView<'a> {
    canonical: String,
    title: &'a str,
    facet: &'a str,
    status: ReviewStatus,
    awaiting: Await,
    /// The `reviews`-edge label (`SL-024` or `SL-024@PHASE-03`).
    target: String,
    raiser: &'a str,
    responder: &'a str,
    tags: &'a [String],
    findings: Vec<Finding>,
    /// The ledger's closed-vocabulary defects (SL-268 D15), for disclosure.
    warnings: Vec<ReviewWarning>,
}

/// One closed-vocabulary defect as disclosed on a structured channel (`--json`,
/// MCP): the defect plus the RV it sits on, so a multi-review surface (`list`)
/// needs no second shape. Serialises as `{effect, field, finding, raw, rv}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReviewWarning {
    pub(crate) rv: String,
    #[serde(flatten)]
    pub(crate) defect: VocabDefect,
}

impl ReviewWarning {
    /// The text-channel line (`warning: RV-NNN F-n …`), newline-terminated.
    pub(crate) fn line(&self) -> String {
        let mut line = self.defect.warning_line(&self.rv);
        line.push('\n');
        line
    }
}

/// Every closed-vocabulary defect in one ledger, as [`ReviewWarning`]s.
fn warnings_of(doc: &ReviewDoc) -> Vec<ReviewWarning> {
    let rv = canonical_id(doc.id);
    vocabulary_defects(doc)
        .into_iter()
        .map(|defect| ReviewWarning {
            rv: rv.clone(),
            defect,
        })
        .collect()
}

/// The text-channel rendering of a set of warnings: one line each.
fn warning_lines(warnings: &[ReviewWarning]) -> String {
    warnings.iter().map(ReviewWarning::line).collect()
}

impl<'a> ReviewView<'a> {
    /// Build the projection from the authored doc — the single raw→typed parse of
    /// a review. Pure.
    pub(super) fn of(doc: &'a ReviewDoc) -> Self {
        let (status, awaiting) = doc.derived();
        Self {
            canonical: canonical_id(doc.id),
            title: &doc.title,
            facet: &doc.review.facet,
            status,
            awaiting,
            target: edge_label(doc),
            raiser: &doc.review.raiser,
            responder: &doc.review.responder,
            tags: &doc.tags,
            findings: doc.finding.iter().map(finding_of_row).collect(),
            warnings: warnings_of(doc),
        }
    }
}

/// One authored `[[finding]]` row → the typed [`Finding`] (the MCP/`--json` shape
/// and the index's row). No fallback (SL-268 D15): an out-of-vocabulary status
/// or severity is carried as [`Vocab::Unknown`] with its raw string, so every
/// render shows what the ledger says; the view's `warnings` disclose it.
fn finding_of_row(row: &FindingRow) -> Finding {
    Finding {
        id: row.id.clone(),
        status: Vocab::read(&row.status),
        severity: Vocab::read(&row.severity),
        title: row.title.clone(),
        detail: row.detail.clone(),
        disposition: row.disposition.clone(),
        response: row.response.clone(),
    }
}

/// The em-dash rendered in the index's `disposition` column when a finding
/// carries none (it is set only by `dispose`).
const NO_DISPOSITION: &str = "—";

/// Render the finding index (RFC-032 D5): one row per finding —
/// `id │ severity │ status │ disposition │ title`. Rendered through the shared
/// `listing::render_table` (not bespoke formatting) so colour/width policy and
/// the census's column machinery stay in one place. An empty ledger renders
/// nothing (the `findings: 0` count line stands alone).
pub(super) fn render_finding_index(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return String::new();
    }
    let mut grid: Vec<Vec<String>> = Vec::with_capacity(findings.len() + 1);
    grid.push(
        ["id", "severity", "status", "disposition", "title"]
            .iter()
            .map(|header| (*header).to_owned())
            .collect(),
    );
    grid.extend(findings.iter().map(|f| {
        vec![
            f.id.clone(),
            f.severity.as_str().to_owned(),
            f.status.as_str().to_owned(),
            f.disposition
                .clone()
                .unwrap_or_else(|| NO_DISPOSITION.to_owned()),
            f.title.clone(),
        ]
    }));
    listing::render_table(&grid, None)
}

/// Render the `Table` show: identity header, the derived status + await, the
/// `reviews` edge, the finding index, then the brief body. House style —
/// `Vec<String>` joined by `concat` (avoids the `push_str(&format!)` lint).
pub(super) fn format_show(
    view: &ReviewView<'_>,
    body: &str,
    _estimation_unit: &str,
    value_line: Option<&str>,
    estimate_line: Option<&str>,
    _lower_pct: f64,
    _upper_pct: f64,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", view.canonical, view.title));
    parts.push(format!(
        "{} · {} · await={}\n",
        view.facet,
        view.status.as_str(),
        view.awaiting.as_str()
    ));
    parts.push(format!("{} ──reviews──▶ {}\n", view.canonical, view.target));
    parts.push(format!(
        "findings: {} (raiser {} · responder {})\n",
        view.findings.len(),
        view.raiser,
        view.responder
    ));
    parts.push(render_finding_index(&view.findings));
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
    parts.push(format!("\n{body}"));
    parts.concat()
}

/// The faithful JSON `show` row — the toml-as-data plus the derived status (the
/// one computed field surfaced; never stored) and the brief body.
#[derive(Debug, Serialize)]
struct ShowJson<'a> {
    #[serde(flatten)]
    doc: &'a ReviewDoc,
    status: &'a str,
    awaiting: &'a str,
}

/// Render the `Json` show under the shared `{kind, …}` envelope. A top-level
/// `warnings` array rides beside it only when the ledger has a defect, so a clean
/// ledger's JSON is unchanged.
fn show_json(doc: &ReviewDoc, body: &str, warnings: &[ReviewWarning]) -> anyhow::Result<String> {
    let (status, awaited) = doc.derived();
    let row = ShowJson {
        doc,
        status: status.as_str(),
        awaiting: awaited.as_str(),
    };
    let mut value = serde_json::json!({ "kind": "review", "review": row, "body": body });
    with_warnings(&mut value, warnings)?;
    serde_json::to_string_pretty(&value).context("failed to serialize review show JSON")
}

/// Attach a non-empty `warnings` array to a JSON object envelope (absent when
/// empty — the clean-ledger shape stays byte-identical).
fn with_warnings(value: &mut serde_json::Value, warnings: &[ReviewWarning]) -> anyhow::Result<()> {
    if warnings.is_empty() {
        return Ok(());
    }
    let warnings = serde_json::to_value(warnings).context("failed to serialize review warnings")?;
    if let Some(object) = value.as_object_mut() {
        object.insert("warnings".to_owned(), warnings);
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

/// What `list_rows` computes: the rendered list, its JSON rows, and the
/// closed-vocabulary warnings across the listed RVs (in id order).
type Listing = (String, Vec<ListRow>, Vec<ReviewWarning>);

/// `review list` rows as a string — the compute half of [`run_list`]. No hide-set
/// (an RV is either Active or Done; both are listed), sorted by id, each row
/// carrying its derived status. The listed (post-filter) RVs' vocabulary defects
/// are gathered once here, for the CLI's stderr and the MCP `warnings` field
/// alike; `--json` also carries them top-level when there are any.
fn list_rows(root: &Path, mut args: ListArgs, target: Option<&str>) -> anyhow::Result<Listing> {
    listing::validate_statuses(&args.status, REVIEW_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let review_root = root.join(REVIEW_DIR);
    let mut docs = read_reviews(&review_root)?;
    // RFC-032 D5: `--target` admits only reviews on the given subject edge. The
    // filter is on the BARE `[target].ref`, so `SL-024` admits a
    // `SL-024@PHASE-03` edge too (phase scope is not part of the subject id).
    if let Some(want) = target {
        docs.retain(|d| d.target.reference == want);
    }
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
        // The shared `listing::json_envelope` shape, plus `warnings` when any.
        Format::Json => {
            let mut value = serde_json::json!({ "kind": "review", "rows": json_rows(&rows) });
            with_warnings(&mut value, &warnings)?;
            serde_json::to_string_pretty(&value)
                .context("failed to serialize list JSON envelope")?
        }
    };
    Ok((formatted, json_rows(&rows), warnings))
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

/// `doctrine review list` — list reviews by id with derived status, facet, and
/// the `reviews`-edge target.
pub(crate) fn run_list(
    path: Option<PathBuf>,
    args: ListArgs,
    target: Option<&str>,
) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (formatted, rows, warnings) = list_rows(&root, args, target)?;
    Ok(ReviewOutput::Listed {
        rows,
        total: None,
        warnings,
        formatted,
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

    // The marker rides the status line only when set: silence is the far commoner
    // state, and a `concluded=no` on every unconcluded pass would be noise on the
    // line a reader scans for the derived state.
    let concluded = if doc.review.concluded {
        " · concluded"
    } else {
        ""
    };
    let mut formatted = format!(
        "{} — {} · await={} · findings {} · rounds {}{concluded}\n",
        canonical_id(id),
        status.as_str(),
        awaited.as_str(),
        doc.finding.len(),
        rounds
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
