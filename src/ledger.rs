// SPDX-License-Identifier: GPL-3.0-only
//! `ledger` — the dispatch run-ledger read/write model (SL-064 PHASE-03).
//!
//! The committed coordination state that lives on `dispatch/<slice>` at
//! `.doctrine/dispatch/<slice>/{journal,boundaries,orthogonal}.toml` (design
//! §4.1). Three manifests, written on different funnel events, that
//! prepare-review (PHASE-04) and integrate (PHASE-05) consume:
//!
//! - `journal.toml`   — CAS projection rows (ADR-012 D4); written at sync.
//! - `boundaries.toml` — per-phase code-commit OIDs (design §4.3); written per
//!   phase during the funnel, the claude-arm `phase/<slice>-NN` cut's input.
//! - `orthogonal.toml` — entities projected ahead independently (design §4.2);
//!   written per projection, the `review/<slice>` EXCLUDE's input.
//!
//! Tier carve-out: this is runtime-coordination state that is *git-committed*
//! (ADR-012 D4 crash-durability), kept off `.doctrine/state/` (which is
//! gitignored) — a blessed exception, bounded to `dispatch/<slice>` branches.
//!
//! The top of this module is a pure read model (serde + `toml`, no clock/disk/
//! git — the `parse`/`to_toml` pair mirrors `crate::plan`). The clearly-marked
//! impure shell below (`record_*`/`read_*`) does the read-modify-write against
//! the manifest paths; it is the tested "recording surface" of EX-5 — a verb,
//! not hand-authored prose.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// --- pure read model ---------------------------------------------------------

/// Lifecycle status shared by a journal CAS row and an orthogonal-projection
/// mark. `verified` is the success terminus the `review/<slice>` EXCLUDE keys on
/// (design §4.2: an entity is excluded only when its ahead-projection is
/// journal-verified); `failed` falls back into the review bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LedgerStatus {
    /// Intent recorded; the ref mutation has not yet been confirmed applied.
    Pending,
    /// The projection applied and is confirmed.
    Verified,
    /// The projection failed or crashed; not safe to treat as applied.
    Failed,
}

/// `journal.toml` — the CAS projection rows (ADR-012 D4).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct Journal {
    #[serde(default, rename = "row")]
    pub rows: Vec<JournalRow>,
}

/// One CAS projection row. The compare-and-swap is the native `update-ref
/// <target_ref> <planned_new_oid> <expected_old_oid>` (design §4.1); replay
/// recomputes the planned output from `source_oid`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JournalRow {
    /// The source object the projection was computed from (replay input).
    pub source_oid: String,
    /// The ref this row mutates.
    pub target_ref: String,
    /// The ref's value the CAS requires (zero oid for a creation).
    pub expected_old_oid: String,
    /// The value the projection plans to write.
    pub planned_new_oid: String,
    /// The value actually written once applied (empty until applied).
    #[serde(default)]
    pub applied_new_oid: String,
    /// Lifecycle status of this projection.
    pub status: LedgerStatus,
}

/// `boundaries.toml` — per-phase code-commit OIDs for the claude-arm cut.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct Boundaries {
    #[serde(default, rename = "boundary")]
    pub rows: Vec<BoundaryRow>,
}

/// One phase's code boundary (design §4.3): `code_end_oid` is the worker code
/// commit *before* the knowledge record commit; an empty-code phase has
/// `code_start_oid == code_end_oid`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BoundaryRow {
    /// The `PHASE-NN` id this boundary belongs to.
    pub phase: String,
    /// HEAD before the phase's code landed.
    pub code_start_oid: String,
    /// The phase's cumulative code tip (pre-knowledge-record).
    pub code_end_oid: String,
}

/// `orthogonal.toml` — entities projected ahead of the impl bundle (design
/// §4.2). `review/<slice>` excludes a mark's `path` only when its `status` is
/// `verified`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct Orthogonal {
    #[serde(default, rename = "mark")]
    pub rows: Vec<OrthogonalMark>,
}

/// One slice-orthogonal projection mark.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OrthogonalMark {
    /// The entity that projected ahead (canonical id, e.g. `mem.…` / `ADR-012`).
    pub entity: String,
    /// The committed path excluded from the review bundle when verified.
    pub path: String,
    /// Whether the ahead-projection is confirmed (the EXCLUDE gate).
    pub status: LedgerStatus,
}

// Symbols below are test-live but have no *non-test* caller yet: the round-trip
// `parse`/`to_toml` surface, `read_journal` (stage-2 integrate, PHASE-05), and the
// funnel-time recording shell (`record_*`/`store`, wired by the dispatch funnel
// rewiring, PHASE-06). Each carries a per-symbol `cfg_attr(not(test))` expect so
// the test build — where they ARE called — sees no unfulfilled expect
// (mem.pattern.lint.dead-code-expect-vs-cfg-test); per-symbol, not a module
// blanket, so a regression in a now-live sibling still surfaces
// (mem.pattern.lint.blanket-dead-code-suppression-masks-siblings).
impl Journal {
    /// Parse a `journal.toml` body. An absent file is the caller's concern
    /// ([`read_journal`]); this parses a present body.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "stage-2 integrate is the first non-test reader (PHASE-05)"
        )
    )]
    pub(crate) fn parse(text: &str) -> anyhow::Result<Journal> {
        Ok(toml::from_str(text)?)
    }

    /// Serialize to a `journal.toml` body (serde-escaped — no raw splicing).
    pub(crate) fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string(self)?)
    }
}

impl Boundaries {
    /// Parse a `boundaries.toml` body.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "stage-2 integrate / funnel are the first non-test callers"
        )
    )]
    pub(crate) fn parse(text: &str) -> anyhow::Result<Boundaries> {
        Ok(toml::from_str(text)?)
    }

    /// Serialize to a `boundaries.toml` body.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "funnel-time recording is the first non-test writer (PHASE-06)"
        )
    )]
    pub(crate) fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string(self)?)
    }
}

impl Orthogonal {
    /// Parse an `orthogonal.toml` body.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "stage-2 integrate / funnel are the first non-test callers"
        )
    )]
    pub(crate) fn parse(text: &str) -> anyhow::Result<Orthogonal> {
        Ok(toml::from_str(text)?)
    }

    /// Serialize to an `orthogonal.toml` body.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "funnel-time recording is the first non-test writer (PHASE-06)"
        )
    )]
    pub(crate) fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string(self)?)
    }
}

// --- impure recording shell (the EX-5 recording surface) ---------------------

/// The `.doctrine/dispatch/<slice>/` coordination directory (design §4.1).
fn dispatch_dir(root: &Path, slice: u32) -> PathBuf {
    root.join(".doctrine")
        .join("dispatch")
        .join(slice.to_string())
}

/// Load a manifest from `<dispatch_dir>/<file>`, defaulting to empty when the
/// file is absent (VT-4 absent-file defaults). A present-but-malformed file is
/// a hard error.
fn load<T: DeserializeOwned + Default>(path: &Path) -> anyhow::Result<T> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(toml::from_str(&text)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(e.into()),
    }
}

/// Write a manifest to `path`, creating the coordination dir on first write.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "funnel-time recording is the first non-test writer (PHASE-06)"
    )
)]
fn store<T: Serialize>(path: &Path, manifest: &T) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, toml::to_string(manifest)?)?;
    Ok(())
}

/// Read `journal.toml` for `slice` (empty when absent).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "stage-2 integrate is the first non-test reader (PHASE-05)"
    )
)]
pub(crate) fn read_journal(root: &Path, slice: u32) -> anyhow::Result<Journal> {
    load(&dispatch_dir(root, slice).join("journal.toml"))
}

/// Read `boundaries.toml` for `slice` (empty when absent).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "funnel-time read-modify-write side; the sync verb tree-reads instead (read_path_at)"
    )
)]
pub(crate) fn read_boundaries(root: &Path, slice: u32) -> anyhow::Result<Boundaries> {
    load(&dispatch_dir(root, slice).join("boundaries.toml"))
}

/// Read `orthogonal.toml` for `slice` (empty when absent).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "funnel-time read-modify-write side; the sync verb tree-reads instead (read_path_at)"
    )
)]
pub(crate) fn read_orthogonal(root: &Path, slice: u32) -> anyhow::Result<Orthogonal> {
    load(&dispatch_dir(root, slice).join("orthogonal.toml"))
}

/// Append a per-phase code boundary to `boundaries.toml` (EX-5). Read-modify-
/// write — the dir/file are created on first write.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "funnel-time recording is the first non-test writer (PHASE-06)"
    )
)]
pub(crate) fn record_boundary(root: &Path, slice: u32, row: BoundaryRow) -> anyhow::Result<()> {
    let path = dispatch_dir(root, slice).join("boundaries.toml");
    let mut manifest: Boundaries = load(&path)?;
    manifest.rows.push(row);
    store(&path, &manifest)
}

/// Append an orthogonal-projection mark to `orthogonal.toml` (EX-5).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "funnel-time recording is the first non-test writer (PHASE-06)"
    )
)]
pub(crate) fn record_orthogonal(
    root: &Path,
    slice: u32,
    mark: OrthogonalMark,
) -> anyhow::Result<()> {
    let path = dispatch_dir(root, slice).join("orthogonal.toml");
    let mut manifest: Orthogonal = load(&path)?;
    manifest.rows.push(mark);
    store(&path, &manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- VT-4: round-trip incl. field-name + status-token pinning ----------

    #[test]
    fn journal_round_trips_and_pins_field_names() {
        let journal = Journal {
            rows: vec![JournalRow {
                source_oid: "aaa".into(),
                target_ref: "refs/review/64".into(),
                expected_old_oid: "0".repeat(40),
                planned_new_oid: "bbb".into(),
                applied_new_oid: String::new(),
                status: LedgerStatus::Pending,
            }],
        };
        let text = journal.to_toml().expect("serialize");
        // Pin the on-disk vocab the downstream stages key on.
        assert!(text.contains("[[row]]"), "table header: {text}");
        assert!(text.contains("source_oid ="), "{text}");
        assert!(text.contains("target_ref ="), "{text}");
        assert!(text.contains("expected_old_oid ="), "{text}");
        assert!(text.contains("planned_new_oid ="), "{text}");
        assert!(text.contains("applied_new_oid ="), "{text}");
        assert!(
            text.contains("status = \"pending\""),
            "lowercase token: {text}"
        );
        assert_eq!(Journal::parse(&text).expect("parse"), journal);
    }

    #[test]
    fn boundaries_round_trip_and_orthogonal_round_trip() {
        let boundaries = Boundaries {
            rows: vec![BoundaryRow {
                phase: "PHASE-03".into(),
                code_start_oid: "s".into(),
                code_end_oid: "e".into(),
            }],
        };
        let text = boundaries.to_toml().expect("ser");
        assert!(text.contains("[[boundary]]"), "{text}");
        assert!(text.contains("phase = \"PHASE-03\""), "{text}");
        assert_eq!(Boundaries::parse(&text).expect("parse"), boundaries);

        let orthogonal = Orthogonal {
            rows: vec![OrthogonalMark {
                entity: "ADR-012".into(),
                path: ".doctrine/adr/012".into(),
                status: LedgerStatus::Verified,
            }],
        };
        let text = orthogonal.to_toml().expect("ser");
        assert!(text.contains("[[mark]]"), "{text}");
        assert!(text.contains("status = \"verified\""), "{text}");
        assert_eq!(Orthogonal::parse(&text).expect("parse"), orthogonal);
    }

    #[test]
    fn empty_manifests_round_trip() {
        for text in [
            Journal::default().to_toml().unwrap(),
            Boundaries::default().to_toml().unwrap(),
            Orthogonal::default().to_toml().unwrap(),
        ] {
            // An empty manifest serializes to (effectively) nothing and parses back.
            assert!(Journal::parse(&text).is_ok());
        }
        assert_eq!(Journal::parse("").unwrap(), Journal::default());
        assert_eq!(Boundaries::parse("").unwrap(), Boundaries::default());
        assert_eq!(Orthogonal::parse("").unwrap(), Orthogonal::default());
    }

    // --- VT-5: recording surface writes rows prepare-review reads back ------

    #[test]
    fn record_then_read_round_trips_through_disk() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let slice = 64;

        // Absent-file path: reads default to empty before anything is recorded.
        assert_eq!(read_boundaries(root, slice).unwrap(), Boundaries::default());
        assert_eq!(read_orthogonal(root, slice).unwrap(), Orthogonal::default());

        record_boundary(
            root,
            slice,
            BoundaryRow {
                phase: "PHASE-01".into(),
                code_start_oid: "s1".into(),
                code_end_oid: "e1".into(),
            },
        )
        .expect("record boundary 1");
        record_boundary(
            root,
            slice,
            BoundaryRow {
                phase: "PHASE-02".into(),
                code_start_oid: "s2".into(),
                code_end_oid: "e2".into(),
            },
        )
        .expect("record boundary 2");
        record_orthogonal(
            root,
            slice,
            OrthogonalMark {
                entity: "ADR-012".into(),
                path: ".doctrine/adr/012".into(),
                status: LedgerStatus::Verified,
            },
        )
        .expect("record mark");

        // The recording surface created the dir at the design path.
        assert!(root.join(".doctrine/dispatch/64/boundaries.toml").exists());

        // prepare-review's read-back contract: appended rows, in order.
        let boundaries = read_boundaries(root, slice).unwrap();
        let phases: Vec<&str> = boundaries.rows.iter().map(|r| r.phase.as_str()).collect();
        assert_eq!(phases, vec!["PHASE-01", "PHASE-02"]);

        let orthogonal = read_orthogonal(root, slice).unwrap();
        assert_eq!(orthogonal.rows.len(), 1);
        assert_eq!(orthogonal.rows[0].status, LedgerStatus::Verified);
        // The untouched journal manifest is still an absent-file empty default.
        assert_eq!(read_journal(root, slice).unwrap(), Journal::default());
    }
}
