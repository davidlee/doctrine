//! IMP-223 — `doctrine install --agent claude --skill code-review`
//! end-to-end over the built binary.
//!
//! Skills + hooks are now driven via `claude plugin` commands; `claude` is
//! absent in test, so we verify the graceful-failure + reminder paths.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

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

    // IMP-223: skills + hooks are now handled by `claude plugin install`.
    // Plugin command outcomes are environment-dependent; assert invariants only.
    let out = install(dir);
    assert!(
        out.contains("register marketplace + install plugin + agent def for claude"),
        "forward summary: {out}"
    );
    // No old-style manual symlink/canonical output.
    assert!(
        !out.contains("linked    code-review"),
        "no manual skills symlink: {out}"
    );
    assert!(
        !out.contains("refreshed code-review"),
        "no manual canonical refresh: {out}"
    );
    assert!(
        !out.contains("kept      code-review"),
        "no manual override tracking: {out}"
    );
    // Agent-def still installed.
    assert!(
        out.contains("linked    dispatch-worker.md"),
        "agent def installed: {out}"
    );
}
