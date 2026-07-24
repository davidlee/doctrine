// SPDX-License-Identifier: GPL-3.0-only
//! SL-225 PHASE-01 — the `just validate` worker-context skip (fix #1) and the
//! close-gate freshness belt (ii).
//!
//! These drive the REAL `validate` recipe via
//! `just --justfile <root>/justfile --working-directory <fixture>` — never a
//! copy of the bash — so the proof is of the shipped recipe itself. The fixture
//! is an empty temp dir: off the worker signal the recipe shells `doctrine
//! prompt check`, which reds ("No project root found") — a faithful
//! broken-authored-state proxy for the ISS-218 stale-binary red (DEC-003). The
//! green-vs-red pivot on the signal IS the skip.

mod common;

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

/// The worker-marker file, relative to a repo root (mirrors marker.rs:114).
const WORKER_MARKER_REL: &str = ".doctrine/state/dispatch/worker";

/// True iff `just` is on PATH — these e2e proofs drive the real task runner.
fn just_available() -> bool {
    Command::new("just")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run the real `validate` recipe with cwd = `fixture`, applying `envs`. The
/// inherited worker signals are stripped first so each case controls its own.
fn run_validate(fixture: &Path, envs: &[(&str, &str)]) -> Output {
    let justfile = common::repo_root().join("justfile");
    let mut cmd = Command::new("just");
    cmd.arg("--justfile")
        .arg(&justfile)
        .arg("--working-directory")
        .arg(fixture)
        .arg("validate");
    cmd.env_remove("DOCTRINE_DISPATCH_GATE")
        .env_remove("DOCTRINE_WORKER");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("spawn just validate")
}

/// VT-1 — discriminating skip: a broken-authored fixture that `doctrine doctor` /
/// `prompt check` would RED drives `validate` GREEN under `DOCTRINE_DISPATCH_GATE`,
/// because the governance legs are skipped in a worker context.
#[test]
fn dispatch_gate_signal_skips_governance_legs() {
    if !just_available() {
        eprintln!("skipping: `just` not on PATH");
        return;
    }
    let fixture = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");
    let out = run_validate(fixture.path(), &[("DOCTRINE_DISPATCH_GATE", "1")]);
    assert!(
        out.status.success(),
        "validate must SKIP (exit 0) under DOCTRINE_DISPATCH_GATE on a broken fixture; got {:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

/// VT-1b — generic-host no-mask (safety negative): with NO worker signal, the
/// same broken fixture still REDS. The skip never masks a real governance
/// regression on the main arm.
#[test]
fn no_signal_runs_governance_legs_and_reds_on_broken_state() {
    if !just_available() {
        eprintln!("skipping: `just` not on PATH");
        return;
    }
    let fixture = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");
    let out = run_validate(fixture.path(), &[]);
    assert!(
        !out.status.success(),
        "validate must RUN the governance legs (nonzero) with no worker signal on a broken fixture; got exit 0\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

/// VT-1c — the three signal legs skip independently, and the exact-match negative
/// (`DOCTRINE_WORKER=0`) does NOT skip (mirrors env_worker_set, marker.rs:127).
#[test]
fn each_signal_leg_skips_and_worker_zero_does_not() {
    if !just_available() {
        eprintln!("skipping: `just` not on PATH");
        return;
    }

    // Leg 1: DOCTRINE_DISPATCH_GATE set.
    let f1 = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");
    assert!(
        run_validate(f1.path(), &[("DOCTRINE_DISPATCH_GATE", "1")])
            .status
            .success(),
        "DOCTRINE_DISPATCH_GATE leg must skip",
    );

    // Leg 2: DOCTRINE_WORKER=1 (exact).
    let f2 = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");
    assert!(
        run_validate(f2.path(), &[("DOCTRINE_WORKER", "1")])
            .status
            .success(),
        "DOCTRINE_WORKER=1 leg must skip",
    );

    // Leg 3: the marker file at the fixture root.
    let f3 = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");
    let marker = f3.path().join(WORKER_MARKER_REL);
    std::fs::create_dir_all(marker.parent().expect("marker parent")).expect("mkdir marker dir");
    std::fs::write(&marker, b"").expect("write marker");
    assert!(
        run_validate(f3.path(), &[]).status.success(),
        "marker-file leg must skip",
    );

    // Exact-match negative: DOCTRINE_WORKER=0 must NOT skip → reds on broken fixture.
    let f4 = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");
    assert!(
        !run_validate(f4.path(), &[("DOCTRINE_WORKER", "0")])
            .status
            .success(),
        "DOCTRINE_WORKER=0 must NOT skip (exact-match) — it must run and red",
    );
}

/// Write an executable stub binary that exits `code` regardless of its args, so
/// `validate`'s resolved `"$doc" prompt check && "$doc" doctor` inherit that code.
fn stub_binary(dir: &Path, code: u8) -> std::path::PathBuf {
    let path = dir.join("stub-doctrine");
    let mut f = std::fs::File::create(&path).expect("create stub");
    write!(f, "#!/usr/bin/env bash\nexit {code}\n").expect("write stub");
    drop(f);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod stub");
    path
}

/// VT-1d (resolution leg) — off the worker signal, `validate` runs the binary it
/// RESOLVES (`${DOCTRINE_BIN:-./target/debug/doctrine}`), not bare `doctrine`: a
/// `DOCTRINE_BIN` stub that exits 1 reds the recipe; one that exits 0 greens it.
#[test]
fn off_signal_runs_the_resolved_binary() {
    if !just_available() {
        eprintln!("skipping: `just` not on PATH");
        return;
    }
    let home = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");
    let fixture = tempfile::tempdir_in(common::marker_free_base()).expect("tempdir");

    let red = stub_binary(home.path(), 1);
    assert!(
        !run_validate(
            fixture.path(),
            &[("DOCTRINE_BIN", red.to_str().expect("utf8"))]
        )
        .status
        .success(),
        "DOCTRINE_BIN → an exit-1 stub must red `validate` (it runs the resolved binary)",
    );

    let green = stub_binary(home.path(), 0);
    assert!(
        run_validate(
            fixture.path(),
            &[("DOCTRINE_BIN", green.to_str().expect("utf8"))]
        )
        .status
        .success(),
        "DOCTRINE_BIN → an exit-0 stub must green `validate`",
    );
}

/// VT-1d (belt-order leg) — `check` and `gate` run `build` BEFORE `validate`, so
/// `doctrine check gate` (close) validates the landed corpus with a
/// this-invocation-fresh binary. Proven light via `just -n` dry-run order.
#[test]
fn check_and_gate_build_before_validate() {
    if !just_available() {
        eprintln!("skipping: `just` not on PATH");
        return;
    }
    for recipe in ["check", "gate"] {
        let out = Command::new("just")
            .current_dir(common::repo_root())
            .args(["-n", recipe])
            .output()
            .expect("spawn just -n");
        // `just -n` prints its command trace to stderr.
        let plan = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        let build_at = plan
            .find("cargo build")
            .unwrap_or_else(|| panic!("`{recipe}` dry-run must invoke `cargo build`:\n{plan}"));
        let validate_at = plan
            .find("prompt check")
            .unwrap_or_else(|| panic!("`{recipe}` dry-run must invoke `validate`:\n{plan}"));
        assert!(
            build_at < validate_at,
            "`{recipe}` must run `build` before `validate` (fresh binary for the close gate):\n{plan}",
        );
    }
}
