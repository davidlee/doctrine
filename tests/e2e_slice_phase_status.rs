//! SL-190 PHASE-02 VT-1 — `slice status <ID> --across-trees [--assert]`, the
//! cross-tree phase-status query verb.
//!
//! Pins, over the built binary: (a) `--across-trees` renders the composite
//! per-phase truth + a per-tree divergence table (landed | coord | local | →
//! truth) over a fixture with a primary tree and a live `dispatch/<slice>`
//! coordination tree holding DIVERGENT per-phase sheets; (b) `--assert` exits
//! NON-ZERO on a seeded CONFLICT (a phase landed via a `phase/<slice>-NN` ref yet
//! the live coord sheet is `in_progress`); (c) `--assert` exits ZERO on a fresh
//! handoff tree with no landed refs / no coord (a machine mid-handoff must not be
//! permanently red).

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

/// Write a runtime phase sheet (`.doctrine/state/slice/<NNN>/phases/phase-NN.toml`)
/// under `tree_root` with the given status — the gitignored per-tree runtime tier.
fn write_phase_sheet(tree_root: &Path, slice3: &str, stem: &str, status: &str) {
    let dir = tree_root
        .join(".doctrine/state/slice")
        .join(slice3)
        .join("phases");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{stem}.toml")),
        format!("status = \"{status}\"\n"),
    )
    .unwrap();
}

/// Run `doctrine slice status <id> --across-trees [--assert] -p <root>`.
fn across(root: &Path, id: &str, assert: bool) -> Output {
    let mut args = vec!["slice", "status", id, "--across-trees"];
    if assert {
        args.push("--assert");
    }
    args.extend_from_slice(&["-p", root.to_str().unwrap()]);
    Command::new(bin())
        .current_dir(root)
        .args(&args)
        .output()
        .expect("spawn doctrine")
}

#[test]
fn across_trees_renders_the_divergence_table_and_assert_fires_on_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    let primary = init_repo(&tmp.path().join("primary"));

    // PHASE-01 landed via a durable phase ref, but a live coord tree still has it
    // `in_progress` → CONFLICT (rework). PHASE-02 is in_progress in coord only.
    git(
        &primary,
        &["branch", "phase/190-01", "refs/heads/main"],
    );

    // A live coordination worktree checked out on dispatch/190 with divergent sheets.
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
    let coord = std::fs::canonicalize(&coord).unwrap();
    write_phase_sheet(&coord, "190", "phase-01", "in_progress");
    write_phase_sheet(&coord, "190", "phase-02", "in_progress");

    // The primary's own local sheets: PHASE-01 believes it completed.
    write_phase_sheet(&primary, "190", "phase-01", "completed");

    // (a) --across-trees renders the composite truth + per-tree divergence table.
    let out = across(&primary, "190", false);
    assert!(
        out.status.success(),
        "plain --across-trees exits zero; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("landed") && stdout.contains("coord") && stdout.contains("local"),
        "per-tree divergence table header (landed | coord | local | → truth): {stdout}"
    );
    assert!(
        stdout.contains("CONFLICT"),
        "the seeded rework CONFLICT is surfaced: {stdout}"
    );
    assert!(stdout.contains("PHASE-01"), "per-phase rows: {stdout}");

    // (b) --assert exits NON-ZERO on the seeded CONFLICT.
    let out = across(&primary, "190", true);
    assert!(
        !out.status.success(),
        "--assert fires non-zero on CONFLICT; stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn assert_exits_zero_on_a_fresh_handoff_tree_with_no_landed_and_no_coord() {
    let tmp = tempfile::tempdir().unwrap();
    let primary = init_repo(&tmp.path().join("primary"));

    // A fresh machine mid-handoff: no phase refs, no cache, no coord tree — only a
    // local in-flight sheet. Composite == LOCAL; nothing is assert-worthy, so a
    // handoff machine must NOT be permanently red.
    write_phase_sheet(&primary, "190", "phase-01", "in_progress");

    let out = across(&primary, "190", true);
    assert!(
        out.status.success(),
        "--assert exits zero when nothing conflicts; stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
