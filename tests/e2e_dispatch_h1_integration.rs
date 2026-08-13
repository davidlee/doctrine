// SPDX-License-Identifier: GPL-3.0-only
//! SL-152 PHASE-04 VT-4 — H1 (wrong-base fallback) is structurally dead: the worker
//! fork lands at the orchestrator's captured base `B`, never at a `main` that moved
//! past it between capture and fork.
//!
//! **Retargeted at SL-254 PHASE-06.** The original chained `dispatch arm-spawn` (the
//! orchestrator writer) into `worktree create-fork`'s Fork arm (the hook consumer),
//! and its point was that the file contract held ACROSS those two verbs. Both halves
//! are deleted with the claude dispatch arm: the base no longer travels through a file
//! at all, it is an argument to the one surviving worker-fork writer,
//! `worktree fork --worker`. So the seam claim is gone with its seam — what survives,
//! and is asserted here against that writer, is the H1 claim itself plus the four
//! non-marker post-spawn belt conjuncts SL-254 PHASE-05 relocated into this file.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

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

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// `doctrine worktree fork --base B --branch BR --dir D --worker -p <root>` — the one
/// surviving worker-fork writer.
fn worktree_fork(root: &Path, base: &str, branch: &str, dir: &Path) -> Output {
    common::doctrine_cmd(root)
        .args([
            "worktree", "fork", "--base", base, "--branch", branch, "--dir",
        ])
        .arg(dir)
        .args(["--worker", "-p"])
        .arg(root)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("spawn doctrine")
}

// --- VT-4: the worker fork lands at base B under a moving main ---

#[test]
fn worker_fork_lands_at_base_b_under_moving_main() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();

    // B = the tip the orchestrator captured (dispatch setup's `base=`).
    let b = git(root.path(), &["rev-parse", "HEAD"]);

    // main MOVES past B between capture and fork (the H1 hazard window).
    std::fs::write(root.path().join("drift.txt"), "post-B").unwrap();
    git(root.path(), &["add", "drift.txt"]);
    git(root.path(), &["commit", "-q", "-m", "main advances past B"]);
    let moved = git(root.path(), &["rev-parse", "HEAD"]);
    assert_ne!(b, moved, "main advanced past B");

    let dir = root_canon.join(".worktrees/agent-h1");
    let forked = worktree_fork(&root_canon, &b, "dispatch/agent-h1", &dir);
    assert!(
        forked.status.success(),
        "worktree fork must succeed; stderr: {}",
        stderr(&forked)
    );
    assert!(dir.is_dir(), "fork created at <root>/.worktrees/<name>");

    // The headline assertion: the fork is pinned to B (the captured value), NOT the
    // moved main — no wrong-base fallback survives.
    assert_eq!(
        git(&dir, &["rev-parse", "HEAD"]),
        b,
        "fork HEAD is base B (the captured value), not the moved main"
    );
    assert_ne!(
        git(&dir, &["rev-parse", "HEAD"]),
        moved,
        "fork did NOT fall back to the moved main (H1 dead)"
    );

    // VT-2 / F7: the fork satisfies the post-spawn belt.
    //
    // SL-254 PHASE-05: this block used to assert the marker file existed and then
    // shell `worktree verify-worker`. Both retired with the marker (`DEC-207` —
    // identity is the process's `DOCTRINE_WORKER`, so there is nothing about the
    // TREE left for a verify verb to inspect, and the verb no longer parses). The
    // belt's other four conjuncts are NOT marker-derived, so rather than lose them
    // they are asserted here directly, against the same fork: HEAD resolves, the
    // tree is isolated (linked worktree), B is an ancestor of HEAD, and HEAD is the
    // tip of the branch the funnel would import as S.
    assert!(
        !git(&dir, &["rev-parse", "--verify", "HEAD"]).is_empty(),
        "worker HEAD resolves in the fork"
    );
    assert_ne!(
        git(&dir, &["rev-parse", "--git-dir"]),
        git(&dir, &["rev-parse", "--git-common-dir"]),
        "the fork is ISOLATED — a linked worktree, not a second checkout of the root"
    );
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(["merge-base", "--is-ancestor", &b, "HEAD"])
            .status()
            .expect("spawn git")
            .success(),
        "base B is an ancestor of the fork HEAD"
    );
    assert_eq!(
        git(&dir, &["rev-parse", "--verify", "HEAD"]),
        git(
            &dir,
            &["rev-parse", "--verify", "dispatch/agent-h1^{commit}"]
        ),
        "fork HEAD is the tip of `dispatch/agent-h1` — the branch the funnel imports as S"
    );
}
