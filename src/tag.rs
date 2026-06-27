//! Leaf-tier tag normalization — shared write-path chokepoint for backlog &
//! memory. This module imports NOTHING from the command/engine tier; callers
//! (backlog, memory) sit in the command tier and import this leaf.
//!
//! SL-100 PHASE-01 — extracted from `backlog.rs`.

use std::collections::BTreeSet;

use anyhow::Context;

// ---------------------------------------------------------------------------
// TAGGABLE — entity kinds that accept tags
// ---------------------------------------------------------------------------

/// Entity kind prefixes that accept tags (SL-136).
pub(crate) const TAGGABLE: &[&str] = &[
    "SL", "ADR", "POL", "STD", "RFC", "ISS", "IMP", "CHR", "RSK", "IDE", "ASM", "CM", "DEC", "QUE",
    "CON", "EVD", "HYP", "PRD", "SPEC", "REQ", "REV",
];

// ---------------------------------------------------------------------------
// fold_filter_tag — lenient filter-fold (distinct from write normalize_tag)
// ---------------------------------------------------------------------------

/// Lenient fold for `-t/--tag` filter inputs — trim + lowercase only,
/// deliberately DISTINCT from the write-path [`normalize_tag`] so a mixed-case
/// input round-trips the lowercased store silently (a no-match filter succeeds
/// with zero rows, never errors).
pub(crate) fn fold_filter_tag(raw: &str) -> String {
    raw.trim().to_lowercase()
}

// ---------------------------------------------------------------------------
// normalize_tag — the single WRITE chokepoint
// ---------------------------------------------------------------------------

/// Normalise ONE tag on the WRITE path — the single chokepoint that decides what
/// lands in the store (cf. `resolve_slug` for authored slugs). Trim, lowercase,
/// then validate every char is `[a-z0-9_:-]` (colon allowed for namespacing, e.g.
/// `area:backlog`); empty after trim, or any other char, is a HARD user error
/// (`bail!`) NAMING the offending token so the author can fix it.
///
/// DISTINCT from the filter-fold [`fold_filter_tag`] — the filter fold
/// is lenient by design and MUST NOT route through this.
pub(crate) fn normalize_tag(raw: &str) -> anyhow::Result<String> {
    let tag = raw.trim().to_lowercase();
    if tag.is_empty() {
        anyhow::bail!("empty tag `{raw}` — tags must be non-empty `[a-z0-9_:-]`");
    }
    if !tag
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | ':' | '-'))
    {
        anyhow::bail!(
            "invalid tag `{raw}` — tags must be `[a-z0-9_:-]` (lowercased, e.g. `area:backlog`)"
        );
    }
    Ok(tag)
}

// ---------------------------------------------------------------------------
// apply_tags_set — shared write-core for tag set-replace
// ---------------------------------------------------------------------------

/// Pure write core: apply a tag add/remove SET edit to a held `&mut DocumentMut`,
/// edit-preserving. No disk, no clock — the shell injects `today`.
///
/// - **Self-heal**: if the `tags` key is absent, seeds `tags = []` and continues
///   (no `bail!`). CHR-019 proved root `insert` is safe in `toml_edit` 0.22 — no
///   tail-subtable corruption.
/// - **The set algebra**: `new = (current ∪ adds) ∖ removes`, stored SORTED.
///   The current set is read off the existing array verbatim (a hand-authored
///   store may be unsorted — that is fine, the no-op guard compares as SETS).
/// - **No-op guard (set-compare)**: if `set(new) == set(current)`, return
///   `Ok(false)` with NO mutation (content + mtime hold). Set-compare (not
///   ordered-vec) is REQUIRED so an idempotent re-add against an UNSORTED
///   hand-authored store does not spuriously write + stamp `updated`.
/// - Else replace `tags` with the fresh SORTED array. Only stamp
///   `updated = today` if the `updated` key already exists. Everything OUTSIDE
///   the array (comments, inert tables, unknown keys) is preserved by
///   `toml_edit`.
pub(crate) fn apply_tags_set(
    doc: &mut toml_edit::DocumentMut,
    adds: &BTreeSet<String>,
    removes: &BTreeSet<String>,
    today: &str,
) -> anyhow::Result<bool> {
    // Self-heal: seed tags = [] when absent.
    {
        let table = doc.as_table_mut();
        if table.get("tags").is_none() {
            table.insert("tags", toml_edit::value(toml_edit::Array::new()));
        }
    }
    let array = doc
        .as_table()
        .get("tags")
        .and_then(toml_edit::Item::as_array)
        .context("malformed backlog item: tags key exists but is not an array")?;
    let current: BTreeSet<String> = array
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    let mut new: BTreeSet<String> = current.clone();
    new.extend(adds.iter().cloned());
    for r in removes {
        new.remove(r);
    }

    // Set-compare no-op guard.
    if new == current {
        return Ok(false);
    }

    // Full sorted-array replace.
    let mut fresh = toml_edit::Array::new();
    for tag in &new {
        fresh.push(tag.as_str());
    }
    {
        let table = doc.as_table_mut();
        table.insert("tags", toml_edit::value(fresh));
        // Only stamp `updated` if the key already exists.
        if table.contains_key("updated") {
            table.insert("updated", toml_edit::value(today));
        }
    }
    Ok(true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_trim_and_lowercase() {
        assert_eq!(normalize_tag("  Area:Backlog ").unwrap(), "area:backlog");
    }

    #[test]
    fn accepts_valid_charset() {
        // colon namespacing, underscore, hyphen, digits all accepted.
        assert_eq!(normalize_tag("a_b-1:c").unwrap(), "a_b-1:c");
    }

    #[test]
    fn rejects_invalid_chars() {
        for bad in ["a b", "a@b"] {
            let err = normalize_tag(bad).unwrap_err().to_string();
            assert!(
                err.contains(bad),
                "the reject names the offending token: {err}"
            );
        }
    }

    #[test]
    fn rejects_empty_after_trim() {
        assert!(
            normalize_tag("   ").is_err(),
            "empty-after-trim is rejected"
        );
    }

    // ── apply_tags_set ──────────────────────────────────────────────

    #[test]
    fn apply_tags_set_insert_if_missing_seeds_empty_array() {
        let text = "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\n";
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["x".into()].into();
        let removes: BTreeSet<String> = BTreeSet::new();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "today").unwrap();
        assert!(changed, "should write");
        let out = doc.to_string();
        assert!(
            out.contains("tags = [\"x\"]"),
            "tags seeded and populated: {out}"
        );
    }

    #[test]
    fn apply_tags_set_noop_guard_compares_as_sets() {
        let text = "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\ntags = [\"b\", \"a\"]\n";
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["a".into()].into();
        let removes: BTreeSet<String> = BTreeSet::new();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "today").unwrap();
        assert!(!changed, "no-op when set already contains tag");
        assert!(
            doc.to_string().contains("tags = [\"b\", \"a\"]"),
            "unsorted store unchanged on no-op"
        );
    }

    #[test]
    fn apply_tags_set_stores_sorted_union_diff() {
        let text = "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\ntags = [\"a\", \"c\"]\n";
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["b".into()].into();
        let removes: BTreeSet<String> = ["a".into()].into();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "today").unwrap();
        assert!(changed, "should write");
        let out = doc.to_string();
        assert!(
            out.contains("tags = [\"b\", \"c\"]"),
            "sorted union-diff: {out}"
        );
    }

    #[test]
    fn apply_tags_set_updated_stamped_if_present() {
        // File with updated key — stamp it.
        let text =
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\nupdated = \"old\"\ntags = []\n";
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["x".into()].into();
        let removes: BTreeSet<String> = BTreeSet::new();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "today").unwrap();
        assert!(changed, "should write");
        let out = doc.to_string();
        assert!(!out.contains("updated = \"old\""), "updated stamped: {out}");
        assert!(
            out.contains("updated = \"today\""),
            "updated stamped with today: {out}"
        );

        // File without updated key — no updated written.
        let text2 = "id = 2\nslug = \"b\"\ntitle = \"B\"\nkind = \"issue\"\ntags = []\n";
        let mut doc2 = text2.parse::<toml_edit::DocumentMut>().unwrap();
        let changed2 = apply_tags_set(&mut doc2, &adds, &removes, "today").unwrap();
        assert!(changed2, "should write");
        let out2 = doc2.to_string();
        assert!(
            !out2.contains("updated"),
            "no updated key written when absent: {out2}"
        );
    }

    #[test]
    fn apply_tags_set_clear_on_untagged_is_noop() {
        let text = "id = 1\nslug = \"a\"\ntitle = \"A\"\nkind = \"issue\"\ntags = []\n";
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = BTreeSet::new();
        let removes: BTreeSet<String> = ["x".into()].into();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "today").unwrap();
        assert!(!changed, "remove from empty is no-op");
    }

    use crate::kinds;

    // ── fold_filter_tag ─────────────────────────────────────────────

    #[test]
    fn fold_filter_tag_lenient() {
        assert_eq!(fold_filter_tag("  Security "), "security");
        // The lenient fold accepts what the write chokepoint rejects.
        assert_eq!(fold_filter_tag("a b"), "a b");
    }

    /// SL-161 PHASE-01: every record kind (ASM, DEC, QUE, CON) must be
    /// in TAGGABLE so tagging works on knowledge records.
    #[test]
    fn record_kinds_are_taggable() {
        for prefix in kinds::RECORD {
            assert!(TAGGABLE.contains(prefix), "{prefix} missing from TAGGABLE");
        }
    }
}
