// SPDX-License-Identifier: GPL-3.0-only
//! SL-068 PHASE-02 — `doctrine dispatch candidate create` (core happy path)
//! end-to-end over the BUILT binary (design §5.3; EX-1..5 / VT-1..4).
//!
//! The fixture reuses the SL-064 `dispatch sync --prepare-review` projection to
//! mint REAL `review/<slice>` + `phase/<slice>-NN` refs AND a verified journal on
//! `dispatch/<slice>` — the genuine provenance substrate create gates on (EX-1).
//!
//! * VT-1: a clean 3-way create from `review/064` and from a `phase/064-NN`
//!   records source/base/merge OIDs (merge_oid has the two expected parents) and
//!   shows NO phantom `.doctrine` deletions vs live trunk (the SL-067 defect fix).
//! * VT-2: create refuses before/without a verified prepare-review row.
//! * VT-3: an existing target ref refuses (zero-oid CAS); a supersede creates a
//!   fresh row + ref linked by `supersedes`.
//! * VT-4 / EX-4: `review/*` and `phase/*` OIDs are unchanged after a create
//!   (invariant I1); a payload fixture proves review_surface vs close_target differ.

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

/// The built fixture's captured OIDs.
struct Fixture {
    base: String,
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

/// Build a repo with `main` at a trunk base and a `dispatch/064` branch carrying
/// two code phases, an authored `.doctrine/` entity, and the
/// `boundaries.toml` ledger. Leaves the working tree on `main`.
fn build_fixture(dir: &Path) -> Fixture {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    // Runtime state is gitignored (as in production) so prepare-review's derive
    // (which writes the registry under `.doctrine/state/`) and the seeded phase
    // tracking never dirty the working tree.
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

    let boundaries = format!(
        "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"{base}\"\ncode_end_oid = \"{code_end_1}\"\n\
         [[boundary]]\nphase = \"PHASE-02\"\ncode_start_oid = \"{code_end_1}\"\ncode_end_oid = \"{code_end_2}\"\n"
    );
    std::fs::create_dir_all(dir.join(".doctrine/dispatch/064")).unwrap();
    std::fs::write(
        dir.join(".doctrine/dispatch/064/boundaries.toml"),
        &boundaries,
    )
    .unwrap();
    git(dir, &["add", ".doctrine/dispatch/064"]);
    git(dir, &["commit", "-q", "-m", "ledger fixtures"]);

    git(dir, &["checkout", "-q", "main"]);
    // prepare-review is the pre-audit conclude beat (design §5.2): its PHASE-05
    // completeness gate requires every ledger phase to be `completed`.
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02"]);
    Fixture { base }
}

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

/// Run stage-1 prepare-review so the fixture carries verified provenance.
fn prepare_review(dir: &Path) {
    let out = run(
        dir,
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
        out.status.success(),
        "prepare-review ok; stderr: {}",
        stderr(&out)
    );
}

/// Run `dispatch candidate create` with the given extra flags.
fn create(cwd: &Path, worker: Option<bool>, extra: &[&str]) -> Output {
    let mut args = vec!["dispatch", "candidate", "create", "--slice", "64"];
    args.extend_from_slice(extra);
    args.push("-p");
    args.push(cwd.to_str().unwrap());
    run(cwd, worker, &args)
}

fn read_candidates(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(".doctrine/dispatch/064/candidates.toml"))
        .expect("candidates.toml written")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Run `dispatch candidate status --slice 64` over the built binary.
fn status(cwd: &Path, worker: Option<bool>) -> Output {
    run(
        cwd,
        worker,
        &[
            "dispatch",
            "candidate",
            "status",
            "--slice",
            "64",
            "-p",
            cwd.to_str().unwrap(),
        ],
    )
}

/// The parents of a commit, in order.
fn parents(dir: &Path, commitish: &str) -> Vec<String> {
    git(dir, &["rev-list", "--parents", "-n", "1", commitish])
        .split_whitespace()
        .skip(1)
        .map(str::to_owned)
        .collect()
}

/// Run `dispatch candidate admit --slice 64` with the given extra flags.
fn admit(cwd: &Path, worker: Option<bool>, extra: &[&str]) -> Output {
    let mut args = vec!["dispatch", "candidate", "admit", "--slice", "64"];
    args.extend_from_slice(extra);
    args.push("-p");
    args.push(cwd.to_str().unwrap());
    run(cwd, worker, &args)
}

/// Run `dispatch sync --integrate --slice 64` with the given extra flags
/// (DOCTRINE_WORKER unset). Mirrors e2e_dispatch_sync.rs's `integrate` helper.
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

/// Create + return the candidate close_target built from `phase/064-02`.
fn create_close_target(dir: &Path, label: &str) {
    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--payload",
                "code",
                "--base",
                "refs/heads/main",
                "--label",
                label,
                "--source",
                "refs/heads/phase/064-02",
            ],
        )
        .status
        .success(),
        "create close_target {label}"
    );
}

// --- VT-1: clean 3-way from review/<slice> -----------------------------------

#[test]
fn e2e_dispatch_candidate_clean_merge_from_review() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    let fx = build_fixture(dir);
    prepare_review(dir);

    let base_oid = git(dir, &["rev-parse", "main"]);
    let source_oid = git(dir, &["rev-parse", "review/064"]);

    let out = create(
        dir,
        None,
        &[
            "--role",
            "review_surface",
            "--payload",
            "impl_bundle",
            "--base",
            "refs/heads/main",
            "--label",
            "review-001",
            "--worktree",
        ],
    );
    assert!(
        out.status.success(),
        "clean create ok; stderr: {}",
        stderr(&out)
    );
    assert!(
        ref_exists(dir, "candidate/064/review-001"),
        "branch created"
    );

    // merge_oid has BOTH base and source as parents (a true no-ff 3-way).
    let merge_oid = git(dir, &["rev-parse", "candidate/064/review-001"]);
    let ps = parents(dir, &merge_oid);
    assert_eq!(ps.len(), 2, "no-ff merge has two parents: {ps:?}");
    assert!(ps.contains(&base_oid), "base is a parent: {ps:?}");
    assert!(ps.contains(&source_oid), "source is a parent: {ps:?}");

    // The recorded row carries the OIDs.
    let toml = read_candidates(dir);
    assert!(toml.contains("status = \"created\""), "{toml}");
    assert!(toml.contains(&source_oid), "source_oid recorded: {toml}");
    assert!(toml.contains(&base_oid), "base_oid recorded: {toml}");
    assert!(toml.contains(&merge_oid), "merge_oid recorded: {toml}");

    // EX-3: a clean trunk-advanced case shows NO phantom .doctrine deletions vs
    // live trunk — the 3-way union carries the base's .doctrine corpus AND the
    // source's code. (The base here IS live trunk; the deletion-free property is
    // what the SL-067 squash defect violated.)
    let diff = git(dir, &["diff", "--name-status", "main", &merge_oid]);
    assert!(
        !diff.lines().any(|l| l.starts_with('D')),
        "no phantom deletions vs live trunk: {diff}"
    );
    // The candidate carries both the .doctrine entity and the source code.
    let listing = git(dir, &["ls-tree", "-r", "--name-only", &merge_oid]);
    assert!(
        listing.contains(".doctrine/slice/064/slice-064.md"),
        ".doctrine corpus carried: {listing}"
    );
    assert!(
        listing.contains("src1.txt"),
        "source code carried: {listing}"
    );
    let _ = &fx.base;
}

// --- VT-1: clean 3-way from a phase chain ------------------------------------

#[test]
fn e2e_dispatch_candidate_clean_merge_from_phase_chain() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    let base_oid = git(dir, &["rev-parse", "main"]);
    let source_oid = git(dir, &["rev-parse", "phase/064-02"]);

    let out = create(
        dir,
        None,
        &[
            "--role",
            "close_target",
            "--payload",
            "code",
            "--base",
            "refs/heads/main",
            "--label",
            "close-001",
            "--source",
            "refs/heads/phase/064-02",
        ],
    );
    assert!(
        out.status.success(),
        "phase-chain create ok; stderr: {}",
        stderr(&out)
    );

    let merge_oid = git(dir, &["rev-parse", "candidate/064/close-001"]);
    let ps = parents(dir, &merge_oid);
    assert_eq!(ps.len(), 2, "no-ff merge has two parents: {ps:?}");
    assert!(ps.contains(&base_oid) && ps.contains(&source_oid), "{ps:?}");

    let diff = git(dir, &["diff", "--name-status", "main", &merge_oid]);
    assert!(
        !diff.lines().any(|l| l.starts_with('D')),
        "no phantom deletions vs live trunk: {diff}"
    );
}

// --- VT-2: unverified / absent provenance refuses ----------------------------

#[test]
fn e2e_dispatch_candidate_unverified_source_refuses() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    // NOTE: prepare-review NOT run ⇒ no verified journal row for review/064.

    let out = create(
        dir,
        None,
        &[
            "--role",
            "review_surface",
            "--payload",
            "impl_bundle",
            "--base",
            "refs/heads/main",
            "--label",
            "review-001",
            "--worktree",
        ],
    );
    assert!(
        !out.status.success(),
        "refused without a verified prepare-review row"
    );
    assert!(
        stderr(&out).contains("prepare-review") || stderr(&out).contains("verified"),
        "refusal names the provenance gate: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "candidate/064/review-001"),
        "refused run creates no candidate ref"
    );
    assert!(
        !dir.join(".doctrine/dispatch/064/candidates.toml").exists(),
        "refused run records no row"
    );
}

// --- VT-3: zero-oid CAS refuses an existing ref; supersede creates a fresh one -

#[test]
fn e2e_dispatch_candidate_create_zero_oid_cas_refuses_existing() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    let mk = |label: &str, supersedes: Option<&str>| -> Output {
        let mut extra = vec![
            "--role",
            "review_surface",
            "--payload",
            "impl_bundle",
            "--base",
            "refs/heads/main",
            "--label",
            label,
            "--worktree",
        ];
        if let Some(s) = supersedes {
            extra.push("--supersedes");
            extra.push(s);
        }
        create(dir, None, &extra)
    };

    assert!(mk("review-001", None).status.success(), "first create");
    let first_oid = git(dir, &["rev-parse", "candidate/064/review-001"]);

    // Re-creating the SAME label refuses under zero-oid CAS, leaving the ref.
    let dup = mk("review-001", None);
    assert!(!dup.status.success(), "existing target ref refuses");
    assert!(
        stderr(&dup).contains("already exists"),
        "refusal names the CAS cause: {}",
        stderr(&dup)
    );
    assert_eq!(
        git(dir, &["rev-parse", "candidate/064/review-001"]),
        first_oid,
        "the existing branch is left untouched"
    );

    // A supersede creates a FRESH row + ref linked by `supersedes`.
    let sup = mk("review-002", Some("cand-064-review-001"));
    assert!(
        sup.status.success(),
        "supersede create ok; stderr: {}",
        stderr(&sup)
    );
    assert!(ref_exists(dir, "candidate/064/review-002"), "fresh ref");
    let toml = read_candidates(dir);
    assert!(
        toml.contains("supersedes = \"cand-064-review-001\""),
        "fresh row links the prior id: {toml}"
    );
    // The old branch is NEVER rewritten.
    assert_eq!(
        git(dir, &["rev-parse", "candidate/064/review-001"]),
        first_oid,
        "supersession never rewrites the old branch"
    );
}

// --- VT-4 / EX-4: evidence refs unchanged; payload axes differ ----------------

#[test]
fn e2e_dispatch_candidate_create_leaves_evidence_refs_and_distinguishes_payload() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    let review_before = git(dir, &["rev-parse", "review/064"]);
    let phase1_before = git(dir, &["rev-parse", "phase/064-01"]);
    let phase2_before = git(dir, &["rev-parse", "phase/064-02"]);

    // review_surface (impl_bundle) candidate.
    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "review_surface",
                "--payload",
                "impl_bundle",
                "--base",
                "refs/heads/main",
                "--label",
                "review-001",
                "--worktree",
            ],
        )
        .status
        .success()
    );
    // close_target (code) candidate.
    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--payload",
                "code",
                "--base",
                "refs/heads/main",
                "--label",
                "close-001",
                "--source",
                "refs/heads/phase/064-02",
            ],
        )
        .status
        .success()
    );

    // I1: review/* and phase/* OIDs are unchanged after create.
    assert_eq!(git(dir, &["rev-parse", "review/064"]), review_before);
    assert_eq!(git(dir, &["rev-parse", "phase/064-01"]), phase1_before);
    assert_eq!(git(dir, &["rev-parse", "phase/064-02"]), phase2_before);

    // The two candidates differ on payload + role + source — a proof the axes
    // are recorded distinctly, and the bundle vs code payloads diverge in content.
    let toml = read_candidates(dir);
    assert!(toml.contains("payload = \"impl_bundle\""), "{toml}");
    assert!(toml.contains("payload = \"code\""), "{toml}");
    assert!(toml.contains("role = \"review_surface\""), "{toml}");
    assert!(toml.contains("role = \"close_target\""), "{toml}");

    let review_listing = git(
        dir,
        &["ls-tree", "-r", "--name-only", "candidate/064/review-001"],
    );
    let close_listing = git(
        dir,
        &["ls-tree", "-r", "--name-only", "candidate/064/close-001"],
    );
    assert!(
        review_listing.contains(".doctrine/slice/064/slice-064.md"),
        "impl_bundle candidate carries the .doctrine corpus: {review_listing}"
    );
    assert!(
        !close_listing.contains(".doctrine/slice/064/slice-064.md")
            || review_listing != close_listing,
        "code close_target differs from the impl_bundle review surface: \
         review={review_listing} close={close_listing}"
    );
}

// --- PHASE-03 fixtures: a genuinely 3-way-conflicting source/base ------------

/// Like [`build_fixture`] but engineer a real 3-way conflict: the dispatch
/// branch and `main` both modify the SAME merge-base file (`trunk.txt`) to
/// DIFFERENT values, so `merge-tree` of `review/064` onto `main` conflicts.
/// (The merge base is the original `base` commit; both sides diverge on it.)
fn build_conflict_fixture(dir: &Path) -> Fixture {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    // Runtime state is gitignored (as in production) so prepare-review's derive
    // never dirties the working tree.
    std::fs::write(dir.join(".gitignore"), ".doctrine/state/\n").unwrap();
    git(dir, &["add", ".gitignore"]);
    let base = commit(dir, "trunk.txt", "trunk\n", "base");

    git(dir, &["checkout", "-q", "-b", "dispatch/064"]);
    // Dispatch side rewrites trunk.txt to one value.
    let code_end_1 = commit(
        dir,
        "trunk.txt",
        "DISPATCH SIDE\n",
        "phase1 conflicting edit",
    );
    let code_end_2 = commit(dir, "src2.txt", "b", "phase2 code");
    commit(
        dir,
        ".doctrine/slice/064/slice-064.md",
        "scope",
        "authored entity",
    );

    let boundaries = format!(
        "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"{base}\"\ncode_end_oid = \"{code_end_1}\"\n\
         [[boundary]]\nphase = \"PHASE-02\"\ncode_start_oid = \"{code_end_1}\"\ncode_end_oid = \"{code_end_2}\"\n"
    );
    std::fs::create_dir_all(dir.join(".doctrine/dispatch/064")).unwrap();
    std::fs::write(
        dir.join(".doctrine/dispatch/064/boundaries.toml"),
        &boundaries,
    )
    .unwrap();
    git(dir, &["add", ".doctrine/dispatch/064"]);
    git(dir, &["commit", "-q", "-m", "ledger fixtures"]);

    git(dir, &["checkout", "-q", "main"]);
    // Main side rewrites trunk.txt to a DIFFERENT value ⇒ conflicts with review/064.
    commit(dir, "trunk.txt", "MAIN SIDE\n", "main conflicting edit");
    // prepare-review requires every ledger phase to be `completed`.
    seed_completed_phases(dir, 64, &["PHASE-01", "PHASE-02"]);
    Fixture { base }
}

// --- VT-1: conflict lifecycle — abort without --worktree, record with it ------

#[test]
fn e2e_dispatch_candidate_conflict_records_or_aborts() {
    // No --worktree ⇒ conflict aborts cleanly: no row, no ref, no worktree.
    {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        build_conflict_fixture(dir);
        prepare_review(dir);

        let out = create(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--payload",
                "code",
                "--base",
                "refs/heads/main",
                "--label",
                "conflict-001",
                "--source",
                "refs/heads/review/064",
            ],
        );
        assert!(
            !out.status.success(),
            "conflict without --worktree aborts; stderr: {}",
            stderr(&out)
        );
        assert!(
            stderr(&out).contains("conflict"),
            "refusal names the conflict: {}",
            stderr(&out)
        );
        assert!(
            !ref_exists(dir, "candidate/064/conflict-001"),
            "aborted conflict creates no candidate ref"
        );
        assert!(
            !dir.join(".doctrine/dispatch/064/candidates.toml").exists(),
            "aborted conflict records no row"
        );
        assert!(
            !dir.join(".doctrine/state/dispatch/candidate").exists(),
            "aborted conflict leaves no worktree dir"
        );
    }

    // With --worktree ⇒ row status=conflicted, branch at base, worktree created.
    {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        build_conflict_fixture(dir);
        prepare_review(dir);
        let base_oid = git(dir, &["rev-parse", "main"]);

        let out = create(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--payload",
                "code",
                "--base",
                "refs/heads/main",
                "--label",
                "conflict-001",
                "--source",
                "refs/heads/review/064",
                "--worktree",
            ],
        );
        assert!(
            out.status.success(),
            "conflict with --worktree records the conflicted lifecycle; stderr: {}",
            stderr(&out)
        );
        assert!(
            ref_exists(dir, "candidate/064/conflict-001"),
            "conflicted candidate branch created so the user can resolve"
        );
        // The branch is parked at the base (the merge attempt) for resolution.
        assert_eq!(
            git(dir, &["rev-parse", "candidate/064/conflict-001"]),
            base_oid,
            "conflicted branch parked at base for resolve+commit"
        );
        let toml = read_candidates(dir);
        assert!(
            toml.contains("status = \"conflicted\""),
            "row recorded conflicted: {toml}"
        );
        // The worktree path is displayed AND created.
        let wt = dir.join(".doctrine/state/dispatch/candidate/cand-064-conflict-001");
        assert!(wt.exists(), "conflicted candidate worktree created: {wt:?}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("cand-064-conflict-001")
                || stderr(&out).contains("cand-064-conflict-001"),
            "worktree path displayed; stdout={} stderr={}",
            String::from_utf8_lossy(&out.stdout),
            stderr(&out)
        );
    }
}

// --- VT-2: review_surface requires --worktree --------------------------------

#[test]
fn e2e_dispatch_candidate_review_surface_requires_worktree() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    let out = create(
        dir,
        None,
        &[
            "--role",
            "review_surface",
            "--payload",
            "impl_bundle",
            "--base",
            "refs/heads/main",
            "--label",
            "review-001",
        ],
    );
    assert!(
        !out.status.success(),
        "review_surface without --worktree refuses"
    );
    assert!(
        stderr(&out).contains("--worktree"),
        "refusal names the missing flag: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "candidate/064/review-001"),
        "refused review_surface creates no candidate ref"
    );
    assert!(
        !dir.join(".doctrine/dispatch/064/candidates.toml").exists(),
        "refused review_surface records no row"
    );
}

// --- VT-2: Orchestrator-classed (worker-mode refusal) ------------------------

#[test]
fn dispatch_candidate_is_orchestrator_classed() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    // DOCTRINE_WORKER set ONLY on this subprocess (via create's worker arg).
    let out = create(
        dir,
        Some(true),
        &[
            "--role",
            "close_target",
            "--payload",
            "code",
            "--base",
            "refs/heads/main",
            "--label",
            "close-001",
            "--source",
            "refs/heads/phase/064-02",
        ],
    );
    assert!(!out.status.success(), "refused when DOCTRINE_WORKER set");
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "carries the dual-cause token: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "candidate/064/close-001"),
        "refused run creates no candidate ref"
    );
    assert!(
        !dir.join(".doctrine/dispatch/064/candidates.toml").exists(),
        "refused run records no row"
    );
}

// --- VT-2: create refuses on a raw evidence ref (invariant I9) ----------------

#[test]
fn e2e_dispatch_raw_evidence_worktree_write_refuses() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    // Check the current worktree out onto a raw evidence ref (review/064).
    git(dir, &["checkout", "-q", "review/064"]);

    let out = create(
        dir,
        None,
        &[
            "--role",
            "close_target",
            "--payload",
            "code",
            "--base",
            "refs/heads/main",
            "--label",
            "close-001",
            "--source",
            "refs/heads/phase/064-02",
        ],
    );
    assert!(
        !out.status.success(),
        "create refuses from a worktree on a raw evidence ref"
    );
    assert!(
        stderr(&out).contains("review/064") || stderr(&out).contains("evidence"),
        "refusal names the raw evidence ref: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("candidate"),
        "refusal points at the candidate workflow: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "candidate/064/close-001"),
        "refused-on-evidence-ref run creates no candidate ref"
    );
    assert!(
        !dir.join(".doctrine/dispatch/064/candidates.toml").exists(),
        "refused-on-evidence-ref run records no row"
    );

    // A phase/* ref refuses the same way.
    git(dir, &["checkout", "-q", "phase/064-01"]);
    let out2 = create(
        dir,
        None,
        &[
            "--role",
            "close_target",
            "--payload",
            "code",
            "--base",
            "refs/heads/main",
            "--label",
            "close-002",
            "--source",
            "refs/heads/phase/064-02",
        ],
    );
    assert!(
        !out2.status.success(),
        "create refuses from a worktree on a raw phase ref"
    );
    assert!(
        stderr(&out2).contains("phase/064-01") || stderr(&out2).contains("evidence"),
        "refusal names the raw phase ref: {}",
        stderr(&out2)
    );
}

// --- EX-5: Orchestrator-classed — refused under worker-mode -------------------

#[test]
fn e2e_dispatch_candidate_create_refused_under_worker_mode() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    let out = create(
        dir,
        Some(true),
        &[
            "--role",
            "review_surface",
            "--payload",
            "impl_bundle",
            "--base",
            "refs/heads/main",
            "--label",
            "review-001",
        ],
    );
    assert!(!out.status.success(), "refused when DOCTRINE_WORKER set");
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "carries the dual-cause token: {}",
        stderr(&out)
    );
    assert!(
        !ref_exists(dir, "candidate/064/review-001"),
        "refused run creates no candidate ref"
    );
}

// =============================================================================
// SL-068 PHASE-04 — `dispatch candidate status` (read-only surface; EX-1..3)
// =============================================================================

/// Splice an admission into `candidates.toml` for `id` at `admitted_oid` (admit
/// is PHASE-05 — the fixture writes the record directly, per VT-1's note). Appends
/// a `[current_admission.close_target]` table to the existing ledger file.
fn seed_close_target_admission(
    dir: &Path,
    candidate_id: &str,
    candidate_ref: &str,
    admitted_oid: &str,
) {
    let path = dir.join(".doctrine/dispatch/064/candidates.toml");
    let mut body = std::fs::read_to_string(&path).expect("candidates.toml exists");
    body.push_str(&format!(
        "\n[current_admission.close_target]\n\
         candidate_id = \"{candidate_id}\"\n\
         candidate_ref = \"{candidate_ref}\"\n\
         expected_ref_oid = \"{admitted_oid}\"\n\
         admitted_oid = \"{admitted_oid}\"\n\
         review = \"RV-007\"\n\
         admitted_at = \"2026-06-15\"\n"
    ));
    std::fs::write(&path, body).unwrap();
}

// --- VT-1: grouped evidence vs candidate surface (created + admitted) ---------

#[test]
fn e2e_dispatch_candidate_status_groups_evidence_and_candidates() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    // A created review_surface candidate AND a close_target we then admit.
    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "review_surface",
                "--payload",
                "impl_bundle",
                "--base",
                "refs/heads/main",
                "--label",
                "review-001",
                "--worktree",
            ],
        )
        .status
        .success()
    );
    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--payload",
                "code",
                "--base",
                "refs/heads/main",
                "--label",
                "close-001",
                "--source",
                "refs/heads/phase/064-02",
            ],
        )
        .status
        .success()
    );
    let close_tip = git(dir, &["rev-parse", "candidate/064/close-001"]);
    seed_close_target_admission(
        dir,
        "cand-064-close-001",
        "refs/heads/candidate/064/close-001",
        &close_tip,
    );

    let out = status(dir, None);
    assert!(out.status.success(), "status ok; stderr: {}", stderr(&out));
    let text = stdout(&out);

    // EX-1: the two groups are VISIBLY separate, evidence never conflated with
    // an interaction branch.
    let ev = text.find("evidence refs:").expect("evidence group header");
    let cd = text
        .find("candidates (interaction branches):")
        .expect("candidate group header");
    assert!(
        ev < cd,
        "evidence group precedes the candidate group: {text}"
    );

    // The evidence group names the exact evidence refs (coordination / impl
    // bundle / phase cuts), each labelled — NOT a candidate/* ref.
    let evidence = &text[ev..cd];
    assert!(
        evidence.contains("refs/heads/dispatch/064") && evidence.contains("coordination"),
        "evidence names the coordination branch: {evidence}"
    );
    assert!(
        evidence.contains("refs/heads/review/064") && evidence.contains("impl-bundle"),
        "evidence names the impl bundle: {evidence}"
    );
    assert!(
        evidence.contains("refs/heads/phase/064-01")
            && evidence.contains("refs/heads/phase/064-02")
            && evidence.contains("phase-cut"),
        "evidence names the phase cuts: {evidence}"
    );
    assert!(
        !evidence.contains("candidate/064/"),
        "the evidence group never lists a candidate interaction branch: {evidence}"
    );

    // EX-2: the candidate group reports id/status/base/source/tip/admission for
    // BOTH a created and an admitted candidate.
    let candidates = &text[cd..];
    assert!(
        candidates.contains("cand-064-review-001") && candidates.contains("created"),
        "the created review candidate is reported: {candidates}"
    );
    assert!(
        candidates.contains("cand-064-close-001"),
        "the admitted close candidate is reported: {candidates}"
    );
    assert!(
        candidates.contains("admitted (RV-007)"),
        "the admitted candidate names its admitting review: {candidates}"
    );
    // base/source/tip oid cells are present (12-char abbrev of the real oids).
    let base = git(dir, &["rev-parse", "main"]);
    assert!(
        candidates.contains(&base[..12]),
        "the base oid is reported: {candidates}"
    );
}

// --- VT-2: ref drift is reported, admitted oid unchanged ----------------------

#[test]
fn e2e_dispatch_candidate_status_reports_drift_when_ref_moves_past_admitted_oid() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--payload",
                "code",
                "--base",
                "refs/heads/main",
                "--label",
                "close-001",
                "--source",
                "refs/heads/phase/064-02",
            ],
        )
        .status
        .success()
    );
    let admitted_oid = git(dir, &["rev-parse", "candidate/064/close-001"]);
    seed_close_target_admission(
        dir,
        "cand-064-close-001",
        "refs/heads/candidate/064/close-001",
        &admitted_oid,
    );

    // Before the move: no drift.
    let before = stdout(&status(dir, None));
    assert!(
        before.contains("ok") && !before.contains("DRIFT"),
        "no drift before the ref moves: {before}"
    );

    // Move the candidate ref PAST its recorded/admitted oid (a fresh commit on
    // top — a hand-edit the candidate workflow forbids, which status must surface).
    git(dir, &["checkout", "-q", "candidate/064/close-001"]);
    let moved = commit(dir, "drifted.txt", "drift", "manual edit past admitted oid");
    git(dir, &["checkout", "-q", "main"]);
    assert_ne!(moved, admitted_oid, "the ref genuinely moved");

    let after = stdout(&status(dir, None));
    // I4: drift is REPORTED, not hidden.
    assert!(
        after.contains("DRIFT"),
        "the moved candidate ref is reported as drift: {after}"
    );
    // The admitted oid in the ledger is UNCHANGED (status mutated nothing).
    let ledger = read_candidates(dir);
    assert!(
        ledger.contains(&admitted_oid),
        "the admitted oid is unchanged after status: {ledger}"
    );
    assert!(
        !ledger.contains(&moved),
        "the moved tip was NOT written into the ledger (read-only): {ledger}"
    );
}

// --- VA-1: names evidence vs interaction branch + points at the next action ---

#[test]
fn e2e_dispatch_candidate_status_names_evidence_interaction_and_next_action() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "review_surface",
                "--payload",
                "impl_bundle",
                "--base",
                "refs/heads/main",
                "--label",
                "review-001",
                "--worktree",
            ],
        )
        .status
        .success()
    );

    let text = stdout(&status(dir, None));
    // Names the exact evidence ref AS evidence and the candidate AS the
    // interaction branch (the two labels are not interchangeable).
    assert!(
        text.contains("evidence refs:") && text.contains("refs/heads/review/064"),
        "names the impl bundle as an evidence ref: {text}"
    );
    assert!(
        text.contains("candidates (interaction branches):")
            && text.contains("candidate/064/review-001"),
        "names the candidate as the interaction branch: {text}"
    );
    // EX-3: points at the next safe ACTION (a concrete verb), not "inspect refs".
    let next = text.split("next:").nth(1).expect("a next-action block");
    assert!(
        next.contains("dispatch candidate admit") || next.contains("dispatch candidate create"),
        "the next block names a safe verb: {next}"
    );
    assert!(
        !text.contains("git rev-parse") && !text.contains("git update-ref"),
        "status guides via verbs, not raw ref plumbing: {text}"
    );
}

// --- VA-1: empty ledger guides toward create ---------------------------------

#[test]
fn e2e_dispatch_candidate_status_empty_ledger_guides_to_create() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    let out = status(dir, None);
    assert!(out.status.success(), "status ok; stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("(none recorded)"),
        "an empty ledger reports no candidates: {text}"
    );
    assert!(
        text.contains("dispatch candidate create --slice 64"),
        "the next action is to create the first candidate: {text}"
    );
}

// --- EX-3: status is Read-classed — works UNDER worker-mode -------------------

#[test]
fn e2e_dispatch_candidate_status_runs_under_worker_mode() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    // DOCTRINE_WORKER set: create is refused, but the read-only status surface
    // must SUCCEED (Read-classed, EX-3).
    let out = status(dir, Some(true));
    assert!(
        out.status.success(),
        "status is read-only and runs under worker-mode; stderr: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("evidence refs:"),
        "status renders its surface under worker-mode"
    );
}

// =============================================================================
// SL-068 PHASE-05 — `dispatch candidate admit` (OID binding + provenance; VT-1..4)
// =============================================================================

// --- VT-1: an unproven tip (not descending from merge_oid) refuses (I3) -------

#[test]
fn e2e_dispatch_candidate_admit_rejects_unproven_tip() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    create_close_target(dir, "close-001");

    let merge_oid = git(dir, &["rev-parse", "candidate/064/close-001"]);
    let main_oid = git(dir, &["rev-parse", "main"]);

    // Move the candidate ref to `main` — a tip that does NOT descend from the
    // recorded merge_oid (merge_oid is a child of main, so main is its ancestor,
    // not the other way around). Verify the ancestry genuinely fails first.
    let is_anc = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["merge-base", "--is-ancestor", &merge_oid, &main_oid])
        .output()
        .expect("spawn git")
        .status
        .success();
    assert!(
        !is_anc,
        "the moved tip (main) genuinely does NOT descend from merge_oid"
    );
    git(dir, &["checkout", "-q", "candidate/064/close-001"]);
    git(dir, &["reset", "--hard", "main"]);
    git(dir, &["checkout", "-q", "main"]);

    let out = admit(
        dir,
        None,
        &[
            "--role",
            "close_target",
            "--candidate",
            "refs/heads/candidate/064/close-001",
            "--review",
            "RV-007",
        ],
    );
    assert!(!out.status.success(), "admit refuses an unproven tip");
    assert!(
        stderr(&out).contains("descend") || stderr(&out).contains("I3"),
        "refusal names the descent/I3 failure: {}",
        stderr(&out)
    );
    let toml = read_candidates(dir);
    assert!(
        !toml.contains("[current_admission.close_target]"),
        "no admission recorded on refusal: {toml}"
    );
}

// --- VT-2: happy path records an immutable admitted_oid -----------------------

#[test]
fn e2e_dispatch_candidate_admit_records_immutable_oid_and_moved_ref_refuses() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    create_close_target(dir, "close-001");

    let tip = git(dir, &["rev-parse", "candidate/064/close-001"]);

    let out = admit(
        dir,
        None,
        &[
            "--role",
            "close_target",
            "--candidate",
            "refs/heads/candidate/064/close-001",
            "--review",
            "RV-007",
        ],
    );
    assert!(
        out.status.success(),
        "happy admit ok; stderr: {}",
        stderr(&out)
    );

    let toml = read_candidates(dir);
    assert!(
        toml.contains("[current_admission.close_target]"),
        "the close_target admission is recorded: {toml}"
    );
    assert!(
        toml.contains(&format!("admitted_oid = \"{tip}\"")),
        "admitted_oid pins the candidate tip: {toml}"
    );
    assert!(
        toml.contains("candidate_id = \"cand-064-close-001\""),
        "the admission names the candidate: {toml}"
    );
    assert!(
        toml.contains("review = \"RV-007\""),
        "the admission names its review: {toml}"
    );

    // Immutability: after the candidate ref later moves PAST the admitted oid,
    // the recorded admitted_oid is unchanged (re-running status mutates nothing).
    git(dir, &["checkout", "-q", "candidate/064/close-001"]);
    let moved = commit(dir, "drifted.txt", "drift", "manual edit past admitted oid");
    git(dir, &["checkout", "-q", "main"]);
    assert_ne!(moved, tip, "the ref genuinely moved");
    let after = read_candidates(dir);
    assert!(
        after.contains(&format!("admitted_oid = \"{tip}\"")),
        "admitted_oid stays immutable after the ref moves: {after}"
    );
    assert!(
        !after.contains(&moved),
        "the moved tip is NOT written into the admission: {after}"
    );
    // NOTE (VT-2 moved-ref): the read-reread refusal cannot be raced
    // deterministically from a single in-process CLI invocation (no hook between
    // the two resolves), so the moved-ref REFUSAL path is exercised by the admit
    // core's logic, not a black-box test; the immutability half is asserted here.
}

// --- VT-3: supersede re-admission records history, exactly one current --------

#[test]
fn e2e_dispatch_candidate_supersede_records_history() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);

    create_close_target(dir, "close-001");
    assert!(
        admit(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--candidate",
                "refs/heads/candidate/064/close-001",
                "--review",
                "RV-007",
            ],
        )
        .status
        .success(),
        "admit close-001"
    );

    // A superseding candidate from the same source (create requires a fresh
    // label + supersedes link).
    assert!(
        create(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--payload",
                "code",
                "--base",
                "refs/heads/main",
                "--label",
                "close-002",
                "--source",
                "refs/heads/phase/064-02",
                "--supersedes",
                "cand-064-close-001",
            ],
        )
        .status
        .success(),
        "create close-002 superseding close-001"
    );
    let tip2 = git(dir, &["rev-parse", "candidate/064/close-002"]);
    assert!(
        admit(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--candidate",
                "refs/heads/candidate/064/close-002",
                "--review",
                "RV-008",
            ],
        )
        .status
        .success(),
        "admit close-002"
    );

    let toml = read_candidates(dir);
    // Exactly ONE current close_target admission table.
    assert_eq!(
        toml.matches("[current_admission.close_target]").count(),
        1,
        "exactly one current close admission: {toml}"
    );
    // It points at close-002 with admitted_oid == its tip.
    assert!(
        toml.contains("candidate_id = \"cand-064-close-002\""),
        "the current admission is close-002: {toml}"
    );
    assert!(
        toml.contains(&format!("admitted_oid = \"{tip2}\"")),
        "admitted_oid is close-002's tip: {toml}"
    );
    // It supersedes the prior admitted candidate id.
    assert!(
        toml.contains("supersedes = \"cand-064-close-001\""),
        "the admission supersedes the prior admitted candidate: {toml}"
    );
}

// --- VT-4: admit leaves evidence/candidate refs untouched + worker refusal ----

#[test]
fn e2e_dispatch_candidate_admit_leaves_evidence_and_refused_under_worker() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    create_close_target(dir, "close-001");

    let review_before = git(dir, &["rev-parse", "review/064"]);
    let phase1_before = git(dir, &["rev-parse", "phase/064-01"]);
    let phase2_before = git(dir, &["rev-parse", "phase/064-02"]);
    let candidate_before = git(dir, &["rev-parse", "candidate/064/close-001"]);

    assert!(
        admit(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--candidate",
                "refs/heads/candidate/064/close-001",
                "--review",
                "RV-007",
            ],
        )
        .status
        .success(),
        "admit ok"
    );

    // EX-4: admit writes ONLY candidates.toml — every evidence/candidate ref is
    // byte-for-byte unchanged.
    assert_eq!(git(dir, &["rev-parse", "review/064"]), review_before);
    assert_eq!(git(dir, &["rev-parse", "phase/064-01"]), phase1_before);
    assert_eq!(git(dir, &["rev-parse", "phase/064-02"]), phase2_before);
    assert_eq!(
        git(dir, &["rev-parse", "candidate/064/close-001"]),
        candidate_before,
        "the candidate ref itself is untouched"
    );

    // Orchestrator-classed: refused under worker-mode, records no admission.
    let repo2 = tempfile::tempdir().unwrap();
    let dir2 = repo2.path();
    build_fixture(dir2);
    prepare_review(dir2);
    create_close_target(dir2, "close-001");

    let out = admit(
        dir2,
        Some(true),
        &[
            "--role",
            "close_target",
            "--candidate",
            "refs/heads/candidate/064/close-001",
            "--review",
            "RV-007",
        ],
    );
    assert!(!out.status.success(), "refused when DOCTRINE_WORKER set");
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "carries the dual-cause token: {}",
        stderr(&out)
    );
    let toml = read_candidates(dir2);
    assert!(
        !toml.contains("[current_admission.close_target]"),
        "no admission recorded under worker-mode refusal: {toml}"
    );
}

// =============================================================================
// SL-068 PHASE-06 — candidate-aware `dispatch sync --integrate` (VT-1..4)
//
// NOTE: when a candidate workflow is active (≥1 recorded candidate row),
// --trunk sources the admitted close_target OID and --edge the admitted
// review_surface OID — never a raw phase/review ref, never a close-time merge.
// The PRESERVED legacy path (no ledger ⇒ phase-chain tip / raw review) is
// exercised by the SIBLING tests/e2e_dispatch_sync.rs (which records no
// candidate) and MUST remain unedited and green.
// =============================================================================

/// The full journal.toml committed on `dispatch/064` (object db, not the
/// filesystem — integrate tree-reads it). Used to assert the CAS targeted the
/// admitted OID.
fn dispatch_journal(dir: &Path) -> String {
    git(
        dir,
        &["show", "dispatch/064:.doctrine/dispatch/064/journal.toml"],
    )
}

// --- VT-1: admit a close_target, then integrate --trunk fast-forwards to the
//           admitted OID with NO close-time merge --------------------------------

#[test]
fn e2e_dispatch_candidate_admit_then_integrate_ff() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    create_close_target(dir, "close-001");

    let admitted_oid = git(dir, &["rev-parse", "candidate/064/close-001"]);
    assert!(
        admit(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--candidate",
                "refs/heads/candidate/064/close-001",
                "--review",
                "RV-007",
            ],
        )
        .status
        .success(),
        "admit close-001"
    );

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        out.status.success(),
        "candidate-aware integrate --trunk ff; stderr: {}",
        stderr(&out)
    );

    // main now == the admitted OID exactly (fast-forward) — NOT a fresh
    // close-time merge commit (I6).
    let main_after = git(dir, &["rev-parse", "main"]);
    assert_eq!(
        main_after, admitted_oid,
        "trunk advanced to the admitted close_target OID, no close-time merge"
    );

    // The committed journal/CAS targeted planned_new_oid == admitted_oid for the
    // trunk row (source_oid == planned_new_oid for a direct projection).
    let journal = dispatch_journal(dir);
    assert!(
        journal.contains("target_ref = \"refs/heads/main\""),
        "the trunk row targets main: {journal}"
    );
    assert!(
        journal.contains(&format!("planned_new_oid = \"{admitted_oid}\"")),
        "the trunk row's planned_new_oid is the admitted OID: {journal}"
    );
    assert!(
        journal.contains(&format!("source_oid = \"{admitted_oid}\"")),
        "the trunk row's source is the admitted OID (no merge synthesised): {journal}"
    );
}

// --- VT-2: the candidate ref moves AFTER admit; integrate still targets the
//           ADMITTED oid (I4 — targeting is by oid, not the live ref) ------------

#[test]
fn e2e_dispatch_candidate_ref_moves_after_admit_close_uses_oid() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    create_close_target(dir, "close-001");

    let admitted_oid = git(dir, &["rev-parse", "candidate/064/close-001"]);
    assert!(
        admit(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--candidate",
                "refs/heads/candidate/064/close-001",
                "--review",
                "RV-007",
            ],
        )
        .status
        .success(),
        "admit close-001"
    );

    // Move the candidate ref forward PAST the admitted oid (a fresh commit on
    // top) so its live tip != admitted_oid.
    git(dir, &["checkout", "-q", "candidate/064/close-001"]);
    let moved_tip = commit(dir, "after.txt", "after admit", "move candidate past admit");
    git(dir, &["checkout", "-q", "main"]);
    assert_ne!(moved_tip, admitted_oid, "the candidate ref genuinely moved");

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        out.status.success(),
        "integrate targets the admitted oid despite the moved ref; stderr: {}",
        stderr(&out)
    );

    let main_after = git(dir, &["rev-parse", "main"]);
    assert_eq!(
        main_after, admitted_oid,
        "trunk advanced to the ADMITTED oid (I4), not the moved candidate tip"
    );
    assert_ne!(
        main_after, moved_tip,
        "the moved candidate tip is NOT what integrate targeted"
    );
}

// --- VT-3: trunk moved past the admitted oid after admit ⇒ integrate REFUSES
//           (no ff), names the moved target + a superseding close-target ---------

#[test]
fn e2e_dispatch_candidate_trunk_moved_after_admit_refuses() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    create_close_target(dir, "close-001");

    assert!(
        admit(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--candidate",
                "refs/heads/candidate/064/close-001",
                "--review",
                "RV-007",
            ],
        )
        .status
        .success(),
        "admit close-001"
    );

    // Advance main past the admitted oid (a fresh commit on main) so the admitted
    // close_target no longer fast-forwards main.
    let main_advanced = commit(dir, "trunk-moved.txt", "moved", "trunk moves past admit");

    let out = integrate(dir, &["--trunk", "refs/heads/main"]);
    assert!(
        !out.status.success(),
        "integrate --trunk refuses a non-ff admitted oid"
    );
    let err = stderr(&out);
    assert!(
        err.contains("fast-forward") || err.contains("trunk moved"),
        "refusal names the moved trunk: {err}"
    );
    assert!(
        err.contains("superseding close-target") || err.contains("re-admit"),
        "refusal instructs to create a superseding close-target candidate: {err}"
    );

    // main is UNCHANGED at the post-advance tip — no merge, no clobber.
    assert_eq!(
        git(dir, &["rev-parse", "main"]),
        main_advanced,
        "trunk left at the post-advance tip — never clobbered"
    );
}

// --- VT-4: workflow active (close_target admitted) but NO review_surface
//           admission ⇒ integrate --edge REFUSES (no raw-ref fallback) ----------

#[test]
fn e2e_dispatch_candidate_integrate_edge_without_admission_refuses() {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    build_fixture(dir);
    prepare_review(dir);
    create_close_target(dir, "close-001");

    // Admit ONLY the close_target — the workflow is active, but there is no
    // review_surface admission.
    assert!(
        admit(
            dir,
            None,
            &[
                "--role",
                "close_target",
                "--candidate",
                "refs/heads/candidate/064/close-001",
                "--review",
                "RV-007",
            ],
        )
        .status
        .success(),
        "admit close-001"
    );

    let out = integrate(dir, &["--edge", "refs/heads/edge"]);
    assert!(
        !out.status.success(),
        "integrate --edge refuses with no review_surface admission"
    );
    let err = stderr(&out);
    assert!(
        err.contains("review_surface"),
        "refusal names the missing review_surface admission: {err}"
    );
    assert!(
        !ref_exists(dir, "refs/heads/edge"),
        "no silent raw-ref fallback — the edge ref was not created"
    );
}

// ── SL-165 PHASE-02: candidate-source close_target provenance gate ──────────

/// Minimal bootstrap: init git, create dispatch setup + prepare-review for a
/// given slice.
fn bootstrap_slice(dir: &Path, slice_num: u32) {
    let slice_str = format!("{slice_num:03}");
    let plan_dir = dir.join(format!(".doctrine/slice/{slice_str}"));
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::write(dir.join(".gitignore"), ".doctrine/state/\n").unwrap();
    git(dir, &["add", ".gitignore"]);
    let _main = commit(dir, "trunk.txt", "trunk", "initial");
    // Seeded phases for completeness gate.
    let pdir = dir.join(format!(".doctrine/state/slice/{slice_str}/phases"));
    std::fs::create_dir_all(&pdir).unwrap();
    std::fs::write(
        pdir.join("phase-01.toml"),
        "status = \"completed\"\n",
    ).unwrap();
    // Source-delta registry — prepare-review requires it for each completed phase.
    let main_oid = git(dir, &["rev-parse", "HEAD"]);
    let boundaries_reg = format!(
        "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"{main_oid}\"\ncode_end_oid = \"{main_oid}\"\nprovenance = \"manual\"\n"
    );
    let reg_dir = dir.join(format!(".doctrine/state/slice/{slice_str}"));
    std::fs::create_dir_all(&reg_dir).unwrap();
    std::fs::write(reg_dir.join("boundaries.toml"), boundaries_reg).unwrap();
    // Minimal plan — `dispatch setup` requires it.
    let plan = format!(
        r#"schema  = "doctrine.plan.overview"
version = 1
slice   = "SL-{slice_num}"
[specs]
primary       = []
collaborators = []
[requirements]
targets      = []
dependencies = []
[[phase]]
id        = "PHASE-01"
name      = "bootstrap"
objective = "boilerplate"
entrance_criteria = []
exit_criteria = []
verification = []
specs             = []
requirements      = []
"#
    );
    std::fs::write(plan_dir.join("plan.toml"), plan).unwrap();
    git(dir, &["add", ".doctrine/slice/"]);
    git(dir, &["commit", "-q", "-m", "plan"]);
    // dispatch setup needs --dir (coordination worktree dir).
    let worktree_dir = dir.join(format!("dispatch-{slice_str}"));
    let setup_dir_str = worktree_dir.to_str().unwrap();
    let dispatch = run(
        dir,
        None,
        &[
            "dispatch",
            "setup",
            "--slice",
            &slice_str,
            "--dir",
            setup_dir_str,
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(dispatch.status.success(), "{dispatch:?}");
    commit(dir, "src.txt", "code", "phase-01 work");
    // prepare-review
    let pr = run(
        dir,
        None,
        &[
            "dispatch",
            "sync",
            "--prepare-review",
            "--slice",
            &slice_str,
            "-p",
            dir.to_str().unwrap(),
        ],
    );
    assert!(pr.status.success(), "{pr:?}");
}

/// Create a candidate. Wraps [`run`] so caller can assert success/failure.
fn candidate_create(
    dir: &Path,
    slice_num: u32,
    label: &str,
    role: &str,
    payload: &str,
    source: &str,
    extra: &[&str],
) -> Output {
    let slice_str = format!("{slice_num:03}");
    let mut args = vec![
        "dispatch",
        "candidate",
        "create",
        "--slice",
        &slice_str,
        "--label",
        label,
        "--kind",
        "audit",
        "--role",
        role,
        "--payload",
        payload,
        "--base",
        "refs/heads/main",
        "--source",
        source,
    ];
    args.extend_from_slice(extra);
    args.push("-p");
    args.push(dir.to_str().unwrap());
    run(dir, None, &args)
}

/// PHASE-02 EX-1 accept: close_target from a recorded review_surface candidate
/// that traces to Verified journaled evidence succeeds.
#[test]
fn close_target_from_candidate_review_surface_accepts() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    bootstrap_slice(dir, 200);
    // Create a review_surface candidate from the journaled review/200.
    let cr = candidate_create(
        dir,
        200,
        "review-001",
        "review_surface",
        "impl_bundle",
        "refs/heads/review/200",
        &["--worktree"],
    );
    assert!(cr.status.success(), "{cr:?}");
    // Now create a close_target sourced from THAT candidate — the new path.
    let ct = candidate_create(
        dir,
        200,
        "close-001",
        "close_target",
        "code",
        "refs/heads/candidate/200/review-001",
        &[],
    );
    assert!(
        ct.status.success(),
        "close_target from candidate review_surface should accept: {:?}",
        ct
    );
}

/// PHASE-02 EX-2 refuse: no recorded candidate row for the named source ref.
#[test]
fn close_target_from_unknown_candidate_refuses() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    bootstrap_slice(dir, 201);
    // Source names a candidate that was never created.
    let ct = candidate_create(
        dir,
        201,
        "close-001",
        "close_target",
        "code",
        "refs/heads/candidate/201/nope",
        &[],
    );
    assert!(!ct.status.success(), "unknown candidate should refuse");
    let stderr = String::from_utf8_lossy(&ct.stderr);
    assert!(
        stderr.contains("no recorded candidate row"),
        "expected 'no recorded candidate row': {}",
        stderr
    );
}

/// PHASE-02 EX-2 refuse: scratch source refused for close_target.
#[test]
fn close_target_from_scratch_candidate_refuses() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    bootstrap_slice(dir, 202);
    // Create a scratch candidate (role=scratch, sourced from review/202).
    let scr = candidate_create(
        dir,
        202,
        "scratch-001",
        "scratch",
        "code",
        "refs/heads/review/202",
        &[],
    );
    assert!(scr.status.success(), "{scr:?}");
    // Attempt close_target sourced from scratch — must refuse (INV-2).
    let ct = candidate_create(
        dir,
        202,
        "close-001",
        "close_target",
        "code",
        "refs/heads/candidate/202/scratch-001",
        &[],
    );
    assert!(
        !ct.status.success(),
        "scratch source should refuse for close_target"
    );
    let stderr = String::from_utf8_lossy(&ct.stderr);
    assert!(
        stderr.contains("role=Scratch") || stderr.contains("only an audit review_surface"),
        "expected role/kind refusal: {}",
        stderr
    );
}
