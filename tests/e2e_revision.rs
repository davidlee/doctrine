//! SL-066 PHASE-02 — the REV change-axis kind, black-box over the built binary
//! (design VT-1/VT-3). REV rides the REC eager-materialise seam: it scaffolds
//! `revision-NNN.{toml,md}` + the `NNN-slug` alias, registers in `integrity::KINDS`,
//! and carries the work-lifecycle FSM (proposed→started→done; abandoned from any
//! non-terminal) PLUS the orthogonal `approval` axis.
//!
//! VT-1 — new/show/status round-trip: the FSM advances and abandons, and a `started`
//! REV at `approval=none` accepts the advance (approval orthogonal, lifecycle
//! approval-blind). VT-3 — minting a REV does not trip `outbound_for`'s
//! `debug_assert!(false)` fallthrough (G3) nor a missing-partition mis-class (G1):
//! `inspect REV-1` and `next` both run clean over a corpus containing a REV.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

/// A throwaway git repo with a `.doctrine/revision` tree, pinned identity on `main`.
struct RevRepo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

impl RevRepo {
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

    fn git(&self, args: &[&str]) -> String {
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
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .args(args)
            .arg("-p")
            .arg(&self.path)
            .output()
            .expect("spawn doctrine")
    }
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "verb failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// VT-1: `revision new` scaffolds the fileset, `show` renders it, the FSM advances
/// (proposed→started→done) with `approval` left orthogonal (a started REV at
/// `approval=none` advances fine), and the `NNN-slug` alias resolves.
#[test]
fn revision_new_show_status_round_trip_and_approval_orthogonal() {
    let repo = RevRepo::new();

    ok(&repo.run(&["revision", "new", "revise ADR-006 layering"]));
    assert!(
        repo.path
            .join(".doctrine/revision/001/revision-001.toml")
            .is_file(),
        "revision new must scaffold revision-001.toml"
    );

    // NNN-slug alias resolves to the numeric dir (the reused alias machinery).
    let alias = repo
        .path
        .join(".doctrine/revision/001-revise-adr-006-layering");
    assert!(
        std::fs::read_link(&alias)
            .map(|t| t == Path::new("001"))
            .unwrap_or(false),
        "the NNN-slug alias must symlink to 001"
    );

    // show renders the identity header + the status/approval line, seeded values.
    let shown = ok(&repo.run(&["revision", "show", "REV-001"]));
    assert!(
        shown.contains("REV-001 — revise ADR-006 layering"),
        "show header: {shown}"
    );
    assert!(
        shown.contains("status=proposed") && shown.contains("approval=none"),
        "show renders seeded status + approval: {shown}"
    );

    // FSM advance: proposed → started → done. The REV sits at approval=none
    // throughout — lifecycle transitions are approval-blind (the orthogonality).
    let started = ok(&repo.run(&["revision", "status", "REV-001", "started"]));
    assert!(started.contains("proposed → started"), "advance: {started}");
    let toml = std::fs::read_to_string(repo.path.join(".doctrine/revision/001/revision-001.toml"))
        .unwrap();
    assert!(
        toml.contains("status = \"started\""),
        "status written: {toml}"
    );
    assert!(
        toml.contains("approval = \"none\""),
        "approval untouched by a lifecycle transition (orthogonal): {toml}"
    );

    let done = ok(&repo.run(&["revision", "status", "REV-001", "done"]));
    assert!(done.contains("started → done"), "advance to done: {done}");
}

/// VT-1 (abandon + refusal): a REV abandons from a non-terminal status, and an
/// illegal transition (a skip, or leaving a terminal) is refused.
#[test]
fn revision_status_abandons_and_refuses_illegal() {
    let repo = RevRepo::new();
    ok(&repo.run(&["revision", "new", "retire POL-001"]));

    // abandon from proposed (a non-terminal) is legal.
    ok(&repo.run(&["revision", "status", "REV-001", "abandoned"]));

    // leaving a terminal (abandoned) is refused — no transition out of terminal.
    let leave = repo.run(&["revision", "status", "REV-001", "started"]);
    assert!(
        !leave.status.success(),
        "leaving a terminal status must be refused"
    );

    // a skip (proposed → done) on a fresh REV is refused.
    ok(&repo.run(&["revision", "new", "skip probe"]));
    let skip = repo.run(&["revision", "status", "REV-002", "done"]);
    assert!(!skip.status.success(), "a skip transition must be refused");
    assert!(
        String::from_utf8_lossy(&skip.stderr).contains("illegal"),
        "skip names the illegal move: {}",
        String::from_utf8_lossy(&skip.stderr)
    );
}

/// VT-3: minting a REV does not trip `outbound_for`'s `debug_assert!(false)`
/// fallthrough (G3) — `inspect REV-1` scans the corpus (which now walks the
/// `revision/` dir) clean. In a debug build a missing arm would panic the scan.
#[test]
fn minting_a_rev_does_not_trip_the_corpus_scan_debug_assert() {
    let repo = RevRepo::new();
    ok(&repo.run(&["revision", "new", "revise REQ-201"]));

    // inspect drives the all-kind relation-graph scan (G3 outbound_for over every
    // KINDS row, including the REV row); a missing arm would `debug_assert!(false)`.
    let inspected = repo.run(&["inspect", "REV-001"]);
    assert!(
        inspected.status.success(),
        "inspect REV-001 must scan clean (no outbound_for debug_assert trip): {}",
        String::from_utf8_lossy(&inspected.stderr)
    );

    // validate also walks the REV tree via the new KINDS row.
    let validated = ok(&repo.run(&["validate"]));
    assert!(
        validated.contains("REV"),
        "validate scans the REV kind: {validated}"
    );
    assert!(
        validated.contains("corpus clean"),
        "validate clean over a corpus with a REV: {validated}"
    );
}

/// VT-3 (G1 + G2): `next` drives the priority scan (status_class over every kind +
/// dep_seq_for over every kind). With a REV in the corpus, neither a missing
/// partition row (G1 mis-class) nor a missing dep_seq arm (G2) trips the scan.
#[test]
fn priority_scan_tolerates_a_rev_in_the_corpus() {
    let repo = RevRepo::new();
    ok(&repo.run(&["revision", "new", "spike revision"]));
    let next = repo.run(&["next"]);
    assert!(
        next.status.success(),
        "next must scan clean over a corpus containing a REV (G1/G2): {}",
        String::from_utf8_lossy(&next.stderr)
    );
}
