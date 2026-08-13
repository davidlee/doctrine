// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-05 — worker-mode guard (ADR-006 D2a / design §3) as BLACK-BOX
//! goldens over the BUILT binary.
//!
//! SL-254 PHASE-05 (`DEC-207`): worker mode is the `DOCTRINE_WORKER` env var ALONE
//! — a property of the PROCESS, not of a tree. The disk marker and the
//! "marker-primary / env-optimisation" two-leg split it anchored are gone, so these
//! goldens no longer discriminate between legs. In their place they pin the
//! stronger property that replaced the split: the SAME refusal fires regardless of
//! tree topology (genuine linked fork, plain repo, bare tempdir alike), and writes
//! are allowed in all of them once the env is unset. The refusal is still a
//! verb-named `bail!` (stderr `Error: <msg>\n`, nonzero exit) carrying the NAMED
//! cause, never a bare "worker refused"; Read paths stay open (INV-3). The unit
//! table (`write_class_tests`) proves the Read/Write split; these prove the gate
//! fires end-to-end.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

// SL-254 PHASE-05: the stable refusal cause (`marker::WORKER_ENV_CAUSE`). Replaces
// SL-056's `DUAL_CAUSE` — that message's "set outside a worker worktree" horn was a
// claim about tree topology, and topology no longer participates in the verdict.
const WORKER_CAUSE: &str =
    "`DOCTRINE_WORKER` is set, so this process is a worker: if that is wrong, unset it";

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
    common::doctrine_cmd(cwd)
        .args(args)
        .env("DOCTRINE_WORKER", "1")
        .output()
        .expect("spawn doctrine")
}

/// `doctrine <args...>` with `DOCTRINE_WORKER` explicitly UNSET, in `cwd`.
fn run_no_env(cwd: &Path, args: &[&str]) -> Output {
    common::doctrine_cmd(cwd)
        .args(args)
        .output()
        .expect("spawn doctrine")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

// VT-5: ≥1 representative Write verb per top-level command + every nested arm.
// Each must refuse under the worker env on a NON-linked tree: nonzero exit, the
// verb named, the bare-`bail!` shape (`Error: …`, no `Caused by:`), AND the named
// cause substance (never a bare `DOCTRINE_WORKER=1: refusing …`).
const WRITE_VERBS: &[(&[&str], &str)] = &[
    (&["install"], "install"),
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
fn write_verbs_refuse_under_worker_env_with_named_cause() {
    // A bare (non-doctrine, non-worktree) cwd: the env is the whole verdict, so the
    // guard trips here exactly as it does in a genuine fork.
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
            err.contains(WORKER_CAUSE),
            "{args:?} refusal must carry the NAMED cause; stderr: {err}"
        );
    }
}

// VT-1(a): a worker PROCESS standing in a genuine linked worktree fork refuses an
// authoring verb AND a status-transition verb, naming the verb and the cause.
//
// SL-254 PHASE-05: was `marker_in_linked_worktree_refuses_writes_env_unset`, which
// provoked the refusal by stamping `.doctrine/state/dispatch/worker` and left the
// env UNSET. The marker leg is gone (`DEC-207`), so the provocation moves to
// `DOCTRINE_WORKER=1` on the child. The linked worktree is kept — not as the signal
// (it no longer is one) but because it is the tree shape a real worker occupies, so
// this remains the in-situ proof rather than a bare-tempdir one. The old
// `!err.contains(DUAL_CAUSE)` assertion, which discriminated the fork refusal from
// the on-main one, retired with the second cause it discriminated against; the
// positive cause assertion below is what replaces it.
#[test]
fn worker_env_in_linked_worktree_refuses_writes() {
    let src = tmp();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);

    // Real linked worktree fork — the shape a dispatched worker actually runs in.
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

    for (args, verb) in [
        (["slice", "new", "x"].as_slice(), "slice new"),
        (
            ["adr", "status", "1", "--status", "accepted"].as_slice(),
            "adr status",
        ),
    ] {
        let out = run_worker(&fork_dir, args);
        let err = stderr(&out);
        assert!(
            !out.status.success(),
            "{args:?} should refuse in a worker process; stderr: {err}"
        );
        assert!(
            err.contains(&format!("`{verb}`")),
            "{args:?} refusal should name the verb `{verb}`; stderr: {err}"
        );
        assert!(
            err.contains(WORKER_CAUSE),
            "{args:?} refusal should carry the NAMED cause; stderr: {err}"
        );
    }
}

// VT-5 (SL-088 PHASE-01): the consolidated `install` verb is a worker-mode write —
// refused wherever a worker process stands. Drives the real write seam (the spawned
// `run()`), and asserts the refusal HAPPENED (nonzero exit + the `install` verb
// named + the named cause).
//
// SL-254 PHASE-05: the old form ran the two legs — marked linked fork vs env-set
// non-linked tree — and asserted a DIFFERENT message from each. One signal now
// answers for both, so the two cases are re-pointed at the property that survived
// their merger and is worth more than either: identity is a property of the
// PROCESS, so topology is irrelevant — the linked fork and the bare tempdir must
// produce the SAME refusal (`DEC-207`).
#[test]
fn install_refuses_in_worker_mode_regardless_of_topology() {
    let src = tmp();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);

    // A real linked worktree — the shape a dispatched worker occupies.
    let fork = tmp();
    let fork_dir = fork.path().join("fork");
    git(
        src.path(),
        &[
            "worktree",
            "add",
            "-b",
            "wkr-ci",
            fork_dir.to_str().unwrap(),
            &base,
        ],
    );

    // A non-linked, non-doctrine tree — a worker dropped anywhere else.
    let plain_cwd = tmp();
    let entry: &[&str] = &["install"];

    let mut refusals = Vec::new();
    for (label, cwd) in [
        ("linked worktree fork", fork_dir.as_path()),
        ("non-linked tempdir", plain_cwd.path()),
    ] {
        let out = run_worker(cwd, entry);
        let err = stderr(&out);
        assert!(
            !out.status.success(),
            "install must refuse in a worker process ({label}); stderr: {err}"
        );
        assert!(
            err.contains("`install`"),
            "refusal names the `install` verb ({label}); stderr: {err}"
        );
        assert!(
            err.contains(WORKER_CAUSE),
            "refusal carries the NAMED cause ({label}); stderr: {err}"
        );
        refusals.push(err);
    }
    assert_eq!(
        refusals[0], refusals[1],
        "topology must not change the refusal: a worker is a worker anywhere (`DEC-207`)"
    );
}

// VT-1(c): a linked worktree with the worker env UNSET (solo) ⇒ writes allowed
// (the verb runs; it is not the worker refusal).
//
// SL-254 PHASE-05: renamed from `linked_worktree_without_marker_allows_writes` —
// what makes the tree writable is now the absent env, not an absent marker.
#[test]
fn linked_worktree_without_worker_env_allows_writes() {
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
    // `slice new` succeeds (env unset).
    let out = run_no_env(&fork_dir, &["slice", "new", "demo"]);
    let err = stderr(&out);
    assert!(
        out.status.success(),
        "no worker env in a linked worktree ⇒ writes allowed; stderr: {err}"
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
