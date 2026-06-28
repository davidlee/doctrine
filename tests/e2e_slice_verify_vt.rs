// SPDX-License-Identifier: GPL-3.0-only
//! SL-170 PHASE-03 VT-4 — `doctrine slice verify-vt <id>` as a BLACK-BOX golden
//! over the BUILT binary against a temp root (`-p/--path`).
//!
//! Covers what an in-process `vtgate` unit cannot: the `std::process::exit`
//! exit-code forwarding (non-zero iff any `Fail`, INV-4) and the shell call path
//! that reads the authored `plan.toml` and fs-reads the mandated files relative
//! to the root. The fixture plan mixes all four verdicts — `Pass` / `Fail` /
//! `Uncheckable` / `Waived` — plus a non-gated `VA` row, and reconstructs the
//! SL-169 (b) failure mode: a `relation` census conformance matrix that exists
//! but OMITS the mandated `census` keyword → `Fail` → halt.
//!
//! Matching is plain raw substring (Option D / POL-002 — no host-language
//! comment/string stripping); the binary is resolved at RUNTIME via
//! `common::doctrine_bin()`, never `env!("CARGO_BIN_EXE_…")`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

/// A fixture plan exercising every verdict. PHASE-01 mixes Pass / Fail (missing
/// file) / Uncheckable (keywords, no `test_file`) / Waived, plus a non-gated VA
/// row. PHASE-02 is the SL-169 (b) replay: a mandated `relation` matrix whose
/// `census` keyword is OMITTED → Fail.
const PLAN: &str = r#"
schema  = "doctrine.plan.overview"
version = 1
slice   = "SL-001"

[[phase]]
id   = "PHASE-01"
name = "verdict mix"
verification = [
  { id = "VT-1", expects = "good file with the keyword", test_file = "tests/good.rs", keywords = ["census"] },
  { id = "VT-2", expects = "mandated file absent", test_file = "tests/missing.rs", keywords = ["whatever"] },
  { id = "VT-3", expects = "keywords but no test_file is Uncheckable", keywords = ["x"] },
  { id = "VT-4", expects = "infeasible mandate", waived = true, waived_reason = "infeasible — see /consult" },
  { id = "VA-1", expects = "agent-checked, never gated" },
]

[[phase]]
id   = "PHASE-02"
name = "relation census matrix"
verification = [
  { id = "VT-1", expects = "the relation census conformance matrix", test_file = "tests/relation.rs", keywords = ["relation", "census"] },
]
"#;

/// Build a temp root with the fixture plan at `.doctrine/slice/001/plan.toml`
/// and the mandated source files: `tests/good.rs` carries `census`;
/// `tests/relation.rs` carries `relation` but OMITS `census` (the SL-169 hole);
/// `tests/missing.rs` is deliberately absent.
fn fixture_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let slice = root.join(".doctrine/slice/001");
    std::fs::create_dir_all(&slice).expect("mkdir slice");
    std::fs::write(slice.join("plan.toml"), PLAN).expect("write plan.toml");

    std::fs::create_dir_all(root.join("tests")).expect("mkdir tests");
    // VT-1 satisfied: the keyword is present (even as a bare token).
    std::fs::write(root.join("tests/good.rs"), "fn t() { let census = 1; }").expect("good.rs");
    // PHASE-02 VT-1: file exists, `relation` present, `census` OMITTED → Fail.
    std::fs::write(
        root.join("tests/relation.rs"),
        "fn matrix() { check(\"relation\"); }",
    )
    .expect("relation.rs");
    dir
}

/// Run `doctrine slice verify-vt <id> -p <root>`.
fn run(root: &Path, id: &str) -> Output {
    let mut cmd = Command::new(common::doctrine_bin());
    cmd.arg("slice").arg("verify-vt").arg(id);
    cmd.arg("-p").arg(root);
    cmd.output().expect("spawn doctrine slice verify-vt")
}

/// VT-4: the mixed fixture exits non-zero (a Fail is present), and the report
/// surfaces all four verdicts — Pass, Fail, Uncheckable, Waived — distinctly,
/// the waived reason included.
#[test]
fn mixed_plan_halts_and_renders_all_four_verdicts() {
    let root = fixture_root();
    let out = run(root.path(), "1");
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        !out.status.success(),
        "a Fail must force a non-zero exit (INV-4), got: {out:?}"
    );
    assert!(stdout.contains("PASS"), "expected a Pass line: {stdout}");
    assert!(stdout.contains("FAIL"), "expected a Fail line: {stdout}");
    assert!(
        stdout.contains("UNCHECKABLE"),
        "Uncheckable must render distinctly: {stdout}"
    );
    assert!(
        stdout.contains("WAIVED"),
        "Waived must render distinctly: {stdout}"
    );
    assert!(
        stdout.contains("infeasible"),
        "the waived reason must surface: {stdout}"
    );
}

/// SL-169 (b) replay: a mandated `relation` matrix that OMITS the `census`
/// keyword → Fail naming the absent keyword → the gate halts. This is the
/// completeness-blindness hole the slice closes.
#[test]
fn relation_census_omission_fails_and_halts() {
    let root = fixture_root();
    let out = run(root.path(), "1");
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        !out.status.success(),
        "the census omission must halt: {out:?}"
    );
    assert!(
        stdout.contains("census"),
        "the Fail reason must name the omitted `census` keyword: {stdout}"
    );
}

/// A non-gated `VA` row produces no VT line (it is parsed, never gated).
#[test]
fn va_row_is_not_gated() {
    let root = fixture_root();
    let out = run(root.path(), "1");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("VA-1"),
        "VA criteria are parsed but never gated — no line: {stdout}"
    );
}
