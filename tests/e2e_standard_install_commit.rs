//! SL-033 PHASE-01 VT-5 — the three standard install surfaces, end to end.
//!
//! A new authored governance kind is silently broken on two axes
//! (`mem.pattern.install.authored-entity-wiring`): the manifest must scaffold its
//! tree, and — under THIS repo's blanket `.doctrine/*` + per-tree negation model —
//! the tree must be negated or a scaffolded entity is `git add`-rejected with
//! "paths are ignored", invisibly uncommittable. These tests pin both: a fresh
//! first `standard new` lazily scaffolds `.doctrine/standard` (SL-227 FR-008 — bare
//! install no longer eagerly creates it), and a scaffolded `standard-NNN.toml` is
//! committable under the negation (and provably NOT, without it — the guard bites).
//! Mirrors `e2e_policy_install_commit.rs`.

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

/// `git check-ignore <path>` exits 0 when the path IS ignored, 1 when it is not.
fn is_ignored(root: &Path, rel: &str) -> bool {
    git(root, &["check-ignore", rel]).status.success()
}

// --- Surface 1: the authored tree is scaffolded lazily on first use ------

#[test]
fn first_scaffold_creates_the_standard_tree() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let repo = git_repo();
    let root = repo.path();

    // SL-227 FR-008: bare install no longer eagerly scaffolds entity roots —
    // it projects only the three-file base. The standard root must be ABSENT here.
    let out = doctrine(root, &["install", "-y"]);
    assert!(
        out.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !root.join(".doctrine/standard").exists(),
        "bare install must NOT eagerly scaffold .doctrine/standard after the minimal-projection flip"
    );

    // First `standard new` materialises the tree lazily (entity.rs materialise*).
    let out = doctrine(root, &["standard", "new", "Two space indent"]);
    assert!(out.status.success(), "standard new failed");
    assert!(
        root.join(".doctrine/standard").is_dir(),
        "first `standard new` must lazily scaffold .doctrine/standard"
    );
}

// --- Surfaces 2 & 3: the negation closes the silent-uncommittable trap ----

#[test]
fn a_scaffolded_standard_is_committable_under_the_blanket_negation_model() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let repo = git_repo();
    let root = repo.path();

    // Reproduce THIS repo's dogfood model: blanket-ignore .doctrine/* with a
    // per-tree negation for the authored standard tree (surface 2).
    std::fs::write(
        root.join(".gitignore"),
        ".doctrine/*\n!.doctrine/standard/\n",
    )
    .unwrap();

    let out = doctrine(root, &["standard", "new", "Two space indent"]);
    assert!(out.status.success(), "standard new failed");

    let toml_rel = ".doctrine/standard/001/standard-001.toml";
    assert!(
        !is_ignored(root, toml_rel),
        "the negation must make {toml_rel} committable"
    );

    // Surface 3: `git add` actually stages it (not just check-ignore agreement).
    let add = git(root, &["add", toml_rel]);
    assert!(
        add.status.success(),
        "git add must succeed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let staged = git(root, &["status", "--porcelain", toml_rel]);
    assert!(
        String::from_utf8_lossy(&staged.stdout).starts_with("A "),
        "standard-001.toml must be staged (Added)"
    );
}

#[test]
fn without_the_negation_the_standard_is_silently_ignored() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    // The guard bites: drop the negation and the same scaffolded standard becomes
    // uncommittable — proving the trap is real and surface 2 is load-bearing.
    let repo = git_repo();
    let root = repo.path();
    std::fs::write(root.join(".gitignore"), ".doctrine/*\n").unwrap();

    let out = doctrine(root, &["standard", "new", "Two space indent"]);
    assert!(out.status.success(), "standard new failed");

    assert!(
        is_ignored(root, ".doctrine/standard/001/standard-001.toml"),
        "without !.doctrine/standard/ the scaffolded standard is ignored (the trap)"
    );
}

// --- The dogfood guard: THIS repo's own .gitignore carries the negation ---

#[test]
fn this_repos_standard_tree_is_tracked() {
    // Cargo runs tests with CWD = crate root. Model-agnostic dogfood sentinel:
    // whatever the .gitignore model, the authored standard tree must actually be
    // tracked — assert git lists it, not that a specific gitignore line is present.
    let out = Command::new("git")
        .args(["ls-files", ".doctrine/standard"])
        .output()
        .expect("run git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    let tracked = String::from_utf8_lossy(&out.stdout);
    assert!(
        tracked.lines().any(|l| !l.trim().is_empty()),
        "this repo's authored standard tree must be tracked (git ls-files .doctrine/standard is empty)"
    );
}
