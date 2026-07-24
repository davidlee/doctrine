//! SL-190 PHASE-03 VT-1 — `slice reconcile-phases <ID>`, the status-only writer
//! that rewrites the PRIMARY tree's runtime phase sheets from composite truth.
//!
//! Pins, over the built binary: (a) reconciling a STALE primary tree fixes the
//! sheet to composite truth and is IDEMPOTENT (a second run leaves the sheets
//! byte-identical); (b) non-regression on a mixed inline+dispatch slice — a
//! locally-`completed` inline phase with no landed ref / no registry row survives
//! (the composite is silent/`Local` there, so it is left untouched); (c) the verb
//! REFUSES (non-zero) when a live `dispatch/<slice>` coordination worktree exists
//! (during a drive the coord tree is the writer).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "Doctrine Test")
        .env("GIT_AUTHOR_EMAIL", "test@doctrine.invalid")
        .env("GIT_COMMITTER_NAME", "Doctrine Test")
        .env("GIT_COMMITTER_EMAIL", "test@doctrine.invalid")
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A fresh repo on `main` with one commit; returns its canonical path.
fn init_repo(dir: &Path) -> std::path::PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["commit", "-q", "--allow-empty", "-m", "root"]);
    std::fs::canonicalize(dir).unwrap()
}

fn phases_dir(tree_root: &Path, slice3: &str) -> std::path::PathBuf {
    tree_root
        .join(".doctrine/state/slice")
        .join(slice3)
        .join("phases")
}

/// Write a runtime phase sheet under `tree_root` with the given status.
fn write_phase_sheet(tree_root: &Path, slice3: &str, stem: &str, status: &str) {
    let dir = phases_dir(tree_root, slice3);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{stem}.toml")),
        format!("status = \"{status}\"\n"),
    )
    .unwrap();
}

fn read_sheet(tree_root: &Path, slice3: &str, stem: &str) -> String {
    std::fs::read_to_string(phases_dir(tree_root, slice3).join(format!("{stem}.toml"))).unwrap()
}

/// A durable landed signal: the `phase/<slice3>-NN` ref (a branch off main).
fn land_phase_ref(primary: &Path, slice3: &str, nn: &str) {
    git(
        primary,
        &["branch", &format!("phase/{slice3}-{nn}"), "refs/heads/main"],
    );
}

/// Run `doctrine slice reconcile-phases <id> -p <root>`.
fn reconcile(root: &Path, id: &str) -> Output {
    Command::new(bin())
        .current_dir(root)
        .args([
            "slice",
            "reconcile-phases",
            id,
            "-p",
            root.to_str().unwrap(),
        ])
        .output()
        .expect("spawn doctrine")
}

#[test]
fn reconcile_phases_fixes_a_stale_primary_and_is_idempotent() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let tmp = tempfile::tempdir().unwrap();
    let primary = init_repo(&tmp.path().join("primary"));

    // PHASE-01 landed via a durable phase ref, but the primary sheet is STALE
    // (still in_progress) → composite truth is `completed`, so reconcile must fix
    // it. PHASE-02 has no landed signal → `Local(in_progress)` → left untouched.
    land_phase_ref(&primary, "190", "01");
    write_phase_sheet(&primary, "190", "phase-01", "in_progress");
    write_phase_sheet(&primary, "190", "phase-02", "in_progress");

    let out = reconcile(&primary, "190");
    assert!(
        out.status.success(),
        "reconcile exits zero; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // The landed phase was fixed to composite truth; the Local phase was left as-is.
    assert!(
        read_sheet(&primary, "190", "phase-01").contains("status = \"completed\""),
        "stale landed PHASE-01 reconciled to completed: {}",
        read_sheet(&primary, "190", "phase-01")
    );
    assert!(
        read_sheet(&primary, "190", "phase-02").contains("status = \"in_progress\""),
        "silent (Local) PHASE-02 left untouched: {}",
        read_sheet(&primary, "190", "phase-02")
    );

    // Idempotent: a second run rewrites nothing — the sheets are byte-identical.
    let snapshot_01 = std::fs::read(phases_dir(&primary, "190").join("phase-01.toml")).unwrap();
    let snapshot_02 = std::fs::read(phases_dir(&primary, "190").join("phase-02.toml")).unwrap();

    let out2 = reconcile(&primary, "190");
    assert!(out2.status.success(), "second reconcile exits zero");
    assert_eq!(
        std::fs::read(phases_dir(&primary, "190").join("phase-01.toml")).unwrap(),
        snapshot_01,
        "idempotent: phase-01 byte-identical on re-run"
    );
    assert_eq!(
        std::fs::read(phases_dir(&primary, "190").join("phase-02.toml")).unwrap(),
        snapshot_02,
        "idempotent: phase-02 byte-identical on re-run"
    );
}

#[test]
fn reconcile_phases_never_regresses_a_completed_inline_phase() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let tmp = tempfile::tempdir().unwrap();
    let primary = init_repo(&tmp.path().join("primary"));

    // A mixed inline+dispatch slice: PHASE-01 was dispatched (landed via a ref),
    // PHASE-02 was completed INLINE (no ref, no registry row). The composite is
    // silent about PHASE-02 (`Local`), so reconcile must NOT regress it.
    land_phase_ref(&primary, "190", "01");
    write_phase_sheet(&primary, "190", "phase-01", "in_progress"); // stale → completed
    write_phase_sheet(&primary, "190", "phase-02", "completed"); // inline, must survive

    let out = reconcile(&primary, "190");
    assert!(
        out.status.success(),
        "reconcile exits zero; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        read_sheet(&primary, "190", "phase-01").contains("status = \"completed\""),
        "dispatched PHASE-01 reconciled to completed"
    );
    assert!(
        read_sheet(&primary, "190", "phase-02").contains("status = \"completed\""),
        "inline-completed PHASE-02 NOT regressed: {}",
        read_sheet(&primary, "190", "phase-02")
    );
}

#[test]
fn reconcile_phases_refuses_when_a_live_coord_tree_exists() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let tmp = tempfile::tempdir().unwrap();
    let primary = init_repo(&tmp.path().join("primary"));

    write_phase_sheet(&primary, "190", "phase-01", "in_progress");

    // A live coordination worktree checked out on dispatch/190 → during a drive the
    // coord tree is the phase-sheet writer, so reconcile REFUSES.
    git(&primary, &["branch", "dispatch/190", "refs/heads/main"]);
    let coord = tmp.path().join("coord");
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-q",
            coord.to_str().unwrap(),
            "dispatch/190",
        ],
    );

    let out = reconcile(&primary, "190");
    assert!(
        !out.status.success(),
        "reconcile refuses (non-zero) with a live coord tree; stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("refused"),
        "named refusal on stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The primary sheet was not touched by the refused write.
    assert!(
        read_sheet(&primary, "190", "phase-01").contains("status = \"in_progress\""),
        "refused reconcile leaves the primary sheet untouched"
    );
}
