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

use crate::dtoml;
use crate::entity::{
    self, Artifact, Fileset, Inputs, Kind, LocalFs, MaterialiseRequest, ScaffoldCtx,
};
use crate::lifecycle::{Transition, classify, crosses_closure_seam};
use crate::listing::{self, Format, ListArgs};
use crate::meta::{self, Meta};
use crate::plan::Plan;
use crate::tomlfmt::toml_string;

use std::str::FromStr;

use clap::Subcommand;

// ---------------------------------------------------------------------------
// Selector types (SL-147 PHASE-01)
// ---------------------------------------------------------------------------

/// The author-declared relationship between a selector and the slice's work.
/// `scope-relevant` = read-relevant (L0); `design-target` = will-touch (L1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectorIntent {
    ScopeRelevant,
    DesignTarget,
}

/// One authored `[[selector]]` entry in the slice's TOML — a path|glob
/// annotation carrying an intent and an optional one-line note.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
pub(crate) struct Selector {
    /// Path or glob string — identity of the selector.
    #[expect(
        clippy::struct_field_names,
        reason = "`selector` is the canonical noun from RFC-004"
    )]
    pub(crate) selector: String,
    pub(crate) intent: SelectorIntent,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) note: Option<String>,
}

/// Subcommands under `doctrine slice selector`.
#[derive(Subcommand)]
pub(crate) enum SelectorCommand {
    /// Add or update one or more selectors (batch, one shared intent).
    Add {
        /// Slice id, e.g. 147.
        id: u32,

        /// One intent for all selectors in this batch.
        #[arg(long, value_parser = clap::value_parser!(SelectorIntent))]
        intent: SelectorIntent,

        /// Path-or-glob selector strings to add/update.
        globs: Vec<String>,

        /// Optional shared note.
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Upsert the `note` field on one selector.
    Note {
        /// Slice id.
        id: u32,

        /// The exact selector string to annotate.
        selector: String,

        /// The note text.
        text: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List the selectors for a slice.
    List {
        /// Slice id.
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Remove one or more selectors by exact string match.
    Rm {
        /// Slice id.
        id: u32,

        /// Selector strings to remove.
        globs: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

// ---------------------------------------------------------------------------
// CLI enum & dispatch (PHASE-03 relocation from main.rs)
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub(crate) enum SliceCommand {
    /// Allocate the next id and scaffold a new slice.
    New {
        /// Slice title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Scaffold a design-doc sibling into an existing slice.
    Design {
        /// Slice id to attach the design doc to.
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Scaffold an implementation plan (plan.toml + plan.md) into a slice.
    Plan {
        /// Slice id to attach the plan to.
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Materialise phase tracking from a slice's plan into the state tree.
    Phases {
        /// Slice id whose plan declares the phases.
        id: u32,

        /// Remove orphan tracking whose plan phase is gone (destructive).
        #[arg(long)]
        prune: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Scaffold a durable notes.md scratchpad into a slice (on-demand).
    Notes {
        /// Slice id to attach the notes file to.
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Record a phase status transition into its runtime tracking.
    Phase {
        /// Slice id owning the phase.
        id: u32,

        /// Canonical phase id, e.g. PHASE-01.
        phase_id: String,

        /// New status.
        #[arg(long)]
        status: crate::state::PhaseStatus,

        /// Optional note appended to the progress log.
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Classify and write a slice lifecycle transition; prints the move's
    /// classification (advance / back-edge / skip / abandon). Refuses the closure
    /// seam (→ reconcile only from audit, → done only from reconcile) and leaving
    /// a terminal status (done / abandoned).
    Status {
        /// Slice id to transition.
        id: u32,

        /// Target lifecycle state.
        state: SliceStatus,

        /// Optional note — surfaced in the transition output, not stored.
        #[arg(long)]
        note: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List slices by id: id, status, phases, slug, title.
    List {
        #[command(flatten)]
        list: crate::CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one slice: its metadata and scope body (not design/plan/notes).
    Show {
        /// Slice reference — `SL-025` or the bare id `25`.
        reference: String,

        /// Output format.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each slice entity directory.
    Paths {
        /// Slice reference(s) — `SL-025` or the bare id `25`.
        refs: Vec<String>,

        #[arg(short = 't', long)]
        toml: bool,
        #[arg(short = 'm', long)]
        md: bool,
        #[arg(short = 'e', long)]
        entity: bool,
        #[arg(short = 's', long)]
        single: bool,

        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Manage the selector list (path|glob + intent) for a slice.
    Selector {
        #[command(subcommand)]
        command: SelectorCommand,
    },
}

pub(crate) fn dispatch(cmd: SliceCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        SliceCommand::New { title, slug, path } => run_new(path, title, slug),
        SliceCommand::Design { id, path } => run_design(path, id),
        SliceCommand::Plan { id, path } => run_plan(path, id),
        SliceCommand::Phases { id, prune, path } => run_phases(path, id, prune),
        SliceCommand::Notes { id, path } => run_notes(path, id),
        SliceCommand::Phase {
            id,
            phase_id,
            status,
            note,
            path,
        } => run_phase(path, id, &phase_id, status, note.as_deref()),
        SliceCommand::Status {
            id,
            state,
            note,
            path,
        } => run_status(path, id, state, note.as_deref()),
        SliceCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        SliceCommand::Show {
            reference,
            format,
            json,
            path,
        } => run_show(path, &reference, if json { Format::Json } else { format }),
        SliceCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let slice_root = root.join(SLICE_DIR);
            let sel = crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            };
            let mut all_lines: Vec<String> = Vec::new();
            for r in &refs {
                let id = parse_ref(r)?;
                let name = format!("{id:03}");
                let entity_dir = slice_root.join(&name);
                let toml_name = format!("slice-{name}.toml");
                let md_name = format!("slice-{name}.md");
                let set = crate::paths::scan_entity_dir(
                    &entity_dir,
                    &entity_dir.join(&toml_name),
                    Some(&entity_dir.join(&md_name)),
                    &root,
                )?;
                let lines = crate::paths::select_paths(&set, &sel)?;
                all_lines.extend(lines);
            }
            write!(io::stdout(), "{}", all_lines.join("\n"))?;
            Ok(())
        }
        SliceCommand::Selector { command } => dispatch_selector(command),
    }
}

// ---------------------------------------------------------------------------

/// Relative dir of the slice tree inside the project root.
const SLICE_DIR: &str = ".doctrine/slice";

/// The top-level reserved slice kind: toml + md + slug symlink.
pub(crate) const SLICE_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: crate::kinds::SL,
    stem: "slice",
    scaffold: slice_scaffold,
};

/// The non-reserved design-doc sibling: one `design.md` under an existing slice.
const DESIGN_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: crate::kinds::SL,
    stem: "",
    scaffold: design_scaffold,
};

/// The implementation-plan facet: `plan.toml` (authored relational `plan.overview`
/// rows) + `plan.md` (prose) under an existing slice — the first multi-file
/// sub-artefact, on the transactional writer (slice-004 D1/D4).
const PLAN_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: crate::kinds::SL,
    stem: "",
    scaffold: plan_scaffold,
};

/// The durable per-slice notes scratchpad: one `notes.md` under an existing
/// slice (the `design.md` single-file pattern; on-demand, slice-004 D8).
const NOTES_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: crate::kinds::SL,
    stem: "",
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
            rel_path: entity::rel_path(&SLICE_KIND, id, entity::Ext::Toml),
            body: render_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: entity::rel_path(&SLICE_KIND, id, entity::Ext::Md),
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
pub(crate) fn read_plan(slice_root: &Path, id: u32) -> anyhow::Result<Plan> {
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
    let trunk_ids = crate::git::trunk_entity_ids(&root, SLICE_KIND.dir)?;
    let (backend, mut reserved) =
        crate::reserve::backend(&root, SLICE_KIND.prefix, crate::install::prompt_confirm)?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();
    let out = entity::materialise(
        &SLICE_KIND,
        &*backend,
        &root,
        &MaterialiseRequest::Fresh,
        &Inputs {
            slug: &slug,
            title: &title,
            date: &date,
        },
        &trunk_ids,
        &mut reserved,
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
    let meta = meta::read_meta(&slice_root, "slice", id, "SL")?;
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
        &[], // inert for InExisting (trunk ids only affect Fresh allocation)
        &mut entity::local_reserved(), // inert for InExisting (no id allocation)
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
    let meta = meta::read_meta(&slice_root, "slice", id, "SL")?;
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
        &[], // inert for InExisting (trunk ids only affect Fresh allocation)
        &mut entity::local_reserved(), // inert for InExisting (no id allocation)
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
    let meta = meta::read_meta(&slice_root, "slice", id, "SL")?;
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
        &[], // inert for InExisting (trunk ids only affect Fresh allocation)
        &mut entity::local_reserved(), // inert for InExisting (no id allocation)
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

/// `doctrine slice status <id> <state> [--note …]` — classify and write a slice
/// lifecycle transition (SL-028, design §5.2). Reads the current authored status,
/// classifies the move via [`classify`], writes it edit-preservingly, and prints
/// the classification (e.g. `started → audit [advance]`). The `--note` is
/// *surfaced only*, never stored: `slice-NNN.toml` has no progress-log field
/// (storage rule — runtime progress lives under `.doctrine/state/`); a stored
/// rationale would be a new authored field, out of scope (plan Decisions).
pub(crate) fn run_status(
    path: Option<PathBuf>,
    id: u32,
    state: SliceStatus,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let from = read_status(&slice_root, id)?;
    let to = state.as_str();
    let kind = classify(&from, to);
    // Reverse close-gate (design §7, D8/D-C9b): the gate lives in this close
    // COMMAND SHELL, not the FSM writer (`set_slice_status`) — keeping the writer
    // focused and isolating the one-way `slice-shell → review-query` coupling
    // (ADR-001: `review` never imports `slice`). It fires ONLY on a closure-seam
    // crossing (`audit→reconcile`, `reconcile→done`); a non-seam transition is
    // never gated. A SOLE seam-crossing caller of `set_slice_status` (this shell)
    // means the gate cannot be bypassed (`set_slice_status_is_the_sole_seam_crosser`
    // pins it). The teeth are HERE in the binary — the `slice status …` refusal —
    // not in skill prose.
    if crosses_closure_seam(&from, to) {
        let blockers = crate::review::unresolved_blockers_for(&root, &canonical_id(id))?;
        if !blockers.is_empty() {
            let listed = blockers
                .iter()
                .map(|b| format!("{}/{}", b.rv, b.finding))
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!(
                "slice {} → {to}: refused — unresolved blocker review finding(s): {listed} \
                 (resolve via `review verify`/`review withdraw`, then retry)",
                canonical_id(id)
            );
        }
    }
    // Closure-gate drift predicate (D-B5/D-B3, REQ-113/FR-006): a SECOND gate
    // BESIDE the blocker scan, firing on the SAME closure seam but SPECIFICALLY on
    // the `reconcile → done` crossing only. It COMPOSES with the blocker gate —
    // either can independently refuse this crossing. Undischarged residual drift on
    // any requirement in the gate set (`covered ∪ declared ∪ reconciled`) refuses.
    if from == "reconcile" && to == "done" {
        let undischarged = undischarged_drift(&root, id)?;
        if !undischarged.is_empty() {
            anyhow::bail!(
                "slice {} → {to}: refused — undischarged residual drift on \
                 requirement(s): {} (reconcile each via an `accept` REC whose evidence \
                 covers the current drift, or resolve the drift, then retry)",
                canonical_id(id),
                undischarged.join(", "),
            );
        }
    }
    // Close-integration gate (PHASE-02, EX-1/EX-2; design §3.1): a THIRD gate
    // BESIDE the blocker scan and the drift predicate, firing ONLY on the
    // `reconcile → done` crossing. It COMPOSES — a separate `if` from the drift
    // block, so any one of the three can independently refuse. Dispatched code
    // that never integrated to trunk fails-closed here, in the binary, not in
    // skill prose. The new in-crate edge is `slice → ledger` (command→leaf,
    // downward — no cycle, ADR-001); `ledger` stays ref-agnostic, so the
    // `"refs/heads/main"` literal and the refusal copy live HERE in the shell.
    if from == "reconcile" && to == "done" {
        let deliver_to = crate::dtoml::load_doctrine_toml(&root)?.dispatch.deliver_to;
        match crate::ledger::trunk_integration(&root, id, &deliver_to)? {
            crate::ledger::TrunkIntegration::NotDispatched
            | crate::ledger::TrunkIntegration::Integrated => {}
            crate::ledger::TrunkIntegration::Blocked(reason) => anyhow::bail!(
                "slice {} → {to}: refused — dispatched code not integrated to trunk: \
                 {reason} (run close step-3a `dispatch sync --integrate`, verify, retry)",
                canonical_id(id)
            ),
        }
    }
    set_slice_status(&slice_root, id, &from, state, &crate::clock::today())?;
    // Advisory conduct posture (F15/F19): the SOURCE state's exit posture —
    // `autonomy` governs advancing *out* of `from`. Never blocks; surfaced only.
    let cfg = load_conduct(&root)?;
    let posture = crate::conduct::resolve(&cfg, &from);
    writeln!(
        io::stdout(),
        "{}",
        status_line(&from, to, kind, posture, note)
    )?;
    Ok(())
}

/// Read the project `doctrine.toml [conduct]` table into a [`conduct::ConductConfig`]
/// — the impure shell seam that keeps `conduct` pure (ADR-001). An absent file
/// falls back to the default config (= baked defaults on resolve); a present file
/// is parsed tolerantly (F9), erroring only on genuinely malformed TOML.
fn load_conduct(root: &Path) -> anyhow::Result<crate::conduct::ConductConfig> {
    Ok(crate::dtoml::load_doctrine_toml(root)?.conduct)
}

/// The `slice status` output line (pure — composed from already-resolved data):
/// `{from} → {to} [{classification}] [{posture}]{ — note}`. The posture is the
/// SOURCE state's exit conduct (F19), advisory only. Factored out so the format
/// is unit-testable without capturing stdout (VT-3).
fn status_line(
    from: &str,
    to: &str,
    kind: Transition,
    posture: crate::conduct::Conduct,
    note: Option<&str>,
) -> String {
    let suffix = note.map(|n| format!(" — {n}")).unwrap_or_default();
    format!(
        "{from} → {to} [{}] [{}]{suffix}",
        transition_label(kind),
        posture.label()
    )
}

/// The lower-case label for a [`Transition`] in the verb's output line.
fn transition_label(kind: Transition) -> &'static str {
    match kind {
        Transition::Advance => "advance",
        Transition::BackEdge => "back-edge",
        Transition::Skip => "skip",
        Transition::Abandon => "abandon",
        Transition::Noop => "no-op",
        Transition::FromTerminal => "from-terminal",
        Transition::SeamBreach => "seam-breach",
    }
}

/// Read the current authored `status` of `slice-NNN.toml` (the `from` of a
/// transition). Distinct from the no-op/scaffold guards in [`set_slice_status`]:
/// this surfaces the value for classification + the output line.
fn read_status(slice_root: &Path, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = slice_root.join(&name).join(format!("slice-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("slice {name} not found at {}", path.display()))?;
    let doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    doc.get("status")
        .and_then(toml_edit::Item::as_str)
        .map(str::to_string)
        .with_context(|| format!("malformed slice {name}: missing `status`"))
}

/// Edit-preserving lifecycle transition on one authored `slice-NNN.toml`: gate
/// the move via [`classify`] (refuse `FromTerminal` and `SeamBreach`, F12/F13),
/// then set `status` + stamp `updated`. Mirrors `adr::set_adr_status` — the
/// `toml_edit` in-place mutation preserves the inert `[relationships]` table,
/// comments, and unknown keys (the file is never reserialised); carries the
/// no-op guard and the F-1 malformed refuse. The `from` is supplied by the caller
/// (already read for classification), the date by the shell. Unlike adr's flat
/// any→any setter, this is an *ordered* FSM, so the classification gates the write.
fn set_slice_status(
    slice_root: &Path,
    id: u32,
    from: &str,
    state: SliceStatus,
    today: &str,
) -> anyhow::Result<()> {
    let to = state.as_str();
    let name = format!("{id:03}");

    // Gate before any disk write (design §5.2): refuse leaving a terminal source
    // and the two closure-seam breaches; everything else (advance/back/skip/
    // abandon/no-op) is allowed to write. `to` is in-vocab (the `ValueEnum`).
    match classify(from, to) {
        Transition::FromTerminal => anyhow::bail!(
            "slice {name}: refusing to leave terminal status `{from}` (reopening is deferred)"
        ),
        Transition::SeamBreach => anyhow::bail!(
            "slice {name}: `{to}` is reachable only across the closure seam \
             (→ reconcile from audit, → done from reconcile), not from `{from}`"
        ),
        _ => {}
    }

    // Gate passed; delegate the write-core (no-op guard + F-1 refuse + edit-
    // preserving insert) to the shared authored-TOML seam. The classification above
    // stays in this shell — only the WRITE is shared. Hint preserved verbatim (the
    // behaviour-preservation gate scopes the EX-4 rewording to gov + requirement).
    let path = slice_root.join(&name).join(format!("slice-{name}.toml"));
    let hint = format!(
        "malformed slice {name}: missing `status`/`updated` — restore the missing keys and retry; the file is left untouched"
    );
    crate::dep_seq::set_authored_status(&path, &[("status", to), ("updated", today)], &hint)?;
    Ok(())
}

/// The slice status vocabulary — the authority `validate_statuses` checks
/// `--status` against (A-2, D10) and the `SliceStatus` `ValueEnum` mirrors. The
/// expanded SL-028 FSM vocabulary (`slices-spec.md` § Lifecycle): `{proposed,
/// design, plan, ready, started, audit, reconcile, done, abandoned}` — purely
/// additive over the original six (no `review` state, F11), so existing slices
/// need no migration. It guards READ (filter) input only — the write verb
/// (`set_slice_status`) classifies a move via [`classify`] and refuses the
/// closure seam / a terminal source, but an out-of-vocab *stored* status is
/// tolerated on disk and surfaced with a drift marker, not rejected (§5.5
/// vocabulary-drift invariant); see [`is_drifted`].
pub(crate) const SLICE_STATUSES: &[&str] = &[
    "proposed",
    "design",
    "plan",
    "ready",
    "started",
    "audit",
    "reconcile",
    "done",
    "abandoned",
];

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

/// The slice lifecycle status as a clap `ValueEnum` — the `slice status <state>`
/// argument. Mirrors [`SLICE_STATUSES`]; the two are pinned in lockstep by
/// `slice_status_enum_matches_the_vocabulary` (a drift canary, cf. adr's
/// `adr_known_set_matches_variants`). Unlike adr, slice keeps the `&[&str]` const
/// as the read-filter authority too, so this enum is the *write*-path mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum SliceStatus {
    Proposed,
    Design,
    Plan,
    Ready,
    Started,
    Audit,
    Reconcile,
    Done,
    Abandoned,
}

impl SliceStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Design => "design",
            Self::Plan => "plan",
            Self::Ready => "ready",
            Self::Started => "started",
            Self::Audit => "audit",
            Self::Reconcile => "reconcile",
            Self::Done => "done",
            Self::Abandoned => "abandoned",
        }
    }
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

/// The table columns `slice list` can show (`--columns` tokens over the existing
/// row tuple `R = (Meta, Option<PhaseRollup>)`, SL-037 §4). Extractors are
/// non-capturing `fn(&R)->String` (D5): the `?`/`⚠` drift+divergence markers ride
/// the `status` cell *value* via [`decorated_status`], and the `completed/total`
/// rollup rides the `phases` cell via [`phases_cell`] — neither is a separate
/// column or per-kind config (the SL-037 R1 canary: markers absorb as plain
/// cell values). Declaration order is what the unknown-column error lists.
type SliceRowTuple = (Meta, Option<crate::state::PhaseRollup>);

const SLICE_COLUMNS: [listing::Column<SliceRowTuple>; 5] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |(m, _)| canonical_id(m.id),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: |(m, r)| decorated_status(&m.status, r.as_ref()),
        // F-4: hue from the row's RAW `m.status`, NOT the decorated `done ⚠`/`bogus?`
        // cell — matching the emitted cell would drop colour on decorated rows.
        paint: listing::ColumnPaint::ByValue(|(m, _)| listing::status_hue(&m.status)),
    },
    listing::Column {
        name: "phases",
        header: "phases",
        cell: |(_, r)| phases_cell(r.as_ref()),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "slug",
        header: "slug",
        cell: |(m, _)| m.slug.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "title",
        header: "title",
        cell: |(m, _)| m.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
];

/// The default visible set — slug-free (SL-037 D4); `--columns …,slug` reveals it.
/// `phases` (the variant axis) stays in the default, between status and title.
const SLICE_DEFAULT: &[&str] = &["id", "status", "phases", "title"];

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
        tags: m.tags.clone(),
    }
}

/// The `SL-025` canonical id for a numeric slice id, via the single id-form
/// authority. `SLICE_KIND.prefix` is the stem (`"SL"`).
fn canonical_id(id: u32) -> String {
    listing::canonical_id(SLICE_KIND.prefix, id)
}

// ---------------------------------------------------------------------------
// Closure-gate drift predicate (D-B5/D-B3, REQ-113/FR-006). The shell resolves
// coverage / RECs / authored status from disk+git (ADR-001 impure half); the
// discharge DECISION is pure over those resolved values. One-way coupling: this
// `slice`-close shell QUERIES `coverage`/`coverage_scan`/`rec`/`requirement`; those
// modules NEVER import `slice` — same direction as the blocker gate's
// `slice-shell → review-query` edge (ADR-001).
// ---------------------------------------------------------------------------

/// Read the authored `[gate].extra_reqs` declaration of slice `id` (the `declared`
/// term of the gate set). Absent `[gate]` table ⇒ `∅` (`#[serde(default)]`). Reads
/// only the toml — no `.md` body needed (cf. `read_slice`).
fn read_gate_extra_reqs(slice_root: &Path, id: u32) -> anyhow::Result<Vec<String>> {
    let name = format!("{id:03}");
    let path = slice_root.join(&name).join(format!("slice-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("slice {name} not found at {}", path.display()))?;
    let doc: SliceDoc =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(doc.gate.extra_reqs)
}

/// The closure-gate requirement set (D-B5, LOCKED) = `covered ∪ declared ∪
/// reconciled` for the closing slice `id`, sorted + DISTINCT. ADDITIVE by
/// construction — the gate can never check LESS than this union. Impure (reads
/// disk): `covered` from S's own `coverage.toml`, `declared` from `[gate]`,
/// `reconciled` from the `status_delta`s of S's owning RECs (codex finding 3 — you
/// cannot reconcile a req via a REC then dodge its gate by not covering/declaring
/// it).
fn gate_requirement_set(
    root: &Path,
    id: u32,
    owned_recs: &[crate::rec::RecDoc],
) -> anyhow::Result<Vec<String>> {
    let canonical = canonical_id(id);
    let slice_root = root.join(SLICE_DIR);

    let mut set = std::collections::BTreeSet::new();
    // covered — S's OWN coverage.toml, validated to cite only S (R-B4).
    set.extend(crate::coverage_scan::slice_local_covered_reqs(
        root, id, &canonical,
    )?);
    // declared — the authored additive `[gate].extra_reqs`.
    set.extend(read_gate_extra_reqs(&slice_root, id)?);
    // reconciled — every req named in a status_delta of S's owning RECs (passed in
    // by the caller; the corpus is read once per close, not per helper).
    set.extend(
        owned_recs
            .iter()
            .flat_map(|rec| rec.status_delta.iter().map(|d| d.requirement.clone())),
    );
    Ok(set.into_iter().collect())
}

/// The requirements in the closing slice's gate set that carry UNDISCHARGED residual
/// drift (D-B5/D-B3). For each gate req R: compute today's residual drift (authored
/// status vs the composite of its scanned coverage); `Coherent` ⇒ no drift, skip.
/// `Divergent`/`Indeterminate` ⇒ residual drift — EXCUSED iff R's latest
/// owning-slice REC discharges it ([`rec_discharges`]); otherwise R is undischarged.
/// Impure (disk+git resolution); the per-req discharge DECISION is pure.
fn undischarged_drift(root: &Path, id: u32) -> anyhow::Result<Vec<String>> {
    let canonical = canonical_id(id);
    let owned_recs = crate::rec::recs_owned_by(root, &canonical)?;
    let mut undischarged = Vec::new();
    for req in gate_requirement_set(root, id, &owned_recs)? {
        let entries = crate::coverage_scan::scan_coverage(root, &req);
        let composite = crate::coverage::composite(&entries);
        let authored = crate::requirement::load(root, &req)
            .with_context(|| format!("closure gate: requirement {req} not found"))?
            .status;
        // Residual drift: anything but Coherent. Coherent ⇒ nothing to discharge.
        if matches!(
            crate::coverage::drift(authored, &composite),
            crate::coverage::Verdict::Coherent
        ) {
            continue;
        }
        // The CURRENT residual-drift evidence keys feeding R's composite — the keys
        // `scan_coverage` returned, DEDUPED (ISS-006: the corpus double-walks a
        // slice's dir + slug symlink, so a key can recur; do not over-demand).
        let residual_keys = crate::coverage::distinct_keys(entries.into_iter().map(|(e, _)| e.key));
        let latest = latest_owning_rec_for(&owned_recs, &req);
        if !rec_discharges(latest, &req, authored, &residual_keys) {
            undischarged.push(req);
        }
    }
    Ok(undischarged)
}

/// R's LATEST owning-slice REC naming R in a `status_delta` (D-B3 / ADR-004):
/// the on-demand reverse scan — filter the already-read owning RECs to those whose
/// `status_delta` names R, take MAX id ("latest"; REC ids are authored + monotonic).
/// NO stored `req→last_rec` field, NO reverse index (the denormalization ADR-004
/// prevents). `None` ⇒ no owning REC reconciled R, so nothing can discharge it.
fn latest_owning_rec_for<'a>(
    owned_recs: &'a [crate::rec::RecDoc],
    req: &str,
) -> Option<&'a crate::rec::RecDoc> {
    owned_recs
        .iter()
        .filter(|rec| rec.status_delta.iter().any(|d| d.requirement == req))
        .max_by_key(|rec| rec.id)
}

/// The discharge predicate (R-B3, LOCKED — strengthened). PURE over resolved
/// values: residual drift on R is EXCUSED iff R's latest owning-slice REC satisfies
/// ALL THREE clauses —
/// (a) `rec.move == "accept"` (an affirm — not revise/redesign);
/// (b) its `status_delta` **for R** has `to == authored` (R's CURRENT authored
///     status — guards a status edited away-and-back). The delta must name R: a REC
///     may carry deltas for several requirements (hand-authored TOML), so matching
///     on `to` alone would let one requirement's delta discharge ANOTHER's drift.
/// (c) its `evidence_ref` set ⊇ the current residual-drift evidence keys (so fresh
///     contradictory evidence arriving AFTER the REC re-opens drift a stale REC
///     cannot excuse).
/// `None` (no owning REC for R) ⇒ never discharged.
fn rec_discharges(
    latest: Option<&crate::rec::RecDoc>,
    req: &str,
    authored: crate::requirement::ReqStatus,
    residual_keys: &[crate::coverage::CoverageKey],
) -> bool {
    let Some(rec) = latest else { return false };
    // (a) an affirm.
    if rec.rec.r#move != "accept" {
        return false;
    }
    // (b) affirmed FOR R at the value R now holds (the delta must name R, not just
    // carry a coinciding `to` for some other requirement).
    let affirmed_at_current = rec
        .status_delta
        .iter()
        .any(|d| d.requirement == req && d.to == authored.as_str());
    if !affirmed_at_current {
        return false;
    }
    // (c) the REC's evidence ⊇ today's residual evidence keys.
    residual_keys
        .iter()
        .all(|k| rec.evidence_ref.iter().any(|e| e == k))
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
pub(crate) fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    validate_statuses(&args.status, SLICE_STATUSES)?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let slice_root = root.join(SLICE_DIR);
    let mut metas = listing::retain(
        meta::read_metas(&slice_root, "slice", "SL")?,
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
        Format::Table => {
            let sel = listing::select_columns(&SLICE_COLUMNS, SLICE_DEFAULT, columns.as_deref())?;
            Ok(listing::render_columns(&rows, &sel, render))
        }
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

/// The full `slice-NNN.toml` read as data for `show` — `Meta`'s four list fields
/// plus the dates and the `[gate]` table. JSON-faithful; `Meta` ignores the extra
/// keys on the list path, this surfaces them on the inspect path.
///
/// SL-048 PHASE-04 (the cut): the tier-1 relations (`specs`/`requirements`/
/// `supersedes`/`governed_by`) no longer live in a typed `[relationships]` table —
/// they migrated to uniform `[[relation]]` rows, read by `relation::read_block`. The
/// show paths (`relation_edges`/`format_show`/`show_json`) reconstruct them from the
/// raw TOML text, so this struct no longer carries a `relationships` field (slice has
/// no typed tier-2/3 leftovers). `gate` stays typed.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, Serialize)]
struct SliceDoc {
    id: u32,
    slug: String,
    title: String,
    status: String,
    created: String,
    updated: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    gate: Gate,
    #[serde(default)]
    estimate: Option<crate::estimate::EstimateFacet>,
    #[serde(default)]
    value: Option<crate::value::ValueFacet>,
    /// Selectors — path|glob annotations with intent (SL-147).
    #[serde(default, rename = "selector")]
    selectors: Vec<Selector>,
}

// note: `gate` carries `#[serde(default)]` so a hand-trimmed file with no `[gate]`
// table still parses.

/// The authored `[gate]` table (D-B5): the closure-gate's *declared* requirement
/// term — an additive, risk-calibrated list the slice's own closure gate is
/// answerable for, ON TOP OF its observed coverage. `#[serde(default)]` everywhere
/// so a slice with no `[gate]` table parses to `extra_reqs = ∅` (declared = ∅, the
/// gate still runs on `covered ∪ reconciled`). Authored slice-metadata tier — NOT
/// the observed `coverage.toml`.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
struct Gate {
    #[serde(default)]
    extra_reqs: Vec<String>,
}

/// Parse a slice reference — `SL-025`, `sl-25`, or the bare id `25` — to its
/// numeric id. The prefix is optional and case-insensitive; the id may be padded.
pub(crate) fn parse_ref(reference: &str) -> anyhow::Result<u32> {
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
    let (doc, toml_text, body) = read_slice(&root.join(SLICE_DIR), id)?;
    // Tier-1 relations now live in `[[relation]]`, read generically (SL-048 PHASE-04).
    let tier1 = crate::relation::tier1_edges(&SLICE_KIND, &toml_text)?;
    // The dep/seq payload axes (`needs`/`after`) live in the typed `[relationships]`
    // table (SL-060), read via the shared leaf off the same `slice-NNN.toml` path. An
    // absent table reads to an empty `DepSeq` (a slice that authors no dep/seq).
    let dep_seq = crate::dep_seq::read(&slice_toml_path(&root.join(SLICE_DIR), id))?;
    let out = match format {
        Format::Table => {
            let cfg = crate::dtoml::load_doctrine_toml(&root)?;
            let posture = crate::conduct::resolve(&cfg.conduct, &doc.status);
            let estimation_unit = crate::estimate::resolve_unit(&cfg.estimation);
            let value_unit = crate::value::resolve_unit(&cfg.value);
            let (lower_pct, upper_pct) = crate::estimate::resolve_confidence(&cfg.estimation)?;
            let facets = crate::facet::EntityFacets {
                estimate: doc.estimate.clone(),
                value: doc.value.clone(),
                risk: None,
                tags: doc.tags.clone(),
            };
            format_show(
                &doc,
                &tier1,
                &dep_seq,
                &body,
                posture,
                &facets,
                &estimation_unit,
                &value_unit,
                lower_pct,
                upper_pct,
            )
        }
        // JSON stays byte-stable — posture is a Table-line addition only (design §5.2).
        Format::Json => show_json(&doc, &tier1, &dep_seq, &body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// The on-disk `slice-NNN.toml` metadata path for a slice id under `slice_root`.
/// The single chokepoint the show path and the dep/seq leaf read both key on (DRY).
fn slice_toml_path(slice_root: &Path, id: u32) -> PathBuf {
    let name = format!("{id:03}");
    slice_root.join(&name).join(format!("slice-{name}.toml"))
}

/// Read one slice's `slice-NNN.toml` (as data) and `slice-NNN.md` (scope body)
/// ONLY — never design/plan/notes (A-5). Returns the parsed `SliceDoc`, the raw TOML
/// `text` (so the tier-1 `[[relation]]` block can be read by `relation::read_block` —
/// SL-048 PHASE-04), and the scope body.
fn read_slice(slice_root: &Path, id: u32) -> anyhow::Result<(SliceDoc, String, String)> {
    let name = format!("{id:03}");
    let dir = slice_root.join(&name);
    let toml_path = slice_toml_path(slice_root, id);
    let text = fs::read_to_string(&toml_path)
        .with_context(|| format!("slice {name} not found at {}", toml_path.display()))?;
    let doc: SliceDoc = dtoml::parse_entity_toml(&text, "SL", id)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    let md_path = dir.join(format!("slice-{name}.md"));
    let body = fs::read_to_string(&md_path)
        .with_context(|| format!("Failed to read {}", md_path.display()))?;
    Ok((doc, text, body))
}

/// The slice's authored outbound relations (SL-046 §5.2 extraction seam). SL-048
/// PHASE-04 (the cut): the tier-1 axes (`specs`/`requirements`/`supersedes`/
/// `governed_by`) now live in uniform `[[relation]]` rows, read generically via
/// `relation::tier1_edges` in canonical [`RELATION_RULES`] order (X1). Slice has NO
/// typed tier-2/3 leftovers, so the tier-1 edges ARE the whole edge set. Reads the
/// raw `slice-NNN.toml` text via the show-path reader (no new TOML parse — cohesion).
pub(crate) fn relation_edges(
    root: &Path,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    let (_doc, toml_text, _body) = read_slice(&root.join(SLICE_DIR), id)?;
    crate::relation::tier1_edges(&SLICE_KIND, &toml_text)
}

/// Render the readable whole for `Table` mode: an identity header, the flat
/// fields, the advisory conduct posture line (`resolve(current)`, F15/F19), the
/// non-empty relationship axes, then the scope body verbatim. House style:
/// `Vec<String>` parts joined by `concat` (avoids the `push_str(&format!)`
/// lint). Metadata + scope only (A-5). SL-048 PHASE-04: the tier-1 axes come from
/// `tier1` (read via `read_block`), not a typed `[relationships]` table; the new
/// `governed_by` axis renders only when populated (additive — no current slice
/// authors it, so render output is byte-identical across the migration).
#[expect(
    clippy::too_many_arguments,
    reason = "format_show consolidates all rendering inputs; splitting into a struct adds indirection without reducing coupling"
)]
fn format_show(
    doc: &SliceDoc,
    tier1: &[crate::relation::RelationEdge],
    dep_seq: &crate::dep_seq::DepSeq,
    body: &str,
    posture: crate::conduct::Conduct,
    facets: &crate::facet::EntityFacets,
    estimation_unit: &str,
    value_unit: &str,
    lower_pct: f64,
    upper_pct: f64,
) -> String {
    use crate::relation::{RelationLabel, Role, targets_for, targets_for_role};
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", canonical_id(doc.id), doc.title));
    parts.push(format!("{} · {}\n", doc.slug, doc.status));
    // Advisory conduct posture for the current state (F15/F19) — Table only.
    parts.push(format!("conduct: {}\n", posture.label()));
    // Tags — rendered only when non-empty (additive).
    if !doc.tags.is_empty() {
        parts.push(format!("tags: {}\n", doc.tags.join(", ")));
    }
    parts.push(format!(
        "created {} · updated {}\n",
        doc.created, doc.updated
    ));

    // Estimate row — confidence-framed when present (D2). Absent → no row (D3).
    if let Some(ref est) = facets.estimate {
        parts.push(format!(
            "{}\n",
            crate::estimate::display::format_estimate_confidence(
                est,
                lower_pct,
                upper_pct,
                estimation_unit,
            )
        ));
    }

    // Value row — magnitude + resolved unit (D4). Absent → no row (D3).
    if let Some(ref val) = facets.value {
        parts.push(format!(
            "{}\n",
            crate::value::format_value_normal(val, value_unit)
        ));
    }

    // Tier-1 axes in canonical order (supersedes, governed_by, then references per
    // role). Each renders only when non-empty, the block only when any axis is
    // populated. SL-149 PHASE-05: the legacy `specs`/`requirements` axes are gone —
    // the corpus migration rewrote those edges onto `references(implements)` /
    // `references(concerns)`, which render per role, one line each.
    let axes = [
        ("supersedes", targets_for(tier1, RelationLabel::Supersedes)),
        ("governed_by", targets_for(tier1, RelationLabel::GovernedBy)),
        (
            "references(implements)",
            targets_for_role(tier1, RelationLabel::References, Role::Implements),
        ),
        (
            "references(scoped_from)",
            targets_for_role(tier1, RelationLabel::References, Role::ScopedFrom),
        ),
        (
            "references(concerns)",
            targets_for_role(tier1, RelationLabel::References, Role::Concerns),
        ),
    ];
    // The dep/seq payload axes (SL-060) render under the SAME `relationships:` block,
    // after the structural tier-1 axes. `after` edges render `to (rank N)` (rank
    // suffix omitted at the default 0). Each axis (and the whole block) renders only
    // when populated — an unauthored slice's output stays byte-identical (additive).
    let after_line = dep_seq
        .after
        .iter()
        .map(|e| {
            if e.rank == 0 {
                e.to.clone()
            } else {
                format!("{} (rank {})", e.to, e.rank)
            }
        })
        .collect::<Vec<_>>();
    let any_tier1 = axes.iter().any(|(_, refs)| !refs.is_empty());
    let any_dep_seq = !dep_seq.needs.is_empty() || !after_line.is_empty();
    if any_tier1 || any_dep_seq {
        parts.push("\nrelationships:\n".to_string());
        for (label, refs) in &axes {
            if !refs.is_empty() {
                parts.push(format!("  {label}: {}\n", refs.join(", ")));
            }
        }
        if !dep_seq.needs.is_empty() {
            parts.push(format!("  needs: {}\n", dep_seq.needs.join(", ")));
        }
        if !after_line.is_empty() {
            parts.push(format!("  after: {}\n", after_line.join(", ")));
        }
    }

    parts.push(format!("\n{body}"));
    parts.concat()
}

/// Render the `Json` show: the faithful toml-as-data (`SliceDoc`) plus the scope
/// body, under the shared `{kind, …}` envelope. Metadata + scope only (A-5).
///
/// SL-048 PHASE-04 (R2-C2′): the tier-1 relations migrated out of the typed struct
/// into `[[relation]]`, so the serialized `SliceDoc` no longer carries them — they
/// are reconstructed here from `tier1` (read via `read_block`) and spliced back into
/// the `slice` object under the SAME `relationships` key, preserving the byte-exact
/// JSON shape (OD-2). The `supersedes` axis is ALWAYS present (empty `[]` when
/// unauthored); `governed_by` is emitted only when populated. SL-149 PHASE-05: the
/// legacy `specs`/`requirements` keys are gone — the corpus migration rewrote those
/// edges onto the `references` role object below. `serde_json` sorts object keys, so
/// the emitted order is alphabetical.
fn show_json(
    doc: &SliceDoc,
    tier1: &[crate::relation::RelationEdge],
    dep_seq: &crate::dep_seq::DepSeq,
    body: &str,
) -> anyhow::Result<String> {
    use crate::relation::{RelationLabel, Role, targets_for, targets_for_role};
    let mut relationships = serde_json::Map::new();
    relationships.insert(
        "supersedes".to_string(),
        serde_json::json!(targets_for(tier1, RelationLabel::Supersedes)),
    );
    let governed_by = targets_for(tier1, RelationLabel::GovernedBy);
    if !governed_by.is_empty() {
        relationships.insert("governed_by".to_string(), serde_json::json!(governed_by));
    }
    // SL-149: the `references` label projected by role into a sibling object
    // `{ implements, scoped_from, concerns }`, each an array of targets. ALWAYS present
    // (the three role keys present, empty `[]` when unauthored — the legacy-axis
    // convention). PHASE-05's corpus migration moved the old `specs`/`requirements`
    // edges into `references(implements)`/`references(concerns)` here.
    relationships.insert(
        "references".to_string(),
        serde_json::json!({
            "implements": targets_for_role(tier1, RelationLabel::References, Role::Implements),
            "scoped_from": targets_for_role(tier1, RelationLabel::References, Role::ScopedFrom),
            "concerns": targets_for_role(tier1, RelationLabel::References, Role::Concerns),
        }),
    );
    // The dep/seq payload axes (SL-060). Additive — emitted only when populated, so an
    // unauthored slice's JSON object stays byte-identical to the pre-SL-060 shape.
    // `after` serializes as `[{ to, rank }, …]` (the `AfterEdge` derive).
    if !dep_seq.needs.is_empty() {
        relationships.insert("needs".to_string(), serde_json::json!(dep_seq.needs));
    }
    if !dep_seq.after.is_empty() {
        relationships.insert("after".to_string(), serde_json::json!(dep_seq.after));
    }
    let mut slice = serde_json::to_value(doc).context("failed to serialize slice doc")?;
    if let Some(obj) = slice.as_object_mut() {
        obj.insert(
            "relationships".to_string(),
            serde_json::Value::Object(relationships),
        );
    }
    let value = serde_json::json!({ "kind": "slice", "slice": slice, "body": body });
    serde_json::to_string_pretty(&value).context("failed to serialize slice show JSON")
}

// ---------------------------------------------------------------------------
// Selector handlers (SL-147 PHASE-01)
// ---------------------------------------------------------------------------

fn dispatch_selector(cmd: SelectorCommand) -> anyhow::Result<()> {
    match cmd {
        SelectorCommand::Add {
            id,
            intent,
            globs,
            note,
            path,
        } => run_selector_add(path, id, intent, &globs, note.as_deref()),
        SelectorCommand::Note {
            id,
            selector,
            text,
            path,
        } => run_selector_note(path, id, &selector, &text),
        SelectorCommand::List { id, path } => run_selector_list(path, id),
        SelectorCommand::Rm { id, globs, path } => run_selector_rm(path, id, &globs),
    }
}

/// Open a slice's TOML as a `toml_edit::DocumentMut` for edit-preserving writes.
fn open_selector_doc(root: &Path, id: u32) -> anyhow::Result<(PathBuf, toml_edit::DocumentMut)> {
    let slice_root = root.join(SLICE_DIR);
    let toml_path = slice_toml_path(&slice_root, id);
    let text = fs::read_to_string(&toml_path)
        .with_context(|| format!("slice {} not found at {}", id, toml_path.display()))?;
    let doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    Ok((toml_path, doc))
}

fn run_selector_add(
    path: Option<PathBuf>,
    id: u32,
    intent: SelectorIntent,
    globs: &[String],
    note: Option<&str>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (toml_path, mut doc) = open_selector_doc(&root, id)?;

    let intent_str = serde_rename_kebab(intent);
    for glob in globs {
        selector_upsert(&mut doc, glob, intent_str, note)?;
    }

    crate::fsutil::write_atomic(&toml_path, doc.to_string().as_bytes())?;
    let cid = canonical_id(id);
    writeln!(
        io::stdout(),
        "{cid}: upserted {} selector(s)\n",
        globs.len()
    )?;
    Ok(())
}

fn run_selector_note(
    path: Option<PathBuf>,
    id: u32,
    selector: &str,
    text: &str,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (toml_path, mut doc) = open_selector_doc(&root, id)?;

    let found = selector_set_note(&mut doc, selector, text);
    if !found {
        let cid = canonical_id(id);
        anyhow::bail!("{cid}: no selector `{selector}` — add it first");
    }

    crate::fsutil::write_atomic(&toml_path, doc.to_string().as_bytes())?;
    writeln!(io::stdout(), "{selector}: note set")?;
    Ok(())
}

fn run_selector_list(path: Option<PathBuf>, id: u32) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let (doc, _toml_text, _body) = read_slice(&slice_root, id)?;

    if doc.selectors.is_empty() {
        writeln!(io::stdout(), "(no selectors)")?;
        return Ok(());
    }
    for s in &doc.selectors {
        let note = s.note.as_deref().unwrap_or("-");
        writeln!(
            io::stdout(),
            "{}  {: <16}  {}",
            s.selector,
            serde_rename_kebab(s.intent),
            note
        )?;
    }
    Ok(())
}

fn run_selector_rm(path: Option<PathBuf>, id: u32, globs: &[String]) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (toml_path, mut doc) = open_selector_doc(&root, id)?;

    let removed = selector_remove_many(&mut doc, globs);
    if removed > 0 {
        crate::fsutil::write_atomic(&toml_path, doc.to_string().as_bytes())?;
    }
    writeln!(io::stdout(), "Removed {removed} selector(s)")?;
    Ok(())
}

// -- pure helpers (edit `toml_edit::DocumentMut`) ----------------------------

/// Serialise a `SelectorIntent` as its kebab-case wire value.
fn serde_rename_kebab(intent: SelectorIntent) -> &'static str {
    match intent {
        SelectorIntent::ScopeRelevant => "scope-relevant",
        SelectorIntent::DesignTarget => "design-target",
    }
}

/// Upsert one selector row into `doc`'s `[[selector]]` array-of-tables.
/// Identity = the `selector` string. When a row with the same string exists,
/// its `intent` (and optionally `note`) are updated in-place rather than
/// appended.
fn selector_upsert(
    doc: &mut toml_edit::DocumentMut,
    glob: &str,
    intent: &str,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let array = doc
        .as_table_mut()
        .entry("selector")
        .or_insert_with(|| toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .ok_or_else(|| {
            anyhow::anyhow!("`selector` key exists but is not an array-of-tables (corrupt file)")
        })?;

    // In-place update if the selector string already exists.
    for row in array.iter_mut() {
        if row
            .get("selector")
            .and_then(toml_edit::Item::as_str)
            .is_some_and(|s| s == glob)
        {
            row.insert("intent", toml_edit::value(intent));
            if let Some(n) = note {
                row.insert("note", toml_edit::value(n));
            }
            return Ok(());
        }
    }

    // Append a new row.
    let mut row = toml_edit::Table::new();
    row.insert("selector", toml_edit::value(glob));
    row.insert("intent", toml_edit::value(intent));
    if let Some(n) = note {
        row.insert("note", toml_edit::value(n));
    }
    array.push(row);
    Ok(())
}

/// Set `note` on the selector row whose `selector` string matches exactly.
/// Returns `true` when a row was found and updated.
fn selector_set_note(doc: &mut toml_edit::DocumentMut, selector: &str, text: &str) -> bool {
    let Some(array) = doc
        .as_table_mut()
        .get_mut("selector")
        .and_then(toml_edit::Item::as_array_of_tables_mut)
    else {
        return false;
    };
    for row in array.iter_mut() {
        if row
            .get("selector")
            .and_then(toml_edit::Item::as_str)
            .is_some_and(|s| s == selector)
        {
            row.insert("note", toml_edit::value(text));
            return true;
        }
    }
    false
}

/// Remove every selector row whose selector string matches one of `globs`.
/// Returns the count of rows removed.
fn selector_remove_many(doc: &mut toml_edit::DocumentMut, globs: &[String]) -> usize {
    let Some(array) = doc
        .as_table_mut()
        .get_mut("selector")
        .and_then(toml_edit::Item::as_array_of_tables_mut)
    else {
        return 0;
    };
    let before = array.len();
    array.retain(|row| {
        let s = row
            .get("selector")
            .and_then(toml_edit::Item::as_str)
            .unwrap_or("");
        !globs.iter().any(|g| g == s)
    });
    before - array.len()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::is_transition_terminal;
    use crate::meta::Meta;

    fn meta(id: u32, status: &str, slug: &str, title: &str) -> Meta {
        Meta {
            id,
            slug: slug.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            tags: Vec::new(),
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

    // --- SL-037 column model (the slice grid: prefixed ids + variant axis) ---

    /// Render rows over the default column set (the migrated `render_table` path).
    fn render_default(rows: &[SliceRowTuple]) -> String {
        let sel = listing::select_columns(&SLICE_COLUMNS, SLICE_DEFAULT, None).unwrap();
        listing::render_columns(rows, &sel, listing::RenderOpts::default())
    }

    /// Render rows over an explicit `--columns` set.
    fn render_cols(rows: &[SliceRowTuple], cols: &[&str]) -> String {
        let owned: Vec<String> = cols.iter().map(|s| (*s).to_string()).collect();
        let sel = listing::select_columns(&SLICE_COLUMNS, SLICE_DEFAULT, Some(&owned)).unwrap();
        listing::render_columns(rows, &sel, listing::RenderOpts::default())
    }

    #[test]
    fn slice_list_empty_suppresses_the_header() {
        assert_eq!(render_default(&[]), "");
    }

    #[test]
    fn slice_list_default_renders_prefixed_ids_rollup_and_divergence() {
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
        let out = render_default(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("id"), "header: {:?}", lines[0]);
        assert!(lines[0].contains("phases"), "phases column: {:?}", lines[0]);
        // SL-025: prefixed ids, not bare `001`.
        // consistent terminal slice: no ⚠, full rollup
        assert!(lines[1].starts_with("SL-001 │ done"), "{:?}", lines[1]);
        assert!(lines[1].contains("6/6"));
        // done but 2/6 → divergent ⚠ (marker preserved in the status cell value)
        assert!(lines[2].starts_with("SL-007 │ done ⚠"), "{:?}", lines[2]);
        assert!(lines[2].contains("2/6"));
        // untracked → —
        assert!(lines[3].starts_with("SL-009 │ proposed"), "{:?}", lines[3]);
        assert!(lines[3].contains("—"));
        // no bare numeric id anywhere
        assert!(!out.contains("\n001  "), "no bare numeric id: {out}");
    }

    #[test]
    fn slice_list_default_omits_slug() {
        let rows = vec![(meta(1, "proposed", "entity-v1", "Entity v1"), None)];
        let out = render_default(&rows);
        let header = out.lines().next().unwrap();
        // SL-037 D4: default visible set is [id, status, phases, title] — slug hidden.
        assert!(
            !header.contains("slug"),
            "default header omits slug: {header:?}"
        );
        assert!(
            !out.contains("entity-v1"),
            "slug value hidden by default: {out}"
        );
        assert!(header.contains("title"), "default keeps title: {header:?}");
    }

    #[test]
    fn slice_list_columns_reveals_slug_and_preserves_markers() {
        let rows = vec![(
            meta(7, "done", "anchoring", "Anchoring"),
            Some(rollup(2, 4)),
        )];
        // Reveal slug; status cell still carries the ⚠ divergence marker, phases intact.
        let out = render_cols(&rows, &["id", "status", "phases", "slug"]);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0].split_whitespace().collect::<Vec<_>>(),
            vec!["id", "│", "status", "│", "phases", "│", "slug"]
        );
        assert!(
            out.contains("anchoring"),
            "slug revealed by --columns: {out}"
        );
        assert!(
            lines[1].contains("done ⚠"),
            "⚠ marker preserved: {:?}",
            lines[1]
        );
        assert!(
            lines[1].contains("2/6"),
            "phases cell intact: {:?}",
            lines[1]
        );
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
            &[],
            &mut entity::local_reserved(),
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

        let metas = meta::read_metas(&root.join(SLICE_DIR), "slice", "SL").unwrap();
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
            &[],
            &mut entity::local_reserved(),
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
            &[],
            &mut entity::local_reserved(),
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
            &[],
            &mut entity::local_reserved(),
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
            &[],
            &mut entity::local_reserved(),
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
            &[],
            &mut entity::local_reserved(),
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
            &[],
            &mut entity::local_reserved(),
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
        assert!(out.contains("SL-001 │ proposed"), "prefixed id: {out}");
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
            &[
                "proposed",
                "design",
                "plan",
                "ready",
                "started",
                "audit",
                "reconcile",
                "done",
                "abandoned"
            ]
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

        let (doc, toml_text, body) = read_slice(&slice_root(root), 1).unwrap();
        assert_eq!(doc.id, 1);
        assert_eq!(doc.slug, "my-slug");
        assert_eq!(doc.status, "proposed");
        // SL-048: tier-1 relations come from the `[[relation]]` block; a virgin slice
        // has none, so `read_block` yields no edges.
        let edges = crate::relation::tier1_edges(&SLICE_KIND, &toml_text).unwrap();
        assert!(edges.is_empty());
        // the md scope body is read verbatim.
        assert!(body.contains("My Title"));
    }

    #[test]
    fn format_show_renders_identity_and_scope_body() {
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        // SL-048: tier-1 edges are passed in (read from `[[relation]]`), not a struct.
        let tier1 = vec![RelationEdge::with_role(
            RelationLabel::References,
            Some(Role::Implements),
            "PRD-010".into(),
        )];
        // `started` defaults to self/auto (no plan/reconcile gate) — VT-3 show side.
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let out = format_show(
            &doc,
            &tier1,
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n\nthe scope body.\n",
            posture,
            &crate::facet::EntityFacets::default(),
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        assert!(out.contains("SL-025 — Uniform CLI"), "identity: {out}");
        assert!(out.contains("uniform-cli · started"), "flat fields: {out}");
        assert!(out.contains("conduct: self/auto"), "conduct posture: {out}");
        assert!(out.contains("created 2026-06-01 · updated 2026-06-08"));
        assert!(
            out.contains("references(implements): PRD-010"),
            "relationships axis: {out}"
        );
        assert!(
            out.contains("the scope body."),
            "scope body appended: {out}"
        );
    }

    // VT-5: absent facets → byte-identical to pre-change format_show output (golden).
    #[test]
    fn vt5_format_show_absent_facets_is_byte_identical() {
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let tier1 = vec![RelationEdge::with_role(
            RelationLabel::References,
            Some(Role::Implements),
            "PRD-010".into(),
        )];
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let out = format_show(
            &doc,
            &tier1,
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n\nthe scope body.\n",
            posture,
            &crate::facet::EntityFacets::default(),
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        let expected = concat!(
            "SL-025 — Uniform CLI\n",
            "uniform-cli · started\n",
            "conduct: self/auto\n",
            "created 2026-06-01 · updated 2026-06-08\n",
            "\n",
            "relationships:\n",
            "  references(implements): PRD-010\n",
            "\n",
            "# Scope\n",
            "\n",
            "the scope body.\n",
        );
        assert_eq!(
            out, expected,
            "absent facets must be byte-identical to pre-change format_show output"
        );
    }

    // VT-1: estimate present → confidence row rendered in show output.
    #[test]
    fn vt1_format_show_estimate_present_renders_confidence_row() {
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let tier1 = vec![RelationEdge::with_role(
            RelationLabel::References,
            Some(Role::Implements),
            "PRD-010".into(),
        )];
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let facets = crate::facet::EntityFacets {
            estimate: Some(crate::estimate::EstimateFacet {
                lower: 3.0,
                upper: 5.0,
            }),
            value: None,
            risk: None,
            tags: vec![],
        };
        let out = format_show(
            &doc,
            &tier1,
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n",
            posture,
            &facets,
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        assert!(
            out.contains("estimate: 3.2–4.8 espresso_shots (80% confidence)"),
            "VT-1 estimate row: {out}"
        );
    }

    // VT-2: estimate absent → no estimate: line in output.
    #[test]
    fn vt2_format_show_estimate_absent_no_row() {
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let out = format_show(
            &doc,
            &[],
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n",
            posture,
            &crate::facet::EntityFacets::default(),
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        assert!(
            !out.contains("estimate:"),
            "VT-2: estimate row must not appear when absent: {out}"
        );
    }

    // VT-3: value present → 'value: {magnitude} {unit}' appears.
    #[test]
    fn vt3_format_show_value_present_renders_row() {
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let facets = crate::facet::EntityFacets {
            estimate: None,
            value: Some(crate::value::ValueFacet { value: 5.0 }),
            risk: None,
            tags: vec![],
        };
        let out = format_show(
            &doc,
            &[],
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n",
            posture,
            &facets,
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        assert!(
            out.contains("value: 5.0 magic_beans"),
            "VT-3 value row: {out}"
        );
    }

    // VT-4: value absent → no value: line.
    #[test]
    fn vt4_format_show_value_absent_no_row() {
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let out = format_show(
            &doc,
            &[],
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n",
            posture,
            &crate::facet::EntityFacets::default(),
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        assert!(
            !out.contains("value:"),
            "VT-4: value row must not appear when absent: {out}"
        );
    }

    // VT-9: custom confidence bounds in doctrine.toml → correct percentile band.
    #[test]
    fn vt9_format_show_custom_confidence_bounds() {
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let facets = crate::facet::EntityFacets {
            estimate: Some(crate::estimate::EstimateFacet {
                lower: 3.0,
                upper: 5.0,
            }),
            value: None,
            risk: None,
            tags: vec![],
        };
        // 25%–75% band → 50% confidence
        let out = format_show(
            &doc,
            &[],
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n",
            posture,
            &facets,
            "espresso_shots",
            "magic_beans",
            0.25,
            0.75,
        );
        assert!(
            out.contains("estimate: 3.5–4.5 espresso_shots (50% confidence)"),
            "VT-9 custom bounds: {out}"
        );
    }

    // VT-10: zero-width estimate (lower==upper) → single-value display.
    #[test]
    fn vt10_format_show_zero_width_estimate() {
        let doc = SliceDoc {
            id: 25,
            slug: "uniform-cli".into(),
            title: "Uniform CLI".into(),
            status: "started".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-08".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let facets = crate::facet::EntityFacets {
            estimate: Some(crate::estimate::EstimateFacet {
                lower: 5.0,
                upper: 5.0,
            }),
            value: None,
            risk: None,
            tags: vec![],
        };
        let out = format_show(
            &doc,
            &[],
            &crate::dep_seq::DepSeq::default(),
            "# Scope\n",
            posture,
            &facets,
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        assert!(
            out.contains("estimate: 5.0–5.0 espresso_shots (80% confidence)"),
            "VT-10 zero-width: {out}"
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

        let (doc, toml_text, body) = read_slice(&sr, 1).unwrap();
        let tier1 = crate::relation::tier1_edges(&SLICE_KIND, &toml_text).unwrap();
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let ds = crate::dep_seq::DepSeq::default();
        let table = format_show(
            &doc,
            &tier1,
            &ds,
            &body,
            posture,
            &crate::facet::EntityFacets::default(),
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        let json = show_json(&doc, &tier1, &ds, &body).unwrap();
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
        let (doc, toml_text, body) = read_slice(&slice_root(root), 1).unwrap();
        let tier1 = crate::relation::tier1_edges(&SLICE_KIND, &toml_text).unwrap();

        let out = show_json(&doc, &tier1, &crate::dep_seq::DepSeq::default(), &body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "slice");
        assert_eq!(parsed["slice"]["id"], 1);
        assert_eq!(parsed["slice"]["slug"], "my-slug");
        assert_eq!(parsed["slice"]["status"], "proposed");
        // SL-048: the `relationships` object is reconstructed from `[[relation]]`.
        // SL-149 PHASE-05: the legacy specs/requirements axes are gone; `supersedes` and
        // the `references` role object are present (empty here — a virgin slice).
        let rel = &parsed["slice"]["relationships"];
        assert!(rel.get("specs").is_none());
        assert!(rel.get("requirements").is_none());
        assert!(rel["supersedes"].is_array());
        assert!(rel["references"]["implements"].is_array());
        assert!(parsed["body"].as_str().unwrap().contains("My Title"));
    }

    /// A minimal `SliceDoc` for the show_json schema tests — identity only, no facets.
    fn doc_for_json(id: u32) -> SliceDoc {
        SliceDoc {
            id,
            slug: "refs".into(),
            title: "Refs".into(),
            status: "proposed".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-01".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        }
    }

    #[test]
    fn show_json_groups_references_by_role() {
        // SL-149 PHASE-05: a fixture authoring references(implements/scoped_from/concerns)
        // → the JSON carries a `references` object grouped by role; the legacy
        // `specs`/`requirements` keys are gone (the corpus migration retired them).
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let tier1 = vec![
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Implements),
                "SPEC-018".into(),
            ),
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::ScopedFrom),
                "IMP-012".into(),
            ),
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Concerns),
                "RFC-003".into(),
            ),
        ];
        let json = show_json(
            &doc_for_json(149),
            &tier1,
            &crate::dep_seq::DepSeq::default(),
            "# b\n",
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rel = &v["slice"]["relationships"];
        // references grouped by role
        assert_eq!(
            rel["references"]["implements"],
            serde_json::json!(["SPEC-018"])
        );
        assert_eq!(
            rel["references"]["scoped_from"],
            serde_json::json!(["IMP-012"])
        );
        assert_eq!(
            rel["references"]["concerns"],
            serde_json::json!(["RFC-003"])
        );
        // legacy keys removed (the hard cut)
        assert!(rel.get("specs").is_none(), "legacy specs key removed");
        assert!(
            rel.get("requirements").is_none(),
            "legacy requirements key removed"
        );
    }

    #[test]
    fn show_json_references_implements_carries_spec_and_req() {
        // SL-149 PHASE-05: a slice authoring references(implements) to both a SPEC and a
        // REQ (the migration target of the old specs/requirements edges) → both land in
        // the `implements` bucket; no legacy keys.
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let tier1 = vec![
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Implements),
                "SPEC-018".into(),
            ),
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Implements),
                "REQ-002".into(),
            ),
        ];
        let json = show_json(
            &doc_for_json(149),
            &tier1,
            &crate::dep_seq::DepSeq::default(),
            "# b\n",
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rel = &v["slice"]["relationships"];
        assert_eq!(
            rel["references"]["implements"],
            serde_json::json!(["SPEC-018", "REQ-002"])
        );
        assert!(rel.get("specs").is_none());
        assert!(rel.get("requirements").is_none());
    }

    #[test]
    fn show_surfaces_dep_seq_axes_in_table_and_json() {
        // SL-060: `needs`/`after` render under the same `relationships:` block (Table)
        // and the same `relationships` object (JSON), after the structural tier-1 axes.
        use crate::dep_seq::{AfterEdge, DepSeq};
        let doc = SliceDoc {
            id: 60,
            slug: "dep-seq".into(),
            title: "Dep Seq".into(),
            status: "proposed".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-01".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let tier1 = Vec::new();
        let ds = DepSeq {
            needs: vec!["SL-047".into()],
            after: vec![
                AfterEdge {
                    to: "SL-002".into(),
                    rank: 0,
                },
                AfterEdge {
                    to: "SL-003".into(),
                    rank: 5,
                },
            ],
        };
        let posture =
            crate::conduct::resolve(&crate::conduct::ConductConfig::default(), &doc.status);
        let table = format_show(
            &doc,
            &tier1,
            &ds,
            "# body\n",
            posture,
            &crate::facet::EntityFacets::default(),
            "espresso_shots",
            "magic_beans",
            0.1,
            0.9,
        );
        assert!(table.contains("relationships:"), "block renders: {table}");
        assert!(table.contains("needs: SL-047"), "needs axis: {table}");
        // rank 0 omits the suffix; a non-zero rank renders `(rank N)`.
        assert!(
            table.contains("after: SL-002, SL-003 (rank 5)"),
            "after axis with rank suffix: {table}"
        );

        let json = show_json(&doc, &tier1, &ds, "# body\n").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            v["slice"]["relationships"]["needs"],
            serde_json::json!(["SL-047"])
        );
        assert_eq!(
            v["slice"]["relationships"]["after"],
            serde_json::json!([
                { "to": "SL-002", "rank": 0 },
                { "to": "SL-003", "rank": 5 },
            ])
        );
    }

    #[test]
    fn show_omits_dep_seq_keys_when_unauthored() {
        // Additive: an unauthored slice's relationships object carries NO `needs`/
        // `after` keys (byte-stable with the pre-SL-060 shape).
        let doc = SliceDoc {
            id: 1,
            slug: "v".into(),
            title: "V".into(),
            status: "proposed".into(),
            created: "2026-06-01".into(),
            updated: "2026-06-01".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };
        let json = show_json(&doc, &[], &crate::dep_seq::DepSeq::default(), "# body\n").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rel = &v["slice"]["relationships"];
        assert!(rel.get("needs").is_none(), "no needs key when unauthored");
        assert!(rel.get("after").is_none(), "no after key when unauthored");
    }

    #[test]
    fn slice_doc_round_trips_estimate_facet() {
        let doc = SliceDoc {
            id: 17,
            slug: "estimate-facet".into(),
            title: "Estimate Facet".into(),
            status: "proposed".into(),
            created: "2026-06-18".into(),
            updated: "2026-06-18".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: Some(crate::estimate::EstimateFacet {
                lower: 2.0,
                upper: 8.0,
            }),
            value: None,
            selectors: vec![],
        };

        let toml = toml::to_string_pretty(&doc).unwrap();
        let parsed: SliceDoc = toml::from_str(&toml).unwrap();
        let estimate = parsed.estimate.expect("estimate facet present");
        assert_eq!(estimate.lower, 2.0);
        assert_eq!(estimate.upper, 8.0);
        assert_eq!(parsed.value, None);
    }

    #[test]
    fn slice_doc_round_trips_value_facet() {
        let doc = SliceDoc {
            id: 17,
            slug: "value-facet".into(),
            title: "Value Facet".into(),
            status: "proposed".into(),
            created: "2026-06-18".into(),
            updated: "2026-06-18".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: Some(crate::value::ValueFacet { value: 5.0 }),
            selectors: vec![],
        };

        let toml = toml::to_string_pretty(&doc).unwrap();
        let parsed: SliceDoc = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.estimate, None);
        assert_eq!(
            parsed.value.expect("value facet present"),
            crate::value::ValueFacet { value: 5.0 }
        );
    }

    #[test]
    fn slice_doc_serde_omits_absent_facets() {
        let doc = SliceDoc {
            id: 18,
            slug: "no-facets".into(),
            title: "No Facets".into(),
            status: "proposed".into(),
            created: "2026-06-18".into(),
            updated: "2026-06-18".into(),
            tags: vec![],
            gate: Gate::default(),
            estimate: None,
            value: None,
            selectors: vec![],
        };

        let toml = toml::to_string_pretty(&doc).unwrap();
        assert!(
            !toml.contains("[estimate]"),
            "unexpected estimate table: {toml}"
        );
        assert!(!toml.contains("[value]"), "unexpected value table: {toml}");

        let parsed: SliceDoc = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.estimate, None);
        assert_eq!(parsed.value, None);
    }

    #[test]
    fn slice_doc_malformed_facet_errors_at_parse() {
        // Validation is live via the SliceDoc serde path (VT-3): a non-finite
        // value facet on a slice toml errors at parse, not silently carried.
        let text = "id = 1\nslug = \"x\"\ntitle = \"X\"\nstatus = \"proposed\"\n\
                    created = \"2026-06-18\"\nupdated = \"2026-06-18\"\n[value]\nvalue = nan\n";
        let err = toml::from_str::<SliceDoc>(text).unwrap_err().to_string();
        assert!(err.contains("must be finite"), "got: {err}");
    }

    // --- SL-028 PHASE-02: conduct shell seam (T5/T6) ---

    #[test]
    fn load_conduct_absent_file_is_baked_defaults() {
        // No doctrine.toml at root → default config → resolve gives the baked
        // posture (T5: absent file shows the default; VT-1 absent-file fallback).
        let dir = tempfile::tempdir().unwrap();
        let cfg = load_conduct(dir.path()).unwrap();
        assert_eq!(cfg, crate::conduct::ConductConfig::default());
        // plan-source exit posture is the baked gate even with no file.
        assert_eq!(crate::conduct::resolve(&cfg, "plan").label(), "self/gate");
        // a non-gate source is self/auto.
        assert_eq!(
            crate::conduct::resolve(&cfg, "started").label(),
            "self/auto"
        );
    }

    #[test]
    fn load_conduct_reflects_a_root_override() {
        // A root doctrine.toml [conduct] override is read by the shell and folds
        // into resolve (T5: an override is reflected in the posture).
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(crate::dtoml::DOCTRINE_TOML),
            "[conduct]\ndefault-actor = \"agent\"\n[conduct.ready]\nautonomy = \"gate\"\n",
        )
        .unwrap();
        let cfg = load_conduct(dir.path()).unwrap();
        // ready now gates, actor inherits the project default-actor (agent).
        assert_eq!(crate::conduct::resolve(&cfg, "ready").label(), "agent/gate");
    }

    #[test]
    fn load_conduct_refuses_malformed_doctrine_toml() {
        // A genuinely malformed file surfaces an error (not silent) — but the
        // FSM gates are untouched (this is the conduct read, advisory only).
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(crate::dtoml::DOCTRINE_TOML),
            "[conduct\nbroken =",
        )
        .unwrap();
        assert!(load_conduct(dir.path()).is_err());
    }

    #[test]
    fn run_show_on_a_missing_slice_errors() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_show(Some(dir.path().to_path_buf()), "SL-009", Format::Table).unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
    }

    // VT-11: shell integration — run_show against a fixture slice with [estimate] +
    // doctrine.toml with [estimation] → confidence row in output.
    #[test]
    fn vt11_run_show_fixture_with_estimate_renders_confidence_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // doctrine.toml with custom estimation config
        fs::write(
            root.join("doctrine.toml"),
            "[estimation]\nunit = \"story_points\"\nlower_confidence = 0.1\nupper_confidence = 0.9\n",
        )
        .unwrap();

        // slice TOML with estimate
        let sr = slice_root(root);
        fs::create_dir_all(sr.join("001")).unwrap();
        fs::write(
            sr.join("001/slice-001.toml"),
            "id = 1\nslug = \"test\"\ntitle = \"Test\"\nstatus = \"proposed\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\n[estimate]\nlower = 3\nupper = 8\n",
        )
        .unwrap();
        fs::write(sr.join("001/slice-001.md"), "# Test slice\n").unwrap();

        let (doc, toml_text, body) = read_slice(&sr, 1).unwrap();
        let cfg = crate::dtoml::load_doctrine_toml(root).unwrap();
        let posture = crate::conduct::resolve(&cfg.conduct, &doc.status);
        let estimation_unit = crate::estimate::resolve_unit(&cfg.estimation);
        let value_unit = crate::value::resolve_unit(&cfg.value);
        let (lower_pct, upper_pct) = crate::estimate::resolve_confidence(&cfg.estimation).unwrap();
        let facets = crate::facet::EntityFacets {
            estimate: doc.estimate.clone(),
            value: doc.value.clone(),
            risk: None,
            tags: doc.tags.clone(),
        };
        let tier1 = crate::relation::tier1_edges(&SLICE_KIND, &toml_text).unwrap();
        let dep_seq = crate::dep_seq::DepSeq::default();
        let out = format_show(
            &doc,
            &tier1,
            &dep_seq,
            &body,
            posture,
            &facets,
            &estimation_unit,
            &value_unit,
            lower_pct,
            upper_pct,
        );
        assert!(
            out.contains("estimate: 3.5–7.5 story_points (80% confidence)"),
            "VT-11: {out}"
        );
    }

    // VT-12: malformed doctrine.toml → run_show returns Err (not silently defaulted).
    #[test]
    fn vt12_run_show_malformed_doctrine_toml_propagates_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Malformed TOML — missing `=`
        fs::write(root.join("doctrine.toml"), "[estimation\nunit = broken").unwrap();

        // A valid slice must exist so it's the doctrine.toml that fails
        let sr = slice_root(root);
        fs::create_dir_all(sr.join("001")).unwrap();
        fs::write(
            sr.join("001/slice-001.toml"),
            "id = 1\nslug = \"test\"\ntitle = \"Test\"\nstatus = \"proposed\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
        )
        .unwrap();
        fs::write(sr.join("001/slice-001.md"), "# Test\n").unwrap();

        let err = run_show(Some(root.to_path_buf()), "SL-001", Format::Table).unwrap_err();
        assert!(
            err.to_string().contains("Failed to parse"),
            "VT-12: error must propagate, got: {err}"
        );
    }

    // --- SL-028 PHASE-01: lifecycle FSM ---

    // VT-1: classify table (design §5.4/§9). Edge-table driven; covers advance,
    // each back-edge, skip, abandon, noop, from-terminal, seam-breach (incl. from
    // a drifted source), and the legit seam path audit→reconcile→done = Advance.

    #[test]
    fn classify_forward_chain_is_advance() {
        for (from, to) in [
            ("proposed", "design"),
            ("design", "plan"),
            ("plan", "ready"),
            ("ready", "started"),
            ("started", "audit"),
        ] {
            assert_eq!(classify(from, to), Transition::Advance, "{from} → {to}");
        }
    }

    #[test]
    fn classify_legit_closure_seam_path_is_advance() {
        // audit → reconcile → done — the ADR-003 §7/§8 spine.
        assert_eq!(classify("audit", "reconcile"), Transition::Advance);
        assert_eq!(classify("reconcile", "done"), Transition::Advance);
    }

    #[test]
    fn classify_named_back_edges() {
        for (from, to) in [
            ("audit", "started"),
            ("audit", "design"),
            ("reconcile", "audit"),
            ("reconcile", "design"),
        ] {
            assert_eq!(classify(from, to), Transition::BackEdge, "{from} → {to}");
        }
    }

    #[test]
    fn classify_abandon_from_each_non_terminal() {
        for from in [
            "proposed",
            "design",
            "plan",
            "ready",
            "started",
            "audit",
            "reconcile",
        ] {
            assert_eq!(
                classify(from, "abandoned"),
                Transition::Abandon,
                "{from} → abandoned"
            );
        }
    }

    #[test]
    fn classify_noop_when_unchanged() {
        assert_eq!(classify("started", "started"), Transition::Noop);
        // No-op precedes from-terminal: done → done is a no-op, not a refusal.
        assert_eq!(classify("done", "done"), Transition::Noop);
    }

    #[test]
    fn classify_from_terminal_refused() {
        for from in ["done", "abandoned"] {
            assert_eq!(
                classify(from, "design"),
                Transition::FromTerminal,
                "{from} → design"
            );
        }
    }

    #[test]
    fn classify_seam_breach_to_reconcile_from_non_audit() {
        for from in ["proposed", "design", "plan", "ready", "started"] {
            assert_eq!(
                classify(from, "reconcile"),
                Transition::SeamBreach,
                "{from} → reconcile"
            );
        }
    }

    #[test]
    fn classify_seam_breach_to_done_from_non_reconcile() {
        for from in ["proposed", "design", "plan", "ready", "started", "audit"] {
            assert_eq!(
                classify(from, "done"),
                Transition::SeamBreach,
                "{from} → done"
            );
        }
    }

    #[test]
    fn classify_seam_binds_even_from_a_drifted_source() {
        // The seam is about the target edge, not the source's validity (§5.5).
        assert_eq!(classify("bogus", "reconcile"), Transition::SeamBreach);
        assert_eq!(classify("bogus", "done"), Transition::SeamBreach);
    }

    #[test]
    fn classify_move_out_of_drift_is_skip_not_refused() {
        // Out-of-vocab `from`, non-seam, non-terminal target → Skip (allowed).
        assert_eq!(classify("bogus", "started"), Transition::Skip);
    }

    #[test]
    fn classify_non_chain_move_is_skip() {
        // A legal-vocab pair the FSM never names (and not a seam target) → Skip.
        assert_eq!(classify("proposed", "started"), Transition::Skip);
        assert_eq!(classify("design", "started"), Transition::Skip);
    }

    // VT-1: the third predicate, distinct from the other two (F13).

    #[test]
    fn is_transition_terminal_is_a_distinct_third_predicate() {
        assert!(is_transition_terminal("done"));
        assert!(is_transition_terminal("abandoned"));
        assert!(!is_transition_terminal("started"));
        // Diverges from is_terminal_status ({done}) on `abandoned`...
        assert!(is_transition_terminal("abandoned") && !is_terminal_status("abandoned"));
        // ...and from is_hidden (presentation) which agrees on the set but is a
        // semantically unrelated predicate — they must not be conflated.
        assert_eq!(is_transition_terminal("done"), is_hidden("done"));
    }

    // T5: the ValueEnum ↔ const lockstep canary (cf. adr_known_set_matches_variants).

    #[test]
    fn slice_status_enum_matches_the_vocabulary() {
        let variants = [
            SliceStatus::Proposed,
            SliceStatus::Design,
            SliceStatus::Plan,
            SliceStatus::Ready,
            SliceStatus::Started,
            SliceStatus::Audit,
            SliceStatus::Reconcile,
            SliceStatus::Done,
            SliceStatus::Abandoned,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|v| v.as_str()).collect();
        assert_eq!(from_variants, SLICE_STATUSES.to_vec());
    }

    // VT-2: set_slice_status — round-trip, no-op, malformed/refusal guards.

    /// Read the raw on-disk slice toml text.
    fn slice_text(root: &Path, id: u32) -> String {
        let name = format!("{id:03}");
        fs::read_to_string(
            slice_root(root)
                .join(&name)
                .join(format!("slice-{name}.toml")),
        )
        .unwrap()
    }

    #[test]
    fn set_slice_status_advances_and_preserves_comments_and_relationships() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        let before = slice_text(root, 1);
        assert!(
            before.contains("[relationships]"),
            "fixture has relationships"
        );
        let comment = before
            .lines()
            .find(|l| l.trim_start().starts_with('#'))
            .is_some();

        // proposed → design (advance).
        set_slice_status(
            &slice_root(root),
            1,
            "proposed",
            SliceStatus::Design,
            "2099-01-01",
        )
        .unwrap();
        let after = slice_text(root, 1);
        assert!(
            after.contains("status = \"design\""),
            "status written: {after}"
        );
        assert!(
            after.contains("updated = \"2099-01-01\""),
            "date stamped: {after}"
        );
        assert!(
            after.contains("[relationships]"),
            "relationships survive: {after}"
        );
        if comment {
            assert!(after.contains('#'), "comments survive: {after}");
        }
    }

    #[test]
    fn set_slice_status_noop_holds_content_and_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        let p = slice_root(root).join("001").join("slice-001.toml");
        let before = fs::read_to_string(&p).unwrap();
        let mtime_before = fs::metadata(&p).unwrap().modified().unwrap();

        // proposed → proposed: no-op, nothing written.
        set_slice_status(
            &slice_root(root),
            1,
            "proposed",
            SliceStatus::Proposed,
            "2099-01-01",
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), before, "content held");
        assert_eq!(
            fs::metadata(&p).unwrap().modified().unwrap(),
            mtime_before,
            "mtime held"
        );
    }

    #[test]
    fn set_slice_status_refuses_from_terminal() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        for from in [SliceStatus::Done, SliceStatus::Abandoned] {
            let err = set_slice_status(
                &slice_root(root),
                1,
                from.as_str(),
                SliceStatus::Design,
                "x",
            )
            .unwrap_err()
            .to_string();
            assert!(err.contains("terminal"), "{}: {err}", from.as_str());
        }
        // Disk untouched (still proposed).
        assert!(slice_text(root, 1).contains("status = \"proposed\""));
    }

    #[test]
    fn set_slice_status_refuses_seam_breach() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        // started → done (skip-to-done) is a seam breach.
        let err = set_slice_status(&slice_root(root), 1, "started", SliceStatus::Done, "x")
            .unwrap_err()
            .to_string();
        assert!(err.contains("closure seam"), "skip-to-done refused: {err}");
        // design → reconcile (non-audit source) is a seam breach.
        let err2 = set_slice_status(&slice_root(root), 1, "design", SliceStatus::Reconcile, "x")
            .unwrap_err()
            .to_string();
        assert!(
            err2.contains("closure seam"),
            "non-audit → reconcile refused: {err2}"
        );
        assert!(
            slice_text(root, 1).contains("status = \"proposed\""),
            "disk untouched"
        );
    }

    #[test]
    fn set_slice_status_seam_breach_from_a_drifted_source() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        set_status_raw(root, 1, "bogus");
        // → done from a drifted source still breaches the seam (target edge).
        let err = set_slice_status(&slice_root(root), 1, "bogus", SliceStatus::Done, "x")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("closure seam"),
            "drifted → done refused: {err}"
        );
    }

    #[test]
    fn set_slice_status_refuses_malformed_toml() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // A slice toml missing the `updated` scaffold key (hand-edited corruption).
        let d = slice_root(root).join("001");
        fs::create_dir_all(&d).unwrap();
        fs::write(
            d.join("slice-001.toml"),
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"started\"\n",
        )
        .unwrap();
        let err = set_slice_status(&slice_root(root), 1, "started", SliceStatus::Audit, "x")
            .unwrap_err()
            .to_string();
        assert!(err.contains("malformed"), "missing key refused: {err}");
    }

    #[test]
    fn run_status_prints_classification_with_note() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        set_status_raw(root, 1, "started");
        // started → audit (advance); run_status is the thin shell, asserts no error
        // and the write landed (output goes to stdout — the writer is the unit).
        run_status(
            Some(root.to_path_buf()),
            1,
            SliceStatus::Audit,
            Some("done impl"),
        )
        .unwrap();
        assert!(
            slice_text(root, 1).contains("status = \"audit\""),
            "write landed"
        );
    }

    #[test]
    fn status_line_carries_the_source_exit_posture() {
        // VT-3 (status side): the design's example line — reconcile gates by
        // default, so its exit posture is self/gate (F19, resolve(from)).
        let cfg = crate::conduct::ConductConfig::default();
        let line = status_line(
            "reconcile",
            "done",
            classify("reconcile", "done"),
            crate::conduct::resolve(&cfg, "reconcile"),
            None,
        );
        assert_eq!(line, "reconcile → done [advance] [self/gate]");
    }

    #[test]
    fn status_line_appends_the_note_after_the_posture() {
        let cfg = crate::conduct::ConductConfig::default();
        let line = status_line(
            "started",
            "audit",
            classify("started", "audit"),
            crate::conduct::resolve(&cfg, "started"),
            Some("done impl"),
        );
        assert_eq!(line, "started → audit [advance] [self/auto] — done impl");
    }

    #[test]
    fn read_status_surfaces_the_current_authored_status() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        set_status_raw(root, 1, "reconcile");
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    // --- PHASE-04: reverse close-gate (design §7, D8/D-C9b) ---

    /// VT-4: `crosses_closure_seam` is true for EXACTLY the two terminal advances
    /// and false for every other edge — the gate's firing predicate.
    #[test]
    fn vt4_crosses_closure_seam_is_only_the_two_terminal_advances() {
        assert!(crosses_closure_seam("audit", "reconcile"));
        assert!(crosses_closure_seam("reconcile", "done"));
        // Non-seam transitions — never gated.
        for (from, to) in [
            ("started", "audit"),
            ("ready", "started"),
            ("plan", "ready"),
            ("audit", "started"),   // a back-edge
            ("reconcile", "audit"), // a back-edge
            ("started", "abandoned"),
            ("audit", "audit"), // no-op
        ] {
            assert!(
                !crosses_closure_seam(from, to),
                "{from} → {to} must NOT be a closure-seam crossing"
            );
        }
    }

    /// Raise one `blocker` finding on a fresh RV targeting `SL-<target_id>`. Returns
    /// the project root unchanged. Drives the real verb path (raise under the turn
    /// guard) so the ledger is authentic.
    fn raise_blocker_rv(root: &Path, target_id: u32) {
        let target = canonical_id(target_id);
        crate::review::run_new(
            Some(root.to_path_buf()),
            &crate::review::NewArgs {
                facet: crate::review::Facet::Reconciliation,
                target: target.clone(),
                phase: None,
                title: None,
                raiser: None,
                responder: None,
            },
        )
        .unwrap();
        crate::review::run_raise(
            Some(root.to_path_buf()),
            &crate::review::RaiseArgs {
                reference: "RV-001".to_owned(),
                severity: crate::review::Severity::Blocker,
                title: "must fix".to_owned(),
                detail: "d".to_owned(),
            },
            crate::review::Role::Raiser,
        )
        .unwrap();
    }

    /// VT-2 (refuse half): crossing the closure seam `audit → reconcile` is REFUSED
    /// while an Active RV targeting the slice holds an unresolved blocker, the
    /// refusal naming `RV-NNN/F-n`; the authored status is left untouched.
    #[test]
    fn vt2_close_seam_refused_on_an_unresolved_blocker() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        set_status_raw(root, 1, "audit");
        raise_blocker_rv(root, 1);

        let err = run_status(Some(root.to_path_buf()), 1, SliceStatus::Reconcile, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("RV-001/F-1"), "names the blocker: {err}");
        assert!(err.contains("refused"), "refusal wording: {err}");
        // The transition was refused BEFORE the write — status unchanged.
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "audit");
    }

    /// VT-2 (pass half): the SAME seam crossing PASSES once the blocker is verified
    /// (terminal ⇒ the RV is Done ⇒ no unresolved blocker remains).
    #[test]
    fn vt2_close_seam_passes_after_the_blocker_is_verified() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        set_status_raw(root, 1, "audit");
        raise_blocker_rv(root, 1);

        // Resolve the blocker: dispose (answered) then verify (terminal).
        crate::review::run_dispose(
            Some(root.to_path_buf()),
            &crate::review::DisposeArgs {
                reference: "RV-001".to_owned(),
                finding: "F-1".to_owned(),
                disposition: "fixed".to_owned(),
                response: "done".to_owned(),
            },
            crate::review::Role::Responder,
        )
        .unwrap();
        crate::review::run_verify(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            None,
            crate::review::Role::Raiser,
        )
        .unwrap();

        run_status(Some(root.to_path_buf()), 1, SliceStatus::Reconcile, None).unwrap();
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    /// VT-2 (withdraw variant): withdrawing the blocker also unblocks the seam.
    #[test]
    fn vt2_close_seam_passes_after_the_blocker_is_withdrawn() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        set_status_raw(root, 1, "audit");
        raise_blocker_rv(root, 1);

        crate::review::run_withdraw(
            Some(root.to_path_buf()),
            "RV-001",
            "F-1",
            crate::review::Role::Raiser,
        )
        .unwrap();
        run_status(Some(root.to_path_buf()), 1, SliceStatus::Reconcile, None).unwrap();
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    /// VT-4: the gate fires ONLY on the closure seam — a NON-seam slice transition
    /// (`started → audit`) is NOT gated even with an unresolved blocker present.
    #[test]
    fn vt4_non_seam_transition_is_not_gated() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "s", "S", "2026-06-04");
        set_status_raw(root, 1, "started");
        raise_blocker_rv(root, 1);

        // started → audit is a forward Advance but NOT the closure seam — passes.
        run_status(Some(root.to_path_buf()), 1, SliceStatus::Audit, None).unwrap();
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "audit");
    }

    /// VT-5 (bypass guard, Charge VIII): the close command shell (`run_status` via
    /// `set_slice_status`) is the SOLE caller crossing the closure seam. This is a
    /// SOURCE-level assertion: `set_slice_status` is private to this module, and a
    /// grep of the whole module body finds exactly ONE call site (in `run_status`,
    /// the close shell) — so no other path can cross `audit→reconcile` /
    /// `reconcile→done` and thereby bypass the gate. If a SECOND call site ever
    /// appears, this test fails, forcing that caller to re-invoke the gate (or the
    /// design to move the gate into the FSM writer).
    #[test]
    fn vt5_close_shell_is_the_sole_seam_crossing_caller_of_set_slice_status() {
        let src = include_str!("slice.rs");
        // Scope to PRODUCTION code only — `set_slice_status` is module-private, so
        // the FSM writer reaches disk via exactly the call sites in this module.
        // Test-only callers (which exercise the writer directly) are excluded by
        // cutting at the `#[cfg(test)]` boundary; my own comment mentions live past
        // it too. The production region must hold exactly ONE call site.
        let production = src.split_once("#[cfg(test)]").map_or(src, |(head, _)| head);
        let call_sites = production
            .match_indices("set_slice_status(")
            .filter(|(i, _)| {
                // Exclude the definition `fn set_slice_status(`.
                !production.get(..*i).unwrap_or("").ends_with("fn ")
            })
            .count();
        assert_eq!(
            call_sites, 1,
            "exactly ONE production caller may cross the closure seam (the close \
             shell `run_status`); a second `set_slice_status(` call site bypasses \
             the close-gate (design §7 Charge VIII — re-invoke the gate, or move \
             it into the FSM writer)"
        );
    }

    // -----------------------------------------------------------------------
    // B·P3 — closure-gate drift predicate (D-B5/D-B3, REQ-113/FR-006).
    //
    // A real born git repo so coverage staleness resolves (anchor..HEAD over the
    // touched paths): Fresh evidence ⇒ live drift; the discharge clause (c) demands
    // the REC's evidence cover today's residual keys.
    // -----------------------------------------------------------------------

    use crate::coverage::CoverageKey;
    use crate::rec::{RecDoc, RecMeta, StatusDelta};
    use crate::requirement::{self, ReqKind, ReqStatus};

    fn git(root: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }

    /// A born git repo at a tempdir root, with one committed source file whose HEAD
    /// SHA is returned as the universal coverage anchor (Fresh when paths untouched
    /// since). Caller keeps the `TempDir` alive.
    fn drift_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git(root, &["init", "-q", "-b", "main"]);
        std::fs::write(root.join("src.rs"), "fn a() {}\n").unwrap();
        git(root, &["add", "src.rs"]);
        git(root, &["commit", "-q", "-m", "seed"]);
        let anchor = git(root, &["rev-parse", "HEAD"]);
        (dir, anchor)
    }

    /// Mint a requirement at `status`, returning its canonical FK (`REQ-NNN`).
    fn mint_req(root: &Path, status: ReqStatus) -> String {
        let id = requirement::reserve(root, "fast-boot", "Fast boot", "2026-06-12")
            .unwrap()
            .eid
            .numeric_id()
            .unwrap();
        requirement::set_kind(root, id, ReqKind::Functional).unwrap();
        requirement::set_status(root, id, status).unwrap();
        requirement::canonical_id(id)
    }

    /// Write a slice's OWN `coverage.toml` carrying one `Verified` entry for `req`,
    /// anchored at `anchor` over `src.rs` (Fresh in the seeded repo). `cov_slice` is
    /// the entry's `slice =` field — usually the owning slice, foreign for the
    /// integrity test.
    fn write_own_coverage(root: &Path, dir_id: u32, cov_slice: &str, req: &str, anchor: &str) {
        let d = root.join(SLICE_DIR).join(format!("{dir_id:03}"));
        fs::create_dir_all(&d).unwrap();
        let body = format!(
            "[[entry]]\nslice = \"{cov_slice}\"\nrequirement = \"{req}\"\n\
             contributing_change = \"{cov_slice}\"\nmode = \"VT\"\n\
             status = \"verified\"\ngit_anchor = \"{anchor}\"\n\
             touched_paths = [\"src.rs\"]\n"
        );
        fs::write(d.join("coverage.toml"), body).unwrap();
    }

    /// The DISTINCT coverage key the seeded `write_own_coverage` cell carries — what
    /// a discharging REC's `evidence_ref` must cover (clause c).
    fn cov_key(cov_slice: &str, req: &str) -> CoverageKey {
        CoverageKey {
            slice: cov_slice.to_owned(),
            requirement: req.to_owned(),
            contributing_change: cov_slice.to_owned(),
            mode: "VT".to_owned(),
        }
    }

    /// Materialise an owning-slice REC: `move`, one `status_delta` (req: from→to),
    /// and the given evidence keys. Returns the assigned REC id.
    fn mint_rec(
        root: &Path,
        owning: &str,
        r#move: &str,
        req: &str,
        from: ReqStatus,
        to: ReqStatus,
        evidence: Vec<CoverageKey>,
    ) -> u32 {
        let doc = RecDoc {
            id: 0,
            slug: format!("{move}-{}", req.to_lowercase()),
            title: format!("{move} {req}"),
            rec: RecMeta {
                r#move: r#move.to_owned(),
                owning_slice: Some(owning.to_owned()),
                decision_ref: None,
            },
            status_delta: vec![StatusDelta {
                requirement: req.to_owned(),
                from: from.as_str().to_owned(),
                to: to.as_str().to_owned(),
            }],
            evidence_ref: evidence,
        };
        crate::rec::materialise_populated(root, &doc).unwrap()
    }

    /// Drive the closing slice to `reconcile` (the legal source for `→ done`).
    fn slice_at_reconcile(root: &Path) {
        make_slice(root, "s", "S", "2026-06-12");
        set_status_raw(root, 1, "reconcile");
    }

    /// Attempt the `reconcile → done` crossing; return the error string (the gate
    /// refusal) — panics if it unexpectedly SUCCEEDS.
    fn expect_close_refused(root: &Path) -> String {
        run_status(Some(root.to_path_buf()), 1, SliceStatus::Done, None)
            .expect_err("reconcile → done should be refused")
            .to_string()
    }

    // --- VT-1: residual drift on a COVERED req refuses; F12 topology refuses an
    //           out-of-seam crossing INDEPENDENT of the drift check. ------------

    #[test]
    fn vt1_covered_req_residual_drift_refuses_close() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        // `pending` req with Fresh Verified coverage ⇒ Divergent(EvidenceOutruns).
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 1, "SL-001", &req, &anchor);

        let err = expect_close_refused(root);
        assert!(err.contains("undischarged residual drift"), "{err}");
        assert!(err.contains(&req), "names the offending req: {err}");
        // Refused BEFORE the write — status stays at reconcile.
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    #[test]
    fn vt1_f12_topology_refuses_out_of_seam_independent_of_drift() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        // A slice in `started` (NOT `reconcile`) → `done` is an F12 SeamBreach,
        // refused structurally — no coverage/REC in sight, the drift gate never runs.
        make_slice(root, "s", "S", "2026-06-12");
        set_status_raw(root, 1, "started");
        let err = run_status(Some(root.to_path_buf()), 1, SliceStatus::Done, None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("reconcile") && !err.contains("residual drift"),
            "F12 topology refusal, not the drift gate: {err}"
        );
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "started");
    }

    // --- VT-2 (D-B5): declared + reconciled reqs each block; additive floor. ----

    #[test]
    fn vt2_declared_extra_req_blocks_on_residual_drift() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        // The req is NOT in S's coverage.toml — only DECLARED via [gate].extra_reqs;
        // its drift must still gate. Coverage lives under a foreign slice dir so the
        // composite (corpus scan) sees it, but S's own coverage stays empty.
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 999, "SL-999", &req, &anchor);
        // Declare it on the closing slice.
        let p = slice_root(root).join("001").join("slice-001.toml");
        let mut toml = fs::read_to_string(&p).unwrap();
        toml.push_str(&format!("\n[gate]\nextra_reqs = [\"{req}\"]\n"));
        fs::write(&p, toml).unwrap();

        let err = expect_close_refused(root);
        assert!(err.contains(&req), "declared req gates: {err}");
    }

    #[test]
    fn vt2_reconciled_req_blocks_on_residual_drift() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        // Reconciled-only: named in an owning REC's status_delta, NOT in S's
        // coverage.toml nor declared. A `revise` REC (not accept) cannot discharge,
        // so the drift it left behind still gates (the opt-in dodge is closed).
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 999, "SL-999", &req, &anchor);
        mint_rec(
            root,
            "SL-001",
            "revise",
            &req,
            ReqStatus::Pending,
            ReqStatus::Pending,
            vec![cov_key("SL-999", &req)],
        );

        let err = expect_close_refused(root);
        assert!(err.contains(&req), "reconciled req gates: {err}");
    }

    #[test]
    fn vt2_no_gate_table_runs_on_covered_union_reconciled() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        // No [gate] table at all (declared = ∅). A covered req with residual drift
        // still gates — the floor is `covered ∪ reconciled`.
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 1, "SL-001", &req, &anchor);
        let toml = fs::read_to_string(slice_root(root).join("001").join("slice-001.toml")).unwrap();
        assert!(!toml.contains("[gate]"), "fixture has no [gate] table");

        let err = expect_close_refused(root);
        assert!(err.contains(&req), "{err}");
    }

    // --- VT-3 (R-B4): slice-local reader — distinct reqs; foreign slice refused. -

    #[test]
    fn vt3_slice_local_reader_returns_distinct_reqs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let d = root.join(SLICE_DIR).join("001");
        fs::create_dir_all(&d).unwrap();
        // Two entries for REQ-001 (distinct modes) + one for REQ-002 → {001, 002}.
        let body = "\
            [[entry]]\nslice = \"SL-001\"\nrequirement = \"REQ-001\"\n\
            contributing_change = \"SL-001\"\nmode = \"VT\"\nstatus = \"planned\"\n\
            git_anchor = \"a\"\n\
            [[entry]]\nslice = \"SL-001\"\nrequirement = \"REQ-001\"\n\
            contributing_change = \"SL-001\"\nmode = \"VA\"\nstatus = \"planned\"\n\
            git_anchor = \"a\"\n\
            [[entry]]\nslice = \"SL-001\"\nrequirement = \"REQ-002\"\n\
            contributing_change = \"SL-001\"\nmode = \"VT\"\nstatus = \"planned\"\n\
            git_anchor = \"a\"\n";
        fs::write(d.join("coverage.toml"), body).unwrap();
        let reqs = crate::coverage_scan::slice_local_covered_reqs(root, 1, "SL-001").unwrap();
        assert_eq!(reqs, vec!["REQ-001".to_owned(), "REQ-002".to_owned()]);
    }

    #[test]
    fn vt3_foreign_slice_in_own_coverage_is_an_integrity_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let d = root.join(SLICE_DIR).join("001");
        fs::create_dir_all(&d).unwrap();
        // SL-001's OWN coverage.toml citing SL-042 — a foreign slice =, refused.
        let body = "[[entry]]\nslice = \"SL-042\"\nrequirement = \"REQ-001\"\n\
            contributing_change = \"SL-042\"\nmode = \"VT\"\nstatus = \"planned\"\n\
            git_anchor = \"a\"\n";
        fs::write(d.join("coverage.toml"), body).unwrap();
        let err = crate::coverage_scan::slice_local_covered_reqs(root, 1, "SL-001")
            .unwrap_err()
            .to_string();
        assert!(err.contains("integrity error"), "{err}");
        assert!(err.contains("SL-042"), "names the foreign slice: {err}");
    }

    #[test]
    fn vt3_absent_coverage_is_empty_not_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(
            crate::coverage_scan::slice_local_covered_reqs(root, 1, "SL-001")
                .unwrap()
                .is_empty()
        );
    }

    // --- F-3 regression: the discharge is FOR R — a multi-delta REC must not -----
    // discharge R's drift on the strength of a foreign requirement's coinciding `to`.

    #[test]
    fn multi_delta_rec_does_not_discharge_via_foreign_requirement() {
        // A single accept REC carrying deltas for TWO requirements: R1's own delta
        // moves it to `active` (NOT R1's current authored `pending`), while R2's
        // delta happens to land on `pending`. R1 is queried at authored=pending with
        // no residual evidence. Clause (b) must look at R1's OWN delta (to=active ≠
        // pending ⇒ NOT discharged), never R2's coinciding to=pending.
        let rec = RecDoc {
            id: 1,
            slug: "accept-req-001".to_owned(),
            title: "accept REQ-001".to_owned(),
            rec: RecMeta {
                r#move: "accept".to_owned(),
                owning_slice: Some("SL-001".to_owned()),
                decision_ref: None,
            },
            status_delta: vec![
                StatusDelta {
                    requirement: "REQ-001".to_owned(),
                    from: "pending".to_owned(),
                    to: "active".to_owned(),
                },
                StatusDelta {
                    requirement: "REQ-002".to_owned(),
                    from: "active".to_owned(),
                    to: "pending".to_owned(),
                },
            ],
            evidence_ref: Vec::new(),
        };
        assert!(
            !rec_discharges(Some(&rec), "REQ-001", ReqStatus::Pending, &[]),
            "a foreign requirement's coinciding `to` laundered R1's drift"
        );
        // Control: a single-delta REC that affirms R1 AT its current status discharges.
        let affirm = RecDoc {
            status_delta: vec![StatusDelta {
                requirement: "REQ-001".to_owned(),
                from: "pending".to_owned(),
                to: "pending".to_owned(),
            }],
            ..rec
        };
        assert!(
            rec_discharges(Some(&affirm), "REQ-001", ReqStatus::Pending, &[]),
            "an accept REC affirming R1 at its current status should discharge"
        );
    }

    // --- VT-4 (discharge i): accept + to==current + evidence ⊇ residual ⇒ close. -

    #[test]
    fn vt4_matching_accept_rec_discharges_the_drift() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        // `active` req with Fresh Verified coverage: drift is Coherent for active —
        // so instead use a residual-drift status the REC then affirms. A `pending`
        // req carries Divergent drift; an accept REC with `to == pending` (current)
        // whose evidence covers the key discharges it.
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 1, "SL-001", &req, &anchor);
        mint_rec(
            root,
            "SL-001",
            "accept",
            &req,
            ReqStatus::Pending,
            ReqStatus::Pending,
            vec![cov_key("SL-001", &req)],
        );

        run_status(Some(root.to_path_buf()), 1, SliceStatus::Done, None).unwrap();
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "done");
    }

    // --- VT-5 (discharge ii, clause c): post-REC fresh evidence re-opens drift. --

    #[test]
    fn vt5_post_rec_fresh_evidence_undischarges() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        let req = mint_req(root, ReqStatus::Pending);
        // S's coverage has the original VT cell; the REC affirms over THAT key only.
        write_own_coverage(root, 1, "SL-001", &req, &anchor);
        mint_rec(
            root,
            "SL-001",
            "accept",
            &req,
            ReqStatus::Pending,
            ReqStatus::Pending,
            vec![cov_key("SL-001", &req)],
        );
        // Fresh contradictory evidence arrives AFTER the REC: a NEW key (different
        // contributing_change) the REC's evidence_ref does not cover. The composite
        // now carries a residual key clause (c) lacks → undischarged.
        let d = root.join(SLICE_DIR).join("001");
        let extra = format!(
            "\n[[entry]]\nslice = \"SL-001\"\nrequirement = \"{req}\"\n\
             contributing_change = \"SL-002\"\nmode = \"VT\"\nstatus = \"verified\"\n\
             git_anchor = \"{anchor}\"\ntouched_paths = [\"src.rs\"]\n"
        );
        let p = d.join("coverage.toml");
        let mut body = fs::read_to_string(&p).unwrap();
        body.push_str(&extra);
        fs::write(&p, body).unwrap();

        let err = expect_close_refused(root);
        assert!(
            err.contains(&req),
            "stale REC cannot excuse fresh evidence: {err}"
        );
    }

    // --- VT-6 (discharge iii/iv): revise/redesign + foreign-slice REC don't. -----

    #[test]
    fn vt6_revise_rec_does_not_discharge() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 1, "SL-001", &req, &anchor);
        // A `revise` REC (move != accept) cannot discharge — clause (a) fails.
        mint_rec(
            root,
            "SL-001",
            "revise",
            &req,
            ReqStatus::Pending,
            ReqStatus::Pending,
            vec![cov_key("SL-001", &req)],
        );
        let err = expect_close_refused(root);
        assert!(err.contains(&req), "revise REC does not discharge: {err}");
    }

    #[test]
    fn vt6_foreign_owning_slice_rec_does_not_discharge() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 1, "SL-001", &req, &anchor);
        // An accept REC that would discharge — but owned by a DIFFERENT slice. A
        // gate honours only its OWN slice's RECs, so it is not even in scope.
        mint_rec(
            root,
            "SL-777",
            "accept",
            &req,
            ReqStatus::Pending,
            ReqStatus::Pending,
            vec![cov_key("SL-001", &req)],
        );
        let err = expect_close_refused(root);
        assert!(
            err.contains(&req),
            "foreign-slice REC does not discharge: {err}"
        );
    }

    // --- VT-7: composes with D-C9b — blocker AND drift each independently refuse. -

    #[test]
    fn vt7_blocker_and_drift_each_independently_refuse() {
        let (dir, anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        let req = mint_req(root, ReqStatus::Pending);
        write_own_coverage(root, 1, "SL-001", &req, &anchor);

        // (i) Blocker alone refuses: discharge the drift (matching accept REC), then
        // raise an unresolved blocker — the blocker gate must still refuse.
        let rec_id = mint_rec(
            root,
            "SL-001",
            "accept",
            &req,
            ReqStatus::Pending,
            ReqStatus::Pending,
            vec![cov_key("SL-001", &req)],
        );
        let _ = rec_id;
        raise_blocker_rv(root, 1);
        let err_blocker = expect_close_refused(root);
        assert!(
            err_blocker.contains("blocker review finding"),
            "blocker gate refuses independently: {err_blocker}"
        );

        // (ii) Drift alone refuses: a SEPARATE slice with residual drift but NO
        // blocker — the drift gate refuses on its own.
        let (dir2, anchor2) = drift_repo();
        let root2 = dir2.path();
        slice_at_reconcile(root2);
        let req2 = mint_req(root2, ReqStatus::Pending);
        write_own_coverage(root2, 1, "SL-001", &req2, &anchor2);
        let err_drift = expect_close_refused(root2);
        assert!(
            err_drift.contains("residual drift"),
            "drift gate refuses independently: {err_drift}"
        );
    }

    // NF-001: the closure gate cannot branch on estimate/value presence — the facet is
    // structurally absent from Gate's input type. Exhaustive (no `..`): a future
    // estimate/value field on Gate breaks this compile.
    #[test]
    fn nf001_gate_destructure_is_exhaustive_and_facet_free() {
        let Gate { extra_reqs: _ } = Gate::default();
    }

    // -----------------------------------------------------------------------
    // PHASE-02 close-integration gate (EX-1/EX-2, VT-1..VT-6). The THIRD reverse
    // close-gate: `reconcile → done` refuses when a DISPATCHED slice's code never
    // integrated to trunk. Composes with the blocker + drift gates above.
    //
    // Fixtures mirror `ledger::tests::JournalRepo` — a `journal.toml` committed on
    // an orphan `refs/heads/dispatch/<slice:03>` branch (the coordination ref tree
    // the query reads, never the working filesystem), built BEFORE the slice
    // entity so the slice's untracked working-tree files are never disturbed.
    // -----------------------------------------------------------------------

    /// A single journal row in on-disk TOML form (only the query-relevant fields
    /// carry meaning; the rest satisfy the non-`default` serde requirements).
    /// Mirrors `ledger::tests::journal_row_toml` (leaf-tier sibling — no shared
    /// cross-module test harness).
    fn dispatch_row_toml(target_ref: &str, planned_new_oid: &str) -> String {
        format!(
            "[[row]]\n\
             source_oid = \"src\"\n\
             target_ref = \"{target_ref}\"\n\
             expected_old_oid = \"{zero}\"\n\
             planned_new_oid = \"{planned_new_oid}\"\n\
             status = \"pending\"\n",
            zero = "0".repeat(40),
        )
    }

    /// Commit `body` as `journal.toml` onto an orphan `dispatch/<slice:03>` branch,
    /// leaving the working branch on `main`. Mirrors `JournalRepo::commit_journal`.
    fn commit_dispatch_journal(root: &Path, slice: u32, body: &str) {
        let branch = format!("dispatch/{slice:03}");
        git(root, &["checkout", "-q", "--orphan", &branch]);
        git(root, &["rm", "-rf", "--cached", "--ignore-unmatch", "."]);
        let rel = format!(".doctrine/dispatch/{slice:03}/journal.toml");
        let full = root.join(&rel);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, body).unwrap();
        git(root, &["add", &rel]);
        git(root, &["commit", "-q", "-m", "coordinate: journal"]);
        git(root, &["checkout", "-f", "main"]);
    }

    /// Create a fresh dispatch branch carrying ONLY a placeholder (no `journal.toml`),
    /// leaving the working branch on `main`.
    fn commit_dispatch_no_journal(root: &Path, slice: u32) {
        let branch = format!("dispatch/{slice:03}");
        git(root, &["checkout", "-q", "--orphan", &branch]);
        git(root, &["rm", "-rf", "--cached", "--ignore-unmatch", "."]);
        fs::write(root.join("placeholder.txt"), "x").unwrap();
        git(root, &["add", "placeholder.txt"]);
        git(root, &["commit", "-q", "-m", "coordinate: no journal"]);
        git(root, &["checkout", "-f", "main"]);
    }

    /// Drive `reconcile → done` to SUCCESS; panic if the gate refuses.
    fn expect_close_succeeds(root: &Path) {
        run_status(Some(root.to_path_buf()), 1, SliceStatus::Done, None)
            .expect("reconcile → done should succeed");
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "done");
    }

    // VT-1: never dispatched (no `dispatch/001` ref) ⇒ the gate is silent; the
    // crossing succeeds.
    #[test]
    fn vt1_close_integration_not_dispatched_succeeds() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        slice_at_reconcile(root);
        expect_close_succeeds(root);
    }

    // VT-1b: dispatch ref present but the journal has zero rows ⇒ NotDispatched ⇒
    // the crossing succeeds (a coordinated-but-never-projected slice).
    #[test]
    fn vt1b_close_integration_dispatched_empty_journal_succeeds() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        commit_dispatch_journal(root, 1, "");
        slice_at_reconcile(root);
        expect_close_succeeds(root);
    }

    // VT-2: dispatched, the trunk row's planned oid IS an ancestor of
    // `refs/heads/main` ⇒ Integrated ⇒ the crossing succeeds.
    #[test]
    fn vt2_close_integration_planned_on_trunk_succeeds() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        // Advance `main` so the recorded planned oid is a strict ancestor of the tip.
        std::fs::write(root.join("src.rs"), "fn a() {}\nfn b() {}\n").unwrap();
        git(root, &["add", "src.rs"]);
        git(root, &["commit", "-q", "-m", "landed"]);
        let landed = git(root, &["rev-parse", "HEAD"]);
        std::fs::write(root.join("src.rs"), "fn a() {}\nfn b() {}\nfn c() {}\n").unwrap();
        git(root, &["add", "src.rs"]);
        git(root, &["commit", "-q", "-m", "advance trunk"]);
        commit_dispatch_journal(root, 1, &dispatch_row_toml("refs/heads/main", &landed));
        slice_at_reconcile(root);
        expect_close_succeeds(root);
    }

    // VT-3: dispatched, the planned oid is NOT on trunk ⇒ Blocked ⇒ REFUSED. The
    // refusal carries both the anomaly reason token AND the retry guidance.
    #[test]
    fn vt3_close_integration_planned_off_trunk_refuses() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        // A divergent commit on a side branch, never merged into main.
        git(root, &["checkout", "-q", "-b", "side"]);
        std::fs::write(root.join("side.rs"), "fn x() {}\n").unwrap();
        git(root, &["add", "side.rs"]);
        git(root, &["commit", "-q", "-m", "divergent"]);
        let orphaned = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-f", "main"]);
        commit_dispatch_journal(root, 1, &dispatch_row_toml("refs/heads/main", &orphaned));
        slice_at_reconcile(root);

        let err = expect_close_refused(root);
        assert!(
            err.contains("not integrated to trunk"),
            "names the integration anomaly: {err}"
        );
        assert!(
            err.contains("planned tip not on trunk"),
            "carries the leaf reason token: {err}"
        );
        assert!(
            err.contains("dispatch sync --integrate"),
            "carries the retry guidance: {err}"
        );
        // Refused BEFORE the write — status stays at reconcile.
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    // VT-4: dispatched (journal HAS rows) but NO `refs/heads/main` row ⇒ Blocked ⇒
    // REFUSED (fail-closed — integrate --trunk never completed).
    #[test]
    fn vt4_close_integration_no_trunk_row_refuses_fail_closed() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        let oid = git(root, &["rev-parse", "HEAD"]);
        commit_dispatch_journal(root, 1, &dispatch_row_toml("refs/heads/edge", &oid));
        slice_at_reconcile(root);

        let err = expect_close_refused(root);
        assert!(
            err.contains("no trunk row") && err.contains("integrate --trunk never completed"),
            "fail-closed on a missing trunk row: {err}"
        );
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    // VT-5: the integration gate fires ONLY on `reconcile → done`. An unintegrated
    // dispatched slice crossing `audit → reconcile` is NOT refused by THIS gate
    // (the blocker gate runs there, but with no blocker the crossing passes).
    #[test]
    fn vt5_close_integration_does_not_fire_off_the_reconcile_done_crossing() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        // A planned oid off trunk — would Block the `reconcile → done` gate.
        git(root, &["checkout", "-q", "-b", "side"]);
        std::fs::write(root.join("side.rs"), "fn x() {}\n").unwrap();
        git(root, &["add", "side.rs"]);
        git(root, &["commit", "-q", "-m", "divergent"]);
        let orphaned = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-f", "main"]);
        commit_dispatch_journal(root, 1, &dispatch_row_toml("refs/heads/main", &orphaned));
        // Slice at `audit` — the legal source for `→ reconcile`.
        make_slice(root, "s", "S", "2026-06-12");
        set_status_raw(root, 1, "audit");

        run_status(Some(root.to_path_buf()), 1, SliceStatus::Reconcile, None)
            .expect("audit → reconcile is not gated by the integration check");
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    // VT-6: composition — an unintegrated slice that ALSO has an unresolved blocker
    // is refused; each gate independently suffices on the `reconcile → done`
    // crossing (the blocker gate runs first and refuses here).
    #[test]
    fn vt6_close_integration_composes_with_the_blocker_gate() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        git(root, &["checkout", "-q", "-b", "side"]);
        std::fs::write(root.join("side.rs"), "fn x() {}\n").unwrap();
        git(root, &["add", "side.rs"]);
        git(root, &["commit", "-q", "-m", "divergent"]);
        let orphaned = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-f", "main"]);
        commit_dispatch_journal(root, 1, &dispatch_row_toml("refs/heads/main", &orphaned));
        slice_at_reconcile(root);
        raise_blocker_rv(root, 1);

        // Both gates would refuse; the crossing is refused (blocker first).
        let err = expect_close_refused(root);
        assert!(
            err.contains("blocker review finding"),
            "an unresolved blocker independently refuses: {err}"
        );
        assert_eq!(read_status(&slice_root(root), 1).unwrap(), "reconcile");
    }

    // VT-6 (integration-alone half): the SAME unintegrated slice with NO blocker is
    // still refused — proving the integration gate refuses on its own.
    #[test]
    fn vt6_close_integration_refuses_independently_of_the_blocker() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        git(root, &["checkout", "-q", "-b", "side"]);
        std::fs::write(root.join("side.rs"), "fn x() {}\n").unwrap();
        git(root, &["add", "side.rs"]);
        git(root, &["commit", "-q", "-m", "divergent"]);
        let orphaned = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-f", "main"]);
        commit_dispatch_journal(root, 1, &dispatch_row_toml("refs/heads/main", &orphaned));
        slice_at_reconcile(root);

        let err = expect_close_refused(root);
        assert!(
            err.contains("not integrated to trunk"),
            "integration gate refuses with no blocker present: {err}"
        );
    }

    // Belt-and-braces: silence the unused-helper lint if `commit_dispatch_no_journal`
    // is the only at-ref-present-no-journal path exercised via the leaf unit suite;
    // here it drives a NotDispatched success the gate must wave through.
    #[test]
    fn vt1c_close_integration_dispatch_ref_present_no_journal_succeeds() {
        let (dir, _anchor) = drift_repo();
        let root = dir.path();
        commit_dispatch_no_journal(root, 1);
        slice_at_reconcile(root);
        expect_close_succeeds(root);
    }

    // --- SL-139 PHASE-03 paths verb tests ---

    fn paths_slice_fixture(root: &Path, id: u32, extra: &[&str]) {
        let slice_root = root.join(SLICE_DIR);
        let name = format!("{id:03}");
        let dir = slice_root.join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("slice-{name}.toml")), "toml").unwrap();
        std::fs::write(dir.join(format!("slice-{name}.md")), "md").unwrap();
        for e in extra {
            std::fs::write(dir.join(e), e).unwrap();
        }
    }

    #[test]
    fn paths_slice_full_output() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        paths_slice_fixture(root, 1, &["notes.md"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: false,
        };
        let slice_root = root.join(SLICE_DIR);
        let entity_dir = slice_root.join("001");
        let set = crate::paths::scan_entity_dir(
            &entity_dir,
            &entity_dir.join("slice-001.toml"),
            Some(&entity_dir.join("slice-001.md")),
            root,
        )
        .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(
            lines,
            vec![
                ".doctrine/slice/001/slice-001.toml",
                ".doctrine/slice/001/slice-001.md",
                ".doctrine/slice/001/notes.md"
            ]
        );
    }

    #[test]
    fn paths_slice_single() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        paths_slice_fixture(root, 1, &["design.md"]);
        let sel = crate::paths::PathSelection {
            toml: false,
            md: false,
            entity: false,
            single: true,
        };
        let slice_root = root.join(SLICE_DIR);
        let entity_dir = slice_root.join("001");
        let set = crate::paths::scan_entity_dir(
            &entity_dir,
            &entity_dir.join("slice-001.toml"),
            Some(&entity_dir.join("slice-001.md")),
            root,
        )
        .unwrap();
        let lines = crate::paths::select_paths(&set, &sel).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], ".doctrine/slice/001/slice-001.toml");
    }

    #[test]
    fn paths_slice_missing_entity_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No slice dir at all.
        let slice_root = root.join(SLICE_DIR);
        let result = crate::paths::scan_entity_dir(
            &slice_root.join("999"),
            &slice_root.join("999/slice-999.toml"),
            Some(&slice_root.join("999/slice-999.md")),
            root,
        );
        assert!(result.is_err());
    }

    // -- SL-147 PHASE-01: selector tests -------------------------------

    #[test]
    fn selector_intent_serde_kebab_case_round_trip() {
        // VT-2: kebab-case round-trip via Selector struct
        let selector: Selector =
            toml::from_str("selector = \"src/x.rs\"\nintent = \"scope-relevant\"\n").unwrap();
        assert_eq!(selector.intent, SelectorIntent::ScopeRelevant);

        let selector: Selector =
            toml::from_str("selector = \"src/y.rs\"\nintent = \"design-target\"\nnote = \"hi\"\n")
                .unwrap();
        assert_eq!(selector.intent, SelectorIntent::DesignTarget);
        assert_eq!(selector.note.as_deref(), Some("hi"));

        // Unknown variant rejected
        let err = toml::from_str::<Selector>("selector = \"x\"\nintent = \"bogus\"\n");
        assert!(err.is_err(), "unknown intent should be rejected");
    }

    #[test]
    fn slice_doc_without_selectors_deserializes_empty() {
        // EX-1 / VT-2: absent [[selector]] → empty Vec
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "no-selectors", "No Selectors", "2026-06-24");
        let (doc, _toml_text, _body) = read_slice(&slice_root(root), 1).unwrap();
        assert!(doc.selectors.is_empty());
    }

    #[test]
    fn selector_add_and_read_back() {
        // VT-1: add selectors, read back via (re)deserialization
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "test", "Test", "2026-06-24");

        let slice_root = root.join(SLICE_DIR);
        let toml_path = slice_toml_path(&slice_root, 1);

        // Add 3 selectors
        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        selector_upsert(&mut doc, "src/x.rs", "design-target", None).unwrap();
        selector_upsert(&mut doc, "src/y.rs", "design-target", Some("shared note")).unwrap();
        selector_upsert(&mut doc, "docs/*.md", "scope-relevant", None).unwrap();
        fs::write(&toml_path, doc.to_string()).unwrap();

        // Re-read as SliceDoc
        let (doc, _toml_text, _body) = read_slice(&slice_root, 1).unwrap();
        assert_eq!(doc.selectors.len(), 3);
        assert_eq!(doc.selectors[0].selector, "src/x.rs");
        assert_eq!(doc.selectors[0].intent, SelectorIntent::DesignTarget);
        assert_eq!(doc.selectors[0].note, None);
        assert_eq!(doc.selectors[1].selector, "src/y.rs");
        assert_eq!(doc.selectors[1].note.as_deref(), Some("shared note"));
        assert_eq!(doc.selectors[2].selector, "docs/*.md");
        assert_eq!(doc.selectors[2].intent, SelectorIntent::ScopeRelevant);
    }

    #[test]
    fn selector_upsert_is_idempotent() {
        // VT-1: re-add same selector updates intent/note
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "test", "Test", "2026-06-24");

        let slice_root = root.join(SLICE_DIR);
        let toml_path = slice_toml_path(&slice_root, 1);

        // First add
        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        selector_upsert(&mut doc, "src/x.rs", "scope-relevant", Some("first")).unwrap();
        fs::write(&toml_path, doc.to_string()).unwrap();

        // Re-add with different intent + note
        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        selector_upsert(&mut doc, "src/x.rs", "design-target", Some("updated")).unwrap();
        fs::write(&toml_path, doc.to_string()).unwrap();

        // Read back — should have ONE entry with updated values
        let (doc, _toml_text, _body) = read_slice(&slice_root, 1).unwrap();
        assert_eq!(doc.selectors.len(), 1, "upsert should not duplicate");
        assert_eq!(doc.selectors[0].selector, "src/x.rs");
        assert_eq!(doc.selectors[0].intent, SelectorIntent::DesignTarget);
        assert_eq!(doc.selectors[0].note.as_deref(), Some("updated"));
    }

    #[test]
    fn selector_note_sets_per_file_note() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "test", "Test", "2026-06-24");

        let slice_root = root.join(SLICE_DIR);
        let toml_path = slice_toml_path(&slice_root, 1);

        // Add two selectors with shared note
        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        selector_upsert(&mut doc, "src/x.rs", "design-target", Some("shared")).unwrap();
        selector_upsert(&mut doc, "src/y.rs", "design-target", Some("shared")).unwrap();
        fs::write(&toml_path, doc.to_string()).unwrap();

        // Override note on just one
        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert!(selector_set_note(&mut doc, "src/x.rs", "per-file override"));
        fs::write(&toml_path, doc.to_string()).unwrap();

        let (doc, _toml_text, _body) = read_slice(&slice_root, 1).unwrap();
        assert_eq!(doc.selectors.len(), 2);
        assert_eq!(doc.selectors[0].note.as_deref(), Some("per-file override"));
        assert_eq!(doc.selectors[1].note.as_deref(), Some("shared"));
    }

    #[test]
    fn selector_note_missing_fails() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "test", "Test", "2026-06-24");

        let slice_root = root.join(SLICE_DIR);
        let toml_path = slice_toml_path(&slice_root, 1);
        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert!(!selector_set_note(&mut doc, "nonexistent", "note"));
    }

    #[test]
    fn selector_rm_variadic() {
        // VT-1: add 3, rm 2, verify 1 remains
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "test", "Test", "2026-06-24");

        let slice_root = root.join(SLICE_DIR);
        let toml_path = slice_toml_path(&slice_root, 1);

        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        selector_upsert(&mut doc, "src/a.rs", "design-target", None).unwrap();
        selector_upsert(&mut doc, "src/b.rs", "design-target", None).unwrap();
        selector_upsert(&mut doc, "src/c.rs", "design-target", None).unwrap();
        fs::write(&toml_path, doc.to_string()).unwrap();

        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        let removed = selector_remove_many(&mut doc, &["src/a.rs".into(), "src/b.rs".into()]);
        assert_eq!(removed, 2);
        fs::write(&toml_path, doc.to_string()).unwrap();

        let (doc, _toml_text, _body) = read_slice(&slice_root, 1).unwrap();
        assert_eq!(doc.selectors.len(), 1);
        assert_eq!(doc.selectors[0].selector, "src/c.rs");
    }

    #[test]
    fn selector_rm_noop_on_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice(root, "test", "Test", "2026-06-24");

        let slice_root = root.join(SLICE_DIR);
        let toml_path = slice_toml_path(&slice_root, 1);
        let mut doc = fs::read_to_string(&toml_path)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        let removed = selector_remove_many(&mut doc, &["nonexistent".into()]);
        assert_eq!(removed, 0);
    }
}
