// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine review findings` — the corpus findings census (SL-278 design
//! sec-5, DEC-370): one row per finding across every review.
//!
//! Filtering splits its axes strictly. Finding axes (`-s`, `-a`, `-S`) go
//! through [`FindingFilter`], the predicate `review show` uses (DEC-363), so
//! open/terminal/unknown classification has one implementation. Text axes
//! (`-f`, `-r`, `-i`, `-t`) go through `listing::retain`, with `status`/`all`
//! cleared first so `retain` applies no status rule or hide-set of its own.

use std::collections::BTreeSet;

use super::read::{Listing, NO_DISPOSITION, ReviewScope, edge_label, severity_cell, with_array};
use super::{
    Column, Context, Facet, FindingRow, FindingStatus, Format, ListArgs, Path, PathBuf, REVIEW_DIR,
    ReviewDoc, ReviewOutput, ReviewWarning, Serialize, Severity, Vocab, canonical_id, listing,
    scan_reviews, warnings_of,
};
use crate::review_ledger::FindingFilter;

/// The census JSON envelope's `kind` token (EX-5, STD-001). Lives here, not
/// in `crate::kinds`: that module holds entity kind specs, and a finding is a
/// row of a review, not an entity.
pub(crate) const FINDING_ROW_KIND: &str = "finding";

/// One census row: a finding, tagged with its review's id, facet and target.
#[derive(Debug, Serialize)]
pub(crate) struct CensusRow {
    pub(crate) review: String,
    pub(crate) facet: String,
    pub(crate) target: String,
    pub(crate) id: String,
    pub(crate) severity: Vocab<Severity>,
    pub(crate) status: Vocab<FindingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) disposition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) route: Option<String>,
    pub(crate) title: String,
}

impl CensusRow {
    fn of(doc: &ReviewDoc, finding: &FindingRow) -> Self {
        Self {
            review: canonical_id(doc.id),
            facet: doc.review.facet.clone(),
            target: edge_label(doc),
            id: finding.id.clone(),
            severity: Vocab::read(&finding.severity),
            status: Vocab::read(&finding.status),
            disposition: finding.disposition.clone(),
            route: finding.route.clone(),
            title: finding.title.clone(),
        }
    }
}

pub(super) const CENSUS_COLUMNS: [Column<CensusRow>; 9] = [
    Column {
        name: "review",
        header: "review",
        cell: |r| r.review.clone(),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    Column {
        name: "facet",
        header: "facet",
        cell: |r| r.facet.clone(),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "target",
        header: "target",
        cell: |r| r.target.clone(),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "id",
        header: "id",
        cell: |r| r.id.clone(),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "severity",
        header: "severity",
        cell: |r| severity_cell(&r.severity),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "status",
        header: "status",
        cell: |r| r.status.as_str().to_owned(),
        paint: listing::ColumnPaint::ByValue(|r| listing::status_hue(r.status.as_str())),
    },
    Column {
        name: "disposition",
        header: "disposition",
        cell: |r| {
            r.disposition
                .clone()
                .unwrap_or_else(|| NO_DISPOSITION.to_owned())
        },
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "route",
        header: "route",
        cell: |r| r.route.clone().unwrap_or_default(),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "title",
        header: "title",
        cell: |r| r.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
];

/// The default visible column set for `review findings`; `--columns` reaches
/// the rest.
pub(super) const CENSUS_DEFAULT: &[&str] =
    &["review", "id", "severity", "status", "target", "title"];

/// The census's finding-axis filter from its own flags: `-s` is the finding
/// status, and terminal findings hide unless `-a` or any `-s` is given —
/// expressed as `open`, not as a second hide rule. A bad `-s`/`-S` is refused
/// with `review show`'s message. Public so the MCP surface can validate (and
/// classify a refusal as invalid arguments) before running.
pub(crate) fn finding_filter(
    args: &ListArgs,
    severity: &[String],
) -> anyhow::Result<FindingFilter> {
    FindingFilter::parse(!args.all && args.status.is_empty(), &args.status, severity)
}

/// A census row's text-axis projection (design sec-5): each identifier its own
/// field, so an anchored regex matches the review id or the finding id whole;
/// the substring domain (slug + title) is finding id + finding title. Status is
/// inert — the census cleared it before `listing::build`.
fn key(doc: &ReviewDoc, finding: &FindingRow) -> listing::FilterFields {
    listing::FilterFields {
        canonical: canonical_id(doc.id),
        slug: finding.id.clone(),
        title: finding.title.clone(),
        status: finding.status.clone(),
        tags: doc.tags.clone(),
    }
}

fn census_rows(
    root: &Path,
    mut args: ListArgs,
    severity: &[String],
    facet: Option<Facet>,
    target: Option<&str>,
) -> anyhow::Result<Listing<CensusRow>> {
    let findings = finding_filter(&args, severity)?;
    // `-s`/`-a` are finding axes, consumed above: left on, `retain` would
    // apply a status rule of its own over the inert status field.
    args.status.clear();
    args.all = false;
    let render = args.render;
    let columns = args.columns.take();
    let (text, format) = listing::build(args)?;
    let scope = ReviewScope::parse(target, facet)?;
    let (mut docs, unreadable) = scan_reviews(&root.join(REVIEW_DIR))?;
    docs.retain(|d| scope.admits(d));
    docs.sort_by_key(|d| d.id);
    let flat: Vec<(&ReviewDoc, &FindingRow)> = docs
        .iter()
        .flat_map(|d| {
            d.finding
                .iter()
                .filter(|f| findings.admits(f))
                .map(move |f| (d, f))
        })
        .collect();
    let flat = listing::retain(flat, &text, |_| false, |(d, f)| key(d, f));
    let contributing: BTreeSet<u32> = flat.iter().map(|(d, _)| d.id).collect();
    let warnings: Vec<ReviewWarning> = docs
        .iter()
        .filter(|d| contributing.contains(&d.id))
        .flat_map(warnings_of)
        .collect();
    let rows: Vec<CensusRow> = flat.into_iter().map(|(d, f)| CensusRow::of(d, f)).collect();
    let formatted = match format {
        Format::Table => {
            let sel = listing::select_columns(&CENSUS_COLUMNS, CENSUS_DEFAULT, columns.as_deref())?;
            listing::render_columns(&rows, &sel, render)
        }
        Format::Json => {
            let mut value = serde_json::json!({ "kind": FINDING_ROW_KIND, "rows": &rows });
            with_array(&mut value, "warnings", &warnings)?;
            with_array(&mut value, "unreadable", &unreadable)?;
            serde_json::to_string_pretty(&value)
                .context("failed to serialize findings JSON envelope")?
        }
    };
    Ok(Listing {
        formatted,
        rows,
        warnings,
        unreadable,
    })
}

/// `doctrine review findings` — the findings census across every review.
pub(crate) fn run_findings(
    path: Option<PathBuf>,
    args: ListArgs,
    severity: &[String],
    facet: Option<Facet>,
    target: Option<&str>,
) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let listing = census_rows(&root, args, severity, facet, target)?;
    Ok(ReviewOutput::FindingsListed {
        rows: listing.rows,
        total: None,
        warnings: listing.warnings,
        unreadable: listing.unreadable,
        formatted: listing.formatted,
    })
}
