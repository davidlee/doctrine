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
//! - **OD-3** — the supersession pair stays typed, proven by constraining the array:
//!   a governance `[[relation]]` block contains ONLY `related` rows, so `supersedes`
//!   (tier-1-by-shape) can never have leaked in. This is the negative form — the corpus
//!   test does not assert the typed keys are *present* (a hand-trimmed file may omit
//!   one); the positive rendered shape is proven by the scaffold-path guard below.
//! - **Same-label row order** — within one label, `[[relation]]` rows preserve a
//!   stable, contiguous run. The bulk one-shot migrator that originally emitted
//!   per-axis is gone (SL-058 D1); edges now arrive incrementally via `doctrine link`
//!   append + strip, so the invariant guards the live authoring seam — a careless
//!   link-then-other-label-then-link CAN interleave a label's rows, and must not.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

mod common;

/// The committed corpus — `.doctrine/` beside `Cargo.toml` in the invoking tree.
fn doctrine_root() -> PathBuf {
    common::repo_root().join(".doctrine")
}

/// The freshly-built binary under test (SL-058 PHASE-01 scaffold goldens).

/// The on-disk template-source dir (the source that RustEmbed snapshots into the
/// binary). The PHASE-01 template guard scans these directly: it guards the SOURCE
/// shape, while the black-box scaffold test below proves the embedded/rendered shape.
fn templates_dir() -> PathBuf {
    common::repo_root().join("install/templates")
}

/// The numeric (`NNN`+) entity dirs directly under `tree` (skips the `NNN-slug` symlink
/// alias and any non-numeric entry). Accepts ≥3 ascii digits, not exactly 3, so a
/// 4-digit corpus (entity ≥ 1000) is scanned rather than silently dropped — the
/// `NNN-slug` alias still falls out on the all-digit guard (it carries a `-`).
fn numeric_dirs(tree: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(tree) {
        for e in rd.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            let numeric = name.len() >= 3 && name.chars().all(|c| c.is_ascii_digit());
            if numeric && e.path().is_dir() {
                out.push(e.path());
            }
        }
    }
    out.sort();
    out
}

/// The `{stem}-{NNN}.toml` files under one entity tree (`tree` is the dir holding the
/// numeric entity dirs). One construction site for every corpus invariant's iteration
/// set, so they cannot silently diverge.
fn entity_tomls(tree: &Path, stem: &str) -> Vec<PathBuf> {
    numeric_dirs(tree)
        .into_iter()
        .map(|dir| {
            let name = dir.file_name().unwrap().to_string_lossy().to_string();
            dir.join(format!("{stem}-{name}.toml"))
        })
        .collect()
}

fn slice_files() -> Vec<PathBuf> {
    entity_tomls(&doctrine_root().join("slice"), "slice")
}

/// The STRICT governance trio (adr/policy/standard): tier-1 array is `related`-ONLY.
/// RFC is deliberately excluded — it is the one governance kind in the wide `concerns`
/// set, so it may ALSO carry `references(concerns)` rows (handled separately below).
fn strict_governance_files() -> Vec<PathBuf> {
    ["adr", "policy", "standard"]
        .into_iter()
        .flat_map(|stem| entity_tomls(&doctrine_root().join(stem), stem))
        .collect()
}

fn rfc_files() -> Vec<PathBuf> {
    entity_tomls(&doctrine_root().join("rfc"), "rfc")
}

/// Every governance-kind TOML (strict trio + rfc) — the union the contiguity invariant
/// scans (RFC's `references` rows are still contiguous, so it belongs here).
fn governance_files() -> Vec<PathBuf> {
    let mut v = strict_governance_files();
    v.extend(rfc_files());
    v
}

fn backlog_files() -> Vec<PathBuf> {
    ["issue", "improvement", "chore", "risk", "idea"]
        .into_iter()
        .flat_map(|sub| entity_tomls(&doctrine_root().join("backlog").join(sub), "backlog"))
        .collect()
}

/// Every relation-bearing entity TOML in the corpus — the union the contiguity
/// invariant scans, and the same files the per-kind tests scan in subsets.
fn all_relation_files() -> Vec<PathBuf> {
    let mut v = slice_files();
    v.extend(governance_files());
    v.extend(backlog_files());
    v
}

/// A lightweight line scan of one TOML file: the index of the first `[relationships]`
/// header, the first `[[relation]]` header, every `[relationships]`-table bare key, and
/// every `[[relation]]` row's `label`. Deliberately NOT a TOML parser — we assert
/// TEXTUAL ordering (F1 is a textual-position invariant) over the raw `{{slug}}`
/// template AND the rendered entity, both of which a real parser would either reject
/// (placeholders) or normalise (exactly the row order we test). The trade is that the
/// header heuristics below are corpus-safe, not TOML-complete (see `line_view`).
struct LineView {
    first_relationships: Option<usize>,
    first_relation_array: Option<usize>,
    /// Bare keys of the `[relationships]` table (until the next header).
    relationships_keys: Vec<String>,
    /// The `label = "..."` of each `[[relation]]` row, in file order.
    relation_labels: Vec<String>,
}

fn line_view(text: &str) -> LineView {
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
        // Header detection strips a trailing comment with a corpus-safe heuristic
        // (F-C): a trailing `# …` on the header line (e.g. `[relationships]   #
        // outbound-only`) must not defeat the match, and a sub-table
        // `[relationships.x]` must NOT match. NOTE: `split('#')` is NOT TOML-aware —
        // a `#` inside a string value would be mis-cut. Sound here because the only
        // lines this `head` feeds are header matches (`[...]`), whose table names
        // never contain a `#`; string-valued lines are handled by the `key`/`label`
        // paths below, which split on `=` and never on `#`.
        let head = line.split('#').next().unwrap_or("").trim();
        if head == "[relationships]" {
            first_relationships.get_or_insert(i);
            cur = In::Relationships;
            continue;
        }
        if head == "[[relation]]" {
            first_relation_array.get_or_insert(i);
            cur = In::Relation;
            continue;
        }
        if head.starts_with('[') {
            cur = In::Other;
            continue;
        }
        // A `key = value` line inside the current table. Strip surrounding quotes so
        // a legal quoted key (`"slices" = []`) cannot evade the migrated-key scan (F-H).
        let key = line
            .split('=')
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"')
            .to_string();
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
    LineView {
        first_relationships,
        first_relation_array,
        relationships_keys,
        relation_labels,
    }
}

/// F1: if both a `[relationships]` table and a `[[relation]]` array exist, the table
/// header must come first (textually). A migrated kind with no typed leftovers (slice)
/// has no `[relationships]` table at all — vacuously fine.
fn assert_f1(path: &Path, v: &LineView) {
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
fn assert_no_migrated_key_left(path: &Path, v: &LineView, migrated: &[&str]) {
    for k in &v.relationships_keys {
        assert!(
            !migrated.contains(&k.as_str()),
            "{}: migrated tier-1 label `{k}` is still a typed [relationships] key",
            path.display()
        );
    }
}

// --- On the literal label allow-lists below (oracle independence) ---
// The legal-label and migrated-axis lists in these corpus tests are deliberately
// written as literals, NOT derived from `RELATION_RULES` (src/relation.rs, ADR-010).
// This file is the migration ORACLE: `RELATION_RULES` is the production source of
// truth that DRIVES the migration, so an expectation derived from it would agree with
// any vocabulary bug the migration itself shipped — the oracle would rubber-stamp the
// SUT instead of pinning it. (The using-doctrine.md "don't transcribe RELATION_RULES"
// guidance targets entity AUTHORS choosing edge labels, not oracle tests independently
// restating expected values.) An in-tree derivation already exists for the coupled
// view: relation.rs's own unit tests (`sources_match_shipped_accessors` et al.) check
// the table against the shipped accessors. The "kept typed" axes (needs/after/triggers,
// supersedes/superseded_by, tags) are not in `RELATION_RULES` at all — they are the
// typed-tier complement and can only be literals. Pins below say `SSoT: RELATION_RULES`
// so a vocabulary evolution is greppable from here.

#[test]
fn slice_corpus_relationships_table_holds_only_dep_seq_keys() {
    for f in slice_files() {
        let text = std::fs::read_to_string(&f).unwrap();
        let v = line_view(&text);
        assert_f1(&f, &v);
        // SL-060 (§5.3/E9) supersedes the SL-058 PHASE-02 table-absent invariant.
        // The SL-048 cut moved the STRUCTURAL axes to `[[relation]]`, but SL-060
        // RE-CARRIES a `[relationships]` table holding ONLY the dep/seq payload axes
        // (`needs`/`after`). A slice MAY now carry the table — the F-D detection gap is
        // closed POSITIVELY: every typed `[relationships]` key must be a dep/seq key, so
        // no migrated structural axis (specs/requirements/supersedes/governed_by/related)
        // can leak back as a typed key (the subset check subsumes the old
        // whole-table-absence guard). Mirrors `assert_backlog_shape`.
        for k in &v.relationships_keys {
            assert!(
                ["needs", "after"].contains(&k.as_str()),
                "{}: slice [relationships] carries non-dep/seq key `{k}` — only needs/after \
                 may be typed for a slice (SL-060)",
                f.display()
            );
        }
        // Every `[[relation]]` label must be a slice tier-1 label.
        // SSoT: RELATION_RULES rows whose `sources` include SL (literal by oracle
        // independence — see the note above slice_corpus_*).
        for label in &v.relation_labels {
            assert!(
                // SL-149 PHASE-05: specs/requirements collapsed into references.
                // SL-176 PHASE-04: `fulfils` (slice→backlog) is a slice tier-1 label.
                [
                    "references",
                    "supersedes",
                    "governed_by",
                    "related",
                    "fulfils"
                ]
                .contains(&label.as_str()),
                "{}: unexpected slice [[relation]] label `{label}`",
                f.display()
            );
        }
    }
}

#[test]
fn governance_corpus_supersession_pair_and_tags_stay_typed_relation_is_related_only() {
    // The STRICT trio (adr/policy/standard): tier-1 array is `related` ONLY.
    for f in strict_governance_files() {
        let text = std::fs::read_to_string(&f).unwrap();
        let v = line_view(&text);
        assert_f1(&f, &v);
        // OD-3 negative: `related` must NOT be a typed key (it migrated).
        // SSoT: RELATION_RULES `Related` row (ADR/POL/STD sources); literal by oracle
        // independence — see the note above slice_corpus_*.
        assert_no_migrated_key_left(&f, &v, &["related"]);
        // OD-3 POSITIVE: the supersession pair stays typed, never migrated. We assert
        // this as the array's contents: every `[[relation]]` row is `related` ONLY, so
        // `supersedes`/`superseded_by` (tier-1-by-shape) can never have leaked into the
        // array. Presence of the typed keys is NOT asserted here — a hand-trimmed file
        // may legitimately omit one — so "stays typed" is proven negatively (absent from
        // the array) rather than positively (present in the table). The freshly-rendered
        // positive shape is proven by `assert_governance_shape` on the scaffold path.
        for label in &v.relation_labels {
            assert_eq!(
                label,
                "related",
                "{}: governance [[relation]] must contain ONLY `related`, found `{label}` \
                 (supersedes/superseded_by stay typed — OD-3)",
                f.display()
            );
        }
    }
    // RFC: the one governance kind in the wide `concerns` set (SL-149 PHASE-05). It
    // may carry `references(concerns)` rows IN ADDITION TO `related` — but the
    // supersession pair still stays typed (no `supersedes` in the array).
    // SSoT: RELATION_RULES `References`/`Related` rows with RFC in `sources`; literal
    // by oracle independence — see the note above slice_corpus_*.
    for f in rfc_files() {
        let text = std::fs::read_to_string(&f).unwrap();
        let v = line_view(&text);
        assert_f1(&f, &v);
        assert_no_migrated_key_left(&f, &v, &["related"]);
        for label in &v.relation_labels {
            assert!(
                ["related", "references"].contains(&label.as_str()),
                "{}: RFC [[relation]] must be `related`/`references` only, found `{label}` \
                 (supersedes/superseded_by stay typed — OD-3)",
                f.display()
            );
        }
    }
}

#[test]
fn backlog_corpus_keeps_dep_seq_typed_migrates_cross_kind_axes() {
    for f in backlog_files() {
        let text = std::fs::read_to_string(&f).unwrap();
        let v = line_view(&text);
        assert_f1(&f, &v);
        // slices/specs/drift migrated OUT of the typed table.
        // SSoT: RELATION_RULES rows whose `sources` include the backlog kinds
        // (ISS/IMP/CHR/RSK/IDE); literal by oracle independence — see the note above
        // slice_corpus_*. needs/after/triggers are NOT in RELATION_RULES (typed-tier
        // dep/seq complement) and so are necessarily literals.
        assert_no_migrated_key_left(&f, &v, &["slices", "specs", "drift"]);
        // Every `[[relation]]` label is a backlog tier-1 label (NOT needs/after/
        // triggers — those stay typed with their per-edge payloads).
        for label in &v.relation_labels {
            assert!(
                // SL-149 PHASE-05: backlog specs collapsed into references(concerns).
                // SL-176 PHASE-04: `slices` retired → backlog→slice provenance is
                // references(originates_from); addressed-by is the derived fulfils inbound.
                ["references", "drift", "related", "governed_by"].contains(&label.as_str()),
                "{}: unexpected backlog [[relation]] label `{label}` (dep/seq axes \
                 needs/after/triggers must stay typed)",
                f.display()
            );
        }
    }
}

/// Same-label row order: within one label, the `[[relation]]` rows of any file keep a
/// contiguous, stable run. Edges arrive via `doctrine link` append (the bulk per-axis
/// migrator is gone — SL-058 D1), so this guards the append seam: a careless
/// link-then-other-label-then-link CAN interleave one label's rows. We assert that for
/// each file the rows of a given label are contiguous (no A,B,A interleave) — the
/// per-label authored order is preserved.
#[test]
fn relation_rows_of_one_label_are_contiguous() {
    for f in all_relation_files() {
        let text = std::fs::read_to_string(&f).unwrap();
        let labels = line_view(&text).relation_labels;
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

// === SL-058 PHASE-01: post-cut template shape (source guard + scaffold golden) ===
//
// Two complementary proofs over the SIX scaffold templates the cut left stale:
// - the **template guard** scans the on-disk `install/templates/*.toml` SOURCE
//   (catches a future bad template edit at the source);
// - the **black-box scaffold golden** spawns the freshly-built binary and reads
//   what `<kind> new` actually renders (catches the RustEmbed false-green and
//   proves the rendered shape — the embed must be re-snapshotted: `touch
//   src/install.rs`).
// Both reuse the same `line_view()` line scan, so the post-cut shape is asserted once
// per kind. `line_view()` is comment-skipping and placeholder-tolerant, so it reads the
// raw `{{slug}}` templates and the rendered entities identically.

/// The property: the post-cut template steers the author to the validated relation
/// seam (`doctrine link`) from a COMMENT, not via a stale typed key. Asserting the
/// `doctrine link` mention lives on a `#` comment line (not merely somewhere in the
/// bytes) is the strongest cheap proxy here — it rules out the phrase leaking from a
/// value or a key and confirms it is genuine guidance. The fully-rendered steer is
/// proven end-to-end by the scaffold-path goldens below.
fn assert_guidance_comment_present(label: &str, text: &str) {
    assert!(
        text.lines()
            .any(|l| l.trim_start().starts_with('#') && l.contains("doctrine link")),
        "{label}: post-cut template must carry the `doctrine link` guidance on a comment line:\n{text}"
    );
}

/// Slice kind (F-D): the whole `[relationships]` table is gone — NO header at all
/// (the STRUCTURAL axes survive only as commented examples).
///
/// SL-060 (D4/INV-1): the slice template now RE-CARRIES a `[relationships]` table —
/// but ONLY for the dep/seq payload axes (`needs`/`after`), the same typed shape
/// backlog carries. The SL-048 cut still holds for every STRUCTURAL axis (specs/
/// requirements/supersedes/governed_by → `[[relation]]` via `doctrine link`), so the
/// guidance comment must still steer to `doctrine link`, and no migrated structural
/// key may leak back as a typed key.
fn assert_slice_shape(label: &str, text: &str) {
    let v = line_view(text);
    assert!(
        v.first_relationships.is_some(),
        "{label}: SL-060 slice template carries a `[relationships]` table for the dep/seq axes:\n{text}"
    );
    for kept in ["needs", "after"] {
        assert!(
            v.relationships_keys.iter().any(|k| k == kept),
            "{label}: slice template must carry `{kept}` typed in [relationships] (SL-060):\n{text}"
        );
    }
    // The structural axes stay CUT — none may reappear as a typed `[relationships]` key.
    assert_no_migrated_key_left(
        Path::new(label),
        &v,
        &[
            "specs",
            "requirements",
            "supersedes",
            "governed_by",
            "related",
        ],
    );
    assert_guidance_comment_present(label, text);
}

/// Governance kinds (adr/policy/standard): `related` and `supersedes` migrated OUT
/// (SL-048 / SL-095). Only `superseded_by` stays typed. `tags` moved to root-level
/// (SL-136) — asserted via root-position grep (outside `[relationships]`).
fn assert_governance_shape(label: &str, text: &str) {
    let v = line_view(text);
    // ADR-010: supersedes stays typed; only related migrated to [[relation]].
    assert_no_migrated_key_left(Path::new(label), &v, &["related"]);
    assert!(
        v.relationships_keys.iter().any(|k| k == "supersedes"),
        "{label}: governance template must keep `supersedes` typed in [relationships]:\n{text}"
    );
    assert!(
        v.relationships_keys.iter().any(|k| k == "superseded_by"),
        "{label}: governance template must keep `superseded_by` typed in [relationships]:\n{text}"
    );
    // tags now lives at root level (SL-136). Grep for `^tags` before any header.
    let has_root_tags = text
        .lines()
        .take_while(|l| !l.trim_start().starts_with('['))
        .any(|l| l.trim_start().starts_with("tags"));
    assert!(
        has_root_tags,
        "{label}: governance template must have `tags` at root (SL-136):\n{text}"
    );
    assert_guidance_comment_present(label, text);
}

/// Backlog kinds (backlog/backlog-risk): slices/specs/drift migrated OUT; the
/// dep/seq axes needs/after/triggers stay typed with their per-edge payloads.
fn assert_backlog_shape(label: &str, text: &str) {
    let v = line_view(text);
    assert_no_migrated_key_left(Path::new(label), &v, &["slices", "specs", "drift"]);
    for kept in ["needs", "after", "triggers"] {
        assert!(
            v.relationships_keys.iter().any(|k| k == kept),
            "{label}: backlog template must keep `{kept}` typed in [relationships]:\n{text}"
        );
    }
    assert_guidance_comment_present(label, text);
}

/// Read one on-disk template source.
fn template_text(name: &str) -> String {
    let p = templates_dir().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read template {}: {e}", p.display()))
}

#[test]
fn template_source_is_post_cut_shape_kind_specific() {
    assert_slice_shape("slice.toml", &template_text("slice.toml"));
    for gov in ["adr.toml", "policy.toml", "standard.toml"] {
        assert_governance_shape(gov, &template_text(gov));
    }
    for bk in ["backlog.toml", "backlog-risk.toml"] {
        assert_backlog_shape(bk, &template_text(bk));
    }
}

/// Scaffold one entity via the real `<kind> new` verb into a throwaway project and
/// return the rendered TOML of the created file (`toml_rel` is its path under
/// `.doctrine/`). DOCTRINE_WORKER is unset — `new` is an authored write the
/// self-arm guard would refuse under a stray inherited worker var.
fn scaffold(new_args: &[&str], toml_rel: &str) -> String {
    let t = tempfile::tempdir().expect("tempdir");
    let root = t.path();
    let out = Command::new(bin())
        .args(new_args)
        .arg("-p")
        .arg(root)
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine");
    assert!(
        out.status.success(),
        "`{new_args:?}` scaffold failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read_to_string(root.join(".doctrine").join(toml_rel))
        .unwrap_or_else(|e| panic!("read scaffolded {toml_rel}: {e}"))
}

#[test]
fn scaffolded_entities_are_post_cut_shape_all_six_paths() {
    assert_slice_shape(
        "slice new",
        &scaffold(&["slice", "new", "Fixture"], "slice/001/slice-001.toml"),
    );
    assert_governance_shape(
        "adr new",
        &scaffold(&["adr", "new", "Fixture"], "adr/001/adr-001.toml"),
    );
    assert_governance_shape(
        "policy new",
        &scaffold(&["policy", "new", "Fixture"], "policy/001/policy-001.toml"),
    );
    assert_governance_shape(
        "standard new",
        &scaffold(
            &["standard", "new", "Fixture"],
            "standard/001/standard-001.toml",
        ),
    );
    assert_backlog_shape(
        "backlog new improvement",
        &scaffold(
            &["backlog", "new", "improvement", "Fixture"],
            "backlog/improvement/001/backlog-001.toml",
        ),
    );
    assert_backlog_shape(
        "backlog new risk",
        &scaffold(
            &["backlog", "new", "risk", "Fixture"],
            "backlog/risk/001/backlog-001.toml",
        ),
    );
}
