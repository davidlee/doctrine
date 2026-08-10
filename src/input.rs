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
//!
//! The same argument admits the `--body` / `--body-mode` resolution below
//! (SL-249 PHASE-08 D1): resolving a flag's raw value — including reading prose
//! from stdin — is CLI-input resolution, shared by `memory edit` and
//! `knowledge edit` with one implementation and no per-kind copy.

use std::io::{self, Read, Write};

use anyhow::{Context, Result, bail};

use crate::entity::{self, BodyMode};

/// Resolve `--body`'s raw value: `-` reads `stdin` in full (a literal one-hyphen
/// body has no way to spell itself through the flag — SL-230 PHASE-02 EX-5 — it
/// must come via stdin instead); anything else is used verbatim. Takes
/// `&mut impl Read` (not `io::stdin()` directly) so the stdin path is testable
/// without driving a real pipe (mirrors `run_surface`'s in-file idiom,
/// `memory.rs`, but propagates the read error rather than swallowing it — a
/// `--body -` that fails to read is a real error, not an advisory that degrades
/// to silence).
pub(crate) fn resolve_body(raw: &str, stdin: &mut impl Read) -> Result<String> {
    if raw == "-" {
        let mut s = String::new();
        stdin
            .read_to_string(&mut s)
            .context("Failed to read --body from stdin")?;
        Ok(s)
    } else {
        Ok(raw.to_owned())
    }
}

/// The ONE wording for "a body mode with nothing to apply it to" (SL-230
/// PHASE-05 D-P5-3, STD-001). Raised in exactly one place per verb — `memory`'s
/// `run_edit` and `knowledge`'s `run_edit` — so the CLI and the MCP
/// `memory_edit` adapter, which delegates to the former, carry the identical
/// message *by construction* rather than by two assertions happening to agree.
/// It names both spellings because one message serves both surfaces.
pub(crate) const BODY_MODE_REQUIRES_BODY: &str = "body_mode requires body — a mode with no body to \
     apply it to is never an edit (CLI: --body-mode requires --body)";

/// Resolve `--body-mode`'s raw value to the engine's [`BodyMode`]. Lives here,
/// not in `entity`: `entity` is clap-free and knows nothing of flag spellings
/// (ADR-001 — the command layer depends downward, never the reverse). Normalizes
/// like `--trust`/`--severity` (trim + lowercase) and refuses anything else with
/// a worded error rather than silently defaulting. SL-230 PHASE-03.
pub(crate) fn parse_body_mode(raw: &str) -> Result<BodyMode> {
    match raw.trim().to_lowercase().as_str() {
        "replace" => Ok(BodyMode::Replace),
        "append" => Ok(BodyMode::Append),
        other => bail!("unknown body mode {other:?} (known: replace, append)"),
    }
}

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

/// Resolve the slug: an explicit `--slug`, else derive it from the title.
/// Both paths are normalised through [`entity::derive_slug`] (IMP-005), so
/// uppercase, spaces, underscores, edge dashes, dots, and separators are all
/// folded to safe kebab-case. An explicit slug that normalises to empty bails;
/// one that exceeds `SLUG_MAX` is truncated (not rejected — matching the
/// derived path). A title that derives to nothing bails for both paths.
pub(crate) fn resolve_slug(title: &str, slug: Option<String>) -> anyhow::Result<String> {
    if let Some(s) = slug {
        let normalised = entity::derive_slug(&s);
        if normalised.is_empty() {
            bail!("--slug must not be empty after normalisation");
        }
        return Ok(truncate_slug(&normalised, SLUG_MAX));
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
    fn resolve_slug_normalises_an_explicit_flag() {
        // IMP-005: explicit --slug is normalised through derive_slug, matching
        // the derived-slug path — uppercase, spaces, underscores, edge dashes,
        // dots, and separators are all folded to safe kebab-case.
        assert_eq!(
            resolve_slug("My Title", Some("My Custom Slug".into())).unwrap(),
            "my-custom-slug"
        );
        assert_eq!(
            resolve_slug("My Title", Some("UPPER".into())).unwrap(),
            "upper"
        );
        assert_eq!(
            resolve_slug("My Title", Some("under_score".into())).unwrap(),
            "under-score"
        );
        assert_eq!(
            resolve_slug("My Title", Some("-leading".into())).unwrap(),
            "leading"
        );
        assert_eq!(
            resolve_slug("My Title", Some("trailing-".into())).unwrap(),
            "trailing"
        );
        assert_eq!(
            resolve_slug("My Title", Some("has space".into())).unwrap(),
            "has-space"
        );
        assert_eq!(
            resolve_slug("My Title", Some("tab\tslug".into())).unwrap(),
            "tab-slug"
        );
        // Path-hostile inputs are normalised to safe values by derive_slug.
        assert_eq!(
            resolve_slug("My Title", Some("../../etc".into())).unwrap(),
            "etc"
        );
        assert_eq!(resolve_slug("My Title", Some("a/b".into())).unwrap(), "ab");
        assert_eq!(
            resolve_slug("My Title", Some(".hidden".into())).unwrap(),
            "hidden"
        );
        // Normalises to empty → rejected.
        let err = resolve_slug("My Title", Some("..".into())).unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
        let err = resolve_slug("My Title", Some("!!!".into())).unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn resolve_slug_truncates_an_overlong_explicit_flag() {
        // IMP-005: overlong explicit slugs are truncated, not rejected — matching
        // the derived-slug path.
        let long = "a".repeat(SLUG_MAX + 1);
        let slug = resolve_slug("My Title", Some(long)).unwrap();
        assert!(slug.len() <= SLUG_MAX, "len {}", slug.len());
        assert!(!slug.is_empty());
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
