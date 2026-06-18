//! Leaf-tier tag normalization — shared write-path chokepoint for backlog &
//! memory. This module imports NOTHING from the command/engine tier; callers
//! (backlog, memory) sit in the command tier and import this leaf.
//!
//! SL-100 PHASE-01 — extracted from `backlog.rs`.

// ---------------------------------------------------------------------------
// normalize_tag — the single WRITE chokepoint
// ---------------------------------------------------------------------------

/// Normalise ONE tag on the WRITE path — the single chokepoint that decides what
/// lands in the store (cf. `resolve_slug` for authored slugs). Trim, lowercase,
/// then validate every char is `[a-z0-9_:-]` (colon allowed for namespacing, e.g.
/// `area:backlog`); empty after trim, or any other char, is a HARD user error
/// (`bail!`) NAMING the offending token so the author can fix it.
///
/// DISTINCT from the filter-fold in `backlog::fold_filter_tag` — the filter fold
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
// Tests — charset gate
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
}
