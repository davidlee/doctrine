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

use std::fs;
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

/// Relative dir of the ADR tree inside the project root. Distinct top-level tree,
/// not nested under slice (D2 — ADRs are project-global governance).
const ADR_DIR: &str = ".doctrine/adr";

/// The top-level reserved ADR kind: `adr-NNN.toml` + `adr-NNN.md` + slug symlink.
/// `prefix` is the canonical-id stem (`ADR-007`); the file stem is `"adr"` — see
/// `meta` on why prefix ≠ stem.
const ADR_KIND: Kind = Kind {
    dir: ADR_DIR,
    prefix: "ADR",
    scaffold: adr_scaffold,
};

/// The status transitions `adr status` writes. Distinct from the `proposed`
/// scaffold seed: these are the moves an ADR makes over its life. A flat enum —
/// no lifecycle ladder (unlike `state::PhaseStatus`), so no per-state stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Deprecated,
}

impl AdrStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
        }
    }
}

/// The ADR status known-set — the authority `validate_statuses` checks `--status`
/// against (A-2). It mirrors `AdrStatus`'s variants; the two are kept in lockstep
/// by `adr_known_set_matches_variants` (a drift canary). The enum kinds cannot
/// store an out-of-vocab status, so this doubles as the complete vocabulary.
const ADR_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "superseded",
    "deprecated",
];

/// The `adr list` hide-set (design §5.3): superseded / rejected / deprecated ADRs
/// are decisions that no longer govern, so they drop from the default list. The
/// override (`--all` or any explicit `--status`) reveals them — handled in
/// `listing::retain`, not here.
fn is_hidden(status: &str) -> bool {
    matches!(status, "rejected" | "superseded" | "deprecated")
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold
// ---------------------------------------------------------------------------

/// Render `adr-<id>.toml` from the embedded template by token substitution. The
/// `id/slug/title/status` keys round-trip into `meta::Meta` (VT-3).
fn render_adr_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/adr.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
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
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// `doctrine adr new` — allocate the next id and scaffold a new ADR. ADRs always
/// slug the title (no slug-less facet); `--slug` overrides. Touches disk via the
/// shared `Fresh` engine path — the monotonic id and race-retry are inherited.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        &ADR_KIND,
        &LocalFs,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
    )?;

    let id = out
        .eid
        .numeric_id()
        .context("adr kind must yield a numeric id")?;
    writeln!(io::stdout(), "Created ADR {id:03}: {}", out.dir.display())?;
    Ok(())
}

/// `doctrine adr list` — the migrated read surface (SL-025): prefixed `ADR-` ids
/// and a header, the shared filter flags (`-f/-r/-i/-s/-t/-a` plus
/// `--format/--json`), the rejected/superseded/deprecated hide-set by default,
/// sorted by id. Reads the authored `adr-NNN.toml` status field (D5 — status is
/// authored, not symlink-indexed).
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(&root, args)?)?;
    Ok(())
}

/// One ADR projected to its faithful JSON row (D7) — the variant axis adr owns.
/// `id` is the prefixed canonical id; the table column projection lives in
/// [`list_rows`].
#[derive(Debug, Serialize)]
struct AdrRow {
    id: String,
    status: String,
    slug: String,
    title: String,
}

/// The `adr list` rows as a string — the compute half of `run_list`, extracted so
/// the boot snapshot (SL-011) projects the same rows in-process. Rides the shared
/// spine: `listing::build` resolves the filter + format, `validate_statuses` guards
/// `--status` against the ADR known-set (A-2), `retain` applies the hide-set, adr
/// owns the sort (by id) and the column/JSON projection (the variant axis).
pub(crate) fn list_rows(root: &Path, args: ListArgs) -> anyhow::Result<String> {
    validate_statuses(&args.status, ADR_STATUSES)?;
    let (filter, format) = listing::build(args)?;
    let adr_root = root.join(ADR_DIR);
    let mut metas = listing::retain(meta::read_metas(&adr_root, "adr")?, &filter, is_hidden, key);
    metas.sort_by_key(|m| m.id);
    match format {
        Format::Table => Ok(render_table(&metas)),
        Format::Json => listing::json_envelope("adr", &json_rows(&metas)),
    }
}

/// Project an ADR `Meta` to its filterable fields (design §5.2). The `canonical`
/// field is the prefixed id (`ADR-007`) — the regex domain; `tags` come from the
/// authored `[relationships].tags` (filterable, read-only — there is no adr tag
/// write verb yet).
fn key(m: &Meta) -> listing::FilterFields {
    listing::FilterFields {
        canonical: canonical_id(m.id),
        slug: m.slug.clone(),
        title: m.title.clone(),
        status: m.status.clone(),
        tags: Vec::new(),
    }
}

/// The `ADR-007` canonical id for a numeric ADR id, via the single id-form
/// authority. `ADR_KIND.prefix` is the stem (`"ADR"`).
fn canonical_id(id: u32) -> String {
    listing::canonical_id(ADR_KIND.prefix, id)
}

/// Re-export of the spine's status validator, scoped to adr so callers (and tests)
/// read intent locally.
fn validate_statuses(given: &[String], known: &[&str]) -> anyhow::Result<()> {
    listing::validate_statuses(given, known)
}

/// The table grid: a header row then one `ADR-id status slug title` row per ADR
/// (prefixed ids + header, design §5.5). Rendered over the shared layout.
fn render_table(metas: &[Meta]) -> String {
    let mut grid: Vec<Vec<String>> = vec![vec![
        "id".to_string(),
        "status".to_string(),
        "slug".to_string(),
        "title".to_string(),
    ]];
    grid.extend(metas.iter().map(|m| {
        vec![
            canonical_id(m.id),
            m.status.clone(),
            m.slug.clone(),
            m.title.clone(),
        ]
    }));
    // Header-only (no rows) collapses to "" — keep the empty-list contract (§5.5).
    if metas.is_empty() {
        return String::new();
    }
    listing::render_table(&grid)
}

/// Faithful JSON rows (D7) — the prefixed id plus the authored list fields.
fn json_rows(metas: &[Meta]) -> Vec<AdrRow> {
    metas
        .iter()
        .map(|m| AdrRow {
            id: canonical_id(m.id),
            status: m.status.clone(),
            slug: m.slug.clone(),
            title: m.title.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// show — reassemble adr-NNN.toml (as data) + adr-NNN.md (prose)
// ---------------------------------------------------------------------------

/// The inert `[relationships]` table, read as data for `show` (it is preserved on
/// disk, ignored by `Meta`). Every axis defaults to empty so a hand-trimmed file
/// still parses.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
struct Relationships {
    #[serde(default)]
    supersedes: Vec<String>,
    #[serde(default)]
    superseded_by: Vec<String>,
    #[serde(default)]
    related: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// The full `adr-NNN.toml` read as data for `show` — `Meta`'s four list fields
/// plus the dates and the relationships table. JSON-faithful (D7); `Meta` ignores
/// the extra keys on the list path, this surfaces them on the inspect path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
struct AdrDoc {
    id: u32,
    slug: String,
    title: String,
    status: String,
    created: String,
    updated: String,
    #[serde(default)]
    relationships: Relationships,
}

/// Parse an ADR reference — `ADR-007`, `adr-7`, or the bare id `7` — to its numeric
/// id. The prefix is optional and case-insensitive; the id may be zero-padded.
fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("ADR-")
        .or_else(|| reference.strip_prefix("adr-"))
        .unwrap_or(reference);
    digits
        .parse::<u32>()
        .with_context(|| format!("not an ADR reference: `{reference}` (expected `ADR-007` or `7`)"))
}

/// `doctrine adr show <ADR-NNN>` — the inspect verb (SL-025 §5.2 show seam).
/// READ-ONLY: resolve the ref to its id, read THAT ADR's toml (as data) + md
/// (prose body), render the readable whole (`Table`) or the faithful
/// toml-as-data + body (`Json`). No cross-corpus scan; only the one ADR's files
/// are opened.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(reference)?;
    let (doc, body) = read_adr(&root.join(ADR_DIR), id)?;
    let out = match format {
        Format::Table => format_show(&doc, &body),
        Format::Json => show_json(&doc, &body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Read one ADR's `adr-NNN.toml` (as data) and `adr-NNN.md` (prose body).
fn read_adr(adr_root: &Path, id: u32) -> anyhow::Result<(AdrDoc, String)> {
    let name = format!("{id:03}");
    let dir = adr_root.join(&name);
    let toml_path = dir.join(format!("adr-{name}.toml"));
    let text = fs::read_to_string(&toml_path)
        .with_context(|| format!("adr {name} not found at {}", toml_path.display()))?;
    let doc: AdrDoc = toml::from_str(&text)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    let md_path = dir.join(format!("adr-{name}.md"));
    let body = fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    Ok((doc, body))
}

/// Render the readable whole for `Table` mode: an identity header, the flat
/// fields, the non-empty relationship axes, then the prose body verbatim. House
/// style: `Vec<String>` parts each carrying their own newline, joined by `concat`
/// (the `backlog::format_show` precedent — avoids the `push_str(&format!)` lint).
fn format_show(doc: &AdrDoc, body: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", canonical_id(doc.id), doc.title));
    parts.push(format!("{} · {}\n", doc.slug, doc.status));
    parts.push(format!(
        "created {} · updated {}\n",
        doc.created, doc.updated
    ));

    let rel = &doc.relationships;
    if !rel.supersedes.is_empty()
        || !rel.superseded_by.is_empty()
        || !rel.related.is_empty()
        || !rel.tags.is_empty()
    {
        parts.push("\nrelationships:\n".to_string());
        for (label, refs) in [
            ("supersedes", &rel.supersedes),
            ("superseded_by", &rel.superseded_by),
            ("related", &rel.related),
            ("tags", &rel.tags),
        ] {
            if !refs.is_empty() {
                parts.push(format!("  {label}: {}\n", refs.join(", ")));
            }
        }
    }

    parts.push(format!("\n{body}"));
    parts.concat()
}

/// Render the `Json` show: the faithful toml-as-data (`AdrDoc`) plus the prose
/// body, under the shared `{kind, …}` envelope (OQ-2 — relationships included,
/// toml-as-data is faithful).
fn show_json(doc: &AdrDoc, body: &str) -> anyhow::Result<String> {
    let value = serde_json::json!({ "kind": "adr", "adr": doc, "body": body });
    serde_json::to_string_pretty(&value).context("failed to serialize adr show JSON")
}

/// `doctrine adr status` — flip an ADR's authored status and bump `updated`.
/// The clock is read here and passed in (the pure/imperative split); the
/// transition itself is edit-preserving and no-ops when unchanged.
pub(crate) fn run_status(path: Option<PathBuf>, id: u32, status: AdrStatus) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let adr_root = root.join(ADR_DIR);
    set_adr_status(&adr_root, id, status, &crate::clock::today())?;
    writeln!(io::stdout(), "ADR {id:03}: {}", status.as_str())?;
    Ok(())
}

/// Edit-preserving status transition on one authored `adr-NNN.toml`: set
/// `status`, stamp `updated`. `toml_edit` mutates the file in place, so the inert
/// `[relationships]` table, hand-added comments, and unknown keys all survive
/// (the file is never reserialised). Local to this module (D3 — single consumer);
/// deliberately unlike `state::set_phase_status`: no `[[progress]]` row (git is
/// the audit trail — Q1/Q2), no `started`/`completed` stamps (a flat enum, not a
/// ladder), and it carries the I5 no-op guard. The date is supplied by the shell.
fn set_adr_status(adr_root: &Path, id: u32, status: AdrStatus, today: &str) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = adr_root.join(&name).join(format!("adr-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("adr {name} not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    // I5 no-op guard: an unchanged status writes nothing, so mtime/content hold.
    if doc.get("status").and_then(toml_edit::Item::as_str) == Some(status.as_str()) {
        return Ok(());
    }

    let table = doc.as_table_mut();
    // F-1: `status`/`updated` are scaffold-seeded — this verb edits in place, never
    // creates. Their absence means a malformed (hand-edited) ADR; a tail `insert`
    // would append the key *after* the trailing `[relationships]` header, landing it
    // inside that subtable (silent corruption). Refuse instead.
    if !table.contains_key("status") || !table.contains_key("updated") {
        anyhow::bail!(
            "malformed adr {name}: missing `status`/`updated` (regenerate via `adr new`)"
        );
    }
    table.insert("status", toml_edit::value(status.as_str()));
    table.insert("updated", toml_edit::value(today));
    fs::write(&path, doc.to_string()).with_context(|| format!("Failed to write {}", path.display()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;

    fn adr_root(root: &Path) -> PathBuf {
        root.join(ADR_DIR)
    }

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
    fn render_adr_toml_escapes_hostile_title_and_slug() {
        // SL-024: a title / explicit slug carrying the quoted-literal breakers
        // (`"`, `\`, newline) must still render a parseable toml that round-trips.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_adr_toml(7, slug, title, "2026-06-04").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
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

    // --- VT-1: `adr new` writes the tree and allocates monotonically ---

    #[test]
    fn run_new_writes_the_adr_tree_and_allocates_monotonically() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // explicit path short-circuits root detection; the title arg avoids stdin.
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Adopt CI".into()), None).unwrap();

        let adr = adr_root(root);
        assert!(adr.join("001/adr-001.toml").is_file());
        assert!(adr.join("001/adr-001.md").is_file());
        assert_eq!(
            fs::read_link(adr.join("001-use-rust")).unwrap(),
            Path::new("001")
        );
        // a second `new` lands the next id (monotonic, engine race-retry inherited).
        assert!(adr.join("002/adr-002.toml").is_file());
        assert_eq!(
            fs::read_link(adr.join("002-adopt-ci")).unwrap(),
            Path::new("002")
        );
    }

    // --- EX-1 / VT-1: the full chain through the real verbs end to end ---

    #[test]
    fn end_to_end_new_x2_list_status_accept_then_filtered_list() {
        // EX-1: new x2 -> list (both) -> status 1 accepted -> list --status accepted
        // (only 001). Unlike the piecemeal tests above, this drives the *real*
        // status verb (no raw rewrite) across a single tree — Fresh alloc, authored
        // mutation, filtered list, all composed (VT-1).
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        run_new(Some(root.clone()), Some("Use Rust".into()), None).unwrap();
        run_new(Some(root.clone()), Some("Adopt CI".into()), None).unwrap();
        let adr = adr_root(&root);

        // list (the run_list pipeline): both ADRs, sorted by id. `--all` reveals
        // every status; the spine owns the filter, adr owns the id sort.
        let all = list_rows(
            &root,
            ListArgs {
                all: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(all.contains("ADR-001"));
        assert!(all.contains("ADR-002"));

        // authored mutation via the real verb core (not a rewrite).
        set_adr_status(&adr, 1, AdrStatus::Accepted, &crate::clock::today()).unwrap();

        // list --status accepted: only 001 survives the filter.
        let accepted = list_rows(
            &root,
            ListArgs {
                status: vec!["accepted".into()],
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(accepted.contains("ADR-001"));
        assert!(!accepted.contains("ADR-002"));
    }

    // --- SL-025: list_rows on the spine — prefixed ids, header, hide-set, filters ---

    /// A no-constraint `ListArgs` (the default `adr list`).
    fn args() -> ListArgs {
        ListArgs::default()
    }

    /// Build a small tree: two ADRs, the first flipped to a given status.
    fn two_adrs(root: &Path, first_status: AdrStatus) {
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Adopt CI".into()), None).unwrap();
        set_adr_status(&adr_root(root), 1, first_status, &crate::clock::today()).unwrap();
    }

    #[test]
    fn list_rows_emits_prefixed_ids_and_a_header() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        let out = list_rows(root, args()).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        // VT-1: a header row, then prefixed ADR- ids — not bare `001`.
        assert!(lines[0].starts_with("id"), "header row: {:?}", lines[0]);
        assert!(lines[0].contains("status"), "header names columns");
        assert!(out.contains("ADR-001  accepted"), "prefixed id: {out}");
        assert!(out.contains("ADR-002"), "second ADR present: {out}");
        assert!(!out.contains("\n001  "), "no bare numeric id: {out}");
    }

    #[test]
    fn list_rows_hide_set_drops_rejected_superseded_deprecated_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Keep".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Gone".into()), None).unwrap();
        set_adr_status(&adr_root(root), 2, AdrStatus::Superseded, "2099-01-01").unwrap();

        // default: the superseded ADR-002 is hidden.
        let out = list_rows(root, args()).unwrap();
        assert!(out.contains("ADR-001"), "non-hidden ADR kept: {out}");
        assert!(
            !out.contains("ADR-002"),
            "superseded hidden by default: {out}"
        );
    }

    #[test]
    fn list_rows_all_and_explicit_status_reveal_the_hide_set() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Keep".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Gone".into()), None).unwrap();
        set_adr_status(&adr_root(root), 2, AdrStatus::Superseded, "2099-01-01").unwrap();

        // --all reveals it.
        let all = list_rows(
            root,
            ListArgs {
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(all.contains("ADR-002"), "--all reveals superseded: {all}");

        // an explicit --status also reveals it (terminal-hide override).
        let by_status = list_rows(
            root,
            ListArgs {
                status: vec!["superseded".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            by_status.contains("ADR-002"),
            "explicit status reveals: {by_status}"
        );
        assert!(
            !by_status.contains("ADR-001"),
            "and filters to it: {by_status}"
        );
    }

    #[test]
    fn list_rows_filter_matches_slug_and_title() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        let out = list_rows(
            root,
            ListArgs {
                substr: Some("adopt".into()),
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.contains("ADR-002"), "substr matches adopt-ci: {out}");
        assert!(!out.contains("ADR-001"), "use-rust filtered out: {out}");
    }

    #[test]
    fn list_rows_regexp_matches_canonical_id() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        // a regex over the canonical id (the slug/title do not contain `ADR-002`).
        let out = list_rows(
            root,
            ListArgs {
                regexp: Some("ADR-002".into()),
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.contains("ADR-002"), "regex matches canonical: {out}");
        assert!(!out.contains("ADR-001"), "non-matching dropped: {out}");
    }

    #[test]
    fn list_rows_json_is_the_shared_envelope_with_prefixed_ids() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        let out = list_rows(
            root,
            ListArgs {
                json: true,
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "adr");
        let rows = parsed["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["id"], "ADR-001");
        assert_eq!(rows[0]["status"], "accepted");
        assert_eq!(rows[0]["slug"], "use-rust");
    }

    #[test]
    fn list_rows_empty_tree_is_the_empty_string() {
        let dir = tempfile::tempdir().unwrap();
        // no ADRs at all → "" (header suppressed on empty, §5.5).
        assert_eq!(list_rows(dir.path(), args()).unwrap(), "");
    }

    // --- VT-4: --status validates against the adr known-set (A-2) ---

    #[test]
    fn list_rows_rejects_an_unknown_status_with_the_uniform_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = list_rows(
            dir.path(),
            ListArgs {
                status: vec!["bogus".into()],
                ..Default::default()
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("bogus"), "names the bad value: {err}");
        assert!(err.contains("accepted"), "lists the known set: {err}");
    }

    #[test]
    fn list_rows_accepts_every_known_status() {
        let dir = tempfile::tempdir().unwrap();
        for s in ADR_STATUSES {
            assert!(
                list_rows(
                    dir.path(),
                    ListArgs {
                        status: vec![(*s).to_string()],
                        ..Default::default()
                    },
                )
                .is_ok(),
                "known status `{s}` accepted"
            );
        }
    }

    /// Drift canary: the `ADR_STATUSES` known-set must stay in lockstep with the
    /// `AdrStatus` variants (the enum kinds cannot store an out-of-vocab value, so
    /// this is the complete vocabulary).
    #[test]
    fn adr_known_set_matches_variants() {
        let variants = [
            AdrStatus::Proposed,
            AdrStatus::Accepted,
            AdrStatus::Rejected,
            AdrStatus::Superseded,
            AdrStatus::Deprecated,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|v| v.as_str()).collect();
        assert_eq!(from_variants, ADR_STATUSES.to_vec());
    }

    // --- SL-025 PHASE-06 EX-2 / VT-2: ordering-preservation through list_rows ---

    /// Write an ADR's authored toml directly at an explicit id (creating its dir).
    /// Bypasses the monotonic `Fresh` allocator so the fixture's creation order can
    /// be made deliberately out of id-order — the spine's per-kind sort, not read
    /// order, must produce the result. Only the fields the spine reads are written.
    fn adr_at(root: &Path, id: u32, status: &str, slug: &str, title: &str) {
        let name = format!("{id:03}");
        let dir = adr_root(root).join(&name);
        fs::create_dir_all(&dir).unwrap();
        let toml = format!(
            "schema = \"doctrine.adr\"\nversion = 1\n\nid = {id}\nslug = \"{slug}\"\ntitle = \"{title}\"\nstatus = \"{status}\"\ncreated = \"2026-06-04\"\nupdated = \"2026-06-04\"\n"
        );
        fs::write(dir.join(format!("adr-{name}.toml")), toml).unwrap();
    }

    /// The byte offsets of each prefixed id in render order — ascending offsets
    /// iff the rows are emitted in that sequence.
    fn id_order(out: &str, ids: &[&str]) -> Vec<usize> {
        ids.iter()
            .map(|id| {
                out.find(id)
                    .unwrap_or_else(|| panic!("{id} present: {out}"))
            })
            .collect()
    }

    #[test]
    fn list_rows_orders_by_id_ascending_regardless_of_creation_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Create OUT of id order: 003, then 001, then 002.
        adr_at(root, 3, "accepted", "gamma", "Gamma");
        adr_at(root, 1, "accepted", "alpha", "Alpha");
        adr_at(root, 2, "accepted", "beta", "Beta");

        let out = list_rows(root, args()).unwrap();
        let offsets = id_order(&out, &["ADR-001", "ADR-002", "ADR-003"]);
        assert!(
            offsets[0] < offsets[1] && offsets[1] < offsets[2],
            "ADR rows must render in ascending id order (sort, not read order): {out}"
        );
    }

    // --- VT-2: adr show — table + json, reassembling toml + md ---

    #[test]
    fn parse_ref_accepts_prefixed_padded_and_bare_ids() {
        assert_eq!(parse_ref("ADR-007").unwrap(), 7);
        assert_eq!(parse_ref("adr-7").unwrap(), 7);
        assert_eq!(parse_ref("7").unwrap(), 7);
        assert_eq!(parse_ref("042").unwrap(), 42);
        assert!(parse_ref("nope").is_err());
    }

    #[test]
    fn read_adr_reassembles_toml_as_data_and_md_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();

        let (doc, body) = read_adr(&adr_root(root), 1).unwrap();
        assert_eq!(doc.id, 1);
        assert_eq!(doc.slug, "use-rust");
        assert_eq!(doc.status, "proposed");
        // the inert relationships table parses as data (empty by default).
        assert!(doc.relationships.supersedes.is_empty());
        // the md prose body is read verbatim.
        assert!(body.contains("ADR-001: Use Rust"));
        assert!(body.contains("## Context"));
    }

    #[test]
    fn format_show_renders_identity_relationships_and_body() {
        let doc = AdrDoc {
            id: 7,
            slug: "use-rust".into(),
            title: "Use Rust".into(),
            status: "accepted".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            relationships: Relationships {
                supersedes: vec!["ADR-003".into()],
                superseded_by: vec![],
                related: vec![],
                tags: vec!["lang".into()],
            },
        };
        let out = format_show(&doc, "# ADR-007: Use Rust\n\nbody.\n");
        assert!(out.contains("ADR-007 — Use Rust"), "identity: {out}");
        assert!(out.contains("use-rust · accepted"), "flat fields: {out}");
        assert!(out.contains("created 2026-06-01 · updated 2026-06-08"));
        assert!(out.contains("supersedes: ADR-003"), "relationships: {out}");
        assert!(out.contains("tags: lang"), "tags axis: {out}");
        assert!(
            out.contains("# ADR-007: Use Rust"),
            "prose body appended: {out}"
        );
    }

    #[test]
    fn show_json_is_faithful_toml_as_data_plus_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();
        let (doc, body) = read_adr(&adr_root(root), 1).unwrap();

        let out = show_json(&doc, &body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "adr");
        assert_eq!(parsed["adr"]["id"], 1);
        assert_eq!(parsed["adr"]["slug"], "use-rust");
        assert_eq!(parsed["adr"]["status"], "proposed");
        // OQ-2: relationships are included (toml-as-data is faithful).
        assert!(parsed["adr"]["relationships"]["supersedes"].is_array());
        assert!(
            parsed["body"].as_str().unwrap().contains("## Context"),
            "body carried in json"
        );
    }

    #[test]
    fn run_show_on_a_missing_adr_errors() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_show(Some(dir.path().to_path_buf()), "ADR-009", Format::Table).unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
    }

    // --- VT-2: an empty / symbol-only title bails for an explicit --slug ---

    #[test]
    fn run_new_bails_for_a_slug_on_a_symbol_only_title() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_new(Some(dir.path().to_path_buf()), Some("!!!".into()), None).unwrap_err();
        assert!(err.to_string().contains("pass --slug"));
    }

    // --- VT-1 read + VT-3: `adr list`'s pipeline reads stem "adr" and filters ---

    #[test]
    fn read_metas_round_trips_created_adrs_and_filters_by_status() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();
        run_new(Some(root.to_path_buf()), Some("Adopt CI".into()), None).unwrap();
        let adr = adr_root(root);

        // flip 002 to accepted — the status verb is PHASE-04; a raw rewrite is
        // enough to prove the list filter selects on the authored toml field (D5).
        let p = adr.join("002/adr-002.toml");
        let flipped = fs::read_to_string(&p)
            .unwrap()
            .replace("status = \"proposed\"", "status = \"accepted\"");
        fs::write(&p, flipped).unwrap();

        // read_metas reads the stem faithfully (the reader round-trip, VT-3); the
        // spine owns the sort/filter, so sort the read set here to pin id 1's fields.
        let mut all = meta::read_metas(&adr, "adr").unwrap();
        all.sort_by_key(|m| m.id);
        assert_eq!(all.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(
            all.first(),
            Some(&Meta {
                id: 1,
                slug: "use-rust".into(),
                title: "Use Rust".into(),
                status: "proposed".into(),
            })
        );

        // list --status accepted selects on the authored field (the spine filter).
        let accepted = list_rows(
            root,
            ListArgs {
                status: vec!["accepted".into()],
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(accepted.contains("ADR-002"));
        assert!(!accepted.contains("ADR-001"));
    }

    // --- VT-1: status flips, `updated` bumps, the rest of the file survives ---

    #[test]
    fn set_adr_status_flips_status_bumps_updated_and_preserves_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();
        let adr = adr_root(root);

        // an injected date distinct from today() so the bump is visible (VT-1).
        set_adr_status(&adr, 1, AdrStatus::Accepted, "2099-01-01").unwrap();

        // re-read through the shared reader: the authored status flipped.
        assert_eq!(meta::read_meta(&adr, "adr", 1).unwrap().status, "accepted");

        let body = fs::read_to_string(adr.join("001/adr-001.toml")).unwrap();
        // `updated` bumped to the injected date; `created` (the seed) untouched.
        assert!(body.contains("updated = \"2099-01-01\""));
        assert!(!body.contains("created = \"2099-01-01\""));
        // toml_edit preserved the inert table and its hand-authored comments.
        assert!(body.contains("[relationships]"));
        assert!(body.contains("# Reserved."));
        assert!(body.contains("supersedes"));
    }

    // --- VT-2: the I5 no-op guard — an unchanged status writes nothing ---

    #[test]
    fn set_adr_status_to_the_current_value_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();
        let p = adr_root(root).join("001/adr-001.toml");
        let before = fs::read_to_string(&p).unwrap();

        // seed status is "proposed"; the distinct date would bump `updated` IF it
        // wrote — so byte-equality proves the guard short-circuited (I5).
        set_adr_status(&adr_root(root), 1, AdrStatus::Proposed, "2099-01-01").unwrap();

        assert_eq!(fs::read_to_string(&p).unwrap(), before);
    }

    // --- VT-3: a missing id among existing ADRs is a hard error (I3) ---

    #[test]
    fn set_adr_status_on_a_missing_id_among_existing_adrs_errors() {
        // F-2: prove I3 — a missing id *among existing ADRs* is a hard error, not an
        // implicit create. (The bare empty-root case only proved "file absent".)
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Use Rust".into()), None).unwrap();
        let err =
            set_adr_status(&adr_root(root), 9, AdrStatus::Accepted, "2099-01-01").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // --- F-1: a malformed ADR missing template-seeded keys is refused, not corrupted ---

    #[test]
    fn set_adr_status_on_an_adr_missing_updated_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = adr_root(dir.path()).join("003/adr-003.toml");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        // `updated` omitted; a tail `insert` would have landed it in `[relationships]`.
        fs::write(
            &p,
            "status = \"proposed\"\n\n[relationships]\nsupersedes = []\n",
        )
        .unwrap();
        let err = set_adr_status(&adr_root(dir.path()), 3, AdrStatus::Accepted, "2099-01-01")
            .unwrap_err();
        assert!(err.to_string().contains("malformed"));
    }
}
