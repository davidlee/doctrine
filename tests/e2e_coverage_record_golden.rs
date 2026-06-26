//! SL-057 PHASE-05 — black-box CLI goldens for the `coverage` write path
//! (`record`/`verify`/`forget`) and the relocated `coverage show` (design VT-1/VT-2).
//!
//! These pin the observed-tier write surface over the BUILT binary on
//! non-git / fixed-input temp roots (deterministic):
//!   * `record` happy path — a `VT` cell lands at `planned` with its check persisted;
//!   * the validity REJECTs — non-zero exit + the exact error surface (empty-matcher
//!     on a shared base, an escaping `file:` glob, a bad `--regex` pattern, the
//!     both-base alias/command conflict);
//!   * `verify` — the Report print incl. the loud backfill line + the exit-code-only
//!     audit line;
//!   * `forget` — the withdrawal line on a hit, the not-found line on a miss;
//!   * the relocated `coverage show` happy path (behaviour preserved under the group).
//!
//! Determinism: the temp root is NOT a git repo, so `git_anchor` resolves empty; the
//! attestation date is injected via `--attested-date` so no clock is read. The reject
//! surfaces are config-free (`coverage::valid` / `verify::resolve`), so they are stable.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

// --- run / output helpers -------------------------------------------------

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Hand-seed one requirement's authored tree (so `coverage show` resolves it).
fn seed_requirement(root: &Path, id: u32, slug: &str, title: &str, status: &str, kind: &str) {
    let dir = root.join(format!(".doctrine/requirement/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("requirement-{id:03}.toml")),
        format!(
            "schema = \"doctrine.requirement\"\n\
             version = 1\n\
             id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             status = \"{status}\"\n\
             kind = \"{kind}\"\n"
        ),
    )
    .unwrap();
}

/// Read a slice's coverage.toml body back from disk.
fn coverage_body(root: &Path, slice_id: u32) -> String {
    fs::read_to_string(root.join(format!(".doctrine/slice/{slice_id:03}/coverage.toml")))
        .expect("coverage.toml exists")
}

// =========================================================================
// VT-1 (record): the happy path lands a VT cell at planned with its check.
// =========================================================================

/// `coverage record … --mode VT --command true` lands ONE `[[entry]]` keyed by the
/// canonical 4-tuple, status `planned` (the verifier derives it later), the literal
/// command check persisted, and an empty `git_anchor` (non-git root). Bare numeric
/// `--slice`/`--change` canonicalize to `SL-NNN` in the stored key.
#[test]
fn coverage_record_vt_lands_planned_with_check_byte_exact() {
    let dir = tmp();
    let root = dir.path();

    let out = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
            "--command",
            "true",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "recorded SL-057/REQ-001/SL-057/VT\n");

    // The stored cell: canonical key, planned, the persisted check, empty anchor.
    assert_eq!(
        coverage_body(root, 57),
        "[[entry]]\n\
         slice = \"SL-057\"\n\
         requirement = \"REQ-001\"\n\
         contributing_change = \"SL-057\"\n\
         mode = \"VT\"\n\
         status = \"planned\"\n\
         git_anchor = \"\"\n\
         \n\
         [entry.check]\n\
         command = [\"true\"]\n"
    );
}

/// A `VA`/`VH` attestation (no check fields) takes the supplied status and the
/// injected `--attested-date` (no clock read) — F-VI on the CLI seam.
#[test]
fn coverage_record_attestation_stamps_injected_date_byte_exact() {
    let dir = tmp();
    let root = dir.path();

    let out = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "SL-057",
            "--requirement",
            "REQ-001",
            "--change",
            "SL-057",
            "--mode",
            "VH",
            "--attested-date",
            "2020-01-01",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "recorded SL-057/REQ-001/SL-057/VH\n");
    assert_eq!(
        coverage_body(root, 57),
        "[[entry]]\n\
         slice = \"SL-057\"\n\
         requirement = \"REQ-001\"\n\
         contributing_change = \"SL-057\"\n\
         mode = \"VH\"\n\
         status = \"verified\"\n\
         git_anchor = \"\"\n\
         attested_date = \"2020-01-01\"\n"
    );
}

// =========================================================================
// VT-1 (record): each validity REJECT — non-zero exit + exact error surface.
// A blocked record leaves NO coverage.toml (fail-fast, store unchanged).
// =========================================================================

/// An alias check with NO matcher on a shared base is rejected (`MatcherRequired`):
/// the D3/A mandatory-matcher rule. The write is blocked — no file written.
#[test]
fn coverage_record_rejects_empty_matcher_on_shared_base() {
    let dir = tmp();
    let root = dir.path();
    let out = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
            "--alias",
            "unit",
        ],
    );
    assert!(!out.status.success());
    assert_eq!(stderr(&out), "Error: invalid VT-check: MatcherRequired\n");
    assert!(
        !root.join(".doctrine/slice/057/coverage.toml").exists(),
        "a blocked record writes nothing"
    );
}

/// A `file:` matcher glob that ascends out of the tree is rejected
/// (`GlobEscapesTree`) — the F-III confinement, statically caught.
#[test]
fn coverage_record_rejects_escaping_file_glob() {
    let dir = tmp();
    let root = dir.path();
    let out = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
            "--command",
            "true",
            "--matcher-source",
            "file:../x",
            "--matcher-pattern",
            "P",
        ],
    );
    assert!(!out.status.success());
    assert_eq!(stderr(&out), "Error: invalid VT-check: GlobEscapesTree\n");
}

/// A `--regex` matcher whose pattern does not parse is rejected (`BadRegex`).
#[test]
fn coverage_record_rejects_bad_regex() {
    let dir = tmp();
    let root = dir.path();
    let out = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
            "--command",
            "true",
            "--matcher-source",
            "stdout",
            "--matcher-pattern",
            "(",
            "--regex",
        ],
    );
    assert!(!out.status.success());
    assert_eq!(stderr(&out), "Error: invalid VT-check: BadRegex\n");
}

/// Setting BOTH `--alias` and `--command` is the "both base" conflict
/// (`AliasCommandConflict`) — mutually exclusive bases.
#[test]
fn coverage_record_rejects_both_alias_and_command() {
    let dir = tmp();
    let root = dir.path();
    let out = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
            "--alias",
            "unit",
            "--command",
            "true",
            "--matcher-source",
            "stdout",
            "--matcher-pattern",
            "ok",
        ],
    );
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: invalid VT-check: AliasCommandConflict\n"
    );
}

// =========================================================================
// VT-2 (verify): the Report print — transition line + backfill + audit lines.
// =========================================================================

/// `coverage verify <slice>` re-derives each VT entry and prints the Report: an
/// exit-code-only transition line, the loud backfill count, and the audit line.
/// Here one VT (literal `true`, no matcher) goes Planned→Verified, exit-code-only.
#[test]
fn coverage_verify_prints_transition_and_audit_lines() {
    let dir = tmp();
    let root = dir.path();
    // Seed a recordable VT cell, then re-derive it.
    let rec = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
            "--command",
            "true",
        ],
    );
    assert!(rec.status.success(), "stderr: {}", stderr(&rec));

    let out = run(root, &["coverage", "verify", "SL-057"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-057/REQ-001/SL-057/VT: Planned\u{2192}Verified [exit-code-only]\n\
         0 VT entries lack a check \u{2014} backfill\n\
         1 exit-code-only cells (no matcher) \u{2014} audit\n"
    );
}

/// A check-less VT entry (hand-seeded) is REPORTED in the backfill list and LEFT
/// untouched; the loud backfill count names it (the F-VII backfill loudness).
#[test]
fn coverage_verify_reports_checkless_vt_in_backfill() {
    let dir = tmp();
    let root = dir.path();
    let sdir = root.join(".doctrine/slice/057");
    fs::create_dir_all(&sdir).unwrap();
    fs::write(
        sdir.join("coverage.toml"),
        "[[entry]]\n\
         slice = \"SL-057\"\n\
         requirement = \"REQ-001\"\n\
         contributing_change = \"SL-057\"\n\
         mode = \"VT\"\n\
         status = \"in-progress\"\n\
         git_anchor = \"anchor-x\"\n",
    )
    .unwrap();

    let out = run(root, &["coverage", "verify", "SL-057"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-057/REQ-001/SL-057/VT: no check \u{2014} backfill\n\
         1 VT entries lack a check \u{2014} backfill\n"
    );
}

/// `coverage verify` requires exactly one of `<slice>` / `--all`.
#[test]
fn coverage_verify_requires_slice_xor_all() {
    let dir = tmp();
    let root = dir.path();

    let both = run(root, &["coverage", "verify", "SL-057", "--all"]);
    assert!(!both.status.success());
    assert_eq!(
        stderr(&both),
        "Error: pass a single <slice> OR --all, not both\n"
    );

    let neither = run(root, &["coverage", "verify"]);
    assert!(!neither.status.success());
    assert_eq!(stderr(&neither), "Error: pass a single <slice> or --all\n");
}

// =========================================================================
// VT-2 (forget): the withdrawal line + the not-found line.
// =========================================================================

/// `coverage forget <key>` on a hit prints the withdrawal line naming the erased
/// cell + its status; a second forget of the same key prints the not-found line.
#[test]
fn coverage_forget_prints_withdrawal_then_not_found() {
    let dir = tmp();
    let root = dir.path();
    let rec = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
            "--command",
            "true",
        ],
    );
    assert!(rec.status.success(), "stderr: {}", stderr(&rec));

    // Hit: the withdrawal line (status `Planned`, the leaned VT record status).
    let hit = run(
        root,
        &[
            "coverage",
            "forget",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
        ],
    );
    assert!(hit.status.success(), "stderr: {}", stderr(&hit));
    assert_eq!(
        stdout(&hit),
        "withdrew SL-057/REQ-001/SL-057/VT [Planned]\n"
    );

    // Miss: the terse not-found line (idempotent).
    let miss = run(
        root,
        &[
            "coverage",
            "forget",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
        ],
    );
    assert!(miss.status.success(), "stderr: {}", stderr(&miss));
    assert_eq!(stdout(&miss), "no coverage cell SL-057/REQ-001/SL-057/VT\n");
}

// =========================================================================
// VT-1 (show): the relocated read view under the group (behaviour preserved).
// =========================================================================

/// `coverage show REQ-NNN` is the former bare-`coverage <ref>` view, now under the
/// subcommand group — byte-identical output.
#[test]
fn coverage_show_relocated_view_byte_exact() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");

    let out = run(root, &["coverage", "show", "REQ-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      \u{2502} status  \u{2502} observed \u{2502} verdict\n\
         REQ-001 \u{2502} pending \u{2502} none     \u{2502} Coherent\n"
    );
}

// =========================================================================
// VT-1 (record): requirement-ref canonicalization — every id axis the read view
// canonicalizes is canonicalized at the write seam too (RV-017 F-1).
// =========================================================================

/// A non-canonical `--requirement REQ-1` keys the SAME canonical `REQ-001` cell
/// the read view resolves: the stored key is `REQ-001`, and a `forget` spelled
/// with the canonical `REQ-001` erases the cell the non-canonical `record` wrote.
#[test]
fn coverage_record_canonicalizes_requirement_ref() {
    let dir = tmp();
    let root = dir.path();

    let rec = run(
        root,
        &[
            "coverage",
            "record",
            "--slice",
            "57",
            "--requirement",
            "REQ-1",
            "--change",
            "57",
            "--mode",
            "VT",
            "--command",
            "true",
        ],
    );
    assert!(rec.status.success(), "stderr: {}", stderr(&rec));
    // Confirmation + stored key both carry the canonical `REQ-001`.
    assert_eq!(stdout(&rec), "recorded SL-057/REQ-001/SL-057/VT\n");
    assert!(
        coverage_body(root, 57).contains("requirement = \"REQ-001\""),
        "stored key must be canonical: {}",
        coverage_body(root, 57)
    );

    // A canonical-spelling forget erases the cell the non-canonical record wrote.
    let hit = run(
        root,
        &[
            "coverage",
            "forget",
            "--slice",
            "57",
            "--requirement",
            "REQ-001",
            "--change",
            "57",
            "--mode",
            "VT",
        ],
    );
    assert!(hit.status.success(), "stderr: {}", stderr(&hit));
    assert_eq!(
        stdout(&hit),
        "withdrew SL-057/REQ-001/SL-057/VT [Planned]\n"
    );
}
