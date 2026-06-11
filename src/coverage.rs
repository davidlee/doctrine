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

// The whole coverage substrate is a leaf built ahead of its consumer: P2 lands the
// types + pure folds; the reconcile *reader* that constructs and queries them is
// the dependent P3/P4. Until then every item here is dead in the bins/lib build,
// so the module's surface carries a self-clearing `not(test)` dead_code expect (the
// `dead-code-self-clearing-leaf` precedent). It scopes to `not(test)` because under
// `cfg(test)` the VTs below exercise every item, so `dead_code` would not fire and an
// unconditional `expect` would itself be unfulfilled. The gate runs plain `cargo
// clippy` (bins/lib, no test cfg) where the items are genuinely dead — the
// expectation is fulfilled exactly where the lint applies. This expect retires
// itself the moment P3/P4 wires a consumer.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "coverage substrate (SL-042 P2) is a leaf built ahead of its \
                  P3/P4 reconcile-reader consumer — every item is dead in the \
                  bins/lib build until that consumer is wired"
    )
)]

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
}

/// The full `coverage.toml` read/written as data: a `[[entry]]` array-of-tables.
/// Defaults to empty so a fresh / absent file parses.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub(crate) struct CoverageFile {
    #[serde(default)]
    pub(crate) entry: Vec<CoverageEntry>,
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
}
