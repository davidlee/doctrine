// SPDX-License-Identifier: GPL-3.0-only
//! SL-062 PHASE-03 — the transactional, ADR-first `doctrine supersede <NEW> <OLD>`
//! verb, as BLACK-BOX tests over the built binary (design §5.4). Every assertion
//! drives the real CLI shell — never the pure cores directly.
//!
//! - VT-1: happy path — all three surfaces (NEW.supersedes, OLD.superseded_by,
//!   OLD.status==superseded) plus structure preservation.
//! - VT-2: idempotent re-run — a second supersede is a byte-stable no-op.
//! - VT-3: F-D — a different supersessor refused (names the prior); a hand-drift
//!   (status==superseded, empty carve-out) refused → `doctrine validate` (no self-heal).
//! - VT-4: refusals — non-ADR (POL/slice) ADR-first message; cross-kind; self-edge.
//! - VT-5: pre-flight malformed (OLD missing seeded array) refused with NO write to NEW.
//! - VT-6: a hand-induced torn state is FLAGGED by `doctrine validate` (not the verb).

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

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Run the binary against the temp corpus. DOCTRINE_WORKER is explicitly UNSET — the
/// self-arm guard refuses authored writes under it (mem.pattern.dispatch.worker-verify-unset).
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .arg("-p")
        .arg(root)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn new_adr(root: &Path, title: &str, slug: &str) {
    let out = run(root, &["adr", "new", title, "--slug", slug]);
    assert!(out.status.success(), "adr new {slug}: {}", stderr(&out));
}

fn adr_path(root: &Path, id: u32) -> std::path::PathBuf {
    root.join(format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"))
}
fn adr_toml(root: &Path, id: u32) -> String {
    fs::read_to_string(adr_path(root, id)).unwrap()
}

// --- VT-1: happy path — all three surfaces + structure preservation ----------

#[test]
fn supersede_records_all_three_surfaces_and_preserves_structure() {
    let t = tmp();
    let root = t.path();
    new_adr(root, "Old way", "old-way"); // ADR-001
    new_adr(root, "New way", "new-way"); // ADR-002

    let out = run(root, &["supersede", "ADR-002", "ADR-001"]);
    assert!(out.status.success(), "supersede: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ADR-002 supersedes ADR-001\n",
        "success message"
    );

    let new_toml = adr_toml(root, 2);
    let old_toml = adr_toml(root, 1);
    // NEW.supersedes ∋ OLD — now a [[relation]] row (SL-095).
    assert!(
        new_toml.contains("label = \"supersedes\"") && new_toml.contains("target = \"ADR-001\""),
        "NEW.supersedes [[relation]] row lists OLD:\n{new_toml}"
    );
    // OLD.superseded_by ∋ NEW (the single sanctioned reverse carve-out — still typed).
    assert!(
        old_toml.contains("superseded_by = [\"ADR-002\"]"),
        "OLD.superseded_by lists NEW:\n{old_toml}"
    );
    // OLD.status flipped to superseded.
    assert!(
        old_toml.contains("status = \"superseded\""),
        "OLD.status flipped:\n{old_toml}"
    );
    // Both [relationships] blocks survive.
    assert!(new_toml.contains("[relationships]"), "NEW rel block kept");
    assert!(old_toml.contains("[relationships]"), "OLD rel block kept");
    // NEW has [[relation]] array.
    assert!(
        new_toml.contains("[[relation]]"),
        "NEW [[relation]] block present"
    );
}

// --- VT-2: idempotent re-run is a byte-stable no-op --------------------------

#[test]
fn supersede_re_run_is_byte_stable_no_op() {
    let t = tmp();
    let root = t.path();
    new_adr(root, "Old", "old"); // ADR-001
    new_adr(root, "New", "new"); // ADR-002

    assert!(
        run(root, &["supersede", "ADR-002", "ADR-001"])
            .status
            .success()
    );
    let new_once = adr_toml(root, 2);
    let old_once = adr_toml(root, 1);

    let again = run(root, &["supersede", "ADR-002", "ADR-001"]);
    assert!(
        again.status.success(),
        "re-run succeeds: {}",
        stderr(&again)
    );
    assert_eq!(
        stdout(&again),
        "already recorded: ADR-002 supersedes ADR-001\n",
        "re-run reports already recorded"
    );
    assert_eq!(adr_toml(root, 2), new_once, "NEW byte-stable on re-run");
    assert_eq!(adr_toml(root, 1), old_once, "OLD byte-stable on re-run");
}

// --- VT-3: F-D — different supersessor + hand-drift -------------------------

#[test]
fn supersede_refuses_a_different_supersessor() {
    let t = tmp();
    let root = t.path();
    new_adr(root, "Old", "old"); // ADR-001
    new_adr(root, "New one", "new-one"); // ADR-002
    new_adr(root, "New two", "new-two"); // ADR-003

    assert!(
        run(root, &["supersede", "ADR-002", "ADR-001"])
            .status
            .success()
    );
    // A SECOND, DIFFERENT supersessor for the same OLD → refused, names the prior.
    let conflict = run(root, &["supersede", "ADR-003", "ADR-001"]);
    assert!(!conflict.status.success(), "different supersessor refused");
    let msg = stderr(&conflict);
    assert!(
        msg.contains("already superseded by ADR-002") && msg.contains("deferred"),
        "names the prior supersessor + deferral: {msg}"
    );
    // ADR-003 was NOT written — no supersedes [[relation]] row.
    assert!(
        !adr_toml(root, 3).contains("label = \"supersedes\""),
        "the refused NEW wrote nothing:\n{}",
        adr_toml(root, 3)
    );
}

#[test]
fn supersede_refuses_hand_drift_status_without_carveout() {
    let t = tmp();
    let root = t.path();
    new_adr(root, "Old", "old"); // ADR-001
    new_adr(root, "New", "new"); // ADR-002

    // Hand-drift OLD: status==superseded but superseded_by stays empty (no <X>).
    let old = adr_path(root, 1);
    let drifted = fs::read_to_string(&old)
        .unwrap()
        .replace("status = \"proposed\"", "status = \"superseded\"");
    fs::write(&old, &drifted).unwrap();

    let out = run(root, &["supersede", "ADR-002", "ADR-001"]);
    assert!(!out.status.success(), "hand-drift refused");
    let msg = stderr(&out);
    assert!(
        msg.contains("empty/inconsistent") && msg.contains("doctrine validate"),
        "drift refuse points at validate, no <X>: {msg}"
    );
    // NOT self-healed — OLD is left exactly as hand-drifted.
    assert_eq!(
        adr_toml(root, 1),
        drifted,
        "drift refuse does not self-heal"
    );
    // And NEW was not written — no supersedes [[relation]] row.
    assert!(
        !adr_toml(root, 2).contains("label = \"supersedes\""),
        "NEW untouched on drift refuse"
    );
}

// --- VT-4: refusals — non-ADR / cross-kind / self-edge ----------------------

#[test]
fn supersede_refuses_non_adr_cross_kind_and_self() {
    let t = tmp();
    let root = t.path();
    new_adr(root, "A", "a"); // ADR-001
    new_adr(root, "B", "b"); // ADR-002
    // A slice to exercise the non-ADR + cross-kind refusals.
    assert!(
        run(root, &["slice", "new", "Slc", "--slug", "slc"])
            .status
            .success()
    );

    // (a) non-ADR kind (slice) → ADR-first message.
    let non_adr = run(root, &["supersede", "SL-001", "SL-001"]);
    assert!(!non_adr.status.success());
    // self-edge fires first for SL-001/SL-001; use a distinct pair for the kind gate.
    let slc_pair = run(root, &["supersede", "SL-001", "ADR-001"]);
    assert!(!slc_pair.status.success(), "non-ADR/cross-kind refused");

    // (b) cross-family: ADR new, slice old.
    let cross = run(root, &["supersede", "ADR-001", "SL-001"]);
    assert!(!cross.status.success(), "cross-family refused");
    assert!(
        stderr(&cross).contains("cross-family"),
        "names cross-family: {}",
        stderr(&cross)
    );

    // (c) self-edge.
    let selfedge = run(root, &["supersede", "ADR-001", "ADR-001"]);
    assert!(!selfedge.status.success(), "self-edge refused");
    assert!(
        stderr(&selfedge).contains("cannot supersede itself"),
        "names self-supersession: {}",
        stderr(&selfedge)
    );

    // (d) a non-ADR kind that IS within-kind (slice→slice) → ADR-first F2 message.
    assert!(
        run(root, &["slice", "new", "Other", "--slug", "other"])
            .status
            .success()
    );
    let slice_pair = run(root, &["supersede", "SL-001", "SL-002"]);
    assert!(!slice_pair.status.success(), "slice supersession refused");
    assert!(
        stderr(&slice_pair).contains("not yet supported for SL")
            && stderr(&slice_pair).contains("F2"),
        "ADR-first follow-up message: {}",
        stderr(&slice_pair)
    );
}

// --- VT-5: pre-flight malformed — OLD missing a seeded array, NO write to NEW -

#[test]
fn supersede_preflight_malformed_old_refuses_with_no_write_to_new() {
    let t = tmp();
    let root = t.path();
    new_adr(root, "Old", "old"); // ADR-001
    new_adr(root, "New", "new"); // ADR-002

    // Hand-corrupt OLD: drop the seeded `superseded_by` array entirely.
    let old = adr_path(root, 1);
    let corrupt: String = fs::read_to_string(&old)
        .unwrap()
        .lines()
        .filter(|l| !l.trim_start().starts_with("superseded_by"))
        .map(|l| format!("{l}\n"))
        .collect();
    fs::write(&old, &corrupt).unwrap();
    let new_before = adr_toml(root, 2);

    let out = run(root, &["supersede", "ADR-002", "ADR-001"]);
    assert!(!out.status.success(), "malformed OLD refused in pre-flight");
    let msg = stderr(&out);
    assert!(
        msg.contains("superseded_by") && !msg.to_lowercase().contains("regenerate"),
        "non-destructive pre-flight refuse: {msg}"
    );
    // The KEY guarantee: NEW was NOT written (pre-flight refuses before any write).
    assert_eq!(
        adr_toml(root, 2),
        new_before,
        "NEW untouched on pre-flight refuse"
    );
    assert!(
        !new_before.contains("label = \"supersedes\""),
        "NEW.supersedes still empty"
    );
}

// --- VT-6: partial-write detectability — validate FLAGS a torn state ---------

#[test]
fn validate_flags_a_torn_supersession_state() {
    let t = tmp();
    let root = t.path();
    new_adr(root, "Old", "old"); // ADR-001
    new_adr(root, "New", "new"); // ADR-002

    // Hand-induce the torn state the NEW-then-OLD write order would leave on a crash:
    // NEW.supersedes ∋ OLD, but OLD.superseded_by NOT yet written.
    let new = adr_path(root, 2);
    let new_torn = fs::read_to_string(&new).unwrap()
        + "\n[[relation]]\nlabel = \"supersedes\"\ntarget = \"ADR-001\"\n";
    fs::write(&new, &new_torn).unwrap();

    let out = run(root, &["validate"]);
    assert!(!out.status.success(), "torn state fails validate");
    let report = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        report.contains("supersession drift"),
        "validate flags the torn supersession (not the verb):\n{report}"
    );
}
