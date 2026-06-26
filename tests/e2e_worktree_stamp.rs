// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-10 — `doctrine worktree marker --stamp-subagent` end-to-end over
//! the BUILT binary (design — the claude harness spawn path's mark step). Claude
//! creates the worker's worktree; this verb runs from the matcher-scoped
//! SubagentStart hook to PROVISION + STAMP it. The SubagentStart payload is JSON
//! on STDIN: `{ "cwd": "<worktree path>", "agent_type": "dispatch-worker" }`.
//!
//! * VT-1: a valid payload (real linked worktree cwd + agent_type=dispatch-worker)
//!   provisions AND stamps the marker into the PAYLOAD cwd (not the process cwd),
//!   and prints NO worktree path to stdout.
//! * VT-2: M3 failure posture — a forced provision/mark failure exits non-zero with
//!   a LOUD stderr message AND leaves the worktree in place (no `git worktree
//!   remove`).
//! * VT-3: bad-payload refusals — missing/empty cwd ⇒ `missing-cwd`; cwd not under
//!   the repo / not a linked worktree ⇒ `bad-dir`; missing/non-matching agent_type
//!   ⇒ `missing-agent-type`. Each a distinct non-zero exit.
//! * VT-5: Hookmint refusal — from a MARKED linked fork AND a DOCTRINE_WORKER-set
//!   process, `--stamp-subagent` is refused (worker-mode); against a marker-absent
//!   worktree (worker_mode false, env unset) it runs.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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

/// Make a real linked worktree fork of `src` at `holder/linked` and return it.
fn add_linked_fork(src: &Path, holder: &Path, branch: &str) -> PathBuf {
    let base = git(src, &["rev-parse", "HEAD"]);
    let fork = holder.join("linked");
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

/// Stamp the worker marker directly on disk (test fixture for the refusal arms).
fn stamp_marker(root: &Path) {
    let dir = root.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("worker"), b"").unwrap();
}

fn marker_path(root: &Path) -> PathBuf {
    root.join(".doctrine/state/dispatch/worker")
}

/// Run `doctrine <args>` in `cwd`, feeding `payload` on STDIN. `worker`:
/// Some(true) sets DOCTRINE_WORKER=1; None removes it. CARGO_TARGET_DIR removed so
/// provisioning into the fork is deterministic under the test.
fn run(cwd: &Path, worker: Option<bool>, payload: &str, args: &[&str]) -> Output {
    let mut cmd = Command::new(bin());
    cmd.args(args)
        .current_dir(cwd)
        .env_remove("CARGO_TARGET_DIR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match worker {
        Some(true) => {
            cmd.env("DOCTRINE_WORKER", "1");
        }
        Some(false) | None => {
            cmd.env_remove("DOCTRINE_WORKER");
        }
    }
    let mut child = cmd.spawn().expect("spawn doctrine");
    child
        .stdin
        .take()
        .expect("stdin piped")
        .write_all(payload.as_bytes())
        .expect("write payload");
    child.wait_with_output().expect("wait doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn assert_refusal(out: &Output, token: &str) {
    assert!(
        !out.status.success(),
        "must refuse ({token}); stdout: {}, stderr: {}",
        stdout(out),
        stderr(out)
    );
    assert!(
        stderr(out).contains(token),
        "refusal names `{token}`; stderr: {}",
        stderr(out)
    );
}

const STAMP: &[&str] = &["worktree", "marker", "--stamp-subagent"];

// --- VT-1: a valid payload provisions + stamps the PAYLOAD cwd ---

#[test]
fn stamp_provisions_and_marks_the_payload_worktree() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr");
    let fork = std::fs::canonicalize(&fork).unwrap();
    assert!(!marker_path(&fork).exists(), "no marker before stamp");

    // Run from the SOURCE tree (NOT the fork) — the verb must read cwd from the
    // PAYLOAD, not the process cwd. Payload cwd = the fork.
    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        fork.display()
    );
    let out = run(src.path(), None, &payload, STAMP);
    assert!(
        out.status.success(),
        "valid stamp must succeed; stderr: {}",
        stderr(&out)
    );
    // The marker landed in the PAYLOAD cwd (the fork), not the process cwd (src).
    assert!(
        marker_path(&fork).exists(),
        "marker stamped into the payload worktree"
    );
    assert!(
        !marker_path(src.path()).exists(),
        "marker NOT stamped into the process cwd (read from payload, not cwd)"
    );
    // Stamp emits NOTHING on stdout: unlike `run_fork`, the stamp verb stamps an
    // EXISTING worktree, so it emits no `KEY=value` contract. `run_provision`'s
    // own "provisioned …" report is human status and lands on stderr (ISS-044) —
    // it is the sole copier shared by every consumer (fork/coordinate/stamp), so
    // routing it to stderr keeps each consumer's stdout a pure machine surface.
    assert!(
        stdout(&out).trim().is_empty(),
        "stamp emits nothing on stdout; got: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("provisioned "),
        "run_provision's report lands on stderr (ISS-044); got: {}",
        stderr(&out)
    );
    // The stamp verb's own confirmation lands on stderr, not stdout.
    assert!(
        stderr(&out).contains("stamped worker worktree"),
        "stamp confirmation on stderr; got: {}",
        stderr(&out)
    );
}

// --- VT-1 (SL-125): the Defect-C pin — hook process cwd == the worker worktree ---

#[test]
fn stamp_provisions_from_primary_when_hook_fires_inside_the_worker() {
    // ISS-011 Defect C: the SubagentStart hook fires with PROCESS cwd == the worker
    // worktree (== the fork). The provision SOURCE must resolve to the PRIMARY tree,
    // not the process cwd — else source==fork and `verify_sibling_worktree` bails.
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    // An allowlisted, gitignored UNTRACKED file in the primary tree. `worktree add`
    // checks out tracked files only, so it exists solely in the primary — its arrival
    // in the fork proves the copy SOURCE is the primary worktree, not the fork.
    std::fs::write(src.path().join(".gitignore"), "provisioned.txt\n").unwrap();
    std::fs::write(src.path().join(".worktreeinclude"), "provisioned.txt\n").unwrap();
    std::fs::write(src.path().join("provisioned.txt"), "from primary").unwrap();

    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr");
    let fork = std::fs::canonicalize(&fork).unwrap();
    assert!(!marker_path(&fork).exists(), "no marker before stamp");
    assert!(
        !fork.join("provisioned.txt").exists(),
        "fork lacks the untracked file before stamp"
    );

    // PROCESS cwd = the fork itself (the exact Defect-C condition); payload cwd = fork.
    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        fork.display()
    );
    let out = run(&fork, None, &payload, STAMP);
    assert!(
        out.status.success(),
        "stamp must succeed even when the hook fires inside the worker worktree; stderr: {}",
        stderr(&out)
    );
    assert!(
        marker_path(&fork).exists(),
        "marker stamped into the worker worktree"
    );
    assert_eq!(
        std::fs::read_to_string(fork.join("provisioned.txt")).unwrap(),
        "from primary",
        "the allowlisted untracked file was provisioned FROM the primary worktree"
    );
}

// --- VT-4 (SL-125): cross-repo payload still rejected bad-dir (codex BLOCKER) ---

#[test]
fn stamp_bad_dir_for_payload_worktree_in_a_different_repo() {
    // The binding anchor (root::find on the PROCESS cwd) must still reject a payload
    // cwd that is a real linked worktree of a DIFFERENT repo — proving validation is
    // not self-authenticating from the payload alone. Process cwd binds repo A; the
    // payload names a linked worktree of repo B.
    let repo_a = tempfile::tempdir().unwrap();
    init_repo(repo_a.path());
    let repo_b = tempfile::tempdir().unwrap();
    init_repo(repo_b.path());
    let holder = tempfile::tempdir().unwrap();
    let fork_b = add_linked_fork(repo_b.path(), holder.path(), "wkr");
    let fork_b = std::fs::canonicalize(&fork_b).unwrap();

    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        fork_b.display()
    );
    let out = run(repo_a.path(), None, &payload, STAMP);
    assert_refusal(&out, "bad-dir");
}

// --- VT-2: M3 failure posture — loud fail, worktree LEFT in place ---

#[test]
fn stamp_failure_leaves_the_worktree_in_place() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr");
    let fork = std::fs::canonicalize(&fork).unwrap();

    // Force a write_marker failure: create a FILE where the marker DIR must go, so
    // `create_dir_all(.doctrine/state/dispatch)` errors (a file blocks the dir).
    let state = fork.join(".doctrine/state");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(state.join("dispatch"), b"not a dir").unwrap();

    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        fork.display()
    );
    let out = run(src.path(), None, &payload, STAMP);
    assert!(
        !out.status.success(),
        "a provision/mark failure must exit non-zero; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("STAMP FAILED"),
        "loud stderr diagnostic on failure; got: {}",
        stderr(&out)
    );
    // The property: the worktree is STILL present (no `git worktree remove`).
    assert!(
        fork.exists(),
        "worktree dir LEFT in place after a stamp failure"
    );
    let live = git(src.path(), &["worktree", "list", "--porcelain"]);
    assert!(
        live.contains(&fork.display().to_string()),
        "fork still a registered live worktree; list: {live}"
    );
}

// --- VT-3: bad-payload refusals, each a distinct token ---

#[test]
fn stamp_missing_cwd_refuses() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    // Empty cwd.
    let out = run(
        src.path(),
        None,
        "{\"cwd\": \"\", \"agent_type\": \"dispatch-worker\"}",
        STAMP,
    );
    assert_refusal(&out, "missing-cwd");
    // Absent cwd field.
    let out = run(
        src.path(),
        None,
        "{\"agent_type\": \"dispatch-worker\"}",
        STAMP,
    );
    assert_refusal(&out, "missing-cwd");
    // Malformed JSON folds to missing-cwd.
    let out = run(src.path(), None, "not json at all", STAMP);
    assert_refusal(&out, "missing-cwd");
}

#[test]
fn stamp_bad_dir_refuses_for_non_linked_or_outside_repo() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());

    // cwd = the SOURCE tree (under repo but NOT a linked worktree) ⇒ bad-dir.
    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        std::fs::canonicalize(src.path()).unwrap().display()
    );
    let out = run(src.path(), None, &payload, STAMP);
    assert_refusal(&out, "bad-dir");

    // cwd = a path OUTSIDE the repo (a separate temp tree) ⇒ bad-dir.
    let outside = tempfile::tempdir().unwrap();
    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        outside.path().display()
    );
    let out = run(src.path(), None, &payload, STAMP);
    assert_refusal(&out, "bad-dir");
}

#[test]
fn stamp_missing_agent_type_refuses() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr");
    let fork = std::fs::canonicalize(&fork).unwrap();

    // Absent agent_type, valid cwd ⇒ missing-agent-type.
    let payload = format!("{{\"cwd\": \"{}\"}}", fork.display());
    let out = run(src.path(), None, &payload, STAMP);
    assert_refusal(&out, "missing-agent-type");

    // Present but non-matching agent_type ⇒ missing-agent-type.
    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"some-other-agent\"}}",
        fork.display()
    );
    let out = run(src.path(), None, &payload, STAMP);
    assert_refusal(&out, "missing-agent-type");
}

// --- VT-5: Hookmint worker-mode refusal (marked fork + env), positive when absent ---

#[test]
fn stamp_refused_from_a_marked_linked_worktree() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "marked");
    let fork = std::fs::canonicalize(&fork).unwrap();
    // The PROCESS runs inside a marked linked worktree ⇒ worker_mode true.
    stamp_marker(&fork);

    // A well-formed valid payload still refuses: the guard fires BEFORE the verb.
    let target = add_linked_fork(src.path(), &holder.path().join("h2"), "tgt");
    let target = std::fs::canonicalize(&target).unwrap();
    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        target.display()
    );
    let out = run(&fork, None, &payload, STAMP);
    assert!(
        !out.status.success(),
        "stamp refused from a marked linked worktree; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("marker --stamp-subagent"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );
}

#[test]
fn stamp_refused_under_worker_env() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let target = add_linked_fork(src.path(), holder.path(), "tgt");
    let target = std::fs::canonicalize(&target).unwrap();

    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        target.display()
    );
    // DOCTRINE_WORKER set ⇒ worker_mode true ⇒ refused with the dual-cause.
    let out = run(src.path(), Some(true), &payload, STAMP);
    assert!(
        !out.status.success(),
        "stamp refused when DOCTRINE_WORKER set; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "env refusal carries the dual-cause; stderr: {}",
        stderr(&out)
    );
}

#[test]
fn stamp_refused_when_payload_worktree_already_marked() {
    // F-9: a re-entrant stamp of an ALREADY-marked PAYLOAD worktree must be
    // refused — re-provisioning could overwrite live worker state on a resume.
    // The PROCESS cwd (source) is unmarked, so the worker-mode guard does NOT
    // fire; the new gate is the payload-cwd marker, which only this verb sees
    // (design §5 Hook-mint: only the first stamp, marker-absent, is exempt).
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr");
    let fork = std::fs::canonicalize(&fork).unwrap();
    // The payload worktree already bears the worker marker (a prior stamp).
    stamp_marker(&fork);

    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        fork.display()
    );
    let out = run(src.path(), None, &payload, STAMP);
    assert_refusal(&out, "already-marked");
}

#[test]
fn stamp_runs_against_a_marker_absent_worktree() {
    // The positive control for VT-5: worker_mode false (env unset, target bears no
    // marker) ⇒ the legit first stamp passes the guard and runs.
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr");
    let fork = std::fs::canonicalize(&fork).unwrap();

    let payload = format!(
        "{{\"cwd\": \"{}\", \"agent_type\": \"dispatch-worker\"}}",
        fork.display()
    );
    let out = run(src.path(), None, &payload, STAMP);
    assert!(
        out.status.success(),
        "marker-absent first stamp must pass the guard and run; stderr: {}",
        stderr(&out)
    );
    assert!(marker_path(&fork).exists(), "marker now present");
}
