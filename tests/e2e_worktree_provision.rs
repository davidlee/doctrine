//! SL-029 PHASE-01 T8 — end-to-end over the built binary.
//!
//! Drives `doctrine worktree provision` / `check-allowlist` against a real temp
//! git repo with a linked sibling worktree: an allowlisted gitignored file is
//! copied into the fork, a tier file (`**/handover.md`) is withheld with a
//! warning at exit 0 (VT-5), a newline path survives the `-z` enumeration
//! (VT-7), a non-sibling fork is refused (VT-6), a statically-bad allowlist
//! exits nonzero (VT-7), a tier-naming `.worktreeinclude` fails closed copying
//! nothing (VT-8), and an absent `.worktreeinclude` copies nothing at exit 0
//! (VT-9). SL-285 PHASE-03 adds the repair case over an installed source: two
//! `provision` runs converge, and a user file at a link slot is kept (VT-2).

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

/// The derived runtime tier, relative to a fork: what `provision` REGENERATES
/// from the source after the copy (design §3) — never what it copies.
const DERIVED_TIER: &[&str] = &[
    ".doctrine/skills",
    ".doctrine/agents",
    ".doctrine/workflows",
    ".doctrine/memory/shipped",
    ".agents/skills",
    ".claude/skills",
    ".claude/agents",
    ".claude/workflows",
    ".codex",
];

/// The stderr row whose verb is `verb` and whose remainder names `name` — the
/// progress rows pad the verb to a fixed column (`install::row`), so a substring
/// join like `"kept code-review"` never occurs in the output.
fn row_naming<'a>(stderr: &'a str, verb: &str, name: &str) -> Option<&'a str> {
    let needle = format!("  {verb} ");
    stderr
        .lines()
        .find(|line| line.starts_with(&needle) && line.contains(name))
}

/// SL-285 PHASE-03 EX-3 / VT-2: repairing a provisioned fork is idempotent, and
/// a user-owned file standing at a doctrine link slot is kept, not clobbered.
///
/// The source is a real install ([`common::installed_source`]), so the fork
/// inherits a tracked `installed-version` and regeneration runs its legs instead
/// of taking the uninstalled-source skip — a no-op repair would otherwise pass
/// every assertion by silence. Run 1's `linked code-review` row is the positive
/// control that the name is in the default desired set (`install.rs:1169`);
/// `kept` in run 2 is `Notice::Shadows` (a real entry at a doctrine slot is
/// reported and left alone, `install_ownership.rs:197`), and any red there is a
/// real defect — `converge_leg` already forwards the writer (`install.rs:998`).
/// Run 3 re-asserts the same `kept` row, so the convergence digest compares two
/// runs that both demonstrably executed their legs.
#[test]
fn provision_from_installed_source_twice_converges_and_keeps_a_user_file_under_a_link_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let source = common::installed_source(&tmp.path().join("src"));
    let fork = tmp.path().join("fork");
    add_worktree(&source, &fork);

    // Run 1: the derived tier is materialised and linked in the fork.
    let first = provision(&source, &fork);
    assert!(
        first.status.success(),
        "first provision failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_stderr = String::from_utf8_lossy(&first.stderr);
    assert!(
        row_naming(&first_stderr, "linked", "code-review").is_some(),
        "run 1 must link a skill from the default desired set; stderr: {first_stderr}"
    );
    let before_plant = common::digest_domain(&fork, DERIVED_TIER);

    // R9: a plain user-owned FILE at a slot run 1 filled with a symlink.
    let slot = fork.join(".agents/skills/code-review");
    assert!(
        fs::symlink_metadata(&slot)
            .unwrap()
            .file_type()
            .is_symlink(),
        "control: run 1 left a symlink at {}",
        slot.display()
    );
    fs::remove_file(&slot).unwrap();
    fs::write(&slot, "mine\n").unwrap();

    // Run 2: the repair keeps the user's file and reports it.
    let second = provision(&source, &fork);
    assert!(
        second.status.success(),
        "second provision failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        row_naming(&second_stderr, "kept", "code-review").is_some(),
        "run 2 must keep the user file and say so; stderr: {second_stderr}"
    );
    assert_eq!(
        fs::read_to_string(&slot).unwrap(),
        "mine\n",
        "the user's bytes survive the repair"
    );
    assert!(
        !fs::symlink_metadata(&slot)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the slot is still the user's plain file, not a reinstated link"
    );

    // The repair changed that one slot and nothing else in the tier.
    let after_plant = common::digest_domain(&fork, DERIVED_TIER);
    let changed: Vec<PathBuf> = after_plant
        .iter()
        .filter(|(path, digest)| before_plant.get(*path) != Some(*digest))
        .map(|(path, _)| path.clone())
        .collect();
    assert_eq!(
        changed,
        vec![PathBuf::from(".agents/skills/code-review")],
        "only the withheld slot differs from run 1's projection"
    );

    // Run 3: convergence — no further change to the derived tier, and the run
    // proves it ran: the same kept row as run 2 (a converged run is not a silent
    // none — without this control the digest equality passes on a no-op).
    let third = provision(&source, &fork);
    assert!(
        third.status.success(),
        "third provision failed: {}",
        String::from_utf8_lossy(&third.stderr)
    );
    let third_stderr = String::from_utf8_lossy(&third.stderr);
    assert!(
        row_naming(&third_stderr, "kept", "code-review").is_some(),
        "run 3 must keep the user file and say so; stderr: {third_stderr}"
    );
    assert_eq!(
        common::digest_domain(&fork, DERIVED_TIER),
        after_plant,
        "a third run converges: a kept slot is stable, not oscillating"
    );
}
