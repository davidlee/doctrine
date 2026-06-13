// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-08 — `doctrine worktree land --fork <branch>` end-to-end over the
//! BUILT binary (design §6). Solo `/execute`'s non-squash coordination merge.
//!
//! * VT-1: happy path DRIVES `run()` — a solo MULTI-commit fork branch WITH a live
//!   linked worktree, no marker, clean coordination tree → `land` succeeds with a
//!   `--no-ff` MERGE commit (2 parents), the fork tip is an ANCESTOR of HEAD, and
//!   the verb has NO `--squash` flag (structurally impossible).
//! * VT-2: precond refusals — distinct named tokens: tree-unclean / no-such-fork /
//!   worktree-gone / dispatch-fork.
//! * VT-3: merge-time refusals — merge-conflict (with abort-first ⇒ clean tree) and
//!   inconsistent-merge-state (merge fails leaving no MERGE_HEAD). wedged-merge is
//!   not deterministically black-box reproducible (it needs `git merge --abort`
//!   ITSELF to fail, i.e. corrupted git state); its token is pinned by a focused
//!   unit test of the refusal table instead (see the note at that test).
//! * VT-4: land Orchestrator refusal DRIVES `run()` — refused from a marked
//!   linked-worktree fork (names the verb `land`) AND from a DOCTRINE_WORKER-set
//!   process (carries the dual-cause). Mirrors `import_refused_under_worker_mode`.

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

/// Create `<branch>` as a SOLO multi-commit fork via a live linked worktree at
/// `holder/<branch>`, off `<src>` HEAD. Each entry in `commits` is `(rel, body)`
/// committed as its own commit (≥2 ⇒ genuine multi-commit ancestry). The source
/// tree is left at its current HEAD. Returns the live linked worktree path.
fn make_solo_fork(
    src: &Path,
    holder: &Path,
    branch: &str,
    commits: &[(&str, &str)],
) -> PathBuf {
    let base = git(src, &["rev-parse", "HEAD"]);
    let wt = holder.join(branch);
    git(
        src,
        &["worktree", "add", "-b", branch, wt.to_str().unwrap(), &base],
    );
    for (rel, body) in commits {
        let p = wt.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, body).unwrap();
        git(&wt, &["add", "-f", rel]);
        git(&wt, &["commit", "-q", "-m", &format!("S: {branch} {rel}")]);
    }
    wt
}

/// A bare branch with NO live linked worktree: create it from HEAD, then remove
/// its worktree so only the branch ref survives.
fn make_worktreeless_branch(src: &Path, holder: &Path, branch: &str) {
    let wt = make_solo_fork(src, holder, branch, &[("only.rs", "x")]);
    git(src, &["worktree", "remove", "--force", wt.to_str().unwrap()]);
}

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

// --- VT-1: happy path DRIVES run() — --no-ff merge, ancestry preserved ---

#[test]
fn land_happy_no_ff_merge_preserves_ancestry() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // Solo MULTI-commit fork (two commits) with a live linked worktree, no marker.
    make_solo_fork(
        src.path(),
        holder.path(),
        "solo-1",
        &[("one.rs", "fn a() {}"), ("two.rs", "fn b() {}")],
    );
    let fork_tip = git(src.path(), &["rev-parse", "solo-1"]);

    let out = run(src.path(), None, &["worktree", "land", "--fork", "solo-1"]);
    assert!(
        out.status.success(),
        "happy land must succeed; stderr: {}",
        stderr(&out)
    );

    // HEAD is a --no-ff MERGE commit: exactly two parents.
    let parents = git(src.path(), &["rev-list", "--parents", "-n1", "HEAD"]);
    assert_eq!(
        parents.split_whitespace().count(),
        3,
        "merge commit = self + 2 parents (3 fields); got: {parents:?}"
    );
    let parent_lines = git(src.path(), &["cat-file", "-p", "HEAD"]);
    assert_eq!(
        parent_lines.lines().filter(|l| l.starts_with("parent ")).count(),
        2,
        "merge commit has two parent lines; got: {parent_lines}"
    );

    // The fork tip is an ANCESTOR of the new HEAD (ancestry preserved ⇒ reachable).
    let (is_ancestor, _) = git_try(
        src.path(),
        &["merge-base", "--is-ancestor", &fork_tip, "HEAD"],
    );
    assert!(is_ancestor, "fork tip must be an ancestor of HEAD");

    // The fork's files are present in the merged tree.
    let (ok, _) = git_try(src.path(), &["cat-file", "-e", "HEAD:two.rs"]);
    assert!(ok, "fork's files land in the coordination tree");
}

#[test]
fn land_verb_has_no_squash_flag() {
    // The verb must not be able to EXPRESS a squash (design §6: --no-ff only). An
    // unknown flag is rejected by clap ⇒ non-zero, proving --squash is not wired.
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    make_solo_fork(src.path(), holder.path(), "solo-sq", &[("f.rs", "x")]);
    let out = run(
        src.path(),
        None,
        &["worktree", "land", "--fork", "solo-sq", "--squash"],
    );
    assert!(
        !out.status.success(),
        "--squash must NOT be a valid flag; stdout: {}",
        stdout(&out)
    );
}

// --- VT-2: precond refusals — distinct named tokens ---

#[test]
fn land_refuses_tree_unclean() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    make_solo_fork(src.path(), holder.path(), "solo-tu", &[("f.rs", "x")]);
    // Dirty a TRACKED file in the coordination tree.
    std::fs::write(src.path().join("a.txt"), "dirtied").unwrap();

    let out = run(src.path(), None, &["worktree", "land", "--fork", "solo-tu"]);
    assert_refusal(&out, "tree-unclean");
}

#[test]
fn land_refuses_no_such_fork() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());

    let out = run(
        src.path(),
        None,
        &["worktree", "land", "--fork", "ghost-branch"],
    );
    assert_refusal(&out, "no-such-fork");
}

#[test]
fn land_refuses_worktree_gone() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // Branch exists, but its worktree is removed ⇒ no live linked worktree.
    make_worktreeless_branch(src.path(), holder.path(), "solo-wg");

    let out = run(src.path(), None, &["worktree", "land", "--fork", "solo-wg"]);
    assert_refusal(&out, "worktree-gone");
}

#[test]
fn land_refuses_dispatch_fork() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // A live linked worktree that BEARS the worker marker ⇒ dispatch worker.
    let wt = make_solo_fork(src.path(), holder.path(), "solo-df", &[("f.rs", "x")]);
    stamp_marker(&wt);

    let out = run(src.path(), None, &["worktree", "land", "--fork", "solo-df"]);
    assert_refusal(&out, "dispatch-fork");
}

// --- VT-3: merge-time refusals ---

#[test]
fn land_merge_conflict_aborts_first_then_refuses() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // Fork changes a.txt one way (live linked worktree, no marker).
    make_solo_fork(src.path(), holder.path(), "solo-cf", &[("a.txt", "fork-side")]);
    // Coordination side changes the SAME file the other way ⇒ guaranteed conflict.
    std::fs::write(src.path().join("a.txt"), "coord-side").unwrap();
    git(src.path(), &["add", "a.txt"]);
    git(src.path(), &["commit", "-q", "-m", "coord-side change"]);

    let out = run(src.path(), None, &["worktree", "land", "--fork", "solo-cf"]);
    assert_refusal(&out, "merge-conflict");

    // The abort ran FIRST: the coordination tree is CLEAN and no MERGE_HEAD remains.
    let status = git(src.path(), &["status", "--porcelain"]);
    assert!(status.is_empty(), "tree must be clean after abort; got: {status:?}");
    let (mh, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "MERGE_HEAD"]);
    assert!(!mh, "no MERGE_HEAD must remain after the abort");
}

#[test]
fn land_inconsistent_merge_state_on_unrelated_histories() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // Build a fork branch with a live linked worktree but an UNRELATED history:
    // git refuses to even begin the merge (no MERGE_HEAD) ⇒ inconsistent-merge-state.
    let base = git(src.path(), &["rev-parse", "HEAD"]);
    let wt = holder.path().join("solo-ums");
    git(
        src.path(),
        &["worktree", "add", "--detach", wt.to_str().unwrap(), &base],
    );
    git(&wt, &["checkout", "-q", "--orphan", "solo-ums"]);
    git(&wt, &["rm", "-rfq", "."]);
    std::fs::write(wt.join("other.rs"), "x").unwrap();
    git(&wt, &["add", "other.rs"]);
    git(&wt, &["commit", "-q", "-m", "orphan root"]);

    let out = run(src.path(), None, &["worktree", "land", "--fork", "solo-ums"]);
    assert_refusal(&out, "inconsistent-merge-state");
    // And the tree is left clean — no MERGE_HEAD, nothing half-applied.
    let status = git(src.path(), &["status", "--porcelain"]);
    assert!(status.is_empty(), "tree clean after inconsistent-merge-state; got: {status:?}");
}

// VT-3 (wedged-merge): the wedged-merge refusal fires ONLY when `git merge
// --abort` itself fails after a conflicted merge — i.e. corrupted/locked git
// internals, which cannot be produced deterministically from a black-box CLI
// test without racy index-lock contention. Per the worker contract's fallback,
// its token is pinned by a focused unit test of the refusal table; the branch's
// reachability is evident in `run_land` (the `else` of a successful abort).
// (The LandRefusal table lives in the crate, exercised by the crate's unit
// tests; this e2e file documents the deliberate gap so the next reader knows it
// is intentional, not missing.)

// --- VT-4: Orchestrator refusal DRIVES run() ---

/// Make a real linked worktree fork of `src` and return its path.
fn add_linked_fork(src: &Path, holder: &Path, branch: &str) -> PathBuf {
    let base = git(src, &["rev-parse", "HEAD"]);
    let fork = holder.join("linked");
    git(
        src,
        &["worktree", "add", "-b", branch, fork.to_str().unwrap(), &base],
    );
    fork
}

#[test]
fn land_refused_under_worker_mode() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "wkr-guard");

    // (1) Marked linked worktree, env unset ⇒ refused (signal: marker), names verb.
    stamp_marker(&fork);
    let out = run(&fork, None, &["worktree", "land", "--fork", "wkr-guard"]);
    assert!(
        !out.status.success(),
        "land refused from a marked linked worktree; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("land"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );

    // (2) DOCTRINE_WORKER set ⇒ refused before any land work, carries dual-cause.
    let out = run(
        src.path(),
        Some(true),
        &["worktree", "land", "--fork", "wkr-guard"],
    );
    assert!(
        !out.status.success(),
        "land refused when DOCTRINE_WORKER set; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "env carries the dual-cause; stderr: {}",
        stderr(&out)
    );
}
