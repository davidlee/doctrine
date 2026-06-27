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

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

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
    Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Seed runtime phase-tracking in the PRIMARY tree marking each phase
/// `completed`, so prepare-review's PHASE-05 completeness gate (design §5.2) sees
/// a complete slice.
fn seed_completed_phases(dir: &Path, slice: u32, phases: &[&str]) {
    let pdir = dir.join(format!(".doctrine/state/slice/{slice:03}/phases"));
    std::fs::create_dir_all(&pdir).unwrap();
    for p in phases {
        let stem = format!("phase-{}", p.strip_prefix("PHASE-").unwrap_or(p));
        std::fs::write(
            pdir.join(format!("{stem}.toml")),
            "status = \"completed\"\n",
        )
        .unwrap();
    }
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
    // Mirror the real repo invariant: `.doctrine/state/` is gitignored runtime
    // state (the storage model). The funnel's recorded source-delta registry
    // (SL-147) lands there in the PRIMARY tree — a gitignored runtime write that
    // must NOT dirty the session tree's `status --porcelain` (the EX-1 invariant
    // is "no tracked/authored write to the session tree", not "no runtime write").
    std::fs::write(dir.join(".gitignore"), ".doctrine/state/\n").unwrap();
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
    // The completeness gate roots on the PRIMARY worktree (git worktree list first
    // entry), NOT the coord worktree — seed phases there.
    seed_completed_phases(root, 64, &["PHASE-01"]);
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

// ── SL-165 PHASE-03: repair → close → integrate → status done (IMP-188 anchor)

/// Full candidate lifecycle anchor: prepare-review → review_surface candidate →
/// fix-now repair on the candidate branch → admit review_surface → close_target
/// from candidate (PHASE-02 gate) → admit close_target → integrate → status done.
/// Asserts the fix-now lands on trunk through the first-class path, not a manual
/// fold or hand-FF.
#[test]
fn repair_to_close_to_integrate_to_status_done() {
    let src = tempfile::tempdir().unwrap();
    let root = src.path();
    init_repo(root);
    let trunk = git(root, &["rev-parse", "HEAD"]);
    seed_completed_phases(root, 64, &["PHASE-01"]);

    let holder = tempfile::tempdir().unwrap();
    let coord = holder.path().join("coord");

    // --- 1. Create coordination worktree --------------------------------------
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

    // --- 2. Land a phase's code + record its boundary ON the coord tree ---------
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
    assert!(out.status.success(), "record-boundary; stderr: {}", stderr(&out));
    git(&coord, &["add", ".doctrine/dispatch/064"]);
    git(&coord, &["commit", "-q", "-m", "PHASE-01 boundary ledger"]);

    // --- 3. Prepare-review: create review/064 + journal rows -------------------
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
    assert!(out.status.success(), "prepare-review; stderr: {}", stderr(&out));
    assert!(ref_exists(root, "review/064"), "review/064 created");

    // --- 4. Create review_surface candidate (--worktree) -----------------------
    let out = run(
        root,
        &[
            "dispatch",
            "candidate",
            "create",
            "--slice",
            "64",
            "--label",
            "review-001",
            "--kind",
            "audit",
            "--role",
            "review_surface",
            "--payload",
            "impl_bundle",
            "--base",
            "refs/heads/main",
            "--source",
            "refs/heads/review/064",
            "--worktree",
        ],
    );
    assert!(out.status.success(), "review_surface create; stderr: {}", stderr(&out));
    let candidate_ref = "refs/heads/candidate/064/review-001";
    assert!(ref_exists(root, candidate_ref), "candidate branch exists");

    // --- 5. Commit a fix-now repair on the candidate branch --------------------
    let candidate_worktree = holder.path().join("cand");
    git(
        root,
        &[
            "worktree",
            "add",
            "--force",
            candidate_worktree.to_str().unwrap(),
            candidate_ref,
        ],
    );
    let fix_oid = commit(
        &candidate_worktree,
        "src/fix-now.rs",
        "// audit fix-now: add missing null check\nfn guard(x: Option<i32>) -> i32 { x.unwrap_or(0) }\n",
        "fix-now: add null guard",
    );
    // Candidates built from a worktree have their OWN index; the ref is advanced
    // in the shared object db (and the worktree sees it on next checkout).
    // Force-update the ref to the fix-now tip, which is the repaired content.
    git(root, &["update-ref", candidate_ref, &fix_oid]);
    let _ = git(root, &["worktree", "remove", "--force", candidate_worktree.to_str().unwrap()]);

    // --- 6. Admit the review_surface -------------------------------------------
    let out = run(
        root,
        &[
            "dispatch",
            "candidate",
            "admit",
            "--slice",
            "64",
            "--role",
            "review_surface",
            "--candidate",
            candidate_ref,
            "--review",
            "RV-001",
        ],
    );
    assert!(out.status.success(), "admit review_surface; stderr: {}", stderr(&out));

    // --- 7. Create close_target sourced from the candidate (PHASE-02 gate) -----
    let out = run(
        root,
        &[
            "dispatch",
            "candidate",
            "create",
            "--slice",
            "64",
            "--label",
            "close-001",
            "--kind",
            "audit",
            "--role",
            "close_target",
            "--payload",
            "code",
            "--base",
            "refs/heads/main",
            "--source",
            candidate_ref,
        ],
    );
    assert!(out.status.success(), "close_target from candidate; stderr: {}", stderr(&out));
    let close_ref = "refs/heads/candidate/064/close-001";
    assert!(ref_exists(root, close_ref), "close_target candidate exists");

    // --- 8. Admit the close_target ---------------------------------------------
    let out = run(
        root,
        &[
            "dispatch",
            "candidate",
            "admit",
            "--slice",
            "64",
            "--role",
            "close_target",
            "--candidate",
            close_ref,
            "--review",
            "RV-001",
        ],
    );
    assert!(out.status.success(), "admit close_target; stderr: {}", stderr(&out));

    // --- 9. Integrate — land the repair on trunk ------------------------------
    // Stage-2 runs from parent/root after the coordination worktree is removed
    // (sync --help). Route through root so admitted runtime state is visible.
    let out = run(
        root,
        &[
            "dispatch",
            "sync",
            "--integrate",
            "--slice",
            "64",
            "--trunk",
            "refs/heads/main",
            "-p",
            root.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "integrate; stderr: {}", stderr(&out));

    // --- 10. Assert trunk carries the fix-now ----------------------------------
    let trunk_tip = git(root, &["rev-parse", "refs/heads/main"]);
    let trunk_tree = git(root, &["ls-tree", "-r", "--name-only", &trunk_tip]);
    assert!(
        trunk_tree.contains("src/fix-now.rs"),
        "trunk tip tree must contain the fix-now file:\n{}",
        trunk_tree
    );

    // --- 11. Assert slice status done passes natively --------------------------
    // The integration gate checks: dispatched code integrated to trunk + no
    // open blockers. When those hold, the reconcile→done transition succeeds.
    // Transition through necessary intermediate states. The slice TOML is a
    // prerequisite (the status verb reads authored slice metadata); it needs
    // `status` + `updated` (the authored-TOML write keying).
    let slice_toml = root.join(".doctrine/slice/064/slice-064.toml");
    std::fs::write(
        &slice_toml,
        "id = 64\nslug = \"fixture\"\ntitle = \"fixture\"\nstatus = \"started\"\ncreated = \"2026-06-27\"\nupdated = \"2026-06-27\"\n\n[relationships]\nneeds = []\nafter = []\n",
    )
    .unwrap();
    git(root, &["add", ".doctrine/slice/064/slice-064.toml"]);
    git(root, &["commit", "-q", "-m", "add slice-064.toml for status transition"]);
    for state in ["audit", "reconcile", "done"] {
        let out = run(root, &["slice", "status", "64", state]);
        assert!(
            out.status.success(),
            "slice status {}; stderr: {}",
            state,
            stderr(&out)
        );
    }

    // --- 12. Assert no manual fold / hand-FF — the path is first-class --------
    // The review/064 ref still exists (was NOT deleted — no manual fold).
    assert!(
        ref_exists(root, "review/064"),
        "review/064 survives — no manual fold (branch -D review/064)"
    );
    // Trunk advanced past the prepare-review point (integration happened).
    let main_tip = git(root, &["rev-parse", "refs/heads/main"]);
    assert!(
        main_tip != trunk,
        "trunk advanced past the initial tip — integration ran, not a hand-FF standin"
    );
}
