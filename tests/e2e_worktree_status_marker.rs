// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-05 — `doctrine worktree status [--assert]` and `worktree marker
//! --clear --operator` end-to-end over the BUILT binary.
//!
//! * VT-2: the four-state `status` golden (force_no_tty-stable plain lines):
//!   no-signal → allowed; marker-only → refused signal: marker; env-only → refused
//!   signal: env; both → refused signal: both.
//! * VT-3: `status --assert` gate — clean linked-worktree entry → exit 0; a stale
//!   marker in a linked worktree → non-zero `stale-marker` naming the remedy; exit
//!   0 after `marker --clear --operator`. The human line and the `--assert` exit
//!   read ONE describe_mode and never disagree.
//! * VT-4: `marker --clear` self-brick cure — a stale marker refuses writes;
//!   `marker --clear --operator` (env unset) restores writes from within the CLI;
//!   refused when DOCTRINE_WORKER set / cwd outside the marker's tree / bare
//!   `--clear` in a linked worktree (the accident-fence).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
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

fn stamp_marker(root: &Path) {
    let dir = root.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("worker"), b"").unwrap();
}

fn marker_exists(root: &Path) -> bool {
    root.join(".doctrine/state/dispatch/worker").exists()
}

/// Run `doctrine <args>` in `cwd`; env governed by `worker` (Some(true) sets
/// DOCTRINE_WORKER=1; None removes it).
fn run(cwd: &Path, worker: Option<bool>, args: &[&str]) -> Output {
    let mut cmd = Command::new(bin());
    cmd.args(args).current_dir(cwd);
    match worker {
        Some(true) => {
            cmd.env("DOCTRINE_WORKER", "1");
        }
        Some(false) | None => {
            cmd.env_remove("DOCTRINE_WORKER");
        }
    }
    cmd.output().expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

// --- VT-2: the four-state status golden ---

#[test]
fn status_four_states() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-status");

    // (1) no signal: a fresh fork, env unset ⇒ allowed.
    let out = run(&fork, None, &["worktree", "status"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "worker fork: no — writes allowed\n");

    // (2) marker only: stamp the marker, env unset ⇒ refused signal: marker.
    stamp_marker(&fork);
    let out = run(&fork, None, &["worktree", "status"]);
    assert!(
        out.status.success(),
        "plain status (no --assert) always exits 0"
    );
    assert_eq!(
        stdout(&out),
        "worker fork: yes — writes refused; signal: marker\n"
    );

    // (3) env only: clear the marker, set the env ⇒ refused signal: env.
    std::fs::remove_file(fork.join(".doctrine/state/dispatch/worker")).unwrap();
    let out = run(&fork, Some(true), &["worktree", "status"]);
    assert_eq!(
        stdout(&out),
        "worker fork: yes — writes refused; signal: env\n"
    );

    // (4) both: marker + env ⇒ refused signal: both.
    stamp_marker(&fork);
    let out = run(&fork, Some(true), &["worktree", "status"]);
    assert_eq!(
        stdout(&out),
        "worker fork: yes — writes refused; signal: both\n"
    );
}

// --- VT-3: status --assert gate (one describe_mode, never disagree) ---

#[test]
fn status_assert_gate() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-assert");

    // Clean linked-worktree entry (no marker, no env) ⇒ exit 0, human says allowed.
    let plain = run(&fork, None, &["worktree", "status"]);
    let asserted = run(&fork, None, &["worktree", "status", "--assert"]);
    assert!(asserted.status.success(), "clean entry ⇒ --assert exit 0");
    assert_eq!(stdout(&plain), "worker fork: no — writes allowed\n");
    // The human line is identical whether or not --assert is passed (one source).
    assert_eq!(stdout(&asserted), stdout(&plain));

    // A stale marker in a linked worktree ⇒ non-zero `stale-marker` naming remedy.
    stamp_marker(&fork);
    let asserted = run(&fork, None, &["worktree", "status", "--assert"]);
    assert!(
        !asserted.status.success(),
        "stale marker ⇒ --assert nonzero; stderr: {}",
        stderr(&asserted)
    );
    let err = stderr(&asserted);
    assert!(
        err.contains("stale-marker"),
        "must carry the stale-marker token; stderr: {err}"
    );
    assert!(
        err.contains("marker --clear --operator"),
        "must NAME the remedy; stderr: {err}"
    );
    // The human line still reports refused: marker (the SAME state the assert read).
    assert!(
        stdout(&asserted).contains("signal: marker"),
        "human line and --assert read one describe_mode; stdout: {}",
        stdout(&asserted)
    );

    // After marker --clear --operator (env unset) ⇒ --assert back to exit 0.
    let cleared = run(
        &fork,
        None,
        &["worktree", "marker", "--clear", "--operator"],
    );
    assert!(
        cleared.status.success(),
        "clear must succeed; stderr: {}",
        stderr(&cleared)
    );
    let asserted = run(&fork, None, &["worktree", "status", "--assert"]);
    assert!(
        asserted.status.success(),
        "after clear ⇒ --assert exit 0; stderr: {}",
        stderr(&asserted)
    );
}

// --- VT-4: marker --clear self-brick cure + fences ---

#[test]
fn marker_clear_cures_self_brick() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-cure");

    // Stale marker ⇒ writes refused (the self-brick).
    stamp_marker(&fork);
    let refused = run(&fork, None, &["slice", "new", "x"]);
    assert!(!refused.status.success(), "stale marker bricks writes");

    // marker --clear --operator (env unset) restores writes from within the CLI.
    let cleared = run(
        &fork,
        None,
        &["worktree", "marker", "--clear", "--operator"],
    );
    assert!(
        cleared.status.success(),
        "clear must succeed; stderr: {}",
        stderr(&cleared)
    );
    assert!(
        stdout(&cleared).contains("CLEARED"),
        "loud receipt; stdout: {}",
        stdout(&cleared)
    );
    assert!(!marker_exists(&fork), "marker gone after clear");

    // Writes restored: slice new now passes the guard (succeeds).
    let allowed = run(&fork, None, &["slice", "new", "demo"]);
    assert!(
        allowed.status.success(),
        "writes restored after clear; stderr: {}",
        stderr(&allowed)
    );
}

#[test]
fn marker_clear_refused_when_worker_env_set() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-envfence");
    stamp_marker(&fork);

    let out = run(
        &fork,
        Some(true),
        &["worktree", "marker", "--clear", "--operator"],
    );
    assert!(
        !out.status.success(),
        "clear refused while DOCTRINE_WORKER set; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "refusal names the env leg; stderr: {}",
        stderr(&out)
    );
    assert!(marker_exists(&fork), "marker untouched on a refused clear");
}

#[test]
fn marker_clear_refused_in_linked_worktree_without_operator() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-acc");
    stamp_marker(&fork);

    // Bare --clear (no --operator) in a LINKED worktree ⇒ refused (accident-fence).
    let out = run(&fork, None, &["worktree", "marker", "--clear"]);
    assert!(
        !out.status.success(),
        "bare --clear in a linked worktree must refuse; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("--operator"),
        "refusal names the --operator fence; stderr: {}",
        stderr(&out)
    );
    assert!(marker_exists(&fork), "marker untouched on a refused clear");

    // With --operator it goes through.
    let out = run(
        &fork,
        None,
        &["worktree", "marker", "--clear", "--operator"],
    );
    assert!(out.status.success(), "--operator confirms the clear");
    assert!(!marker_exists(&fork));
}

#[test]
fn marker_clear_refused_when_cwd_not_tree_root() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-cwd");
    stamp_marker(&fork);

    // A subdir of the fork: cwd != the marker's tree root ⇒ refused.
    let sub = fork.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let out = run(&sub, None, &["worktree", "marker", "--clear", "--operator"]);
    assert!(
        !out.status.success(),
        "clear from a subdir (cwd != tree root) must refuse; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("tree root"),
        "refusal names the tree-root fence; stderr: {}",
        stderr(&out)
    );
    assert!(marker_exists(&fork), "marker untouched on a refused clear");
}
