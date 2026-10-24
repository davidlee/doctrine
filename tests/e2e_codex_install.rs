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
//! (`common::CODEX_MCP_SKIPPED`).
//!
//! Hooks are unchanged (`VT-4`'s hook half): `.codex/hooks.json` is written, a
//! second run refreshes nothing, and the activation notice reads the project
//! file's `[features] hooks` key.
//!
//! Where the leg can PROPOSE (`common::ProposingEnv`), the tests also pin what it
//! would do: exactly one `codex mcp add` argv carrying the baked `PATH` entry, and
//! the owned project wrapper removed beside a foreign server left alone. Such an
//! env is laid out BESIDE the project (`<tmp>/scratch`, project `<tmp>/project`),
//! never inside the tree being installed — the fixture env is not project state.

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
/// Why the leg degrades under the pinned `CODEX_HOME` — mirrors
/// `src/codex_mcp.rs`'s `CAUSE_BAD_CODEX_HOME`, the clause
/// [`common::CODEX_MCP_SKIPPED`] leaves out.
const CAUSE_BAD_CODEX_HOME: &str = "CODEX_HOME is not an existing absolute directory";

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

/// The wrapper line `src/codex_mcp.rs` emits as the codex `args[1]`, copied here
/// as a literal: integration tests compile against the binary, never its private
/// items. Byte-identical on purpose — the fixture's whole job is a wrapper the
/// classifier recognises as doctrine's own.
const CODEX_MCP_WRAPPER: &str = "exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp";

/// Seed the project config with doctrine's owned wrapper and a foreign server the
/// write must not touch; return the foreign table as the caller's `before`.
fn seed_wrapped_project_config(root: &Path) -> toml::Value {
    fs::create_dir_all(root.join(".codex")).unwrap();
    fs::write(
        root.join(CONFIG_REL),
        format!(
            "[mcp_servers.doctrine]\ncommand = \"sh\"\nargs = [\"-c\", '{CODEX_MCP_WRAPPER}']\n\
             \n[mcp_servers.other]\ncommand = \"/usr/bin/other\"\n"
        ),
    )
    .unwrap();
    project_server(root, "other").expect("the seeded foreign server parses")
}

/// One `[mcp_servers.<key>]` table from the project config, if present.
fn project_server(root: &Path, key: &str) -> Option<toml::Value> {
    let text = fs::read_to_string(root.join(CONFIG_REL)).unwrap();
    let doc: toml::Value = toml::from_str(&text).unwrap();
    doc.get("mcp_servers")
        .and_then(|servers| servers.get(key))
        .cloned()
}

/// What the user leg did: one `codex mcp add` carrying the baked `PATH` entry,
/// the owned project wrapper gone, every foreign server as it was.
fn assert_user_leg_wired(env: &common::ProposingEnv, root: &Path, before_other: &toml::Value) {
    // Element by element: the bake is one argument, not a string that happens to
    // contain spaces, and the separator is its own.
    let bake = env.bin.join("doctrine").display().to_string();
    let argv: Vec<String> = ["mcp", "add", "doctrine", "--", &bake, "serve", "--mcp"]
        .iter()
        .map(|word| (*word).to_string())
        .collect();
    assert_eq!(
        env.calls(),
        vec![argv],
        "exactly one `codex mcp add`, baking the `PATH` entry rather than its target"
    );
    assert_eq!(
        project_server(root, "doctrine"),
        None,
        "the owned project wrapper is removed with the user write: {}",
        fs::read_to_string(root.join(CONFIG_REL)).unwrap(),
    );
    assert_eq!(
        project_server(root, "other").as_ref(),
        Some(before_other),
        "a foreign server is left untouched"
    );
}

/// The degraded MCP line's text, if the run printed one.
fn skipped_line(out: &str) -> Option<String> {
    out.lines()
        .find(|line| line.contains(common::CODEX_MCP_SKIPPED))
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
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(&project).unwrap();
    let root = project.as_path();
    // The environment in which the user-MCP leg proposes rather than degrades.
    let env = common::ProposingEnv::new(&dir.path().join("scratch"));

    let out = env
        .doctrine(root)
        .args(["boot", "install", "--agent", "codex", "-y", "--dry-run"])
        .arg("-p")
        .arg(root)
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
    assert!(
        env.calls().is_empty(),
        "nothing is executed under --dry-run"
    );
}

#[test]
fn install_yes_records_the_mcp_add_argv_and_removes_the_wrapper() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(&project).unwrap();
    let root = project.as_path();
    let before_other = seed_wrapped_project_config(root);
    let env = common::ProposingEnv::new(&dir.path().join("scratch"));

    let out = env
        .doctrine(root)
        .args([
            "install",
            "--agent",
            "codex",
            "--skill",
            "code-review",
            "--yes",
        ])
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "install failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert_user_leg_wired(&env, root, &before_other);
    assert_hooks_written(root);
}

#[test]
fn boot_install_yes_wires_the_user_leg_as_install_does() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(&project).unwrap();
    let root = project.as_path();
    let before_other = seed_wrapped_project_config(root);
    let env = common::ProposingEnv::new(&dir.path().join("scratch"));

    let out = env
        .doctrine(root)
        .args(["boot", "install", "--agent", "codex", "-y"])
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "boot install failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert_user_leg_wired(&env, root, &before_other);
    assert_hooks_written(root);
}

/// Nothing happened: codex was never run, the project config is byte identical,
/// and the user config was never created.
fn assert_leg_stood_down(env: &common::ProposingEnv, root: &Path, before: &[u8]) {
    assert!(
        env.calls().is_empty(),
        "no `codex mcp add`: {:?}",
        env.calls()
    );
    assert_eq!(
        fs::read(root.join(CONFIG_REL)).unwrap(),
        before,
        "the owned project wrapper stays — no consent, no removal"
    );
    assert!(
        !env.user_config().exists(),
        "the user config is codex's to write, and codex never ran: {}",
        env.user_config().display()
    );
}

#[test]
fn interactive_decline_calls_no_codex_and_changes_neither_config() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(&project).unwrap();
    let root = project.as_path();
    seed_wrapped_project_config(root);
    let before = fs::read(root.join(CONFIG_REL)).unwrap();
    let env = common::ProposingEnv::new(&dir.path().join("scratch"));

    // Two questions, in source order: the run's `Proceed? [y/N/d]`, then the
    // codex consent block's `Write user config? [y/N]` (the user write is
    // pending, so that is the ask — `src/codex_mcp.rs::consent_block`).
    let mut cmd = env.doctrine(root);
    cmd.args(["install", "--agent", "codex", "--skill", "code-review"])
        .arg("-p")
        .arg(root);
    let out = common::output_with_stdin(cmd, "y\nn\n");
    assert!(
        out.status.success(),
        "a declined consent is not a failure: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("Write user config? [y/N]"),
        "the codex consent was the second question: {stdout}"
    );
    assert!(
        stdout.contains("codex: user MCP registration declined; nothing changed"),
        "the decline is reported: {stdout}"
    );
    assert_leg_stood_down(&env, root, &before);
}

#[test]
fn dry_run_calls_no_codex_and_writes_neither_config() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(&project).unwrap();
    let root = project.as_path();
    seed_wrapped_project_config(root);
    let before = fs::read(root.join(CONFIG_REL)).unwrap();
    let env = common::ProposingEnv::new(&dir.path().join("scratch"));

    // `--dry-run` is non-interactive by construction (`is_interactive`), so no
    // `--yes` is needed for it to proceed; it discloses the block instead.
    let out = env
        .doctrine(root)
        .args([
            "install",
            "--agent",
            "codex",
            "--skill",
            "code-review",
            "--dry-run",
        ])
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "dry run failed: {}",
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("Codex user MCP (not asked under --dry-run):"),
        "the dry run discloses the block rather than asking it: {stdout}"
    );
    assert_leg_stood_down(&env, root, &before);
}

/// EX-5 — the shared spawn seam is what keeps codex out of every other suite.
///
/// [`common::doctrine_cmd`] pins `CODEX_HOME` to a path that never exists, so the
/// user leg degrades instead of proposing REGARDLESS of the environment around
/// it: here the fake `codex` and the `doctrine` link of a
/// [`common::ProposingEnv`] are all in place, and codex still never runs.
/// `install_yes_records_the_mcp_add_argv_and_removes_the_wrapper` is this test's
/// positive control — the same env minus the pin calls codex exactly once.
#[test]
fn the_default_environment_never_invokes_codex() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(&project).unwrap();
    let root = project.as_path();
    seed_wrapped_project_config(root);
    let env = common::ProposingEnv::new(&dir.path().join("scratch"));

    // Deliberately NOT `env.doctrine`: that spawn seam removes the `CODEX_HOME`
    // pin. Only `HOME`/`PATH` are taken from the fixture env.
    let out = common::doctrine_cmd(root)
        .env("HOME", &env.home)
        .env("PATH", &env.bin)
        .args([
            "install",
            "--agent",
            "codex",
            "--skill",
            "code-review",
            "--yes",
        ])
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        out.status.success(),
        "install failed: {}\n{stdout}",
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        skipped_line(&stdout).is_some_and(|line| line.contains(CAUSE_BAD_CODEX_HOME)),
        "the leg degrades for the PIN's reason, not some other one: {stdout}"
    );
    assert!(
        env.calls().is_empty(),
        "the CODEX_HOME pin keeps codex from being run: {:?}",
        env.calls()
    );
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

/// The `PATH` a `worktree provision` child needs: the fixture's `bin` first, then
/// every directory this test process's own `PATH` finds `git` in, in order.
///
/// `bin` leads so its fake `codex` and `doctrine` link shadow any real ones that
/// share a directory with git — else a leaking leg could reach a real codex, or
/// the env could stop proposing and the test pass vacuously.
///
/// `worktree provision` shells out to git, and [`common::ProposingEnv`] narrows
/// `PATH` to its fake-codex `bin` alone — so git has to be put back. Every entry,
/// not just the first: a `git` shim (this workspace installs a guard wrapper
/// ahead of the real git) resolves the real one by skipping its own directory, so
/// it needs the later entries too. Nothing is hardcoded; the list comes from this
/// process.
fn provision_path(env: &common::ProposingEnv) -> String {
    let path = std::env::var_os("PATH").expect("this process has a PATH");
    let dirs: Vec<std::path::PathBuf> = std::env::split_paths(&path)
        .filter(|dir| dir.join("git").is_file())
        .collect();
    assert!(!dirs.is_empty(), "git on this process's PATH");
    std::env::join_paths(std::iter::once(&env.bin).chain(&dirs))
        .expect("join the provision PATH")
        .into_string()
        .expect("utf8 provision PATH")
}

/// DEC-411 (EX-3): the codex user leg runs only from an explicit `install` /
/// `boot install` — never from fork regeneration. `worktree provision` is the one
/// caller of `install::regenerate`, so it is the surface that could leak.
///
/// The stale seed is what keeps this non-vacuous: an explicit install would
/// PROPOSE to replace that entry, so a leaking leg would either run
/// `codex mcp add` or rewrite the file. Both are asserted absent here, rather
/// than inferred from a plan that had nothing to do.
#[test]
fn fork_regeneration_leaves_the_user_config_alone_and_calls_no_codex() {
    let dir = tempfile::tempdir().unwrap();
    let source = common::installed_source(&dir.path().join("src"));
    let fork = dir.path().join("fork");
    common::add_sibling_worktree(&source, &fork);

    // An environment of its own — the fake codex and the doctrine link live in
    // this scratch dir, beside the source and the fork, never inside them.
    let env = common::ProposingEnv::new(&dir.path().join("scratch"));
    fs::create_dir_all(env.user_config().parent().unwrap()).unwrap();
    fs::write(
        env.user_config(),
        "[mcp_servers.doctrine]\ncommand = \"/stale/doctrine\"\nargs = [\"serve\", \"--mcp\"]\n",
    )
    .unwrap();
    let before = fs::read(env.user_config()).unwrap();

    let path = provision_path(&env);
    let out = env
        .doctrine(&source)
        .env("PATH", &path)
        .args(["worktree", "provision"])
        .arg(&fork)
        .arg("-p")
        .arg(&source)
        .output()
        .expect("spawn doctrine");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(out.status.success(), "provision failed: {stdout}\n{stderr}",);
    assert!(
        fork.join(HOOKS_REL).is_file(),
        "regeneration reached the fork's install legs — else nothing here was exercised"
    );
    for (stream, text) in [("stdout", &stdout), ("stderr", &stderr)] {
        assert!(
            !text.contains("codex mcp add"),
            "regeneration discloses no MCP plan on {stream}: {text}"
        );
    }
    assert_eq!(
        fs::read(env.user_config()).unwrap(),
        before,
        "regeneration leaves the user config byte identical: {}",
        String::from_utf8_lossy(&fs::read(env.user_config()).unwrap()),
    );
    assert!(
        env.calls().is_empty(),
        "regeneration never runs codex: {:?}",
        env.calls()
    );
}
