// SPDX-License-Identifier: GPL-3.0-only
//! SL-064 PHASE-07 — end-to-end lifecycle proof (VT-1; EX-1 / EX-2).
//!
//! Threads the WHOLE dispatch coordination lifecycle from one fixture and
//! asserts the seam-level properties the per-stage suites
//! (`e2e_worktree_coordinate`, `e2e_dispatch_sync`) verify only in isolation:
//!
//! ```text
//!   worktree coordinate            create the markerless coordination worktree
//!     -> commit phase code + record-boundary    ON the coordination tree
//!     -> dispatch sync --prepare-review          review/064 + phase/064-NN refs
//!     -> [INVARIANT] session `main` tree + trunk ref UNTOUCHED across the run
//!     -> dispatch sync --integrate --trunk       controlled trunk advance; idempotent
//!     -> conclude: remove the worktree DIR; deliverable refs SURVIVE
//! ```
//!
//! * EX-1 — the orchestrator wrote the *linked* coordination worktree, never the
//!   session `main` working tree (design §6: contention #1/#2 unreachable).
//! * EX-2 — reviewable refs are left for audit; the worktree directory is removed
//!   at conclude; `dispatch/064` + `phase/064-NN` deliverables are preserved until
//!   integration (today's GC would have deleted them — the bug §2 fixes).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
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

/// Does `refname` resolve in `dir`? (No assert — the survives-removal oracle.)
fn ref_exists(dir: &Path, refname: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--verify", "--quiet", refname])
        .output()
        .expect("spawn git")
        .status
        .success()
}

/// Run `doctrine <args>` in `cwd` with `DOCTRINE_WORKER` removed — the
/// orchestrator path is never worker-mode (mem.pattern.dispatch.worker-verify-
/// unset-doctrine-worker).
fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(cwd)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn commit(dir: &Path, path: &str, content: &str, msg: &str) -> String {
    let full = dir.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(&full, content).unwrap();
    git(dir, &["add", path]);
    git(dir, &["commit", "-q", "-m", msg]);
    git(dir, &["rev-parse", "HEAD"])
}

/// A `main` repo with a committed `plan.toml` — `coordinate` runs `slice phases`
/// off the committed plan, so the fixture must be committed (the coord worktree
/// is a checkout of trunk).
fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::write(dir.join("root-sentinel.txt"), "untouched").unwrap();
    let plan = ".doctrine/slice/064/plan.toml";
    let full = dir.join(plan);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(
        &full,
        "schema = \"doctrine.plan.overview\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n",
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base + plan fixture"]);
}

/// EX-1 / EX-2 / VT-1 — the full lifecycle, threaded.
#[test]
fn full_lifecycle_coordinate_to_integrate_preserves_main_and_deliverables() {
    let src = tempfile::tempdir().unwrap();
    let root = src.path();
    init_repo(root);
    let trunk = git(root, &["rev-parse", "HEAD"]);

    let holder = tempfile::tempdir().unwrap();
    let coord = holder.path().join("coord");

    // --- 1. create the markerless coordination worktree (orchestrator's tree) --
    let out = run(
        root,
        &[
            "worktree",
            "coordinate",
            "--slice",
            "64",
            "--dir",
            coord.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "coordinate; stderr: {}", stderr(&out));
    assert_eq!(
        git(&coord, &["rev-parse", "HEAD"]),
        trunk,
        "coord worktree forks off the resolved trunk"
    );

    // --- 2. land a phase's code + record its boundary ON the coordination tree --
    let code_tip = commit(&coord, "src/feature.rs", "fn f() {}\n", "PHASE-01 code");
    let out = run(
        &coord,
        &[
            "dispatch",
            "record-boundary",
            "--slice",
            "64",
            "--phase",
            "PHASE-01",
            "--code-start",
            &trunk,
            "--code-end",
            &code_tip,
            "-p",
            coord.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "record-boundary; stderr: {}",
        stderr(&out)
    );
    // sync tree-reads the ledger from the committed `dispatch/064` tree, never the
    // working filesystem (mem.pattern...sync-tree-reads-ledger) — so commit it.
    git(&coord, &["add", ".doctrine/dispatch/064"]);
    git(&coord, &["commit", "-q", "-m", "PHASE-01 boundary ledger"]);

    // --- 3. prepare-review: project the audit-ready refs from the coord tree ----
    let out = run(
        &coord,
        &[
            "dispatch",
            "sync",
            "--prepare-review",
            "--slice",
            "64",
            "-p",
            coord.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "prepare-review; stderr: {}",
        stderr(&out)
    );
    assert!(
        ref_exists(root, "review/064"),
        "review/064 left for audit (visible from the shared common dir)"
    );
    assert!(
        ref_exists(root, "phase/064-01"),
        "phase/064-01 synthesized cut left for audit"
    );

    // --- 4. EX-1 INVARIANT: the session `main` tree + trunk are UNTOUCHED -------
    // Everything above ran on the linked coordination worktree. The orchestrator
    // never wrote the session `main` working tree, and trunk has not moved.
    assert_eq!(
        git(root, &["rev-parse", "main"]),
        trunk,
        "trunk ref unmoved by the run (advance is integrate's act, not the run's)"
    );
    assert_eq!(
        git(root, &["status", "--porcelain"]),
        "",
        "session `main` working tree is byte-clean — orchestrator wrote the coord tree, not root"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("root-sentinel.txt")).unwrap(),
        "untouched",
        "root working file untouched"
    );

    // --- 5. integrate: the controlled trunk advance (idempotent) ---------------
    let phase_tip = git(root, &["rev-parse", "phase/064-01"]);
    let integrate = |extra: &[&str]| {
        let mut args = vec![
            "dispatch",
            "sync",
            "--integrate",
            "--slice",
            "64",
            "-p",
            coord.to_str().unwrap(),
        ];
        args.extend_from_slice(extra);
        run(&coord, &args)
    };
    let out = integrate(&["--trunk", "refs/heads/main"]);
    assert!(out.status.success(), "integrate; stderr: {}", stderr(&out));
    assert_eq!(
        git(root, &["rev-parse", "main"]),
        phase_tip,
        "trunk fast-forwards to the cumulative code tip at integration"
    );
    let out = integrate(&["--trunk", "refs/heads/main"]);
    assert!(
        out.status.success(),
        "idempotent re-integrate; stderr: {}",
        stderr(&out)
    );
    assert_eq!(
        git(root, &["rev-parse", "main"]),
        phase_tip,
        "no second advance — replay no-ops when current == planned"
    );

    // --- 6. conclude: remove the worktree DIR; deliverable refs SURVIVE --------
    // --force: provisioned (gitignored) runtime state lives in the coord dir.
    git(
        root,
        &["worktree", "remove", "--force", coord.to_str().unwrap()],
    );
    assert!(
        !coord.exists(),
        "coordination worktree directory removed at conclude"
    );
    assert!(
        ref_exists(root, COORD_BRANCH),
        "dispatch/064 deliverable preserved past worktree removal"
    );
    assert!(
        ref_exists(root, "phase/064-01"),
        "phase/064-01 deliverable preserved past worktree removal"
    );
    assert!(
        ref_exists(root, "review/064"),
        "review/064 stays reviewable after the worktree is gone"
    );
}
