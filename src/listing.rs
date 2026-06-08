// SPDX-License-Identifier: GPL-3.0-only
//! The kind-blind read spine: the invariant axes of every `list` surface —
//! filtering, id form, output format, the generic table layout, and the JSON
//! envelope — lifted out of the five bespoke per-kind implementations (design
//! SL-025 §5.2).
//!
//! This is a **pure leaf** (ADR-001 module layering: leaf ← engine ← command).
//! It imports no `clap` and no `entity`: the clap-facing arg bundle lives
//! command-side and maps parsed args onto [`Filter`] / [`Format`] here, and the
//! id prefix is passed into [`canonical_id`] as a plain `&str` rather than read
//! from `entity::Kind`. Nothing here reads the clock, rng, git, or disk.
//!
//! What is shared (the *invariant* axis): the filter semantics ([`retain`] over
//! [`FilterFields`]), the status known-set check ([`validate_statuses`]), the
//! canonical id form ([`canonical_id`]), the table layout ([`render_table`],
//! relocated from `meta.rs`), and the JSON envelope ([`json_envelope`]). What
//! stays per-kind (the *variant* axis): the row type, the column projection, the
//! ordering, and the kind-specific flags — none of which live here.
//!
//! The kinds migrate onto this leaf one per phase (SL-025 PHASE-02+). A symbol
//! not yet consumed by a migrated kind carries a narrowed per-symbol
//! `#[expect(dead_code)]` that retires itself as each phase wires its consumer.

use std::fmt;
use std::str::FromStr;

use anyhow::Context;
use regex_lite::Regex;
use serde::Serialize;

/// `SL` + `25` → `"SL-25"`; zero-padded to three digits like the citation
/// convention (`SL-025`). The single id-form authority for prefixed kinds.
/// Memory is conformant-by-exception — its uid *is* its canonical id, so it does
/// not route through here (design §5.3).
pub(crate) fn canonical_id(prefix: &str, id: u32) -> String {
    format!("{prefix}-{id:03}")
}

/// Output format for a `list`/`show` surface. A plain enum (NOT `clap::ValueEnum`,
/// which would drag clap into this leaf — A-3); the command layer wires it via
/// `#[arg(value_parser = Format::from_str)]`. `Display` is required by clap's
/// `default_value_t` (C-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Format {
    /// The default human-readable surface; `--format`/`--json` opt out of it.
    #[default]
    Table,
    Json,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Table => "table",
            Format::Json => "json",
        })
    }
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "table" => Ok(Format::Table),
            "json" => Ok(Format::Json),
            other => Err(anyhow::anyhow!(
                "unknown format `{other}` (expected `table` or `json`)"
            )),
        }
    }
}

/// One row projected to exactly its filterable fields — computed **once** per row
/// by the kind, then reused across both match domains (A-1). `substr` matches
/// `slug`+`title`; `regex` matches `canonical`+`slug`+`title` — distinct domains,
/// both derivable from this single projection. Filter-only; never a render type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FilterFields {
    /// The regex domain's leading field: `SL-025` / `ADR-001` / `mem_…`.
    pub(crate) canonical: String,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) tags: Vec<String>,
}

/// The resolved, pre-compiled filter. Built once command-side from the parsed
/// flags via [`build`]; the leaf never sees clap. `regex_lite::Regex` is not
/// `Eq`, so this type intentionally does not derive `PartialEq` — compare by
/// behaviour in tests.
pub(crate) struct Filter {
    /// Lowercased once at build time (case-insensitive substring match).
    pub(crate) substr: Option<String>,
    /// Pre-compiled; the case flag is baked into the pattern at build time.
    pub(crate) regex: Option<Regex>,
    /// Empty = no status constraint; otherwise a row matches iff its status is in
    /// the set (OR within `--status`).
    pub(crate) status: Vec<String>,
    /// Empty = no tag constraint; otherwise OR within tags.
    pub(crate) tags: Vec<String>,
    /// Show every state, including the kind's hide-set.
    pub(crate) all: bool,
}

impl fmt::Debug for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Filter")
            .field("substr", &self.substr)
            .field("regex", &self.regex.as_ref().map(Regex::as_str))
            .field("status", &self.status)
            .field("tags", &self.tags)
            .field("all", &self.all)
            .finish()
    }
}

/// The raw, parsed flag values the command layer hands to [`build`] — the
/// clap-free mirror of its `CommonListArgs` bundle (no clap types cross this
/// seam, A-3). One struct rather than a long positional argument list keeps the
/// call site self-documenting and the seam stable as flags accrete.
#[derive(Debug, Default)]
pub(crate) struct ListArgs {
    /// Substring filter on slug+title (case-insensitive).
    pub(crate) substr: Option<String>,
    /// Regex over canonical-id + slug + title.
    pub(crate) regexp: Option<String>,
    /// Make the regex case-insensitive.
    pub(crate) case_insensitive: bool,
    /// Status filter, multi-value; any value reveals the hide-set.
    pub(crate) status: Vec<String>,
    /// Tag filter, OR within the axis.
    pub(crate) tags: Vec<String>,
    /// Show every state, including the hide-set.
    pub(crate) all: bool,
    /// Output format from `--format` (overridden by `json` below).
    pub(crate) format: Format,
    /// `--json` sugar — forces [`Format::Json`] regardless of `format` (A-9).
    pub(crate) json: bool,
}

/// Build a [`Filter`] + resolved [`Format`] from the parsed [`ListArgs`].
/// Lowercases the substring once, pre-compiles the regex (a bad pattern is a
/// clean `anyhow` error, never a panic), and folds `--json` over `--format`:
/// `--json` forces [`Format::Json`] and wins over any `--format` value (A-9).
pub(crate) fn build(args: ListArgs) -> anyhow::Result<(Filter, Format)> {
    let regex = match args.regexp {
        None => None,
        Some(pattern) => {
            let pattern = if args.case_insensitive {
                format!("(?i){pattern}")
            } else {
                pattern
            };
            let compiled =
                Regex::new(&pattern).with_context(|| format!("invalid regex `{pattern}`"))?;
            Some(compiled)
        }
    };
    let filter = Filter {
        substr: args.substr.map(|s| s.to_lowercase()),
        regex,
        status: args.status,
        tags: args.tags,
        all: args.all,
    };
    let resolved = if args.json { Format::Json } else { args.format };
    Ok((filter, resolved))
}

/// Keep a row iff every active axis admits it. Axes AND across each other; within
/// `--status` and `--tag` the match is OR. Concretely a row is kept when:
/// not hidden (by the kind's LIST hide-set) AND substr-match (slug+title) AND
/// regex-match (canonical+slug+title) AND status-match AND tag-match.
///
/// The hide-set is suppressed when `--all` is set OR any explicit `--status` is
/// given (design §5.5 terminal-hide override). `is_hidden` is the kind's
/// presentation hide-set predicate — **distinct** from any divergence /
/// lifecycle-terminal predicate (design §5.3); `key` projects each row to its
/// [`FilterFields`].
///
/// FILTER-ONLY: ordering is the caller's (per-kind) concern (design §5.3) — this
/// preserves input order and never sorts.
pub(crate) fn retain<R>(
    rows: Vec<R>,
    f: &Filter,
    is_hidden: impl Fn(&str) -> bool,
    key: impl Fn(&R) -> FilterFields,
) -> Vec<R> {
    let reveal_hidden = f.all || !f.status.is_empty();
    rows.into_iter()
        .filter(|row| {
            let fields = key(row);
            if !reveal_hidden && is_hidden(&fields.status) {
                return false;
            }
            substr_admits(f, &fields)
                && regex_admits(f, &fields)
                && status_admits(f, &fields)
                && tags_admit(f, &fields)
        })
        .collect()
}

/// Substr match domain: lowercased slug+title (the substring was lowercased once
/// at build, so compare lowercased haystacks).
fn substr_admits(f: &Filter, fields: &FilterFields) -> bool {
    match &f.substr {
        None => true,
        Some(needle) => {
            fields.slug.to_lowercase().contains(needle)
                || fields.title.to_lowercase().contains(needle)
        }
    }
}

/// Regex match domain: canonical+slug+title (distinct from the substr domain —
/// the canonical id is searchable only here, A-1).
fn regex_admits(f: &Filter, fields: &FilterFields) -> bool {
    match &f.regex {
        None => true,
        Some(re) => {
            re.is_match(&fields.canonical)
                || re.is_match(&fields.slug)
                || re.is_match(&fields.title)
        }
    }
}

fn status_admits(f: &Filter, fields: &FilterFields) -> bool {
    f.status.is_empty() || f.status.contains(&fields.status)
}

fn tags_admit(f: &Filter, fields: &FilterFields) -> bool {
    f.tags.is_empty() || f.tags.iter().any(|t| fields.tags.contains(t))
}

/// Validate a stringly `--status` set against a kind's known statuses, with one
/// uniform error message (A-2 — recovers the correctness the shared
/// `Vec<String>` bundle loses versus a typed clap enum). Validates READ (filter)
/// input only — never a stored-status write/transition.
pub(crate) fn validate_statuses(given: &[String], known: &[&str]) -> anyhow::Result<()> {
    if let Some(bad) = given.iter().find(|s| !known.contains(&s.as_str())) {
        let known = known.join(", ");
        anyhow::bail!("unknown status `{bad}` (known: {known})");
    }
    Ok(())
}

/// The two-space inter-column gap — the single source of column spacing for every
/// `*list` surface.
const COL_GAP: &str = "  ";

/// Render `rows` as a left-aligned, two-space-gapped text table: each column is
/// padded to its widest cell, the final column of each row is never padded, and a
/// trailing newline terminates non-empty output (no rows → `""`, which keeps the
/// header suppressed on an empty list — design §5.5).
///
/// The single layout authority for every list surface (relocated from `meta.rs`
/// — it served only numeric kinds there, but the spine serves named (memory) and
/// own-struct (backlog) kinds too). It carries no per-kind knowledge and renders
/// nothing but a grid of strings (not a column framework). Callers bake any
/// markers — and the header row — into the cell strings, so it stays
/// presentation-neutral.
pub(crate) fn render_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths: Vec<usize> = (0..cols)
        .map(|c| {
            rows.iter()
                .filter_map(|r| r.get(c))
                .map(|cell| cell.chars().count())
                .max()
                .unwrap_or(0)
        })
        .collect();
    let mut out = String::new();
    for row in rows {
        let last = row.len().saturating_sub(1);
        for (c, cell) in row.iter().enumerate() {
            if c > 0 {
                out.push_str(COL_GAP);
            }
            out.push_str(cell);
            if c != last {
                let pad = widths
                    .get(c)
                    .copied()
                    .unwrap_or(0)
                    .saturating_sub(cell.chars().count());
                out.extend(std::iter::repeat_n(' ', pad));
            }
        }
        out.push('\n');
    }
    out
}

/// Wrap kind-faithful row values in the shared envelope: `{ "kind": …, "rows":
/// [ … ] }`. Each kind owns its row serde shape (a faithful mirror, D7); this
/// just supplies the uniform outer frame.
pub(crate) fn json_envelope<T: Serialize>(kind: &str, rows: &[T]) -> anyhow::Result<String> {
    let envelope = serde_json::json!({ "kind": kind, "rows": rows });
    serde_json::to_string_pretty(&envelope).context("failed to serialize list JSON envelope")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- canonical_id ------------------------------------------------------

    #[test]
    fn canonical_id_prefixes_and_zero_pads_to_three() {
        assert_eq!(canonical_id("SL", 25), "SL-025");
        assert_eq!(canonical_id("ADR", 1), "ADR-001");
        assert_eq!(canonical_id("PRD", 123), "PRD-123");
        // four+ digits are not truncated — padding is a minimum, not a cap.
        assert_eq!(canonical_id("REQ", 1234), "REQ-1234");
    }

    // -- Format from_str / Display ----------------------------------------

    #[test]
    fn format_parses_table_and_json() {
        assert_eq!(Format::from_str("table").unwrap(), Format::Table);
        assert_eq!(Format::from_str("json").unwrap(), Format::Json);
    }

    #[test]
    fn format_rejects_unknown_value() {
        let err = Format::from_str("yaml").unwrap_err().to_string();
        assert!(err.contains("yaml"), "error names the bad value: {err}");
    }

    #[test]
    fn format_display_round_trips_from_str() {
        for f in [Format::Table, Format::Json] {
            assert_eq!(Format::from_str(&f.to_string()).unwrap(), f);
        }
    }

    // -- build: --json precedence + regex compile -------------------------

    /// Build with everything at its default — the no-constraint filter.
    fn no_filter() -> (Filter, Format) {
        build(ListArgs::default()).unwrap()
    }

    #[test]
    fn build_json_flag_forces_json_over_format_table() {
        // A-9: --json wins over --format table, no error.
        let (_f, fmt) = build(ListArgs {
            format: Format::Table,
            json: true,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(fmt, Format::Json);
    }

    #[test]
    fn build_without_json_flag_honours_format() {
        let (_f, fmt) = build(ListArgs {
            format: Format::Json,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(fmt, Format::Json);
    }

    #[test]
    fn build_lowercases_the_substring_once() {
        let (f, _) = build(ListArgs {
            substr: Some("HeLLo".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(f.substr.as_deref(), Some("hello"));
    }

    #[test]
    fn build_compiles_a_valid_regex() {
        let (f, _) = build(ListArgs {
            regexp: Some("^SL-".into()),
            ..Default::default()
        })
        .unwrap();
        assert!(f.regex.is_some());
    }

    #[test]
    fn build_invalid_regex_is_a_clean_error_not_a_panic() {
        let err = build(ListArgs {
            regexp: Some("(unclosed".into()),
            ..Default::default()
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("invalid regex"), "got: {err}");
    }

    #[test]
    fn build_case_insensitive_bakes_the_flag_into_the_pattern() {
        let (f, _) = build(ListArgs {
            regexp: Some("sl".into()),
            case_insensitive: true,
            ..Default::default()
        })
        .unwrap();
        let re = f.regex.unwrap();
        assert!(
            re.is_match("SL-025"),
            "case-insensitive should match uppercase"
        );
    }

    // -- retain matrix -----------------------------------------------------

    /// A test row carrying everything `retain` projects.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Row {
        canonical: &'static str,
        slug: &'static str,
        title: &'static str,
        status: &'static str,
        tags: Vec<&'static str>,
    }

    fn row(
        canonical: &'static str,
        slug: &'static str,
        title: &'static str,
        status: &'static str,
        tags: &[&'static str],
    ) -> Row {
        Row {
            canonical,
            slug,
            title,
            status,
            tags: tags.to_vec(),
        }
    }

    fn fields(r: &Row) -> FilterFields {
        FilterFields {
            canonical: r.canonical.to_string(),
            slug: r.slug.to_string(),
            title: r.title.to_string(),
            status: r.status.to_string(),
            tags: r.tags.iter().map(|t| (*t).to_string()).collect(),
        }
    }

    /// The slice hide-set, for the hide-set arm of the matrix.
    fn hidden(status: &str) -> bool {
        matches!(status, "done" | "abandoned")
    }

    fn never_hidden(_: &str) -> bool {
        false
    }

    fn sample() -> Vec<Row> {
        vec![
            row("SL-001", "alpha-thing", "Alpha Thing", "proposed", &["x"]),
            row("SL-002", "beta-widget", "Beta Widget", "done", &["y"]),
            row(
                "SL-003",
                "gamma-gadget",
                "Gamma Gadget",
                "started",
                &["x", "z"],
            ),
            row(
                "SL-004",
                "delta-doohickey",
                "Delta Doohickey",
                "abandoned",
                &["y"],
            ),
        ]
    }

    fn canonicals(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.canonical).collect()
    }

    #[test]
    fn retain_default_hides_the_hide_set() {
        let (f, _) = no_filter();
        let kept = retain(sample(), &f, hidden, fields);
        // done (SL-002) + abandoned (SL-004) dropped by default.
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_all_reveals_the_hide_set() {
        let (f, _) = build(ListArgs {
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(
            canonicals(&kept),
            vec!["SL-001", "SL-002", "SL-003", "SL-004"]
        );
    }

    #[test]
    fn retain_explicit_status_reveals_the_hide_set() {
        // --status done reveals the otherwise-hidden done row.
        let (f, _) = build(ListArgs {
            status: vec!["done".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-002"]);
    }

    #[test]
    fn retain_multi_status_is_or_within_the_axis() {
        let (f, _) = build(ListArgs {
            status: vec!["proposed".into(), "started".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_substr_matches_slug_or_title_case_insensitively() {
        let (f, _) = build(ListArgs {
            substr: Some("WIDGET".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-002"]);
    }

    #[test]
    fn retain_tag_is_or_and_ands_with_other_axes() {
        // tag x (alpha + gamma); default hide-set drops nothing here.
        let (f, _) = build(ListArgs {
            tags: vec!["x".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_axes_and_across_each_other() {
        // substr "a" matches all four slugs/titles; tag y narrows to beta+delta;
        // both are hide-set (done/abandoned) but tag isn't a status so the
        // hide-set still applies (no --all / --status) → empty.
        let (f, _) = build(ListArgs {
            substr: Some("a".into()),
            tags: vec!["y".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert!(
            kept.is_empty(),
            "hide-set still drops done+abandoned: {:?}",
            canonicals(&kept)
        );
    }

    #[test]
    fn retain_regex_over_canonical() {
        let (f, _) = build(ListArgs {
            regexp: Some("SL-00[13]".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_preserves_input_order_never_sorts() {
        // Intentionally unsorted; retain must not reorder (ordering is per-kind).
        let rows = vec![
            row("SL-003", "c", "C", "proposed", &[]),
            row("SL-001", "a", "A", "proposed", &[]),
            row("SL-002", "b", "B", "proposed", &[]),
        ];
        let (f, _) = no_filter();
        let kept = retain(rows, &f, never_hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-003", "SL-001", "SL-002"]);
    }

    // -- VT-2 domain distinction ------------------------------------------

    #[test]
    fn retain_regex_matches_canonical_only_keeps_the_row() {
        // canonical matches the regex; slug+title do NOT — proves regex searches
        // the canonical domain (A-1 guard).
        let rows = vec![row(
            "SL-042",
            "nomatch-slug",
            "Nomatch Title",
            "proposed",
            &[],
        )];
        let (f, _) = build(ListArgs {
            regexp: Some("SL-042".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(rows, &f, never_hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-042"]);
    }

    #[test]
    fn retain_substr_does_not_search_canonical() {
        // substr "042" appears in canonical but NOT in slug/title → dropped.
        // Proves the substr domain excludes canonical (the other half of A-1).
        let rows = vec![row(
            "SL-042",
            "nomatch-slug",
            "Nomatch Title",
            "proposed",
            &[],
        )];
        let (f, _) = build(ListArgs {
            substr: Some("042".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(rows, &f, never_hidden, fields);
        assert!(
            kept.is_empty(),
            "substr must not see canonical: {:?}",
            canonicals(&kept)
        );
    }

    #[test]
    fn retain_substr_only_match_is_kept() {
        // slug matches the substr; canonical does not contain it → kept via the
        // substr domain (independent of regex, which is absent).
        let rows = vec![row("SL-042", "special-slug", "Title", "proposed", &[])];
        let (f, _) = build(ListArgs {
            substr: Some("special".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(rows, &f, never_hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-042"]);
    }

    // -- validate_statuses -------------------------------------------------

    #[test]
    fn validate_statuses_accepts_known_values() {
        let known = ["proposed", "ready", "done"];
        assert!(validate_statuses(&["proposed".into(), "done".into()], &known).is_ok());
        assert!(validate_statuses(&[], &known).is_ok());
    }

    #[test]
    fn validate_statuses_rejects_an_unknown_value_naming_it_and_the_set() {
        let known = ["proposed", "ready", "done"];
        let err = validate_statuses(&["bogus".into()], &known)
            .unwrap_err()
            .to_string();
        assert!(err.contains("bogus"), "names the bad value: {err}");
        assert!(err.contains("proposed"), "lists the known set: {err}");
    }

    // -- render_table (relocated; behaviour preserved) --------------------

    fn cells(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn render_table_empty_is_empty_string() {
        assert_eq!(render_table(&[]), "");
    }

    #[test]
    fn render_table_aligns_ragged_columns_and_leaves_last_unpadded() {
        let out = render_table(&[
            cells(&["a", "longvalue", "x"]),
            cells(&["bb", "y", "trailing"]),
        ]);
        assert_eq!(out, "a   longvalue  x\nbb  y          trailing\n");
    }

    #[test]
    fn render_table_aligns_a_middle_column_the_slice_case() {
        let out = render_table(&[
            cells(&["001", "done", "4/6", "memory-entity-v1", "Memory entity v1"]),
            cells(&[
                "009",
                "proposed",
                "—",
                "slice-status-rollup",
                "Slice status rollup",
            ]),
        ]);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0],
            "001  done      4/6  memory-entity-v1     Memory entity v1"
        );
        assert_eq!(
            lines[1],
            "009  proposed  —    slice-status-rollup  Slice status rollup"
        );
    }

    // -- json_envelope -----------------------------------------------------

    #[derive(Serialize)]
    struct JsonRow {
        id: &'static str,
        status: &'static str,
    }

    #[test]
    fn json_envelope_wraps_rows_under_kind() {
        let rows = [
            JsonRow {
                id: "SL-001",
                status: "proposed",
            },
            JsonRow {
                id: "SL-002",
                status: "done",
            },
        ];
        let out = json_envelope("slice", &rows).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "slice");
        assert_eq!(parsed["rows"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["rows"][0]["id"], "SL-001");
        assert_eq!(parsed["rows"][1]["status"], "done");
    }

    #[test]
    fn json_envelope_empty_rows_is_an_empty_array() {
        let rows: [JsonRow; 0] = [];
        let out = json_envelope("adr", &rows).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "adr");
        assert_eq!(parsed["rows"].as_array().unwrap().len(), 0);
    }
}
