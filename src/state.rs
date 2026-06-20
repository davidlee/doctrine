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

    fs::write(&path, doc.to_string()).with_context(|| format!("Failed to write {}", path.display()))
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
}
