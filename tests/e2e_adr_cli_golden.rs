//! SL-030 PHASE-01 — the behaviour-preservation gate as BLACK-BOX goldens.
//!
//! The migration in PHASE-02 extracts adr.rs's read/show/status logic onto a
//! shared `governance.rs` spine. The existing adr unit tests poke the very symbols
//! that move and write to `io::stdout()` WITHOUT capturing it (Codex MAJOR-6) — so
//! they prove helper self-consistency, not that the CLI surface is unchanged. These
//! tests pin `adr show` / `adr status` / `adr list` at the CLI surface (byte-exact
//! stdout + JSON + error text) over the BUILT binary. PHASE-02 holds them green
//! UNCHANGED; that is the proof the extraction is behaviour-identical.
//!
//! Determinism: `adr new`/`adr status` stamp `clock::today()` into created/updated,
//! so fixtures are hand-seeded with FIXED dates. Two carve-outs are asserted
//! structurally rather than byte-exact (documented at each call site): the absolute
//! tempdir path in a "not found" error, and the `updated→today()` bump a real
//! `status` transition writes.

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

/// Hand-seed one ADR's authored tree (`.doctrine/adr/NNN/adr-NNN.{toml,md}`) with
/// the caller's exact toml + md bytes — fixed dates make show/list deterministic.
fn seed(root: &Path, id: u32, toml: &str, md: &str) {
    let dir = root.join(format!(".doctrine/adr/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("adr-{id:03}.toml")), toml).unwrap();
    fs::write(dir.join(format!("adr-{id:03}.md")), md).unwrap();
}

/// `doctrine adr <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("adr")
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

// --- fixtures -------------------------------------------------------------

/// ADR-001: accepted, a non-empty `related` + `tags` axis, and a hand-added
/// comment — exercises the relationships block (show) and edit-preservation (status).
fn adr001_toml() -> &'static str {
    "id = 1\n\
     slug = \"use-rust\"\n\
     title = \"Use Rust\"\n\
     status = \"accepted\"\n\
     created = \"2026-01-02\"\n\
     updated = \"2026-01-03\"\n\
     \n\
     # hand-added comment — must survive a status edit\n\
     [relationships]\n\
     supersedes = []\n\
     superseded_by = []\n\
     tags = [\"lang\"]\n\
     \n\
     [[relation]]\n\
     label = \"related\"\n\
     target = \"ADR-002\"\n"
}
fn adr001_md() -> &'static str {
    "# ADR-001: Use Rust\n\nbody text here.\n"
}

/// A minimal ADR at `status`, no relationships — for list rows + status flips.
fn minimal_toml(id: u32, slug: &str, title: &str, status: &str) -> String {
    format!(
        "id = {id}\n\
         slug = \"{slug}\"\n\
         title = \"{title}\"\n\
         status = \"{status}\"\n\
         created = \"2026-01-04\"\n\
         updated = \"2026-01-04\"\n\
         \n\
         [relationships]\n\
         supersedes = []\n\
         superseded_by = []\n\
         related = []\n\
         tags = []\n"
    )
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

// === T2 — `adr show` Table golden ========================================

#[test]
fn adr_show_table_is_byte_exact() {
    let dir = tmp();
    seed(dir.path(), 1, adr001_toml(), adr001_md());

    let out = run(dir.path(), &["show", "ADR-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "ADR-001 — Use Rust\n\
         use-rust · accepted\n\
         created 2026-01-02 · updated 2026-01-03\n\
         \n\
         relationships:\n\
         \x20\x20related: ADR-002\n\
         \x20\x20tags: lang\n\
         \n\
         # ADR-001: Use Rust\n\
         \n\
         body text here.\n"
    );
}

// === T3 — `adr show --json` golden =======================================

#[test]
fn adr_show_json_is_byte_exact() {
    let dir = tmp();
    seed(dir.path(), 1, adr001_toml(), adr001_md());

    let out = run(dir.path(), &["show", "ADR-001", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    // Pretty JSON, BTreeMap key order (no `preserve_order`), NO trailing newline
    // (`write!`, not `writeln!`). The dynamic stem key (`"adr"`) is the one
    // PHASE-02's hand-built serde_json::Map must reproduce.
    assert_eq!(
        stdout(&out),
        "{\n  \"adr\": {\n    \"created\": \"2026-01-02\",\n    \"id\": 1,\n    \"relationships\": {\n      \"related\": [\n        \"ADR-002\"\n      ],\n      \"superseded_by\": [],\n      \"supersedes\": [],\n      \"tags\": [\n        \"lang\"\n      ]\n    },\n    \"slug\": \"use-rust\",\n    \"status\": \"accepted\",\n    \"title\": \"Use Rust\",\n    \"updated\": \"2026-01-03\"\n  },\n  \"body\": \"# ADR-001: Use Rust\\n\\nbody text here.\\n\",\n  \"kind\": \"adr\"\n}"
    );
}

// === T4 — `adr show` error goldens =======================================

#[test]
fn adr_show_garbage_ref_errors_with_exact_text() {
    let dir = tmp();
    seed(dir.path(), 1, adr001_toml(), adr001_md());

    let out = run(dir.path(), &["show", "not-an-adr"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: not an ADR reference: `not-an-adr` (expected `ADR-007` or `7`)\n\
         \n\
         Caused by:\n\
         \x20\x20\x20\x20invalid digit found in string\n"
    );
}

#[test]
fn adr_show_missing_id_errors_with_stable_text() {
    let dir = tmp();
    seed(dir.path(), 1, adr001_toml(), adr001_md());

    let out = run(dir.path(), &["show", "ADR-099"]);
    assert!(!out.status.success());
    let err = stderr(&out);
    // Carve-out 1: the absolute tempdir path floats per run, so pin the stable
    // prefix + the relative path suffix + the source line — the migrating TEXT.
    assert!(
        err.starts_with("Error: adr 099 not found at "),
        "got: {err}"
    );
    assert!(
        err.contains("/.doctrine/adr/099/adr-099.toml\n\nCaused by:\n    No such file or directory (os error 2)\n"),
        "got: {err}"
    );
}

// === T5 — `adr status` goldens ===========================================

#[test]
fn adr_status_transition_prints_exact_and_preserves_edits() {
    let dir = tmp();
    // Seed at `proposed` with a comment + non-empty rels to prove edit-preservation.
    let toml = adr001_toml().replace("status = \"accepted\"", "status = \"proposed\"");
    seed(dir.path(), 1, &toml, adr001_md());

    let out = run(dir.path(), &["status", "001", "--status", "accepted"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "ADR 001: accepted\n");

    let after = fs::read_to_string(dir.path().join(".doctrine/adr/001/adr-001.toml")).unwrap();
    assert!(after.contains("status = \"accepted\""), "status flipped");
    assert!(
        after.contains("created = \"2026-01-02\""),
        "created untouched"
    );
    // Carve-out 2: `updated` bumps to today() — non-deterministic, so assert it
    // MOVED off the seeded value rather than byte-pinning it.
    assert!(
        !after.contains("updated = \"2026-01-03\""),
        "updated bumped"
    );
    // Edit-preservation (the toml_edit in-place contract): comment + rels survive.
    // SL-048 PHASE-04: `related` is now a `[[relation]]` row (migrated out of the
    // typed `[relationships]` table); the typed supersedes/superseded_by/tags stay.
    assert!(after.contains("# hand-added comment"), "comment preserved");
    assert!(
        after.contains("[[relation]]") && after.contains("target = \"ADR-002\""),
        "related [[relation]] row preserved"
    );
    assert!(after.contains("tags = [\"lang\"]"), "tags preserved");
}

#[test]
fn adr_status_no_op_prints_but_writes_nothing() {
    let dir = tmp();
    seed(
        dir.path(),
        2,
        &minimal_toml(2, "adopt-ci", "Adopt CI", "accepted"),
        "# ADR-002\n",
    );
    let path = dir.path().join(".doctrine/adr/002/adr-002.toml");
    let before = fs::read_to_string(&path).unwrap();

    let out = run(dir.path(), &["status", "002", "--status", "accepted"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "ADR 002: accepted\n"); // always prints, even on no-op

    // I5 no-op guard: identical status writes nothing — bytes hold exactly.
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn adr_status_on_malformed_toml_refuses_and_leaves_file_untouched() {
    let dir = tmp();
    // Missing `status` key → the tail-insert-into-[relationships] corruption trap;
    // set_adr_status must REFUSE, not append.
    let toml = "id = 50\n\
                slug = \"bad\"\n\
                title = \"Bad\"\n\
                created = \"2026-01-01\"\n\
                updated = \"2026-01-01\"\n\
                \n\
                [relationships]\n\
                supersedes = []\n\
                superseded_by = []\n\
                related = []\n\
                tags = []\n";
    seed(dir.path(), 50, toml, "# ADR-050\n");
    let path = dir.path().join(".doctrine/adr/050/adr-050.toml");

    let out = run(dir.path(), &["status", "050", "--status", "accepted"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: malformed adr 050: missing `status`/`updated` (regenerate via `adr new`)\n"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), toml, "file untouched");
}

// === T6 — `adr list` golden (closes the EX-4 false-premise gap) ==========

/// Seed three ADRs OUT of id order on disk, spanning visible (accepted/proposed)
/// AND hidden (rejected) statuses, so the golden pins hide-set + ascending sort +
/// `ADR-NNN` prefix + the header — none of which e2e_list_conformance covers.
fn seed_list_corpus(root: &Path) {
    seed(
        root,
        2,
        &minimal_toml(2, "adopt-ci", "Adopt CI", "proposed"),
        "# ADR-002\n",
    );
    seed(
        root,
        3,
        &minimal_toml(3, "old-idea", "Old Idea", "rejected"),
        "# ADR-003\n",
    );
    seed(root, 1, adr001_toml(), adr001_md());
}

#[test]
fn adr_list_table_default_hides_and_sorts_byte_exact() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    // Header + ONLY visible rows, ascending by id, ADR- prefixed; rejected ABSENT.
    // Slug-free default (SL-037 D4) — `--columns …,slug` reveals it.
    assert_eq!(
        stdout(&out),
        "id      │ status   │ title\n\
         ADR-001 │ accepted │ Use Rust\n\
         ADR-002 │ proposed │ Adopt CI\n"
    );
}

#[test]
fn adr_list_json_default_is_the_shared_envelope_byte_exact() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"adr\",\n  \"rows\": [\n    {\n      \"id\": \"ADR-001\",\n      \"slug\": \"use-rust\",\n      \"status\": \"accepted\",\n      \"title\": \"Use Rust\"\n    },\n    {\n      \"id\": \"ADR-002\",\n      \"slug\": \"adopt-ci\",\n      \"status\": \"proposed\",\n      \"title\": \"Adopt CI\"\n    }\n  ]\n}"
    );
}

#[test]
fn adr_list_all_reveals_the_hidden_row() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list", "--all"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ status   │ title\n\
         ADR-001 │ accepted │ Use Rust\n\
         ADR-002 │ proposed │ Adopt CI\n\
         ADR-003 │ rejected │ Old Idea\n"
    );
}
