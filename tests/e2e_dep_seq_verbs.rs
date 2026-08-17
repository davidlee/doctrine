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
use std::process::Output;

mod common;

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Run the binary against the temp corpus. `DOCTRINE_WORKER` is explicitly UNSET — the
/// self-arm guard refuses authored writes under it, and a stray inherited var would
/// spuriously red an authored round-trip (mem.pattern.dispatch.worker-verify-unset).
fn run(root: &Path, args: &[&str]) -> Output {
    common::doctrine_cmd(root)
        .args(args)
        .arg("-p")
        .arg(root)
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

fn seed_adr(root: &Path, id: u32, title: &str) {
    let name = format!("{id:03}");
    let dir = root.join(format!(".doctrine/adr/{name}"));
    std::fs::create_dir_all(&dir).unwrap();
    let toml = format!(
        "id = {id}\n\
         slug = \"adr-{name}\"\n\
         title = \"{title}\"\n\
         status = \"accepted\"\n\
         created = \"2026-06-14\"\n\
         updated = \"2026-06-14\"\n"
    );
    std::fs::write(dir.join(format!("adr-{name}.toml")), &toml).unwrap();
    std::fs::write(
        dir.join(format!("adr-{name}.md")),
        format!("# {title}\n\n## Context\n\n## Decision\n\n## Consequences\n"),
    )
    .unwrap();
}

fn slice_toml(root: &Path, id: u32) -> String {
    fs::read_to_string(root.join(format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"))).unwrap()
}

/// Hand-seed ANY entity's authored pair with FIXED dates, given its kind directory
/// and file stem — the generalisation of [`seed_slice`], added by SL-238 PHASE-07.
///
/// `--prune`'s collapsed probe judges its target through `authored_class`, so the
/// fixtures now have to span kinds with *different* terminal vocabularies (a slice's
/// `done`, a question's `answered`) and the status-LESS kind whose class is decided
/// without reading a status at all. Three more near-copies of `seed_slice` was the
/// alternative.
///
/// `status: None` authors no `status` key — the on-disk shape of a `kinds::STATUS_LESS`
/// kind (`REC`), and the fixture behind the fifth consequence (`EX-4`, `VT-5`).
fn seed_entity(root: &Path, dir: &str, stem: &str, id: u32, title: &str, status: Option<&str>) {
    let name = format!("{id:03}");
    let d = root.join(dir).join(&name);
    fs::create_dir_all(&d).unwrap();
    let status_line = match status {
        Some(s) => format!("status = \"{s}\"\n"),
        None => String::new(),
    };
    fs::write(
        d.join(format!("{stem}-{name}.toml")),
        format!(
            "id = {id}\n\
             slug = \"{stem}-{name}\"\n\
             title = \"{title}\"\n\
             {status_line}\
             created = \"2026-06-14\"\n\
             updated = \"2026-06-14\"\n"
        ),
    )
    .unwrap();
    fs::write(d.join(format!("{stem}-{name}.md")), format!("# {title}\n")).unwrap();
}

/// Seed a knowledge question (`QUE-NNN`) — `gating: ["open"]`,
/// `terminal: ["answered", "obsolete"]` per `priority::partition`. An admissible
/// `after` target, and the second vocabulary `VT-1` needs.
fn seed_question(root: &Path, id: u32, status: &str) {
    seed_entity(
        root,
        ".doctrine/knowledge/question",
        "record",
        id,
        "A question",
        Some(status),
    );
}

/// Seed a reconciliation record (`REC-NNN`) — the one `kinds::STATUS_LESS` kind,
/// authoring no `status` key. Not an admissible `after` target, so only
/// [`push_raw_after_edge`] can point an edge at it.
fn seed_rec(root: &Path, id: u32) {
    seed_entity(root, ".doctrine/rec", "rec", id, "A record", None);
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
    // Seed ADR-001 so parse_resolvable_ref can resolve it; the work-like
    // kind gate must fire, not the existence check.
    seed_adr(root, 1, "Module layering");
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

/// SUPERSEDED BY PHASE-06 (SL-238, design §6 "The remove path gates the source,
/// not the target") — the remove path stops resolving the TARGET on disk, so
/// `SL-999` no longer trips the target resolver. The verb still exits non-zero
/// here (no such edge is present to clear), but with `SL-105 has no after edge to
/// SL-999`; the `does not resolve` assertion below goes red by design. Superseded
/// on the message, not on the exit status — annotated under SL-238 PHASE-02 D2.
/// `after_remove_pins_the_refusal_of_an_unresolvable_target_leaving_the_edge` is
/// the pin that demonstrates the actual gap this one misses.
/// SL-105 origin; **reason superseded by SL-238 PHASE-06 / EX-3.**
///
/// The removal still fails and the exit code is unchanged — but the reason moved.
/// It used to fail at the author-time gate ("`SL-999` does not resolve to an
/// entity"); now the remove path gates the SOURCE only, so `SL-999` canonicalises
/// happily and the refusal comes one layer later, from the zero-count bail. The
/// distinction matters: refusing because the TARGET is unknown blocks repair, while
/// refusing because there is NO SUCH EDGE is the honest answer to this input.
#[test]
fn after_remove_nonexistent() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 105, "Five", "five"); // SL-105

    // Try to remove an edge to non-existent SL-999 — SL-105 has no edges at all.
    let rm = run(root, &["after", "SL-105", "SL-999", "--remove"]);
    assert!(!rm.status.success(), "non-existent edge refused");
    assert_eq!(
        stderr(&rm),
        "Error: SL-105 has no after edge to SL-999\n",
        "refused for the absent EDGE, not the unresolvable target"
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

/// SL-238 QUE-221 — the before-state pin for `backlog after`'s cross-kind target
/// refusal, which SL-238 §6 deliberately removes.
///
/// Today both target-bearing legs of `backlog after` gate the TARGET through
/// `require_item` (`src/backlog.rs:2406` append, `:2374` remove), and that resolves
/// via `parse_ref`, which knows only backlog kinds. So a slice target is refused
/// even when it exists on disk. §6 routes those legs through the kind-neutral
/// `commands::dep_seq` operations, whose canonicaliser accepts any resolvable ref —
/// the refusal goes away, on purpose, in PHASE-08.
///
/// **This test is written to be superseded.** §7's rule is that deliberately changed
/// behaviour needs its before-state pinned first, so the change is visible AS a
/// change; without the pin, PHASE-08's widening lands as a fresh green test that
/// cannot distinguish "I made this work" from "this always worked". PHASE-08 must
/// find this red and rewrite it in place into its opposite — cross-kind target
/// ACCEPTED, edge written — rather than deleting it.
///
/// Raised as `D4` in PHASE-02's runtime sheet and left to the owner as a scope call;
/// accepted 2026-08-17, between PHASE-06 and PHASE-07, because after PHASE-08 lands
/// the pin cannot be written at all.
///
/// **The refusal text is pinned by EQUALITY, deliberately.** A `contains("SL-154")`
/// assertion is vacuous here: once §6's canonicaliser accepts the ref, the remove leg
/// falls through to the zero-count bail and emits `ISS-001 has no after edge to
/// SL-154` — still non-zero, still naming the target. A loose pin would stay green
/// across the exact change it exists to catch. Only the message distinguishes
/// "refused because the KIND is inadmissible" from "refused because there is no such
/// edge", so only the message is load-bearing.
#[test]
fn backlog_after_pins_the_cross_kind_target_refusal_on_both_legs() {
    /// The author-time kind gate's refusal — `require_item` → `parse_ref`, which
    /// knows only backlog prefixes. PHASE-08 deletes this reason from both legs.
    const KIND_REFUSAL: &str =
        "Error: unknown backlog prefix `SL` in `SL-154` (expected ISS/IMP/CHR/RSK/IDE)\n";

    let t = tmp();
    let root = t.path();
    new_issue(root, "One", "one"); // ISS-001
    new_issue(root, "Two", "two"); // ISS-002
    // A REAL slice on disk: the refusal below is provably the kind gate, not absence.
    seed_slice(root, 154, "Cross kind", "cross-kind");

    // Positive control — the same verb, same source, a backlog target: succeeds.
    // Without this, the two refusals below could equally be a broken fixture.
    let ok = run(root, &["backlog", "after", "ISS-001", "ISS-002"]);
    assert!(
        ok.status.success(),
        "backlog target accepted: {}",
        stderr(&ok)
    );

    // Append leg: refused on the target's KIND.
    let append = run(root, &["backlog", "after", "ISS-001", "SL-154"]);
    assert!(
        !append.status.success(),
        "append refuses a cross-kind target"
    );
    assert_eq!(stderr(&append), KIND_REFUSAL, "append refused on the KIND");

    // Remove leg: refused at the same gate, before the edge lookup is even reached.
    let remove = run(root, &["backlog", "after", "ISS-001", "SL-154", "--remove"]);
    assert!(
        !remove.status.success(),
        "remove refuses a cross-kind target"
    );
    assert_eq!(stderr(&remove), KIND_REFUSAL, "remove refused on the KIND");

    // The refused append left no edge behind — the ISS-002 control edge is the only one.
    let iss1 = fs::read_to_string(backlog_toml(root, "issue", 1)).unwrap();
    assert!(
        !iss1.contains("SL-154"),
        "refused append wrote nothing: {iss1}"
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

/// Set an entity's `status` and, when `resolution` is `Some`, its `resolution`
/// (edit-preserving via toml_edit). The resolution word is a parameter because
/// `--prune`'s reason string renders `{status}/{resolution}` verbatim, so a
/// hardcoded `done` cannot pin it (SL-238 PHASE-02).
fn set_entity_fields(toml_path: &Path, status: &str, resolution: Option<&str>) {
    let text = fs::read_to_string(toml_path).unwrap();
    let mut doc: toml_edit::DocumentMut = text.parse().unwrap();
    doc["status"] = toml_edit::value(status);
    if let Some(resolution) = resolution {
        doc["resolution"] = toml_edit::value(resolution);
    }
    fs::write(toml_path, doc.to_string()).unwrap();
}

/// Set an entity's status in its TOML (edit-preserving via toml_edit).
/// For backlog items, also sets a resolution when status is terminal.
fn set_entity_status(toml_path: &Path, status: &str) {
    let terminal = status == "resolved" || status == "closed";
    set_entity_fields(toml_path, status, terminal.then_some("done"));
}

/// Push a raw `{ to, rank }` row onto an entity's `after` array, bypassing the
/// CLI's canonicalising author-time gate. The only way to plant the hand-authored
/// refs whose handling is under characterisation (`"154"`, `"SL-9999"`,
/// `"not-a-ref"`) — the gate refuses to *write* exactly the refs this slice is
/// about clearing.
fn push_raw_after_edge(toml_path: &Path, to: &str, rank: i64) {
    let text = fs::read_to_string(toml_path).unwrap();
    let mut doc: toml_edit::DocumentMut = text.parse().unwrap();
    let after = doc["relationships"]["after"].as_array_mut().unwrap();
    let mut edge = toml_edit::InlineTable::new();
    edge.insert("to", to.into());
    edge.insert("rank", rank.into());
    after.push(edge);
    fs::write(toml_path, doc.to_string()).unwrap();
}

/// Push a raw string onto an entity's `needs` array, bypassing the canonicalising
/// author-time gate — the `needs`-axis twin of [`push_raw_after_edge`], and the only
/// route to a planted `"SL-9999"` / `"not-a-ref"` / `"SL-1"` prerequisite.
fn push_raw_needs_ref(toml_path: &Path, to: &str) {
    let text = fs::read_to_string(toml_path).unwrap();
    let mut doc: toml_edit::DocumentMut = text.parse().unwrap();
    doc["relationships"]["needs"]
        .as_array_mut()
        .unwrap()
        .push(to);
    fs::write(toml_path, doc.to_string()).unwrap();
}

/// Resolve a backlog item's TOML path from kind and id.
fn backlog_toml(root: &Path, kind: &str, id: u32) -> std::path::PathBuf {
    let name = format!("{id:03}");
    root.join(format!(
        ".doctrine/backlog/{kind}/{name}/backlog-{name}.toml"
    ))
}

// --- VT-1: after_prune_drops_resolved ---

/// SUPERSEDED ON ITS FIXTURE BY SL-238 PHASE-07 (design §6 "`--prune`'s probe,
/// collapsed", first consequence) — was `after_prune_drops_resolved`.
///
/// The behaviour it pins is unchanged: a terminal target's edges are dropped, at
/// every rank. What changed is that the fixture was **invalid for its kind**. It
/// set a SLICE's status to `resolved`, which is a backlog word — ADR-009's slice
/// vocabulary is `proposed…reconcile`, `done`, `abandoned`. It only ever passed
/// because the old probe hardcoded `resolved || closed` and applied it to every
/// kind alike; that cross-kind leak IS the STD-001 duplicated-table violation and
/// the REQ-238 routing breach this phase closes.
///
/// So the fixture moves to `done`, the slice's real terminal word. Under the
/// collapsed probe the old fixture would now be `Unrecognised` and KEPT — which is
/// §6's second consequence working, not a regression.
#[test]
fn after_prune_drops_a_terminal_target() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 201, "Alpha", "alpha"); // SL-201
    seed_slice(root, 202, "Beta", "beta"); // SL-202

    // Append two after edges (rank 0 and rank 3)
    assert!(
        run(root, &["after", "SL-201", "SL-202"]).status.success(),
        "after rank 0"
    );
    assert!(
        run(root, &["after", "SL-201", "SL-202", "--rank", "3"])
            .status
            .success(),
        "after rank 3"
    );

    // Settle SL-202 — `done` is the SLICE terminal word (ADR-009); see the doc above.
    let sl202 = root.join(".doctrine/slice/202/slice-202.toml");
    set_entity_status(&sl202, "done");

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
    assert!(
        out.contains("done"),
        "reason is the kind's own terminal word: {out}"
    );

    // Verify SL-201 after array is empty
    let toml = slice_toml(root, 201);
    assert!(toml.contains("after = []"), "after array empty:\n{toml}");
}

// --- VT-2: after_prune_noop ---

#[test]
fn after_prune_noop() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 203, "Charlie", "charlie"); // SL-203
    seed_slice(root, 204, "Delta", "delta"); // SL-204 (stays proposed)

    // Append edge
    assert!(run(root, &["after", "SL-203", "SL-204"]).status.success());

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
    assert!(toml.contains("SL-204"), "edge still present:\n{toml}");
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
    assert!(run(root, &["after", "SL-205", "SL-206"]).status.success());
    assert!(run(root, &["after", "SL-205", "SL-207"]).status.success());

    // Settle SL-207. `done`, not `resolved`: SL-238 PHASE-07 routes terminality
    // through each kind's own table, and `resolved` is a backlog word that only
    // ever worked here via the hardcoded cross-kind literal this phase removes.
    let sl207 = root.join(".doctrine/slice/207/slice-207.toml");
    set_entity_status(&sl207, "done");

    // Prune
    let prune = run(root, &["after", "SL-205", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    let out = stdout(&prune);
    assert!(
        out.contains("SL-205 after SL-207") && out.contains("dropped"),
        "SL-207 dropped: {out}"
    );
    assert!(!out.contains("SL-206"), "SL-206 NOT dropped: {out}");

    // Verify TOML: only SL-206 remains
    let toml = slice_toml(root, 205);
    assert!(
        toml.contains("SL-206") && !toml.contains("SL-207"),
        "only SL-206 remains in after:\n{toml}"
    );
}

// --- after_prune_absent_target (bonus) ---

/// SUPERSEDED BY SL-238 PHASE-07 (design §6 "`--prune`'s probe, collapsed", fourth
/// consequence) — DONE. The drop survives; its reason word did not. The three
/// strings the two copies rendered (`absent`, `(unparseable)`,
/// `absent (unparseable ref)`) collapsed to one, `unresolved`, reusing §4's token
/// for that state rather than minting a fourth. The `contains("absent")` assertion
/// this test used to carry is what went red, exactly as PHASE-02 predicted.
///
/// PHASE-02 annotated this under D2: §7's preservation list does not name this
/// SL-105 test, because the design believed `--prune` had no coverage at all
/// (SL-238 `notes.md`, F-1).
#[test]
fn after_prune_absent_target() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 208, "Hotel", "hotel"); // SL-208

    // Manually write an after edge to a non-existent target
    let toml_path = root.join(".doctrine/slice/208/slice-208.toml");
    push_raw_after_edge(&toml_path, "SL-999", 0);

    // Prune
    let prune = run(root, &["after", "SL-208", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    let out = stdout(&prune);
    assert!(
        out.contains("unresolved"),
        "the three reason strings collapsed to one: {out}"
    );

    // Edge is gone
    let toml = fs::read_to_string(&toml_path).unwrap();
    assert!(!toml.contains("SL-999"), "edge to SL-999 removed:\n{toml}");
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
    assert!(out.contains("dropped"), "backlog prune dropped: {out}");
}

/// IMP-140 (F-13): the top-level `needs`/`after` verbs echo the CANONICAL id,
/// matching the backlog path — a non-canonical (unpadded) input normalizes in the
/// echo, so the two write paths no longer diverge.
#[test]
fn top_level_needs_after_echo_canonical_id() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 1, "One", "one"); // SL-001
    seed_slice(root, 2, "Two", "two"); // SL-002

    // Unpadded input `SL-1`/`SL-2` must echo the canonical `SL-001`/`SL-002`.
    let needs = run(root, &["needs", "SL-1", "SL-2"]);
    assert!(needs.status.success(), "needs: {}", stderr(&needs));
    assert_eq!(
        stdout(&needs),
        "SL-001 needs SL-002\n",
        "needs echoes the canonical id, not the raw input"
    );
    // The STORED target is canonical too — echo and storage agree.
    let stored = fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
    assert!(
        stored.contains("needs = [\"SL-002\"]"),
        "stored target is canonicalized, not raw `SL-2`: {stored}"
    );

    let after = run(root, &["after", "SL-1", "SL-2"]);
    assert!(after.status.success(), "after: {}", stderr(&after));
    assert_eq!(
        stdout(&after),
        "SL-001 after SL-002\n",
        "after echoes the canonical id, not the raw input"
    );
}

// --- SL-238 PHASE-02: characterisation ---------------------------------------
//
// Pins of TODAY's behaviour, added BEFORE the legs that produce it are changed
// (design §7 "Characterisation first"), so PHASE-06/07/08 land as a visible change
// rather than as a test that was always going to pass. Every test below names the
// phase that supersedes it and the design section authorising the change (EX-4).
//
// What is deliberately NOT re-pinned: `resolved` and `absent` already have SL-105
// goldens above. These add the four behaviours those goldens miss — the `closed`
// vocabulary, the `/resolution` reason suffix, the silent keep on an unreadable
// target, and the bare ref deleted without the target being read at all.

/// SUPERSEDED BY SL-238 PHASE-07 — DONE. Was
/// `after_prune_pins_the_closed_vocabulary_and_the_resolution_suffix`, which pinned
/// `dropped (dangling: closed/wont-do)`.
///
/// **Both halves of that assertion are gone, and for the same reason.** The old
/// probe hardcoded `resolved || closed` and applied it to every kind, then re-read
/// the raw toml to append a `/resolution` suffix. §6 routes terminality through
/// `partition::authored_class` onto each kind's OWN table, and `closed` is not in
/// ADR-009's slice vocabulary — so a slice carrying it is `Unrecognised`, and
/// `Unrecognised` keeps the edge (§6, second consequence). The suffix went with the
/// re-read: `Meta` carries no `resolution` field, and adding one to decorate a
/// repair message was not the trade (§6, third consequence).
///
/// So the test now asserts the *opposite verdict* on the same fixture, which is a
/// sharper statement of what this phase fixed than any new fixture would be: the
/// cross-kind vocabulary leak WAS the STD-001 violation.
///
/// The source is typed UNPADDED (`SL-9`) and echoes CANONICALLY (`SL-009`), which is
/// PHASE-07 `T3` and supersedes PHASE-02's pin of the opposite. The divergence it
/// pinned — this copy echoing as typed where the *append* verb two tests above and
/// `backlog after --prune` both echo canonically — could not be preserved: PHASE-08
/// `EX-3` routes `backlog after --prune` through this very function, and `EX-6`
/// requires the routed legs to echo the canonical source id. Preserving as-typed
/// would have regressed backlog's existing canonical echo (`notes.md`, `D-3`).
#[test]
fn after_prune_no_longer_applies_backlog_vocabulary_to_a_slice() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 9, "Nine", "nine"); // SL-009 — the source
    seed_slice(root, 10, "Ten", "ten"); // SL-010 — the target

    let append = run(root, &["after", "SL-9", "SL-10"]);
    assert!(append.status.success(), "append: {}", stderr(&append));

    // `closed/wont-do` — the exact fixture the old pin used, kept deliberately.
    // These are BACKLOG words. A slice authoring them is out-of-vocabulary, and the
    // point is that the probe now says so instead of pruning on them.
    set_entity_fields(
        &root.join(".doctrine/slice/010/slice-010.toml"),
        "closed",
        Some("wont-do"),
    );

    let prune = run(root, &["after", "SL-9", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "SL-009: nothing to prune\n",
        "`closed` is Unrecognised for a SLICE — conservative, so the edge is kept"
    );
    assert!(
        slice_toml(root, 9).contains("{ to = \"SL-010\", rank = 0 }"),
        "the edge survives: {}",
        slice_toml(root, 9)
    );
}

// `after_prune_pins_the_silent_keep_on_an_unreadable_target` (SL-238 PHASE-02) was
// REMOVED here by PHASE-07, not left failing. It pinned `stderr == ""` — the defect,
// not an endorsed behaviour — and its replacement is
// `after_prune_keeps_an_unreadable_target_and_says_so_on_stderr` below, which
// asserts the surviving half (the KEEP) and the changed half (the disclosure, EX-3
// / STD-003) on the same fixture. Two tests over one behaviour is duplication; the
// replacement's doc names what it supersedes.

// `after_prune_pins_a_bare_ref_dropped_without_reading_the_target` (SL-238 PHASE-02)
// was REMOVED here by PHASE-07, not left failing. Every assertion in it went red by
// design: the probe moved to `parse_resolvable_ref`, so a bare ref resolves and is
// judged on its target's status. Its replacement is
// `after_prune_resolves_a_bare_ref_instead_of_dropping_it` below, which covers both
// directions — bare-onto-live survives, bare-onto-terminal is pruned with the
// ordinary reason — where the old pin could only assert the first.

/// SUPERSEDED BY PHASE-07 + PHASE-08 (design §6 "`--prune`'s probe, collapsed";
/// PHASE-08/EX-3 removes `backlog.rs`'s duplicate leg outright, so this test is
/// removed WITH the code it pins, not left failing).
///
/// The paired top-level pin is
/// `after_prune_pins_the_closed_vocabulary_and_the_resolution_suffix`. Same
/// unpadded source (`ISS-1`), and the echo diverges: this copy renders
/// `target.0.canonical_id(..)`, so it prints `ISS-001`. That divergence is
/// PRESERVED across the collapse — do not align the two lines.
#[test]
fn backlog_after_prune_pins_the_closed_vocabulary_and_the_resolution_suffix() {
    let t = tmp();
    let root = t.path();
    new_issue(root, "Prune Kilo", "prune-kilo"); // ISS-001 — the source
    new_issue(root, "Prune Lima", "prune-lima"); // ISS-002 — the target

    let append = run(root, &["backlog", "after", "ISS-1", "ISS-002"]);
    assert!(append.status.success(), "append: {}", stderr(&append));
    set_entity_fields(&backlog_toml(root, "issue", 2), "closed", Some("wont-do"));

    let prune = run(root, &["backlog", "after", "ISS-1", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "ISS-001 after ISS-002 (rank 0) dropped (dangling: closed/wont-do)\n",
        "today: same reason wording as the top-level copy, but the echo is CANONICAL"
    );
}

/// SUPERSEDED BY PHASE-07 + PHASE-08 — see
/// `after_prune_pins_the_silent_keep_on_an_unreadable_target` for the behaviour;
/// this is the duplicate leg's copy of it, pinned so the collapse can be shown
/// behaviour-preserving here. Removed with the leg by PHASE-08/EX-3.
#[test]
fn backlog_after_prune_pins_the_silent_keep_on_an_unreadable_target() {
    let t = tmp();
    let root = t.path();
    new_issue(root, "Prune Mike", "prune-mike"); // ISS-001
    new_issue(root, "Prune November", "prune-november"); // ISS-002

    let append = run(root, &["backlog", "after", "ISS-001", "ISS-002"]);
    assert!(append.status.success(), "append: {}", stderr(&append));
    // Corrupt, never delete — a deleted file takes the `absent` arm instead.
    let target = backlog_toml(root, "issue", 2);
    fs::write(&target, "not = = toml\n").unwrap();
    assert!(
        target.exists(),
        "the fixture corrupts the target, never deletes it"
    );

    let prune = run(root, &["backlog", "after", "ISS-001", "--prune"]);
    assert!(prune.status.success(), "prune exit 0: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "ISS-001: nothing to prune\n",
        "the unreadable target is kept, and reported as an ordinary no-op"
    );
    assert_eq!(
        stderr(&prune),
        "",
        "today the keep is SILENT in this copy too"
    );
    let src = fs::read_to_string(backlog_toml(root, "issue", 1)).unwrap();
    assert!(
        src.contains("{ to = \"ISS-002\", rank = 0 }"),
        "the edge survives: {src}"
    );
}

/// SUPERSEDED BY PHASE-07 + PHASE-08 (design §6 "`--prune`'s probe, collapsed",
/// fourth consequence) — the bare ref will resolve to the live `SL-154` and be
/// KEPT. Pinned separately from the top-level copy because the reason wording
/// DIVERGES: this leg renders `(unparseable)` where the top-level one renders
/// `absent (unparseable ref)`. Both collapse to `unresolved`, so the divergence is
/// closed by PHASE-07 — unlike the echo divergence above, which is preserved.
#[test]
fn backlog_after_prune_pins_a_bare_ref_dropped_with_the_divergent_unparseable_reason() {
    let t = tmp();
    let root = t.path();
    new_issue(root, "Prune Oscar", "prune-oscar"); // ISS-001 — the source
    seed_slice(root, 154, "Live", "live"); // SL-154 — a LIVE target

    push_raw_after_edge(&backlog_toml(root, "issue", 1), "154", 0);
    let target_before = slice_toml(root, 154);
    assert!(
        target_before.contains("status = \"proposed\""),
        "the target of the dropped edge is live, not terminal"
    );

    let prune = run(root, &["backlog", "after", "ISS-001", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "ISS-001 after 154 (rank 0) dropped (dangling: (unparseable))\n",
        "today: this copy's unparseable reason is `(unparseable)`, not `absent (unparseable ref)`"
    );
    let src = fs::read_to_string(backlog_toml(root, "issue", 1)).unwrap();
    assert!(
        src.contains("after    = []"),
        "the edge onto a LIVE target is removed: {src}"
    );
    assert_eq!(
        slice_toml(root, 154),
        target_before,
        "the target was never consulted — it is untouched and still live"
    );
}

/// **SL-238 PHASE-06 / `VT-2` — supersedes PHASE-02's
/// `after_remove_pins_the_refusal_of_an_unresolvable_target_leaving_the_edge`.**
///
/// The old test pinned today's behaviour: both refs below were unremovable, for
/// different reasons — `SL-9999` parses but resolves to nothing (refused by the
/// author-time gate's disk probe), `not-a-ref` does not parse at all (refused one
/// tier earlier, by the ref-shape parse). It asserted the refusals AND that both
/// edges survived them, because that is the gap: the refs the doctor check reports
/// at Error severity were exactly the refs `--remove` would not touch, leaving
/// hand-editing the TOML as the only repair path.
///
/// **Why the old assertion no longer holds.** `EX-3` moves both remove paths onto
/// a source-only gate, canonicalising the target through the three-tier needle
/// instead of resolving it. Neither refusal survives that by construction — they
/// were the same gate. design.md §6 calls this "a deliberate behaviour change on
/// `after --remove` ... and it is what makes §5's check repairable"; PHASE-02
/// pinned it precisely so this flip would read as intentional rather than as a
/// regression at audit.
///
/// The `needs` axis gets the same repair through
/// `needs_remove_clears_a_ref_that_does_not_resolve` (in-module), which supersedes
/// nothing — that axis had no removal at all before this phase.
#[test]
fn after_remove_clears_an_unresolvable_target_the_gate_used_to_refuse() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 214, "November", "november"); // SL-214

    // The author-time gate still refuses to WRITE either ref, so they are planted
    // directly — which is the situation this repair path exists for.
    let src = root.join(".doctrine/slice/214/slice-214.toml");
    push_raw_after_edge(&src, "SL-9999", 0);
    push_raw_after_edge(&src, "not-a-ref", 0);

    // (a) well-formed, absent on disk — tier 1 misses, tier 2 canonicalises.
    let rm = run(root, &["after", "SL-214", "SL-9999", "--remove"]);
    assert!(
        rm.status.success(),
        "was refused, now clears: {}",
        stderr(&rm)
    );
    assert_eq!(
        stdout(&rm),
        "SL-214 after SL-9999 removed (1 edge)\n",
        "the canonicalised needle matched the stored ref"
    );

    // (b) free text — tiers 1 and 2 both miss, tier 3 matches it verbatim.
    let rm2 = run(root, &["after", "SL-214", "not-a-ref", "--remove"]);
    assert!(
        rm2.status.success(),
        "free text clears too: {}",
        stderr(&rm2)
    );
    assert_eq!(
        stdout(&rm2),
        "SL-214 after not-a-ref removed (1 edge)\n",
        "a ref that names nothing is still a string that has to come out"
    );

    // The gap is closed: neither edge survives its own removal any more.
    let toml = slice_toml(root, 214);
    assert!(
        !toml.contains("SL-9999") && !toml.contains("not-a-ref"),
        "both previously-unremovable edges are gone:\n{toml}"
    );

    // The AUTHOR-time gate is untouched — this widening is remove-only.
    let append = run(root, &["after", "SL-214", "SL-9999"]);
    assert!(
        !append.status.success(),
        "appending an edge to a non-entity is still refused"
    );
}

/// SL-238 PHASE-06 / `VT-1` — the rendered echo of `needs --remove`.
///
/// The in-module tests in `src/commands/dep_seq.rs` assert the *state* a removal
/// leaves behind; they cannot assert this line, because the unit tests write to
/// `io::stdout()` without capturing it. The count is only observable black-box, so
/// the "reports the count" half of `VT-1` is pinned here.
#[test]
fn needs_remove_echoes_the_canonical_ids_and_the_edge_count() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 215, "Oscar", "oscar"); // SL-215
    seed_slice(root, 216, "Papa", "papa"); // SL-216

    assert!(run(root, &["needs", "SL-215", "SL-216"]).status.success());

    let rm = run(root, &["needs", "SL-215", "SL-216", "--remove"]);
    assert!(rm.status.success(), "remove: {}", stderr(&rm));
    assert_eq!(
        stdout(&rm),
        "SL-215 needs SL-216 removed (1 edge)\n",
        "canonical ids both ends, singular edge"
    );

    // Nothing left to remove — bail, naming both endpoints.
    let again = run(root, &["needs", "SL-215", "SL-216", "--remove"]);
    assert!(!again.status.success(), "second remove should fail");
    assert_eq!(
        stderr(&again),
        "Error: SL-215 has no needs edge to SL-216\n",
        "the refusal mirrors `after --remove`'s"
    );
}

/// SL-238 PHASE-06 / `VT-1` — the plural, and that a non-canonical source input
/// normalises in the echo the same way the append path already does.
#[test]
fn needs_remove_echoes_the_plural_and_normalises_a_bare_source() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 217, "Quebec", "quebec"); // SL-217

    // Two refs to the same target: only reachable by planting, since `append` is
    // idempotent on string membership.
    let src = root.join(".doctrine/slice/217/slice-217.toml");
    push_raw_needs_ref(&src, "SL-9999");
    push_raw_needs_ref(&src, "SL-9999");

    let rm = run(root, &["needs", "217", "SL-9999", "--remove"]);
    assert!(rm.status.success(), "remove: {}", stderr(&rm));
    assert_eq!(
        stdout(&rm),
        "SL-217 needs SL-9999 removed (2 edges)\n",
        "bare source normalises to canonical; plural on 2"
    );
}

// --- SL-238 PHASE-07: the collapsed --prune probe -----------------------------
//
// Red-first against today's tree, EXCEPT where noted. Each names the consequence
// of design §6 "`--prune`'s probe, collapsed" it verifies, and the PHASE-02 pin it
// supersedes. The source is typed CANONICALLY throughout so that T3's echo change
// (`D-3` — `{source}` as typed becomes the canonical id) cannot move these tests:
// they assert the probe, not the prefix.

/// `VT-1` — the first consequence: the deliberate vocabulary change. Terminality
/// stops being the hardcoded `resolved || closed` pair and routes through
/// `partition::authored_class`, so each kind's OWN terminal set applies — a slice's
/// `done`, a question's `answered`.
///
/// RED today: neither word is in the hardcoded pair, so both edges are kept and the
/// verb reports `nothing to prune`.
///
/// Two kinds, not one, on purpose. A single slice fixture would pass against a
/// probe that had merely swapped one hardcoded literal for another; the question is
/// what makes "the table decides" an assertion rather than a hope.
#[test]
fn after_prune_clears_a_done_slice_and_an_answered_question() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 20, "Source", "source"); // SL-020
    seed_slice(root, 21, "Target", "target"); // SL-021
    seed_question(root, 1, "answered"); // QUE-001

    for tgt in ["SL-021", "QUE-001"] {
        let a = run(root, &["after", "SL-020", tgt]);
        assert!(a.status.success(), "append {tgt}: {}", stderr(&a));
    }
    // Terminal AFTER the append, so this cannot be read as the gate refusing a
    // terminal target — the edge was legitimately authored, then the target settled.
    set_entity_status(&root.join(".doctrine/slice/021/slice-021.toml"), "done");

    let prune = run(root, &["after", "SL-020", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "SL-020 after SL-021 (rank 0) dropped (dangling: done)\n\
         SL-020 after QUE-001 (rank 0) dropped (dangling: answered)\n",
        "each kind's own terminal vocabulary, and the reason word carries no /resolution suffix"
    );
    assert!(
        slice_toml(root, 20).contains("after = []"),
        "both terminal edges are gone: {}",
        slice_toml(root, 20)
    );
}

/// `VT-2` — the second consequence, the conservative half: `Workable`, `Gating` and
/// `Unrecognised` all KEEP the edge.
///
/// **Declared: GREEN from the start, not red-first.** Today's probe also keeps all
/// three, for a different reason (none of the statuses is `resolved`/`closed`). It
/// earns its place by pinning the conservative half *across* the collapse — but
/// nobody should read its green as evidence that this phase changed something.
#[test]
fn after_prune_keeps_workable_gating_and_unrecognised_targets() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 30, "Source", "source"); // SL-030
    seed_slice(root, 31, "Live", "live"); // SL-031 — proposed  => Workable
    seed_slice(root, 32, "Odd", "odd"); // SL-032 — see below   => Unrecognised
    seed_question(root, 2, "open"); // QUE-002 — open           => Gating

    for tgt in ["SL-031", "SL-032", "QUE-002"] {
        let a = run(root, &["after", "SL-030", tgt]);
        assert!(a.status.success(), "append {tgt}: {}", stderr(&a));
    }
    // A status in NO partition bucket for its kind — the `Unrecognised` arm. Set
    // after the append for the same reason as VT-1's `done`.
    set_entity_status(
        &root.join(".doctrine/slice/032/slice-032.toml"),
        "not-a-lifecycle-word",
    );

    let prune = run(root, &["after", "SL-030", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "SL-030: nothing to prune\n",
        "a live, a gating and an unclassifiable target are all kept"
    );
    let src = slice_toml(root, 30);
    for tgt in ["SL-031", "SL-032", "QUE-002"] {
        assert!(src.contains(tgt), "{tgt} edge survives: {src}");
    }
}

/// `VT-3` — the second consequence's other half, and `EX-3`: the unreadable target
/// keeps its edge (conservative) **and says why on stderr** (STD-003).
///
/// SUPERSEDES `after_prune_pins_the_silent_keep_on_an_unreadable_target`. The KEEP
/// survives the collapse; the SILENCE does not — that pin asserted `stderr == ""`,
/// which is the defect, not the endorsed behaviour.
///
/// RED today on the stderr half only. The disclosure WORDING is this phase's to
/// author — `EX-3` mandates that it says why, and the design states no string — so
/// it is shaped to mirror the drop line (`... kept (unreadable: ...)` against
/// `... dropped (dangling: ...)`). The error detail carries a tempdir path, so the
/// stable prefix is pinned and the cause is asserted as present, not byte-exact.
#[test]
fn after_prune_keeps_an_unreadable_target_and_says_so_on_stderr() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 40, "Source", "source"); // SL-040
    seed_slice(root, 41, "Corrupt", "corrupt"); // SL-041

    let append = run(root, &["after", "SL-040", "SL-041"]);
    assert!(append.status.success(), "append: {}", stderr(&append));
    // Corrupt, never delete: a deleted target takes the unresolvable arm, which is a
    // different consequence (and VT-4's second half).
    let target = root.join(".doctrine/slice/041/slice-041.toml");
    fs::write(&target, "not = = toml\n").unwrap();
    assert!(target.exists(), "the fixture corrupts, never deletes");

    let prune = run(root, &["after", "SL-040", "--prune"]);
    assert!(prune.status.success(), "prune exit 0: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "SL-040: nothing to prune\n",
        "the unreadable target is KEPT — uncertainty never causes a removal"
    );
    let err = stderr(&prune);
    assert!(
        err.starts_with("SL-040 after SL-041 (rank 0) kept (unreadable:"),
        "the keep is DISCLOSED, naming the edge it declined to judge: {err}"
    );
    assert!(
        slice_toml(root, 40).contains("{ to = \"SL-041\", rank = 0 }"),
        "the edge survives: {}",
        slice_toml(root, 40)
    );
}

/// `VT-4` — the fourth consequence: the probe moves from `parse_canonical_ref`,
/// which rejects the bare form and routes its `Err` to *prunable*, onto
/// `parse_resolvable_ref`, which accepts it. So a bare ref is judged on its
/// target's status like any other ref.
///
/// SUPERSEDES `after_prune_pins_a_bare_ref_dropped_without_reading_the_target`.
///
/// RED today on both halves: today's probe deletes a bare ref either way, without
/// reading the target at all. Both halves are needed — a probe that merely stopped
/// deleting bare refs would pass the first and fail the second.
#[test]
fn after_prune_resolves_a_bare_ref_instead_of_dropping_it() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 50, "Live source", "live-source"); // SL-050
    seed_slice(root, 154, "Live", "live"); // SL-154 — proposed
    seed_slice(root, 51, "Done source", "done-source"); // SL-051
    seed_slice(root, 155, "Settled", "settled"); // SL-155 — set done below

    // Hand-authored: the author-time gate canonicalises, so the bare form can only
    // reach the array by writing it directly.
    push_raw_after_edge(&root.join(".doctrine/slice/050/slice-050.toml"), "154", 0);
    push_raw_after_edge(&root.join(".doctrine/slice/051/slice-051.toml"), "155", 0);
    set_entity_status(&root.join(".doctrine/slice/155/slice-155.toml"), "done");

    // Half 1 — the bare ref RESOLVES to a live target, and survives.
    let live = run(root, &["after", "SL-050", "--prune"]);
    assert!(live.status.success(), "prune exit: {}", stderr(&live));
    assert_eq!(
        stdout(&live),
        "SL-050: nothing to prune\n",
        "a bare ref onto a LIVE target is no longer deleted for being bare"
    );
    assert!(
        slice_toml(root, 50).contains("to = \"154\""),
        "the edge survives verbatim: {}",
        slice_toml(root, 50)
    );

    // Half 2 — the same bare form onto a TERMINAL target is pruned, with the
    // ordinary terminal reason rather than a ref-shape complaint.
    let done = run(root, &["after", "SL-051", "--prune"]);
    assert!(done.status.success(), "prune exit: {}", stderr(&done));
    assert_eq!(
        stdout(&done),
        "SL-051 after 155 (rank 0) dropped (dangling: done)\n",
        "judged on its target's status, and echoed as authored"
    );
    assert!(
        slice_toml(root, 51).contains("after = []"),
        "the terminal edge is gone: {}",
        slice_toml(root, 51)
    );
}

/// `VT-5` / `EX-4`'s third token — the FIFTH consequence, which design §6 does not
/// name (owner accepted 2026-08-17; `notes.md ### Open` carries the reconcile
/// action).
///
/// `authored_class(kind, AuthoredStatus::Absent)` is `Terminal`, and `Absent` is
/// returned for `kinds::STATUS_LESS` — `[REC]`. So an `after` edge onto a `REC`
/// becomes prunable. The reason cannot be the status word, because there is none;
/// it names the CLASS instead.
///
/// RED today: a `REC` toml carries no `status` key, so `unwrap_or("")` matches
/// neither hardcoded literal and the edge is KEPT.
///
/// `REC` is not an admissible `after` target, so only a hand-authored edge reaches
/// this path — which is the same population the bare-ref consequence serves.
#[test]
fn after_prune_clears_a_status_less_target_naming_the_class() {
    let t = tmp();
    let root = t.path();
    seed_slice(root, 60, "Source", "source"); // SL-060
    seed_rec(root, 1); // REC-001 — authors no `status` key at all

    push_raw_after_edge(
        &root.join(".doctrine/slice/060/slice-060.toml"),
        "REC-001",
        0,
    );

    let prune = run(root, &["after", "SL-060", "--prune"]);
    assert!(prune.status.success(), "prune exit: {}", stderr(&prune));
    assert_eq!(
        stdout(&prune),
        "SL-060 after REC-001 (rank 0) dropped (dangling: status-less)\n",
        "the class is named where no status word exists — never an empty reason"
    );
    assert!(
        slice_toml(root, 60).contains("after = []"),
        "the edge is gone: {}",
        slice_toml(root, 60)
    );
}
