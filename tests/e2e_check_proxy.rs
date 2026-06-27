// SPDX-License-Identifier: GPL-3.0-only
//! SL-163 PHASE-01 VT-3 — `doctrine check {quick|gate}` proxy as BLACK-BOX goldens
//! over the BUILT binary against a temp root (via `-p/--path`).
//!
//! These cover what an in-process unit cannot: the `std::process::exit` exit-code
//! forwarding (the `e2e_*` precedent). The binary is resolved at RUNTIME via
//! `common::doctrine_bin()` (current_exe sibling) — never `env!("CARGO_BIN_EXE_…")`
//! (the `e2e_no_baked_paths` / SL-162 guard).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

/// The owned no-op note (mirrors `verify::QUICK_UNSET_NOTE`, not importable from a
/// binary-only crate).
const QUICK_UNSET_NOTE: &str = "doctrine check quick: no [verification].quick set — skipping";

/// Write `.doctrine/doctrine.toml` with `body` under a fresh temp root.
fn root_with(body: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = dir.path().join(common::DOCTRINE_TOML);
    std::fs::create_dir_all(cfg.parent().unwrap()).expect("mkdir .doctrine");
    std::fs::write(&cfg, body).expect("write doctrine.toml");
    dir
}

/// Run `doctrine check <args…> -p <root>` against the built binary.
fn run_check(root: &Path, args: &[&str]) -> Output {
    Command::new(common::doctrine_bin())
        .arg("check")
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine check")
}

/// (i) A configured `gate` forwards the child's exit code verbatim AND streams its
/// output (the child owns the inherited stdout, captured transitively here).
#[test]
fn gate_forwards_child_exit_code_and_streams_output() {
    let root =
        root_with("[verification]\ngate = [\"sh\", \"-c\", \"echo streamed-marker; exit 7\"]\n");
    let out = run_check(root.path(), &["gate"]);
    assert_eq!(
        out.status.code(),
        Some(7),
        "child exit 7 forwarded verbatim"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("streamed-marker"),
        "child stdout streamed through inherited fds, got: {stdout:?}"
    );
}

/// (ii) A missing program errors toward the OWNED key, not a raw spawn error.
#[test]
fn missing_program_errors_naming_the_key() {
    let root = root_with("[verification]\ngate = [\"doctrine-no-such-binary-xyz\"]\n");
    let out = run_check(root.path(), &["gate"]);
    assert!(!out.status.success(), "missing program is a failure");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("[verification].gate"),
        "error names the owned config key, got: {stderr:?}"
    );
}

/// (iii) A signal-killed child re-exits `128 + signo` (CR-F5), not a flattened 1.
#[test]
fn signal_killed_child_exits_128_plus_signo() {
    let root = root_with("[verification]\ngate = [\"sh\", \"-c\", \"kill -TERM $$\"]\n");
    let out = run_check(root.path(), &["gate"]);
    assert_eq!(
        out.status.code(),
        Some(143),
        "SIGTERM (15) ⇒ 128 + 15 = 143"
    );
}

/// (iv) Unconfigured `quick` is an OWNED no-op: exit 0, the note on stdout, NO
/// child spawned (CR-F3). A `[verification]` with no `quick` key exercises the
/// `None` → `Noop` arm.
#[test]
fn unconfigured_quick_is_owned_noop() {
    let root = root_with("[verification]\ngate = [\"just\", \"gate\"]\n");
    let out = run_check(root.path(), &["quick"]);
    assert_eq!(out.status.code(), Some(0), "owned no-op exits 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(QUICK_UNSET_NOTE),
        "the owned note prints, got: {stdout:?}"
    );
}
