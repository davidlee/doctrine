//! SL-029 PHASE-01 T8 — end-to-end over the built binary.
//!
//! Drives `doctrine worktree provision` / `check-allowlist` against a real temp
//! git repo with a linked sibling worktree: an allowlisted gitignored file is
//! copied into the fork, a tier file (`**/handover.md`) is withheld with a
//! warning at exit 0 (VT-5), a newline path survives the `-z` enumeration
//! (VT-7), a non-sibling fork is refused (VT-6), a statically-bad allowlist
//! exits nonzero (VT-7), a tier-naming `.worktreeinclude` fails closed copying
//! nothing (VT-8), and an absent `.worktreeinclude` copies nothing at exit 0
//! (VT-9).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

/// Run `git -C <dir> <args>`, asserting success.
fn git(dir: &Path, args: &[&str]) {
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
}

/// A source repo with a tracked base commit. Returns the canonical source root.
fn init_source(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    // Ignore by extension so each file is enumerated individually (a fully
    // ignored *directory* would collapse in `ls-files`).
    fs::write(dir.join(".gitignore"), "*.txt\n*.md\n.doctrine/\n").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
    fs::canonicalize(dir).unwrap()
}

/// Add a linked sibling worktree at `fork`, branch `feat`.
fn add_worktree(source: &Path, fork: &Path) {
    git(
        source,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feat",
            fork.to_str().unwrap(),
        ],
    );
}

/// `worktree provision <fork> -p <source>` plus any extra `flags` (e.g.
/// `--allow-downgrade`, SL-285 PHASE-02).
fn provision_with(source: &Path, fork: &Path, flags: &[&str]) -> Output {
    common::doctrine_cmd(source)
        .args(["worktree", "provision"])
        .arg(fork)
        .args(flags)
        .arg("-p")
        .arg(source)
        .output()
        .expect("spawn doctrine")
}

fn provision(source: &Path, fork: &Path) -> Output {
    provision_with(source, fork, &[])
}

fn check_allowlist(source: &Path) -> Output {
    common::doctrine_cmd(source)
        .args(["worktree", "check-allowlist", "-p"])
        .arg(source)
        .output()
        .expect("spawn doctrine")
}

#[test]
fn provision_copies_allowlisted_files_and_withholds_the_tier() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));
    let fork = tmp.path().join("fork");
    add_worktree(&source, &fork);

    // Ignored (untracked) candidate files under an allowlisted dir.
    fs::create_dir_all(source.join("artifacts")).unwrap();
    fs::write(source.join("artifacts/keep.txt"), "keep").unwrap();
    fs::write(source.join("artifacts/two\nlines.txt"), "nl").unwrap(); // -z safety (VT-7)
    fs::write(source.join("artifacts/handover.md"), "ho").unwrap(); // **/handover.md → withheld
    fs::write(source.join(".worktreeinclude"), "artifacts/**\n").unwrap();

    let out = provision(&source, &fork);
    assert!(
        out.status.success(),
        "provision failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(
        fs::read_to_string(fork.join("artifacts/keep.txt")).unwrap(),
        "keep"
    );
    assert_eq!(
        fs::read_to_string(fork.join("artifacts/two\nlines.txt")).unwrap(),
        "nl",
        "the newline path survived the -z enumeration"
    );
    assert!(
        !fork.join("artifacts/handover.md").exists(),
        "the tier file was withheld, not copied"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("withheld") && stderr.contains("handover.md"),
        "withheld file is warned: {stderr}"
    );
}

// SL-285 PHASE-02 EX-4 / RV-425 F-6: a repair whose fork carries a stamp newer
// than the running binary is refused by its regeneration, and `--allow-downgrade`
// is the lever that lets the regeneration proceed. The allowlist copy runs FIRST,
// so the refusal guards regenerate's own writes, not every write in the repair;
// nothing under test deletes the fork, so `fork.exists()` would be tautological.
#[test]
fn provision_refuses_a_downgrade_and_keeps_the_fork() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));
    // The lever must ride the COMMIT: the fork below is a checkout of this HEAD,
    // so its own tracked stamp is what the downgrade check reads.
    common::commit_stamp(&source, "999.0.0");
    let fork = tmp.path().join("fork");
    add_worktree(&source, &fork);

    let out = provision(&source, &fork);
    assert!(
        !out.status.success(),
        "a recorded version newer than the binary must refuse; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("999.0.0"),
        "the refusal names the recorded version; stderr: {stderr}"
    );

    let ok = provision_with(&source, &fork, &["--allow-downgrade"]);
    assert!(
        ok.status.success(),
        "--allow-downgrade proceeds: {}",
        String::from_utf8_lossy(&ok.stderr)
    );
}

#[test]
fn provision_refuses_a_non_sibling_fork() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));
    // A *separate* repo, not a worktree of source.
    let other = init_source(&tmp.path().join("other"));
    fs::write(source.join(".worktreeinclude"), "artifacts/**\n").unwrap();

    let out = provision(&source, &other);
    assert!(
        !out.status.success(),
        "a non-sibling fork must be refused: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn provision_fails_closed_on_a_tier_naming_allowlist() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));
    let fork = tmp.path().join("fork");
    add_worktree(&source, &fork);

    fs::create_dir_all(source.join("artifacts")).unwrap();
    fs::write(source.join("artifacts/keep.txt"), "keep").unwrap();
    fs::write(source.join(".worktreeinclude"), ".doctrine/state/*\n").unwrap();

    let out = provision(&source, &fork);
    assert!(
        !out.status.success(),
        "a tier-naming allowlist must fail closed"
    );
    assert!(
        !fork.join("artifacts/keep.txt").exists(),
        "fail-closed copies nothing"
    );
}

#[test]
fn provision_with_absent_allowlist_copies_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));
    let fork = tmp.path().join("fork");
    add_worktree(&source, &fork);

    fs::create_dir_all(source.join("artifacts")).unwrap();
    fs::write(source.join("artifacts/keep.txt"), "keep").unwrap();
    // no .worktreeinclude

    let out = provision(&source, &fork);
    assert!(
        out.status.success(),
        "absent allowlist is tolerated: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !fork.join("artifacts/keep.txt").exists(),
        "empty allowlist copies nothing"
    );
}

#[test]
fn check_allowlist_exit_codes() {
    let tmp = tempfile::tempdir().unwrap();
    let source = init_source(&tmp.path().join("src"));

    // Clean allowlist → exit 0.
    fs::write(source.join(".worktreeinclude"), "artifacts/**\n").unwrap();
    assert!(check_allowlist(&source).status.success(), "clean → 0");

    // Names a withheld tier → nonzero.
    fs::write(source.join(".worktreeinclude"), ".doctrine/state/*\n").unwrap();
    assert!(
        !check_allowlist(&source).status.success(),
        "tier-naming → nonzero"
    );

    // Unsupported syntax (`!`) → nonzero.
    fs::write(source.join(".worktreeinclude"), "!secret\n").unwrap();
    assert!(
        !check_allowlist(&source).status.success(),
        "negation → nonzero"
    );
}
