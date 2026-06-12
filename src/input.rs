// SPDX-License-Identifier: GPL-3.0-only
//! CLI-input resolution for a freshly-created numeric entity.
//!
//! The thin-shell counterpart to `crate::meta` (list *output*): where meta reads
//! and formats authored toml, this resolves the two creation inputs a `new` verb
//! takes from the command line — the title (argument or stdin prompt) and the
//! slug (explicit `--slug` or derived from the title). Both slice and ADR `new`
//! share it (SL-006 VT-2 — one implementation, no per-kind copy).
//!
//! The stdin prompt is impurity, so it lives here in the shell, never in
//! `entity.rs` (the kind-blind scaffold engine, free of presentation). The pure
//! `derive_slug` helper stays in the engine; this only sequences arg/prompt/bail.

use std::io::{self, Write};

use anyhow::bail;

use crate::entity;

/// Resolve the title: use the argument, else prompt on stdin. Must be non-empty.
pub(crate) fn resolve_title(title: Option<String>) -> anyhow::Result<String> {
    if let Some(t) = title {
        let t = t.trim().to_string();
        if t.is_empty() {
            bail!("Title must not be empty");
        }
        return Ok(t);
    }
    let mut stdout = io::stdout();
    write!(stdout, "Title: ")?;
    stdout.flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let entered = line.trim().to_string();
    if entered.is_empty() {
        bail!("Title must not be empty");
    }
    Ok(entered)
}

/// Symlink filenames are `NNN-slug` / `requirement-NNN-slug`; the filesystem caps
/// a single name at 255 bytes. Cap the slug well under that. Both a derived slug
/// and a validated explicit `--slug` are slug-charset (ASCII, 1 byte/char), so the
/// cap in bytes equals a cap in chars — it is expressed in bytes because the FS
/// limit it defends is a byte limit.
const SLUG_MAX: usize = 100;

/// An explicit `--slug` must conform to the shape `derive_slug` already
/// guarantees: lowercase ASCII alphanumerics joined by interior `-`, no leading or
/// trailing dash (`^[a-z0-9]([a-z0-9-]*[a-z0-9])?$`). The slug is spliced straight
/// into the `NNN-slug` symlink name (e.g. `requirement.rs`), so admitting a `/`,
/// `.`, whitespace, or control character would let it traverse or break out of the
/// entity directory — a path-component injection. The charset cap is the wall.
fn slug_is_well_formed(s: &str) -> bool {
    !s.starts_with('-')
        && !s.ends_with('-')
        && !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Resolve the slug: an explicit `--slug`, else derive it from the title. An
/// explicit slug is validated to the slug charset (empty / over-`SLUG_MAX` /
/// malformed bails) then taken verbatim; a title that derives to nothing
/// (symbol-only) bails for an explicit `--slug`, and a derived slug over
/// `SLUG_MAX` is truncated rather than left to overflow the FS name cap.
pub(crate) fn resolve_slug(title: &str, slug: Option<String>) -> anyhow::Result<String> {
    if let Some(s) = slug {
        if s.is_empty() {
            bail!("--slug must not be empty");
        }
        if s.len() > SLUG_MAX {
            bail!("--slug too long ({} bytes; max {SLUG_MAX})", s.len());
        }
        if !slug_is_well_formed(&s) {
            bail!("--slug must be lowercase a-z, 0-9 and '-' (no leading/trailing '-'): {s:?}");
        }
        return Ok(s);
    }
    let derived = entity::derive_slug(title);
    if derived.is_empty() {
        bail!("Could not derive a slug from the title; pass --slug");
    }
    Ok(truncate_slug(&derived, SLUG_MAX))
}

/// Truncate a slug-charset string to `max` bytes, preferring a clean cut at a `-`.
///
/// Operates on a slug-charset string (ASCII, 1 byte/char), so byte length equals
/// char count and any byte prefix is a char boundary. Within `max` it is returned
/// unchanged. Over `max`, the longest `max`-byte prefix is taken; if that prefix
/// contains a `-` past position 0 the cut moves back to the last such `-`
/// (trimming it), so the slug ends on a word boundary. Defensive on the empty
/// edge: a non-empty input is never emptied — with no usable interior `-` (or only
/// a dash at position 0) the hard byte prefix stands.
fn truncate_slug(slug: &str, max: usize) -> String {
    if slug.len() <= max {
        return slug.to_string();
    }
    let prefix = slug.get(..max).unwrap_or(slug);
    match prefix.rfind('-') {
        Some(cut) if cut > 0 => prefix.get(..cut).unwrap_or(prefix).to_string(),
        _ => prefix.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_title_uses_and_trims_the_argument() {
        assert_eq!(
            resolve_title(Some("  My Title  ".into())).unwrap(),
            "My Title"
        );
    }

    #[test]
    fn resolve_title_rejects_an_empty_argument() {
        let err = resolve_title(Some("   ".into())).unwrap_err();
        assert!(err.to_string().contains("Title must not be empty"));
    }

    #[test]
    fn resolve_slug_prefers_the_explicit_flag() {
        assert_eq!(
            resolve_slug("My Title", Some("custom".into())).unwrap(),
            "custom"
        );
    }

    #[test]
    fn resolve_slug_derives_from_the_title_when_unset() {
        assert_eq!(resolve_slug("My Title", None).unwrap(), "my-title");
    }

    #[test]
    fn resolve_slug_bails_when_a_symbol_only_title_derives_to_nothing() {
        let err = resolve_slug("!!!", None).unwrap_err();
        assert!(err.to_string().contains("pass --slug"));
    }

    #[test]
    fn resolve_slug_rejects_an_empty_explicit_flag() {
        let err = resolve_slug("My Title", Some(String::new())).unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn resolve_slug_accepts_a_well_formed_explicit_flag_verbatim() {
        assert_eq!(
            resolve_slug("My Title", Some("a-clean-slug".into())).unwrap(),
            "a-clean-slug"
        );
        // single char is well-formed (the interior group is optional).
        assert_eq!(resolve_slug("My Title", Some("a".into())).unwrap(), "a");
    }

    #[test]
    fn resolve_slug_rejects_a_path_hostile_explicit_flag() {
        // F-1 (RV-008): the slug is spliced into the `NNN-slug` symlink name, so a
        // separator / traversal / control char must never reach the filesystem.
        for hostile in [
            "../../etc",   // traversal
            "a/b",         // separator
            "..",          // parent ref
            ".hidden",     // dot
            "has space",   // whitespace
            "UPPER",       // uppercase (not in derive_slug's charset)
            "under_score", // underscore
            "-leading",    // edge dash
            "trailing-",   // edge dash
            "tab\tslug",   // control
        ] {
            let err = resolve_slug("My Title", Some(hostile.into())).unwrap_err();
            assert!(
                err.to_string().contains("lowercase"),
                "{hostile:?} must be rejected with the charset message: {err}"
            );
        }
    }

    #[test]
    fn resolve_slug_rejects_an_overlong_explicit_flag() {
        let long = "a".repeat(SLUG_MAX + 1);
        let err = resolve_slug("My Title", Some(long)).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("too long"), "{msg}");
        assert!(msg.contains(&SLUG_MAX.to_string()), "{msg}");
    }

    #[test]
    fn resolve_slug_truncates_a_derived_slug_over_the_cap() {
        // A title that derives to an over-cap slug must come back bounded, never
        // abort, and never empty.
        let title = "word ".repeat(40); // derives to "word-word-…", ~199 bytes
        let slug = resolve_slug(&title, None).unwrap();
        assert!(slug.len() <= SLUG_MAX, "len {}", slug.len());
        assert!(!slug.is_empty());
    }

    #[test]
    fn truncate_slug_returns_a_within_cap_slug_unchanged() {
        assert_eq!(truncate_slug("short-slug", SLUG_MAX), "short-slug");
        assert_eq!(truncate_slug("abc", 3), "abc");
    }

    #[test]
    fn truncate_slug_cuts_at_the_last_dash_within_the_prefix() {
        // 10-byte cap; "alpha-beta-gamma" → prefix "alpha-beta" → cut at last dash.
        assert_eq!(truncate_slug("alpha-beta-gamma", 10), "alpha");
    }

    #[test]
    fn truncate_slug_hard_cuts_on_a_boundary_when_no_usable_dash() {
        // No interior dash in the prefix ⇒ the hard byte prefix stands.
        assert_eq!(truncate_slug("supercalifragilistic", 5), "super");
        // A leading dash at position 0 is not usable; hard prefix stands.
        assert_eq!(truncate_slug("-leadingdash", 4), "-lea");
    }

    #[test]
    fn truncate_slug_never_empties_a_non_empty_slug() {
        assert!(!truncate_slug("aaaaaaaaaa", 3).is_empty());
        assert!(!truncate_slug("a-b-c-d-e-f", 4).is_empty());
    }
}
