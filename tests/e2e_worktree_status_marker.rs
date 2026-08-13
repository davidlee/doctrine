// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-05 / SL-254 PHASE-05 — `doctrine worktree status` end-to-end over
//! the BUILT binary.
//!
//! * VT-2: the `status` golden (force_no_tty-stable plain lines). SL-254 `DEC-207`
//!   collapsed worker identity to the `DOCTRINE_WORKER` env var alone, so the
//!   four-state table (no-signal / marker-only / env-only / both) is a TWO-state
//!   table over one input, and it is topology-independent: the same two lines come
//!   out of a linked worktree fork and a plain repo alike.
//!
//! RETIRED HERE (subject deleted, not skipped — SL-254 PHASE-05):
//!   * `status_assert_gate` — `--assert` existed to detect a STALE marker. An env
//!     var cannot go stale (it dies with the process), so the whole stale class,
//!     the flag, and the `stale-marker` exit token retired together.
//!   * `marker_clear_cures_self_brick` and the three `marker --clear` fence tests
//!     (`--operator` confirmation, cwd-must-be-tree-root, refused-while-env-set) —
//!     `worktree marker --clear` was the CURE for a stale marker. With no stale
//!     class there is no self-brick to cure, and the verb no longer parses.
//!     The one assertion inside them that was NOT about the marker — that
//!     worker mode bricks `slice new` — survives in `e2e_worker_guard.rs`
//!     (`worker_env_in_linked_worktree_refuses_writes`).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
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
    std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// Make a real linked worktree fork of `src` at `<holder>/fork` on branch `branch`.
fn add_fork(src: &Path, holder: &Path, branch: &str) -> PathBuf {
    let base = git(src, &["rev-parse", "HEAD"]);
    let fork = holder.join("fork");
    git(
        src,
        &[
            "worktree",
            "add",
            "-b",
            branch,
            fork.to_str().unwrap(),
            &base,
        ],
    );
    fork
}

/// Run `doctrine <args>` in `cwd`; `worker` governs `DOCTRINE_WORKER`.
fn run(cwd: &Path, worker: bool, args: &[&str]) -> Output {
    let mut cmd = common::doctrine_cmd(cwd);
    cmd.args(args);
    if worker {
        cmd.env("DOCTRINE_WORKER", "1");
    } else {
        cmd.env_remove("DOCTRINE_WORKER");
    }
    cmd.output().expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

const ALLOWED: &str = "worker fork: no — writes allowed\n";
const REFUSED: &str = "worker fork: yes — writes refused; signal: env\n";

// --- VT-2: the status golden ---

#[test]
fn status_two_states_in_a_linked_fork() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-status");

    // (1) env unset ⇒ allowed, even standing in a genuine fork.
    let out = run(&fork, false, &["worktree", "status"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), ALLOWED);

    // (2) env set ⇒ refused, naming the one surviving signal.
    let out = run(&fork, true, &["worktree", "status"]);
    assert!(
        out.status.success(),
        "status always exits 0 — it reports, it does not gate; stderr: {}",
        stderr(&out)
    );
    assert_eq!(stdout(&out), REFUSED);
}

// SL-254 PHASE-05: new. The four-state table's two deleted rows (marker-only, both)
// were the only place `status` was proven to read something OTHER than the env, so
// removing them would leave the topology-independence of `DEC-207` unpinned. This
// replaces them: the primary (non-linked) tree reports the SAME two lines as the
// fork above, so no residual tree-shape input can have crept back into the verdict.
#[test]
fn status_is_topology_independent() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());

    let out = run(src.path(), false, &["worktree", "status"]);
    assert_eq!(
        stdout(&out),
        ALLOWED,
        "a non-linked tree with the env unset reports allowed"
    );

    let out = run(src.path(), true, &["worktree", "status"]);
    assert_eq!(
        stdout(&out),
        REFUSED,
        "a worker process on a NON-linked tree is still a worker (`DEC-207`)"
    );
}

// SL-254 PHASE-05: new, and load-bearing. `--assert` retired with the stale-marker
// class; this pins that it stayed retired rather than quietly resurfacing as an
// ignored no-op flag, which would leave callers believing they still had a gate.
#[test]
fn status_assert_flag_is_rejected_by_the_parser() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());

    let out = run(src.path(), false, &["worktree", "status", "--assert"]);
    assert!(
        !out.status.success(),
        "`--assert` must not be silently accepted; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("unexpected argument '--assert'"),
        "the PARSER must reject it; stderr: {}",
        stderr(&out)
    );
}

// SL-254 PHASE-05: new. `worktree marker` was the whole subject of the four deleted
// tests in this file; this one line is what is left worth asserting about it —
// that the verb is gone from the CLI rather than surviving as a stub that silently
// succeeds.
#[test]
fn worktree_marker_verb_no_longer_parses() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());

    for args in [
        ["worktree", "marker", "--clear", "--operator"].as_slice(),
        ["worktree", "marker", "--stamp-subagent"].as_slice(),
        ["worktree", "verify-worker"].as_slice(),
    ] {
        let out = run(src.path(), false, args);
        assert!(
            !out.status.success(),
            "{args:?} must not parse; stdout: {}",
            stdout(&out)
        );
        assert!(
            stderr(&out).contains("unrecognized subcommand"),
            "{args:?} must be rejected by the PARSER; stderr: {}",
            stderr(&out)
        );
    }
}
