//! SL-031 PHASE-01 (deliverable A) — production minting is trunk-aware.
//!
//! Black-box goldens over the built binary proving each of the five minting
//! verbs wires `git::trunk_entity_ids` rather than the old `&[]` placeholder
//! (VT-1 / VT-5). The discriminating fixture: trunk (`main`) carries id `005`
//! for the kind, but the working tree does NOT — so the local scan alone would
//! mint `001`. Only a verb that reads trunk mints `006`. A no-trunk repo proves
//! the local-only degradation still mints `001` (VT-2 / X-5).
//!
//! EX-3 (codex F2): a no-trunk temp repo cannot tell `&[]` from a wired call, so
//! every one of the five verbs is exercised against a TRUNK-RESOLVED repo. One
//! verb green does not prove the other four wired.

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

/// A throwaway git repo with `main` as the initial branch + pinned identity, so
/// `trunk_tree_ish` resolves `main` (no origin, no `DOCTRINE_TRUNK_REF`).
struct TrunkRepo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

impl TrunkRepo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        let repo = Self { _dir: dir, path };
        repo.git(&["init", "-b", "main"]);
        repo.git(&["config", "user.name", "Doctrine Test"]);
        repo.git(&["config", "user.email", "test@doctrine.invalid"]);
        repo
    }

    fn path(&self) -> &Path {
        &self.path
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

    /// Commit `<kind_dir>/<id>/<stem>-<id>.toml` on `main`, then delete the dir
    /// from the working tree. Trunk's tree retains id `id`; the local scan is
    /// blind to it — the discriminating divergence.
    fn seed_trunk_only(&self, kind_dir: &str, stem: &str, id: u32) {
        let rel = format!("{kind_dir}/{id:03}/{stem}-{id:03}.toml");
        let full = self.path.join(&rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, format!("id = {id}\n")).unwrap();
        self.git(&["add", "-A"]);
        self.git(&["commit", "-m", "seed trunk id"]);
        std::fs::remove_dir_all(self.path.join(format!("{kind_dir}/{id:03}"))).unwrap();
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

fn ok(out: &Output) {
    assert!(
        out.status.success(),
        "verb failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
}

/// The four standalone minting verbs: trunk carries `005`, the working tree is
/// blind to it, so a trunk-aware mint lands at `006`.
fn assert_trunk_aware_mint(kind_dir: &str, stem: &str, verb: &[&str]) {
    let repo = TrunkRepo::new();
    repo.seed_trunk_only(kind_dir, stem, 5);
    ok(&repo.run(verb));
    assert!(
        repo.path().join(format!("{kind_dir}/006")).is_dir(),
        "{:?} minted blind to trunk — expected {kind_dir}/006",
        verb
    );
    assert!(
        !repo.path().join(format!("{kind_dir}/001")).exists(),
        "{:?} minted 001 — trunk id 005 was ignored (placeholder still wired)",
        verb
    );
}

#[test]
fn slice_new_mints_above_trunk() {
    assert_trunk_aware_mint(
        ".doctrine/slice",
        "slice",
        &["slice", "new", "fixture slice"],
    );
}

#[test]
fn adr_new_mints_above_trunk() {
    assert_trunk_aware_mint(".doctrine/adr", "adr", &["adr", "new", "fixture adr"]);
}

#[test]
fn spec_new_mints_above_trunk() {
    assert_trunk_aware_mint(
        ".doctrine/spec/product",
        "spec",
        &["spec", "new", "product", "fixture spec"],
    );
}

#[test]
fn backlog_new_mints_above_trunk() {
    assert_trunk_aware_mint(
        ".doctrine/backlog/issue",
        "backlog",
        &["backlog", "new", "issue", "fixture issue"],
    );
}

#[test]
fn spec_req_add_mints_requirement_above_trunk() {
    // The fifth site (requirement::reserve) is spec-mediated: create a product
    // spec in the working tree, then add a requirement. Trunk carries REQ-005;
    // the reserve must land REQ-006.
    let repo = TrunkRepo::new();
    repo.seed_trunk_only(".doctrine/requirement", "requirement", 5);
    ok(&repo.run(&["spec", "new", "product", "host spec"]));
    ok(&repo.run(&[
        "spec",
        "req",
        "add",
        "PRD-001",
        "fixture requirement",
        "--kind",
        "functional",
    ]));
    assert!(
        repo.path().join(".doctrine/requirement/006").is_dir(),
        "requirement reserve minted blind to trunk — expected REQ-006",
    );
    assert!(
        !repo.path().join(".doctrine/requirement/001").exists(),
        "requirement reserve minted REQ-001 — trunk id 005 ignored",
    );
}

#[test]
fn no_trunk_repo_degrades_to_local_only_mint() {
    // VT-2 / X-5: an unborn repo (no commit ⇒ no main peel ⇒ no trunk) must mint
    // the local-only id `001` — the wired call degrades, it does not error.
    let repo = TrunkRepo::new();
    ok(&repo.run(&["slice", "new", "first slice"]));
    assert!(
        repo.path().join(".doctrine/slice/001").is_dir(),
        "no-trunk mint must land at 001",
    );
}
