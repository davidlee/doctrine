// SPDX-License-Identifier: GPL-3.0-only
//! Full-text search over the entity corpus (SL-141 PHASE-03).
//!
//! Uses `BM25` ranking over entity bodies, with kind filtering via [`KindSelector`]
//! and contextual snippets extracted around matching tokens.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::str::FromStr;

use anyhow::Result;
use clap::Args;

use crate::catalog::hydrate::{CatalogEntity, CatalogKey, scan_catalog};
use crate::catalog::scan::ScanMode;
use crate::integrity;
use crate::lexical::{
    Bm25Ranker, LexDoc, LexicalCorpus, LexicalRanker, tokenize, tokenize_with_spans,
};
use crate::listing::{Column, ColumnPaint, Format, RenderOpts, render_columns};

// ── KindSelector ──────────────────────────────────────────────────────────

const DEFAULT_SEARCH_KINDS: &[&str] = &[
    "SL", "ADR", "PRD", "SPEC", "RFC", "ISS", "IMP", "CHR", "RSK", "IDE", "ASM", "DEC", "QUE",
    "CON",
];

const GROUP_ALIASES: &[(&str, &[&str])] = &[
    ("backlog", &["ISS", "IMP", "CHR", "RSK", "IDE"]),
    ("governance", &["ADR", "POL", "STD"]),
    ("specs", &["PRD", "SPEC"]),
    ("knowledge", &["ASM", "DEC", "QUE", "CON"]),
    (
        "all",
        &[
            "SL", "ADR", "PRD", "SPEC", "RFC", "ISS", "IMP", "CHR", "RSK", "IDE", "REQ", "RV",
            "REC", "REV", "CM", "POL", "STD", "ASM", "DEC", "QUE", "CON",
        ],
    ),
];

#[derive(Debug, Clone)]
pub(crate) struct KindSelector {
    prefixes: Vec<String>,
}

impl KindSelector {
    /// Parse from CLI args. `kind_opt` replaces default; `with_list` adds; `without_list` removes.
    pub(crate) fn resolve(
        kind_opt: Option<&str>,
        with_list: &[String],
        without_list: &[String],
    ) -> Result<Self> {
        let mut prefixes: BTreeSet<String> = if let Some(k) = kind_opt {
            Self::expand(k)?
        } else {
            DEFAULT_SEARCH_KINDS
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        };

        for w in with_list {
            let expanded = Self::expand(w)?;
            prefixes.extend(expanded);
        }
        for w in without_list {
            let expanded = Self::expand(w)?;
            for p in expanded {
                prefixes.remove(&p);
            }
        }

        Ok(Self {
            prefixes: prefixes.into_iter().collect(),
        })
    }

    /// Expand a comma-separated list of prefixes/aliases into a `BTreeSet` of valid prefixes.
    /// Returns `Err` if any token is unrecognized.
    fn expand(input: &str) -> Result<BTreeSet<String>> {
        let mut result = BTreeSet::new();
        let known: BTreeSet<&str> = integrity::KINDS.iter().map(|kr| kr.kind.prefix).collect();

        for token in input.split(',') {
            let t = token.trim().to_uppercase();
            if t.is_empty() {
                continue;
            }

            // Check group aliases first
            let mut expanded = false;
            for (alias, members) in GROUP_ALIASES {
                if t == alias.to_uppercase() {
                    for m in *members {
                        result.insert((*m).to_string());
                    }
                    expanded = true;
                    break;
                }
            }
            if expanded {
                continue;
            }

            // Check if it's a known prefix
            if known.contains(t.as_str()) {
                result.insert(t);
            } else {
                let mut valid: Vec<&str> = known.iter().copied().collect();
                valid.sort_unstable();
                let mut group_names: Vec<&str> = GROUP_ALIASES.iter().map(|(n, _)| *n).collect();
                group_names.sort_unstable();
                return Err(anyhow::anyhow!(
                    "unknown kind prefix or group: '{token}'. Valid prefixes: {}. Valid groups: {}",
                    valid.join(", "),
                    group_names.join(", "),
                ));
            }
        }
        Ok(result)
    }

    pub(crate) fn matches(&self, prefix: &str) -> bool {
        self.prefixes.iter().any(|p| p == prefix)
    }
}

// ── entity_lex_doc ────────────────────────────────────────────────────────

pub(crate) fn entity_lex_doc(entity: &CatalogEntity) -> LexDoc {
    let text = match &entity.body {
        Some(body) => format!("{}\n{}", entity.title, body),
        None => entity.title.clone(),
    };
    LexDoc {
        id: match &entity.key {
            CatalogKey::Numbered(k) => k.canonical(),
            CatalogKey::Memory(uid) => uid.clone(),
        },
        text,
    }
}

// ── snippet ───────────────────────────────────────────────────────────────

/// Extract a context window around the first query-token match.
/// Returns empty string if no query token found in `doc_text`.
pub(crate) fn snippet(doc_text: &str, query: &str, context_chars: usize) -> String {
    let query_tokens: BTreeSet<String> = tokenize(query).into_iter().collect();
    if query_tokens.is_empty() {
        return String::new();
    }

    let spans = tokenize_with_spans(doc_text);
    // Find first span whose token matches any query token
    let first_match = spans.iter().find(|ts| query_tokens.contains(&ts.token));

    let Some(matched) = first_match else {
        return String::new();
    };

    let doc_len = doc_text.len();
    let window_start = matched.start.saturating_sub(context_chars);
    let window_end = std::cmp::min(matched.end + context_chars, doc_len);

    let mut result = String::new();
    if window_start > 0 {
        result.push('\u{2026}'); // ellipsis
        // Get the slice safely (already bounded)
        if let Some(s) = doc_text.get(window_start..window_end) {
            result.push_str(s);
        }
    } else if let Some(s) = doc_text.get(..window_end) {
        result.push_str(s);
    }
    if window_end < doc_len {
        result.push('\u{2026}');
    }
    result
}

// ── SearchArgs ────────────────────────────────────────────────────────────

/// Full-text search over the entity corpus.
#[derive(Args, Debug)]
pub(crate) struct SearchArgs {
    /// Free-text lexical query (positional, required)
    pub(crate) query: String,

    /// Replace default search kinds (comma-separated prefixes/aliases)
    #[arg(short = 'k', long = "kinds")]
    pub(crate) kinds: Option<String>,

    /// Add kinds to the effective set (can be repeated)
    #[arg(long)]
    pub(crate) with: Vec<String>,

    /// Remove kinds from the effective set (can be repeated)
    #[arg(long)]
    pub(crate) no: Vec<String>,

    /// Output format [default: table]
    #[arg(short = 'f', long = "format", value_parser = Format::from_str, default_value_t = Format::Table)]
    pub(crate) format: Format,

    /// Show body snippet for each result
    #[arg(short = 'c', long)]
    pub(crate) context: bool,

    /// Max results to show [default: 20]
    #[arg(long, default_value = "20")]
    pub(crate) limit: usize,

    /// Skip first N results [default: 0]
    #[arg(long, default_value = "0")]
    pub(crate) offset: usize,

    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<std::path::PathBuf>,
}

// ── helpers ───────────────────────────────────────────────────────────────

/// Build a `serde_json::Value` object for one search hit.
fn build_json_hit(
    id: &str,
    score: u32,
    entity: Option<&&CatalogEntity>,
    opts: &SearchArgs,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("id".to_string(), serde_json::json!(id));
    map.insert("score".to_string(), serde_json::json!(score));

    if let Some(e) = entity {
        map.insert("title".to_string(), serde_json::json!(e.title));
        let kind_lower = match &e.key {
            CatalogKey::Numbered(k) => k.prefix.to_lowercase(),
            CatalogKey::Memory(_) => "mem".to_string(),
        };
        map.insert("kind".to_string(), serde_json::json!(kind_lower));
        let prefix = match &e.key {
            CatalogKey::Numbered(k) => k.prefix.to_string(),
            CatalogKey::Memory(_) => "MEM".to_string(),
        };
        map.insert("prefix".to_string(), serde_json::json!(prefix));
        if let Some(status) = &e.status {
            map.insert("status".to_string(), serde_json::json!(status));
        }
        if opts.context {
            let body = e.body.as_deref().unwrap_or(&e.title);
            let snip = snippet(body, &opts.query, 40);
            if !snip.is_empty() {
                map.insert("snippet".to_string(), serde_json::json!(snip));
            }
        }
    }
    serde_json::Value::Object(map)
}

// ── SearchRow ──────────────────────────────────────────────────────────────

/// One rendered row in the search results table.
pub(crate) struct SearchRow {
    id: String,
    kind: String,
    status: String,
    title: String,
}

const SEARCH_COLUMNS: [Column<SearchRow>; 4] = [
    Column {
        name: "id",
        header: "ID",
        cell: |r| r.id.clone(),
        paint: ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    Column {
        name: "kind",
        header: "Kind",
        cell: |r| r.kind.clone(),
        paint: ColumnPaint::None,
    },
    Column {
        name: "status",
        header: "Status",
        cell: |r| r.status.clone(),
        paint: ColumnPaint::ByValue(|r: &SearchRow| crate::listing::status_hue(&r.status)),
    },
    Column {
        name: "title",
        header: "Title",
        cell: |r| r.title.clone(),
        paint: ColumnPaint::Alternate([crate::listing::TITLE_EVEN, crate::listing::TITLE_ODD]),
    },
];

// ── run() ─────────────────────────────────────────────────────────────────

pub(crate) fn run(mut args: SearchArgs, render: RenderOpts) -> Result<()> {
    let path = args.path.take();
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let selector = KindSelector::resolve(args.kinds.as_deref(), &args.with, &args.no)?;
    let catalog = scan_catalog(&root, ScanMode::include_bodies())?;

    // Filter by kind
    let matching: Vec<&CatalogEntity> = catalog
        .entities
        .iter()
        .filter(|e| match &e.key {
            CatalogKey::Numbered(k) => selector.matches(k.prefix),
            CatalogKey::Memory(_) => false,
        })
        .collect();

    // Build LexDocs
    let docs: Vec<LexDoc> = matching.iter().map(|e| entity_lex_doc(e)).collect();

    if docs.is_empty() {
        writeln!(std::io::stdout(), "No results.")?;
        return Ok(());
    }

    // Fit BM25
    let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
    let corpus = LexicalCorpus::Raw(&docs);
    let ranker = Bm25Ranker;
    let scored = ranker.score(Some(&args.query), &corpus, &ids);

    // Filter zero scores, sort descending, apply offset/limit
    let mut results: Vec<_> = scored.into_iter().filter(|(_, score)| *score > 0).collect();
    results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let total = results.len();
    let offset = args.offset.min(total);
    let limit = args.limit.min(total.saturating_sub(offset));
    let page: Vec<_> = results.into_iter().skip(offset).take(limit).collect();

    // Build entity map for output fields
    let entity_map: BTreeMap<String, &CatalogEntity> = catalog
        .entities
        .iter()
        .map(|e| {
            let id = match &e.key {
                CatalogKey::Numbered(k) => k.canonical(),
                CatalogKey::Memory(uid) => uid.clone(),
            };
            (id, e)
        })
        .collect();

    match args.format {
        Format::Table => {
            // Build rows from the scored page (no Score column).
            let rows: Vec<SearchRow> = page
                .iter()
                .filter_map(|(id, _score)| {
                    let entity = entity_map.get(id)?;
                    let status = entity.status.as_deref().unwrap_or("-");
                    let kind_label = match &entity.key {
                        CatalogKey::Numbered(k) => k.prefix.to_lowercase(),
                        CatalogKey::Memory(_) => "mem".to_string(),
                    };
                    Some(SearchRow {
                        id: id.clone(),
                        kind: kind_label,
                        status: status.to_string(),
                        title: entity.title.clone(),
                    })
                })
                .collect();

            let cols: Vec<&Column<SearchRow>> = SEARCH_COLUMNS.iter().collect();
            let table = render_columns(&rows, &cols, render);
            writeln!(std::io::stdout(), "{table}")?;

            // Snippet lines for --context (indented below each row).
            if args.context {
                for (id, _score) in &page {
                    if let Some(entity) = entity_map.get(id) {
                        let body = entity.body.as_deref().unwrap_or(&entity.title);
                        let snip = snippet(body, &args.query, 40);
                        if !snip.is_empty() {
                            writeln!(std::io::stdout(), "  {snip}")?;
                        }
                    }
                }
            }

            if total > 0 {
                writeln!(std::io::stdout(), "{total} result(s)")?;
            } else {
                writeln!(std::io::stdout(), "No results.")?;
            }
        }
        Format::Json => {
            let json_results: Vec<serde_json::Value> = page
                .iter()
                .map(|(id, score)| {
                    let entity = entity_map.get(id);
                    build_json_hit(id, *score, entity, &args)
                })
                .collect();

            let output = serde_json::json!({
                "query": args.query,
                "count": json_results.len(),
                "total": total,
                "results": json_results,
            });
            writeln!(
                std::io::stdout(),
                "{}",
                serde_json::to_string_pretty(&output)?
            )?;
        }
    }

    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    reason = "test code"
)]
mod tests {
    use super::*;
    use crate::catalog::test_helpers::*;

    // ── KindSelector tests ────────────────────────────────────────────────

    #[test]
    fn kind_selector_default_has_correct_prefixes() {
        let ks = KindSelector::resolve(None, &[], &[]).unwrap();
        for prefix in DEFAULT_SEARCH_KINDS {
            assert!(ks.matches(prefix), "default should include {prefix}");
        }
        // IDE is in defaults — confirm it.
        assert!(ks.matches("IDE"), "default should include IDE");
        assert!(!ks.matches("REQ"), "default should not include REQ");
    }

    #[test]
    fn kind_selector_replace_works() {
        let ks = KindSelector::resolve(Some("adr,sl"), &[], &[]).unwrap();
        assert!(ks.matches("ADR"));
        assert!(ks.matches("SL"));
        assert!(!ks.matches("PRD"));
        assert_eq!(ks.prefixes.len(), 2);
    }

    #[test]
    fn kind_selector_add_remove() {
        // Start with default, add PRD, remove ADR
        let ks = KindSelector::resolve(None, &["prd".to_string()], &["adr".to_string()]).unwrap();
        assert!(ks.matches("PRD"));
        assert!(!ks.matches("ADR"));
        assert!(ks.matches("SL"));
    }

    #[test]
    fn kind_selector_unknown_prefix_errors() {
        let r = KindSelector::resolve(Some("ZZ,sl"), &[], &[]);
        assert!(r.is_err());
        let err = r.unwrap_err().to_string();
        assert!(err.contains("unknown kind prefix"), "got: {err}");
        assert!(err.contains("ZZ"), "got: {err}");
    }

    #[test]
    fn kind_selector_group_alias() {
        let ks = KindSelector::resolve(Some("backlog"), &[], &[]).unwrap();
        // backlog group: ISS, IMP, CHR, RSK, IDE
        assert!(ks.matches("ISS"));
        assert!(ks.matches("IMP"));
        assert!(ks.matches("CHR"));
        assert!(ks.matches("RSK"));
        assert!(ks.matches("IDE"));
        assert_eq!(ks.prefixes.len(), 5);
    }

    #[test]
    fn kind_selector_group_all_covers_known_prefixes() {
        let ks = KindSelector::resolve(Some("all"), &[], &[]).unwrap();
        let known: BTreeSet<String> = integrity::KINDS
            .iter()
            .map(|kr| kr.kind.prefix.to_string())
            .collect();
        for p in &known {
            assert!(ks.matches(p), "all should include {p}");
        }
    }

    #[test]
    fn kind_selector_matches_case_insensitive_input() {
        let ks = KindSelector::resolve(Some("sl"), &[], &[]).unwrap();
        assert!(ks.matches("SL"));
        let ks2 = KindSelector::resolve(Some("SL"), &[], &[]).unwrap();
        assert!(ks2.matches("SL"));
    }

    // ── entity_lex_doc tests ──────────────────────────────────────────────

    #[test]
    fn entity_lex_doc_with_body() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        let catalog = scan_catalog(root, ScanMode::include_bodies()).unwrap();
        let e = catalog
            .entities
            .iter()
            .find(|ce| matches!(&ce.key, CatalogKey::Numbered(k) if k.id == 1))
            .unwrap();

        let doc = entity_lex_doc(e);
        assert_eq!(doc.id, "SL-001");
        assert!(doc.text.contains("S1"));
        assert!(doc.text.contains("scope"));
    }

    #[test]
    fn entity_lex_doc_without_body() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        let catalog = scan_catalog(root, ScanMode::default()).unwrap();
        let e = catalog
            .entities
            .iter()
            .find(|ce| matches!(&ce.key, CatalogKey::Numbered(k) if k.id == 1))
            .unwrap();

        let doc = entity_lex_doc(e);
        assert_eq!(doc.id, "SL-001");
        assert_eq!(doc.text, "S1");
    }

    // ── snippet tests ─────────────────────────────────────────────────────

    #[test]
    fn snippet_matches_and_extracts_context() {
        let text = "The quick brown fox jumps over the lazy dog";
        let result = snippet(text, "fox", 10);
        assert!(
            result.contains("fox"),
            "result should contain fox: {result:?}"
        );
        assert!(
            result.starts_with('\u{2026}'),
            "should start with ellipsis: {result:?}"
        );
        assert!(
            result.ends_with('\u{2026}'),
            "should end with ellipsis: {result:?}"
        );
        assert!(
            result.contains("brown"),
            "context should include brown: {result:?}"
        );
    }

    #[test]
    fn snippet_no_match_returns_empty() {
        let result = snippet("hello world", "xyz", 10);
        assert!(result.is_empty());
    }

    #[test]
    fn snippet_empty_query_returns_empty() {
        let result = snippet("hello world", "", 10);
        assert!(result.is_empty());
    }

    #[test]
    fn snippet_at_start_no_leading_ellipsis() {
        let text = "fox jumps over the lazy dog";
        let result = snippet(text, "fox", 20);
        assert!(
            !result.starts_with('\u{2026}'),
            "no leading ellipsis at start: {result:?}"
        );
    }

    #[test]
    fn snippet_at_end_no_trailing_ellipsis() {
        let text = "the quick brown fox";
        let result = snippet(text, "fox", 20);
        assert!(
            !result.ends_with('\u{2026}'),
            "no trailing ellipsis at end: {result:?}"
        );
    }

    // ── search integration tests ──────────────────────────────────────────

    fn seed_search_fixture(root: &std::path::Path) {
        seed_slice(root, 1, &[]);
        seed_slice(root, 2, &[]);
        seed_adr(root, 1, &[]);
    }

    #[test]
    fn search_integration_finds_results() {
        let dir = tmp();
        let root = dir.path();
        seed_search_fixture(root);

        let catalog = scan_catalog(root, ScanMode::include_bodies()).unwrap();
        let selector = KindSelector::resolve(Some("sl,adr"), &[], &[]).unwrap();
        let matching: Vec<&CatalogEntity> = catalog
            .entities
            .iter()
            .filter(|e| match &e.key {
                CatalogKey::Numbered(k) => selector.matches(k.prefix),
                CatalogKey::Memory(_) => false,
            })
            .collect();
        let docs: Vec<LexDoc> = matching.iter().map(|e| entity_lex_doc(e)).collect();
        let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
        let corpus = LexicalCorpus::Raw(&docs);
        let ranker = Bm25Ranker;
        let scored = ranker.score(Some("scope"), &corpus, &ids);

        // "scope" is in all slice md bodies
        let positive: Vec<_> = scored.iter().filter(|(_, s)| *s > 0).collect();
        assert!(!positive.is_empty(), "should find matches for 'scope'");
        assert_eq!(positive.len(), 2, "both SL-001 and SL-002 have 'scope'");
    }

    #[test]
    fn search_json_format_produces_valid_json() {
        let dir = tmp();
        let root = dir.path();
        seed_search_fixture(root);

        let catalog = scan_catalog(root, ScanMode::include_bodies()).unwrap();
        let selector = KindSelector::resolve(Some("sl"), &[], &[]).unwrap();
        let matching: Vec<&CatalogEntity> = catalog
            .entities
            .iter()
            .filter(|e| match &e.key {
                CatalogKey::Numbered(k) => selector.matches(k.prefix),
                CatalogKey::Memory(_) => false,
            })
            .collect();
        let docs: Vec<LexDoc> = matching.iter().map(|e| entity_lex_doc(e)).collect();
        let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
        let corpus = LexicalCorpus::Raw(&docs);
        let ranker = Bm25Ranker;
        let scored = ranker.score(Some("scope"), &corpus, &ids);

        let mut results: Vec<_> = scored.into_iter().filter(|(_, s)| *s > 0).collect();
        results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let entity_map: BTreeMap<String, &CatalogEntity> = catalog
            .entities
            .iter()
            .map(|e| {
                let id = match &e.key {
                    CatalogKey::Numbered(k) => k.canonical(),
                    CatalogKey::Memory(uid) => uid.clone(),
                };
                (id, e)
            })
            .collect();

        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|(id, score)| {
                let entity = entity_map.get(id);
                build_json_hit(
                    id,
                    *score,
                    entity,
                    &SearchArgs {
                        query: "scope".to_string(),
                        kinds: None,
                        with: vec![],
                        no: vec![],
                        format: Format::Json,
                        context: false,
                        limit: 20,
                        offset: 0,
                        path: None,
                    },
                )
            })
            .collect();

        let output = serde_json::json!({
            "query": "scope",
            "count": json_results.len(),
            "total": results.len(),
            "results": json_results,
        });

        let json_str = serde_json::to_string_pretty(&output).unwrap();
        assert!(json_str.contains("\"query\""));
        assert!(json_str.contains("\"results\""));
        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    }

    #[test]
    fn search_empty_results() {
        let dir = tmp();
        let root = dir.path();
        seed_search_fixture(root);

        let catalog = scan_catalog(root, ScanMode::include_bodies()).unwrap();
        let selector = KindSelector::resolve(Some("sl"), &[], &[]).unwrap();
        let matching: Vec<&CatalogEntity> = catalog
            .entities
            .iter()
            .filter(|e| match &e.key {
                CatalogKey::Numbered(k) => selector.matches(k.prefix),
                CatalogKey::Memory(_) => false,
            })
            .collect();
        let docs: Vec<LexDoc> = matching.iter().map(|e| entity_lex_doc(e)).collect();
        let ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
        let corpus = LexicalCorpus::Raw(&docs);
        let ranker = Bm25Ranker;
        let scored = ranker.score(Some("nonexistentwordxyzzy"), &corpus, &ids);

        let positive: Vec<_> = scored.iter().filter(|(_, s)| *s > 0).collect();
        assert!(positive.is_empty(), "no results for nonexistent query");
    }
}
