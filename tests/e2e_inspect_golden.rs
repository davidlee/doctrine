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
use std::process::Output;

mod common;

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
    common::doctrine_cmd(root)
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
         \x20\x20score: 1.0\n"
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
         \x20\x20score: 1.0\n"
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
         \x20\x20score: 1.0\n"
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
         \x20\x20score: 1.0\n"
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
        "{\n  \"actionability\": {\n    \"actionable\": true,\n    \"blockers\": [],\n    \"blocking\": [],\n    \"eligible\": true,\n    \"score\": 1.0\n  },\n  \"danglers\": [\n    {\n      \"label\": \"references\",\n      \"target\": \"PRD-099\"\n    }\n  ],\n  \"id\": \"SL-003\",\n  \"inbound\": [],\n  \"kind\": \"inspect\",\n  \"outbound\": [\n    {\n      \"label\": \"references\",\n      \"role\": \"implements\",\n      \"targets\": [\n        \"REQ-005\",\n        \"SPEC-001\",\n        \"PRD-099\"\n      ]\n    },\n    {\n      \"label\": \"supersedes\",\n      \"targets\": [\n        \"SL-001\"\n      ]\n    }\n  ]\n}"
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
        "{\n  \"actionability\": {\n    \"actionable\": true,\n    \"blockers\": [],\n    \"blocking\": [],\n    \"eligible\": true,\n    \"score\": 1.0\n  },\n  \"danglers\": [],\n  \"id\": \"SL-001\",\n  \"inbound\": [\n    {\n      \"label\": \"supersedes\",\n      \"targets\": [\n        \"SL-003\"\n      ]\n    }\n  ],\n  \"kind\": \"inspect\",\n  \"outbound\": []\n}"
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

// === SL-246 PHASE-04 — `--knowledge <skip|facets|full>`: the generic composed
// === read (EX-1..EX-6). ====================================================

/// T2 (EX-1): `--knowledge bogus` fails with the `FromStr` error naming the three
/// levels — before any corpus scan or render is attempted.
#[test]
fn knowledge_flag_rejects_an_unknown_level_naming_the_three_names() {
    let dir = tmp();
    seed_corpus(dir.path());

    let out = run(dir.path(), &["SL-001", "--knowledge", "bogus"]);
    assert!(
        !out.status.success(),
        "an unknown --knowledge token must fail"
    );
    let err = stderr(&out);
    assert!(err.contains("skip"), "names skip: {err}");
    assert!(err.contains("facets"), "names facets: {err}");
    assert!(err.contains("full"), "names full: {err}");
}

/// VT-4 / T3 (EX-2): `--knowledge` at a non-Skip level with `--transitive` is
/// refused, naming IMP-398 S5. A one-sided test (only asserting the refusal)
/// would pass under a clap `conflicts_with`, which is NOT the mechanism (a clap
/// conflict refuses the flag's mere PRESENCE, not a non-Skip LEVEL) — so this
/// also asserts the explicit-default case succeeds and composes nothing.
#[test]
fn a_non_skip_level_with_transitive_is_refused_naming_imp_398_s5() {
    let dir = tmp();
    seed_corpus(dir.path());

    let facets = run(
        dir.path(),
        &["SL-003", "--transitive", "--knowledge", "facets"],
    );
    assert!(
        !facets.status.success(),
        "facets + transitive must be refused"
    );
    let err = stderr(&facets);
    assert!(
        err.starts_with("Error: "),
        "bare bail! shape, no Caused by: {err}"
    );
    assert!(!err.contains("Caused by"), "bare bail!, not sourced: {err}");
    assert!(err.contains("IMP-398"), "names the carrier: {err}");

    let full = run(
        dir.path(),
        &["SL-003", "--transitive", "--knowledge", "full"],
    );
    assert!(!full.status.success(), "full + transitive must be refused");
    assert!(
        stderr(&full).contains("IMP-398"),
        "names the carrier: {}",
        stderr(&full)
    );

    // The explicit default composes nothing and must be ACCEPTED — a clap
    // `conflicts_with` would wrongly refuse even this pairing.
    let implicit = run(dir.path(), &["SL-003", "--transitive"]);
    let explicit_skip = run(
        dir.path(),
        &["SL-003", "--transitive", "--knowledge", "skip"],
    );
    assert!(
        implicit.status.success() && explicit_skip.status.success(),
        "explicit --knowledge skip + --transitive must succeed: {} / {}",
        stderr(&implicit),
        stderr(&explicit_skip)
    );
    assert_eq!(
        stdout(&implicit),
        stdout(&explicit_skip),
        "explicit skip is byte-identical to the implicit default under --transitive"
    );
}

// === SL-246 PHASE-04 T5 — the DEC-151 synthetic knowledge fixture corpus =====

/// One knowledge record's `.toml` + `.md`, modelled on
/// `.doctrine/knowledge/decision/140/record-140.toml` (STD-001, memory 6: more
/// than ~5 params gets an args struct rather than an `#[allow]`).
struct RecordSpec<'a> {
    /// The kebab dir word (`"decision"`, `"concept"`, …) — also `record_kind`'s
    /// authored value, the two always agree.
    dir_word: &'a str,
    id: u32,
    slug: &'a str,
    title: &'a str,
    status: &'a str,
    /// The raw `key = value` lines under `[facet]` (empty string ⇒ every field
    /// defaults blank — `RawFacet`'s `#[serde(default)]`, mirroring
    /// `record-001.toml`'s empty concept facet).
    facet_toml: &'a str,
    /// Raw `[[relation]]` blocks (one or more), authored verbatim.
    relation_rows: &'a str,
    /// `.md` body — mandatory on every fixture (`read_record` reads it
    /// unconditionally; a missing `.md` prunes the record at scan — R-3).
    body: &'a str,
}

fn seed_record(root: &Path, spec: &RecordSpec<'_>) {
    let dir = format!(".doctrine/knowledge/{}/{:03}", spec.dir_word, spec.id);
    write(
        root,
        &format!("{dir}/record-{:03}.toml", spec.id),
        &format!(
            "schema = \"doctrine.knowledge\"\nversion = 1\n\n\
             id = {}\nslug = \"{}\"\ntitle = \"{}\"\nrecord_kind = \"{}\"\n\
             status = \"{}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n[facet]\n{}\n{}",
            spec.id,
            spec.slug,
            spec.title,
            spec.dir_word,
            spec.status,
            spec.facet_toml,
            spec.relation_rows
        ),
    );
    write(root, &format!("{dir}/record-{:03}.md", spec.id), spec.body);
}

/// Write a MALFORMED `record-<id>.toml` (invalid TOML syntax) plus a normal `.md` —
/// T5 shape 7: the scan PRUNES this record (names it on stderr) rather than
/// reaching the in-block unreadable marker (T6 / design §5.2, §9.5).
fn seed_malformed_record(root: &Path, dir_word: &str, id: u32) {
    let dir = format!(".doctrine/knowledge/{dir_word}/{id:03}");
    write(
        root,
        &format!("{dir}/record-{id:03}.toml"),
        "not valid toml {{{\n",
    );
    write(root, &format!("{dir}/record-{id:03}.md"), "m\n");
}

/// A backlog issue (T5 shape 5) authoring an inbound `references(concerns)` edge —
/// source-kind selection EXCLUDES it (`DEC-148`, `kinds::is_record`): it renders in
/// the relations section but never in the knowledge block.
fn seed_issue_concerning(root: &Path, id: u32, target: &str) {
    let dir = format!(".doctrine/backlog/issue/{id:03}");
    write(
        root,
        &format!("{dir}/backlog-{id:03}.toml"),
        &format!(
            "schema = \"doctrine.backlog\"\nversion = 1\n\n\
             id = {id}\nslug = \"i{id}\"\ntitle = \"I{id}\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\ncreated = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\ntags = []\n\n\
             [relationships]\nneeds = []\nafter = []\ntriggers = []\n\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\n\
             descriptor = \"d\"\ntarget = \"{target}\"\n"
        ),
    );
    write(root, &format!("{dir}/backlog-{id:03}.md"), "i\n");
}

/// A review (T5 shape 6) — its single outbound `reviews` edge is DERIVED from
/// `[target].ref` (`review::relation_edges`), never an authored `[[relation]]`
/// row; `reviews` is `LinkPolicy::TypedVerbOnly` so there is no `link` path to it.
/// Same exclusion as the backlog issue — a non-record source, never in the
/// knowledge block.
fn seed_review_of(root: &Path, id: u32, target: &str) {
    let dir = format!(".doctrine/review/{id:03}");
    write(
        root,
        &format!("{dir}/review-{id:03}.toml"),
        &format!(
            "id    = {id}\nslug  = \"r{id}\"\ntitle = \"R{id}\"\n\n\
             [review]\nfacet     = \"design\"\nraiser    = \"a\"\nresponder = \"b\"\n\n\
             [target]\nref   = \"{target}\"\n"
        ),
    );
    write(root, &format!("{dir}/review-{id:03}.md"), "r\n");
}

/// The DEC-151 synthetic fixture corpus (EX-5), consumed by every `VT-1`..`VT-4`
/// row — see the phase sheet's `T5` table for the shape → row map. Subjects
/// (`SL-9xx`) are plain slices with NO outbound of their own; every fixture record
/// authors an inbound edge AT one of them.
fn seed_knowledge_corpus(root: &Path) {
    // Subjects.
    seed_slice(root, 900, ""); // the multi-shape hub: DEC-901, DEC-902, CPT-903 (+ISS-904, RV-905)
    seed_slice(root, 901, ""); // isolated: DEC-901's SECOND edge — a clean single-entry Full golden
    seed_slice(root, 906, ""); // DEC-907's dual-labeled target (dedup / first-caption-wins)
    seed_slice(root, 908, ""); // DEC-998/999/1000/1001 — numeric ordering past 999
    seed_slice(root, 909, ""); // no inbound records at all
    seed_slice(root, 910, ""); // DEC-911 (malformed) + DEC-912 (clean) — the pruning fact
    seed_slice(root, 913, ""); // the seven-kind parity subject (A-d)

    // Shape 1 — a filled DEC. TWO outbound `shapes` rows: SL-900 (the hub, shared
    // with shapes 2/3) and SL-901 (isolated — a clean single-entry Full golden).
    seed_record(
        root,
        &RecordSpec {
            dir_word: "decision",
            id: 901,
            slug: "filled-decision",
            title: "Filled decision",
            status: "accepted",
            facet_toml: "context      = \"Because things\"\n\
                         choice       = \"Do the thing\"\n\
                         alternatives = [\"Skip it\", \"Wait\"]\n\
                         rationale    = \"It reduces risk\"\n\
                         consequences = [\"More work\", \"Less risk\"]\n\
                         decided_by   = \"epoch\"\n\
                         decided_on   = \"2026-01-05\"\n",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-900\"\n\
                            [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-901\"\n",
            body: "Decision body line.\n",
        },
    );

    // Shape 2 — an unfilled DEC, `.md` EXACTLY 200 bytes (0.2 KB, DEC-149's hint).
    seed_record(
        root,
        &RecordSpec {
            dir_word: "decision",
            id: 902,
            slug: "unfilled-decision",
            title: "Unfilled decision",
            status: "proposed",
            facet_toml: "",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-900\"\n",
            body: &format!("{}\n", "x".repeat(199)),
        },
    );

    // Shape 3 — a CPT, `[facet]` header with no keys (the by-design marker).
    seed_record(
        root,
        &RecordSpec {
            dir_word: "concept",
            id: 903,
            slug: "a-concept",
            title: "A concept",
            status: "active",
            facet_toml: "",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-900\"\n",
            body: "concept body\n",
        },
    );

    // Shape 4 — reachable under TWO inbound labels at SL-906: `references(concerns)`
    // AND `shapes`. `References` sorts before `Shapes` (declaration order), so I3's
    // first-caption-wins makes the dedup'd caption "concerned by".
    seed_record(
        root,
        &RecordSpec {
            dir_word: "decision",
            id: 907,
            slug: "dual-labeled-decision",
            title: "Dual labeled decision",
            status: "accepted",
            facet_toml: "",
            relation_rows: "[[relation]]\nlabel = \"references\"\nrole = \"concerns\"\n\
                            descriptor = \"d\"\ntarget = \"SL-906\"\n\
                            [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-906\"\n",
            body: "d907\n",
        },
    );

    // Shape 5 — a backlog item pointing inbound at the hub; excluded from the
    // knowledge block (not a record kind).
    seed_issue_concerning(root, 904, "SL-900");

    // Shape 6 — a review pointing inbound at the hub; same exclusion.
    seed_review_of(root, 905, "SL-900");

    // Shape 7 — DEC-911 malformed `.toml` (pruned at scan, named on stderr) beside
    // clean DEC-912, both `shapes SL-910`.
    seed_malformed_record(root, "decision", 911);
    seed_record(
        root,
        &RecordSpec {
            dir_word: "decision",
            id: 912,
            slug: "clean-decision",
            title: "Clean decision",
            status: "accepted",
            facet_toml: "",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-910\"\n",
            body: "d912\n",
        },
    );

    // Shape 8 — numeric ordering survives past 999 (I4): 998, 999, 1000, 1001, all
    // `shapes SL-908`.
    for id in [998, 999, 1000, 1001] {
        seed_record(
            root,
            &RecordSpec {
                dir_word: "decision",
                id,
                slug: &format!("ordering-{id}"),
                title: &format!("Ordering {id}"),
                status: "accepted",
                facet_toml: "",
                relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-908\"\n",
                body: "o\n",
            },
        );
    }

    // Shape 9 (SL-909) — no fixture record needed; the subject itself has zero
    // inbound (seeded above, referenced by nobody).

    // A-d — the seven-kind parity subject (SL-913): one record of EACH kind, every
    // facet field filled (deciding AND argument tiers), all `shapes SL-913`.
    seed_record(
        root,
        &RecordSpec {
            dir_word: "assumption",
            id: 920,
            slug: "parity-assumption",
            title: "Parity assumption",
            status: "held",
            facet_toml: "claim           = \"Claim A\"\n\
                         confidence      = \"high\"\n\
                         basis           = \"observation\"\n\
                         validation_plan = \"Plan A\"\n\
                         validated_by    = \"epoch\"\n\
                         validated_on    = \"2026-01-06\"\n\
                         invalidated_by  = \"nobody\"\n\
                         invalidated_on  = \"2026-01-07\"\n",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-913\"\n",
            body: "asm body\n",
        },
    );
    seed_record(
        root,
        &RecordSpec {
            dir_word: "decision",
            id: 921,
            slug: "parity-decision",
            title: "Parity decision",
            status: "accepted",
            facet_toml: "context      = \"Context A\"\n\
                         choice       = \"Choice A\"\n\
                         alternatives = [\"Alt A1\", \"Alt A2\"]\n\
                         rationale    = \"Rationale A\"\n\
                         consequences = [\"Cons A1\", \"Cons A2\"]\n\
                         decided_by   = \"epoch\"\n\
                         decided_on   = \"2026-01-06\"\n",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-913\"\n",
            body: "dec body\n",
        },
    );
    seed_record(
        root,
        &RecordSpec {
            dir_word: "question",
            id: 922,
            slug: "parity-question",
            title: "Parity question",
            status: "open",
            facet_toml: "question    = \"Question A\"\n\
                         why_matters = \"Why A\"\n\
                         answer      = \"Answer A\"\n\
                         answered_by = \"epoch\"\n\
                         answered_on = \"2026-01-06\"\n",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-913\"\n",
            body: "que body\n",
        },
    );
    seed_record(
        root,
        &RecordSpec {
            dir_word: "constraint",
            id: 923,
            slug: "parity-constraint",
            title: "Parity constraint",
            status: "active",
            facet_toml: "statement     = \"Statement A\"\n\
                         source        = \"canon\"\n\
                         applies_to    = [\"Area A1\", \"Area A2\"]\n\
                         waiver_reason = \"Waiver A\"\n\
                         waived_by     = \"epoch\"\n\
                         waived_on     = \"2026-01-06\"\n",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-913\"\n",
            body: "con body\n",
        },
    );
    seed_record(
        root,
        &RecordSpec {
            dir_word: "evidence",
            id: 924,
            slug: "parity-evidence",
            title: "Parity evidence",
            status: "captured",
            facet_toml: "datum      = \"Datum A\"\n\
                         provenance = \"inspection\"\n\
                         confidence = \"high\"\n",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-913\"\n",
            body: "evd body\n",
        },
    );
    seed_record(
        root,
        &RecordSpec {
            dir_word: "hypothesis",
            id: 925,
            slug: "parity-hypothesis",
            title: "Parity hypothesis",
            status: "proposed",
            facet_toml: "proposition = \"Proposition A\"\npredicts    = \"Predicts A\"\n",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-913\"\n",
            body: "hyp body\n",
        },
    );
    seed_record(
        root,
        &RecordSpec {
            dir_word: "concept",
            id: 926,
            slug: "parity-concept",
            title: "Parity concept",
            status: "active",
            facet_toml: "",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"SL-913\"\n",
            body: "cpt body\n",
        },
    );

    // 11 — a record AS SUBJECT with one hop, no recursion (EX-4 / X6): EVD-914
    // supports DEC-901; CPT-915 shapes EVD-914 (DEC-901's own inbound stays 1-hop).
    seed_record(
        root,
        &RecordSpec {
            dir_word: "evidence",
            id: 914,
            slug: "supporting-evidence",
            title: "Supporting evidence",
            status: "captured",
            facet_toml: "",
            relation_rows: "[[relation]]\nlabel = \"supports\"\ntarget = \"DEC-901\"\n",
            body: "evd914 body\n",
        },
    );
    seed_record(
        root,
        &RecordSpec {
            dir_word: "concept",
            id: 915,
            slug: "second-hop-concept",
            title: "Second hop concept",
            status: "active",
            facet_toml: "",
            relation_rows: "[[relation]]\nlabel = \"shapes\"\ntarget = \"EVD-914\"\n",
            body: "cpt915 body\n",
        },
    );
}

/// The knowledge-block substring of a table-arm `stdout`, between the `"\nknowledge:\n"`
/// frame and the `"\nactionability:\n"` frame that always follows it — so a
/// byte-exact assertion targets the BLOCK this phase adds, not the whole surface
/// (which also carries the shared corpus's relations + score, out of scope here).
fn knowledge_section(body: &str) -> &str {
    let open = "\nknowledge:\n";
    let start = body
        .find(open)
        .unwrap_or_else(|| panic!("no knowledge: section in: {body}"))
        + open.len();
    let close = "\nactionability:\n";
    let end = body[start..]
        .find(close)
        .unwrap_or_else(|| panic!("no actionability: section after knowledge: in: {body}"))
        + start;
    &body[start..end]
}

// === VT-1 (T7 / EX-3 / I1): Skip is byte-identical; JSON omits the key ======

/// Claim 2 of `EX-3`'s decomposition (T7): the EXPLICIT `--knowledge skip` is
/// byte-identical to no flag at all, on BOTH arms, for several subjects — at
/// minimum an entity with no records AND SL-900 (5 inbound edges, 3 of them
/// records), so an accidental composition at Skip would show.
#[test]
fn skip_is_byte_identical_to_the_prior_surface() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    for id in ["SL-001", "SL-003", "SL-900"] {
        // Table arm.
        let implicit = run(root, &[id]);
        let explicit = run(root, &[id, "--knowledge", "skip"]);
        assert!(implicit.status.success(), "stderr: {}", stderr(&implicit));
        assert!(explicit.status.success(), "stderr: {}", stderr(&explicit));
        assert_eq!(
            stdout(&implicit),
            stdout(&explicit),
            "no-flag vs --knowledge skip must be byte-identical for {id}"
        );

        // --json arm.
        let implicit_json = run(root, &[id, "--json"]);
        let explicit_json = run(root, &[id, "--json", "--knowledge", "skip"]);
        assert!(
            implicit_json.status.success(),
            "stderr: {}",
            stderr(&implicit_json)
        );
        assert!(
            explicit_json.status.success(),
            "stderr: {}",
            stderr(&explicit_json)
        );
        assert_eq!(
            stdout(&implicit_json),
            stdout(&explicit_json),
            "--json byte-identical for {id}"
        );
    }
}

/// `VT-1`'s second keyword (T7): the JSON arm at `Skip` OMITS the `knowledge` key
/// entirely — never `null`, never `[]`. Both breaches are asserted explicitly
/// because `v["knowledge"].is_null()` is ALSO true for an absent key, so a sloppy
/// test would pass on a `null`. `is_none()` on `.get(...)` is the correct check.
#[test]
fn skip_omits_the_knowledge_key_rather_than_emitting_an_empty_one() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-900", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(
        v.get("knowledge").is_none(),
        "Skip must omit the key entirely, not null/[]: {v}"
    );

    // Paired with an EMPTY selection at a non-Skip level (SL-909, no inbound): the
    // key is PRESENT and `[]` — proving absent (Skip) vs present-and-empty
    // (empty selection) are the two distinct states `knowledge_value`'s `Option`
    // exists to express.
    let empty = run(root, &["SL-909", "--json", "--knowledge", "facets"]);
    assert!(empty.status.success(), "stderr: {}", stderr(&empty));
    let ev: serde_json::Value = serde_json::from_str(&stdout(&empty)).expect("valid JSON");
    assert!(
        ev.get("knowledge").is_some(),
        "an empty selection at a non-Skip level carries the key: {ev}"
    );
    assert_eq!(ev["knowledge"], serde_json::json!([]));
}

// === VT-3 (T6): the two block-level cases the record markers cannot reach =====

/// X1/D3: an entity with NO inbound records says so explicitly on BOTH arms —
/// this is the one empty state that sits on the BLOCK rather than on a record.
#[test]
fn an_entity_with_no_inbound_records_says_so_on_both_arms() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    for level in ["facets", "full"] {
        let out = run(root, &["SL-909", "--knowledge", level]);
        assert!(out.status.success(), "stderr: {}", stderr(&out));
        assert_eq!(
            knowledge_section(&stdout(&out)),
            "(no knowledge records point at SL-909)\n",
            "table arm at --knowledge {level}"
        );

        let jout = run(root, &["SL-909", "--json", "--knowledge", level]);
        assert!(jout.status.success(), "stderr: {}", stderr(&jout));
        let v: serde_json::Value = serde_json::from_str(&stdout(&jout)).expect("valid JSON");
        assert_eq!(v["knowledge"], serde_json::json!([]), "json arm at {level}");
    }
}

/// STD-003's disjoint "failed BEFORE selection" half (T6 / design §5.2): DEC-911's
/// malformed `.toml` is PRUNED by the scan and named on stderr — the in-block
/// unreadable marker is unreachable e2e (design §9.5, PHASE-03's unit tests hold
/// it). The block still renders DEC-912, the rest of the selection.
#[test]
fn a_pruned_record_is_named_on_stderr_and_the_block_renders_the_rest() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-910", "--knowledge", "facets"]);
    assert!(
        out.status.success(),
        "a pruned record degrades, it does not fail the read: {}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(
        err.contains("record-911"),
        "stderr names the pruned file: {err}"
    );

    let body = stdout(&out);
    assert!(
        body.contains("DEC-912 (shaped_by)"),
        "the rest of the selection still renders: {body}"
    );
    assert!(
        !body.contains("DEC-911"),
        "the pruned record itself never enters the block: {body}"
    );
    assert!(
        !body.contains("unreadable:"),
        "this is the OTHER disjoint half — no in-block marker here: {body}"
    );
}

// === VT-2 (T8): the levels carry what DEC-150 says, and the two arms agree =====

/// `facets_carries_deciding_fields_only` — `inspect SL-900 --knowledge facets`,
/// table arm, byte-exact. Derived from `DEC-150`'s per-kind table (the phase
/// sheet), NOT pasted from a run: DEC-901's deciding tier is `context, choice,
/// rationale` in TABLE order (not `alternatives`/`consequences`/`decided_by`/
/// `decided_on`); DEC-902 (every kept field blank) renders the unfilled marker
/// with the 200-byte (0.2 KB) prose hint; CPT-903 renders the by-design marker.
/// Group order is [`EntityKey`]'s derived `Ord` — `(prefix, id)` lexicographic on
/// the PREFIX first (confirmed empirically against the built binary, per
/// DERIVE-DON'T-PASTE: the initial kind-table-order derivation was WRONG and
/// this replaced it after investigation) — so `CPT-903` (prefix `"CPT"`) sorts
/// BEFORE `DEC-901`/`DEC-902` (prefix `"DEC"`, `'C' < 'D'`), and the two DECs sort
/// by ascending id within their shared prefix.
#[test]
fn facets_carries_deciding_fields_only() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-900", "--knowledge", "facets"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let out_stdout = stdout(&out);
    let block = knowledge_section(&out_stdout);
    assert_eq!(
        block,
        "CPT-903 (shaped_by)\n\
         \n(no facet by design — a concept rides its prose body)\n\
         \n\
         DEC-901 (shaped_by)\n\
         \n[facet]\n\
         \x20\x20context: Because things\n\
         \x20\x20choice: Do the thing\n\
         \x20\x20rationale: It reduces risk\n\
         \n\
         DEC-902 (shaped_by)\n\
         \n(no facet recorded — 0.2 KB of prose: doctrine knowledge show DEC-902)\n"
    );
    // Absences, by name — not by line count (memory 12 / the sheet's guidance).
    assert!(
        !block.contains("alternatives"),
        "argument tier absent: {block}"
    );
    assert!(
        !block.contains("consequences"),
        "argument tier absent: {block}"
    );
    assert!(
        !block.contains("decided_by"),
        "argument tier absent: {block}"
    );
    assert!(
        !block.contains("decided_on"),
        "argument tier absent: {block}"
    );
}

/// `full_carries_every_field_and_the_prose_body` — `--knowledge full` on SL-901
/// (DEC-901's SECOND, isolated `shapes` edge — a clean single-entry golden). All
/// seven DEC fields in TABLE order, plus the prose body, plus the DELIBERATE
/// doubled id (`entry_header`'s `DEC-901 (shaped_by)` line followed by
/// `format_metadata_with_facet`'s own `DEC-901 — Filled decision` line) — pinned
/// as-is per the orchestrator's standing ruling, not "fixed".
#[test]
fn full_carries_every_field_and_the_prose_body() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-901", "--knowledge", "full"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let out_stdout = stdout(&out);
    let block = knowledge_section(&out_stdout);
    assert_eq!(
        block,
        "DEC-901 (shaped_by)\n\
         DEC-901 — Filled decision\n\
         filled-decision · decision · accepted\n\
         created 2026-01-01 · updated 2026-01-01\n\
         \n[facet]\n\
         \x20\x20context: Because things\n\
         \x20\x20choice: Do the thing\n\
         \x20\x20alternatives: Skip it, Wait\n\
         \x20\x20rationale: It reduces risk\n\
         \x20\x20consequences: More work, Less risk\n\
         \x20\x20decided_by: epoch\n\
         \x20\x20decided_on: 2026-01-05\n\
         shapes: [SL-900, SL-901]\n\
         \n\
         Decision body line.\n"
    );
}

/// Ordered header list for the seven-kind parity subject (SL-913) — group order
/// is `EntityKey`'s derived `Ord`, `(prefix, id)` lexicographic on the PREFIX
/// first (confirmed against the built binary — see
/// `facets_carries_deciding_fields_only`'s doc comment): `ASM < CON < CPT < DEC <
/// EVD < HYP < QUE` (byte-wise string order, NOT `RecordKind::ALL`'s declaration
/// order).
const PARITY_HEADERS: [&str; 7] = [
    "ASM-920 (shaped_by)\n",
    "CON-923 (shaped_by)\n",
    "CPT-926 (shaped_by)\n",
    "DEC-921 (shaped_by)\n",
    "EVD-924 (shaped_by)\n",
    "HYP-925 (shaped_by)\n",
    "QUE-922 (shaped_by)\n",
];

/// Slice a knowledge block into per-record substrings at each known header's
/// start (headers are literal, unique text — no fragile line-counting).
fn split_entries<'a>(block: &'a str, headers: &[&str]) -> Vec<&'a str> {
    let starts: Vec<usize> = headers
        .iter()
        .map(|h| {
            block
                .find(h)
                .unwrap_or_else(|| panic!("header {h:?} not found in: {block}"))
        })
        .collect();
    (0..starts.len())
        .map(|i| {
            let end = starts.get(i + 1).copied().unwrap_or(block.len());
            &block[starts[i]..end]
        })
        .collect()
}

/// The facet field-KEY SET of one text-arm entry, or `None` when the slot is a
/// marker (by-design / unfilled) rather than a rendered `[facet]` block. Facet
/// lines are exactly the `  key: value` lines directly after `"[facet]\n"` — the
/// first line WITHOUT the two-space indent ends the block (works at both `facets`
/// and `full`: the un-indented `shapes: […]` relationship line, or nothing at
/// `facets`, both terminate it).
fn table_facet_keys(entry: &str) -> Option<std::collections::BTreeSet<String>> {
    let marker = "[facet]\n";
    let idx = entry.find(marker)?;
    let after = &entry[idx + marker.len()..];
    let mut keys = std::collections::BTreeSet::new();
    for line in after.lines() {
        match line
            .strip_prefix("  ")
            .and_then(|rest| rest.split_once(": "))
        {
            Some((key, _)) => {
                keys.insert(key.to_string());
            }
            None => break,
        }
    }
    Some(keys)
}

/// The facet field-KEY SET of one JSON-arm entry's `facet` value, or `None` when
/// it is a marker object (`{"marker": …}`, EX-3 — the marker rides INSIDE `facet`,
/// never as a sibling key).
fn json_facet_keys(facet: &serde_json::Value) -> Option<std::collections::BTreeSet<String>> {
    let obj = facet.as_object()?;
    if obj.len() == 1 && obj.contains_key("marker") {
        None
    } else {
        Some(obj.keys().cloned().collect())
    }
}

/// `json_and_table_carry_the_same_fields_at_the_same_level_for_every_kind` — the
/// parity subject SL-913 (A-d), at BOTH levels. A CONTENT-PARITY claim VERIFIED
/// between the two arms (PHASE-03 made it true by construction — one
/// `facet_fields` table, one `TierFilter`, rendered twice), never RE-DERIVED
/// against a second hardcoded per-kind list (R-6) — this test extracts both
/// arms' actual key sets and compares them to EACH OTHER. The `CPT` row is the
/// interesting one: parity there means the marker is present on BOTH arms, not
/// that both carry zero fields.
#[test]
fn json_and_table_carry_the_same_fields_at_the_same_level_for_every_kind() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    for level in ["facets", "full"] {
        let table_out = run(root, &["SL-913", "--knowledge", level]);
        assert!(table_out.status.success(), "stderr: {}", stderr(&table_out));
        let table_stdout = stdout(&table_out);
        let block = knowledge_section(&table_stdout);
        let table_entries = split_entries(block, &PARITY_HEADERS);
        assert_eq!(table_entries.len(), 7, "seven entries at {level}: {block}");

        let json_out = run(root, &["SL-913", "--json", "--knowledge", level]);
        assert!(json_out.status.success(), "stderr: {}", stderr(&json_out));
        let v: serde_json::Value = serde_json::from_str(&stdout(&json_out)).expect("valid JSON");
        let json_entries = v["knowledge"].as_array().expect("knowledge array");
        assert_eq!(json_entries.len(), 7, "seven json entries at {level}");

        for (prefix, table_entry) in ["ASM", "CON", "CPT", "DEC", "EVD", "HYP", "QUE"]
            .into_iter()
            .zip(&table_entries)
        {
            let json_entry = json_entries
                .iter()
                .find(|e| {
                    e.get("reference")
                        .or_else(|| e.get("id"))
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|s| s.starts_with(prefix))
                })
                .unwrap_or_else(|| {
                    panic!("no json entry for {prefix} at {level}: {json_entries:?}")
                });
            let table_keys = table_facet_keys(table_entry);
            let json_keys = json_facet_keys(&json_entry["facet"]);
            assert_eq!(
                table_keys, json_keys,
                "{prefix} at {level}: table vs json facet key-set parity"
            );
        }
    }
}

/// `full_json_carries_the_complete_knowledge_show_payload` — at `full`, an
/// entry's JSON keys are EXACTLY `show_value`'s twelve (`id, record_kind, slug,
/// title, status, created, updated, tags, facet, evidence, relationships, body`)
/// PLUS `caption`, and NO `reference` key (`show_value`'s `id` already carries
/// it). Asserts every key (memory 2 — surface, not envelope), so a thirteenth key
/// added to `show_json` shows up here without anyone editing this test.
#[test]
fn full_json_carries_the_complete_knowledge_show_payload() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-901", "--json", "--knowledge", "full"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    let entries = v["knowledge"].as_array().expect("knowledge array");
    assert_eq!(
        entries.len(),
        1,
        "SL-901 is DEC-901's isolated edge: {entries:?}"
    );
    let entry = entries[0].as_object().expect("entry object");

    let keys: std::collections::BTreeSet<&str> = entry.keys().map(String::as_str).collect();
    let expected: std::collections::BTreeSet<&str> = [
        "id",
        "record_kind",
        "slug",
        "title",
        "status",
        "created",
        "updated",
        "tags",
        "facet",
        "evidence",
        "relationships",
        "body",
        "caption",
    ]
    .into_iter()
    .collect();
    assert_eq!(keys, expected, "exactly show_value's twelve plus caption");
    assert!(
        !entry.contains_key("reference"),
        "no reference key — show_value's id already carries it: {entry:?}"
    );
    assert_eq!(entry["id"], "DEC-901");
    assert_eq!(entry["caption"], "shaped_by");
}

// === VT-2 (T8, extra): I4 — numeric ordering survives selection past 999 ======

#[test]
fn selection_order_survives_past_999_in_the_composed_block() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-908", "--knowledge", "facets"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let out_stdout = stdout(&out);
    let block = knowledge_section(&out_stdout);
    let positions: Vec<usize> = ["DEC-998", "DEC-999", "DEC-1000", "DEC-1001"]
        .iter()
        .map(|id| {
            block
                .find(id)
                .unwrap_or_else(|| panic!("{id} missing from block: {block}"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|w| w[0] < w[1]),
        "998, 999, 1000, 1001 — never the lexical order: {positions:?} in {block}"
    );
}

// === EX-4 / X6 (T9): a record as the subject, one hop, no recursion ==========

/// `inspect DEC-901 --knowledge facets` is well-formed (`shapes` legally targets
/// record kinds) and renders ONE hop — `EVD-914 (supported_by)` — WITHOUT
/// recursing into `EVD-914`'s own inbound `CPT-915`. Asserted by an ABSENCE, not
/// a count (memory 12): a count is a proxy a future recursive change could still
/// satisfy.
#[test]
fn dec_901_as_subject_composes_one_hop_without_recursing() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["DEC-901", "--knowledge", "facets"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let out_stdout = stdout(&out);
    let block = knowledge_section(&out_stdout);
    assert!(
        block.contains("EVD-914 (supported_by)"),
        "one hop renders: {block}"
    );
    assert!(
        !block.contains("CPT-915"),
        "no recursion into EVD-914's own inbound: {block}"
    );

    let jout = run(root, &["DEC-901", "--json", "--knowledge", "facets"]);
    assert!(jout.status.success(), "stderr: {}", stderr(&jout));
    let v: serde_json::Value = serde_json::from_str(&stdout(&jout)).expect("valid JSON");
    let entries = v["knowledge"].as_array().expect("knowledge array");
    assert_eq!(entries.len(), 1, "one hop only: {entries:?}");
    assert_eq!(entries[0]["reference"], "EVD-914");
    let body = stdout(&jout);
    assert!(
        !body.contains("CPT-915"),
        "no recursion on the json arm: {body}"
    );
}

/// T5 shape 4, witnessed end-to-end (the dedup CLAIM itself is PHASE-02's unit
/// test): DEC-907 authors BOTH `references(concerns)` and `shapes` at SL-906.
/// `References` sorts before `Shapes` (declaration order — `src/relation.rs`), so
/// `I3`'s first-caption-wins rule collapses the two edges to ONE entry, captioned
/// `concerned by`.
#[test]
fn a_record_reachable_under_two_inbound_labels_dedups_to_the_first_caption() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-906", "--knowledge", "facets"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let out_stdout = stdout(&out);
    let block = knowledge_section(&out_stdout);
    assert_eq!(
        block.matches("DEC-907 (").count(),
        1,
        "exactly one HEADER — dedup'd, not two entries: {block}"
    );
    assert!(
        block.contains("DEC-907 (concerned by)"),
        "first caption wins (References sorts before Shapes): {block}"
    );
    assert!(
        !block.contains("shaped_by"),
        "the second label's caption never appears: {block}"
    );
}

/// T5 shapes 5/6 (`DEC-148` / `kinds::is_record`): a backlog item (ISS-904) and a
/// review (RV-905) both point inbound at the hub (SL-900) — source-kind selection
/// EXCLUDES them from the knowledge block (they are not RECORD kinds), while both
/// still appear in the RELATIONS section above it.
#[test]
fn non_record_inbound_sources_appear_in_relations_but_never_in_the_knowledge_block() {
    let dir = tmp();
    let root = dir.path();
    seed_corpus(root);
    seed_knowledge_corpus(root);

    let out = run(root, &["SL-900", "--knowledge", "full"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    assert!(
        body.contains("ISS-904"),
        "the backlog item appears somewhere (relations): {body}"
    );
    assert!(
        body.contains("RV-905"),
        "the review appears somewhere (relations): {body}"
    );

    let out_stdout = stdout(&out);
    let block = knowledge_section(&out_stdout);
    assert!(
        !block.contains("ISS-904"),
        "backlog items are excluded from the knowledge block: {block}"
    );
    assert!(
        !block.contains("RV-905"),
        "reviews are excluded from the knowledge block: {block}"
    );
}
