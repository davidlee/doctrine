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
