//! SL-282 PHASE-02 VT-1 — install-currency advisories at their sites (DEC-380)
//! over the BUILT binary: unbanded `prompt resolve` (text and `--json` hook
//! envelope), `boot` / `boot --check`, and `status` (structured in `--json`,
//! rendered in text).
//!
//! Each finding row is paired with the same fixture made current (local marker
//! and tracked stamp both at the binary's version), so every site's output is
//! asserted exactly: the current output plus the advisory block, nothing else.
//! The pair also proves the block's absence when current is not vacuous.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic"
)]

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod common;

use common::{doctrine_cmd, doctrine_toml, git, init_repo, linked_fork, write_marker, write_stamp};

/// The running binary's version: what a current install records.
const B: &str = env!("CARGO_PKG_VERSION");
const OLDER: &str = "0.0.1";
const NEWER: &str = "999.0.0";

const NOT_INSTALLED: &str = "doctrine isn't installed in this checkout: run `doctrine install`";

// ── fixture writers ─────────────────────────────────────────────────────────

fn make_current(root: &Path) {
    write_marker(root, B);
    write_stamp(root, B);
}

// ── site runners ────────────────────────────────────────────────────────────

fn run_ok(root: &Path, args: &[&str]) -> String {
    let out = doctrine_cmd(root)
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn resolve(root: &Path) -> String {
    run_ok(root, &["prompt", "resolve", "--role", "worker"])
}

fn status_json(root: &Path) -> Value {
    serde_json::from_str(&run_ok(root, &["status", "--json"])).expect("status JSON")
}

/// What every site printed for one fixture state, plus the snapshot bytes.
struct Sites {
    resolve: String,
    boot: String,
    check: String,
    currency: Value,
    snapshot: Vec<u8>,
}

/// Run every site. `resolve` runs first, so `boot.md` is already written and
/// `boot` reports `Unchanged` in both halves of a pair.
fn observe(root: &Path) -> Sites {
    let resolve = resolve(root);
    let boot = run_ok(root, &["boot"]);
    let check = run_ok(root, &["boot", "--check"]);
    let currency = status_json(root)["boot"]["currency"].clone();
    let snapshot = fs::read(root.join(".doctrine/state/boot.md")).unwrap();
    Sites {
        resolve,
        boot,
        check,
        currency,
        snapshot,
    }
}

/// Assert each site's output with findings is its current output plus `block`
/// (A3 placement), and that the reader left the boot snapshot untouched.
fn assert_block_appended(with: &Sites, current: &Sites, block: &[String]) {
    assert!(!block.is_empty(), "a finding row needs a block");
    let lines: String = block.iter().map(|l| format!("{l}\n")).collect();
    assert_eq!(
        with.resolve,
        format!("{}\n\n{lines}", current.resolve.trim_end())
    );
    assert_eq!(with.boot, format!("{}{lines}", current.boot));
    assert_eq!(with.check, format!("{}{lines}", current.check));
    assert_eq!(current.currency, json!([]));
    assert_eq!(with.snapshot, current.snapshot, "boot.md bytes unchanged");
}

/// Observe `root` in its finding state, make it current, observe again, and
/// assert the block and the structured findings.
fn assert_row(root: &Path, block: &[String], currency: &Value) {
    let with = observe(root);
    assert_eq!(&with.currency, currency);
    make_current(root);
    assert_block_appended(&with, &observe(root), block);
}

/// A doctrine-rooted git repo whose `doctrine.toml` is committed.
fn repo(dir: &Path) -> PathBuf {
    let root = dir.join("repo");
    init_repo(&root);
    root
}

// ── rows: one checkout ──────────────────────────────────────────────────────

#[test]
fn current_checkout_prints_no_advisory_anywhere() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    make_current(&root);
    let sites = observe(&root);
    assert_eq!(sites.currency, json!([]));
    assert_eq!(sites.boot.lines().count(), 1, "{}", sites.boot);
    // `boot --check` prints only its own vocabulary.
    for line in sites.check.lines() {
        assert!(
            ["clean ", "stale ", "unpopulated sections: "]
                .iter()
                .any(|p| line.starts_with(p)),
            "{line}"
        );
    }
    assert!(!sites.resolve.contains(NOT_INSTALLED));
}

#[test]
fn missing_marker_is_checkout_not_installed() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    write_stamp(&root, B);
    assert_row(
        &root,
        &[NOT_INSTALLED.to_string()],
        &json!([{"kind": "checkout_not_installed"}]),
    );
}

#[test]
fn older_marker_is_checkout_older() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    write_stamp(&root, B);
    write_marker(&root, OLDER);
    assert_row(
        &root,
        &[format!(
            "this checkout was installed by doctrine {OLDER} (you have {B}): re-run `doctrine install`"
        )],
        &json!([{"kind": "checkout_older", "checkout": OLDER}]),
    );
}

#[test]
fn newer_marker_is_checkout_newer() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    write_stamp(&root, B);
    write_marker(&root, NEWER);
    assert_row(
        &root,
        &[format!(
            "this checkout was installed by doctrine {NEWER}, newer than yours ({B}): upgrade doctrine or check which one is on PATH"
        )],
        &json!([{"kind": "checkout_newer", "checkout": NEWER}]),
    );
}

#[test]
fn older_stamp_is_repo_older() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    write_stamp(&root, OLDER);
    write_marker(&root, B);
    assert_row(
        &root,
        &[format!(
            "your doctrine {B} is newer than this repo's {OLDER}: run `doctrine install` and commit to move the repo up, or use {OLDER}"
        )],
        &json!([{"kind": "repo_older", "repo": OLDER}]),
    );
}

#[test]
fn newer_stamp_is_repo_newer() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    write_stamp(&root, NEWER);
    write_marker(&root, B);
    assert_row(
        &root,
        &[format!(
            "this repo is on doctrine {NEWER} and yours is {B}: upgrade doctrine"
        )],
        &json!([{"kind": "repo_newer", "repo": NEWER}]),
    );
}

#[test]
fn missing_stamp_is_repo_unstamped() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    fs::write(doctrine_toml(&root), "[install]\n").unwrap();
    write_marker(&root, B);
    assert_row(
        &root,
        &[
            "this repo has no recorded doctrine version: run `doctrine install` and commit"
                .to_string(),
        ],
        &json!([{"kind": "repo_unstamped"}]),
    );
}

#[test]
fn unparsable_stamp_is_unreadable_naming_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = repo(dir.path());
    fs::write(
        doctrine_toml(&root),
        "[install]\ninstalled-version = \"garbage\"\n",
    )
    .unwrap();
    write_marker(&root, B);

    // The reason is the semver parser's own text; read it back from the JSON
    // and pin everything else.
    let found = status_json(&root)["boot"]["currency"].clone();
    assert_eq!(found[0]["kind"], "unreadable", "{found}");
    let path = found[0]["path"].as_str().unwrap();
    assert_eq!(
        fs::canonicalize(path).unwrap(),
        fs::canonicalize(doctrine_toml(&root)).unwrap()
    );
    let reason = found[0]["reason"].as_str().unwrap();
    assert!(!reason.is_empty());
    assert_eq!(found.as_array().unwrap().len(), 1, "{found}");

    assert_row(
        &root,
        &[format!(
            "cannot read {path} ({reason}): fix or remove it, then run `doctrine install`"
        )],
        &found,
    );
}

// ── once each: hook envelope, banded resolve, status text ───────────────────

/// A not-installed checkout: tracked stamp current, no local marker.
fn not_installed_repo(dir: &Path) -> PathBuf {
    let root = repo(dir);
    write_stamp(&root, B);
    root
}

#[test]
fn resolve_json_hook_envelope_carries_the_block() {
    let dir = tempfile::tempdir().unwrap();
    let root = not_installed_repo(dir.path());
    let raw = run_ok(&root, &["prompt", "resolve", "--role", "worker", "--json"]);
    let parsed: Value = serde_json::from_str(raw.trim_end()).unwrap();
    let ctx = parsed["additional_context"].as_str().unwrap();
    assert!(ctx.ends_with(&format!("\n\n{NOT_INSTALLED}")), "{ctx}");
    assert_eq!(ctx.trim_end(), resolve(&root).trim_end());
}

#[test]
fn banded_resolve_never_carries_the_block() {
    let dir = tempfile::tempdir().unwrap();
    let root = not_installed_repo(dir.path());
    // Reachable: the unbanded resolve of this fixture carries it.
    assert!(resolve(&root).contains(NOT_INSTALLED));
    let banded = run_ok(
        &root,
        &["prompt", "resolve", "--role", "worker", "--band", "role"],
    );
    assert!(!banded.contains(NOT_INSTALLED), "{banded}");
}

#[test]
fn status_text_renders_the_line_under_the_boot_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = not_installed_repo(dir.path());
    // Active work, so the dashboard renders its Boot block.
    run_ok(&root, &["backlog", "new", "issue", "something"]);
    let text = run_ok(&root, &["status"]);
    let boot = text.split("\nBoot\n").nth(1).expect("Boot block");
    let mut lines = boot.lines();
    assert!(lines.next().unwrap().starts_with("  boot.md "), "{text}");
    assert_eq!(lines.next().unwrap(), format!("  {NOT_INSTALLED}"));
}

/// An idle repo (no active work) still carries the advisory under its one line.
#[test]
fn idle_status_text_carries_the_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = not_installed_repo(dir.path());
    let text = run_ok(&root, &["status"]);
    assert_eq!(
        text.trim_end(),
        format!("No active work.\n\n{NOT_INSTALLED}")
    );
}

// ── linked worktrees (RV-421 F-1) ───────────────────────────────────────────

/// A primary repo with a committed, current tracked stamp (so a fork checks out
/// a `.doctrine/` and a current stamp), and a linked fork of it. The primary's
/// local marker is left to the caller: it is never committed.
fn primary_and_fork(dir: &Path) -> (PathBuf, PathBuf) {
    let primary = repo(dir);
    write_stamp(&primary, B);
    git(&primary, &["add", ".doctrine/doctrine.toml"]);
    git(&primary, &["commit", "-q", "-m", "stamp"]);
    let fork = dir.join("fork");
    linked_fork(&primary, &fork, "fork");
    (primary, fork)
}

/// L1 (the control): a linked worktree never receives `.doctrine/state/`, so it
/// reads its main worktree's marker. The rejected candidate — a reader that
/// looks only at the fork's own marker and reports
/// `Currency::CheckoutNotInstalled` for a marker-less fork — fails this row.
#[test]
fn linked_fork_of_an_installed_primary_reports_current() {
    let dir = tempfile::tempdir().unwrap();
    let (primary, fork) = primary_and_fork(dir.path());
    write_marker(&primary, B);
    assert!(!fork.join(".doctrine/state/install.toml").exists());
    assert_eq!(status_json(&fork)["boot"]["currency"], json!([]));
    assert!(!resolve(&fork).contains(NOT_INSTALLED));
}

/// L2: the positive control for L1 — the same shape with an uninstalled primary
/// does print the block, naming the primary, so L1's absence is reachable.
#[test]
fn linked_fork_of_an_uninstalled_primary_names_the_primary() {
    let dir = tempfile::tempdir().unwrap();
    let (primary, fork) = primary_and_fork(dir.path());
    let main = fs::canonicalize(&primary).unwrap();
    let with = observe(&fork);
    assert_eq!(with.currency, json!([{"kind": "checkout_not_installed"}]));
    write_marker(&primary, B);
    assert_block_appended(
        &with,
        &observe(&fork),
        &[format!("{NOT_INSTALLED} in {}", main.display())],
    );
}

/// L3: a fork with its own marker reads that, and names no other checkout.
#[test]
fn linked_fork_with_its_own_marker_reads_it() {
    let dir = tempfile::tempdir().unwrap();
    let (_primary, fork) = primary_and_fork(dir.path());
    write_marker(&fork, OLDER);
    assert_row(
        &fork,
        &[format!(
            "this checkout was installed by doctrine {OLDER} (you have {B}): re-run `doctrine install`"
        )],
        &json!([{"kind": "checkout_older", "checkout": OLDER}]),
    );
}

/// L4: under `git init --separate-git-dir` the main worktree cannot be located
/// from a linked worktree, and the line says so (STD-003).
#[test]
fn linked_fork_with_an_unlocatable_main_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let primary = dir.path().join("wt");
    let git_dir = dir.path().join("gd");
    fs::create_dir_all(&primary).unwrap();
    git(
        &primary,
        &[
            "init",
            "-q",
            "-b",
            "main",
            "--separate-git-dir",
            git_dir.to_str().unwrap(),
        ],
    );
    git(&primary, &["config", "user.email", "t@example.com"]);
    git(&primary, &["config", "user.name", "Test"]);
    fs::create_dir_all(primary.join(".doctrine")).unwrap();
    write_stamp(&primary, B);
    git(&primary, &["add", "."]);
    git(&primary, &["commit", "-q", "-m", "base"]);
    write_marker(&primary, B);
    let fork = dir.path().join("fork");
    linked_fork(&primary, &fork, "fork");

    let with = observe(&fork);
    assert_eq!(with.currency, json!([{"kind": "checkout_not_installed"}]));
    // Current again only through the fork's own marker.
    write_marker(&fork, B);
    assert_block_appended(
        &with,
        &observe(&fork),
        &[format!("{NOT_INSTALLED} (main worktree not located)")],
    );
}
