//! SL-042 PHASE-01 — the REC reconciliation-record kind, black-box over the built
//! binary (design VT-1/VT-2/VT-3).
//!
//! REC is a status-less numbered kind riding the SL-040 review-kind seam: it
//! scaffolds eagerly, scans via `meta::read_id` (no authored `status` field), and
//! registers in `integrity::KINDS`. These goldens prove the end-to-end surface
//! `rec new` / `show` / `list` / `validate` and the `NNN-slug` alias, and that a
//! REC carries NO status axis (so nothing can desync it from a slice's lifecycle —
//! the VT-3 id-stability intent: a status-less kind has no transition to drift).

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

/// A throwaway git repo with a `.doctrine/rec` tree, pinned identity on `main`.
struct RecRepo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

impl RecRepo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        let repo = Self { _dir: dir, path };
        repo.git(&["init", "-b", "main"]);
        repo.git(&["config", "user.name", "Doctrine Test"]);
        repo.git(&["config", "user.email", "test@doctrine.invalid"]);
        std::fs::create_dir_all(repo.path.join(".doctrine/rec")).unwrap();
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
        Command::new(bin())
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

/// VT-2: `rec new` scaffolds the fileset, `show`/`list` render it, `validate` scans
/// the new kind clean, and the `NNN-slug` alias resolves.
#[test]
fn rec_new_show_list_validate_and_alias() {
    let repo = RecRepo::new();

    // new — a freestanding redesign REC (empty deltas, the F7 shape).
    ok(&repo.run(&[
        "rec",
        "new",
        "--move",
        "redesign",
        "--title",
        "escalate REQ-110 drift",
    ]));
    assert!(
        repo.path.join(".doctrine/rec/001/rec-001.toml").is_file(),
        "rec new must scaffold rec-001.toml"
    );

    // NNN-slug alias resolves to the numeric dir (the reused alias machinery).
    let alias = repo.path.join(".doctrine/rec/001-escalate-req-110-drift");
    assert!(
        std::fs::read_link(&alias)
            .map(|t| t == Path::new("001"))
            .unwrap_or(false),
        "the NNN-slug alias must symlink to 001"
    );

    // show resolves by canonical ref and renders the move + a status-less header.
    let shown = ok(&repo.run(&["rec", "show", "REC-001"]));
    assert!(
        shown.contains("REC-001 — escalate REQ-110 drift"),
        "show header: {shown}"
    );
    assert!(
        shown.contains("move=redesign"),
        "show renders the move: {shown}"
    );

    // list renders the row with move + owning columns.
    let listed = ok(&repo.run(&["rec", "list"]));
    assert!(listed.contains("REC-001"), "list row: {listed}");
    assert!(
        listed.contains("redesign"),
        "list renders the move: {listed}"
    );

    // validate scans REC clean with the new KINDS row.
    let validated = ok(&repo.run(&["validate"]));
    assert!(
        validated.contains("REC"),
        "validate scans the REC kind: {validated}"
    );
    assert!(
        validated.contains("corpus clean"),
        "validate clean: {validated}"
    );
}

/// VT-3 intent: a REC is status-LESS — its authored toml carries no `status` field,
/// so there is no lifecycle axis to drift from a slice's. The id resolves stably
/// regardless of any slice transition (a freestanding REC has no slice coupling at
/// all; an owning_slice is an optional outbound edge, never a status dependency).
#[test]
fn rec_is_status_less_so_its_id_is_stable() {
    let repo = RecRepo::new();
    ok(&repo.run(&[
        "rec",
        "new",
        "--move",
        "accept",
        "--title",
        "accept REQ-108",
    ]));

    let toml = std::fs::read_to_string(repo.path.join(".doctrine/rec/001/rec-001.toml")).unwrap();
    assert!(
        !toml.contains("status ="),
        "a REC must carry no authored status field (status-less, D-Q3): {toml}"
    );

    // The id resolves before and after committing the tree — the authored entity is
    // durable, with no status to reconcile against a slice close.
    ok(&repo.run(&["rec", "show", "1"]));
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-m", "land REC-001"]);
    let shown = ok(&repo.run(&["rec", "show", "1"]));
    assert!(shown.contains("REC-001"), "id stable after commit: {shown}");
}

/// A dangling `--owning-slice` is refused BEFORE an id is minted (the forward-edge
/// guard reused from the review seam); no REC dir is created.
#[test]
fn rec_new_refuses_a_dangling_owning_slice() {
    let repo = RecRepo::new();
    let out = repo.run(&["rec", "new", "--move", "accept", "--owning-slice", "SL-999"]);
    assert!(
        !out.status.success(),
        "a dangling owning_slice must be refused"
    );
    assert!(
        !repo.path.join(".doctrine/rec/001").exists(),
        "no id may be minted when the edge dangles"
    );
}
