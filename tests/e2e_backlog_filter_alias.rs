//! SL-025 PHASE-04 A-7 — end-to-end over the built binary.
//!
//! `backlog list` keeps the legacy `[SUBSTR]` positional as a DEPRECATED alias of
//! `--filter`. The precedence lives in the clap dispatch (main.rs), so it is only
//! reachable through the real binary: `--filter` WINS when both are given; the
//! positional folds into the substr only when `--filter` is absent (documented
//! precedence, never an error).

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

/// Create a backlog issue with the given title under `dir`.
fn new_issue(dir: &Path, title: &str) {
    let out = Command::new(bin())
        .args(["backlog", "new", "issue", title, "-p"])
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "backlog new failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Run `backlog list` with the given extra args, returning stdout.
fn list(dir: &Path, extra: &[&str]) -> String {
    let mut cmd = Command::new(bin());
    cmd.args(["backlog", "list"]).args(extra).arg("-p").arg(dir);
    let out = cmd.output().expect("spawn doctrine");
    assert!(
        out.status.success(),
        "backlog list failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

#[test]
fn positional_substr_is_a_deprecated_alias_of_filter_which_wins() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    new_issue(dir, "Auth bug"); // ISS-001
    new_issue(dir, "Login flow"); // ISS-002

    // 1. The bare positional behaves as a substr filter (the legacy surface).
    let positional = list(dir, &["auth"]);
    assert!(
        positional.contains("ISS-001"),
        "positional filters: {positional}"
    );
    assert!(
        !positional.contains("ISS-002"),
        "positional excludes the non-match: {positional}"
    );

    // 2. `--filter` alone behaves identically (the canonical surface).
    let flag = list(dir, &["--filter", "login"]);
    assert!(flag.contains("ISS-002"), "--filter filters: {flag}");
    assert!(!flag.contains("ISS-001"), "{flag}");

    // 3. A-7: when BOTH are given, `--filter` WINS — the positional is ignored.
    let both = list(dir, &["auth", "--filter", "login"]);
    assert!(
        both.contains("ISS-002") && !both.contains("ISS-001"),
        "--filter wins over the positional alias (A-7): {both}"
    );
}
