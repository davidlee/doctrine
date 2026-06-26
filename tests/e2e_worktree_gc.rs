// SPDX-License-Identifier: GPL-3.0-only
//! SL-056 PHASE-09 — `doctrine worktree gc --fork <branch>` end-to-end over the
//! BUILT binary (design §8/§8.1/§8.2). The idempotent spent-fork reaper + the
//! two-leg (ancestry ∪ patch-id) durable-git landed oracle.
//!
//! * VT-1: a positive verdict reaps worktree+branch (the fork's in-tree `target/`
//!   dies with the worktree dir — SL-156, no separate target leg). Two legs: the
//!   ANCESTOR leg (a landed-via-`land` fork — `merge --no-ff` so the tip is an
//!   ancestor of HEAD) and the all-`-` PATCH-ID leg (a landed-via-`import` fork —
//!   apply the fork's diff + commit at coord HEAD so ancestry is severed but every
//!   patch landed). A non-ancestor tip with a `+` (unlanded) refuses UNLESS
//!   `--superseded-head <head>` / `--force`.
//! * VT-2: a SQUASH-merged fork (trips neither leg) → NAMED `squash-uncertifiable`
//!   refusal mentioning `worktree land` / `--no-ff` / `--force`; `--superseded-head`
//!   honesty (reaps iff SHA == current head; a WRONG SHA refuses → safe side);
//!   `--dry-run` prints the verdict and destroys NOTHING.
//! * VT-3 (idempotent rerun): crash AFTER each destructive step completes / names a
//!   leftover on rerun — the W-removed-before-B ordering case, a fully-reaped no-op,
//!   and a stale admin worktree entry folded via `git worktree prune`.
//! * VT-4 (EXHAUSTIVE Orchestrator refusal): from a marked linked-worktree fork
//!   (env unset) AND from a DOCTRINE_WORKER-set process, EACH of fork/import/land/gc
//!   is refused. `marker --clear` is deliberately OUT of this class.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

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
/// DOCTRINE_WORKER=1; None removes it). CARGO_TARGET_DIR is removed so the spawned
/// binary inherits no jail target redirect — the fork's `target/` is in-tree
/// (SL-156); gc no longer reads CARGO_TARGET_DIR at all.
fn run(cwd: &Path, worker: Option<bool>, args: &[&str]) -> Output {
    let mut cmd = Command::new(bin());
    cmd.args(args).current_dir(cwd);
    cmd.env_remove("CARGO_TARGET_DIR");
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

/// Create `<branch>` via a live linked worktree at `holder/<branch>`, off `<src>`
/// HEAD. Each entry in `commits` is `(rel, body)` committed as its own commit.
/// Returns the live linked worktree path.
fn make_fork_branch(src: &Path, holder: &Path, branch: &str, commits: &[(&str, &str)]) -> PathBuf {
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

// --- VT-1: positive verdict reaps worktree+branch (both oracle legs) ---

#[test]
fn gc_ancestor_leg_reaps_a_landed_via_land_fork() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let wt = make_fork_branch(src.path(), holder.path(), "anc", &[("one.rs", "fn a() {}")]);
    // Materialise the fork's in-tree target/ so we can prove it dies with the
    // worktree dir (SL-156 — no separate target-reap step).
    std::fs::create_dir_all(wt.join("target")).unwrap();
    // Merge --no-ff so the fork tip becomes an ANCESTOR of coordination HEAD (the
    // `land` route — ancestry preserved ⇒ tip reachable from HEAD).
    git(src.path(), &["merge", "--no-ff", "--no-edit", "anc"]);
    let (is_ancestor, _) = git_try(src.path(), &["merge-base", "--is-ancestor", "anc", "HEAD"]);
    assert!(is_ancestor, "precondition: fork tip is an ancestor of HEAD");

    let out = run(src.path(), None, &["worktree", "gc", "--fork", "anc"]);
    assert!(
        out.status.success(),
        "ancestor-leg gc must reap; stderr: {}",
        stderr(&out)
    );
    // Branch gone, worktree gone — and the in-tree target/ went with the worktree.
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "anc"]);
    assert!(!br, "fork branch reaped");
    assert!(!wt.exists(), "fork worktree dir reaped");
    assert!(
        !wt.join("target").exists(),
        "in-tree target/ died with the worktree"
    );
    // The recompile WARN fires on a successful reap.
    assert!(
        stderr(&out).contains("recompile"),
        "stderr WARNs about stale CARGO_MANIFEST_DIR test binaries; got: {}",
        stderr(&out)
    );
}

#[test]
fn gc_patch_id_leg_reaps_a_landed_via_import_fork() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // A fork with one commit adding new.rs.
    let wt = make_fork_branch(src.path(), holder.path(), "imp", &[("new.rs", "fn x() {}")]);
    std::fs::create_dir_all(wt.join("target")).unwrap();
    // Simulate the IMPORT route: apply the fork's patch onto coordination HEAD and
    // commit it as a NEW commit. Ancestry is severed (the fork tip is NOT reachable
    // from HEAD), but the patch landed ⇒ `git cherry HEAD imp` lists every commit
    // with a `-` prefix.
    let patch = git(src.path(), &["diff", "main..imp"]);
    let patch_file = src.path().join("imp.patch");
    std::fs::write(&patch_file, &patch).unwrap();
    git(
        src.path(),
        &["apply", "--index", patch_file.to_str().unwrap()],
    );
    std::fs::remove_file(&patch_file).unwrap();
    git(src.path(), &["add", "-A"]);
    git(src.path(), &["commit", "-q", "-m", "imported imp delta"]);
    // Ancestry severed, patch landed.
    let (is_ancestor, _) = git_try(src.path(), &["merge-base", "--is-ancestor", "imp", "HEAD"]);
    assert!(!is_ancestor, "precondition: ancestry severed");
    let cherry = git(src.path(), &["cherry", "HEAD", "imp"]);
    assert!(
        cherry.lines().all(|l| l.starts_with('-')),
        "precondition: every cherry line is `-` (patch landed); got: {cherry:?}"
    );

    let out = run(src.path(), None, &["worktree", "gc", "--fork", "imp"]);
    assert!(
        out.status.success(),
        "patch-id-leg gc must reap; stderr: {}",
        stderr(&out)
    );
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "imp"]);
    assert!(!br, "fork branch reaped");
    assert!(!wt.exists(), "fork worktree dir reaped");
    assert!(
        !wt.join("target").exists(),
        "in-tree target/ died with the worktree"
    );
}

#[test]
fn gc_non_ancestor_with_plus_refuses_then_force_and_superseded_reap() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // A fork whose patch is NOT upstream (never landed) ⇒ `git cherry` shows `+`.
    make_fork_branch(
        src.path(),
        holder.path(),
        "unl",
        &[("only.rs", "fn u() {}")],
    );
    let cherry = git(src.path(), &["cherry", "HEAD", "unl"]);
    assert!(
        cherry.lines().any(|l| l.starts_with('+')),
        "precondition: a `+` line (patch not upstream); got: {cherry:?}"
    );

    // (1) Refuses not-landed without an override.
    let out = run(src.path(), None, &["worktree", "gc", "--fork", "unl"]);
    assert_refusal(&out, "not-landed");
    // Nothing destroyed.
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "unl"]);
    assert!(br, "branch survives a refusal");

    // (2) --superseded-head <current-head> reaps (operator asserts spent-abandoned).
    let head = git(src.path(), &["rev-parse", "unl"]);
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "gc",
            "--fork",
            "unl",
            "--superseded-head",
            &head,
        ],
    );
    assert!(
        out.status.success(),
        "matching --superseded-head reaps; stderr: {}",
        stderr(&out)
    );
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "unl"]);
    assert!(!br, "branch reaped under matching --superseded-head");

    // (3) --force reaps a fresh unlanded fork.
    make_fork_branch(
        src.path(),
        holder.path(),
        "unl2",
        &[("two.rs", "fn v() {}")],
    );
    let out = run(
        src.path(),
        None,
        &["worktree", "gc", "--fork", "unl2", "--force"],
    );
    assert!(
        out.status.success(),
        "--force reaps an unlanded fork; stderr: {}",
        stderr(&out)
    );
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "unl2"]);
    assert!(!br, "branch reaped under --force");
}

// --- VT-2: squash-uncertifiable; --superseded honesty; --dry-run ---

#[test]
fn gc_squash_merge_refuses_not_landed_naming_the_remedy() {
    // A manual squash-merge is STRUCTURALLY INDISTINGUISHABLE from a never-landed
    // fork: a MULTI-commit `git merge --squash` collapses the commits into one, so
    // no fork commit's patch-id matches ⇒ `git cherry HEAD <fork>` lists `+` lines
    // (exactly like a never-landed fork; a single-commit squash would list `-` and
    // be correctly certified as landed). The oracle cannot split the two states, so
    // gc refuses `not-landed` with a message that NAMES the squash remedy — the
    // right guidance either way (design-faithful collapse; see GcRefusal).
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    make_fork_branch(
        src.path(),
        holder.path(),
        "sq",
        &[("s1.rs", "fn s1() {}"), ("s2.rs", "fn s2() {}")],
    );
    git(src.path(), &["merge", "--squash", "sq"]);
    git(src.path(), &["commit", "-q", "-m", "squashed sq"]);
    let (is_ancestor, _) = git_try(src.path(), &["merge-base", "--is-ancestor", "sq", "HEAD"]);
    assert!(!is_ancestor, "precondition: squash severs ancestry");
    let cherry = git(src.path(), &["cherry", "HEAD", "sq"]);
    assert!(
        cherry.lines().any(|l| l.starts_with('+')),
        "precondition: a multi-commit squash shows `+` (not certifiable); got: {cherry:?}"
    );

    let out = run(src.path(), None, &["worktree", "gc", "--fork", "sq"]);
    assert_refusal(&out, "not-landed");
    // The refusal message NAMES the squash remedy.
    let err = stderr(&out);
    assert!(err.contains("land"), "names `worktree land`; got: {err}");
    assert!(err.contains("--no-ff"), "names `--no-ff`; got: {err}");
    assert!(err.contains("--force"), "names `--force`; got: {err}");
    // --force still reaps a squashed fork (the operator chose to).
    let out = run(
        src.path(),
        None,
        &["worktree", "gc", "--fork", "sq", "--force"],
    );
    assert!(
        out.status.success(),
        "--force reaps over a squash; stderr: {}",
        stderr(&out)
    );
}

#[test]
fn gc_superseded_head_wrong_sha_refuses() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    make_fork_branch(src.path(), holder.path(), "ss", &[("f.rs", "x")]);
    // A SHA that is not the branch's current head (the base commit) ⇒ no match ⇒
    // refuse (the safe side: a stale/wrong SHA never reaps a live, moved head).
    let base = git(src.path(), &["rev-parse", "main"]);
    let head = git(src.path(), &["rev-parse", "ss"]);
    assert_ne!(base, head, "precondition: base != fork head");
    let out = run(
        src.path(),
        None,
        &["worktree", "gc", "--fork", "ss", "--superseded-head", &base],
    );
    assert_refusal(&out, "not-landed");
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "ss"]);
    assert!(br, "a wrong --superseded-head does not reap");
}

#[test]
fn gc_dry_run_prints_verdict_and_destroys_nothing() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // Unlanded fork ⇒ verdict is not-landed.
    let wt = make_fork_branch(src.path(), holder.path(), "dr", &[("f.rs", "x")]);

    let out = run(
        src.path(),
        None,
        &["worktree", "gc", "--fork", "dr", "--dry-run"],
    );
    assert!(
        out.status.success(),
        "dry-run exits 0; stderr: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("not-landed"),
        "dry-run prints the verdict; stdout: {}",
        stdout(&out)
    );
    // NOTHING destroyed: worktree + branch survive.
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "dr"]);
    assert!(br, "dry-run leaves the branch");
    assert!(wt.exists(), "dry-run leaves the worktree");

    // A LANDED fork's dry-run prints the positive verdict, still destroying nothing.
    git(src.path(), &["merge", "--no-ff", "--no-edit", "dr"]);
    let out = run(
        src.path(),
        None,
        &["worktree", "gc", "--fork", "dr", "--dry-run"],
    );
    assert!(
        stdout(&out).contains("landed"),
        "dry-run prints the landed verdict; stdout: {}",
        stdout(&out)
    );
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "dr"]);
    assert!(br, "landed dry-run still destroys nothing");
    assert!(wt.exists(), "landed dry-run leaves the worktree");
}

// F-5: a --force/--superseded dry-run over a NOT-landed fork must NOT claim
// `landed ✓` — the reap is authorised by the operator's override, not the
// oracle. Telling the operator it landed defeats the dry-run's purpose (design
// §8.1: "so the operator never --forces blind").
#[test]
fn gc_dry_run_force_over_unlanded_reports_override_not_landed() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let wt = make_fork_branch(src.path(), holder.path(), "frc", &[("f.rs", "x")]);
    // Precondition: genuinely not landed (a `+` cherry line).
    let cherry = git(src.path(), &["cherry", "HEAD", "frc"]);
    assert!(
        cherry.lines().any(|l| l.starts_with('+')),
        "precondition: fork is NOT landed; got: {cherry:?}"
    );

    let out = run(
        src.path(),
        None,
        &["worktree", "gc", "--fork", "frc", "--force", "--dry-run"],
    );
    assert!(
        out.status.success(),
        "dry-run exits 0; stderr: {}",
        stderr(&out)
    );
    let so = stdout(&out);
    assert!(
        !so.contains("landed ✓"),
        "a forced reap of an unlanded fork must NOT claim `landed ✓`; stdout: {so}"
    );
    assert!(
        so.contains("--force") || so.contains("override"),
        "names the override basis; stdout: {so}"
    );
    assert!(
        so.contains("would reap"),
        "reports the would-be reap; stdout: {so}"
    );
    // Still destroys nothing.
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "frc"]);
    assert!(br, "dry-run leaves the branch");
    assert!(wt.exists(), "dry-run leaves the worktree");
}

// --- VT-3: idempotent crash-rerun ---

#[test]
fn gc_fully_reaped_rerun_is_a_clean_noop() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let wt = make_fork_branch(src.path(), holder.path(), "noop", &[("f.rs", "x")]);
    // Everything already gone (a completed gc). A rerun is a clean no-op.
    git(
        src.path(),
        &["worktree", "remove", "--force", wt.to_str().unwrap()],
    );
    git(src.path(), &["branch", "-D", "noop"]);

    let out = run(src.path(), None, &["worktree", "gc", "--fork", "noop"]);
    assert!(
        out.status.success(),
        "fully-reaped rerun is a clean no-op; stderr: {}",
        stderr(&out)
    );
}

#[test]
fn gc_worktree_removed_before_branch_completes_the_branch_delete() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // Crash AFTER worktree-remove but BEFORE branch -D: the worktree is gone, the
    // branch lives (the W-before-B ordering case). The branch is landed so the gate
    // passes; the rerun completes the branch delete (skipping the absent worktree).
    let wt = make_fork_branch(src.path(), holder.path(), "wbb", &[("f.rs", "x")]);
    git(src.path(), &["merge", "--no-ff", "--no-edit", "wbb"]);
    git(
        src.path(),
        &["worktree", "remove", "--force", wt.to_str().unwrap()],
    );
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "wbb"]);
    assert!(br, "precondition: branch still lives");

    let out = run(src.path(), None, &["worktree", "gc", "--fork", "wbb"]);
    assert!(
        out.status.success(),
        "W-before-B rerun completes the branch delete; stderr: {}",
        stderr(&out)
    );
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "wbb"]);
    assert!(!br, "the branch is deleted on the rerun");
}

#[test]
fn gc_stale_admin_worktree_entry_folds_via_prune() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    // A landed fork whose worktree DIR was deleted out from under git (rm -rf),
    // leaving a STALE administrative worktree entry. gc's worktree-remove may fail
    // on the missing dir; `git worktree prune` folds the stale entry so the rerun
    // does not strand it.
    let wt = make_fork_branch(src.path(), holder.path(), "stale", &[("f.rs", "x")]);
    git(src.path(), &["merge", "--no-ff", "--no-edit", "stale"]);
    std::fs::remove_dir_all(&wt).unwrap();
    assert!(
        !wt.exists(),
        "precondition: the worktree dir is gone, entry stale"
    );

    let out = run(src.path(), None, &["worktree", "gc", "--fork", "stale"]);
    assert!(
        out.status.success(),
        "stale admin entry folds via prune; stderr: {}",
        stderr(&out)
    );
    let (br, _) = git_try(src.path(), &["rev-parse", "--verify", "--quiet", "stale"]);
    assert!(!br, "branch reaped after folding the stale entry");
    // No stale entry remains.
    let listing = git(src.path(), &["worktree", "list", "--porcelain"]);
    assert!(
        !listing.contains("stale"),
        "no stale worktree entry remains; got: {listing}"
    );
}

// --- VT-4: EXHAUSTIVE Orchestrator-class refusal (fork/import/land/gc) ---

/// Make a real linked worktree fork of `src` and return its path.
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

/// Every Orchestrator-classed verb's argv (the verb name is asserted in the marker
/// refusal). `marker --clear` is deliberately EXCLUDED — it is the bespoke
/// MarkerClear class, never refused by the worker-mode conjunct.
fn orchestrator_verbs() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "fork",
            vec![
                "worktree", "fork", "--base", "HEAD", "--branch", "x", "--dir", "/tmp/x",
            ],
        ),
        (
            "import",
            vec!["worktree", "import", "--base", "HEAD", "--fork", "x"],
        ),
        ("land", vec!["worktree", "land", "--fork", "x"]),
        ("gc", vec!["worktree", "gc", "--fork", "x"]),
    ]
}

#[test]
fn every_orchestrator_verb_refused_from_a_marked_linked_worktree() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let fork = add_linked_fork(src.path(), holder.path(), "guard-marker");
    stamp_marker(&fork);

    for (verb, argv) in orchestrator_verbs() {
        let out = run(&fork, None, &argv);
        assert!(
            !out.status.success(),
            "{verb} refused from a marked linked worktree; stdout: {}",
            stdout(&out)
        );
        assert!(
            stderr(&out).contains(verb),
            "{verb} refusal names the verb; stderr: {}",
            stderr(&out)
        );
    }
}

#[test]
fn every_orchestrator_verb_refused_under_worker_env() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());

    for (verb, argv) in orchestrator_verbs() {
        let out = run(src.path(), Some(true), &argv);
        assert!(
            !out.status.success(),
            "{verb} refused when DOCTRINE_WORKER set; stdout: {}",
            stdout(&out)
        );
        assert!(
            stderr(&out).contains("DOCTRINE_WORKER"),
            "{verb} env refusal carries the dual-cause; stderr: {}",
            stderr(&out)
        );
    }
}
