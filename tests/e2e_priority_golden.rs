//! SL-047 PHASE-03 — the priority surfaces (`survey`/`next`/`blockers`/`explain` +
//! the `inspect` actionability block) as BLACK-BOX CLI goldens.
//!
//! Pins the operator-facing priority layer at the CLI surface (byte-exact human
//! stdout + `--json` conformance) over the BUILT binary
//! (`mem.pattern.testing.black-box-cli-golden`). Asserts EVERY surface, not just the
//! JSON envelope (`mem.pattern.testing.conformance-asserts-surface-not-just-envelope`).
//!
//! Determinism: every surface reads only authored TOML (no clock / rng / map-order;
//! `BTreeMap`/`BTreeSet` throughout) — a hand-seeded corpus yields byte-exact output.
//! The corpus spans multiple kinds (backlog issue/risk + an Active RV) to exercise
//! cross-kind comparison (VT-3), the workable-but-BLOCKED divergence (VT-1), the
//! transitive chain (VT-2), terminal + promoted exclusion (VT-4), and the structured
//! reasons + `policy_version` stamp (VT-5).

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

/// Write `root/<rel>` with `body`, creating parent dirs.
fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

/// Seed a backlog issue (toml + md) with status, resolution, and a relationships body.
fn seed_issue(root: &Path, id: u32, title: &str, status: &str, resolution: &str, rels: &str) {
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"i{id}\"\ntitle = \"{title}\"\nkind = \"issue\"\n\
             status = \"{status}\"\nresolution = \"{resolution}\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{rels}"
        ),
    );
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
        "b\n",
    );
}

/// Seed a backlog risk (a second backlog kind, for the dep prereq).
fn seed_risk(root: &Path, id: u32, title: &str, status: &str, rels: &str) {
    write(
        root,
        &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"k{id}\"\ntitle = \"{title}\"\nkind = \"risk\"\n\
             status = \"{status}\"\nresolution = \"\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{rels}"
        ),
    );
    write(
        root,
        &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.md"),
        "k\n",
    );
}

/// Seed a review with one OPEN finding ⇒ DERIVED status `active` (the cross-kind
/// workable+unblocked node — VT-3).
fn seed_active_review(root: &Path, id: u32, title: &str, target: &str) {
    write(
        root,
        &format!(".doctrine/review/{id:03}/review-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"rv{id}\"\ntitle = \"{title}\"\n\
             [review]\nfacet = \"reconciliation\"\nraiser = \"a\"\nresponder = \"b\"\n\
             [target]\nref = \"{target}\"\n\
             [[finding]]\nid = \"F-1\"\nstatus = \"open\"\nseverity = \"minor\"\n\
             title = \"t\"\ndetail = \"d\"\n"
        ),
    );
}

/// `doctrine <verb> <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    let mut a: Vec<&str> = args.to_vec();
    a.push("-p");
    let root_s = root.to_str().expect("utf8 path");
    a.push(root_s);
    Command::new(BIN).args(&a).output().expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// The shared multi-kind corpus:
/// - ISS-001 open, `needs RSK-001` → workable but BLOCKED.
/// - ISS-002 open, no prereqs → actionable.
/// - ISS-003 closed → terminal (same-kind omission).
/// - ISS-004 open + `resolution = promoted` → excluded by its own reason (F1).
/// - RSK-001 open → the actionable prereq (a second backlog kind).
/// - RV-001 with an open finding → derived `active` (cross-kind workable+unblocked).
fn seed_corpus(root: &Path) {
    seed_issue(
        root,
        1,
        "Blocked work",
        "open",
        "",
        "needs = [\"RSK-001\"]\n",
    );
    seed_issue(root, 2, "Free work", "open", "", "");
    seed_issue(root, 3, "Done work", "closed", "", "");
    seed_issue(root, 4, "Promoted work", "open", "promoted", "");
    seed_risk(root, 1, "The prereq", "open", "");
    seed_active_review(root, 1, "The review", "SL-001");
}

// === VT-1 / VT-3 / VT-4 — survey human (byte-exact) ======================

/// survey (default): every ELIGIBLE node in importance order — actionable first
/// (score desc, then canonical id), the workable-but-BLOCKED ISS-001 LAST with
/// its badge + direct blocker (the divergence, D10). Terminal ISS-003 and promoted
/// ISS-004 are EXCLUDED. The cross-kind RV-001 (Active) appears (VT-3).
#[test]
fn survey_human_importance_order_blocked_last_terminal_promoted_excluded() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["survey"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ kind │ status │         │ score │ blocker │ title\n\
         ISS-002 │ ISS  │ open   │         │ 0.0   │         │ Free work\n\
         RSK-001 │ RSK  │ open   │         │ 0.0   │         │ The prereq\n\
         RV-001  │ RV   │ active │         │ 0.0   │         │ The review\n\
         ISS-001 │ ISS  │ open   │ BLOCKED │ 0.0   │ RSK-001 │ Blocked work\n"
    );
}

/// survey --all: terminal (ISS-003 closed) and promoted (ISS-004) rows are REVEALED
/// (VT-4) — the complete view, same importance order with the extra rows folded in.
#[test]
fn survey_all_reveals_terminal_and_promoted() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["survey", "--all"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    assert!(
        body.contains("ISS-003 │ ISS  │ closed"),
        "terminal revealed: {body}"
    );
    assert!(
        body.contains("ISS-004 │ ISS  │ open"),
        "promoted revealed: {body}"
    );
}

// === VT-1 — next human: actionable-only, blocked ABSENT ==================

/// next: the ACTIONABLE nodes only, in composed order_key order (D9). The
/// workable-but-BLOCKED ISS-001 is ABSENT (the divergence feature); the promoted
/// ISS-004 and terminal ISS-003 are absent too. RSK-001 shows it unblocks one item.
#[test]
fn next_human_actionable_only_blocked_absent() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["next"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ kind │ status │ score │ unblocks │ title\n\
         ISS-002 │ ISS  │ open   │ 0.0   │ 0        │ Free work\n\
         RSK-001 │ RSK  │ open   │ 0.0   │ 1        │ The prereq\n\
         RV-001  │ RV   │ active │ 0.0   │ 0        │ The review\n"
    );
    // The blocked item is absent from the actionable worklist.
    assert!(
        !stdout(&out).contains("ISS-001"),
        "blocked item absent from next"
    );
}

// === VT-2 — blockers + explain surface the chain; rows direct-only =======

/// blockers ISS-001 (direct): its direct blocked-by is RSK-001; it blocks nothing.
#[test]
fn blockers_direct_byte_exact() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["blockers", "ISS-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ISS-001 — blockers (direct)\n\
         \n\
         blocked by:\n\
         \x20\x20RSK-001\n"
    );
}

/// blockers RSK-001 --transitive: the transitive blocking chain (it blocks ISS-001);
/// it is blocked by nothing. The header annotates the display depth — never reorders.
#[test]
fn blockers_transitive_byte_exact() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["blockers", "RSK-001", "--transitive"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "RSK-001 — blockers (transitive)\n\
         \n\
         blocking:\n\
         \x20\x20ISS-001\n"
    );
}

/// explain ISS-001: the full structured account — eligibility, the blocker chain,
/// and the score breakdown, each from a structured reason.
#[test]
fn explain_human_byte_exact() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["explain", "ISS-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ISS-001 — explain\n\
         \x20\x20eligibility: open → Workable\n\
         \x20\x20blocked by: RSK-001\n\
         \x20\x20score: 0.0 (base 0.0 [value 0.0, risk 0.0], leverage 0.0, optionality 0.0)\n"
    );
}

// === VT-5 — --json stamps policy_version + carries structured reasons =====

/// survey --json: every row surface present (id/title/kind/status/actionability/
/// consequence/blockers/reasons) AND the `policy_version` stamp (D6 / REQ-094).
/// Asserts every surface, not just the envelope.
#[test]
fn survey_json_every_surface_and_policy_version() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["survey", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(v["kind"], "survey");
    assert_eq!(v["policy_version"], "priority.v3");
    let rows = v["rows"].as_array().expect("rows array");
    // ISS-002 leads; every surface present on it.
    let lead = &rows[0];
    assert_eq!(lead["id"], "ISS-002");
    assert_eq!(lead["title"], "Free work");
    assert_eq!(lead["kind"], "ISS");
    assert_eq!(lead["status"], "open");
    assert_eq!(lead["actionability"], "actionable");
    assert_eq!(lead["score"], 0.0);
    assert!(lead["blockers"].is_array(), "blockers surface present");
    assert!(lead["reasons"].is_array(), "reasons surface present");
    // The blocked row carries the structured blocked_by reason + its direct blocker.
    let blocked = rows
        .iter()
        .find(|r| r["id"] == "ISS-001")
        .expect("ISS-001 row");
    assert_eq!(blocked["actionability"], "blocked");
    assert_eq!(blocked["blockers"][0], "RSK-001");
    let has_blocked_by = blocked["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["kind"] == "blocked_by");
    assert!(has_blocked_by, "blocked row carries a blocked_by reason");
}

/// explain --json: every structured reason serialized + the policy stamp.
#[test]
fn explain_json_structured_reasons_and_policy_version() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["explain", "ISS-001", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(v["kind"], "explain");
    assert_eq!(v["policy_version"], "priority.v3");
    assert_eq!(v["id"], "ISS-001");
    assert_eq!(v["eligibility"]["kind"], "eligibility");
    assert_eq!(v["eligibility"]["class"], "Workable");
    assert_eq!(v["blocker_chain"][0]["kind"], "blocked_by");
    assert_eq!(v["blocker_chain"][0]["items"][0], "RSK-001");
    assert!(
        v.get("order_contrib").is_none(),
        "order_contrib field dropped from the explain --json envelope (SL-050 F5)"
    );
    // SL-133 VA-1: the score breakdown exposes every dimension.
    assert_eq!(v["score"]["kind"], "score");
    assert_eq!(v["score"]["base"], 0.0);
    assert_eq!(v["score"]["value_dim"], 0.0);
    assert_eq!(v["score"]["risk_dim"], 0.0);
    assert_eq!(v["score"]["leverage"], 0.0);
    assert_eq!(v["score"]["optionality"], 0.0);
    assert_eq!(v["score"]["total"], 0.0);
}

/// next --json: actionable rows only, with the policy stamp + structured reasons.
#[test]
fn next_json_actionable_only_policy_version() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["next", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(v["kind"], "next");
    assert_eq!(v["policy_version"], "priority.v3");
    let ids: Vec<&str> = v["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["ISS-002", "RSK-001", "RV-001"]);
    assert!(
        !ids.contains(&"ISS-001"),
        "blocked item absent from next --json"
    );
}

// === EX-3 — clean error / empty channels ================================

/// An unknown prefix is a clean non-zero error (never a panic) on a priority verb.
#[test]
fn blockers_unknown_prefix_clean_error() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["blockers", "ZZZ-001"]);
    assert!(!out.status.success(), "unknown prefix must exit non-zero");
    let err = stderr(&out);
    assert!(err.starts_with("Error: "), "clean anyhow error: {err}");
    assert!(err.contains("ZZZ"), "error names the prefix: {err}");
    assert!(!err.contains("panic"), "must not panic: {err}");
}

/// An entity with no relations / no prereqs yields empty channels, not an error
/// (EX-3): explain over the unblocked ISS-002 shows an empty blocker chain.
#[test]
fn explain_unblocked_entity_empty_channels_not_error() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["explain", "ISS-002", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(v["blocker_chain"].as_array().unwrap().len(), 0);
    assert_eq!(v["eligibility"]["class"], "Workable");
}

// === SL-050 F6 — keyed-surface existence gate ===========================

/// The exact existence-gate failure: a well-formed but never-minted id exits non-zero
/// with EXACTLY `SL-999: no such entity` on stderr, no stdout.
fn assert_no_such_entity(out: &Output, expected_ref: &str) {
    assert!(
        !out.status.success(),
        "a never-minted id must exit non-zero"
    );
    let err = stderr(out);
    assert!(err.starts_with("Error: "), "clean anyhow error: {err}");
    let msg = format!("{expected_ref}: no such entity");
    assert!(
        err.contains(&msg),
        "exact existence-gate message ({msg}): {err}"
    );
    assert!(!err.contains("panic"), "must not panic: {err}");
    assert!(
        stdout(out).is_empty(),
        "no partial output on the error path"
    );
}

/// VT-1/VT-3 — `explain` over a never-minted id errors with the existence message,
/// instead of explaining a phantom node.
#[test]
fn explain_nonexistent_id_is_no_such_entity_error() {
    let dir = tmp();
    seed_corpus(dir.path());
    let out = run(dir.path(), &["explain", "SL-999"]);
    assert_no_such_entity(&out, "SL-999");
    // The same under --json (the gate fires before any rendering).
    let out = run(dir.path(), &["explain", "SL-999", "--json"]);
    assert_no_such_entity(&out, "SL-999");
}

/// VT-1/VT-3 — `blockers` over a never-minted id errors with the existence message,
/// instead of rendering empty blocked-by / blocking lists.
#[test]
fn blockers_nonexistent_id_is_no_such_entity_error() {
    let dir = tmp();
    seed_corpus(dir.path());
    let out = run(dir.path(), &["blockers", "SL-999"]);
    assert_no_such_entity(&out, "SL-999");
    // --transitive errors identically.
    let out = run(dir.path(), &["blockers", "SL-999", "--transitive"]);
    assert_no_such_entity(&out, "SL-999");
}

/// VT-1 — `inspect` over a never-minted id errors with the existence message (the
/// appended actionability block is never reached). The relation-golden suite pins the
/// human stdout; this confirms the priority-verb corpus errors identically.
#[test]
fn inspect_nonexistent_id_is_no_such_entity_error() {
    let dir = tmp();
    seed_corpus(dir.path());
    let out = run(dir.path(), &["inspect", "SL-999"]);
    assert_no_such_entity(&out, "SL-999");
    let out = run(dir.path(), &["inspect", "SL-999", "--json"]);
    assert_no_such_entity(&out, "SL-999");
}

// === inspect — the appended actionability block ==========================

/// inspect ISS-001: the relation view (here `(no relations)`) with the actionability
/// block appended below (SL-046 D1). The relation portion stays byte-identical; the
/// block is purely additive.
#[test]
fn inspect_appends_actionability_block_human() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["inspect", "ISS-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ISS-001 — relations\n\
         \n\
         (no relations)\n\
         \n\
         actionability:\n\
         \x20\x20eligible: true\n\
         \x20\x20actionable: false\n\
         \x20\x20score: 0.0\n\
         \x20\x20blocked by: RSK-001\n"
    );
}

/// inspect --json: the relation envelope with an additive `actionability` key — the
/// relation surfaces (outbound/inbound/danglers) unchanged.
#[test]
fn inspect_json_additive_actionability_key() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["inspect", "ISS-001", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    // Relation surfaces unchanged.
    assert_eq!(v["kind"], "inspect");
    assert_eq!(v["id"], "ISS-001");
    assert!(v["outbound"].is_array());
    assert!(v["inbound"].is_array());
    assert!(v["danglers"].is_array());
    // The additive actionability block.
    assert_eq!(v["actionability"]["eligible"], true);
    assert_eq!(v["actionability"]["actionable"], false);
    assert_eq!(v["actionability"]["blockers"][0], "RSK-001");
    assert_eq!(v["actionability"]["score"], 0.0);
}
