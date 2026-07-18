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

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

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
    Command::new(bin())
        .args(&a)
        .output()
        .expect("spawn doctrine")
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
        "id      │ kind │ status │ score │ blocker │ title\n\
         RSK-001 │ RSK  │ open   │ 1.5   │         │ The prereq\n\
         ISS-002 │ ISS  │ open   │ 1.0   │         │ Free work\n\
         RV-001  │ RV   │ active │ 0.0   │         │ The review\n\
         ISS-001 │ ISS  │ open   │ 1.0   │ RSK-001 │ Blocked work\n"
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
/// ISS-004 and terminal ISS-003 are absent too. Default columns (SL-171 PHASE-01):
/// id, status, score, estimate, value, title — kind and unblocks are gone.
#[test]
fn next_human_actionable_only_blocked_absent() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["next"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ status │ score │ estimate │ value │ title\n\
         RSK-001 │ open   │ 1.5   │ ·        │ 1.0*  │ The prereq\n\
         ISS-002 │ open   │ 1.0   │ ·        │ 1.0*  │ Free work\n\
         RV-001  │ active │ 0.0   │ ·        │ ·     │ The review\n"
    );
    // The blocked item is absent from the actionable worklist.
    assert!(
        !stdout(&out).contains("ISS-001"),
        "blocked item absent from next"
    );
}

// === SL-171 PHASE-02 — next pagination at the CLI surface =================
// Actionable order over the shared corpus: ISS-002, RSK-001, RV-001.
// The pure slice/footer/D7 logic is unit-tested in src/priority/render.rs;
// these black-box cases pin the CLI page→offset resolution + --page validation
// that the dispatch arm owns (cli.rs), unreachable from a unit test.

/// `next --limit N` clips the worklist and emits the shared truncation footer.
#[test]
fn next_limit_truncates_with_footer() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["next", "--limit", "2"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    assert!(body.contains("RSK-001"), "page row 1: {body}");
    assert!(body.contains("ISS-002"), "page row 2: {body}");
    assert!(!body.contains("RV-001"), "third row clipped: {body}");
    assert!(body.contains("2 of 3"), "footer count: {body}");
    assert!(body.contains("--page 2"), "footer next-page: {body}");
}

/// `--page N` is exact sugar for `--offset (N-1)*limit` (CLI resolution).
#[test]
fn next_page_resolves_to_offset() {
    let dir = tmp();
    seed_corpus(dir.path());

    let via_page = run(dir.path(), &["next", "--limit", "1", "--page", "2"]);
    let via_offset = run(dir.path(), &["next", "--limit", "1", "--offset", "1"]);
    assert!(
        via_page.status.success(),
        "page stderr: {}",
        stderr(&via_page)
    );
    assert!(
        via_offset.status.success(),
        "offset stderr: {}",
        stderr(&via_offset)
    );
    assert_eq!(
        stdout(&via_page),
        stdout(&via_offset),
        "--page 2 == --offset 1 at limit 1"
    );
    assert!(
        stdout(&via_page).contains("ISS-002"),
        "second actionable row on page 2: {}",
        stdout(&via_page)
    );
}

/// `--page 0` is rejected (1-based).
#[test]
fn next_page_zero_errors() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["next", "--page", "0"]);
    assert!(!out.status.success(), "page 0 must error");
    assert!(
        stderr(&out).contains("--page must be >= 1"),
        "stderr: {}",
        stderr(&out)
    );
}

/// `--limit 0 --page N` is rejected (no page size to resolve against).
#[test]
fn next_limit_zero_page_errors() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["next", "--limit", "0", "--page", "2"]);
    assert!(!out.status.success(), "limit 0 + page must error");
    assert!(
        stderr(&out).contains("--page requires a positive --limit"),
        "stderr: {}",
        stderr(&out)
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
    // SL-218 PHASE-03: ISS-001 is blocked (not actionable) ⇒ off the frontier, so
    // it gets the "not on the current frontier" tension disclosure (design §2).
    assert_eq!(
        stdout(&out),
        "ISS-001 — explain\n\
         \x20\x20eligibility: open → Workable\n\
         \x20\x20blocked by: RSK-001\n\
         \x20\x20score: 1.0 (base 1.0 [value 1.0, risk 0.0], leverage 0.0, optionality 0.0)\n\
         \x20\x20not on the current frontier — no tension analysis\n"
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
    assert_eq!(lead["id"], "RSK-001");
    assert_eq!(lead["title"], "The prereq");
    assert_eq!(lead["kind"], "RSK");
    assert_eq!(lead["status"], "open");
    assert_eq!(lead["actionability"], "actionable");
    assert_eq!(lead["score"], 1.5);
    assert!(lead["blockers"].is_array(), "blockers surface present");
    assert!(lead["reasons"].is_array(), "reasons surface present");
    // The blocked row carries the structured blocked_by reason + its direct blocker.
    let blocked = rows
        .iter()
        .find(|r| r["id"] == "ISS-001")
        .expect("ISS-001 row");
    assert_eq!(blocked["actionability"], "blocked");
    assert_eq!(blocked["blockers"][0], "RSK-001");
    // SL-177 PHASE-02: burndown raw_value retrofit — valueless value-bearing items
    // now denominate (default 1.0) and deliver → score 1.0 (no fulfils r=0, burn=value_dim).
    assert_eq!(blocked["score"], 1.0);
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
    assert_eq!(v["score"]["base"], 1.0);
    assert_eq!(v["score"]["value_dim"], 1.0);
    assert_eq!(v["score"]["risk_dim"], 0.0);
    assert_eq!(v["score"]["leverage"], 0.0);
    assert_eq!(v["score"]["optionality"], 0.0);
    // SL-177 PHASE-02: burndown raw_value retrofit — denominate default 1.0 →
    // valueless items now score 1.0 (no fulfils r=0, burn = value_dim).
    assert_eq!(v["score"]["total"], 1.0);
}

/// next --json: actionable rows only, with the policy stamp + structured reasons.
/// VT-5 (SL-171 PHASE-01): payload is byte-identical to pre-slice — the new
/// NextRow fields (estimate/value/tags) do NOT leak into the JSON.
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
    assert_eq!(ids, vec!["RSK-001", "ISS-002", "RV-001"]);
    assert!(
        !ids.contains(&"ISS-001"),
        "blocked item absent from next --json"
    );
    // VT-5 negative assertion: no estimate/value/tags keys in the JSON payload
    // (catches a future field-add to the json! macro).
    for row in v["rows"].as_array().unwrap() {
        assert!(
            row.get("estimate").is_none(),
            "estimate must not leak into next --json"
        );
        assert!(
            row.get("value").is_none(),
            "value must not leak into next --json"
        );
        assert!(
            row.get("tags").is_none(),
            "tags must not leak into next --json"
        );
    }
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
         \x20\x20score: 1.0\n\
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
    // SL-177 PHASE-02: burndown raw_value retrofit — valueless value-bearing items
    // denominate (default 1.0) → score 1.0 (no fulfils r=0, burn = value_dim).
    assert_eq!(v["actionability"]["score"], 1.0);
}

// =============================================================================
// SL-218 PHASE-03 — tension narrative render (design §3)
// =============================================================================

/// Seed a value-bearing issue with a verbatim `[relationships]` tail (may carry
/// `after`/`needs` + a `[value]` table). The tail is appended after
/// `[relationships]\n`, so it can open new tables (`[value]`).
fn seed_val(root: &Path, id: u32, tail: &str) {
    seed_issue(root, id, &format!("Issue {id}"), "open", "", tail);
}

/// Hand-author a one-row prefer-a comparison session (the wire shape `compare
/// record` mints), `a` preferred over `b` in the value domain, by `rater`.
fn capture(root: &Path, slot: &str, a: &str, b: &str, rater: &str) {
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
        &format!(
            "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
             [session]\nuid = \"sess-{slot}\"\ndate = \"2026-01-01\"\n\n\
             [[judgement]]\nuid = \"row-{slot}\"\nseq = 0\na = \"{a}\"\nb = \"{b}\"\n\
             response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\n\
             form = \"order\"\nrater = \"{rater}\"\ndate = \"2026-01-01\"\n"
        ),
    );
}

/// Hand-author a HUMAN value-anchor claim (SL-220 §1 wire v3: `form = anchor`,
/// `value-anchor` frame, magnitude payload) — the post-deletion way a fixture
/// pins a value magnitude: a `[value]` facet no longer anchors anything
/// (SL-222 PHASE-09).
fn capture_value_anchor(root: &Path, slot: &str, item: &str, magnitude: f64) {
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
        &format!(
            "schema = \"doctrine.comparison-session\"\nversion = 3\n\n\
             [session]\nuid = \"sess-{slot}\"\ndate = \"2026-01-01\"\n\n\
             [[judgement]]\nuid = \"row-{slot}\"\nseq = 0\na = \"{item}\"\n\
             domain = \"value\"\nframe = \"value-anchor\"\nform = \"anchor\"\n\
             magnitude = {magnitude}\nrater = \"human\"\ndate = \"2026-01-01\"\n"
        ),
    );
}

/// Turn the D7 knob on (`.doctrine/doctrine.toml` — the config load path, NOT the
/// project-root `doctrine.toml`).
fn knob_on(root: &Path) {
    write(
        root,
        ".doctrine/doctrine.toml",
        "[priority.compare]\ndemote_agent_evidence = true\n",
    );
}

/// The structure-determined callout (design §3 sample 1) reaches BOTH `next` and
/// `explain` (surfaced + off-page preferred), byte-exact, from a real corpus:
/// ISS-014 (value 5, `after ISS-010`) outranks ISS-010 (value 1) on value_dim, but
/// the surviving `after` sequence surfaces ISS-010 first (VT-1 / VT-2 / VT-E).
#[test]
fn next_structure_tension_reaches_next_and_explain() {
    let dir = tmp();
    let root = dir.path();
    seed_val(root, 10, "[value]\nvalue = 1.0\n");
    seed_val(
        root,
        14,
        "after = [[\"ISS-010\", 0]]\n[value]\nvalue = 5.0\n",
    );
    capture(root, "h1", "ISS-014", "ISS-010", "human");

    let callout = "tension: ISS-014 ranks above ISS-010 on value_dim \
                   (determined — 2 human judgements); ISS-010 surfaces first — \
                   `after ISS-010` sequence survives.";

    // next: the callout rides a trailing block under the page.
    let next = stdout(&run(root, &["next"]));
    assert!(
        next.contains(&format!("\ntensions:\n  {callout}\n")),
        "next tension block: {next}"
    );

    // explain of the SURFACED member (ISS-010) renders its tension after score.
    let ex_surfaced = stdout(&run(root, &["explain", "ISS-010"]));
    assert!(
        ex_surfaced.contains(&format!("  {callout}\n")),
        "{ex_surfaced}"
    );

    // explain of the PREFERRED member (ISS-014) — off the page but on the frontier
    // — renders the same displaced-counterparty tension (F-4 / VT-H).
    let ex_pref = stdout(&run(root, &["explain", "ISS-014"]));
    assert!(ex_pref.contains(&format!("  {callout}\n")), "{ex_pref}");
}

/// `next --json` carries the full structured tension list (design §3 schema:
/// preferred/surfaced/cause/edge/grade/counts) — additive, uncapped (VT-5).
#[test]
fn next_json_carries_structured_tensions() {
    let dir = tmp();
    let root = dir.path();
    seed_val(root, 10, "[value]\nvalue = 1.0\n");
    seed_val(
        root,
        14,
        "after = [[\"ISS-010\", 0]]\n[value]\nvalue = 5.0\n",
    );
    capture(root, "h1", "ISS-014", "ISS-010", "human");

    let out = run(root, &["next", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    let t = &v["tensions"][0];
    assert_eq!(t["cause"], "structure");
    assert_eq!(t["preferred"], "ISS-014");
    assert_eq!(t["surfaced"], "ISS-010");
    assert_eq!(t["edge"]["from"], "ISS-010");
    assert_eq!(t["edge"]["verb"], "after");
    assert_eq!(t["grade"], "determined");
    assert_eq!(t["counts"]["human"], 2);
    assert_eq!(t["counts"]["agent"], 0);
    // Existing row surface is unchanged (lean-JSON reads tolerate the new key).
    assert_eq!(v["rows"][0]["id"], "ISS-010");
    assert!(v["zero_weight"].is_null());
}

/// Composition tensions are `next`-suppressed by default and pulled in by
/// `--verbose` (design D5): ISS-024 (value 2, leverage from unblocking 3) surfaces
/// above ISS-020 (value 3) on full score, but ISS-020 outranks it on value_dim
/// alone. No structural path ⇒ Composition (VT-1 verbose clause).
#[test]
fn next_composition_hidden_by_default_shown_verbose() {
    let dir = tmp();
    let root = dir.path();
    seed_val(root, 20, "");
    seed_val(root, 24, "");
    capture_value_anchor(root, "va20", "ISS-020", 3.0);
    capture_value_anchor(root, "va24", "ISS-024", 2.0);
    for d in [30u32, 31, 32] {
        seed_val(root, d, "needs = [\"ISS-024\"]\n");
    }

    // Default: structure-only ⇒ no tension block at all.
    let default = stdout(&run(root, &["next"]));
    assert!(
        !default.contains("tensions:"),
        "no callout by default: {default}"
    );

    // --verbose pulls the composition callout in.
    let verbose = stdout(&run(root, &["next", "--verbose"]));
    assert!(
        verbose.contains(
            "ISS-024 surfaces above ISS-020 on full score (leverage +1.5); on value_dim \
             alone ISS-020 ranks higher (projected order — no determining evidence)."
        ),
        "verbose composition callout: {verbose}"
    );
    // JSON carries composition regardless of --verbose.
    let v: serde_json::Value =
        serde_json::from_str(&stdout(&run(root, &["next", "--json"]))).expect("json");
    assert_eq!(v["tensions"][0]["cause"], "composition");
    assert_eq!(v["tensions"][0]["deltas"]["leverage"], 1.5);
}

/// The human callout block caps at `TENSION_MAX_CALLOUTS` (3); `--json` carries
/// the full uncapped list (VT-6). Four higher-value items all `after` a single
/// low-value item ⇒ four structure tensions on one page.
#[test]
fn next_tension_callouts_capped_json_uncapped() {
    let dir = tmp();
    let root = dir.path();
    seed_val(root, 1, "");
    capture_value_anchor(root, "va1", "ISS-001", 1.0);
    for d in [2u32, 3, 4, 5] {
        seed_val(root, d, "after = [[\"ISS-001\", 0]]\n");
        capture_value_anchor(
            root,
            &format!("va{d}"),
            &format!("ISS-00{d}"),
            f64::from(d + 5),
        );
    }

    let human = stdout(&run(root, &["next"]));
    let callouts = human.matches("  tension:").count();
    assert_eq!(callouts, 3, "human callouts capped at 3: {human}");

    let v: serde_json::Value =
        serde_json::from_str(&stdout(&run(root, &["next", "--json"]))).expect("json");
    assert_eq!(
        v["tensions"].as_array().expect("array").len(),
        4,
        "JSON tension list uncapped"
    );
}

/// Knob-state wording (design VT-J): an agent-only-determined pair reads
/// `agent-proposed … unconfirmed` with the demotion disclosure when the D7 knob is
/// on; the SAME corpus reads `determined … agent judgements` (T7 disclosure) with
/// the knob off (VT-4).
#[test]
fn tension_grade_tracks_knob_agent_proposed_vs_determined() {
    let dir = tmp();
    let root = dir.path();
    seed_val(root, 10, "");
    seed_val(root, 14, "after = [[\"ISS-010\", 0]]\n");
    capture(root, "a1", "ISS-014", "ISS-010", "agent");

    // Knob OFF: agent evidence determines the order, disclosed (T7 ship posture).
    let off = stdout(&run(root, &["next"]));
    assert!(
        off.contains("(determined — 2 agent judgements)"),
        "knob-off determined: {off}"
    );

    // Knob ON: the human system cannot retire it ⇒ agent-proposed, unconfirmed.
    knob_on(root);
    let on = stdout(&run(root, &["explain", "ISS-010"]));
    assert!(
        on.contains("(agent-proposed — 2 agent judgements, unconfirmed)"),
        "knob-on agent-proposed: {on}"
    );
    assert!(
        on.contains("agent evidence demoted:"),
        "knob-on disclosure present: {on}"
    );
}

/// Cross-surface agreement (design VT-I / F-1/F-7): grades and the elicit queue
/// never disagree about the same pair. Knob-off, the agent-determined pair is
/// retired (`next` grades it `determined`, `compare elicit` offers nothing); knob-on
/// it is `agent-proposed` and re-enters the elicit queue as a live candidate.
#[test]
fn tension_grade_agrees_with_elicit_queue_both_knob_states() {
    let dir = tmp();
    let root = dir.path();
    seed_val(root, 10, "");
    seed_val(root, 14, "after = [[\"ISS-010\", 0]]\n");
    capture(root, "a1", "ISS-014", "ISS-010", "agent");

    // Knob off: determined ⇒ elicit retires it (no candidate for the pair).
    let next_off = stdout(&run(root, &["next", "--json"]));
    let v: serde_json::Value = serde_json::from_str(&next_off).expect("json");
    assert_eq!(v["tensions"][0]["grade"], "determined");
    let elicit_off = stdout(&run(root, &["compare", "elicit"]));
    assert!(
        !elicit_off.contains("ISS-014 vs ISS-010") && !elicit_off.contains("ISS-010 vs ISS-014"),
        "knob-off: determined pair NOT offered: {elicit_off}"
    );

    // Knob on: agent-proposed ⇒ elicit offers the pair (human confirmation pressure).
    knob_on(root);
    let v_on: serde_json::Value =
        serde_json::from_str(&stdout(&run(root, &["next", "--json"]))).expect("json");
    assert_eq!(v_on["tensions"][0]["grade"], "agent_proposed");
    let elicit_on = stdout(&run(root, &["compare", "elicit"]));
    assert!(
        elicit_on.contains("ISS-014 vs ISS-010") || elicit_on.contains("ISS-010 vs ISS-014"),
        "knob-on: agent-proposed pair offered: {elicit_on}"
    );
}

/// A minted-but-non-actionable id (blocked ⇒ off the frontier) gets the "not on
/// the current frontier" disclosure, never invented tensions (design §2 / VT-H).
#[test]
fn explain_off_frontier_id_gets_disclosure() {
    let dir = tmp();
    let root = dir.path();
    seed_risk(root, 1, "Prereq", "open", "");
    seed_val(root, 10, "needs = [\"RSK-001\"]\n"); // blocked ⇒ not actionable

    let out = stdout(&run(root, &["explain", "ISS-010"]));
    assert!(
        out.contains("  not on the current frontier — no tension analysis\n"),
        "off-frontier disclosure: {out}"
    );
    assert!(!out.contains("tension:"), "no invented tensions: {out}");
}
