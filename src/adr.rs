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

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::meta;
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

/// `doctrine adr list` — rows of `id status slug title`, sorted by id; `--status`
/// keeps only matching ADRs. Reads the authored `adr-NNN.toml` status field (D5 —
/// status is authored, not symlink-indexed). The stem is `"adr"`, not the `ADR`
/// prefix; `read_metas` is unsorted, so `sort_and_filter` owns the ordering.
pub(crate) fn run_list(path: Option<PathBuf>, status: Option<&str>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(&root, status)?)?;
    Ok(())
}

/// The `adr list` rows as a string — the compute half of `run_list`, extracted so
/// the boot snapshot (SL-011) can project the same rows in-process without a
/// subprocess. `run_list` prints this verbatim, so output is byte-identical
/// (`format_list` already carries its own trailing newline; no extra is added).
pub(crate) fn list_rows(root: &Path, status: Option<&str>) -> anyhow::Result<String> {
    let adr_root = root.join(ADR_DIR);
    let rows = meta::sort_and_filter(meta::read_metas(&adr_root, "adr")?, status);
    Ok(meta::format_list(&rows))
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
        let title = "a\"b\\c\nd";
        let slug = "p\"q";
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

        // list (the run_list pipeline): both ADRs, sorted by id.
        let all = meta::sort_and_filter(meta::read_metas(&adr, "adr").unwrap(), None);
        assert_eq!(all.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1, 2]);

        // authored mutation via the real verb core (not a rewrite).
        set_adr_status(&adr, 1, AdrStatus::Accepted, &crate::clock::today()).unwrap();

        // list --status accepted: only 001 survives the filter.
        let accepted =
            meta::sort_and_filter(meta::read_metas(&adr, "adr").unwrap(), Some("accepted"));
        assert_eq!(accepted.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1]);
    }

    // --- SL-011: list_rows is run_list's compute, byte-identical to format_list ---

    #[test]
    fn list_rows_filters_by_status_and_is_empty_when_none_match() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        run_new(Some(root.clone()), Some("Use Rust".into()), None).unwrap();
        run_new(Some(root.clone()), Some("Adopt CI".into()), None).unwrap();
        set_adr_status(
            &adr_root(&root),
            1,
            AdrStatus::Accepted,
            &crate::clock::today(),
        )
        .unwrap();

        // unfiltered → both rows, exactly what format_list renders for the pipeline.
        let all = meta::sort_and_filter(meta::read_metas(&adr_root(&root), "adr").unwrap(), None);
        assert_eq!(list_rows(&root, None).unwrap(), meta::format_list(&all));

        // accepted → only 001's row.
        let accepted = list_rows(&root, Some("accepted")).unwrap();
        assert!(accepted.contains("001  accepted"));
        assert!(!accepted.contains("adopt-ci"));

        // a status nobody holds → the empty string (the agreed empty marker upstream).
        assert_eq!(list_rows(&root, Some("superseded")).unwrap(), "");
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

        // read_metas is unsorted; sort_and_filter owns the ordering (VT-3).
        let all = meta::sort_and_filter(meta::read_metas(&adr, "adr").unwrap(), None);
        assert_eq!(all.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(
            all[0],
            Meta {
                id: 1,
                slug: "use-rust".into(),
                title: "Use Rust".into(),
                status: "proposed".into(),
            }
        );

        let accepted =
            meta::sort_and_filter(meta::read_metas(&adr, "adr").unwrap(), Some("accepted"));
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].id, 2);
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
