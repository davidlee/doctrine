// SPDX-License-Identifier: GPL-3.0-only
//! `coverage_store` — the observed-tier write path (SL-057 PHASE-03).
//!
//! The IMPURE shell over the slice-side `coverage.toml`: it reads and writes the
//! store on disk, stamps the git anchor, and funnels every coverage mutation
//! through one [`record`] / [`forget`] seam. The pure leaf
//! (`crate::coverage`) owns the types and folds; this module owns the disk + git
//! contact. The DATE is always INJECTED (a `today` param), NEVER read from
//! `crate::clock` here (design F-VI — no hidden clock); the CLI shell (PHASE-05)
//! reads the clock and passes `today` in.
//!
//! Path shape mirrors `crate::coverage_scan`: a slice's store lives at
//! `<root>/.doctrine/slice/{NNN}/coverage.toml`. Reuse, not parallel impl:
//! - reads parse through [`crate::coverage::parse`], absent-file ⇒ empty;
//! - writes render through [`crate::coverage::render`] then land via
//!   [`crate::fsutil::write_atomic`] (atomic, overwrite-safe — no torn write);
//! - the within-file no-clobber fold is [`crate::coverage::upsert`].

// This whole write-path shell is a leaf built ahead of its consumer: PHASE-03
// lands `load`/`save`/`record`/`forget`; the CLI surface that drives them is the
// dependent PHASE-05. Until then every item here is dead in the bins/lib build, so
// the module carries a self-clearing `not(test)` dead_code expect (the `verify.rs`
// / `coverage.rs` leaf-ahead-of-consumer precedent). It scopes to `not(test)`
// because the VTs below exercise every item under `cfg(test)`, where `dead_code`
// would not fire; the gate runs plain `cargo clippy` (bins/lib, no test cfg) where
// the items are genuinely dead. The expectation is fulfilled exactly where the lint
// applies, and retires itself the moment PHASE-05 wires the CLI.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "coverage_store write-path shell (SL-057 PHASE-03) is built ahead \
                  of its PHASE-05 CLI consumer"
    )
)]

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::coverage::{self, CoverageEntry, CoverageFile, CoverageKey, VtCheck};
use crate::fsutil;
use crate::git;
use crate::requirement::CoverageStatus;
use crate::verify::{self, VerificationConfig};

/// Repo-relative slice tree — the dir under which each slice's `coverage.toml`
/// lives. Mirrors `coverage_scan::SLICE_DIR` / `state::SLICE_DIR`; kept local so
/// this shell owns its one path (the established per-module convention).
const SLICE_DIR: &str = ".doctrine/slice";

/// The on-disk path of slice `slice_id`'s coverage store:
/// `<root>/.doctrine/slice/{NNN}/coverage.toml`. Same shape as
/// [`crate::coverage_scan::slice_local_covered_reqs`] reads.
fn coverage_path(root: &Path, slice_id: u32) -> PathBuf {
    root.join(SLICE_DIR)
        .join(format!("{slice_id:03}"))
        .join("coverage.toml")
}

/// Load slice `slice_id`'s [`CoverageFile`] from disk. An ABSENT file
/// (`NotFound`) is the empty store (`Ok(CoverageFile::default())`) — a slice may
/// have recorded nothing yet; any other read error or a parse error propagates.
pub(crate) fn load(root: &Path, slice_id: u32) -> Result<CoverageFile> {
    let path = coverage_path(root, slice_id);
    match fs::read_to_string(&path) {
        Ok(body) => coverage::parse(&body)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(CoverageFile::default()),
        Err(e) => Err(anyhow::anyhow!("failed to read {}: {e}", path.display())),
    }
}

/// Persist `file` as slice `slice_id`'s `coverage.toml`. Ensures the slice dir
/// exists, renders through [`coverage::render`], and lands the body ATOMICALLY
/// via [`fsutil::write_atomic`] (temp-then-rename, overwrite-safe — a concurrent
/// reader sees the old or the new file, never a torn one).
pub(crate) fn save(root: &Path, slice_id: u32, file: &CoverageFile) -> Result<()> {
    let path = coverage_path(root, slice_id);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .map_err(|e| anyhow::anyhow!("failed to create {}: {e}", dir.display()))?;
    }
    let body = coverage::render(file)?;
    fsutil::write_atomic(&path, body.as_bytes())
}

/// The inputs `record` needs that are independent of the date/git stamping it
/// performs. `check` present ⇒ this is a `VT` record (the recipe to verify
/// later); absent ⇒ a `VA`/`VH` attestation, whose `status` the caller supplies.
pub(crate) struct RecordInput {
    /// The 4-tuple identity this evidence is recorded under.
    pub(crate) key: CoverageKey,
    /// The observed status to land. For a `VT` record this is OVERRIDDEN to
    /// [`CoverageStatus::Planned`] (the check has not run yet); for a `VA`/`VH`
    /// attestation it is taken verbatim.
    pub(crate) status: CoverageStatus,
    /// The `VT`-check recipe — `Some` ⇒ `VT` (validated + resolved before any
    /// write); `None` ⇒ a `VA`/`VH` attestation (no check).
    pub(crate) check: Option<VtCheck>,
    /// The repo-relative path set this evidence stands on (staleness input).
    pub(crate) touched_paths: Vec<String>,
}

/// Record one coverage cell into slice `slice_id`'s store (IMPURE: disk + git).
///
/// Contract (design §5.3/§5.4, plan EX-2):
/// - **Fail-fast (F-1).** When a `check` is present (`VT`), BOTH
///   [`coverage::valid`] AND [`verify::resolve`] run BEFORE any write; a failure
///   of either BLOCKS the write — the on-disk file is left UNCHANGED.
/// - **`VT`** (`check.is_some()`) ⇒ status leans [`CoverageStatus::Planned`],
///   `attested_date = None`, `check = Some(..)`.
/// - **`VA`/`VH`** (`check.is_none()`) ⇒ the caller's `status` is kept,
///   `attested_date = Some(attested_override.unwrap_or(today))`, no check.
/// - `git_anchor = git::head_sha(root).unwrap_or_default()`.
/// - Loads → [`coverage::upsert`]s the built entry → [`save`]s.
///
/// The DATE is the `today` PARAM, NEVER `crate::clock::today()` here (F-VI).
pub(crate) fn record(
    root: &Path,
    slice_id: u32,
    input: RecordInput,
    cfg: &VerificationConfig,
    today: &str,
    attested_override: Option<&str>,
) -> Result<()> {
    let RecordInput {
        key,
        status,
        check,
        touched_paths,
    } = input;

    // Fail-fast (F-1): for a VT record, validate the check's shape AND resolve it
    // against the project config BEFORE touching disk. Either failure blocks the
    // write — the store stays byte-identical.
    if let Some(check) = &check {
        coverage::valid(check).map_err(|e| anyhow::anyhow!("invalid VT-check: {e:?}"))?;
        verify::resolve(cfg, check).map_err(|e| anyhow::anyhow!("unresolvable VT-check: {e:?}"))?;
    }

    let is_vt = check.is_some();
    // VT evidence has not run yet ⇒ Planned, no attestation date. VA/VH carry the
    // supplied status and the injected (or overridden) attestation date.
    let (status, attested_date) = if is_vt {
        (CoverageStatus::Planned, None)
    } else {
        let date = attested_override.unwrap_or(today).to_owned();
        (status, Some(date))
    };

    let entry = CoverageEntry {
        key,
        status,
        git_anchor: git::head_sha(root).unwrap_or_default(),
        attested_date,
        touched_paths,
        check,
    };

    let mut file = load(root, slice_id)?;
    coverage::upsert(&mut file, entry);
    save(root, slice_id, &file)
}

/// Erase the coverage cell keyed by `key` from slice `slice_id`'s store (IMPURE).
/// Returns `Some((key, erased_status))` of the removed cell, or `None` if no such
/// cell exists. A deletion that flips a composite green must never be silent — the
/// caller pairs this with [`withdrawal_line`] to name what it erased (F-IV).
pub(crate) fn forget(
    root: &Path,
    slice_id: u32,
    key: &CoverageKey,
) -> Result<Option<(CoverageKey, CoverageStatus)>> {
    let mut file = load(root, slice_id)?;
    let Some(pos) = file.entry.iter().position(|e| &e.key == key) else {
        return Ok(None);
    };
    let removed = file.entry.remove(pos);
    save(root, slice_id, &file)?;
    Ok(Some((removed.key, removed.status)))
}

/// The terse withdrawal line naming the erased 4-tuple + its status (PURE):
/// `withdrew <slice>/<requirement>/<change>/<mode> [<Status>]`,
/// e.g. `withdrew SL-057/REQ-256/SL-057/VT [Failed]`. Single `format!` (no
/// push/format-in-loop assembly).
pub(crate) fn withdrawal_line(key: &CoverageKey, status: CoverageStatus) -> String {
    format!(
        "withdrew {}/{}/{}/{} [{status:?}]",
        key.slice, key.requirement, key.contributing_change, key.mode,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on disk round-trip / parse is idiomatic"
)]
mod tests {
    use super::*;
    use crate::coverage::{MatchSource, Matcher};

    fn key(slice: &str, req: &str, change: &str, mode: &str) -> CoverageKey {
        CoverageKey {
            slice: slice.to_owned(),
            requirement: req.to_owned(),
            contributing_change: change.to_owned(),
            mode: mode.to_owned(),
        }
    }

    /// A well-formed VT check: an `alias` the supplied config resolves, plus a
    /// non-empty stdout matcher (so `valid` + `resolve` both pass).
    fn good_vtcheck() -> VtCheck {
        VtCheck {
            alias: Some("unit".to_owned()),
            command: None,
            extra_args: Vec::new(),
            matcher: Some(Matcher {
                source: Some(MatchSource::Stdout),
                pattern: "ok".to_owned(),
                regex: false,
            }),
        }
    }

    /// A config whose `unit` alias resolves (so a `good_vtcheck` is runnable).
    fn cfg_with_unit() -> VerificationConfig {
        crate::dtoml::parse("[verification.aliases]\nunit = [\"cargo\", \"test\"]\n")
            .unwrap()
            .verification
    }

    fn input(key: CoverageKey, status: CoverageStatus, check: Option<VtCheck>) -> RecordInput {
        RecordInput {
            key,
            status,
            check,
            touched_paths: Vec::new(),
        }
    }

    // --- VT-1: store load/save round-trip + atomic overwrite + NF-001 ---------

    #[test]
    fn load_absent_file_is_empty_store() {
        let tmp = tempfile::tempdir().unwrap();
        let file = load(tmp.path(), 57).unwrap();
        assert_eq!(file, CoverageFile::default(), "absent ⇒ empty store");
    }

    #[test]
    fn save_then_load_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let mut file = CoverageFile::default();
        coverage::upsert(
            &mut file,
            CoverageEntry {
                key: key("SL-057", "REQ-200", "SL-057", "VH"),
                status: CoverageStatus::Verified,
                git_anchor: "anchor-abc".to_owned(),
                attested_date: Some("2026-06-14".to_owned()),
                touched_paths: vec!["src/x.rs".to_owned()],
                check: None,
            },
        );
        save(tmp.path(), 57, &file).unwrap();
        assert_eq!(
            load(tmp.path(), 57).unwrap(),
            file,
            "save → load round-trips"
        );
    }

    #[test]
    fn save_overwrites_atomically_leaving_no_temp() {
        let tmp = tempfile::tempdir().unwrap();

        let mut first = CoverageFile::default();
        coverage::upsert(
            &mut first,
            CoverageEntry {
                key: key("SL-057", "REQ-200", "SL-057", "VT"),
                status: CoverageStatus::Planned,
                git_anchor: String::new(),
                attested_date: None,
                touched_paths: Vec::new(),
                check: None,
            },
        );
        save(tmp.path(), 57, &first).unwrap();

        // A second save over the existing file: same key, latest payload wins.
        let mut second = first.clone();
        coverage::upsert(
            &mut second,
            CoverageEntry {
                key: key("SL-057", "REQ-200", "SL-057", "VT"),
                status: CoverageStatus::Verified,
                git_anchor: String::new(),
                attested_date: None,
                touched_paths: Vec::new(),
                check: None,
            },
        );
        save(tmp.path(), 57, &second).unwrap();

        assert_eq!(load(tmp.path(), 57).unwrap(), second, "overwrite landed");

        // No temp file (.coverage.toml.<pid>.tmp) survives the atomic rename.
        let dir = tmp.path().join(SLICE_DIR).join("057");
        let strays: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n != "coverage.toml")
            .collect();
        assert!(strays.is_empty(), "no temp left behind, found: {strays:?}");
    }

    #[test]
    fn record_touches_only_coverage_toml_not_a_sibling_entity_file() {
        // NF-001: a record into a slice tree that also holds a sibling requirement
        // entity file must change ONLY coverage.toml — the entity file is untouched.
        let tmp = tempfile::tempdir().unwrap();
        let slice_dir = tmp.path().join(SLICE_DIR).join("057");
        fs::create_dir_all(&slice_dir).unwrap();
        let sibling = slice_dir.join("slice-057.toml");
        let sibling_body = "id = 57\ntitle = \"x\"\n";
        fs::write(&sibling, sibling_body).unwrap();

        record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VH"),
                CoverageStatus::Verified,
                None,
            ),
            &VerificationConfig::default(),
            "2026-06-14",
            None,
        )
        .unwrap();

        assert!(slice_dir.join("coverage.toml").exists(), "coverage written");
        assert_eq!(
            fs::read_to_string(&sibling).unwrap(),
            sibling_body,
            "the sibling entity file is byte-identical"
        );
    }

    // --- VT-2: record happy paths + fail-fast blocks the write ----------------

    #[test]
    fn vt_record_leans_planned_no_date_and_persists_the_check() {
        let tmp = tempfile::tempdir().unwrap();
        record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                // A caller-supplied status that must be OVERRIDDEN to Planned.
                CoverageStatus::Verified,
                Some(good_vtcheck()),
            ),
            &cfg_with_unit(),
            "2026-06-14",
            None,
        )
        .unwrap();

        let file = load(tmp.path(), 57).unwrap();
        let entry = file.entry.first().unwrap();
        assert_eq!(entry.status, CoverageStatus::Planned, "VT leans Planned");
        assert!(entry.attested_date.is_none(), "VT carries no attested_date");
        assert_eq!(
            entry.check.as_ref(),
            Some(&good_vtcheck()),
            "check persisted"
        );
    }

    #[test]
    fn va_record_stamps_the_injected_today() {
        let tmp = tempfile::tempdir().unwrap();
        record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VA"),
                CoverageStatus::Verified,
                None,
            ),
            &VerificationConfig::default(),
            "2026-06-14",
            None,
        )
        .unwrap();
        let file = load(tmp.path(), 57).unwrap();
        let entry = file.entry.first().unwrap();
        assert_eq!(
            entry.status,
            CoverageStatus::Verified,
            "VA keeps the status"
        );
        assert_eq!(entry.attested_date.as_deref(), Some("2026-06-14"));
        assert!(entry.check.is_none(), "VA carries no check");
    }

    #[test]
    fn attested_override_is_honoured_over_today() {
        let tmp = tempfile::tempdir().unwrap();
        record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VH"),
                CoverageStatus::Verified,
                None,
            ),
            &VerificationConfig::default(),
            "2026-06-14",
            Some("2020-01-01"),
        )
        .unwrap();
        let entry = load(tmp.path(), 57)
            .unwrap()
            .entry
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(
            entry.attested_date.as_deref(),
            Some("2020-01-01"),
            "the override wins over today"
        );
    }

    #[test]
    fn valid_failure_blocks_the_write() {
        let tmp = tempfile::tempdir().unwrap();
        // An alias with an EMPTY matcher fails `coverage::valid` (MatcherRequired).
        let bad = VtCheck {
            alias: Some("unit".to_owned()),
            command: None,
            extra_args: Vec::new(),
            matcher: None,
        };
        let err = record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(bad),
            ),
            &cfg_with_unit(),
            "2026-06-14",
            None,
        );
        assert!(err.is_err(), "a valid() failure blocks the write");
        assert!(
            !coverage_path(tmp.path(), 57).exists(),
            "no file written — store unchanged"
        );
    }

    #[test]
    fn resolve_failure_blocks_the_write() {
        let tmp = tempfile::tempdir().unwrap();
        // A well-formed check naming an alias the config does NOT define: `valid`
        // passes, `resolve` fails (UnknownAlias) — the write is still blocked.
        let err = record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(good_vtcheck()),
            ),
            &VerificationConfig::default(), // no `unit` alias
            "2026-06-14",
            None,
        );
        assert!(err.is_err(), "a resolve() failure blocks the write");
        assert!(
            !coverage_path(tmp.path(), 57).exists(),
            "no file written — store unchanged"
        );
    }

    #[test]
    fn record_does_not_overwrite_an_existing_store_on_blocked_write() {
        // Stronger: an existing store stays byte-identical when a later record is
        // blocked at the fail-fast gate.
        let tmp = tempfile::tempdir().unwrap();
        record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VA"),
                CoverageStatus::Verified,
                None,
            ),
            &VerificationConfig::default(),
            "2026-06-14",
            None,
        )
        .unwrap();
        let before = fs::read_to_string(coverage_path(tmp.path(), 57)).unwrap();

        let _blocked = record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-201", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(good_vtcheck()),
            ),
            &VerificationConfig::default(), // resolve fails
            "2026-06-14",
            None,
        );
        let after = fs::read_to_string(coverage_path(tmp.path(), 57)).unwrap();
        assert_eq!(before, after, "blocked write left the store byte-identical");
    }

    // --- VT-3: F-VI — the date is a param, never a hidden clock ----------------

    #[test]
    fn injected_sentinel_date_lands_in_attested_date() {
        let tmp = tempfile::tempdir().unwrap();
        record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-200", "SL-057", "VA"),
                CoverageStatus::Verified,
                None,
            ),
            &VerificationConfig::default(),
            "2099-01-01", // a sentinel no clock would ever return
            None,
        )
        .unwrap();
        let entry = load(tmp.path(), 57)
            .unwrap()
            .entry
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(
            entry.attested_date.as_deref(),
            Some("2099-01-01"),
            "the injected date is what lands — record reads no clock (F-VI)"
        );
    }

    // --- VT-4: forget + withdrawal_line ---------------------------------------

    #[test]
    fn forget_removes_the_keyed_cell_and_returns_its_status() {
        let tmp = tempfile::tempdir().unwrap();
        record(
            tmp.path(),
            57,
            input(
                key("SL-057", "REQ-256", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(good_vtcheck()),
            ),
            &cfg_with_unit(),
            "2026-06-14",
            None,
        )
        .unwrap();

        let k = key("SL-057", "REQ-256", "SL-057", "VT");
        let erased = forget(tmp.path(), 57, &k).unwrap();
        assert_eq!(
            erased,
            Some((k.clone(), CoverageStatus::Planned)),
            "forget returns the erased key + status"
        );
        assert!(load(tmp.path(), 57).unwrap().entry.is_empty(), "cell gone");

        // A second forget of the same key finds nothing.
        assert_eq!(forget(tmp.path(), 57, &k).unwrap(), None, "idempotent miss");
    }

    #[test]
    fn withdrawal_line_names_key_and_erased_status() {
        let line = withdrawal_line(
            &key("SL-057", "REQ-256", "SL-057", "VT"),
            CoverageStatus::Failed,
        );
        assert_eq!(line, "withdrew SL-057/REQ-256/SL-057/VT [Failed]");
    }
}
