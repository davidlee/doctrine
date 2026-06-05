// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine slice` — create, list, and add design-doc siblings to slices,
//! doctrine's unit of change.
//!
//! A slice is a numeric directory under `.doctrine/slice/` holding a sister
//! TOML (structured metadata) and a scaffolded markdown prose body, with a
//! `<id>-<slug>` symlink as a human alias (slices-spec). A design-doc sibling is
//! a single prose `design.md` under an existing slice dir.
//!
//! Both are `entity::Kind` values over one kind-blind engine: the slice is a
//! top-level reserved 2-file-plus-symlink kind, the design doc a non-reserved
//! single-file sub-artefact. This module owns the *slice-specific* parts — the
//! Kinds and their scaffolds, the `Plan` reader, and thin CLI wiring; the
//! kind-agnostic machinery lives in `crate::entity`, and the shared
//! metadata-list substrate (`Meta`, list reader/formatter) in `crate::meta`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::meta;
use crate::plan::Plan;

/// Relative dir of the slice tree inside the project root.
const SLICE_DIR: &str = ".doctrine/slice";

/// The top-level reserved slice kind: toml + md + slug symlink.
const SLICE_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: "SL",
    scaffold: slice_scaffold,
};

/// The non-reserved design-doc sibling: one `design.md` under an existing slice.
const DESIGN_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: "SL",
    scaffold: design_scaffold,
};

/// The implementation-plan facet: `plan.toml` (authored relational `plan.overview`
/// rows) + `plan.md` (prose) under an existing slice — the first multi-file
/// sub-artefact, on the transactional writer (slice-004 D1/D4).
const PLAN_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: "SL",
    scaffold: plan_scaffold,
};

/// The durable per-slice notes scratchpad: one `notes.md` under an existing
/// slice (the `design.md` single-file pattern; on-demand, slice-004 D8).
const NOTES_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: "SL",
    scaffold: notes_scaffold,
};

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
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/slice-{name}.toml")),
            body: render_toml(id, ctx.slug, ctx.title, ctx.date)?,
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
    let (id, canonical) = (ctx.id, ctx.canonical);
    let name = format!("{id:03}");
    Ok(vec![Artifact::File {
        rel_path: PathBuf::from(format!("{name}/design.md")),
        body: render_design(canonical, ctx.title)?,
    }])
}

/// Render `plan.toml` from the template: `{{ref}}` is the parent canonical id.
fn render_plan_toml(canonical_id: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/plan.toml")?.replace("{{ref}}", canonical_id))
}

/// Render `plan.md` from the template: `{{ref}}` + parent `{{title}}`.
fn render_plan_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/plan.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The IP fileset: authored `plan.toml` + prose `plan.md` under the slice dir.
fn plan_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let (id, canonical) = (ctx.id, ctx.canonical);
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/plan.toml")),
            body: render_plan_toml(canonical)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/plan.md")),
            body: render_plan_md(canonical, ctx.title)?,
        },
    ])
}

/// Render `notes.md` from the template: `{{ref}}` + parent `{{title}}`.
fn render_notes(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/notes.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The notes fileset: one durable `notes.md` under the parent slice dir.
fn notes_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let (id, canonical) = (ctx.id, ctx.canonical);
    let name = format!("{id:03}");
    Ok(vec![Artifact::File {
        rel_path: PathBuf::from(format!("{name}/notes.md")),
        body: render_notes(canonical, ctx.title)?,
    }])
}

// ---------------------------------------------------------------------------
// Imperative: the slice-specific reader (clock lives in crate::clock,
// the shared metadata reader in crate::meta)
// ---------------------------------------------------------------------------

/// Read and validate a slice's authored `plan.toml`.
fn read_plan(slice_root: &Path, id: u32) -> anyhow::Result<Plan> {
    let name = format!("{id:03}");
    let path = slice_root.join(&name).join("plan.toml");
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Plan for slice {name} not found at {}", path.display()))?;
    Plan::parse(&text)
}

// ---------------------------------------------------------------------------
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// `doctrine slice new`.
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
        &SLICE_KIND,
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
        .context("slice kind must yield a numeric id")?;
    writeln!(io::stdout(), "Created slice {id:03}: {}", out.dir.display())?;
    Ok(())
}

/// `doctrine slice design <id>` — scaffold `design.md` into an existing slice.
pub(crate) fn run_design(path: Option<PathBuf>, id: u32) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    // The design doc inherits its parent's title (the only context its template
    // needs); reading it confirms the parent exists before we materialise.
    let meta = meta::read_meta(&slice_root, "slice", id)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        &DESIGN_KIND,
        &LocalFs,
        &root,
        &MaterialiseRequest::InExisting { id },
        &Inputs {
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

/// `doctrine slice plan <id>` — scaffold `plan.{toml,md}` into an existing slice.
pub(crate) fn run_plan(path: Option<PathBuf>, id: u32) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    // Reading the parent confirms it exists and supplies the prose title.
    let meta = meta::read_meta(&slice_root, "slice", id)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        &PLAN_KIND,
        &LocalFs,
        &root,
        &MaterialiseRequest::InExisting { id },
        &Inputs {
            slug: "",
            title: &meta.title,
            date: &date,
        },
    )?;

    writeln!(
        io::stdout(),
        "Created implementation plan: {}",
        out.dir.join("plan.toml").display()
    )?;
    Ok(())
}

/// `doctrine slice phases <id>` — read the plan and materialise phase tracking
/// into the state tree. Reports plan drift (orphans); `--prune` removes them.
pub(crate) fn run_phases(path: Option<PathBuf>, id: u32, prune: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let plan = read_plan(&slice_root, id)?;
    let report = crate::state::init_phases(&root, id, &plan, prune)?;

    let mut out = io::stdout();
    for phase_id in &report.created {
        writeln!(out, "  materialised {phase_id}")?;
    }
    for phase_id in &report.orphan {
        writeln!(
            out,
            "  orphan       {phase_id} (plan phase gone; --prune to remove)"
        )?;
    }
    for phase_id in &report.pruned {
        writeln!(out, "  pruned       {phase_id}")?;
    }
    if report.created.is_empty() && report.orphan.is_empty() && report.pruned.is_empty() {
        writeln!(out, "Phases up to date.")?;
    }
    Ok(())
}

/// `doctrine slice notes <id>` — scaffold a durable `notes.md` into a slice.
pub(crate) fn run_notes(path: Option<PathBuf>, id: u32) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let meta = meta::read_meta(&slice_root, "slice", id)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        &NOTES_KIND,
        &LocalFs,
        &root,
        &MaterialiseRequest::InExisting { id },
        &Inputs {
            slug: "",
            title: &meta.title,
            date: &date,
        },
    )?;

    writeln!(
        io::stdout(),
        "Created notes: {}",
        out.dir.join("notes.md").display()
    )?;
    Ok(())
}

/// `doctrine slice phase <id> <phase-id> --status <s> [--note …]` — fold a
/// runtime status transition into the phase tracking (the `toml_edit` path).
pub(crate) fn run_phase(
    path: Option<PathBuf>,
    id: u32,
    phase_id: &str,
    status: crate::state::PhaseStatus,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let now = crate::clock::now_timestamp()?;
    crate::state::set_phase_status(&root, id, phase_id, status, note, &now)?;
    writeln!(io::stdout(), "Updated {phase_id}: {}", status.as_str())?;
    Ok(())
}

/// Whether an authored *slice* lifecycle status is terminal (work is meant to be
/// finished). The single source of the terminal-token set — the deferred slice
/// lifecycle-transition verb reuses this rather than re-hardcoding `"done"`
/// (design D3 / R-F2). v1 set: `{"done"}`; membership is provisional, the
/// predicate shape is not. Lives here, beside `is_divergent` and the future
/// transition verb — slice-authored-status semantics, not phase-runtime state.
fn is_terminal_status(authored: &str) -> bool {
    authored == "done"
}

/// Whether the authored status and the derived phase rollup disagree (design
/// § 5.5). Conservative: suppressed when tracking is anomalous (corruption is not
/// a lifecycle mismatch) or untracked, and keyed on `is_terminal_status` — never
/// a bare `"done"` literal — so a future terminal synonym stops false-flagging in
/// one place.
fn is_divergent(authored: &str, rollup: Option<&crate::state::PhaseRollup>) -> bool {
    let Some(r) = rollup else { return false };
    if r.anomalies() > 0 {
        return false;
    }
    let terminal = is_terminal_status(authored);
    // marked terminal, work outstanding | work complete, not marked terminal
    (terminal && r.completed < r.total())
        || (!terminal && r.total() > 0 && r.completed == r.total())
}

/// The `phases` cell: `completed/total`, with a `!N` blocked marker and a `?N`
/// anomaly marker appended when non-zero; `—` when untracked.
fn phases_cell(rollup: Option<&crate::state::PhaseRollup>) -> String {
    let Some(r) = rollup else {
        return "—".to_string();
    };
    let blocked = if r.blocked > 0 {
        format!(" !{}", r.blocked)
    } else {
        String::new()
    };
    let anomalies = if r.anomalies() > 0 {
        format!(" ?{}", r.anomalies())
    } else {
        String::new()
    };
    format!("{}/{}{blocked}{anomalies}", r.completed, r.total())
}

/// Render slice rows with the derived phase rollup. Human-only output: a header
/// row plus `id status[⚠] phases slug title`, aligned via the shared
/// `meta::render_table`. Empty input → `""` (header suppressed). Pure.
fn format_slice_rows(rows: &[(meta::Meta, Option<crate::state::PhaseRollup>)]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut grid: Vec<Vec<String>> = vec![
        ["id", "status", "phases", "slug", "title"]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    ];
    for (m, rollup) in rows {
        let status = if is_divergent(&m.status, rollup.as_ref()) {
            format!("{} ⚠", m.status)
        } else {
            m.status.clone()
        };
        grid.push(vec![
            format!("{:03}", m.id),
            status,
            phases_cell(rollup.as_ref()),
            m.slug.clone(),
            m.title.clone(),
        ]);
    }
    meta::render_table(&grid)
}

/// `doctrine slice list`.
pub(crate) fn run_list(path: Option<PathBuf>, status: Option<&str>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let metas = meta::sort_and_filter(meta::read_metas(&slice_root, "slice")?, status);
    let rows: Vec<(meta::Meta, Option<crate::state::PhaseRollup>)> = metas
        .into_iter()
        .map(|m| {
            let rollup = crate::state::phase_rollup(&root, m.id)?;
            Ok((m, rollup))
        })
        .collect::<anyhow::Result<_>>()?;

    let mut out = io::stdout();
    write!(out, "{}", format_slice_rows(&rows))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;

    fn meta(id: u32, status: &str, slug: &str, title: &str) -> Meta {
        Meta {
            id,
            slug: slug.to_string(),
            title: title.to_string(),
            status: status.to_string(),
        }
    }

    use crate::state::PhaseRollup;

    /// A rollup with the given completed/planned counts (total = sum); other
    /// buckets default to zero unless a test overrides them.
    fn rollup(completed: u32, planned: u32) -> PhaseRollup {
        PhaseRollup {
            completed,
            planned,
            ..Default::default()
        }
    }

    // --- is_divergent ---

    #[test]
    fn divergence_flags_the_two_unambiguous_mismatches() {
        // marked terminal ("done"), work outstanding
        assert!(is_divergent("done", Some(&rollup(2, 4))));
        // work complete, not marked terminal
        assert!(is_divergent("proposed", Some(&rollup(6, 0))));
    }

    #[test]
    fn divergence_is_quiet_when_consistent_untracked_or_anomalous() {
        // terminal + complete → consistent
        assert!(!is_divergent("done", Some(&rollup(6, 0))));
        // non-terminal + incomplete → consistent
        assert!(!is_divergent("proposed", Some(&rollup(2, 4))));
        // untracked → nothing to compare
        assert!(!is_divergent("done", None));
        // anomalies present → corruption, not a lifecycle mismatch (suppressed)
        let anomalous = PhaseRollup {
            completed: 2,
            planned: 3,
            unknown: 1,
            ..Default::default()
        };
        assert!(!is_divergent("done", Some(&anomalous)));
    }

    // --- phases_cell ---

    #[test]
    fn phases_cell_renders_markers() {
        assert_eq!(phases_cell(None), "—");
        assert_eq!(phases_cell(Some(&rollup(4, 2))), "4/6");
        let blocked = PhaseRollup {
            completed: 2,
            planned: 3,
            blocked: 1,
            ..Default::default()
        };
        assert_eq!(phases_cell(Some(&blocked)), "2/6 !1");
        let anomalous = PhaseRollup {
            completed: 3,
            planned: 2,
            unknown: 1,
            ..Default::default()
        };
        assert_eq!(phases_cell(Some(&anomalous)), "3/6 ?1");
    }

    // --- format_slice_rows ---

    #[test]
    fn format_slice_rows_empty_suppresses_the_header() {
        assert_eq!(format_slice_rows(&[]), "");
    }

    #[test]
    fn format_slice_rows_renders_header_rollup_and_divergence() {
        let rows = vec![
            (
                meta(1, "done", "entity-v1", "Entity v1"),
                Some(rollup(6, 0)),
            ),
            (
                meta(7, "done", "anchoring", "Anchoring"),
                Some(rollup(2, 4)),
            ),
            (meta(9, "proposed", "rollup", "Rollup"), None),
        ];
        let out = format_slice_rows(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "id   status    phases  slug       title");
        // consistent terminal slice: no ⚠, full rollup
        assert!(lines[1].starts_with("001  done      6/6     entity-v1"));
        // done but 2/6 → divergent ⚠
        assert!(lines[2].starts_with("007  done ⚠    2/6     anchoring"));
        // untracked → —
        assert!(lines[3].starts_with("009  proposed  —       rollup"));
    }

    /// Materialise a slice the way `run_new` does, for behaviour-preservation
    /// tests (the slice-001 gate).
    fn make_slice(root: &Path, slug: &str, title: &str, date: &str) -> entity::Materialised {
        entity::materialise(
            &SLICE_KIND,
            &LocalFs,
            root,
            &MaterialiseRequest::Fresh,
            &Inputs { slug, title, date },
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
            canonical: "SL-003",
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
    fn render_plan_toml_substitutes_ref_and_parses() {
        let body = render_plan_toml("SL-004").unwrap();
        assert!(body.contains("slice   = \"SL-004\""));
        assert!(!body.contains("{{ref}}"));
        // it is valid TOML carrying the plan.overview shape
        let doc: toml::Value = toml::from_str(&body).unwrap();
        assert_eq!(doc["schema"].as_str(), Some("doctrine.plan.overview"));
        assert_eq!(doc["version"].as_integer(), Some(1));
        assert_eq!(doc["phase"][0]["id"].as_str(), Some("PHASE-01"));
    }

    #[test]
    fn render_plan_md_substitutes_ref_and_title() {
        let body = render_plan_md("SL-004", "My Title").unwrap();
        assert!(body.contains("Implementation Plan SL-004: My Title"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
    }

    #[test]
    fn plan_scaffold_lays_out_toml_and_md() {
        let ctx = ScaffoldCtx {
            id: 4,
            canonical: "SL-004",
            slug: "",
            title: "Plan title",
            date: "2026-06-04",
        };
        let fileset = plan_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 2);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("004/plan.toml") && body.contains("SL-004")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("004/plan.md") && body.contains("Plan title")));
    }

    // --- Plan read model (pure parser tests live in `crate::plan`; SL-016) ---

    #[test]
    fn plan_parse_accepts_the_scaffold_template() {
        let body = render_plan_toml("SL-004").unwrap();
        let plan = Plan::parse(&body).unwrap();
        assert_eq!(plan.phases.len(), 1);
        assert_eq!(plan.phases[0].id, "PHASE-01");
    }

    #[test]
    fn design_scaffold_is_a_single_file_no_symlink() {
        let ctx = ScaffoldCtx {
            id: 3,
            canonical: "SL-003",
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

        assert_eq!(s.eid.numeric_id(), Some(1));
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

    // --- list: slice ↔ meta integration (the pure list helpers are unit-tested
    //     in crate::meta; this proves `slice list` reads what `slice new` writes) ---

    #[test]
    fn meta_read_metas_round_trips_a_created_slice() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-03");

        let metas = meta::read_metas(&root.join(SLICE_DIR), "slice").unwrap();
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
            &MaterialiseRequest::InExisting { id: 1 },
            &Inputs {
                slug: "",
                title: "My Title",
                date: "2026-06-03",
            },
        )
        .unwrap();

        assert_eq!(out.eid.numeric_id(), Some(1));
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
            &MaterialiseRequest::InExisting { id: 1 },
            &Inputs {
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

    // --- plan facet: the first multi-file sub-artefact ---

    #[test]
    fn plan_materialises_two_files_under_an_existing_slice() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-04");
        let slice_root = root.join(SLICE_DIR);

        let out = entity::materialise(
            &PLAN_KIND,
            &LocalFs,
            root,
            &MaterialiseRequest::InExisting { id: 1 },
            &Inputs {
                slug: "",
                title: "My Title",
                date: "2026-06-04",
            },
        )
        .unwrap();

        assert_eq!(out.eid.numeric_id(), Some(1));
        let toml_body = fs::read_to_string(slice_root.join("001/plan.toml")).unwrap();
        assert!(toml_body.contains("slice   = \"SL-001\""));
        let md_body = fs::read_to_string(slice_root.join("001/plan.md")).unwrap();
        assert!(md_body.contains("Implementation Plan SL-001: My Title"));
        // no second numeric dir, no extra symlink
        assert!(!slice_root.join("002").exists());
    }

    #[test]
    fn plan_refuses_to_clobber_an_existing_plan() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-04");
        let slice_root = root.join(SLICE_DIR);
        fs::write(slice_root.join("001/plan.toml"), "hand-written").unwrap();

        let err = entity::materialise(
            &PLAN_KIND,
            &LocalFs,
            root,
            &MaterialiseRequest::InExisting { id: 1 },
            &Inputs {
                slug: "",
                title: "My Title",
                date: "2026-06-04",
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Refusing to overwrite"));
        assert_eq!(
            fs::read_to_string(slice_root.join("001/plan.toml")).unwrap(),
            "hand-written"
        );
        // the partial sibling write was rolled back — no plan.md leftover
        assert!(!slice_root.join("001/plan.md").exists());
    }

    // --- notes facet: durable single-file scaffold ---

    #[test]
    fn notes_materialises_under_an_existing_slice() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-04");
        let slice_root = root.join(SLICE_DIR);

        entity::materialise(
            &NOTES_KIND,
            &LocalFs,
            root,
            &MaterialiseRequest::InExisting { id: 1 },
            &Inputs {
                slug: "",
                title: "My Title",
                date: "2026-06-04",
            },
        )
        .unwrap();

        let body = fs::read_to_string(slice_root.join("001/notes.md")).unwrap();
        assert!(body.contains("Notes SL-001: My Title"));
    }

    #[test]
    fn notes_refuses_to_clobber() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-04");
        let slice_root = root.join(SLICE_DIR);
        fs::write(slice_root.join("001/notes.md"), "hand-written").unwrap();

        let err = entity::materialise(
            &NOTES_KIND,
            &LocalFs,
            root,
            &MaterialiseRequest::InExisting { id: 1 },
            &Inputs {
                slug: "",
                title: "My Title",
                date: "2026-06-04",
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Refusing to overwrite"));
        assert_eq!(
            fs::read_to_string(slice_root.join("001/notes.md")).unwrap(),
            "hand-written"
        );
    }
}
