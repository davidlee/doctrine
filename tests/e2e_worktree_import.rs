// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-07 — `doctrine worktree import --base <B> --fork <branch>` end-to-end
//! over the BUILT binary. v1 = stationary-head case only.
//!
//! * VT-1: happy path DRIVES `run()` — a single non-merge fork commit, HEAD==B,
//!   clean tree → after `import` the coordination tree has the delta STAGED and
//!   UN-committed (`git diff --cached` shows it; HEAD unchanged == B). The belt is
//!   the load-bearing protection, so the invariant knocks on the real CLI wall.
//! * VT-2: each refusal golden exits non-zero with a DISTINCT named token —
//!   head-moved / tree-unclean / multi-commit / doctrine-touch / claude-touch.
//! * VT-3: untracked scratch files do NOT trip tree-unclean (`--untracked-files=no`).
//! * VT-4: import Orchestrator refusal drives `run()` — refused from a marked
//!   linked-worktree fork AND from a DOCTRINE_WORKER-set process.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

fn git(dir: &Path, args: &[&str]) -> String {
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
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A git invocation allowed to fail; returns success + trimmed stdout.
fn git_try(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("spawn git");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    )
}

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

fn stamp_marker(root: &Path) {
    let dir = root.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("worker"), b"").unwrap();
}

/// Run `doctrine <args>` in `cwd`; env governed by `worker` (Some(true) sets
/// DOCTRINE_WORKER=1; None removes it).
fn run(cwd: &Path, worker: Option<bool>, args: &[&str]) -> Output {
    let mut cmd = Command::new(BIN);
    cmd.args(args).current_dir(cwd);
    match worker {
        Some(true) => {
            cmd.env("DOCTRINE_WORKER", "1");
        }
        Some(false) | None => {
            cmd.env_remove("DOCTRINE_WORKER");
        }
    }
    cmd.output().expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// Create a `<branch>` carrying exactly one non-merge commit at `<src>` HEAD==B
/// that writes `path`=`body`, then restore the working tree to B. Uses a linked
/// worktree so the source tree stays at B (the stationary coordination head).
/// Returns `B` (the source HEAD sha).
fn make_fork_branch(src: &Path, holder: &Path, branch: &str, files: &[(&str, &str)]) -> String {
    let base = git(src, &["rev-parse", "HEAD"]);
    let wt = holder.join(branch);
    git(
        src,
        &["worktree", "add", "-b", branch, wt.to_str().unwrap(), &base],
    );
    for (rel, body) in files {
        let p = wt.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, body).unwrap();
        // Force-add so a .claude/ (gitignored-in-real-repos) path is staged too;
        // harmless for ordinary tracked paths.
        git(&wt, &["add", "-f", rel]);
    }
    git(&wt, &["commit", "-q", "-m", &format!("S: {branch}")]);
    // The fork branch now carries S; the source tree is untouched at B.
    base
}

// --- VT-1: happy path DRIVES run() — delta staged, uncommitted, HEAD == B ---

#[test]
fn import_happy_stages_delta_uncommitted() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let base = make_fork_branch(
        src.path(),
        holder.path(),
        "wkr-1",
        &[("feature.rs", "fn f() {}")],
    );

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-1"],
    );
    assert!(
        out.status.success(),
        "happy import must succeed; stderr: {}",
        stderr(&out)
    );

    // HEAD UNCHANGED == B (import != commit, ADR-006 D7).
    assert_eq!(
        git(src.path(), &["rev-parse", "HEAD"]),
        base,
        "import must NOT move HEAD"
    );
    // The delta is STAGED (in the index, uncommitted): the new file shows in
    // `git diff --cached --name-only`.
    let staged = git(src.path(), &["diff", "--cached", "--name-only"]);
    assert!(
        staged.lines().any(|l| l == "feature.rs"),
        "delta staged in the index; got: {staged:?}"
    );
    // And it is genuinely UNcommitted: the file is not present in the B tree.
    let (ok, _) = git_try(
        src.path(),
        &["cat-file", "-e", &format!("{base}:feature.rs")],
    );
    assert!(!ok, "feature.rs must not exist in the B commit");
}

// --- ISS-032: a patch whose final hunk ends at EOF must import cleanly ---
// The diff for a fork that edits the LAST line of a newline-terminated file
// ends on a `+` line; `git apply` rejects the stream ("corrupt patch") if the
// trailing newline is stripped before it reaches stdin. Regression guard for
// the `import` → `git apply` byte path (the delta is captured raw, not trimmed).

#[test]
fn import_applies_patch_ending_at_eof() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    // B carries a multi-line, newline-terminated file.
    std::fs::write(src.path().join("multi.txt"), "l1\nl2\nl3\nl4\nl5\n").unwrap();
    git(src.path(), &["add", "."]);
    git(src.path(), &["commit", "-q", "-m", "seed multi"]);

    let holder = tempfile::tempdir().unwrap();
    // The fork rewrites the FINAL line — the patch's last hunk lands on EOF.
    let base = make_fork_branch(
        src.path(),
        holder.path(),
        "wkr-nl",
        &[("multi.txt", "l1\nl2\nl3\nl4\nMODIFIED\n")],
    );

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-nl"],
    );
    assert!(
        out.status.success(),
        "import of a patch ending at EOF must succeed (ISS-032); stderr: {}",
        stderr(&out)
    );
    let staged = git(src.path(), &["diff", "--cached", "--name-only"]);
    assert!(
        staged.lines().any(|l| l == "multi.txt"),
        "delta staged in the index; got: {staged:?}"
    );
}

// --- VT-2: each refusal golden — distinct non-zero token ---

fn assert_refusal(out: &Output, token: &str) {
    assert!(
        !out.status.success(),
        "must refuse ({token}); stdout: {}, stderr: {}",
        stdout(out),
        stderr(out)
    );
    assert!(
        stderr(out).contains(token),
        "refusal names `{token}`; stderr: {}",
        stderr(out)
    );
}

#[test]
fn import_refuses_head_moved() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let base = make_fork_branch(src.path(), holder.path(), "wkr-hm", &[("f.rs", "x")]);
    // Move coordination HEAD off B.
    std::fs::write(src.path().join("drift.txt"), "drift").unwrap();
    git(src.path(), &["add", "drift.txt"]);
    git(src.path(), &["commit", "-q", "-m", "head moves off B"]);

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-hm"],
    );
    assert_refusal(&out, "head-moved");
}

#[test]
fn import_refuses_tree_unclean() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let base = make_fork_branch(src.path(), holder.path(), "wkr-tu", &[("f.rs", "x")]);
    // Dirty a TRACKED file in the coordination tree.
    std::fs::write(src.path().join("a.txt"), "dirtied").unwrap();

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-tu"],
    );
    assert_refusal(&out, "tree-unclean");
}

#[test]
fn import_refuses_multi_commit() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    // Two commits on the fork ⇒ S^ != B.
    let wt = holder.path().join("wkr-mc");
    git(
        src.path(),
        &[
            "worktree",
            "add",
            "-b",
            "wkr-mc",
            wt.to_str().unwrap(),
            &base,
        ],
    );
    std::fs::write(wt.join("one.rs"), "1").unwrap();
    git(&wt, &["add", "one.rs"]);
    git(&wt, &["commit", "-q", "-m", "first"]);
    std::fs::write(wt.join("two.rs"), "2").unwrap();
    git(&wt, &["add", "two.rs"]);
    git(&wt, &["commit", "-q", "-m", "second"]);

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-mc"],
    );
    assert_refusal(&out, "multi-commit");
}

#[test]
fn import_refuses_doctrine_touch() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let base = make_fork_branch(
        src.path(),
        holder.path(),
        "wkr-dt",
        &[(".doctrine/state/sneaky.txt", "nope")],
    );

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-dt"],
    );
    assert_refusal(&out, "doctrine-touch");
}

#[test]
fn import_refuses_claude_touch() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // make_fork_branch force-adds, so a .claude/ path lands even if gitignored.
    let base = make_fork_branch(
        src.path(),
        holder.path(),
        "wkr-ct",
        &[(".claude/settings.json", "{}")],
    );

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-ct"],
    );
    assert_refusal(&out, "claude-touch");
}

// --- belt hardening: the prefix-match must see the REAL governance path ---

// F-3: a governance path carrying a non-ASCII byte + space is C-quoted by git's
// default core.quotePath=true (".doctrine/\303\251 vil.toml"), so the belt's
// `starts_with(".doctrine/")` misses unless the belt diff pins quotePath=false.
#[test]
fn import_refuses_doctrine_touch_quoted_path() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let base = make_fork_branch(
        src.path(),
        holder.path(),
        "wkr-qp",
        &[(".doctrine/é vil.toml", "nope")],
    );

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-qp"],
    );
    assert_refusal(&out, "doctrine-touch");
}

// F-4: a governance DELETION paired by git's default rename detection with a
// same-content add elsewhere is emitted by `--name-only` as the destination
// only; the `.doctrine/` SOURCE never appears unless the belt diff pins
// --no-renames.
#[test]
fn import_refuses_rename_disguised_doctrine_deletion() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    // Commit a substantial governance file into B so a 100% rename is detected.
    let gov_body = "GOVERNANCE\n".repeat(40);
    std::fs::create_dir_all(src.path().join(".doctrine/state")).unwrap();
    std::fs::write(src.path().join(".doctrine/state/gov.txt"), &gov_body).unwrap();
    git(src.path(), &["add", "-f", ".doctrine/state/gov.txt"]);
    git(src.path(), &["commit", "-q", "-m", "seed governance file"]);
    let base = git(src.path(), &["rev-parse", "HEAD"]);

    // Fork: delete the governance file, re-add identical content elsewhere.
    let holder = tempfile::tempdir().unwrap();
    let wt = holder.path().join("wkr-rn");
    git(
        src.path(),
        &[
            "worktree",
            "add",
            "-b",
            "wkr-rn",
            wt.to_str().unwrap(),
            &base,
        ],
    );
    std::fs::remove_file(wt.join(".doctrine/state/gov.txt")).unwrap();
    std::fs::create_dir_all(wt.join("moved")).unwrap();
    std::fs::write(wt.join("moved/gov.txt"), &gov_body).unwrap();
    git(&wt, &["add", "-A"]);
    git(
        &wt,
        &["commit", "-q", "-m", "S: rename-disguised gov deletion"],
    );

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-rn"],
    );
    assert_refusal(&out, "doctrine-touch");
}

// --- VT-3: untracked scratch files do NOT trip tree-unclean ---

#[test]
fn import_ignores_untracked_scratch() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let base = make_fork_branch(src.path(), holder.path(), "wkr-ut", &[("g.rs", "ok")]);
    // An untracked scratch file in the coordination tree — must NOT trip the guard.
    std::fs::write(src.path().join("scratch.tmp"), "ephemeral").unwrap();

    let out = run(
        src.path(),
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-ut"],
    );
    assert!(
        out.status.success(),
        "untracked scratch must NOT trip tree-unclean; stderr: {}",
        stderr(&out)
    );
    let staged = git(src.path(), &["diff", "--cached", "--name-only"]);
    assert!(
        staged.lines().any(|l| l == "g.rs"),
        "delta still staged; got: {staged:?}"
    );
}

// --- VT-4: Orchestrator refusal drives the real CLI ---

/// Make a real linked worktree fork of `src` and return its path (mirrors the
/// fork-verb test's marked-fork driver).
fn add_linked_fork(src: &Path, holder: &Path, branch: &str) -> PathBuf {
    let base = git(src, &["rev-parse", "HEAD"]);
    let fork = holder.join("linked");
    git(
        src,
        &[
            "worktree",
            "add",
            "-b",
            branch,
            fork.to_str().unwrap(),
            &base,
        ],
    );
    fork
}

#[test]
fn import_refused_under_worker_mode() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr-guard");

    // (1) Marked linked worktree, env unset ⇒ refused (signal: marker), names verb.
    stamp_marker(&fork);
    let out = run(
        &fork,
        None,
        &["worktree", "import", "--base", &base, "--fork", "wkr-guard"],
    );
    assert!(
        !out.status.success(),
        "import refused from a marked linked worktree; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("import"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );

    // (2) DOCTRINE_WORKER set ⇒ refused before any import work.
    let out = run(
        src.path(),
        Some(true),
        &["worktree", "import", "--base", &base, "--fork", "wkr-guard"],
    );
    assert!(
        !out.status.success(),
        "import refused when DOCTRINE_WORKER set; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "env carries the dual-cause; stderr: {}",
        stderr(&out)
    );
}
