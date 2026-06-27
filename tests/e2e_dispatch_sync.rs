// SPDX-License-Identifier: GPL-3.0-only
//! SL-064 PHASE-04 — `doctrine dispatch sync --prepare-review --slice <n>`
//! end-to-end over the BUILT binary (design §4.2 B / §4.3 C; ADR-012 D4/D5).
//!
//! * VT-1: default prepare-review creates `review/<slice>` and leaves trunk
//!   (`main`) byte-identical.
//! * VT-2: golden review composition — `review/064` carries the impl bundle
//!   (code + authored `.doctrine/` entity), excludes `.doctrine/dispatch/064/**`
//!   AND every journal-verified orthogonal path, retains a failed-mark path.
//! * VT-3: synthesized `phase/064-NN` refs contain exactly each phase's code
//!   delta, carry no `.doctrine/` path, and skip the empty-code phase.
//! * VT-4: impersonation — a marker-present linked worktree AND `DOCTRINE_WORKER=1`
//!   each refuse the verb, creating no external ref.
//! * EX-5: a stale pre-existing `review/064` is reported, never clobbered.
//!
//! PHASE-05 — stage-2 `--integrate` (appended below):
//! * EX-1/VT-3: integrate replays from a checkout with `dispatch/064` not checked
//!   out (plumbing-only, no coordination worktree).
//! * VT-1: the 3-way replay matrix — no-op on intact refs, idempotent re-run
//!   (crash-after-apply), refusal on a clobbered/diverged target. (The four CAS
//!   arms are exhaustively unit-pinned in `git::tests::replay_ref_no_op_apply_refuse`.)
//! * VT-2: trunk projection opt-in + fast-forward-only; refuses non-ff/moved trunk.
//! * EX-4: optional `edge` aggregate, default off.
//! * VT-5: `--integrate` refused under worker-mode (marker / `DOCTRINE_WORKER`).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
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

fn commit(dir: &Path, path: &str, content: &str, msg: &str) -> String {
    let full = dir.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(&full, content).unwrap();
    git(dir, &["add", path]);
    git(dir, &["commit", "-q", "-m", msg]);
    git(dir, &["rev-parse", "HEAD"])
}

/// Seed runtime phase-tracking in the PRIMARY tree marking each phase
/// `completed`, so prepare-review's PHASE-05 completeness gate (design §5.2) sees
/// a complete slice — the post-completion conclude beat it now is. Runtime state:
/// gitignored, written to the working filesystem, never committed.
fn seed_completed_phases(dir: &Path, slice: u32, phases: &[&str]) {
    let pdir = dir.join(format!(".doctrine/state/slice/{slice:03}/phases"));
    std::fs::create_dir_all(&pdir).unwrap();
    for p in phases {
        let stem = format!("phase-{}", p.strip_prefix("PHASE-").unwrap_or(p));
        std::fs::write(
            pdir.join(format!("{stem}.toml")),
            "status = \"completed\"\n",
        )
        .unwrap();
    }
}

/// The built fixture's captured OIDs.
struct Fixture {
    base: String,
    code_end_1: String,
    code_end_2: String,
}

/// Build a repo with `main` at a trunk base and a `dispatch/064` branch carrying
/// two code phases, an authored entity, a verified-orthogonal file, a
/// failed-orthogonal file, and the `boundaries.toml`/`orthogonal.toml` ledger.
/// Leaves the working tree on `main`.
fn build_fixture(dir: &Path) -> Fixture {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    // Runtime state is gitignored (as in production) so prepare-review's derive
    // (which writes the registry under `.doctrine/state/`) and the seeded phase
    // tracking never dirty the working tree — the integrate tests assert clean status.
    std::fs::write(dir.join(".gitignore"), ".doctrine/state/\n").unwrap();
    git(dir, &["add", ".gitignore"]);
    let base = commit(dir, "trunk.txt", "trunk", "base");

    git(dir, &["checkout", "-q", "-b", "dispatch/064"]);
    let code_end_1 = commit(dir, "src1.txt", "a", "phase1 code");
    let code_end_2 = commit(dir, "src2.txt", "b", "phase2 code");
    commit(
        dir,
        ".doctrine/slice/064/slice-064.md",
        "scope",
        "authored entity",
    );
    commit(dir, "ahead.txt", "ahead", "orthogonal ahead-projected");
    commit(dir, "notahead.txt", "notahead", "orthogonal failed");

    let boundaries = format!(
        "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"{base}\"\ncode_end_oid = \"{code_end_1}\"\n\
         [[boundary]]\nphase = \"PHASE-02\"\ncode_start_oid = \"{code_end_1}\"\ncode_end_oid = \"{code_end_2}\"\n\
         [[boundary]]\nphase = \"PHASE-03\"\ncode_start_oid = \"{code_end_2}\"\ncode_end_oid = \"{code_end_2}\"\n"
    );
    let orthogonal = "[[mark]]\nentity = \"ahead\"\npath = \"ahead.txt\"\nstatus = \"verified\"\n\
         [[mark]]\nentity = \"notahead\"\npath = \"notahead.txt\"\nstatus = \"failed\"\n";
    std::fs::create_dir_all(dir.join(".doctrine/dispatch/064")).unwrap();
    std::fs::write(
        dir.join(".doctrine/dispatch/064/boundaries.toml"),
        &boundaries,
    )
    .unwrap();
    std::fs::write(
        dir.join(".doctrine/dispatch/064/orthogonal.toml"),
        orthogonal,
    )
    .unwrap();
    git(dir, &["add", ".doctrine/dispatch/064"]);
    git(dir, &["commit", "-q", "-m", "ledger fixtures"]);

    git(dir, &["checkout", "-q", "main"]);
    // prepare-review is the pre-audit conclude beat (design §5.2): its PHASE-05
    // completeness gate requires every ledger phase to be `completed`.
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02", "PHASE-03"]);
    Fixture {
        base,
        code_end_1,
        code_end_2,
    }
}

/// Run `doctrine <args>` in `cwd`; `worker = Some(true)` sets DOCTRINE_WORKER=1.
fn run(cwd: &Path, worker: Option<bool>, args: &[&str]) -> Output {
    let mut cmd = Command::new(bin());
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

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn ref_exists(dir: &Path, refname: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--verify", "--quiet", refname])
        .output()
        .expect("spawn git")
        .status
        .success()
}

fn prepare_review(cwd: &Path) -> Output {
    run(
        cwd,
        None,
        &[
            "dispatch",
            "sync",
            "--prepare-review",
            "--slice",
            "64",
            "-p",
            cwd.to_str().unwrap(),
        ],
    )
}

// --- VT-1: review/<slice> created, trunk untouched ---------------------------

#[test]
fn prepare_review_creates_review_ref_and_leaves_trunk_unchanged() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    let main_before = git(dir, &["rev-parse", "main"]);

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "prepare-review ok; stderr: {}",
        stderr(&out)
    );

    assert!(ref_exists(dir, "review/064"), "review/064 created");
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        main_before,
        "trunk untouched"
    );
    assert!(!ref_exists(dir, "edge"), "edge never written");
}

// --- VT-2: review composition (golden) ---------------------------------------

#[test]
fn review_bundle_excludes_ledger_and_verified_orthogonal_retains_impl() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fixture = build_fixture(dir);

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "prepare-review ok; stderr: {}",
        stderr(&out)
    );

    let listing = git(dir, &["ls-tree", "-r", "--name-only", "review/064"]);
    let paths: Vec<&str> = listing.lines().collect();
    // impl bundle retained:
    for kept in [
        "trunk.txt",
        "src1.txt",
        "src2.txt",
        ".doctrine/slice/064/slice-064.md",
    ] {
        assert!(paths.contains(&kept), "review retains {kept}: {listing}");
    }
    // run ledger excluded:
    assert!(
        !paths
            .iter()
            .any(|p| p.starts_with(".doctrine/dispatch/064")),
        "review excludes the run-ledger dir: {listing}"
    );
    // verified-orthogonal path excluded; failed-mark path retained:
    assert!(
        !paths.contains(&"ahead.txt"),
        "verified-orthogonal path excluded: {listing}"
    );
    assert!(
        paths.contains(&"notahead.txt"),
        "failed-mark path falls back into the bundle: {listing}"
    );

    // parented to the trunk base ⇒ diff base...review is exactly the bundle.
    let parent = git(dir, &["rev-parse", "review/064^"]);
    assert_eq!(parent, fixture.base, "review parented to the trunk base");
}

// --- VT-3: per-phase synthesis -----------------------------------------------

#[test]
fn phase_refs_are_exact_code_deltas_with_no_doctrine_and_skip_empty() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fixture = build_fixture(dir);

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "prepare-review ok; stderr: {}",
        stderr(&out)
    );

    assert!(ref_exists(dir, "phase/064-01"), "phase 01 cut");
    assert!(ref_exists(dir, "phase/064-02"), "phase 02 cut");
    assert!(
        !ref_exists(dir, "phase/064-03"),
        "empty-code phase 03 emits no ref"
    );

    // phase 01 = exactly src1.txt over the trunk base; no .doctrine.
    assert_eq!(
        git(dir, &["diff", "--name-only", &fixture.base, "phase/064-01"]),
        "src1.txt",
        "phase 01 diff is exactly its code delta"
    );
    // phase 02 chained off phase 01 = exactly src2.txt.
    assert_eq!(
        git(
            dir,
            &["diff", "--name-only", "phase/064-01", "phase/064-02"]
        ),
        "src2.txt",
        "phase 02 diff is exactly its code delta"
    );
    let p1_listing = git(dir, &["ls-tree", "-r", "--name-only", "phase/064-01"]);
    assert!(
        !p1_listing.lines().any(|p| p.starts_with(".doctrine")),
        "phase ref carries no .doctrine/ path: {p1_listing}"
    );
    // The empty phase used code_end_2 == code_start_2; phase 02 already covers it.
    let _ = &fixture.code_end_1;
    let _ = &fixture.code_end_2;
}

// --- VT-3 (cont): EX-2 journal committed onto the branch with verified rows ---

#[test]
fn journal_recorded_on_branch_with_verified_status() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "prepare-review ok; stderr: {}",
        stderr(&out)
    );

    let journal = git(
        dir,
        &["show", "dispatch/064:.doctrine/dispatch/064/journal.toml"],
    );
    assert!(
        journal.contains("review/064"),
        "journal records the review ref: {journal}"
    );
    assert!(
        journal.contains("phase/064-01"),
        "journal records a phase ref: {journal}"
    );
    assert!(
        journal.contains("verified"),
        "applied rows are verified: {journal}"
    );
    assert!(
        !journal.contains("pending"),
        "no rows left pending after a clean run: {journal}"
    );
}

// --- VT-4: impersonation refusal ---------------------------------------------

#[test]
fn prepare_review_refused_under_worker_mode() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);

    // (1) Marker-present linked worktree, env unset ⇒ refused, names verb.
    let holder = tempfile::tempdir().unwrap();
    let base = git(dir, &["rev-parse", "HEAD"]);
    let linked = holder.path().join("fork");
    git(
        dir,
        &[
            "worktree",
            "add",
            "-b",
            "wkr-guard",
            linked.to_str().unwrap(),
            &base,
        ],
    );
    let marker_dir = linked.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&marker_dir).unwrap();
    std::fs::write(marker_dir.join("worker"), b"").unwrap();

    let out = run(
        &linked,
        None,
        &[
            "dispatch",
            "sync",
            "--prepare-review",
            "--slice",
            "64",
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "refused from a marked linked worktree"
    );
    assert!(
        stderr(&out).contains("dispatch-sync"),
        "refusal names the verb: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "review/064"),
        "refused run creates no external ref"
    );

    // (2) DOCTRINE_WORKER set ⇒ dual-cause refusal.
    let out = run(
        dir,
        Some(true),
        &[
            "dispatch",
            "sync",
            "--prepare-review",
            "--slice",
            "64",
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "refused when DOCTRINE_WORKER set");
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "carries the dual-cause: {}",
        stderr(&out)
    );
    assert!(!ref_exists(dir, "review/064"), "still no external ref");
}

// --- EX-5: a stale pre-existing review ref is reported, never clobbered -------

#[test]
fn stale_review_ref_is_reported_not_clobbered() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    // A crashed prior run left review/064 at the trunk base (bogus content).
    let bogus = git(dir, &["rev-parse", "main"]);
    git(dir, &["update-ref", "refs/heads/review/064", &bogus]);

    let out = prepare_review(dir);
    assert!(
        !out.status.success(),
        "stale ref makes the run report failure"
    );
    assert!(
        stderr(&out).contains("not clobbered"),
        "stale ref is reported, not clobbered: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "review/064"]),
        bogus,
        "the stale review/064 is left untouched"
    );
}

// --- VT-4 (IMP-075): apply-contract persistence. A semantic refusal returns
//     Ok(Refused) — NOT a `?`-Err — so the post-loop recovery commit_journal runs
//     and durably records `status=Failed`. Tree-read the COMMITTED journal to
//     prove the recovery commit ran rather than an early Err abort. ------------

#[test]
fn refused_row_persists_failed_status_in_committed_journal() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    // A crashed prior run left review/064 at the trunk base — its zero-oid CAS
    // creation will be refused (the row is a SEMANTIC refusal, not fatal).
    let bogus = git(dir, &["rev-parse", "main"]);
    git(dir, &["update-ref", "refs/heads/review/064", &bogus]);

    let out = prepare_review(dir);
    assert!(!out.status.success(), "stale ref makes the run bail");

    // The committed journal on dispatch/064 must carry the refused row as failed:
    // proves the recovery commit_journal ran AFTER the apply loop (Ok(Refused),
    // not an early Err abort that would skip the recovery commit).
    let journal = git(
        dir,
        &["show", "dispatch/064:.doctrine/dispatch/064/journal.toml"],
    );
    assert!(
        journal.contains("review/064"),
        "journal records the refused review row: {journal}"
    );
    assert!(
        journal.contains("failed"),
        "the refused row persisted status=failed (recovery commit ran): {journal}"
    );
}

// ====================================================================
// PHASE-04 — commit the boundaries ledger at prepare-review (ISS-039)
// ====================================================================
//
// VT-1: splice from an UNCOMMITTED working ledger living in a live coordination
//   worktree → prepare-review commits boundaries.toml onto dispatch/064 (beside
//   journal.toml), read_ledger reads N rows, phase/064-NN refs project.
// VT-2: content-idempotent — a second prepare-review on the unchanged working
//   ledger adds NO second `ledger: boundaries` commit and the committed
//   boundaries blob is stable (commit_boundaries no-op via TREE-oid compare).
// VT-3: malformed working boundaries.toml → the run fails and the dispatch tip
//   is unchanged (commit_boundaries validates before committing — no garbage).

/// Like [`build_fixture`] but the boundaries ledger is left **uncommitted** in a
/// live coordination worktree on `dispatch/064` (no `ledger fixtures` commit). The
/// dispatch tip carries the code + authored entity only; the working `boundaries.toml`
/// sits in the coord worktree, so `prepare_review` must splice it via
/// `commit_boundaries` (guarded by `live_worktree_for_ref`) before any read. Returns
/// the coord worktree path. `ledger` is written verbatim — callers pass a malformed
/// body to exercise the validate-before-commit path (VT-3).
fn build_fixture_uncommitted_ledger(dir: &Path, ledger: &str) -> std::path::PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    // Gitignore runtime state (as production) — prepare-review's derive writes the
    // registry under `.doctrine/state/`; without this it would dirty the tree.
    std::fs::write(dir.join(".gitignore"), ".doctrine/state/\n").unwrap();
    git(dir, &["add", ".gitignore"]);
    commit(dir, "trunk.txt", "trunk", "base");

    git(dir, &["checkout", "-q", "-b", "dispatch/064"]);
    commit(dir, "src1.txt", "a", "phase1 code");
    commit(dir, "src2.txt", "b", "phase2 code");
    commit(
        dir,
        ".doctrine/slice/064/slice-064.md",
        "scope",
        "authored entity",
    );
    // Return the primary tree to `main` so `dispatch/064` is free to check out in
    // a linked worktree (git refuses a branch checked out in two worktrees).
    git(dir, &["checkout", "-q", "main"]);

    // A LIVE coordination worktree on dispatch/064 — what `live_worktree_for_ref`
    // keys on, and where the uncommitted ledger lives on the working filesystem.
    let coord = dir.join(".coord-064");
    git(
        dir,
        &[
            "worktree",
            "add",
            "-q",
            coord.to_str().unwrap(),
            "dispatch/064",
        ],
    );
    std::fs::create_dir_all(coord.join(".doctrine/dispatch/064")).unwrap();
    std::fs::write(coord.join(".doctrine/dispatch/064/boundaries.toml"), ledger).unwrap();
    // The PHASE-05 completeness gate roots on the PRIMARY tree (`dir`), not the
    // coord worktree — seed completion there (design §5.2: gate is primary-rooted).
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02", "PHASE-03"]);
    coord
}

/// A well-formed three-phase boundaries body (PHASE-03 is empty-code: start==end).
fn working_boundaries(dir: &Path) -> String {
    let base = git(dir, &["rev-parse", "dispatch/064~3"]);
    let code_end_1 = git(dir, &["rev-parse", "dispatch/064~2"]);
    let code_end_2 = git(dir, &["rev-parse", "dispatch/064~1"]);
    format!(
        "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"{base}\"\ncode_end_oid = \"{code_end_1}\"\nprovenance = \"funnel\"\n\
         [[boundary]]\nphase = \"PHASE-02\"\ncode_start_oid = \"{code_end_1}\"\ncode_end_oid = \"{code_end_2}\"\nprovenance = \"funnel\"\n\
         [[boundary]]\nphase = \"PHASE-03\"\ncode_start_oid = \"{code_end_2}\"\ncode_end_oid = \"{code_end_2}\"\nprovenance = \"funnel\"\n"
    )
}

// --- VT-1: splice from an uncommitted working ledger, then project -----------

#[test]
fn prepare_review_splices_uncommitted_ledger_then_projects() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    // Seed with a placeholder, then write the oid-bearing ledger once the branch exists.
    let coord = build_fixture_uncommitted_ledger(dir, "");
    std::fs::write(
        coord.join(".doctrine/dispatch/064/boundaries.toml"),
        working_boundaries(dir),
    )
    .unwrap();

    // Precondition: the dispatch tip carries NO committed ledger yet.
    assert!(
        !git(dir, &["ls-tree", "-r", "--name-only", "dispatch/064"]).contains("boundaries.toml"),
        "precondition: boundaries ledger is uncommitted"
    );

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "prepare-review ok; stderr: {}",
        stderr(&out)
    );

    // commit_boundaries spliced the working ledger onto dispatch/064, beside journal.toml.
    let listing = git(dir, &["ls-tree", "-r", "--name-only", "dispatch/064"]);
    assert!(
        listing.contains(".doctrine/dispatch/064/boundaries.toml"),
        "boundaries.toml committed onto dispatch/064: {listing}"
    );
    assert!(
        listing.contains(".doctrine/dispatch/064/journal.toml"),
        "journal.toml committed beside it: {listing}"
    );

    // read_ledger read the now-committed N rows → phase cuts project (PHASE-03 empty, skipped).
    assert!(ref_exists(dir, "review/064"), "review/064 projected");
    assert!(ref_exists(dir, "phase/064-01"), "phase/064-01 projected");
    assert!(ref_exists(dir, "phase/064-02"), "phase/064-02 projected");
    assert!(
        !ref_exists(dir, "phase/064-03"),
        "empty-code PHASE-03 emits no cut"
    );
}

// --- VT-2: commit_boundaries is content-idempotent on an unchanged re-run -----
//
// Verified at the commit_boundaries grain (the literal "same dispatch tip oid"
// full-rerun assertion is unsatisfiable at PHASE-04 — a second prepare-review
// collides on the already-created refs and the journal churns, pre-existing
// EX-5/VT-4 behaviour; the clean re-run needs the PHASE-05 gate). The content-
// idempotency claim (design F1 / EX-2): identical working content ⇒ no second
// `ledger: boundaries` commit and a stable committed blob.

#[test]
fn commit_boundaries_is_content_idempotent_on_rerun() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let coord = build_fixture_uncommitted_ledger(dir, "");
    std::fs::write(
        coord.join(".doctrine/dispatch/064/boundaries.toml"),
        working_boundaries(dir),
    )
    .unwrap();

    assert!(prepare_review(dir).status.success(), "first run projects");
    let blob_before = git(
        dir,
        &[
            "rev-parse",
            "dispatch/064:.doctrine/dispatch/064/boundaries.toml",
        ],
    );

    // Re-run on the UNCHANGED working ledger. The full run bails on the already-
    // created refs (out of PHASE-04 scope) — but commit_boundaries must no-op:
    // no new boundaries commit, identical committed blob.
    let _ = prepare_review(dir);
    let blob_after = git(
        dir,
        &[
            "rev-parse",
            "dispatch/064:.doctrine/dispatch/064/boundaries.toml",
        ],
    );
    assert_eq!(
        blob_before, blob_after,
        "committed boundaries blob is stable across the re-run (TREE-oid no-op)"
    );

    let boundaries_commits = git(
        dir,
        &[
            "log",
            "--grep",
            "ledger: boundaries",
            "--oneline",
            "dispatch/064",
        ],
    );
    assert_eq!(
        boundaries_commits.lines().count(),
        1,
        "commit_boundaries fired exactly once — content-idempotent: {boundaries_commits}"
    );
}

// --- VT-3: malformed working ledger → run fails, dispatch tip unchanged -------

#[test]
fn malformed_working_ledger_fails_and_leaves_dispatch_tip_unchanged() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture_uncommitted_ledger(dir, "this is not valid boundaries toml ][\n");
    let tip_before = git(dir, &["rev-parse", "dispatch/064"]);

    let out = prepare_review(dir);
    assert!(
        !out.status.success(),
        "malformed working ledger makes the run fail: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("malformed"),
        "error names the malformed ledger: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "dispatch/064"]),
        tip_before,
        "dispatch tip unchanged — validate-before-commit committed no garbage"
    );
}

// ====================================================================
// PHASE-05 — stage-2 `dispatch sync --integrate` (design §4 / ADR-012 D4/D5)
// ====================================================================

/// Run `dispatch sync --integrate --slice 64` in `cwd` with `extra` flags.
fn integrate(cwd: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "dispatch",
        "sync",
        "--integrate",
        "--slice",
        "64",
        "-p",
        cwd.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    run(cwd, None, &args)
}

/// EX-1 / VT-3 + VT-1(no-op): integrate runs from a checkout where `dispatch/064`
/// is NOT checked out (plumbing-only, no coordination worktree). Default
/// integration replays the prepared journal as verified no-ops and leaves trunk
/// untouched (EX-3).
#[test]
fn integrate_default_replays_prepared_refs_no_checkout_no_trunk_write() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());

    let trunk_before = git(dir, &["rev-parse", "main"]);
    let review_before = git(dir, &["rev-parse", "review/064"]);
    let phase_before = git(dir, &["rev-parse", "phase/064-02"]);
    // The working tree is on `main`; dispatch/064 is a branch with no worktree.
    assert_eq!(git(dir, &["rev-parse", "--abbrev-ref", "HEAD"]), "main");

    let out = integrate(dir, &[]);
    assert!(
        out.status.success(),
        "default integrate replays cleanly; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        trunk_before,
        "trunk untouched"
    );
    assert_eq!(git(dir, &["rev-parse", "review/064"]), review_before);
    assert_eq!(git(dir, &["rev-parse", "phase/064-02"]), phase_before);

    let journal = git(
        dir,
        &["show", "dispatch/064:.doctrine/dispatch/064/journal.toml"],
    );
    assert!(
        !journal.contains("pending"),
        "no pending rows after replay: {journal}"
    );
}

/// VT-2: trunk projection is opt-in, fast-forward-only. `--trunk` advances the
/// trunk ref to the cumulative code tip; a second run is an idempotent no-op
/// (VT-1: crash-after-apply recovery).
#[test]
fn integrate_trunk_fast_forwards_then_is_idempotent() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    let phase_tip = git(dir, &["rev-parse", "phase/064-02"]);

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        out.status.success(),
        "ff trunk integrate; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        phase_tip,
        "trunk fast-forwarded to the cumulative code tip"
    );

    // Re-run: current == planned ⇒ verified no-op, trunk unchanged.
    let out2 = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        out2.status.success(),
        "idempotent re-run; stderr: {}",
        stderr(&out2)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        phase_tip,
        "no second advance"
    );
}

/// VT-2: explicit trunk mode refuses a non-fast-forward / moved trunk and writes
/// nothing (IMP-043 re-anchor is reported, never auto-resolved).
#[test]
fn integrate_trunk_refuses_non_fast_forward() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());

    // Move trunk onto a divergent commit — not an ancestor of the phase chain.
    let moved = commit(dir, "trunk2.txt", "moved", "trunk advanced divergently");
    assert_eq!(git(dir, &["rev-parse", "main"]), moved);

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(!out.status.success(), "non-ff trunk is refused");
    assert!(
        stderr(&out).contains("fast-forward") || stderr(&out).contains("trunk moved"),
        "refusal names the non-ff/moved-trunk cause: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        moved,
        "trunk left untouched"
    );
}

/// RV-030 F-1: a foreign commit landing on trunk BETWEEN coordinate and
/// prepare-review must NOT reparent the projection. Stage-1 projects off the
/// pinned fork-point `merge-base(dispatch/064, trunk)`, not the live trunk tip —
/// the coordination worktree isolates the working tree, not the trunk ref. So the
/// per-phase cut stays an exact code delta (no foreign leak), and the §3/IMP-043
/// moved-trunk net fires: a subsequent `integrate --trunk` refuses the non-ff
/// instead of silently absorbing the pre-stage-1 movement.
#[test]
fn prepare_review_projects_off_pinned_fork_point_not_moved_trunk() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fixture = build_fixture(dir);

    // A foreign writer advances trunk after dispatch/064 forked from `base`.
    let foreign = commit(dir, "foreign.txt", "foreign", "foreign trunk advance");
    assert_eq!(git(dir, &["rev-parse", "main"]), foreign);
    assert_ne!(
        foreign, fixture.base,
        "trunk genuinely moved off the fork-point"
    );

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "prepare-review ok despite moved trunk; stderr: {}",
        stderr(&out)
    );

    // The phase cut is parented on the FORK-POINT, not the moved trunk tip — so
    // its diff is exactly the phase code, with no foreign trunk delta leaking in.
    assert_eq!(
        git(dir, &["rev-parse", "phase/064-01^"]),
        fixture.base,
        "phase parented on the pinned fork-point, not the moved trunk tip"
    );
    assert_eq!(
        git(dir, &["diff", "--name-only", &fixture.base, "phase/064-01"]),
        "src1.txt",
        "phase diff is the exact code delta — foreign trunk file excluded"
    );
    assert!(
        !git(dir, &["ls-tree", "-r", "--name-only", "phase/064-02"]).contains("foreign.txt"),
        "the foreign trunk file never leaks into the phase bundle"
    );

    // The moved-trunk net now fires: the phase chain descends from the fork-point,
    // not the moved tip, so integrate --trunk refuses the non-ff (was silently
    // absorbed when stage-1 parented on the live trunk tip).
    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        !out.status.success(),
        "integrate --trunk refuses the moved trunk; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("fast-forward") || stderr(&out).contains("trunk moved"),
        "refusal names the non-ff cause: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        foreign,
        "trunk left untouched on refusal"
    );
}

/// VT-1 (replay refusal on divergence): a prepared ref clobbered after
/// prepare-review diverges from both its journaled `expected_old` and `planned`
/// ⇒ integrate refuses and leaves it untouched.
#[test]
fn integrate_refuses_clobbered_prepared_ref() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());

    // A foreign writer moved review/064 to the trunk base (≠ ZERO, ≠ planned).
    let bogus = git(dir, &["rev-parse", "main"]);
    git(dir, &["update-ref", "refs/heads/review/064", &bogus]);

    let out = integrate(dir, &[]);
    assert!(
        !out.status.success(),
        "diverged prepared ref refuses replay"
    );
    assert!(
        stderr(&out).contains("moved target") || stderr(&out).contains("not clobbered"),
        "divergence is reported: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "review/064"]),
        bogus,
        "the clobbered ref is left untouched, never force-resolved"
    );
}

/// EX-4: the optional `edge` aggregate advances to the `review/<slice>` bundle,
/// isolated to this sync point; default integration writes no edge ref.
#[test]
fn integrate_edge_is_opt_in_and_aggregates_the_review_bundle() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    let review_tip = git(dir, &["rev-parse", "review/064"]);

    // Default integrate writes no edge ref.
    assert!(integrate(dir, &[]).status.success());
    assert!(
        !ref_exists(dir, "refs/heads/edge"),
        "edge is opt-in, default off"
    );

    // Opt-in: edge is created at the review bundle.
    let out = integrate(dir, &["--edge", "refs/heads/edge"]);
    assert!(
        out.status.success(),
        "edge projection; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "edge"]),
        review_tip,
        "edge aggregates the review/<slice> bundle"
    );
}

// === SL-121 PHASE-02 — worktree-aware integrate advance + report ============

/// VT-2 (the ISS-022/030 regression): a checked-out trunk target fast-forwards via
/// `merge --ff-only`, so ref + index + worktree all land on the planned tip and
/// `git status` is empty — no phantom reverse-diff. (Pre-SL-121 pure
/// `update_ref_cas` advanced the ref but left the live tree stale.)
#[test]
fn integrate_trunk_checked_out_ff_leaves_clean_tree() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    let phase_tip = git(dir, &["rev-parse", "phase/064-02"]);
    // build_fixture leaves the working tree on main → the trunk target is checked out.
    assert_eq!(git(dir, &["rev-parse", "--abbrev-ref", "HEAD"]), "main");

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        out.status.success(),
        "ff integrate on a checked-out trunk; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        phase_tip,
        "trunk ref advanced to the cumulative code tip"
    );
    assert_eq!(
        git(dir, &["rev-parse", "HEAD"]),
        phase_tip,
        "HEAD advanced with the live checkout"
    );
    assert!(
        git(dir, &["status", "--porcelain"]).is_empty(),
        "index + worktree resynced — no phantom reverse-diff (ISS-022/030)",
    );
    assert!(
        dir.join("src1.txt").exists() && dir.join("src2.txt").exists(),
        "the advanced code is materialised in the live worktree",
    );
}

/// VT-1: a NOT-checked-out target advances by pure ref-CAS; the live (main)
/// checkout is left untouched — a pure-ref advance introduces no phantom diff.
#[test]
fn integrate_trunk_not_checked_out_advances_ref_leaves_live_checkout_clean() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fixture = build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    let phase_tip = git(dir, &["rev-parse", "phase/064-02"]);
    // A trunk-like ref parked at the fork base, checked out nowhere.
    git(dir, &["update-ref", "refs/heads/release", &fixture.base]);

    let out = integrate(dir, &["--trunk", "refs/heads/release"]);
    assert!(
        out.status.success(),
        "ff integrate on a not-checked-out ref; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "release"]),
        phase_tip,
        "the unchecked-out ref advanced by CAS"
    );
    assert!(
        git(dir, &["status", "--porcelain"]).is_empty(),
        "the live main checkout is untouched by a pure-ref advance",
    );
}

/// VT-3 (EX-1, M4): a DIRTY checked-out target refuses the whole integrate with
/// `integrate-dirty-worktree` and moves ZERO refs — including the coordination ref
/// `dispatch/064` (the gate runs before the first `commit_journal`).
#[test]
fn integrate_dirty_checked_out_target_refuses_zero_refs_moved() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());

    // Snapshot every ref the integrate would touch + the coordination ref.
    let main_before = git(dir, &["rev-parse", "main"]);
    let dispatch_before = git(dir, &["rev-parse", "dispatch/064"]);
    let review_before = git(dir, &["rev-parse", "review/064"]);
    let phase_before = git(dir, &["rev-parse", "phase/064-02"]);

    // Dirty the checked-out trunk (tracked modification).
    std::fs::write(dir.join("trunk.txt"), "locally dirtied").unwrap();

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(!out.status.success(), "a dirty checked-out target refuses");
    assert!(
        stderr(&out).contains("integrate-dirty-worktree"),
        "refusal names the dirty-worktree token: {}",
        stderr(&out),
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        main_before,
        "trunk unmoved"
    );
    assert_eq!(
        git(dir, &["rev-parse", "dispatch/064"]),
        dispatch_before,
        "coordination ref unmoved (gate precedes the first commit_journal, M4)"
    );
    assert_eq!(
        git(dir, &["rev-parse", "review/064"]),
        review_before,
        "review unmoved"
    );
    assert_eq!(
        git(dir, &["rev-parse", "phase/064-02"]),
        phase_before,
        "phase unmoved"
    );
}

/// VT-4 (B2): a checked-out target needing a NON-ff advance refuses
/// `integrate-nonff-checkout` (never `reset --hard` a live ref); the ref is left
/// unmoved and the row persists `Failed` in the committed journal. Exercised via
/// the edge row (not ff-gated at plan time) checked out in a linked worktree on a
/// commit divergent from `review/064`.
#[test]
fn integrate_nonff_checked_out_edge_refuses_and_persists_failed() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fixture = build_fixture(dir);
    assert!(prepare_review(dir).status.success());

    // An `edge` branch checked out in a linked worktree, advanced divergently so a
    // fast-forward to review/064 is impossible.
    git(dir, &["update-ref", "refs/heads/edge", &fixture.base]);
    let holder = tempfile::tempdir().unwrap();
    let linked = holder.path().join("edgewt");
    git(dir, &["worktree", "add", linked.to_str().unwrap(), "edge"]);
    std::fs::write(linked.join("divergent.txt"), "x").unwrap();
    git(&linked, &["add", "divergent.txt"]);
    git(&linked, &["commit", "-q", "-m", "edge divergent"]);
    let edge_before = git(dir, &["rev-parse", "edge"]);

    let out = integrate(dir, &["--edge", "refs/heads/edge"]);
    assert!(!out.status.success(), "a non-ff checked-out edge refuses");
    assert!(
        stderr(&out).contains("integrate-nonff-checkout"),
        "refusal names the non-ff-checkout token: {}",
        stderr(&out),
    );
    assert_eq!(
        git(dir, &["rev-parse", "edge"]),
        edge_before,
        "edge ref left untouched (never reset --hard a live ref)"
    );
    let journal = git(
        dir,
        &["show", "dispatch/064:.doctrine/dispatch/064/journal.toml"],
    );
    assert!(
        journal.contains("failed"),
        "the refused edge row persisted status=failed: {journal}"
    );
}

/// VT-6 (§4 / IMP-078): integrate emits per-row disposition detail on stderr with
/// the EXACT tokens AND preserves the machine-readable stdout ref-list contract.
#[test]
fn integrate_report_emits_disposition_and_preserves_stdout_reflist() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let so = String::from_utf8_lossy(&out.stdout);
    let se = stderr(&out);

    // stdout: the changed-ref list (advanced rows) — contract preserved.
    assert!(
        so.lines().any(|l| l == "refs/heads/main"),
        "stdout carries the advanced trunk ref verbatim: {so:?}",
    );
    // stderr: per-row disposition line for the trunk (main was checked out → the
    // ff-merge resync disposition) plus the trailing replayed count.
    assert!(
        se.contains("(advanced+resynced)"),
        "stderr names the exact advanced+resynced disposition: {se}",
    );
    assert!(
        se.contains("integrate: refs/heads/main "),
        "stderr carries the per-row trunk detail line: {se}",
    );
    assert!(
        se.contains("ref(s) replayed"),
        "the trailing summary line is preserved: {se}",
    );
    // The already-at-tip prepared rows report the no-op token.
    assert!(
        se.contains("(no-op)"),
        "no-op rows are reported with the exact token: {se}",
    );
}

/// VT-5: `--integrate` is the same Orchestrator verb class — a marker-present
/// linked worktree AND `DOCTRINE_WORKER=1` each refuse it, writing no trunk.
#[test]
fn integrate_refused_under_worker_mode() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    let trunk_before = git(dir, &["rev-parse", "main"]);

    // (1) Marker-present linked worktree, env unset ⇒ refused, names the verb.
    let holder = tempfile::tempdir().unwrap();
    let base = git(dir, &["rev-parse", "HEAD"]);
    let linked = holder.path().join("fork");
    git(
        dir,
        &[
            "worktree",
            "add",
            "-b",
            "wkr-int",
            linked.to_str().unwrap(),
            &base,
        ],
    );
    let marker_dir = linked.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&marker_dir).unwrap();
    std::fs::write(marker_dir.join("worker"), b"").unwrap();

    let out = run(
        &linked,
        None,
        &[
            "dispatch",
            "sync",
            "--integrate",
            "--trunk",
            "refs/heads/main",
            "--slice",
            "64",
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "refused from a marked linked worktree"
    );
    assert!(
        stderr(&out).contains("dispatch-sync"),
        "refusal names the verb: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        trunk_before,
        "no trunk write"
    );

    // (2) DOCTRINE_WORKER set ⇒ dual-cause refusal, still no trunk write.
    let out = run(
        dir,
        Some(true),
        &[
            "dispatch",
            "sync",
            "--integrate",
            "--trunk",
            "refs/heads/main",
            "--slice",
            "64",
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "refused when DOCTRINE_WORKER set");
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "carries the dual-cause: {}",
        stderr(&out)
    );
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        trunk_before,
        "still no trunk write"
    );
}

// --- PHASE-06 VT-2: the funnel-time `record-boundary` recording verb ----------
//
// The claude-arm phase cut (§4.3) consumes `boundaries.toml`; the orchestrator
// populates it during the funnel via this verb (the surface the skills cite).
// Pins: (a) it appends a `[[boundary]]` row at the CANONICAL padded ledger path
// `.doctrine/dispatch/064/` (the path `dispatch sync` tree-reads — writer↔reader
// agreement), and (b) it is Orchestrator-classed (refused under worker-mode).

fn record_boundary(cwd: &Path, root: &Path, phase: &str, start: &str, end: &str) -> Output {
    run(
        cwd,
        None,
        &[
            "dispatch",
            "record-boundary",
            "--slice",
            "64",
            "--phase",
            phase,
            "--code-start",
            start,
            "--code-end",
            end,
            "-p",
            root.to_str().unwrap(),
        ],
    )
}

#[test]
fn record_boundary_appends_row_at_canonical_padded_ledger_path() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fx = build_fixture(dir);
    let ledger = dir.join(".doctrine/dispatch/064/boundaries.toml");

    let out = record_boundary(dir, dir, "PHASE-09", &fx.base, &fx.code_end_1);
    assert!(
        out.status.success(),
        "record-boundary ok; stderr: {}",
        stderr(&out)
    );
    let body = std::fs::read_to_string(&ledger).expect("ledger written at padded path");
    assert!(body.contains("[[boundary]]"), "row header: {body}");
    assert!(body.contains("phase = \"PHASE-09\""), "phase row: {body}");
    // Stores the resolved code tip (full oid) the phase cut snapshots.
    assert!(body.contains(&fx.code_end_1), "code_end oid: {body}");

    // Append-only: a second record adds a second row, keeps the first.
    let out = record_boundary(dir, dir, "PHASE-10", &fx.code_end_1, &fx.code_end_2);
    assert!(out.status.success(), "second record ok: {}", stderr(&out));
    let body = std::fs::read_to_string(&ledger).unwrap();
    assert!(
        body.contains("PHASE-09") && body.contains("PHASE-10"),
        "both rows present: {body}"
    );
}

#[test]
fn record_boundary_refused_under_worker_mode() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fx = build_fixture(dir);
    let ledger = dir.join(".doctrine/dispatch/064/boundaries.toml");

    // (1) Marker-present linked worktree, env unset ⇒ refused, names the verb.
    let holder = tempfile::tempdir().unwrap();
    let base = git(dir, &["rev-parse", "HEAD"]);
    let linked = holder.path().join("fork");
    git(
        dir,
        &[
            "worktree",
            "add",
            "-b",
            "wkr-guard-rb",
            linked.to_str().unwrap(),
            &base,
        ],
    );
    let marker_dir = linked.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&marker_dir).unwrap();
    std::fs::write(marker_dir.join("worker"), b"").unwrap();

    let out = record_boundary(&linked, dir, "PHASE-09", &fx.base, &fx.code_end_1);
    assert!(
        !out.status.success(),
        "refused from a marked linked worktree"
    );
    assert!(
        stderr(&out).contains("dispatch-record-boundary"),
        "refusal names the verb: {}",
        stderr(&out)
    );
    assert!(!ledger.exists(), "refused run records nothing");

    // (2) DOCTRINE_WORKER set ⇒ refused, dual-cause token.
    let out = run(
        dir,
        Some(true),
        &[
            "dispatch",
            "record-boundary",
            "--slice",
            "64",
            "--phase",
            "PHASE-09",
            "--code-start",
            &fx.base,
            "--code-end",
            &fx.code_end_1,
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(!out.status.success(), "refused when DOCTRINE_WORKER set");
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "carries the dual-cause: {}",
        stderr(&out)
    );
    assert!(!ledger.exists(), "still records nothing");
}

// SL-147 PHASE-04 T3 — the funnel record beat ALSO writes the arm-neutral
// recorded source-delta registry (`.doctrine/state/slice/<NNN>/boundaries.toml`),
// ALONGSIDE — never replacing — the committed claude-arm ledger
// (`.doctrine/dispatch/<NNN>/boundaries.toml`). Both files get the row; the two
// are independent artifacts.
#[test]
fn record_boundary_also_writes_the_arm_neutral_registry() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fx = build_fixture(dir);
    let committed_ledger = dir.join(".doctrine/dispatch/064/boundaries.toml");
    let neutral_registry = dir.join(".doctrine/state/slice/064/boundaries.toml");

    let out = record_boundary(dir, dir, "PHASE-09", &fx.base, &fx.code_end_1);
    assert!(out.status.success(), "record-boundary ok: {}", stderr(&out));

    // (1) The committed ledger still carries the row (untouched behaviour).
    let committed = std::fs::read_to_string(&committed_ledger).expect("committed ledger written");
    assert!(
        committed.contains("phase = \"PHASE-09\""),
        "committed: {committed}"
    );

    // (2) The arm-neutral registry under the runtime state tree ALSO carries it.
    let neutral = std::fs::read_to_string(&neutral_registry).expect("neutral registry written");
    assert!(
        neutral.contains("[[boundary]]"),
        "neutral header: {neutral}"
    );
    assert!(
        neutral.contains("phase = \"PHASE-09\""),
        "neutral phase: {neutral}"
    );
    assert!(
        neutral.contains(&fx.code_end_1),
        "neutral end oid: {neutral}"
    );

    // (3) SL-154 PHASE-05 EX-1 pin: the funnel stamps `provenance = funnel` on
    // BOTH writes — the committed ledger and the registry — so the prepare-review
    // guard/derive (D11) can discriminate funnel-owned rows from solo/manual.
    assert!(
        committed.contains("provenance = \"funnel\""),
        "committed ledger stamps funnel provenance: {committed}"
    );
    assert!(
        neutral.contains("provenance = \"funnel\""),
        "registry stamps funnel provenance: {neutral}"
    );
}

// ====================================================================
// PHASE-03 (SL-121) — close-verify read surface (EX-1 / VT-1, design §3(b))
// ====================================================================

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Read the committed `dispatch/064` journal's trunk-row `planned_new_oid` via the
/// new read flag, naming the row with `--trunk <ref>`. Runs in `cwd`.
fn show_journal_trunk_oid(cwd: &Path, trunk: &str) -> Output {
    run(
        cwd,
        None,
        &[
            "dispatch",
            "sync",
            "--show-journal-trunk-oid",
            "--slice",
            "64",
            "--trunk",
            trunk,
            "-p",
            cwd.to_str().unwrap(),
        ],
    )
}

/// VT-1 / EX-1: after a trunk integrate journals the trunk row, the read surface
/// returns that row's full `planned_new_oid` from a checkout where `dispatch/064`
/// is NOT checked out (the working tree is on `main`) — a tree-read of the
/// committed journal (the `sync-tree-reads-ledger-not-worktree` invariant), not a
/// transient `candidate admit` stdout. The value equals the cumulative code tip
/// that `main` was advanced to.
#[test]
fn show_journal_trunk_oid_returns_committed_planned_oid_from_any_checkout() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    let phase_tip = git(dir, &["rev-parse", "phase/064-02"]);

    assert!(
        integrate(dir, &["--trunk", "refs/heads/main"])
            .status
            .success(),
        "trunk integrate journals the trunk row"
    );
    // The working tree is on `main`; dispatch/064 has no live worktree.
    assert_eq!(git(dir, &["rev-parse", "--abbrev-ref", "HEAD"]), "main");

    let out = show_journal_trunk_oid(dir, "refs/heads/main");
    assert!(
        out.status.success(),
        "read surface succeeds; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        stdout(&out),
        phase_tip,
        "prints the trunk row's full planned_new_oid (== the projected tip)"
    );
}

/// EX-1 (error path): a journal with no row for the named trunk ref (only
/// `--prepare-review` ran — no trunk row journaled) refuses with a named token and
/// prints no oid to stdout, so the close skill never diffs against an empty value.
#[test]
fn show_journal_trunk_oid_errors_when_no_trunk_row() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());

    let out = show_journal_trunk_oid(dir, "refs/heads/main");
    assert!(
        !out.status.success(),
        "no trunk row in the journal ⇒ refused"
    );
    assert!(
        stderr(&out).contains("show-journal-trunk-oid"),
        "refusal names the read surface: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).is_empty(),
        "no oid emitted on the error path: {:?}",
        stdout(&out)
    );
}

/// SL-128 VT-1: with no `--trunk`, the read stage defaults the ref from
/// `[dispatch] deliver_to` (absent config ⇒ refs/heads/main) — resolving to
/// the same trunk row as an explicit `--trunk refs/heads/main`.
#[test]
fn show_journal_trunk_oid_defaults_trunk_from_deliver_to() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    assert!(
        integrate(dir, &["--trunk", "refs/heads/main"])
            .status
            .success()
    );

    let explicit = show_journal_trunk_oid(dir, "refs/heads/main");
    assert!(explicit.status.success(), "explicit: {}", stderr(&explicit));

    let defaulted = run(
        dir,
        None,
        &[
            "dispatch",
            "sync",
            "--show-journal-trunk-oid",
            "--slice",
            "64",
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(
        defaulted.status.success(),
        "no `--trunk` defaults from config: {}",
        stderr(&defaulted)
    );
    assert_eq!(
        stdout(&explicit),
        stdout(&defaulted),
        "config default resolves to the same trunk row as explicit refs/heads/main"
    );
}

/// SL-128 VT-1 (precedence): an explicit `--trunk` overrides a configured
/// `deliver_to`. Config names a ref with no trunk row, but the explicit
/// `refs/heads/main` still reads the real row.
#[test]
fn show_journal_trunk_oid_explicit_trunk_overrides_config() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    assert!(
        integrate(dir, &["--trunk", "refs/heads/main"])
            .status
            .success()
    );
    std::fs::write(
        dir.join(common::DOCTRINE_TOML),
        "[dispatch]\ndeliver-to = \"refs/heads/other\"\n",
    )
    .unwrap();

    let out = show_journal_trunk_oid(dir, "refs/heads/main");
    assert!(
        out.status.success(),
        "explicit --trunk wins over config deliver_to: {}",
        stderr(&out)
    );
    assert!(!stdout(&out).trim().is_empty(), "prints the trunk-row oid");
}

/// SL-128 VT-2: the `dispatch deliver-to` verb prints the resolved ref —
/// the default when unset, the configured value when present.
#[test]
fn dispatch_deliver_to_prints_default_and_override() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);

    let out = run(
        dir,
        None,
        &["dispatch", "deliver-to", "-p", dir.to_str().unwrap()],
    );
    assert!(out.status.success(), "default: {}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "refs/heads/main");

    std::fs::write(
        dir.join(common::DOCTRINE_TOML),
        "[dispatch]\ndeliver-to = \"refs/heads/release\"\n",
    )
    .unwrap();
    let out = run(
        dir,
        None,
        &["dispatch", "deliver-to", "-p", dir.to_str().unwrap()],
    );
    assert!(out.status.success(), "override: {}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "refs/heads/release");
}

// ====================================================================
// PHASE-02 (SL-126) — close-integration gate, end-to-end against a REAL
// journal. The keystone: PHASE-01's unit fixtures encode the row shape, but
// only a real `prepare_review` + `integrate --trunk` proves git produces a
// trunk row the gate reads as Integrated. Both arms drive a slice fixture at
// `reconcile` to `done` via the BUILT binary's `slice status`.
// ====================================================================

/// Materialise slice 064's authored `slice-064.toml` at `status = "reconcile"` in
/// the fixture's working tree (the legal source for `→ done`). The gate reads the
/// working filesystem for the authored status; no reqs/coverage ⇒ the blocker and
/// drift gates pass, leaving the integration gate as the sole arbiter.
fn write_reconcile_slice_64(dir: &Path) {
    let body = "id = 64\n\
         slug = \"dispatched-slice\"\n\
         title = \"Dispatched slice\"\n\
         status = \"reconcile\"\n\
         created = \"2026-06-20\"\n\
         updated = \"2026-06-20\"\n\
         \n[relationships]\nneeds = []\nafter = []\n";
    let path = dir.join(".doctrine/slice/064/slice-064.toml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, body).unwrap();
}

/// Drive `slice status 64 done -p <dir>` over the built binary.
fn slice_status_done(dir: &Path) -> Output {
    run(
        dir,
        None,
        &["slice", "status", "64", "done", "-p", dir.to_str().unwrap()],
    )
}

/// VT-7(a): after a real `prepare_review` + `integrate --trunk refs/heads/main`,
/// the trunk row's planned oid is an ancestor of `main` — the gate reads the
/// git-produced journal as Integrated and `reconcile → done` SUCCEEDS.
#[test]
fn vt7_close_integration_succeeds_after_real_trunk_integrate() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        out.status.success(),
        "trunk integrate; stderr: {}",
        stderr(&out)
    );

    write_reconcile_slice_64(dir);
    let out = slice_status_done(dir);
    assert!(
        out.status.success(),
        "integrated slice closes; stderr: {}",
        stderr(&out)
    );
    // The authored status advanced to done.
    let toml = std::fs::read_to_string(dir.join(".doctrine/slice/064/slice-064.toml")).unwrap();
    assert!(
        toml.contains("status = \"done\""),
        "close wrote the terminal status: {toml}"
    );
}

/// VT-7(b): `prepare_review` ran but `--trunk` was NOT integrated, so the journal
/// carries no `refs/heads/main` row — the gate fails closed and `reconcile → done`
/// is REFUSED, naming the missing-trunk-row anomaly.
#[test]
fn vt7_close_integration_refused_without_trunk_integrate() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    // Default integrate (no `--trunk`) — replays review/phase refs, no trunk row.
    assert!(integrate(dir, &[]).status.success());

    write_reconcile_slice_64(dir);
    let out = slice_status_done(dir);
    assert!(
        !out.status.success(),
        "an unintegrated dispatched slice is refused close"
    );
    assert!(
        stderr(&out).contains("not integrated to trunk")
            && stderr(&out).contains("integrate --trunk never completed"),
        "refusal names the no-trunk-row anomaly: {}",
        stderr(&out)
    );
    // Refused before the write — the authored status stays at reconcile.
    let toml = std::fs::read_to_string(dir.join(".doctrine/slice/064/slice-064.toml")).unwrap();
    assert!(
        toml.contains("status = \"reconcile\""),
        "no write on refusal: {toml}"
    );
}

/// SL-128 VT-2: the close-integration gate reads `[dispatch] deliver_to`, not a
/// hardcoded ref. Trunk is integrated on refs/heads/main, but project config
/// redirects deliver_to to a ref with NO trunk row, so the gate checks the
/// configured ref and REFUSES — proving the literal was retired.
#[test]
fn vt2_close_integration_honours_deliver_to_override() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    assert!(prepare_review(dir).status.success());
    // Real trunk row lands on main.
    assert!(
        integrate(dir, &["--trunk", "refs/heads/main"])
            .status
            .success()
    );
    // Redirect the delivery ref away from main via project config.
    let cfg = dir.join(common::DOCTRINE_TOML);
    let mut body = std::fs::read_to_string(&cfg).unwrap_or_default();
    body.push_str("\n[dispatch]\ndeliver-to = \"refs/heads/other\"\n");
    std::fs::write(&cfg, body).unwrap();

    write_reconcile_slice_64(dir);
    let out = slice_status_done(dir);
    assert!(
        !out.status.success(),
        "gate must check the configured ref (refs/heads/other), which has no trunk row; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("not integrated to trunk"),
        "refusal cites the integration gate: {}",
        stderr(&out)
    );
}

// ============================================================================
// SL-154 PHASE-05 (ISS-052) — projection-source guard (D11) + derive + the
// primary-rooted completeness gate at prepare-review, ALL before the ref
// projection (design §5.2). A halt creates NO review/phase refs.
// ============================================================================

/// A dispatch repo: `main` (trunk + a gitignored runtime tier) and a
/// `dispatch/064` branch carrying two code phases, but NO committed boundaries
/// ledger — the caller commits a ledger / seeds the registry per scenario.
fn build_guard_repo(dir: &Path) -> (String, String, String) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::write(dir.join(".gitignore"), ".doctrine/state/\n").unwrap();
    git(dir, &["add", ".gitignore"]);
    let base = commit(dir, "trunk.txt", "trunk", "base");
    git(dir, &["checkout", "-q", "-b", "dispatch/064"]);
    let c1 = commit(dir, "src1.txt", "a", "phase1 code");
    let c2 = commit(dir, "src2.txt", "b", "phase2 code");
    git(dir, &["checkout", "-q", "main"]);
    (base, c1, c2)
}

/// Commit a boundaries-ledger body onto `dispatch/064`, then return to `main`.
fn commit_ledger_on_dispatch(dir: &Path, body: &str) {
    git(dir, &["checkout", "-q", "dispatch/064"]);
    let p = dir.join(".doctrine/dispatch/064");
    std::fs::create_dir_all(&p).unwrap();
    std::fs::write(p.join("boundaries.toml"), body).unwrap();
    git(dir, &["add", ".doctrine/dispatch/064"]);
    git(dir, &["commit", "-q", "-m", "ledger"]);
    git(dir, &["checkout", "-q", "main"]);
}

/// Write the runtime registry (primary tree) directly — a pre-existing row state.
fn seed_registry(dir: &Path, body: &str) {
    let p = dir.join(".doctrine/state/slice/064");
    std::fs::create_dir_all(&p).unwrap();
    std::fs::write(p.join("boundaries.toml"), body).unwrap();
}

fn boundary_row(phase: &str, start: &str, end: &str, provenance: &str) -> String {
    format!(
        "[[boundary]]\nphase = \"{phase}\"\ncode_start_oid = \"{start}\"\ncode_end_oid = \"{end}\"\nprovenance = \"{provenance}\"\n"
    )
}

/// `doctrine slice record-delta 64 <phase> --start <a> --end <b>` (Manual escape hatch).
fn record_delta(dir: &Path, phase: &str, start: &str, end: &str) -> Output {
    run(
        dir,
        None,
        &[
            "slice",
            "record-delta",
            "64",
            phase,
            "--start",
            start,
            "--end",
            end,
            "-p",
            dir.to_str().unwrap(),
        ],
    )
}

/// Run prepare-review from an arbitrary `cwd`, rooted at `root` (for the
/// coord-cwd case `primary_worktree(root)` resolves up to the main tree).
fn prepare_review_from(cwd: &Path, root: &Path) -> Output {
    run(
        cwd,
        None,
        &[
            "dispatch",
            "sync",
            "--prepare-review",
            "--slice",
            "64",
            "-p",
            root.to_str().unwrap(),
        ],
    )
}

// VT-1: total loss — the registry records funnel rows the committed ledger never
// carried (coord worktree removed before prepare-review) → the guard halts naming
// the phases, and NO review/phase refs are created.
#[test]
fn vt1_guard_halts_on_total_loss_creating_no_refs() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let (base, c1, c2) = build_guard_repo(dir);
    seed_registry(
        dir,
        &format!(
            "{}{}",
            boundary_row("PHASE-01", &base, &c1, "funnel"),
            boundary_row("PHASE-02", &c1, &c2, "funnel"),
        ),
    );
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02"]);

    let out = prepare_review(dir);
    assert!(!out.status.success(), "guard halts on total loss");
    let err = stderr(&out);
    assert!(
        err.contains("PHASE-01") && err.contains("PHASE-02"),
        "names the lost phases: {err}"
    );
    assert!(
        !ref_exists(dir, "review/064"),
        "no review ref created on halt"
    );
    assert!(
        !ref_exists(dir, "phase/064-01"),
        "no phase ref created on halt"
    );
}

// VT-3: no false-halt — a Solo row (the binding, D11-excluded) plus a funnel
// empty-code phase (start==end, recorded in BOTH ledgers) → prepare-review
// succeeds. The guard never inspects code paths.
#[test]
fn vt3_no_false_halt_on_solo_rows_and_empty_code_phase() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let (base, c1, c2) = build_guard_repo(dir);
    // Committed ledger: only the empty-code funnel phase 02 (start==end).
    commit_ledger_on_dispatch(dir, &boundary_row("PHASE-02", &c2, &c2, "funnel"));
    // Registry: a Solo binding row for 01 + the funnel empty-code 02.
    seed_registry(
        dir,
        &format!(
            "{}{}",
            boundary_row("PHASE-01", &base, &c1, "solo"),
            boundary_row("PHASE-02", &c2, &c2, "funnel"),
        ),
    );
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02"]);

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "no false-halt on Solo + empty-code; stderr: {}",
        stderr(&out)
    );
}

// VT-5: derive is authoritative — a binding-written garbage row for a funnel
// phase is overwritten by the committed-ledger row, and an N-row committed ledger
// populates N registry rows.
#[test]
fn vt5_derive_overwrites_garbage_and_populates_every_row() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let (base, c1, c2) = build_guard_repo(dir);
    commit_ledger_on_dispatch(
        dir,
        &format!(
            "{}{}",
            boundary_row("PHASE-01", &base, &c1, "funnel"),
            boundary_row("PHASE-02", &c1, &c2, "funnel"),
        ),
    );
    // A mis-firing binding wrote a GARBAGE range for funnel PHASE-01 (base..base).
    seed_registry(dir, &boundary_row("PHASE-01", &base, &base, "funnel"));
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02"]);

    let out = prepare_review(dir);
    assert!(
        out.status.success(),
        "derive heals; stderr: {}",
        stderr(&out)
    );
    let reg =
        std::fs::read_to_string(dir.join(".doctrine/state/slice/064/boundaries.toml")).unwrap();
    assert!(
        reg.contains("PHASE-01") && reg.contains("PHASE-02"),
        "all committed phases populate the registry: {reg}"
    );
    assert!(
        reg.contains(&c1),
        "the garbage PHASE-01 range is overwritten by the committed end {c1}: {reg}"
    );
}

// VT-6: the gate is primary-rooted — run prepare-review from a COORD-tree cwd; a
// phase completed only in the PRIMARY tree (no row) still halts the gate, proving
// it reads the primary completed-set, not the coord cwd (the ISS-052 regression).
#[test]
fn vt6_gate_is_primary_rooted_when_run_from_coord_cwd() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    // Live coord on dispatch/064; the builder seeds PRIMARY completed 01/02/03.
    let coord = build_fixture_uncommitted_ledger(dir, "");
    let base = git(dir, &["rev-parse", "dispatch/064~3"]);
    let c1 = git(dir, &["rev-parse", "dispatch/064~2"]);
    let c2 = git(dir, &["rev-parse", "dispatch/064~1"]);
    // The working ledger covers only 01/02 → PHASE-03 (completed in primary) is a gap.
    std::fs::write(
        coord.join(".doctrine/dispatch/064/boundaries.toml"),
        format!(
            "{}{}",
            boundary_row("PHASE-01", &base, &c1, "funnel"),
            boundary_row("PHASE-02", &c1, &c2, "funnel"),
        ),
    )
    .unwrap();

    let out = prepare_review_from(&coord, &coord);
    assert!(
        !out.status.success(),
        "gate halts on the primary-completed gap even when run from the coord cwd"
    );
    assert!(
        stderr(&out).contains("PHASE-03"),
        "the halt names the primary-completed gap PHASE-03: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "review/064"),
        "no refs created on the gate halt"
    );
}

// VT-7: gate-before-projection — an incomplete registry halts with NO refs; after
// the operator records the missing phase, a re-run projects cleanly (the headline
// ISS-052 outcome — the clean re-run the PHASE-05 gate makes possible).
#[test]
fn vt7_gate_halts_before_projection_then_rerun_projects_after_fix() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let (base, c1, c2) = build_guard_repo(dir);
    // Committed ledger covers 01/02 (funnel); PHASE-03 is completed but unrecorded.
    commit_ledger_on_dispatch(
        dir,
        &format!(
            "{}{}",
            boundary_row("PHASE-01", &base, &c1, "funnel"),
            boundary_row("PHASE-02", &c1, &c2, "funnel"),
        ),
    );
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02", "PHASE-03"]);

    // Run 1: the gate halts (03 has no row) BEFORE projection — no refs created.
    let out = prepare_review(dir);
    assert!(!out.status.success(), "gate halts: {}", stderr(&out));
    assert!(
        stderr(&out).contains("PHASE-03"),
        "the halt names the gap PHASE-03: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "review/064") && !ref_exists(dir, "phase/064-01"),
        "gate-before-projection: NO refs exist after the halt"
    );

    // The operator records the missing phase (Manual escape hatch).
    let rd = record_delta(dir, "PHASE-03", &c2, &c2);
    assert!(rd.status.success(), "record-delta ok: {}", stderr(&rd));

    // Run 2: the registry is complete → projects cleanly, refs created.
    let out2 = prepare_review(dir);
    assert!(
        out2.status.success(),
        "the clean re-run projects: {}",
        stderr(&out2)
    );
    assert!(
        ref_exists(dir, "review/064"),
        "review ref created on the clean re-run"
    );
    assert!(
        ref_exists(dir, "phase/064-01") && ref_exists(dir, "phase/064-02"),
        "phase cuts created on the clean re-run"
    );
}
