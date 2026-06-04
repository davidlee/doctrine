// SPDX-License-Identifier: GPL-3.0-only
//! Memory retrieval — the reader over the SL-007 store (slice SL-008).
//!
//! PHASE-01: the **pure predicate layer**. Scope matching and the hard filters
//! that DROP disallowed memories before any ordering happens (design § 5.5,
//! review B1 — filters are predicates, never `Ord` keys). No clock, no git, no
//! disk: `today` arrives as a `&str`, the git/partition facts arrive as plain
//! data resolved at the shell (PHASE-04).
//!
//! Scope matching uses the **location-probe** model (SL-008 PHASE-01 decision):
//! the query is a working location (`paths` ∪ `globs`, treated as path subjects)
//! plus facets (`commands`, `tags`). A memory matches if its scope ADMITS that
//! location via any dimension; the highest-specificity dimension wins.
#![expect(
    dead_code,
    reason = "PHASE-01 pure predicate layer; its only caller is the PHASE-04 \
              shell (SL-008). The expectation self-clears when that lands."
)]

use std::collections::BTreeSet;

use crate::memory::{Memory, MemoryType, Status};

/// A `thread` memory is expired unless verified within this many days (design
/// D6 — the verification window, distinct from `staleness`'s 30-day boundary).
const THREAD_FRESH_DAYS: i64 = 14;

/// The scope dimension a memory matched on, carrying the spec's specificity
/// weight (memory-spec § Retrieval). Higher specificity ranks first (PHASE-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dimension {
    Paths,
    Globs,
    Commands,
    Tags,
}

impl Dimension {
    /// The spec's specificity weight: paths 3 > globs 2 > commands 1 > tags 0.
    pub(crate) fn specificity(self) -> u8 {
        match self {
            Self::Paths => 3,
            Self::Globs => 2,
            Self::Commands => 1,
            Self::Tags => 0,
        }
    }
}

/// The single highest-specificity dimension a query matched a memory on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScopeMatch {
    pub(crate) specificity: u8,
    pub(crate) dim: Dimension,
}

impl ScopeMatch {
    fn of(dim: Dimension) -> Self {
        Self {
            specificity: dim.specificity(),
            dim,
        }
    }
}

/// A retrieval query — the location + facets a caller probes the store with.
/// `query` (free-text lexical) is carried for PHASE-02 ranking; it is NOT a
/// scope constraint (design D15/B3).
#[derive(Debug, Clone, Default)]
pub(crate) struct QueryContext {
    pub(crate) paths: Vec<String>,
    pub(crate) globs: Vec<String>,
    pub(crate) commands: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) query: Option<String>,
}

impl QueryContext {
    /// Whether the query carries any *scope* constraint (a flag), as opposed to
    /// only free-text `--query`. A scope-bearing query excludes no-scope
    /// memories; a bare `--query` does not (design D15/D20/B3).
    pub(crate) fn has_scope_constraints(&self) -> bool {
        !self.paths.is_empty()
            || !self.globs.is_empty()
            || !self.commands.is_empty()
            || !self.tags.is_empty()
    }
}

/// The frozen partition coordinates a query runs against (design § 5.3). `repo`
/// is `None` outside a git repo. Frozen once per query at the shell (PHASE-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryPartition {
    pub(crate) workspace: String,
    pub(crate) repo: Option<String>,
}

/// Split a path into non-empty `/`-components (trailing/leading/`//` tolerant).
fn path_components(p: &str) -> Vec<&str> {
    p.split('/').filter(|s| !s.is_empty()).collect()
}

/// A scope path admits a query path iff it equals or is a component-prefix of it
/// (`src` admits `src/memory.rs`; `src` does NOT admit `srcfoo`). An empty scope
/// path admits nothing (defensive — it would otherwise admit everything).
fn path_admits(scope: &str, query: &str) -> bool {
    let s = path_components(scope);
    let q = path_components(query);
    !s.is_empty() && s.len() <= q.len() && s.iter().zip(&q).all(|(a, b)| a == b)
}

/// A scope glob (the pattern) matches a query path (`**`-aware via the `glob`
/// crate). A malformed stored pattern is treated as a non-match (the store is
/// tool-authored; defensive, never a hard error in the reader).
fn glob_admits(pattern: &str, query: &str) -> bool {
    glob::Pattern::new(pattern).is_ok_and(|p| p.matches(query))
}

/// A scope command admits a query command iff its whitespace tokens are a prefix
/// of the query's (`cargo` admits `cargo test`). Empty scope admits nothing.
fn command_admits(scope: &str, query: &str) -> bool {
    let s: Vec<&str> = scope.split_whitespace().collect();
    let q: Vec<&str> = query.split_whitespace().collect();
    !s.is_empty() && s.len() <= q.len() && s.iter().zip(&q).all(|(a, b)| a == b)
}

/// Match a query against a memory's scope, returning the highest-specificity
/// dimension that admitted the query's location/facets, or `None` (drop). The
/// query's path-like inputs (`paths` ∪ `globs`) probe BOTH `scope.paths`
/// (prefix → 3) and `scope.globs` (glob → 2); commands and tags probe their like
/// dimension. Checked highest-specificity first; first hit wins.
pub(crate) fn match_scope(m: &Memory, q: &QueryContext) -> Option<ScopeMatch> {
    let locations = || q.paths.iter().chain(q.globs.iter());

    if locations().any(|l| m.scope.paths.iter().any(|sp| path_admits(sp, l))) {
        return Some(ScopeMatch::of(Dimension::Paths));
    }
    if locations().any(|l| m.scope.globs.iter().any(|g| glob_admits(g, l))) {
        return Some(ScopeMatch::of(Dimension::Globs));
    }
    if q.commands
        .iter()
        .any(|c| m.scope.commands.iter().any(|sc| command_admits(sc, c)))
    {
        return Some(ScopeMatch::of(Dimension::Commands));
    }
    let scope_tags: BTreeSet<&str> = m.scope.tags.iter().map(String::as_str).collect();
    if q.tags.iter().any(|t| scope_tags.contains(t.as_str())) {
        return Some(ScopeMatch::of(Dimension::Tags));
    }
    None
}

/// Hard partition + lifecycle filter (design § 5.4, review B4/B20). DROPS a
/// memory outside the query's workspace/repo partition, and every lifecycle
/// status except `active` (and `draft` when `include_draft`). `quarantined` and
/// `retracted` are never admitted — there is no flag for them here.
pub(crate) fn base_filter(m: &Memory, part: &QueryPartition, include_draft: bool) -> bool {
    if m.scope.workspace != part.workspace {
        return false;
    }
    // Repo partition (B20): a repo-scoped memory needs the query's repo to equal
    // its repo; a repo-empty (global) memory is admitted in any partition.
    if !m.scope.repo.is_empty() && part.repo.as_deref() != Some(m.scope.repo.as_str()) {
        return false;
    }
    match m.status {
        Status::Active => true,
        Status::Draft => include_draft,
        Status::Superseded | Status::Archived | Status::Retracted | Status::Quarantined => false,
    }
}

/// Thread-expiry filter (design D6/B9). Runs AFTER `match_scope` — the
/// `ScopeMatch` proves the memory was scope-matched. A non-`thread` always
/// passes; a `thread` passes only if verified AND reviewed within
/// `THREAD_FRESH_DAYS` of `today`. An unparseable `reviewed` date drops it.
pub(crate) fn thread_expiry(m: &Memory, _matched: ScopeMatch, today: &str) -> bool {
    if m.kind != MemoryType::Thread {
        return true;
    }
    if m.verification_state != "verified" {
        return false;
    }
    match days_between(&m.reviewed, today) {
        Some(d) => (0..=THREAD_FRESH_DAYS).contains(&d),
        None => false,
    }
}

/// Parse a `YYYY-MM-DD` calendar date (no time, no zone). `None` on any
/// malformed component or trailing garbage. Pure — the lexer for the date axis.
fn parse_ymd(s: &str) -> Option<time::Date> {
    let mut it = s.split('-');
    let year: i32 = it.next()?.parse().ok()?;
    let month: u8 = it.next()?.parse().ok()?;
    let day: u8 = it.next()?.parse().ok()?;
    if it.next().is_some() {
        return None;
    }
    let month = time::Month::try_from(month).ok()?;
    time::Date::from_calendar_date(year, month, day).ok()
}

/// Whole days from `a` to `b` (`b - a`), both `YYYY-MM-DD`. `None` if either is
/// unparseable. Negative when `b` precedes `a`. Pure (design F3) — reused by
/// PHASE-02's staleness + review-recency sort key.
pub(crate) fn days_between(a: &str, b: &str) -> Option<i64> {
    let from = parse_ymd(a)?;
    let to = parse_ymd(b)?;
    Some((to - from).whole_days())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    // A valid uid for fixtures (32 lowercase hex after `mem_`).
    const UID: &str = "mem_018f3a1b2c3d4e5f60718293a4b5c6d7";

    /// A minimal valid `memory.toml`, parameterised for the field under test.
    /// Mirrors the SL-005 fixture shape so we exercise the real parser, not a
    /// hand-built struct (test behaviour over construction).
    struct Fixture {
        kind: &'static str,
        status: &'static str,
        workspace: &'static str,
        repo: &'static str,
        paths: &'static [&'static str],
        globs: &'static [&'static str],
        commands: &'static [&'static str],
        tags: &'static [&'static str],
        verification_state: &'static str,
        reviewed: &'static str,
    }

    impl Default for Fixture {
        fn default() -> Self {
            Self {
                kind: "fact",
                status: "active",
                workspace: "default",
                repo: "",
                paths: &[],
                globs: &[],
                commands: &[],
                tags: &[],
                verification_state: "unverified",
                reviewed: "2026-06-01",
            }
        }
    }

    fn toml_list(items: &[&str]) -> String {
        let inner = items
            .iter()
            .map(|s| format!("{s:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{inner}]")
    }

    fn memory(f: &Fixture) -> Memory {
        let text = format!(
            r#"
memory_uid = "{UID}"
schema_version = 1
memory_type = "{kind}"
status = "{status}"
title = "t"
summary = "s"
created = "2026-06-01"
updated = "2026-06-01"

[scope]
workspace = "{workspace}"
repo = "{repo}"
paths = {paths}
globs = {globs}
commands = {commands}
tags = {tags}

[review]
verification_state = "{vs}"
reviewed = "{reviewed}"
review_by = ""

[trust]
trust_level = "medium"

[ranking]
severity = "none"
weight = 0
"#,
            kind = f.kind,
            status = f.status,
            workspace = f.workspace,
            repo = f.repo,
            paths = toml_list(f.paths),
            globs = toml_list(f.globs),
            commands = toml_list(f.commands),
            tags = toml_list(f.tags),
            vs = f.verification_state,
            reviewed = f.reviewed,
        );
        Memory::parse(&text).unwrap()
    }

    fn q(paths: &[&str]) -> QueryContext {
        QueryContext {
            paths: paths.iter().map(|s| (*s).to_owned()).collect(),
            ..Default::default()
        }
    }

    // -- has_scope_constraints (EX-1) ---------------------------------------

    #[test]
    fn has_scope_constraints_query_alone_is_not_scope_bearing() {
        let mut c = QueryContext {
            query: Some("auth bug".to_owned()),
            ..Default::default()
        };
        assert!(!c.has_scope_constraints(), "free-text query is not scope");
        c.tags.push("rust".to_owned());
        assert!(c.has_scope_constraints(), "a tag is a scope constraint");
    }

    #[test]
    fn has_scope_constraints_any_dimension_counts() {
        for c in [
            q(&["src"]),
            QueryContext {
                globs: vec!["**/*.rs".to_owned()],
                ..Default::default()
            },
            QueryContext {
                commands: vec!["cargo".to_owned()],
                ..Default::default()
            },
            QueryContext {
                tags: vec!["x".to_owned()],
                ..Default::default()
            },
        ] {
            assert!(c.has_scope_constraints());
        }
        assert!(!QueryContext::default().has_scope_constraints());
    }

    // -- match_scope per-dimension semantics (EX-5 / VT-1 / VT-4) -----------

    #[test]
    fn paths_match_is_exact_or_component_prefix() {
        let m = memory(&Fixture {
            paths: &["src"],
            ..Default::default()
        });
        // exact
        assert!(
            match_scope(
                &memory(&Fixture {
                    paths: &["src/memory.rs"],
                    ..Default::default()
                }),
                &q(&["src/memory.rs"])
            )
            .is_some()
        );
        // component-prefix: scope `src` admits `src/memory.rs`
        let hit = match_scope(&m, &q(&["src/memory.rs"])).unwrap();
        assert_eq!(hit.dim, Dimension::Paths);
        assert_eq!(hit.specificity, 3);
        // near-miss: `src` must NOT admit `srcfoo`
        assert!(match_scope(&m, &q(&["srcfoo/x.rs"])).is_none());
    }

    #[test]
    fn globs_match_is_star_star_aware() {
        let m = memory(&Fixture {
            globs: &["src/**/*.rs"],
            ..Default::default()
        });
        let hit = match_scope(&m, &q(&["src/a/b/c.rs"])).unwrap();
        assert_eq!(hit.dim, Dimension::Globs);
        assert_eq!(hit.specificity, 2);
        assert!(match_scope(&m, &q(&["src/a/b/c.txt"])).is_none());
    }

    #[test]
    fn commands_match_is_token_prefix() {
        let m = memory(&Fixture {
            commands: &["cargo test"],
            ..Default::default()
        });
        let probe = QueryContext {
            commands: vec!["cargo test --release".to_owned()],
            ..Default::default()
        };
        let hit = match_scope(&m, &probe).unwrap();
        assert_eq!(hit.dim, Dimension::Commands);
        assert_eq!(hit.specificity, 1);
        // a non-prefix token does not match
        let miss = QueryContext {
            commands: vec!["cargo build".to_owned()],
            ..Default::default()
        };
        assert!(match_scope(&m, &miss).is_none());
    }

    #[test]
    fn tags_match_is_set_intersection() {
        let m = memory(&Fixture {
            tags: &["rust", "cli"],
            ..Default::default()
        });
        let hit = match_scope(
            &m,
            &QueryContext {
                tags: vec!["cli".to_owned()],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(hit.dim, Dimension::Tags);
        assert_eq!(hit.specificity, 0);
        let miss = QueryContext {
            tags: vec!["python".to_owned()],
            ..Default::default()
        };
        assert!(match_scope(&m, &miss).is_none());
    }

    #[test]
    fn highest_specificity_dimension_wins() {
        // scoped on both paths and tags; a query hitting both reports Paths (3).
        let m = memory(&Fixture {
            paths: &["src"],
            tags: &["rust"],
            ..Default::default()
        });
        let probe = QueryContext {
            paths: vec!["src/memory.rs".to_owned()],
            tags: vec!["rust".to_owned()],
            ..Default::default()
        };
        assert_eq!(match_scope(&m, &probe).unwrap().dim, Dimension::Paths);
    }

    #[test]
    fn no_scope_query_path_probes_memory_globs() {
        // location-probe: a query PATH is tested against memory GLOBS (spec 2).
        let m = memory(&Fixture {
            globs: &["src/**"],
            ..Default::default()
        });
        assert_eq!(
            match_scope(&m, &q(&["src/memory.rs"])).unwrap().dim,
            Dimension::Globs
        );
    }

    #[test]
    fn unscoped_memory_never_matches_a_scope_query() {
        let m = memory(&Fixture::default()); // no scope arrays
        assert!(match_scope(&m, &q(&["src/memory.rs"])).is_none());
    }

    // -- base_filter partition + lifecycle (EX-3 / VT-2) --------------------

    fn part(repo: Option<&str>) -> QueryPartition {
        QueryPartition {
            workspace: "default".to_owned(),
            repo: repo.map(str::to_owned),
        }
    }

    #[test]
    fn base_filter_drops_cross_workspace() {
        let m = memory(&Fixture {
            workspace: "other",
            ..Default::default()
        });
        assert!(!base_filter(&m, &part(None), false));
    }

    #[test]
    fn base_filter_repo_partition() {
        let repo_scoped = memory(&Fixture {
            repo: "repo:abc",
            ..Default::default()
        });
        // matching repo admits; wrong repo + None partition drop it
        assert!(base_filter(&repo_scoped, &part(Some("repo:abc")), false));
        assert!(!base_filter(&repo_scoped, &part(Some("repo:xyz")), false));
        assert!(!base_filter(&repo_scoped, &part(None), false));
        // a repo-empty (global) memory is admitted in any partition
        let global = memory(&Fixture::default());
        assert!(base_filter(&global, &part(None), false));
        assert!(base_filter(&global, &part(Some("repo:abc")), false));
    }

    #[test]
    fn base_filter_lifecycle() {
        let active = memory(&Fixture::default());
        assert!(base_filter(&active, &part(None), false));
        for bad in ["superseded", "archived", "retracted", "quarantined"] {
            let m = memory(&Fixture {
                status: bad,
                ..Default::default()
            });
            assert!(!base_filter(&m, &part(None), false), "{bad} must drop");
            assert!(
                !base_filter(&m, &part(None), true),
                "{bad} must drop even with include_draft"
            );
        }
    }

    #[test]
    fn base_filter_draft_is_gated() {
        let m = memory(&Fixture {
            status: "draft",
            ..Default::default()
        });
        assert!(!base_filter(&m, &part(None), false));
        assert!(base_filter(&m, &part(None), true));
    }

    // -- thread_expiry (EX-4 / VT-3) ----------------------------------------

    fn matched() -> ScopeMatch {
        ScopeMatch::of(Dimension::Paths)
    }

    #[test]
    fn thread_expiry_non_thread_always_passes() {
        let m = memory(&Fixture {
            kind: "fact",
            verification_state: "unverified",
            ..Default::default()
        });
        assert!(thread_expiry(&m, matched(), "2030-01-01"));
    }

    #[test]
    fn thread_expiry_requires_verified() {
        let m = memory(&Fixture {
            kind: "thread",
            verification_state: "unverified",
            reviewed: "2026-06-05",
            ..Default::default()
        });
        assert!(!thread_expiry(&m, matched(), "2026-06-05"));
    }

    #[test]
    fn thread_expiry_window_is_14_days() {
        let fresh = memory(&Fixture {
            kind: "thread",
            verification_state: "verified",
            reviewed: "2026-06-01",
            ..Default::default()
        });
        assert!(thread_expiry(&fresh, matched(), "2026-06-15")); // 14d, inclusive
        assert!(!thread_expiry(&fresh, matched(), "2026-06-16")); // 15d, expired
    }

    #[test]
    fn thread_expiry_unparseable_review_drops() {
        let m = memory(&Fixture {
            kind: "thread",
            verification_state: "verified",
            reviewed: "not-a-date",
            ..Default::default()
        });
        assert!(!thread_expiry(&m, matched(), "2026-06-05"));
    }

    // -- days_between (pure date primitive) ---------------------------------

    #[test]
    fn days_between_basics() {
        assert_eq!(days_between("2026-06-01", "2026-06-15"), Some(14));
        assert_eq!(days_between("2026-06-15", "2026-06-01"), Some(-14));
        assert_eq!(days_between("2026-06-01", "2026-06-01"), Some(0));
        assert_eq!(days_between("bad", "2026-06-01"), None);
        assert_eq!(days_between("2026-06-01", ""), None);
        assert_eq!(days_between("2026-13-01", "2026-06-01"), None); // bad month
    }
}
