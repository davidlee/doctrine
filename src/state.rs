// SPDX-License-Identifier: GPL-3.0-only
//! Mutable runtime state under `.doctrine/state/` — phase tracking.
//!
//! Separate from the scaffold engine by *contract*, not IO (slice-004 D3): the
//! engine writes write-once authored filesets and refuses clobber; this module
//! owns idempotently-rewritten runtime state, which is disposable and
//! gitignored. The two share only `fsutil` primitives. State paths are always
//! computed from the slice id — the convenience symlink is never followed
//! (id is identity).

use std::collections::BTreeSet;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::ValueEnum;
use serde::Deserialize;
use toml_edit::Item;

use crate::boundary::BoundaryRow;
use crate::fsutil;
// `Plan`/`PlanPhase` are now an engine-tier leaf (`crate::plan`, SL-016): the
// state layer depends *down* on a neutral home, not up into the slice-CLI
// module — the slice↔state cycle is gone (ADR-001). Residual debt: `PhaseStatus`
// below still carries `clap::ValueEnum`, so the arg parser leaks into the state
// layer — out of scope here; split the CLI enum from the stored value if a
// second consumer appears.
use crate::plan::{Plan, PlanPhase};

/// Slice-scoped runtime-state tree, relative to the project root.
const STATE_SLICE_DIR: &str = ".doctrine/state/slice";
/// The authored slice tree — only the convenience `phases` symlink is written
/// here (gitignored); never tracking data (the runtime/authored boundary).
const SLICE_DIR: &str = ".doctrine/slice";

/// A phase's runtime lifecycle status. The CLI surface for `slice phase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum PhaseStatus {
    Planned,
    #[value(name = "in_progress")]
    InProgress,
    Completed,
    Blocked,
}

impl PhaseStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Blocked => "blocked",
        }
    }
}

/// A phase's contribution to the rollup before folding: either a parsed status
/// string from its `phase-NN.toml`, or a `.md`-only crash-partial whose status
/// is unreadable. Keeping the latter explicit is what stops `total` from
/// silently shrinking (design D8 / R-F4).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StemStatus<'a> {
    Toml(&'a str),
    MissingToml,
}

/// Derived completion counts for one slice's phases. Every phase lands in exactly
/// one bucket, so the total is their sum and can never undercount; `unknown` (a
/// status string outside the `PhaseStatus` set) and `missing_toml` are kept
/// distinct so corruption is surfaced, not folded into "incomplete" (R-F3/R-F5).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct PhaseRollup {
    pub(crate) planned: u32,
    pub(crate) in_progress: u32,
    pub(crate) completed: u32,
    pub(crate) blocked: u32,
    pub(crate) unknown: u32,
    pub(crate) missing_toml: u32,
}

impl PhaseRollup {
    /// Every phase, summed across buckets — never undercounts.
    pub(crate) fn total(&self) -> u32 {
        self.planned
            + self.in_progress
            + self.completed
            + self.blocked
            + self.unknown
            + self.missing_toml
    }

    /// Phases whose tracking is malformed (unrecognised status or unreadable
    /// `.toml`) — surfaced as the `?N` marker, and suppresses divergence.
    pub(crate) fn anomalies(&self) -> u32 {
        self.unknown + self.missing_toml
    }
}

/// Fold per-stem statuses into the bucket counts. Pure — no IO, no clock. An
/// unrecognised status string lands in `unknown`; a `.md`-only stem in
/// `missing_toml`. The known set is `PhaseStatus`'s own value names, so the
/// vocabulary has one source (no parallel string list).
pub(crate) fn fold_rollup(stems: &[StemStatus<'_>]) -> PhaseRollup {
    let mut r = PhaseRollup::default();
    for stem in stems {
        match stem {
            StemStatus::MissingToml => r.missing_toml += 1,
            StemStatus::Toml(s) => match PhaseStatus::from_str(s, false) {
                Ok(PhaseStatus::Planned) => r.planned += 1,
                Ok(PhaseStatus::InProgress) => r.in_progress += 1,
                Ok(PhaseStatus::Completed) => r.completed += 1,
                Ok(PhaseStatus::Blocked) => r.blocked += 1,
                Err(_) => r.unknown += 1,
            },
        }
    }
    r
}

/// What `init_phases` did, so the caller reports drift without this module
/// printing. `created`: phases that got a file written this run (new, or a
/// crash-partial phase completed per-file). `orphan`: tracking on disk whose
/// plan phase is gone (a rename presents as orphan + a fresh phase) — reported,
/// never silently consumed. `pruned`: orphans removed under explicit `--prune`.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct InitReport {
    pub created: Vec<String>,
    pub orphan: Vec<String>,
    pub pruned: Vec<String>,
}

/// Canonical state path for a slice's phase tracking, computed from the id.
pub(crate) fn phases_dir(project_root: &Path, slice_id: u32) -> PathBuf {
    project_root
        .join(STATE_SLICE_DIR)
        .join(format!("{slice_id:03}"))
        .join("phases")
}

/// Validate a phase id and derive its filename stem. Enforces the canonical
/// `PHASE-<digits>` form (→ `phase-NN`), which both makes the derivation total
/// and rejects filesystem-unsafe input — empty, separators, `..`, leading dot
/// all fail the single rule, since a phase id reaches the filesystem (finding
/// 4). `NN` is kept verbatim; only the `PHASE-` prefix is lowercased.
pub(crate) fn phase_stem(phase_id: &str) -> anyhow::Result<String> {
    let digits = phase_id
        .strip_prefix("PHASE-")
        .filter(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
        .with_context(|| format!("Phase id {phase_id:?} must match PHASE-<digits>"))?;
    Ok(format!("phase-{digits}"))
}

/// Reverse of `phase_stem`: recover the canonical `PHASE-NN` id from an on-disk
/// stem so the drift report speaks one dialect — `orphan`/`pruned` match
/// `created`'s canonical ids, never the derived filename form. Stems reach this
/// only from `existing_phase_stems`, which guarantees the `phase-` prefix.
fn phase_id_from_stem(stem: &str) -> String {
    format!("PHASE-{}", stem.strip_prefix("phase-").unwrap_or(stem))
}

/// Materialise, per declared phase, a `phase-NN.toml` (tracking) + `phase-NN.md`
/// (disposable sheet) under the state tree. Ensures the (gitignored,
/// possibly-absent) parent, then writes any **missing file of each pair** —
/// per-file skip, so a phase left half-written by a crash completes on re-run.
/// Diffs on-disk tracking against the plan: orphans (plan phase gone) are
/// reported, removed only under `prune`. Refreshes the verified convenience
/// symlink. Idempotent on the no-drift path. All phase ids are validated up
/// front (before any write); uniqueness is guaranteed by `Plan::parse`.
pub(crate) fn init_phases(
    project_root: &Path,
    slice_id: u32,
    plan: &Plan,
    prune: bool,
) -> anyhow::Result<InitReport> {
    // Validate + derive every phase stem before touching the filesystem, so one
    // malformed id fails the whole init clean rather than after the earlier phases
    // are already written (a non-atomic init rejecting input knowable up front).
    // `phase_stem` is the single `PHASE-<digits>` validator.
    let stems = plan
        .phases
        .iter()
        .map(|phase| Ok((phase, phase_stem(&phase.id)?)))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let dir = phases_dir(project_root, slice_id);
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;

    let mut report = InitReport::default();
    let mut plan_stems = BTreeSet::new();

    for (phase, stem) in &stems {
        plan_stems.insert(stem.clone());
        let wrote_toml = write_if_absent(
            &dir.join(format!("{stem}.toml")),
            &render_tracking(&phase.id),
        )?;
        let wrote_md =
            write_if_absent(&dir.join(format!("{stem}.md")), &render_phase_sheet(phase)?)?;
        if wrote_toml || wrote_md {
            report.created.push(phase.id.clone());
        }
    }

    for stem in existing_phase_stems(&dir)? {
        if plan_stems.contains(&stem) {
            continue;
        }
        if prune {
            drop(fs::remove_file(dir.join(format!("{stem}.toml"))));
            drop(fs::remove_file(dir.join(format!("{stem}.md"))));
            report.pruned.push(phase_id_from_stem(&stem));
        } else {
            report.orphan.push(phase_id_from_stem(&stem));
        }
    }

    refresh_symlink(project_root, slice_id)?;
    Ok(report)
}

/// Write `body` to `path` only if absent. `create_new` makes the skip atomic;
/// the partial-init recovery (finding round-2) rides on this per-file skip.
fn write_if_absent(path: &Path, body: &str) -> anyhow::Result<bool> {
    match fsutil::create_new_file(path) {
        Ok(mut f) => {
            f.write_all(body.as_bytes())
                .with_context(|| format!("Failed to write {}", path.display()))?;
            Ok(true)
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(e).with_context(|| format!("Failed to create {}", path.display())),
    }
}

/// Distinct `phase-*` stems already present on disk (from either file of a
/// pair). A missing dir yields an empty set.
pub(crate) fn existing_phase_stems(dir: &Path) -> anyhow::Result<BTreeSet<String>> {
    let mut stems = BTreeSet::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(stems),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", dir.display())),
    };
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if let Some(stem) = name
            .strip_suffix(".toml")
            .or_else(|| name.strip_suffix(".md"))
            && stem.starts_with("phase-")
        {
            stems.insert(stem.to_string());
        }
    }
    Ok(stems)
}

/// The one field the rollup reads from a `phase-NN.toml`. Optional + tolerant:
/// unknown keys are ignored, and a malformed file parses to `status = None`
/// (treated as `missing_toml`, never an error — design § 5.5).
#[derive(Deserialize)]
struct TrackingStatus {
    status: Option<String>,
}

/// Read one phase stem's status string. `None` when the `.toml` is absent (a
/// `.md`-only crash-partial), unparseable, or carries no `status` — all of which
/// the fold counts as `missing_toml` rather than dropping the phase (R-F4). A
/// genuine IO error (not "not found") propagates.
pub(crate) fn read_phase_status(dir: &Path, stem: &str) -> anyhow::Result<Option<String>> {
    let path = dir.join(format!("{stem}.toml"));
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", path.display())),
    };
    Ok(toml::from_str::<TrackingStatus>(&text)
        .ok()
        .and_then(|t| t.status))
}

/// Derive the phase-completion rollup for a slice from its runtime tracking tree.
/// The phase set comes from `existing_phase_stems` — the same notion `init_phases`
/// uses (either half of the pair) — so `.md`-only crash-partials count as
/// `missing_toml` and the total never silently shrinks (D8). `None` only when no
/// phase exists at all (dir absent or empty): the *untracked* signal. The path is
/// id-derived; the convenience `phases` symlink is never followed.
pub(crate) fn phase_rollup(
    project_root: &Path,
    slice_id: u32,
) -> anyhow::Result<Option<PhaseRollup>> {
    let dir = phases_dir(project_root, slice_id);
    let stems = existing_phase_stems(&dir)?;
    if stems.is_empty() {
        return Ok(None);
    }
    let statuses: Vec<Option<String>> = stems
        .iter()
        .map(|stem| read_phase_status(&dir, stem))
        .collect::<anyhow::Result<_>>()?;
    let stem_statuses: Vec<StemStatus<'_>> = statuses
        .iter()
        .map(|s| match s {
            Some(status) => StemStatus::Toml(status),
            None => StemStatus::MissingToml,
        })
        .collect();
    Ok(Some(fold_rollup(&stem_statuses)))
}

/// The minimal v1 phase-tracking skeleton (slice-004 §5.2): a phase-level
/// status plus an (initially empty) append-only progress log. Richer
/// per-criterion/task rows graduate to TOML when a consumer lands (D5/Q2).
/// Mutated later by
/// `set_phase_status` via `toml_edit` (comment/unknown-key preserving), so it
/// is rendered, never reserialised.
fn render_tracking(phase_id: &str) -> String {
    format!(
        "schema  = \"doctrine.phase.tracking\"\n\
         version = 1\n\
         phase   = \"{phase_id}\"\n\
         status  = \"planned\"   # planned | in_progress | completed | blocked\n\
         started      = \"\"\n\
         completed    = \"\"\n\
         last_updated = \"\"\n\
         \n\
         # Append-only runtime progress log, written by `doctrine slice phase`.\n"
    )
}

/// Render the disposable phase sheet from the embedded template.
fn render_phase_sheet(phase: &PlanPhase) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/phase.md")?
        .replace("{{phase_id}}", &phase.id)
        .replace("{{name}}", &phase.name)
        .replace("{{objective}}", &phase.objective))
}

/// Refresh the gitignored convenience symlink
/// `.doctrine/slice/<id>/phases → ../../state/slice/<id>/phases`. Verified
/// (a wrong link is replaced, a real file/dir errors); never authority.
fn refresh_symlink(project_root: &Path, slice_id: u32) -> anyhow::Result<()> {
    let name = format!("{slice_id:03}");
    let link = project_root.join(SLICE_DIR).join(&name).join("phases");
    let target = PathBuf::from(format!("../../state/slice/{name}/phases"));
    fsutil::set_symlink(&link, &target)
}

/// Edit-preserving status transition on one phase: set `status`, stamp
/// `last_updated` (and `started`/`completed` on first entry), and append a
/// `[[progress]]` row. Uses `toml_edit` so hand-added comments and unknown keys
/// survive (entity-model § Rust model) — the file is mutated, never
/// reserialised. The path is computed from the id (`phase_stem` validates it);
/// the convenience symlink is never followed. `now` is supplied by the shell
/// (the clock stays out of this layer).
// fn-level expect: the lone fs::write is the function's tail expression, which
// cannot carry a stmt-level attribute on stable Rust (stmt_expr_attributes).
#[expect(
    clippy::disallowed_methods,
    reason = "runtime phase sheet — disposable, atomicity not required"
)]
pub(crate) fn set_phase_status(
    project_root: &Path,
    slice_id: u32,
    phase_id: &str,
    status: PhaseStatus,
    note: Option<&str>,
    now: &str,
) -> anyhow::Result<()> {
    let stem = phase_stem(phase_id)?;
    let path = phases_dir(project_root, slice_id).join(format!("{stem}.toml"));
    let text = fs::read_to_string(&path).with_context(|| {
        format!(
            "Phase tracking not found at {} — run `doctrine slice phases` first",
            path.display()
        )
    })?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let table = doc.as_table_mut();
    table.insert("status", toml_edit::value(status.as_str()));
    table.insert("last_updated", toml_edit::value(now));
    if status == PhaseStatus::InProgress && table.get("started").and_then(Item::as_str) == Some("")
    {
        table.insert("started", toml_edit::value(now));
    }
    // `completed` holds a stamp iff the phase is completed: stamp once on first
    // completion (a re-complete keeps the original time), and clear it on any
    // non-completed status so a reopen leaves no stale completion time on an
    // in-progress/blocked phase (an otherwise internally-contradictory record).
    if status == PhaseStatus::Completed {
        if table.get("completed").and_then(Item::as_str) == Some("") {
            table.insert("completed", toml_edit::value(now));
        }
    } else {
        table.insert("completed", toml_edit::value(""));
    }

    // Solo phase-binding capture (SL-147 PHASE-04, design D5): a SEPARATE,
    // ADDITIVE, DEGRADING step that records the per-phase code boundary into the
    // arm-neutral registry. It NEVER alters the status-flip behaviour above — on
    // any git/bare-repo failure (or in a dispatch coordination context) it
    // degrades to a no-op with a NAMED warning and the transition still
    // completes. On InProgress it stamps `code_start_oid` (HEAD) into the sheet
    // once; on Completed it reads that back and records `(start, HEAD)` via the
    // F-6 guard + upsert. The stamp must ride THIS doc write, so it mutates the
    // table before the write; the registry record happens AFTER the write.
    let capture_end = capture_phase_boundary(project_root, slice_id, phase_id, status, table);

    let mut row = toml_edit::Table::new();
    row.insert("timestamp", toml_edit::value(now));
    row.insert("status", toml_edit::value(status.as_str()));
    if let Some(note) = note {
        row.insert("note", toml_edit::value(note));
    }
    table
        .entry("progress")
        .or_insert_with(|| Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .context("`progress` exists but is not an array of tables")?
        .push(row);

    fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))?;

    // The registry write is the tail of the DEGRADING capture: it runs only on a
    // completion that resolved a `(start, end)` pair, and its failure is reported
    // (named warning) but never returned — the status transition above already
    // succeeded and must stand.
    if let Some(CaptureCompletion { start, end }) = capture_end
        && let Err(e) = record_source_delta(
            project_root,
            slice_id,
            BoundaryRow {
                phase: phase_id.to_string(),
                code_start_oid: start,
                code_end_oid: end,
            },
        )
    {
        warn_capture(phase_id, &format!("recording source delta failed: {e:#}"));
    }

    Ok(())
}

/// The resolved completion boundary handed back by [`capture_phase_boundary`]
/// when a `Completed` transition has both a stamped start and a readable HEAD —
/// the input to the registry record that runs AFTER the sheet write.
struct CaptureCompletion {
    start: String,
    end: String,
}

/// Solo phase-binding capture step (SL-147 PHASE-04). Mutates `table` (the phase
/// sheet) to stamp `code_start_oid` on entry to `InProgress`, and returns the
/// `(start, end)` pair to record on `Completed`. Returns `None` (no registry
/// write) for every other status, when the arm guard fires (current branch ==
/// `dispatch/<slice_id>`), or when any git read degrades — emitting a NAMED
/// warning so the operator sees why no row landed, WITHOUT blocking the
/// transition. `project_root` doubles as the git cwd (the repo it lives in).
fn capture_phase_boundary(
    project_root: &Path,
    slice_id: u32,
    phase_id: &str,
    status: PhaseStatus,
    table: &mut toml_edit::Table,
) -> Option<CaptureCompletion> {
    if status != PhaseStatus::InProgress && status != PhaseStatus::Completed {
        return None;
    }

    // Arm guard: a dispatch coordination context records via the funnel recorder,
    // never the solo binding — they must NEVER both record a phase. Key on the
    // doctrine-owned branch, not "is a linked worktree" (a solo /worktree fork
    // must still capture) and not any host commit convention (POL-002).
    match crate::git::current_branch(project_root) {
        Ok(Some(branch)) if branch == format!("dispatch/{slice_id:03}") => return None,
        Ok(_) => {}
        Err(e) => {
            warn_capture(phase_id, &format!("branch probe failed: {e}"));
            return None;
        }
    }

    let head = match crate::git::resolve_ref(project_root, "HEAD") {
        Ok(Some(oid)) => oid,
        Ok(None) => {
            warn_capture(phase_id, "HEAD does not resolve (unborn/detached)");
            return None;
        }
        Err(e) => {
            warn_capture(phase_id, &format!("HEAD probe failed: {e}"));
            return None;
        }
    };

    if status == PhaseStatus::InProgress {
        // Stamp the start once: an in-progress re-entry (or a reopen → re-start)
        // keeps the original start so the boundary spans the whole phase.
        if table
            .get("code_start_oid")
            .and_then(Item::as_str)
            .is_none_or(str::is_empty)
        {
            table.insert("code_start_oid", toml_edit::value(&head));
        }
        return None;
    }

    // status == Completed: read the stamped start back. An absent start (the
    // phase was never flipped in_progress under the binding) degrades — without a
    // start there is no boundary to record.
    let Some(start) = table
        .get("code_start_oid")
        .and_then(Item::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
    else {
        warn_capture(
            phase_id,
            "no code_start_oid stamped (phase never entered in_progress under the binding) — no boundary recorded",
        );
        return None;
    };

    Some(CaptureCompletion { start, end: head })
}

/// Emit a single NAMED capture-degradation warning to stderr (the binding never
/// blocks a status transition — design D5). Routed through one helper so the
/// message shape is uniform and greppable.
fn warn_capture(phase_id: &str, detail: &str) {
    let _ignored = writeln!(
        std::io::stderr(),
        "warning: phase-binding capture skipped for {phase_id}: {detail}"
    );
}

// ---------------------------------------------------------------------------
// Recorded source-delta registry (SL-147 PHASE-02)
// ---------------------------------------------------------------------------
//
// The arm-neutral record of each phase's committed code boundary — the same
// per-phase `(code_start, code_end)` pair the claude arm writes into its
// committed dispatch ledger, but for ANY arm and persisted as gitignored
// runtime state. ONE file per slice, shared across every worktree of the repo:
// it resolves against the PRIMARY working tree (`crate::git::primary_worktree`),
// NOT `root::find(cwd)`, so a worker recording from a linked worktree writes the
// row a later integrator reads from the main tree.
//
// Tier: `.doctrine/state/slice/<NNN>/boundaries.toml` — disposable, never
// authored (mirrors `phases_dir`'s path idiom one level up). No funnel consumer
// is wired yet; PHASE-03 reads it (slice conformance) and PHASE-04 writes
// through it (record-delta / slice phase binding), so the symbols are
// dead-code-phased per-symbol until then.

/// The slice's recorded source-delta registry: a thin container over the
/// arm-neutral [`BoundaryRow`] (the row type is owned by the `boundary` leaf,
/// not re-declared here — this is a container, not a parallel row). Round-trips
/// through `boundaries.toml` with `[[boundary]]` table headers.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct SourceDeltas {
    #[serde(default, rename = "boundary")]
    pub rows: Vec<BoundaryRow>,
}

/// Canonical registry path for a slice, resolved against the PRIMARY working
/// tree so every worktree shares one file:
/// `<primary>/.doctrine/state/slice/<NNN>/boundaries.toml`. `cwd` may be any
/// path in the repo (a linked worker worktree is the typical caller). A
/// bare/not-a-repo `cwd` yields a clean named error via `primary_worktree`.
pub(crate) fn boundaries_path(cwd: &Path, slice_id: u32) -> anyhow::Result<PathBuf> {
    let primary = crate::git::primary_worktree(cwd)?;
    Ok(primary
        .join(STATE_SLICE_DIR)
        .join(format!("{slice_id:03}"))
        .join("boundaries.toml"))
}

/// Read the recorded source deltas for `slice_id` (empty Vec when the file is
/// absent or empty — never an error). Mirrors the `read_phase_status`
/// not-found-is-empty idiom; a present-but-malformed file is a hard error.
pub(crate) fn read_source_deltas(cwd: &Path, slice_id: u32) -> anyhow::Result<Vec<BoundaryRow>> {
    let path = boundaries_path(cwd, slice_id)?;
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", path.display())),
    };
    let registry: SourceDeltas =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(registry.rows)
}

/// Record (UPSERT by phase) one phase's committed source delta into the slice's
/// registry. The guard runs BEFORE the write: `code_start` must be an ancestor
/// of `code_end` (a real forward delta, possibly empty when equal) AND
/// `code_end` must be a non-merge commit (`parents().len() <= 1`) — the boundary
/// is a single linear code tip, never a merge. A bare/not-a-repo `cwd`, or a
/// `code_*` oid git cannot resolve, surfaces as a clean named error (the git
/// leaf's `CaptureError`), never a panic. Read-modify-write; the dir/file are
/// created on first write. `row.phase` keys the upsert so a re-record of the
/// same phase replaces (never duplicates) its row.
#[expect(
    clippy::disallowed_methods,
    reason = "runtime registry — disposable gitignored state, atomicity not required"
)]
pub(crate) fn record_source_delta(
    cwd: &Path,
    slice_id: u32,
    row: BoundaryRow,
) -> anyhow::Result<()> {
    if !crate::git::is_ancestor(cwd, &row.code_start_oid, &row.code_end_oid)? {
        anyhow::bail!(
            "record_source_delta: code_start {} is not an ancestor of code_end {} (not a forward delta)",
            row.code_start_oid,
            row.code_end_oid
        );
    }
    if crate::git::parents(cwd, &row.code_end_oid)?.len() > 1 {
        anyhow::bail!(
            "record_source_delta: code_end {} is a merge commit (boundary must be a non-merge code tip)",
            row.code_end_oid
        );
    }

    let path = boundaries_path(cwd, slice_id)?;
    let mut registry: SourceDeltas = match fs::read_to_string(&path) {
        Ok(text) => {
            toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?
        }
        Err(e) if e.kind() == ErrorKind::NotFound => SourceDeltas::default(),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", path.display())),
    };

    match registry.rows.iter_mut().find(|r| r.phase == row.phase) {
        Some(existing) => *existing = row,
        None => registry.rows.push(row),
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let body = toml::to_string(&registry).context("serialize source-delta registry")?;
    fs::write(&path, body).with_context(|| format!("Failed to write {}", path.display()))
}

/// The completeness verdict of the recorded registry against a slice's completed
/// phases (design F-2): either every completed phase has exactly one row (and no
/// row belongs to a non-completed phase), or the registry is incomplete with a
/// named gap. `slice conformance` refuses to emit a clean diff when this returns
/// `Incomplete` — partial coverage must never read as conformance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Completeness {
    /// One row per completed phase, no extras — the registry covers the work.
    Complete,
    /// A coverage gap, naming the offending phase(s) so the operator can act.
    Incomplete { gaps: Vec<CompletenessGap> },
}

/// A single named coverage gap between recorded rows and completed phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompletenessGap {
    /// A completed phase with no recorded boundary row.
    Missing { phase: String },
    /// A recorded row whose phase is not completed (or does not exist).
    Extra { phase: String },
    /// More than one recorded row for the same phase.
    Duplicate { phase: String },
}

impl CompletenessGap {
    /// One-line, phase-naming description for the `incomplete` render.
    pub(crate) fn describe(&self) -> String {
        match self {
            CompletenessGap::Missing { phase } => {
                format!("completed phase {phase} has no recorded source-delta row")
            }
            CompletenessGap::Extra { phase } => {
                format!("recorded row for {phase}, which is not a completed phase")
            }
            CompletenessGap::Duplicate { phase } => {
                format!("recorded more than one row for phase {phase}")
            }
        }
    }
}

/// Pure cross-check (design F-2): every completed phase id MUST have exactly one
/// recorded row, and every recorded row's phase MUST be completed. `completed`
/// is the set of completed `PHASE-NN` ids; `recorded` is the ordered list of
/// `row.phase` strings (duplicates surfaced). Zero-delta rows (start==end) are
/// not special-cased here — a recorded row is a recorded row; the writer's guard
/// owns range validity. Gaps are reported in a stable order (missing, extra,
/// duplicate) so the message is deterministic. No IO — the reads are the caller's.
pub(crate) fn check_completeness(
    completed: &BTreeSet<String>,
    recorded: &[String],
) -> Completeness {
    let mut counts: std::collections::BTreeMap<&str, u32> = std::collections::BTreeMap::new();
    for phase in recorded {
        *counts.entry(phase.as_str()).or_insert(0) += 1;
    }

    let mut gaps = Vec::new();
    for phase in completed {
        if !counts.contains_key(phase.as_str()) {
            gaps.push(CompletenessGap::Missing {
                phase: phase.clone(),
            });
        }
    }
    for (phase, count) in &counts {
        if !completed.contains(*phase) {
            gaps.push(CompletenessGap::Extra {
                phase: (*phase).to_string(),
            });
        } else if *count > 1 {
            gaps.push(CompletenessGap::Duplicate {
                phase: (*phase).to_string(),
            });
        }
    }

    if gaps.is_empty() {
        Completeness::Complete
    } else {
        Completeness::Incomplete { gaps }
    }
}

/// The set of completed `PHASE-NN` ids for a slice, read from its runtime phase
/// sheets (the same id-derived tree `phase_rollup` reads — the `phases` symlink
/// is never followed). A `.md`-only crash-partial or unreadable `.toml` is not
/// "completed", so it is excluded (and will surface as a `MissingRow` gap if a
/// row was nonetheless recorded — fail-closed). Empty when no phase exists.
pub(crate) fn completed_phase_ids(
    project_root: &Path,
    slice_id: u32,
) -> anyhow::Result<BTreeSet<String>> {
    let dir = phases_dir(project_root, slice_id);
    let mut completed = BTreeSet::new();
    for stem in existing_phase_stems(&dir)? {
        if let Some(status) = read_phase_status(&dir, &stem)?
            && PhaseStatus::from_str(&status, false) == Ok(PhaseStatus::Completed)
        {
            completed.insert(phase_id_from_stem(&stem));
        }
    }
    Ok(completed)
}

/// IO wrapper over [`check_completeness`] (design F-2): reads the recorded rows
/// (from the primary-tree registry, via `cwd`) and the completed phase ids (from
/// the `project_root` state tree), then cross-checks them. The conformance shell
/// invokes ONLY this — no phase-sheet reading leaks into the command/algebra
/// layers. `cwd` resolves the shared registry; `project_root` is the local state
/// tree (they coincide in the primary worktree).
pub(crate) fn registry_completeness(
    cwd: &Path,
    project_root: &Path,
    slice_id: u32,
) -> anyhow::Result<Completeness> {
    let recorded: Vec<String> = read_source_deltas(cwd, slice_id)?
        .into_iter()
        .map(|row| row.phase)
        .collect();
    let completed = completed_phase_ids(project_root, slice_id)?;
    Ok(check_completeness(&completed, &recorded))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(ids: &[&str]) -> Plan {
        Plan {
            phases: ids
                .iter()
                .map(|id| PlanPhase {
                    id: (*id).to_string(),
                    name: String::new(),
                    objective: String::new(),
                })
                .collect(),
        }
    }

    /// A slice dir must exist for the symlink refresh to land.
    fn make_slice_dir(root: &Path, slice_id: u32) {
        fs::create_dir_all(root.join(SLICE_DIR).join(format!("{slice_id:03}"))).unwrap();
    }

    // --- phase_stem ---

    #[test]
    fn phase_stem_derives_and_validates() {
        assert_eq!(phase_stem("PHASE-01").unwrap(), "phase-01");
        assert_eq!(phase_stem("PHASE-137").unwrap(), "phase-137");
    }

    #[test]
    fn phase_stem_rejects_malformed_ids() {
        for bad in [
            "",
            "PHASE-",
            "phase-01",
            "PHASE-1a",
            "PHASE-../x",
            "PHASE-0/1",
            ".PHASE-01",
            "P01",
        ] {
            assert!(phase_stem(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    // --- init_phases ---

    #[test]
    fn init_phases_materialises_each_declared_phase() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);

        let report = init_phases(root, 4, &plan(&["PHASE-01", "PHASE-02"]), false).unwrap();
        assert_eq!(report.created, vec!["PHASE-01", "PHASE-02"]);

        let phases = phases_dir(root, 4);
        for stem in ["phase-01", "phase-02"] {
            assert!(phases.join(format!("{stem}.toml")).is_file());
            assert!(phases.join(format!("{stem}.md")).is_file());
        }
        let tracking = fs::read_to_string(phases.join("phase-01.toml")).unwrap();
        assert!(tracking.contains("phase   = \"PHASE-01\""));
        assert!(tracking.contains("status  = \"planned\""));

        // writes land ONLY under .doctrine/state (the boundary invariant);
        // nothing tracking-shaped under the authored slice dir.
        assert!(!root.join(SLICE_DIR).join("004/phase-01.toml").exists());
    }

    #[test]
    fn init_phases_is_idempotent_and_preserves_edits() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);
        let p = plan(&["PHASE-01"]);

        init_phases(root, 4, &p, false).unwrap();
        // hand-edit the sheet
        let sheet = phases_dir(root, 4).join("phase-01.md");
        fs::write(&sheet, "EDITED").unwrap();

        let report = init_phases(root, 4, &p, false).unwrap();
        assert!(
            report.created.is_empty(),
            "no re-creation on the no-drift path"
        );
        assert_eq!(
            fs::read_to_string(&sheet).unwrap(),
            "EDITED",
            "edits survive"
        );
    }

    #[test]
    fn init_phases_completes_a_crash_partial_phase() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);
        let phases = phases_dir(root, 4);
        fs::create_dir_all(&phases).unwrap();
        // simulate a crash after the .toml but before the .md
        fs::write(phases.join("phase-01.toml"), "partial").unwrap();

        let report = init_phases(root, 4, &plan(&["PHASE-01"]), false).unwrap();
        assert_eq!(
            report.created,
            vec!["PHASE-01"],
            "the missing file completes the phase"
        );
        assert!(phases.join("phase-01.md").is_file());
        // the pre-existing .toml is left as-is (per-file skip)
        assert_eq!(
            fs::read_to_string(phases.join("phase-01.toml")).unwrap(),
            "partial"
        );
    }

    #[test]
    fn init_phases_reports_orphans_without_removing_them() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);

        // start with two phases, then rename PHASE-02 → PHASE-03 in the plan
        init_phases(root, 4, &plan(&["PHASE-01", "PHASE-02"]), false).unwrap();
        let report = init_phases(root, 4, &plan(&["PHASE-01", "PHASE-03"]), false).unwrap();

        assert_eq!(report.created, vec!["PHASE-03"]);
        assert_eq!(report.orphan, vec!["PHASE-02"]);
        assert!(report.pruned.is_empty());
        // orphan tracking is NOT silently removed
        assert!(phases_dir(root, 4).join("phase-02.toml").is_file());
    }

    #[test]
    fn init_phases_rejects_a_malformed_id_before_writing_anything() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);

        let err = init_phases(root, 4, &plan(&["PHASE-01", "bad", "PHASE-03"]), false).unwrap_err();
        assert!(err.to_string().contains("must match PHASE-<digits>"));
        // the valid earlier phase did not leak — nothing was materialised
        assert!(!phases_dir(root, 4).join("phase-01.toml").exists());
    }

    #[test]
    fn init_phases_prunes_orphans_only_when_asked() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);

        init_phases(root, 4, &plan(&["PHASE-01", "PHASE-02"]), false).unwrap();
        let report = init_phases(root, 4, &plan(&["PHASE-01"]), true).unwrap();

        assert_eq!(report.pruned, vec!["PHASE-02"]);
        assert!(report.orphan.is_empty());
        let phases = phases_dir(root, 4);
        assert!(!phases.join("phase-02.toml").exists());
        assert!(!phases.join("phase-02.md").exists());
        assert!(phases.join("phase-01.toml").is_file());
    }

    #[test]
    fn init_phases_refreshes_the_convenience_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);

        init_phases(root, 4, &plan(&["PHASE-01"]), false).unwrap();
        let link = root.join(SLICE_DIR).join("004/phases");
        // the link must *resolve* to the canonical state dir — not merely string-
        // match a hand-written relative target, which would pass even if the two
        // sides of the convention drifted apart.
        assert_eq!(
            fs::canonicalize(&link).unwrap(),
            fs::canonicalize(phases_dir(root, 4)).unwrap(),
        );
    }

    #[test]
    fn init_phases_creates_the_state_parent_on_demand() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);
        assert!(!root.join(STATE_SLICE_DIR).exists());

        init_phases(root, 4, &plan(&["PHASE-01"]), false).unwrap();
        assert!(phases_dir(root, 4).is_dir());
    }

    // --- set_phase_status ---

    fn init_one(root: &Path) {
        make_slice_dir(root, 4);
        init_phases(root, 4, &plan(&["PHASE-01"]), false).unwrap();
    }

    #[test]
    fn set_phase_status_sets_fields_and_appends_progress() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        init_one(root);

        set_phase_status(
            root,
            4,
            "PHASE-01",
            PhaseStatus::InProgress,
            Some("kickoff"),
            "2026-06-04T10:00:00Z",
        )
        .unwrap();
        let doc = fs::read_to_string(phases_dir(root, 4).join("phase-01.toml"))
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();

        assert_eq!(doc["status"].as_str(), Some("in_progress"));
        assert_eq!(doc["started"].as_str(), Some("2026-06-04T10:00:00Z"));
        assert_eq!(doc["last_updated"].as_str(), Some("2026-06-04T10:00:00Z"));
        assert_eq!(doc["completed"].as_str(), Some("")); // not yet
        let progress = doc["progress"].as_array_of_tables().unwrap();
        assert_eq!(progress.len(), 1);
        let row = progress.get(0).unwrap();
        assert_eq!(row["status"].as_str(), Some("in_progress"));
        assert_eq!(row["note"].as_str(), Some("kickoff"));
    }

    #[test]
    fn set_phase_status_stamps_started_once_and_completed_on_completion() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        init_one(root);

        set_phase_status(root, 4, "PHASE-01", PhaseStatus::InProgress, None, "T1").unwrap();
        set_phase_status(root, 4, "PHASE-01", PhaseStatus::InProgress, None, "T2").unwrap();
        set_phase_status(root, 4, "PHASE-01", PhaseStatus::Completed, None, "T3").unwrap();

        let doc = fs::read_to_string(phases_dir(root, 4).join("phase-01.toml"))
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert_eq!(
            doc["started"].as_str(),
            Some("T1"),
            "started stamped once, not overwritten"
        );
        assert_eq!(doc["completed"].as_str(), Some("T3"));
        assert_eq!(doc["status"].as_str(), Some("completed"));
        assert_eq!(doc["progress"].as_array_of_tables().unwrap().len(), 3);
    }

    #[test]
    fn set_phase_status_clears_completed_on_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        init_one(root);

        set_phase_status(root, 4, "PHASE-01", PhaseStatus::Completed, None, "T1").unwrap();
        set_phase_status(root, 4, "PHASE-01", PhaseStatus::InProgress, None, "T2").unwrap();

        let doc = fs::read_to_string(phases_dir(root, 4).join("phase-01.toml"))
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert_eq!(doc["status"].as_str(), Some("in_progress"));
        assert_eq!(
            doc["completed"].as_str(),
            Some(""),
            "reopen clears the stale completion stamp"
        );
    }

    #[test]
    fn set_phase_status_preserves_comments_and_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        init_one(root);
        let path = phases_dir(root, 4).join("phase-01.toml");
        // hand-edit: a comment and an unknown key the tool does not model
        let edited = format!(
            "{}\n# a hand-written note\nowner = \"alice\"\n",
            fs::read_to_string(&path).unwrap()
        );
        fs::write(&path, edited).unwrap();

        set_phase_status(root, 4, "PHASE-01", PhaseStatus::Blocked, None, "T1").unwrap();

        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# a hand-written note"), "comment survives");
        assert!(after.contains("owner = \"alice\""), "unknown key survives");
        assert!(after.contains("status  = \"blocked\"") || after.contains("status = \"blocked\""));
    }

    #[test]
    fn set_phase_status_resolves_by_id_blind_to_the_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        init_one(root);
        // remove the convenience symlink entirely
        fs::remove_file(root.join(SLICE_DIR).join("004/phases")).unwrap();

        set_phase_status(root, 4, "PHASE-01", PhaseStatus::Completed, None, "T1").unwrap();
        let doc = fs::read_to_string(phases_dir(root, 4).join("phase-01.toml"))
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert_eq!(doc["status"].as_str(), Some("completed"));
    }

    #[test]
    fn set_phase_status_errors_when_tracking_absent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_slice_dir(root, 4);
        let err =
            set_phase_status(root, 4, "PHASE-99", PhaseStatus::InProgress, None, "T1").unwrap_err();
        assert!(err.to_string().contains("Phase tracking not found"));
    }

    // --- fold_rollup (pure) ---

    #[test]
    fn fold_rollup_empty_is_all_zero() {
        assert_eq!(fold_rollup(&[]), PhaseRollup::default());
    }

    #[test]
    fn fold_rollup_counts_each_known_status_into_its_bucket() {
        let stems = [
            StemStatus::Toml("completed"),
            StemStatus::Toml("completed"),
            StemStatus::Toml("in_progress"),
            StemStatus::Toml("planned"),
            StemStatus::Toml("blocked"),
        ];
        let r = fold_rollup(&stems);
        assert_eq!(
            r,
            PhaseRollup {
                planned: 1,
                in_progress: 1,
                completed: 2,
                blocked: 1,
                unknown: 0,
                missing_toml: 0,
            }
        );
    }

    #[test]
    fn fold_rollup_buckets_unknown_status_and_missing_toml_separately() {
        let stems = [
            StemStatus::Toml("completed"),
            StemStatus::Toml("garbage"), // typo / outside the enum
            StemStatus::MissingToml,     // .md-only crash-partial
        ];
        let r = fold_rollup(&stems);
        assert_eq!(r.completed, 1);
        assert_eq!(r.unknown, 1);
        assert_eq!(r.missing_toml, 1);
        // every stem is counted — nothing silently dropped (R-F4)
        let counted =
            r.planned + r.in_progress + r.completed + r.blocked + r.unknown + r.missing_toml;
        assert_eq!(counted, 3);
    }

    // --- phase_rollup (IO) ---

    fn write_phase_toml(dir: &Path, stem: &str, status: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join(format!("{stem}.toml")),
            format!("status = \"{status}\"\n"),
        )
        .unwrap();
    }

    #[test]
    fn phase_rollup_is_none_when_untracked() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // no state tree at all
        assert_eq!(phase_rollup(root, 9).unwrap(), None);
        // dir exists but holds no phase files
        fs::create_dir_all(phases_dir(root, 9)).unwrap();
        assert_eq!(phase_rollup(root, 9).unwrap(), None);
    }

    #[test]
    fn phase_rollup_counts_materialised_phases() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let phases = phases_dir(root, 9);
        write_phase_toml(&phases, "phase-01", "completed");
        write_phase_toml(&phases, "phase-02", "completed");
        write_phase_toml(&phases, "phase-03", "in_progress");

        let r = phase_rollup(root, 9).unwrap().unwrap();
        assert_eq!(r.completed, 2);
        assert_eq!(r.in_progress, 1);
    }

    #[test]
    fn phase_rollup_md_only_stem_is_missing_toml_not_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let phases = phases_dir(root, 9);
        write_phase_toml(&phases, "phase-01", "completed");
        // phase-02 has only its .md sheet (crash-partial) — still a phase
        fs::write(phases.join("phase-02.md"), "# sheet\n").unwrap();

        let r = phase_rollup(root, 9).unwrap().unwrap();
        assert_eq!(r.completed, 1);
        assert_eq!(r.missing_toml, 1);
    }

    #[test]
    fn phase_rollup_unparseable_or_typo_status_is_surfaced() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let phases = phases_dir(root, 9);
        write_phase_toml(&phases, "phase-01", "donezo"); // typo → unknown
        fs::write(phases.join("phase-02.toml"), "this is not = valid toml\n").unwrap();

        let r = phase_rollup(root, 9).unwrap().unwrap();
        assert_eq!(r.unknown, 1, "typo status → unknown");
        assert_eq!(r.missing_toml, 1, "unparseable .toml → missing_toml");
    }

    // --- recorded source-delta registry (SL-147 PHASE-02) ------------------

    /// Run git in `dir` with a pinned identity; panic on failure, return stdout.
    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "Doctrine Test")
            .env("GIT_AUTHOR_EMAIL", "test@doctrine.invalid")
            .env("GIT_COMMITTER_NAME", "Doctrine Test")
            .env("GIT_COMMITTER_EMAIL", "test@doctrine.invalid")
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A fresh repo with one commit on `main`; returns its canonical path.
    fn init_repo(dir: &Path) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        git(dir, &["init", "-q", "-b", "main"]);
        git(dir, &["commit", "-q", "--allow-empty", "-m", "root"]);
        fs::canonicalize(dir).unwrap()
    }

    fn row(phase: &str, start: &str, end: &str) -> BoundaryRow {
        BoundaryRow {
            phase: phase.into(),
            code_start_oid: start.into(),
            code_end_oid: end.into(),
        }
    }

    // VT-2: round-trip — write rows then read them back identically.
    #[test]
    fn source_deltas_round_trip_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        let a = git(&repo, &["rev-parse", "HEAD"]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "two"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);

        // Absent file reads as empty, not an error.
        assert!(read_source_deltas(&repo, 147).unwrap().is_empty());

        record_source_delta(&repo, 147, row("PHASE-01", &a, &a)).unwrap();
        record_source_delta(&repo, 147, row("PHASE-02", &a, &head)).unwrap();

        let rows = read_source_deltas(&repo, 147).unwrap();
        assert_eq!(
            rows,
            vec![row("PHASE-01", &a, &a), row("PHASE-02", &a, &head)]
        );
    }

    // VT-2: the file lands under the slice's runtime tree with `[[boundary]]`.
    #[test]
    fn source_deltas_path_is_slice_scoped_runtime_state() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        let head = git(&repo, &["rev-parse", "HEAD"]);
        record_source_delta(&repo, 147, row("PHASE-01", &head, &head)).unwrap();

        let path = boundaries_path(&repo, 147).unwrap();
        assert!(
            path.ends_with(".doctrine/state/slice/147/boundaries.toml"),
            "{path:?}"
        );
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("[[boundary]]"), "table header: {text}");
        assert!(text.contains("phase = \"PHASE-01\""), "{text}");
    }

    // VT-1: a re-record of the same phase UPSERTs (replaces) rather than dupes.
    #[test]
    fn record_upserts_by_phase() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        let a = git(&repo, &["rev-parse", "HEAD"]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "two"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);

        record_source_delta(&repo, 147, row("PHASE-01", &a, &a)).unwrap();
        record_source_delta(&repo, 147, row("PHASE-01", &a, &head)).unwrap();

        let rows = read_source_deltas(&repo, 147).unwrap();
        assert_eq!(rows, vec![row("PHASE-01", &a, &head)], "one row, replaced");
    }

    // VT-2 resolver: a record from a LINKED worktree lands the row in the
    // PRIMARY tree's file (the cross-worktree shared-file contract).
    #[test]
    fn record_from_linked_worktree_targets_primary_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("primary"));
        let head = git(&primary, &["rev-parse", "HEAD"]);
        let fork = tmp.path().join("fork");
        git(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "feat",
                fork.to_str().unwrap(),
            ],
        );
        let fork = fs::canonicalize(&fork).unwrap();

        // Record from the LINKED worktree.
        record_source_delta(&fork, 147, row("PHASE-01", &head, &head)).unwrap();

        // The file exists under the PRIMARY tree, NOT the fork.
        assert!(
            primary
                .join(".doctrine/state/slice/147/boundaries.toml")
                .exists()
        );
        assert!(
            !fork
                .join(".doctrine/state/slice/147/boundaries.toml")
                .exists()
        );
        // And reads back from either worktree (same resolved path).
        assert_eq!(
            read_source_deltas(&fork, 147).unwrap(),
            vec![row("PHASE-01", &head, &head)]
        );
        assert_eq!(
            read_source_deltas(&primary, 147).unwrap(),
            vec![row("PHASE-01", &head, &head)]
        );
    }

    // VT-1 guard: a non-ancestor (start NOT before end) range is rejected.
    #[test]
    fn guard_rejects_non_ancestor_range() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        let a = git(&repo, &["rev-parse", "HEAD"]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "two"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);

        // start = later commit, end = earlier → not an ancestor.
        let err = record_source_delta(&repo, 147, row("PHASE-01", &head, &a)).unwrap_err();
        assert!(format!("{err:#}").contains("not an ancestor"), "{err:#}");
        // Nothing was persisted.
        assert!(read_source_deltas(&repo, 147).unwrap().is_empty());
    }

    // VT-1 guard: a merge commit as code_end is rejected.
    #[test]
    fn guard_rejects_merge_code_end() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        let base = git(&repo, &["rev-parse", "HEAD"]);
        // Branch A.
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "a"]);
        let a = git(&repo, &["rev-parse", "HEAD"]);
        // Branch B off base.
        git(&repo, &["checkout", "-q", "-b", "side", &base]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "b"]);
        // Merge B into A → a 2-parent merge tip.
        git(&repo, &["checkout", "-q", "main"]);
        git(&repo, &["merge", "-q", "--no-ff", "--no-edit", "side"]);
        let merge = git(&repo, &["rev-parse", "HEAD"]);
        assert_eq!(crate::git::parents(&repo, &merge).unwrap().len(), 2);

        let err = record_source_delta(&repo, 147, row("PHASE-01", &a, &merge)).unwrap_err();
        assert!(format!("{err:#}").contains("merge commit"), "{err:#}");
        assert!(read_source_deltas(&repo, 147).unwrap().is_empty());
    }

    // VT-1 guard: a valid ancestor + non-merge pair is accepted and persisted.
    #[test]
    fn guard_accepts_valid_ancestor_non_merge_pair() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        let a = git(&repo, &["rev-parse", "HEAD"]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "two"]);
        let head = git(&repo, &["rev-parse", "HEAD"]);
        assert!(crate::git::parents(&repo, &head).unwrap().len() <= 1);

        record_source_delta(&repo, 147, row("PHASE-01", &a, &head)).unwrap();
        assert_eq!(
            read_source_deltas(&repo, 147).unwrap(),
            vec![row("PHASE-01", &a, &head)]
        );
    }

    // VT-2: a not-a-repo cwd surfaces a clean named error, never a panic.
    #[test]
    fn record_in_non_repo_is_a_named_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = record_source_delta(tmp.path(), 147, row("PHASE-01", "x", "y")).unwrap_err();
        let msg = format!("{err:#}");
        assert!(!msg.is_empty(), "named error, not a panic: {msg}");
    }

    // --- solo phase-binding capture (SL-147 PHASE-04, T1) ------------------

    /// A phase sheet under a repo's runtime tree, ready for `set_phase_status`.
    fn seed_phase_sheet(repo: &Path, slice: u32, stem: &str) {
        let dir = phases_dir(repo, slice);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{stem}.toml")),
            "status = \"planned\"\nstarted = \"\"\ncompleted = \"\"\n",
        )
        .unwrap();
    }

    // VT-1: in_progress → completed records the boundary deterministically; a
    // re-record (reopen → re-complete) upserts to the new tip.
    #[test]
    fn binding_records_boundary_on_completion_and_upserts_on_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        seed_phase_sheet(&repo, 147, "phase-01");

        let start = git(&repo, &["rev-parse", "HEAD"]);
        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::InProgress, None, "T1").unwrap();
        // Land a code commit, then complete: the row spans start → new HEAD.
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "code"]);
        let end1 = git(&repo, &["rev-parse", "HEAD"]);
        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::Completed, None, "T2").unwrap();

        assert_eq!(
            read_source_deltas(&repo, 147).unwrap(),
            vec![row("PHASE-01", &start, &end1)],
            "one boundary spanning the phase"
        );

        // Reopen + re-complete after another commit → the SAME phase upserts, the
        // start is preserved (stamped once), the end advances.
        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::InProgress, None, "T3").unwrap();
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "more"]);
        let end2 = git(&repo, &["rev-parse", "HEAD"]);
        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::Completed, None, "T4").unwrap();

        assert_eq!(
            read_source_deltas(&repo, 147).unwrap(),
            vec![row("PHASE-01", &start, &end2)],
            "upsert: one row, start preserved, end advanced"
        );
    }

    // VT-1: a no-commit phase (no code landed between start and completion) records
    // a zero-delta row (start == end), never an error.
    #[test]
    fn binding_records_zero_delta_for_a_no_commit_phase() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        seed_phase_sheet(&repo, 147, "phase-01");
        let head = git(&repo, &["rev-parse", "HEAD"]);

        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::InProgress, None, "T1").unwrap();
        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::Completed, None, "T2").unwrap();

        assert_eq!(
            read_source_deltas(&repo, 147).unwrap(),
            vec![row("PHASE-01", &head, &head)],
            "zero-delta row, start == end"
        );
    }

    // VT-1: a git-unavailable transition (non-repo cwd) DEGRADES — no row, no
    // panic, and the status flip still COMPLETES.
    #[test]
    fn binding_degrades_without_blocking_when_git_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path(); // not a git repo
        init_one(root); // seeds slice 4 / PHASE-01 sheet via init_phases

        // Transition succeeds despite no resolvable HEAD.
        set_phase_status(root, 4, "PHASE-01", PhaseStatus::InProgress, None, "T1").unwrap();
        set_phase_status(root, 4, "PHASE-01", PhaseStatus::Completed, None, "T2").unwrap();

        let doc = fs::read_to_string(phases_dir(root, 4).join("phase-01.toml"))
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert_eq!(
            doc["status"].as_str(),
            Some("completed"),
            "flip still landed"
        );
        // No registry written (boundaries_path itself errors in a non-repo, so the
        // read returns a named error — the point is the transition above succeeded).
        assert!(read_source_deltas(root, 4).is_err());
    }

    // VT-2: arm mutual-exclusion — a completion in a SIMULATED dispatch context
    // (current branch `dispatch/<NNN>`) records ZERO rows from the binding; a
    // solo-context completion records exactly one.
    #[test]
    fn binding_skips_capture_in_a_dispatch_context() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        seed_phase_sheet(&repo, 147, "phase-01");
        // Put the repo on the doctrine-owned coordination branch for slice 147.
        git(&repo, &["checkout", "-q", "-b", "dispatch/147"]);

        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::InProgress, None, "T1").unwrap();
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "code"]);
        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::Completed, None, "T2").unwrap();

        assert!(
            read_source_deltas(&repo, 147).unwrap().is_empty(),
            "dispatch context: the binding records nothing (the funnel recorder owns it)"
        );
    }

    // VT-2: a solo-context completion (NOT on `dispatch/<NNN>`) records exactly one
    // — the mutual-exclusion's positive arm. A solo `/worktree` fork is on its own
    // feature branch, never the coordination branch, so it still captures.
    #[test]
    fn binding_captures_on_a_solo_feature_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        seed_phase_sheet(&repo, 147, "phase-01");
        // A solo fork's own branch — NOT dispatch/147.
        git(&repo, &["checkout", "-q", "-b", "feat/solo-work"]);
        let start = git(&repo, &["rev-parse", "HEAD"]);

        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::InProgress, None, "T1").unwrap();
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "code"]);
        let end = git(&repo, &["rev-parse", "HEAD"]);
        set_phase_status(&repo, 147, "PHASE-01", PhaseStatus::Completed, None, "T2").unwrap();

        assert_eq!(
            read_source_deltas(&repo, 147).unwrap(),
            vec![row("PHASE-01", &start, &end)],
            "solo context: exactly one boundary"
        );
    }

    // --- completeness check (SL-147 PHASE-03, design F-2) -------------------

    fn completed_set(ids: &[&str]) -> BTreeSet<String> {
        ids.iter().map(|s| (*s).to_string()).collect()
    }

    // VT-2: one row per completed phase, no extras → Complete.
    #[test]
    fn completeness_is_complete_when_rows_cover_completed_phases() {
        let completed = completed_set(&["PHASE-01", "PHASE-02"]);
        let recorded = vec!["PHASE-01".to_string(), "PHASE-02".to_string()];
        assert_eq!(
            check_completeness(&completed, &recorded),
            Completeness::Complete
        );
    }

    // VT-2: a completed phase with no recorded row → Incomplete, naming it.
    #[test]
    fn completeness_flags_a_missing_row_for_a_completed_phase() {
        let completed = completed_set(&["PHASE-01", "PHASE-02"]);
        let recorded = vec!["PHASE-01".to_string()];
        let Completeness::Incomplete { gaps } = check_completeness(&completed, &recorded) else {
            panic!("expected incomplete");
        };
        assert_eq!(
            gaps,
            vec![CompletenessGap::Missing {
                phase: "PHASE-02".to_string()
            }]
        );
        assert!(gaps[0].describe().contains("PHASE-02"));
    }

    // VT-2: a recorded row whose phase is not completed → Incomplete (ExtraRow).
    #[test]
    fn completeness_flags_an_extra_row() {
        let completed = completed_set(&["PHASE-01"]);
        let recorded = vec!["PHASE-01".to_string(), "PHASE-09".to_string()];
        let Completeness::Incomplete { gaps } = check_completeness(&completed, &recorded) else {
            panic!("expected incomplete");
        };
        assert_eq!(
            gaps,
            vec![CompletenessGap::Extra {
                phase: "PHASE-09".to_string()
            }]
        );
    }

    // VT-2: more than one row for the same completed phase → DuplicateRow.
    #[test]
    fn completeness_flags_a_duplicate_row() {
        let completed = completed_set(&["PHASE-01"]);
        let recorded = vec!["PHASE-01".to_string(), "PHASE-01".to_string()];
        assert_eq!(
            check_completeness(&completed, &recorded),
            Completeness::Incomplete {
                gaps: vec![CompletenessGap::Duplicate {
                    phase: "PHASE-01".to_string()
                }]
            }
        );
    }

    // VT-2: `completed_phase_ids` reads only `completed` phases from the sheets.
    #[test]
    fn completed_phase_ids_returns_only_completed_phases() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let phases = phases_dir(root, 147);
        write_phase_toml(&phases, "phase-01", "completed");
        write_phase_toml(&phases, "phase-02", "in_progress");
        write_phase_toml(&phases, "phase-03", "completed");

        let completed = completed_phase_ids(root, 147).unwrap();
        assert_eq!(completed, completed_set(&["PHASE-01", "PHASE-03"]));
    }

    // VT-2 (integration): registry_completeness composes the registry read +
    // phase-sheet read — a completed phase missing its row fails closed.
    #[test]
    fn registry_completeness_fails_closed_on_unrecorded_completed_phase() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("repo"));
        let head = git(&repo, &["rev-parse", "HEAD"]);

        let phases = phases_dir(&repo, 147);
        write_phase_toml(&phases, "phase-01", "completed");
        write_phase_toml(&phases, "phase-02", "completed");
        record_source_delta(&repo, 147, row("PHASE-01", &head, &head)).unwrap();

        let verdict = registry_completeness(&repo, &repo, 147).unwrap();
        assert_eq!(
            verdict,
            Completeness::Incomplete {
                gaps: vec![CompletenessGap::Missing {
                    phase: "PHASE-02".to_string()
                }]
            }
        );
    }
}
