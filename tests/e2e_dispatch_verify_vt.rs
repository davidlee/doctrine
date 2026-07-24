// SPDX-License-Identifier: GPL-3.0-only
//! SL-170 PHASE-04 VT-2/VT-3 — the dispatch **conclude** gate as a BLACK-BOX
//! golden over the BUILT binary. The `/dispatch` conclude cadence is
//! `slice verify-vt <id>` → on green `prepare-review` → remove coord worktree;
//! verify-vt runs in the coord tree (its fs reader sees the committed plan,
//! since the orchestrator is sole writer and commits before the gate). These
//! tests pin the conclude *contract* of that gate, distinct from the PHASE-03
//! `e2e_slice_verify_vt` verdict matrix:
//!
//! - VT-2: a clean plan exits 0 with the VT summary block shown (handover
//!   proceeds); a plan with a failing VT exits non-zero (a Fail HALTS handover).
//! - VT-3: a waiver on the coord-tree plan.toml is honoured — the row renders
//!   WAIVED (non-halting), exit 0, handover proceeds (EX-3 positive half; the
//!   negative working-fs-only-rejection case rides the deferred committed-graph
//!   reader, out of scope here).
//!
//! Matching is plain raw substring (Option D / POL-002); the binary is resolved
//! at RUNTIME via `common::doctrine_bin()`.

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

fn head_sha(dir: &Path) -> String {
    git(dir, &["rev-parse", "HEAD"])
}

/// A CLEAN conclude plan: a satisfied VT, a non-gated VA, an Uncheckable (no
/// `test_file`), and a Waived row. No Fail → the conclude gate passes (exit 0)
/// and handover proceeds, with the block still shown.
const PLAN_CLEAN: &str = r#"
schema  = "doctrine.plan.overview"
version = 1
slice   = "SL-001"

[[phase]]
id   = "PHASE-01"
name = "conclude clean"
verification = [
  { id = "VT-1", expects = "satisfied mandate", test_file = "tests/good.rs", keywords = ["census"] },
  { id = "VT-2", expects = "no structured mandate", keywords = ["x"] },
  { id = "VT-3", expects = "infeasible, waived on the coord tree", waived = true, waived_reason = "infeasible — see /consult" },
  { id = "VA-1", expects = "agent-checked, never gated" },
]
"#;

/// A FAILING conclude plan: a mandated `test_file` whose keyword is absent →
/// Fail → the conclude gate HALTS handover (non-zero).
const PLAN_FAIL: &str = r#"
schema  = "doctrine.plan.overview"
version = 1
slice   = "SL-001"

[[phase]]
id   = "PHASE-01"
name = "conclude fail"
verification = [
  { id = "VT-1", expects = "the mandated keyword is OMITTED from the file", test_file = "tests/good.rs", keywords = ["nonexistent"] },
]
"#;

/// Build a temp coord-tree root as a git repo with one phase's source-delta
/// recorded so the VT gate can attribute keyword matches to slice work
/// (IMP-228). `plan` is written to `.doctrine/slice/001/plan.toml`; a
/// `tests/good.rs` carrying the `census` token is committed in the delta.
fn coord_tree(plan: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // Set up a git repo with an initial empty commit as the baseline.
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.email", "t@example.com"]);
    git(root, &["config", "user.name", "Test"]);
    std::fs::write(root.join("empty"), "").expect("base file");
    git(root, &["add", "empty"]);
    git(root, &["commit", "-q", "-m", "base"]);
    let base_sha = head_sha(root);

    // Write the slice plan and test file, then commit as the delta.
    let slice = root.join(".doctrine/slice/001");
    std::fs::create_dir_all(&slice).expect("mkdir slice");
    std::fs::write(slice.join("plan.toml"), plan).expect("write plan.toml");
    std::fs::create_dir_all(root.join("tests")).expect("mkdir tests");
    std::fs::write(root.join("tests/good.rs"), "fn t() { let census = 1; }").expect("good.rs");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "delta"]);
    let end_sha = head_sha(root);

    // Record the source delta so verify-vt can attribute the test file.
    let rec = Command::new(bin())
        .args([
            "slice",
            "record-delta",
            "1",
            "PHASE-01",
            "--start",
            &base_sha,
            "--end",
            &end_sha,
            "-p",
        ])
        .arg(root)
        .output()
        .expect("record-delta");
    assert!(
        rec.status.success(),
        "record-delta failed: {}",
        String::from_utf8_lossy(&rec.stderr)
    );

    dir
}

/// Run the conclude gate: `doctrine slice verify-vt <id> -p <coord-root>`.
fn run_conclude_gate(root: &Path, id: &str) -> Output {
    let mut cmd = Command::new(common::doctrine_bin());
    cmd.arg("slice").arg("verify-vt").arg(id);
    cmd.arg("-p").arg(root);
    cmd.output().expect("spawn doctrine slice verify-vt")
}

/// VT-2 (pass): a clean plan exits 0 and the conclude surface still emits the VT
/// summary block — handover proceeds with the gaps visible.
#[test]
fn conclude_gate_passes_clean_plan_and_shows_block() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let root = coord_tree(PLAN_CLEAN);
    let out = run_conclude_gate(root.path(), "1");
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        out.status.success(),
        "a clean plan must let handover proceed (exit 0): {out:?}"
    );
    assert!(
        stdout.contains("VT verification summary"),
        "the conclude surface must emit the VT summary block: {stdout}"
    );
    assert!(
        stdout.contains("PASS"),
        "the satisfied VT renders Pass: {stdout}"
    );
}

/// VT-2 (halt): a failing VT forces a non-zero exit — the conclude gate HALTS
/// handover rather than shipping an incomplete mandate as green.
#[test]
fn conclude_gate_halts_handover_on_fail() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let root = coord_tree(PLAN_FAIL);
    let out = run_conclude_gate(root.path(), "1");
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        !out.status.success(),
        "a Fail must halt handover (non-zero): {out:?}"
    );
    assert!(
        stdout.contains("FAIL"),
        "the Fail must render in the block: {stdout}"
    );
}

/// VT-3: a waiver on the coord-tree plan.toml is honoured at conclude — the row
/// renders WAIVED (non-halting), exit 0, handover proceeds. At conclude the
/// coord working tree == the committed graph (sole-writer commits the waiver
/// before the gate), so the fs reader sees it.
#[test]
fn committed_coord_tree_waiver_is_honoured_at_conclude() {
    if common::under_worker_marker() {
        return;
    } // SL-225 #2: skip in a worker fork
    let root = coord_tree(PLAN_CLEAN);
    let out = run_conclude_gate(root.path(), "1");
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        out.status.success(),
        "a waived row is non-halting — handover proceeds: {out:?}"
    );
    assert!(
        stdout.contains("WAIVED"),
        "the committed waiver must render distinctly as WAIVED: {stdout}"
    );
    assert!(
        stdout.contains("infeasible"),
        "the waiver rationale must surface for auditability: {stdout}"
    );
}
