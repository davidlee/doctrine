// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine adr` — architecture decision records, doctrine's unit of governance.
//!
//! An ADR is a numeric directory under `.doctrine/adr/` holding a sister
//! `adr-NNN.toml` (structured, queried metadata: `status`, relationships) and a
//! scaffolded `adr-NNN.md` prose body, with an `NNN-slug` symlink alias — the
//! slice shape exactly (design SL-006 D1/D2), so it rides `entity::Kind` over the
//! same kind-blind engine as a top-level reserved `Fresh` kind.
//!
//! This module owns the *ADR-specific* parts — the Kind, its scaffold, and the
//! two render fns. The kind-agnostic machinery lives in `crate::entity`; the
//! shared metadata-list substrate (`Meta`, list reader/formatter) in `crate::meta`,
//! which an ADR's `adr-NNN.toml` round-trips into (its `id/slug/title/status`
//! keys match `Meta`; the `[relationships]` table is unknown-to-`Meta`, so it is
//! ignored on read and preserved on disk).

use std::path::PathBuf;

use crate::entity::{Artifact, Fileset, Kind, ScaffoldCtx};

/// Relative dir of the ADR tree inside the project root. Distinct top-level tree,
/// not nested under slice (D2 — ADRs are project-global governance).
const ADR_DIR: &str = ".doctrine/adr";

/// The top-level reserved ADR kind: `adr-NNN.toml` + `adr-NNN.md` + slug symlink.
/// `prefix` is the canonical-id stem (`ADR-007`); the file stem is `"adr"` — see
/// `meta` on why prefix ≠ stem.
#[expect(dead_code, reason = "consumed by the adr verbs in PHASE-03 (SL-006)")]
const ADR_KIND: Kind = Kind {
    dir: ADR_DIR,
    prefix: "ADR",
    scaffold: adr_scaffold,
};

// ---------------------------------------------------------------------------
// Pure: render, scaffold
// ---------------------------------------------------------------------------

/// Render `adr-<id>.toml` from the embedded template by token substitution. The
/// `id/slug/title/status` keys round-trip into `meta::Meta` (VT-3).
fn render_adr_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/adr.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", slug)
        .replace("{{title}}", title)
        .replace("{{date}}", date))
}

/// Render `adr-<id>.md` from the embedded template: `{{ref}}` (the canonical id,
/// e.g. `ADR-007`) + `{{title}}`. No YAML frontmatter (D1) — metadata lives in
/// the sister toml, not the prose.
fn render_adr_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/adr.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The ADR fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the ADR tree root — structurally `slice_scaffold` (D2).
fn adr_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/adr-{name}.toml")),
            body: render_adr_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/adr-{name}.md")),
            body: render_adr_md(ctx.canonical, ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;
    use std::path::Path;

    // --- VT-1 / VT-3: render + round-trip ---

    #[test]
    fn render_adr_toml_round_trips_to_metadata() {
        let body = render_adr_toml(7, "use-rust", "Use Rust", "2026-06-04").unwrap();
        // VT-3: the four list fields parse into meta::Meta …
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            parsed,
            Meta {
                id: 7,
                slug: "use-rust".to_string(),
                title: "Use Rust".to_string(),
                status: "proposed".to_string(),
            }
        );
        // VT-1: status seeds proposed, the date is injected, no token survives.
        assert!(body.contains("created = \"2026-06-04\""));
        assert!(!body.contains("{{"));
    }

    #[test]
    fn render_adr_toml_relationships_are_preserved_and_ignored_by_meta() {
        let body = render_adr_toml(1, "s", "T", "2026-06-04").unwrap();
        // VT-3: the [relationships] table parses as a whole document …
        let doc: toml::Value = toml::from_str(&body).unwrap();
        assert!(
            doc["relationships"]["supersedes"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            doc["relationships"]["superseded_by"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            doc["relationships"]["related"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(doc["relationships"]["tags"].as_array().unwrap().is_empty());
        // … yet Meta deserialises fine, ignoring the unknown table.
        assert!(toml::from_str::<Meta>(&body).is_ok());
    }

    #[test]
    fn render_adr_md_substitutes_ref_and_title_without_frontmatter() {
        let body = render_adr_md("ADR-007", "Use Rust").unwrap();
        assert!(body.starts_with("# ADR-007: Use Rust"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
        // VT-1: no YAML frontmatter (D1 — metadata is in the toml, not the prose).
        assert!(!body.starts_with("---"));
        assert!(!body.contains("\n---\n"));
    }

    // --- VT-2: scaffold shape ---

    #[test]
    fn adr_scaffold_lays_out_two_files_and_a_symlink() {
        let ctx = ScaffoldCtx {
            id: 7,
            canonical: "ADR-007",
            slug: "use-rust",
            title: "Use Rust",
            date: "2026-06-04",
        };
        let fileset = adr_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/adr-007.toml") && body.contains("2026-06-04")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/adr-007.md") && body.contains("ADR-007: Use Rust")));
        assert!(matches!(&fileset[2],
            Artifact::Symlink { rel_path, target }
            if rel_path == Path::new("007-use-rust") && target == "007"));
    }
}
