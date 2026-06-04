//! SL-007 PHASE-06 EX-2 / VT-1 — end-to-end over the built binary.
//!
//! Drives the real `doctrine` executable across the producer surface in a temp
//! git repo: `record` anchors to HEAD, `show` projects the born anchor, `verify`
//! stamps the verification axis, and the post-verify `show` reflects it. This is
//! the one test that exercises the whole record→commit→verify→show→list loop
//! against a clean git tree — including the F8 workflow constraint that the store
//! must be committed before `verify` (a dirty tree is refused).

#![allow(
    clippy::expect_used,
    clippy::tests_outside_test_module,
    reason = "integration test: `expect` is the idiomatic fail-fast, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

/// Run `doctrine <args…>` rooted at `dir`, asserting success; return stdout.
fn doctrine(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(BIN)
        .args(args)
        .arg("-p")
        .arg(dir)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// Run `git <args…>` in `dir` under identity flags (no machine config needed),
/// asserting success.
fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("spawn git");
    assert!(status.success(), "git {args:?} failed");
}

/// Parse the uid out of `Recorded memory mem_<hex>[ (key)]: <path>`.
fn parse_uid(stdout: &str) -> String {
    stdout
        .split_whitespace()
        .find(|tok| tok.starts_with("mem_"))
        .expect("record line names a mem_ uid")
        .trim_end_matches(':')
        .to_owned()
}

#[test]
fn record_commit_verify_show_list_against_the_built_binary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    // A born, clean repo: one committed file, no remote (→ local-root identity).
    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("README.md"), "seed\n").expect("write seed");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed"]);

    // record — the tree is clean, so the memory anchors to HEAD (commit).
    let recorded = doctrine(
        dir,
        &[
            "memory",
            "record",
            "anchored fact",
            "--type",
            "fact",
            "--path-scope",
            "README.md",
        ],
    );
    let uid = parse_uid(&recorded);

    // show — before verify: a commit anchor, on a ref, not yet attested.
    let before = doctrine(dir, &["memory", "show", &uid]);
    assert!(
        before.contains("anchor: commit "),
        "pre-verify anchor is a clean commit: {before}"
    );
    assert!(
        before.contains("verified no"),
        "record never attests: {before}"
    );
    assert!(
        before.contains("repo-id local_root/medium"),
        "no remote → local-root identity: {before}"
    );

    // F8: `record` left the store untracked → the tree is dirty. `verify`
    // refuses a dirty tree, so the store must be committed first.
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "store"]);

    // verify — clean tree at a born HEAD: stamps the verification axis.
    doctrine(dir, &["memory", "verify", &uid]);

    // show — after verify: the same anchor, now attested.
    let after = doctrine(dir, &["memory", "show", &uid]);
    assert!(
        after.contains("verified yes"),
        "post-verify anchor is attested: {after}"
    );

    // list — the recorded memory is present (short uid column).
    let listed = doctrine(dir, &["memory", "list"]);
    assert!(
        listed.contains(&uid[..12]),
        "list surfaces the recorded memory: {listed}"
    );
}

/// SL-008 PHASE-04 — `find` over the built binary: the shared shell freezes the
/// snapshot, resolves per-candidate staleness via `commits_touching`, ranks, and
/// renders rows. A scope-bearing query drops a non-matching memory; the matcher's
/// row carries the full uid, the matched `paths` spec, the risk columns, and a
/// commit-mode `fresh` staleness (verified SHA == frozen target ⇒ Some(0)).
#[test]
fn find_ranks_scope_matches_against_the_built_binary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("README.md"), "seed\n").expect("write seed");
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed"]);

    // A: scoped to README.md; B: scoped elsewhere (the non-match).
    let a = parse_uid(&doctrine(
        dir,
        &[
            "memory",
            "record",
            "readme fact",
            "--type",
            "fact",
            "--path-scope",
            "README.md",
        ],
    ));
    let b = parse_uid(&doctrine(
        dir,
        &[
            "memory",
            "record",
            "other fact",
            "--type",
            "fact",
            "--path-scope",
            "src/other.rs",
        ],
    ));

    // Commit the store, then verify A so it enters commit-staleness mode.
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "store"]);
    doctrine(dir, &["memory", "verify", &a]);

    // find scoped to README.md: A matches, B does not.
    let rows = doctrine(dir, &["memory", "find", "--path-scope", "README.md"]);
    assert!(rows.contains(&a), "A (README scope) is found: {rows}");
    assert!(
        !rows.contains(&b),
        "B (other scope) is dropped by a README query: {rows}"
    );
    // A's row: matched dimension + commit-mode fresh (verified_sha == target).
    assert!(
        rows.contains("paths"),
        "spec column shows the matched dimension: {rows}"
    );
    assert!(
        rows.contains("fresh"),
        "verified SHA == frozen target ⇒ Some(0) fresh: {rows}"
    );
}
