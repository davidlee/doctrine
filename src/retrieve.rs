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
//!
//! PHASE-04 wires the impure shell (`freeze`/`query`/`run_find`) over this pure
//! core, so the module is no longer dead — the PHASE-01 `#![expect(dead_code)]`
//! is retired (its self-clearing condition has arrived).

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use serde::Serialize;

use crate::lexical::{Bm25Ranker, LexDoc, LexicalCorpus, LexicalRanker};
use crate::links::{extract_wikilinks, resolve_wikilink};
use crate::memory::{
    self, Lifespan, Memory, MemoryType, Status, collect_all, normalize_key, sort_default,
};

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

    /// The `spec` column label — the matched dimension name (PHASE-04 find rows).
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Paths => "paths",
            Self::Globs => "globs",
            Self::Commands => "commands",
            Self::Tags => "tags",
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
    pub(crate) lifespan: Option<Lifespan>,
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

fn lifespan_factor(lifespan: Option<Lifespan>) -> f64 {
    match lifespan {
        Some(Lifespan::Identity) => 0.0,
        Some(Lifespan::Semantic) => 0.1,
        Some(Lifespan::Procedural) => 0.33,
        Some(Lifespan::Episodic) | None => 1.0,
        Some(Lifespan::Working) => 10.0,
    }
}

fn effective_age(days: i64, lifespan: Option<Lifespan>) -> i64 {
    if days == i64::MAX {
        return i64::MAX;
    }
    #[expect(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "design requires floating scaling then round; no safe std i64→f64 API"
    )]
    let days_f64 = days as f64;
    let scaled = (days_f64 * lifespan_factor(lifespan)).round();
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "bounded finite float→i64 after round(); sentinel handled above"
    )]
    let age = scaled as i64;
    age.min(i64::MAX - 1)
}

// === PHASE-02: scoring, staleness & the total-order rank =====================
// The ordering core over the PHASE-01 filter survivors (design § 5.2). Pure: git
// arrives pre-resolved as `GitFacts`, `today` as a `&str`. Filters already dropped
// disallowed memories; rank never re-encodes a dropped one (review B1).

/// The §5.3 doc-bag projection of a `Memory` into the lexical leaf's `LexDoc`:
/// `id` = uid, `text` = `title summary tags key`, space-joined (body NOT scanned
/// in v1 — Q1/B15). The ONE adapter from `Memory` into the lexical axis (design
/// D6) — `query` builds the fit corpus through it, and the behaviour-preservation
/// parity test scores through it, so there is no parallel projection.
pub(crate) fn lex_doc(m: &Memory) -> LexDoc {
    let tags = m.scope.tags.join(" ");
    let text = [
        m.title.as_str(),
        m.summary.as_str(),
        tags.as_str(),
        m.key.as_deref().unwrap_or_default(),
    ]
    .join(" ");
    LexDoc {
        id: m.uid.clone(),
        text,
    }
}

/// FULL `memory_key` equality only (review B2): the normalized query equals the
/// memory's key. Segment/prefix overlap is the lexical ranker's job, not this. A
/// non-key-shaped query (`normalize_key` errors) is a non-match, never a fault
/// (B16). No query, or a keyless memory ⇒ false.
fn exact_key_match(m: &Memory, q: &QueryContext) -> bool {
    match (&m.key, q.query.as_deref()) {
        (Some(k), Some(query)) => normalize_key(query).ok().as_deref() == Some(k.as_str()),
        _ => false,
    }
}

/// The explicit staleness state surfaced on both query surfaces (never a silent
/// hide — design D19/§5.5). `Unknown` = undecidable; `Unanchored` = no git basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Staleness {
    Fresh,
    Stale,
    Unknown,
    Unanchored,
    /// The global/orientation class (ADR-002): evergreen / non-decaying. Derived
    /// from the class signature, never stored — exempt from days-since-`reviewed`.
    Reference,
}

impl Staleness {
    /// The `staleness` column / header label (PHASE-04 find + PHASE-05 retrieve).
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::Unknown => "unknown",
            Self::Unanchored => "unanchored",
            Self::Reference => "reference",
        }
    }
}

/// Git reachability fact for one candidate, resolved at the shell (PHASE-04) and
/// crossing the pure seam as plain data (design D3). `commits_since` counts
/// commits touching the scoped paths since `verified_sha`; `None` = undecidable
/// (non-ancestor anchor, shallow clone, no target, exec failure — review B18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct GitFacts {
    pub(crate) commits_since: Option<u32>,
}

/// Time-based fresh/stale boundary, inclusive (design § 5.2). Distinct from the
/// 14-day thread verification window.
const FRESH_DAYS: i64 = 30;

/// The global/orientation class signature for the evergreen disposition (ADR-002):
/// `repo=""` (global), `anchor_kind=none` (unanchored), **path/glob/command-scoped**
/// (≥1 — a tag-only global is illegal, so tags do NOT count), and unattested (no
/// `verified_sha` — an attested member earns commit mode instead). The scope floor
/// is load-bearing: it is what distinguishes a class member from a plain unanchored
/// `repo=""` memory, so the special-case never perturbs the latter's decay.
fn is_global_reference(m: &Memory) -> bool {
    let scoped =
        !m.scope.paths.is_empty() || !m.scope.globs.is_empty() || !m.scope.commands.is_empty();
    scoped
        && m.scope.repo.is_empty()
        && m.anchor.kind == crate::git::AnchorKind::None
        && m.anchor.verified_sha.is_empty()
}

/// Staleness by the spec's modes → five explicit states (design § 5.5/§5.4,
/// review F6/B5/B6; SL-018 ADR-002 adds Reference). First match wins, keyed on
/// **attestation**, not `anchor_kind`:
///  1. scoped (`!paths.is_empty()`) + attested (`verified_sha` set) ⇒ commit mode:
///     `commits_since` `Some(0)` Fresh / `Some(≥1)` Stale / `None` Unknown (never
///     Fresh — undecidable reachability). Target absence reaches here as `None`.
///  2. else the global/orientation class — `repo=""` + anchor=none + path/glob/
///     command-scoped + unattested (ADR-002 signature) ⇒ evergreen `Reference`,
///     exempt from the days-since-`reviewed` decay that would brand it `stale`.
///  3. else a parseable `reviewed` ⇒ time mode: `≤ FRESH_DAYS` Fresh, else Stale.
///  4. else git-anchored (`kind != None`) with no usable date ⇒ Unknown.
///  5. else no anchor at all ⇒ Unanchored.
///
/// A memory recorded dirty then `verify`-attested clean lands in branch 1 via its
/// `verified_sha` (verify refuses a dirty tree, so the SHA is always clean). The
/// Reference branch sits AFTER branch 1 (an attested global memory still earns
/// commit mode) and BEFORE the time branch (or the evergreen corpus would decay).
fn staleness(m: &Memory, facts: GitFacts, today: &str) -> Staleness {
    if !m.scope.paths.is_empty() && !m.anchor.verified_sha.is_empty() {
        return match facts.commits_since {
            Some(0) => Staleness::Fresh,
            Some(_) => Staleness::Stale,
            None => Staleness::Unknown,
        };
    }
    if is_global_reference(m) {
        return Staleness::Reference;
    }
    if let Some(days) = days_between(&m.reviewed, today) {
        return if days <= FRESH_DAYS {
            Staleness::Fresh
        } else {
            Staleness::Stale
        };
    }
    if m.anchor.kind == crate::git::AnchorKind::None {
        Staleness::Unanchored
    } else {
        Staleness::Unknown
    }
}

/// `verification_state` → bounded ordinal, lower ranks first (design § 5.2):
/// verified < unverified < stale < disputed; any unknown string ⇒ worst bucket
/// (review B12 — never silently ranked best).
fn verification_rank(s: &str) -> u8 {
    match s {
        "verified" => 0,
        "unverified" => 1,
        "stale" => 2,
        "disputed" => 3,
        _ => 4,
    }
}

/// `trust_level` → bounded ordinal: high < medium < low; unknown ⇒ worst (B12/B13).
fn trust_rank(s: &str) -> u8 {
    match s {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
    }
}

/// `severity` → bounded ordinal: critical < high < medium < low < none; unknown
/// ⇒ worst bucket (B12/B13).
fn severity_rank(s: &str) -> u8 {
    match s {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        "none" => 4,
        _ => 5,
    }
}

/// A filter survivor with its per-query derived signals, ready to rank. Borrows
/// the `Memory` (sort is by computed key, never the ref). `scope_match` is `None`
/// for a bare `--query` (specificity 0 — design D20).
pub(crate) struct Candidate<'a> {
    pub(crate) memory: &'a Memory,
    pub(crate) scope_match: Option<ScopeMatch>,
    pub(crate) staleness: Staleness,
    pub(crate) lexical: u32,
    pub(crate) exact_key: bool,
}

impl<'a> Candidate<'a> {
    /// Assemble a candidate, precomputing the lexical + exact-key + staleness
    /// signals from the resolved git facts (the only impure input, as data).
    pub(crate) fn new(
        m: &'a Memory,
        scope_match: Option<ScopeMatch>,
        q: &QueryContext,
        facts: GitFacts,
        today: &str,
        lexical: u32,
    ) -> Self {
        Self {
            memory: m,
            scope_match,
            staleness: staleness(m, facts, today),
            lexical,
            exact_key: exact_key_match(m, q),
        }
    }
}

/// The 9-key total-order sort key (design § 5.2 table), `today` frozen by the
/// caller. Polarity is load-bearing — asserted per key in tests; do not flip.
/// `uid` then `memory_key` is the final tiebreak ⇒ scan order never perturbs
/// output (review D5). Missing/malformed `reviewed` recency sorts last (`MAX`).
type SortKey<'a> = (
    bool,         // 1: !exact_key — exact-key hit first
    Reverse<u32>, // 2: lexical — descending
    Reverse<u8>,  // 3: scope specificity — descending (0 when unscoped)
    u8,           // 4: verification — verified→disputed
    u8,           // 5: trust — high→low
    u8,           // 6: severity — critical→none
    Reverse<i64>, // 7: weight — descending
    i64,          // 8: review recency (age in days) — fewer first, missing last
    &'a str,      // 9a: uid — ascending
    &'a str,      // 9b: memory_key — ascending
);

fn sort_key<'a>(c: &Candidate<'a>, today: &str) -> SortKey<'a> {
    let m = c.memory;
    (
        !c.exact_key,
        Reverse(c.lexical),
        Reverse(c.scope_match.map_or(0, |s| s.specificity)),
        verification_rank(&m.verification_state),
        trust_rank(&m.trust_level),
        severity_rank(&m.severity),
        Reverse(m.weight),
        effective_age(
            days_between(&m.reviewed, today).unwrap_or(i64::MAX),
            m.lifespan,
        ),
        m.uid.as_str(),
        m.key.as_deref().unwrap_or(""),
    )
}

/// Total-order rank over filter survivors (design § 5.2). Deterministic: a
/// shuffled input yields identical output (the `uid` tiebreak — property test).
pub(crate) fn rank<'a>(mut cands: Vec<Candidate<'a>>, today: &str) -> Vec<Candidate<'a>> {
    cands.sort_by(|a, b| sort_key(a, today).cmp(&sort_key(b, today)));
    cands
}

// === PHASE-04: the impure shell (find) + the shared query pipeline ===========
// Frozen-once-per-query git/clock facts cross into the pure core above as data
// (design §5.1). The pure layer never reads a clock, git, or disk; this shell is
// the only place that does. `retrieve` (PHASE-05) reuses `freeze` + `query`.

use std::path::{Path, PathBuf};

use anyhow::Result;

/// The git/clock facts frozen once per query (design §5.1/§5.2). `target` is the
/// born-frame `base_commit` (HEAD even on a dirty tree — D1/D9/B19), `None`
/// outside a usable git context; a `capture` error degrades to `None` rather than
/// failing the whole query (B18/B19).
pub(crate) struct Snapshot {
    pub(crate) today: String,
    pub(crate) target: Option<String>,
    pub(crate) part: QueryPartition,
}

/// Freeze the query snapshot at the shell edge. `capture(root).ok()` swallows an
/// unstable-frame error (multi-root/submodule/…) into a degraded `target: None` +
/// `repo: None` — a thinner, visibly-`Unknown` result set, never a hard failure
/// (B18/B19). The single git capture + the single clock read per query.
pub(crate) fn freeze(root: &Path) -> Snapshot {
    let frame = crate::git::capture(root).ok();
    let target = frame
        .as_ref()
        .map(|f| f.base_commit.clone())
        .filter(|s| !s.is_empty());
    let repo = frame
        .as_ref()
        .map(|f| f.repo.repo_id.clone())
        .filter(|s| !s.is_empty());
    Snapshot {
        today: crate::clock::today(),
        target,
        part: QueryPartition {
            workspace: crate::memory::WORKSPACE.to_owned(),
            repo,
        },
    }
}

/// Resolve the per-candidate git facts under the §5.1 gate: a candidate counts
/// commits only when it is path-scoped, attested (`verified_sha`), and the
/// snapshot froze a target. Otherwise no subprocess — `commits_since: None`
/// (staleness falls to a non-commit mode). A `commits_touching` failure is
/// per-candidate (`None` ⇒ `Staleness::Unknown`), never a query abort (B18).
fn git_facts(root: &Path, m: &Memory, snap: &Snapshot) -> GitFacts {
    if m.scope.paths.is_empty() || m.anchor.verified_sha.is_empty() {
        return GitFacts::default();
    }
    let Some(target) = snap.target.as_deref() else {
        return GitFacts::default();
    };
    GitFacts {
        commits_since: crate::git::commits_touching(
            root,
            &m.scope.paths,
            &m.anchor.verified_sha,
            target,
        ),
    }
}

/// The shared query pipeline (design §5.1, review B9) — surface-agnostic so
/// `retrieve` reuses it (EX-6/F3). Borrows an owned `&[Memory]` the caller holds;
/// returns the ranked survivors. Each memory runs the filter cascade
/// `base_filter → match_scope → thread_expiry`, then crosses into the ordering
/// core as a `Candidate` carrying its derived signals. A scope-bearing query
/// requires a `match_scope` hit; a bare `--query` keeps every survivor with no
/// scope match (specificity 0, D20).
pub(crate) fn query<'a>(
    mems: &'a [Memory],
    q: &QueryContext,
    snap: &Snapshot,
    include_draft: bool,
    root: &Path,
    ranker: &dyn LexicalRanker,
) -> Vec<Candidate<'a>> {
    let scoped = q.has_scope_constraints();
    // The fit corpus: every `base_filter` survivor (honouring `include_draft`).
    // BM25 df/avgdl fit over this active set; the narrower survivor subset is then
    // scored against it (design §5.3 — fit corpus ⊇ scored targets).
    let active: Vec<&'a Memory> = mems
        .iter()
        .filter(|m| base_filter(m, &snap.part, include_draft))
        .collect();
    let docs: Vec<LexDoc> = active.iter().map(|m| lex_doc(m)).collect();
    let corpus = LexicalCorpus::Raw(&docs);
    // Survivors: the active set further narrowed by `match_scope` + `thread_expiry`,
    // each carrying its (Copy) ScopeMatch. survivors ⊆ active = corpus, so the
    // ranker's hard `targets ⊆ corpus` precondition holds by construction.
    let survivors: Vec<(&'a Memory, Option<ScopeMatch>)> = active
        .iter()
        .filter_map(|&m| {
            let scope_match = match_scope(m, q);
            // Scope-bearing query: a non-match drops. Bare --query: keep (None).
            if scoped && scope_match.is_none() {
                return None;
            }
            if q.lifespan.is_some() && m.lifespan != q.lifespan {
                return None;
            }
            // thread_expiry reads only `m`; its ScopeMatch arg is vestigial.
            let probe = scope_match.unwrap_or_else(|| ScopeMatch::of(Dimension::Tags));
            if !thread_expiry(m, probe, &snap.today) {
                return None;
            }
            Some((m, scope_match))
        })
        .collect();
    let targets: Vec<&str> = survivors.iter().map(|(m, _)| m.uid.as_str()).collect();
    // One `(uid, u32)` per survivor, in order — A1 totality, indexed POSITIONALLY
    // (never `unwrap_or`: an absent entry is a ranker-contract bug, not a 0).
    let scores = ranker.score(q.query.as_deref(), &corpus, &targets);
    let cands: Vec<Candidate<'a>> = survivors
        .iter()
        .zip(scores.iter())
        .map(|(&(m, scope_match), &(_, lexical))| {
            let facts = git_facts(root, m, snap);
            Candidate::new(m, scope_match, q, facts, &snap.today, lexical)
        })
        .collect();
    rank(cands, &snap.today)
}

/// Format `find` rows: aligned `uid type status staleness trust sev spec title`.
/// The **full** uid is printed (F-A11 — actionable for `show`/`verify`, and v7
/// short prefixes collide, F-A12), overriding design §5.2's `uid-short` wording.
/// `trust`+`sev` are always present so a holdback-exempt `find` keeps risk visible
/// (B8/D8/D17). `spec` is the matched dimension (`-` for a bare `--query`). Every
/// free value is `scrub_line`d (F-A10) so a newline cannot forge a row.
/// Render find results as a human-readable column-aligned table.
fn format_find_table(cands: &[&Candidate<'_>]) -> String {
    let scrub = |s: &str| crate::memory::scrub_line(s);
    let rows: Vec<[String; 8]> = cands
        .iter()
        .map(|c| {
            let m = c.memory;
            [
                m.uid.clone(),
                m.kind.as_str().to_owned(),
                m.status.as_str().to_owned(),
                c.staleness.label().to_owned(),
                scrub(&m.trust_level),
                scrub(&m.severity),
                c.scope_match.map_or("-", |s| s.dim.label()).to_owned(),
                scrub(&m.title),
            ]
        })
        .collect();
    // Width-align every column but the last (title) for a scannable table. The
    // zips stop at `widths.len()` (7), so the title (`r`'s 8th cell) is excluded.
    let mut widths = [0usize; 7];
    for r in &rows {
        for (w, cell) in widths.iter_mut().zip(r.iter()) {
            *w = (*w).max(cell.len());
        }
    }
    let lines: Vec<String> = rows
        .iter()
        .map(|r| {
            let mut parts: Vec<String> = r
                .iter()
                .take(widths.len())
                .zip(widths.iter())
                .map(|(cell, w)| format!("{cell:<w$}", w = *w))
                .collect();
            if let Some(title) = r.last() {
                parts.push(title.clone());
            }
            parts.join("  ")
        })
        .collect();
    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    }
}

/// A serde row for `memory find --json`, mirroring the find table columns.
#[derive(Serialize)]
struct MemoryFindRow {
    uid: String,
    #[serde(rename = "type")]
    kind: String,
    status: String,
    staleness: String,
    trust: String,
    severity: String,
    spec: String,
    title: String,
}

impl From<&&Candidate<'_>> for MemoryFindRow {
    fn from(c: &&Candidate<'_>) -> Self {
        let m = c.memory;
        MemoryFindRow {
            uid: m.uid.clone(),
            kind: m.kind.as_str().to_owned(),
            status: m.status.as_str().to_owned(),
            staleness: c.staleness.label().to_owned(),
            trust: crate::memory::scrub_line(&m.trust_level),
            severity: crate::memory::scrub_line(&m.severity),
            spec: c.scope_match.map_or("-", |s| s.dim.label()).to_owned(),
            title: crate::memory::scrub_line(&m.title),
        }
    }
}

/// Render find results as a JSON envelope: `{ "kind": "memory_find", "rows": […] }`.
fn format_find_json(cands: &[&Candidate<'_>]) -> Result<String> {
    let rows: Vec<MemoryFindRow> = cands.iter().map(MemoryFindRow::from).collect();
    crate::listing::json_envelope("memory_find", &rows)
}

/// The frozen-snapshot bundle the `find`/`retrieve` verbs both stand on. The two
/// surfaces diverge ONLY after `query()` (find renders rows, retrieve renders
/// framed blocks), so everything up to the snapshot is one path — see `load_query`.
struct Loaded {
    root: PathBuf,
    mems: Vec<Memory>,
    q: QueryContext,
    snap: Snapshot,
}

/// The shared shell prologue for the query verbs (no parallel impl — CLAUDE.md):
/// resolve the root, collect + hard-filter the store (`--type`/`--status` ride the
/// existing `select_rows` AND-filter — F2; `--tag` is a SCOPE dimension, NOT the
/// `select_rows` tag filter, so it passes `None`), build the `QueryContext`, and
/// `freeze` the snapshot once. Callers run `query()` then render their own surface.
#[expect(clippy::too_many_arguments, reason = "CLI surface fans flags 1:1")]
fn load_query(
    path: Option<PathBuf>,
    paths: Vec<String>,
    globs: Vec<String>,
    commands: Vec<String>,
    tags: Vec<String>,
    lifespan: Option<Lifespan>,
    free_query: Option<String>,
    type_f: Option<MemoryType>,
    status_f: Option<Status>,
) -> Result<Loaded> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mems =
        crate::memory::select_rows(crate::memory::collect_all(&root)?, type_f, status_f, None);
    let q = QueryContext {
        paths,
        globs,
        commands,
        tags,
        lifespan,
        query: free_query,
    };
    let snap = freeze(&root);
    Ok(Loaded {
        root,
        mems,
        q,
        snap,
    })
}

/// `doctrine memory find [--path-scope/--glob/--command/--tag/--query] [--type]
/// [--status] [--include-draft] [--format] [--json] [--offset] [--page]
/// [--limit]`. The find surface over the shared pipeline: `load_query` → `query`
/// → paginate → render. find applies NO holdback (D8/D17); `base_filter`
/// already excludes quarantined/retracted/superseded/archived (and draft unless
/// `--include-draft`). It needs no body read (find renders rows, not framed
/// bodies).
#[expect(clippy::too_many_arguments, reason = "CLI surface fans flags 1:1")]
pub(crate) fn run_find(
    path: Option<PathBuf>,
    paths: Vec<String>,
    globs: Vec<String>,
    commands: Vec<String>,
    tags: Vec<String>,
    lifespan: Option<Lifespan>,
    free_query: Option<String>,
    type_f: Option<MemoryType>,
    status_f: Option<Status>,
    include_draft: bool,
    format: crate::listing::Format,
    offset: usize,
    limit: Option<usize>,
) -> Result<()> {
    let Loaded {
        root,
        mems,
        q,
        snap,
        ..
    } = load_query(
        path, paths, globs, commands, tags, lifespan, free_query, type_f, status_f,
    )?;
    // BM25 is the hard default on both surfaces — no user-facing selector (D5).
    let ranker = Bm25Ranker;
    let ranked = query(&mems, &q, &snap, include_draft, &root, &ranker);
    // Total = all candidates (holdback-exempt for find — D6).
    let total = ranked.len();
    // Paginate: skip(offset).take(limit).
    let visible: Vec<&Candidate<'_>> = ranked
        .iter()
        .skip(offset)
        .take(limit.unwrap_or(usize::MAX))
        .collect();
    let shown = visible.len();
    let mut parts: Vec<String> = Vec::new();
    let body = match format {
        crate::listing::Format::Table => format_find_table(&visible),
        crate::listing::Format::Json => format_find_json(&visible)?,
    };
    parts.push(body);
    // Truncation notice: table mode only, when results are truncated or offset exceeds total.
    if format == crate::listing::Format::Table && shown < total {
        let page_size = limit.unwrap_or(RETRIEVE_LIMIT_DEFAULT);
        parts.push(format_truncation_notice(shown, total, offset, page_size));
    }
    let output = parts.concat();
    write!(io::stdout(), "{output}")?;
    Ok(())
}

// === PHASE-05: the retrieve surface (agent-context boundary) =================
// The bounded, security-framed read. Reuses the PHASE-04 shell verbatim (freeze +
// query — same survivors as find pre-limit), then layers the retrieve-only
// concerns: the trust holdback (pre-render), the limit, and the per-block framed
// render with a fresh nonce each (design §5.1, D2/D8/D17/D19/B7/B10).

/// `--limit` default — an agent-context boundary is bounded by default (B10).
pub(crate) const RETRIEVE_LIMIT_DEFAULT: usize = 5;
/// `--limit` cap — a single query cannot flood the context (D17).
pub(crate) const RETRIEVE_LIMIT_MAX: usize = 20;

/// Validate a `--min-trust` value at the CLI edge (clap `value_parser`). Only the
/// three trust tiers are floors; anything else is a hard error, never a silent
/// worst-rank default.
pub(crate) fn parse_min_trust(s: &str) -> std::result::Result<String, String> {
    match s {
        "high" | "medium" | "low" => Ok(s.to_owned()),
        other => Err(format!(
            "invalid trust level {other:?} (expected high|medium|low)"
        )),
    }
}

/// The trust-floor rank for the holdback (D8). Default `medium` (rank 1) suppresses
/// `low ∧ severity≥high`. `--min-trust L` may only RAISE the floor (require more
/// trust): the `min` clamps a lower-trust request back to the default, so the
/// holdback is non-bypassable downward (B7) — `--min-trust low` is a no-op.
fn holdback_floor(min_trust: Option<&str>) -> u8 {
    let default = trust_rank("medium");
    min_trust.map_or(default, |l| trust_rank(l).min(default))
}

/// The trust holdback (design §5.4, D8): a memory is held back from `retrieve`
/// when it is high-severity AND less trusted than the floor. Read pre-render off
/// the `Memory` fields — a held-back body is never read, never framed (B7). `find`
/// is exempt (it annotates risk instead — the D8 asymmetry).
fn held_back(m: &Memory, floor: u8) -> bool {
    severity_rank(&m.severity) <= severity_rank("high") && trust_rank(&m.trust_level) > floor
}

/// Format a truncation notice line for table-mode output when results are
/// truncated or the offset exceeds the total. Returns empty string when
/// `total == 0` (empty result set — nothing to paginate).
fn format_truncation_notice(shown: usize, total: usize, offset: usize, page_size: usize) -> String {
    if total == 0 {
        return String::new();
    }
    if offset >= total {
        return format!(
            "{shown} of {total}; no results at this offset; reduce --offset or --page\n"
        );
    }
    #[expect(
        clippy::integer_division,
        reason = "floor division for 1-based page calc"
    )]
    let next_page = (offset / page_size) + 2; // 1-based
    format!("{shown} of {total}; use --page {next_page} for next or specify a higher --limit\n")
}

/// A serde row for `memory retrieve --json`.
#[derive(Serialize)]
struct MemoryRetrieveRow {
    uid: String,
    #[serde(rename = "type")]
    kind: String,
    status: String,
    staleness: String,
    trust: String,
    severity: String,
    title: String,
}

impl From<&Candidate<'_>> for MemoryRetrieveRow {
    fn from(c: &Candidate<'_>) -> Self {
        let m = c.memory;
        MemoryRetrieveRow {
            uid: m.uid.clone(),
            kind: m.kind.as_str().to_owned(),
            status: m.status.as_str().to_owned(),
            staleness: c.staleness.label().to_owned(),
            trust: crate::memory::scrub_line(&m.trust_level),
            severity: crate::memory::scrub_line(&m.severity),
            title: crate::memory::scrub_line(&m.title),
        }
    }
}

/// Render retrieve results as a JSON envelope: `{ "kind": "memory_retrieve", "rows": […] }`.
fn format_retrieve_json(cands: &[&Candidate<'_>]) -> Result<String> {
    let rows: Vec<MemoryRetrieveRow> = cands.iter().map(|c| MemoryRetrieveRow::from(*c)).collect();
    crate::listing::json_envelope("memory_retrieve", &rows)
}

/// `doctrine memory retrieve <query/filter flags> [--limit N] [--min-trust L]
/// [--offset N] [--page N] [--format F] [--json]`.
/// The agent-context surface over the shared pipeline: collect → `select_rows` →
/// `freeze` → `query` (identical survivor set to `find` pre-limit), then the
/// retrieve-only layer — the trust floor suppresses held-back memories PRE-render
/// (B7/D8), `skip(offset).take(limit)` over the survivors (holdback-then-offset-
/// then-limit), and per hit `render_show` with a FRESHLY minted nonce each (D2)
/// plus a `staleness:` header line (D19). Bodies are read lazily for the ≤limit
/// shown hits only. Under `--json`, output is a structured envelope and the
/// truncation notice is suppressed (D4).
#[expect(clippy::too_many_arguments, reason = "CLI surface fans flags 1:1")]
pub(crate) fn run_retrieve(
    path: Option<PathBuf>,
    paths: Vec<String>,
    globs: Vec<String>,
    commands: Vec<String>,
    tags: Vec<String>,
    lifespan: Option<Lifespan>,
    free_query: Option<String>,
    type_f: Option<MemoryType>,
    status_f: Option<Status>,
    include_draft: bool,
    limit: usize,
    min_trust: Option<&str>,
    offset: usize,
    format: crate::listing::Format,
    expand: Option<usize>,
) -> Result<()> {
    let Loaded {
        root,
        mems,
        q,
        snap,
    } = load_query(
        path, paths, globs, commands, tags, lifespan, free_query, type_f, status_f,
    )?;
    // BM25 is the hard default on both surfaces — no user-facing selector (D5).
    let ranker = Bm25Ranker;
    let ranked = query(&mems, &q, &snap, include_draft, &root, &ranker);

    let floor = holdback_floor(min_trust);
    // Holdback filter first, THEN count total, THEN offset + limit.
    // Total = post-holdback count (D6).
    let eligible: Vec<&Candidate<'_>> = ranked
        .iter()
        .filter(|c| !held_back(c.memory, floor))
        .collect();
    let total = eligible.len();
    let visible: Vec<&Candidate<'_>> = eligible.iter().skip(offset).take(limit).copied().collect();
    let shown = visible.len();
    let mut parts: Vec<String> = Vec::new();
    match format {
        crate::listing::Format::Table => {
            for c in &visible {
                let body = crate::memory::read_body(&root, &c.memory.uid);
                // FRESH nonce per BLOCK: one nonce across N bodies lets body i forge
                // body i+1's close (D2). Minted inside the loop, never hoisted.
                let nonce = uuid::Uuid::new_v4().simple().to_string();
                parts.push(crate::memory::render_show(
                    c.memory,
                    &body,
                    &nonce,
                    Some(c.staleness.label()),
                    &[],
                ));
            }
            // Truncation notice: suppressed under --json (D4).
            if shown < total {
                parts.push(format_truncation_notice(shown, total, offset, limit));
            }
        }
        crate::listing::Format::Json => {
            parts.push(format_retrieve_json(&visible)?);
        }
    }
    let output = parts.concat();
    write!(io::stdout(), "{output}")?;

    // PHASE-06: --expand N graph expansion
    if let Some(expand_depth) = expand {
        expand_graph(&visible, expand_depth, &root)?;
    }

    Ok(())
}

/// PHASE-06: Graph expansion for --expand N flag
fn expand_graph(visible: &[&Candidate<'_>], max_depth: usize, root: &Path) -> Result<()> {
    // Build edge set from all memories
    let all_memories = collect_all(root)?;
    let mut edges = BTreeMap::new();

    // Build key_to_uid map for resolving wikilink key-form targets
    let key_to_uid: BTreeMap<String, String> = all_memories
        .iter()
        .filter_map(|m| m.key.as_ref().map(|k| (normalize_key(k).unwrap_or_default(), m.uid.clone())))
        .collect();
    let known_uids: BTreeSet<String> = all_memories.iter().map(|m| m.uid.clone()).collect();

    for memory in &all_memories {
        let mut targets = Vec::new();

        // Add wikilink targets — resolve key-form targets to uids
        let body_path = root
            .join("memory/items")
            .join(&memory.uid)
            .join("memory.md");
        let body = std::fs::read_to_string(body_path).unwrap_or_default();
        let wikilinks = extract_wikilinks(&body);
        for link in wikilinks {
            if let Ok(resolved) =
                resolve_wikilink(&known_uids, &key_to_uid, &link.target, link.is_uid)
            {
                targets.push(resolved);
            }
            // Dangling wikilinks are silently skipped
        }

        // Add relation targets
        for relation in &memory.relations {
            targets.push(relation.target.clone());
        }

        edges.insert(memory.uid.clone(), targets);
    }

    // Convert visible candidates to starting uids
    let start_uids: Vec<String> = visible.iter().map(|c| c.memory.uid.clone()).collect();

    // Perform BFS expansion
    let expanded = bfs_expand(&edges, start_uids, max_depth);

    // Render expanded nodes by depth
    let mut first = true;
    for (depth, uids) in expanded.iter().enumerate() {
        if uids.is_empty() {
            continue;
        }

        if !first {
            writeln!(io::stdout())?; // blank line separator between depth groups
        }
        first = false;

        // Collect memories for this depth
        let mut depth_memories: Vec<Memory> = all_memories
            .iter()
            .filter(|m| uids.contains(&m.uid))
            .cloned()
            .collect();

        // Sort by default ordering
        sort_default(&mut depth_memories);

        // Render each memory in this depth
        for memory in &depth_memories {
            let depth_num = depth + 1;
            let staleness_line = format!("depth: {depth_num}");

            // Read body from disk
            let body_path = root
                .join("memory/items")
                .join(&memory.uid)
                .join("memory.md");
            let body = std::fs::read_to_string(body_path).unwrap_or_default();

            // Reuse render_show with proper parameters
            let guard = "memory-show";
            let wikilinks = Vec::new(); // Empty for now
            let rendered =
                memory::render_show(memory, &body, guard, Some(&staleness_line), &wikilinks);
            write!(io::stdout(), "{rendered}")?;
        }
    }

    Ok(())
}

/// BFS expansion following memory-to-memory edges only
fn bfs_expand(
    edges: &BTreeMap<String, Vec<String>>,
    start: Vec<String>,
    max_depth: usize,
) -> Vec<BTreeSet<String>> {
    let mut visited = BTreeSet::new();
    let mut result = Vec::new();
    let mut current_level: BTreeSet<String> = start.into_iter().collect();

    // Mark starting nodes as visited
    for node in &current_level {
        visited.insert(node.clone());
    }

    for _depth in 1..=max_depth {
        let mut next_level = BTreeSet::new();

        for node in &current_level {
            if let Some(targets) = edges.get(node) {
                for target in targets {
                    // Only traverse memory→memory edges
                    if target.starts_with("mem_") && !visited.contains(target) {
                        visited.insert(target.clone());
                        next_level.insert(target.clone());
                    }
                }
            }
        }

        if next_level.is_empty() {
            break;
        }

        result.push(next_level.clone());
        current_level = next_level;
    }

    result
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
        uid: &'static str,
        key: &'static str,
        kind: &'static str,
        status: &'static str,
        title: &'static str,
        summary: &'static str,
        workspace: &'static str,
        repo: &'static str,
        paths: &'static [&'static str],
        globs: &'static [&'static str],
        commands: &'static [&'static str],
        tags: &'static [&'static str],
        lifespan: Option<&'static str>,
        anchor_kind: &'static str,
        verified_sha: &'static str,
        verification_state: &'static str,
        reviewed: &'static str,
        trust_level: &'static str,
        severity: &'static str,
        weight: i64,
    }

    impl Default for Fixture {
        fn default() -> Self {
            Self {
                uid: UID,
                key: "",
                kind: "fact",
                status: "active",
                title: "t",
                summary: "s",
                workspace: "default",
                repo: "",
                paths: &[],
                globs: &[],
                commands: &[],
                tags: &[],
                lifespan: None,
                anchor_kind: "",
                verified_sha: "",
                verification_state: "unverified",
                reviewed: "2026-06-01",
                trust_level: "medium",
                severity: "none",
                weight: 0,
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
        let key_line = if f.key.is_empty() {
            String::new()
        } else {
            format!("memory_key = {:?}\n", f.key)
        };
        let lifespan_line = f
            .lifespan
            .map(|lifespan| format!("lifespan = {:?}\n", lifespan))
            .unwrap_or_default();
        let text = format!(
            r#"
memory_uid = "{uid}"
{key_line}{lifespan_line}schema_version = 1
memory_type = "{kind}"
status = "{status}"
title = "{title}"
summary = "{summary}"
created = "2026-06-01"
updated = "2026-06-01"

[scope]
workspace = "{workspace}"
repo = "{repo}"
paths = {paths}
globs = {globs}
commands = {commands}
tags = {tags}

[git]
anchor_kind = "{anchor_kind}"
verified_sha = "{verified_sha}"

[review]
verification_state = "{vs}"
reviewed = "{reviewed}"
review_by = ""

[trust]
trust_level = "{trust_level}"

[ranking]
severity = "{severity}"
weight = {weight}
"#,
            uid = f.uid,
            kind = f.kind,
            status = f.status,
            title = f.title,
            summary = f.summary,
            lifespan_line = lifespan_line,
            workspace = f.workspace,
            repo = f.repo,
            paths = toml_list(f.paths),
            globs = toml_list(f.globs),
            commands = toml_list(f.commands),
            tags = toml_list(f.tags),
            anchor_kind = f.anchor_kind,
            verified_sha = f.verified_sha,
            vs = f.verification_state,
            reviewed = f.reviewed,
            trust_level = f.trust_level,
            severity = f.severity,
            weight = f.weight,
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

    /// REQUIRED ADMISSION GOLDEN (ADR-002 / Charge IX / EX-3). The `repo=""` global
    /// hatch in `base_filter` is DORMANT in production (record always derives a
    /// non-empty repo), so there is no lived baseline — this golden IS the baseline.
    /// A `repo=""`/anchor=none/path-scoped memory in an ARBITRARY (foreign) client
    /// partition: base_filter admits it, match_scope surfaces it on a path hit, and
    /// a scope MISS does not surface it (the global class must not dilute focused
    /// queries). No code change to base_filter — this only pins the lit hatch.
    #[test]
    fn global_class_admitted_in_any_partition_and_surfaces_only_on_scope_hit() {
        let global = memory(&Fixture {
            repo: "",
            anchor_kind: "none",
            paths: &["install/manifest.toml"],
            ..Default::default()
        });
        // admitted in a foreign repo partition (and in the no-repo partition).
        assert!(base_filter(
            &global,
            &part(Some("repo:some-other-client")),
            false
        ));
        assert!(base_filter(&global, &part(None), false));
        // surfaces on a path hit …
        assert_eq!(
            match_scope(&global, &q(&["install/manifest.toml"])),
            Some(ScopeMatch::of(Dimension::Paths))
        );
        // … but a scope miss does NOT surface it — no dilution of focused queries.
        assert_eq!(match_scope(&global, &q(&["src/main.rs"])), None);
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

    // tokenize's own unit tests moved with the fn to `lexical` (SL-017 PHASE-02).

    // -- lexical query helper -----------------------------------------------

    fn with_query(q: &str) -> QueryContext {
        QueryContext {
            query: Some(q.to_owned()),
            ..Default::default()
        }
    }

    // -- OverlapRanker preserves the retired overlap (SL-017 PHASE-04, EX-4) -
    // The behaviour-preservation proof. `lexical_score` is retired (PHASE-04);
    // its outputs are FROZEN here as a regression vector. `OverlapRanker` over the
    // §5.3 space-joined `lex_doc` projection must reproduce them byte-for-byte —
    // the concat-vs-segment equivalence (design §5.4/§9): `tokenize` splits on the
    // join space, so the distinct-hit set, hence the count, is unchanged. The
    // literals are lifted verbatim from the deleted `lexical_score` unit tests
    // (and what they pinned), NOT recomputed by eye. A divergence here means the
    // seam extraction altered overlap — a behaviour breach, not a test edit.
    #[test]
    fn overlap_ranker_preserves_retired_overlap() {
        use crate::lexical::{LexicalCorpus, LexicalRanker, OverlapRanker};

        // (fixture, [(query, frozen overlap count)]). Bag = tokenize(title summary
        // tags key) as a SET; count = distinct query tokens hitting the bag.
        let cases = [
            (
                Fixture {
                    key: "mem.auth.flow",
                    title: "Token expiry",
                    summary: "middleware check",
                    tags: &["rust"],
                    ..Default::default()
                },
                [
                    ("token middleware rust auth", 4), // token+middleware+rust+auth
                    ("token token token", 1),          // SET: repeats count once
                    ("python django", 0),              // no overlap
                    ("src memory rs lint clippy", 0),  // no overlap
                    ("", 0),                           // empty query
                ],
            ),
            // separator-laden segments (dots/slashes) — the equivalence stressor
            (
                Fixture {
                    key: "mem.pattern.lint",
                    title: "src/memory.rs clippy",
                    summary: "expiry token",
                    tags: &["lint", "rust"],
                    ..Default::default()
                },
                [
                    ("token middleware rust auth", 2), // token+rust
                    ("token token token", 1),
                    ("python django", 0),
                    ("src memory rs lint clippy", 5), // src+memory+rs+lint+clippy
                    ("", 0),
                ],
            ),
        ];

        for (f, expected) in &cases {
            let m = memory(f);
            let docs = vec![lex_doc(&m)]; // the ONE production projection (T2)
            let corpus = LexicalCorpus::Raw(&docs);
            let uid = m.uid.as_str();
            for (q, want) in expected {
                let got = OverlapRanker.score(Some(q), &corpus, &[uid]);
                assert_eq!(got.len(), 1, "positional: one entry per target");
                assert_eq!(
                    got[0],
                    (m.uid.clone(), *want),
                    "OverlapRanker diverged from frozen overlap for query {q:?}"
                );
            }
            // no query ⇒ 0
            assert_eq!(
                OverlapRanker.score(None, &corpus, &[uid]),
                vec![(m.uid.clone(), 0)]
            );
        }
    }

    // -- exact_key_match (EX-1) ---------------------------------------------

    #[test]
    fn exact_key_match_full_key_only() {
        let m = memory(&Fixture {
            key: "mem.auth.flow",
            ..Default::default()
        });
        // full key, with and without the `mem.` prefix the caller may omit
        assert!(exact_key_match(&m, &with_query("mem.auth.flow")));
        assert!(exact_key_match(&m, &with_query("auth.flow")));
        // a prefix/segment of the key is NOT an exact match (lexical's job)
        assert!(!exact_key_match(&m, &with_query("auth")));
        // a non-key-shaped query is a non-match, never a fault
        assert!(!exact_key_match(&m, &with_query("not a key!")));
        // no query, and a keyless memory, both miss
        assert!(!exact_key_match(&m, &QueryContext::default()));
        let keyless = memory(&Fixture::default());
        assert!(!exact_key_match(&keyless, &with_query("mem.auth.flow")));
    }

    // -- staleness (EX-2 / VT-2 — the 4-branch first-match) -----------------

    fn facts(commits_since: Option<u32>) -> GitFacts {
        GitFacts { commits_since }
    }

    #[test]
    fn staleness_branch1_commit_mode_when_scoped_and_attested() {
        let base = Fixture {
            paths: &["src"],
            anchor_kind: "commit",
            verified_sha: "deadbeef",
            ..Default::default()
        };
        let m = memory(&base);
        assert_eq!(
            staleness(&m, facts(Some(0)), "2026-06-05"),
            Staleness::Fresh
        );
        assert_eq!(
            staleness(&m, facts(Some(3)), "2026-06-05"),
            Staleness::Stale
        );
        // undecidable reachability (or absent target) ⇒ Unknown, never Fresh
        assert_eq!(staleness(&m, facts(None), "2026-06-05"), Staleness::Unknown);
    }

    #[test]
    fn staleness_paths_empty_but_verified_falls_to_time_branch() {
        // verified_sha set but no scope.paths ⇒ not commit mode (B5).
        let m = memory(&Fixture {
            paths: &[],
            verified_sha: "deadbeef",
            reviewed: "2026-06-01",
            ..Default::default()
        });
        assert_eq!(
            staleness(&m, facts(Some(9)), "2026-06-05"),
            Staleness::Fresh
        );
    }

    #[test]
    fn staleness_time_branch_boundary_is_inclusive_30() {
        let m = memory(&Fixture {
            reviewed: "2026-05-01",
            ..Default::default()
        });
        // exactly 30 days ⇒ Fresh (inclusive); 31 ⇒ Stale
        assert_eq!(staleness(&m, facts(None), "2026-05-31"), Staleness::Fresh);
        assert_eq!(staleness(&m, facts(None), "2026-06-01"), Staleness::Stale);
    }

    #[test]
    fn staleness_anchored_unattested_no_date_is_unknown() {
        let m = memory(&Fixture {
            anchor_kind: "commit",
            verified_sha: "",
            reviewed: "not-a-date",
            ..Default::default()
        });
        assert_eq!(staleness(&m, facts(None), "2026-06-05"), Staleness::Unknown);
    }

    #[test]
    fn staleness_no_anchor_no_date_is_unanchored() {
        let m = memory(&Fixture {
            anchor_kind: "",
            verified_sha: "",
            reviewed: "",
            ..Default::default()
        });
        assert_eq!(
            staleness(&m, facts(None), "2026-06-05"),
            Staleness::Unanchored
        );
    }

    #[test]
    fn staleness_dirty_then_verified_uses_verified_sha() {
        // born checkout_state (dirty) but later verify-attested clean ⇒ branch 1.
        let m = memory(&Fixture {
            paths: &["src"],
            anchor_kind: "checkout_state",
            verified_sha: "cleansha",
            ..Default::default()
        });
        assert_eq!(
            staleness(&m, facts(Some(0)), "2026-06-05"),
            Staleness::Fresh
        );
    }

    // -- SL-018 PHASE-02: the evergreen `reference` disposition (EX-4 / VT-4) --

    /// The global/orientation class signature (ADR-002): `repo=""`, anchor=none,
    /// path-scoped, unattested. The default Fixture is repo=""/none/unattested but
    /// SCOPELESS — adding a path is what makes it a class member.
    fn global_class_fixture() -> Fixture {
        Fixture {
            repo: "",
            anchor_kind: "none",
            verified_sha: "",
            paths: &["install/manifest.toml"],
            ..Default::default()
        }
    }

    #[test]
    fn global_class_renders_reference_never_decaying() {
        let m = memory(&global_class_fixture());
        // an arbitrarily-old `reviewed` does NOT decay it — evergreen.
        let aeons_later = "2099-01-01";
        assert_eq!(
            staleness(&m, facts(None), aeons_later),
            Staleness::Reference,
            "the global class is exempt from days-since-reviewed decay"
        );
    }

    #[test]
    fn an_attested_global_scoped_memory_uses_commit_mode_not_reference() {
        // verified_sha present ⇒ branch 1 (commit mode) wins; Reference is the
        // UNATTESTED disposition (ADR-002: "+ no verified_sha").
        let m = memory(&Fixture {
            verified_sha: "deadbeef",
            ..global_class_fixture()
        });
        assert_eq!(
            staleness(&m, facts(Some(0)), "2026-06-05"),
            Staleness::Fresh
        );
        assert_eq!(
            staleness(&m, facts(Some(2)), "2026-06-05"),
            Staleness::Stale
        );
    }

    #[test]
    fn a_scopeless_repo_empty_memory_is_not_reference() {
        // the default Fixture (repo=""/none/unattested but SCOPELESS) is NOT a
        // class member — it keeps its pre-SL-018 disposition (the gate proof that
        // the special-case is scoped to the class, not all repo="" memories).
        let dated = memory(&Fixture {
            reviewed: "2026-06-01",
            ..Default::default()
        });
        assert_eq!(
            staleness(&dated, facts(None), "2026-06-05"),
            Staleness::Fresh
        );
        let undated = memory(&Fixture {
            reviewed: "",
            ..Default::default()
        });
        assert_eq!(
            staleness(&undated, facts(None), "2026-06-05"),
            Staleness::Unanchored
        );
    }

    // -- rank ordinals (EX-3 / VT-3) ----------------------------------------

    #[test]
    fn rank_ordinals_polarity_and_unknown_worst() {
        assert!(verification_rank("verified") < verification_rank("unverified"));
        assert!(verification_rank("unverified") < verification_rank("stale"));
        assert!(verification_rank("stale") < verification_rank("disputed"));
        assert!(verification_rank("disputed") < verification_rank("???"));

        assert!(trust_rank("high") < trust_rank("medium"));
        assert!(trust_rank("medium") < trust_rank("low"));
        assert!(trust_rank("low") < trust_rank("???"));

        assert!(severity_rank("critical") < severity_rank("high"));
        assert!(severity_rank("high") < severity_rank("medium"));
        assert!(severity_rank("medium") < severity_rank("low"));
        assert!(severity_rank("low") < severity_rank("none"));
        assert!(severity_rank("none") < severity_rank("???"));
    }

    // -- rank: the 9-key total order (VT-1 / VT-3) --------------------------

    /// Build a candidate from a fixture + scope dim + lexical/exact signals.
    fn cand<'a>(
        m: &'a Memory,
        lexical: u32,
        exact_key: bool,
        dim: Option<Dimension>,
    ) -> Candidate<'a> {
        Candidate {
            memory: m,
            scope_match: dim.map(ScopeMatch::of),
            staleness: Staleness::Unknown,
            lexical,
            exact_key,
        }
    }

    const TODAY: &str = "2026-06-05";

    fn uids() -> [&'static str; 3] {
        [
            "mem_018f3a1b2c3d4e5f60718293a4b5c601",
            "mem_018f3a1b2c3d4e5f60718293a4b5c602",
            "mem_018f3a1b2c3d4e5f60718293a4b5c603",
        ]
    }

    #[test]
    fn rank_is_deterministic_under_shuffle() {
        // three memories distinguished only by uid + lexical; every permutation
        // of the input must yield one identical order (the total-order proof, D5).
        let [u0, u1, u2] = uids();
        let m0 = memory(&Fixture {
            uid: u0,
            ..Default::default()
        });
        let m1 = memory(&Fixture {
            uid: u1,
            ..Default::default()
        });
        let m2 = memory(&Fixture {
            uid: u2,
            ..Default::default()
        });
        let order_of = |cs: Vec<Candidate<'_>>| {
            rank(cs, TODAY)
                .iter()
                .map(|c| c.memory.uid.clone())
                .collect::<Vec<_>>()
        };
        // identical lexical/exact ⇒ tiebreak is uid ascending
        let baseline = order_of(vec![
            cand(&m0, 1, false, None),
            cand(&m1, 1, false, None),
            cand(&m2, 1, false, None),
        ]);
        assert_eq!(baseline, vec![u0.to_owned(), u1.to_owned(), u2.to_owned()]);
        // fixed permutations (no rng — the layer is pure)
        for perm in [[2usize, 0, 1], [1, 2, 0], [2, 1, 0], [0, 2, 1]] {
            let ms = [&m0, &m1, &m2];
            let cs = perm.iter().map(|&i| cand(ms[i], 1, false, None)).collect();
            assert_eq!(order_of(cs), baseline, "perm {perm:?} must match");
        }
    }

    /// Assert `win` outranks `lose` — i.e. rank places `win` first.
    fn assert_ranks_first(win: Candidate<'_>, lose: Candidate<'_>) {
        let win_uid = win.memory.uid.clone();
        let ranked = rank(vec![lose, win], TODAY);
        assert_eq!(ranked[0].memory.uid, win_uid);
    }

    #[test]
    fn rank_key1_exact_key_beats_everything() {
        // the loser dominates on lexical, yet an exact-key hit still wins (key 1).
        let [u0, u1, _] = uids();
        let exact = memory(&Fixture {
            uid: u0,
            key: "mem.k",
            ..Default::default()
        });
        let lexy = memory(&Fixture {
            uid: u1,
            ..Default::default()
        });
        assert_ranks_first(
            cand(&exact, 0, true, None),
            cand(&lexy, 99, false, Some(Dimension::Paths)),
        );
    }

    #[test]
    fn rank_key2_higher_lexical_first() {
        let [u0, u1, _] = uids();
        let hi = memory(&Fixture {
            uid: u0,
            ..Default::default()
        });
        let lo = memory(&Fixture {
            uid: u1,
            ..Default::default()
        });
        assert_ranks_first(cand(&hi, 5, false, None), cand(&lo, 1, false, None));
    }

    #[test]
    fn rank_key3_higher_specificity_first() {
        let [u0, u1, _] = uids();
        let paths = memory(&Fixture {
            uid: u0,
            ..Default::default()
        });
        let tags = memory(&Fixture {
            uid: u1,
            ..Default::default()
        });
        assert_ranks_first(
            cand(&paths, 1, false, Some(Dimension::Paths)), // 3
            cand(&tags, 1, false, Some(Dimension::Tags)),   // 0
        );
    }

    #[test]
    fn rank_key4_verification_better_first() {
        let [u0, u1, _] = uids();
        let verified = memory(&Fixture {
            uid: u0,
            verification_state: "verified",
            ..Default::default()
        });
        let disputed = memory(&Fixture {
            uid: u1,
            verification_state: "disputed",
            ..Default::default()
        });
        assert_ranks_first(
            cand(&verified, 1, false, None),
            cand(&disputed, 1, false, None),
        );
    }

    #[test]
    fn rank_key5_higher_trust_first() {
        let [u0, u1, _] = uids();
        let high = memory(&Fixture {
            uid: u0,
            trust_level: "high",
            ..Default::default()
        });
        let low = memory(&Fixture {
            uid: u1,
            trust_level: "low",
            ..Default::default()
        });
        assert_ranks_first(cand(&high, 1, false, None), cand(&low, 1, false, None));
    }

    #[test]
    fn rank_key6_higher_severity_first() {
        let [u0, u1, _] = uids();
        let crit = memory(&Fixture {
            uid: u0,
            severity: "critical",
            ..Default::default()
        });
        let none = memory(&Fixture {
            uid: u1,
            severity: "none",
            ..Default::default()
        });
        assert_ranks_first(cand(&crit, 1, false, None), cand(&none, 1, false, None));
    }

    #[test]
    fn rank_key7_higher_weight_first() {
        let [u0, u1, _] = uids();
        let heavy = memory(&Fixture {
            uid: u0,
            weight: 9,
            ..Default::default()
        });
        let light = memory(&Fixture {
            uid: u1,
            weight: 0,
            ..Default::default()
        });
        assert_ranks_first(cand(&heavy, 1, false, None), cand(&light, 1, false, None));
    }

    #[test]
    fn rank_key8_more_recent_first_missing_last() {
        let [u0, u1, u2] = uids();
        let recent = memory(&Fixture {
            uid: u0,
            reviewed: "2026-06-04",
            ..Default::default()
        });
        let old = memory(&Fixture {
            uid: u1,
            reviewed: "2026-01-01",
            ..Default::default()
        });
        assert_ranks_first(cand(&recent, 1, false, None), cand(&old, 1, false, None));
        // a missing/malformed reviewed date sorts LAST (i64::MAX recency)
        let dated = memory(&Fixture {
            uid: u0,
            reviewed: "2026-01-01",
            ..Default::default()
        });
        let undated = memory(&Fixture {
            uid: u2,
            reviewed: "garbage",
            ..Default::default()
        });
        assert_ranks_first(cand(&dated, 1, false, None), cand(&undated, 1, false, None));
    }

    #[test]
    fn effective_age_scales_by_lifespan_factor() {
        assert_eq!(
            effective_age(10, Some(crate::memory::Lifespan::Semantic)),
            1
        );
        assert_eq!(
            effective_age(10, Some(crate::memory::Lifespan::Working)),
            100
        );
        assert_eq!(effective_age(10, None), 10);
    }

    #[test]
    fn effective_age_preserves_the_missing_review_sentinel() {
        assert_eq!(
            effective_age(i64::MAX, Some(crate::memory::Lifespan::Working)),
            i64::MAX
        );
    }

    #[test]
    fn rank_key8_prefers_longer_lived_memories_at_equal_review_date() {
        let [u0, u1, _] = uids();
        let working = memory(&Fixture {
            uid: u0,
            lifespan: Some("working"),
            reviewed: "2026-06-01",
            ..Default::default()
        });
        let semantic = memory(&Fixture {
            uid: u1,
            lifespan: Some("semantic"),
            reviewed: "2026-06-01",
            ..Default::default()
        });
        assert_ranks_first(
            cand(&semantic, 1, false, None),
            cand(&working, 1, false, None),
        );
    }

    #[test]
    fn rank_verification_stale_not_double_penalised_by_staleness() {
        // verification_state and the Staleness column are separate axes: two
        // candidates equal on every Ord key but differing in `staleness` must
        // tiebreak on uid alone (staleness is display-only, never an Ord key).
        let [u0, u1, _] = uids();
        let a = memory(&Fixture {
            uid: u0,
            ..Default::default()
        });
        let b = memory(&Fixture {
            uid: u1,
            ..Default::default()
        });
        let mut ca = cand(&a, 1, false, None);
        ca.staleness = Staleness::Stale;
        let mut cb = cand(&b, 1, false, None);
        cb.staleness = Staleness::Fresh;
        // despite a=Stale, b=Fresh, order is uid-ascending (u0 before u1)
        let ranked = rank(vec![cb, ca], TODAY);
        assert_eq!(ranked[0].memory.uid, u0);
    }

    // === PHASE-04: shell — labels, freeze, the gate, query pipeline, rows. ====

    fn snap(target: Option<&str>, repo: Option<&str>) -> Snapshot {
        Snapshot {
            today: TODAY.to_owned(),
            target: target.map(str::to_owned),
            part: part(repo),
        }
    }

    #[test]
    fn staleness_and_dimension_labels() {
        assert_eq!(Staleness::Fresh.label(), "fresh");
        assert_eq!(Staleness::Stale.label(), "stale");
        assert_eq!(Staleness::Unknown.label(), "unknown");
        assert_eq!(Staleness::Unanchored.label(), "unanchored");
        assert_eq!(Staleness::Reference.label(), "reference");
        assert_eq!(Dimension::Paths.label(), "paths");
        assert_eq!(Dimension::Globs.label(), "globs");
        assert_eq!(Dimension::Commands.label(), "commands");
        assert_eq!(Dimension::Tags.label(), "tags");
    }

    #[test]
    fn freeze_outside_a_repo_degrades_to_no_target_no_repo() {
        // A bare temp dir is not a git repo ⇒ capture yields a None-anchor frame:
        // target/repo both None, but the query still runs (today is set).
        let dir = tempfile::tempdir().expect("tempdir");
        let s = freeze(dir.path());
        assert_eq!(s.target, None);
        assert_eq!(s.part.repo, None);
        assert_eq!(s.part.workspace, "default");
        assert!(!s.today.is_empty());
    }

    #[test]
    fn git_facts_gate_skips_without_spawning() {
        // None of the three gate conditions reaches git, so a non-repo root is
        // safe — each skip case yields commits_since None with no subprocess.
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        // unscoped (no paths)
        let unscoped = memory(&Fixture {
            verified_sha: "abc",
            ..Default::default()
        });
        assert_eq!(
            git_facts(root, &unscoped, &snap(Some("dead"), None)).commits_since,
            None
        );
        // scoped but unattested (no verified_sha)
        let unattested = memory(&Fixture {
            paths: &["src/x.rs"],
            ..Default::default()
        });
        assert_eq!(
            git_facts(root, &unattested, &snap(Some("dead"), None)).commits_since,
            None
        );
        // scoped + attested but no frozen target
        let no_target = memory(&Fixture {
            paths: &["src/x.rs"],
            verified_sha: "abc",
            ..Default::default()
        });
        assert_eq!(
            git_facts(root, &no_target, &snap(None, None)).commits_since,
            None
        );
    }

    #[test]
    fn query_scope_bearing_drops_nonmatching() {
        let [u0, u1, _] = uids();
        let hit = memory(&Fixture {
            uid: u0,
            paths: &["src/main.rs"],
            ..Default::default()
        });
        let miss = memory(&Fixture {
            uid: u1,
            paths: &["docs/guide.md"],
            ..Default::default()
        });
        let mems = vec![hit, miss];
        let ranked = query(
            &mems,
            &q(&["src/main.rs"]),
            &snap(None, None),
            false,
            Path::new("."),
            &Bm25Ranker,
        );
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].memory.uid, u0);
    }

    #[test]
    fn query_bare_query_keeps_all_active_ranked_lexically() {
        let [u0, u1, _] = uids();
        // u0 matches the query token, u1 does not — both kept (D20), u0 ranks first.
        let matchy = memory(&Fixture {
            uid: u0,
            title: "auth token",
            ..Default::default()
        });
        let other = memory(&Fixture {
            uid: u1,
            title: "unrelated",
            ..Default::default()
        });
        let mems = vec![matchy, other];
        // Re-pointed through OverlapRanker (the retired overlap): this pins the
        // pre-BM25 ordering, so its assertions are the behaviour-preservation
        // witness and stay UNCHANGED across the seam extraction (EX-4).
        let ranked = query(
            &mems,
            &with_query("auth"),
            &snap(None, None),
            false,
            Path::new("."),
            &crate::lexical::OverlapRanker,
        );
        assert_eq!(ranked.len(), 2, "bare --query keeps all active");
        assert_eq!(ranked[0].memory.uid, u0, "lexical hit ranks first");
    }

    #[test]
    fn query_base_filter_excludes_non_active() {
        let retracted = memory(&Fixture {
            status: "retracted",
            paths: &["src/main.rs"],
            ..Default::default()
        });
        let mems = vec![retracted];
        let ranked = query(
            &mems,
            &q(&["src/main.rs"]),
            &snap(None, None),
            false,
            Path::new("."),
            &Bm25Ranker,
        );
        assert!(ranked.is_empty(), "retracted is dropped by base_filter");
    }

    #[test]
    fn query_lifespan_filter_keeps_only_exact_matches() {
        let [u0, u1, _] = uids();
        let semantic = memory(&Fixture {
            uid: u0,
            paths: &["src/main.rs"],
            lifespan: Some("semantic"),
            ..Default::default()
        });
        let working = memory(&Fixture {
            uid: u1,
            paths: &["src/main.rs"],
            lifespan: Some("working"),
            ..Default::default()
        });
        let mems = vec![semantic, working];
        let ranked = query(
            &mems,
            &QueryContext {
                paths: vec!["src/main.rs".to_owned()],
                lifespan: Some(crate::memory::Lifespan::Semantic),
                ..Default::default()
            },
            &snap(None, None),
            false,
            Path::new("."),
            &Bm25Ranker,
        );
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].memory.uid, u0);
    }

    // VT-1 — Key-1 (exact_key) dominates Key-2 (BM25 magnitude). An exact
    // memory_key hit with a LOW bm25 (long, length-normalised-down doc) still
    // outranks a non-key hit with a HIGHER bm25 (short doc), through the wired
    // query(…, &Bm25Ranker). The polarity is structural, not score-tuned.
    #[test]
    fn query_exact_key_dominates_higher_bm25() {
        let [u0, u1, _] = uids();
        // keyhit: "mem.zzz" hits its key exactly (exact_key); doc padded long so
        // its bm25 on mem/zzz is length-normalised DOWN.
        let keyhit = memory(&Fixture {
            uid: u0,
            key: "mem.zzz",
            title: "zzz",
            summary: "alpha beta gamma delta epsilon zeta eta theta iota kappa",
            ..Default::default()
        });
        // lexhit: no key, SHORT doc on the same tokens ⇒ HIGHER bm25.
        let lexhit = memory(&Fixture {
            uid: u1,
            title: "mem zzz",
            summary: "",
            ..Default::default()
        });
        let mems = vec![keyhit, lexhit];
        let ranked = query(
            &mems,
            &with_query("mem.zzz"),
            &snap(None, None),
            false,
            Path::new("."),
            &Bm25Ranker,
        );
        assert_eq!(ranked.len(), 2);
        assert!(ranked[0].exact_key, "exact-key memory ranks first");
        assert_eq!(ranked[0].memory.uid, u0);
        assert!(
            ranked[0].lexical < ranked[1].lexical,
            "exact-key wins DESPITE a lower BM25 (Key-1 over Key-2): {} vs {}",
            ranked[0].lexical,
            ranked[1].lexical
        );
    }

    // VT-2 — determinism: a permuted input store yields byte-identical query
    // output (corpus-order-independent BM25 + uid tiebreak). Shuffle-invariance
    // with Bm25Ranker wired through query().
    #[test]
    fn query_is_shuffle_invariant_under_bm25() {
        let [u0, u1, u2] = uids();
        let mk = |uid, title| {
            memory(&Fixture {
                uid,
                title,
                ..Default::default()
            })
        };
        let a = mk(u0, "rare token");
        let b = mk(u1, "common token");
        let c = mk(u2, "common other");
        let forward = vec![a.clone(), b.clone(), c.clone()];
        let reversed = vec![c, b, a];
        let run = |mems: &[Memory]| -> Vec<(String, u32)> {
            query(
                mems,
                &with_query("rare token"),
                &snap(None, None),
                false,
                Path::new("."),
                &Bm25Ranker,
            )
            .iter()
            .map(|c| (c.memory.uid.clone(), c.lexical))
            .collect()
        };
        assert_eq!(
            run(&forward),
            run(&reversed),
            "permuted store ⇒ identical ranked (uid, lexical)"
        );
    }

    // VT-4 — the intended quality change: BM25 (rare-term IDF) and the retired
    // overlap (raw distinct-hit count) order the SAME survivors OPPOSITELY. `a`
    // matches TWO common query tokens (overlap 2); `b` matches ONE rare token
    // (overlap 1) whose IDF — inflated by common-term fillers in the fit corpus
    // — lifts its BM25 above `a`'s. Two rankers, one query, reversed first place.
    #[test]
    fn query_bm25_and_overlap_order_oppositely() {
        let uids5 = [
            "mem_018f3a1b2c3d4e5f60718293a4b5c601",
            "mem_018f3a1b2c3d4e5f60718293a4b5c602",
            "mem_018f3a1b2c3d4e5f60718293a4b5c6f3",
            "mem_018f3a1b2c3d4e5f60718293a4b5c6f4",
            "mem_018f3a1b2c3d4e5f60718293a4b5c6f5",
        ];
        let mk = |uid, title| {
            memory(&Fixture {
                uid,
                title,
                ..Default::default()
            })
        };
        let a = mk(uids5[0], "common ubiq"); // overlap 2 (both common terms)
        let b = mk(uids5[1], "rare"); // overlap 1 (rare term, high IDF)
        // fillers inflate df(common)/df(ubiq), depressing their IDF below rare's
        let f1 = mk(uids5[2], "common ubiq");
        let f2 = mk(uids5[3], "common ubiq");
        let f3 = mk(uids5[4], "common ubiq");
        let mems = vec![a, b, f1, f2, f3];
        let qctx = with_query("common ubiq rare");
        let order = |ranker: &dyn LexicalRanker| -> Vec<String> {
            query(
                &mems,
                &qctx,
                &snap(None, None),
                false,
                Path::new("."),
                ranker,
            )
            .iter()
            .map(|c| c.memory.uid.clone())
            .collect()
        };
        let bm25 = order(&Bm25Ranker);
        let overlap = order(&crate::lexical::OverlapRanker);
        assert_eq!(
            bm25[0], uids5[1],
            "BM25 lifts the rare-term match: {bm25:?}"
        );
        assert_eq!(
            overlap[0], uids5[0],
            "overlap ranks the higher raw-count first: {overlap:?}"
        );
        assert_ne!(
            bm25[0], overlap[0],
            "the two rankers disagree — the intended quality change"
        );
    }

    // VT-5 — the lexical signal is DERIVED per query, never persisted: the Memory
    // storage model gains no field and no float this slice (R3, by construction).
    // Debug enumerates every Memory field, so its absence of `lexical` is the
    // structural guard; and scoring borrows `&Memory` immutably, so the
    // representation is unchanged after a nonzero BM25 score.
    #[test]
    fn query_bm25_score_is_derived_not_persisted_on_memory() {
        let m = memory(&Fixture {
            title: "rare token",
            ..Default::default()
        });
        let before = format!("{m:?}");
        assert!(
            !before.contains("lexical"),
            "Memory carries no lexical field"
        );
        let mems = vec![m];
        let ranked = query(
            &mems,
            &with_query("rare token"),
            &snap(None, None),
            false,
            Path::new("."),
            &Bm25Ranker,
        );
        assert!(ranked[0].lexical > 0, "BM25 scored the survivor nonzero");
        assert_eq!(
            format!("{:?}", mems[0]),
            before,
            "Memory representation unchanged by scoring (immutable borrow)"
        );
    }

    #[test]
    fn format_find_row_carries_full_uid_and_required_columns() {
        let m = memory(&Fixture {
            paths: &["src/main.rs"],
            trust_level: "low",
            severity: "high",
            title: "be careful",
            ..Default::default()
        });
        let mut c = cand(&m, 0, false, Some(Dimension::Paths));
        c.staleness = Staleness::Unknown;
        let out = format_find_table(&[&c]);
        // full uid (not a short prefix), the matched dim, and the risk columns.
        assert!(out.contains(UID), "full uid printed");
        assert!(out.contains("paths"), "spec column = matched dim");
        assert!(out.contains("low"), "trust visible");
        assert!(out.contains("high"), "severity visible");
        assert!(out.contains("unknown"), "staleness column");
        assert!(out.contains("be careful"), "title");
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn format_find_scrubs_a_newline_title() {
        let m = memory(&Fixture {
            title: "real",
            ..Default::default()
        });
        // Inject a newline post-parse to prove the row formatter scrubs it (F-A10).
        let mut m2 = m.clone();
        m2.title = "row1\nforged-row2".to_owned();
        let c = cand(&m2, 0, false, None);
        let out = format_find_table(&[&c]);
        assert!(
            !out.contains("\nforged-row2"),
            "newline must not forge a row"
        );
        assert!(out.contains("\\nforged-row2"), "newline rendered as escape");
    }

    #[test]
    fn format_find_empty_is_empty_string() {
        let empty: [&Candidate<'_>; 0] = [];
        assert_eq!(format_find_table(&empty), "");
    }

    // === PHASE-05: the retrieve holdback, floor, and suppress-then-take. =======

    // EX-2 / R2: `--min-trust` validates at the edge — no silent worst-rank default.
    #[test]
    fn parse_min_trust_accepts_only_the_three_tiers() {
        for ok in ["high", "medium", "low"] {
            assert_eq!(parse_min_trust(ok).as_deref(), Ok(ok));
        }
        assert!(parse_min_trust("banana").is_err());
        assert!(parse_min_trust("").is_err());
        assert!(parse_min_trust("High").is_err()); // case-sensitive, like the store
    }

    // EX-2 / D8 / B7: the floor defaults to medium and only RAISES.
    #[test]
    fn holdback_floor_defaults_medium_and_only_raises() {
        let medium = trust_rank("medium");
        assert_eq!(holdback_floor(None), medium, "default floor is medium");
        assert_eq!(
            holdback_floor(Some("high")),
            trust_rank("high"),
            "high raises"
        );
        assert_eq!(
            holdback_floor(Some("medium")),
            medium,
            "medium is the default"
        );
        // --min-trust low cannot LOWER below the default (non-bypassable, B7).
        assert_eq!(holdback_floor(Some("low")), medium, "low is clamped up");
    }

    fn risky(uid: &'static str, trust: &'static str, severity: &'static str) -> Memory {
        memory(&Fixture {
            uid,
            trust_level: trust,
            severity,
            ..Default::default()
        })
    }

    // VT-1 / EX-2 / D8: the holdback predicate — `low ∧ severity≥high` at default.
    #[test]
    fn held_back_is_low_trust_and_high_severity_at_default_floor() {
        let floor = holdback_floor(None);
        // suppressed: low trust ∧ {critical, high}
        assert!(held_back(&risky(UID, "low", "critical"), floor));
        assert!(held_back(&risky(UID, "low", "high"), floor));
        // NOT suppressed: low trust but the severity is below high (just low quality)
        assert!(!held_back(&risky(UID, "low", "medium"), floor));
        assert!(!held_back(&risky(UID, "low", "none"), floor));
        // NOT suppressed at default: medium/high trust, even at high severity
        assert!(!held_back(&risky(UID, "medium", "high"), floor));
        assert!(!held_back(&risky(UID, "high", "critical"), floor));
    }

    // VT-1 / EX-2: `--min-trust high` raises the floor — now medium∧high is held too,
    // but a high-trust memory always passes.
    #[test]
    fn min_trust_high_raises_the_floor_over_medium() {
        let floor = holdback_floor(Some("high"));
        assert!(
            held_back(&risky(UID, "medium", "high"), floor),
            "medium now held"
        );
        assert!(
            held_back(&risky(UID, "low", "high"), floor),
            "low still held"
        );
        assert!(
            !held_back(&risky(UID, "high", "critical"), floor),
            "high passes"
        );
        // low severity is never held, whatever the floor (holdback targets risk).
        assert!(!held_back(&risky(UID, "low", "none"), floor));
    }

    fn ranked_cand(m: &Memory) -> Candidate<'_> {
        cand(m, 1, false, None)
    }

    // VT-2 / EX-3: a held-back memory is dropped before render (its body is never
    // read), while clean memories pass through.
    #[test]
    fn holdback_suppresses_low_trust_critical_pre_offset_limit() {
        let [u0, u1, _] = uids();
        let clean = risky(u0, "medium", "high"); // medium trust ⇒ kept
        let held = risky(u1, "low", "critical"); // low ∧ critical ⇒ suppressed
        let ranked = vec![ranked_cand(&clean), ranked_cand(&held)];
        let floor = holdback_floor(None);
        let eligible: Vec<&Candidate<'_>> = ranked
            .iter()
            .filter(|c| !held_back(c.memory, floor))
            .collect();
        let uids: Vec<&str> = eligible.iter().map(|c| c.memory.uid.as_str()).collect();
        assert_eq!(
            uids,
            vec![u0],
            "held-back memory absent from the eligible set"
        );
    }

    // K3: holdback-then-limit — a held-back memory ranked first does NOT steal the
    // single slot; the clean memory behind it is still shown.
    #[test]
    fn holdback_does_not_consume_a_limit_slot() {
        let [u0, u1, _] = uids();
        // `held` ranks first (uid u0 < u1), but is suppressed; `clean` takes the slot.
        let held = risky(u0, "low", "high");
        let clean = risky(u1, "high", "high");
        let ranked = vec![ranked_cand(&held), ranked_cand(&clean)];
        let floor = holdback_floor(None);
        let eligible: Vec<&Candidate<'_>> = ranked
            .iter()
            .filter(|c| !held_back(c.memory, floor))
            .collect();
        let visible: Vec<&Candidate<'_>> = eligible.iter().take(1).copied().collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(
            visible[0].memory.uid, u1,
            "clean memory takes the slot, not held"
        );
    }

    // EX-2 / VT-3: the limit caps the shown count.
    #[test]
    fn holdback_then_limit_caps_at_limit() {
        let [u0, u1, u2] = uids();
        let ms = [
            risky(u0, "high", "none"),
            risky(u1, "high", "none"),
            risky(u2, "high", "none"),
        ];
        let ranked: Vec<Candidate<'_>> = ms.iter().map(ranked_cand).collect();
        let floor = holdback_floor(None);
        let eligible: Vec<&Candidate<'_>> = ranked
            .iter()
            .filter(|c| !held_back(c.memory, floor))
            .collect();
        let v2: Vec<&Candidate<'_>> = eligible.iter().take(2).copied().collect();
        assert_eq!(v2.len(), 2);
        let v10: Vec<&Candidate<'_>> = eligible.iter().take(10).copied().collect();
        assert_eq!(v10.len(), 3);
    }
}
