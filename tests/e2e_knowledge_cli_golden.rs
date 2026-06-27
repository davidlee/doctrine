//! SL-059 PHASE-03 VT-2 — the `doctrine knowledge` CLI as BLACK-BOX goldens.
//!
//! Pins `knowledge new` / `show` / `list` / `status` at the CLI surface (byte-exact
//! stdout + JSON + error text) over the BUILT binary — the
//! `mem.pattern.testing.black-box-cli-golden` idiom, mirroring `e2e_adr_cli_golden.rs`.
//! The six kinds share one kind-blind engine but diverge in prefix, status vocabulary,
//! and typed `[facet]`; these goldens pin the read/write surface and the FR-002/FR-004
//! prefix→kind routing + foreign-kind refuse.
//!
//! Determinism: `knowledge new`/`status` stamp `clock::today()` into created/updated, so
//! fixtures are hand-seeded with FIXED dates; the two non-deterministic surfaces (the
//! `new` mint and the `status` `updated→today()` bump) are asserted structurally at the
//! documented call sites.

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

/// Hand-seed one record's authored tree (`.doctrine/knowledge/<dir>/NNN/record-NNN.{toml,md}`)
/// — fixed dates make show/list deterministic.
fn seed(root: &Path, dir: &str, id: u32, toml: &str) {
    let d = root.join(format!(".doctrine/knowledge/{dir}/{id:03}"));
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join(format!("record-{id:03}.toml")), toml).unwrap();
    fs::write(d.join(format!("record-{id:03}.md")), "# body\n").unwrap();
}

/// `doctrine knowledge <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .arg("knowledge")
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

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

// --- fixtures -------------------------------------------------------------

/// A populated assumption (facet + evidence) — exercises every show axis.
fn asm007_toml() -> &'static str {
    "schema = \"doctrine.knowledge\"\n\
     version = 1\n\
     \n\
     id = 7\n\
     slug = \"token-expiry\"\n\
     title = \"Token expiry\"\n\
     record_kind = \"assumption\"\n\
     status = \"testing\"\n\
     created = \"2026-01-02\"\n\
     updated = \"2026-01-03\"\n\
     tags = [\"auth\", \"security\"]\n\
     \n\
     # hand-added comment — must survive a status edit\n\
     [facet]\n\
     claim = \"tokens expire in 1h\"\n\
     confidence = \"high\"\n\
     basis = \"observation\"\n\
     validation_plan = \"probe the IdP\"\n\
     validated_by = \"\"\n\
     validated_on = \"\"\n\
     invalidated_by = \"\"\n\
     invalidated_on = \"\"\n\
     \n\
     [evidence]\n\
     supports = [\"DEC-005-C\"]\n\
     contradicts = []\n\
     notes = [\"see the audit\"]\n"
}

/// A minimal record at any kind/status, empty facet+evidence — for list rows + status.
fn minimal_toml(id: u32, slug: &str, title: &str, record_kind: &str, status: &str) -> String {
    format!(
        "schema = \"doctrine.knowledge\"\n\
         version = 1\n\
         \n\
         id = {id}\n\
         slug = \"{slug}\"\n\
         title = \"{title}\"\n\
         record_kind = \"{record_kind}\"\n\
         status = \"{status}\"\n\
         created = \"2026-01-04\"\n\
         updated = \"2026-01-04\"\n\
         tags = []\n\
         \n\
         [facet]\n\
         \n\
         [evidence]\n"
    )
}

// === T1 — `knowledge new` mints in the kind's namespace (structural) =======

#[test]
fn knowledge_new_mints_canonical_id_and_seeds_default_status() {
    let dir = tmp();
    // `new` needs a project-root marker; an empty `.git` dir is enough.
    fs::create_dir_all(dir.path().join(".git")).unwrap();

    let out = run(dir.path(), &["new", "decision", "First decision"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    // The mint stamps today() into created/updated — assert the canonical id + dir
    // structurally (the prefix is load-bearing; the absolute path floats per run).
    let so = stdout(&out);
    assert!(
        so.starts_with("Created DEC-001: "),
        "mint prints the canonical id; got: {so}"
    );
    assert!(so.trim_end().ends_with("/.doctrine/knowledge/decision/001"));

    // The seeded record carries the kind's default status (`proposed`) and the empty
    // `[facet]`/`[evidence]` scaffold (F-A2: the seed == default_status).
    let toml = fs::read_to_string(
        dir.path()
            .join(".doctrine/knowledge/decision/001/record-001.toml"),
    )
    .unwrap();
    assert!(toml.contains("record_kind = \"decision\""));
    assert!(toml.contains("status = \"proposed\""));
    assert!(toml.contains("[facet]") && toml.contains("[evidence]"));
}

// === T2 — `knowledge show` Table golden ===================================

#[test]
fn knowledge_show_table_is_byte_exact() {
    let dir = tmp();
    seed(dir.path(), "assumption", 7, asm007_toml());

    let out = run(dir.path(), &["show", "ASM-007"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ASM-007 — Token expiry\n\
         token-expiry · assumption · testing\n\
         created 2026-01-02 · updated 2026-01-03\n\
         tags: auth, security\n\
         \n\
         [facet]\n\
         \x20\x20claim: tokens expire in 1h\n\
         \x20\x20confidence: high\n\
         \x20\x20basis: observation\n\
         \x20\x20validation_plan: probe the IdP\n\
         \n\
         [evidence]\n\
         \x20\x20supports: DEC-005-C\n\
         \x20\x20notes: see the audit\n\
         \n\
         # body\n"
    );
}

// === T3 — `knowledge show --json` golden ==================================

#[test]
fn knowledge_show_json_is_byte_exact() {
    let dir = tmp();
    seed(dir.path(), "assumption", 7, asm007_toml());

    let out = run(dir.path(), &["show", "ASM-007", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    // Pretty JSON, BTreeMap key order (serde_json sorts), NO trailing newline
    // (`write!`, not `writeln!`). Absent optional facet fields render as `null`.
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"knowledge\",\n  \"knowledge\": {\n    \"body\": \"# body\\n\",\n    \"created\": \"2026-01-02\",\n    \"evidence\": {\n      \"contradicts\": [],\n      \"notes\": [\n        \"see the audit\"\n      ],\n      \"supports\": [\n        \"DEC-005-C\"\n      ]\n    },\n    \"facet\": {\n      \"basis\": \"observation\",\n      \"claim\": \"tokens expire in 1h\",\n      \"confidence\": \"high\",\n      \"invalidated_by\": null,\n      \"invalidated_on\": null,\n      \"validated_by\": null,\n      \"validated_on\": null,\n      \"validation_plan\": \"probe the IdP\"\n    },\n    \"id\": \"ASM-007\",\n    \"record_kind\": \"assumption\",\n    \"relationships\": {\n      \"disputes\": [],\n      \"governed_by\": [],\n      \"shapes\": [],\n      \"spawns\": [],\n      \"supports\": []\n    },\n    \"slug\": \"token-expiry\",\n    \"status\": \"testing\",\n    \"tags\": [\n      \"auth\",\n      \"security\"\n    ],\n    \"title\": \"Token expiry\",\n    \"updated\": \"2026-01-03\"\n  }\n}"
    );
}

// === T4 — `knowledge show` prefix→kind routing + error goldens ============

#[test]
fn knowledge_show_routes_each_prefix_to_its_kind() {
    // FR-004: ASM/DEC/QUE/CON/EVD/HYP each route `show` to the right tree. The six counters
    // are independent — id 1 in each kind is a distinct record.
    let dir = tmp();
    seed(
        dir.path(),
        "assumption",
        1,
        &minimal_toml(1, "a", "An A", "assumption", "held"),
    );
    seed(
        dir.path(),
        "decision",
        1,
        &minimal_toml(1, "d", "A D", "decision", "accepted"),
    );
    seed(
        dir.path(),
        "question",
        1,
        &minimal_toml(1, "q", "A Q", "question", "open"),
    );
    seed(
        dir.path(),
        "constraint",
        1,
        &minimal_toml(1, "c", "A C", "constraint", "active"),
    );
    seed(
        dir.path(),
        "evidence",
        1,
        &minimal_toml(1, "e", "An E", "evidence", "captured"),
    );
    seed(
        dir.path(),
        "hypothesis",
        1,
        &minimal_toml(1, "h", "A H", "hypothesis", "proposed"),
    );

    for (reference, kind, status) in [
        ("ASM-001", "assumption", "held"),
        ("DEC-001", "decision", "accepted"),
        ("QUE-001", "question", "open"),
        ("CON-001", "constraint", "active"),
        ("EVD-001", "evidence", "captured"),
        ("HYP-001", "hypothesis", "proposed"),
    ] {
        let out = run(dir.path(), &["show", reference]);
        assert!(out.status.success(), "{reference} stderr: {}", stderr(&out));
        let so = stdout(&out);
        assert!(
            so.contains(&format!("· {kind} · {status}")),
            "{reference} routed to {kind}: {so}"
        );
    }
}

#[test]
fn knowledge_show_garbage_ref_errors_with_exact_text() {
    let dir = tmp();
    // `ASM-x` splits on the last `-`: prefix `ASM` resolves, but the tail `x` is not a
    // numeric id — the numeric-parse error.
    let out = run(dir.path(), &["show", "ASM-x"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: `x` is not a numeric id in `ASM-x`\n\
         \n\
         Caused by:\n\
         \x20\x20\x20\x20invalid digit found in string\n"
    );
}

#[test]
fn knowledge_show_unknown_prefix_errors() {
    let dir = tmp();
    let out = run(dir.path(), &["show", "REQ-001"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: unknown record prefix `REQ` in `REQ-001` (expected ASM/DEC/QUE/CON/EVD/HYP)\n"
    );
}

// === T4b — `knowledge inspect` goldens (no body) =========================

#[test]
fn knowledge_inspect_table_is_byte_exact() {
    let dir = tmp();
    seed(dir.path(), "assumption", 7, asm007_toml());

    let out = run(dir.path(), &["inspect", "ASM-007"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ASM-007 — Token expiry\n\
         token-expiry · assumption · testing\n\
         created 2026-01-02 · updated 2026-01-03\n\
         tags: auth, security\n\
         \n\
         [facet]\n\
         \x20\x20claim: tokens expire in 1h\n\
         \x20\x20confidence: high\n\
         \x20\x20basis: observation\n\
         \x20\x20validation_plan: probe the IdP\n\
         \n\
         [evidence]\n\
         \x20\x20supports: DEC-005-C\n\
         \x20\x20notes: see the audit\n"
    );
}

#[test]
fn knowledge_inspect_json_omits_body() {
    let dir = tmp();
    seed(dir.path(), "assumption", 7, asm007_toml());

    let out = run(dir.path(), &["inspect", "ASM-007", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let so = stdout(&out);
    // The JSON must NOT contain `"body"` — inspect is metadata-only.
    assert!(
        !so.contains("\"body\""),
        "inspect --json must not leak body: {so}"
    );
    // Sanity: it must still have the identity fields.
    assert!(so.contains("\"id\""), "inspect --json missing id: {so}");
    assert!(
        so.contains("\"title\""),
        "inspect --json missing title: {so}"
    );
}

// === T5 — `knowledge status` goldens (no resolution coupling) =============

#[test]
fn knowledge_status_transition_prints_exact_and_preserves_edits() {
    let dir = tmp();
    seed(dir.path(), "assumption", 7, asm007_toml());

    let out = run(dir.path(), &["status", "ASM-007", "validated"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "ASM-007: validated\n");

    let after = fs::read_to_string(
        dir.path()
            .join(".doctrine/knowledge/assumption/007/record-007.toml"),
    )
    .unwrap();
    assert!(after.contains("status = \"validated\""), "status flipped");
    assert!(
        after.contains("created = \"2026-01-02\""),
        "created untouched"
    );
    // `updated` bumps to today() — assert it MOVED off the seeded value.
    assert!(
        !after.contains("updated = \"2026-01-03\""),
        "updated bumped"
    );
    // Edit-preservation (toml_edit in place): comment + facet + evidence survive.
    assert!(after.contains("# hand-added comment"), "comment preserved");
    assert!(
        after.contains("claim = \"tokens expire in 1h\""),
        "facet preserved"
    );
    assert!(
        after.contains("supports = [\"DEC-005-C\"]"),
        "evidence preserved"
    );
}

#[test]
fn knowledge_status_refuses_a_foreign_kind_state() {
    // FR-002: `accepted` is a DECISION status; on an ASM it is out-of-vocab — refused.
    let dir = tmp();
    seed(dir.path(), "assumption", 7, asm007_toml());
    let path = dir
        .path()
        .join(".doctrine/knowledge/assumption/007/record-007.toml");
    let before = fs::read_to_string(&path).unwrap();

    let out = run(dir.path(), &["status", "ASM-007", "accepted"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: `accepted` is not a assumption status (known: held, testing, validated, invalidated, obsolete)\n"
    );
    // refused → file untouched.
    assert_eq!(fs::read_to_string(&path).unwrap(), before, "file untouched");
}

#[test]
fn knowledge_status_no_op_writes_nothing() {
    let dir = tmp();
    seed(
        dir.path(),
        "question",
        1,
        &minimal_toml(1, "q", "A Q", "question", "open"),
    );
    let path = dir
        .path()
        .join(".doctrine/knowledge/question/001/record-001.toml");
    let before = fs::read_to_string(&path).unwrap();

    let out = run(dir.path(), &["status", "QUE-001", "open"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "QUE-001: open\n");
    // no-op guard: identical status writes nothing — bytes hold exactly.
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn knowledge_status_on_malformed_toml_refuses_and_leaves_file_untouched() {
    // Hand-stripped `status`/`updated` → a tail `insert` would land AFTER the trailing
    // `[facet]`/`[evidence]` header, inside that subtable (silent corruption).
    // set_record_status must REFUSE, not append (mirrors the adr/standard guard goldens).
    let dir = tmp();
    let toml = "schema = \"doctrine.knowledge\"\n\
                version = 1\n\
                \n\
                id = 50\n\
                slug = \"bad\"\n\
                title = \"Bad\"\n\
                record_kind = \"assumption\"\n\
                created = \"2026-01-01\"\n\
                tags = []\n\
                \n\
                [facet]\n\
                \n\
                [evidence]\n";
    seed(dir.path(), "assumption", 50, toml);
    let path = dir
        .path()
        .join(".doctrine/knowledge/assumption/050/record-050.toml");

    let out = run(dir.path(), &["status", "ASM-050", "validated"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: malformed record 050: missing seeded `status`/`updated` — restore the missing keys and retry; the file is left untouched\n"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), toml, "file untouched");
}

// === T6 — `knowledge list` goldens (cross-kind, hide-set, shared token) ===

/// Seed a six-record cross-kind corpus spanning visible + hidden (settled) states,
/// including the SHARED `superseded`/`obsolete` tokens across two kinds each.
fn seed_list_corpus(root: &Path) {
    seed(
        root,
        "assumption",
        1,
        &minimal_toml(1, "alpha", "Alpha", "assumption", "held"),
    );
    seed(
        root,
        "assumption",
        2,
        &minimal_toml(2, "beta", "Beta", "assumption", "obsolete"),
    );
    seed(
        root,
        "decision",
        1,
        &minimal_toml(1, "choose", "Choose", "decision", "accepted"),
    );
    seed(
        root,
        "decision",
        2,
        &minimal_toml(2, "oldway", "Old Way", "decision", "superseded"),
    );
    seed(
        root,
        "constraint",
        1,
        &minimal_toml(1, "limit", "Limit", "constraint", "superseded"),
    );
    seed(
        root,
        "question",
        1,
        &minimal_toml(1, "howmany", "How Many", "question", "open"),
    );
}

#[test]
fn knowledge_list_table_default_hides_settled_byte_exact() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    // Cross-kind, grouped by (kind ordinal, id); settled states (obsolete/superseded)
    // ABSENT; held/accepted/open visible.
    assert_eq!(
        stdout(&out),
        "id      │ kind       │ status   │ title\n\
         ASM-001 │ assumption │ held     │ Alpha\n\
         DEC-001 │ decision   │ accepted │ Choose\n\
         QUE-001 │ question   │ open     │ How Many\n"
    );
}

#[test]
fn knowledge_list_all_reveals_settled_rows() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list", "--all"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ kind       │ status     │ title\n\
         ASM-001 │ assumption │ held       │ Alpha\n\
         ASM-002 │ assumption │ obsolete   │ Beta\n\
         DEC-001 │ decision   │ accepted   │ Choose\n\
         DEC-002 │ decision   │ superseded │ Old Way\n\
         QUE-001 │ question   │ open       │ How Many\n\
         CON-001 │ constraint │ superseded │ Limit\n"
    );
}

#[test]
fn knowledge_list_shared_status_token_spans_kinds() {
    // VT-4: a cross-kind `--status` on a SHARED token (`superseded`) returns items
    // across kinds (DEC + CON), AND reveals the hide-set (explicit `--status` reveals).
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list", "-s", "superseded"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ kind       │ status     │ title\n\
         DEC-002 │ decision   │ superseded │ Old Way\n\
         CON-001 │ constraint │ superseded │ Limit\n"
    );
}

#[test]
fn knowledge_list_json_default_is_the_shared_envelope_byte_exact() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"knowledge\",\n  \"rows\": [\n    {\n      \"id\": \"ASM-001\",\n      \"record_kind\": \"assumption\",\n      \"slug\": \"alpha\",\n      \"status\": \"held\",\n      \"title\": \"Alpha\"\n    },\n    {\n      \"id\": \"DEC-001\",\n      \"record_kind\": \"decision\",\n      \"slug\": \"choose\",\n      \"status\": \"accepted\",\n      \"title\": \"Choose\"\n    },\n    {\n      \"id\": \"QUE-001\",\n      \"record_kind\": \"question\",\n      \"slug\": \"howmany\",\n      \"status\": \"open\",\n      \"title\": \"How Many\"\n    }\n  ]\n}"
    );
}

#[test]
fn knowledge_list_unknown_status_errors_with_union_vocab() {
    // The `--status` known-set is the UNION of the four vocabs; an out-of-union token
    // is the uniform vocab error (the `listing::validate_statuses` opt-in surface).
    let dir = tmp();
    let out = run(dir.path(), &["list", "-s", "bogus"]);
    assert!(!out.status.success());
    let err = stderr(&out);
    assert!(err.contains("unknown status `bogus`"), "got: {err}");
    // a few union members from distinct kinds must appear in the known-set.
    assert!(err.contains("held") && err.contains("proposed") && err.contains("active"));
}
