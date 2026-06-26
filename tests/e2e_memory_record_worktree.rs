// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine memory record` on a linked worktree emits the ADR-006 squash-orphan
//! warning to stderr but STILL records (non-blocking, D6a — VT-2); recording on
//! the primary tree is silent (VT-3). SL-032 PHASE-04.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

/// Run `git -C <dir> <args>`, asserting success.
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

/// A source repo with a `.doctrine/` marker and a tracked base commit. The
/// linked worktree inherits the marker, so `root::find` resolves there.
fn init_source(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir.join(".doctrine")).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    fs::write(dir.join(".doctrine/.keep"), "").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
    fs::canonicalize(dir).unwrap()
}

/// Add a linked sibling worktree at `fork`, branch `feat`.
fn add_worktree(source: &Path, fork: &Path) {
    git(
        source,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feat",
            fork.to_str().unwrap(),
        ],
    );
}

fn record(root: &Path) -> Output {
    Command::new(bin())
        .args(["memory", "record", "--type", "fact", "A fact", "-p"])
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

#[test]
fn record_on_a_linked_worktree_warns_but_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));
    let fork = tmp.path().join("fork");
    add_worktree(&source, &fork);
    let fork = fs::canonicalize(&fork).unwrap();

    let out = record(&fork);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(out.status.success(), "record must still succeed: {stderr}");
    assert!(
        stderr.contains("worktree"),
        "expected a worktree warning on stderr, got: {stderr}"
    );
    assert!(
        fork.join(".doctrine/memory/items").is_dir(),
        "the item must have been written"
    );
}

#[test]
fn record_on_the_primary_tree_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));

    let out = record(&source);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(out.status.success(), "record must succeed: {stderr}");
    assert!(
        !stderr.contains("worktree"),
        "the primary tree must record silently, got: {stderr}"
    );
}
