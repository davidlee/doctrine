// SPDX-License-Identifier: GPL-3.0-only
//! SL-060 PHASE-03 — the generic cross-kind `needs`/`after` capture verbs + the
//! author-time work-like target gate + the slice `[relationships]` scaffold, as
//! BLACK-BOX goldens over the built binary (design §5.4 / D2 / D4).
//!
//! - VT-1: a slice→slice `needs`/`after` authored via `doctrine needs`/`after`
//!   round-trips through `slice show` (Table, byte-exact) and `slice show --json`.
//! - VT-2: the closed-allowlist refusals, each a clear message — unresolvable TGT,
//!   free-text TGT, self-edge, non-authoring SRC kind, non-work-like TGT kind. The
//!   widened arm mints a REAL review (RV) so a RESOLVABLE non-work-like target is
//!   refused by the kind assertion (not merely the unresolvable path).
//! - VT-3: the backlog `needs`/`after` success-message text is byte-identical via
//!   the shared leaf delegate, and the backlog author-time cycle refuse still fires.
//! - VT-4: INV-1 ordering — a freshly scaffolded slice with a `[[relation]]` row
//!   (via `link`) PLUS `needs`/`after` keeps `[relationships]` + both arrays BEFORE
//!   the first `[[relation]]` row on disk.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Run the binary against the temp corpus. `DOCTRINE_WORKER` is explicitly UNSET — the
/// self-arm guard refuses authored writes under it, and a stray inherited var would
/// spuriously red an authored round-trip (mem.pattern.dispatch.worker-verify-unset).
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
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

/// Hand-seed a slice's authored TOML + MD with FIXED dates (the
/// `e2e_adr_cli_golden.rs` pattern) so `slice show` determinism doesn't couple
/// to wall-clock `clock::today()`.
fn seed_slice(root: &Path, id: u32, title: &str, slug: &str) {
    let name = format!("{id:03}");
    let dir = root.join(format!(".doctrine/slice/{name}"));
    std::fs::create_dir_all(&dir).unwrap();
    let toml = format!(
        "id = {id}\n\
         slug = \"{slug}\"\n\
         title = \"{title}\"\n\
         status = \"proposed\"\n\
         created = \"2026-06-14\"\n\
         updated = \"2026-06-14\"\n\
         \n\
         [relationships]\n\
         needs = []\n\
         after = []\n"
    );
    std::fs::write(dir.join(format!("slice-{name}.toml")), &toml).unwrap();
    std::fs::write(
        dir.join(format!("slice-{name}.md")),
        format!("# {title}\n\n## Context\n\n## Scope & Objectives\n\n## Non-Goals\n\n## Summary\n\n## Follow-Ups\n"),
    )
    .unwrap();
}

fn new_slice(root: &Path, title: &str, slug: &str) {
    let out = run(root, &["slice", "new", title, "--slug", slug]);
    assert!(out.status.success(), "slice new {slug}: {}", stderr(&out));
}

fn new_issue(root: &Path, title: &str, slug: &str) {
    let out = run(root, &["backlog", "new", "issue", title, "--slug", slug]);
    assert!(out.status.success(), "backlog new {slug}: {}", stderr(&out));
}

fn slice_toml(root: &Path, id: u32) -> String {
    fs::read_to_string(root.join(format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"))).unwrap()
}

// --- VT-1: slice→slice needs/after round-trips through show + show --json ---

#[test]
fn slice_needs_after_round_trip_table_and_json() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 1, "Alpha", "alpha"); // SL-001
    seed_slice(root, 2, "Beta", "beta"); // SL-002
    // The `clock::today()` stamp would make `slice show` non-deterministic, so the
    // slices are hand-seeded with fixed dates (the e2e_adr_cli_golden.rs pattern).

    assert!(
        run(root, &["needs", "SL-001", "SL-002"]).status.success(),
        "needs authored"
    );
    let after = run(root, &["after", "SL-001", "SL-002", "--rank", "3"]);
    assert!(after.status.success(), "after authored: {}", stderr(&after));

    // Table show — byte-exact (doctrine forces no-tty styling, so the bytes are stable).
    let show = run(root, &["slice", "show", "1"]);
    assert!(show.status.success(), "slice show: {}", stderr(&show));
    let expected = "\
SL-001 — Alpha
alpha · proposed
conduct: self/auto
created 2026-06-14 · updated 2026-06-14

relationships:
  needs: SL-002
  after: SL-002 (rank 3)

# Alpha

## Context

## Scope & Objectives

## Non-Goals

## Summary

## Follow-Ups
";
    assert_eq!(stdout(&show), expected, "Table show byte-exact");

    // JSON show — the dep/seq axes surface alongside the tier-1 axes.
    let json = run(root, &["slice", "show", "1", "--json"]);
    assert!(
        json.status.success(),
        "slice show --json: {}",
        stderr(&json)
    );
    let v: serde_json::Value = serde_json::from_str(&stdout(&json)).expect("valid JSON");
    let rel = v
        .get("slice")
        .and_then(|s| s.get("relationships"))
        .expect("relationships");
    assert_eq!(rel["needs"], serde_json::json!(["SL-002"]), "json needs");
    assert_eq!(
        rel["after"],
        serde_json::json!([{ "to": "SL-002", "rank": 3 }]),
        "json after"
    );
}

// --- VT-2: the closed-allowlist refusals, each a clear message --------------

#[test]
fn dep_seq_verbs_refuse_off_allowlist_targets_and_sources() {
    let t = tmp();
    let root = t.path();
    new_slice(root, "Alpha", "alpha"); // SL-001
    new_issue(root, "Eye", "eye"); // ISS-001
    // A REAL review (RV-001) — a RESOLVABLE non-work-like target (widened coverage).
    let rv = run(
        root,
        &[
            "review",
            "new",
            "--facet",
            "reconciliation",
            "--target",
            "SL-001",
        ],
    );
    assert!(rv.status.success(), "review new: {}", stderr(&rv));

    // (a) unresolvable TGT.
    let unresolved = run(root, &["needs", "SL-001", "SL-999"]);
    assert!(!unresolved.status.success(), "unresolvable TGT refused");
    assert!(
        stderr(&unresolved).contains("does not resolve"),
        "names the dangler: {}",
        stderr(&unresolved)
    );

    // (b) free-text TGT (not a canonical ref).
    let freetext = run(root, &["needs", "SL-001", "just-some-words"]);
    assert!(!freetext.status.success(), "free-text TGT refused");
    assert!(
        stderr(&freetext).contains("unknown kind prefix"),
        "names the bad ref shape: {}",
        stderr(&freetext)
    );

    // (c) self-edge.
    let selfedge = run(root, &["after", "SL-001", "SL-001"]);
    assert!(!selfedge.status.success(), "self-edge refused");
    assert!(
        stderr(&selfedge).contains("self-edge") || stderr(&selfedge).contains("to itself"),
        "names the self-edge: {}",
        stderr(&selfedge)
    );

    // (d) non-authoring SRC kind (an ADR cannot author dep/seq).
    let bad_src = run(root, &["needs", "ADR-001", "SL-001"]);
    assert!(!bad_src.status.success(), "non-authoring SRC refused");
    assert!(
        stderr(&bad_src).contains("cannot author needs/after"),
        "names the non-authoring source: {}",
        stderr(&bad_src)
    );

    // (e) WIDENED: a RESOLVABLE non-work-like TGT (RV) — the allowlist refuses EVERY
    // non-{slice,backlog} kind, not just the obvious gov/spec/req/knowledge. RV-001
    // passes ensure_ref_resolves, so this exercises the work-like KIND assertion.
    let bad_tgt = run(root, &["needs", "SL-001", "RV-001"]);
    assert!(!bad_tgt.status.success(), "non-work-like TGT (RV) refused");
    assert!(
        stderr(&bad_tgt).contains("may only target work"),
        "names the work-only gate: {}",
        stderr(&bad_tgt)
    );

    // Nothing was written on any refusal — SL-001 keeps both arrays empty.
    let toml = slice_toml(root, 1);
    assert!(
        toml.contains("needs = []") && toml.contains("after = []"),
        "every refusal wrote nothing: {toml}"
    );
}

// --- VT-3: backlog needs/after byte-identical via delegate + cycle refuse ----

#[test]
fn backlog_needs_after_message_byte_identical_and_cycle_still_refuses() {
    let t = tmp();
    let root = t.path();
    new_issue(root, "Auth", "auth"); // ISS-001
    new_issue(root, "Login", "login"); // ISS-002

    // backlog needs success message — byte-identical via the shared leaf delegate.
    let needs = run(root, &["backlog", "needs", "ISS-001", "ISS-002"]);
    assert!(needs.status.success(), "backlog needs: {}", stderr(&needs));
    assert_eq!(
        stdout(&needs),
        "ISS-001 needs ISS-002\n",
        "needs message byte-exact"
    );

    // backlog after success message — byte-identical (rank suffix at non-zero).
    let after = run(
        root,
        &["backlog", "after", "ISS-002", "ISS-001", "--rank", "2"],
    );
    assert!(after.status.success(), "backlog after: {}", stderr(&after));
    assert_eq!(
        stdout(&after),
        "ISS-002 after ISS-001 (rank 2)\n",
        "after message byte-exact"
    );

    // The backlog author-time cycle refuse still fires: ISS-001 needs ISS-002 exists,
    // so `needs ISS-002 ISS-001` would close the {ISS-001, ISS-002} cycle.
    let cycle = run(root, &["backlog", "needs", "ISS-002", "ISS-001"]);
    assert!(!cycle.status.success(), "closing cycle is refused");
    let msg = stderr(&cycle);
    assert!(
        msg.contains("cycle") && msg.contains("ISS-001") && msg.contains("ISS-002"),
        "cycle refuse names members: {msg}"
    );
}

// --- VT-4: INV-1 ordering — [relationships] before the first [[relation]] ----

#[test]
fn scaffolded_slice_keeps_relationships_before_first_relation_row() {
    let t = tmp();
    let root = t.path();
    new_slice(root, "Alpha", "alpha"); // SL-001
    new_slice(root, "Beta", "beta"); // SL-002
    // A real ADR — a valid `governed_by` link target.
    let adr = run(root, &["adr", "new", "Layering", "--slug", "layering"]);
    assert!(adr.status.success(), "adr new: {}", stderr(&adr));

    assert!(run(root, &["needs", "SL-001", "SL-002"]).status.success());
    assert!(run(root, &["after", "SL-001", "SL-002"]).status.success());
    assert!(
        run(root, &["link", "SL-001", "governed_by", "ADR-001"])
            .status
            .success(),
        "link a structural [[relation]] row"
    );

    let toml = slice_toml(root, 1);
    let rel_table = toml
        .find("[relationships]")
        .expect("[relationships] present");
    let first_row = toml.find("[[relation]]").expect("[[relation]] present");
    assert!(
        rel_table < first_row,
        "[relationships] table precedes the first [[relation]] row:\n{toml}"
    );
    // Both seeded arrays are populated and live inside the table (before the row).
    let needs_at = toml.find("needs = [\"SL-002\"]").expect("needs array");
    let after_at = toml
        .find("after = [{ to = \"SL-002\"")
        .expect("after array");
    assert!(
        needs_at < first_row && after_at < first_row,
        "both dep/seq arrays precede the first [[relation]] row:\n{toml}"
    );
}
