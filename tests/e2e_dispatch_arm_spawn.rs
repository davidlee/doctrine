// SPDX-License-Identifier: GPL-3.0-only
//! SL-152 PHASE-03 — `doctrine dispatch arm-spawn` end-to-end over the BUILT binary
//! (design §5.2/§5.3). The orchestrator-side writer: create the arming dir
//! `<coord>/.doctrine/state/dispatch/spawn/`, write `base` = `"<sha>\n"` (the ONLY
//! thing it carries), print the dir's absolute path on stdout so the orchestrator
//! `cd`s in before the Agent spawn. The reader is `worktree create-fork` (PHASE-02).
//!
//! * VT-1 — exact sha + idempotent: `base` holds exactly `<sha>\n`; re-arm at B'
//!   overwrites it (no append, no dup).
//! * VT-2 — the spawn dir resolves to `<coord>/.doctrine/state/dispatch/spawn` (the
//!   path the reader realpath-matches). The "never provisioned" leg is the withheld
//!   unit assertion in `worktree::allowlist`.
//! * fail-closed (D-P3-1): a base outside create-fork's 4..=64-hex envelope refuses
//!   and writes no base file.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
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

/// A real repo standing in for the coordination tree — `.git` is the root marker.
fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// Run `doctrine dispatch arm-spawn --base <base> -p <root>` (argv-driven; no stdin).
fn arm_spawn(root: &Path, base: &str) -> Output {
    Command::new(BIN)
        .args(["dispatch", "arm-spawn", "--base", base, "-p"])
        .arg(root)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

// --- VT-1 (exact sha) + VT-2 (spawn dir path) + stdout-is-the-dir ---

#[test]
fn arm_spawn_writes_exact_base_and_prints_the_spawn_dir() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();
    let b = git(root.path(), &["rev-parse", "HEAD"]);

    let out = arm_spawn(&root_canon, &b);
    assert!(
        out.status.success(),
        "arm-spawn must succeed; stderr: {}",
        stderr(&out)
    );

    // stdout is exactly the canonical spawn dir, one line.
    let printed = stdout(&out);
    assert_eq!(
        printed.lines().count(),
        1,
        "stdout is one line (the dir); got: {printed:?}"
    );
    let spawn = std::fs::canonicalize(printed.trim()).unwrap();
    assert_eq!(
        spawn,
        root_canon.join(".doctrine/state/dispatch/spawn"),
        "spawn dir is <coord>/.doctrine/state/dispatch/spawn (the reader's realpath)"
    );

    // VT-1: base file holds EXACTLY `<sha>\n`.
    let base_file = root_canon.join(".doctrine/state/dispatch/spawn/base");
    assert_eq!(
        std::fs::read_to_string(&base_file).unwrap(),
        format!("{b}\n"),
        "base file is exactly the sha plus a trailing newline"
    );
}

// --- VT-1 (idempotent): re-arm at B' overwrites base ---

#[test]
fn arm_spawn_is_idempotent_rewriting_base_at_a_new_tip() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();
    let b = git(root.path(), &["rev-parse", "HEAD"]);

    assert!(arm_spawn(&root_canon, &b).status.success(), "first arm");

    // Advance HEAD and re-arm at B'.
    std::fs::write(root.path().join("next.txt"), "more").unwrap();
    git(root.path(), &["add", "next.txt"]);
    git(root.path(), &["commit", "-q", "-m", "advance"]);
    let b2 = git(root.path(), &["rev-parse", "HEAD"]);
    assert_ne!(b, b2, "HEAD advanced");

    assert!(arm_spawn(&root_canon, &b2).status.success(), "re-arm at B'");

    let base_file = root_canon.join(".doctrine/state/dispatch/spawn/base");
    assert_eq!(
        std::fs::read_to_string(&base_file).unwrap(),
        format!("{b2}\n"),
        "re-arming overwrites base to B' (no append, no stale value)"
    );
}

// --- fail-closed (D-P3-1): a base outside the 4..=64-hex envelope refuses ---

#[test]
fn arm_spawn_rejects_a_non_hex_base_and_writes_nothing() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();

    let out = arm_spawn(&root_canon, "not-a-sha");
    assert!(
        !out.status.success(),
        "a malformed base must fail closed; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("bad-base"),
        "refusal names `bad-base`; stderr: {}",
        stderr(&out)
    );
    assert!(
        !root_canon
            .join(".doctrine/state/dispatch/spawn/base")
            .exists(),
        "no base file written on a fail-closed refusal"
    );
}
