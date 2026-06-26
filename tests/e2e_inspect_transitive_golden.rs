//! SL-138 PHASE-03 — `doctrine inspect <ID> --transitive` as BLACK-BOX CLI goldens.
//!
//! Pins the transitive relation surface at the CLI boundary (byte-exact human stdout
//! + `--json` conformance + clean error text) over the BUILT binary
//! (`mem.pattern.testing.black-box-cli-golden`). Proves the whole PHASE-03 wiring
//! end-to-end: the `--transitive`/`--direction`/`--labels`/`--max-depth` grammar, the
//! `DirArg`→`TransitiveDir` DOWN-map (ADR-001), the depth cap + truncation indicator,
//! the up/down direction aliases, the relation-only render (NO actionability block),
//! the F2 memory-ref rejection, and the table-derived label validation.
//!
//! Determinism: `inspect` reads only authored TOML — no clock, no rng — so a
//! hand-seeded corpus with fixed bytes yields byte-exact output.

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

/// Seed a slice authoring raw `[[relation]]` rows verbatim. Each row is
/// `(label, role?, target)`; `role = None` authors a label-only row.
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

/// Seed a memory under `memory/items/<uid>/` with an optional key (for the F2 tests).
fn seed_memory(root: &Path, uid: &str, key: Option<&str>) {
    let key_line = key
        .map(|key| format!("memory_key = \"{key}\"\n"))
        .unwrap_or_default();
    write(
        root,
        &format!(".doctrine/memory/items/{uid}/memory.toml"),
        &format!(
            "memory_uid = \"{uid}\"\n{key_line}\
             schema_version = 1\nmemory_type = \"pattern\"\nstatus = \"active\"\n\
             title = \"{uid}\"\nsummary = \"summary\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [scope]\nworkspace = \"default\"\n[git]\nrepo = \"repo\"\n\
             [trust]\nlevel = \"medium\"\n[ranking]\nseverity = \"none\"\nweight = 0\n"
        ),
    );
    write(
        root,
        &format!(".doctrine/memory/items/{uid}/memory.md"),
        "m\n",
    );
    if let Some(key) = key {
        #[cfg(unix)]
        std::os::unix::fs::symlink(uid, root.join(".doctrine/memory/items").join(key)).unwrap();
    }
}

/// The transitive chain corpus:
/// `SL-001 --supersedes--> SL-002 --supersedes--> SL-003 --supersedes--> SL-004`,
/// plus `SL-001 --references(implements)--> REQ-005` (a second label, one hop).
/// Seeded OUT of id order on disk.
fn seed_chain(root: &Path) {
    seed_slice_rows(root, 3, &[("supersedes", None, "SL-004")]);
    seed_slice_rows(
        root,
        1,
        &[
            ("supersedes", None, "SL-002"),
            ("references", Some("implements"), "REQ-005"),
        ],
    );
    seed_slice_rows(root, 2, &[("supersedes", None, "SL-003")]);
    seed_slice_rows(root, 4, &[]);
    seed_req(root, 5);
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

// === VT-1 — human render goldens (byte-exact) ============================

/// `--direction outbound`: only the outbound section (inbound omitted), two label
/// groups sorted by name (`references` < `supersedes`), the supersedes chain walked
/// transitively to SL-004, default depth 5, no truncation.
#[test]
fn transitive_outbound_human_byte_exact() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &["SL-001", "--transitive", "--direction", "outbound"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-001 — transitive (depth 5)\n\
         \n\
         this depends on (outbound):\n\
         \x20\x20references: REQ-005\n\
         \x20\x20supersedes: SL-002, SL-003, SL-004\n"
    );
}

/// `--direction inbound`: only the inbound (blast-radius) section. From SL-004,
/// `Against` reaches the three transitive predecessors id-ascending.
#[test]
fn transitive_inbound_human_byte_exact() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &["SL-004", "--transitive", "--direction", "inbound"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-004 — transitive (depth 5)\n\
         \n\
         depends on this (inbound):\n\
         \x20\x20supersedes: SL-001, SL-002, SL-003\n"
    );
}

/// Default direction (`both`): inbound section FIRST (blast-radius framing), then
/// outbound — the awareness view from a mid-chain node.
#[test]
fn transitive_both_default_human_byte_exact() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(dir.path(), &["SL-002", "--transitive"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-002 — transitive (depth 5)\n\
         \n\
         depends on this (inbound):\n\
         \x20\x20supersedes: SL-001\n\
         \n\
         this depends on (outbound):\n\
         \x20\x20supersedes: SL-003, SL-004\n"
    );
}

/// `--max-depth 2`: the cap bites the 3-hop supersedes chain (SL-004 excluded) and
/// sets the truncation indicator; the 1-hop references group is untouched.
#[test]
fn transitive_max_depth_truncates_byte_exact() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &[
            "SL-001",
            "--transitive",
            "--direction",
            "outbound",
            "--max-depth",
            "2",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-001 — transitive (depth 2)\n\
         \n\
         this depends on (outbound):\n\
         \x20\x20references: REQ-005\n\
         \x20\x20supersedes: SL-002, SL-003\n\
         \n\
         … some chains truncated at depth 2 — re-run with --max-depth all\n"
    );
}

/// `--max-depth all`: unbounded — the full chain reaches SL-004, no truncation line,
/// header reads `depth all`.
#[test]
fn transitive_max_depth_all_reaches_leaf() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &[
            "SL-001",
            "--transitive",
            "--direction",
            "outbound",
            "--max-depth",
            "all",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-001 — transitive (depth all)\n\
         \n\
         this depends on (outbound):\n\
         \x20\x20references: REQ-005\n\
         \x20\x20supersedes: SL-002, SL-003, SL-004\n"
    );
}

/// `--labels supersedes` narrows to the one label (references group omitted).
#[test]
fn transitive_labels_narrows() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &[
            "SL-001",
            "--transitive",
            "--direction",
            "outbound",
            "--labels",
            "supersedes",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "SL-001 — transitive (depth 5)\n\
         \n\
         this depends on (outbound):\n\
         \x20\x20supersedes: SL-002, SL-003, SL-004\n"
    );
}

// === VT-1 — direction aliases (up == inbound, down == outbound) ==========

/// `--direction up` is an alias for `inbound`; `--direction down` for `outbound`
/// (design §5: `Inbound`/alias=`up`, `Outbound`/alias=`down`). Alias output is
/// byte-identical to the canonical spelling.
#[test]
fn transitive_direction_aliases_equivalent() {
    let dir = tmp();
    seed_chain(dir.path());

    let up = run(dir.path(), &["SL-004", "--transitive", "--direction", "up"]);
    let inbound = run(
        dir.path(),
        &["SL-004", "--transitive", "--direction", "inbound"],
    );
    assert!(up.status.success(), "stderr: {}", stderr(&up));
    assert_eq!(stdout(&up), stdout(&inbound), "up must equal inbound");

    let down = run(
        dir.path(),
        &["SL-001", "--transitive", "--direction", "down"],
    );
    let outbound = run(
        dir.path(),
        &["SL-001", "--transitive", "--direction", "outbound"],
    );
    assert!(down.status.success(), "stderr: {}", stderr(&down));
    assert_eq!(stdout(&down), stdout(&outbound), "down must equal outbound");
}

// === VT-1 — `--json` conformance (byte-exact) ============================

/// `--json` over an outbound walk: `kind=inspect-transitive`, alphabetical keys, the
/// non-requested `inbound` direction key OMITTED, `max_depth: 5`, each group
/// `{label, targets, truncated}`, no trailing newline.
#[test]
fn transitive_json_outbound_byte_exact() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &[
            "SL-001",
            "--transitive",
            "--direction",
            "outbound",
            "--json",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    // Structural assertions first (envelope + every surface — not just the bytes).
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(v["kind"], "inspect-transitive");
    assert_eq!(v["id"], "SL-001");
    assert_eq!(v["max_depth"], 5);
    assert_eq!(v["truncated"], false);
    assert!(
        v.get("inbound").is_none(),
        "non-requested direction omitted"
    );
    assert!(v["outbound"].is_array(), "outbound surface present");

    assert_eq!(
        body,
        "{\n  \"id\": \"SL-001\",\n  \"kind\": \"inspect-transitive\",\n  \"max_depth\": 5,\n  \"outbound\": [\n    {\n      \"label\": \"references\",\n      \"targets\": [\n        \"REQ-005\"\n      ],\n      \"truncated\": false\n    },\n    {\n      \"label\": \"supersedes\",\n      \"targets\": [\n        \"SL-002\",\n        \"SL-003\",\n        \"SL-004\"\n      ],\n      \"truncated\": false\n    }\n  ],\n  \"truncated\": false\n}"
    );
}

/// `--json --max-depth all`: `max_depth` serializes as JSON `null` when unbounded.
#[test]
fn transitive_json_unbounded_max_depth_null() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &[
            "SL-001",
            "--transitive",
            "--direction",
            "outbound",
            "--max-depth",
            "all",
            "--json",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(v["max_depth"], serde_json::Value::Null);
    assert_eq!(v["truncated"], false);
}

/// `--json --max-depth 2`: view-level `truncated` is true and the capped group
/// carries its own `truncated: true`.
#[test]
fn transitive_json_truncated_flag() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &[
            "SL-001",
            "--transitive",
            "--direction",
            "outbound",
            "--max-depth",
            "2",
            "--json",
        ],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(v["truncated"], true);
    let groups = v["outbound"].as_array().expect("outbound array");
    let supersedes = groups
        .iter()
        .find(|g| g["label"] == "supersedes")
        .expect("supersedes group");
    assert_eq!(supersedes["truncated"], true);
}

// === VT-2 — F2 memory-ref rejection ======================================

/// A memory KEY + `--transitive` → clean non-zero error naming `retrieve --expand`,
/// gated BEFORE the memory inspect early-return.
#[test]
fn transitive_memory_key_rejected_points_at_retrieve_expand() {
    let dir = tmp();
    seed_chain(dir.path());
    seed_memory(
        dir.path(),
        "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        Some("mem.pattern.alpha"),
    );

    let out = run(dir.path(), &["mem.pattern.alpha", "--transitive"]);
    assert!(!out.status.success(), "memory + --transitive must error");
    let err = stderr(&out);
    assert!(err.starts_with("Error: "), "clean anyhow error: {err}");
    assert!(
        err.contains("retrieve --expand"),
        "error must point at retrieve --expand: {err}"
    );
    assert!(
        stdout(&out).is_empty(),
        "no partial render on the error path"
    );
}

/// A memory UID + `--transitive` → same rejection.
#[test]
fn transitive_memory_uid_rejected() {
    let dir = tmp();
    seed_chain(dir.path());
    seed_memory(dir.path(), "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", None);

    let out = run(
        dir.path(),
        &["mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "--transitive"],
    );
    assert!(
        !out.status.success(),
        "memory uid + --transitive must error"
    );
    assert!(
        stderr(&out).contains("retrieve --expand"),
        "error must point at retrieve --expand: {}",
        stderr(&out)
    );
}

/// A memory ref WITHOUT `--transitive` still renders normally (the gate is
/// transitive-only — it does not poison the existing memory inspect bridge).
#[test]
fn memory_ref_without_transitive_still_renders() {
    let dir = tmp();
    seed_memory(
        dir.path(),
        "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        Some("mem.pattern.alpha"),
    );

    let out = run(dir.path(), &["mem.pattern.alpha"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(stdout(&out).starts_with("mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa — relations\n"));
}

// === VT-4 — label validation at the CLI surface ==========================

/// `--labels contextualizes` (a KNOWN but no-overlay label) → "not transitively
/// walkable" error listing the valid set.
#[test]
fn transitive_labels_no_overlay_rejected() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(
        dir.path(),
        &["SL-001", "--transitive", "--labels", "contextualizes"],
    );
    assert!(!out.status.success(), "no-overlay label must error");
    let err = stderr(&out);
    assert!(
        err.contains("not transitively walkable"),
        "expected walkability error: {err}"
    );
    assert!(
        err.contains("contextualizes"),
        "names the offending label: {err}"
    );
}

/// `--labels bogus` (an UNKNOWN name) → the same "not transitively walkable" error
/// (unknown-name and no-overlay cases share one message, F4).
#[test]
fn transitive_labels_unknown_name_rejected() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(dir.path(), &["SL-001", "--transitive", "--labels", "bogus"]);
    assert!(!out.status.success(), "unknown label must error");
    assert!(
        stderr(&out).contains("not transitively walkable"),
        "expected walkability error: {}",
        stderr(&out)
    );
}

// === VT-3 — clap `requires` + bare-inspect regression ====================

/// `--direction inbound` WITHOUT `--transitive` trips clap `requires` → non-zero
/// with a usage error (never a transitive render).
#[test]
fn modifier_without_transitive_errors() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(dir.path(), &["SL-001", "--direction", "inbound"]);
    assert!(
        !out.status.success(),
        "a modifier without --transitive must error (clap requires)"
    );
    assert!(
        stderr(&out).contains("transitive"),
        "clap requires error names --transitive: {}",
        stderr(&out)
    );
}

/// Bare `inspect <ID>` (no `--transitive`) does NOT trip `requires` (the
/// `default_value_t` on `--direction` is a default, not a CLI-present value) and
/// renders the ordinary relation + actionability view — the regression gate.
#[test]
fn bare_inspect_unaffected_by_requires() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(dir.path(), &["SL-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    assert!(
        body.starts_with("SL-001 — relations\n"),
        "ordinary relation view: {body}"
    );
    assert!(
        body.contains("actionability:"),
        "actionability block present: {body}"
    );
    assert!(
        !body.contains("transitive"),
        "no transitive surface leaked: {body}"
    );
}

/// Existence gate holds on the transitive path too: a never-minted id → the shared
/// `no such entity` error (EX-4 — `require_minted` reused).
#[test]
fn transitive_nonexistent_id_is_no_such_entity_error() {
    let dir = tmp();
    seed_chain(dir.path());

    let out = run(dir.path(), &["SL-999", "--transitive"]);
    assert!(!out.status.success(), "never-minted id must exit non-zero");
    assert!(
        stderr(&out).contains("SL-999: no such entity"),
        "shared existence-gate message: {}",
        stderr(&out)
    );
}
