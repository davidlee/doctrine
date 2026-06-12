//! SL-049 PHASE-02 (ISS-004) — `spec req add` slug escape + shared slug cap.
//!
//! Black-box goldens over the BUILT binary. `run_req_add` previously called
//! `resolve_slug(&title, None)` unconditionally and `derive_slug` was
//! length-unbounded, so a long title overflowed the 255-byte `NNN-slug` symlink
//! name (ENAMETOOLONG). These pin the fix at the CLI seam:
//!   * VT-1 — `--slug <short>` is taken verbatim (the escape).
//!   * VT-2 — a formerly-aborting long title now succeeds, slug bounded, symlink made.
//!   * VT-3 — an over-cap `--slug` errors naming the cap; nothing reserved.
//!   * VT-5 — two long titles sharing a 100-byte prefix land in distinct REQ dirs.
//!
//! Minting is git-anchored, so the temp root is a real `main`-on-init repo. The
//! spawned binary runs with `DOCTRINE_WORKER` REMOVED — a dispatch worker exports
//! it, and it would otherwise refuse the entity mint these tests need.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");
const SLUG_MAX: usize = 100;

/// A throwaway git repo on `main` with pinned identity — enough for the minting
/// path to resolve a trunk tree-ish without an origin.
struct Repo {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl Repo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        let repo = Self { _dir: dir, path };
        repo.git(&["init", "-b", "main"]);
        repo.git(&["config", "user.name", "Doctrine Test"]);
        repo.git(&["config", "user.email", "test@doctrine.invalid"]);
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

    /// Run the built binary against this root with `DOCTRINE_WORKER` removed (a
    /// dispatch worker exports it; minting would otherwise refuse).
    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .args(args)
            .arg("-p")
            .arg(&self.path)
            .env_remove("DOCTRINE_WORKER")
            .output()
            .expect("spawn doctrine")
    }

    fn req_dir(&self) -> PathBuf {
        self.path.join(".doctrine/requirement")
    }
}

fn ok(out: &Output) {
    assert!(
        out.status.success(),
        "verb failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
}

/// Create a host product spec to roster requirements onto.
fn host_spec(repo: &Repo) {
    ok(&repo.run(&["spec", "new", "product", "host spec"]));
}

/// The slug recorded on a reserved requirement (read from its sister TOML).
fn slug_of(repo: &Repo, id: u32) -> String {
    let toml = fs::read_to_string(
        repo.req_dir()
            .join(format!("{id:03}/requirement-{id:03}.toml")),
    )
    .expect("read requirement toml");
    let line = toml
        .lines()
        .find(|l| l.trim_start().starts_with("slug ="))
        .expect("slug key");
    line.split('=')
        .nth(1)
        .unwrap()
        .trim()
        .trim_matches('"')
        .to_string()
}

/// The `NNN-slug` symlink alias beside the numeric requirement dir, if present.
fn symlink_name(repo: &Repo, id: u32) -> Option<String> {
    let prefix = format!("{id:03}-");
    fs::read_dir(repo.req_dir())
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.starts_with(&prefix))
}

// --- VT-1: --slug escape, verbatim --------------------------------------------

#[test]
fn explicit_slug_is_taken_verbatim() {
    let repo = Repo::new();
    host_spec(&repo);
    ok(&repo.run(&[
        "spec",
        "req",
        "add",
        "PRD-001",
        "Some Long Descriptive Title",
        "--kind",
        "functional",
        "--slug",
        "short",
    ]));
    assert_eq!(slug_of(&repo, 1), "short");
    assert_eq!(symlink_name(&repo, 1).as_deref(), Some("001-short"));
}

// --- VT-2: long title no longer aborts; slug bounded; symlink made ------------

#[test]
fn an_overlong_title_succeeds_with_a_bounded_slug() {
    let repo = Repo::new();
    host_spec(&repo);
    // ~50 words → a derived slug far over the 255-byte FS name cap (and over 100).
    let title = "alpha beta gamma delta epsilon zeta eta theta iota kappa ".repeat(5);
    let out = repo.run(&[
        "spec",
        "req",
        "add",
        "PRD-001",
        title.trim(),
        "--kind",
        "functional",
    ]);
    ok(&out); // previously ENAMETOOLONG (OS error 36)

    let slug = slug_of(&repo, 1);
    assert!(
        slug.len() <= SLUG_MAX,
        "slug not bounded: {} bytes",
        slug.len()
    );
    assert!(!slug.is_empty());

    let link = symlink_name(&repo, 1).expect("NNN-slug symlink created");
    assert!(
        link.len() < 255,
        "symlink name overflows FS cap: {}",
        link.len()
    );

    // Files reserved.
    assert!(repo.req_dir().join("001/requirement-001.toml").is_file());
    assert!(repo.req_dir().join("001/requirement-001.md").is_file());
}

// --- VT-3: over-cap --slug errors naming the cap; nothing written -------------

#[test]
fn an_overlong_explicit_slug_errors_and_reserves_nothing() {
    let repo = Repo::new();
    host_spec(&repo);
    let long = "a".repeat(SLUG_MAX + 1);
    let out = repo.run(&[
        "spec",
        "req",
        "add",
        "PRD-001",
        "A Title",
        "--kind",
        "functional",
        "--slug",
        &long,
    ]);
    assert!(!out.status.success(), "over-cap --slug must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("too long"), "stderr: {stderr}");
    assert!(
        stderr.contains(&SLUG_MAX.to_string()),
        "stderr names the cap: {stderr}"
    );
    // No requirement reserved (001 dir absent).
    assert!(!repo.req_dir().join("001").exists());
}

// --- VT-5: collision safety — distinct REQ dirs despite a shared 100-byte slug -

#[test]
fn two_long_titles_sharing_a_prefix_land_in_distinct_dirs() {
    let repo = Repo::new();
    host_spec(&repo);
    // Both titles derive to the SAME truncated slug: the shared base derives to a
    // 97-byte slug, so the divergent suffix word sits past the 100-byte cap and is
    // truncated off — leaving an identical label for two distinct requirements.
    let base = "alpha beta gamma delta epsilon zeta eta theta iota kappa \
                lambda mu nu xi omicron pi rho sigma tau";
    let title_a = format!("{base} aardvark");
    let title_b = format!("{base} zebra");
    ok(&repo.run(&[
        "spec",
        "req",
        "add",
        "PRD-001",
        &title_a,
        "--kind",
        "functional",
    ]));
    ok(&repo.run(&[
        "spec",
        "req",
        "add",
        "PRD-001",
        &title_b,
        "--kind",
        "functional",
    ]));

    // Distinct numeric dirs (001, 002) — NNN identity, slug only labels the link.
    assert!(repo.req_dir().join("001").is_dir());
    assert!(repo.req_dir().join("002").is_dir());
    assert_eq!(
        slug_of(&repo, 1),
        slug_of(&repo, 2),
        "prefix collision expected"
    );
}
