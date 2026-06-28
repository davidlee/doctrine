// SPDX-License-Identifier: GPL-3.0-only
//! SL-168 PHASE-06 — byte-exact golden for `doctrine validate`.
//!
//! Guards the D12 native re-point: validate output must be identical
//! before and after the extraction.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic"
)]

use std::process::Command;

mod common;

#[test]
fn validate_byte_exact_golden() {
    let output = Command::new(common::doctrine_bin())
        .args(["validate"])
        .arg("-p")
        .arg(common::repo_root())
        .output()
        .expect("failed to run validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Capture the full output (stdout + stderr since validate uses stdout)
    let combined = format!("{stdout}{stderr}");

    // Run this test ONCE to capture the real output, then paste it as the expected string.
    // This is the byte-exact golden — ANY change to validate output will fail this test.
    let expected = concat!(
        "validate: scanned SL, ADR, POL, STD, PRD, SPEC, REQ, ISS, IMP, CHR, RSK, IDE, RV, REC, ASM, DEC, QUE, CON, EVD, HYP, CM, REV, RFC\n",
        "validate: corpus clean",
    );

    // Normalize newlines for cross-platform comparison
    let combined = combined.replace("\r\n", "\n");
    let expected = expected.replace("\r\n", "\n");

    assert_eq!(
        combined.trim(),
        expected.trim(),
        "validate output changed — the native re-point must produce identical output"
    );
}
