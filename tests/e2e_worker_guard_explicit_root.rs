// SPDX-License-Identifier: GPL-3.0-only
//! ISS-028 / SL-236 — the worker guard's CURRENT, ACTOR-BASED contract, pinned.
//!
//! The guard answers *"is the process that is running me confined?"* — not *"is the
//! tree being written to protected?"*. That distinction was load-bearing and only
//! implicit until RV-319 F-2.
//!
//! SL-254 PHASE-05 (`DEC-207`) settles ISS-028 at the root: worker identity is the
//! `DOCTRINE_WORKER` env var ALONE, so the guard consults NO tree at all and the
//! actor-vs-target skew that RV-319 F-2 left unsettled is now unrepresentable.
//! What these tests pin is the resulting contract:
//!
//!   * a worker process refuses writes with no `-p` at all (the CWD path);
//!   * that refusal is root-INDEPENDENT — an innocent `-p` target cannot buy it off;
//!   * a guarded verb that consumes no project root cannot be handed one;
//!   * a Read verb never resolves a root at all (laziness).
//!
//! FIXTURES: the fork here is a GENUINE linked git worktree, built by
//! `common::linked_fork`, which self-validates the topology. Topology no longer
//! decides the verdict, but a real fork is still the shape a real worker occupies,
//! so the in-situ proof is kept rather than degraded to a bare tempdir.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::Output;

mod common;

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// `doctrine <args…>` in `cwd` with `DOCTRINE_WORKER` explicitly UNSET (the
/// `doctrine_cmd` default) — a NON-worker process.
fn run_no_env(cwd: &Path, args: &[&str]) -> Output {
    common::doctrine_cmd(cwd)
        .args(args)
        .output()
        .expect("spawn doctrine")
}

/// `doctrine <args…>` in `cwd` as a WORKER process.
fn run_worker(cwd: &Path, args: &[&str]) -> Output {
    common::doctrine_cmd(cwd)
        .args(args)
        .env("DOCTRINE_WORKER", "1")
        .output()
        .expect("spawn doctrine")
}

/// A markerless project root a write can legitimately target.
fn target_root(dir: &Path) -> PathBuf {
    let root = dir.join("target");
    std::fs::create_dir_all(root.join(".doctrine")).expect("create target root");
    root
}

/// The refusal substance (`src/worktree/marker.rs` `WORKER_ENV_CAUSE`).
///
/// SL-254 PHASE-05: replaces the pair of constants this file carried — `MARKER_SIGNAL`
/// (`"signal: marker"`) and `DUAL_CAUSE`. With one signal there is one message, so
/// the "the legs must stay distinguishable" assertions below collapse into asserting
/// this exact substance.
const WORKER_CAUSE: &str =
    "`DOCTRINE_WORKER` is set, so this process is a worker: if that is wrong, unset it";

// VT-c — no `-p` at all, a worker process CWD'd inside a genuine fork ⇒ REFUSED.
// The regression guard: teaching the guard about explicit roots must not blunt the
// no-`-p` path.
//
// SL-254 PHASE-05: was `marked_cwd_still_refuses_write_without_explicit_root`, which
// provoked the refusal by stamping the fork. The fork fixture stays (it is the tree
// shape a worker occupies); the provocation moves to the env.
#[test]
fn worker_process_still_refuses_write_without_explicit_root() {
    let src = tmp();
    common::init_repo(src.path());
    let fork = tmp();
    let fork_dir = fork.path().join("fork");
    common::linked_fork(src.path(), &fork_dir, "wkr-vtc");

    let out = run_worker(&fork_dir, &["adr", "new", "vt-c"]);
    let err = stderr(&out);
    assert!(
        !out.status.success(),
        "a worker process with no -p must still refuse; stderr: {err}"
    );
    assert!(
        err.contains(WORKER_CAUSE),
        "refusal must carry the named cause; stderr: {err}"
    );
}

// VT-d — the verdict is root-INDEPENDENT (ADR-006 D2a, hardened by `DEC-207`):
// `DOCTRINE_WORKER` set still refuses even when `-p` names a perfectly innocent
// third-party root, from a CWD that is not a fork at all. Asserting only "refused"
// would let a path-aware implementation pass, so the exact cause is asserted too.
//
// SL-254 PHASE-05: the old form additionally asserted `!err.contains(MARKER_SIGNAL)`
// — "an env-leg refusal must not masquerade as a marker refusal". There is no second
// leg left to masquerade as; the positive `WORKER_CAUSE` assertion is now the whole
// discrimination.
#[test]
fn refusal_stays_root_independent_under_explicit_root() {
    let out_dir = tmp();
    let target = target_root(out_dir.path());
    let cwd = tmp();

    let out = run_worker(
        cwd.path(),
        &["adr", "new", "vt-d", "-p", target.to_str().unwrap()],
    );
    let err = stderr(&out);
    assert!(
        !out.status.success(),
        "a worker process must refuse regardless of -p; stderr: {err}"
    );
    assert!(
        err.contains(WORKER_CAUSE),
        "refusal must carry the named cause; stderr: {err}"
    );
}

// RV-319 F-1 — a guarded verb that CONSUMES no project root must not ACCEPT one
// either. These are guarded (Write / Orchestrator) unit variants
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
        !err.contains(WORKER_CAUSE),
        "a Read verb must never reach the worker verdict at all (laziness); stderr: {err}"
    );
}
