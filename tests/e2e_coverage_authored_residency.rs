//! SL-042 PHASE-02 VT-4 — coverage.toml rides the slice tree's default-track.
//!
//! A coverage entry is authored-tier evidence stored slice-side at
//! `.doctrine/slice/NNN/coverage.toml`. Under THIS repo's blanket `.doctrine/*` +
//! per-tree negation model, the slice tree is negated (`!.doctrine/slice/`) with
//! only `phases`/`handover.md`/`inquisition.md` carved back out — so a plain
//! `coverage.toml` is tracked by DEFAULT with NO negation row of its own. This test
//! pins that: lay the repo's REAL `.gitignore` into a temp git repo, write a
//! rendered coverage.toml, and assert `git check-ignore` exits non-zero (NOT
//! ignored = tracked). If this needed a fresh negation row, the residency model
//! would be wrong. Mirrors `e2e_standard_install_commit.rs`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn git")
}

/// `git check-ignore <path>` exits 0 when the path IS ignored, 1 when it is not.
fn is_ignored(root: &Path, rel: &str) -> bool {
    git(root, &["check-ignore", rel]).status.success()
}

#[test]
fn coverage_toml_rides_the_slice_tree_default_track_untracked_negation() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    git(root, &["init", "-q"]);

    // Lay down THIS repo's REAL authored .gitignore — the negation model under test.
    // Cargo runs tests with CWD = crate root, so the repo's .gitignore is here.
    let real_gitignore = std::fs::read_to_string(".gitignore").expect("read repo .gitignore");
    std::fs::write(root.join(".gitignore"), real_gitignore).unwrap();

    // Write a rendered coverage.toml into the slice tree.
    let rel = ".doctrine/slice/042/coverage.toml";
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        "[[entry]]\nslice = \"SL-042\"\nrequirement = \"REQ-109\"\n\
         contributing_change = \"SL-042\"\nmode = \"VT\"\nstatus = \"verified\"\n\
         git_anchor = \"anchor-abc123\"\n",
    )
    .unwrap();

    assert!(
        !is_ignored(root, rel),
        "{rel} must NOT be ignored — coverage rides the slice-tree default-track \
         (!.doctrine/slice/) with no negation row of its own"
    );

    // Surface 2: git add actually stages it (not just check-ignore agreement).
    let add = git(root, &["add", rel]);
    assert!(
        add.status.success(),
        "git add must succeed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
}
