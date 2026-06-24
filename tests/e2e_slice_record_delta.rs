//! SL-147 PHASE-04 VT-3 — `slice record-delta`, the manual escape hatch beside
//! the automatic solo phase-binding.
//!
//! Pins, over the built binary: (a) happy path — a valid `(start, end)` range is
//! resolved to oids, guarded, and UPSERTed into the slice's arm-neutral registry
//! at the runtime path `.doctrine/state/slice/<NNN>/boundaries.toml`; (b) guard
//! rejection — a non-ancestor (backwards) range and a merge `end` are refused
//! with nothing persisted; (c) the registry resolves to the PRIMARY tree even
//! when the verb runs from a LINKED worktree (the cross-worktree shared-file
//! contract).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

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

/// Run `doctrine slice record-delta <id> <phase> --start <a> --end <b> -p <root>`
/// from `cwd` (which may differ from the project root passed via `-p`).
fn record_delta(cwd: &Path, root: &Path, id: &str, phase: &str, start: &str, end: &str) -> Output {
    Command::new(BIN)
        .current_dir(cwd)
        .args([
            "slice",
            "record-delta",
            id,
            phase,
            "--start",
            start,
            "--end",
            end,
            "-p",
            root.to_str().unwrap(),
        ])
        .output()
        .expect("spawn doctrine")
}

fn deltas_path(root: &Path) -> std::path::PathBuf {
    root.join(".doctrine/state/slice/147/boundaries.toml")
}

#[test]
fn happy_path_upserts_a_guarded_row_at_the_runtime_registry() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = init_repo(&tmp.path().join("repo"));
    let start = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "code"]);
    let end = git(&repo, &["rev-parse", "HEAD"]);

    let out = record_delta(&repo, &repo, "147", "PHASE-01", &start, &end);
    assert!(
        out.status.success(),
        "record-delta ok; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let body = std::fs::read_to_string(deltas_path(&repo)).expect("registry written");
    assert!(body.contains("[[boundary]]"), "row header: {body}");
    assert!(body.contains("phase = \"PHASE-01\""), "phase row: {body}");
    assert!(body.contains(&end), "resolved code_end oid: {body}");

    // Re-record the same phase with a different end → UPSERT (one row, replaced).
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "more"]);
    let end2 = git(&repo, &["rev-parse", "HEAD"]);
    let out = record_delta(&repo, &repo, "147", "PHASE-01", &start, &end2);
    assert!(
        out.status.success(),
        "re-record ok: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = std::fs::read_to_string(deltas_path(&repo)).unwrap();
    assert_eq!(
        body.matches("[[boundary]]").count(),
        1,
        "upsert, not append: {body}"
    );
    assert!(body.contains(&end2), "advanced end persisted: {body}");
}

#[test]
fn guard_rejects_a_non_ancestor_range_and_persists_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = init_repo(&tmp.path().join("repo"));
    let a = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "code"]);
    let head = git(&repo, &["rev-parse", "HEAD"]);

    // start = later, end = earlier → not a forward delta.
    let out = record_delta(&repo, &repo, "147", "PHASE-01", &head, &a);
    assert!(!out.status.success(), "backwards range refused");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not an ancestor"),
        "named guard refusal: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!deltas_path(&repo).exists(), "refused run records nothing");
}

#[test]
fn guard_rejects_a_merge_end_and_persists_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = init_repo(&tmp.path().join("repo"));
    let base = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "a"]);
    let a = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["checkout", "-q", "-b", "side", &base]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "b"]);
    git(&repo, &["checkout", "-q", "main"]);
    git(&repo, &["merge", "-q", "--no-ff", "--no-edit", "side"]);
    let merge = git(&repo, &["rev-parse", "HEAD"]);

    let out = record_delta(&repo, &repo, "147", "PHASE-01", &a, &merge);
    assert!(!out.status.success(), "merge end refused");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("merge commit"),
        "named merge refusal: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!deltas_path(&repo).exists(), "refused run records nothing");
}

#[test]
fn record_from_a_linked_worktree_resolves_the_primary_tree_registry() {
    let tmp = tempfile::tempdir().unwrap();
    let primary = init_repo(&tmp.path().join("primary"));
    let head = git(&primary, &["rev-parse", "HEAD"]);
    let fork = tmp.path().join("fork");
    git(
        &primary,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feat",
            fork.to_str().unwrap(),
        ],
    );
    let fork = std::fs::canonicalize(&fork).unwrap();

    // Run from the LINKED worktree, with `-p` pointed at the linked worktree too:
    // the registry still resolves to the PRIMARY tree's file.
    let out = record_delta(&fork, &fork, "147", "PHASE-01", &head, &head);
    assert!(
        out.status.success(),
        "record-delta from a linked worktree ok: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        deltas_path(&primary).exists(),
        "row lands in the PRIMARY tree"
    );
    assert!(!deltas_path(&fork).exists(), "not in the linked worktree");
}
