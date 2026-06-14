//! SL-033 PHASE-01 VT-4 — `doctrine standard`'s CLI surface as BLACK-BOX goldens.
//!
//! STD is the third governance kind (after ADR/POL), a thin data module over the
//! shared `governance` spine. These tests pin `standard show` / `standard status`
//! / `standard list` at the CLI surface (byte-exact stdout + JSON + error text)
//! over the BUILT binary — the kind-specific render (STD- prefix, the
//! draft/default/required/deprecated/retired vocab, the deprecated/retired
//! hide-set) that the spine parameterizes over. A one-char edit to a render/format
//! fn turns a golden red. Mirrors `e2e_adr_cli_golden.rs`.
//!
//! Determinism: `standard new`/`standard status` stamp `clock::today()` into
//! created/updated, so fixtures are hand-seeded with FIXED dates. Two carve-outs
//! are asserted structurally rather than byte-exact (documented at each call site):
//! the absolute tempdir path in a "not found" error, and the `updated→today()` bump
//! a real `status` transition writes.

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

/// Hand-seed one standard's authored tree
/// (`.doctrine/standard/NNN/standard-NNN.{toml,md}`) with the caller's exact toml +
/// md bytes — fixed dates make show/list deterministic.
fn seed(root: &Path, id: u32, toml: &str, md: &str) {
    let dir = root.join(format!(".doctrine/standard/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("standard-{id:03}.toml")), toml).unwrap();
    fs::write(dir.join(format!("standard-{id:03}.md")), md).unwrap();
}

/// `doctrine standard <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("standard")
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

/// STD-001: default, a non-empty `related` + `tags` axis, and a hand-added comment
/// — exercises the relationships block (show) and edit-preservation (status).
fn std001_toml() -> &'static str {
    "id = 1\n\
     slug = \"two-space-indent\"\n\
     title = \"Two-space indent\"\n\
     status = \"default\"\n\
     created = \"2026-01-02\"\n\
     updated = \"2026-01-03\"\n\
     \n\
     # hand-added comment — must survive a status edit\n\
     [relationships]\n\
     supersedes = []\n\
     superseded_by = []\n\
     tags = [\"style\"]\n\
     \n\
     [[relation]]\n\
     label = \"related\"\n\
     target = \"STD-002\"\n"
}
fn std001_md() -> &'static str {
    "# STD-001: Two-space indent\n\nbody text here.\n"
}

/// A minimal standard at `status`, no relationships — for list rows + status flips.
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

// === `standard show` Table golden ========================================

#[test]
fn standard_show_table_is_byte_exact() {
    let dir = tmp();
    seed(dir.path(), 1, std001_toml(), std001_md());

    let out = run(dir.path(), &["show", "STD-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "STD-001 — Two-space indent\n\
         two-space-indent · default\n\
         created 2026-01-02 · updated 2026-01-03\n\
         \n\
         relationships:\n\
         \x20\x20related: STD-002\n\
         \x20\x20tags: style\n\
         \n\
         # STD-001: Two-space indent\n\
         \n\
         body text here.\n"
    );
}

// === `standard show --json` golden =======================================

#[test]
fn standard_show_json_is_byte_exact() {
    let dir = tmp();
    seed(dir.path(), 1, std001_toml(), std001_md());

    let out = run(dir.path(), &["show", "STD-001", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    // Pretty JSON, BTreeMap key order (no `preserve_order`), NO trailing newline
    // (`write!`, not `writeln!`). The dynamic stem key (`"standard"`) is the spine's
    // hand-built serde_json::Map.
    assert_eq!(
        stdout(&out),
        "{\n  \"body\": \"# STD-001: Two-space indent\\n\\nbody text here.\\n\",\n  \"kind\": \"standard\",\n  \"standard\": {\n    \"created\": \"2026-01-02\",\n    \"id\": 1,\n    \"relationships\": {\n      \"related\": [\n        \"STD-002\"\n      ],\n      \"superseded_by\": [],\n      \"supersedes\": [],\n      \"tags\": [\n        \"style\"\n      ]\n    },\n    \"slug\": \"two-space-indent\",\n    \"status\": \"default\",\n    \"title\": \"Two-space indent\",\n    \"updated\": \"2026-01-03\"\n  }\n}"
    );
}

// === `standard show` error goldens =======================================

#[test]
fn standard_show_garbage_ref_errors_with_exact_text() {
    let dir = tmp();
    seed(dir.path(), 1, std001_toml(), std001_md());

    let out = run(dir.path(), &["show", "not-a-standard"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: not an STD reference: `not-a-standard` (expected `STD-007` or `7`)\n\
         \n\
         Caused by:\n\
         \x20\x20\x20\x20invalid digit found in string\n"
    );
}

#[test]
fn standard_show_missing_id_errors_with_stable_text() {
    let dir = tmp();
    seed(dir.path(), 1, std001_toml(), std001_md());

    let out = run(dir.path(), &["show", "STD-099"]);
    assert!(!out.status.success());
    let err = stderr(&out);
    // Carve-out 1: the absolute tempdir path floats per run, so pin the stable
    // prefix + the relative path suffix + the source line.
    assert!(
        err.starts_with("Error: standard 099 not found at "),
        "got: {err}"
    );
    assert!(
        err.contains("/.doctrine/standard/099/standard-099.toml\n\nCaused by:\n    No such file or directory (os error 2)\n"),
        "got: {err}"
    );
}

// === `standard status` goldens ===========================================

#[test]
fn standard_status_transition_prints_exact_and_preserves_edits() {
    let dir = tmp();
    // Seed at `draft` with a comment + non-empty rels to prove edit-preservation.
    let toml = std001_toml().replace("status = \"default\"", "status = \"draft\"");
    seed(dir.path(), 1, &toml, std001_md());

    let out = run(dir.path(), &["status", "001", "--status", "required"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "STD 001: required\n");

    let after =
        fs::read_to_string(dir.path().join(".doctrine/standard/001/standard-001.toml")).unwrap();
    assert!(after.contains("status = \"required\""), "status flipped");
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
    // SL-048 PHASE-04: `related` migrated to a `[[relation]]` row; the row + the typed
    // supersedes/superseded_by/tags all survive the in-place status edit.
    assert!(after.contains("# hand-added comment"), "comment preserved");
    assert!(
        after.contains("[[relation]]") && after.contains("target = \"STD-002\""),
        "related [[relation]] row preserved"
    );
    assert!(after.contains("tags = [\"style\"]"), "tags preserved");
}

#[test]
fn standard_status_no_op_prints_but_writes_nothing() {
    let dir = tmp();
    seed(
        dir.path(),
        2,
        &minimal_toml(2, "tabs-bad", "Tabs bad", "required"),
        "# STD-002\n",
    );
    let path = dir.path().join(".doctrine/standard/002/standard-002.toml");
    let before = fs::read_to_string(&path).unwrap();

    let out = run(dir.path(), &["status", "002", "--status", "required"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "STD 002: required\n"); // always prints, even on no-op

    // I5 no-op guard: identical status writes nothing — bytes hold exactly.
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn standard_status_on_malformed_toml_refuses_and_leaves_file_untouched() {
    let dir = tmp();
    // Missing `status` key → the tail-insert-into-[relationships] corruption trap;
    // set_status must REFUSE, not append.
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
    seed(dir.path(), 50, toml, "# STD-050\n");
    let path = dir.path().join(".doctrine/standard/050/standard-050.toml");

    let out = run(dir.path(), &["status", "050", "--status", "required"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: malformed standard 050: missing seeded `status`/`updated` — restore the seeded keys before the transition; the file is left untouched\n"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), toml, "file untouched");
}

// === `standard list` goldens (hide-set + ascending sort + STD- prefix) ====

/// Seed three standards OUT of id order on disk, spanning visible (default/required)
/// AND hidden (retired) statuses, so the golden pins hide-set + ascending sort +
/// `STD-NNN` prefix + the header.
fn seed_list_corpus(root: &Path) {
    seed(
        root,
        2,
        &minimal_toml(2, "tabs-bad", "Tabs bad", "required"),
        "# STD-002\n",
    );
    seed(
        root,
        3,
        &minimal_toml(3, "old-rule", "Old Rule", "retired"),
        "# STD-003\n",
    );
    seed(root, 1, std001_toml(), std001_md());
}

#[test]
fn standard_list_table_default_hides_and_sorts_byte_exact() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    // Header + ONLY visible rows, ascending by id, STD- prefixed; retired ABSENT.
    // Slug-free default (SL-037 D4) — `--columns …,slug` reveals it.
    assert_eq!(
        stdout(&out),
        "id      │ status   │ title\n\
         STD-001 │ default  │ Two-space indent\n\
         STD-002 │ required │ Tabs bad\n"
    );
}

#[test]
fn standard_list_json_default_is_the_shared_envelope_byte_exact() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "{\n  \"kind\": \"standard\",\n  \"rows\": [\n    {\n      \"id\": \"STD-001\",\n      \"slug\": \"two-space-indent\",\n      \"status\": \"default\",\n      \"title\": \"Two-space indent\"\n    },\n    {\n      \"id\": \"STD-002\",\n      \"slug\": \"tabs-bad\",\n      \"status\": \"required\",\n      \"title\": \"Tabs bad\"\n    }\n  ]\n}"
    );
}

#[test]
fn standard_list_all_reveals_the_hidden_row() {
    let dir = tmp();
    seed_list_corpus(dir.path());

    let out = run(dir.path(), &["list", "--all"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ status   │ title\n\
         STD-001 │ default  │ Two-space indent\n\
         STD-002 │ required │ Tabs bad\n\
         STD-003 │ retired  │ Old Rule\n"
    );
}
