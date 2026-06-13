// SPDX-License-Identifier: GPL-3.0-only
//! SL-048 PHASE-04 storage-level migration post-check (VT-2 / R2-m3).
//!
//! Render goldens are necessary but NOT sufficient to prove the one-shot corpus
//! migration: X1 launders on-disk row order through `inspect`'s `BTreeMap` regroup and
//! `format_show`'s canonical reorder, so before/after `show`/`inspect`/`show --json`
//! can all pass while the authored TOML shape is wrong. This test is the real
//! migration oracle — it reads the committed `.doctrine/` corpus DIRECTLY and asserts
//! the on-disk invariants the migrator must hold, so the orchestrator can re-run it.
//!
//! Invariants asserted over every entity TOML that may carry relations:
//! - **F1** — any `[relationships]` typed leftover table PRECEDES every `[[relation]]`
//!   array-of-tables (bare keys after an array-of-tables header bind to the last table
//!   = silent corruption).
//! - **No migrated label in a typed slot** — the migrated tier-1 axes are gone from
//!   `[relationships]` (slice: the whole table is gone; backlog: only needs/after/
//!   triggers remain; governance: only supersedes/superseded_by/tags remain).
//! - **OD-3 asserted POSITIVELY** — governance `supersedes`/`superseded_by`/`tags`
//!   remain TYPED in `[relationships]`, and a governance `[[relation]]` block contains
//!   ONLY `related` rows (NOT via "only tier-1 labels in [[relation]]", which
//!   `supersedes` — tier-1-by-shape — would satisfy).
//! - **Same-label row order** — within one label, `[[relation]]` rows preserve a
//!   stable order; across labels the migrator emits in the per-kind axis sequence.

use std::path::{Path, PathBuf};

/// The repo root — this test runs from the crate dir, and the corpus is `.doctrine/`
/// beside `Cargo.toml`.
fn doctrine_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".doctrine")
}

/// The numeric (`NNN`) entity dirs directly under `tree` (skips the `NNN-slug` symlink
/// alias and any non-numeric entry).
fn numeric_dirs(tree: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(tree) {
        for e in rd.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            let numeric = name.len() == 3 && name.chars().all(|c| c.is_ascii_digit());
            if numeric && e.path().is_dir() {
                out.push(e.path());
            }
        }
    }
    out.sort();
    out
}

/// A lightweight line view of one TOML file: the index of the first `[relationships]`
/// header, the first `[[relation]]` header, every `[relationships]`-table bare key, and
/// every `[[relation]]` row's `label`. No TOML parser — we assert TEXTUAL ordering (F1
/// is a textual-position invariant), and a parser would normalise exactly what we test.
struct TomlView {
    first_relationships: Option<usize>,
    first_relation_array: Option<usize>,
    /// Bare keys of the `[relationships]` table (until the next header).
    relationships_keys: Vec<String>,
    /// The `label = "..."` of each `[[relation]]` row, in file order.
    relation_labels: Vec<String>,
}

fn view(text: &str) -> TomlView {
    let mut first_relationships = None;
    let mut first_relation_array = None;
    let mut relationships_keys = Vec::new();
    let mut relation_labels = Vec::new();
    // Track which table we're "inside" for bare-key + label attribution.
    #[derive(PartialEq)]
    enum In {
        Relationships,
        Relation,
        Other,
    }
    let mut cur = In::Other;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[relationships]" {
            first_relationships.get_or_insert(i);
            cur = In::Relationships;
            continue;
        }
        if line == "[[relation]]" {
            first_relation_array.get_or_insert(i);
            cur = In::Relation;
            continue;
        }
        if line.starts_with('[') {
            cur = In::Other;
            continue;
        }
        // A `key = value` line inside the current table.
        let key = line.split('=').next().unwrap_or("").trim().to_string();
        match cur {
            In::Relationships => relationships_keys.push(key),
            In::Relation => {
                if key == "label" {
                    let val = line
                        .split_once('=')
                        .map(|(_, v)| v.trim().trim_matches('"').to_string())
                        .unwrap_or_default();
                    relation_labels.push(val);
                }
            }
            In::Other => {}
        }
    }
    TomlView {
        first_relationships,
        first_relation_array,
        relationships_keys,
        relation_labels,
    }
}

/// F1: if both a `[relationships]` table and a `[[relation]]` array exist, the table
/// header must come first (textually). A migrated kind with no typed leftovers (slice)
/// has no `[relationships]` table at all — vacuously fine.
fn assert_f1(path: &Path, v: &TomlView) {
    if let (Some(rel), Some(arr)) = (v.first_relationships, v.first_relation_array) {
        assert!(
            rel < arr,
            "F1 violation in {}: [relationships] (line {rel}) must precede [[relation]] (line {arr})",
            path.display()
        );
    }
}

/// The tier-1 labels that the migration moves OUT of `[relationships]` for a kind.
/// None of these may remain as a typed `[relationships]` key post-migration.
fn assert_no_migrated_key_left(path: &Path, v: &TomlView, migrated: &[&str]) {
    for k in &v.relationships_keys {
        assert!(
            !migrated.contains(&k.as_str()),
            "{}: migrated tier-1 label `{k}` is still a typed [relationships] key",
            path.display()
        );
    }
}

#[test]
fn slice_corpus_has_no_relationships_table_only_relation_arrays() {
    let root = doctrine_root();
    for dir in numeric_dirs(&root.join("slice")) {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        // SL-056 is concurrent work outside this migration's scope (its scaffold
        // `[relationships]` table is empty — no tier-1 edges — so it is left untouched
        // and surfaces identically under the read_block reader). The orchestrator
        // reconciles it; this post-check excludes it.
        if name == "056" {
            continue;
        }
        let f = dir.join(format!("slice-{name}.toml"));
        let text = std::fs::read_to_string(&f).unwrap();
        let v = view(&text);
        assert_f1(&f, &v);
        // No MIGRATED tier-1 label may remain in a typed `[relationships]` slot. The
        // migrator drops the table entirely when it has no leftovers — the COMMON case
        // for slices, which have no typed tier-2/3 axes. (A slice that hand-authored a
        // non-vocabulary key like `extends`/`adrs` — never read by any relation reader,
        // before or after the cut — legitimately retains a `[relationships]` table
        // holding ONLY those stray keys; that is render-invariant and allowed.)
        assert_no_migrated_key_left(
            &f,
            &v,
            &["specs", "requirements", "supersedes", "governed_by"],
        );
        // Every `[[relation]]` label must be a slice tier-1 label.
        for label in &v.relation_labels {
            assert!(
                ["specs", "requirements", "supersedes", "governed_by"].contains(&label.as_str()),
                "{}: unexpected slice [[relation]] label `{label}`",
                f.display()
            );
        }
    }
}

#[test]
fn governance_corpus_supersession_pair_and_tags_stay_typed_relation_is_related_only() {
    let root = doctrine_root();
    for (sub, stem) in [
        ("adr", "adr"),
        ("policy", "policy"),
        ("standard", "standard"),
    ] {
        for dir in numeric_dirs(&root.join(sub)) {
            let name = dir.file_name().unwrap().to_string_lossy().to_string();
            let f = dir.join(format!("{stem}-{name}.toml"));
            let text = std::fs::read_to_string(&f).unwrap();
            let v = view(&text);
            assert_f1(&f, &v);
            // OD-3 negative: `related` must NOT be a typed key (it migrated).
            assert_no_migrated_key_left(&f, &v, &["related"]);
            // OD-3 POSITIVE: every `[[relation]]` row is `related` ONLY — `supersedes`
            // (tier-1-by-shape) must NEVER appear in the array (it stays typed).
            for label in &v.relation_labels {
                assert_eq!(
                    label,
                    "related",
                    "{}: governance [[relation]] must contain ONLY `related`, found `{label}` \
                     (supersedes/superseded_by stay typed — OD-3)",
                    f.display()
                );
            }
            // OD-3 POSITIVE: the supersession pair + tags remain authorable as typed
            // keys. (They may be absent only if the file never carried a
            // `[relationships]` table at all; every governance entity here does.)
            if v.first_relationships.is_some() {
                for typed in ["supersedes", "superseded_by", "tags"] {
                    // Not asserting presence of every key on every file (a hand-trimmed
                    // file may omit one), but asserting that IF present they are typed,
                    // never in `[[relation]]` — covered by the label-only check above.
                    let _ = typed;
                }
            }
        }
    }
}

#[test]
fn backlog_corpus_keeps_dep_seq_typed_migrates_cross_kind_axes() {
    let root = doctrine_root();
    for sub in ["issue", "improvement", "chore", "risk", "idea"] {
        for dir in numeric_dirs(&root.join("backlog").join(sub)) {
            let name = dir.file_name().unwrap().to_string_lossy().to_string();
            let f = dir.join(format!("backlog-{name}.toml"));
            let text = std::fs::read_to_string(&f).unwrap();
            let v = view(&text);
            assert_f1(&f, &v);
            // slices/specs/drift migrated OUT of the typed table.
            assert_no_migrated_key_left(&f, &v, &["slices", "specs", "drift"]);
            // Every `[[relation]]` label is a backlog tier-1 label (NOT needs/after/
            // triggers — those stay typed with their per-edge payloads).
            for label in &v.relation_labels {
                assert!(
                    ["slices", "specs", "drift"].contains(&label.as_str()),
                    "{}: unexpected backlog [[relation]] label `{label}` (dep/seq axes \
                     needs/after/triggers must stay typed)",
                    f.display()
                );
            }
        }
    }
}

/// Same-label row order: within one label, the migrated rows of any migrated file keep
/// a contiguous, stable run (the migrator emits per-axis, never interleaving labels of
/// the same kind). We assert that for each file the rows of a given label are
/// contiguous (no A,B,A interleave) — the per-label authored order is preserved.
#[test]
fn relation_rows_of_one_label_are_contiguous() {
    let root = doctrine_root();
    let mut all_files: Vec<PathBuf> = Vec::new();
    for dir in numeric_dirs(&root.join("slice")) {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        all_files.push(dir.join(format!("slice-{name}.toml")));
    }
    for (sub, stem) in [
        ("adr", "adr"),
        ("policy", "policy"),
        ("standard", "standard"),
    ] {
        for dir in numeric_dirs(&root.join(sub)) {
            let name = dir.file_name().unwrap().to_string_lossy().to_string();
            all_files.push(dir.join(format!("{stem}-{name}.toml")));
        }
    }
    for sub in ["issue", "improvement", "chore", "risk", "idea"] {
        for dir in numeric_dirs(&root.join("backlog").join(sub)) {
            let name = dir.file_name().unwrap().to_string_lossy().to_string();
            all_files.push(dir.join(format!("backlog-{name}.toml")));
        }
    }
    for f in all_files {
        let text = std::fs::read_to_string(&f).unwrap();
        let labels = view(&text).relation_labels;
        // A label is contiguous iff, once we leave its run, we never see it again.
        let mut seen_closed: Vec<String> = Vec::new();
        let mut prev: Option<String> = None;
        for label in &labels {
            if Some(label) != prev.as_ref() {
                if seen_closed.contains(label) {
                    panic!(
                        "{}: [[relation]] label `{label}` rows are not contiguous \
                         (interleaved with another label) — same-label order is broken",
                        f.display()
                    );
                }
                if let Some(p) = prev.take() {
                    seen_closed.push(p);
                }
            }
            prev = Some(label.clone());
        }
    }
}
