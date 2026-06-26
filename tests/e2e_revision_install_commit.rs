//! SL-066 PHASE-02 VT-4 — the REV install surfaces, end to end (the `adr` trap).
//!
//! A new authored kind is silently broken on two axes
//! (`mem.pattern.install.authored-entity-wiring`): the manifest must scaffold its
//! tree, and — under THIS repo's blanket `.doctrine/*` + per-tree negation model —
//! the tree must be negated or a scaffolded entity is `git add`-rejected, invisibly
//! uncommittable. These pin both: a fresh install creates `.doctrine/revision`, and
//! a scaffolded `revision-NNN.toml` is committable under the negation (and provably
//! NOT without it). Plus the dogfood sentinel: this repo's own `.gitignore` carries
//! the negation.

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

// --- Surface 1: the manifest scaffolds the authored tree -----------------

#[test]
fn fresh_install_scaffolds_the_revision_tree() {
    let repo = git_repo();
    let root = repo.path();

    let out = doctrine(root, &["install", "-y"]);
    assert!(
        out.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        root.join(".doctrine/revision").is_dir(),
        "fresh install must scaffold .doctrine/revision (manifest [dirs].create)"
    );
}

// --- Surfaces 2 & 3: the negation closes the silent-uncommittable trap ----

#[test]
fn a_scaffolded_revision_is_committable_under_the_blanket_negation_model() {
    let repo = git_repo();
    let root = repo.path();

    // Reproduce THIS repo's dogfood model: blanket-ignore .doctrine/* with a
    // per-tree negation for the authored revision tree (surface 2).
    std::fs::write(
        root.join(".gitignore"),
        ".doctrine/*\n!.doctrine/revision/\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join(".doctrine/revision")).unwrap();

    let out = doctrine(root, &["revision", "new", "revise ADR-006"]);
    assert!(
        out.status.success(),
        "revision new failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let toml_rel = ".doctrine/revision/001/revision-001.toml";
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
        "revision-001.toml must be staged (Added)"
    );
}

#[test]
fn without_the_negation_the_revision_is_silently_ignored() {
    // The guard bites: drop the negation and the same scaffolded revision becomes
    // uncommittable — proving the trap is real and the negation is load-bearing.
    let repo = git_repo();
    let root = repo.path();
    std::fs::write(root.join(".gitignore"), ".doctrine/*\n").unwrap();
    std::fs::create_dir_all(root.join(".doctrine/revision")).unwrap();

    let out = doctrine(root, &["revision", "new", "revise ADR-006"]);
    assert!(out.status.success(), "revision new failed");

    assert!(
        is_ignored(root, ".doctrine/revision/001/revision-001.toml"),
        "without !.doctrine/revision/ the scaffolded revision is ignored (the trap)"
    );
}

// --- The dogfood guard: THIS repo's own .gitignore carries the negation ---

#[test]
fn this_repos_gitignore_negates_the_revision_tree() {
    // Cargo runs tests with CWD = crate root, so the repo's authored .gitignore is
    // readable directly — a cheap sentinel that the dogfood surface stays wired.
    let gitignore = std::fs::read_to_string(".gitignore").expect("read .gitignore");
    assert!(
        gitignore
            .lines()
            .any(|l| l.trim() == "!.doctrine/revision/"),
        "this repo's .gitignore must negate the authored revision tree"
    );
}
