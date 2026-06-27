//! SL-025 PHASE-06 EX-1 / VT-1 — behavioural parse-conformance (R5 / A-4).
//!
//! The shared list surface (`CommonListArgs`, main.rs §5.2) is the MANDATORY spine
//! of every kind's `list` subcommand. clap exposes no structural "is this flattened?"
//! check (A-4), so conformance is proven BEHAVIOURALLY: for every spine kind,
//! each shared spine flag must PARSE and the command must SUCCEED. A kind
//! that quietly dropped the flatten — or shadowed a shared flag with a bespoke one —
//! would fail to parse the flag (clap error, non-zero exit) and trip this test.
//!
//! Run over the built binary because the crate is binary-only (the `Cli` clap type
//! is private; there is no lib to `try_parse_from` against). An empty temp project
//! root is enough: parse-conformance is about the ARG GRAMMAR, not the data.

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

/// The kinds that ride the shared list spine (SL-025), including the three
/// governance kinds (adr/policy/standard, SL-030/SL-033) — closing the gap that
/// the matrix previously omitted even policy (SL-033 VT-7). `skills list` is NOT
/// on the spine — it does not flatten `CommonListArgs` — so it is deliberately
/// excluded.
const SPINE_KINDS: [&str; 8] = [
    "adr",
    "policy",
    "standard",
    "slice",
    "spec",
    "backlog",
    "memory",
    "knowledge",
];

/// Every shared spine flag, in both its short and long forms where it has one,
/// plus the two output-format flags. Each entry is the arg vector appended to
/// `<kind> list` — it must parse and the command must succeed on an empty root.
/// (`-f`/`-r`/`-s`/`-t` take a value; `-i`/`-a`/`--json` are boolean.)
const SPINE_FLAGS: [&[&str]; 14] = [
    &["--filter", "x"],
    &["-f", "x"],
    &["--regexp", "x"],
    &["-r", "x"],
    &["--case-insensitive", "--regexp", "x"],
    &["-i", "-r", "x"],
    &["--status", "draft"],
    &["-s", "draft"],
    &["--tag", "x"],
    &["-t", "x"],
    &["--all"],
    &["-a"],
    &["--format", "json"],
    &["--json"],
];

/// Run `<kind> list <extra...> -p <dir>` over the built binary.
fn list(kind: &str, dir: &Path, extra: &[&str]) -> Output {
    Command::new(bin())
        .arg(kind)
        .arg("list")
        .args(extra)
        .arg("-p")
        .arg(dir)
        .output()
        .expect("spawn doctrine")
}

#[test]
fn every_spine_flag_parses_and_succeeds_on_every_kind() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    for kind in SPINE_KINDS {
        for flag in SPINE_FLAGS {
            // The status rows carry a concrete value (`draft`), which is in-vocab
            // only for spec + memory. For the other kinds `-s draft` is a VOCAB
            // rejection, not a parse failure — so skip it in this PARSE matrix and
            // prove `-s`/`--status` grammar separately in
            // `status_flag_is_recognised_grammar_on_every_kind`.
            let is_status = flag.first() == Some(&"--status") || flag.first() == Some(&"-s");
            if is_status && !status_vocab_has_draft(kind) {
                continue;
            }
            let out = list(kind, dir, flag);
            assert!(
                out.status.success(),
                "{kind} list {flag:?} must parse + succeed (spine flag present); stderr: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}

/// `--status draft` is in-vocab for spec + memory + policy + standard; adr/slice/
/// backlog reject `draft` (a vocab error, NOT a parse error). The parse-conformance contract is
/// "the FLAG parses", which the `--all`/`--filter`/`--regexp`/`--tag` rows already
/// prove for `-s`/`--status` grammar via the in-vocab kinds; for the others we
/// assert the flag is RECOGNISED (clap-level) by checking the error is the uniform
/// vocab error, not an "unexpected argument" parse error.
fn status_vocab_has_draft(kind: &str) -> bool {
    matches!(kind, "spec" | "memory" | "policy" | "standard")
}

#[test]
fn status_flag_is_recognised_grammar_on_every_kind() {
    // For kinds whose vocab lacks `draft`, `-s draft` is rejected — but by the
    // UNIFORM VOCAB validator, never by clap as an unknown flag. That proves the
    // `-s/--status` flag is present (parsed) even when its value is out of vocab.
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    for kind in ["adr", "slice", "backlog", "knowledge"] {
        let out = list(kind, dir, &["-s", "draft"]);
        assert!(
            !out.status.success(),
            "{kind} should reject the out-of-vocab status `draft`"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stderr.contains("unexpected argument") && !stderr.contains("unrecognized"),
            "{kind} `-s draft` must be a VOCAB error (flag recognised), not a parse error: {stderr}"
        );
    }
}

#[test]
fn columns_flag_is_accepted_on_memory_list() {
    // SL-049 IMP-017: memory is now ON the shared column model, so `--columns`
    // is accepted (no longer rejected with the SL-037 D9/R4 unsupported message).
    // On an empty root it succeeds with no header (render_columns suppresses it).
    // Per-column projection behaviour is pinned byte-exact in
    // `tests/e2e_list_columns_golden.rs` (memory T8 block).
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = list("memory", tmp.path(), &["--columns", "key"]);
    assert!(
        out.status.success(),
        "memory list --columns is accepted post-IMP-017: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("--columns is not supported"),
        "the old unsupported-message rejection is gone: {stderr}"
    );
}

#[test]
fn relation_list_and_census_accept_columns() {
    // SL-169 D1: `relation list` and `relation census` are the two `list`-family
    // subcommands that joined the `--columns` projection contract (SPEC-013). They
    // do NOT ride the `CommonListArgs` spine (bespoke arg sets: source/label/state,
    // include-memory), so they are proven here as a focused parse-conformance pair
    // rather than as `SPINE_KINDS` rows. Per-column select/order is pinned byte-exact
    // in `tests/e2e_list_columns_golden.rs` (RelationRow/CensusRow blocks); here we
    // only prove the new `--columns` grammar parses + succeeds on an empty root.
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    let relation = |extra: &[&str]| -> Output {
        Command::new(bin())
            .arg("relation")
            .args(extra)
            .arg("-p")
            .arg(dir)
            .output()
            .expect("spawn doctrine")
    };

    let out = relation(&["list", "--columns", "source,label,state"]);
    assert!(
        out.status.success(),
        "relation list --columns must parse + succeed (SL-169 D1); stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = relation(&["census", "--columns", "label,count"]);
    assert!(
        out.status.success(),
        "relation census --columns must parse + succeed (SL-169 D1); stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn the_filter_x_json_canonical_combination_parses_on_every_kind() {
    // The exact invocation the design names (§9 / R5 / A-4):
    //   `<kind> list --filter x --json`
    // must parse and succeed for every kind.
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();
    for kind in SPINE_KINDS {
        let out = list(kind, dir, &["--filter", "x", "--json"]);
        assert!(
            out.status.success(),
            "{kind} list --filter x --json must parse + succeed; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        // and it must emit the shared envelope (the kind tag proves the spine
        // render path, not a bespoke one).
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("\"kind\"") && stdout.contains("\"rows\""),
            "{kind} --json must emit the shared {{kind, rows}} envelope: {stdout}"
        );
    }
}
