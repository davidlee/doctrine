// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-06 — `doctrine worktree fork --base <B> --branch <name> --dir
//! <path> [--worker]` end-to-end over the BUILT binary.
//!
//! * VT-1: happy path — human status on stderr, machine-clean (empty) stdout, the
//!   worktree+branch exist at `<B>`, and the status line DECLARES `--worker` (and
//!   does not, solo); the three pre-`add` refusals (dir-exists / branch-exists /
//!   B-not-a-commit) each exit non-zero and leave NO fork.
//! * VT-2: compensating cleanup — a provision failure after `git worktree add`
//!   exits non-zero AND leaves NO leftover worktree/branch (asserted GONE).
//! * VT-4: `fork` Orchestrator refusal drives the real CLI — refused in a worker
//!   PROCESS whatever tree it stands in, naming the verb and the cause.
//!
//! SL-254 PHASE-05 (`DEC-207`): `--worker` no longer stamps
//! `.doctrine/state/dispatch/worker` — nothing does; worker identity is the
//! `DOCTRINE_WORKER` env var of the process. The flag survives as the fork's
//! declaration of intent (it drives the durable `(slice, phase)` binding and the
//! status line), so VT-1's marker-present/marker-absent pair becomes a
//! declaration-present/absent pair on the observable status line.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

/// SL-254 PHASE-05: the one refusal cause (`marker::WORKER_ENV_CAUSE`).
const WORKER_CAUSE: &str =
    "`DOCTRINE_WORKER` is set, so this process is a worker: if that is wrong, unset it";

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

/// Run `doctrine <args>` in `cwd`; env governed by `worker` (Some(true) sets
/// DOCTRINE_WORKER=1; None removes it).
fn run(cwd: &Path, worker: Option<bool>, args: &[&str]) -> Output {
    let mut cmd = common::doctrine_cmd(cwd);
    cmd.args(args);
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

    // --- solo: no worker declaration ---
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
    // Human status on STDERR; stdout stays empty (SL-156 — the fork builds into its
    // own in-tree `<dir>/target`, no env contract is emitted).
    assert!(
        stderr(&out).contains("forked wkr-solo"),
        "human status on stderr; got: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).trim().is_empty(),
        "fork stdout is machine-clean (empty); got: {}",
        stdout(&out)
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
    // SL-254 PHASE-05: was `!marker_exists(&solo_dir)` — "solo fork has no marker".
    // Nothing stamps a marker any more, so that assertion would be vacuously true
    // whatever `fork` did. The observable that still discriminates solo from
    // `--worker` is the status line, so it is asserted in both directions here and
    // below.
    assert!(
        !stderr(&out).contains("(worker)"),
        "solo fork does NOT declare itself a worker fork; stderr: {}",
        stderr(&out)
    );

    // --- worker: declared on the status line ---
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
        stderr(&out).contains("(worker)"),
        "worker fork DECLARES itself on the status line; stderr: {}",
        stderr(&out)
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

    // SL-254 PHASE-05: the two cases below were "marked linked worktree, env unset"
    // and "DOCTRINE_WORKER on a non-linked tree", asserting two DIFFERENT messages.
    // One signal now answers for both, so they are re-pointed at what survived their
    // merger: the same worker process is refused in a linked fork and on the primary
    // tree alike, both naming the verb and the cause (`DEC-207`).

    // (1) A worker process standing in a linked worktree ⇒ refused, names verb.
    let target = holder.path().join("nope1");
    let out = run(
        &fork,
        Some(true),
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
        "fork refused in a worker process inside a linked worktree; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("`fork`"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains(WORKER_CAUSE),
        "refusal carries the NAMED cause; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused fork creates nothing");

    // (2) The SAME worker process on the primary (non-linked) tree ⇒ same refusal.
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
        stderr(&out).contains("`fork`"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains(WORKER_CAUSE),
        "topology does not change the cause; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused fork creates nothing");
}
