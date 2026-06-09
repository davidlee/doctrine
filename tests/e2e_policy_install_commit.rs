//! SL-030 PHASE-03 VT-2 — the three policy install surfaces, end to end.
//!
//! A new authored governance kind is silently broken on two axes
//! (`mem.pattern.install.authored-entity-wiring`): the manifest must scaffold its
//! tree, and — under THIS repo's blanket `.doctrine/*` + per-tree negation model —
//! the tree must be negated or a scaffolded entity is `git add`-rejected with
//! "paths are ignored", invisibly uncommittable. These tests pin both: a fresh
//! install creates `.doctrine/policy`, and a scaffolded `policy-NNN.toml` is
//! committable under the negation (and provably NOT, without it — the guard bites).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

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
    Command::new(BIN)
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
fn fresh_install_scaffolds_the_policy_tree() {
    let repo = git_repo();
    let root = repo.path();

    let out = doctrine(root, &["install", "-y"]);
    assert!(
        out.status.success(),
        "install failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        root.join(".doctrine/policy").is_dir(),
        "fresh install must scaffold .doctrine/policy (manifest [dirs].create)"
    );
}

// --- Surfaces 2 & 3: the negation closes the silent-uncommittable trap ----

#[test]
fn a_scaffolded_policy_is_committable_under_the_blanket_negation_model() {
    let repo = git_repo();
    let root = repo.path();

    // Reproduce THIS repo's dogfood model: blanket-ignore .doctrine/* with a
    // per-tree negation for the authored policy tree (surface 2).
    std::fs::write(root.join(".gitignore"), ".doctrine/*\n!.doctrine/policy/\n").unwrap();

    let out = doctrine(root, &["policy", "new", "Two space indent"]);
    assert!(out.status.success(), "policy new failed");

    let toml_rel = ".doctrine/policy/001/policy-001.toml";
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
        "policy-001.toml must be staged (Added)"
    );
}

#[test]
fn without_the_negation_the_policy_is_silently_ignored() {
    // The guard bites: drop the negation and the same scaffolded policy becomes
    // uncommittable — proving the trap is real and surface 2 is load-bearing.
    let repo = git_repo();
    let root = repo.path();
    std::fs::write(root.join(".gitignore"), ".doctrine/*\n").unwrap();

    let out = doctrine(root, &["policy", "new", "Two space indent"]);
    assert!(out.status.success(), "policy new failed");

    assert!(
        is_ignored(root, ".doctrine/policy/001/policy-001.toml"),
        "without !.doctrine/policy/ the scaffolded policy is ignored (the trap)"
    );
}

// --- The dogfood guard: THIS repo's own .gitignore carries the negation ---

#[test]
fn this_repos_gitignore_negates_the_policy_tree() {
    // Cargo runs tests with CWD = crate root, so the repo's authored .gitignore
    // is readable directly — a cheap sentinel that the dogfood surface stays wired.
    let gitignore = std::fs::read_to_string(".gitignore").expect("read .gitignore");
    assert!(
        gitignore.lines().any(|l| l.trim() == "!.doctrine/policy/"),
        "this repo's .gitignore must negate the authored policy tree"
    );
}
