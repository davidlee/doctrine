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

/// Resolve the slug: an explicit `--slug`, else derive it from the title. A title
/// that derives to nothing (symbol-only) bails for an explicit `--slug`.
pub(crate) fn resolve_slug(title: &str, slug: Option<String>) -> anyhow::Result<String> {
    let slug = match slug {
        Some(s) => s,
        None => entity::derive_slug(title),
    };
    if slug.is_empty() {
        bail!("Could not derive a slug from the title; pass --slug");
    }
    Ok(slug)
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
}
