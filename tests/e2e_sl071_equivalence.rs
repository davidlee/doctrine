//! SL-071 PHASE-02 — `doctrine inspect <ID>` byte-identical goldens
//! over the shared equivalence fixture.
//!
//! Pins the inspect CLI surface (PHASE-01 re-home did not drift the
//! output) over the BUILT binary. The fixture is hand-seeded with fixed
//! dates — no `doctrine` CLI verbs, no clock dependency.
//!
//! Golden files checked in under `tests/fixtures/` and loaded via
//! `include_str!` (VA-1: no inlined string literals).

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

/// Write `root/<rel>` with `body`, creating parent dirs.
fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

/// Seed the shared equivalence fixture (SL-001, SL-003, ADR-002, REQ-005).
fn seed_fixture(root: &Path) {
    // SL-001 — outbound references(implements) edge to REQ-005 (SL-149 PHASE-05).
    write(
        root,
        ".doctrine/slice/001/slice-001.toml",
        "id = 1\nslug = \"s1\"\ntitle = \"S1\"\nstatus = \"proposed\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
         [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-005\"\n",
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

/// `doctrine inspect <id> --json -p <root>`.
fn run_inspect(root: &Path, id: &str) -> String {
    let out = Command::new(BIN)
        .args(["inspect", id, "--json", "-p"])
        .arg(root)
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "inspect {} failed: {}",
        id,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

// === VT: byte-identical inspect --json goldens ===

#[test]
fn inspect_sl001_json_byte_identical() {
    let dir = tmp();
    seed_fixture(dir.path());

    let got = run_inspect(dir.path(), "SL-001");
    let golden = include_str!("fixtures/sl071_inspect_sl001_golden.json");
    assert_eq!(
        got, golden,
        "inspect SL-001 --json output drifted from golden"
    );
}

#[test]
fn inspect_sl003_json_byte_identical() {
    let dir = tmp();
    seed_fixture(dir.path());

    let got = run_inspect(dir.path(), "SL-003");
    let golden = include_str!("fixtures/sl071_inspect_sl003_golden.json");
    assert_eq!(
        got, golden,
        "inspect SL-003 --json output drifted from golden"
    );
}
