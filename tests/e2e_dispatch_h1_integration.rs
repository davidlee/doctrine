// SPDX-License-Identifier: GPL-3.0-only
//! SL-152 PHASE-04 VT-4 — the headline H1 path chained END-TO-END at the verb
//! boundary over the BUILT binary: `dispatch arm-spawn` (the orchestrator writer,
//! PHASE-03) writes the arming `base`, then `worktree create-fork` (the hook verb,
//! PHASE-02) consumes that SAME file and forks at base B — even after `main` has
//! moved past B. This proves the file contract holds across the two verbs and that
//! H1 (wrong-base fallback) is structurally dead: the fork lands at B, never at the
//! moved HEAD.
//!
//! PHASE-02's `fork_pins_base...` proves drift-immunity with the base file written
//! directly; this test instead lets `arm-spawn` author it, so it is the integration
//! seam (arm-spawn output == create-fork input), not a re-test of the core.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::io::Write;
use std::path::Path;
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
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// `doctrine dispatch arm-spawn --base <base> -p <root>` (argv-driven; no stdin).
fn arm_spawn(root: &Path, base: &str) -> Output {
    Command::new(bin())
        .args(["dispatch", "arm-spawn", "--base", base, "-p"])
        .arg(root)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

/// `doctrine worktree verify-worker --base <B> --dir <worktree> --branch <S>` —
/// the post-spawn base==B belt the funnel runs against a returned worker.
fn verify_worker(base: &str, dir: &Path, branch: &str) -> Output {
    Command::new(bin())
        .args([
            "worktree",
            "verify-worker",
            "--base",
            base,
            "--branch",
            branch,
            "--dir",
        ])
        .arg(dir)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

/// `doctrine worktree create-fork` with `{cwd,name}` on STDIN, process cwd = `cwd`.
fn create_fork(cwd: &Path, payload: &str) -> Output {
    let mut child = Command::new(bin())
        .args(["worktree", "create-fork"])
        .current_dir(cwd)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("DOCTRINE_WORKER")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn doctrine");
    child
        .stdin
        .take()
        .expect("stdin piped")
        .write_all(payload.as_bytes())
        .expect("write payload");
    child.wait_with_output().expect("wait doctrine")
}

// --- VT-4: arm-spawn → create-fork lands at base B under a moving main ---

#[test]
fn arm_spawn_then_create_fork_lands_at_base_b_under_moving_main() {
    let root = tempfile::tempdir().unwrap();
    init_repo(root.path());
    let root_canon = std::fs::canonicalize(root.path()).unwrap();

    // B = the tip the orchestrator arms at (dispatch setup's `base=`).
    let b = git(root.path(), &["rev-parse", "HEAD"]);

    // arm-spawn authors the base file and prints the spawn dir (the payload cwd).
    let armed = arm_spawn(&root_canon, &b);
    assert!(
        armed.status.success(),
        "arm-spawn must succeed; stderr: {}",
        stderr(&armed)
    );
    let spawn_line = stdout(&armed);
    let spawn = std::fs::canonicalize(spawn_line.trim()).unwrap();

    // main MOVES past B between arm and create (the H1 hazard window).
    std::fs::write(root.path().join("drift.txt"), "post-B").unwrap();
    git(root.path(), &["add", "drift.txt"]);
    git(root.path(), &["commit", "-q", "-m", "main advances past B"]);
    let moved = git(root.path(), &["rev-parse", "HEAD"]);
    assert_ne!(b, moved, "main advanced past B");

    // The hook fires from the armed cwd: create-fork consumes arm-spawn's base file.
    let payload = format!(
        "{{\"cwd\": \"{}\", \"name\": \"agent-h1\"}}",
        spawn.display()
    );
    let forked = create_fork(&spawn, &payload);
    assert!(
        forked.status.success(),
        "create-fork must succeed; stderr: {}",
        stderr(&forked)
    );

    let dir = std::path::PathBuf::from(stdout(&forked).trim());
    assert_eq!(
        dir,
        root_canon.join(".worktrees/agent-h1"),
        "fork created at <root>/.worktrees/<name>"
    );
    // The headline assertion: the fork is pinned to B (arm-spawn's value), NOT the
    // moved main — no wrong-base fallback survives the chained verbs.
    assert_eq!(
        git(&dir, &["rev-parse", "HEAD"]),
        b,
        "fork HEAD is base B (arm-spawn's value), not the moved main"
    );
    assert_ne!(
        git(&dir, &["rev-parse", "HEAD"]),
        moved,
        "fork did NOT fall back to the moved main (H1 dead)"
    );

    // VT-2 / F7: the worker is marked by the in-fork provision+mark (the NEW seam,
    // not the retired SubagentStart stamp), so the post-spawn verify-worker belt
    // passes — base==B holds, the marker is present, the tree is isolated.
    assert!(
        dir.join(".doctrine/state/dispatch/worker").exists(),
        "worker marked via create-fork's in-fork provision+mark (F7)"
    );
    let verified = verify_worker(&b, &dir, "dispatch/agent-h1");
    assert!(
        verified.status.success(),
        "verify-worker passes for the create-fork'd worker; stderr: {}",
        stderr(&verified)
    );
}
