// SPDX-License-Identifier: GPL-3.0-only
//! `heresy slice` — create, list, and add design-doc siblings to slices,
//! Heresiarch's unit of change.
//!
//! A slice is a numeric directory under `.doctrine/slice/` holding a sister
//! TOML (structured metadata) and a scaffolded markdown prose body, with a
//! `<id>-<slug>` symlink as a human alias (slices-spec). A design-doc sibling is
//! a single prose `design.md` under an existing slice dir.
//!
//! Both are `entity::Kind` values over one kind-blind engine: the slice is a
//! top-level reserved 2-file-plus-symlink kind, the design doc a non-reserved
//! single-file sub-artefact. This module owns the *slice-specific* parts — the
//! two Kinds and their scaffolds, the `Meta` reader, list formatting, and thin
//! CLI wiring; the kind-agnostic machinery lives in `crate::entity`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::entity::{self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseMode, ScaffoldCtx};

/// Relative dir of the slice tree inside the project root.
const SLICE_DIR: &str = ".doctrine/slice";

/// The top-level reserved slice kind: toml + md + slug symlink.
const SLICE_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: "SL",
    mode: MaterialiseMode::AllocateFreshEntity,
    scaffold: slice_scaffold,
};

/// The non-reserved design-doc sibling: one `design.md` under an existing slice.
const DESIGN_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: "SL",
    mode: MaterialiseMode::CreateInExistingEntity,
    scaffold: design_scaffold,
};

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// The fields a reader extracts from `slice-<id>.toml`. Unknown keys (the
/// `[relationships]` table, future sections) are ignored and preserved on disk.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Meta {
    id: u32,
    slug: String,
    title: String,
    status: String,
}

// ---------------------------------------------------------------------------
// Pure: render, scaffolds, list
// ---------------------------------------------------------------------------

/// Render `slice-<id>.toml` from the embedded template by token substitution.
fn render_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/slice.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", slug)
        .replace("{{title}}", title)
        .replace("{{date}}", date))
}

/// Render `slice-<id>.md` from the embedded template by token substitution.
fn render_md(title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/slice.md")?.replace("{{title}}", title))
}

/// Render `design.md` from the embedded template: `{{ref}}` (parent canonical
/// id) + `{{title}}` (parent title) — a design doc has no id/slug of its own.
fn render_design(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/design.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The slice fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the slice tree root (the symlink sits beside the numeric dir).
fn slice_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let name = format!("{:03}", ctx.id);
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/slice-{name}.toml")),
            body: render_toml(ctx.id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/slice-{name}.md")),
            body: render_md(ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

/// The design-doc fileset: one prose `design.md` under the parent slice dir.
fn design_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let name = format!("{:03}", ctx.id);
    Ok(vec![Artifact::File {
        rel_path: PathBuf::from(format!("{name}/design.md")),
        body: render_design(ctx.canonical_id, ctx.title)?,
    }])
}

/// Sort by id and, when a status is given, keep only matching rows.
fn sort_and_filter(mut rows: Vec<Meta>, status: Option<&str>) -> Vec<Meta> {
    rows.retain(|m| status.is_none_or(|s| m.status == s));
    rows.sort_by_key(|m| m.id);
    rows
}

/// Format slice rows as aligned `id  status  slug  title` lines.
fn format_list(rows: &[Meta]) -> String {
    let status_w = rows.iter().map(|m| m.status.len()).max().unwrap_or(0);
    let slug_w = rows.iter().map(|m| m.slug.len()).max().unwrap_or(0);
    let lines: Vec<String> = rows
        .iter()
        .map(|m| {
            format!(
                "{:03}  {:<status_w$}  {:<slug_w$}  {}",
                m.id, m.status, m.slug, m.title
            )
        })
        .collect();
    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    }
}

// ---------------------------------------------------------------------------
// Imperative: clock, the slice-specific reader
// ---------------------------------------------------------------------------

/// Today as `YYYY-MM-DD` (UTC). The clock lives only in the shell; the pure
/// layer takes the date as a parameter (slices-spec § Architecture).
fn today() -> String {
    let d = time::OffsetDateTime::now_utc().date();
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

/// Parse the `Meta` of a single slice by id.
fn read_meta(slice_root: &Path, id: u32) -> anyhow::Result<Meta> {
    let name = format!("{id:03}");
    let path = slice_root.join(&name).join(format!("slice-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Slice {name} not found at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// Read and parse every `slice-<id>.toml` under `slice_root`.
fn read_metas(slice_root: &Path) -> anyhow::Result<Vec<Meta>> {
    let mut metas = Vec::new();
    for id in entity::scan_ids(slice_root)? {
        metas.push(read_meta(slice_root, id)?);
    }
    Ok(metas)
}

// ---------------------------------------------------------------------------
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// Resolve the title: use the argument, else prompt on stdin. Must be non-empty.
fn resolve_title(title: Option<String>) -> anyhow::Result<String> {
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

/// `heresy slice new`.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = resolve_title(title)?;
    // Slug *resolution policy* is slice-specific (a design doc has no slug);
    // only the pure `derive_slug` helper lives in the engine.
    let slug = match slug {
        Some(s) => s,
        None => entity::derive_slug(&title),
    };
    if slug.is_empty() {
        bail!("Could not derive a slug from the title; pass --slug");
    }
    let date = today();
    let out = entity::materialise(
        &SLICE_KIND,
        &LocalFs,
        &root,
        &Inputs {
            existing_id: None,
            slug: &slug,
            title: &title,
            date: &date,
        },
    )?;

    writeln!(
        io::stdout(),
        "Created slice {:03}: {}",
        out.id,
        out.dir.display()
    )?;
    Ok(())
}

/// `heresy slice design <id>` — scaffold `design.md` into an existing slice.
pub(crate) fn run_design(path: Option<PathBuf>, id: u32) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    // The design doc inherits its parent's title (the only context its template
    // needs); reading it confirms the parent exists before we materialise.
    let meta = read_meta(&slice_root, id)?;
    let date = today();
    let out = entity::materialise(
        &DESIGN_KIND,
        &LocalFs,
        &root,
        &Inputs {
            existing_id: Some(id),
            slug: "",
            title: &meta.title,
            date: &date,
        },
    )?;

    writeln!(
        io::stdout(),
        "Created design doc: {}",
        out.dir.join("design.md").display()
    )?;
    Ok(())
}

/// `heresy slice list`.
pub(crate) fn run_list(path: Option<PathBuf>, status: Option<&str>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let rows = sort_and_filter(read_metas(&slice_root)?, status);

    let mut out = io::stdout();
    write!(out, "{}", format_list(&rows))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: u32, status: &str, slug: &str, title: &str) -> Meta {
        Meta {
            id,
            slug: slug.to_string(),
            title: title.to_string(),
            status: status.to_string(),
        }
    }

    /// Materialise a slice the way `run_new` does, for behaviour-preservation
    /// tests (the slice-001 gate).
    fn make_slice(root: &Path, slug: &str, title: &str, date: &str) -> entity::Materialised {
        entity::materialise(
            &SLICE_KIND,
            &LocalFs,
            root,
            &Inputs {
                existing_id: None,
                slug,
                title,
                date,
            },
        )
        .unwrap()
    }

    // --- render / round-trip ---

    #[test]
    fn render_toml_round_trips_to_metadata() {
        let body = render_toml(7, "my-slug", "My Title", "2026-06-03").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed, meta(7, "proposed", "my-slug", "My Title"));
        // injected date survives
        assert!(body.contains("created = \"2026-06-03\""));
    }

    #[test]
    fn render_md_substitutes_title() {
        let body = render_md("My Title").unwrap();
        assert!(body.contains("My Title"));
        assert!(!body.contains("{{title}}"));
    }

    #[test]
    fn render_design_substitutes_ref_and_title() {
        let body = render_design("SL-003", "My Title").unwrap();
        assert!(body.contains("Design SL-003: My Title"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
    }

    // --- scaffolds ---

    #[test]
    fn slice_scaffold_lays_out_two_files_and_a_symlink() {
        let ctx = ScaffoldCtx {
            id: 3,
            canonical_id: "SL-003",
            slug: "vendor-skills",
            title: "Vendor skills",
            date: "2026-06-03",
        };
        let fileset = slice_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("003/slice-003.toml") && body.contains("2026-06-03")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("003/slice-003.md") && body.contains("Vendor skills")));
        assert!(matches!(&fileset[2],
            Artifact::Symlink { rel_path, target }
            if rel_path == Path::new("003-vendor-skills") && target == "003"));
    }

    #[test]
    fn design_scaffold_is_a_single_file_no_symlink() {
        let ctx = ScaffoldCtx {
            id: 3,
            canonical_id: "SL-003",
            slug: "",
            title: "Vendor skills",
            date: "2026-06-03",
        };
        let fileset = design_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 1);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("003/design.md") && body.contains("Design SL-003: Vendor skills")));
    }

    // --- behaviour preservation: a materialised slice is well-formed ---

    #[test]
    fn materialise_writes_well_formed_slice() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let s = make_slice(root, "my-slug", "My Title", "2026-06-03");
        let slice_root = root.join(SLICE_DIR);

        assert_eq!(s.id, 1);
        assert!(slice_root.join("001").is_dir());
        assert!(slice_root.join("001/slice-001.toml").is_file());
        assert!(slice_root.join("001/slice-001.md").is_file());
        assert_eq!(
            fs::read_link(slice_root.join("001-my-slug")).unwrap(),
            Path::new("001")
        );

        let toml_body = fs::read_to_string(slice_root.join("001/slice-001.toml")).unwrap();
        assert!(toml_body.contains("id = 1"));
        assert!(toml_body.contains("2026-06-03"));
    }

    // --- list ---

    #[test]
    fn sort_and_filter_orders_by_id_and_filters_status() {
        let rows = vec![
            meta(2, "proposed", "b", "Two"),
            meta(1, "done", "a", "One"),
            meta(3, "proposed", "c", "Three"),
        ];

        let all = sort_and_filter(rows.clone(), None);
        assert_eq!(all.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1, 2, 3]);

        let proposed = sort_and_filter(rows, Some("proposed"));
        assert_eq!(
            proposed.iter().map(|m| m.id).collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn format_list_renders_aligned_rows() {
        let rows = vec![
            meta(1, "started", "add-skill-removal", "Add skill removal"),
            meta(2, "proposed", "vendor-skills", "Vendor skills"),
        ];
        let out = format_list(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        // "started" (7) pads to the width of "proposed" (8) for column alignment.
        assert!(lines[0].starts_with("001  started   add-skill-removal"));
        assert!(lines[0].ends_with("Add skill removal"));
        assert!(lines[1].starts_with("002  proposed  vendor-skills"));
    }

    #[test]
    fn read_metas_round_trips_a_created_slice() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-03");

        let metas = read_metas(&root.join(SLICE_DIR)).unwrap();
        assert_eq!(metas, vec![meta(1, "proposed", "my-slug", "My Title")]);
    }

    // --- design verb: non-reserved sibling over an existing slice ---

    #[test]
    fn design_materialises_under_an_existing_slice_with_no_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-03");
        let slice_root = root.join(SLICE_DIR);

        let out = entity::materialise(
            &DESIGN_KIND,
            &LocalFs,
            root,
            &Inputs {
                existing_id: Some(1),
                slug: "",
                title: "My Title",
                date: "2026-06-03",
            },
        )
        .unwrap();

        assert_eq!(out.id, 1);
        let body = fs::read_to_string(slice_root.join("001/design.md")).unwrap();
        assert!(body.contains("Design SL-001: My Title"));
        // no second numeric dir, no extra symlink
        assert!(!slice_root.join("002").exists());
    }

    #[test]
    fn design_refuses_to_clobber_an_existing_doc() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-03");
        let slice_root = root.join(SLICE_DIR);
        fs::write(slice_root.join("001/design.md"), "hand-written").unwrap();

        let err = entity::materialise(
            &DESIGN_KIND,
            &LocalFs,
            root,
            &Inputs {
                existing_id: Some(1),
                slug: "",
                title: "My Title",
                date: "2026-06-03",
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Refusing to overwrite"));
        assert_eq!(
            fs::read_to_string(slice_root.join("001/design.md")).unwrap(),
            "hand-written"
        );
    }
}
