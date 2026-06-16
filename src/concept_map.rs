// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine concept-map` — create, list, and show concept maps, doctrine's
//! DSL-driven relationship-diagram entity.
//!
//! A concept map is a numeric directory under `.doctrine/concept-map/` holding a
//! sister TOML (structured metadata including a raw DSL block) and a scaffolded
//! markdown prose body, with a `<id>-<slug>` symlink as a human alias. It is an
//! `entity::Kind` over the kind-blind engine — this module owns the
//! concept-map-specific parts (the Kind, scaffold, and thin CLI wiring); the
//! kind-agnostic machinery lives in `crate::entity`, and the shared
//! metadata-list substrate (`Meta`, list reader/formatter) in `crate::meta`.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Serialize;

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::listing::{self, Format, ListArgs};
use crate::meta::{self, Meta};
use crate::tomlfmt::toml_string;

/// Relative dir of the concept-map tree inside the project root.
const CONCEPT_MAP_DIR: &str = ".doctrine/concept-map";

/// Statuses for concept maps (SL-074 § Memory: simple drafting lifecycle).
const CONCEPT_MAP_STATUSES: &[&str] = &["draft", "active", "done", "abandoned"];

/// The `concept-map list` hide-set: terminal (`done`, `abandoned`) drop from the
/// default list. `--all` or any explicit `--status` reveals them.
fn is_hidden(status: &str) -> bool {
    matches!(status, "done" | "abandoned")
}

/// The top-level reserved concept-map kind: toml + md + slug symlink.
pub(crate) const CONCEPT_MAP_KIND: Kind = Kind {
    dir: CONCEPT_MAP_DIR,
    prefix: "CM",
    scaffold: concept_map_scaffold,
};

// ---------------------------------------------------------------------------
// Pure: render, scaffolds
// ---------------------------------------------------------------------------

/// Render `concept-map-<id>.toml` from the embedded template by token substitution.
fn render_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/concept-map.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `concept-map-<id>.md` from the embedded template by token substitution.
fn render_md(title: &str, id: u32) -> anyhow::Result<String> {
    let canonical = crate::listing::canonical_id("CM", id);
    Ok(crate::install::asset_text("templates/concept-map.md")?
        .replace("{{title}}", title)
        .replace("{{id}}", &canonical))
}

/// The concept-map fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the concept-map tree root (the symlink sits beside the numeric dir).
fn concept_map_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    let stem = format!("concept-map-{name}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{stem}.toml")),
            body: render_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{stem}.md")),
            body: render_md(ctx.title, id)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// Shell: run_new, run_list, run_show
// ---------------------------------------------------------------------------

/// `doctrine concept-map new` — allocate the next id and scaffold a concept map.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let trunk_ids = crate::git::trunk_entity_ids(&root, CONCEPT_MAP_KIND.dir)?;
    let out = entity::materialise(
        &CONCEPT_MAP_KIND,
        &LocalFs,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
        &trunk_ids,
    )?;

    let id = out
        .eid
        .numeric_id()
        .context("concept-map kind must yield a numeric id")?;
    writeln!(
        io::stdout(),
        "Created concept map CM-{id:03}: {}",
        out.dir.display()
    )?;
    Ok(())
}

/// The full `concept-map-NNN.toml` read as data for `show` — `Meta`'s four list
/// fields plus dates, description, and the raw DSL block.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
struct ConceptMapDoc {
    id: u32,
    slug: String,
    title: String,
    status: String,
    created: String,
    updated: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    dsl: String,
}

/// Parse a concept-map reference — `CM-001`, `cm-1`, or the bare id `1` — to its
/// numeric id. The prefix is optional and case-insensitive; the id may be padded.
fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("CM-")
        .or_else(|| reference.strip_prefix("cm-"))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        format!("not a concept-map reference: `{reference}` (expected `CM-001` or `1`)")
    })
}

/// Read one concept-map's `concept-map-NNN.toml` (as data) and
/// `concept-map-NNN.md` (body).
fn read_concept_map(cm_root: &Path, id: u32) -> anyhow::Result<(ConceptMapDoc, String, String)> {
    let name = format!("{id:03}");
    let stem = format!("concept-map-{name}");
    let toml_path = cm_root.join(&name).join(format!("{stem}.toml"));
    let md_path = cm_root.join(&name).join(format!("{stem}.md"));
    let toml_text = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;
    let body = std::fs::read_to_string(&md_path).unwrap_or_default();
    let doc: ConceptMapDoc = toml::from_str(&toml_text)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    Ok((doc, toml_text, body))
}

/// `doctrine concept-map show <ref>` — display a concept map's metadata, DSL,
/// and optionally edge/node tables.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
    edges: bool,
    nodes: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(reference)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let (doc, _toml_text, body) = read_concept_map(&cm_root, id)?;

    let out = match format {
        Format::Table => format_show(&doc, &body, edges, nodes),
        Format::Json => show_json(&doc, &body, edges, nodes)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Render the table-format show output for a concept map.
fn format_show(doc: &ConceptMapDoc, body: &str, edges: bool, nodes: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!(
        "CM-{:03}\n\
     {}\n\n\
     Status:    {}\n\
     Created:   {}\n\
     Updated:   {}\n\
     Slug:      {}",
        doc.id, doc.title, doc.status, doc.created, doc.updated, doc.slug
    ));
    if !doc.description.is_empty() {
        parts.push(format!("\nDescription: {}", doc.description));
    }
    if !body.trim().is_empty() {
        parts.push(format!("\n\n---\n\n{body}"));
    }
    if !doc.dsl.trim().is_empty() {
        parts.push(format!("\n\n---\nDSL:\n{}", doc.dsl));
    }
    if edges {
        parts.push("\nEdges: (table available in PHASE-02)".to_string());
    }
    if nodes {
        parts.push("Nodes: (table available in PHASE-02)".to_string());
    }
    parts.concat()
}

/// Render JSON show output.
fn show_json(doc: &ConceptMapDoc, body: &str, edges: bool, nodes: bool) -> anyhow::Result<String> {
    let mut value = serde_json::json!({
      "id": crate::listing::canonical_id("CM", doc.id),
      "slug": doc.slug,
      "title": doc.title,
      "status": doc.status,
      "created": doc.created,
      "updated": doc.updated,
      "description": doc.description,
      "dsl": doc.dsl,
      "body": body,
    });
    if edges && let serde_json::Value::Object(ref mut map) = value {
        map.insert(
            "edges".into(),
            serde_json::json!("table available in PHASE-02"),
        );
    }
    if nodes && let serde_json::Value::Object(ref mut map) = value {
        map.insert(
            "nodes".into(),
            serde_json::json!("table available in PHASE-02"),
        );
    }
    serde_json::to_string_pretty(&value).context("failed to serialize concept-map show JSON")
}

// ---------------------------------------------------------------------------
// list — the read surface
// ---------------------------------------------------------------------------

/// The inner list pipeline: read, filter, sort, render.
fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    listing::validate_statuses(&args.status, CONCEPT_MAP_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let cm_root = root.join(CONCEPT_MAP_DIR);
    let mut metas = listing::retain(
        meta::read_metas(&cm_root, "concept-map")?,
        &filter,
        is_hidden,
        key,
    );
    metas.sort_by_key(|m| m.id);
    let rows = metas
        .into_iter()
        .map(|m| ConceptMapRow {
            id: m.id,
            status: m.status,
            slug: m.slug,
            title: m.title,
        })
        .collect::<Vec<_>>();
    match format {
        Format::Table => {
            let sel = listing::select_columns(
                CONCEPT_MAP_COLUMNS,
                CONCEPT_MAP_DEFAULT,
                columns.as_deref(),
            )?;
            Ok(listing::render_columns(&rows, &sel, render))
        }
        Format::Json => listing::json_envelope("concept-map", &rows),
    }
}

/// `doctrine concept-map list` — the read surface: prefixed `CM-` ids, a header,
/// the shared filter flags, the `{done, abandoned}` hide-set by default, sorted
/// by id.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(&root, args)?)?;
    Ok(())
}

/// A concept-map row for list rendering. `Serialize` for JSON; cell extractors
/// for the table.
#[derive(Debug, Clone, Serialize)]
struct ConceptMapRow {
    #[serde(serialize_with = "serialize_cm_id")]
    id: u32,
    status: String,
    slug: String,
    title: String,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde serialize_with contract requires a reference"
)]
fn serialize_cm_id<S: serde::Serializer>(id: &u32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&crate::listing::canonical_id("CM", *id))
}

/// The `FilterFields` projection for a `Meta` — used by `listing::retain`.
fn key(m: &Meta) -> listing::FilterFields {
    listing::FilterFields {
        canonical: crate::listing::canonical_id("CM", m.id),
        slug: m.slug.clone(),
        title: m.title.clone(),
        status: m.status.clone(),
        tags: Vec::new(),
    }
}

/// The table columns for concept-map list.
const CONCEPT_MAP_COLUMNS: &[listing::Column<ConceptMapRow>] = &[
    listing::Column {
        name: "id",
        header: "ID",
        cell: |r: &ConceptMapRow| crate::listing::canonical_id("CM", r.id),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "status",
        header: "Status",
        cell: |r: &ConceptMapRow| r.status.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "slug",
        header: "Slug",
        cell: |r: &ConceptMapRow| r.slug.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "title",
        header: "Title",
        cell: |r: &ConceptMapRow| r.title.clone(),
        paint: listing::ColumnPaint::None,
    },
];

/// The default visible columns.
const CONCEPT_MAP_DEFAULT: &[&str] = &["id", "status", "slug", "title"];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- scaffold / render ---

    #[test]
    fn scaffold_template_substitution_has_no_residual_tokens() {
        let toml_body = render_toml(1, "my-map", "My Map", "2026-06-15").unwrap();
        let md_body = render_md("My Map", 1).unwrap();
        assert!(!toml_body.contains("{{"));
        assert!(toml_body.contains("id = 1"));
        assert!(toml_body.contains("status = \"draft\""));
        assert!(!md_body.contains("{{"));
        assert!(md_body.contains("CM-001"));
        assert!(md_body.contains("My Map"));
    }

    #[test]
    fn scaffold_renders_three_artifacts() {
        let ctx = ScaffoldCtx {
            id: 1,
            canonical: "CM-001",
            slug: "my-map",
            title: "My Map",
            date: "2026-06-15",
        };
        let fileset = concept_map_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        // Verify the symlink exists with correct slug
        let symlink = fileset
            .iter()
            .find(|a| matches!(a, Artifact::Symlink { .. }))
            .unwrap();
        if let Artifact::Symlink { rel_path, target } = symlink {
            assert_eq!(rel_path, Path::new("001-my-map"));
            assert_eq!(target, "001");
        } else {
            panic!("expected symlink");
        }
        // Verify TOML and MD files
        let mut found_toml = false;
        let mut found_md = false;
        for a in &fileset {
            if let Artifact::File { rel_path, body } = a {
                if rel_path == Path::new("001/concept-map-001.toml") {
                    found_toml = true;
                    assert!(body.contains("id = 1"));
                }
                if rel_path == Path::new("001/concept-map-001.md") {
                    found_md = true;
                    assert!(body.contains("CM-001"));
                }
            }
        }
        assert!(found_toml);
        assert!(found_md);
    }

    // --- parse_ref ---

    #[test]
    fn parse_ref_accepts_prefixed_padded_and_bare_ids() {
        assert_eq!(parse_ref("CM-001").unwrap(), 1);
        assert_eq!(parse_ref("CM-1").unwrap(), 1);
        assert_eq!(parse_ref("cm-001").unwrap(), 1);
        assert_eq!(parse_ref("cm-1").unwrap(), 1);
        assert_eq!(parse_ref("1").unwrap(), 1);
        assert_eq!(parse_ref("42").unwrap(), 42);
    }

    #[test]
    fn parse_ref_rejects_bad_input() {
        assert!(parse_ref("foo").is_err());
        assert!(parse_ref("XX-001").is_err());
    }

    // --- materialise ---

    #[test]
    fn materialise_creates_correct_directory_layout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Test Map".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        // Directory structure
        assert!(cm_root.join("001").is_dir());
        assert!(cm_root.join("001/concept-map-001.toml").is_file());
        assert!(cm_root.join("001/concept-map-001.md").is_file());
        let symlink = cm_root.join("001-test-map");
        assert!(symlink.is_symlink());

        // Read back the TOML and verify Meta fields
        let meta = meta::read_meta(&cm_root, "concept-map", 1).unwrap();
        assert_eq!(meta.id, 1);
        assert_eq!(meta.slug, "test-map");
        assert_eq!(meta.title, "Test Map");
        assert_eq!(meta.status, "draft");
    }

    #[test]
    fn materialise_allocates_next_id() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("First".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Second".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        assert!(cm_root.join("001").is_dir());
        assert!(cm_root.join("002").is_dir());
    }

    // --- list ---

    #[test]
    fn list_returns_correct_entries() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Alpha".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Beta".into()), None).unwrap();

        let output = list_rows(root, ListArgs::default()).unwrap();
        assert!(output.contains("CM-001"));
        assert!(output.contains("CM-002"));
        assert!(output.contains("Alpha"));
        assert!(output.contains("Beta"));
    }

    #[test]
    fn list_hides_terminal_by_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Active One".into()), None).unwrap();
        // Simulate a done concept map by changing the status in the TOML
        let cm_root = root.join(CONCEPT_MAP_DIR);
        let toml_path = cm_root.join("001").join("concept-map-001.toml");
        let text = std::fs::read_to_string(&toml_path).unwrap();
        let replaced = text.replace("draft", "done");
        std::fs::write(&toml_path, replaced).unwrap();

        // With --all it should appear
        let output_all = list_rows(
            root,
            ListArgs {
                all: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(output_all.contains("CM-001"));

        // Default should hide done
        let output = list_rows(root, ListArgs::default()).unwrap();
        assert!(!output.contains("CM-001"));
    }

    // --- show ---

    #[test]
    fn show_prints_metadata_and_dsl() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Domain Model".into()), None).unwrap();

        // Write some DSL into the TOML
        let cm_root = root.join(CONCEPT_MAP_DIR);
        let toml_path = cm_root.join("001").join("concept-map-001.toml");
        let mut text = std::fs::read_to_string(&toml_path).unwrap();
        text = text.replace(
            "dsl = '''\n'''",
            "dsl = '''\nUser > identity > Identity\n'''",
        );
        std::fs::write(&toml_path, text).unwrap();

        let (doc, _toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();
        let out = format_show(&doc, "", false, false);
        assert!(out.contains("CM-001"));
        assert!(out.contains("Domain Model"));
        assert!(out.contains("draft"));
        assert!(out.contains("User > identity > Identity"));
    }

    #[test]
    fn show_with_edges_and_nodes_prints_placeholders() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Map".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        let (doc, _toml_text, _body) = read_concept_map(&cm_root, 1).unwrap();

        // --edges
        let out = format_show(&doc, "", true, false);
        assert!(out.contains("PHASE-02"));

        // --nodes
        let out = format_show(&doc, "", false, true);
        assert!(out.contains("PHASE-02"));
    }

    #[test]
    fn show_json_includes_all_fields() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();

        run_new(Some(root.to_path_buf()), Some("Map".into()), None).unwrap();

        let cm_root = root.join(CONCEPT_MAP_DIR);
        let (doc, _toml_text, body) = read_concept_map(&cm_root, 1).unwrap();
        let json = show_json(&doc, &body, false, false).unwrap();
        assert!(json.contains("\"CM-001\""));
        assert!(json.contains("\"draft\""));
        assert!(json.contains("\"Map\""));
        assert!(json.contains("\"dsl\""));
        assert!(json.contains("\"body\""));
    }

    // --- statuses ---

    #[test]
    fn concept_map_statuses_matches_expected_variants() {
        assert_eq!(
            CONCEPT_MAP_STATUSES,
            &["draft", "active", "done", "abandoned"]
        );
    }
}
