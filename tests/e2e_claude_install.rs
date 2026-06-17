// SPDX-License-Identifier: GPL-3.0-only
//! SL-088 PHASE-04 — `doctrine install --agent claude` end-to-end over the built binary.
//!
//! Drives the consolidated `doctrine install` handler against a temp project and
//! proves the Claude-surface install does three things in one verb (design §9):
//!   * VT-1: `install --agent claude --skill code-review` wires skills, agent def,
//!     and the SubagentStart hook into `.claude/`.
//!   * VT-2: the dispatch-worker agent def resolves at `.claude/agents/`.
//!   * the `SubagentStart` hook is merged into `.claude/settings.local.json`,
//!     matcher-scoped to the dispatch-worker agent type, idempotent on reinstall.

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

/// The `hooks.SubagentStart` array of a settings file (empty if absent).
fn subagent_entries(settings: &Path) -> Vec<Value> {
    let json = fs::read_to_string(settings).expect("settings readable");
    let value: Value = serde_json::from_str(&json).expect("valid settings JSON");
    value
        .get("hooks")
        .and_then(|h| h.get("SubagentStart"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// Assert the post-install state holds for a project at `dir`: the agent def
/// resolves, and the SubagentStart hook is wired matcher-scoped exactly once.
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

    // The SubagentStart hook: exactly one entry, matcher-scoped to dispatch-worker,
    // command is `<exec> worktree marker --stamp-subagent`.
    let settings = dir.join(".claude/settings.local.json");
    let subs = subagent_entries(&settings);
    assert_eq!(subs.len(), 1, "exactly one SubagentStart entry");
    assert_eq!(
        subs[0].get("matcher").and_then(Value::as_str),
        Some("dispatch-worker"),
        "matcher-scoped to the dispatch-worker agent type"
    );
    let cmd = subs[0]
        .get("hooks")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|h| h.get("command"))
        .and_then(Value::as_str)
        .expect("SubagentStart command");
    assert!(
        cmd.ends_with(" worktree marker --stamp-subagent"),
        "stamp hook command: {cmd}"
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
    assert!(out.contains("subagent hook: wired"), "hook leg: {out}");
    assert_installed(dir);

    // Reinstall is idempotent: no duplicate SubagentStart entry, hook now current.
    let out = install(dir);
    assert!(
        out.contains("subagent hook: already current"),
        "reinstall hook no-op: {out}"
    );
    assert_installed(dir);
}
