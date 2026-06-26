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

/// Rewrite a legacy slice `[relationships]` body (`label = [refs]` lines) into the
/// SL-048 migrated `[[relation]]` rows. Slice has no typed tier-2/3 leftovers, so
/// every authored axis becomes rows (read_block launders order — emit order here is
/// irrelevant). An empty body yields no rows.
fn slice_relation_rows(rels: &str) -> String {
    let mut rows = String::new();
    for line in rels.lines() {
        let line = line.trim();
        let Some((label, rest)) = line.split_once('=') else {
            continue;
        };
        let label = label.trim();
        let inner = rest.trim().trim_start_matches('[').trim_end_matches(']');
        for t in inner.split(',') {
            let t = t.trim().trim_matches('"');
            if !t.is_empty() {
                rows.push_str(&format!(
                    "[[relation]]\nlabel = \"{label}\"\ntarget = \"{t}\"\n"
                ));
            }
        }
    }
    rows
}

/// Seed a slice entity (toml + md) with the given relations (SL-048 migrated shape —
/// the legacy axis body is rewritten to `[[relation]]` rows).
fn seed_slice(root: &Path, id: u32, rels: &str) {
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
            slice_relation_rows(rels)
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

/// Seed a memory under `memory/items/<uid>/` with optional key, relation rows, and body.
fn seed_memory(root: &Path, uid: &str, key: Option<&str>, relations: &[(&str, &str)], body: &str) {
    let key_line = key
        .map(|key| format!("memory_key = \"{key}\"\n"))
        .unwrap_or_default();
    let relation_rows: String = relations
        .iter()
        .map(|(label, target)| {
            format!("[[relation]]\nlabel = \"{label}\"\ntarget = \"{target}\"\n")
        })
        .collect();
    write(
        root,
        &format!(".doctrine/memory/items/{uid}/memory.toml"),
        &format!(
            "memory_uid = \"{uid}\"\n\
             {key_line}\
             schema_version = 1\n\
             memory_type = \"pattern\"\n\
             status = \"active\"\n\
             title = \"{uid}\"\n\
             summary = \"summary\"\n\
             created = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\n\
             [scope]\n\
             workspace = \"default\"\n\
             [git]\n\
             repo = \"repo\"\n\
             [trust]\n\
             level = \"medium\"\n\
             [ranking]\n\
             severity = \"none\"\n\
             weight = 0\n\
             {relation_rows}"
        ),
    );
    write(
        root,
        &format!(".doctrine/memory/items/{uid}/memory.md"),
        body,
    );
    if let Some(key) = key {
        #[cfg(unix)]
        std::os::unix::fs::symlink(uid, root.join(".doctrine/memory/items").join(key)).unwrap();
    }
}

/// `doctrine inspect <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
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
/// - SL-003 supersedes SL-001 (label-only), and `references(implements)` REQ-005,
///   a resolvable SPEC-001 AND a dangling PRD-099 (SL-149 PHASE-05 hard cut: the
///   old `requirements`/`specs` axes collapsed into `references(implements)`).
/// - SL-001 authors nothing — its only relation is the DERIVED inbound "superseded
///   by SL-003".
/// - REQ-005 is an edge target only — derived inbound `references(implements)` from
///   SL-003 ("implemented by").
/// - SPEC-001 is a tech spec with an outbound interaction to a dangling SPEC-002
///   (free-text type "calls"); it is ALSO a resolvable `references(implements)`
///   target of SL-003.
fn seed_corpus(root: &Path) {
    // Out of order on disk: 3 before 1.
    seed_slice_rows(
        root,
        3,
        &[
            ("references", Some("implements"), "REQ-005"),
            ("supersedes", None, "SL-001"),
            ("references", Some("implements"), "SPEC-001"),
            ("references", Some("implements"), "PRD-099"),
        ],
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
    // SL-047 PHASE-03: the actionability block is appended below the relation view
    // (SL-046 D1). The relation portion (inbound) stays byte-identical; only the
    // trailing `actionability:` block is new (a proposed slice with no prereqs is
    // eligible + actionable).
    assert_eq!(
        stdout(&out),
        "SL-001 — relations\n\
         \n\
         inbound:\n\
         \x20\x20superseded by: SL-003\n\
         \n\
         actionability:\n\
         \x20\x20eligible: true\n\
         \x20\x20actionable: true\n\
         \x20\x20score: 0.0\n"
    );
}

/// The SUPERSEDOR: outbound grouped by (label, role) — `references(implements)`
/// collects REQ-005, SPEC-001, PRD-099 (the old `specs`/`requirements` axes
/// collapsed into it, SL-149 PHASE-05), then the label-only `supersedes`. The
/// danglers section lists the unresolved PRD-099 under the bare `references` label
/// (danglers drop the role). The resolvable SPEC-001/REQ-005 are in outbound but
/// NOT danglers; PRD-099 is in both (outbound lists every authored target).
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
         \x20\x20references(implements): REQ-005, SPEC-001, PRD-099\n\
         \x20\x20supersedes: SL-001\n\
         \n\
         danglers:\n\
         \x20\x20references: PRD-099\n\
         \n\
         actionability:\n\
         \x20\x20eligible: true\n\
         \x20\x20actionable: true\n\
         \x20\x20score: 0.0\n"
    );
}

/// A tech spec: its outbound `interactions` target carries the per-edge free-text
/// `type` annotation, RE-READ from the source `interactions.toml` at render (C2 /
/// EX-4) — `SPEC-002 (calls)`. The same SPEC-002 dangles (no such entity). SPEC-001
/// also has a DERIVED inbound from SL-003's `references(implements)` — rendered as
/// the role verb "implemented by" (SL-149 PHASE-05).
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
         \x20\x20implemented by: SL-003\n\
         \n\
         danglers:\n\
         \x20\x20interactions: SPEC-002\n\
         \n\
         actionability:\n\
         \x20\x20eligible: true\n\
         \x20\x20actionable: true\n\
         \x20\x20score: 0.0\n"
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
    // The relation portion (`(no relations)`) stays byte-identical; the actionability
    // block is appended (a proposed slice is eligible + actionable).
    assert_eq!(
        stdout(&out),
        "SL-050 — relations\n\n(no relations)\n\
         \n\
         actionability:\n\
         \x20\x20eligible: true\n\
         \x20\x20actionable: true\n\
         \x20\x20score: 0.0\n"
    );
}

/// A well-formed ref to a NON-EXISTENT id is now an ERROR (SL-050 F6 — flips the old
/// empty-view contract): a never-minted id is indistinguishable from a real isolated
/// node at the render layer, so the existence gate makes it a clean non-zero failure
/// with EXACTLY `SL-999: no such entity`.
#[test]
fn inspect_nonexistent_id_is_no_such_entity_error() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SL-999"]);
    assert!(
        !out.status.success(),
        "a never-minted id must exit non-zero"
    );
    let err = stderr(&out);
    assert!(err.starts_with("Error: "), "clean anyhow error: {err}");
    assert!(
        err.contains("SL-999: no such entity"),
        "exact existence-gate message: {err}"
    );
    assert!(!err.contains("panic"), "must not panic: {err}");
    assert!(
        stdout(&out).is_empty(),
        "no partial render on the error path"
    );
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

    // The additive priority actionability block (SL-047 PHASE-03 / SL-046 D1) — the
    // relation surfaces stay byte-identical; only this key is new.
    assert!(
        v["actionability"].is_object(),
        "actionability block present"
    );
    assert_eq!(v["actionability"]["eligible"], true);
    assert_eq!(v["actionability"]["actionable"], true);

    // Byte-exact: the faithful serialized InspectView shape + the additive
    // actionability block (serde_json sorts keys, so `actionability` leads).
    assert_eq!(
        body,
        "{\n  \"actionability\": {\n    \"actionable\": true,\n    \"blockers\": [],\n    \"blocking\": [],\n    \"eligible\": true,\n    \"score\": 0.0\n  },\n  \"danglers\": [\n    {\n      \"label\": \"references\",\n      \"target\": \"PRD-099\"\n    }\n  ],\n  \"id\": \"SL-003\",\n  \"inbound\": [],\n  \"kind\": \"inspect\",\n  \"outbound\": [\n    {\n      \"label\": \"references\",\n      \"role\": \"implements\",\n      \"targets\": [\n        \"REQ-005\",\n        \"SPEC-001\",\n        \"PRD-099\"\n      ]\n    },\n    {\n      \"label\": \"supersedes\",\n      \"targets\": [\n        \"SL-001\"\n      ]\n    }\n  ]\n}"
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
    // The relation surfaces stay byte-identical; the additive `actionability` key
    // (serde sorts keys, so it leads) is the only change (SL-047 PHASE-03).
    assert_eq!(
        stdout(&out),
        "{\n  \"actionability\": {\n    \"actionable\": true,\n    \"blockers\": [],\n    \"blocking\": [],\n    \"eligible\": true,\n    \"score\": 0.0\n  },\n  \"danglers\": [],\n  \"id\": \"SL-001\",\n  \"inbound\": [\n    {\n      \"label\": \"supersedes\",\n      \"targets\": [\n        \"SL-003\"\n      ]\n    }\n  ],\n  \"kind\": \"inspect\",\n  \"outbound\": []\n}"
    );
}

// === VT-4 — REQ-091's three acceptance criteria discharged ===============

/// REQ-091 (1) adapter-minted opaque ids + (3) re-mapped to canonical refs: every
/// rendered ref is a canonical `KIND-NNN` (the view re-maps the opaque cordage
/// NodeIds back through `key_of`→`canonical_id`; an agent never sees a raw NodeId).
/// (2) every edge traces to an authored outbound relation: REQ-005's inbound is the
/// references(implements) edge SL-003 *authored* — no synthetic edges, and an entity that
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
    // SL-149 PHASE-05: the old `requirements` inbound is now `references` + a
    // sibling `role` key — the structural label is faithful, the role recovers the verb.
    assert_eq!(v["inbound"][0]["label"], "references");
    assert_eq!(v["inbound"][0]["role"], "implements");
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

// === SL-149 PHASE-04 — references role rendering (VT-1, VT-2) ===========

/// Seed a slice authoring raw `[[relation]]` rows verbatim — the only way to author a
/// `references` row with a `role` cell (the legacy-axis `seed_slice` cannot). Each row
/// is `(label, role?, target)`; `role = None` authors a label-only row.
fn seed_slice_rows(root: &Path, id: u32, rows: &[(&str, Option<&str>, &str)]) {
    let mut block = String::new();
    for (label, role, target) in rows {
        block.push_str(&format!("[[relation]]\nlabel = \"{label}\"\n"));
        if let Some(role) = role {
            block.push_str(&format!("role = \"{role}\"\n"));
        }
        block.push_str(&format!("target = \"{target}\"\n"));
    }
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{block}"
        ),
    );
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
        "scope\n",
    );
}

/// VT-1: a slice with MIXED `references` roles (implements + concerns) AND a label-only
/// edge (supersedes). Outbound renders `references(implements)` / `references(concerns)`
/// as distinct grouped lines, and the label-only `supersedes` renders bare — proving the
/// role rides the outbound payload without disturbing the label-only surface.
#[test]
fn inspect_references_outbound_roles_rendered_byte_exact() {
    let dir = tmp();
    let root = dir.path();
    // SL-001 implements REQ-005, concerns ADR-001, supersedes SL-002 (label-only).
    seed_slice_rows(
        root,
        1,
        &[
            ("references", Some("implements"), "REQ-005"),
            ("references", Some("concerns"), "ADR-001"),
            ("supersedes", None, "SL-002"),
        ],
    );
    seed_slice(root, 2, "");
    seed_req(root, 5);
    write(
        root,
        ".doctrine/adr/001/adr-001.toml",
        "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
         created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
    );
    write(root, ".doctrine/adr/001/adr-001.md", "a\n");

    let out = run(root, &["SL-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    // Outbound: references rows carry their role verb; supersedes stays bare. References
    // groups sort by role declaration order (implements < concerns); supersedes (the
    // `References` label sorts before `Supersedes` in the enum) follows.
    assert!(
        body.contains(
            "outbound:\n\
             \x20\x20references(implements): REQ-005\n\
             \x20\x20references(concerns): ADR-001\n\
             \x20\x20supersedes: SL-002\n"
        ),
        "outbound role verbs not rendered distinctly: {body}"
    );
}

/// VT-2: the inbound buckets do NOT collapse — two slices reference REQ-005 under
/// DIFFERENT roles (SL-001 implements, SL-003 concerns). `inspect REQ-005` must render
/// TWO distinct inbound lines with distinct derived verbs ("implemented by" /
/// "concerned by"), proving role rides the projection past the single label-keyed
/// `references` overlay (F1 — the bug this phase fixes).
#[test]
fn inspect_references_inbound_roles_do_not_collapse_byte_exact() {
    let dir = tmp();
    let root = dir.path();
    seed_slice_rows(root, 1, &[("references", Some("implements"), "REQ-005")]);
    seed_slice_rows(root, 3, &[("references", Some("concerns"), "REQ-005")]);
    seed_req(root, 5);

    let out = run(root, &["REQ-005"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    // Inbound: TWO buckets, role-derived verbs, NOT a single collapsed `references` line.
    assert!(
        body.contains(
            "inbound:\n\
             \x20\x20implemented by: SL-001\n\
             \x20\x20concerned by: SL-003\n"
        ),
        "inbound role verbs collapsed or mis-rendered: {body}"
    );
}

/// VT-2 (JSON): the `--json` inbound carries the STRUCTURAL label faithfully
/// (`references`, not the human verb) with the `role` as an additive sibling key, so an
/// agent recovers the `(label, role)` grouping. Two distinct inbound groups, each with
/// its own `role`.
#[test]
fn inspect_references_inbound_json_carries_role_key() {
    let dir = tmp();
    let root = dir.path();
    seed_slice_rows(root, 1, &[("references", Some("implements"), "REQ-005")]);
    seed_slice_rows(root, 3, &[("references", Some("concerns"), "REQ-005")]);
    seed_req(root, 5);

    let out = run(root, &["REQ-005", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    let inbound = v["inbound"].as_array().expect("inbound array");
    assert_eq!(
        inbound.len(),
        2,
        "two distinct (label, role) inbound groups"
    );
    // Both groups carry the structural label `references` + their role.
    let roles: Vec<&str> = inbound
        .iter()
        .map(|g| {
            assert_eq!(
                g["label"], "references",
                "structural label faithful in JSON"
            );
            g["role"].as_str().expect("role key present")
        })
        .collect();
    assert!(
        roles.contains(&"implements") && roles.contains(&"concerns"),
        "both roles surface in JSON: {roles:?}"
    );
}

// === SL-099 PHASE-04 — memory inspect bridge ============================

#[test]
fn inspect_memory_uid_renders_outbound_danglers_and_wikilinks() {
    let dir = tmp();
    seed_corpus(dir.path());
    let target = "mem_11111111111111111111111111111111";
    let source = "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    seed_memory(
        dir.path(),
        source,
        Some("mem.pattern.alpha"),
        &[
            ("supports", "SL-099"),
            ("relates", target),
            ("drift", "mem.dead"),
        ],
        &format!("See [[{target}]] and [[mem.dead]]."),
    );
    seed_memory(
        dir.path(),
        target,
        Some("mem.pattern.target"),
        &[],
        "target",
    );

    let out = run(dir.path(), &[source]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa — relations\n\
         \n\
         outbound:\n\
         \x20\x20drift: mem.dead\n\
         \x20\x20relates: mem_11111111111111111111111111111111\n\
         \x20\x20supports: SL-099\n\
         \n\
         danglers:\n\
         \x20\x20drift: mem.dead\n\
         \x20\x20supports: SL-099\n\
         \n\
         wikilinks:\n\
         \x20\x20mem_11111111111111111111111111111111\n\
         \x20\x20mem.dead (dangling)\n"
    );
}

#[test]
fn inspect_memory_uid_renders_inbound_edges() {
    let dir = tmp();
    let target = "mem_11111111111111111111111111111111";
    let source = "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    seed_memory(
        dir.path(),
        target,
        Some("mem.pattern.target"),
        &[],
        "target",
    );
    seed_memory(
        dir.path(),
        source,
        Some("mem.pattern.alpha"),
        &[("relates", target)],
        "body",
    );

    let out = run(dir.path(), &[target]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "mem_11111111111111111111111111111111 — relations\n\
         \n\
         inbound:\n\
         \x20\x20relates: mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"
    );
}

#[test]
fn inspect_memory_key_resolves_and_renders() {
    let dir = tmp();
    let uid = "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    seed_memory(
        dir.path(),
        uid,
        Some("mem.pattern.alpha"),
        &[("relates", "mem.dead")],
        "[[mem.pattern.alpha]]",
    );

    let out = run(dir.path(), &["mem.pattern.alpha"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(stdout(&out).starts_with("mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa — relations\n"));
}

#[test]
fn inspect_memory_nonexistent_uid_prefix_is_error() {
    let dir = tmp();
    seed_memory(
        dir.path(),
        "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        Some("mem.pattern.alpha"),
        &[],
        "body",
    );

    let out = run(dir.path(), &["mem_deadbeef"]);
    assert!(
        !out.status.success(),
        "missing memory ref must exit non-zero"
    );
    let err = stderr(&out);
    assert!(err.starts_with("Error: "), "clean anyhow error: {err}");
    assert!(
        err.contains("no memory matches uid prefix \"mem_deadbeef\""),
        "clear memory-prefix error: {err}"
    );
}
