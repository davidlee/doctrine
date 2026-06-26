//! SL-088 PHASE-04 — `doctrine install --agent claude --skill code-review`
//! end-to-end over the built binary.
//!
//! Drives the consolidated `doctrine install` against a temp project: the Claude
//! path materialises a canonical `.doctrine/skills/<id>` tree and links
//! `.claude/skills/<id>` into it. Proves the slice's whole point — a re-install
//! refreshes a stale canonical (the silent no-op the old copy-and-skip dropped) —
//! and that a real-dir override survives + is reported `kept`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::Command;

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

/// Install `code-review` for Claude rooted at `dir`, asserting success; return stdout.
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

#[test]
fn install_links_then_refreshes_and_keeps_an_override() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    let link = dir.join(".claude/skills/code-review");
    let canon = dir.join(".doctrine/skills/code-review");

    // 1. First install: a relative symlink into the materialised canonical tree.
    let out = install(dir);
    assert!(
        out.contains("linked    code-review"),
        "first install links: {out}"
    );
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "agent path is a symlink"
    );
    assert!(
        link.join("SKILL.md").is_file(),
        "link resolves to canonical content"
    );
    assert!(canon.join("SKILL.md").is_file(), "canonical materialised");
    // The derived tree self-enforces its gitignore (F4), no prior `doctrine install`.
    let gi = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(
        gi.contains(".doctrine/skills/*"),
        "skills install ignores its tree"
    );

    // 2. Re-install refreshes a stale canonical — the no-op the old skip dropped.
    fs::write(canon.join("STALE.md"), "old").unwrap();
    let out = install(dir);
    assert!(
        !canon.join("STALE.md").exists(),
        "re-install refreshes the canonical (stale file gone): {out}"
    );
    assert!(
        out.contains("refreshed code-review") && out.contains("relinked  code-review"),
        "re-install refreshes + relinks our own link: {out}"
    );

    // 3. Override: replace the managed link with a real copy → kept, untouched.
    fs::remove_file(&link).unwrap();
    fs::create_dir_all(&link).unwrap();
    fs::write(link.join("MINE.md"), "pinned").unwrap();
    let out = install(dir);
    assert!(
        out.contains("kept      code-review (real dir)"),
        "a real-dir override is reported kept: {out}"
    );
    assert!(
        !fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the override is still a real dir, not relinked"
    );
    assert_eq!(
        fs::read_to_string(link.join("MINE.md")).unwrap(),
        "pinned",
        "the user's pinned copy is intact"
    );
}
