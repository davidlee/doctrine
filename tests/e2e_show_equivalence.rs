//! SL-265 PHASE-02 — the slice's central property as a BLACK-BOX fidelity gate.
//!
//! For every prefixed ref a kind's own `show` accepts, the kind-blind
//! `doctrine show <REF>` must emit stdout byte-identical to that kind's own
//! invocation, in both `--format table` and `--format json` (design sec-7). Both
//! sides are the SAME built binary invoked twice — the reference is the kind's
//! own verb, never a stored golden (RV-384 F-21), so the test reads the corpus it
//! is run in and compares this build against itself.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::process::Output;

mod common;

/// One fixture entity per `KINDS` row, paired with the kind's own `show`
/// subcommand args. `REQ` is the one prefix off the uniform path — its reference
/// command is `doctrine spec req show` (design sec-7; RV-384 F-12). The seven
/// knowledge records route through `doctrine knowledge show` and the five backlog
/// kinds through `doctrine backlog show`; the rest through `doctrine <kind> show`.
const FIXTURES: &[(&str, &[&str])] = &[
    ("SL-001", &["slice", "show"]),
    ("ADR-001", &["adr", "show"]),
    ("POL-001", &["policy", "show"]),
    ("STD-001", &["standard", "show"]),
    ("RFC-001", &["rfc", "show"]),
    ("PRD-001", &["spec", "show"]),
    ("SPEC-001", &["spec", "show"]),
    ("REQ-197", &["spec", "req", "show"]),
    ("ISS-008", &["backlog", "show"]),
    ("IMP-012", &["backlog", "show"]),
    ("CHR-001", &["backlog", "show"]),
    ("RSK-232", &["backlog", "show"]),
    ("IDE-007", &["backlog", "show"]),
    ("RV-001", &["review", "show"]),
    ("REC-001", &["rec", "show"]),
    ("ASM-001", &["knowledge", "show"]),
    ("DEC-001", &["knowledge", "show"]),
    ("QUE-001", &["knowledge", "show"]),
    ("CON-001", &["knowledge", "show"]),
    ("EVD-001", &["knowledge", "show"]),
    ("HYP-001", &["knowledge", "show"]),
    ("CPT-001", &["knowledge", "show"]),
    ("CM-001", &["concept-map", "show"]),
    ("REV-001", &["revision", "show"]),
];

/// The built binary, spawned against the ambient repo root with the worker-mode
/// env leg stripped (`common::doctrine_cmd`).
fn run(args: &[&str]) -> Output {
    common::doctrine_cmd(&common::repo_root())
        .args(args)
        .output()
        .expect("spawn doctrine")
}

/// A short stderr tail for an assertion message — the last few lines, so a
/// failure names what the binary actually said.
fn stderr_tail(out: &Output) -> String {
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut lines: Vec<&str> = stderr.lines().collect();
    if lines.len() > 3 {
        lines.drain(..lines.len() - 3);
    }
    lines.join(" | ")
}

/// Assert `doctrine show <REF>` and the kind's own verb produce byte-identical
/// stdout across the whole fixture table, in the given format.
fn assert_equivalence(format: &str) {
    for (reference, kind_args) in FIXTURES {
        let router = run(&["show", reference, "--format", format]);

        let mut kind_argv: Vec<&str> = kind_args.to_vec();
        kind_argv.push(reference);
        kind_argv.extend(["--format", format]);
        let kind = run(&kind_argv);

        assert!(
            router.status.success(),
            "`doctrine show {reference} --format {format}` failed ({}) — stderr tail: {}",
            router.status,
            stderr_tail(&router)
        );
        assert!(
            kind.status.success(),
            "`doctrine {} {reference} --format {format}` failed ({}) — stderr tail: {}",
            kind_args.join(" "),
            kind.status,
            stderr_tail(&kind)
        );
        assert!(
            router.stdout == kind.stdout,
            "`doctrine show {reference} --format {format}` is NOT byte-identical to \
             `doctrine {} {reference} --format {format}`\n\
             router stderr tail: {}\nkind stderr tail: {}",
            kind_args.join(" "),
            stderr_tail(&router),
            stderr_tail(&kind),
        );
    }
}

/// VT-1: the table-format half of the byte-equivalence property.
#[test]
fn show_matches_the_kind_verb_in_table() {
    assert_equivalence("table");
}

/// VT-1: the json-format half of the byte-equivalence property (`--format json`,
/// never the `--json` shorthand, on both sides).
#[test]
fn show_matches_the_kind_verb_in_json() {
    assert_equivalence("json");
}
