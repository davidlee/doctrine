// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-06 — `doctrine worktree fork --base <B> --branch <name> --dir
//! <path> [--worker]` end-to-end over the BUILT binary.
//!
//! * VT-1: happy path — env contract on stdout, human status on stderr, the
//!   worktree+branch exist at `<B>`, marker WRITTEN under `--worker` and ABSENT
//!   solo; the three pre-`add` refusals (dir-exists / branch-exists / B-not-a-
//!   commit) each exit non-zero and leave NO fork.
//! * VT-2: compensating cleanup — a provision failure after `git worktree add`
//!   exits non-zero AND leaves NO leftover worktree/branch (asserted GONE).
//! * VT-4: `fork` Orchestrator refusal drives the real CLI — refused from a marked
//!   linked worktree AND from a DOCTRINE_WORKER-set process, naming the verb /
//!   dual-cause.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

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
    let mut cmd = Command::new(BIN);
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

/// Branch exists in `src`'s repo.
fn branch_exists(src: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(src)
        .args(["rev-parse", "--verify", "--quiet", branch])
        .output()
        .expect("spawn git")
        .status
        .success()
}

/// The fork dir is a registered worktree of `src`.
fn worktree_registered(src: &Path, dir: &Path) -> bool {
    git(src, &["worktree", "list"]).contains(&dir.to_string_lossy().into_owned())
}

// --- VT-1: happy path (solo + worker) + pre-add refusals ---

#[test]
fn fork_happy_path_solo_and_worker() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();

    // --- solo: no marker ---
    let solo_dir = holder.path().join("solo");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "wkr-solo",
            "--dir",
            solo_dir.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "solo fork must succeed; stderr: {}",
        stderr(&out)
    );
    // env contract on STDOUT (KEY=value); human status on STDERR.
    assert!(
        stdout(&out).contains("CARGO_TARGET_DIR="),
        "env contract on stdout; got: {}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("wt/wkr-solo"),
        "contract maps target to wt/<branch>; got: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("forked wkr-solo"),
        "human status on stderr; got: {}",
        stderr(&out)
    );
    // The worktree + branch exist AT B.
    assert!(
        worktree_registered(src.path(), &solo_dir),
        "worktree exists"
    );
    assert!(branch_exists(src.path(), "wkr-solo"), "branch exists");
    assert_eq!(
        git(&solo_dir, &["rev-parse", "HEAD"]),
        base,
        "fork branch sits at B"
    );
    // Solo OMITS the marker.
    assert!(!marker_exists(&solo_dir), "solo fork has no marker");

    // --- worker: marker stamped ---
    let wkr_dir = holder.path().join("wkr");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "wkr-worker",
            "--dir",
            wkr_dir.to_str().unwrap(),
            "--worker",
        ],
    );
    assert!(
        out.status.success(),
        "worker fork must succeed; stderr: {}",
        stderr(&out)
    );
    assert!(
        marker_exists(&wkr_dir),
        "worker fork stamps the marker before returning"
    );
    assert_eq!(
        git(&wkr_dir, &["rev-parse", "HEAD"]),
        base,
        "worker fork at B"
    );
}

#[test]
fn fork_pre_add_refusals_leave_no_fork() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();

    // (a) dir already exists.
    let existing = holder.path().join("exists");
    std::fs::create_dir_all(&existing).unwrap();
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "r-dir",
            "--dir",
            existing.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "dir-exists must refuse");
    assert!(stderr(&out).contains("already exists"), "names dir-exists");
    assert!(!branch_exists(src.path(), "r-dir"), "no branch created");

    // (b) branch already exists.
    let _ = add_fork(src.path(), holder.path(), "taken");
    let dir_b = holder.path().join("b");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "taken",
            "--dir",
            dir_b.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "branch-exists must refuse");
    assert!(stderr(&out).contains("branch taken"), "names branch-exists");
    assert!(!dir_b.exists(), "no fork dir created");

    // (c) base is not a commit.
    let dir_c = holder.path().join("c");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            "deadbeefdeadbeef",
            "--branch",
            "r-base",
            "--dir",
            dir_c.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "B-not-a-commit must refuse");
    assert!(stderr(&out).contains("not a commit"), "names bad base");
    assert!(!branch_exists(src.path(), "r-base"), "no branch created");
    assert!(!dir_c.exists(), "no fork dir created");
}

// --- VT-2: compensating cleanup — provision fails after add, fork rolled back ---

#[test]
fn fork_rolls_back_on_provision_failure() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    // A `.worktreeinclude` naming a withheld tier makes `run_provision` bail
    // (allowlist_violations fail-closed) — a deterministic failure AFTER the
    // `git worktree add` step, forcing the compensating rollback.
    std::fs::write(src.path().join(".worktreeinclude"), ".doctrine/state/**\n").unwrap();
    git(src.path(), &["add", ".worktreeinclude"]);
    git(src.path(), &["commit", "-q", "-m", "bad allowlist"]);

    let holder = tempfile::tempdir().unwrap();
    let dir = holder.path().join("rb");
    let head_before = git(src.path(), &["rev-parse", "HEAD"]);
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &head_before,
            "--branch",
            "wkr-rb",
            "--dir",
            dir.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "provision failure must fail the fork; stdout: {}",
        stdout(&out)
    );
    // GONE — not merely unspawned: the worktree + branch + dir are all reversed.
    assert!(
        !worktree_registered(src.path(), &dir),
        "worktree rolled back (gone); worktree list: {}",
        git(src.path(), &["worktree", "list"])
    );
    assert!(
        !branch_exists(src.path(), "wkr-rb"),
        "branch rolled back (gone)"
    );
    assert!(!dir.exists(), "fork dir reaped");
}

// --- VT-4: Orchestrator refusal drives the real CLI ---

#[test]
fn fork_refused_under_worker_mode() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let fork = add_fork(src.path(), holder.path(), "wkr-guard");

    // (1) Marked linked worktree, env unset ⇒ refused (signal: marker), names verb.
    stamp_marker(&fork);
    let target = holder.path().join("nope1");
    let out = run(
        &fork,
        None,
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "child1",
            "--dir",
            target.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "fork refused from a marked linked worktree; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("fork"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused fork creates nothing");

    // (2) DOCTRINE_WORKER set on a NON-linked tree ⇒ dual-cause refusal.
    let target = holder.path().join("nope2");
    let out = run(
        src.path(),
        Some(true),
        &[
            "worktree",
            "fork",
            "--base",
            &base,
            "--branch",
            "child2",
            "--dir",
            target.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "fork refused when DOCTRINE_WORKER set; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "env-on-nonlinked carries the dual-cause; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused fork creates nothing");
}
