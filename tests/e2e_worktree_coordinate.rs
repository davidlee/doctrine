// SPDX-License-Identifier: GPL-3.0-only
//! SL-064 PHASE-02 — `doctrine worktree coordinate --slice <n> --dir <path>`
//! end-to-end over the BUILT binary. The markerless coordination-worktree
//! create/resume path (design §2).
//!
//! * VT-1: create — markerless (NO `.doctrine/state/dispatch/worker`), branch
//!   `dispatch/064` at the resolved trunk, worktree registered, env contract on
//!   stdout / human status on stderr, runtime phase sheets regenerated from the
//!   committed `plan.toml`; a post-`add` provision failure rolls back the
//!   worktree AND the freshly minted branch (Create rollback drops the branch).
//! * VT-2: impersonation — a marker-present linked worktree AND a
//!   `DOCTRINE_WORKER=1` process each refuse `coordinate` through the shared
//!   Orchestrator-verb guard, naming the verb / the dual cause.
//! * VT-3: collision — a live worktree already on `dispatch/064` refuses
//!   (`coordination-live`) before mutating refs or dirs (no second branch).
//! * VT-4: resume — branch `dispatch/064` exists with no live worktree ⇒
//!   reattaches the SAME branch (no refuse, no second coordination branch).
//! * VT-5: id-minting (D3) — a mint from inside the linked coordination worktree
//!   resolves the configured trunk ref (off-root), minting ABOVE a trunk-only id
//!   the worktree's own tree is blind to.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

/// The coordination branch for slice 64 — `dispatch/{64:03}`.
const COORD_BRANCH: &str = "dispatch/064";

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

/// Commit a minimal `[[phase]]` `plan.toml` for `slice` on `main`. The
/// coordination verb calls `slice::run_phases`, which `read_plan`s
/// `.doctrine/slice/<slice>/plan.toml` from the freshly-checked-out worktree —
/// so the fixture must be COMMITTED (the worktree is a checkout of trunk).
fn seed_plan(dir: &Path, slice: u32) {
    let rel = format!(".doctrine/slice/{slice:03}/plan.toml");
    let full = dir.join(&rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(
        &full,
        "schema = \"doctrine.plan.overview\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n",
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "seed plan fixture"]);
}

/// Make a real linked worktree of `src` at `<holder>/fork` on a new `branch`.
fn add_worktree(src: &Path, holder: &Path, branch: &str) -> PathBuf {
    let base = git(src, &["rev-parse", "HEAD"]);
    let fork = holder.join("fork");
    git(
        src,
        &[
            "worktree",
            "add",
            "-b",
            branch,
            fork.to_str().unwrap(),
            &base,
        ],
    );
    fork
}

fn stamp_marker(root: &Path) {
    let dir = root.join(".doctrine/state/dispatch");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("worker"), b"").unwrap();
}

fn marker_exists(root: &Path) -> bool {
    root.join(".doctrine/state/dispatch/worker").exists()
}

/// Run `doctrine <args>` in `cwd`; env governed by `worker` (Some(true) sets
/// DOCTRINE_WORKER=1; None removes it).
fn run(cwd: &Path, worker: Option<bool>, args: &[&str]) -> Output {
    let mut cmd = Command::new(BIN);
    cmd.args(args).current_dir(cwd);
    match worker {
        Some(true) => {
            cmd.env("DOCTRINE_WORKER", "1");
        }
        Some(false) | None => {
            cmd.env_remove("DOCTRINE_WORKER");
        }
    }
    cmd.output().expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn branch_exists(src: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(src)
        .args(["rev-parse", "--verify", "--quiet", branch])
        .output()
        .expect("spawn git")
        .status
        .success()
}

fn worktree_registered(src: &Path, dir: &Path) -> bool {
    git(src, &["worktree", "list"]).contains(&dir.to_string_lossy().into_owned())
}

/// Count of local branches under `dispatch/` — the "no second coordination
/// branch" oracle.
fn dispatch_branch_count(src: &Path) -> usize {
    git(src, &["branch", "--list", "dispatch/*"])
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
}

// --- VT-1: create — markerless, at trunk, registered, sheets regenerated ---

#[test]
fn coordinate_create_is_markerless_at_trunk_with_sheets() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    seed_plan(src.path(), 64);
    let trunk = git(src.path(), &["rev-parse", "HEAD"]);
    let holder = tempfile::tempdir().unwrap();
    let coord = holder.path().join("coord");

    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            coord.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "create must succeed; stderr: {}",
        stderr(&out)
    );

    // MARKERLESS: the coordination tree is the orchestrator (worker-mode OFF),
    // so it stamps no worker marker (D2a, never a positive coordination marker).
    assert!(
        !marker_exists(&coord),
        "coordination create stamps NO marker"
    );

    // Branch `dispatch/064` exists, registered, and sits at the resolved trunk.
    assert!(branch_exists(src.path(), COORD_BRANCH), "branch exists");
    assert!(
        worktree_registered(src.path(), &coord),
        "worktree registered"
    );
    assert_eq!(
        git(&coord, &["rev-parse", "HEAD"]),
        trunk,
        "coordination branch forks off the resolved trunk (integration base)"
    );

    // env contract on STDOUT (KEY=value), human status on STDERR.
    assert!(
        stdout(&out).contains("CARGO_TARGET_DIR="),
        "env contract on stdout; got: {}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("wt/dispatch/064"),
        "contract maps target to wt/<branch>; got: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("created") && stderr(&out).contains(COORD_BRANCH),
        "human status names the create; got: {}",
        stderr(&out)
    );

    // Provisioned + sheets REGENERATED from the committed plan.toml (proves the
    // post-provision run_phases ran, not merely a checkout).
    assert!(
        coord
            .join(".doctrine/state/slice/064/phases/phase-01.toml")
            .exists(),
        "runtime phase sheet regenerated from plan.toml"
    );
}

#[test]
fn coordinate_create_rolls_back_branch_on_provision_failure() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    // A `.worktreeinclude` naming a withheld tier makes `run_provision` bail
    // (allowlist_violations fail-closed) — a deterministic failure AFTER the
    // `git worktree add`, forcing the Create compensation (drops the branch too).
    std::fs::write(src.path().join(".worktreeinclude"), ".doctrine/state/**\n").unwrap();
    git(src.path(), &["add", ".worktreeinclude"]);
    git(src.path(), &["commit", "-q", "-m", "bad allowlist"]);

    let holder = tempfile::tempdir().unwrap();
    let coord = holder.path().join("coord");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            coord.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "provision failure must fail the create; stdout: {}",
        stdout(&out)
    );
    // GONE — Create rollback reverses the worktree, the branch, and the dir.
    assert!(
        !worktree_registered(src.path(), &coord),
        "worktree rolled back; list: {}",
        git(src.path(), &["worktree", "list"])
    );
    assert!(
        !branch_exists(src.path(), COORD_BRANCH),
        "Create rollback drops the freshly minted branch"
    );
    assert!(!coord.exists(), "coordination dir reaped");
}

// --- VT-2: impersonation — marker-present + DOCTRINE_WORKER refuse ---

#[test]
fn coordinate_refused_under_worker_mode() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    let holder = tempfile::tempdir().unwrap();
    let linked = add_worktree(src.path(), holder.path(), "wkr-guard");

    // (1) Marked linked worktree, env unset ⇒ refused (signal: marker), names verb.
    stamp_marker(&linked);
    let target = holder.path().join("nope1");
    let out = run(
        &linked,
        None,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            target.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "coordinate refused from a marked linked worktree; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("coordinate"),
        "refusal names the verb; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused coordinate creates nothing");

    // (2) DOCTRINE_WORKER set on the non-linked tree ⇒ dual-cause refusal.
    let target = holder.path().join("nope2");
    let out = run(
        src.path(),
        Some(true),
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            target.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "coordinate refused when DOCTRINE_WORKER set; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("DOCTRINE_WORKER"),
        "env-on-nonlinked carries the dual-cause; stderr: {}",
        stderr(&out)
    );
    assert!(!target.exists(), "refused coordinate creates nothing");
}

// --- VT-3: collision — a live worktree on dispatch/064 refuses before mutating ---

#[test]
fn coordinate_refuses_collision_with_live_worktree() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    seed_plan(src.path(), 64);
    let holder = tempfile::tempdir().unwrap();
    // A live linked worktree already checked out on `dispatch/064`.
    let _live = add_worktree(src.path(), holder.path(), COORD_BRANCH);
    assert_eq!(dispatch_branch_count(src.path()), 1, "one dispatch branch");

    let target = holder.path().join("second");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            target.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "collision must refuse; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("coordination-live"),
        "refusal carries the distinct token; stderr: {}",
        stderr(&out)
    );
    // Refused BEFORE mutating: no second dir, no second branch.
    assert!(!target.exists(), "no second coordination dir created");
    assert_eq!(
        dispatch_branch_count(src.path()),
        1,
        "no second coordination branch minted"
    );
}

// --- VT-4: resume — branch exists, no live worktree ⇒ reattach same branch ---

#[test]
fn coordinate_resumes_existing_branch_without_second_branch() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    seed_plan(src.path(), 64);
    let holder = tempfile::tempdir().unwrap();

    // First create establishes `dispatch/064` + a worktree.
    let first = holder.path().join("first");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            first.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "first create; stderr: {}",
        stderr(&out)
    );
    assert!(branch_exists(src.path(), COORD_BRANCH));

    // Remove the worktree, KEEPING the branch — the handover-resume condition
    // (branch exists, no live worktree). --force: provisioned runtime state is
    // untracked, which a bare remove would refuse.
    git(
        src.path(),
        &["worktree", "remove", "--force", first.to_str().unwrap()],
    );
    assert!(
        branch_exists(src.path(), COORD_BRANCH),
        "branch survives removal"
    );
    assert_eq!(dispatch_branch_count(src.path()), 1, "one dispatch branch");

    // Second coordinate ⇒ Resume: reattaches the SAME branch, never a second.
    let second = holder.path().join("second");
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            second.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "resume must not refuse; stderr: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("resumed"),
        "human status names the resume; stderr: {}",
        stderr(&out)
    );
    assert!(
        worktree_registered(src.path(), &second),
        "resumed worktree registered at the new dir"
    );
    assert_eq!(
        dispatch_branch_count(src.path()),
        1,
        "resume reattaches — no second coordination branch"
    );
}

// --- VT-5: id-minting (D3) resolves trunk from the linked coord worktree ---

#[test]
fn mint_from_coordination_worktree_resolves_trunk_off_root() {
    let src = tempfile::tempdir().unwrap();
    init_repo(src.path());
    seed_plan(src.path(), 64);
    let holder = tempfile::tempdir().unwrap();
    let coord = holder.path().join("coord");

    // Create the coordination worktree (checkout of trunk: sees slice 064).
    let out = run(
        src.path(),
        None,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            coord.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "create; stderr: {}", stderr(&out));

    // Advance trunk with a trunk-ONLY slice id 070 the coord worktree is blind
    // to (committed on `main` AFTER the worktree forked). A trunk-aware mint
    // reads `main`'s tip (max 70) ⇒ 071; a local-only scan of the coord tree
    // (max 64) ⇒ 065. Minting 071 proves trunk resolves off-root.
    let trunk_only = src.path().join(".doctrine/slice/070/slice-070.toml");
    std::fs::create_dir_all(trunk_only.parent().unwrap()).unwrap();
    std::fs::write(&trunk_only, "id = 70\n").unwrap();
    git(src.path(), &["add", "-A"]);
    git(src.path(), &["commit", "-q", "-m", "trunk-only id 070"]);

    let out = run(&coord, None, &["slice", "new", "fixture from coord"]);
    assert!(
        out.status.success(),
        "mint from coord worktree; stderr: {}",
        stderr(&out)
    );
    assert!(
        coord.join(".doctrine/slice/071").is_dir(),
        "minted ABOVE trunk-only 070 — trunk resolved from the linked worktree"
    );
    assert!(
        !coord.join(".doctrine/slice/065").exists(),
        "did NOT mint 065 — the local-only (blind to trunk) id was not used"
    );
}
