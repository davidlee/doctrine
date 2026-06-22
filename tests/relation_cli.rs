// SPDX-License-Identifier: GPL-3.0-only
//! SL-137 PHASE-02 — e2e black-box CLI goldens for `doctrine relation list`
//! and `doctrine relation census`, including diagnostics policy (VT-10, VT-11).
//!
//! Covers:
//! - VT-10: `relation list` and `relation census` wire end-to-end over a fixture corpus
//! - VT-11: diagnostics policy — Error-only stderr, Warning suppression, edge-dropping summary

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

/// Seed a minimal project root with doctrine.toml.
fn seed_project(root: &Path) {
    fs::create_dir_all(root.join(".doctrine")).unwrap();
    fs::write(root.join("doctrine.toml"), "").unwrap();
}

/// Seed a slice entity with given [[relation]] edges.
fn seed_slice(root: &Path, id: u32, edges: &[(&str, &[&str])]) {
    let dir = root.join(format!(".doctrine/slice/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    let mut rels = String::new();
    for (label, targets) in edges {
        for t in *targets {
            rels.push_str(&format!(
                "[[relation]]\nlabel = \"{label}\"\ntarget = \"{t}\"\n"
            ));
        }
    }
    fs::write(
        dir.join(format!("slice-{id:03}.toml")),
        format!(
            "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{rels}"
        ),
    )
    .unwrap();
    fs::write(
        dir.join(format!("slice-{id:03}.md")),
        "# fixture\n\nbody.\n",
    )
    .unwrap();
}

/// Seed a requirement entity (for an edge target to resolve against).
fn seed_requirement(root: &Path, id: u32) {
    let dir = root.join(format!(".doctrine/requirement/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("requirement-{id:03}.toml")),
        format!(
            "id = {id}\nslug = \"r{id}\"\ntitle = \"R{id}\"\n\
             status = \"active\"\nkind = \"functional\"\n"
        ),
    )
    .unwrap();
    fs::write(dir.join(format!("requirement-{id:03}.md")), "body\n").unwrap();
}

/// Seed a memory entity with given relations.
fn seed_memory(root: &Path, uid: &str, title: &str, relations: &[(&str, &str)]) {
    let items_dir = root.join(".doctrine/memory/items").join(uid);
    fs::create_dir_all(&items_dir).unwrap();
    let rels: Vec<String> = relations
        .iter()
        .map(|(l, t)| format!("[[relation]]\nlabel = \"{l}\"\ntarget = \"{t}\"\n"))
        .collect();
    fs::write(
        items_dir.join("memory.toml"),
        format!(
            "schema = \"doctrine.memory\"\nversion = 1\nmemory_uid = \"{uid}\"\n\
             title = \"{title}\"\nstatus = \"active\"\nmemory_type = \"pattern\"\n{}",
            rels.concat()
        ),
    )
    .unwrap();
}

/// Seed a malformed entity (parse failure).
fn seed_malformed_slice(root: &Path, id: u32) {
    let dir = root.join(format!(".doctrine/slice/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("slice-{id:03}.toml")), "id = notanumber\n").unwrap();
    fs::write(dir.join(format!("slice-{id:03}.md")), "malformed\n").unwrap();
}

// ── VT-10: e2e black-box CLI golden ──────────────────────────────────────────

#[test]
fn relation_list_filters_and_renders_table() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    // SL-001 → REQ-005 (requirements, resolved), SL-001 → REQ-999 (requirements, unresolved)
    seed_requirement(root, 5);
    seed_slice(root, 1, &[("requirements", &["REQ-005", "REQ-999"])]);
    // SL-002 → REQ-005 (requirements, resolved)
    seed_requirement(root, 5);
    _ = seed_requirement; // REQ-005 already seeded above
    seed_slice(root, 2, &[("requirements", &["REQ-005"])]);

    // Unfiltered list
    let out = run(root, &["relation", "list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("source"), "has header: {s}");
    assert!(s.contains("SL-001"), "has SL-001: {s}");
    assert!(s.contains("SL-002"), "has SL-002: {s}");
    assert!(s.contains("REQ-005"), "has REQ-005: {s}");
    assert!(s.contains("REQ-999"), "has REQ-999: {s}");

    // --label filter
    let out = run(root, &["relation", "list", "--label", "requirements"]);
    let s = stdout(&out);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(s.lines().count(), 4, "header + 3 rows: {s}");

    // --unresolved filter
    let out = run(root, &["relation", "list", "--unresolved"]);
    let s = stdout(&out);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(s.contains("REQ-999"), "has unresolved: {s}");
    assert!(!s.contains("REQ-005"), "no resolved: {s}");
    assert_eq!(s.lines().count(), 2, "header + 1 row: {s}");

    // --source-kind SL
    let out = run(root, &["relation", "list", "--source-kind", "SL"]);
    let s = stdout(&out);
    assert!(out.status.success());
    assert!(s.contains("SL-001"), "has SL-001: {s}");

    // --source-kind MEM returns empty (no memory entities)
    let out = run(root, &["relation", "list", "--source-kind", "MEM"]);
    let s = stdout(&out);
    assert!(out.status.success());
    assert!(s.is_empty(), "expected empty, got: {s}");
}

#[test]
fn relation_list_json_format() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_requirement(root, 5);
    seed_slice(root, 1, &[("requirements", &["REQ-005"])]);

    let out = run(root, &["relation", "list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
    assert_eq!(v["kind"].as_str(), Some("relation"));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["source"].as_str(), Some("SL-001"));
    assert_eq!(rows[0]["label"].as_str(), Some("requirements"));
    assert_eq!(rows[0]["target"].as_str(), Some("REQ-005"));
    assert_eq!(rows[0]["state"].as_str(), Some("resolved"));
}

#[test]
fn relation_census_table_and_json() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_requirement(root, 5);
    // Two requirements edges + one related edge (both legal for SL).
    // SL-001 → REQ-005 (requirements, resolved), SL-001 → REQ-999 (requirements, unresolved)
    // SL-001 → some-free-text (related, free-text — "related" is AnyNumbered so target
    //   validation is write-time only; scan includes it as UnvalidatedText)
    seed_slice(
        root,
        1,
        &[
            ("requirements", &["REQ-005", "REQ-999"]),
            ("related", &["some free text"]),
        ],
    );

    // Table census
    let out = run(root, &["relation", "census"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.contains("label"), "has header: {s}");
    assert!(s.contains("count"), "has count: {s}");
    assert!(s.contains("requirements"), "has requirements: {s}");
    // requirements: count 2, resolved 1 (REQ-005), unresolved 1 (REQ-999), free_text 0
    assert!(s.contains("2"), "requirements count: {s}");
    // related: count 1, resolved 0, unresolved 0, free_text 1
    assert!(s.contains("related"), "has related: {s}");

    // JSON census
    let out = run(root, &["relation", "census", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
    assert_eq!(v["kind"].as_str(), Some("census"));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    // requirements should be first (count 2 > count 1)
    assert_eq!(rows[0]["label"].as_str(), Some("requirements"));
    assert_eq!(rows[0]["count"].as_u64(), Some(2));
    assert_eq!(rows[0]["resolved"].as_u64(), Some(1));
    assert_eq!(rows[0]["unresolved"].as_u64(), Some(1));
    assert_eq!(rows[0]["free_text"].as_u64(), Some(0));
}

#[test]
fn relation_list_empty_result_is_empty_string() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_slice(root, 1, &[]);

    let out = run(root, &["relation", "list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.is_empty(), "empty results → empty string, got: '{s}'");
}

#[test]
fn relation_census_empty_result_is_empty_string() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_slice(root, 1, &[]);

    let out = run(root, &["relation", "census"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    assert!(s.is_empty(), "empty results → empty string, got: '{s}'");
}

// ── VT-11: diagnostics policy ────────────────────────────────────────────────

#[test]
fn malformed_entity_produces_error_line_on_stderr() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_slice(root, 1, &[("requirements", &["REQ-005"])]);
    seed_malformed_slice(root, 2);

    // relation list should report the malformed entity error on stderr
    let out = run(root, &["relation", "list"]);
    // Still succeeds (Error-tolerant scan)
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    // SL-001's edge still appears in the table
    assert!(s.contains("SL-001"), "valid entity still rendered: {s}");

    let err = stderr(&out);
    assert!(
        err.contains("SL-002") || err.contains("002"),
        "malformed entity reported on stderr: {err}"
    );
    assert!(
        err.contains("failed to read"),
        "error message present: {err}"
    );
}

#[test]
fn dangling_ref_warning_is_suppressed_per_row() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_requirement(root, 5);
    // SL-001 → REQ-005 (resolved) and SL-001 → REQ-999 (dangling — not seeded)
    seed_slice(root, 1, &[("requirements", &["REQ-005", "REQ-999"])]);

    let out = run(root, &["relation", "list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let s = stdout(&out);
    // Both resolved and unresolved rows appear in the table
    assert!(s.contains("REQ-005"), "resolved rendered: {s}");
    assert!(s.contains("REQ-999"), "unresolved rendered: {s}");

    // No per-row Warning diagnostic on stderr (dangling ref is a Warning)
    let err = stderr(&out);
    assert!(
        !err.contains("dangling") && !err.contains("does not resolve"),
        "dangling-ref Warning must be suppressed on stderr, got: {err}"
    );
}

#[test]
fn empty_field_memory_relation_produces_edge_dropping_summary() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_slice(root, 1, &[]);
    // Memory entity with an empty-label relation (edge-dropping Warning)
    seed_memory(
        root,
        "mem_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "Test Memory",
        &[("", "SL-001"), ("refs", "")],
    );

    let out = run(root, &["relation", "list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));

    // No valid relation edges in the output (both were dropped)
    let s = stdout(&out);
    assert!(s.is_empty(), "no edges rendered after drops: {s}");

    // Single summary line on stderr
    let err = stderr(&out);
    assert!(
        err.contains("edge(s) dropped"),
        "edge-dropping summary present: {err}"
    );
    assert!(
        err.contains("doctrine validate"),
        "references validate command: {err}"
    );
    assert!(
        err.contains("2"), // two edges dropped: empty label, empty target
        "count correct in summary: {err}"
    );

    // No per-row message for those specific warnings
    assert!(
        !err.contains("empty relation"),
        "per-row empty-relation warning must NOT appear on stderr, got: {err}"
    );
}

#[test]
fn edge_dropping_and_error_coexist_on_stderr() {
    let t = tmp();
    let root = t.path();
    seed_project(root);
    seed_malformed_slice(root, 1);
    seed_memory(
        root,
        "mem_cccccccccccccccccccccccccccccccc",
        "Bad Memory",
        &[("", "SL-001")],
    );

    let out = run(root, &["relation", "list"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let err = stderr(&out);

    // Error from malformed entity
    assert!(
        err.contains("failed to read") || err.contains("SL-001"),
        "error diagnostic: {err}"
    );
    // Edge-dropping summary
    assert!(
        err.contains("edge(s) dropped"),
        "edge-dropping summary: {err}"
    );
}
