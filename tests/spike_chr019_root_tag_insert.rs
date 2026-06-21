//! SPIKE CHR-019 — settle SL-136 F-1 / RV-129.
//!
//! D4 (SL-136 design §5.5) rests on one load-bearing, previously-unproven premise:
//! `toml_edit 0.22` root insert of a `tags` key via `doc.as_table_mut().insert(..)`
//! lands the key ABOVE all trailing `[relationships]` / `[[relation]]` / named
//! subtables, and the rendered doc RE-PARSES with `root.tags` set (a semantic, not
//! textual, check). The original evidence was a throwaway `/tmp/tomlprobe` that
//! tested only synthetic shapes — never committed, never run against the REAL
//! worst-case corpus shapes, and never reconciled against the IDENTICAL F-1 refusal
//! that both live write seams currently carry:
//!   - `apply_status`  (src/dep_seq.rs:305) — bails on a missing managed key
//!   - `apply_tags`    (src/backlog.rs:1942) — bails on a missing `tags` key
//! both citing "a tail insert would land the key inside a trailing subtable (silent
//! corruption)".
//!
//! This spike reads the LIVE committed corpus (not synthetic) and exercises the exact
//! toml_edit API D4 depends on, then asserts by RE-PARSING. Pin: toml_edit 0.22.27.
//!
//! Three hypotheses, pass/fail per fixture:
//!   H1 — root insert is safe: a missing root key inserts ABOVE trailing tables and
//!        re-parses at root (not nested inside a trailing subtable).
//!   H2 — strip-typed-then-insert-root relocation round-trips; values preserved;
//!        every pre-existing trailing table / relation row / comment intact.
//!   H3 — THE CONTRADICTION: is the both-seam refusal premise FALSE (then both could
//!        safely insert; D4 is sound) or TRUE (then D4 is unsafe → redesign)?
//!
//! Worst-case real shapes (the original probe never tested these):
//!   - SL-118      `[relationships]` → `[[relation]]` → named `[estimate]` → comment
//!   - spec-016    root `tags` already present, then `[[source]]` + `[[member]]` AoT
//!   - RFC-002     16× `[[relation]]` AND the only non-empty LIVE tag set
//!   - SL-048      comment block AFTER the last `[[relation]]`
//!   - ADR-014 /   carry BOTH root `status` AND `[relationships].tags` (the same-file
//!     POL-001     overlap the design's "disjoint seams" framing glosses)

use std::path::PathBuf;
use toml_edit::{Array, DocumentMut, Item, value};

/// Read a live committed corpus file under `.doctrine/`. Decisive evidence wants the
/// REAL bytes, not a synthetic transcription (F-1's exact complaint about the probe).
fn load(rel: &str) -> DocumentMut {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".doctrine")
        .join(rel);
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
    text.parse::<DocumentMut>()
        .unwrap_or_else(|e| panic!("parse {}: {e}", p.display()))
}

/// The exact op D4 / `apply_tags` would use to seed a missing key: a plain root
/// `insert`. Mirrors `apply_status`'s insert too (same API, scalar vs array value).
fn root_insert_tags(doc: &mut DocumentMut, tags: &[&str]) {
    let mut arr = Array::new();
    for t in tags {
        arr.push(*t);
    }
    doc.as_table_mut().insert("tags", value(arr));
}

/// Strip the typed `[relationships].tags`, returning its values — the relocation
/// SOURCE half of the migration. `None` if the table/key is absent.
#[allow(dead_code)]
fn strip_relationship_tags(doc: &mut DocumentMut) -> Option<Vec<String>> {
    let rel = doc.get_mut("relationships")?.as_table_mut()?;
    let removed = rel.remove("tags")?;
    let vals = removed
        .as_value()
        .and_then(toml_edit::Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    Some(vals)
}

fn reparse(doc: &DocumentMut) -> DocumentMut {
    doc.to_string()
        .parse::<DocumentMut>()
        .unwrap_or_else(|e| panic!("RE-PARSE FAILED (corruption): {e}\n---\n{doc}"))
}

/// A root-level array key, read off the RE-PARSED doc (the semantic check).
fn root_tags(doc: &DocumentMut) -> Vec<String> {
    doc.get("tags")
        .and_then(Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_else(|| panic!("root `tags` absent after re-parse:\n{doc}"))
}

/// Count `[[relation]]` rows on the re-parsed doc.
fn relation_count(doc: &DocumentMut) -> usize {
    doc.get("relation")
        .and_then(Item::as_array_of_tables)
        .map_or(0, |a| a.len())
}

/// Assert the root `tags = ` line renders ABOVE the first child header — i.e. the
/// insert did NOT land inside a trailing subtable. This is the textual half; the
/// re-parse `root_tags` call is the semantic half. Both must agree.
fn assert_root_tags_above_subtables(rendered: &str) {
    let tags_at = rendered
        .lines()
        .position(|l| l.starts_with("tags = ") || l.starts_with("tags="))
        .expect("root `tags` line not found in render");
    let first_header = rendered
        .lines()
        .position(|l| l.starts_with('['))
        .expect("no subtable header found");
    assert!(
        tags_at < first_header,
        "root `tags` (line {tags_at}) rendered BELOW first header (line {first_header}) — \
         it landed inside a trailing table (H1 FAIL):\n{rendered}"
    );
}

// ---------------------------------------------------------------------------
// H1 — root insert lands above trailing tables and re-parses at root.
// ---------------------------------------------------------------------------

#[test]
fn h1_sl118_relation_then_named_estimate_subtable_then_comment() {
    // AoT then a NAMED subtable then a trailing comment — the probe never tested this.
    let mut doc = load("slice/118/slice-118.toml");
    root_insert_tags(&mut doc, &["alpha", "beta"]);
    let rendered = doc.to_string();
    assert_root_tags_above_subtables(&rendered);

    let parsed = reparse(&doc);
    assert_eq!(root_tags(&parsed), vec!["alpha", "beta"]);
    // Trailing structure survives, correctly positioned.
    assert_eq!(
        relation_count(&parsed),
        2,
        "both [[relation]] rows must survive"
    );
    assert!(
        parsed.get("estimate").and_then(Item::as_table).is_some(),
        "[estimate] subtable lost"
    );
    assert_eq!(parsed["estimate"]["lower"].as_float(), Some(3.0));
    assert!(
        parsed["relationships"].get("needs").is_some(),
        "[relationships].needs lost"
    );
    // Trailing comment intact AND still after the last relation row.
    let comment_at = rendered
        .find("STRUCTURAL relations are uniform")
        .expect("trailing comment lost");
    let last_target = rendered
        .rfind("target = \"IDE-013\"")
        .expect("last relation lost");
    assert!(
        comment_at > last_target,
        "trailing comment moved above the relations"
    );
}

#[test]
fn h1_sl048_comment_after_last_relation() {
    let mut doc = load("slice/048/slice-048.toml");
    root_insert_tags(&mut doc, &["x"]);
    let rendered = doc.to_string();
    assert_root_tags_above_subtables(&rendered);

    let parsed = reparse(&doc);
    assert_eq!(root_tags(&parsed), vec!["x"]);
    // The trailing comment block after the last [[relation]] stays at the tail.
    let comment_at = rendered
        .find("Capture-surface homes")
        .expect("trailing comment block lost");
    let last_target = rendered
        .rfind("target = \"ADR-010\"")
        .expect("last relation lost");
    assert!(
        comment_at > last_target,
        "trailing comment block moved above the relations"
    );
}

#[test]
fn h1_status_scalar_insert_mirrors_apply_status() {
    // `apply_status` (dep_seq.rs:305) bails rather than insert a MISSING scalar root
    // key, on the SAME premise as `apply_tags`. Prove the scalar insert is equally
    // safe — settles the status half of H3.
    let mut doc = load("slice/118/slice-118.toml");
    doc.as_table_mut().insert("status_probe", value("draft"));
    let rendered = doc.to_string();
    let probe_at = rendered
        .lines()
        .position(|l| l.starts_with("status_probe = "))
        .expect("probe line");
    let first_header = rendered.lines().position(|l| l.starts_with('[')).unwrap();
    assert!(
        probe_at < first_header,
        "scalar root insert landed inside a trailing table"
    );
    let parsed = reparse(&doc);
    assert_eq!(
        parsed.get("status_probe").and_then(Item::as_str),
        Some("draft")
    );
}

// ---------------------------------------------------------------------------
// H2 — strip-typed-then-insert-root relocation round-trips; values preserved.
// ---------------------------------------------------------------------------

#[test]
fn h2_rfc002_live_tags_at_root_16_relations_preserved() {
    // RFC-002's live tags are now at root (SL-136 PHASE-04 migrated the corpus).
    let mut doc = load("rfc/002/rfc-002.toml");
    let expected = vec![
        "consumption-surfaces",
        "estimate",
        "program",
        "scoring",
        "value",
    ];
    // Verify the post-migration shape: root tags, no typed tags.
    let parsed = reparse(&doc);
    assert_eq!(root_tags(&parsed), expected, "live tags at root");
    assert!(
        parsed["relationships"].get("tags").is_none(),
        "no typed [relationships].tags"
    );
    assert!(
        parsed["relationships"].get("superseded_by").is_some(),
        "sibling superseded_by axis present"
    );
    assert_eq!(relation_count(&parsed), 16, "all 16 [[relation]] rows survive");

    // Exercise a root-insert edit that replaces the tags array. Root tags + 16 relations survive.
    let existing_tags = root_tags(&parsed);
    let mut all_tags = existing_tags.clone();
    all_tags.push("new-tag".to_string());
    all_tags.sort();
    root_insert_tags(
        &mut doc,
        &all_tags.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
    );
    let rendered = doc.to_string();
    assert_root_tags_above_subtables(&rendered);
    let parsed2 = reparse(&doc);
    assert_eq!(root_tags(&parsed2), all_tags, "root tags survive insert edit");
    assert_eq!(relation_count(&parsed2), 16, "relations survive edit");
    assert!(
        parsed2["relationships"].get("superseded_by").is_some(),
        "superseded_by survives edit"
    );
}

#[test]
fn h2_adr014_same_file_root_status_and_root_tags_survive_edit() {
    let mut doc = load("adr/014/adr-014.toml");
    assert_eq!(
        doc.get("status").and_then(Item::as_str),
        Some("accepted"),
        "precondition: root status"
    );
    // Post-migration (SL-136 PHASE-04): tags at root or absent; no typed tags.
    let parsed = reparse(&doc);
    assert!(
        parsed["relationships"].get("tags").is_none(),
        "no typed [relationships].tags"
    );
    assert_eq!(
        parsed.get("status").and_then(Item::as_str),
        Some("accepted"),
        "root status untouched pre-edit"
    );
    assert!(parsed["relationships"].get("superseded_by").is_some());
    assert_eq!(relation_count(&parsed), 1);

    // Exercise a root-insert edit: set a tag. Root status + relations survive.
    root_insert_tags(&mut doc, &["x"]);
    let parsed2 = reparse(&doc);
    assert_eq!(root_tags(&parsed2), vec!["x".to_string()]);
    assert!(
        parsed2["relationships"].get("tags").is_none(),
        "no typed tags after root edit"
    );
    assert_eq!(
        parsed2.get("status").and_then(Item::as_str),
        Some("accepted"),
        "root status must survive untouched"
    );
    assert!(parsed2["relationships"].get("superseded_by").is_some());
    assert_eq!(relation_count(&parsed2), 1);
}

#[test]
fn h2_pol001_same_file_root_status_and_root_tags_survive_edit() {
    let mut doc = load("policy/001/policy-001.toml");
    assert_eq!(doc.get("status").and_then(Item::as_str), Some("required"));
    // Post-migration (SL-136 PHASE-04): tags at root or absent; no typed tags.
    let parsed = reparse(&doc);
    assert!(parsed["relationships"].get("tags").is_none());
    assert_eq!(
        parsed.get("status").and_then(Item::as_str),
        Some("required"),
        "root status untouched pre-edit"
    );

    // Exercise a root-insert edit: set a tag. Root status survives.
    root_insert_tags(&mut doc, &["x"]);
    let parsed2 = reparse(&doc);
    assert_eq!(root_tags(&parsed2), vec!["x".to_string()]);
    assert!(parsed2["relationships"].get("tags").is_none());
    assert_eq!(
        parsed2.get("status").and_then(Item::as_str),
        Some("required"),
        "root status untouched after edit"
    );
}

#[test]
fn h2_spec016_root_tags_already_present_edits_in_place_aot_intact() {
    // Key ALREADY at root, followed by the `[[source]]` AoT. Re-inserting must edit in
    // place and leave the array-of-tables intact. (`[[member]]` lives in a sibling
    // members.toml, not this file.)
    let mut doc = load("spec/tech/016/spec-016.toml");
    let before: Vec<String> = doc["tags"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    assert!(
        !before.is_empty(),
        "precondition: spec-016 has live root tags"
    );
    // Idempotent re-seed (what apply_tags_set does when the key is present).
    root_insert_tags(
        &mut doc,
        &before.iter().map(String::as_str).collect::<Vec<_>>(),
    );

    let parsed = reparse(&doc);
    assert_eq!(
        root_tags(&parsed),
        before,
        "root tags preserved on in-place edit"
    );
    let sources = parsed
        .get("source")
        .and_then(Item::as_array_of_tables)
        .expect("[[source]] AoT lost");
    assert!(sources.len() >= 3, "[[source]] rows dropped");
}
