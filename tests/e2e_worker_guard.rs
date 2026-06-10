//! SL-032 PHASE-01 — worker-mode guard (ADR-006 D2a) as BLACK-BOX goldens.
//!
//! `DOCTRINE_WORKER=1` makes the CLI hard-refuse every authored/memory/runtime
//! write BEFORE dispatch, with a verb-named `bail!` (stderr `Error: <msg>\n`,
//! nonzero exit) — the conformance basis for ADR-006's "writes refuse under
//! `DOCTRINE_WORKER=1`". Read paths stay open (INV-3). The unit table
//! (`write_class_tests`) proves the full Read/Write split + every label; these
//! tests prove the env gate fires end-to-end over the BUILT binary: VT-2 (each
//! representative Write verb refuses, named) + VT-3 (a Read verb is unaffected).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// `doctrine <args...>` under `DOCTRINE_WORKER=1`, rooted in a throwaway cwd so a
/// (never-reached) write could not touch the repo.
fn run_worker(cwd: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .env("DOCTRINE_WORKER", "1")
        .current_dir(cwd)
        .output()
        .expect("spawn doctrine")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

// VT-2: ≥1 representative Write verb per top-level command + every nested arm
// (memory sync None/Some, boot None/Some, spec req add). Each must refuse under
// the worker env: nonzero exit, the verb named, and the bare-`bail!` shape
// (`Error: …` with no `Caused by:` context chain).
const WRITE_VERBS: &[(&[&str], &str)] = &[
    (&["install"], "install"),
    (&["skills", "install"], "skills install"),
    (&["slice", "new", "x"], "slice new"),
    (
        &["memory", "record", "t", "--type", "concept"],
        "memory record",
    ),
    (&["memory", "sync"], "memory sync"),
    (&["memory", "sync", "install"], "memory sync install"),
    (&["adr", "new", "t"], "adr new"),
    (
        &["adr", "status", "1", "--status", "accepted"],
        "adr status",
    ),
    (&["policy", "new", "t"], "policy new"),
    (&["spec", "new", "product", "t"], "spec new"),
    (
        &["spec", "req", "add", "PRD-001", "--kind", "functional"],
        "spec req add",
    ),
    (&["backlog", "new", "issue", "t"], "backlog new"),
    (
        &["backlog", "edit", "ISS-001", "--status", "open"],
        "backlog edit",
    ),
    (&["boot"], "boot"),
    (&["boot", "install"], "boot install"),
    (&["reseat", "SL-001"], "reseat"),
];

#[test]
fn write_verbs_refuse_under_worker() {
    let dir = tmp();
    for (args, verb) in WRITE_VERBS {
        let out = run_worker(dir.path(), args);
        let err = stderr(&out);
        assert!(
            !out.status.success(),
            "{args:?} should refuse (nonzero exit); stderr: {err}"
        );
        assert!(
            err.starts_with("Error: "),
            "{args:?} should bail with the bare `Error: ` shape; stderr: {err}"
        );
        assert!(
            !err.contains("Caused by"),
            "{args:?} bail should have no `Caused by` context chain; stderr: {err}"
        );
        assert!(
            err.contains(&format!("`{verb}`")),
            "{args:?} refusal should name the verb `{verb}`; stderr: {err}"
        );
    }
}

// VT-3 (INV-3): a Read verb runs unaffected under the worker env — `slice list`
// on an empty tree exits 0 and is NOT the refusal.
#[test]
fn read_verb_unaffected_under_worker() {
    let dir = tmp();
    let out = run_worker(dir.path(), &["slice", "list", "-p", "."]);
    let err = stderr(&out);
    assert!(
        out.status.success(),
        "`slice list` should run under DOCTRINE_WORKER=1; stderr: {err}"
    );
    assert!(
        !err.contains("DOCTRINE_WORKER"),
        "`slice list` must not hit the worker guard; stderr: {err}"
    );
}

// VT-3 (INV-3): the corpus integrity scan is a Read verb — it must run, not
// refuse, under the worker env (an empty tree validates clean).
#[test]
fn validate_unaffected_under_worker() {
    let dir = tmp();
    let out = run_worker(dir.path(), &["validate", "-p", "."]);
    let err = stderr(&out);
    assert!(
        out.status.success(),
        "`validate` should run under DOCTRINE_WORKER=1; stderr: {err}"
    );
    assert!(
        !err.contains("DOCTRINE_WORKER"),
        "`validate` must not hit the worker guard; stderr: {err}"
    );
}
