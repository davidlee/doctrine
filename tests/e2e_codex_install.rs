// SPDX-License-Identifier: GPL-3.0-only
//! SL-271 PHASE-02 — the codex MCP registration leg, end-to-end over the built binary.
//!
//! `VT-4`: a project with `[features] hooks = true` and a comment gains the emitted
//! `[mcp_servers.doctrine]` entry (asserted by PARSED value, never bytes) with keys
//! and comment intact and a second run a no-op; a project with no
//! `.codex/config.toml` gets the file and parent directory; a non-UTF-8 file is
//! unchanged with the fallback line; a foreign entry's repeat output is stable; no
//! host abspath is written; and a Claude-only run prints the invocation once with no
//! codex line.
//! `VT-5`: a dry run over an absent entry renders "would write" and writes nothing.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;

mod common;

const CONFIG_REL: &str = ".codex/config.toml";
const WRAPPER: &str = "exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp";
const WROTE_LINE: &str = "wrote MCP server registration in .codex/config.toml";
const WOULD_LINE: &str = "would write MCP server registration in .codex/config.toml";
const FALLBACK_MARKER: &str = "could not be interpreted";

/// Run `doctrine install --agent <agent>` rooted at `dir`, asserting success;
/// return stdout. `--skill code-review` keeps the install small and offline.
fn install(dir: &Path, agent: &str) -> String {
    let out = common::doctrine_cmd(dir)
        .args([
            "install",
            "--agent",
            agent,
            "--skill",
            "code-review",
            "--yes",
        ])
        .arg("-p")
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "install ({agent}) failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// Run `doctrine boot install --agent <agent>` rooted at `dir` — the path that
/// reaches `wire(..., dry_run)`. `install --dry-run` short-circuits before wire.
fn boot_install(dir: &Path, agent: &str, dry_run: bool) -> String {
    let mut cmd = common::doctrine_cmd(dir);
    cmd.args(["boot", "install", "--agent", agent, "-y"]);
    if dry_run {
        cmd.arg("--dry-run");
    }
    let out = cmd.arg("-p").arg(dir).output().expect("spawn doctrine");
    assert!(
        out.status.success(),
        "boot install ({agent}) failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// The written `[mcp_servers.doctrine]` entry, by parsed value.
fn codex_entry(dir: &Path) -> (Option<String>, Option<Vec<String>>, Option<Vec<String>>) {
    let text = fs::read_to_string(dir.join(CONFIG_REL)).expect("config exists");
    let doc: toml_edit::DocumentMut = text.parse().expect("config parses");
    let servers = doc
        .get("mcp_servers")
        .and_then(|i| i.as_table_like())
        .expect("mcp_servers table");
    let entry = servers
        .get("doctrine")
        .and_then(|i| i.as_table_like())
        .expect("doctrine entry");
    let command = entry
        .get("command")
        .and_then(|i| i.as_str())
        .map(String::from);
    let args = entry.get("args").and_then(|i| i.as_array()).map(|a| {
        a.iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect()
    });
    let env = entry.get("env_vars").and_then(|i| i.as_array()).map(|a| {
        a.iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect()
    });
    (command, args, env)
}

#[test]
fn codex_install_preserves_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".codex")).unwrap();
    fs::write(
        root.join(CONFIG_REL),
        "# keep me\n[features]\nhooks = true\n",
    )
    .unwrap();

    let out = install(root, "codex");
    assert!(out.contains(WROTE_LINE), "codex MCP line: {out}");

    let (command, args, env) = codex_entry(root);
    assert_eq!(command.as_deref(), Some("sh"));
    assert_eq!(args, Some(vec!["-c".to_string(), WRAPPER.to_string()]));
    assert_eq!(env, Some(vec!["DOCTRINE_BIN".to_string()]));

    let text = fs::read_to_string(root.join(CONFIG_REL)).unwrap();
    assert!(text.contains("# keep me"), "comment preserved: {text}");
    assert!(
        text.contains("hooks = true"),
        "[features] preserved: {text}"
    );
    assert!(
        !text.contains("/workspace") && !text.contains("/nix/store"),
        "no host abspath in the tracked config: {text}"
    );

    // A second run over an unchanged file is a no-op MCP line, and the file holds.
    let before = fs::read_to_string(root.join(CONFIG_REL)).unwrap();
    let again = install(root, "codex");
    assert!(
        !again.contains(WROTE_LINE),
        "second run is a no-op MCP line: {again}"
    );
    assert_eq!(fs::read_to_string(root.join(CONFIG_REL)).unwrap(), before);
}

#[test]
fn codex_install_creates_the_file_from_absent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    assert!(!root.join(CONFIG_REL).exists());

    let out = install(root, "codex");
    assert!(out.contains(WROTE_LINE), "codex MCP line: {out}");
    assert!(root.join(CONFIG_REL).is_file(), "config created");
    let (command, _, _) = codex_entry(root);
    assert_eq!(command.as_deref(), Some("sh"));
}

#[test]
fn codex_install_does_not_clobber_a_non_utf8_config() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".codex")).unwrap();
    let bytes = [0xff_u8, 0xfe, 0x00];
    fs::write(root.join(CONFIG_REL), bytes).unwrap();

    let out = install(root, "codex");
    assert!(out.contains(FALLBACK_MARKER), "fallback line: {out}");
    assert_eq!(
        fs::read(root.join(CONFIG_REL)).unwrap(),
        bytes,
        "an unreadable file is never renamed or rewritten"
    );
}

#[test]
fn codex_install_foreign_entry_repeat_output_is_stable() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".codex")).unwrap();
    fs::write(
        root.join(CONFIG_REL),
        "[mcp_servers.doctrine]\ncommand = \"doctrine\"\nargs = [\"serve\", \"--mcp\"]\n",
    )
    .unwrap();

    let first = install(root, "codex");
    let second = install(root, "codex");
    let fallback = |s: &str| {
        s.lines()
            .find(|l| l.contains(FALLBACK_MARKER))
            .map(str::to_string)
    };
    assert_eq!(
        fallback(&first),
        fallback(&second),
        "a foreign entry yields the same line each install, not a fresh instruction"
    );
    assert!(
        fs::read_to_string(root.join(CONFIG_REL))
            .unwrap()
            .contains("command = \"doctrine\""),
        "the user entry is untouched"
    );
}

#[test]
fn codex_dry_run_says_would_write_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let out = boot_install(root, "codex", true);
    assert!(out.contains(WOULD_LINE), "dry-run codex MCP line: {out}");
    assert!(
        !root.join(CONFIG_REL).exists(),
        "dry run writes no codex config"
    );
    // After PHASE-03 the activation notice also says "would write", so NO dry-run
    // output line contains "wrote" at all.
    assert!(out.contains("would write .codex/hooks.json"), "{out}");
    assert!(
        !out.contains("wrote"),
        "no dry-run line says 'wrote': {out}"
    );
}

#[test]
fn codex_notice_fires_on_a_hook_write_even_when_the_mcp_entry_is_current() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    boot_install(root, "codex", false);
    // Keep the config current, force a fresh hook write.
    fs::remove_file(root.join(".codex/hooks.json")).unwrap();

    let out = boot_install(root, "codex", false);
    assert!(
        out.contains("To activate:"),
        "the hook write fires the notice: {out}"
    );
    assert!(
        !out.contains("MCP server registration"),
        "the MCP entry was already current: {out}"
    );
    assert!(
        !out.contains("not active until you trust"),
        "the activation notice already carries the trust step: {out}"
    );
}

#[test]
fn codex_notice_does_not_fire_when_only_the_mcp_entry_refreshed() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    boot_install(root, "codex", false);
    // Keep the hooks current, force the MCP leg to write.
    fs::remove_file(root.join(CONFIG_REL)).unwrap();

    let out = boot_install(root, "codex", false);
    assert!(out.contains(WROTE_LINE), "the MCP leg wrote: {out}");
    assert!(
        !out.contains("To activate:"),
        "a hook-clean run prints no activation notice: {out}"
    );
    assert!(
        out.contains("not active until you trust"),
        "the trust caveat prints once, since the notice did not: {out}"
    );
}

#[test]
fn codex_activation_notice_reads_the_project_hooks_key() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Seed the project file with hooks OFF. The hook leg writes a fresh
    // `.codex/hooks.json`, firing the activation notice; RV-403 F-1's case is
    // that the pre-trust probe suppressed step 1 here, and the project-file read
    // must not.
    fs::create_dir_all(root.join(".codex")).unwrap();
    let config = root.join(CONFIG_REL);
    fs::write(&config, "[features]\nhooks = false\n").unwrap();

    let out = boot_install(root, "codex", false);
    assert!(out.contains("To activate:"), "{out}");
    assert!(
        out.contains("Enable [features] hooks = true"),
        "hooks = false still prints step 1: {out}"
    );

    // Flip the project file to enabled: step 1 is omitted, the rest still prints.
    fs::write(&config, "[features]\nhooks = true\n").unwrap();
    fs::remove_file(root.join(".codex/hooks.json")).unwrap();
    let out = boot_install(root, "codex", false);
    assert!(out.contains("To activate:"), "{out}");
    assert!(
        !out.contains("hooks = true in .codex/config.toml"),
        "hooks = true omits step 1: {out}"
    );
    assert!(out.contains("Start codex"), "step 2 still prints: {out}");

    // A config without a `[features] hooks` key falls back to the unconditional
    // instruction.
    fs::write(&config, "# no features table\n").unwrap();
    fs::remove_file(root.join(".codex/hooks.json")).unwrap();
    let out = boot_install(root, "codex", false);
    assert!(
        out.contains("Ensure [features] hooks = true"),
        "an absent key still prints step 1: {out}"
    );
}

#[test]
fn claude_only_run_prints_the_invocation_once_and_no_codex_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let out = install(root, "claude");
    assert!(
        out.contains("registered MCP server in .mcp.json: ${DOCTRINE_BIN:-doctrine} serve --mcp"),
        "the Claude MCP line carries the full invocation: {out}"
    );
    assert!(
        !out.contains("serve --mcp serve --mcp"),
        "the invocation is printed once, never re-appended: {out}"
    );
    assert!(
        !out.contains(".codex/config.toml"),
        "a Claude-only run emits no codex line: {out}"
    );
    assert!(
        !out.contains("To activate:") && !out.contains("not active until you trust"),
        "a Claude-only run emits neither the hooks notice nor the codex trust caveat: {out}"
    );
    assert!(
        !root.join(CONFIG_REL).exists(),
        "a Claude-only run writes no codex config"
    );
}
