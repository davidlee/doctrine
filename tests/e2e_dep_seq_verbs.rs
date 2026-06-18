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

// --- VT-5: after --remove (SL-105 PHASE-02) ----------------------------------

#[test]
fn after_remove_single() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 101, "One", "one"); // SL-101
    seed_slice(root, 102, "Two", "two"); // SL-102

    // Append after edge
    let append = run(root, &["after", "SL-101", "SL-102"]);
    assert!(append.status.success(), "append: {}", stderr(&append));

    // Remove the edge
    let remove = run(root, &["after", "SL-101", "SL-102", "--remove"]);
    assert!(remove.status.success(), "remove: {}", stderr(&remove));
    assert!(
        stdout(&remove).contains("removed (1 edge)"),
        "remove message: {}",
        stdout(&remove)
    );

    // Second remove — no edge left
    let again = run(root, &["after", "SL-101", "SL-102", "--remove"]);
    assert!(!again.status.success(), "second remove should fail");
    assert!(
        stderr(&again).contains("no after edge"),
        "error names the missing edge: {}",
        stderr(&again)
    );
}

#[test]
fn after_remove_rank_ceiling() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 103, "Three", "three"); // SL-103
    seed_slice(root, 104, "Four", "four"); // SL-104

    // Append two edges: rank 0 and rank 5
    assert!(run(root, &["after", "SL-103", "SL-104"]).status.success());
    assert!(
        run(root, &["after", "SL-103", "SL-104", "--rank", "5"])
            .status
            .success()
    );

    // Remove with rank ceiling 2 — only rank-0 edge removed, rank-5 kept
    let rm = run(
        root,
        &["after", "SL-103", "SL-104", "--remove", "--rank", "2"],
    );
    assert!(rm.status.success(), "rank-ceiling remove: {}", stderr(&rm));
    assert!(
        stdout(&rm).contains("removed (1 edge)"),
        "only rank-0 removed: {}",
        stdout(&rm)
    );

    // Remove remaining (no ceiling) → rank-5 edge removed
    let rm2 = run(root, &["after", "SL-103", "SL-104", "--remove"]);
    assert!(rm2.status.success(), "remove all: {}", stderr(&rm2));
    assert!(
        stdout(&rm2).contains("removed (1 edge)"),
        "rank-5 removed: {}",
        stdout(&rm2)
    );
}

#[test]
fn after_remove_nonexistent() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 105, "Five", "five"); // SL-105

    // Try to remove an edge to non-existent SL-999
    let rm = run(root, &["after", "SL-105", "SL-999", "--remove"]);
    assert!(!rm.status.success(), "non-existent target refused");
    assert!(
        stderr(&rm).contains("does not resolve"),
        "error names unresolvable target: {}",
        stderr(&rm)
    );
}

#[test]
fn after_remove_backlog() {
    let t = tmp();
    let root = t.path();
    new_issue(root, "One", "one"); // ISS-001
    new_issue(root, "Two", "two"); // ISS-002

    // Append edge
    let append = run(root, &["backlog", "after", "ISS-001", "ISS-002"]);
    assert!(append.status.success(), "append: {}", stderr(&append));

    // Remove the edge
    let remove = run(
        root,
        &["backlog", "after", "ISS-001", "ISS-002", "--remove"],
    );
    assert!(remove.status.success(), "remove: {}", stderr(&remove));
    assert!(
        stdout(&remove).contains("removed (1 edge)"),
        "remove message: {}",
        stdout(&remove)
    );
}

#[test]
fn after_append_still_works() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 106, "Six", "six"); // SL-106
    seed_slice(root, 107, "Seven", "seven"); // SL-107

    // Plain append (no flags)
    let out = run(root, &["after", "SL-106", "SL-107"]);
    assert!(out.status.success(), "append: {}", stderr(&out));
    assert_eq!(stdout(&out), "SL-106 after SL-107\n");

    // Append with rank
    let out2 = run(root, &["after", "SL-106", "SL-107", "--rank", "3"]);
    assert!(out2.status.success(), "append with rank: {}", stderr(&out2));
    assert_eq!(stdout(&out2), "SL-106 after SL-107 (rank 3)\n");
}

// --- SL-105 PHASE-03: prune goldens ---

/// Set an entity's status in its TOML (edit-preserving via toml_edit).
/// For backlog items, also sets a resolution when status is terminal.
fn set_entity_status(toml_path: &Path, status: &str) {
    let text = fs::read_to_string(toml_path).unwrap();
    let mut doc: toml_edit::DocumentMut = text.parse().unwrap();
    doc["status"] = toml_edit::value(status);
    if status == "resolved" || status == "closed" {
        doc["resolution"] = toml_edit::value("done");
    }
    fs::write(toml_path, doc.to_string()).unwrap();
}

/// Resolve a backlog item's TOML path from kind and id.
fn backlog_toml(root: &Path, kind: &str, id: u32) -> std::path::PathBuf {
    let name = format!("{id:03}");
    root.join(format!(".doctrine/backlog/{kind}/{name}/backlog-{name}.toml"))
}

// --- VT-1: after_prune_drops_resolved ---

#[test]
fn after_prune_drops_resolved() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 201, "Alpha", "alpha"); // SL-201
    seed_slice(root, 202, "Beta", "beta"); // SL-202

    // Append two after edges (rank 0 and rank 3)
    assert!(
        run(root, &["after", "SL-201", "SL-202"])
            .status
            .success(),
        "after rank 0"
    );
    assert!(
        run(root, &["after", "SL-201", "SL-202", "--rank", "3"])
            .status
            .success(),
        "after rank 3"
    );

    // Resolve SL-202
    let sl202 = root.join(".doctrine/slice/202/slice-202.toml");
    set_entity_status(&sl202, "resolved");

    // Prune
    let prune = run(root, &["after", "SL-201", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    let out = stdout(&prune);
    // 2 edges dropped
    assert!(
        out.contains("SL-201 after SL-202 (rank 0) dropped")
            && out.contains("SL-201 after SL-202 (rank 3) dropped"),
        "both edges dropped: {out}"
    );
    assert!(out.contains("resolved"), "reason contains resolved: {out}");

    // Verify SL-201 after array is empty
    let toml = slice_toml(root, 201);
    assert!(
        toml.contains("after = []"),
        "after array empty:\n{toml}"
    );
}

// --- VT-2: after_prune_noop ---

#[test]
fn after_prune_noop() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 203, "Charlie", "charlie"); // SL-203
    seed_slice(root, 204, "Delta", "delta"); // SL-204 (stays proposed)

    // Append edge
    assert!(
        run(root, &["after", "SL-203", "SL-204"])
            .status
            .success()
    );

    // Prune — nothing to prune (SL-204 is proposed/open, not terminal)
    let prune = run(root, &["after", "SL-203", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    assert!(
        stdout(&prune).contains("nothing to prune"),
        "no-op: {}",
        stdout(&prune)
    );

    // Edge still present
    let toml = slice_toml(root, 203);
    assert!(
        toml.contains("SL-204"),
        "edge still present:\n{toml}"
    );
}

// --- VT-3: after_prune_mixed ---

#[test]
fn after_prune_mixed() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 205, "Echo", "echo"); // SL-205
    seed_slice(root, 206, "Foxtrot", "foxtrot"); // SL-206 (live)
    seed_slice(root, 207, "Golf", "golf"); // SL-207 (will be resolved)

    // Append both edges
    assert!(
        run(root, &["after", "SL-205", "SL-206"])
            .status
            .success()
    );
    assert!(
        run(root, &["after", "SL-205", "SL-207"])
            .status
            .success()
    );

    // Resolve SL-207
    let sl207 = root.join(".doctrine/slice/207/slice-207.toml");
    set_entity_status(&sl207, "resolved");

    // Prune
    let prune = run(root, &["after", "SL-205", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    let out = stdout(&prune);
    assert!(
        out.contains("SL-205 after SL-207") && out.contains("dropped"),
        "SL-207 dropped: {out}"
    );
    assert!(
        !out.contains("SL-206"),
        "SL-206 NOT dropped: {out}"
    );

    // Verify TOML: only SL-206 remains
    let toml = slice_toml(root, 205);
    assert!(
        toml.contains("SL-206") && !toml.contains("SL-207"),
        "only SL-206 remains in after:\n{toml}"
    );
}

// --- after_prune_absent_target (bonus) ---

#[test]
fn after_prune_absent_target() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 208, "Hotel", "hotel"); // SL-208

    // Manually write an after edge to a non-existent target
    let toml_path = root.join(".doctrine/slice/208/slice-208.toml");
    let text = fs::read_to_string(&toml_path).unwrap();
    let mut doc: toml_edit::DocumentMut = text.parse().unwrap();
    let after = doc["relationships"]["after"].as_array_mut().unwrap();
    let mut edge = toml_edit::InlineTable::new();
    edge.insert("to", "SL-999".into());
    edge.insert("rank", 0.into());
    after.push(edge);
    fs::write(&toml_path, doc.to_string()).unwrap();

    // Prune
    let prune = run(root, &["after", "SL-208", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    let out = stdout(&prune);
    assert!(
        out.contains("absent"),
        "absent in reason: {out}"
    );

    // Edge is gone
    let toml = fs::read_to_string(&toml_path).unwrap();
    assert!(
        !toml.contains("SL-999"),
        "edge to SL-999 removed:\n{toml}"
    );
}

// --- VT-4: backlog_after_prune ---

#[test]
fn backlog_after_prune() {
    let t = tmp();
    let root = t.path();
    new_issue(root, "Prune Alpha", "prune-alpha"); // ISS-001
    new_issue(root, "Prune Beta", "prune-beta"); // ISS-002

    // Append edge: ISS-001 after ISS-002
    let after = run(root, &["backlog", "after", "ISS-001", "ISS-002"]);
    assert!(after.status.success(), "backlog after: {}", stderr(&after));

    // Resolve ISS-002
    let iss2 = backlog_toml(root, "issue", 2);
    set_entity_status(&iss2, "resolved");

    // Prune
    let prune = run(root, &["backlog", "after", "ISS-001", "--prune"]);
    assert!(
        prune.status.success(),
        "backlog prune exit: {}",
        stderr(&prune)
    );
    let out = stdout(&prune);
    assert!(
        out.contains("dropped"),
        "backlog prune dropped: {out}"
    );
}
