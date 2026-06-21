//! Leaf-tier tag normalization + shared write leaf — the sole write-path
//! chokepoint for backlog & memory, plus the lenient filter-fold. This module
//! imports NOTHING from the command/engine tier; callers (backlog, memory) sit
//! in the command tier and import this leaf.
//!
//! SL-100 PHASE-01 — extracted from `backlog.rs`.
//! SL-136 PHASE-01 — `apply_tags_set`, `fold_filter_tag`, `TAGGABLE` hoisted.

use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// TAGGABLE — the kind-prefix set that carries top-level `tags`
// ---------------------------------------------------------------------------

/// Kinds that carry a top-level `tags` array. This is the authoritative list;
/// callers consult it instead of maintaining their own copies.
#[expect(dead_code, reason = "PHASE-02 consumer")]
pub(crate) const TAGGABLE: &[&str] = &[
    "SL", "ADR", "POL", "STD", "RFC", "ISS", "IMP", "CHR", "RSK", "IDE", "ASM", "DEC", "QUE",
    "CON", "PRD", "SPEC", "REQ",
];

// ---------------------------------------------------------------------------
// fold_filter_tag — the LENIENT filter fold (never bails)
// ---------------------------------------------------------------------------

/// Lenient filter fold: trim + lowercase, for list `-t/--tag` matching. MUST NOT
/// route through [`normalize_tag`] — the filter is lenient by design so a no-match
/// input succeeds silently (zero rows, no error). Distinct from the strict write
/// chokepoint (SL-067 PHASE-01, EX-5/§5).
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
/// DISTINCT from [`fold_filter_tag`] — the filter fold is lenient by design and
/// MUST NOT route through this.
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
// apply_tags_set — the edit-preserving set-merge write core
// ---------------------------------------------------------------------------

/// Pure write core: apply a tag add/remove SET edit to a held `&mut DocumentMut`,
/// edit-preserving. No disk, no clock — the shell injects `today`.
///
/// - **Insert-if-missing**: the `tags` key absent from the top-level table → seed
///   `tags = []` (empty array) then add. Proven safe by CHR-019: `toml_edit 0.22`
///   root insert lands above all trailing `[relationships]` / `[[relation]]` /
///   named subtables.
/// - **Conditional `updated` stamp**: stamp `updated = today` ONLY if the key
///   already exists. If absent, skip the stamp (no implicit schema expansion).
/// - **No-op guard (set-compare)**: if `set(new) == set(current)`, return
///   `Ok(false)` with NO mutation (content + mtime hold). Set-compare (not
///   ordered-vec) is REQUIRED so an idempotent re-add against an UNSORTED
///   hand-authored store does not spuriously write + stamp `updated`.
/// - **The set algebra**: `new = (current ∪ adds) ∖ removes`, stored SORTED.
///   Everything OUTSIDE the array (comments, inert tables, unknown keys) is
///   preserved by `toml_edit`.
/// - Else replace `tags` with the fresh SORTED array, returning `true`.
pub(crate) fn apply_tags_set(
    doc: &mut toml_edit::DocumentMut,
    adds: &BTreeSet<String>,
    removes: &BTreeSet<String>,
    today: &str,
) -> bool {
    // Read the current tag set — seed empty array if the key is absent (insert-if-
    // missing rather than F-1 bail; proven safe by CHR-019).
    let current: BTreeSet<String> = doc
        .as_table()
        .get("tags")
        .and_then(toml_edit::Item::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut new: BTreeSet<String> = current.clone();
    new.extend(adds.iter().cloned());
    for r in removes {
        new.remove(r);
    }

    // Set-compare no-op guard: an idempotent re-add / absent-remove (or an UNSORTED
    // hand store whose set is already correct) writes nothing — mtime + content hold.
    if new == current {
        return false;
    }

    // Full sorted-array replace, preserving the doc outside the array. `BTreeSet`
    // iterates sorted, so the stored array is sorted.
    let mut fresh = toml_edit::Array::new();
    for tag in &new {
        fresh.push(tag.as_str());
    }
    // Insert-if-missing: if the key is absent, root-insert seeds it (CHR-019 safe).
    let table = doc.as_table_mut();
    table.insert("tags", toml_edit::value(fresh));
    // Conditional stamp: only stamp `updated` if the key already exists.
    if table.contains_key("updated") {
        table.insert("updated", toml_edit::value(today));
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- normalize_tag charset tests (SL-100) --

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

    // -- fold_filter_tag tests --

    #[test]
    fn fold_filter_tag_lenient() {
        // trim + lowercase, no reject — accepts what write chokepoint rejects.
        assert_eq!(fold_filter_tag("  Security "), "security");
        assert_eq!(fold_filter_tag("a b"), "a b");
    }

    // -- apply_tags_set tests (SL-136 PHASE-01) --

    #[test]
    fn apply_tags_set_insert_if_missing_seeds_empty_array() {
        // File with NO `tags` key — self-heal: seed `tags = []` then add.
        let toml = "id = 1\nkind = \"issue\"\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["x".into()].into();
        let removes = BTreeSet::new();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "2026-06-21");
        assert!(changed, "seeding a missing key + adding is a write");
        let rendered = doc.to_string();
        assert!(
            rendered.contains("tags = [\"x\"]"),
            "seeded tags with the new tag: {rendered}"
        );
    }

    #[test]
    fn apply_tags_set_noop_guard_compares_as_sets() {
        // Unsorted hand-authored store — idempotent re-add → Ok(false), no mutation.
        let toml = "id = 1\ntags = [\"b\", \"a\"]\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["a".into()].into();
        let removes = BTreeSet::new();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "2026-06-21");
        assert!(!changed, "set-equal should be a no-op");
    }

    #[test]
    fn apply_tags_set_stores_sorted_union_diff() {
        // Add "b", remove "a" → sorted union-diff.
        let toml = "id = 1\ntags = [\"a\", \"c\"]\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["b".into()].into();
        let removes: BTreeSet<String> = ["a".into()].into();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "2026-06-21");
        assert!(changed);
        let rendered = doc.to_string();
        assert!(
            rendered.contains("tags = [\"b\", \"c\"]"),
            "sorted union-diff: {rendered}"
        );
    }

    #[test]
    fn apply_tags_set_updated_stamped_if_present() {
        // File WITH `updated` key → gets stamped.
        let toml = "id = 1\ntags = []\nupdated = \"2026-01-01\"\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        let adds: BTreeSet<String> = ["x".into()].into();
        let removes = BTreeSet::new();
        apply_tags_set(&mut doc, &adds, &removes, "2026-06-21");
        let rendered = doc.to_string();
        assert!(
            rendered.contains("updated = \"2026-06-21\""),
            "updated stamped: {rendered}"
        );

        // File WITHOUT `updated` key → tags are written but no `updated` stamp.
        let toml2 = "id = 2\ntags = []\n";
        let mut doc2 = toml2.parse::<toml_edit::DocumentMut>().unwrap();
        apply_tags_set(&mut doc2, &adds, &removes, "2026-06-21");
        let rendered2 = doc2.to_string();
        assert!(
            !rendered2.contains("updated ="),
            "no updated key seeded when absent: {rendered2}"
        );
        assert!(
            rendered2.contains("tags = [\"x\"]"),
            "tags still written: {rendered2}"
        );
    }

    #[test]
    fn apply_tags_set_clear_on_untagged_is_noop() {
        // Empty tags, remove a tag that's not there → no-op.
        let toml = "id = 1\ntags = []\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        let adds = BTreeSet::new();
        let removes: BTreeSet<String> = ["x".into()].into();
        let changed = apply_tags_set(&mut doc, &adds, &removes, "2026-06-21");
        assert!(
            !changed,
            "removing a non-existent tag from empty is a no-op"
        );
    }
}
