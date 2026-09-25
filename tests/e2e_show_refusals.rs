//! SL-265 PHASE-02 — the router inherits `kinds::parse_resolvable_ref`'s error
//! contract unchanged (`DEC-297`, design sec-7). An unknown prefix, a dangling
//! ref and an ambiguous bare id each exit non-zero carrying the resolver's own
//! text.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::process::Output;

mod common;

/// `doctrine show <arg>`, over the built binary against the ambient repo root.
fn show(arg: &str) -> Output {
    common::doctrine_cmd(&common::repo_root())
        .args(["show", arg])
        .output()
        .expect("spawn doctrine")
}

/// Assert the invocation fails and its stderr carries `expected`.
fn assert_refused(arg: &str, expected: &str) {
    let out = show(arg);
    assert!(
        !out.status.success(),
        "`doctrine show {arg}` unexpectedly succeeded ({}): {}",
        out.status,
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(expected),
        "`doctrine show {arg}` stderr missing {expected:?}: {stderr}"
    );
}

/// VT-2: a prefix outside the numbered address space is refused.
#[test]
fn unknown_prefix_is_refused() {
    assert_refused("ZZ-001", "unknown kind prefix");
}

/// VT-2: a well-formed prefix with no entity is refused as dangling.
#[test]
fn dangling_ref_is_refused() {
    assert_refused("SL-999", "does not resolve to an entity");
}

/// VT-2: a bare id held by more than one kind is genuinely ambiguous.
#[test]
fn ambiguous_bare_id_is_refused() {
    assert_refused("001", "is ambiguous");
}
