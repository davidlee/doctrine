// SPDX-License-Identifier: GPL-3.0-only
//! SL-282 PHASE-03 VT-1 — the install executor over the built binary: one
//! harness set (`AGENTS.md` rule shared with `boot install`), leg outcomes, and
//! the version stamps (tracked `installed-version` in `doctrine.toml`, local
//! marker) written last and only when every leg finished (RV-421 F-14).
//!
//! Every project is a fresh tempdir; global runs override `HOME` on the child
//! only. Every "neither stamp" row pairs with a positive control that the leg
//! under test really ran.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

mod common;

use common::{INSTALL_MARKER, doctrine_cmd, doctrine_toml};

/// The running binary's version: what a complete install records.
const B: &str = env!("CARGO_PKG_VERSION");
/// One harness, one skill — small and fast.
const BASE_ARGS: &[&str] = &["--agent", "claude", "--skill", "code-review", "--yes"];
const INCOMPLETE: &str = "install incomplete:";
const CODEX_HOOKS: &str = ".codex/hooks.json";

/// `doctrine <verb…> -p <root>` with extra env on the child; asserts success
/// and returns stdout.
fn run(root: &Path, args: &[&str], env: &[(&str, &OsStr)]) -> String {
    let mut cmd = doctrine_cmd(root);
    cmd.args(args).arg("-p").arg(root);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("spawn doctrine");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {stdout}\n{}",
        String::from_utf8_lossy(&out.stderr),
    );
    stdout
}

fn install(root: &Path, extra: &[&str]) -> String {
    let args: Vec<&str> = ["install"]
        .iter()
        .chain(BASE_ARGS)
        .chain(extra)
        .copied()
        .collect();
    run(root, &args, &[])
}

fn tracked_text(root: &Path) -> Option<String> {
    fs::read_to_string(doctrine_toml(root)).ok()
}

fn stamp_line() -> String {
    format!("installed-version = \"{B}\"")
}

/// Neither the tracked key nor the local marker exists.
fn assert_unstamped(root: &Path, out: &str) {
    assert!(
        !tracked_text(root).is_some_and(|t| t.contains("installed-version")),
        "no tracked stamp: {out}"
    );
    assert!(
        !root.join(INSTALL_MARKER).exists(),
        "no local marker: {out}"
    );
}

#[test]
fn a_complete_install_records_both_stamps_and_is_idempotent() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let out = install(root, &[]);

    assert!(!out.contains(INCOMPLETE), "{out}");
    let tracked = tracked_text(root).expect("doctrine.toml");
    assert!(tracked.contains(&stamp_line()), "{tracked}");
    let marker = fs::read_to_string(root.join(INSTALL_MARKER)).unwrap();
    let marker: toml::Value = toml::from_str(&marker).unwrap();
    assert_eq!(marker["version"].as_str(), Some(B));
    assert_eq!(
        marker["harnesses"].as_array().unwrap(),
        &vec![toml::Value::from("claude")]
    );

    install(root, &[]);
    assert_eq!(
        tracked_text(root).unwrap(),
        tracked,
        "a second run leaves doctrine.toml byte-identical"
    );
}

/// RV-421 F-14 control: a harness refresh that fails is caught and printed;
/// a candidate that stamps after the caught failure fails this row.
#[test]
fn a_failed_boot_refresh_blocks_both_stamps() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // A directory squatting the settings path makes the atomic rename fail.
    fs::create_dir_all(root.join(".claude/settings.json")).unwrap();

    let out = install(root, &[]);

    assert!(out.contains("claude: refresh failed"), "{out}");
    assert!(
        out.contains("install incomplete: boot wiring; version not recorded"),
        "{out}"
    );
    assert!(
        root.join("CLAUDE.md").is_file(),
        "control: the boot leg ran and wired the import"
    );
    assert_unstamped(root, &out);
}

#[test]
fn malformed_gitignore_markers_block_both_stamps_and_exit_zero() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let body = "keep/\n# >>> doctrine managed (hand-edited)\nstale\n";
    fs::write(root.join(".gitignore"), body).unwrap();

    let out = install(root, &[]);

    assert!(
        out.contains("install incomplete: gitignore; version not recorded"),
        "{out}"
    );
    assert_eq!(fs::read_to_string(root.join(".gitignore")).unwrap(), body);
    assert_unstamped(root, &out);
}

#[test]
fn global_and_only_memory_installs_record_no_version() {
    if common::under_worker_marker() {
        return;
    }
    let home = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let args: Vec<&str> = ["install", "--global"]
        .iter()
        .chain(BASE_ARGS)
        .copied()
        .collect();
    let out = run(root, &args, &[("HOME", home.path().as_os_str())]);
    assert!(!out.contains(INCOMPLETE), "{out}");
    assert!(
        home.path().join(".doctrine/skills").exists(),
        "control: the global leg ran: {out}"
    );
    assert_unstamped(root, &out);

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let out = run(
        root,
        &["install", "--agent", "claude", "--only-memory", "--yes"],
        &[],
    );
    assert!(!out.contains(INCOMPLETE), "{out}");
    assert!(root.join("CLAUDE.md").is_file(), "control: legs ran: {out}");
    assert_unstamped(root, &out);
}

/// One harness set: an `AGENTS.md`-only repo wires the codex import and runs
/// codex's skills leg (delegated to npx under the `universal` route).
#[test]
fn an_agents_md_only_repo_wires_and_installs_codex_from_one_set() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("AGENTS.md"), "# agents\n").unwrap();
    // No npx on PATH: the delegation is attempted (and reported) offline.
    let empty_path = tempfile::tempdir().unwrap();

    let out = run(
        root,
        &["install", "--skill", "code-review", "--yes"],
        &[("PATH", empty_path.path().as_os_str())],
    );

    let agents_md = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(
        agents_md.starts_with("@.doctrine/state/boot.md\n"),
        "codex import wired: {agents_md}"
    );
    assert!(root.join(CODEX_HOOKS).is_file(), "codex hooks: {out}");
    assert!(
        out.contains("agents universal (delegate):"),
        "codex skills leg ran: {out}"
    );
}

/// EX-1: a lone `CLAUDE.md` ↔ `AGENTS.md` symlink pair with no `.claude/` is a
/// codex surface for `install` and `boot install` alike.
#[test]
fn a_lone_symlinked_pair_is_codex_for_both_verbs() {
    if common::under_worker_marker() {
        return;
    }
    for verb in [
        &["install", "--skill", "code-review", "--yes"][..],
        &["boot", "install", "--yes"],
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("CLAUDE.md"), "# shared\n").unwrap();
        std::os::unix::fs::symlink("CLAUDE.md", root.join("AGENTS.md")).unwrap();
        let empty_path = tempfile::tempdir().unwrap();

        let out = run(root, verb, &[("PATH", empty_path.path().as_os_str())]);

        assert!(
            root.join(CODEX_HOOKS).is_file(),
            "{verb:?} wired codex: {out}"
        );
        assert!(
            !root.join(".claude/settings.json").exists(),
            "{verb:?} did not wire claude: {out}"
        );
    }
}
