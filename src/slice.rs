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

use serde::Serialize;

use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::listing::{self, Format, ListArgs};
use crate::meta::{self, Meta};
use crate::plan::Plan;
use crate::tomlfmt::toml_string;

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
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
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

/// The slice status vocabulary — the authority `validate_statuses` checks
/// `--status` against (A-2, D10). Mirrors `slices-spec.md` § Lifecycle
/// (`{proposed, ready, started, audit, done, abandoned}`). Slice has no status
/// *enum* (unlike adr): the lifecycle is hand-advanced and write-time gating is
/// deferred, so this `&[&str]` IS the sole vocabulary authority. It guards READ
/// (filter) input only — never a stored-status write. An out-of-vocab *stored*
/// status is tolerated on disk and surfaced with a drift marker, not rejected
/// (§5.5 vocabulary-drift invariant); see [`is_drifted`].
const SLICE_STATUSES: &[&str] = &["proposed", "ready", "started", "audit", "done", "abandoned"];

/// The `slice list` hide-set (design §5.3): terminal slices — `done` (reconciled)
/// and `abandoned` (dropped before completion) — no longer govern, so they drop
/// from the default list. `--all` or any explicit `--status` reveals them (handled
/// in `listing::retain`). **Distinct from [`is_terminal_status`]**: the hide-set is
/// a presentation predicate fed only to `retain`; the divergence-terminal set
/// stays `{done}` so an `abandoned` slice with incomplete phases is not false-
/// flagged divergent. The two sets diverge deliberately — see notes / design §5.3.
fn is_hidden(status: &str) -> bool {
    matches!(status, "done" | "abandoned")
}

/// Whether an authored status is *out of vocabulary* (§5.5 vocabulary-drift
/// invariant). The read surface guards its own coherence: write-time enforcement
/// is deferred, so a hand-edited `slice-NNN.toml` may carry an unknown status. Such
/// a status is never hidden (the hide-set lists only known terminals) and renders
/// with a trailing `?` drift marker — DISTINCT from the divergence marker `⚠`
/// ([`is_divergent`]); the two are independent predicates on the same column.
fn is_drifted(status: &str) -> bool {
    !SLICE_STATUSES.contains(&status)
}

/// Whether an authored *slice* lifecycle status is terminal (work is meant to be
/// finished). The single source of the terminal-token set — the deferred slice
/// lifecycle-transition verb reuses this rather than re-hardcoding `"done"`
/// (design D3 / R-F2). v1 set: `{"done"}`; membership is provisional, the
/// predicate shape is not. Lives here, beside `is_divergent` and the future
/// transition verb — slice-authored-status semantics, not phase-runtime state.
/// **Not the list hide-set** ([`is_hidden`]): this feeds `is_divergent` only —
/// adding `abandoned` here would false-flag `⚠` on abandoned-incomplete slices.
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

/// The decorated status cell: the authored status plus, independently, a trailing
/// `?` when out of vocabulary (drift, §5.5) and a trailing ` ⚠` when the authored
/// status and the phase rollup disagree (divergence, §5.5). The two markers are
/// computed by separate predicates ([`is_drifted`] / [`is_divergent`]) and compose
/// — a drifted *and* divergent slice shows both (`bogus? ⚠`). Order is fixed:
/// drift hugs the token, the divergence marker trails.
fn decorated_status(status: &str, rollup: Option<&crate::state::PhaseRollup>) -> String {
    let drift = if is_drifted(status) { "?" } else { "" };
    let divergence = if is_divergent(status, rollup) {
        " ⚠"
    } else {
        ""
    };
    format!("{status}{drift}{divergence}")
}

/// The table grid: a header row then one `id status[?][ ⚠] phases slug title` row
/// per slice, rendered over the shared layout. The `phases`/`⚠`/`?` columns are
/// slice's variant axis (design §5.3) — they ride the grid, not `retain`. Empty
/// input → `""` (header suppressed, §5.5). Pure.
fn render_table(rows: &[(Meta, Option<crate::state::PhaseRollup>)]) -> String {
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
        grid.push(vec![
            canonical_id(m.id),
            decorated_status(&m.status, rollup.as_ref()),
            phases_cell(rollup.as_ref()),
            m.slug.clone(),
            m.title.clone(),
        ]);
    }
    listing::render_table(&grid)
}

/// One slice projected to its faithful JSON row (design §5.3 — slice owns its
/// serde shape). `phases` is a STRUCTURED value (`completed`/`total`/`blocked`),
/// NOT the rendered `4/6 !1` cell (OQ-1); `null` when phases are untracked. The
/// `?`/`⚠` table markers are display-only and do not appear here.
#[derive(Debug, Serialize)]
struct SliceRow {
    id: String,
    status: String,
    slug: String,
    title: String,
    phases: Option<PhasesJson>,
}

/// The structured `phases` value for JSON (OQ-1) — the rollup's queryable counts,
/// not its rendered cell.
#[derive(Debug, Serialize)]
struct PhasesJson {
    completed: u32,
    total: u32,
    blocked: u32,
}

/// Project a slice `Meta` to its filterable fields (design §5.2). `canonical` is
/// the prefixed id (`SL-025`) — the regex domain. Slice has no tag write verb, so
/// the tag axis is empty (parity with adr).
fn key(m: &Meta) -> listing::FilterFields {
    listing::FilterFields {
        canonical: canonical_id(m.id),
        slug: m.slug.clone(),
        title: m.title.clone(),
        status: m.status.clone(),
        tags: Vec::new(),
    }
}

/// The `SL-025` canonical id for a numeric slice id, via the single id-form
/// authority. `SLICE_KIND.prefix` is the stem (`"SL"`).
fn canonical_id(id: u32) -> String {
    listing::canonical_id(SLICE_KIND.prefix, id)
}

/// Re-export of the spine's status validator, scoped to slice so callers read
/// intent locally. Guards `--status` against [`SLICE_STATUSES`] (READ input only).
fn validate_statuses(given: &[String], known: &[&str]) -> anyhow::Result<()> {
    listing::validate_statuses(given, known)
}

/// The `slice list` rows as a string — the compute half of [`run_list`], on the
/// shared spine. `validate_statuses` guards `--status` against the slice vocab
/// (A-2); `listing::build` resolves the filter + format; `retain` applies the
/// hide-set `{done, abandoned}`; slice owns the sort (by id), the phase-rollup join
/// (its variant axis), and the column/JSON projection. The rollup is joined AFTER
/// `retain` — `retain` filters `Meta` alone, so the (impure) state read only runs
/// for the surviving rows.
pub(crate) fn list_rows(root: &Path, args: ListArgs) -> anyhow::Result<String> {
    validate_statuses(&args.status, SLICE_STATUSES)?;
    let (filter, format) = listing::build(args)?;
    let slice_root = root.join(SLICE_DIR);
    let mut metas = listing::retain(
        meta::read_metas(&slice_root, "slice")?,
        &filter,
        is_hidden,
        key,
    );
    metas.sort_by_key(|m| m.id);
    let rows: Vec<(Meta, Option<crate::state::PhaseRollup>)> = metas
        .into_iter()
        .map(|m| {
            let rollup = crate::state::phase_rollup(root, m.id)?;
            Ok((m, rollup))
        })
        .collect::<anyhow::Result<_>>()?;
    match format {
        Format::Table => Ok(render_table(&rows)),
        Format::Json => listing::json_envelope("slice", &json_rows(&rows)),
    }
}

/// Faithful JSON rows (design §5.3) — the prefixed id, the authored list fields,
/// and the structured phase rollup (OQ-1).
fn json_rows(rows: &[(Meta, Option<crate::state::PhaseRollup>)]) -> Vec<SliceRow> {
    rows.iter()
        .map(|(m, rollup)| SliceRow {
            id: canonical_id(m.id),
            status: m.status.clone(),
            slug: m.slug.clone(),
            title: m.title.clone(),
            phases: rollup.as_ref().map(|r| PhasesJson {
                completed: r.completed,
                total: r.total(),
                blocked: r.blocked,
            }),
        })
        .collect()
}

/// `doctrine slice list` — the migrated read surface (SL-025): prefixed `SL-` ids
/// and a header, the shared filter flags (`-f/-r/-i/-s/-t/-a` plus
/// `--format/--json`), the `{done, abandoned}` hide-set by default, sorted by id,
/// each row carrying the derived phase rollup (the variant axis).
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(&root, args)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// show — reassemble slice-NNN.toml (as data) + slice-NNN.md (scope body)
// ---------------------------------------------------------------------------

/// The inert `[relationships]` table, read as data for `show` (preserved on disk,
/// ignored by `Meta`). Every axis defaults to empty so a hand-trimmed file parses.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
struct Relationships {
    #[serde(default)]
    specs: Vec<String>,
    #[serde(default)]
    requirements: Vec<String>,
    #[serde(default)]
    supersedes: Vec<String>,
}

/// The full `slice-NNN.toml` read as data for `show` — `Meta`'s four list fields
/// plus the dates and the relationships table. JSON-faithful; `Meta` ignores the
/// extra keys on the list path, this surfaces them on the inspect path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
struct SliceDoc {
    id: u32,
    slug: String,
    title: String,
    status: String,
    created: String,
    updated: String,
    #[serde(default)]
    relationships: Relationships,
}

// note: `relationships` carries `#[serde(default)]` so a hand-trimmed file with
// no `[relationships]` table still parses.

/// Parse a slice reference — `SL-025`, `sl-25`, or the bare id `25` — to its
/// numeric id. The prefix is optional and case-insensitive; the id may be padded.
fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("SL-")
        .or_else(|| reference.strip_prefix("sl-"))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        format!("not a slice reference: `{reference}` (expected `SL-025` or `25`)")
    })
}

/// `doctrine slice show <SL-NNN>` — the inspect verb (SL-025 §5.2 show seam).
/// READ-ONLY: resolve the ref, read THAT slice's `slice-NNN.toml` (as data) +
/// `slice-NNN.md` (scope body), render the readable whole (`Table`) or the faithful
/// toml-as-data + body (`Json`). Reassembles **metadata + scope only** —
/// `design.md`/`plan.*`/`notes.md` are distinct artifacts with their own surfaces
/// (A-5), never folded in. No cross-corpus scan.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = parse_ref(reference)?;
    let (doc, body) = read_slice(&root.join(SLICE_DIR), id)?;
    let out = match format {
        Format::Table => format_show(&doc, &body),
        Format::Json => show_json(&doc, &body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Read one slice's `slice-NNN.toml` (as data) and `slice-NNN.md` (scope body)
/// ONLY — never design/plan/notes (A-5).
fn read_slice(slice_root: &Path, id: u32) -> anyhow::Result<(SliceDoc, String)> {
    let name = format!("{id:03}");
    let dir = slice_root.join(&name);
    let toml_path = dir.join(format!("slice-{name}.toml"));
    let text = fs::read_to_string(&toml_path)
        .with_context(|| format!("slice {name} not found at {}", toml_path.display()))?;
    let doc: SliceDoc = toml::from_str(&text)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    let md_path = dir.join(format!("slice-{name}.md"));
    let body = fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    Ok((doc, body))
}

/// Render the readable whole for `Table` mode: an identity header, the flat
/// fields, the non-empty relationship axes, then the scope body verbatim. House
/// style: `Vec<String>` parts joined by `concat` (avoids the `push_str(&format!)`
/// lint). Metadata + scope only (A-5).
fn format_show(doc: &SliceDoc, body: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", canonical_id(doc.id), doc.title));
    parts.push(format!("{} · {}\n", doc.slug, doc.status));
    parts.push(format!(
        "created {} · updated {}\n",
        doc.created, doc.updated
    ));

    let rel = &doc.relationships;
    if !rel.specs.is_empty() || !rel.requirements.is_empty() || !rel.supersedes.is_empty() {
        parts.push("\nrelationships:\n".to_string());
        for (label, refs) in [
            ("specs", &rel.specs),
            ("requirements", &rel.requirements),
            ("supersedes", &rel.supersedes),
        ] {
            if !refs.is_empty() {
                parts.push(format!("  {label}: {}\n", refs.join(", ")));
            }
        }
    }

    parts.push(format!("\n{body}"));
    parts.concat()
}

/// Render the `Json` show: the faithful toml-as-data (`SliceDoc`) plus the scope
/// body, under the shared `{kind, …}` envelope. Metadata + scope only (A-5).
fn show_json(doc: &SliceDoc, body: &str) -> anyhow::Result<String> {
    let value = serde_json::json!({ "kind": "slice", "slice": doc, "body": body });
    serde_json::to_string_pretty(&value).context("failed to serialize slice show JSON")
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

    // --- render_table (the slice grid: prefixed ids + variant axis) ---

    #[test]
    fn render_table_empty_suppresses_the_header() {
        assert_eq!(render_table(&[]), "");
    }

    #[test]
    fn render_table_renders_header_prefixed_ids_rollup_and_divergence() {
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
        let out = render_table(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("id"), "header: {:?}", lines[0]);
        assert!(lines[0].contains("phases"), "phases column: {:?}", lines[0]);
        // SL-025: prefixed ids, not bare `001`.
        // consistent terminal slice: no ⚠, full rollup
        assert!(lines[1].starts_with("SL-001  done"), "{:?}", lines[1]);
        assert!(lines[1].contains("6/6"));
        // done but 2/6 → divergent ⚠
        assert!(lines[2].starts_with("SL-007  done ⚠"), "{:?}", lines[2]);
        assert!(lines[2].contains("2/6"));
        // untracked → —
        assert!(lines[3].starts_with("SL-009  proposed"), "{:?}", lines[3]);
        assert!(lines[3].contains("—"));
        // no bare numeric id anywhere
        assert!(!out.contains("\n001  "), "no bare numeric id: {out}");
    }

    // --- decorated_status: drift `?` and divergence `⚠` are independent ---

    #[test]
    fn decorated_status_composes_drift_and_divergence() {
        // in-vocab, consistent → bare
        assert_eq!(decorated_status("proposed", None), "proposed");
        // in-vocab, divergent (done + work outstanding) → ⚠ only
        assert_eq!(decorated_status("done", Some(&rollup(2, 4))), "done ⚠");
        // out-of-vocab, consistent → `?` only (never hidden, §5.5)
        assert_eq!(decorated_status("bogus", None), "bogus?");
        // out-of-vocab AND divergent → both markers, drift hugs the token
        assert_eq!(decorated_status("bogus", Some(&rollup(6, 0))), "bogus? ⚠");
        // abandoned + incomplete is NOT divergent (terminal set stays {done}) → bare
        assert_eq!(
            decorated_status("abandoned", Some(&rollup(2, 4))),
            "abandoned"
        );
    }

    // --- is_drifted / is_hidden: vocab vs hide-set are distinct ---

    #[test]
    fn is_drifted_flags_only_out_of_vocab() {
        for s in SLICE_STATUSES {
            assert!(!is_drifted(s), "in-vocab `{s}` is not drift");
        }
        assert!(is_drifted("bogus"));
        assert!(is_drifted("superseded")); // the migrated-away value is now drift
    }

    #[test]
    fn is_hidden_is_the_terminal_presentation_set_not_divergence() {
        // hide-set: terminal slices drop from the default list
        assert!(is_hidden("done"));
        assert!(is_hidden("abandoned"));
        assert!(!is_hidden("proposed"));
        // an out-of-vocab status is NEVER hidden (§5.5)
        assert!(!is_hidden("bogus"));
        // the divergence-terminal set is narrower (done only) — abandoned is NOT
        // terminal for divergence even though it IS hidden.
        assert!(is_terminal_status("done"));
        assert!(!is_terminal_status("abandoned"));
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
    fn render_toml_escapes_hostile_title_and_slug() {
        // SL-024: quoted-literal breakers (`"`, `\`, newline) round-trip.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_toml(7, slug, title, "2026-06-03").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
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

    // --- SL-025: list_rows on the spine — prefixed ids, header, hide-set, drift ---

    fn slice_root(root: &Path) -> PathBuf {
        root.join(SLICE_DIR)
    }

    /// Raw-rewrite a created slice's authored status (slice has no status verb;
    /// this proves the list reads/filters the authored field). The status is the
    /// only field touched — `created`/`updated`/`[relationships]` survive.
    fn set_status_raw(root: &Path, id: u32, status: &str) {
        let name = format!("{id:03}");
        let p = slice_root(root)
            .join(&name)
            .join(format!("slice-{name}.toml"));
        let flipped = fs::read_to_string(&p)
            .unwrap()
            .replace("status = \"proposed\"", &format!("status = \"{status}\""));
        fs::write(&p, flipped).unwrap();
    }

    /// A no-constraint `ListArgs` (the default `slice list`).
    fn list_args() -> ListArgs {
        ListArgs::default()
    }

    #[test]
    fn list_rows_emits_prefixed_ids_and_a_header() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "first", "First", "2026-06-04");
        make_slice(root, "second", "Second", "2026-06-04");

        let out = list_rows(root, list_args()).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("id"), "header row: {:?}", lines[0]);
        assert!(lines[0].contains("phases"), "phases column named");
        assert!(out.contains("SL-001  proposed"), "prefixed id: {out}");
        assert!(out.contains("SL-002"), "second slice present: {out}");
        assert!(!out.contains("\n001  "), "no bare numeric id: {out}");
    }

    #[test]
    fn list_rows_hide_set_drops_done_and_abandoned_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "live", "Live", "2026-06-04");
        make_slice(root, "shipped", "Shipped", "2026-06-04");
        make_slice(root, "dropped", "Dropped", "2026-06-04");
        set_status_raw(root, 2, "done");
        set_status_raw(root, 3, "abandoned");

        let out = list_rows(root, list_args()).unwrap();
        assert!(out.contains("SL-001"), "live slice kept: {out}");
        assert!(!out.contains("SL-002"), "done hidden by default: {out}");
        assert!(
            !out.contains("SL-003"),
            "abandoned hidden by default: {out}"
        );
    }

    #[test]
    fn list_rows_all_and_explicit_status_reveal_the_hide_set() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "live", "Live", "2026-06-04");
        make_slice(root, "dropped", "Dropped", "2026-06-04");
        set_status_raw(root, 2, "abandoned");

        // --all reveals it.
        let all = list_rows(
            root,
            ListArgs {
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(all.contains("SL-002"), "--all reveals abandoned: {all}");

        // an explicit --status also reveals it (terminal-hide override).
        let by_status = list_rows(
            root,
            ListArgs {
                status: vec!["abandoned".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            by_status.contains("SL-002"),
            "explicit status reveals: {by_status}"
        );
        assert!(
            !by_status.contains("SL-001"),
            "and filters to it: {by_status}"
        );
    }

    #[test]
    fn list_rows_out_of_vocab_stored_status_is_never_hidden_and_drift_marked() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "weird", "Weird", "2026-06-04");
        set_status_raw(root, 1, "bogus");

        // §5.5: a drifted stored status is NOT hidden (hide-set lists known
        // terminals only) and renders with a trailing `?` drift marker.
        let out = list_rows(root, list_args()).unwrap();
        assert!(out.contains("SL-001"), "drifted slice not hidden: {out}");
        assert!(out.contains("bogus?"), "drift `?` marker present: {out}");
        // distinct from divergence: no ⚠ here (no rollup → not divergent).
        assert!(!out.contains("⚠"), "no spurious divergence marker: {out}");
    }

    #[test]
    fn list_rows_filter_matches_slug_and_title() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "use-rust", "Use Rust", "2026-06-04");
        make_slice(root, "adopt-ci", "Adopt CI", "2026-06-04");

        let out = list_rows(
            root,
            ListArgs {
                substr: Some("adopt".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.contains("SL-002"), "substr matches adopt-ci: {out}");
        assert!(!out.contains("SL-001"), "use-rust filtered out: {out}");
    }

    #[test]
    fn list_rows_regexp_matches_canonical_id() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "one", "One", "2026-06-04");
        make_slice(root, "two", "Two", "2026-06-04");

        let out = list_rows(
            root,
            ListArgs {
                regexp: Some("SL-002".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.contains("SL-002"), "regex matches canonical: {out}");
        assert!(!out.contains("SL-001"), "non-matching dropped: {out}");
    }

    #[test]
    fn list_rows_json_is_the_shared_envelope_with_structured_phases() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "first", "First", "2026-06-04");

        let out = list_rows(
            root,
            ListArgs {
                json: true,
                ..Default::default()
            },
        )
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "slice");
        let rows = parsed["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["id"], "SL-001");
        assert_eq!(rows[0]["status"], "proposed");
        assert_eq!(rows[0]["slug"], "first");
        // OQ-1: phases is structured (null when untracked), NOT a rendered cell.
        assert!(
            rows[0]["phases"].is_null(),
            "untracked phases → null: {out}"
        );
        assert!(!out.contains("4/6"), "no rendered phase cell in json");
    }

    #[test]
    fn list_rows_empty_tree_is_the_empty_string() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(list_rows(dir.path(), list_args()).unwrap(), "");
    }

    // --- VT-3: --status validates against the slice vocabulary (A-2 / D10) ---

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
        assert!(err.contains("abandoned"), "lists the known set: {err}");
    }

    #[test]
    fn list_rows_accepts_every_known_status() {
        let dir = tempfile::tempdir().unwrap();
        for s in SLICE_STATUSES {
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

    #[test]
    fn list_rows_accepts_abandoned_and_rejects_superseded() {
        // The migrated vocabulary: `abandoned` is in, `superseded` (the old ADR
        // value once stored on SL-002) is out.
        let dir = tempfile::tempdir().unwrap();
        assert!(
            list_rows(
                dir.path(),
                ListArgs {
                    status: vec!["abandoned".into()],
                    ..Default::default()
                },
            )
            .is_ok()
        );
        let err = list_rows(
            dir.path(),
            ListArgs {
                status: vec!["superseded".into()],
                ..Default::default()
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("superseded"), "superseded rejected: {err}");
    }

    /// The vocabulary known-set must mirror `slices-spec.md` § Lifecycle. Slice has
    /// no status enum, so this pins the set against the spec's stated members.
    #[test]
    fn slice_statuses_matches_the_spec_vocabulary() {
        assert_eq!(
            SLICE_STATUSES,
            &["proposed", "ready", "started", "audit", "done", "abandoned"]
        );
    }

    // --- SL-025 PHASE-06 EX-2 / VT-2: ordering-preservation through list_rows ---

    /// Write a slice's authored toml directly at an explicit id (creating its dir),
    /// bypassing the monotonic `Fresh` allocator so the fixture's creation order
    /// can differ from id order. Only the spine-read fields are written.
    fn slice_at(root: &Path, id: u32, status: &str, slug: &str, title: &str) {
        let name = format!("{id:03}");
        let dir = slice_root(root).join(&name);
        fs::create_dir_all(&dir).unwrap();
        let toml = format!(
            "id = {id}\nslug = \"{slug}\"\ntitle = \"{title}\"\nstatus = \"{status}\"\ncreated = \"2026-06-04\"\nupdated = \"2026-06-04\"\n"
        );
        fs::write(dir.join(format!("slice-{name}.toml")), toml).unwrap();
    }

    #[test]
    fn list_rows_orders_by_id_ascending_regardless_of_creation_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Create OUT of id order: 003, then 001, then 002.
        slice_at(root, 3, "proposed", "gamma", "Gamma");
        slice_at(root, 1, "proposed", "alpha", "Alpha");
        slice_at(root, 2, "proposed", "beta", "Beta");

        let out = list_rows(root, list_args()).unwrap();
        let off = |id: &str| {
            out.find(id)
                .unwrap_or_else(|| panic!("{id} present: {out}"))
        };
        assert!(
            off("SL-001") < off("SL-002") && off("SL-002") < off("SL-003"),
            "slice rows must render in ascending id order (sort, not read order): {out}"
        );
    }

    // --- VT-4: slice show — table + json, metadata + scope only (A-5) ---

    #[test]
    fn parse_ref_accepts_prefixed_padded_and_bare_ids() {
        assert_eq!(parse_ref("SL-025").unwrap(), 25);
        assert_eq!(parse_ref("sl-25").unwrap(), 25);
        assert_eq!(parse_ref("25").unwrap(), 25);
        assert_eq!(parse_ref("002").unwrap(), 2);
        assert!(parse_ref("nope").is_err());
    }

    #[test]
    fn read_slice_reassembles_toml_as_data_and_md_scope_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-04");

        let (doc, body) = read_slice(&slice_root(root), 1).unwrap();
        assert_eq!(doc.id, 1);
        assert_eq!(doc.slug, "my-slug");
        assert_eq!(doc.status, "proposed");
        // the inert relationships table parses as data (empty by default).
        assert!(doc.relationships.specs.is_empty());
        // the md scope body is read verbatim.
        assert!(body.contains("My Title"));
    }

    #[test]
    fn format_show_renders_identity_and_scope_body() {
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            relationships: Relationships {
                specs: vec!["PRD-010".into()],
                requirements: vec![],
                supersedes: vec![],
            },
        };
        let out = format_show(&doc, "# Scope\n\nthe scope body.\n");
        assert!(out.contains("SL-025 — Uniform CLI"), "identity: {out}");
        assert!(out.contains("uniform-cli · started"), "flat fields: {out}");
        assert!(out.contains("created 2026-06-01 · updated 2026-06-08"));
        assert!(out.contains("specs: PRD-010"), "relationships axis: {out}");
        assert!(
            out.contains("the scope body."),
            "scope body appended: {out}"
        );
    }

    #[test]
    fn show_does_not_fold_in_design_plan_or_notes() {
        // A-5: slice show reassembles metadata + scope ONLY. Even with sibling
        // artifacts on disk, neither table nor json surfaces them.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-04");
        let sr = slice_root(root);
        fs::write(sr.join("001/design.md"), "DESIGN_SECRET").unwrap();
        fs::write(sr.join("001/plan.md"), "PLAN_SECRET").unwrap();
        fs::write(sr.join("001/notes.md"), "NOTES_SECRET").unwrap();

        let (doc, body) = read_slice(&sr, 1).unwrap();
        let table = format_show(&doc, &body);
        let json = show_json(&doc, &body).unwrap();
        for needle in ["DESIGN_SECRET", "PLAN_SECRET", "NOTES_SECRET"] {
            assert!(!table.contains(needle), "table leaked {needle}: {table}");
            assert!(!json.contains(needle), "json leaked {needle}: {json}");
        }
    }

    #[test]
    fn show_json_is_faithful_toml_as_data_plus_scope_body() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "my-slug", "My Title", "2026-06-04");
        let (doc, body) = read_slice(&slice_root(root), 1).unwrap();

        let out = show_json(&doc, &body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "slice");
        assert_eq!(parsed["slice"]["id"], 1);
        assert_eq!(parsed["slice"]["slug"], "my-slug");
        assert_eq!(parsed["slice"]["status"], "proposed");
        assert!(parsed["slice"]["relationships"]["specs"].is_array());
        assert!(parsed["body"].as_str().unwrap().contains("My Title"));
    }

    #[test]
    fn run_show_on_a_missing_slice_errors() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_show(Some(dir.path().to_path_buf()), "SL-009", Format::Table).unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
    }
}
