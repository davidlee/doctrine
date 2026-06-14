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
    Fixture {
        base,
        code_end_1,
        code_end_2,
    }
}

/// Run `doctrine <args>` in `cwd`; `worker = Some(true)` sets DOCTRINE_WORKER=1.
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
    assert_ne!(foreign, fixture.base, "trunk genuinely moved off the fork-point");

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
