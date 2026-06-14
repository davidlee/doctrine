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

// --- candidate ledger (SL-068 PHASE-01, design §5.3) -------------------------

/// A candidate's flavour: an `audit` review surface vs an `experiment` scratch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateKind {
    /// A review/audit surface candidate.
    Audit,
    /// An exploratory experiment candidate.
    Experiment,
}

/// What role a candidate plays in the funnel: a review surface, the close
/// target close will land, or a throwaway scratch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateRole {
    /// The surface an adversarial review reads.
    ReviewSurface,
    /// The immutable target close lands onto.
    CloseTarget,
    /// A throwaway exploration.
    Scratch,
}

/// What a candidate's merge carries: the full impl bundle vs raw code only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidatePayload {
    /// The full impl bundle (code + knowledge).
    ImplBundle,
    /// Code only.
    Code,
}

/// Lifecycle status of a candidate row — the ONLY mutable field on a recorded
/// row (EX-3). Supersession is explicit history (`supersedes` + a fresh row),
/// never an in-place OID rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateStatus {
    /// The candidate ref + merge commit were created.
    Created,
    /// The Doctrine-created merge hit a conflict.
    Conflicted,
    /// The candidate was abandoned.
    Abandoned,
    /// The candidate was superseded by a fresher one.
    Superseded,
}

/// `candidates.toml` — the candidate ledger (design §5.3). Carries the
/// recorded candidate rows plus the current role-keyed admission record.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct Candidates {
    #[serde(default, rename = "candidate")]
    pub rows: Vec<CandidateRow>,
    #[serde(default)]
    pub current_admission: CurrentAdmission,
}

/// One candidate row. Every field but `status` is immutable once recorded
/// (EX-3): supersession appends a fresh row, never an in-place OID rewrite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CandidateRow {
    /// The candidate id (e.g. `cand-068-review-001`).
    pub id: String,
    /// The human-facing label (e.g. `review-001`).
    pub label: String,
    /// Audit vs experiment.
    pub kind: CandidateKind,
    /// Review surface / close target / scratch.
    pub role: CandidateRole,
    /// Impl bundle vs code-only.
    pub payload: CandidatePayload,
    /// The ref this candidate is published at.
    pub target_ref: String,
    /// The source ref the candidate was built from.
    pub source_ref: String,
    /// The source ref's oid at build time.
    pub source_oid: String,
    /// The base ref the merge was computed against.
    pub base_ref: String,
    /// The base ref's oid at build time.
    pub base_oid: String,
    /// The Doctrine-created no-ff merge commit.
    pub merge_oid: String,
    /// Lifecycle status — the only mutable field (EX-3).
    pub status: CandidateStatus,
    /// Optional candidate id this row supersedes.
    #[serde(default)]
    pub supersedes: String,
    /// Free-text reason (e.g. for abandonment).
    #[serde(default)]
    pub reason: String,
    /// The verb that created this row.
    pub created_by: String,
    /// Creation timestamp.
    pub created_at: String,
}

/// The current admission record, keyed by role. The design shows only
/// `close_target`; the lifecycle admits a `review_surface` too — both modelled,
/// each optional and skipped on serialize when empty.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct CurrentAdmission {
    /// The admitted close-target candidate, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_target: Option<Admission>,
    /// The admitted review-surface candidate, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_surface: Option<Admission>,
}

/// One role's admission: pins the candidate ref + the immutable oid the
/// downstream verb (close / review) targets. Re-admission appends a fresh
/// record (with `supersedes`), never rewrites a prior admission's oids (EX-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Admission {
    /// The admitted candidate id.
    pub candidate_id: String,
    /// The candidate ref at admission.
    pub candidate_ref: String,
    /// The candidate ref's oid observed at admission.
    pub expected_ref_oid: String,
    /// The immutable oid the downstream verb targets.
    pub admitted_oid: String,
    /// The governing review (e.g. `RV-007`).
    pub review: String,
    /// Optional prior admission id this supersedes.
    #[serde(default)]
    pub supersedes: String,
    /// Admission timestamp.
    pub admitted_at: String,
}

impl Candidates {
    /// Parse a `candidates.toml` body. An absent file is the caller's concern
    /// ([`read_candidates`]); this parses a present body.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "candidate create/status/admit are the first non-test callers (PHASE-02+)"
        )
    )]
    pub(crate) fn parse(text: &str) -> anyhow::Result<Candidates> {
        Ok(toml::from_str(text)?)
    }

    /// Serialize to a `candidates.toml` body (serde-escaped — no raw splicing).
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "candidate create/status/admit are the first non-test callers (PHASE-02+)"
        )
    )]
    pub(crate) fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string(self)?)
    }

    /// Transition a recorded row's `status` — the ONLY mutable field (EX-3).
    /// No identity/OID setter exists; supersession is a fresh row, not an
    /// in-place rewrite. Returns `true` when a matching row was updated.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "candidate create/status/admit are the first non-test callers (PHASE-02+)"
        )
    )]
    pub(crate) fn set_candidate_status(&mut self, id: &str, status: CandidateStatus) -> bool {
        match self.rows.iter_mut().find(|r| r.id == id) {
            Some(row) => {
                row.status = status;
                true
            }
            None => false,
        }
    }
}

// Some symbols below are test-live but have no *non-test* caller yet: the
// round-trip `parse`/`to_toml` surface, the filesystem `read_*` (the sync verb
// tree-reads via `read_path_at` instead), and `record_orthogonal` (its driver is
// the deferred OQ-B classifier). `record_boundary`/`store` ARE now live — wired to
// `dispatch record-boundary` (PHASE-06). Each still-dead symbol carries a
// per-symbol `cfg_attr(not(test))` expect so
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
/// `<slice>` is the canonical 3-digit zero-padded form (`064`) — the SAME path
/// the `dispatch sync` reader tree-reads (`dispatch.rs`) and the `dispatch/064`
/// branch name; an unpadded dir here would make the funnel writer and the sync
/// reader disagree.
fn dispatch_dir(root: &Path, slice: u32) -> PathBuf {
    root.join(".doctrine")
        .join("dispatch")
        .join(format!("{slice:03}"))
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
/// write — the dir/file are created on first write. Wired to the
/// `dispatch record-boundary` funnel verb (PHASE-06).
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
        reason = "no funnel verb yet — its driver is the OQ-B orthogonal classifier (deferred plan-gate); empty orthogonal.toml is the conservative EXCLUDE fallback (IMP backlog)"
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

/// Read `candidates.toml` for `slice` (empty when absent — VT-2). The
/// create/status/admit verbs (PHASE-02+) are the first non-test callers.
pub(crate) fn read_candidates(root: &Path, slice: u32) -> anyhow::Result<Candidates> {
    load(&dispatch_dir(root, slice).join("candidates.toml"))
}

/// Write the whole candidate ledger for `slice` to `candidates.toml`
/// (read-modify-write at the create/supersede/status verbs). The dir/file are
/// created on first write. Pairs with [`read_candidates`]; serde escapes all
/// free-text, so no value is hand-spliced into the TOML.
pub(crate) fn write_candidates(
    root: &Path,
    slice: u32,
    candidates: &Candidates,
) -> anyhow::Result<()> {
    store(
        &dispatch_dir(root, slice).join("candidates.toml"),
        candidates,
    )
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

        // The recording surface created the dir at the canonical padded path
        // (the same `<slice>` form the sync reader and `dispatch/064` use).
        assert!(root.join(".doctrine/dispatch/064/boundaries.toml").exists());

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

    // --- candidate ledger (SL-068 PHASE-01) --------------------------------

    fn sample_candidate(id: &str, label: &str, status: CandidateStatus) -> CandidateRow {
        CandidateRow {
            id: id.into(),
            label: label.into(),
            kind: CandidateKind::Audit,
            role: CandidateRole::ReviewSurface,
            payload: CandidatePayload::ImplBundle,
            target_ref: format!("refs/heads/candidate/068/{label}"),
            source_ref: "refs/heads/review/068".into(),
            source_oid: "src-oid".into(),
            base_ref: "refs/heads/main".into(),
            base_oid: "base-oid".into(),
            merge_oid: "merge-oid".into(),
            status,
            supersedes: String::new(),
            reason: String::new(),
            created_by: "dispatch candidate create".into(),
            created_at: "2026-06-15".into(),
        }
    }

    // VT-1: round-trip + on-disk vocab pinning.
    #[test]
    fn candidates_round_trip_and_pin_field_names() {
        let manifest = Candidates {
            rows: vec![
                sample_candidate(
                    "cand-068-review-001",
                    "review-001",
                    CandidateStatus::Created,
                ),
                sample_candidate(
                    "cand-068-review-002",
                    "review-002",
                    CandidateStatus::Conflicted,
                ),
            ],
            current_admission: CurrentAdmission {
                close_target: Some(Admission {
                    candidate_id: "cand-068-close-001".into(),
                    candidate_ref: "refs/heads/candidate/068/close-001".into(),
                    expected_ref_oid: "ref-oid".into(),
                    admitted_oid: "admitted-oid".into(),
                    review: "RV-007".into(),
                    supersedes: String::new(),
                    admitted_at: "2026-06-15".into(),
                }),
                review_surface: None,
            },
        };
        let text = manifest.to_toml().expect("serialize");
        assert!(text.contains("[[candidate]]"), "table header: {text}");
        assert!(text.contains("id ="), "{text}");
        assert!(text.contains("label ="), "{text}");
        assert!(text.contains("target_ref ="), "{text}");
        assert!(text.contains("source_oid ="), "{text}");
        assert!(text.contains("base_oid ="), "{text}");
        assert!(text.contains("merge_oid ="), "{text}");
        assert!(text.contains("created_by ="), "{text}");
        assert!(text.contains("created_at ="), "{text}");
        assert!(text.contains("status = \"created\""), "{text}");
        assert!(text.contains("role = \"review_surface\""), "{text}");
        assert!(text.contains("kind = \"audit\""), "{text}");
        assert!(text.contains("payload = \"impl_bundle\""), "{text}");
        assert!(
            text.contains("[current_admission.close_target]"),
            "admission table: {text}"
        );
        assert_eq!(Candidates::parse(&text).expect("parse"), manifest);
    }

    // VT-2: absent / empty defaults, incl. read_candidates absent-file.
    #[test]
    fn candidates_empty_and_absent_default() {
        assert_eq!(Candidates::parse("").unwrap(), Candidates::default());
        let text = Candidates::default().to_toml().unwrap();
        assert_eq!(Candidates::parse(&text).unwrap(), Candidates::default());

        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            read_candidates(dir.path(), 68).unwrap(),
            Candidates::default()
        );
    }

    // VT-3: an unknown enum token fails to parse, never round-trips.
    #[test]
    fn candidates_reject_unknown_tokens() {
        let bad_role = r#"
[[candidate]]
id = "x"
label = "x"
kind = "audit"
role = "bogus"
payload = "impl_bundle"
target_ref = "r"
source_ref = "r"
source_oid = "o"
base_ref = "r"
base_oid = "o"
merge_oid = "o"
status = "created"
created_by = "v"
created_at = "d"
"#;
        assert!(Candidates::parse(bad_role).is_err(), "bogus role must fail");

        let bad_kind = bad_role
            .replace("role = \"bogus\"", "role = \"review_surface\"")
            .replace("kind = \"audit\"", "kind = \"bogus\"");
        assert!(
            Candidates::parse(&bad_kind).is_err(),
            "bogus kind must fail"
        );

        let bad_payload = bad_role
            .replace("role = \"bogus\"", "role = \"review_surface\"")
            .replace("payload = \"impl_bundle\"", "payload = \"bogus\"");
        assert!(
            Candidates::parse(&bad_payload).is_err(),
            "bogus payload must fail"
        );
    }

    // EX-3: set_candidate_status mutates only status; identity/OID untouched.
    #[test]
    fn set_candidate_status_mutates_only_status() {
        let mut manifest = Candidates {
            rows: vec![sample_candidate(
                "cand-068-review-001",
                "review-001",
                CandidateStatus::Created,
            )],
            current_admission: CurrentAdmission::default(),
        };
        let before = manifest.rows[0].clone();

        assert!(manifest.set_candidate_status("cand-068-review-001", CandidateStatus::Abandoned));
        assert!(!manifest.set_candidate_status("nope", CandidateStatus::Abandoned));

        let after = &manifest.rows[0];
        assert_eq!(after.status, CandidateStatus::Abandoned);
        // Every identity/OID field is byte-identical to before the transition.
        assert_eq!(after.id, before.id);
        assert_eq!(after.label, before.label);
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.role, before.role);
        assert_eq!(after.payload, before.payload);
        assert_eq!(after.target_ref, before.target_ref);
        assert_eq!(after.source_ref, before.source_ref);
        assert_eq!(after.source_oid, before.source_oid);
        assert_eq!(after.base_ref, before.base_ref);
        assert_eq!(after.base_oid, before.base_oid);
        assert_eq!(after.merge_oid, before.merge_oid);
        assert_eq!(after.created_by, before.created_by);
        assert_eq!(after.created_at, before.created_at);
    }
}
