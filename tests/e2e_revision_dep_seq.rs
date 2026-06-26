// SPDX-License-Identifier: GPL-3.0-only
//! SL-066 PHASE-04 — REV wired into the dep/seq overlay end-to-end (the IDE-010
//! payoff), as BLACK-BOX goldens over the built binary (design §PHASE-04 / plan
//! EX-1..3 / VT-1..3). REV is admitted to the work-like predicate as BOTH a dep/seq
//! source and target; governance docs stay EXCLUDED (the SL-060 invariant).
//!
//! - VT-1: `slice needs REV-NNN` is accepted, and the dependent is BLOCKED while
//!   REV-NNN is non-terminal, then UNBLOCKS once REV-NNN reaches a terminal status
//!   (`done`) — the G1 partition payoff realised end-to-end.
//! - VT-2 (G2): a REV-as-source `needs`/`after` edge reaches the blocker/next view —
//!   `revision needs SL-NNN` is accepted and surfaces in `blockers REV-NNN` (the
//!   `dep_seq_for` REV arm is read back, not silently dropped).
//! - VT-3: SL-060 regression — `needs ADR-X` / `needs SPEC-X` are still REFUSED
//!   (governance excluded as a dep/seq target — depending on governance routes
//!   THROUGH a Revision, never the evergreen doc).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

/// A throwaway git repo with a pinned identity and the `.doctrine/revision` tree
/// pre-materialised (REV scaffolds eagerly into an existing dir). `revision status`
/// captures a born-frame from git, so the repo is a real git repo (mirrors the
/// RevRepo harness in `e2e_revision.rs` — no parallel test scaffold).
struct Repo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

impl Repo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        let repo = Self { _dir: dir, path };
        repo.git(&["init", "-b", "main"]);
        repo.git(&["config", "user.name", "Doctrine Test"]);
        repo.git(&["config", "user.email", "test@doctrine.invalid"]);
        std::fs::create_dir_all(repo.path.join(".doctrine/revision")).unwrap();
        repo
    }

    fn git(&self, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    /// Run the binary against the temp corpus. DOCTRINE_WORKER is explicitly UNSET —
    /// the self-arm guard refuses authored writes under it, and a stray inherited var
    /// would spuriously red an authored round-trip
    /// (mem.pattern.dispatch.worker-verify-unset).
    fn run(&self, args: &[&str]) -> Output {
        Command::new(bin())
            .args(args)
            .arg("-p")
            .arg(&self.path)
            .env_remove("DOCTRINE_WORKER")
            .output()
            .expect("spawn doctrine")
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn new_slice(repo: &Repo, title: &str, slug: &str) {
    let out = repo.run(&["slice", "new", title, "--slug", slug]);
    assert!(out.status.success(), "slice new {slug}: {}", stderr(&out));
}

/// Mint a REV via `revision new` (scaffolds REV-NNN into `.doctrine/revision`).
fn new_revision(repo: &Repo, title: &str) {
    let out = repo.run(&["revision", "new", title]);
    assert!(out.status.success(), "revision new: {}", stderr(&out));
}

/// Advance a REV's lifecycle FSM (proposed→started→done; abandoned).
fn rev_status(repo: &Repo, id: &str, state: &str) {
    let out = repo.run(&["revision", "status", id, state]);
    assert!(
        out.status.success(),
        "revision status {id} {state}: {}",
        stderr(&out)
    );
}

/// The id set surfaced by `next --json`.
fn next_ids(repo: &Repo) -> Vec<String> {
    let next = repo.run(&["next", "--json"]);
    assert!(next.status.success(), "next: {}", stderr(&next));
    let nv: serde_json::Value = serde_json::from_str(&stdout(&next)).expect("valid JSON");
    nv["rows"]
        .as_array()
        .expect("rows array")
        .iter()
        .map(|e| e["id"].as_str().expect("id str").to_owned())
        .collect()
}

// --- VT-1: `slice needs REV-NNN` blocks until the REV is terminal, then unblocks ---

#[test]
fn slice_needs_revision_blocks_until_terminal_then_unblocks() {
    let repo = Repo::new();
    new_slice(&repo, "Alpha", "alpha"); // SL-001 (the dependent)
    new_revision(&repo, "revise ADR-006 layering"); // REV-001 (the prerequisite)

    // EX-1 / VT-1: a slice may `needs` a REV — REV is admitted as a dep/seq TARGET.
    let needs = repo.run(&["needs", "SL-001", "REV-001"]);
    assert!(
        needs.status.success(),
        "slice needs REV accepted: {}",
        stderr(&needs)
    );

    // blockers SL-001 surfaces REV-001 as the direct cross-kind blocker.
    let blk = repo.run(&["blockers", "SL-001", "--json"]);
    assert!(blk.status.success(), "blockers SL-001: {}", stderr(&blk));
    let v: serde_json::Value = serde_json::from_str(&stdout(&blk)).expect("valid JSON");
    assert_eq!(
        v["blocked_by"],
        serde_json::json!(["REV-001"]),
        "REV-001 is the direct blocked-by prerequisite of SL-001"
    );

    // EX-2 / VT-1: while REV-001 is non-terminal (proposed), SL-001 is BLOCKED — absent
    // from next; the prerequisite REV-001 (workable) is present.
    let ids = next_ids(&repo);
    assert!(
        !ids.contains(&"SL-001".to_owned()),
        "SL-001 BLOCKED by non-terminal REV-001 → absent from next: {ids:?}"
    );
    assert!(
        ids.contains(&"REV-001".to_owned()),
        "REV-001 (workable) is in next: {ids:?}"
    );

    // Drive REV-001 to a terminal status: proposed → started → done.
    rev_status(&repo, "REV-001", "started");
    rev_status(&repo, "REV-001", "done");

    // EX-2 / VT-1: the G1 partition payoff — a `done` REV classifies Terminal, so the
    // dependent UNBLOCKS. SL-001 now appears in next; the terminal REV-001 drops out.
    let ids = next_ids(&repo);
    assert!(
        ids.contains(&"SL-001".to_owned()),
        "SL-001 UNBLOCKED once REV-001 reached terminal → present in next: {ids:?}"
    );
    assert!(
        !ids.contains(&"REV-001".to_owned()),
        "terminal REV-001 is not actionable → absent from next: {ids:?}"
    );
}

// --- VT-2 (G2): a REV-as-SOURCE needs/after edge reaches the blocker/next view -----

#[test]
fn revision_as_source_needs_after_reaches_blocker_and_next_views() {
    let repo = Repo::new();
    new_slice(&repo, "Spike", "spike"); // SL-001 (the prerequisite work item)
    new_slice(&repo, "Follow", "follow"); // SL-002 (a soft-sequence predecessor)
    new_revision(&repo, "revise ADR-006 layering"); // REV-001 (the dep/seq SOURCE)

    // EX-1 / VT-1: a REV may itself `needs`/`after` a work item — REV as dep/seq SOURCE.
    let needs = repo.run(&["needs", "REV-001", "SL-001"]);
    assert!(
        needs.status.success(),
        "revision needs SL accepted: {}",
        stderr(&needs)
    );
    let after = repo.run(&["after", "REV-001", "SL-002", "--rank", "4"]);
    assert!(
        after.status.success(),
        "revision after SL accepted: {}",
        stderr(&after)
    );

    // VT-2 / G2: the `dep_seq_for` REV arm is READ BACK — REV-001's needs edge surfaces
    // SL-001 as its direct blocker (not silently dropped).
    let blk = repo.run(&["blockers", "REV-001", "--json"]);
    assert!(blk.status.success(), "blockers REV-001: {}", stderr(&blk));
    let v: serde_json::Value = serde_json::from_str(&stdout(&blk)).expect("valid JSON");
    assert_eq!(
        v["blocked_by"],
        serde_json::json!(["SL-001"]),
        "SL-001 is REV-001's direct blocked-by prerequisite (dep_seq_for REV arm read back)"
    );

    // Reciprocally, SL-001 is blocking REV-001.
    let blk2 = repo.run(&["blockers", "SL-001", "--json"]);
    let v2: serde_json::Value = serde_json::from_str(&stdout(&blk2)).expect("valid JSON");
    assert_eq!(
        v2["blocking"],
        serde_json::json!(["REV-001"]),
        "SL-001 blocks REV-001"
    );

    // next: REV-001 is held behind its `needs` prerequisite SL-001 (absent, blocked),
    // and its `after` predecessor SL-002 is present.
    let ids = next_ids(&repo);
    assert!(
        !ids.contains(&"REV-001".to_owned()),
        "REV-001 BLOCKED by its needs SL-001 → absent from next: {ids:?}"
    );
    assert!(
        ids.contains(&"SL-001".to_owned()) && ids.contains(&"SL-002".to_owned()),
        "both work prerequisites are actionable in next: {ids:?}"
    );
}

// --- VT-3: SL-060 regression — governance docs still refused as dep/seq targets ----

#[test]
fn needs_governance_doc_still_refused_after_revision_widen() {
    let repo = Repo::new();
    new_slice(&repo, "Alpha", "alpha"); // SL-001
    // A real ADR — a RESOLVABLE governance target (so the refusal is the work-like KIND
    // gate, not a dangling-ref miss).
    let adr = repo.run(&["adr", "new", "Layering", "--slug", "layering"]);
    assert!(adr.status.success(), "adr new: {}", stderr(&adr));

    // EX-3 / VT-3: `needs ADR-001` is REFUSED — governance is excluded as a dep/seq
    // target even after REV joined the work-like set (the SL-060 invariant: depending on
    // governance routes THROUGH a Revision, never the evergreen doc directly).
    let bad = repo.run(&["needs", "SL-001", "ADR-001"]);
    assert!(
        !bad.status.success(),
        "needs ADR-X refused (governance excluded as dep/seq target)"
    );
    assert!(
        stderr(&bad).contains("may only target work"),
        "names the work-only gate for ADR: {}",
        stderr(&bad)
    );

    // And `after` a governance doc is refused on the same gate.
    let bad_after = repo.run(&["after", "SL-001", "ADR-001"]);
    assert!(
        !bad_after.status.success(),
        "after ADR-X refused (governance excluded as dep/seq target)"
    );
    assert!(
        stderr(&bad_after).contains("may only target work"),
        "names the work-only gate for ADR (after): {}",
        stderr(&bad_after)
    );
}
