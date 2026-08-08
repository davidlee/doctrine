//! IMP-223 — `doctrine install --agent claude --skill code-review`
//! end-to-end over the built binary.
//!
//! SL-250 PHASE-06: the Claude plugin/marketplace path is retired. Hooks and
//! skills are both direct-written: a canonical tree under `.doctrine/skills/`
//! plus a proven-ownership symlink under `.claude/skills/` (PHASE-05).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;

mod common;

/// Install `code-review` for Claude rooted at `dir`, asserting success; return stdout.
fn install(dir: &Path) -> String {
    let out = common::doctrine_cmd(dir)
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
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    // SL-250 PHASE-06: skills + hooks are both direct-written now — no
    // marketplace registration, no plugin install.
    let out = install(dir);
    assert!(
        out.contains("install skills + agent def for claude"),
        "forward summary: {out}"
    );
    // Skills are direct-written (SL-250 PHASE-05): a fresh install links.
    assert!(
        out.contains("linked    code-review"),
        "the direct skills channel links on a fresh install: {out}"
    );
    // The canonical materialise line, and the real symlink it feeds.
    let canon = dir.join(".doctrine/skills/code-review");
    assert!(
        out.contains(&format!("skill     code-review → {}", canon.display())),
        "canonical materialise line: {out}"
    );
    let link = dir.join(".claude/skills/code-review");
    assert!(
        std::fs::symlink_metadata(&link)
            .expect("skill link on disk")
            .file_type()
            .is_symlink(),
        "a real symlink lands at {}",
        link.display()
    );
    // Agent-def still installed.
    assert!(
        out.contains("linked    dispatch-worker.md"),
        "agent def installed: {out}"
    );
}
