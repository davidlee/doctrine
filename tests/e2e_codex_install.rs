// SPDX-License-Identifier: GPL-3.0-only
//! SL-271 PHASE-02, retargeted by SL-286 PHASE-04 (EX-7) — the codex install legs,
//! end-to-end over the built binary.
//!
//! Codex's MCP registration is user-level now: neither `boot install` nor the
//! boot leg of `install` writes a project wrapper, so these tests assert its
//! ABSENCE rather than its shape — an existing `.codex/config.toml` (a non-UTF-8
//! one included) survives byte identical, an absent one stays absent, and a dry
//! run writes nothing. Both runs do carry the user-MCP leg (SL-286 T7), which
//! degrades under the suite's never-existing `CODEX_HOME` and says so
//! (`MCP_SKIPPED`).
//!
//! Hooks are unchanged (`VT-4`'s hook half): `.codex/hooks.json` is written, a
//! second run refreshes nothing, and the activation notice reads the project
//! file's `[features] hooks` key.

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
const HOOKS_REL: &str = ".codex/hooks.json";
/// The degraded user-MCP line, printed because `common::doctrine_cmd` pins
/// `CODEX_HOME` to a path that never exists (`common::SAFE_CODEX_HOME`). The
/// cause is pinned in `src/codex_mcp.rs`; here the header is what matters.
const MCP_SKIPPED: &str = "codex: user MCP registration skipped";

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

/// Run `doctrine boot install --agent <agent>` rooted at `dir`. `install
/// --dry-run` short-circuits before the boot leg, so this is the only path that
/// reaches the dry-run renderer.
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

/// The degraded MCP line's text, if the run printed one.
fn skipped_line(out: &str) -> Option<String> {
    out.lines()
        .find(|line| line.contains(MCP_SKIPPED))
        .map(str::to_string)
}

/// Behaviour-preservation gate: hooks are untouched by SL-286, so every codex
/// install still writes `.codex/hooks.json`.
fn assert_hooks_written(root: &Path) {
    assert!(
        root.join(HOOKS_REL).is_file(),
        "the codex hooks leg still writes {HOOKS_REL}"
    );
}

#[test]
fn codex_install_leaves_an_existing_config_byte_identical() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".codex")).unwrap();
    let config = b"# keep me\n[features]\nhooks = true\n";
    fs::write(root.join(CONFIG_REL), config).unwrap();

    let out = install(root, "codex");
    assert!(
        skipped_line(&out).is_some(),
        "the user MCP leg runs from install and degrades under the test CODEX_HOME: {out}"
    );
    assert_eq!(
        fs::read(root.join(CONFIG_REL)).unwrap(),
        config,
        "no project MCP wrapper is written, so the file is untouched: {}",
        String::from_utf8_lossy(&fs::read(root.join(CONFIG_REL)).unwrap()),
    );
    assert_hooks_written(root);

    // A second run over an unchanged tree is equally inert: the config still
    // holds, and the hooks file is not rewritten.
    let hooks = fs::read(root.join(HOOKS_REL)).unwrap();
    let again = install(root, "codex");
    assert!(skipped_line(&again).is_some(), "still degrades: {again}");
    assert_eq!(fs::read(root.join(CONFIG_REL)).unwrap(), config);
    assert_eq!(
        fs::read(root.join(HOOKS_REL)).unwrap(),
        hooks,
        "the hooks refresh is a no-op"
    );
}

#[test]
fn codex_install_writes_no_config_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    assert!(!root.join(CONFIG_REL).exists());

    let out = install(root, "codex");
    assert!(
        skipped_line(&out).is_some(),
        "the user MCP leg runs from install and degrades under the test CODEX_HOME: {out}"
    );
    assert!(
        !root.join(CONFIG_REL).exists(),
        "boot install creates no project config any more"
    );
    assert_hooks_written(root);
}

#[test]
fn codex_install_does_not_clobber_a_non_utf8_config() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".codex")).unwrap();
    let bytes = [0xff_u8, 0xfe, 0x00];
    fs::write(root.join(CONFIG_REL), bytes).unwrap();

    let out = install(root, "codex");
    assert!(
        skipped_line(&out).is_some(),
        "the user MCP leg runs from install and degrades under the test CODEX_HOME: {out}"
    );
    assert_eq!(
        fs::read(root.join(CONFIG_REL)).unwrap(),
        bytes,
        "an unreadable file is never renamed or rewritten"
    );
    assert_hooks_written(root);
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
    assert!(
        skipped_line(&first).is_some(),
        "the leg ran and said so: {first}"
    );
    assert_eq!(
        skipped_line(&first),
        skipped_line(&second),
        "the same degraded line each install, not a fresh instruction"
    );
    assert!(
        fs::read_to_string(root.join(CONFIG_REL))
            .unwrap()
            .contains("command = \"doctrine\""),
        "the user entry is untouched"
    );
    assert_hooks_written(root);
}

#[test]
fn codex_dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let out = boot_install(root, "codex", true);
    assert!(
        !root.join(CONFIG_REL).exists(),
        "dry run writes no codex config"
    );
    assert!(out.contains("would write .codex/hooks.json"), "{out}");
    assert!(
        !out.contains("wrote"),
        "no dry-run line says 'wrote': {out}"
    );
    assert!(
        skipped_line(&out).is_none() && !out.contains("Codex user MCP (not asked"),
        "a degraded plan proposes nothing, so the dry run discloses no MCP block: {out}"
    );
}

#[test]
fn boot_install_runs_the_user_leg_and_degrades_under_the_safe_codex_home() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let out = boot_install(root, "codex", false);
    assert!(
        skipped_line(&out).is_some(),
        "boot install runs the user leg, which degrades under the test CODEX_HOME: {out}"
    );
    assert_hooks_written(root);
}

#[test]
fn boot_install_dry_run_shows_the_consent_block_and_runs_nothing() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // A proposing environment for the child: a HOME with no user config, and a
    // PATH holding `doctrine` (symlinked to the running binary, so the plan's
    // exec/PATH comparison matches) plus a fake `codex` that records its argv.
    let home = root.join("home");
    let bin = root.join("bin");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&bin).unwrap();
    common::link_doctrine_into(&bin);
    let argv_log = root.join("codex-argv");
    let codex = bin.join("codex");
    fs::write(
        &codex,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> {}\n",
            argv_log.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&codex, fs::Permissions::from_mode(0o755)).unwrap();

    let out = common::doctrine_cmd(root)
        .args(["boot", "install", "--agent", "codex", "-y", "--dry-run"])
        .arg("-p")
        .arg(root)
        .env_remove("CODEX_HOME")
        .env("HOME", &home)
        .env("PATH", &bin)
        .output()
        .expect("spawn doctrine");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        out.status.success(),
        "boot install --dry-run failed: {}\n{stdout}",
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        stdout.contains("Codex user MCP (not asked under --dry-run):"),
        "the dry run discloses the block it would ask: {stdout}"
    );
    assert!(
        stdout.contains("codex mcp add doctrine --"),
        "the block names the command it would run: {stdout}"
    );
    assert!(!argv_log.exists(), "nothing is executed under --dry-run");
}

#[test]
fn codex_notice_fires_only_on_a_hook_write() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    boot_install(root, "codex", false);
    // Keep everything else current, force a fresh hook write.
    fs::remove_file(root.join(HOOKS_REL)).unwrap();

    let out = boot_install(root, "codex", false);
    assert!(
        out.contains("To activate:"),
        "the hook write fires the notice: {out}"
    );
    assert!(
        !out.contains("MCP server registration") && skipped_line(&out).is_some(),
        "the user leg degrades under the test CODEX_HOME, and no project registration is reported: {out}"
    );
}

#[test]
fn codex_hook_clean_run_ignores_a_project_mcp_entry() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    boot_install(root, "codex", false);
    // A project entry the retired boot leg used to report on: untouched and
    // unreported now.
    let config = b"[mcp_servers.doctrine]\ncommand = \"doctrine\"\n";
    fs::write(root.join(CONFIG_REL), config).unwrap();

    let out = boot_install(root, "codex", false);
    assert!(
        !out.contains("To activate:"),
        "a hook-clean run prints no activation notice: {out}"
    );
    assert!(
        !out.contains("MCP server registration") && skipped_line(&out).is_some(),
        "the degraded user leg says so, and the project entry is not reported: {out}"
    );
    assert_eq!(
        fs::read(root.join(CONFIG_REL)).unwrap(),
        config,
        "the project entry is untouched"
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
    fs::remove_file(root.join(HOOKS_REL)).unwrap();
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
    fs::remove_file(root.join(HOOKS_REL)).unwrap();
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
        !out.contains(".codex/config.toml") && skipped_line(&out).is_none(),
        "a Claude-only run emits no codex MCP line: {out}"
    );
    assert!(
        !out.contains("To activate:"),
        "a Claude-only run emits no codex hooks notice: {out}"
    );
    assert!(
        !root.join(CONFIG_REL).exists(),
        "a Claude-only run writes no codex config"
    );
}
