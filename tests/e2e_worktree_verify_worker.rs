// SPDX-License-Identifier: GPL-3.0-only
//! SL-064 PHASE-08 VT-3 — `doctrine worktree verify-worker` end-to-end over the
//! built binary.
//!
//! The claude `/dispatch` arm's post-spawn base==B check (design §8.4 / option Y):
//! `--base <B> --dir <worktree>` exits 0 only when the worker worktree's HEAD
//! resolves, bears the worker marker, and descends from `B`. Each failure fails
//! LOUD with a distinct token and LEAVES the fork in place (diagnostic only).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

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

/// Stamp the withheld worker marker at `wt` (the SubagentStart stamp's effect).
fn stamp_marker(wt: &Path) {
    let dir = wt.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("worker"), b"").unwrap();
}

/// Run `doctrine worktree verify-worker --base <base> --dir <wt> [--branch <s>]`.
fn verify_worker(base: &str, wt: &Path, branch: Option<&str>) -> Output {
    let mut args = vec![
        "worktree".to_string(),
        "verify-worker".to_string(),
        "--base".to_string(),
        base.to_string(),
        "--dir".to_string(),
        wt.to_str().unwrap().to_string(),
    ];
    if let Some(s) = branch {
        args.push("--branch".to_string());
        args.push(s.to_string());
    }
    Command::new(BIN)
        .args(&args)
        .output()
        .expect("spawn doctrine")
}

/// A linked worktree of `root` checked out at `at` (detached), placed under `path`.
fn add_worktree(root: &Path, path: &Path, at: &str) {
    git(
        root,
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            path.to_str().unwrap(),
            at,
        ],
    );
}

/// A linked worktree of `root` checked out on branch `branch`, under `path`.
fn add_worktree_on_branch(root: &Path, path: &Path, branch: &str) {
    git(root, &["branch", branch]);
    git(
        root,
        &["worktree", "add", "-q", path.to_str().unwrap(), branch],
    );
}

#[test]
fn stamped_b_based_worker_verifies_ok() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    let wt = tmp.path().join("wt-ok");
    add_worktree(root, &wt, &base);
    stamp_marker(&wt);

    let out = verify_worker(&base, &wt, None);
    assert!(
        out.status.success(),
        "stamped B-based worker must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn unstamped_worker_refuses_unstamped() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // B-based worktree but NO marker stamped ⇒ unstamped (named before base).
    let wt = tmp.path().join("wt-unstamped");
    add_worktree(root, &wt, &base);

    let out = verify_worker(&base, &wt, None);
    assert!(
        !out.status.success(),
        "unstamped worker must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unstamped"),
        "must name the unstamped token; stderr: {stderr}"
    );
    // Diagnostic only: the fork is LEFT in place.
    assert!(wt.exists(), "verify-worker must NOT remove the fork");
}

#[test]
fn stale_base_worker_refuses_wrong_base() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // Worker forked at the ORIGINAL base, stamped.
    let wt = tmp.path().join("wt-wrongbase");
    add_worktree(root, &wt, &base);
    stamp_marker(&wt);

    // The orchestrator's base then MOVED on (a later commit B'); the worker HEAD
    // does NOT descend from B'.
    std::fs::write(root.join("c.txt"), "moved").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "moved"]);
    let moved = git(root, &["rev-parse", "HEAD"]);

    let out = verify_worker(&moved, &wt, None);
    assert!(
        !out.status.success(),
        "stale-base worker must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("wrong-base"),
        "must name the wrong-base token; stderr: {stderr}"
    );
    assert!(wt.exists(), "verify-worker must NOT remove the fork");
}

#[test]
fn unresolvable_head_refuses_no_worker_head() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // An empty git repo: HEAD does not resolve (no commits). Stamp it so the
    // marker leg can't fire first — the head-resolve verdict must win.
    let empty = tmp.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    git(&empty, &["init", "-q", "-b", "main"]);
    stamp_marker(&empty);

    let out = verify_worker(&base, &empty, None);
    assert!(
        !out.status.success(),
        "unresolvable HEAD must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no-worker-head"),
        "must name the no-worker-head token; stderr: {stderr}"
    );
}

// --- SL-123 PHASE-01 VT-4: not-isolated + branch-mismatch belts end-to-end ---

#[test]
fn primary_tree_refuses_not_isolated() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // The PRIMARY tree itself (git-dir == git-common-dir): stamped AND base is an
    // ancestor (HEAD == base), so marker/base would both pass — but is_isolated
    // is checked FIRST (after head), so not-isolated must win. This is the G1 fix:
    // the silent fallback-to-main where B is coincidentally an ancestor.
    stamp_marker(root);

    let out = verify_worker(&base, root, None);
    assert!(
        !out.status.success(),
        "primary tree must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not-isolated"),
        "must name the not-isolated token; stderr: {stderr}"
    );
    assert!(root.exists(), "verify-worker must NOT remove the tree");
}

#[test]
fn incoherent_branch_refuses_branch_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // Isolated, stamped worker on B — passes head/isolated/marker/base.
    let wt = tmp.path().join("wt-incoherent");
    add_worktree(root, &wt, &base);
    stamp_marker(&wt);

    // A branch S whose tip has MOVED past the worktree HEAD: the footer dir↔branch
    // are incoherent (verify-worker --dir and import --S would see different states).
    git(root, &["branch", "s"]);
    std::fs::write(root.join("d.txt"), "ahead").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "ahead"]);
    git(root, &["branch", "-f", "s", "HEAD"]);

    let out = verify_worker(&base, &wt, Some("s"));
    assert!(
        !out.status.success(),
        "incoherent --branch must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("branch-mismatch"),
        "must name the branch-mismatch token; stderr: {stderr}"
    );
    assert!(wt.exists(), "verify-worker must NOT remove the fork");
}

#[test]
fn coherent_branch_verifies_ok() {
    let tmp = tempfile::tempdir().unwrap();
    let root = init_repo(tmp.path());
    let base = git(root, &["rev-parse", "HEAD"]);

    // Isolated, stamped worker checked out ON branch S at B: HEAD(dir) == tip(S),
    // so the coherence belt passes and the whole belt-set is Ok.
    let wt = tmp.path().join("wt-coherent");
    add_worktree_on_branch(root, &wt, "s");
    stamp_marker(&wt);

    let out = verify_worker(&base, &wt, Some("s"));
    assert!(
        out.status.success(),
        "coherent --branch must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
