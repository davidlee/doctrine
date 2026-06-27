// SPDX-License-Identifier: GPL-3.0-only
//! SL-168 PHASE-05 — black-box golden tests for `doctrine doctor`.
//!
//! Runs the BUILT binary against the real project corpus and against
//! a controlled temp corpus. Asserts:
//! - VT-1: human output structure (category headers, finding lines, summary)
//! - VT-2: JSON mode produces the {kind, rows} envelope with severity rows
//! - VT-3: doctor is a strict superset of validate (superset invariant)
//! - VT-4: clean corpus exits 0 with "corpus clean"

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic"
)]

use std::process::Command;

mod common;

/// ---- helpers ----

fn run_doctor(args: &[&str]) -> std::process::Output {
    Command::new(common::doctrine_bin())
        .arg("doctor")
        .args(args)
        .output()
        .expect("spawn doctrine doctor")
}

fn run_doctor_in(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(common::doctrine_bin())
        .arg("doctor")
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine doctor")
}

/// ---- VT-1: human output structure ----

#[test]
fn doctor_human_output_has_expected_structure() {
    let out = run_doctor(&[]);
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Every doctor run ends with either "corpus clean" or "N finding(s)".
    assert!(
        stdout.contains("corpus clean") || stdout.contains(" finding(s)"),
        "doctor output must carry a summary line; got: {stdout}"
    );

    // Category headers are bracketed, e.g. "[Id Integrity]"
    // If there are findings, there must be at least one bracketed category header.
    if stdout.contains(" finding(s)") && !stdout.contains("corpus clean") {
        assert!(
            stdout.contains('['),
            "findings output must include category headers; got: {stdout}"
        );
    }
}

/// ---- VT-2: JSON mode ----

#[test]
fn doctor_json_produces_envelope_with_severity_rows() {
    let out = run_doctor(&["--json"]);
    // JSON mode always writes to stdout regardless of exit code.

    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("doctor --json must emit valid JSON");

    // Shared list envelope `{kind, rows}` (design §5.4 / F7).
    assert_eq!(
        parsed.get("kind").and_then(|k| k.as_str()),
        Some("doctor"),
        "doctor --json must carry the envelope kind; got: {stdout}"
    );
    let rows = parsed
        .get("rows")
        .and_then(|r| r.as_array())
        .expect("doctor --json must carry a rows array");

    // Every row carries category, severity (derived from category), message.
    for (i, obj) in rows.iter().enumerate() {
        assert!(
            obj.get("category").is_some(),
            "row {i} missing category: {obj}"
        );
        let sev = obj.get("severity").and_then(|s| s.as_str());
        assert!(
            matches!(sev, Some("error" | "warning")),
            "row {i} missing/invalid severity: {obj}"
        );
        assert!(
            obj.get("message").is_some(),
            "row {i} missing message: {obj}"
        );
    }
}

/// ---- VT-3: superset invariant (doctor ⊇ validate) ----

#[test]
fn doctor_is_superset_of_validate() {
    // Run both against the real project corpus.
    let doctor_out = run_doctor(&[]);
    let validate_out = Command::new(common::doctrine_bin())
        .arg("validate")
        .arg("-p")
        .arg(common::repo_root())
        .output()
        .expect("spawn doctrine validate");

    let doctor_stdout = String::from_utf8_lossy(&doctor_out.stdout);
    let validate_stdout = String::from_utf8_lossy(&validate_out.stdout);

    // Validate finding lines start with "  " (indent) and carry a message
    // after the indent. Doctor lines add a severity prefix ("error: " or
    // "warning: ") before the same message. Strip the severity prefix from
    // doctor lines for comparison.
    let doctor_stripped: Vec<String> = doctor_stdout
        .lines()
        .filter(|l| l.starts_with("  ") && l.contains(':'))
        .map(|l| {
            let trimmed = l.trim_start();
            if let Some(rest) = trimmed.strip_prefix("error: ") {
                rest.to_string()
            } else if let Some(rest) = trimmed.strip_prefix("warning: ") {
                rest.to_string()
            } else {
                trimmed.to_string()
            }
        })
        .collect();

    let validate_lines: Vec<&str> = validate_stdout
        .lines()
        .filter(|l| l.starts_with("  "))
        .map(|l| l.trim_start())
        .collect();

    for line in &validate_lines {
        assert!(
            doctor_stripped.iter().any(|d| d == *line),
            "doctor must contain validate finding: {line:?}\n\
             doctor output:\n{doctor_stdout}\n\
             validate output:\n{validate_stdout}"
        );
    }

    // Doctor should have at least as many findings as validate has lines.
    if !validate_stdout.contains("validate: corpus clean") {
        let validate_count = validate_lines.len();
        let doctor_count = doctor_stripped.len();
        assert!(
            doctor_count >= validate_count,
            "doctor findings ({doctor_count}) must be >= validate findings ({validate_count})"
        );
    }
}

/// ---- VT-4: clean exit on empty corpus ----

#[test]
fn doctor_clean_corpus_exits_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Create the .doctrine directory to satisfy root detection.
    std::fs::create_dir_all(dir.path().join(".doctrine")).expect("mkdir .doctrine");

    let out = run_doctor_in(dir.path(), &[]);
    assert!(out.status.success(), "clean corpus must exit 0");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("corpus clean"),
        "clean corpus must say 'corpus clean'; got: {stdout}"
    );
}

/// ---- VT-5: non-zero exit on errors ----

#[test]
fn doctor_exits_nonzero_on_id_integrity_error() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Create a slice with a duplicate-id violation: basename ≠ toml id.
    let slice_dir = dir.path().join(".doctrine/slice/001");
    std::fs::create_dir_all(&slice_dir).expect("mkdir slice/001");
    std::fs::write(
        slice_dir.join("slice-001.toml"),
        "id = 2\nslug = \"dup\"\ntitle = \"Dup\"\nstatus = \"proposed\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
    )
    .expect("write toml");
    std::fs::write(slice_dir.join("slice-001.md"), "body\n").expect("write md");

    let out = run_doctor_in(dir.path(), &[]);
    assert!(!out.status.success(), "error findings must exit non-zero");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Id Integrity"),
        "should have Id Integrity header"
    );
    assert!(stdout.contains("finding(s)"), "should have finding count");
}
