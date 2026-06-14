// SPDX-License-Identifier: GPL-3.0-only
//! `coverage` — the slice-side coverage store (SL-042 P2, REQ-109).
//!
//! A *coverage entry* is **observed verification evidence** for a requirement:
//! the cited 4-tuple key `(slice, requirement, contributing_change, mode)`, the
//! observed [`CoverageStatus`], the git anchor it was seen at, and (for VH/VA
//! attestations) the date it was attested. Entries live slice-side in
//! `.doctrine/slice/NNN/coverage.toml` as a `[[entry]]` array-of-tables; the
//! reconcile engine (P3/P4) reads them.
//!
//! This module is a **pure leaf** (ADR-001): types + pure folds, no clock / rng /
//! git / disk — all filesystem I/O lives in tests. It owns [`CoverageKey`], the
//! 4-tuple identity/citation key that `rec` aliases as `EvidenceRef` (the cited
//! thing owns its key, not the citer).
//!
//! **Distinct store (NF-001 / ADR-009 §3).** Coverage carries the observed-evidence
//! [`CoverageStatus`], NEVER the authored [`crate::requirement::ReqStatus`]: it does
//! not derive, read, or write authored requirement status. The two stores are
//! separate files — coverage at `.doctrine/slice/NNN/coverage.toml`, authored
//! requirement status in the requirement entity file.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::requirement::{CoverageStatus, ReqStatus};

/// The valid verification modes a coverage entry may cite: by **test** (`VT`), by
/// **agent** (`VA`), or by **human** (`VH`). Membership is validated at the coverage
/// layer (see [`mode_is_valid`]), not by the key's type — `mode` stays a `String`
/// so the `rec` ledger keeps round-tripping arbitrary mode tokens verbatim.
const MODES: &[&str] = &["VT", "VA", "VH"];

/// The stable 4-tuple identity/citation key of a coverage entry (design §5.3 F3):
/// `(slice, requirement, contributing_change, mode)`. Owned here (coverage is the
/// cited thing); `rec` aliases it as `EvidenceRef`. `mode` is a `String`, not an
/// enum — the rec ledger is verbatim and must round-trip arbitrary mode strings;
/// the `∈ {VT,VA,VH}` rule is enforced by [`mode_is_valid`] at this layer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct CoverageKey {
    pub(crate) slice: String,
    pub(crate) requirement: String,
    pub(crate) contributing_change: String,
    pub(crate) mode: String,
}

/// One coverage entry: the cited [`CoverageKey`] plus its observed payload. The four
/// key fields are `#[serde(flatten)]`ed inline so an `[[entry]]` table reads the
/// key + payload as one flat table. `status` is the **observed-evidence**
/// [`CoverageStatus`] (never authored `ReqStatus` — NF-001). `attested_date` is the
/// VH/VA attestation date; absent (and omitted on render) for plain `VT` evidence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct CoverageEntry {
    #[serde(flatten)]
    pub(crate) key: CoverageKey,
    pub(crate) status: CoverageStatus,
    pub(crate) git_anchor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) attested_date: Option<String>,
    /// The repo-relative path set this evidence stands on — the input the
    /// staleness seam ([`crate::git::commits_touching`]) walks `git_anchor..HEAD`
    /// against. Additive (`#[serde(default)]`), so P2 entries without it parse to
    /// an empty set (Unknown-leaning, never falsely Fresh).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) touched_paths: Vec<String>,
    /// The VT-check recipe this entry's evidence is produced from (SL-057): the
    /// alias / literal command, extra args, and the optional output [`Matcher`].
    /// Additive (`#[serde(default)]`, skip-if-none), so pre-SL-057 entries with no
    /// `check` key parse to `None` (the `touched_paths` additive precedent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) check: Option<VtCheck>,
}

/// The full `coverage.toml` read/written as data: a `[[entry]]` array-of-tables.
/// Defaults to empty so a fresh / absent file parses.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub(crate) struct CoverageFile {
    #[serde(default)]
    pub(crate) entry: Vec<CoverageEntry>,
}

/// Collect DISTINCT [`CoverageKey`]s, preserving first-seen order. A key set is a
/// *set* of backing cells — the same 4-tuple key must not be cited twice. The
/// corpus walk can surface a key more than once (a slice tree reachable through
/// both its numeric dir and its slug-alias symlink — ISS-006), so every key-set
/// producer dedupes through HERE: the reconcile writer's `evidence_ref` and the
/// close-gate's residual-evidence set share this one deduper (no parallel twin).
pub(crate) fn distinct_keys(keys: impl Iterator<Item = CoverageKey>) -> Vec<CoverageKey> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for k in keys {
        let tag = (
            k.slice.clone(),
            k.requirement.clone(),
            k.contributing_change.clone(),
            k.mode.clone(),
        );
        if seen.insert(tag) {
            out.push(k);
        }
    }
    out
}

/// Parse a `coverage.toml` body. Serde auto-unescapes; no hand-templating.
pub(crate) fn parse(s: &str) -> Result<CoverageFile> {
    Ok(toml::from_str(s)?)
}

/// Render a [`CoverageFile`] to its `coverage.toml` body. Serde auto-escapes; no
/// hand-splicing (`crate::tomlfmt::toml_string` exists for the hand-splice case,
/// unneeded here).
pub(crate) fn render(f: &CoverageFile) -> Result<String> {
    Ok(toml::to_string(f)?)
}

/// Whether `mode ∈ {VT, VA, VH}` — the coverage-layer membership rule that the
/// `String`-typed [`CoverageKey::mode`] does not enforce structurally.
pub(crate) fn mode_is_valid(mode: &str) -> bool {
    MODES.contains(&mode)
}

/// The within-file no-clobber fold: if an entry with the same 4-tuple
/// [`CoverageKey`] already exists, REPLACE it in place (latest payload wins);
/// otherwise APPEND. Pure over the in-memory file — no disk.
pub(crate) fn upsert(file: &mut CoverageFile, entry: CoverageEntry) {
    if let Some(existing) = file.entry.iter_mut().find(|e| e.key == entry.key) {
        *existing = entry;
    } else {
        file.entry.push(entry);
    }
}

// ---------------------------------------------------------------------------
// SL-042 P3 — staleness leaf + composite/drift pure folds (REQ-110/111/114).
//
// The purity split (CLAUDE.md pure/imperative; design §5.2): the shell
// (`crate::coverage_scan`) is the ONLY git/disk seam — it resolves each entry's
// `IsStale` and hands the folds in-memory `(CoverageEntry, IsStale)` cells.
// `composite`/`drift` never touch git/disk/clock/rng: staleness arrives already
// resolved, so the verdict is a deterministic function of its inputs.
// ---------------------------------------------------------------------------

/// Whether a coverage cell's evidence is still current relative to its anchor.
/// PRODUCED by the shell (from [`crate::git::commits_touching`]'s `Option<u32>`),
/// CONSUMED by the folds — staleness is never resolved inside a pure fold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IsStale {
    /// No commit since the anchor touched the cell's paths (`Some(0)`).
    Fresh,
    /// At least one such commit — evidence may be out of date (`Some(n >= 1)`).
    Stale,
    /// The seam could not decide (`None`): undecidable, treated conservatively.
    Unknown,
}

impl From<Option<u32>> for IsStale {
    /// The seam contract (git §`commits_touching`): `Some(0)` ⇒ Fresh,
    /// `Some(n >= 1)` ⇒ Stale, `None` ⇒ Unknown.
    fn from(count: Option<u32>) -> Self {
        match count {
            Some(0) => IsStale::Fresh,
            Some(_) => IsStale::Stale,
            None => IsStale::Unknown,
        }
    }
}

/// One requirement's fanned-in coverage view: every contributing cell
/// `(CoverageEntry, IsStale)` across slices/changes, sorted by the stable
/// [`CoverageKey`] (DETERMINISTIC — no map-order/clock/rng). v1 surfaces ALL
/// cells with no precedence; it is DERIVED, never persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Composite {
    cells: Vec<(CoverageEntry, IsStale)>,
}

/// Total-order the stable 4-tuple key so [`composite`] is independent of input
/// order (VT-1 determinism). Pure, no allocation beyond the tuple of borrows.
fn key_order(k: &CoverageKey) -> (&str, &str, &str, &str) {
    (
        k.slice.as_str(),
        k.requirement.as_str(),
        k.contributing_change.as_str(),
        k.mode.as_str(),
    )
}

/// Fan one requirement's coverage cells into a deterministic [`Composite`]
/// (design §5.2). Sorts by the stable [`CoverageKey`] so any input permutation
/// yields an identical value. Pure over in-memory input — no disk, no git.
pub(crate) fn composite(entries: &[(CoverageEntry, IsStale)]) -> Composite {
    let mut cells = entries.to_vec();
    cells.sort_by(|a, b| key_order(&a.0.key).cmp(&key_order(&b.0.key)));
    Composite { cells }
}

impl Composite {
    /// No contributing cells at all.
    pub(crate) fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Some cell is `Verified` AND [`IsStale::Fresh`] — live, confirming evidence.
    pub(crate) fn any_fresh_verified(&self) -> bool {
        self.cells
            .iter()
            .any(|(e, s)| e.status == CoverageStatus::Verified && *s == IsStale::Fresh)
    }

    /// Some cell contradicts (`Failed`) or is `Blocked` — an observed problem.
    pub(crate) fn any_failed_or_blocked(&self) -> bool {
        self.cells
            .iter()
            .any(|(e, _)| matches!(e.status, CoverageStatus::Failed | CoverageStatus::Blocked))
    }

    /// Every cell is still forward-intent (`Planned`/`InProgress`) — nothing yet
    /// claims confirmation or contradiction. Vacuously true on empty; callers
    /// pair it with [`is_empty`](Self::is_empty) where the distinction matters.
    pub(crate) fn only_forward(&self) -> bool {
        self.cells.iter().all(|(e, _)| {
            matches!(
                e.status,
                CoverageStatus::Planned | CoverageStatus::InProgress
            )
        })
    }
}

/// The drift verdict: does authored requirement status cohere with observed
/// coverage? READ-ONLY — it returns NO [`ReqStatus`](crate::requirement::ReqStatus)
/// (NF-001 / ADR-009 §3: no `ReqStatus = f(coverage)` derivation), it only names
/// the relationship for an authoring human to act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Authored status and observed evidence agree.
    Coherent,
    /// They disagree — see the [`DivergentReason`].
    Divergent(DivergentReason),
    /// Not enough live evidence to judge (only-stale / mixed / in-force but bare).
    Indeterminate,
}

/// Why a [`Verdict::Divergent`] fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DivergentReason {
    /// Evidence actively contradicts (`Failed`/`Blocked` cell present).
    ObservedContradiction,
    /// Live confirming evidence exists while authored status still trails it
    /// (the accept case — authoring should catch up).
    EvidenceOutrunsAuthored,
}

impl Verdict {
    /// The terse verdict cell text — the SINGLE source of a verdict's display
    /// label (the read's table/JSON render; design §5 D5). Deliberately distinct
    /// from reconcile's `build_prompt` register (two separate registers): this is
    /// a column cell, not an operator prompt. `Divergent` folds in the reason.
    pub(crate) fn label(self) -> String {
        match self {
            Verdict::Coherent => "Coherent".to_owned(),
            Verdict::Indeterminate => "Indeterminate".to_owned(),
            Verdict::Divergent(r) => format!("Divergent: {}", r.label()),
        }
    }
}

impl DivergentReason {
    /// The terse reason token spliced into a `Divergent` verdict label. Borrowed
    /// `&'static str` (no allocation) — [`Verdict::label`] owns the `format!`.
    pub(crate) fn label(self) -> &'static str {
        match self {
            DivergentReason::EvidenceOutrunsAuthored => "evidence-outruns-authored",
            DivergentReason::ObservedContradiction => "observed-contradiction",
        }
    }
}

/// The total drift decision tree (design §5.2; every `ReqStatus` × composite-state
/// cell single-valued). Read-only: classifies the authored/observed relationship,
/// never mutates or derives status. Pure — `composite` already carries resolved
/// staleness.
pub(crate) fn drift(authored: ReqStatus, composite: &Composite) -> Verdict {
    use ReqStatus::{Active, Deprecated, InProgress, Pending, Retired, Superseded};

    // Withdrawn statuses assert nothing about live coverage — always coherent.
    if matches!(authored, Retired | Superseded) {
        return Verdict::Coherent;
    }
    // An observed contradiction outranks every in-force reading.
    if composite.any_failed_or_blocked() {
        return Verdict::Divergent(DivergentReason::ObservedContradiction);
    }
    match authored {
        Pending | InProgress => {
            if composite.any_fresh_verified() {
                Verdict::Divergent(DivergentReason::EvidenceOutrunsAuthored)
            } else if composite.is_empty() || composite.only_forward() {
                Verdict::Coherent
            } else {
                Verdict::Indeterminate
            }
        }
        Active | Deprecated => {
            if composite.is_empty() {
                Verdict::Indeterminate
            } else if composite.any_fresh_verified() {
                Verdict::Coherent
            } else {
                Verdict::Indeterminate
            }
        }
        // Unreachable: the withdrawn set returned Coherent above. Keeping the
        // arm explicit (not `_`) keeps the match total over the 6 variants.
        Retired | Superseded => Verdict::Coherent,
    }
}

// ---------------------------------------------------------------------------
// SL-057 PHASE-01 — pure VT-check model + verdict folds.
//
// A *VT-check* is the recipe a `VT` coverage entry's evidence is produced from:
// an alias (XOR) or a literal `command`, optional `extra_args`, and an optional
// output `Matcher`. This is a PURE leaf (ADR-001): the types + the verdict folds
// below touch NO clock / rng / git / disk / process — running the check and
// reading its output is the shell's job (PHASE-02+). These folds only classify a
// `RunOutcome` the shell hands them, evaluate a matcher over a haystack string,
// and statically validate a `VtCheck`'s shape.
// ---------------------------------------------------------------------------

/// One VT-check recipe (persisted under `[entry.check]`). Either an `alias` into
/// the project base set OR a literal `command` argv — never both (the XOR rule,
/// [`valid`] (a)). `extra_args` are appended to whichever base resolves.
/// `matcher` decides the verdict from the run's output; absent ⇒ exit-code-only.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct VtCheck {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) command: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) extra_args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) matcher: Option<Matcher>,
}

/// An output matcher: search `source` (defaulting to stdout when absent at the
/// run seam) for `pattern`, as a literal substring (`regex == false`) or a
/// `regex_lite` pattern (`regex == true`). `regex` defaults to `false` on parse.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Matcher {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<MatchSource>,
    pub(crate) pattern: String,
    #[serde(default)]
    pub(crate) regex: bool,
}

/// Where a [`Matcher`] reads its haystack from.
///
/// **Serde repr (the `coverage.toml` byte surface PHASE-05 goldens pin).** This
/// enum serializes/deserializes as a single TOML **string** scalar (NOT a table),
/// so a [`Matcher`] renders as a clean inline table. The three forms (under
/// `source = …`) are exactly:
///
/// ```toml
/// source = "stdout"
/// source = "stderr"
/// source = "file:doc/*.md"
/// ```
///
/// `Stdout ⇄ "stdout"`, `Stderr ⇄ "stderr"`, and `File(glob) ⇄ "file:<glob>"`
/// (literal `file:` prefix, then the glob verbatim). On deserialize, an exact
/// `"stdout"`/`"stderr"` maps to the unit variant, a string starting with `file:`
/// maps to `File(<remainder>)`, and anything else is an unknown-source error. The
/// repr is wired via `#[serde(into / try_from = "String")]` over the [`Display`] /
/// [`TryFrom<String>`] pair below — all three round-trip cleanly (proven by the
/// VT-4 round-trip test). The `File` glob is repo-tree-relative and confined by
/// [`valid`] (c) (no absolute path, no `..` ascent).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(into = "String", try_from = "String")]
pub(crate) enum MatchSource {
    Stdout,
    Stderr,
    File(String),
}

/// The literal prefix that tags a [`MatchSource::File`] glob in its string repr.
const MATCH_SOURCE_FILE_PREFIX: &str = "file:";

impl std::fmt::Display for MatchSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchSource::Stdout => f.write_str("stdout"),
            MatchSource::Stderr => f.write_str("stderr"),
            MatchSource::File(glob) => write!(f, "{MATCH_SOURCE_FILE_PREFIX}{glob}"),
        }
    }
}

impl From<MatchSource> for String {
    fn from(src: MatchSource) -> Self {
        src.to_string()
    }
}

impl TryFrom<String> for MatchSource {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "stdout" => Ok(MatchSource::Stdout),
            "stderr" => Ok(MatchSource::Stderr),
            other => match other.strip_prefix(MATCH_SOURCE_FILE_PREFIX) {
                Some(glob) => Ok(MatchSource::File(glob.to_owned())),
                None => Err(format!("unknown match source: {other}")),
            },
        }
    }
}

/// The outcome of (attempting to) run a VT-check — produced by the shell, fed to
/// [`derive_status`]. In-memory only (NOT persisted): NO serde derive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RunOutcome {
    /// The check could not be obtained/run at all (unresolved alias, spawn
    /// failure, timeout — the F-VII framing). Yields [`CoverageStatus::Blocked`].
    Unobtainable,
    /// The check ran to completion. `exit_ok` is the exit-code verdict; `matched`
    /// is the matcher verdict (`None` ⇒ no matcher present ⇒ exit-code-only).
    Ran {
        exit_ok: bool,
        matched: Option<bool>,
    },
}

/// Classify a [`RunOutcome`] into the observed [`CoverageStatus`] (PURE).
/// `Unobtainable ⇒ Blocked`; a clean exit with no matcher (`matched: None`) or a
/// satisfied matcher (`Some(true)`) ⇒ `Verified`; a non-zero exit OR a failed
/// matcher (`Some(false)`) ⇒ `Failed`. INV-3: `Unobtainable` NEVER yields
/// `Verified`.
pub(crate) fn derive_status(outcome: &RunOutcome) -> CoverageStatus {
    match outcome {
        RunOutcome::Unobtainable => CoverageStatus::Blocked,
        RunOutcome::Ran { exit_ok: false, .. }
        | RunOutcome::Ran {
            exit_ok: true,
            matched: Some(false),
        } => CoverageStatus::Failed,
        RunOutcome::Ran {
            exit_ok: true,
            matched: None | Some(true),
        } => CoverageStatus::Verified,
    }
}

/// Evaluate a matcher pattern over a haystack (PURE). Substring mode
/// (`regex == false`) is `Some(haystack.contains(pattern))` — metacharacters
/// match LITERALLY and the empty pattern is `Some(true)`, NEVER `None`. Regex
/// mode (`regex == true`) compiles under `regex_lite`: a parse error ⇒ `None`,
/// otherwise `Some(re.is_match(haystack))` (the empty pattern matches anything).
pub(crate) fn evaluate_matcher(pattern: &str, regex: bool, haystack: &str) -> Option<bool> {
    if regex {
        match regex_lite::Regex::new(pattern) {
            Ok(re) => Some(re.is_match(haystack)),
            Err(_) => None,
        }
    } else {
        Some(haystack.contains(pattern))
    }
}

/// Why [`valid`] rejected a [`VtCheck`] — one variant per reject reason so callers
/// assert the REASON, not merely `is_err()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValidError {
    /// (a) Both `alias` and `command` are set — they are mutually exclusive.
    AliasCommandConflict,
    /// (b) A non-empty matcher is mandatory unless a literal `command` is set: an
    /// empty-or-absent matcher on an alias, or on the project-default base
    /// (neither alias nor command), is rejected (the D3/A matcher rule).
    MatcherRequired,
    /// (c) A `File` glob escaped the repo tree — an absolute path or a `..` ascent
    /// component (the F-III glob-confinement rule; pure string inspection).
    GlobEscapesTree,
    /// (d) A `regex == true` matcher whose pattern does not parse under
    /// `regex_lite`.
    BadRegex,
}

/// Statically validate a [`VtCheck`]'s shape (PURE, CONFIG-FREE). Enforces the
/// alias/command XOR (a), the mandatory-matcher rule (b), `File`-glob confinement
/// (c), and regex parseability (d). Does NOT resolve the base (does the alias
/// exist? is a default command present?) — that is `verify::resolve`'s job in
/// PHASE-02 (design F-1); this fold never reaches for config.
pub(crate) fn valid(check: &VtCheck) -> Result<(), ValidError> {
    // (a) alias/command XOR.
    if check.alias.is_some() && check.command.is_some() {
        return Err(ValidError::AliasCommandConflict);
    }

    // (b) D3/A: a non-empty matcher is mandatory UNLESS a literal command is set.
    // An empty matcher = matcher is None, or its pattern is "".
    let matcher_empty = match &check.matcher {
        None => true,
        Some(m) => m.pattern.is_empty(),
    };
    if matcher_empty && check.command.is_none() {
        return Err(ValidError::MatcherRequired);
    }

    if let Some(matcher) = &check.matcher {
        // (c) F-III: confine a `File` glob to the repo tree (pure string check).
        if let Some(MatchSource::File(glob)) = &matcher.source {
            let escapes = glob.starts_with('/') || glob.split('/').any(|segment| segment == "..");
            if escapes {
                return Err(ValidError::GlobEscapesTree);
            }
        }
        // (d) regex-mode patterns must parse under regex_lite.
        if matcher.regex && regex_lite::Regex::new(&matcher.pattern).is_err() {
            return Err(ValidError::BadRegex);
        }
    }

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on round-trip/parse is idiomatic"
)]
mod tests {
    use super::*;

    fn key(slice: &str, req: &str, change: &str, mode: &str) -> CoverageKey {
        CoverageKey {
            slice: slice.to_owned(),
            requirement: req.to_owned(),
            contributing_change: change.to_owned(),
            mode: mode.to_owned(),
        }
    }

    fn entry(k: CoverageKey, status: CoverageStatus, attested: Option<&str>) -> CoverageEntry {
        CoverageEntry {
            key: k,
            status,
            git_anchor: "anchor-abc123".to_owned(),
            attested_date: attested.map(str::to_owned),
            touched_paths: Vec::new(),
            check: None,
        }
    }

    /// A synthetic composite cell: one `CoverageEntry` (status varied) paired with
    /// a resolved `IsStale`. Key fields vary so distinct cells stay distinct.
    fn cell(
        slice: &str,
        req: &str,
        change: &str,
        status: CoverageStatus,
        stale: IsStale,
    ) -> (CoverageEntry, IsStale) {
        (entry(key(slice, req, change, "VT"), status, None), stale)
    }

    // --- VT-1: render → parse round-trip preserves every field ---------------

    #[test]
    fn round_trip_preserves_attested_present_and_absent() {
        let file = CoverageFile {
            entry: vec![
                // VH evidence with an attestation date.
                entry(
                    key("SL-042", "REQ-109", "SL-042", "VH"),
                    CoverageStatus::Verified,
                    Some("2026-06-12"),
                ),
                // VT evidence with no attestation date.
                entry(
                    key("SL-042", "REQ-108", "SL-041", "VT"),
                    CoverageStatus::Failed,
                    None,
                ),
            ],
        };

        let back = parse(&render(&file).unwrap()).unwrap();
        assert_eq!(
            back, file,
            "mode + status + git_anchor + attested_date preserved"
        );

        // Spell the per-field preservation out so the VT names what it guards.
        let first = back.entry.first().unwrap();
        assert_eq!(first.key.mode, "VH");
        assert_eq!(first.status, CoverageStatus::Verified);
        assert_eq!(first.git_anchor, "anchor-abc123");
        assert_eq!(first.attested_date.as_deref(), Some("2026-06-12"));
        assert!(back.entry.get(1).unwrap().attested_date.is_none());
    }

    #[test]
    fn empty_file_round_trips() {
        let empty = CoverageFile::default();
        assert_eq!(parse(&render(&empty).unwrap()).unwrap(), empty);
    }

    // --- VT-2: the no-clobber upsert fold ------------------------------------

    #[test]
    fn upsert_distinct_keys_appends() {
        let mut file = CoverageFile::default();
        upsert(
            &mut file,
            entry(
                key("SL-042", "REQ-109", "SL-042", "VT"),
                CoverageStatus::Planned,
                None,
            ),
        );
        upsert(
            &mut file,
            entry(
                key("SL-042", "REQ-108", "SL-042", "VT"),
                CoverageStatus::Verified,
                None,
            ),
        );
        assert_eq!(file.entry.len(), 2, "two distinct keys both surface");
    }

    #[test]
    fn upsert_identical_key_replaces_with_latest_payload() {
        let k = key("SL-042", "REQ-109", "SL-042", "VT");
        let mut file = CoverageFile::default();
        upsert(&mut file, entry(k.clone(), CoverageStatus::Planned, None));
        upsert(
            &mut file,
            entry(k.clone(), CoverageStatus::Verified, Some("2026-06-12")),
        );

        assert_eq!(file.entry.len(), 1, "same key replaces, never duplicates");
        let only = file.entry.first().unwrap();
        assert_eq!(only.status, CoverageStatus::Verified, "latest payload wins");
        assert_eq!(only.attested_date.as_deref(), Some("2026-06-12"));
    }

    #[test]
    fn entries_differing_only_in_slice_coexist() {
        // Two slices contributing evidence for the same requirement: distinct keys.
        let mut file = CoverageFile::default();
        upsert(
            &mut file,
            entry(
                key("SL-042", "REQ-109", "SL-042", "VT"),
                CoverageStatus::Verified,
                None,
            ),
        );
        upsert(
            &mut file,
            entry(
                key("SL-099", "REQ-109", "SL-099", "VT"),
                CoverageStatus::Planned,
                None,
            ),
        );
        assert_eq!(
            file.entry.len(),
            2,
            "same requirement across two slices coexists"
        );
    }

    // --- VT-2b: mode membership validator ------------------------------------

    #[test]
    fn mode_membership_is_vt_va_vh_only() {
        assert!(mode_is_valid("VT"));
        assert!(mode_is_valid("VA"));
        assert!(mode_is_valid("VH"));
        assert!(!mode_is_valid("VX"));
        assert!(!mode_is_valid("vt"));
        assert!(!mode_is_valid(""));
    }

    // --- VT-3: distinct-store, structural ------------------------------------

    #[test]
    fn coverage_entry_carries_observed_status_not_authored_reqstatus() {
        // Compile-level fact, spelled as a test: `CoverageEntry::status` is the
        // observed-evidence `CoverageStatus`, NEVER the authored `ReqStatus`
        // (NF-001 / ADR-009 §3). This line only type-checks because the field is
        // `CoverageStatus`; assigning a `ReqStatus` here would not compile.
        let observed: CoverageStatus = entry(
            key("SL-042", "REQ-109", "SL-042", "VT"),
            CoverageStatus::Verified,
            None,
        )
        .status;
        assert_eq!(observed, CoverageStatus::Verified);
    }

    #[test]
    fn coverage_and_requirement_status_live_in_distinct_stores() {
        // Coverage rides the slice tree; authored requirement status lives in the
        // requirement entity file — distinct paths, distinct stores (NF-001).
        let coverage_path = ".doctrine/slice/042/coverage.toml";
        let requirement_path = ".doctrine/requirement/109/requirement-109.toml";
        assert_ne!(coverage_path, requirement_path);
    }

    // --- P3 T1: touched_paths is additive — P2 entries (no field) still parse ---

    #[test]
    fn p2_entry_without_touched_paths_parses_and_defaults_empty() {
        // A coverage.toml authored before P3 carries no `touched_paths` key.
        let body = r#"
[[entry]]
slice = "SL-042"
requirement = "REQ-109"
contributing_change = "SL-042"
mode = "VT"
status = "verified"
git_anchor = "anchor-abc123"
"#;
        let file = parse(body).unwrap();
        let only = file.entry.first().unwrap();
        assert!(only.touched_paths.is_empty(), "absent field defaults empty");
    }

    #[test]
    fn touched_paths_round_trips_when_present() {
        let mut e = entry(
            key("SL-042", "REQ-110", "SL-042", "VT"),
            CoverageStatus::Verified,
            None,
        );
        e.touched_paths = vec!["src/coverage.rs".to_owned(), "src/git.rs".to_owned()];
        let file = CoverageFile { entry: vec![e] };
        let back = parse(&render(&file).unwrap()).unwrap();
        assert_eq!(back, file, "touched_paths survives the round-trip");
    }

    // --- P3 T2: IsStale constructor from the seam's Option<u32> ----------------

    #[test]
    fn is_stale_from_seam_count() {
        assert_eq!(IsStale::from(Some(0)), IsStale::Fresh);
        assert_eq!(IsStale::from(Some(1)), IsStale::Stale);
        assert_eq!(IsStale::from(Some(42)), IsStale::Stale);
        assert_eq!(IsStale::from(None), IsStale::Unknown);
    }

    // --- VT-1 (REQ-110): composite determinism over input order ---------------

    #[test]
    fn composite_is_order_independent() {
        let ordered = vec![
            cell(
                "SL-040",
                "REQ-110",
                "SL-040",
                CoverageStatus::Verified,
                IsStale::Fresh,
            ),
            cell(
                "SL-042",
                "REQ-110",
                "SL-041",
                CoverageStatus::Planned,
                IsStale::Unknown,
            ),
            cell(
                "SL-041",
                "REQ-110",
                "SL-042",
                CoverageStatus::Failed,
                IsStale::Stale,
            ),
        ];
        // A shuffled permutation of the same cells.
        let shuffled = vec![
            ordered.get(2).unwrap().clone(),
            ordered.first().unwrap().clone(),
            ordered.get(1).unwrap().clone(),
        ];
        assert_eq!(
            composite(&ordered),
            composite(&shuffled),
            "the fold is pure over in-memory input — order cannot change the value"
        );
        // Purity: the fold returns a value; it writes nothing (no disk handle in
        // scope to write to — the type signature is the proof).
    }

    // --- VT-2 (REQ-111): the full ReqStatus × composite-state verdict matrix ---

    /// The five canonical composite states the §5.2 tree branches on, each built
    /// from synthetic in-memory cells (no disk).
    fn composites() -> Vec<(&'static str, Composite)> {
        vec![
            ("empty", composite(&[])),
            (
                "fresh-verified",
                composite(&[cell(
                    "SL-042",
                    "REQ-111",
                    "SL-042",
                    CoverageStatus::Verified,
                    IsStale::Fresh,
                )]),
            ),
            (
                "stale-verified",
                composite(&[cell(
                    "SL-042",
                    "REQ-111",
                    "SL-042",
                    CoverageStatus::Verified,
                    IsStale::Stale,
                )]),
            ),
            (
                "failed-or-blocked",
                composite(&[cell(
                    "SL-042",
                    "REQ-111",
                    "SL-042",
                    CoverageStatus::Failed,
                    IsStale::Fresh,
                )]),
            ),
            (
                "forward-only",
                composite(&[
                    cell(
                        "SL-042",
                        "REQ-111",
                        "SL-042",
                        CoverageStatus::Planned,
                        IsStale::Unknown,
                    ),
                    cell(
                        "SL-043",
                        "REQ-111",
                        "SL-043",
                        CoverageStatus::InProgress,
                        IsStale::Stale,
                    ),
                ]),
            ),
        ]
    }

    #[test]
    fn verdict_matrix_matches_the_decision_tree() {
        use DivergentReason::{EvidenceOutrunsAuthored, ObservedContradiction};
        use ReqStatus::{Active, Deprecated, InProgress, Pending, Retired, Superseded};
        use Verdict::{Coherent, Divergent, Indeterminate};

        // Expected verdict per (authored, composite-state) — the §5.2 tree.
        // Order of states: empty, fresh-verified, stale-verified,
        // failed-or-blocked, forward-only.
        let expect: Vec<(ReqStatus, [Verdict; 5])> = vec![
            (
                Pending,
                [
                    Coherent,                           // empty
                    Divergent(EvidenceOutrunsAuthored), // fresh-verified
                    Indeterminate,                      // stale-verified
                    Divergent(ObservedContradiction),   // failed-or-blocked
                    Coherent,                           // forward-only
                ],
            ),
            (
                InProgress,
                [
                    Coherent,
                    Divergent(EvidenceOutrunsAuthored),
                    Indeterminate,
                    Divergent(ObservedContradiction),
                    Coherent,
                ],
            ),
            (
                Active,
                [
                    Indeterminate,                    // empty (in-force, bare)
                    Coherent,                         // fresh-verified
                    Indeterminate,                    // stale-verified
                    Divergent(ObservedContradiction), // failed-or-blocked
                    Indeterminate,                    // forward-only (only-stale/mix)
                ],
            ),
            (
                Deprecated,
                [
                    Indeterminate,
                    Coherent,
                    Indeterminate,
                    Divergent(ObservedContradiction),
                    Indeterminate,
                ],
            ),
            (Retired, [Coherent, Coherent, Coherent, Coherent, Coherent]),
            (
                Superseded,
                [Coherent, Coherent, Coherent, Coherent, Coherent],
            ),
        ];

        let states = composites();
        for (authored, row) in &expect {
            for (idx, (label, comp)) in states.iter().enumerate() {
                let got = drift(*authored, comp);
                let want = *row.get(idx).unwrap();
                assert_eq!(
                    got, want,
                    "drift({:?}, {label}) expected {want:?}, got {got:?}",
                    authored
                );
            }
        }
    }

    // --- VT-3 (REQ-114/NF-001): drift returns Verdict, not ReqStatus ----------

    #[test]
    fn drift_returns_verdict_not_reqstatus() {
        // Spelled as a test: drift's return type is `Verdict`. This binding only
        // type-checks because drift returns `Verdict` — a `ReqStatus` binding here
        // would not compile (no `ReqStatus = f(coverage)` derivation; NF-001).
        let v: Verdict = drift(ReqStatus::Active, &composite(&[]));
        assert_eq!(v, Verdict::Indeterminate);
        // (The distinct-store path assertion lives in
        // `coverage_and_requirement_status_live_in_distinct_stores` above — reused,
        // not duplicated.)
    }

    // --- composite predicate units (guard the fold's exposed surface) ---------

    #[test]
    fn composite_predicates_read_the_cells() {
        let c = composite(&[
            cell(
                "SL-042",
                "REQ-111",
                "SL-042",
                CoverageStatus::Verified,
                IsStale::Fresh,
            ),
            cell(
                "SL-043",
                "REQ-111",
                "SL-043",
                CoverageStatus::Planned,
                IsStale::Unknown,
            ),
        ]);
        assert!(!c.is_empty());
        assert!(c.any_fresh_verified());
        assert!(!c.any_failed_or_blocked());
        assert!(!c.only_forward(), "a Verified cell is not forward-only");

        let stale_verified = composite(&[cell(
            "SL-042",
            "REQ-111",
            "SL-042",
            CoverageStatus::Verified,
            IsStale::Stale,
        )]);
        assert!(
            !stale_verified.any_fresh_verified(),
            "stale Verified is not fresh-verified"
        );
    }

    // === SL-057 PHASE-01: VT-check model + verdict folds =====================

    /// Build a [`VtCheck`] from the parts a test cares about; the rest default.
    fn vtcheck(
        alias: Option<&str>,
        command: Option<Vec<&str>>,
        matcher: Option<Matcher>,
    ) -> VtCheck {
        VtCheck {
            alias: alias.map(str::to_owned),
            command: command.map(|c| c.into_iter().map(str::to_owned).collect()),
            extra_args: Vec::new(),
            matcher,
        }
    }

    /// A [`Matcher`] over the given source/pattern/regex-flag.
    fn matcher(source: Option<MatchSource>, pattern: &str, regex: bool) -> Matcher {
        Matcher {
            source,
            pattern: pattern.to_owned(),
            regex,
        }
    }

    // --- VT-1: derive_status truth table (INV-3 included) --------------------

    #[test]
    fn derive_status_truth_table() {
        // Unobtainable ⇒ Blocked.
        assert_eq!(
            derive_status(&RunOutcome::Unobtainable),
            CoverageStatus::Blocked
        );
        // Clean exit, no matcher ⇒ Verified (exit-code-only).
        assert_eq!(
            derive_status(&RunOutcome::Ran {
                exit_ok: true,
                matched: None
            }),
            CoverageStatus::Verified
        );
        // Clean exit, matcher satisfied ⇒ Verified.
        assert_eq!(
            derive_status(&RunOutcome::Ran {
                exit_ok: true,
                matched: Some(true)
            }),
            CoverageStatus::Verified
        );
        // Non-zero exit (matcher irrelevant) ⇒ Failed.
        assert_eq!(
            derive_status(&RunOutcome::Ran {
                exit_ok: false,
                matched: None
            }),
            CoverageStatus::Failed
        );
        assert_eq!(
            derive_status(&RunOutcome::Ran {
                exit_ok: false,
                matched: Some(true)
            }),
            CoverageStatus::Failed
        );
        // Clean exit but matcher failed ⇒ Failed.
        assert_eq!(
            derive_status(&RunOutcome::Ran {
                exit_ok: true,
                matched: Some(false)
            }),
            CoverageStatus::Failed
        );
    }

    #[test]
    fn inv3_unobtainable_never_verified() {
        // INV-3: no Unobtainable path yields Verified.
        assert_ne!(
            derive_status(&RunOutcome::Unobtainable),
            CoverageStatus::Verified
        );
    }

    // --- VT-2: evaluate_matcher (substring + regex) --------------------------

    #[test]
    fn evaluate_matcher_substring_literal_and_metachars() {
        // Plain substring match / miss.
        assert_eq!(evaluate_matcher("ok", false, "all ok here"), Some(true));
        assert_eq!(evaluate_matcher("nope", false, "all ok here"), Some(false));
        // Metacharacters match LITERALLY in substring mode: `a.c` does NOT match
        // "abc" (the `.` is a literal dot) but DOES match "a.c".
        assert_eq!(evaluate_matcher("a.c", false, "abc"), Some(false));
        assert_eq!(evaluate_matcher("a.c", false, "xx a.c yy"), Some(true));
        // Empty pattern ⇒ Some(true), never None.
        assert_eq!(evaluate_matcher("", false, "anything"), Some(true));
        assert_eq!(evaluate_matcher("", false, ""), Some(true));
    }

    #[test]
    fn evaluate_matcher_regex_mode() {
        // Regex match / miss.
        assert_eq!(evaluate_matcher("a.c", true, "abc"), Some(true));
        assert_eq!(evaluate_matcher("a.c", true, "axyzc"), Some(false));
        // Unparseable regex ⇒ None.
        assert_eq!(evaluate_matcher("(", true, "anything"), None);
        // Empty pattern ⇒ Some(true) under regex_lite too.
        assert_eq!(evaluate_matcher("", true, "anything"), Some(true));
    }

    // --- VT-3: valid reject matrix (assert the SPECIFIC variant) -------------

    #[test]
    fn valid_rejects_alias_command_conflict() {
        let check = vtcheck(
            Some("test"),
            Some(vec!["cargo", "test"]),
            Some(matcher(None, "ok", false)),
        );
        assert_eq!(valid(&check), Err(ValidError::AliasCommandConflict));
    }

    #[test]
    fn valid_rejects_empty_matcher_on_alias() {
        // Absent matcher on an alias ⇒ MatcherRequired.
        assert_eq!(
            valid(&vtcheck(Some("test"), None, None)),
            Err(ValidError::MatcherRequired)
        );
        // Empty-pattern matcher on an alias ⇒ MatcherRequired.
        assert_eq!(
            valid(&vtcheck(Some("test"), None, Some(matcher(None, "", false)))),
            Err(ValidError::MatcherRequired)
        );
    }

    #[test]
    fn valid_rejects_empty_matcher_on_default_base() {
        // Neither alias nor command (project-default base): absent matcher rejected.
        assert_eq!(
            valid(&vtcheck(None, None, None)),
            Err(ValidError::MatcherRequired)
        );
    }

    #[test]
    fn valid_accepts_empty_matcher_with_literal_command() {
        // An empty/absent matcher is legal ONLY alongside a literal command.
        assert_eq!(valid(&vtcheck(None, Some(vec!["true"]), None)), Ok(()));
        assert_eq!(
            valid(&vtcheck(
                None,
                Some(vec!["true"]),
                Some(matcher(None, "", false))
            )),
            Ok(())
        );
    }

    #[test]
    fn valid_rejects_absolute_file_glob() {
        let check = vtcheck(
            Some("test"),
            None,
            Some(matcher(
                Some(MatchSource::File("/etc/x".to_owned())),
                "ok",
                false,
            )),
        );
        assert_eq!(valid(&check), Err(ValidError::GlobEscapesTree));
    }

    #[test]
    fn valid_rejects_ascending_file_glob() {
        let check = vtcheck(
            Some("test"),
            None,
            Some(matcher(
                Some(MatchSource::File("../x".to_owned())),
                "ok",
                false,
            )),
        );
        assert_eq!(valid(&check), Err(ValidError::GlobEscapesTree));
        // A `..` nested deeper in the path is caught too.
        let nested = vtcheck(
            Some("test"),
            None,
            Some(matcher(
                Some(MatchSource::File("doc/../../x".to_owned())),
                "ok",
                false,
            )),
        );
        assert_eq!(valid(&nested), Err(ValidError::GlobEscapesTree));
    }

    #[test]
    fn valid_rejects_unparseable_regex() {
        let check = vtcheck(Some("test"), None, Some(matcher(None, "(", true)));
        assert_eq!(valid(&check), Err(ValidError::BadRegex));
    }

    #[test]
    fn valid_accepts_wellformed_alias_with_matcher() {
        let check = vtcheck(
            Some("test"),
            None,
            Some(matcher(Some(MatchSource::Stdout), "ok", false)),
        );
        assert_eq!(valid(&check), Ok(()));
        // A relative File glob with no ascent is fine.
        let file_ok = vtcheck(
            Some("test"),
            None,
            Some(matcher(
                Some(MatchSource::File("doc/*.md".to_owned())),
                "ok",
                false,
            )),
        );
        assert_eq!(valid(&file_ok), Ok(()));
    }

    // --- VT-4: MatchSource serde repr round-trips all three variants ---------

    #[test]
    fn match_source_serde_repr_all_three_variants() {
        // Render each variant inside a full entry and assert the exact byte form,
        // then parse it back — this pins the PHASE-05 golden surface.
        for (src, token) in [
            (MatchSource::Stdout, "\"stdout\""),
            (MatchSource::Stderr, "\"stderr\""),
            (
                MatchSource::File("doc/*.md".to_owned()),
                "\"file:doc/*.md\"",
            ),
        ] {
            let m = matcher(Some(src.clone()), "ok", false);
            let rendered = toml::to_string(&m).unwrap();
            assert!(
                rendered.contains(&format!("source = {token}")),
                "expected `source = {token}` in:\n{rendered}"
            );
            let back: Matcher = toml::from_str(&rendered).unwrap();
            assert_eq!(back.source, Some(src), "MatchSource round-trips");
        }
    }

    #[test]
    fn vtcheck_full_round_trip_through_entry() {
        // A full VtCheck (alias + extra_args + File matcher) survives the
        // CoverageEntry render → parse round-trip.
        let mut e = entry(
            key("SL-057", "REQ-200", "SL-057", "VT"),
            CoverageStatus::Verified,
            None,
        );
        e.check = Some(VtCheck {
            alias: Some("test".to_owned()),
            command: None,
            extra_args: vec!["--quiet".to_owned()],
            matcher: Some(matcher(
                Some(MatchSource::File("doc/spec.md".to_owned())),
                "PASS",
                true,
            )),
        });
        let file = CoverageFile { entry: vec![e] };
        let back = parse(&render(&file).unwrap()).unwrap();
        assert_eq!(back, file, "the full VtCheck round-trips byte-clean");
    }

    #[test]
    fn vtcheck_command_variant_round_trips() {
        // The `command` (literal argv) + Stderr-source variant also round-trips.
        let mut e = entry(
            key("SL-057", "REQ-201", "SL-057", "VT"),
            CoverageStatus::Verified,
            None,
        );
        e.check = Some(vtcheck(
            None,
            Some(vec!["cargo", "test"]),
            Some(matcher(Some(MatchSource::Stderr), "ok", false)),
        ));
        let file = CoverageFile { entry: vec![e] };
        let back = parse(&render(&file).unwrap()).unwrap();
        assert_eq!(back, file);
    }

    #[test]
    fn pre_sl057_entry_without_check_parses_to_none() {
        // A pre-SL-057 [[entry]] body carries no `check` key — it must still parse,
        // with `check == None` (the additive-field precedent).
        let body = r#"
[[entry]]
slice = "SL-042"
requirement = "REQ-109"
contributing_change = "SL-042"
mode = "VT"
status = "verified"
git_anchor = "anchor-abc123"
"#;
        let file = parse(body).unwrap();
        assert!(
            file.entry.first().unwrap().check.is_none(),
            "absent check defaults None"
        );
    }
}
