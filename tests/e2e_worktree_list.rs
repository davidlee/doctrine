// SPDX-License-Identifier: GPL-3.0-only
//! SL-190 PHASE-05 — `doctrine worktree list` end-to-end over the BUILT binary
//! (VT-2). Inventory + provenance: enumerate every linked worktree, classify each
//! (primary / coordination / worker-fork / benign), and render the
//! `path·role·slice·branch·head·marker·live?·landed` table (or `--json`), with a
//! role-conditional, fail-soft `landed` column.
//!
//! Fixture: a primary tree + two live coordination trees (`dispatch/190`,
//! `dispatch/191`) + a worker fork (`dispatch/agent-x`, nested under an `SL-190`
//! path) that is landed back into its coordination ref. Asserts: the table renders
//! with all four roles; `--slice` filters to one slice; `--json` is a valid array of
//! row objects; the `landed` column is present by default and suppressed by
//! `--no-landed`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

fn bin() -> PathBuf {
    common::doctrine_bin()
}

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

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// Add a linked worktree on a fresh `branch` off `src` HEAD at `path`.
fn add_worktree(src: &Path, branch: &str, path: &Path) {
    let base = git(src, &["rev-parse", "HEAD"]);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    git(
        src,
        &["worktree", "add", "-b", branch, path.to_str().unwrap(), &base],
    );
}

/// Commit `body` into `wt` on its checked-out branch.
fn commit_in(wt: &Path, rel: &str, body: &str) {
    std::fs::write(wt.join(rel), body).unwrap();
    git(wt, &["add", "-f", rel]);
    git(wt, &["commit", "-q", "-m", &format!("edit {rel}")]);
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(bin());
    cmd.args(args).current_dir(cwd);
    cmd.env_remove("CARGO_TARGET_DIR");
    cmd.env_remove("DOCTRINE_WORKER");
    cmd.output().expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// The full fixture: primary + two coord trees + a landed worker fork.
/// Returns `(src_tempdir, holder_tempdir, worker_path)`.
fn fixture() -> (tempfile::TempDir, tempfile::TempDir, PathBuf) {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();

    // Two coordination trees for distinct slices.
    add_worktree(src.path(), "dispatch/190", &holder.path().join("coord190"));
    add_worktree(src.path(), "dispatch/191", &holder.path().join("coord191"));

    // A worker fork nested under an `SL-190` path (so its slice resolves off the
    // path, its role off the `dispatch/agent-*` branch).
    let worker = holder.path().join("SL-190").join(".worktrees").join("agent-x");
    add_worktree(src.path(), "dispatch/agent-x", &worker);
    commit_in(&worker, "w.rs", "fn w() {}");

    // Land the worker back into its coordination ref (dispatch/190) via --no-ff, so
    // the worker-fork row's landed verdict is provable (ancestry leg).
    git(
        &holder.path().join("coord190"),
        &["merge", "--no-ff", "--no-edit", "dispatch/agent-x"],
    );

    (src, holder, worker)
}

// --- VT-2: the table renders with every role -------------------------------

#[test]
fn worktree_list_renders_table_with_roles_and_landed_column() {
    let (src, _holder, _worker) = fixture();

    let out = run(src.path(), &["worktree", "list"]);
    assert!(
        out.status.success(),
        "worktree list must succeed; stderr: {}",
        stderr(&out)
    );
    let table = stdout(&out);

    // Header carries the landed column.
    assert!(table.contains("landed"), "landed column header; got:\n{table}");
    assert!(table.contains("role"), "role header; got:\n{table}");

    // Every role is represented.
    assert!(table.contains("primary"), "primary row; got:\n{table}");
    assert!(
        table.contains("coordination"),
        "coordination row; got:\n{table}"
    );
    assert!(
        table.contains("worker-fork"),
        "worker-fork row; got:\n{table}"
    );

    // The landed worker fork reads `landed`.
    assert!(
        table.contains("landed"),
        "the landed verdict is rendered; got:\n{table}"
    );
}

// --- VT-2: --slice filters -------------------------------------------------

#[test]
fn worktree_list_slice_filter_scopes_to_one_slice() {
    let (src, _holder, _worker) = fixture();

    let out = run(src.path(), &["worktree", "list", "--slice", "190"]);
    assert!(out.status.success(), "list --slice must succeed");
    let table = stdout(&out);

    // Slice 190's coordination + worker fork are present; slice 191's coord is not.
    assert!(
        table.contains("dispatch/190"),
        "slice 190 coordination present; got:\n{table}"
    );
    assert!(
        table.contains("dispatch/agent-x"),
        "slice 190 worker fork present; got:\n{table}"
    );
    assert!(
        !table.contains("dispatch/191"),
        "slice 191 filtered out; got:\n{table}"
    );
}

// --- VT-2: --json shape ----------------------------------------------------

#[test]
fn worktree_list_json_is_a_valid_array_of_rows() {
    let (src, _holder, _worker) = fixture();

    let out = run(src.path(), &["worktree", "list", "--json"]);
    assert!(out.status.success(), "list --json must succeed");

    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    let rows = value.as_array().expect("JSON is an array");
    assert!(!rows.is_empty(), "at least the primary + linked rows");

    // Every row object carries the provenance keys, including landed.
    for row in rows {
        let obj = row.as_object().expect("row is an object");
        for key in ["path", "role", "slice", "branch", "head", "marker", "live", "landed"] {
            assert!(obj.contains_key(key), "row has `{key}`; row: {row}");
        }
    }

    // The worker-fork row's landed verdict is `landed`.
    let worker = rows
        .iter()
        .find(|r| r["role"] == "worker-fork")
        .expect("a worker-fork row");
    assert_eq!(worker["landed"], "landed", "worker fork landed; row: {worker}");
    assert_eq!(worker["slice"], 190, "worker fork slice from path; row: {worker}");
}

// --- VT-2: --no-landed suppresses the column -------------------------------

#[test]
fn worktree_list_no_landed_suppresses_the_column() {
    let (src, _holder, _worker) = fixture();

    // JSON is the crisp check: the `landed` key is absent per row.
    let out = run(src.path(), &["worktree", "list", "--json", "--no-landed"]);
    assert!(out.status.success(), "list --json --no-landed must succeed");
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    for row in value.as_array().expect("array") {
        let obj = row.as_object().expect("object");
        assert!(
            !obj.contains_key("landed"),
            "--no-landed drops the landed key; row: {row}"
        );
        assert!(obj.contains_key("role"), "other columns remain; row: {row}");
    }
}
