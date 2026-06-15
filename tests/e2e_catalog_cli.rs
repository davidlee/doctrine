//! SL-071 PHASE-06 — `doctrine catalog scan --json` and `doctrine catalog graph --json`
//! integration tests over the shared equivalence fixture.
//!
//! Thin JSON dump of `Catalog` / `CatalogGraph` — no colour, no pagination,
//! no table format. Optional per design D12; not gating for acceptance.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic"
)]

use std::fs;
use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

// --------------- fixture helpers (same seed as e2e_sl071_equivalence) ---------------

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

fn seed_fixture(root: &Path) {
    // SL-001 — outbound requirements edge to REQ-005.
    write(
        root,
        ".doctrine/slice/001/slice-001.toml",
        "id = 1\nslug = \"s1\"\ntitle = \"S1\"\nstatus = \"proposed\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
         [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-005\"\n",
    );
    write(root, ".doctrine/slice/001/slice-001.md", "scope\n");

    // SL-003 — no relations (tests empty outbound).
    write(
        root,
        ".doctrine/slice/003/slice-003.toml",
        "id = 3\nslug = \"s3\"\ntitle = \"S3\"\nstatus = \"proposed\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
    );
    write(root, ".doctrine/slice/003/slice-003.md", "scope\n");

    // ADR-002 — governance entity with supersedes edge.
    write(
        root,
        ".doctrine/adr/002/adr-002.toml",
        "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"accepted\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
         [relationships]\nsupersedes = [\"ADR-001\"]\n",
    );
    write(root, ".doctrine/adr/002/adr-002.md", "body\n");

    // REQ-005 — edge target.
    write(
        root,
        ".doctrine/requirement/005/requirement-005.toml",
        "id = 5\nslug = \"r5\"\ntitle = \"R5\"\nstatus = \"active\"\n",
    );
    write(root, ".doctrine/requirement/005/requirement-005.md", "r\n");
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

// --------------- VT-1: `catalog scan --json` produces valid JSON ---------------

#[test]
fn catalog_scan_json_valid() {
    let tmp = tempfile::tempdir().unwrap();
    seed_fixture(tmp.path());

    let out = Command::new(BIN)
        .args(["catalog", "scan", "--json", "--root"])
        .arg(tmp.path())
        .output()
        .expect("spawn doctrine");

    assert!(
        out.status.success(),
        "catalog scan failed: {}",
        stderr(&out)
    );

    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");

    // Must carry the three top-level keys.
    assert!(v.get("entities").is_some(), "missing entities key");
    assert!(v.get("edges").is_some(), "missing edges key");
    assert!(v.get("diagnostics").is_some(), "missing diagnostics key");
}

// --------------- VT-2: `catalog graph --json` produces valid JSON ---------------

#[test]
fn catalog_graph_json_valid() {
    let tmp = tempfile::tempdir().unwrap();
    seed_fixture(tmp.path());

    let out = Command::new(BIN)
        .args(["catalog", "graph", "--json", "--root"])
        .arg(tmp.path())
        .output()
        .expect("spawn doctrine");

    assert!(
        out.status.success(),
        "catalog graph failed: {}",
        stderr(&out)
    );

    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");

    // Must carry nodes (as an object keyed by node key) and edges (as an array).
    assert!(v.get("nodes").is_some(), "missing nodes key");
    assert!(v.get("edges").is_some(), "missing edges key");
}

// --------------- VT-3: non-existent root exits non-zero ---------------

#[test]
fn catalog_scan_nonexistent_root_exits_nonzero() {
    let out = Command::new(BIN)
        .args(["catalog", "scan", "--json", "--root", "/nonexistent"])
        .output()
        .expect("spawn doctrine");

    assert!(!out.status.success(), "expected non-zero exit");
    assert!(!stderr(&out).is_empty(), "expected stderr message on error");
}
