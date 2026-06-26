// SPDX-License-Identifier: GPL-3.0-only
//! SL-031 PHASE-02 VT-2 — `doctrine worktree branch-point-check` end-to-end over
//! the built binary.
//!
//! HEAD-stationarity assert at the batch-commit boundary (design §5.2, D5
//! concurrency extension, C-V): `--base <HEAD>` exits 0 while HEAD is stationary;
//! after a stray commit moves HEAD, the default-`--head` run (`git rev-parse
//! HEAD`) exits 1. A ref-equality compare, not a merge-base.

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

/// Run `git -C <dir> <args>`, asserting success; returns trimmed stdout.
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A repo with one tracked commit. Returns its root.
fn init_repo(dir: &Path) -> &Path {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// Run `doctrine worktree branch-point-check` against `root`.
fn branch_point_check(root: &Path, args: &[&str]) -> Output {
    let mut full = vec![
        "worktree",
        "branch-point-check",
        "-p",
        root.to_str().unwrap(),
    ];
    full.extend_from_slice(args);
    Command::new(bin())
        .args(&full)
        .output()
        .expect("spawn doctrine")
}

#[test]
fn stationary_head_exits_zero_moved_head_exits_one() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // Stationary: --base == HEAD (default --head reads HEAD) ⇒ exit 0.
    let out = branch_point_check(root, &["--base", &base]);
    assert!(
        out.status.success(),
        "stationary HEAD must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Explicit matching --head ⇒ exit 0, no git read needed.
    let out = branch_point_check(root, &["--base", &base, "--head", &base]);
    assert!(out.status.success(), "explicit equal head must exit 0");

    // A stray commit moves HEAD; the old base no longer matches ⇒ exit 1.
    std::fs::write(root.join("b.txt"), "stray").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "stray"]);
    let out = branch_point_check(root, &["--base", &base]);
    assert!(
        !out.status.success(),
        "moved HEAD must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );

    // Explicit differing --head ⇒ exit 1 without resolving HEAD.
    let out = branch_point_check(root, &["--base", &base, "--head", "deadbeef"]);
    assert!(
        !out.status.success(),
        "explicit differing head must exit nonzero"
    );
}

// --- SL-041 VT-2/3/4 — both ends resolved in the shell (ISS-002) ---

#[test]
fn symbolic_base_resolves_against_head() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());

    // VT-2: --base HEAD (symbolic) vs stationary HEAD ⇒ exit 0. The pre-fix verb
    // string-compared "HEAD" != <sha> and falsely reported "moved" (exit 1).
    let out = branch_point_check(root, &["--base", "HEAD"]);
    assert!(
        out.status.success(),
        "symbolic --base HEAD on a stationary tree must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // A branch name is also a symbolic ref ⇒ resolves and is stationary.
    let out = branch_point_check(root, &["--base", "main"]);
    assert!(out.status.success(), "symbolic --base main must exit 0");
}

#[test]
fn both_ends_are_resolved_not_string_compared() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // VT-3a (decisive): resolved sha base vs symbolic --head HEAD on a stationary
    // tree ⇒ exit 0. Pre-fix this string-compared <sha> != "HEAD" and falsely
    // reported "moved" — so it proves the *passed* --head is resolved, not raw.
    let out = branch_point_check(root, &["--base", &base, "--head", "HEAD"]);
    assert!(
        out.status.success(),
        "resolved base vs symbolic --head HEAD (stationary) must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // A stray commit moves HEAD.
    std::fs::write(root.join("b.txt"), "stray").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "stray"]);

    // VT-3b: stale base sha vs resolved --head HEAD ⇒ exit 1 (the passed --head
    // is itself resolved, not trusted verbatim).
    let out = branch_point_check(root, &["--base", &base, "--head", "HEAD"]);
    assert!(
        !out.status.success(),
        "stale base vs resolved symbolic head must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn unresolvable_base_bails() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());

    // VT-4: a ref that resolves to nothing ⇒ the guard bails (non-zero), it does
    // not silently treat the unresolved symbol as stationary or moved.
    let out = branch_point_check(root, &["--base", "no-such-ref"]);
    assert!(
        !out.status.success(),
        "unresolvable --base must bail (nonzero); stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
