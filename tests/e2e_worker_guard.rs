// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-05 — worker-mode guard (ADR-006 D2a / design §3) as BLACK-BOX
//! goldens over the BUILT binary.
//!
//! Worker mode is now MARKER-PRIMARY: a disk marker in a LINKED worktree refuses
//! writes harness-agnostically; the `DOCTRINE_WORKER` env is the codex/pi
//! worker-on-main OPTIMISATION (the catch for a worker dropped on the coordination
//! root). Both legs hard-refuse every authored/memory/runtime write BEFORE
//! dispatch with a verb-named `bail!` (stderr `Error: <msg>\n`, nonzero exit);
//! Read paths stay open (INV-3). The unit table (`write_class_tests`) proves the
//! full Read/Write/MarkerClear split; these tests prove the gate fires end-to-end:
//!   * VT-1: marker-in-a-linked-worktree refuses (env UNSET); solo / non-worktree
//!     allow.
//!   * VT-5: the env leg on a NON-linked tree carries the NAMED dual-cause message
//!     (never a bare "worker refused"), still names the verb, still bare-`Error:`
//!     shape with no `Caused by:` chain.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

// The stable dual-cause tokens (design §3). Goldens assert this substance.
const DUAL_CAUSE: &str = "`DOCTRINE_WORKER` set outside a worker worktree";

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
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

/// A doctrine-rooted git repo with one commit. `.git` + `.doctrine` make it a
/// project root that `root::find` resolves.
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

/// `doctrine <args...>` under `DOCTRINE_WORKER=1`, rooted in a throwaway cwd so a
/// (never-reached) write could not touch the repo.
fn run_worker(cwd: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .env("DOCTRINE_WORKER", "1")
        .current_dir(cwd)
        .output()
        .expect("spawn doctrine")
}

/// `doctrine <args...>` with `DOCTRINE_WORKER` explicitly UNSET, in `cwd`.
fn run_no_env(cwd: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .env_remove("DOCTRINE_WORKER")
        .current_dir(cwd)
        .output()
        .expect("spawn doctrine")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

// VT-5: ≥1 representative Write verb per top-level command + every nested arm.
// Each must refuse under the worker env on a NON-linked tree: nonzero exit, the
// verb named, the bare-`bail!` shape (`Error: …`, no `Caused by:`), AND the named
// dual-cause substance (the migration from the old `DOCTRINE_WORKER=1: refusing …`
// message).
const WRITE_VERBS: &[(&[&str], &str)] = &[
    (&["install"], "install"),
    (&["skills", "install"], "skills install"),
    (&["slice", "new", "x"], "slice new"),
    (
        &["memory", "record", "t", "--type", "concept"],
        "memory record",
    ),
    (&["memory", "sync"], "memory sync"),
    (&["memory", "sync", "install"], "memory sync install"),
    (&["adr", "new", "t"], "adr new"),
    (
        &["adr", "status", "1", "--status", "accepted"],
        "adr status",
    ),
    (&["policy", "new", "t"], "policy new"),
    (&["standard", "new", "t"], "standard new"),
    (
        &["standard", "status", "1", "--status", "required"],
        "standard status",
    ),
    (&["spec", "new", "product", "t"], "spec new"),
    (
        &["spec", "req", "add", "PRD-001", "--kind", "functional"],
        "spec req add",
    ),
    (&["backlog", "new", "issue", "t"], "backlog new"),
    (
        &["backlog", "edit", "ISS-001", "--status", "open"],
        "backlog edit",
    ),
    (&["boot"], "boot"),
    (&["boot", "install"], "boot install"),
    (&["reseat", "SL-001"], "reseat"),
];

#[test]
fn write_verbs_refuse_under_worker_env_with_dual_cause() {
    // A bare (non-doctrine, non-worktree) cwd: the env leg trips on a NON-linked
    // tree ⇒ the dual-cause message.
    let dir = tmp();
    for (args, verb) in WRITE_VERBS {
        let out = run_worker(dir.path(), args);
        let err = stderr(&out);
        assert!(
            !out.status.success(),
            "{args:?} should refuse (nonzero exit); stderr: {err}"
        );
        assert!(
            err.starts_with("Error: "),
            "{args:?} should bail with the bare `Error: ` shape; stderr: {err}"
        );
        assert!(
            !err.contains("Caused by"),
            "{args:?} bail should have no `Caused by` context chain; stderr: {err}"
        );
        assert!(
            err.contains(&format!("`{verb}`")),
            "{args:?} refusal should name the verb `{verb}`; stderr: {err}"
        );
        assert!(
            err.contains(DUAL_CAUSE),
            "{args:?} env-leg refusal on a non-linked tree must carry the named dual-cause; stderr: {err}"
        );
    }
}

// VT-1(a): the PRIMARY signal — a marker in a LINKED worktree with the env UNSET
// refuses an authoring verb AND a status-transition verb, naming the verb. NOT the
// dual-cause (it is a genuine fork).
#[test]
fn marker_in_linked_worktree_refuses_writes_env_unset() {
    let src = tmp();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);

    // Real linked worktree fork.
    let fork = tmp();
    let fork_dir = fork.path().join("fork");
    git(
        src.path(),
        &[
            "worktree",
            "add",
            "-b",
            "wkr",
            fork_dir.to_str().unwrap(),
            &base,
        ],
    );

    // Stamp the marker (orchestrator's job; we write the file directly).
    let marker_dir = fork_dir.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&marker_dir).unwrap();
    std::fs::write(marker_dir.join("worker"), b"").unwrap();

    for (args, verb) in [
        (["slice", "new", "x"].as_slice(), "slice new"),
        (
            ["adr", "status", "1", "--status", "accepted"].as_slice(),
            "adr status",
        ),
    ] {
        let out = run_no_env(&fork_dir, args);
        let err = stderr(&out);
        assert!(
            !out.status.success(),
            "{args:?} should refuse via the marker (PRIMARY signal); stderr: {err}"
        );
        assert!(
            err.contains(&format!("`{verb}`")),
            "{args:?} refusal should name the verb `{verb}`; stderr: {err}"
        );
        assert!(
            err.contains("signal: marker"),
            "{args:?} should refuse with signal: marker; stderr: {err}"
        );
        assert!(
            !err.contains(DUAL_CAUSE),
            "{args:?} marker refusal is a genuine fork, NOT the dual-cause; stderr: {err}"
        );
    }
}

// VT-1(c): a linked worktree WITHOUT a marker, no env (solo) ⇒ writes allowed
// (the verb runs; it is not the worker refusal).
#[test]
fn linked_worktree_without_marker_allows_writes() {
    let src = tmp();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let fork = tmp();
    let fork_dir = fork.path().join("fork");
    git(
        src.path(),
        &[
            "worktree",
            "add",
            "-b",
            "wkr2",
            fork_dir.to_str().unwrap(),
            &base,
        ],
    );

    // `slice list` is a Read; use a write verb to prove the guard does NOT fire:
    // `slice new` succeeds (no marker, no env).
    let out = run_no_env(&fork_dir, &["slice", "new", "demo"]);
    let err = stderr(&out);
    assert!(
        out.status.success(),
        "no marker + no env in a linked worktree ⇒ writes allowed; stderr: {err}"
    );
    assert!(
        !err.contains("refusing"),
        "must not hit the worker guard; stderr: {err}"
    );
}

// VT-1(d): a non-worktree tempdir, no env ⇒ writes allowed (the verb runs).
#[test]
fn non_worktree_without_env_allows_writes() {
    let dir = tmp();
    init_repo(dir.path());
    let out = run_no_env(dir.path(), &["slice", "new", "demo"]);
    let err = stderr(&out);
    assert!(
        out.status.success(),
        "non-worktree + no env ⇒ writes allowed; stderr: {err}"
    );
    assert!(
        !err.contains("refusing"),
        "must not hit the worker guard; stderr: {err}"
    );
}

// VT-3 (INV-3): a Read verb runs unaffected under the worker env — `slice list`
// on an empty tree exits 0 and is NOT the refusal.
#[test]
fn read_verb_unaffected_under_worker() {
    let dir = tmp();
    let out = run_worker(dir.path(), &["slice", "list", "-p", "."]);
    let err = stderr(&out);
    assert!(
        out.status.success(),
        "`slice list` should run under DOCTRINE_WORKER=1; stderr: {err}"
    );
    assert!(
        !err.contains("DOCTRINE_WORKER") && !err.contains("refusing"),
        "`slice list` must not hit the worker guard; stderr: {err}"
    );
}

// VT-3 (INV-3): the corpus integrity scan is a Read verb — it must run, not
// refuse, under the worker env (an empty tree validates clean).
#[test]
fn validate_unaffected_under_worker() {
    let dir = tmp();
    let out = run_worker(dir.path(), &["validate", "-p", "."]);
    let err = stderr(&out);
    assert!(
        out.status.success(),
        "`validate` should run under DOCTRINE_WORKER=1; stderr: {err}"
    );
    assert!(
        !err.contains("DOCTRINE_WORKER") && !err.contains("refusing"),
        "`validate` must not hit the worker guard; stderr: {err}"
    );
}
