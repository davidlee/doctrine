// SPDX-License-Identifier: GPL-3.0-only
//! SL-088 PHASE-04 — `doctrine install --agent claude` end-to-end over the built binary.
//!
//! Drives the consolidated `doctrine install` handler against a temp project and
//! proves the Claude-surface install (design §9):
//!   * VT-1: `install --agent claude --skill code-review` wires skills + agent def.
//!   * VT-2: the dispatch-worker agent def resolves at `.claude/agents/`.
//!   * SL-152 PHASE-06: the Claude hooks now ship via the doctrine PLUGIN, so the
//!     install verb DELEGATES wiring via printed instructions rather than wiring
//!     the boot (`SessionStart`) / create-fork (`WorktreeCreate`) hooks into
//!     `.claude/settings.local.json` (they double-fired with the plugin). No
//!     SessionStart, WorktreeCreate, or retired SubagentStart hook is emitted.

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

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

/// Run `doctrine install --agent claude --skill code-review` rooted at `dir`,
/// asserting success; return stdout.
fn install(dir: &Path) -> String {
    let out = Command::new(bin())
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

/// The `hooks.<event>` array of a settings file (empty if the file or event is
/// absent — SL-152 PHASE-06: with no hooks settings-wired the file may not exist).
fn event_entries(settings: &Path, event: &str) -> Vec<Value> {
    let Ok(json) = fs::read_to_string(settings) else {
        return Vec::new();
    };
    let value: Value = serde_json::from_str(&json).expect("valid settings JSON");
    value
        .get("hooks")
        .and_then(|h| h.get(event))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// Assert the post-install state holds for a project at `dir`: the agent def
/// resolves, and NO Claude hooks are settings-wired (they ship via the plugin —
/// SL-152 PHASE-06).
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

    // SL-152 PHASE-06: no hooks are settings-wired — they ship via the doctrine
    // plugin. The boot (SessionStart) and create-fork (WorktreeCreate) hooks, plus
    // the retired SubagentStart stamp, are all absent (settings file may carry only
    // baseRef, or not exist at all). `event_entries` treats absent-file as empty.
    let settings = dir.join(".claude/settings.local.json");
    assert!(
        event_entries(&settings, "WorktreeCreate").is_empty(),
        "no WorktreeCreate hook settings-wired (ships via plugin)"
    );
    assert!(
        event_entries(&settings, "SessionStart").is_empty(),
        "no SessionStart boot hook settings-wired (ships via plugin)"
    );
    assert!(
        event_entries(&settings, "SubagentStart").is_empty(),
        "no SubagentStart stamp hook after retirement"
    );
}

#[test]
fn install_wires_skills_agent_and_delegates_hooks_to_plugin() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    let out = install(dir);
    assert!(out.contains("linked    code-review"), "skills leg: {out}");
    assert!(
        out.contains("linked    dispatch-worker.md"),
        "agents leg: {out}"
    );
    // SL-152 PHASE-06: hooks are delegated to the plugin via printed instructions,
    // not settings-wired.
    assert!(
        out.contains("/plugin marketplace add davidlee/doctrine"),
        "plugin marketplace instruction: {out}"
    );
    assert!(
        out.contains("/plugin install doctrine@doctrine"),
        "plugin install instruction: {out}"
    );
    assert!(
        !out.contains("worktree hook: wired"),
        "no settings-wired hook line: {out}"
    );
    assert_installed(dir);

    // Reinstall is idempotent: skills/agent re-reconcile, the plugin instructions
    // print again, and still no hooks are settings-wired.
    let out = install(dir);
    assert!(
        out.contains("/plugin install doctrine@doctrine"),
        "reinstall prints plugin instructions: {out}"
    );
    assert!(
        !out.contains("worktree hook:"),
        "reinstall wires no settings hook: {out}"
    );
    assert_installed(dir);
}

#[test]
fn install_agent_pi_dry_run_prints_delegation_plan() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    let out = Command::new(bin())
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
