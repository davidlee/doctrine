// SPDX-License-Identifier: GPL-3.0-only
//! SL-088 PHASE-04 — `doctrine install --agent claude` end-to-end over the built binary.
//!
//! Drives the consolidated `doctrine install` handler against a temp project and
//! proves the Claude-surface install does three things in one verb (design §9):
//!   * VT-1: `install --agent claude --skill code-review` wires skills, agent def,
//!     and the `WorktreeCreate` hook into `.claude/`.
//!   * VT-2: the dispatch-worker agent def resolves at `.claude/agents/`.
//!   * the `WorktreeCreate` hook (`worktree create-fork`) is merged into
//!     `.claude/settings.local.json`, idempotent on reinstall — and the retired
//!     SL-123 `SubagentStart` stamp hook is NEVER emitted (SL-152 D2).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

/// Run `doctrine install --agent claude --skill code-review` rooted at `dir`,
/// asserting success; return stdout.
fn install(dir: &Path) -> String {
    let out = Command::new(BIN)
        .args([
            "install",
            "--agent",
            "claude",
            "--skill",
            "code-review",
            "--yes",
            "-p",
        ])
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "install failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// The `hooks.<event>` array of a settings file (empty if absent).
fn event_entries(settings: &Path, event: &str) -> Vec<Value> {
    let json = fs::read_to_string(settings).expect("settings readable");
    let value: Value = serde_json::from_str(&json).expect("valid settings JSON");
    value
        .get("hooks")
        .and_then(|h| h.get(event))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// Assert the post-install state holds for a project at `dir`: the agent def
/// resolves, and the WorktreeCreate hook is wired exactly once (with the retired
/// SubagentStart stamp absent).
fn assert_installed(dir: &Path) {
    // VT-2: the agent def is a link resolving to materialised content.
    let agent_link = dir.join(".claude/agents/dispatch-worker.md");
    assert!(
        fs::symlink_metadata(&agent_link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "agent path is a symlink"
    );
    let body = fs::read_to_string(&agent_link).expect("agent def resolves");
    assert!(
        body.contains("dispatch worker"),
        "agent link resolves to the dispatch-worker def: {body:.80}"
    );
    assert!(
        dir.join(".doctrine/agents/dispatch-worker.md").is_file(),
        "canonical agent def materialised"
    );

    // The WorktreeCreate hook: exactly one entry, command `<exec> worktree create-fork`.
    let settings = dir.join(".claude/settings.local.json");
    let wc = event_entries(&settings, "WorktreeCreate");
    assert_eq!(wc.len(), 1, "exactly one WorktreeCreate entry");
    let cmd = wc[0]
        .get("hooks")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|h| h.get("command"))
        .and_then(Value::as_str)
        .expect("WorktreeCreate command");
    assert!(
        cmd.ends_with(" worktree create-fork"),
        "create-fork hook command: {cmd}"
    );

    // VT-1 (negative, D2): the retired SubagentStart stamp hook is never emitted.
    assert!(
        event_entries(&settings, "SubagentStart").is_empty(),
        "no SubagentStart stamp hook after retirement"
    );
}

#[test]
fn install_wires_skills_agent_and_hook_idempotently() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    let out = install(dir);
    assert!(out.contains("linked    code-review"), "skills leg: {out}");
    assert!(
        out.contains("linked    dispatch-worker.md"),
        "agents leg: {out}"
    );
    assert!(out.contains("worktree hook: wired"), "hook leg: {out}");
    assert_installed(dir);

    // Reinstall is idempotent: no duplicate WorktreeCreate entry, hook now current.
    let out = install(dir);
    assert!(
        out.contains("worktree hook: already current"),
        "reinstall hook no-op: {out}"
    );
    assert_installed(dir);
}

#[test]
fn install_agent_pi_dry_run_prints_delegation_plan() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    let out = Command::new(BIN)
        .args(["install", "--agent", "pi", "--dry-run", "-p"])
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "install --agent pi --dry-run failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("pi"),
        "pi agent mentioned in plan: {stdout}"
    );
    assert!(
        stdout.contains("delegates to npx"),
        "npx delegation shown: {stdout}"
    );
    assert!(
        stdout.contains("not executed"),
        "dry-run indicator present: {stdout}"
    );
    // Dry-run must NOT create any files beyond what the temp dir started with.
    assert!(
        !dir.join(".doctrine").exists(),
        "dry-run created no .doctrine dir"
    );
    assert!(!dir.join(".pi").exists(), "dry-run created no .pi dir");
}
