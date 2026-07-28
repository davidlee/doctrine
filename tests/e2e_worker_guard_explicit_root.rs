// SPDX-License-Identifier: GPL-3.0-only
//! ISS-028 / SL-236 — the worker guard's CURRENT, ACTOR-BASED contract, pinned.
//!
//! `worker_guard` resolves its root by walking up from CWD, so the marker answers
//! *"is the process that is running me confined?"* — not *"is the tree being written
//! to protected?"*. That distinction is load-bearing and was, until RV-319 F-2, only
//! implicit: **every tree a worker must not write to is markerless** (the
//! coordination tree, the primary repo), and the only marked tree in a dispatch
//! topology is the worker's own fork. Re-keying the guard to an explicit `-p` target
//! therefore inverts the protection rather than sharpening it — measured, and the
//! reason ISS-028's path-threading fix is not landed here.
//!
//! These tests pin the parts of that contract that hold under ANY resolution of
//! ISS-028, so a future fix cannot regress them silently:
//!
//!   * a marked CWD refuses writes (the primary signal);
//!   * the env leg stays root-independent and keeps its own distinct message;
//!   * a guarded verb that consumes no project root cannot be handed one;
//!   * a Read verb never resolves a root at all (laziness).
//!
//! The two SKEW cases — `-p <markerless>` from a marked CWD, and `-p <marked>` from
//! an unmarked CWD — are deliberately ABSENT. Their correct behaviour is exactly
//! what is unsettled (RV-319 F-2); asserting either direction here would pin a
//! disputed semantic as though it were settled.
//!
//! FIXTURES: every "marked tree" here is a GENUINE linked git worktree carrying the
//! marker, built by `common::marked_linked_fork`, which self-validates both legs. A
//! marker file in a bare tempdir is never refused (`resolve_mode` requires
//! `is_linked && marker_present`), so a tempdir fixture would pass identically
//! whatever the guard did, and prove nothing.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

fn bin() -> PathBuf {
    common::doctrine_bin()
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// `doctrine <args…>` in `cwd` with `DOCTRINE_WORKER` explicitly UNSET — isolates
/// the MARKER leg, which is what root-resolution skew affects.
fn run_no_env(cwd: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .env_remove("DOCTRINE_WORKER")
        .current_dir(cwd)
        .output()
        .expect("spawn doctrine")
}

/// A markerless project root a write can legitimately target.
fn target_root(dir: &Path) -> PathBuf {
    let root = dir.join("target");
    std::fs::create_dir_all(root.join(".doctrine")).expect("create target root");
    root
}

/// The marker-leg refusal substance (`src/worktree/marker.rs`).
const MARKER_SIGNAL: &str = "signal: marker";
/// The env-leg-on-a-non-linked-tree substance — VT-d asserts the legs stay distinct.
const DUAL_CAUSE: &str = "`DOCTRINE_WORKER` set outside a worker worktree";

// VT-c — no `-p` at all, CWD inside a marked fork ⇒ still REFUSED. The regression
// guard: teaching the guard about explicit roots must not blunt the CWD path.
#[test]
fn marked_cwd_still_refuses_write_without_explicit_root() {
    let src = tmp();
    common::init_repo(src.path());
    let fork = tmp();
    let fork_dir = fork.path().join("fork");
    common::marked_linked_fork(src.path(), &fork_dir, "wkr-vtc");

    let out = run_no_env(&fork_dir, &["adr", "new", "vt-c"]);
    let err = stderr(&out);
    assert!(
        !out.status.success(),
        "a marked CWD with no -p must still refuse; stderr: {err}"
    );
    assert!(
        err.contains(MARKER_SIGNAL),
        "refusal must carry the marker signal; stderr: {err}"
    );
}

// VT-d — the ENV leg is root-independent (ADR-006 D2a) and must not be infected by
// making the MARKER leg path-aware: `DOCTRINE_WORKER` set still refuses even when
// `-p` names a perfectly innocent markerless root, and does so with the DUAL-CAUSE
// message rather than the marker-leg one. Asserting only "refused" would let an
// implementation that made the env leg path-aware pass.
#[test]
fn env_leg_stays_root_independent_under_explicit_root() {
    let out_dir = tmp();
    let target = target_root(out_dir.path());
    let cwd = tmp();

    let out = Command::new(bin())
        .args(["adr", "new", "vt-d", "-p", target.to_str().unwrap()])
        .env("DOCTRINE_WORKER", "1")
        .current_dir(cwd.path())
        .output()
        .expect("spawn doctrine");
    let err = stderr(&out);
    assert!(
        !out.status.success(),
        "the env leg must refuse regardless of -p; stderr: {err}"
    );
    assert!(
        err.contains(DUAL_CAUSE),
        "env-leg refusal on a non-linked tree must carry the dual-cause, not the marker leg; stderr: {err}"
    );
    assert!(
        !err.contains(MARKER_SIGNAL),
        "an env-leg refusal must not masquerade as a marker refusal; stderr: {err}"
    );
}

// RV-319 F-1 — a guarded verb that CONSUMES no project root must not ACCEPT one
// either. These four are guarded (Write / Orchestrator / Hookmint) unit variants
// with no `path` field; `worktree create-fork` in particular derives its root from
// the stdin payload cwd and never the process cwd, so a `-p` it accepted but ignored
// would steer the guard away from the tree it actually writes to.
//
// This pins the invariant that keeps that bypass UNREPRESENTABLE: clap rejects the
// flag at parse time. It fails the moment `-p` is made a global argument.
#[test]
fn pathless_guarded_verbs_reject_explicit_root() {
    let cwd = tmp();
    let elsewhere = tmp();
    let p = elsewhere.path().to_str().unwrap();

    for args in [
        ["onboard", "-p", p].as_slice(),
        ["worktree", "create-fork", "-p", p].as_slice(),
        ["worktree", "nominate", "-p", p].as_slice(),
        ["worktree", "denominate", "-p", p].as_slice(),
    ] {
        let out = run_no_env(cwd.path(), args);
        let err = stderr(&out);
        assert!(
            !out.status.success(),
            "{args:?} declares no project root, so -p must be rejected; stderr: {err}"
        );
        assert!(
            err.contains("unexpected argument '-p'"),
            "{args:?} must be refused by the PARSER (a guarded verb that consumes no \
             root must not accept one — RV-319 F-1); stderr: {err}"
        );
    }
}

// VT-e — guard laziness (design §3): a Read verb never resolves a root, so a Read in
// a rootless CWD gains no new failure path from the guard. Asserted on the guard's
// own substance rather than the exit code, which a rootless Read may legitimately
// set for its own reasons.
#[test]
fn read_verb_in_rootless_cwd_never_trips_the_guard() {
    let base = common::marker_free_base();
    let cwd = tempfile::tempdir_in(base).expect("marker-free tempdir");

    let out = run_no_env(cwd.path(), &["adr", "list"]);
    let err = stderr(&out);
    assert!(
        !err.contains("refusing authored write"),
        "a Read verb must never trip the worker guard; stderr: {err}"
    );
    assert!(
        !err.contains(MARKER_SIGNAL),
        "a Read verb must not resolve a root at all (laziness); stderr: {err}"
    );
}
