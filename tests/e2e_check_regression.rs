// SPDX-License-Identifier: GPL-3.0-only
//! SL-170 PHASE-02 VT-7 / VT-9 — `doctrine check regression {capture,diff}` as
//! BLACK-BOX goldens over the BUILT binary against a temp root (`-p/--path`).
//!
//! These cover what an in-process unit cannot: the `std::process::exit` exit-code
//! forwarding of the gate verdict (INV-7), and the orchestrator call path that
//! sources `B` SOLELY from `--base` and binds the run-fingerprint (INV-2 / INV-8).
//!
//! The suite is hermetic (SL-168 F-2): `[verification].regression` is a `cat` of a
//! `suite.txt` we rewrite between capture and diff, so a NEW failure can be
//! injected without a real (slow, flaky) `cargo test`. The binary is resolved at
//! RUNTIME via `common::doctrine_bin()` — never `env!("CARGO_BIN_EXE_…")`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

/// `[verification].regression` runs `cat suite.txt` (cwd == root) — a hermetic,
/// mutable stand-in for `cargo test` per-test output.
const CONFIG: &str = "[verification]\nregression = [\"sh\", \"-c\", \"cat suite.txt\"]\n";

/// A clean cargo run (one pre-existing failure `pre` in target `app`).
const BASELINE: &str = "\
     Running unittests src/main.rs (target/debug/deps/app-aaa111)
running 2 tests
test ok_one ... ok
test pre ... FAILED

failures:

---- pre stdout ----
thread 'pre' panicked at src/x.rs:1:1:
pre-existing boom

failures:
    pre

test result: FAILED. 1 passed; 1 failed; 0 ignored
";

/// The same run with a NEW failure `regressed` injected alongside the pre-existing
/// one — the SL-169 ship-as-env scenario.
const REGRESSED: &str = "\
     Running unittests src/main.rs (target/debug/deps/app-aaa111)
running 3 tests
test ok_one ... ok
test pre ... FAILED
test regressed ... FAILED

failures:

---- pre stdout ----
thread 'pre' panicked at src/x.rs:1:1:
pre-existing boom

---- regressed stdout ----
thread 'regressed' panicked at src/y.rs:2:2:
the slice broke this

failures:
    pre
    regressed

test result: FAILED. 1 passed; 2 failed; 0 ignored
";

/// Fresh temp root carrying the regression config and an initial `suite.txt`.
fn root_with(suite: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = dir.path().join(common::DOCTRINE_TOML);
    std::fs::create_dir_all(cfg.parent().unwrap()).expect("mkdir .doctrine");
    std::fs::write(&cfg, CONFIG).expect("write doctrine.toml");
    std::fs::write(dir.path().join("suite.txt"), suite).expect("write suite.txt");
    dir
}

/// Rewrite `suite.txt` (inject a different suite run between capture and diff).
fn set_suite(root: &Path, suite: &str) {
    std::fs::write(root.join("suite.txt"), suite).expect("rewrite suite.txt");
}

/// Run `doctrine check regression <args…> -p <root>` (optionally with env).
fn run(root: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(common::doctrine_bin());
    cmd.arg("check").arg("regression").args(args);
    cmd.arg("-p").arg(root);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.output().expect("spawn doctrine check regression")
}

/// VT-7 (regression): a failure injected between B and S lands in `new` → the
/// diff exits non-zero. Reconstructs the `e2e_standard_cli_golden` regression that
/// SL-169 shipped as "env".
#[test]
fn injected_failure_lands_in_new_and_halts() {
    let root = root_with(BASELINE);
    let cap = run(root.path(), &["capture", "--base", "B0"], &[]);
    assert!(cap.status.success(), "capture: {cap:?}");

    set_suite(root.path(), REGRESSED);
    let out = run(root.path(), &["diff", "--base", "B0"], &[]);
    assert_eq!(out.status.code(), Some(1), "a new failure halts (INV-7)");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("new:") && stdout.contains("app::regressed"),
        "the injected failure is reported as new, got: {stdout}"
    );
    assert!(
        stdout.contains("persistent") && stdout.contains("app::pre"),
        "the pre-existing failure stays persistent, got: {stdout}"
    );
}

/// VT-7 (green): a pre-existing failure unchanged between B and S → `persistent`,
/// exit zero. The env-artifact that SL-169 misattributed, correctly tolerated.
#[test]
fn unchanged_preexisting_failure_is_persistent_and_green() {
    let root = root_with(BASELINE);
    assert!(
        run(root.path(), &["capture", "--base", "B0"], &[])
            .status
            .success()
    );
    // S == B (suite.txt unchanged): the pre-existing failure is shared.
    let out = run(root.path(), &["diff", "--base", "B0"], &[]);
    assert_eq!(out.status.code(), Some(0), "no new/changed ⇒ exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("no new or changed failures"),
        "got: {stdout}"
    );
}

/// VT-9 (base source): `diff` with no captured baseline for `B` is a hard non-zero
/// — never a silent green ∅. `B` is sourced solely from `--base` (no registry).
#[test]
fn diff_without_baseline_halts_not_silently_green() {
    let root = root_with(BASELINE);
    let out = run(root.path(), &["diff", "--base", "NEVER_CAPTURED"], &[]);
    assert!(
        !out.status.success(),
        "missing baseline halts, got: {out:?}"
    );
}

/// VT-9 (fingerprint): a filter-state change (`DOCTRINE_WORKER=1`) between capture
/// and diff changes the run-fingerprint → the diff cannot find the baseline → a
/// cache miss that halts honestly, never carry-forward poisoning (INV-8).
#[test]
fn changed_filter_state_misses_the_baseline() {
    let root = root_with(BASELINE);
    assert!(
        run(root.path(), &["capture", "--base", "B0"], &[])
            .status
            .success()
    );
    let out = run(
        root.path(),
        &["diff", "--base", "B0"],
        &[("DOCTRINE_WORKER", "1")],
    );
    assert!(
        !out.status.success(),
        "a changed fingerprint must miss the baseline (INV-8), got: {out:?}"
    );
}
