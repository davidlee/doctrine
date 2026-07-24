//! SL-223 PHASE-02 VT-6 — publication is NOT projection.
//!
//! The publication manifest lives in its own `publication/` embed root, which
//! install's projection (`build_plan`, which walks `InstallAssets` only) never
//! touches. A fresh install into a clean repo must therefore leave
//! `.doctrine/publication/` ABSENT — the projection and publication manifests are
//! independent surfaces (REQ-380 / D-C / R4 / codex RV-286 F1). This is the
//! regression proof for the blanket-projection footgun that reusing the `install/`
//! root would have re-introduced.

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

/// A throwaway git repo with identity configured, so `git add` works headless.
fn git_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "t@t"]);
    git(root, &["config", "user.name", "t"]);
    dir
}

fn git(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn git")
}

fn doctrine(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

#[test]
fn install_does_not_project_the_publication_manifest() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let repo = git_repo();
    let root = repo.path();

    let out = doctrine(root, &["install", "-y"]);
    assert!(
        out.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Sanity: install genuinely ran and projected the install surface.
    assert!(
        root.join(".doctrine").is_dir(),
        "install must scaffold .doctrine/"
    );

    // The point (REQ-380 / D-C): the publication manifest's separate embed root is
    // outside projection, so nothing publication-shaped reaches the client repo.
    assert!(
        !root.join(".doctrine/publication").exists(),
        "publication manifest must NOT be projected — publication is not projection (REQ-380)"
    );
    assert!(
        !root.join(".doctrine/publication/manifest.toml").exists(),
        "publication/manifest.toml must not appear in a client repo"
    );
}
