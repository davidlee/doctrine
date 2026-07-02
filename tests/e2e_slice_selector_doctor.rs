// SPDX-License-Identifier: GPL-3.0-only
//! SL-190 PHASE-06 VT-2 — `doctrine slice selector doctor <id>` as a BLACK-BOX
//! golden over the BUILT binary against a temp git repo (`-p/--path`).
//!
//! Covers what the pure `conformance::diagnose_selector` unit cannot: the shell's
//! `git ls-files` universe read, the authored-selector + intent read, and the
//! `--assert` `std::process::exit` gate. The fixture seeds a MATCHED design-target
//! selector (`src/a.rs`, healthy) plus a STALE unmatched one (`docs/ghost.md`),
//! then asserts advisory exit 0 vs `--assert` non-zero.
//!
//! Selectors are seeded as authored `[[selector]]` TOML directly (a plain fs
//! write, like the catalog fixture) — the confined dispatch worktree's worker
//! guard refuses `slice selector add`, and `doctor` is the read-only subject
//! under test anyway. Matching is plain raw substring (POL-002).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

fn git(dir: &Path, args: &[&str]) {
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
}

/// One authored `[[selector]]` block (`selector`/`intent`).
fn selector_block(glob: &str, intent: &str) -> String {
    format!("\n[[selector]]\nselector = \"{glob}\"\nintent = \"{intent}\"\n")
}

/// A temp git repo with a minimal SL-001 (carrying `selectors` as authored TOML)
/// and two tracked source files, so `git ls-files` gives the doctor a real
/// universe to match selectors against.
fn fixture_root(selectors: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.email", "t@example.com"]);
    git(root, &["config", "user.name", "Test"]);

    let slice = root.join(".doctrine/slice/001");
    std::fs::create_dir_all(&slice).expect("mkdir slice");
    let mut toml = "id = 1\nslug = \"s1\"\ntitle = \"S1\"\nstatus = \"proposed\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
        .to_string();
    for (glob, intent) in selectors {
        toml.push_str(&selector_block(glob, intent));
    }
    std::fs::write(slice.join("slice-001.toml"), toml).expect("slice-001.toml");
    std::fs::write(slice.join("slice-001.md"), "scope\n").expect("slice-001.md");

    std::fs::create_dir_all(root.join("src")).expect("mkdir src");
    std::fs::write(root.join("src/a.rs"), "// a\n").expect("a.rs");
    std::fs::write(root.join("src/b.rs"), "// b\n").expect("b.rs");

    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "seed"]);

    dir
}

/// Run `doctrine slice selector doctor 1 [--assert] -p <root>`.
fn doctor(root: &Path, assert: bool) -> Output {
    let mut cmd = Command::new(bin());
    cmd.args(["slice", "selector", "doctor", "1"]);
    if assert {
        cmd.arg("--assert");
    }
    cmd.arg("-p").arg(root);
    cmd.output().expect("spawn selector doctor")
}

/// A matched selector plus a stale (unmatched) one: the advisory report exits 0
/// and names the unmatched selector.
#[test]
fn advisory_doctor_reports_stale_selector_and_exits_zero() {
    let root = fixture_root(&[
        ("src/a.rs", "design-target"),
        ("docs/ghost.md", "design-target"),
    ]);

    let out = doctor(root.path(), false);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        out.status.success(),
        "advisory doctor is exit 0 regardless of findings: {out:?}"
    );
    assert!(
        stdout.contains("unmatched"),
        "the stale selector must surface as unmatched: {stdout}"
    );
    assert!(
        stdout.contains("docs/ghost.md"),
        "the offending selector string must be named: {stdout}"
    );
}

/// `--assert` turns the stale finding into a non-zero exit gate.
#[test]
fn assert_doctor_exits_nonzero_on_a_stale_selector() {
    let root = fixture_root(&[
        ("src/a.rs", "design-target"),
        ("docs/ghost.md", "design-target"),
    ]);

    let out = doctor(root.path(), true);
    assert!(
        !out.status.success(),
        "--assert must exit non-zero on any finding: {out:?}"
    );
}

/// A clean slice — every selector matches — is healthy under `--assert` (exit 0).
#[test]
fn assert_doctor_exits_zero_when_all_selectors_match() {
    let root = fixture_root(&[
        ("src/a.rs", "design-target"),
        ("src/b.rs", "scope-relevant"),
    ]);

    let out = doctor(root.path(), true);
    assert!(
        out.status.success(),
        "no findings → --assert stays green: {out:?} / {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
