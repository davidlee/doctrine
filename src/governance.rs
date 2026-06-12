// SPDX-License-Identifier: GPL-3.0-only
//! Command-tier governance spine — the shared CLI/status machinery the per-kind
//! governance modules (`adr`, and `policy` from SL-030) bind with a `GovKind`
//! descriptor. Extracted from `adr.rs` (SL-030 PHASE-02) so a second kind rides
//! the same compute/io rather than copying it (design §5.1, D1).
//!
//! Layering (ADR-001): this is a **command-tier** module — it legitimately uses
//! `root::find`/`clock::today` (shell concerns), so it sits *above* the pure leaf
//! `listing.rs`, not beside it. It depends downward on `entity`/`meta`/`listing`
//! and sideways on `root`/`clock`/`input`; the per-kind modules (`adr`/`policy`)
//! depend on it, and `boot` calls `list_rows` directly. No engine/leaf module
//! depends on `governance`, so no cycle is introduced.
//!
//! Two faces: **io/compute** helpers that take a resolved `root`/`path`
//! (`list_rows`, `set_status`, `read_doc`, `parse_ref`, `format_show`,
//! `show_json` — boot calls `list_rows`) and the thin **shell** wrappers
//! (`run_*`) that do `root::find` + `clock::today` + stdout.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

use serde::Serialize;

use crate::entity::{self, Inputs, Kind, LocalFs, MaterialiseRequest};
use crate::listing::{self, Format, ListArgs};
use crate::meta::{self, Meta};

/// The per-kind descriptor the spine is parameterized over (design §5.2). Four
/// fields, all exercised by every governance kind from day one — no dead field.
/// `stem` serves both the file naming (`<stem>-NNN.toml`) AND the JSON
/// envelope/object key, so a kind can never name its files and its JSON
/// incoherently (Codex MINOR-7 — `json_label` dropped).
pub(crate) struct GovKind {
    /// The entity-engine kind: dir, canonical-id prefix, scaffold fn.
    pub kind: Kind,
    /// File stem AND JSON envelope/object key: `"adr"` / `"policy"`.
    pub stem: &'static str,
    /// Known-set — the authority `validate_statuses` checks `--status` against.
    pub statuses: &'static [&'static str],
    /// The default-list hide-set predicate.
    pub hidden: fn(&str) -> bool,
}

// ---------------------------------------------------------------------------
// list — the shared filter/format/project pipeline
// ---------------------------------------------------------------------------

/// One governance entity projected to its faithful JSON row (D7): the prefixed
/// canonical id plus the authored list fields. Replaces the per-kind `AdrRow`.
#[derive(Debug, Serialize)]
struct GovRow {
    id: String,
    status: String,
    slug: String,
    title: String,
}

/// The list rows as a string — the compute half of `run_list`, extracted so the
/// boot snapshot (SL-011) projects the same rows in-process. Rides the shared
/// spine: `listing::build` resolves the filter + format, `validate_statuses`
/// guards `--status` against the kind's known-set (A-2), `retain` applies the
/// hide-set, the kind owns the sort (by id) and the column/JSON projection.
pub(crate) fn list_rows(g: &GovKind, root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    listing::validate_statuses(&args.status, g.statuses)?;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let gov_root = root.join(g.kind.dir);
    let mut metas = listing::retain(
        meta::read_metas(&gov_root, g.stem)?,
        &filter,
        g.hidden,
        |m| key(g, m),
    );
    metas.sort_by_key(|m| m.id);
    // One materialisation feeds both surfaces — governance's table and JSON
    // rows coincide (SL-037 A4: GovRow is all-String, id pre-prefixed).
    let rows = gov_rows(g, &metas);
    match format {
        Format::Table => {
            let sel = listing::select_columns(&GOV_COLUMNS, GOV_DEFAULT, columns.as_deref())?;
            Ok(listing::render_columns(&rows, &sel))
        }
        Format::Json => listing::json_envelope(g.stem, &rows),
    }
}

/// Project a governance `Meta` to its filterable fields (design §5.2). `tags` is
/// empty — governance kinds carry no tag reader yet (Codex BLOCKER-2, parity
/// limitation: ADR's `--tag` matched nothing either; a real reader is a follow-up).
fn key(g: &GovKind, m: &Meta) -> listing::FilterFields {
    listing::FilterFields {
        canonical: listing::canonical_id(g.kind.prefix, m.id),
        slug: m.slug.clone(),
        title: m.title.clone(),
        status: m.status.clone(),
        tags: Vec::new(),
    }
}

/// The table columns every governance kind can show (`--columns` tokens over
/// `R = GovRow` — extractors are non-capturing, SL-037 D5; the prefixed id is
/// already materialised in the row). Selection-token order: declaration order
/// is what the unknown-column error lists.
const GOV_COLUMNS: [listing::Column<GovRow>; 4] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |r| r.id.clone(),
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: |r| r.status.clone(),
    },
    listing::Column {
        name: "slug",
        header: "slug",
        cell: |r| r.slug.clone(),
    },
    listing::Column {
        name: "title",
        header: "title",
        cell: |r| r.title.clone(),
    },
];

/// The default visible set — slug-free (SL-037 D4); `--columns …,slug` reveals it.
const GOV_DEFAULT: &[&str] = &["id", "status", "title"];

/// Faithful rows (D7) — the prefixed id plus the authored list fields. Feeds
/// both the column-projected table and the JSON envelope (table+JSON rows
/// coincide for governance, SL-037 A4).
fn gov_rows(g: &GovKind, metas: &[Meta]) -> Vec<GovRow> {
    metas
        .iter()
        .map(|m| GovRow {
            id: listing::canonical_id(g.kind.prefix, m.id),
            status: m.status.clone(),
            slug: m.slug.clone(),
            title: m.title.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// show — reassemble <stem>-NNN.toml (as data) + <stem>-NNN.md (prose)
// ---------------------------------------------------------------------------

/// The inert `[relationships]` table, read as data for `show` (preserved on disk,
/// ignored by `Meta`). Every axis defaults to empty so a hand-trimmed file still
/// parses. Replaces the per-kind `Relationships`.
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

/// The full `<stem>-NNN.toml` read as data for `show` — `Meta`'s four list fields
/// plus the dates and the relationships table. JSON-faithful (D7). Replaces the
/// per-kind `AdrDoc`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
struct Doc {
    id: u32,
    slug: String,
    title: String,
    status: String,
    created: String,
    updated: String,
    #[serde(default)]
    relationships: Relationships,
}

/// Parse a governance reference — `ADR-007`, `adr-7`, or the bare id `7` — to its
/// numeric id. The prefix is stripped in exactly two literal cases (`PREFIX-` or
/// its lowercase), **not** case-insensitively: a case-insensitive strip would
/// newly accept `AdR-7`, an observable ADR behaviour change (Codex MAJOR-3).
fn parse_ref(g: &GovKind, reference: &str) -> anyhow::Result<u32> {
    let upper = format!("{}-", g.kind.prefix);
    let lower = format!("{}-", g.kind.prefix.to_lowercase());
    let digits = reference
        .strip_prefix(&upper)
        .or_else(|| reference.strip_prefix(&lower))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        let p = g.kind.prefix;
        format!("not an {p} reference: `{reference}` (expected `{p}-007` or `7`)")
    })
}

/// Read one entity's `<stem>-NNN.toml` (as data) and `<stem>-NNN.md` (prose body).
fn read_doc(g: &GovKind, gov_root: &Path, id: u32) -> anyhow::Result<(Doc, String)> {
    let name = format!("{id:03}");
    let dir = gov_root.join(&name);
    let toml_path = dir.join(format!("{}-{name}.toml", g.stem));
    let text = fs::read_to_string(&toml_path)
        .with_context(|| format!("{} {name} not found at {}", g.stem, toml_path.display()))?;
    let doc: Doc = toml::from_str(&text)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    let md_path = dir.join(format!("{}-{name}.md", g.stem));
    let body = fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    Ok((doc, body))
}

/// A governance entity's authored outbound relations (SL-046 §5.2). Emits
/// `supersedes` → [`RelationLabel::Supersedes`] and `related` →
/// [`RelationLabel::Related`] ONLY. NEVER `superseded_by` (ADR-004 §3/§5: a
/// derived-inbound carve-out, not projected — the reader derives "superseded by"
/// from `in_edges`) and NEVER `tags` (free-text classification, not entity refs).
/// Reads via the shared `read_doc` reader (no new TOML parse). Shared by ADR / POL /
/// STD via the caller-supplied `g`. An empty axis emits nothing.
pub(crate) fn relation_edges(
    g: &GovKind,
    root: &Path,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    use crate::relation::{RelationEdge, RelationLabel};
    let (doc, _body) = read_doc(g, &root.join(g.kind.dir), id)?;
    let rel = &doc.relationships;
    let mut edges = Vec::new();
    for (label, refs) in [
        (RelationLabel::Supersedes, &rel.supersedes),
        (RelationLabel::Related, &rel.related),
    ] {
        edges.extend(refs.iter().map(|t| RelationEdge::new(label, t.clone())));
    }
    Ok(edges)
}

/// Render the readable whole for `Table` mode: an identity header, the flat
/// fields, the non-empty relationship axes, then the prose body verbatim. House
/// style: `Vec<String>` parts each carrying their own newline, joined by `concat`
/// (the `backlog::format_show` precedent — avoids the `push_str(&format!)` lint).
fn format_show(g: &GovKind, doc: &Doc, body: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!(
        "{} — {}\n",
        listing::canonical_id(g.kind.prefix, doc.id),
        doc.title
    ));
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

/// Render the `Json` show: the faithful toml-as-data (`Doc`) plus the prose body,
/// under the shared `{kind, <stem>, body}` envelope. The dynamic `<stem>` object
/// key forces a hand-built `serde_json::Map` — the `json!` macro cannot take a
/// runtime key (design R2). Keys serialize BTreeMap-sorted (no `preserve_order`)
/// with no trailing newline — the contract the black-box golden pins.
fn show_json(g: &GovKind, doc: &Doc, body: &str) -> anyhow::Result<String> {
    let mut map = serde_json::Map::new();
    map.insert(
        "kind".to_string(),
        serde_json::Value::String(g.stem.to_string()),
    );
    map.insert(
        g.stem.to_string(),
        serde_json::to_value(doc)
            .with_context(|| format!("failed to serialize {} show JSON", g.stem))?,
    );
    map.insert(
        "body".to_string(),
        serde_json::Value::String(body.to_string()),
    );
    serde_json::to_string_pretty(&serde_json::Value::Object(map))
        .with_context(|| format!("failed to serialize {} show JSON", g.stem))
}

// ---------------------------------------------------------------------------
// status — edit-preserving authored-toml transition
// ---------------------------------------------------------------------------

/// Edit-preserving status transition on one authored `<stem>-NNN.toml`: set
/// `status`, stamp `updated`. `toml_edit` mutates the file in place, so the inert
/// `[relationships]` table, hand-added comments, and unknown keys all survive (the
/// file is never reserialised). Carries the I5 no-op guard (an unchanged status
/// writes nothing) and the F-1 malformed-refuse guard (a missing scaffold-seeded
/// key would otherwise tail-insert into `[relationships]` — silent corruption).
/// The date is supplied by the shell (pure/imperative split). The concrete clap
/// enum is bound in the per-kind `run_status` wrapper; this takes a `&str`.
pub(crate) fn set_status(
    g: &GovKind,
    gov_root: &Path,
    id: u32,
    status: &str,
    today: &str,
) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = gov_root.join(&name).join(format!("{}-{name}.toml", g.stem));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("{} {name} not found at {}", g.stem, path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    // I5 no-op guard: an unchanged status writes nothing, so mtime/content hold.
    if doc.get("status").and_then(toml_edit::Item::as_str) == Some(status) {
        return Ok(());
    }

    let table = doc.as_table_mut();
    // F-1: `status`/`updated` are scaffold-seeded — this verb edits in place, never
    // creates. Their absence means a malformed (hand-edited) entity; a tail `insert`
    // would append the key *after* the trailing `[relationships]` header, landing it
    // inside that subtable (silent corruption). Refuse instead.
    if !table.contains_key("status") || !table.contains_key("updated") {
        anyhow::bail!(
            "malformed {stem} {name}: missing `status`/`updated` (regenerate via `{stem} new`)",
            stem = g.stem
        );
    }
    table.insert("status", toml_edit::value(status));
    table.insert("updated", toml_edit::value(today));
    fs::write(&path, doc.to_string()).with_context(|| format!("Failed to write {}", path.display()))
}

// ---------------------------------------------------------------------------
// Shell wrappers (root::find + clock + stdout) — bound per kind by &GovKind
// ---------------------------------------------------------------------------

/// `doctrine <kind> new` — allocate the next id and scaffold a new entity. Touches
/// disk via the shared `Fresh` engine path — the monotonic id and race-retry are
/// inherited. The kind's scaffold fn (on `g.kind`) lays out the fileset.
pub(crate) fn run_new(
    g: &GovKind,
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let trunk_ids = crate::git::trunk_entity_ids(&root, g.kind.dir)?;
    let out = entity::materialise(
        &g.kind,
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
        .with_context(|| format!("{} kind must yield a numeric id", g.stem))?;
    writeln!(
        io::stdout(),
        "Created {} {id:03}: {}",
        g.kind.prefix,
        out.dir.display()
    )?;
    Ok(())
}

/// `doctrine <kind> list` — the migrated read surface: prefixed ids + header, the
/// shared filter flags, the kind's hide-set by default, sorted by id.
pub(crate) fn run_list(g: &GovKind, path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(g, &root, args)?)?;
    Ok(())
}

/// `doctrine <kind> show <ref>` — the inspect verb. READ-ONLY: resolve the ref to
/// its id, read THAT entity's toml (as data) + md (prose), render the readable
/// whole (`Table`) or the faithful toml-as-data + body (`Json`). No cross-corpus
/// scan; only the one entity's files are opened.
pub(crate) fn run_show(
    g: &GovKind,
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(g, reference)?;
    let (doc, body) = read_doc(g, &root.join(g.kind.dir), id)?;
    let out = match format {
        Format::Table => format_show(g, &doc, &body),
        Format::Json => show_json(g, &doc, &body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — the shared-spine behaviour, driven by the ADR descriptor (SL-030
// PHASE-02 VT-2; relocated from adr.rs). The ADR-specific render/scaffold tests
// stay in adr.rs. A cfg(test)-only edge to crate::adr (the real descriptor) —
// production code stays adr -> governance only.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr::{ADR_KIND, AdrStatus};

    fn adr_root(root: &Path) -> PathBuf {
        root.join(ADR_KIND.kind.dir)
    }

    /// A no-constraint `ListArgs` (the default `adr list`).
    fn args() -> ListArgs {
        ListArgs::default()
    }

    /// Build a small tree: two ADRs, the first flipped to a given status.
    fn two_adrs(root: &Path, first_status: AdrStatus) {
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Adopt CI".into()),
            None,
        )
        .unwrap();
        set_status(
            &ADR_KIND,
            &adr_root(root),
            1,
            first_status.as_str(),
            &crate::clock::today(),
        )
        .unwrap();
    }

    // --- VT-1: `adr new` writes the tree and allocates monotonically ---

    #[test]
    fn run_new_writes_the_adr_tree_and_allocates_monotonically() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // explicit path short-circuits root detection; the title arg avoids stdin.
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Adopt CI".into()),
            None,
        )
        .unwrap();

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
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        run_new(&ADR_KIND, Some(root.clone()), Some("Use Rust".into()), None).unwrap();
        run_new(&ADR_KIND, Some(root.clone()), Some("Adopt CI".into()), None).unwrap();
        let adr = adr_root(&root);

        // list (the run_list pipeline): both ADRs, sorted by id. `--all` reveals
        // every status; the spine owns the filter, the kind owns the id sort.
        let all = list_rows(
            &ADR_KIND,
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
        set_status(
            &ADR_KIND,
            &adr,
            1,
            AdrStatus::Accepted.as_str(),
            &crate::clock::today(),
        )
        .unwrap();

        // list --status accepted: only 001 survives the filter.
        let accepted = list_rows(
            &ADR_KIND,
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

    // --- list_rows on the spine — prefixed ids, header, hide-set, filters ---

    #[test]
    fn list_rows_emits_prefixed_ids_and_a_header() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        let out = list_rows(&ADR_KIND, root, args()).unwrap();
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
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Keep".into()),
            None,
        )
        .unwrap();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Gone".into()),
            None,
        )
        .unwrap();
        set_status(
            &ADR_KIND,
            &adr_root(root),
            2,
            AdrStatus::Superseded.as_str(),
            "2099-01-01",
        )
        .unwrap();

        // default: the superseded ADR-002 is hidden.
        let out = list_rows(&ADR_KIND, root, args()).unwrap();
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
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Keep".into()),
            None,
        )
        .unwrap();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Gone".into()),
            None,
        )
        .unwrap();
        set_status(
            &ADR_KIND,
            &adr_root(root),
            2,
            AdrStatus::Superseded.as_str(),
            "2099-01-01",
        )
        .unwrap();

        // --all reveals it.
        let all = list_rows(
            &ADR_KIND,
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
            &ADR_KIND,
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
            &ADR_KIND,
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
            &ADR_KIND,
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
            &ADR_KIND,
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

    // --- SL-037 column model: slug-free default, --columns projection ---

    #[test]
    fn list_rows_default_table_omits_slug() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        let out = list_rows(&ADR_KIND, root, args()).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0].split_whitespace().collect::<Vec<_>>(),
            ["id", "status", "title"],
            "default header is slug-free: {out}"
        );
        assert!(!out.contains("use-rust"), "slug cell hidden: {out}");
        assert!(out.contains("Use Rust"), "title cell present: {out}");
    }

    #[test]
    fn list_rows_columns_selects_orders_and_reveals_slug() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        let out = list_rows(
            &ADR_KIND,
            root,
            ListArgs {
                columns: Some(vec!["slug".into(), "id".into()]),
                ..Default::default()
            },
        )
        .unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0].split_whitespace().collect::<Vec<_>>(),
            ["slug", "id"],
            "requested order wins: {out}"
        );
        assert!(out.contains("use-rust"), "slug revealed: {out}");
        assert!(!out.contains("accepted"), "unselected status hidden: {out}");
    }

    #[test]
    fn list_rows_unknown_column_is_the_uniform_error_listing_available() {
        let dir = tempfile::tempdir().unwrap();
        let err = list_rows(
            &ADR_KIND,
            dir.path(),
            ListArgs {
                columns: Some(vec!["bogus".into()]),
                ..Default::default()
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("unknown column `bogus`"), "names it: {err}");
        assert!(
            err.contains("id, status, slug, title"),
            "lists the available set: {err}"
        );
    }

    #[test]
    fn list_rows_json_ignores_columns_and_keeps_slug() {
        // D7: --columns has no effect under --json; JSON rows stay faithful (D2).
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        two_adrs(root, AdrStatus::Accepted);

        let plain = list_rows(
            &ADR_KIND,
            root,
            ListArgs {
                json: true,
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        let projected = list_rows(
            &ADR_KIND,
            root,
            ListArgs {
                json: true,
                all: true,
                columns: Some(vec!["id".into()]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(plain, projected, "--columns is a no-op under --json");
        let parsed: serde_json::Value = serde_json::from_str(&projected).unwrap();
        assert_eq!(parsed["rows"][0]["slug"], "use-rust");
    }

    #[test]
    fn list_rows_empty_tree_is_the_empty_string() {
        let dir = tempfile::tempdir().unwrap();
        // no ADRs at all → "" (header suppressed on empty, §5.5).
        assert_eq!(list_rows(&ADR_KIND, dir.path(), args()).unwrap(), "");
    }

    // --- VT-4: --status validates against the kind known-set (A-2) ---

    #[test]
    fn list_rows_rejects_an_unknown_status_with_the_uniform_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = list_rows(
            &ADR_KIND,
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
        for s in ADR_KIND.statuses {
            assert!(
                list_rows(
                    &ADR_KIND,
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

    // --- ordering-preservation through list_rows ---

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

        let out = list_rows(&ADR_KIND, root, args()).unwrap();
        let offsets = id_order(&out, &["ADR-001", "ADR-002", "ADR-003"]);
        assert!(
            offsets[0] < offsets[1] && offsets[1] < offsets[2],
            "ADR rows must render in ascending id order (sort, not read order): {out}"
        );
    }

    // --- show — table + json, reassembling toml + md ---

    #[test]
    fn parse_ref_accepts_prefixed_padded_and_bare_ids() {
        assert_eq!(parse_ref(&ADR_KIND, "ADR-007").unwrap(), 7);
        assert_eq!(parse_ref(&ADR_KIND, "adr-7").unwrap(), 7);
        assert_eq!(parse_ref(&ADR_KIND, "7").unwrap(), 7);
        assert_eq!(parse_ref(&ADR_KIND, "042").unwrap(), 42);
        assert!(parse_ref(&ADR_KIND, "nope").is_err());
        // R4: the strip is two literal cases, NOT case-insensitive — a mixed-case
        // prefix is NOT stripped, so it fails to parse (an observable ADR contract).
        assert!(parse_ref(&ADR_KIND, "AdR-7").is_err());
    }

    #[test]
    fn read_doc_reassembles_toml_as_data_and_md_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();

        let (doc, body) = read_doc(&ADR_KIND, &adr_root(root), 1).unwrap();
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
        let doc = Doc {
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
        let out = format_show(&ADR_KIND, &doc, "# ADR-007: Use Rust\n\nbody.\n");
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
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();
        let (doc, body) = read_doc(&ADR_KIND, &adr_root(root), 1).unwrap();

        let out = show_json(&ADR_KIND, &doc, &body).unwrap();
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
        let err = run_show(
            &ADR_KIND,
            Some(dir.path().to_path_buf()),
            "ADR-009",
            Format::Table,
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
    }

    // --- `adr list`'s pipeline reads stem "adr" and filters ---

    #[test]
    fn read_metas_round_trips_created_adrs_and_filters_by_status() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Adopt CI".into()),
            None,
        )
        .unwrap();
        let adr = adr_root(root);

        // flip 002 to accepted via a raw rewrite — enough to prove the list filter
        // selects on the authored toml field (D5).
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
            &ADR_KIND,
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

    // --- status flips, `updated` bumps, the rest of the file survives ---

    #[test]
    fn set_status_flips_status_bumps_updated_and_preserves_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();
        let adr = adr_root(root);

        // an injected date distinct from today() so the bump is visible (VT-1).
        set_status(
            &ADR_KIND,
            &adr,
            1,
            AdrStatus::Accepted.as_str(),
            "2099-01-01",
        )
        .unwrap();

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

    // --- the I5 no-op guard — an unchanged status writes nothing ---

    #[test]
    fn set_status_to_the_current_value_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();
        let p = adr_root(root).join("001/adr-001.toml");
        let before = fs::read_to_string(&p).unwrap();

        // seed status is "proposed"; the distinct date would bump `updated` IF it
        // wrote — so byte-equality proves the guard short-circuited (I5).
        set_status(
            &ADR_KIND,
            &adr_root(root),
            1,
            AdrStatus::Proposed.as_str(),
            "2099-01-01",
        )
        .unwrap();

        assert_eq!(fs::read_to_string(&p).unwrap(), before);
    }

    // --- a missing id among existing ADRs is a hard error (I3) ---

    #[test]
    fn set_status_on_a_missing_id_among_existing_adrs_errors() {
        // F-2: prove I3 — a missing id *among existing ADRs* is a hard error, not an
        // implicit create.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(
            &ADR_KIND,
            Some(root.to_path_buf()),
            Some("Use Rust".into()),
            None,
        )
        .unwrap();
        let err = set_status(
            &ADR_KIND,
            &adr_root(root),
            9,
            AdrStatus::Accepted.as_str(),
            "2099-01-01",
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // --- F-1: a malformed entity missing template-seeded keys is refused ---

    #[test]
    fn set_status_on_an_adr_missing_updated_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = adr_root(dir.path()).join("003/adr-003.toml");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        // `updated` omitted; a tail `insert` would have landed it in `[relationships]`.
        fs::write(
            &p,
            "status = \"proposed\"\n\n[relationships]\nsupersedes = []\n",
        )
        .unwrap();
        let err = set_status(
            &ADR_KIND,
            &adr_root(dir.path()),
            3,
            AdrStatus::Accepted.as_str(),
            "2099-01-01",
        )
        .unwrap_err();
        assert!(err.to_string().contains("malformed"));
    }
}
