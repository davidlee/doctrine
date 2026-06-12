//! SL-046 PHASE-04 — `doctrine inspect <ID>` as BLACK-BOX CLI goldens.
//!
//! Pins the cross-kind relation view at the CLI surface (byte-exact human stdout +
//! `--json` conformance + clean error text) over the BUILT binary
//! (`mem.pattern.testing.black-box-cli-golden`). These prove the whole PHASE-03/04
//! stack end-to-end: the all-kind scan, the ascending-id sort (permutation
//! invariance — seeded OUT of order), the derived inbound reciprocal ("superseded
//! by"), the danglers, and the re-read interaction `type` annotation (C2).
//!
//! Determinism: `inspect` reads only authored TOML — no clock, no rng — so a
//! hand-seeded corpus with fixed bytes yields byte-exact output. The corpus is
//! seeded with entity dirs planted out of id order to prove the sort holds (VT-1
//! permutation invariance) through the CLI, not just the unit layer.

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

/// Seed a slice entity (toml + md) with the given `[relationships]` body.
fn seed_slice(root: &Path, id: u32, rels: &str) {
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{rels}"
        ),
    );
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
        "scope\n",
    );
}

/// Seed a requirement (an edge TARGET only — no outbound).
fn seed_req(root: &Path, id: u32) {
    write(
        root,
        &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
        &format!("id = {id}\nslug = \"r{id}\"\ntitle = \"R{id}\"\nstatus = \"active\"\n"),
    );
    write(
        root,
        &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
        "r\n",
    );
}

/// Seed a tech spec with one outbound interaction (target + free-text type).
fn seed_tech_spec_with_interaction(root: &Path, id: u32, target: &str, ty: &str) {
    write(
        root,
        &format!(".doctrine/spec/tech/{id:03}/spec-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"sp{id}\"\ntitle = \"SP{id}\"\nstatus = \"draft\"\nkind = \"tech\"\n"
        ),
    );
    write(
        root,
        &format!(".doctrine/spec/tech/{id:03}/spec-{id:03}.md"),
        "b\n",
    );
    write(
        root,
        &format!(".doctrine/spec/tech/{id:03}/members.toml"),
        "",
    );
    write(
        root,
        &format!(".doctrine/spec/tech/{id:03}/interactions.toml"),
        &format!("[[edge]]\ntarget = \"{target}\"\ntype = \"{ty}\"\nnotes = \"n\"\n"),
    );
}

/// `doctrine inspect <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("inspect")
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

/// The shared multi-kind corpus, seeded OUT of id order on disk (proves the
/// ascending-id sort end-to-end — VT-1 permutation invariance):
/// - SL-003 supersedes SL-001, requires REQ-005, lists a resolvable spec SPEC-001
///   AND a dangling spec PRD-099.
/// - SL-001 authors nothing — its only relation is the DERIVED inbound "superseded
///   by SL-003".
/// - REQ-005 is an edge target only — derived inbound `requirements` from SL-003.
/// - SPEC-001 is a tech spec with an outbound interaction to a dangling SPEC-002
///   (free-text type "calls"); it is ALSO the resolvable `specs` target of SL-003.
fn seed_corpus(root: &Path) {
    // Out of order on disk: 3 before 1.
    seed_slice(
        root,
        3,
        "requirements = [\"REQ-005\"]\nsupersedes = [\"SL-001\"]\n\
         specs = [\"SPEC-001\", \"PRD-099\"]\n",
    );
    seed_slice(root, 1, "");
    seed_req(root, 5);
    seed_tech_spec_with_interaction(root, 1, "SPEC-002", "calls");
}

// === VT-1 — human render goldens (byte-exact) ============================

/// The SUPERSEDED predecessor: its only relation is the derived inbound reciprocal,
/// rendered as the word "superseded by" (ADR-004 §3 — flipped by section, never a
/// stored field). Fixed section order; outbound/danglers omitted (empty).
#[test]
fn inspect_predecessor_human_byte_exact() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SL-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-001 — relations\n\
         \n\
         inbound:\n\
         \x20\x20superseded by: SL-003\n"
    );
}

/// The SUPERSEDOR: outbound grouped by label in label order (specs, requirements,
/// supersedes), then a danglers section for the unresolved spec PRD-099. The
/// resolvable SPEC-001 is in outbound but NOT a dangler; PRD-099 is in both
/// (outbound lists every authored target; danglers is the unresolved subset).
#[test]
fn inspect_supersedor_human_byte_exact() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SL-003"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-003 — relations\n\
         \n\
         outbound:\n\
         \x20\x20specs: SPEC-001, PRD-099\n\
         \x20\x20requirements: REQ-005\n\
         \x20\x20supersedes: SL-001\n\
         \n\
         danglers:\n\
         \x20\x20specs: PRD-099\n"
    );
}

/// A tech spec: its outbound `interactions` target carries the per-edge free-text
/// `type` annotation, RE-READ from the source `interactions.toml` at render (C2 /
/// EX-4) — `SPEC-002 (calls)`. The same SPEC-002 dangles (no such entity). SPEC-001
/// also has a DERIVED inbound `specs` from SL-003.
#[test]
fn inspect_tech_spec_interaction_type_annotated_byte_exact() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SPEC-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SPEC-001 — relations\n\
         \n\
         outbound:\n\
         \x20\x20interactions: SPEC-002 (calls)\n\
         \n\
         inbound:\n\
         \x20\x20specs: SL-003\n\
         \n\
         danglers:\n\
         \x20\x20interactions: SPEC-002\n"
    );
}

// === VT-3 — empty + unknown-prefix render cleanly (never panic) ==========

/// A well-formed ref to an entity with NO relations: header + an explicit
/// "(no relations)" note, never a bare one-liner or an error.
#[test]
fn inspect_no_relations_entity_renders_cleanly() {
    let dir = tmp();
    seed_corpus(dir.path());
    seed_slice(dir.path(), 50, ""); // isolated — referenced by nobody

    let out = run(dir.path(), &["SL-050"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "SL-050 — relations\n\n(no relations)\n");
}

/// A well-formed ref to a NON-EXISTENT id is an empty view, not an error (VT-3).
#[test]
fn inspect_nonexistent_id_is_empty_view_not_error() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SL-999"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(stdout(&out), "SL-999 — relations\n\n(no relations)\n");
}

/// An UNKNOWN prefix → a clean non-zero error mentioning the prefix, never a panic
/// (EX-1 / VT-3). The error comes from `integrity::parse_canonical_ref`.
#[test]
fn inspect_unknown_prefix_clean_error_not_panic() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["ZZZ-001"]);
    assert!(!out.status.success(), "unknown prefix must exit non-zero");
    let err = stderr(&out);
    assert!(err.starts_with("Error: "), "clean anyhow error: {err}");
    assert!(err.contains("ZZZ"), "error names the prefix: {err}");
    assert!(!err.contains("panic"), "must not panic: {err}");
}

// === VT-2 — `--json` conformance (every InspectView surface present) =====

/// `--json` over the supersedor: assert EVERY surface (id / outbound / inbound /
/// danglers), not just the envelope (`conformance-asserts-surface-not-just-envelope`).
/// Byte-exact pins the shape: each label group is `{label, targets}`, each dangler
/// `{label, target}`; pretty JSON, BTreeMap key order, NO trailing newline. The
/// interaction `type` is a human-render extra — `--json` carries the plain view.
#[test]
fn inspect_json_supersedor_byte_exact_every_surface() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SL-003", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    // Envelope + every surface present.
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(v["kind"], "inspect");
    assert_eq!(v["id"], "SL-003");
    assert!(v["outbound"].is_array(), "outbound surface present");
    assert!(v["inbound"].is_array(), "inbound surface present");
    assert!(v["danglers"].is_array(), "danglers surface present");

    // Byte-exact: the faithful serialized InspectView shape.
    assert_eq!(
        body,
        "{\n  \"danglers\": [\n    {\n      \"label\": \"specs\",\n      \"target\": \"PRD-099\"\n    }\n  ],\n  \"id\": \"SL-003\",\n  \"inbound\": [],\n  \"kind\": \"inspect\",\n  \"outbound\": [\n    {\n      \"label\": \"specs\",\n      \"targets\": [\n        \"SPEC-001\",\n        \"PRD-099\"\n      ]\n    },\n    {\n      \"label\": \"requirements\",\n      \"targets\": [\n        \"REQ-005\"\n      ]\n    },\n    {\n      \"label\": \"supersedes\",\n      \"targets\": [\n        \"SL-001\"\n      ]\n    }\n  ]\n}"
    );
}

/// `--json` over the predecessor: the derived inbound reciprocal appears under the
/// `supersedes` label in JSON (the "superseded by" wording is a HUMAN-render flip
/// only — the JSON carries the structural label faithfully).
#[test]
fn inspect_json_predecessor_inbound_supersedes_surface() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SL-001", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "{\n  \"danglers\": [],\n  \"id\": \"SL-001\",\n  \"inbound\": [\n    {\n      \"label\": \"supersedes\",\n      \"targets\": [\n        \"SL-003\"\n      ]\n    }\n  ],\n  \"kind\": \"inspect\",\n  \"outbound\": []\n}"
    );
}

// === VT-4 — REQ-091's three acceptance criteria discharged ===============

/// REQ-091 (1) adapter-minted opaque ids + (3) re-mapped to canonical refs: every
/// rendered ref is a canonical `KIND-NNN` (the view re-maps the opaque cordage
/// NodeIds back through `key_of`→`canonical_id`; an agent never sees a raw NodeId).
/// (2) every edge traces to an authored outbound relation: REQ-005's inbound is the
/// requirements edge SL-003 *authored* — no synthetic edges, and an entity that
/// authors nothing and is referenced by nothing shows no edges.
#[test]
fn inspect_req091_ids_remapped_and_edges_authored() {
    let dir = tmp();
    seed_corpus(dir.path());

    // (3) re-mapped canonical refs: REQ-005's derived inbound is the canonical
    // `SL-003`, not a NodeId integer.
    let req = run(dir.path(), &["REQ-005", "--json"]);
    assert!(req.status.success(), "stderr: {}", stderr(&req));
    let v: serde_json::Value = serde_json::from_str(&stdout(&req)).expect("json");
    assert_eq!(v["id"], "REQ-005");
    assert_eq!(v["inbound"][0]["label"], "requirements");
    assert_eq!(v["inbound"][0]["targets"][0], "SL-003");
    // No raw NodeId leaks: the whole body is canonical-ref / label strings only.
    let body = stdout(&req);
    assert!(
        !body.contains("NodeId"),
        "opaque cordage ids never leak: {body}"
    );

    // (2) every edge is authored: SL-003's outbound supersedes is exactly the one
    // it authored; SL-001 (which authors none) shows zero outbound — no synthetic
    // reverse edge is fabricated on the predecessor's outbound.
    let pred = run(dir.path(), &["SL-001", "--json"]);
    let pv: serde_json::Value = serde_json::from_str(&stdout(&pred)).expect("json");
    assert_eq!(
        pv["outbound"].as_array().expect("array").len(),
        0,
        "predecessor authors no outbound — inbound is derived, not a synthetic edge"
    );
}
