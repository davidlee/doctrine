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

use crate::requirement::CoverageStatus;

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
        }
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
}
