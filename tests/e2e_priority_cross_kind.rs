// SPDX-License-Identifier: GPL-3.0-only
//! SL-060 PHASE-04 — the cross-kind dep/seq READ-gate, as BLACK-BOX goldens over the
//! built binary (design D7/§5.2/§5.4). The priority engine's blocker/next/seq view now
//! reaches SLICE dep/seq edges (and any future authoring kind), not just backlog.
//!
//! - VT-1: a slice→slice `needs` authored via `doctrine needs` surfaces SL-b as a
//!   cross-kind blocker of SL-a in `blockers`, AND holds SL-a behind it in `next`.
//! - VT-2: a slice→slice `after` (a soft sequence) orders the predecessors before the
//!   dependent in the composed `next` worklist — the slice seq overlay behaving as the
//!   backlog seq overlay does. Rank/age eviction-key semantics are pinned at the graph
//!   unit level (`src/priority/graph.rs`); here we pin the operator-facing ordering.

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

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Run the binary against the temp corpus. DOCTRINE_WORKER is explicitly UNSET — the
/// self-arm guard refuses authored writes under it (mem.pattern.dispatch.worker-verify-unset).
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .arg("-p")
        .arg(root)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn new_slice(root: &Path, title: &str, slug: &str) {
    let out = run(root, &["slice", "new", title, "--slug", slug]);
    assert!(out.status.success(), "slice new {slug}: {}", stderr(&out));
}

// --- VT-1: slice→slice needs is a cross-kind blocker; holds the dependent in next ---

#[test]
fn slice_needs_is_cross_kind_blocker_and_holds_next() {
    let t = tmp();
    let root = t.path();
    new_slice(root, "Alpha", "alpha"); // SL-001
    new_slice(root, "Beta", "beta"); // SL-002

    // SL-001 needs SL-002 — a slice→slice hard prerequisite.
    let needs = run(root, &["needs", "SL-001", "SL-002"]);
    assert!(needs.status.success(), "needs authored: {}", stderr(&needs));

    // blockers SL-001 surfaces SL-002 as the direct blocker (cross-kind read-gate).
    let blk = run(root, &["blockers", "SL-001", "--json"]);
    assert!(blk.status.success(), "blockers SL-001: {}", stderr(&blk));
    let v: serde_json::Value = serde_json::from_str(&stdout(&blk)).expect("valid JSON");
    assert_eq!(
        v["blocked_by"],
        serde_json::json!(["SL-002"]),
        "SL-002 is the direct blocked-by prerequisite of SL-001"
    );

    // Reciprocally, SL-002 is blocking SL-001.
    let blk2 = run(root, &["blockers", "SL-002", "--json"]);
    let v2: serde_json::Value = serde_json::from_str(&stdout(&blk2)).expect("valid JSON");
    assert_eq!(
        v2["blocking"],
        serde_json::json!(["SL-001"]),
        "SL-002 blocks SL-001"
    );

    // next holds SL-001 behind SL-002: SL-002 is actionable, SL-001 is absent (blocked).
    let next = run(root, &["next", "--json"]);
    assert!(next.status.success(), "next: {}", stderr(&next));
    let nv: serde_json::Value = serde_json::from_str(&stdout(&next)).expect("valid JSON");
    let ids: Vec<&str> = nv["rows"]
        .as_array()
        .expect("entries array")
        .iter()
        .map(|e| e["id"].as_str().expect("id str"))
        .collect();
    assert!(
        ids.contains(&"SL-002"),
        "SL-002 (unblocked) is in next: {ids:?}"
    );
    assert!(
        !ids.contains(&"SL-001"),
        "SL-001 is BLOCKED → absent from next: {ids:?}"
    );
}

// --- VT-2: slice→slice after orders predecessors before the dependent in next ---

#[test]
fn slice_after_orders_predecessors_before_dependent_in_next() {
    let t = tmp();
    let root = t.path();
    new_slice(root, "Alpha", "alpha"); // SL-001 (the dependent)
    new_slice(root, "Beta", "beta"); // SL-002 (predecessor, rank 7)
    new_slice(root, "Gamma", "gamma"); // SL-003 (predecessor, rank 0)

    // SL-001 after SL-002 (rank 7) then after SL-003 — a soft sequence, all actionable.
    assert!(
        run(root, &["after", "SL-001", "SL-002", "--rank", "7"])
            .status
            .success(),
        "after SL-002 authored"
    );
    assert!(
        run(root, &["after", "SL-001", "SL-003"]).status.success(),
        "after SL-003 authored"
    );

    // next: all three are actionable (after is soft, not blocking), but the composed
    // dependency/sequence order places SL-001 AFTER both its predecessors.
    let next = run(root, &["next", "--json"]);
    assert!(next.status.success(), "next: {}", stderr(&next));
    let nv: serde_json::Value = serde_json::from_str(&stdout(&next)).expect("valid JSON");
    let ids: Vec<&str> = nv["rows"]
        .as_array()
        .expect("entries array")
        .iter()
        .map(|e| e["id"].as_str().expect("id str"))
        .collect();
    let pos = |id: &str| ids.iter().position(|x| *x == id).expect("present in next");
    assert!(
        pos("SL-002") < pos("SL-001") && pos("SL-003") < pos("SL-001"),
        "both after-predecessors precede the dependent SL-001 in next: {ids:?}"
    );
}
