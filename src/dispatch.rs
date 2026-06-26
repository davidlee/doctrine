// SPDX-License-Identifier: GPL-3.0-only
//! SL-064 PHASE-04 — the `dispatch sync` projection verb (stage-1
//! `--prepare-review`). Materialises the reviewable refs from the coordination
//! branch `dispatch/<slice>` **without writing trunk**:
//!
//! * **B** — `review/<slice>`: a single squashed, filtered projection of the
//!   `dispatch/<slice>` tip, parented to the trunk base, excluding the run-ledger
//!   dir and every journal-verified orthogonal path (design §4.2).
//! * **C** — `phase/<slice>-NN`: the claude-arm per-phase cut synthesised from
//!   `boundaries.toml`, code-only (`.doctrine/` stripped), empty-code phases
//!   skipped, chained so each diff is exactly that phase's code delta (§4.3).
//!
//! The CAS journal is committed onto `dispatch/<slice>` (plumbing-only, no
//! checkout) **before** any external ref mutation (EX-2, ADR-012 D4); external
//! refs are created via zero-oid CAS so a crashed prior run's stale `review/*` /
//! `phase/*` is reported, never clobbered (EX-5). Trunk and `edge` are never
//! touched — that is stage-2 `--integrate` (PHASE-05).

use std::collections::BTreeSet;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use clap::Subcommand;

use crate::boundary::{BoundaryRow, Provenance};
use crate::git::{self, MergeTree, RefCas, ZERO_OID};
use crate::ledger::{
    Admission, Boundaries, CandidateKind, CandidatePayload, CandidateRole, CandidateRow,
    CandidateStatus, Candidates, Journal, JournalRow, LedgerStatus, Orthogonal, read_candidates,
};
use crate::listing::render_table;
use crate::root;

#[derive(Subcommand)]
pub(crate) enum DispatchCommand {
    /// Sync reviewable refs from the dispatch branch.
    /// Stage selector required; `--prepare-review` creates `review/<slice>` +
    /// `phase/<slice>-NN` under CAS (never writing trunk). `--integrate` replays
    /// the journal. Orchestrator-classed — refused under worker-mode.
    Sync {
        /// The slice id (bare number, e.g. `64`) whose `dispatch/<slice>`
        /// coordination branch to project.
        #[arg(long)]
        slice: u32,

        /// Stage-1: create the reviewable `review/<slice>` and `phase/<slice>-NN`
        /// refs from the dispatch tip; never writes trunk.
        #[arg(long, group = "stage", required = true)]
        prepare_review: bool,

        /// Stage-2: replay the prepared journal idempotently and project the
        /// audited code units (opt-in `--trunk`/`--edge`); runs from parent/root
        /// after the coordination worktree is removed. Never auto-resolves.
        #[arg(long, group = "stage", required = true)]
        integrate: bool,

        /// Read-only (SL-121 §3(b)): print the committed journal's trunk-row
        /// `planned_new_oid` — the row whose target is `--trunk` — to stdout and
        /// exit; the close step-3a verify read surface. Tree-reads `dispatch/<slice>`,
        /// writes nothing.
        #[arg(long, group = "stage", required = true)]
        show_journal_trunk_oid: bool,

        /// Project the cumulative code units onto this trunk ref, fast-forward-only +
        /// expected-tip CAS (e.g. `refs/heads/main`) under `--integrate`; names the
        /// row to read under `--show-journal-trunk-oid`. Absent under `--integrate` ⇒
        /// trunk is left untouched.
        #[arg(long, conflicts_with = "prepare_review")]
        trunk: Option<String>,

        /// Stage-2 only: advance this standing aggregate ref to the `review/<slice>`
        /// bundle (e.g. `refs/heads/edge`). Absent ⇒ no aggregate written.
        #[arg(long, requires = "integrate")]
        edge: Option<String>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Record a phase code boundary.
    /// Appends a per-phase boundary to `.doctrine/dispatch/<slice>/boundaries.toml`.
    /// Orchestrator-classed — refused under worker-mode.
    RecordBoundary {
        /// The slice id (bare number, e.g. `64`) whose ledger to append.
        #[arg(long)]
        slice: u32,

        /// The `PHASE-NN` id this code boundary belongs to.
        #[arg(long)]
        phase: String,

        /// Commit-ish for HEAD before the phase's code landed (resolved to a
        /// full oid; the empty-phase test compares it to `--code-end`).
        #[arg(long)]
        code_start: String,

        /// Commit-ish for the phase's cumulative code tip, *before* the knowledge
        /// record commit (resolved to a full oid — the tree the cut snapshots).
        #[arg(long)]
        code_end: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Refresh the coordination base from trunk.
    /// Merges current trunk into dispatch/<slice> in the live coordination
    /// worktree. Merge-only; re-run `sync --prepare-review` after.
    /// Orchestrator-classed — refused under worker-mode.
    RefreshBase {
        #[arg(long)]
        slice: u32,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create or resume dispatch coordination.
    /// Emits the dispatch env contract on stdout. Orchestrator-classed — refused
    /// under worker-mode.
    Setup {
        /// The slice id (bare number, e.g. `85`).
        #[arg(long)]
        slice: u32,

        /// The coordination worktree directory (must not already exist).
        #[arg(long)]
        dir: PathBuf,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Manage dispatch candidates.
    /// `create` publishes a reviewable/landable candidate at
    /// `candidate/<slice>/<label>`. Orchestrator-classed — refused under
    /// worker-mode.
    Candidate {
        #[command(subcommand)]
        command: CandidateCommand,
    },

    /// Plan the next actionable phase.
    /// Reads the plan and runtime phase sheets; prints ordered phase rollup.
    /// Read-only — callable from anywhere.
    PlanNext {
        /// The slice id (bare number).
        #[arg(long)]
        slice: u32,

        /// Emit JSON instead of human-readable table.
        #[arg(long)]
        json: bool,

        /// Explicit project root.
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show the dispatch rollup.
    /// Coordination state, phase table, trunk drift, sync state, candidate
    /// summary, next-step guidance.
    Status {
        /// The slice id (bare number, e.g. `85`).
        #[arg(long)]
        slice: u32,

        /// Emit JSON instead of human-readable table.
        #[arg(long)]
        json: bool,

        /// Explicit project root.
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the deliver-to ref.
    /// Resolved `[dispatch] deliver_to` trunk delivery ref. Read-only.
    DeliverTo {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Arm the next claude-arm worker spawn (SL-152 PHASE-03).
    /// Writes the coord tree's arming dir `.doctrine/state/dispatch/spawn/base`
    /// = `<sha>\n` (the ONLY thing it carries) and prints the dir's absolute path,
    /// so the orchestrator `cd`s into it before the Agent spawn — the cwd, not the
    /// file's existence, is the positional discriminator the `worktree create-fork`
    /// hook reads (design §5.3). Idempotent (re-arm at B' overwrites base).
    /// Sole-writer; orchestrator-classed — refused under worker-mode.
    ArmSpawn {
        /// The base commit B every spawn in this batch forks at — `dispatch setup`'s
        /// stdout `base=<dispatch_tip>` (the same tip the subprocess arm feeds
        /// `fork --base`). Must be a 4..=64-char hex oid (the reader's accepted form).
        #[arg(long)]
        base: String,

        /// The slice being dispatched (bare number) — diagnostic only; the arming dir
        /// is per-coord-tree, not per-slice (cross-slice partition is by coord tree).
        #[arg(long)]
        slice: Option<u32>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum CandidateCommand {
    /// Create a candidate (the happy path: provenance gate → no-ff 3-way merge →
    /// zero-oid CAS branch → recorded row). A content conflict aborts cleanly,
    /// writing no row/ref/worktree.
    Create {
        /// The slice id (bare number, e.g. `68`).
        #[arg(long)]
        slice: u32,

        /// The human label (e.g. `review-001`); the ref is
        /// `candidate/<slice>/<label>` and the id `cand-<slice>-<label>`.
        #[arg(long, visible_alias = "target")]
        label: String,

        /// Flavour: `audit` | `experiment`.
        #[arg(long, default_value = "audit")]
        kind: String,

        /// Role: `review_surface` | `close_target` | `scratch`.
        #[arg(long)]
        role: String,

        /// Payload: `impl_bundle` | `code`.
        #[arg(long)]
        payload: String,

        /// The base ref the merge is computed against (e.g. `refs/heads/main`).
        #[arg(long)]
        base: String,

        /// The source ref merged in. Defaults to `review/<slice>` for a
        /// `review_surface`; required otherwise (e.g. a `phase/<slice>-NN`).
        #[arg(long)]
        source: Option<String>,

        /// An optional prior candidate id this fresh row supersedes (EX-2).
        #[arg(long)]
        supersedes: Option<String>,

        /// Also materialise a linked worktree at the candidate branch (opt-in
        /// here; mandatory-for-review is PHASE-03).
        #[arg(long)]
        worktree: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Status (SL-068 PHASE-04): a read-only self-describing surface — lists the
    /// evidence refs and the candidate interaction branches in separate groups,
    /// reports each candidate's base/source/tip/status/admission, surfaces ref
    /// drift, and prints the safe next command(s). Read-classed — never mutates a
    /// ref or the ledger, so it works under worker-mode.
    Status {
        /// The slice id (bare number, e.g. `68`).
        #[arg(long)]
        slice: u32,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Admit (SL-068 PHASE-05): pin a recorded candidate's committed tip as the
    /// immutable OID a downstream verb (close/review) targets, after validating
    /// provenance (the recorded merge is the Doctrine candidate merge and an
    /// ancestor of the admitted tip) and re-reading the ref. Writes ONLY
    /// `candidates.toml` — never an evidence/candidate ref. Orchestrator-classed.
    Admit {
        /// The slice id (bare number, e.g. `68`).
        #[arg(long)]
        slice: u32,

        /// Role: `review_surface` | `close_target` (scratch is not admissible).
        #[arg(long)]
        role: String,

        /// The candidate ref to admit (e.g. `refs/heads/candidate/064/close-001`).
        #[arg(long)]
        candidate: String,

        /// The governing review (e.g. `RV-007`).
        #[arg(long)]
        review: Option<String>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: DispatchCommand, _color: bool) -> anyhow::Result<()> {
    match cmd {
        DispatchCommand::Sync {
            slice,
            integrate,
            show_journal_trunk_oid,
            trunk,
            edge,
            path,
            ..
        } => {
            // The `stage` group is `required = true` single-choice: exactly one
            // of `--prepare-review` / `--integrate` / `--show-journal-trunk-oid`
            // is set, so the booleans select the stage in order (no unreachable
            // arm).
            if show_journal_trunk_oid {
                // SL-128 D3: absent `--trunk` defaults from `[dispatch] deliver_to`;
                // explicit `--trunk` still wins. `--integrate` is unchanged.
                run_show_journal_trunk_oid(path, slice, trunk.as_deref())
            } else if integrate {
                run_integrate(path, slice, trunk.as_deref(), edge.as_deref())
            } else {
                run_prepare_review(path, slice)
            }
        }
        DispatchCommand::RecordBoundary {
            slice,
            phase,
            code_start,
            code_end,
            path,
        } => run_record_boundary(path, slice, &phase, &code_start, &code_end),
        DispatchCommand::RefreshBase { slice, path } => run_refresh_base(path, slice),
        DispatchCommand::Setup { slice, dir, path } => {
            // Read the harness signal here in the shell (ISS-031 placement
            // guard); a `CLAUDE`-prefixed env var marks the Claude arm, whose
            // outside-root coordination dir silently produces a wrong base.
            let claude_harness =
                std::env::vars_os().any(|(k, _v)| k.to_string_lossy().starts_with("CLAUDE"));
            run_setup(path, slice, &dir, claude_harness)
        }
        DispatchCommand::Candidate { command } => match command {
            CandidateCommand::Create {
                slice,
                label,
                kind,
                role,
                payload,
                base,
                source,
                supersedes,
                worktree,
                path,
            } => {
                let req = CreateRequest {
                    slice,
                    label,
                    kind: parse_kind(&kind)?,
                    role: parse_role(&role)?,
                    payload: parse_payload(&payload)?,
                    base,
                    source,
                    supersedes,
                    worktree,
                    created_at: crate::clock::today(),
                };
                run_candidate_create(path, &req)
            }
            CandidateCommand::Status { slice, path } => run_candidate_status(path, slice),
            CandidateCommand::Admit {
                slice,
                role,
                candidate,
                review,
                path,
            } => {
                let req = AdmitRequest {
                    slice,
                    role: parse_role(&role)?,
                    candidate,
                    review,
                    admitted_at: crate::clock::today(),
                };
                run_candidate_admit(path, &req)
            }
        },
        DispatchCommand::PlanNext { slice, json, path } => run_plan_next(path, slice, json),
        DispatchCommand::Status { slice, json, path } => run_status(path, slice, json),
        DispatchCommand::DeliverTo { path } => run_deliver_to(path),
        DispatchCommand::ArmSpawn { base, slice, path } => run_arm_spawn(path, &base, slice),
    }
}

/// `dispatch arm-spawn` — write the arming `base` file and print the spawn dir
/// (SL-152 PHASE-03; design §5.2/§5.3). The arming dir is in the coord tree's own
/// runtime state (gitignored, withheld `Tier::State` ⇒ never provisioned into a
/// worker fork). The path const is SHARED with the `worktree create-fork` reader
/// ([`crate::worktree::ARMING_SUBPATH`]) — one contract anchor, no re-spelling.
fn run_arm_spawn(path: Option<PathBuf>, base: &str, slice: Option<u32>) -> anyhow::Result<()> {
    // Fail closed on a base outside the reader's accepted envelope (4..=64 hex), so a
    // bad base surfaces at arm time, not silently as a no-fork at spawn time.
    let b = base.trim();
    if !(4..=64).contains(&b.len()) || !b.bytes().all(|c| c.is_ascii_hexdigit()) {
        bail!("bad-base: `{base}` is not a 4..=64-char hex oid");
    }

    let root = root::find(path, &root::default_markers())?;
    let spawn = root.join(crate::worktree::ARMING_SUBPATH);
    std::fs::create_dir_all(&spawn)
        .with_context(|| format!("create arming dir {}", spawn.display()))?;
    crate::fsutil::write_atomic(&spawn.join("base"), format!("{b}\n").as_bytes())
        .with_context(|| format!("write arming base in {}", spawn.display()))?;

    let spawn_canon = std::fs::canonicalize(&spawn)
        .with_context(|| format!("canonicalize arming dir {}", spawn.display()))?;
    if let Some(slice) = slice {
        writeln!(io::stderr(), "armed SL-{slice:03} at base {b}")?;
    }
    writeln!(io::stdout(), "{}", spawn_canon.display())?;
    Ok(())
}

/// PURE — coordination-worktree placement guard (no env/disk; CLAUDE.md split).
///
/// The Claude dispatch arm forks the Agent `isolation: worktree` worker off the
/// Bash cwd's HEAD; base==B is achieved by parking the cwd in the coordination
/// worktree before spawn. Under a harness that confines the cwd to the project
/// root (a bubblewrap jail), a `cd` to a path OUTSIDE the root silently reverts —
/// the worker then forks `main`, not B (ISS-031). Fail closed exactly there: an
/// outside-root coordination dir under a Claude harness. Non-Claude arms keep
/// their enforced outside-root worktree isolation (ADR-008) untouched.
fn classify_coord_placement(
    dir_inside_root: bool,
    claude_harness: bool,
) -> Result<(), &'static str> {
    if claude_harness && !dir_inside_root {
        Err("coord-outside-root-under-claude")
    } else {
        Ok(())
    }
}

/// Resolve `p` to an absolute path against the CWD (best-effort; impure shell).
fn absolutize(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().map_or_else(|_unused| p.to_path_buf(), |cwd| cwd.join(p))
    }
}

/// CLI entry — create or resume the dispatch coordination worktree for `slice`
/// and emit the orchestration env contract on stdout (SL-085, design §2).
/// Gates on `plan.toml` existence + non-empty phase list BEFORE creating the
/// coordination worktree. `claude_harness` is the env signal read by the caller
/// (a `CLAUDE`-prefixed var present) — passed in, not read here, so the placement
/// guard is unit-testable independent of the test runner's own environment.
pub(crate) fn run_setup(
    path: Option<PathBuf>,
    slice: u32,
    dir: &Path,
    claude_harness: bool,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    // Placement guard (ISS-031): on the Claude arm a coordination worktree
    // outside the project root silently produces a wrong-base spawn. Fail closed
    // before doing any work.
    let dir_inside_root = absolutize(dir).starts_with(absolutize(&root));
    classify_coord_placement(dir_inside_root, claude_harness).map_err(|token| {
        anyhow::anyhow!(
            "{token}: coordination worktree '{}' is outside the project root '{}'. \
             The Claude dispatch arm forks the Agent worktree off the Bash cwd's HEAD; \
             under a cwd-confining jail a `cd` outside the root silently reverts, so the \
             worker would fork `main` instead of base B. Use a path under the project \
             root — convention: .dispatch/SL-{slice:03}.",
            dir.display(),
            root.display()
        )
    })?;

    // Plan gate: read plan.toml, require existence + non-empty phase list.
    let slice_root = root.join(".doctrine/slice");
    let plan = crate::slice::read_plan(&slice_root, slice).with_context(|| {
        format!("no plan for SL-{slice:03}; run 'doctrine slice plan {slice}' first")
    })?;
    if plan.phases.is_empty() {
        anyhow::bail!("plan for SL-{slice:03} has no phases; add phases to plan.toml first");
    }

    // Delegate to the extracted pure-ish core.
    let outcome = crate::worktree::coordinate(&root, slice, dir)?;

    // Emit the dispatch env contract on stdout (4 KEY=value lines).
    let dispatch_ref = format!("refs/heads/dispatch/{slice:03}");
    writeln!(io::stdout(), "coordination_dir={}", dir.display())?;
    writeln!(io::stdout(), "base={}", outcome.dispatch_tip)?;
    writeln!(io::stdout(), "slice={slice}")?;
    writeln!(io::stdout(), "dispatch_ref={dispatch_ref}")?;

    Ok(())
}

/// One planned projection: a target ref and the commit it should be created at.
/// `source_oid` is the object the projection was computed from (the journal's
/// replay input).
struct Planned {
    target_ref: String,
    source_oid: String,
    commit_oid: String,
}

/// CLI entry — resolve the root and run stage-1 prepare-review for `slice`.
pub(crate) fn run_prepare_review(path: Option<PathBuf>, slice: u32) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    prepare_review(&root, slice)
}

/// CLI entry — print the committed `dispatch/<slice>` journal trunk-row's full
/// `planned_new_oid` to stdout: the close step-3a read surface (SL-121 §3(b)). The
/// row is named by `trunk` (`target_ref == trunk`). Tree-reads the journal from the
/// coordination tip (`ledger::read_journal_at_ref` → `read_path_at`), so it returns
/// the same value from any checkout — the `sync-tree-reads-ledger-not-worktree`
/// invariant — never a transient `candidate admit` stdout. An absent journal/row
/// refuses (named token), emitting no oid, so the skill never diffs an empty value.
pub(crate) fn run_show_journal_trunk_oid(
    path: Option<PathBuf>,
    slice: u32,
    trunk: Option<&str>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    // SL-128 D3: absent `--trunk` defaults from `[dispatch] deliver_to`
    // (explicit `--trunk` already won at the call site).
    let trunk: String = match trunk {
        Some(t) => t.to_string(),
        None => crate::dtoml::load_doctrine_toml(&root)?.dispatch.deliver_to,
    };
    let slice3 = format!("{slice:03}");
    // Absent ref/journal folds to an empty journal — same "no journal row"
    // refusal as before, now via the shared leaf tree-reader (DRY, EX-3).
    let journal = crate::ledger::read_journal_at_ref(&root, slice)?.unwrap_or_default();
    let oid = journal
        .rows
        .iter()
        .find(|r| r.target_ref == trunk)
        .map(|r| r.planned_new_oid.as_str())
        .with_context(|| {
            format!("show-journal-trunk-oid: no journal row for {trunk} on dispatch/{slice3}")
        })?;
    writeln!(io::stdout(), "{oid}")?;
    Ok(())
}

/// Print the resolved `[dispatch] deliver_to` trunk delivery ref to stdout
/// (SL-128 / IMP-124) — the single source the close skill names instead of a
/// `refs/heads/main` literal, and a convenience for hand-driven git work.
/// Read-only; callable from anywhere (like `dispatch status`/`plan-next`).
pub(crate) fn run_deliver_to(path: Option<PathBuf>) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let deliver_to = crate::dtoml::load_doctrine_toml(&root)?.dispatch.deliver_to;
    writeln!(io::stdout(), "{deliver_to}")?;
    Ok(())
}

/// CLI entry — resolve the root and run stage-2 integrate for `slice`. `trunk`
/// names the ref the code units project onto (ff-only); `edge` names an optional
/// aggregate ref. Both default off ⇒ a pure idempotent journal replay (EX-1).
pub(crate) fn run_integrate(
    path: Option<PathBuf>,
    slice: u32,
    trunk: Option<&str>,
    edge: Option<&str>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    integrate(&root, slice, trunk, edge)
}

/// CLI entry — funnel-time recording: append a per-phase code boundary to
/// `boundaries.toml` (design §4.3; the claude-arm phase-cut input the orchestrator
/// records between funnel steps 7 (code) and 8 (knowledge)). `code_start`/
/// `code_end` are resolved to full commit oids so the ledger holds stable shas,
/// not mobile refs. The orchestrator commits the file onto `dispatch/<slice>`;
/// stage-1 prepare-review tree-reads it (`mem.pattern.dispatch.sync-tree-reads`).
pub(crate) fn run_record_boundary(
    path: Option<PathBuf>,
    slice: u32,
    phase: &str,
    code_start: &str,
    code_end: &str,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let resolve = |refish: &str| -> anyhow::Result<String> {
        resolve_commit(&root, refish)?
            .with_context(|| format!("record-boundary: {refish} does not resolve to a commit"))
    };
    let row = crate::boundary::BoundaryRow {
        phase: phase.to_string(),
        code_start_oid: resolve(code_start)?,
        code_end_oid: resolve(code_end)?,
        // The funnel is the dispatch landing writer (design §5.3); the one row is
        // cloned to both the committed ledger and the registry, so this single
        // stamp covers both writes.
        provenance: crate::boundary::Provenance::Funnel,
    };
    // (1) The committed claude-arm ledger (`.doctrine/dispatch/<N>/boundaries.toml`)
    // — UNCHANGED, the phase-cut input prepare-review tree-reads.
    crate::ledger::record_boundary(&root, slice, row.clone())?;
    // (2) ALONGSIDE it (SL-147 PHASE-04, T3): the arm-NEUTRAL recorded source-delta
    // registry. The funnel runs this same `record-boundary` beat for BOTH arms with
    // the per-phase coordination boundary (B → B+1), so this is the funnel's
    // mutually-exclusive counterpart to the solo binding — never both for one phase.
    // It resolves its one shared file against the PRIMARY tree (so a coordination
    // worktree still writes the row the integrator reads) and applies the F-6 guard
    // + upsert. It does NOT touch the committed ledger above.
    crate::state::record_source_delta(&root, slice, row)
}

/// CLI entry — `doctrine dispatch refresh-base --slice N` (SL-127 §3.2). Advance
/// `dispatch/<NNN>`'s base past trunk drift via a REAL `git merge --no-ff` of the
/// current trunk tip into the dispatch branch, run in the LIVE coordination
/// worktree (never the session/main tree). Single responsibility: the merge only —
/// it does NOT regenerate the review bundle (the operator re-runs `sync
/// --prepare-review` afterwards). Per SPEC-021 it REPORTS conflicts, never
/// auto-resolves: a conflicted merge halts non-zero with the conflicting paths
/// named, leaving `MERGE_HEAD` + markers for the operator and the dispatch ref
/// unadvanced.
pub(crate) fn run_refresh_base(path: Option<PathBuf>, slice: u32) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let slice3 = format!("{slice:03}");
    let dispatch_ref = format!("refs/heads/dispatch/{slice3}");

    let trunk_tip = git::trunk_commit(&root)?.with_context(|| "trunk ref not found")?;

    // Resolve the live coordination worktree; ALL subsequent git runs use `coord`
    // as the root so they execute there, never the session tree.
    let coord = git::worktree_for_ref(&root, &dispatch_ref)?.with_context(|| {
        format!(
            "no live coordination worktree for dispatch/{slice3}; \
             run 'dispatch setup --slice {slice}' (or resume) first"
        )
    })?;

    let dispatch_tip = git::git_text(&coord, &["rev-parse", "HEAD"])?;

    // Refuse to merge over WIP — a dirty coord tree is the operator's, untouched.
    let dirty = git::git_text(&coord, &["status", "--porcelain"])?;
    if !dirty.is_empty() {
        bail!("refusing to refresh over a dirty coordination worktree (dispatch/{slice3})");
    }

    // Unrelated histories — refuse BEFORE any merge (codex C7).
    if git::merge_base(&coord, &dispatch_tip, &trunk_tip)?.is_none() {
        bail!("unrelated histories — dispatch/{slice3} and trunk share no common ancestor");
    }

    // Trunk already contained in the dispatch branch ⇒ nothing to do, no write.
    if git::is_ancestor(&coord, &trunk_tip, &dispatch_tip)? {
        writeln!(
            io::stdout(),
            "dispatch/{slice3} already fresh — trunk {} is already merged",
            short(&trunk_tip)
        )?;
        return Ok(());
    }

    // The real merge in the coordination worktree. `git_status_ok` returns the
    // raw exit success (it routes through the single `run_git` capture chokepoint)
    // — exit 0 ⇒ git committed the merge; non-zero ⇒ a conflict left MERGE_HEAD +
    // markers in `coord`.
    let msg = format!("refresh-base: merge trunk into dispatch/{slice3}");
    let clean = git::git_status_ok(&coord, &["merge", "--no-ff", "-m", &msg, &trunk_tip])?;

    if clean {
        let new_tip = git::git_text(&coord, &["rev-parse", "HEAD"])?;
        let merged = git::git_text(
            &coord,
            &[
                "rev-list",
                "--count",
                &format!("{dispatch_tip}..{trunk_tip}"),
            ],
        )?;
        writeln!(
            io::stdout(),
            "dispatch/{slice3} refreshed: merged {merged} trunk commit(s); new tip {}",
            short(&new_tip)
        )?;
        return Ok(());
    }

    // Conflict — collect the unmerged paths, report, and halt. Do NOT abort; the
    // operator resolves the half-merged coord worktree (SPEC-021).
    let conflicts = git::git_text(&coord, &["diff", "--name-only", "--diff-filter=U"])?;
    let paths: Vec<&str> = conflicts.lines().filter(|l| !l.is_empty()).collect();
    bail!(
        "refresh-base merge of trunk into dispatch/{slice3} conflicted in {} path(s):\n  {}\n\
         resolve them in the coordination worktree, then commit the merge \
         (MERGE_HEAD is left in place; the dispatch ref is unadvanced).",
        paths.len(),
        paths.join("\n  ")
    );
}

/// Short form of a commit oid for human report lines (first 7 chars).
fn short(oid: &str) -> &str {
    oid.get(..7).unwrap_or(oid)
}

// --- SL-068 PHASE-02: `dispatch candidate create` (design §5.3) --------------

/// The resolved create request — the CLI flag bundle parsed into typed axes (the
/// clock is read in the shell and passed in, pure/imperative split). `source` is
/// the ref the candidate merges in; `base` the ref the merge is computed against;
/// `supersedes` an optional prior candidate id this fresh row links to (EX-2).
pub(crate) struct CreateRequest {
    pub slice: u32,
    pub label: String,
    pub kind: CandidateKind,
    pub role: CandidateRole,
    pub payload: CandidatePayload,
    pub base: String,
    pub source: Option<String>,
    pub supersedes: Option<String>,
    pub worktree: bool,
    pub created_at: String,
}

/// Parse the `--kind` token into [`CandidateKind`].
pub(crate) fn parse_kind(token: &str) -> anyhow::Result<CandidateKind> {
    match token {
        "audit" => Ok(CandidateKind::Audit),
        "experiment" => Ok(CandidateKind::Experiment),
        other => bail!("unknown candidate kind {other:?} (expected audit|experiment)"),
    }
}

/// Parse the `--role` token into [`CandidateRole`].
pub(crate) fn parse_role(token: &str) -> anyhow::Result<CandidateRole> {
    match token {
        "review_surface" => Ok(CandidateRole::ReviewSurface),
        "close_target" => Ok(CandidateRole::CloseTarget),
        "scratch" => Ok(CandidateRole::Scratch),
        other => {
            bail!("unknown candidate role {other:?} (expected review_surface|close_target|scratch)")
        }
    }
}

/// Parse the `--payload` token into [`CandidatePayload`].
pub(crate) fn parse_payload(token: &str) -> anyhow::Result<CandidatePayload> {
    match token {
        "impl_bundle" => Ok(CandidatePayload::ImplBundle),
        "code" => Ok(CandidatePayload::Code),
        other => bail!("unknown candidate payload {other:?} (expected impl_bundle|code)"),
    }
}

/// CLI entry — resolve the root and create a candidate for `req`.
pub(crate) fn run_candidate_create(
    path: Option<PathBuf>,
    req: &CreateRequest,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_create(&root, req)
}

/// The source ref a create merges in: an explicit `--source`, else the default
/// for the role — `review/<slice>` for a review surface; otherwise an explicit
/// source is required (a close target's `phase/<slice>-NN` has no single default).
fn resolve_source_ref(req: &CreateRequest, slice3: &str) -> anyhow::Result<String> {
    if let Some(src) = &req.source {
        return Ok(src.clone());
    }
    match req.role {
        CandidateRole::ReviewSurface => Ok(format!("refs/heads/review/{slice3}")),
        CandidateRole::CloseTarget | CandidateRole::Scratch => bail!(
            "candidate create: --source is required for a {} candidate",
            role_token(req.role)
        ),
    }
}

/// The persisted token for a role (error messages only; the on-disk form is
/// serde's, never hand-spliced into TOML).
fn role_token(role: CandidateRole) -> &'static str {
    match role {
        CandidateRole::ReviewSurface => "review_surface",
        CandidateRole::CloseTarget => "close_target",
        CandidateRole::Scratch => "scratch",
    }
}

/// EX-1 provenance: the candidate's source ref must correspond to a journal
/// prepare-review row whose `status == Verified`. For a `phase/<slice>-NN` source
/// (a `code` close target) additionally refuse when an EARLIER non-empty
/// phase-chain row `failed` — a hole in the chain means the selected phase does
/// not actually carry verified prior code. Reads the journal from the
/// coordination branch tip (object db). Refuses (no writes) before any verified
/// evidence exists.
fn check_provenance(journal: &Journal, slice3: &str, source_ref: &str) -> anyhow::Result<()> {
    let row = journal
        .rows
        .iter()
        .find(|r| r.target_ref == source_ref)
        .with_context(|| {
            format!(
                "candidate create: no prepare-review journal row for source {source_ref} — \
                 run `dispatch sync --prepare-review` first"
            )
        })?;
    anyhow::ensure!(
        row.status == LedgerStatus::Verified,
        "candidate create: source {source_ref} is not verified (status {:?}) — \
         no verified evidence to build a candidate from",
        row.status
    );

    // Phase-chain integrity: a close target built off phase/<slice>-NN must have
    // no earlier failed phase row (an unresolved hole below the selected phase).
    let prefix = format!("refs/heads/phase/{slice3}-");
    if let Some(nn) = source_ref
        .strip_prefix(&prefix)
        .and_then(|nn| nn.parse::<u32>().ok())
    {
        for r in &journal.rows {
            if let Some(other) = r
                .target_ref
                .strip_prefix(&prefix)
                .and_then(|n| n.parse::<u32>().ok())
                && other < nn
                && r.status == LedgerStatus::Failed
            {
                bail!(
                    "candidate create: an earlier phase row {} failed — the phase chain \
                     below {source_ref} has an unresolved hole",
                    r.target_ref
                );
            }
        }
    }
    Ok(())
}

/// The no-`--worktree` content-conflict abort message (design §3.3). Pure. When
/// `ahead == 0` the result is BYTE-IDENTICAL to the pre-SL-127 text — the SL-127
/// base-divergence hint is APPENDED only when trunk has advanced past the source
/// (`ahead > 0`), and even then never asserts the cause (codex C5). This is the
/// single source of the abort text, so the production arm and the byte-identity
/// test cannot drift.
fn candidate_conflict_message(source_ref: &str, base: &str, ahead: u32) -> String {
    let hint = if ahead > 0 {
        format!(
            "; trunk has advanced {ahead} commit(s) past this source — \
             the conflict may be base divergence; try `dispatch refresh-base` \
             then re-prepare + re-create"
        )
    } else {
        String::new()
    };
    format!(
        "candidate create: 3-way merge of {source_ref} onto {base} conflicts — \
         pass --worktree to park the candidate branch at the base for \
         manual resolve+commit, or abort (no row/ref/worktree written){hint}"
    )
}

/// Core `candidate create` (design §5.3, EX-1..5). Happy path only — a content
/// conflict aborts cleanly with NO row/ref/worktree written (the conflicted +
/// `--worktree` lifecycle is PHASE-03). Sequencing: provenance gate → compute the
/// no-ff 3-way merge object → zero-oid CAS the candidate branch → record the row.
/// The CAS precedes the row write, so a refused branch creation leaves no partial
/// durable state.
fn candidate_create(root: &Path, req: &CreateRequest) -> anyhow::Result<()> {
    let slice3 = format!("{:03}", req.slice);
    let coord_ref = format!("refs/heads/dispatch/{slice3}");
    let target_ref = format!("refs/heads/candidate/{slice3}/{}", req.label);
    let id = format!("cand-{slice3}-{}", req.label);

    // --- EX-2: raw-evidence-ref write guard FIRST (invariant I9) — refuse a
    //     create driven from a worktree checked out on a `review/*` / `phase/*`
    //     evidence ref, before ANY durable write. The candidate workflow never
    //     edits the raw evidence refs in place (design §5.3). Pure string check
    //     on the branch the shell resolved. --------------------------------------
    if let Some(branch) = current_branch(root)?
        && is_raw_evidence_ref(&branch)
    {
        bail!(
            "candidate create: the current worktree is checked out on raw evidence ref {branch:?} \
             (review/* and phase/* are immutable, invariant I9) — never edit it in place; \
             run `dispatch candidate create` from a safe branch (e.g. the coordination tree) \
             to publish a candidate instead"
        );
    }

    // --- EX-1: review_surface requires an explicit --worktree in v1. Refuse
    //     before any write so a missing flag leaves no partial state. -----------
    if req.role == CandidateRole::ReviewSurface && !req.worktree {
        bail!(
            "candidate create: a review_surface candidate requires an explicit --worktree \
             (v1: the review surface is always materialised for the reviewer to read)"
        );
    }

    // --- EX-1: verified-source provenance gate FIRST (before any ref resolve
    //     or write) — refuse before verified evidence exists, by ref NAME -------
    let source_ref = resolve_source_ref(req, &slice3)?;
    let journal = read_ledger::<Journal>(root, &coord_ref, &slice3, "journal.toml")?;
    check_provenance(&journal, &slice3, &source_ref)?;

    // --- resolve source + base oids (the journal proved the source verified) -
    let source_oid = resolve_commit(root, &source_ref)?
        .with_context(|| format!("candidate create: source {source_ref} does not resolve"))?;
    let base_oid = resolve_commit(root, &req.base)?
        .with_context(|| format!("candidate create: base {} does not resolve", req.base))?;

    // --- EX-2 supersession: a fresh row links to a prior candidate id --------
    let mut ledger = read_candidates(root, req.slice)?;
    let supersedes = match &req.supersedes {
        Some(prior) => {
            anyhow::ensure!(
                ledger.rows.iter().any(|r| r.id == *prior),
                "candidate create: --supersedes {prior} names no recorded candidate"
            );
            prior.clone()
        }
        None => String::new(),
    };

    // --- EX-3: explicit no-ff 3-way merge (object db only) -------------------
    let merge_base = git::merge_base(root, &base_oid, &source_oid)?.with_context(|| {
        format!(
            "candidate create: base {base_oid} and source {source_oid} share no common ancestor"
        )
    })?;

    // The merge outcome decides the lifecycle (EX-1): a clean union commits at
    // the merge tree (status created); a conflict either ABORTS with no durable
    // state (no --worktree) or parks the branch at the base for the user to
    // resolve+commit, recording a conflicted row (--worktree).
    let (branch_oid, merge_oid, status) =
        match git::merge_tree(root, &merge_base, &base_oid, &source_oid)? {
            MergeTree::Clean { tree } => {
                let merge_oid = git::commit_tree_merge(
                    root,
                    &tree,
                    &base_oid,
                    &source_oid,
                    &format!("candidate({slice3}/{}): merge {source_ref}", req.label),
                )?;
                // Clean: the branch points at the merge commit.
                (merge_oid.clone(), merge_oid, CandidateStatus::Created)
            }
            MergeTree::Conflict if !req.worktree => {
                // SL-127 EX-1 (§3.3): diagnostic-only base-divergence hint. The
                // drift count is resolved here in the shell; the (pure) message
                // builder appends a non-asserting hint when trunk has advanced past
                // the source, and renders BYTE-IDENTICAL legacy text when it has not.
                let ahead = trunk_drift(root, &source_oid)?.map_or(0, |d| d.ahead);
                bail!(candidate_conflict_message(&source_ref, &req.base, ahead))
            }
            // Conflicted + --worktree: park the branch at the base so the user
            // resolves+commits in the worktree. No merge commit exists yet.
            MergeTree::Conflict => (base_oid.clone(), String::new(), CandidateStatus::Conflicted),
        };

    // --- EX-3: create the branch under zero-oid CAS (refuses an existing ref).
    //     Precedes the row write so a refused creation leaves no partial state.
    match git::update_ref_cas(root, &target_ref, &branch_oid, ZERO_OID)? {
        RefCas::Updated => {}
        RefCas::Moved { actual } => bail!(
            "candidate create: {target_ref} already exists (at {}) — \
             supersede creates a fresh label, never rewrites a branch",
            actual.as_deref().unwrap_or("?")
        ),
    }

    // --- EX-3: materialise the worktree BEFORE the row write so a worktree
    //     failure rolls the ref back, leaving no orphan branch the ledger does
    //     not know about. The conflicted lifecycle ALWAYS materialises (so the
    //     user can resolve); a clean create only on the opt-in --worktree. -----
    let worktree_path = if req.worktree {
        match add_candidate_worktree(root, &id, &target_ref) {
            Ok(path) => Some(path),
            Err(e) => {
                // Roll back the branch we just created — no partial durable state.
                rollback_ref(root, &target_ref, &branch_oid);
                return Err(e);
            }
        }
    } else {
        None
    };

    // --- EX-3: record the candidate row (status created | conflicted) --------
    let row = CandidateRow {
        id: id.clone(),
        label: req.label.clone(),
        kind: req.kind,
        role: req.role,
        payload: req.payload,
        target_ref: target_ref.clone(),
        source_ref,
        source_oid,
        base_ref: req.base.clone(),
        base_oid,
        merge_oid: merge_oid.clone(),
        status,
        supersedes,
        reason: String::new(),
        created_by: "dispatch candidate create".to_owned(),
        created_at: req.created_at.clone(),
    };
    ledger.rows.push(row);
    crate::ledger::write_candidates(root, req.slice, &ledger)?;

    writeln!(io::stdout(), "{target_ref}")?;
    if let Some(path) = &worktree_path {
        writeln!(io::stdout(), "{}", path.display())?;
    }
    match status {
        CandidateStatus::Conflicted => writeln!(
            io::stderr(),
            "candidate create: {id} conflicted — branch parked at base {branch_oid}; \
             resolve+commit in {}",
            worktree_path
                .as_ref()
                .map_or_else(|| "(worktree)".to_owned(), |p| p.display().to_string())
        )?,
        _ => writeln!(
            io::stderr(),
            "candidate create: {id} created at {merge_oid}"
        )?,
    }
    Ok(())
}

/// Add a linked worktree for candidate `id` at `target_ref` under
/// `.doctrine/state/dispatch/candidate/<id>` (the gitignored runtime tier).
/// Returns the worktree path on success. Impure shell.
fn add_candidate_worktree(root: &Path, id: &str, target_ref: &str) -> anyhow::Result<PathBuf> {
    let wt_path = root.join(".doctrine/state/dispatch/candidate").join(id);
    if let Some(parent) = wt_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let wt_str = wt_path
        .to_str()
        .context("candidate create: worktree path is not valid UTF-8")?;
    git::git_text(root, &["worktree", "add", "--quiet", wt_str, target_ref])?;
    Ok(wt_path)
}

/// Best-effort CAS rollback of a ref this create just created — used when a later
/// step fails after the branch was written (EX-3: no partial durable state). A
/// failed delete is swallowed: the caller is already returning the primary error.
fn rollback_ref(root: &Path, target_ref: &str, expected: &str) {
    let _ignored = git::git_opt(root, &["update-ref", "-d", target_ref, expected]);
}

/// The branch the worktree at `root` is checked out on, short form (e.g.
/// `review/064`), or `None` for a detached HEAD. The raw-evidence-ref guard
/// (EX-2) keys on this. Impure shell.
fn current_branch(root: &Path) -> anyhow::Result<Option<String>> {
    Ok(git::git_opt(
        root,
        &["symbolic-ref", "--quiet", "--short", "HEAD"],
    )?)
}

/// Whether `branch` is a raw evidence ref the candidate workflow must never edit
/// in place (invariant I9): the `review/<slice>` impl bundle or a
/// `phase/<slice>-NN` per-phase cut. Pure.
fn is_raw_evidence_ref(branch: &str) -> bool {
    branch.starts_with("review/") || branch.starts_with("phase/")
}

// --- SL-068 PHASE-05: `dispatch candidate admit` (design §5.2/§5.5) -----------

/// The resolved admit request — pin a recorded candidate's tip as the immutable
/// OID a downstream verb (close/review) targets. The clock (`admitted_at`) is read
/// in the shell and passed in (pure/imperative split, like [`CreateRequest`]).
pub(crate) struct AdmitRequest {
    pub slice: u32,
    pub role: CandidateRole,
    pub candidate: String,
    pub review: Option<String>,
    pub admitted_at: String,
}

/// CLI entry — resolve the root and admit the candidate for `req`.
pub(crate) fn run_candidate_admit(path: Option<PathBuf>, req: &AdmitRequest) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_admit(&root, req)
}

/// Core `candidate admit` (design §5.2 + §5.5 invariants). Pins a recorded
/// candidate's committed tip as the immutable `admitted_oid` a downstream verb
/// targets, after validating provenance (I3, R7): the recorded `merge_oid` is the
/// Doctrine-created candidate merge (its parents are exactly base+source) AND an
/// ancestor of the admitted tip. Re-reads the candidate ref before recording so a
/// ref moved mid-admission is refused (EX-1). Writes ONLY `candidates.toml` — never
/// trunk/edge/`review/*`/`phase/*`/the candidate ref (EX-4). Exactly one current
/// admission per role afterward (the role slot is overwritten; supersession is
/// explicit history via `supersedes`).
fn candidate_admit(root: &Path, req: &AdmitRequest) -> anyhow::Result<()> {
    // --- I9 raw-evidence-ref write guard FIRST (before any read/write) — refuse
    //     an admit driven from a worktree checked out on a `review/*` / `phase/*`
    //     evidence ref. Mirrors create's guard. -----------------------------------
    if let Some(branch) = current_branch(root)?
        && is_raw_evidence_ref(&branch)
    {
        bail!(
            "candidate admit: the current worktree is checked out on raw evidence ref {branch:?} \
             (review/* and phase/* are immutable, invariant I9) — never edit it in place; \
             run `dispatch candidate admit` from a safe branch (e.g. the coordination tree)"
        );
    }

    // scratch is not an admissible role — refuse before any read.
    if req.role == CandidateRole::Scratch {
        bail!("candidate admit: a scratch candidate is not admissible (no review/close target)");
    }

    // --- resolve the candidate tip (must be a committed clean tip) -------------
    let admitted_1 = resolve_commit(root, &req.candidate)?.with_context(|| {
        format!(
            "candidate admit: candidate {} does not resolve to a committed tip",
            req.candidate
        )
    })?;

    // --- find the recorded row pinned by the candidate ref ---------------------
    let mut ledger = read_candidates(root, req.slice)?;
    let row = ledger
        .rows
        .iter()
        .find(|r| r.target_ref == req.candidate)
        .with_context(|| {
            format!(
                "candidate admit: no recorded candidate at {} — admit pins a recorded candidate",
                req.candidate
            )
        })?
        .clone();

    // --- role must match (no mis-slotting) -------------------------------------
    anyhow::ensure!(
        row.role == req.role,
        "candidate admit: candidate {} is role {}, cannot admit as {}",
        row.id,
        role_token(row.role),
        role_token(req.role)
    );

    // --- a conflicted/unresolved row has no Doctrine merge to validate ---------
    anyhow::ensure!(
        !row.merge_oid.is_empty(),
        "candidate admit: candidate {} has no Doctrine merge to validate \
         (conflicted/unresolved) — resolve and re-create before admitting",
        row.id
    );

    // --- provenance (EX-2, I3, R7): merge_oid is the Doctrine candidate merge --
    let merge_parents: std::collections::BTreeSet<String> =
        git::parents(root, &row.merge_oid)?.into_iter().collect();
    let expected_parents: std::collections::BTreeSet<String> =
        [row.base_oid.clone(), row.source_oid.clone()]
            .into_iter()
            .collect();
    anyhow::ensure!(
        merge_parents == expected_parents,
        "candidate admit: merge_oid {} is not the Doctrine candidate merge \
         (parents != base+source)",
        row.merge_oid
    );
    anyhow::ensure!(
        git::is_ancestor(root, &row.merge_oid, &admitted_1)?,
        "candidate admit: admitted tip {admitted_1} does not descend from candidate merge {} (I3)",
        row.merge_oid
    );

    // --- EX-1: re-read the candidate ref before recording — a tip moved between
    //     the first resolve and now is refused (record only the proven oid) -----
    let admitted_2 = resolve_commit(root, &req.candidate)?;
    anyhow::ensure!(
        admitted_2.as_deref() == Some(admitted_1.as_str()),
        "candidate admit: candidate {} moved during admission (was {admitted_1}, now {}) — \
         re-run admit",
        req.candidate,
        admitted_2.as_deref().unwrap_or("absent")
    );

    // --- EX-3, I5: record the admission, overwriting the role slot (exactly one
    //     current admission per role; supersession is explicit history) ---------
    let supersedes = prior_admission(&ledger, req.role)
        .map(|a| a.candidate_id.clone())
        .unwrap_or_default();
    let admission = Admission {
        candidate_id: row.id.clone(),
        candidate_ref: req.candidate.clone(),
        expected_ref_oid: admitted_1.clone(),
        admitted_oid: admitted_1.clone(),
        review: req.review.clone().unwrap_or_default(),
        supersedes,
        admitted_at: req.admitted_at.clone(),
    };
    // scratch was refused above; admit only ever reaches a review/close slot.
    let slot = match req.role {
        CandidateRole::ReviewSurface => &mut ledger.current_admission.review_surface,
        CandidateRole::CloseTarget | CandidateRole::Scratch => {
            &mut ledger.current_admission.close_target
        }
    };
    *slot = Some(admission);
    crate::ledger::write_candidates(root, req.slice, &ledger)?;

    writeln!(io::stdout(), "{admitted_1}")?;
    writeln!(
        io::stderr(),
        "candidate admit: {} admitted at {admitted_1} ({})",
        row.id,
        role_token(req.role)
    )?;
    Ok(())
}

/// The role's current admission, if any — the record a fresh admit supersedes.
fn prior_admission(ledger: &Candidates, role: CandidateRole) -> Option<&Admission> {
    match role {
        CandidateRole::CloseTarget => ledger.current_admission.close_target.as_ref(),
        CandidateRole::ReviewSurface => ledger.current_admission.review_surface.as_ref(),
        CandidateRole::Scratch => None,
    }
}

// --- SL-068 PHASE-04: `dispatch candidate status` (design §5.3, EX-1..3) ------

/// CLI entry — resolve the root and render the candidate status surface for
/// `slice`. Read-only: never mutates a ref or the ledger (EX-3).
pub(crate) fn run_candidate_status(path: Option<PathBuf>, slice: u32) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_status(&root, slice)
}

/// Abbreviate an oid to its leading 12 chars for the human surface; empty stays
/// empty (a conflicted row has no merge oid), `—` is the absent-ref sentinel
/// (kept verbatim). Pure.
fn short_oid(oid: &str) -> String {
    if oid.is_empty() || oid == "—" {
        return oid.to_owned();
    }
    oid.chars().take(12).collect()
}

/// One evidence-ref status row (the EX-1 evidence group): the ref name, its
/// human group label, and its live tip (`—` when the ref is absent). Pure data —
/// the impure shell resolves the tips and builds the rows.
struct EvidenceRow {
    refname: String,
    group: &'static str,
    tip: String,
}

/// Render the candidate status surface (design §5.3, EX-1..3): the evidence-ref
/// group, the candidate-ref group with per-candidate base/source/tip/status/
/// admission + drift, and the safe next command(s). READ-ONLY — it resolves live
/// ref tips and reads `candidates.toml`, never writing a ref or the ledger (EX-3).
/// From a worktree on a raw evidence ref it WARNS (unlike create's refusal, EX-3).
fn candidate_status(root: &Path, slice: u32) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");

    // EX-3: read-only — a raw-evidence-ref worktree only WARNS (never refuses,
    // unlike create's I9 guard) since status mutates nothing.
    if let Some(branch) = current_branch(root)?
        && is_raw_evidence_ref(&branch)
    {
        writeln!(
            io::stderr(),
            "candidate status: the current worktree is checked out on raw evidence ref `{branch}` \
             (review/* and phase/* are immutable) — status is read-only and changes nothing, but \
             never edit an evidence ref in place; publish via `dispatch candidate create`"
        )?;
    }

    let ledger = read_candidates(root, slice)?;

    // --- EX-1: the evidence-ref group, kept VISIBLY SEPARATE from candidates --
    let evidence = collect_evidence(root, &slice3)?;
    let mut grid: Vec<Vec<String>> = vec![cells(&["ref", "group", "tip"])];
    for row in &evidence {
        grid.push(cells(&[&row.refname, row.group, &short_oid(&row.tip)]));
    }
    writeln!(io::stdout(), "evidence refs:")?;
    write!(io::stdout(), "{}", render_table(&grid, None))?;

    // --- EX-2: the candidate-ref group with per-candidate report + drift ------
    writeln!(io::stdout(), "\ncandidates (interaction branches):")?;
    let mut cgrid: Vec<Vec<String>> = vec![cells(&[
        "id",
        "branch",
        "status",
        "base",
        "source",
        "tip",
        "admission",
        "drift",
    ])];
    let mut any_drift = false;
    for row in &ledger.rows {
        let report = candidate_report(root, &ledger, row)?;
        any_drift |= report.drift;
        cgrid.push(cells(&[
            &row.id,
            &row.target_ref,
            status_token(row.status),
            &short_oid(&row.base_oid),
            &short_oid(&row.source_oid),
            &short_oid(&report.tip),
            &report.admission,
            if report.drift { "DRIFT" } else { "ok" },
        ]));
    }
    if ledger.rows.is_empty() {
        writeln!(io::stdout(), "(none recorded)")?;
    } else {
        write!(io::stdout(), "{}", render_table(&cgrid, None))?;
    }

    // --- EX-3: print the safe NEXT command(s), not "inspect raw refs" ---------
    write_next_commands(&slice3, &ledger, any_drift)?;
    Ok(())
}

/// The per-candidate live report (EX-2): the candidate ref's live tip, a human
/// admission summary, and whether the live tip has DRIFTED from the
/// recorded/admitted OID (invariant I4 — reported, never hidden).
struct CandidateReport {
    tip: String,
    admission: String,
    drift: bool,
}

/// Build a candidate's live report (EX-2). The live tip is resolved from the
/// candidate's `target_ref` (`—` when absent); the admission summary names the
/// admitting review when this candidate is the role's admitted one. Drift = the
/// live tip differs from the OID the row pins: the admitted oid when admitted,
/// else the recorded `merge_oid` (skipped for a conflicted row, whose branch is
/// intentionally parked at base with no merge commit).
fn candidate_report(
    root: &Path,
    ledger: &Candidates,
    row: &CandidateRow,
) -> anyhow::Result<CandidateReport> {
    let tip = resolve_commit(root, &row.target_ref)?.unwrap_or_else(|| "—".to_owned());
    let admitted = admission_for(ledger, &row.id);
    let admission = match admitted {
        Some(a) => format!("admitted ({})", a.review),
        None => "—".to_owned(),
    };
    // The OID the row pins: the admitted oid when admitted, else the recorded
    // merge oid. A conflicted row (empty merge_oid, branch parked at base) is not
    // drift-checked — it has no recorded merge tip to compare against.
    let pinned = match admitted {
        Some(a) => Some(a.admitted_oid.as_str()),
        None if row.status == CandidateStatus::Conflicted => None,
        None if row.merge_oid.is_empty() => None,
        None => Some(row.merge_oid.as_str()),
    };
    let drift = match (pinned, tip.as_str()) {
        (Some(pin), live) => live != "—" && live != pin,
        (None, _) => false,
    };
    Ok(CandidateReport {
        tip,
        admission,
        drift,
    })
}

/// The admission record (either role) whose `candidate_id` matches `id`, if this
/// candidate is the currently-admitted one for its role. Pure lookup.
fn admission_for<'a>(ledger: &'a Candidates, id: &str) -> Option<&'a Admission> {
    [
        ledger.current_admission.close_target.as_ref(),
        ledger.current_admission.review_surface.as_ref(),
    ]
    .into_iter()
    .flatten()
    .find(|a| a.candidate_id == id)
}

/// Resolve the evidence-ref group (EX-1): the coordination branch, the impl
/// bundle, and every `phase/<slice>-NN` cut — NEVER conflated with a
/// `candidate/<slice>/*` interaction branch. Impure shell (resolves live tips).
fn collect_evidence(root: &Path, slice3: &str) -> anyhow::Result<Vec<EvidenceRow>> {
    let mut rows: Vec<EvidenceRow> = Vec::new();
    for (refname, group) in [
        (format!("refs/heads/dispatch/{slice3}"), "coordination"),
        (format!("refs/heads/review/{slice3}"), "impl-bundle"),
    ] {
        let tip = resolve_commit(root, &refname)?.unwrap_or_else(|| "—".to_owned());
        rows.push(EvidenceRow {
            refname,
            group,
            tip,
        });
    }
    for refname in for_each_ref(root, &format!("refs/heads/phase/{slice3}-*"))? {
        let tip = resolve_commit(root, &refname)?.unwrap_or_else(|| "—".to_owned());
        rows.push(EvidenceRow {
            refname,
            group: "phase-cut",
            tip,
        });
    }
    Ok(rows)
}

/// Enumerate the full ref names matching `pattern` (a `for-each-ref` glob, e.g.
/// `refs/heads/phase/068-*`), sorted by git's default (lexical). Empty when none
/// match. Impure shell.
fn for_each_ref(root: &Path, pattern: &str) -> anyhow::Result<Vec<String>> {
    let out = git::git_text(root, &["for-each-ref", "--format=%(refname)", pattern])?;
    Ok(out.lines().map(str::to_owned).collect())
}

/// The persisted status token for a candidate row (read view only).
fn status_token(status: CandidateStatus) -> &'static str {
    match status {
        CandidateStatus::Created => "created",
        CandidateStatus::Conflicted => "conflicted",
        CandidateStatus::Abandoned => "abandoned",
        CandidateStatus::Superseded => "superseded",
    }
}

/// Build one cell-row of owned strings from string slices.
fn cells(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_string()).collect()
}

/// EX-3: print the safe NEXT command(s) — concrete verbs the user runs, not
/// "inspect the raw refs". Guidance branches on ledger state: no candidates ⇒
/// create; candidates present ⇒ admit/close guidance; any drift ⇒ a re-admit
/// note (the admitted oid is immutable; a moved tip needs a fresh candidate).
fn write_next_commands(slice3: &str, ledger: &Candidates, any_drift: bool) -> anyhow::Result<()> {
    let slice = slice3.trim_start_matches('0');
    let slice = if slice.is_empty() { "0" } else { slice };
    writeln!(io::stdout(), "\nnext:")?;
    if ledger.rows.is_empty() {
        writeln!(
            io::stdout(),
            "  dispatch candidate create --slice {slice} --role review_surface \
             --payload impl_bundle --base refs/heads/main --label review-001 --worktree"
        )?;
        return Ok(());
    }
    writeln!(
        io::stdout(),
        "  dispatch candidate create --slice {slice} ...   # publish a fresh candidate"
    )?;
    writeln!(
        io::stdout(),
        "  dispatch candidate admit --slice {slice} --id <candidate-id> --review RV-NNN   \
         # pin a candidate for review/close"
    )?;
    if any_drift {
        writeln!(
            io::stdout(),
            "  note: a DRIFTED candidate's live tip moved off its recorded/admitted oid \
             (immutable) — supersede with a fresh candidate rather than editing in place"
        )?;
    }
    Ok(())
}

/// Resolve a commit-ish ref to its commit oid, or `None` when it does not exist.
fn resolve_commit(root: &Path, refish: &str) -> anyhow::Result<Option<String>> {
    Ok(git::git_opt(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{refish}^{{commit}}"),
        ],
    )?)
}

/// The tree oid of a commit.
fn tree_of(root: &Path, commit: &str) -> anyhow::Result<String> {
    Ok(git::git_text(
        root,
        &["rev-parse", &format!("{commit}^{{tree}}")],
    )?)
}

/// PHASE-05 (ISS-052) projection-source guard predicate (design §5.2 / D11).
///
/// The committed boundaries ledger holds **only funnel phases**; `plan_phases`
/// projects a per-phase cut for each. A funnel phase whose committed-ledger row
/// was lost (coord worktree removed before prepare-review, a partial working
/// ledger) under-projects *silently* — yet the funnel double-write already wrote
/// its **registry** row, so `registry_completeness` still passes. Provenance is
/// the discriminator: every registry row that is **not** positively solo/manual
/// (`Funnel`, or legacy `Unknown` we cannot clear) must have a committed-ledger
/// row. `Solo` (the binding) and `Manual` (the record-delta escape hatch, which
/// never asserts a ledger row exists) are excluded. Pure: a phase-id set compare,
/// never a code-delta diff (the pass-5 reshape deleted that path).
fn missing_committed_funnel_phases<'a>(
    registry: &'a [BoundaryRow],
    committed: &BTreeSet<&str>,
) -> Vec<&'a str> {
    registry
        .iter()
        .filter(|r| matches!(r.provenance, Provenance::Funnel | Provenance::Unknown))
        .map(|r| r.phase.as_str())
        .filter(|p| !committed.contains(p))
        .collect()
}

/// Stage-1 prepare-review (design §4.2 B + §4.3 C).
fn prepare_review(root: &Path, slice: u32) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");
    let coord_ref = format!("refs/heads/dispatch/{slice3}");
    let journal_path = format!(".doctrine/dispatch/{slice3}/journal.toml");

    let tip0 = resolve_commit(root, &coord_ref)?
        .with_context(|| format!("prepare-review: dispatch/{slice3} does not exist"))?;
    // (ISS-039, design §5.2 step 1) Splice the live coord worktree's UNCOMMITTED
    // boundaries ledger onto the tip BEFORE any read, mirroring `commit_journal`,
    // so `read_ledger`/`plan_phases` (and the PHASE-05 derive) read one committed,
    // checkout-independent source (SPEC-022-legal, D7/D10). No-op when there is no
    // live coord worktree or no working file (D9 liveness wrapper); content-
    // idempotent, so a re-run with identical content does not advance the ref.
    let tip = match git::live_worktree_for_ref(root, &coord_ref)? {
        Some(coord) => commit_boundaries(root, &tip0, &coord_ref, &coord, slice)?,
        None => tip0,
    };
    let tip_tree = tree_of(root, &tip)?;
    // Project off the PINNED FORK-POINT — merge-base(dispatch/<slice>, trunk) —
    // not the live trunk tip (RV-030 F-1, design §4.2/§4.3 trunk_base_B). The
    // coordination worktree isolates the working tree, NOT the trunk ref: a
    // foreign commit landing on trunk between `coordinate` and `sync` must not
    // reparent the per-phase cuts, else their diffs stop being exact and the
    // §3/IMP-043 "integrate refuses non-ff" net is silently bypassed. The live
    // tip resurfaces only at integrate's actual trunk push, under CAS.
    let trunk_tip = git::trunk_commit(root)?
        .context("prepare-review: no trunk ref resolves — a trunk base is required")?;
    let trunk_base = git::merge_base(root, &tip, &trunk_tip)?.with_context(|| {
        format!(
            "prepare-review: dispatch/{slice3} and trunk ({trunk_tip}) share no common ancestor"
        )
    })?;

    // --- source the run ledger from the dispatch tip (object db, not the
    //     working tree — works stage-1 and stage-2; design §4.1) --------------
    let orthogonal = read_ledger::<Orthogonal>(root, &coord_ref, &slice3, "orthogonal.toml")?;
    let boundaries = read_ledger::<Boundaries>(root, &coord_ref, &slice3, "boundaries.toml")?;

    // --- PHASE-05 (ISS-052): guard → derive → gate, ALL before the ref
    //     projection (the ordering is load-bearing — a halt creates no refs, so
    //     the operator's record-delta → re-run collides with nothing; design
    //     §5.2 steps 3–5 / D11 / F1). All three root on the PRIMARY tree so a
    //     coordination-worktree cwd still reads/writes the registry the
    //     integrator consumes. ----------------------------------------------------
    let primary = git::primary_worktree(root)?;

    // (3) projection-source guard (D11) — read the primary registry PRE-DERIVE:
    //     a funnel/legacy row with no committed-ledger counterpart would
    //     under-project silently (plan_phases emits no cut for it).
    let registry = crate::state::read_source_deltas(&primary, slice)?;
    let committed: BTreeSet<&str> = boundaries.rows.iter().map(|r| r.phase.as_str()).collect();
    let missing = missing_committed_funnel_phases(&registry, &committed);
    if !missing.is_empty() {
        bail!(
            "prepare-review: committed boundaries ledger is missing phase(s) {missing:?} on \
             dispatch/{slice3} that the registry records as funnel-owned (or legacy/unclassified). \
             The registry has them but the dispatch ref does not — the coordination worktree was \
             likely removed before prepare-review, or these are pre-provenance rows. Re-run with \
             the coord worktree present (it persists until integrate), or record-delta + COMMIT \
             the ledger for the named phase(s)."
        );
    }

    // (4) derive: upsert each committed-ledger row (Funnel) into the primary
    //     registry — fills a missing row, overwrites a binding mis-capture.
    for row in &boundaries.rows {
        crate::state::record_source_delta(&primary, slice, row.clone())?;
    }

    // (5) gate: primary-rooted completeness (both the completed-set and the
    //     registry resolve against `primary`) — bail BEFORE projection on any gap.
    if let crate::state::Completeness::Incomplete { gaps } =
        crate::state::registry_completeness(&primary, &primary, slice)?
    {
        let detail = gaps
            .iter()
            .map(crate::state::CompletenessGap::describe)
            .collect::<Vec<_>>()
            .join("; ");
        bail!(
            "prepare-review: conformance registry incomplete: {detail}; \
             record-delta the missing phase(s) before audit"
        );
    }

    // --- compute projections (objects only; no ref mutation yet) ------------
    let mut planned: Vec<Planned> = Vec::new();
    plan_review(
        root,
        &slice3,
        &tip,
        &tip_tree,
        &trunk_base,
        &orthogonal,
        &mut planned,
    )?;
    plan_phases(root, &slice3, &trunk_base, &boundaries, &mut planned)?;

    // --- EX-2: journal intent committed onto the branch BEFORE any external
    //     ref mutation; apply the external ref creations under zero-oid CAS
    //     (EX-5); record applied status back (recoverability) -------------------
    let mut journal = pending_journal(&planned);
    let outcomes = with_journaled_projection(
        root,
        &tip,
        &tip_tree,
        &journal_path,
        &coord_ref,
        &mut journal,
        "journal: prepare-review",
        |root, row| match git::update_ref_cas(
            root,
            &row.target_ref,
            &row.planned_new_oid,
            ZERO_OID,
        )? {
            RefCas::Updated => {
                row.status = LedgerStatus::Verified;
                row.applied_new_oid = row.planned_new_oid.clone();
                writeln!(io::stdout(), "{}", row.target_ref)?;
                Ok(RowOutcome::Done {
                    disposition: Disposition::Created,
                })
            }
            RefCas::Moved { actual } => {
                row.status = LedgerStatus::Failed;
                Ok(RowOutcome::Refused {
                    token: format!(
                        "{} (exists at {})",
                        row.target_ref,
                        actual.as_deref().unwrap_or("?")
                    ),
                })
            }
        },
    )?;

    let stale: Vec<String> = outcomes
        .into_iter()
        .filter_map(|o| match o {
            RowOutcome::Refused { token } => Some(token),
            RowOutcome::Done { .. } => None,
        })
        .collect();
    if stale.is_empty() {
        writeln!(
            io::stderr(),
            "prepare-review: {} ref(s) created",
            journal.rows.len()
        )?;
        Ok(())
    } else {
        bail!(
            "prepare-review: {} stale ref(s) reported, not clobbered: {}",
            stale.len(),
            stale.join(", ")
        )
    }
}

/// Stage-2 integrate (design §4 / §4.3). Sources the prepared journal from the
/// `dispatch/<slice>` tip tree (object db — works after the coordination worktree
/// is removed, EX-1), then **replays every row idempotently** under the 3-way CAS
/// ([`git::replay_ref`]): an intact prepared ref is a verified no-op, a clobbered
/// one is refused. When opted in, it appends and replays projection rows that
/// advance the audited code units onto `trunk` (ff-only, EX-3) and an aggregate
/// `edge` ref (EX-4). Plumbing-only — no checkout; the journal intent commits onto
/// the branch BEFORE any external ref mutation and the applied status commits back
/// after (EX-5). A moved target is reported, never clobbered (no auto-resolve).
fn integrate(
    root: &Path,
    slice: u32,
    trunk: Option<&str>,
    edge: Option<&str>,
) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");
    let coord_ref = format!("refs/heads/dispatch/{slice3}");
    let journal_path = format!(".doctrine/dispatch/{slice3}/journal.toml");

    let tip = resolve_commit(root, &coord_ref)?
        .with_context(|| format!("integrate: dispatch/{slice3} does not exist"))?;
    let tip_tree = tree_of(root, &tip)?;

    // Stage-1 must have prepared the journal (tree-read, never the filesystem —
    // it would silently empty from the parent/root, see the sync-tree-reads-ledger
    // memory). An empty journal ⇒ prepare-review never ran.
    let mut journal = read_ledger::<Journal>(root, &coord_ref, &slice3, "journal.toml")?;
    if journal.rows.is_empty() {
        bail!("integrate: no prepared journal on dispatch/{slice3} — run prepare-review first");
    }

    // --- SL-068 PHASE-06: a candidate workflow is "active for the slice" ⇔ the
    //     ledger carries ≥1 recorded candidate row. When active, --trunk/--edge
    //     source the ADMITTED oid (close_target / review_surface) and REFUSE
    //     rather than fall back to a raw phase/review ref (I6, I4, R4). When NOT
    //     active the legacy paths are preserved UNCHANGED (this is what keeps
    //     e2e_dispatch_sync.rs — which records no candidate — green). -----------
    let candidates = read_candidates(root, slice)?;
    let candidate_active = !candidates.rows.is_empty();

    // --- plan opt-in projection rows (idempotent: skip a target already
    //     journaled by a prior/crashed run — its recorded intent is replayed) ---
    let fresh = |j: &Journal, target: &str| !j.rows.iter().any(|r| r.target_ref == target);
    if let Some(trunk_ref) = trunk.filter(|t| fresh(&journal, t)) {
        let row = if candidate_active {
            plan_candidate_trunk_row(root, &candidates, trunk_ref)?
        } else {
            plan_trunk_row(root, &slice3, &journal, trunk_ref)?
        };
        journal.rows.push(row);
    }
    if let Some(edge_ref) = edge.filter(|e| fresh(&journal, e)) {
        let row = if candidate_active {
            plan_candidate_edge_row(root, &candidates, edge_ref)?
        } else {
            plan_edge_row(root, &slice3, edge_ref)?
        };
        journal.rows.push(row);
    }

    // --- §2.3/M4 dirty pre-gate: BEFORE the first commit_journal (which the
    //     bracket owns and which advances dispatch/<slice>). Any checked-out target
    //     with a DIRTY tracked tree refuses the WHOLE integrate with zero refs
    //     moved — incl. dispatch/<slice> (EX-1). Pre-existing dirt only; concurrent
    //     dirt is a raced-failure-after-advance (§7). Err early-return is correct:
    //     nothing is journaled yet. -----------------------------------------------
    for row in &journal.rows {
        if let Some(wt) = git::worktree_for_ref(root, &row.target_ref)?
            && !git::tree_clean(&wt)?
        {
            bail!("integrate-dirty-worktree ({})", row.target_ref);
        }
    }

    // --- journal the (possibly extended) intent onto the branch BEFORE any
    //     external ref mutation (EX-5, ADR-012 D4); advance every row idempotently
    //     — exact-CAS classification, worktree-aware mechanism (§2.2, EX-2..EX-5);
    //     record applied status back. ---------------------------------------------
    let outcomes = with_journaled_projection(
        root,
        &tip,
        &tip_tree,
        &journal_path,
        &coord_ref,
        &mut journal,
        "journal: integrate",
        advance_row,
    )?;

    report_integrate(&journal, &outcomes)
}

/// Advance one journal row to its planned oid — integrate's worktree-aware apply
/// closure (design §2.2, EX-2..EX-5). Classification is the EXACT `replay_ref`
/// predicate (`current == planned` → no-op; `current != expected_old` → moved;
/// else advance); only the *mechanism* of the advance branches on the target's
/// checkout state. A semantic refusal sets `row.status = Failed` and returns
/// `Ok(RowOutcome::Refused)` (the post-loop recovery commit makes it durable, B3);
/// `Err` is reserved for genuine plumbing failure.
fn advance_row(root: &Path, row: &mut JournalRow) -> anyhow::Result<RowOutcome> {
    let actual = resolve_commit(root, &row.target_ref)?;
    let current = actual.as_deref().unwrap_or(ZERO_OID);
    let planned = row.planned_new_oid.clone();
    let expected_old = row.expected_old_oid.clone();

    if current == planned {
        row.status = LedgerStatus::Verified;
        row.applied_new_oid = planned;
        return Ok(RowOutcome::Done {
            disposition: Disposition::NoOp,
        });
    }
    if current != expected_old {
        row.status = LedgerStatus::Failed;
        return Ok(RowOutcome::Refused {
            token: format!(
                "{} (target at {})",
                row.target_ref,
                actual.as_deref().unwrap_or("?")
            ),
        });
    }

    // current == expected_old → a real advance. The ONLY place the mechanism
    // branches on checkout state.
    match git::worktree_for_ref(root, &row.target_ref)? {
        None => advance_pure_ref(root, row, &planned, &expected_old),
        Some(wt) => advance_checked_out(root, row, &wt, &planned, &expected_old),
    }
}

/// The not-checked-out leg: pure `update_ref_cas`, CAS-and-done. Under Doctrine's
/// dispatch posture the delivery ref is never checked out, so a successful CAS
/// needs no worktree resync (SL-157, superseding SL-121 §2.2).
fn advance_pure_ref(
    root: &Path,
    row: &mut JournalRow,
    planned: &str,
    expected_old: &str,
) -> anyhow::Result<RowOutcome> {
    match git::update_ref_cas(root, &row.target_ref, planned, expected_old)? {
        RefCas::Moved { actual } => {
            row.status = LedgerStatus::Failed;
            Ok(RowOutcome::Refused {
                token: format!(
                    "{} (target at {})",
                    row.target_ref,
                    actual.as_deref().unwrap_or("?")
                ),
            })
        }
        RefCas::Updated => {
            // Not-checked-out advances are pure ref CAS only. Do NOT re-probe and
            // resync a worktree after CAS: under Doctrine's dispatch posture the
            // delivery ref is never checked out, and the post-CAS resync was the
            // RacedDesync / IMP-122 hazard (SL-157).
            row.status = LedgerStatus::Verified;
            planned.clone_into(&mut row.applied_new_oid);
            Ok(RowOutcome::Done {
                disposition: Disposition::AdvancedPureRef,
            })
        }
    }
}

/// The checked-out leg: a fast-forward advance (`expected_old` is an ancestor of
/// `planned`) syncs ref+index+worktree together via `merge --ff-only` under the
/// §2.5 race guard; a non-ff advance on a live ref REFUSES `integrate-nonff-checkout`
/// rather than `reset --hard` a checked-out ref (data loss, B2).
fn advance_checked_out(
    root: &Path,
    row: &mut JournalRow,
    wt: &Path,
    planned: &str,
    expected_old: &str,
) -> anyhow::Result<RowOutcome> {
    if git::is_ancestor(root, expected_old, planned)? {
        match git::ff_advance_in_worktree(wt, &row.target_ref, planned)? {
            git::FfAdvance::Advanced => {
                row.status = LedgerStatus::Verified;
                planned.clone_into(&mut row.applied_new_oid);
                Ok(RowOutcome::Done {
                    disposition: Disposition::AdvancedResynced,
                })
            }
            git::FfAdvance::Raced { token } => {
                row.status = LedgerStatus::Failed;
                Ok(RowOutcome::Refused {
                    token: format!("{} ({token})", row.target_ref),
                })
            }
        }
    } else {
        row.status = LedgerStatus::Failed;
        Ok(RowOutcome::Refused {
            token: format!("integrate-nonff-checkout ({})", row.target_ref),
        })
    }
}

/// Render the integrate outcome (design §4 / IMP-078): the existing machine-readable
/// stdout ref-list (every applied row, byte-for-byte as before) PLUS a per-row
/// stderr disposition line. A refusal bails (moved/raced targets reported, never
/// clobbered). Reads `(row, outcome)` pairs in row order.
fn report_integrate(journal: &Journal, outcomes: &[RowOutcome]) -> anyhow::Result<()> {
    let mut applied_refs: Vec<String> = Vec::new();
    let mut detail: Vec<String> = Vec::new();
    let mut refusals: Vec<String> = Vec::new();

    for (row, outcome) in journal.rows.iter().zip(outcomes) {
        match outcome {
            RowOutcome::Done { disposition } => match disposition {
                Disposition::NoOp => {
                    detail.push(format!("integrate: {} (no-op)", row.target_ref));
                }
                disp => {
                    applied_refs.push(row.target_ref.clone());
                    detail.push(format!(
                        "integrate: {} {}..{} ({})",
                        row.target_ref,
                        short_oid(&row.expected_old_oid),
                        short_oid(&row.applied_new_oid),
                        disp.label(),
                    ));
                }
            },
            RowOutcome::Refused { token } => refusals.push(token.clone()),
        }
    }

    // stdout: the changed-ref list contract (scripts consume it) — unchanged shape.
    for refname in &applied_refs {
        writeln!(io::stdout(), "{refname}")?;
    }
    // stderr: additive per-row human detail.
    for line in &detail {
        writeln!(io::stderr(), "{line}")?;
    }

    if refusals.is_empty() {
        writeln!(
            io::stderr(),
            "integrate: {} ref(s) replayed",
            journal.rows.len()
        )?;
        Ok(())
    } else {
        bail!(
            "integrate: {} moved target(s), not clobbered: {}",
            refusals.len(),
            refusals.join(", ")
        )
    }
}

/// The highest-numbered `refs/heads/phase/<slice>-NN` target in the journal — the
/// cumulative code tip (phase branches are chained off the trunk base, so the max
/// NN holds all prior phases' code). Only **verified** rows count: a failed phase
/// projection must not be mistaken for the chain tip (RV-030 F-8), else integrate
/// would parent the trunk advance on an unresolved ref. `None` when no verified
/// phase row was projected.
fn phase_chain_tip(journal: &Journal, slice3: &str) -> Option<String> {
    let prefix = format!("refs/heads/phase/{slice3}-");
    journal
        .rows
        .iter()
        .filter(|r| r.status == LedgerStatus::Verified)
        .filter_map(|r| {
            r.target_ref
                .strip_prefix(&prefix)
                .and_then(|nn| nn.parse::<u32>().ok())
                .map(|n| (n, r.target_ref.clone()))
        })
        .max_by_key(|(n, _)| *n)
        .map(|(_, refname)| refname)
}

/// Plan the trunk projection row (EX-3): the cumulative code tip advances
/// `trunk_ref` **fast-forward-only**. `expected_old` is the trunk tip (zero if the
/// ref is absent); a planned commit that does not descend from it ⇒ the trunk
/// moved ⇒ refuse (re-anchor is reported, never auto-resolved).
fn plan_trunk_row(
    root: &Path,
    slice3: &str,
    journal: &Journal,
    trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let phase_ref = phase_chain_tip(journal, slice3).with_context(|| {
        format!("integrate --trunk: no phase/{slice3}-NN code units to integrate")
    })?;
    let planned = resolve_commit(root, &phase_ref)?
        .with_context(|| format!("integrate --trunk: {phase_ref} does not resolve"))?;
    let expected_old = resolve_commit(root, trunk_ref)?;
    if let Some(tip) = &expected_old {
        anyhow::ensure!(
            git::is_ancestor(root, tip, &planned)?,
            "integrate --trunk: {planned} does not fast-forward {trunk_ref} (at {tip}) — \
             trunk moved; re-anchor required, not auto-resolved"
        );
    }
    Ok(projection_row(trunk_ref, planned, expected_old))
}

/// Plan the edge aggregate row (EX-4): the `review/<slice>` impl bundle advances
/// the standing `edge_ref`. Not ff-gated (a standing aggregate of local work); the
/// CAS still refuses a concurrently-moved edge — isolated to this sync point.
fn plan_edge_row(root: &Path, slice3: &str, edge_ref: &str) -> anyhow::Result<JournalRow> {
    let review_ref = format!("refs/heads/review/{slice3}");
    let planned = resolve_commit(root, &review_ref)?
        .with_context(|| format!("integrate --edge: {review_ref} does not resolve"))?;
    let expected_old = resolve_commit(root, edge_ref)?;
    Ok(projection_row(edge_ref, planned, expected_old))
}

/// SL-068 PHASE-06 — plan the trunk row when a candidate workflow is active: the
/// admitted **`close_target`** OID advances `trunk_ref` fast-forward-only, sourced
/// from the ledger (never a close-time merge, I6). Targeting is by `admitted_oid`
/// only — moving the candidate ref after admission cannot change the target (I4).
/// REFUSES (no fallback to the phase-chain tip) when no `close_target` admission
/// exists; on a non-ff trunk it refuses and instructs the user to create a
/// superseding close-target candidate on the new base (EX-2, R4 — no auto-reanchor).
fn plan_candidate_trunk_row(
    root: &Path,
    candidates: &Candidates,
    trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let admission = candidates.current_admission.close_target.as_ref().context(
        "integrate --trunk: a candidate workflow is active but no close_target admission \
             exists — run `dispatch candidate admit --role close_target` first; integrate will \
             not fall back to a raw phase ref",
    )?;
    let planned = admission.admitted_oid.clone();
    let expected_old = resolve_commit(root, trunk_ref)?;
    if let Some(tip) = &expected_old {
        anyhow::ensure!(
            git::is_ancestor(root, tip, &planned)?,
            "integrate --trunk: admitted close_target {planned} does not fast-forward {trunk_ref} \
             (at {tip}) — trunk moved; create a superseding close-target candidate on the new \
             base and re-admit (not auto-resolved)"
        );
    }
    Ok(projection_row(trunk_ref, planned, expected_old))
}

/// SL-068 PHASE-06 — plan the edge row when a candidate workflow is active: the
/// admitted **`review_surface`** OID advances `edge_ref`, sourced from the ledger.
/// Same posture as the legacy edge (not ff-gated; the CAS still guards). REFUSES
/// (no silent raw `review/<slice>` fallback) when no `review_surface` admission
/// exists. Targeting is by `admitted_oid` only (I4).
fn plan_candidate_edge_row(
    root: &Path,
    candidates: &Candidates,
    edge_ref: &str,
) -> anyhow::Result<JournalRow> {
    let admission = candidates
        .current_admission
        .review_surface
        .as_ref()
        .context(
            "integrate --edge: a candidate workflow is active but no review_surface admission \
             exists — run `dispatch candidate admit --role review_surface` first; integrate will \
             not fall back to the raw review ref",
        )?;
    let planned = admission.admitted_oid.clone();
    let expected_old = resolve_commit(root, edge_ref)?;
    Ok(projection_row(edge_ref, planned, expected_old))
}

/// A pending CAS journal row advancing `target_ref` to `planned` from its current
/// tip (`expected_old`, zero-oid for a ref creation). `source_oid == planned_new_oid`
/// is **intentional** for these direct-projection (trunk/edge) rows — the source
/// IS the planned ref, so replay recomputes identity and converges to a no-op
/// (RV-030 F-10); unlike prepare-review rows where source (dispatch tip) and the
/// synthesised commit differ.
fn projection_row(target_ref: &str, planned: String, expected_old: Option<String>) -> JournalRow {
    JournalRow {
        source_oid: planned.clone(),
        target_ref: target_ref.to_owned(),
        expected_old_oid: expected_old.unwrap_or_else(|| ZERO_OID.to_owned()),
        planned_new_oid: planned,
        applied_new_oid: String::new(),
        status: LedgerStatus::Pending,
    }
}

/// Read a run-ledger manifest from the `dispatch/<slice>` tip tree (object db,
/// not the working filesystem). Absent ⇒ the type's empty default.
fn read_ledger<T: serde::de::DeserializeOwned + Default>(
    root: &Path,
    coord_ref: &str,
    slice3: &str,
    file: &str,
) -> anyhow::Result<T> {
    let path = format!(".doctrine/dispatch/{slice3}/{file}");
    match git::read_path_at(root, coord_ref, &path)? {
        Some(text) => Ok(toml::from_str(&text)?),
        None => Ok(T::default()),
    }
}

/// B — plan `review/<slice>`: filter the tip tree (drop the run-ledger dir and
/// every journal-verified orthogonal path) and commit it against the trunk base.
fn plan_review(
    root: &Path,
    slice3: &str,
    tip: &str,
    tip_tree: &str,
    trunk_base: &str,
    orthogonal: &Orthogonal,
    planned: &mut Vec<Planned>,
) -> anyhow::Result<()> {
    let mut exclude: Vec<String> = vec![format!(".doctrine/dispatch/{slice3}")];
    for mark in &orthogonal.rows {
        if mark.status == LedgerStatus::Verified {
            exclude.push(mark.path.clone());
        }
    }
    let exclude_refs: Vec<&str> = exclude.iter().map(String::as_str).collect();
    let review_tree = git::filter_tree(root, tip_tree, &exclude_refs)?;
    let review_commit = git::commit_tree(
        root,
        &review_tree,
        trunk_base,
        &format!("review({slice3}): impl bundle"),
    )?;
    planned.push(Planned {
        target_ref: format!("refs/heads/review/{slice3}"),
        source_oid: tip.to_owned(),
        commit_oid: review_commit,
    });
    Ok(())
}

/// C — plan `phase/<slice>-NN` from `boundaries.toml`: each emitted phase is the
/// code-only (`.doctrine/` stripped) cut of its cumulative `code_end_oid` tree,
/// chained off the previous phase (trunk base for the first). Empty-code phases
/// (`code_start_oid == code_end_oid`) emit no ref.
fn plan_phases(
    root: &Path,
    slice3: &str,
    trunk_base: &str,
    boundaries: &Boundaries,
    planned: &mut Vec<Planned>,
) -> anyhow::Result<()> {
    let mut parent = trunk_base.to_owned();
    for boundary in &boundaries.rows {
        if boundary.code_start_oid == boundary.code_end_oid {
            continue; // empty-code phase — no branch cut (design §4.3)
        }
        let nn = boundary
            .phase
            .strip_prefix("PHASE-")
            .unwrap_or(&boundary.phase);
        let code_tree = tree_of(root, &boundary.code_end_oid)?;
        let phase_tree = git::filter_tree(root, &code_tree, &[".doctrine"])?;
        let phase_commit =
            git::commit_tree(root, &phase_tree, &parent, &format!("phase({slice3}-{nn})"))?;
        planned.push(Planned {
            target_ref: format!("refs/heads/phase/{slice3}-{nn}"),
            source_oid: boundary.code_end_oid.clone(),
            commit_oid: phase_commit.clone(),
        });
        parent = phase_commit;
    }
    Ok(())
}

/// Build the pending-intent journal (one row per planned ref, all CAS creations).
fn pending_journal(planned: &[Planned]) -> Journal {
    Journal {
        rows: planned
            .iter()
            .map(|p| JournalRow {
                source_oid: p.source_oid.clone(),
                target_ref: p.target_ref.clone(),
                expected_old_oid: ZERO_OID.to_owned(),
                planned_new_oid: p.commit_oid.clone(),
                applied_new_oid: String::new(),
                status: LedgerStatus::Pending,
            })
            .collect(),
    }
}

/// Commit `journal` onto `dispatch/<slice>` by splicing `journal.toml` into the
/// tip tree and advancing the branch under CAS (no checkout). `base_tree` is the
/// impl tip tree; `parent` is the branch's current tip — by construction both the
/// new commit's parent AND the CAS expected-old (always identical). `msg` is the
/// stage-distinct commit message (`journal: prepare-review` / `journal: integrate`,
/// RV-030 F-4). Returns the new branch commit oid.
fn commit_journal(
    root: &Path,
    base_tree: &str,
    parent: &str,
    journal_path: &str,
    coord_ref: &str,
    journal: &Journal,
    msg: &str,
) -> anyhow::Result<String> {
    let body = journal.to_toml()?;
    let tree = git::tree_with_file(root, base_tree, journal_path, &body)?;
    let commit = git::commit_tree(root, &tree, parent, msg)?;
    match git::update_ref_cas(root, coord_ref, &commit, parent)? {
        RefCas::Updated => Ok(commit),
        RefCas::Moved { actual } => bail!(
            "journal-commit: dispatch branch moved under us (expected {parent}, found {})",
            actual.as_deref().unwrap_or("?")
        ),
    }
}

/// Splice the **uncommitted** working boundaries ledger from the live coordination
/// worktree onto `dispatch/<slice>` (design §5.2 step 1, ISS-039) — the boundaries
/// twin of [`commit_journal`], with two hardenings over a naive byte splice:
///
/// 1. **Validate before commit (F3).** The working file is parsed to [`Boundaries`];
///    a malformed ledger is a clean `Err` and the tip is left untouched — never
///    commit garbage, unlike a verbatim-byte splice.
/// 2. **Content-idempotent via TREE-oid compare (F1).** The parsed ledger is
///    re-serialized to canonical TOML and spliced into the tip tree; if the
///    candidate tree equals the current tip tree the ref is **not** advanced
///    (returns `parent`). git dedups identical content to the same blob — hence
///    the same tree — so a TREE compare is formatting-immune where a raw-blob
///    compare would falsely diff.
///
/// `parent` is the branch's current tip (both the new commit's parent and the CAS
/// expected-old). Absent working file ⇒ no-op (`parent`), so a re-run after the
/// coord worktree's removal is safe. A moved ref bails like `commit_journal` (R6).
fn commit_boundaries(
    root: &Path,
    parent: &str,
    coord_ref: &str,
    coord: &git::WorktreeEntry,
    slice: u32,
) -> anyhow::Result<String> {
    let Some(raw) = crate::ledger::read_boundaries_file(&coord.path, slice)? else {
        return Ok(parent.to_owned()); // no working ledger to splice
    };
    let boundaries = Boundaries::parse(&raw).with_context(|| {
        format!("commit_boundaries: working boundaries.toml for dispatch/{slice:03} is malformed")
    })?;
    let canonical = boundaries.to_toml()?;
    let path = format!(".doctrine/dispatch/{slice:03}/boundaries.toml");
    let tip_tree = tree_of(root, parent)?;
    let candidate = git::tree_with_file(root, &tip_tree, &path, &canonical)?;
    if candidate == tip_tree {
        return Ok(parent.to_owned()); // identical content — no ref advance (F1)
    }
    let commit = git::commit_tree(root, &candidate, parent, "ledger: boundaries")?;
    match git::update_ref_cas(root, coord_ref, &commit, parent)? {
        RefCas::Updated => Ok(commit),
        RefCas::Moved { actual } => bail!(
            "commit_boundaries: dispatch branch moved under us (expected {parent}, found {})",
            actual.as_deref().unwrap_or("?")
        ),
    }
}

/// The per-row disposition of a successful apply. Transient REPORT data — NOT a
/// [`JournalRow`] field: the row schema carries only oids + status, and every
/// success persists as [`LedgerStatus::Verified`], so the disposition cannot be
/// recovered from the row after the fact. The caller renders output from these
/// (SL-121 §4 / IMP-078).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Disposition {
    /// A zero-oid creation succeeded (prepare-review).
    Created,
    /// A replay found the target already at the planned oid (integrate).
    NoOp,
    /// A checked-out target fast-forwarded in its live worktree via
    /// `merge --ff-only` — ref + index + worktree all at the planned oid
    /// (integrate, §2.2 checked-out leg).
    AdvancedResynced,
    /// A not-checked-out target advanced by pure `update_ref_cas`; no worktree to
    /// sync (integrate, §2.2 None leg).
    AdvancedPureRef,
}

impl Disposition {
    /// The exact report token (SL-121 §4). Tests assert these literally — do NOT
    /// paraphrase.
    fn label(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::NoOp => "no-op",
            Self::AdvancedResynced => "advanced+resynced",
            Self::AdvancedPureRef => "advanced+pure-ref",
        }
    }
}

/// Per-row outcome the apply closure hands back. The bracket collects these and
/// returns them; the CALLER renders output and bails from the vec.
#[derive(Debug, Clone, PartialEq, Eq)]
enum RowOutcome {
    /// The row applied successfully with the given disposition.
    Done { disposition: Disposition },
    /// A semantic refusal (moved/stale target) — the row was journaled
    /// [`LedgerStatus::Failed`] inside the closure; `token` is the caller's
    /// report fragment.
    Refused { token: String },
}

/// Journal the planned intent onto `coord_ref` BEFORE any external ref mutation,
/// apply each row via `apply`, then re-journal the applied status so a crashed
/// run is recoverable. The bracket owns ONLY the two [`commit_journal`] calls and
/// the per-row loop; construction stays caller-side before, report-or-bail
/// caller-side after.
///
/// The recovery [`commit_journal`] runs STRICTLY AFTER the loop, so a `?`-`Err`
/// out of `apply` aborts BEFORE applied status is recorded. `apply` must
/// therefore return `Err` ONLY for fatal operational failure; every semantic
/// per-row refusal sets `row.status = Failed` inside the closure and returns
/// `Ok(RowOutcome::Refused { .. })` so the post-loop commit durably records it.
#[expect(
    clippy::too_many_arguments,
    reason = "thin journal-cycle bracket threads the commit_journal arg set plus the apply closure"
)]
fn with_journaled_projection(
    root: &Path,
    tip: &str,
    tip_tree: &str,
    journal_path: &str,
    coord_ref: &str,
    journal: &mut Journal,
    message: &str,
    mut apply: impl FnMut(&Path, &mut JournalRow) -> anyhow::Result<RowOutcome>,
) -> anyhow::Result<Vec<RowOutcome>> {
    let journal_commit = commit_journal(
        root,
        tip_tree,
        tip,
        journal_path,
        coord_ref,
        journal,
        message,
    )?;
    let mut outcomes = Vec::with_capacity(journal.rows.len());
    for row in &mut journal.rows {
        outcomes.push(apply(root, row)?);
    }
    commit_journal(
        root,
        tip_tree,
        &journal_commit,
        journal_path,
        coord_ref,
        journal,
        message,
    )?;
    Ok(outcomes)
}

/// Render an ordered phase-status table. Pure formatting — caller owns data.
/// Designed for reuse by `plan-next` and `status` (PHASE-03).
pub(crate) fn render_phase_table(rows: &[(String, String, String)]) -> String {
    use comfy_table::Table;
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::NOTHING)
        .set_header(vec!["  ID", "  Status", "  Name"])
        .force_no_tty();
    for (id, status, name) in rows {
        table.add_row(vec![
            format!("  {id}"),
            format!("  {status}"),
            format!("  {name}"),
        ]);
    }
    // Trim trailing whitespace (comfy-table last-column cell-fill edge case)
    let out = table.to_string();
    out.lines()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// `doctrine dispatch plan-next` — read the plan and runtime phase sheets;
/// print an ordered phase rollup and identify the next actionable phase(s).
/// Read-only — callable from anywhere.
pub(crate) fn run_plan_next(path: Option<PathBuf>, slice: u32, json: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // 1. Read plan.toml
    let plan = crate::slice::read_plan(&root.join(".doctrine/slice"), slice)?;

    // 2. Read phase statuses from runtime state
    let state_dir = crate::state::phases_dir(&root, slice);

    // Build ordered phase+status list
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for ph in &plan.phases {
        let stem = ph.id.to_lowercase();
        let status = match crate::state::read_phase_status(&state_dir, &stem) {
            Ok(Some(s)) => s,
            Ok(None) => "pending".to_string(), // absent tracking file → pending
            Err(_) => "unknown".to_string(),
        };
        rows.push((ph.id.clone(), status, ph.name.clone()));
    }

    // 3. Compute `next`
    // Scan in plan order, skip completed/blocked.
    // First actionable in_progress → only that phase.
    // First actionable pending → that phase + consecutive pending.
    let mut next: Vec<String> = Vec::new();
    let mut found_actionable = false;
    let mut saw_blocked = false;

    for (id, status, _) in &rows {
        match status.as_str() {
            "completed" => {}
            "blocked" => {
                saw_blocked = true;
                if found_actionable {
                    break; // stop at blocked after we started collecting
                }
            }
            "in_progress" => {
                if !found_actionable {
                    next.push(id.clone());
                    break; // in_progress gates subsequent pending
                }
            }
            _ => {
                // pending or unknown
                if !found_actionable {
                    next.push(id.clone());
                    found_actionable = true;
                    // continue for consecutive pending
                } else if status.as_str() == "pending" {
                    next.push(id.clone());
                } else {
                    break; // non-pending stops the run
                }
            }
        }
    }

    // 4. Render output
    if json {
        #[derive(serde::Serialize)]
        struct PhaseRow {
            id: String,
            name: String,
            status: String,
        }
        #[derive(serde::Serialize)]
        struct Output {
            phases: Vec<PhaseRow>,
            next: Vec<String>,
            batching_requires_phase_plan: bool,
        }
        let output = Output {
            phases: rows
                .iter()
                .map(|(id, status, name)| PhaseRow {
                    id: id.clone(),
                    name: name.clone(),
                    status: status.clone(),
                })
                .collect(),
            next,
            batching_requires_phase_plan: true,
        };
        writeln!(io::stdout(), "{}", serde_json::to_string_pretty(&output)?)?;
    } else {
        // Human output
        let table = render_phase_table(&rows);
        writeln!(io::stdout(), "{table}")?;
        if next.is_empty() {
            if saw_blocked {
                writeln!(
                    io::stdout(),
                    "\nnext: (none — all remaining phases are blocked)"
                )?;
            }
        } else {
            let ids = next.join(", ");
            writeln!(io::stdout(), "\nnext: {ids}")?;
            writeln!(
                io::stdout(),
                "  ⚠ run /phase-plan before parallel spawn; do not assume file-disjointness"
            )?;
        }
    }

    Ok(())
}

/// Drift of a tip against current trunk (SL-127 §3.1).
struct Drift {
    /// The resolved trunk tip the drift was measured against (carried so callers
    /// that already resolved drift need not re-walk the trunk ladder).
    trunk_tip: String,
    fork_point: String,
    ahead: u32,
}

/// Drift of `tip` against current trunk: `fork_point` = `merge_base(tip, trunk)`,
/// `ahead` = `count(fork_point..trunk)`. Resolves the trunk tip itself via the
/// peeled ladder (a None trunk is a hard "trunk ref not found" error, preserving
/// `run_status`' observable behaviour). `Ok(None)` ⇒ tip and trunk share no
/// common ancestor (unrelated histories), which callers surface with their own
/// context. Parameterized on `tip` (F4) so the PHASE-04 classifier can measure the
/// bundle/source, not only the dispatch branch.
fn trunk_drift(root: &Path, tip: &str) -> anyhow::Result<Option<Drift>> {
    let trunk_tip = git::trunk_commit(root)?.with_context(|| "trunk ref not found")?;
    let Some(fork_point) = git::merge_base(root, tip, &trunk_tip)? else {
        return Ok(None);
    };
    let ahead_cnt = git::git_text(
        root,
        &["rev-list", "--count", &format!("{fork_point}..{trunk_tip}")],
    )?;
    let ahead: u32 = ahead_cnt.trim().parse().unwrap_or(0);
    Ok(Some(Drift {
        trunk_tip,
        fork_point,
        ahead,
    }))
}

/// `doctrine dispatch status` — read-only full dispatch rollup: coordination
/// state, phase table, trunk drift, sync state, candidate summary, next-step
/// guidance. Read-only — callable from anywhere.
pub(crate) fn run_status(path: Option<PathBuf>, slice: u32, json: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice3 = format!("{slice:03}");
    let dispatch_ref = format!("refs/heads/dispatch/{slice3}");

    // --- Coordination state ---------------------------------------------------
    let dispatch_tip = resolve_commit(&root, &dispatch_ref)?.with_context(|| {
        format!("dispatch branch not found; run 'dispatch setup --slice {slice}' first")
    })?;
    let dispatch_short = git::git_text(&root, &["rev-parse", "--short=7", &dispatch_tip])?;

    // Find live worktree via git worktree list --porcelain
    let coord_state = find_coordination_worktree(&root, &slice3);

    // --- Trunk drift -----------------------------------------------------------
    let Drift {
        trunk_tip,
        fork_point,
        ahead,
    } = trunk_drift(&root, &dispatch_tip)?
        .with_context(|| format!("dispatch/{slice3} and trunk share no common ancestor"))?;
    let trunk_state = if ahead == 0 { "stable" } else { "moved" };

    // --- Phase table -----------------------------------------------------------
    let plan = crate::slice::read_plan(&root.join(".doctrine/slice"), slice)?;
    let state_dir = crate::state::phases_dir(&root, slice);
    let mut phase_rows: Vec<(String, String, String)> = Vec::new();
    for ph in &plan.phases {
        let stem = ph.id.to_lowercase();
        let status = match crate::state::read_phase_status(&state_dir, &stem) {
            Ok(Some(s)) => s,
            Ok(None) => "pending".to_string(),
            Err(_) => "unknown".to_string(),
        };
        phase_rows.push((ph.id.clone(), status, ph.name.clone()));
    }

    // --- Sync state ------------------------------------------------------------
    let review_ref = format!("refs/heads/review/{slice3}");
    let review_exists = resolve_commit(&root, &review_ref)?.is_some();
    let phase_ref_count = count_phase_refs(&root, &slice3);

    // --- Candidate summary -----------------------------------------------------
    let candidates = read_candidates(&root, slice)?;
    let candidate_total = candidates.rows.len();
    let candidate_admitted = [
        candidates.current_admission.close_target.is_some(),
        candidates.current_admission.review_surface.is_some(),
    ]
    .into_iter()
    .filter(|&x| x)
    .count();

    // --- Next-step guidance ----------------------------------------------------
    let all_completed = phase_rows
        .iter()
        .all(|(_, status, _)| status == "completed");
    let coord_live = !matches!(coord_state.as_str(), "(removed)");
    let admitted_ct = candidates.current_admission.close_target.as_ref();

    // SL-127 EX-2 (§3.4): when all phases are complete, the prepared bundle's tip
    // is the `review/<NNN>` ref if it exists, else the pre-prepare dispatch tip.
    // If trunk has advanced past that tip (a computed fact — codex C6, not a flag),
    // the base is stale and refresh-base must run before prepare-review/audit.
    let review_tip = if review_exists {
        resolve_commit(&root, &review_ref)?.unwrap_or(dispatch_tip)
    } else {
        dispatch_tip
    };
    let bundle_stale = all_completed && trunk_drift(&root, &review_tip)?.map_or(0, |d| d.ahead) > 0;
    // The only git-touching leg (condition 5/6) is resolved here in the shell so
    // the decision itself stays pure + table-testable.
    let admitted_is_ancestor = match admitted_ct {
        Some(ct) if !coord_live => is_ancestor_of_trunk(&root, &ct.admitted_oid, &trunk_tip)?,
        _ => false,
    };

    let next_guidance = select_guidance(GuidanceInputs {
        all_completed,
        bundle_stale,
        review_exists,
        coord_live,
        admitted: admitted_ct.is_some(),
        admitted_is_ancestor,
        next_phases: || compute_next_phases(&phase_rows),
    });

    // --- Output ----------------------------------------------------------------
    if json {
        let output = StatusOutput {
            dispatch: DispatchState {
                r#ref: dispatch_ref,
                tip: dispatch_short,
            },
            coord: CoordState {
                state: if coord_live {
                    "live".to_string()
                } else {
                    "removed".to_string()
                },
                path: if coord_live { Some(coord_state) } else { None },
            },
            trunk: TrunkState {
                state: trunk_state.to_string(),
                fork_point,
                ahead,
            },
            phases: phase_rows
                .iter()
                .map(|(id, status, name)| PhaseState {
                    id: id.clone(),
                    name: name.clone(),
                    status: status.clone(),
                })
                .collect(),
            sync: SyncState {
                state: if review_exists {
                    "prepared".to_string()
                } else {
                    "not_prepared".to_string()
                },
                review_ref: if review_exists {
                    Some(review_ref)
                } else {
                    None
                },
                phase_cuts: phase_ref_count,
            },
            candidates: CandidateSummary {
                total: candidate_total,
                admitted: candidate_admitted,
            },
            next: next_guidance.to_json(),
        };
        writeln!(io::stdout(), "{}", serde_json::to_string_pretty(&output)?)?;
    } else {
        // Human output
        writeln!(io::stdout(), "dispatch: {dispatch_ref}  ({dispatch_short})")?;
        writeln!(io::stdout(), "coord:    {coord_state}")?;
        if ahead > 0 {
            writeln!(
                io::stdout(),
                "trunk:    {trunk_state} ({ahead} commit(s) ahead of fork-point)"
            )?;
        } else {
            writeln!(io::stdout(), "trunk:    {trunk_state}")?;
        }
        writeln!(io::stdout())?;
        writeln!(io::stdout(), "phases:")?;
        write!(io::stdout(), "{}", render_phase_table(&phase_rows))?;
        writeln!(io::stdout())?;
        writeln!(io::stdout())?;
        if review_exists {
            writeln!(
                io::stdout(),
                "sync:     prepared — {review_ref} ({phase_ref_count} phase cut(s))"
            )?;
        } else {
            writeln!(io::stdout(), "sync:     not yet run")?;
        }
        writeln!(
            io::stdout(),
            "candidates: {candidate_total} ({candidate_admitted} admitted)"
        )?;
        match &next_guidance {
            NextGuidance::Phases { phases } => {
                let ids = phases.join(", ");
                writeln!(io::stdout(), "next:     {ids}")?;
            }
            NextGuidance::RefreshBase => {
                writeln!(
                    io::stdout(),
                    "next:     trunk advanced past the prepared base — run 'dispatch refresh-base --slice {slice}' then re-prepare"
                )?;
            }
            NextGuidance::PrepareReview => {
                writeln!(
                    io::stdout(),
                    "next:     all phases completed — run 'dispatch sync --prepare-review'"
                )?;
            }
            NextGuidance::AuditThenIntegrate => {
                writeln!(
                    io::stdout(),
                    "next:     all phases completed — admitted candidate exists; run audit then 'dispatch sync --integrate'"
                )?;
            }
            NextGuidance::AuditOrCandidateStatus => {
                writeln!(
                    io::stdout(),
                    "next:     all phases completed — review ref prepared; run audit or 'dispatch candidate status'"
                )?;
            }
            NextGuidance::Complete => {
                writeln!(
                    io::stdout(),
                    "next:     complete — coordination worktree removed; slice is integrated"
                )?;
            }
            NextGuidance::AwaitingIntegration => {
                writeln!(
                    io::stdout(),
                    "next:     awaiting integration — run 'dispatch sync --integrate' after audit"
                )?;
            }
        }
    }

    Ok(())
}

/// The coordination worktree checked out on `dispatch/<slice3>`, or the
/// `"(removed)"` sentinel. Delegates to the shared [`git::worktree_for_ref`] probe
/// (SL-121 PHASE-01). The pre-extraction parser folded BOTH a git-command failure
/// AND an absent ref into `"(removed)"`; the probe splits those (`Err` vs
/// `Ok(None)`), so this wrapper folds both legs back to the sentinel to preserve
/// behaviour (F4).
fn find_coordination_worktree(root: &Path, slice3: &str) -> String {
    let target_branch = format!("refs/heads/dispatch/{slice3}");
    match git::worktree_for_ref(root, &target_branch) {
        Ok(Some(path)) => path.to_string_lossy().into_owned(),
        Ok(None) | Err(_) => "(removed)".to_string(),
    }
}

/// Count `refs/heads/phase/{slice3}-*` refs via `git for-each-ref`.
fn count_phase_refs(root: &Path, slice3: &str) -> usize {
    let pattern = format!("refs/heads/phase/{slice3}-*");
    let Ok(out) = git::git_text(root, &["for-each-ref", "--format=%(refname)", &pattern]) else {
        return 0;
    };
    if out.trim().is_empty() {
        0
    } else {
        out.lines().count()
    }
}

/// Compute next phases using same logic as plan-next.
fn compute_next_phases(rows: &[(String, String, String)]) -> Vec<String> {
    let mut next: Vec<String> = Vec::new();
    let mut found_actionable = false;
    for (id, status, _) in rows {
        match status.as_str() {
            "completed" => {}
            "blocked" => {
                if found_actionable {
                    break;
                }
            }
            "in_progress" => {
                if !found_actionable {
                    next.push(id.clone());
                    break;
                }
            }
            _ => {
                if !found_actionable {
                    next.push(id.clone());
                    found_actionable = true;
                } else if status.as_str() == "pending" {
                    next.push(id.clone());
                } else {
                    break;
                }
            }
        }
    }
    next
}

/// Check if `oid` is an ancestor of `trunk_tip` (or equal).
fn is_ancestor_of_trunk(root: &Path, oid: &str, trunk_tip: &str) -> anyhow::Result<bool> {
    if oid == trunk_tip {
        return Ok(true);
    }
    let mb = git::merge_base(root, oid, trunk_tip)?;
    Ok(mb.as_deref() == Some(oid))
}

// --- JSON output types -------------------------------------------------------

#[derive(serde::Serialize)]
struct StatusOutput {
    dispatch: DispatchState,
    coord: CoordState,
    trunk: TrunkState,
    phases: Vec<PhaseState>,
    sync: SyncState,
    candidates: CandidateSummary,
    next: NextJson,
}

#[derive(serde::Serialize)]
struct DispatchState {
    #[serde(rename = "ref")]
    r#ref: String,
    tip: String,
}

#[derive(serde::Serialize)]
struct CoordState {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(serde::Serialize)]
struct TrunkState {
    state: String,
    fork_point: String,
    ahead: u32,
}

#[derive(serde::Serialize)]
struct PhaseState {
    id: String,
    name: String,
    status: String,
}

#[derive(serde::Serialize)]
struct SyncState {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_ref: Option<String>,
    phase_cuts: usize,
}

#[derive(serde::Serialize)]
struct CandidateSummary {
    total: usize,
    admitted: usize,
}

#[derive(serde::Serialize)]
struct NextJson {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phases: Option<Vec<String>>,
}

/// Precomputed facts the next-step decision reads (all git/disk resolved in the
/// `run_status` shell). `next_phases` is a thunk so the (only) allocating leg runs
/// solely when phases remain.
struct GuidanceInputs<F: FnOnce() -> Vec<String>> {
    all_completed: bool,
    bundle_stale: bool,
    review_exists: bool,
    coord_live: bool,
    admitted: bool,
    admitted_is_ancestor: bool,
    next_phases: F,
}

/// The deterministic next-step state machine (design §3.4). Pure: every input is
/// precomputed. The `bundle_stale` (SL-127 EX-2) leg fires BEFORE `PrepareReview`
/// so a trunk that advanced past the prepared bundle routes to refresh-base, never
/// to prepare-review/audit on a stale base.
fn select_guidance<F: FnOnce() -> Vec<String>>(inputs: GuidanceInputs<F>) -> NextGuidance {
    let GuidanceInputs {
        all_completed,
        bundle_stale,
        review_exists,
        coord_live,
        admitted,
        admitted_is_ancestor,
        next_phases,
    } = inputs;
    if !all_completed {
        NextGuidance::Phases {
            phases: next_phases(),
        }
    } else if bundle_stale {
        NextGuidance::RefreshBase
    } else if !review_exists {
        NextGuidance::PrepareReview
    } else if coord_live && admitted {
        NextGuidance::AuditThenIntegrate
    } else if coord_live {
        NextGuidance::AuditOrCandidateStatus
    } else if admitted {
        if admitted_is_ancestor {
            NextGuidance::Complete
        } else {
            NextGuidance::AwaitingIntegration
        }
    } else {
        // Fallback (coord removed, nothing admitted — shouldn't normally reach).
        NextGuidance::AuditOrCandidateStatus
    }
}

/// The next-step guidance resolved from the deterministic state machine.
enum NextGuidance {
    Phases {
        phases: Vec<String>,
    },
    /// SL-127 EX-2: trunk advanced past the prepared bundle — refresh the base
    /// before prepare-review/audit.
    RefreshBase,
    PrepareReview,
    AuditThenIntegrate,
    AuditOrCandidateStatus,
    Complete,
    AwaitingIntegration,
}

impl NextGuidance {
    fn to_json(&self) -> NextJson {
        match self {
            NextGuidance::Phases { phases } => NextJson {
                kind: "phases".to_string(),
                phases: Some(phases.clone()),
            },
            NextGuidance::RefreshBase => NextJson {
                kind: "refresh_base".to_string(),
                phases: None,
            },
            NextGuidance::PrepareReview => NextJson {
                kind: "blocked".to_string(),
                phases: None,
            },
            NextGuidance::AuditThenIntegrate | NextGuidance::AuditOrCandidateStatus => NextJson {
                kind: "audit".to_string(),
                phases: None,
            },
            NextGuidance::Complete => NextJson {
                kind: "completed".to_string(),
                phases: None,
            },
            NextGuidance::AwaitingIntegration => NextJson {
                kind: "awaiting_integration".to_string(),
                phases: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn init_repo(dir: &Path) {
        std::fs::create_dir_all(dir).unwrap();
        git(dir, &["init", "-q", "-b", "main"]);
        git(dir, &["config", "user.email", "t@example.com"]);
        git(dir, &["config", "user.name", "Test"]);
        std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
        std::fs::write(dir.join("a.txt"), "hello").unwrap();
        git(dir, &["add", "."]);
        git(dir, &["commit", "-q", "-m", "base"]);
    }

    fn seed_slice_dir(dir: &Path, slice: u32) {
        let rel = format!(".doctrine/slice/{slice:03}");
        let full = dir.join(&rel);
        std::fs::create_dir_all(&full).unwrap();
        std::fs::write(
            full.join("slice.toml"),
            format!("id = {slice}\ntitle = \"test\"\nkind = \"slice\"\nstatus = \"planned\"\n"),
        )
        .unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "seed slice dir"]);
    }

    fn seed_plan(dir: &Path, slice: u32, phases: &str) {
        let rel = format!(".doctrine/slice/{slice:03}/plan.toml");
        let full = dir.join(&rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, phases).unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "seed plan"]);
    }

    #[test]
    fn dispatch_setup_gates_on_no_plan() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        // No plan.toml — the gate should fail before touching git.
        let holder = tempfile::tempdir().unwrap();
        let coord = holder.path().join("coord");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord, false);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("no plan"),
            "error should mention 'no plan'; got: {err}"
        );
    }

    #[test]
    fn dispatch_setup_gates_on_empty_plan() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            "schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n",
        );
        // Plan has zero phases.
        let holder = tempfile::tempdir().unwrap();
        let coord = holder.path().join("coord");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord, false);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("no phases"),
            "error should mention 'no phases'; got: {err}"
        );
    }

    #[test]
    fn dispatch_setup_creates_coordination() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            "schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n",
        );
        // Non-Claude arm with an outside-root coord dir: outside isolation is
        // legitimate (ADR-008), so the placement guard must NOT fire.
        let holder = tempfile::tempdir().unwrap();
        let coord = holder.path().join("coord");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord, false);
        assert!(result.is_ok(), "setup must succeed; err: {result:?}");

        // Verify worktree exists.
        assert!(coord.exists(), "coordination dir exists");
        assert!(coord.join("a.txt").exists(), "checkout exists");

        // Verify env contract keys on stdout (print! from run_setup).
        // Since run_setup uses println!, we test via the returned Ok(()).
        // The actual stdout capture is an integration-test concern; here we
        // verify the function doesn't panic and the worktree is real.
        assert!(coord.join(".doctrine").exists(), "provisioned");
    }

    // --- ISS-031: placement guard — outside-root coord under the Claude arm ---

    #[test]
    fn classify_coord_placement_truth_table() {
        // Only the outside-root × Claude-harness corner fails closed.
        assert!(classify_coord_placement(true, true).is_ok());
        assert!(classify_coord_placement(true, false).is_ok());
        assert!(classify_coord_placement(false, false).is_ok());
        assert_eq!(
            classify_coord_placement(false, true),
            Err("coord-outside-root-under-claude")
        );
    }

    #[test]
    fn dispatch_setup_refuses_outside_root_under_claude() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            "schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n",
        );
        // Outside-root coord dir + Claude harness → fail closed before any work.
        let holder = tempfile::tempdir().unwrap();
        let coord = holder.path().join("coord");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord, true);
        assert!(
            result.is_err(),
            "must refuse outside-root coord under Claude"
        );
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("coord-outside-root-under-claude"),
            "error names the placement token; got: {err}"
        );
        assert!(
            !coord.exists(),
            "no coordination worktree created on refusal"
        );
    }

    #[test]
    fn dispatch_setup_allows_inside_root_under_claude() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            "schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n",
        );
        // Inside-root coord dir is the safe convention; the guard must pass even
        // under the Claude harness.
        let coord = src.path().join(".dispatch/SL-085");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord, true);
        assert!(
            result.is_ok(),
            "inside-root coord must pass; err: {result:?}"
        );
        assert!(coord.join(".doctrine").exists(), "provisioned inside root");
    }

    // --- plan-next helpers ---

    /// Write a `phase-NN.toml` tracking file under
    /// `.doctrine/state/slice/{slice:03}/phases/`.
    fn seed_phase_tracking(dir: &Path, slice: u32, phase_num: u32, status: &str) {
        let state_dir = dir
            .join(".doctrine/state/slice")
            .join(format!("{slice:03}"))
            .join("phases");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::write(
            state_dir.join(format!("phase-{phase_num:02}.toml")),
            format!("status = \"{status}\"\n"),
        )
        .unwrap();
    }

    /// Build a multi-phase plan.toml body from phase ids + names. Each entry is
    /// `(id, name)`; the fixture automatically wraps in a `[[phase]]` array.
    fn plan_body(phases: &[(&str, &str)]) -> String {
        let mut body =
            String::from("schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n");
        for (id, name) in phases {
            body.push_str(&format!(
                "\n[[phase]]\nid = \"{id}\"\nname = \"{name}\"\nobjective = \"fixture\"\n"
            ));
        }
        body
    }

    // --- plan-next tests ---

    #[test]
    fn dispatch_plan_next_orders_phases() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "build"),
                ("PHASE-03", "blocked-one"),
                ("PHASE-04", "final"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        seed_phase_tracking(src.path(), 85, 2, "completed");
        seed_phase_tracking(src.path(), 85, 3, "blocked");
        // PHASE-04 has no tracking → pending

        // run_plan_next prints to stdout; we verify it doesn't panic and
        // check that the return is Ok.
        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_all_blocked() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "blocked-one"),
                ("PHASE-03", "blocked-two"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        seed_phase_tracking(src.path(), 85, 2, "blocked");
        seed_phase_tracking(src.path(), 85, 3, "blocked");

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_stops_at_blocked_mid() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "first-pending"),
                ("PHASE-03", "second-pending"),
                ("PHASE-04", "blocked"),
                ("PHASE-05", "after-blocked"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        // PHASE-02, PHASE-03: no tracking → pending
        seed_phase_tracking(src.path(), 85, 4, "blocked");
        // PHASE-05: no tracking → pending

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_resume_in_progress() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "in-progress"),
                ("PHASE-03", "next-one"),
                ("PHASE-04", "next-two"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        seed_phase_tracking(src.path(), 85, 2, "in_progress");
        // PHASE-03, PHASE-04: no tracking → pending

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_json() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[("PHASE-01", "setup"), ("PHASE-02", "active")]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        // PHASE-02: no tracking → pending

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, true);
        assert!(
            result.is_ok(),
            "plan-next --json should succeed; err: {result:?}"
        );
    }

    #[test]
    fn dispatch_plan_next_no_plan() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        // No plan.toml seeded.

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_err(), "plan-next without plan should fail");
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("not found"),
            "error should mention 'not found'; got: {err}"
        );
    }

    // --- status helpers ---

    /// Create a `refs/heads/dispatch/{slice:03}` ref pointing at the current HEAD.
    fn create_dispatch_ref(dir: &Path, slice: u32) {
        let head = git(dir, &["rev-parse", "HEAD"]);
        git(
            dir,
            &[
                "update-ref",
                &format!("refs/heads/dispatch/{slice:03}"),
                &head,
            ],
        );
    }

    /// Create a `refs/heads/review/{slice:03}` ref pointing at the current HEAD.
    fn create_review_ref(dir: &Path, slice: u32) {
        let head = git(dir, &["rev-parse", "HEAD"]);
        git(
            dir,
            &[
                "update-ref",
                &format!("refs/heads/review/{slice:03}"),
                &head,
            ],
        );
    }

    /// Advance trunk by making a commit on main.
    fn advance_trunk(dir: &Path) -> String {
        std::fs::write(dir.join("b.txt"), "world").unwrap();
        git(dir, &["add", "b.txt"]);
        git(dir, &["commit", "-q", "-m", "advance trunk"]);
        git(dir, &["rev-parse", "HEAD"])
    }

    // --- status tests ---

    /// T3-1: Status fresh after setup → coord live, phases pending, sync not yet run.
    #[test]
    fn dispatch_status_fresh_after_setup() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[("PHASE-01", "setup"), ("PHASE-02", "build")]),
        );
        create_dispatch_ref(src.path(), 85);

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-2: Status missing dispatch ref → non-zero exit (error).
    #[test]
    fn dispatch_status_missing_dispatch_ref() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        // No dispatch ref created.

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_err(), "status without dispatch ref should fail");
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("dispatch branch not found"),
            "error should mention 'dispatch branch not found'; got: {err}"
        );
    }

    /// T3-3: Status missing trunk ref → non-zero exit (error).
    #[test]
    fn dispatch_status_missing_trunk_ref() {
        // Create a repo that initialises with an orphaned initial commit on a
        // non-standard branch, so the trunk ladder (origin/HEAD, main, master)
        // finds nothing.
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path()).unwrap();
        git(src.path(), &["init", "-q", "-b", "other"]);
        git(src.path(), &["config", "user.email", "t@example.com"]);
        git(src.path(), &["config", "user.name", "Test"]);
        std::fs::create_dir_all(src.path().join(".doctrine")).unwrap();
        std::fs::write(src.path().join("a.txt"), "hello").unwrap();
        git(src.path(), &["add", "."]);
        git(src.path(), &["commit", "-q", "-m", "base"]);
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        // No main/master branch — trunk ladder returns None.

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_err(), "status without trunk ref should fail");
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("trunk ref not found"),
            "error should mention 'trunk ref not found'; got: {err}"
        );
    }

    /// T3-4: Status after sync → sync prepared, phase cuts count.
    #[test]
    fn dispatch_status_after_sync() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        create_review_ref(src.path(), 85);

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-5: Status moved trunk → trunk moved.
    #[test]
    fn dispatch_status_moved_trunk() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        // Create dispatch ref BEFORE trunk advances, so the fork point is older.
        create_dispatch_ref(src.path(), 85);
        advance_trunk(src.path());

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-6: Status all phases completed, no review ref → next guidance for prepare-review.
    #[test]
    fn dispatch_status_all_completed_no_review() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        seed_phase_tracking(src.path(), 85, 1, "completed");

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-7: Status all completed, review ref present → guidance references audit.
    #[test]
    fn dispatch_status_all_completed_review_present() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        create_review_ref(src.path(), 85);
        seed_phase_tracking(src.path(), 85, 1, "completed");

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-8: Status coord removed → coord (removed).
    #[test]
    fn dispatch_status_coord_removed() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        // No worktree exists — worktree list won't find it.

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-9: Status JSON → all sections, next.kind structured.
    #[test]
    fn dispatch_status_json() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[("PHASE-01", "setup"), ("PHASE-02", "build")]),
        );
        create_dispatch_ref(src.path(), 85);

        let result = run_status(Some(src.path().to_path_buf()), 85, true);
        assert!(
            result.is_ok(),
            "status --json should succeed; err: {result:?}"
        );
    }

    // --- SL-127 PHASE-03: trunk_drift + refresh-base ---------------------------

    /// Create `refs/heads/dispatch/{slice:03}` at the current HEAD and add a REAL
    /// linked worktree on it under `<dir>/coord`, returning the coord path. The
    /// coordination worktree is just `git worktree add <dir> dispatch/<NNN>`.
    fn add_dispatch_worktree(repo: &Path, slice: u32, holder: &Path) -> std::path::PathBuf {
        let branch = format!("dispatch/{slice:03}");
        let head = git(repo, &["rev-parse", "HEAD"]);
        git(repo, &["branch", &branch, &head]);
        let coord = holder.join("coord");
        git(
            repo,
            &[
                "worktree",
                "add",
                "--quiet",
                coord.to_str().unwrap(),
                &branch,
            ],
        );
        coord
    }

    /// Commit `content` to `file` in `wt`, returning the new HEAD oid.
    fn commit_file(wt: &Path, file: &str, content: &str, msg: &str) -> String {
        std::fs::write(wt.join(file), content).unwrap();
        git(wt, &["add", file]);
        git(wt, &["commit", "-q", "-m", msg]);
        git(wt, &["rev-parse", "HEAD"])
    }

    /// VT-1: `trunk_drift` — fork_point = merge_base(tip, trunk); ahead =
    /// count(fork_point..trunk); ahead == 0 when trunk is an ancestor of tip.
    #[test]
    fn trunk_drift_measures_against_trunk() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let fork = git(src.path(), &["rev-parse", "HEAD"]);
        // A tip parked at the fork: trunk has not moved past it yet.
        let tip = fork.clone();
        let d0 = trunk_drift(src.path(), &tip)
            .unwrap()
            .expect("shared ancestor");
        assert_eq!(d0.fork_point, fork, "fork_point is the merge-base");
        assert_eq!(d0.ahead, 0, "trunk == fork ⇒ zero ahead");

        // Advance trunk twice (distinct content per commit); tip stays at fork.
        commit_file(src.path(), "b.txt", "trunk-1\n", "advance trunk 1");
        let trunk_tip = commit_file(src.path(), "b.txt", "trunk-2\n", "advance trunk 2");
        let d = trunk_drift(src.path(), &tip)
            .unwrap()
            .expect("shared ancestor");
        assert_eq!(d.trunk_tip, trunk_tip, "carries the resolved trunk tip");
        assert_eq!(d.fork_point, fork, "fork unchanged — tip did not move");
        assert_eq!(d.ahead, 2, "trunk is two commits ahead of the fork");

        // A tip that already contains trunk ⇒ ahead == 0 (trunk is its ancestor).
        let d_fresh = trunk_drift(src.path(), &trunk_tip)
            .unwrap()
            .expect("shared ancestor");
        assert_eq!(d_fresh.ahead, 0, "trunk ancestor of tip ⇒ zero ahead");
    }

    /// VT-2: refresh-base CLEAN (reproduces SL-122). Trunk advances past the fork
    /// with a non-overlapping change; the dispatch branch carries its own commit.
    /// `run_refresh_base` merges clean, the coord HEAD advances to a merge commit
    /// with parents [dispatch_tip, trunk_tip], and afterwards
    /// merge_base(dispatch, trunk) == trunk_tip (trunk fully contained).
    #[test]
    fn refresh_base_clean_advances_dispatch() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let holder = tempfile::tempdir().unwrap();
        let coord = add_dispatch_worktree(src.path(), 85, holder.path());

        // Dispatch branch adds a NEW file in the coord worktree.
        let dispatch_tip = commit_file(&coord, "c.txt", "dispatch work\n", "dispatch commit");
        // Trunk advances on main with a same-block rewrite of a.txt that would
        // conflict at candidate-create 3-way time, but here is non-overlapping
        // with the dispatch delta (which touched only c.txt).
        let trunk_tip = commit_file(src.path(), "a.txt", "hello trunk-moved\n", "advance trunk");

        run_refresh_base(Some(src.path().to_path_buf()), 85).expect("clean refresh");

        let new_tip = git(&coord, &["rev-parse", "HEAD"]);
        assert_ne!(new_tip, dispatch_tip, "coord HEAD advanced");
        let parents = git(&coord, &["rev-list", "--parents", "-n", "1", &new_tip]);
        let p: Vec<&str> = parents.split_whitespace().skip(1).collect();
        assert_eq!(
            p,
            vec![dispatch_tip.as_str(), trunk_tip.as_str()],
            "merge parents"
        );

        // Trunk is now fully contained in the dispatch branch.
        let mb = git(&coord, &["merge-base", &new_tip, &trunk_tip]);
        assert_eq!(mb, trunk_tip, "merge_base(dispatch, trunk) == trunk_tip");
    }

    /// VT-3: refresh-base CONFLICT — a genuinely-conflicting trunk merge returns
    /// Err naming the conflicting path(s), leaves `MERGE_HEAD` in the coord
    /// worktree, and does NOT advance the dispatch ref past the pre-merge tip.
    #[test]
    fn refresh_base_conflict_reports_and_halts() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let holder = tempfile::tempdir().unwrap();
        let coord = add_dispatch_worktree(src.path(), 85, holder.path());

        // Both sides rewrite the SAME line of a.txt ⇒ a real conflict.
        let dispatch_tip = commit_file(&coord, "a.txt", "DISPATCH\n", "dispatch edits a.txt");
        commit_file(src.path(), "a.txt", "TRUNK\n", "trunk edits a.txt");

        let result = run_refresh_base(Some(src.path().to_path_buf()), 85);
        let err = format!("{}", result.expect_err("conflict must Err"));
        assert!(
            err.contains("a.txt"),
            "names the conflicting path; got: {err}"
        );
        assert!(err.contains("conflicted"), "reports conflict; got: {err}");

        // MERGE_HEAD persists in the coord worktree (not aborted).
        let merge_head = coord.join(".git");
        // Worktree .git is a file pointing at the gitdir; resolve via rev-parse.
        let _ = merge_head;
        let mh = git(&coord, &["rev-parse", "--verify", "--quiet", "MERGE_HEAD"]);
        assert!(!mh.is_empty(), "MERGE_HEAD left in place");

        // The dispatch ref is unadvanced (the conflicted merge is uncommitted).
        let tip_now = git(&coord, &["rev-parse", "dispatch/085"]);
        assert_eq!(
            tip_now, dispatch_tip,
            "dispatch ref unadvanced past pre-merge tip"
        );
    }

    /// VT-4a: unrelated histories ⇒ refuse before merging.
    #[test]
    fn refresh_base_refuses_unrelated_histories() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let holder = tempfile::tempdir().unwrap();
        let coord = add_dispatch_worktree(src.path(), 85, holder.path());
        // Re-root the dispatch branch onto an orphan with no shared ancestor.
        git(&coord, &["checkout", "-q", "--orphan", "orphan-tmp"]);
        std::fs::write(coord.join("orphan.txt"), "orphan\n").unwrap();
        git(&coord, &["add", "orphan.txt"]);
        git(&coord, &["commit", "-q", "-m", "orphan root"]);
        // Move dispatch/085 to the orphan, restore HEAD onto it cleanly.
        let orphan = git(&coord, &["rev-parse", "HEAD"]);
        git(&coord, &["branch", "-f", "dispatch/085", &orphan]);
        git(&coord, &["checkout", "-q", "dispatch/085"]);
        git(&coord, &["branch", "-D", "orphan-tmp"]);

        let result = run_refresh_base(Some(src.path().to_path_buf()), 85);
        let err = format!("{}", result.expect_err("unrelated histories must Err"));
        assert!(
            err.contains("unrelated histories"),
            "refuses unrelated histories; got: {err}"
        );
    }

    /// VT-4b: already-fresh (trunk is an ancestor of dispatch) ⇒ no-op Ok, no new
    /// commit written.
    #[test]
    fn refresh_base_noop_when_already_fresh() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let holder = tempfile::tempdir().unwrap();
        let coord = add_dispatch_worktree(src.path(), 85, holder.path());
        // Dispatch branch is at trunk tip (no drift) and adds a commit on top, so
        // trunk is strictly an ancestor of dispatch.
        let before = commit_file(&coord, "c.txt", "ahead of trunk\n", "dispatch ahead");

        run_refresh_base(Some(src.path().to_path_buf()), 85).expect("already-fresh is Ok");

        let after = git(&coord, &["rev-parse", "HEAD"]);
        assert_eq!(after, before, "no new commit on a fresh dispatch branch");
    }

    /// VT-4c: dirty coord tree ⇒ refuse (don't merge over WIP).
    #[test]
    fn refresh_base_refuses_dirty_coord() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let holder = tempfile::tempdir().unwrap();
        let coord = add_dispatch_worktree(src.path(), 85, holder.path());
        advance_trunk(src.path()); // make trunk move so a merge would be attempted
        // Leave uncommitted WIP in the coord tree.
        std::fs::write(coord.join("a.txt"), "uncommitted edit\n").unwrap();

        let result = run_refresh_base(Some(src.path().to_path_buf()), 85);
        let err = format!("{}", result.expect_err("dirty coord must Err"));
        assert!(
            err.contains("dirty coordination worktree"),
            "refuses a dirty coord tree; got: {err}"
        );
    }

    /// VT-4d: no coordination worktree ⇒ refuse with the setup/resume hint.
    #[test]
    fn refresh_base_refuses_without_coord_worktree() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        // Create the dispatch ref but NO live worktree on it.
        create_dispatch_ref(src.path(), 85);

        let result = run_refresh_base(Some(src.path().to_path_buf()), 85);
        let err = format!("{}", result.expect_err("missing coord worktree must Err"));
        assert!(
            err.contains("no live coordination worktree") && err.contains("setup"),
            "hints at setup/resume; got: {err}"
        );
    }

    // --- SL-127 PHASE-04: drift diagnostics ------------------------------------

    /// The pre-SL-127 content-conflict abort text, verbatim. VT-1b pins the
    /// `ahead == 0` rendering to these exact bytes — the no-verdict contract.
    const LEGACY_CONFLICT_TEXT: &str = "candidate create: 3-way merge of refs/heads/review/085 onto trunk conflicts — pass --worktree to park the candidate branch at the base for manual resolve+commit, or abort (no row/ref/worktree written)";

    /// VT-1a (EX-1): a content conflict where trunk has advanced past the source
    /// ⇒ the abort message APPENDS the refresh-base hint AND the drift count, while
    /// preserving the original text as a prefix (the hint is additive, never a
    /// replacement, and never asserts the cause).
    #[test]
    fn candidate_conflict_message_appends_drift_hint() {
        let msg = candidate_conflict_message("refs/heads/review/085", "trunk", 3);
        assert!(
            msg.starts_with(LEGACY_CONFLICT_TEXT),
            "legacy text is preserved as a prefix; got: {msg}"
        );
        assert!(
            msg.contains("trunk has advanced 3 commit(s) past this source"),
            "names the drift count; got: {msg}"
        );
        assert!(
            msg.contains("refresh-base") && msg.contains("re-prepare + re-create"),
            "hints the refresh-base remedy; got: {msg}"
        );
        assert!(
            msg.contains("may be base divergence"),
            "non-asserting ('may be'); got: {msg}"
        );
    }

    /// VT-1b (EX-1): a content conflict where trunk has NOT advanced (`ahead == 0`)
    /// ⇒ the abort message is BYTE-IDENTICAL to the pre-SL-127 text. Guards the
    /// no-verdict contract: a plain content conflict carries no drift diagnosis.
    #[test]
    fn candidate_conflict_message_byte_identical_when_not_behind_trunk() {
        let msg = candidate_conflict_message("refs/heads/review/085", "trunk", 0);
        assert_eq!(msg, LEGACY_CONFLICT_TEXT, "ahead==0 ⇒ verbatim legacy text");
    }

    /// A `select_guidance` row with no phases remaining, no admission, coord live —
    /// the common "all done" shape. Individual tests flip the fields under test.
    fn all_done_inputs() -> GuidanceInputs<fn() -> Vec<String>> {
        GuidanceInputs {
            all_completed: true,
            bundle_stale: false,
            review_exists: false,
            coord_live: true,
            admitted: false,
            admitted_is_ancestor: false,
            next_phases: Vec::new,
        }
    }

    /// VT-2a (EX-2): all phases complete AND the prepared bundle is stale past trunk
    /// ⇒ guidance is `RefreshBase`, and it fires BEFORE the prepare-review/audit
    /// legs (even with a review ref + admission present, RefreshBase wins). JSON
    /// kind is the structured `refresh_base`.
    #[test]
    fn select_guidance_refresh_base_precedes_prepare_review_and_audit() {
        // Bare stale bundle, no review yet ⇒ would route to PrepareReview without
        // the stale check; the stale leg must win.
        let g = select_guidance(GuidanceInputs {
            bundle_stale: true,
            ..all_done_inputs()
        });
        assert!(
            matches!(g, NextGuidance::RefreshBase),
            "stale ⇒ RefreshBase"
        );
        assert_eq!(g.to_json().kind, "refresh_base");

        // Even with a review ref AND an admitted close target (the audit legs), a
        // stale bundle still routes to RefreshBase — it precedes audit.
        let g2 = select_guidance(GuidanceInputs {
            bundle_stale: true,
            review_exists: true,
            admitted: true,
            ..all_done_inputs()
        });
        assert!(
            matches!(g2, NextGuidance::RefreshBase),
            "stale wins over the audit legs"
        );
    }

    /// VT-2b (EX-2): a fresh bundle (`bundle_stale == false`) leaves the prior
    /// machine untouched — no review ref ⇒ PrepareReview; review ref present ⇒ the
    /// audit leg. RefreshBase is ABSENT.
    #[test]
    fn select_guidance_fresh_bundle_keeps_existing_guidance() {
        let no_review = select_guidance(all_done_inputs());
        assert!(
            matches!(no_review, NextGuidance::PrepareReview),
            "fresh + no review ⇒ PrepareReview (unchanged)"
        );

        let with_review = select_guidance(GuidanceInputs {
            review_exists: true,
            ..all_done_inputs()
        });
        assert!(
            matches!(with_review, NextGuidance::AuditOrCandidateStatus),
            "fresh + review ⇒ audit leg (unchanged)"
        );
    }

    /// VT-2a (integration): `run_status` drives the stale bundle end-to-end — a
    /// dispatch ref parked at the fork, all phases completed, trunk advanced past
    /// it, no review ref ⇒ Ok (the RefreshBase leg is reached, not a stale-base
    /// prepare-review). Pairs with the table test above for the routing proof.
    #[test]
    fn dispatch_status_stale_bundle_routes_refresh_base() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        // Dispatch ref pinned at the current HEAD (the fork), THEN trunk advances —
        // so trunk_drift(dispatch_tip).ahead > 0 (the bundle is stale).
        create_dispatch_ref(src.path(), 85);
        advance_trunk(src.path());
        seed_phase_tracking(src.path(), 85, 1, "completed");

        let result = run_status(Some(src.path().to_path_buf()), 85, true);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    // --- PHASE-05 (ISS-052) projection-source guard predicate (D11) ----------

    fn reg_row(phase: &str, provenance: Provenance) -> BoundaryRow {
        BoundaryRow {
            phase: phase.to_string(),
            code_start_oid: "s".to_string(),
            code_end_oid: "e".to_string(),
            provenance,
        }
    }

    fn committed_set<'a>(phases: &'a [&str]) -> BTreeSet<&'a str> {
        phases.iter().copied().collect()
    }

    // VT-1: total loss — every registry row is funnel-owned, the committed ledger
    // is empty → all phases are named missing.
    #[test]
    fn guard_total_loss_names_every_funnel_phase() {
        let registry = vec![
            reg_row("PHASE-01", Provenance::Funnel),
            reg_row("PHASE-02", Provenance::Funnel),
        ];
        let committed = committed_set(&[]);
        let missing = missing_committed_funnel_phases(&registry, &committed);
        assert_eq!(missing, vec!["PHASE-01", "PHASE-02"]);
    }

    // VT-2: partial loss — one funnel phase absent from the committed ledger →
    // only that one is named; a complete committed ledger → nothing missing.
    #[test]
    fn guard_partial_loss_names_only_the_uncommitted_phase() {
        let registry = vec![
            reg_row("PHASE-01", Provenance::Funnel),
            reg_row("PHASE-02", Provenance::Funnel),
        ];
        assert_eq!(
            missing_committed_funnel_phases(&registry, &committed_set(&["PHASE-01"])),
            vec!["PHASE-02"],
        );
        assert!(
            missing_committed_funnel_phases(&registry, &committed_set(&["PHASE-01", "PHASE-02"]))
                .is_empty(),
            "a complete committed ledger leaves nothing missing",
        );
    }

    // VT-4: set membership by provenance — Unknown (legacy/unclassified) missing
    // halts; Solo (binding) and a fresh Manual (record-delta) missing do NOT.
    #[test]
    fn guard_includes_unknown_excludes_solo_and_manual() {
        let registry = vec![
            reg_row("PHASE-01", Provenance::Unknown),
            reg_row("PHASE-02", Provenance::Solo),
            reg_row("PHASE-03", Provenance::Manual),
        ];
        // None present in the committed ledger; only the Unknown row is funnel-owned.
        let missing = missing_committed_funnel_phases(&registry, &committed_set(&[]));
        assert_eq!(
            missing,
            vec!["PHASE-01"],
            "Unknown halts; Solo/Manual excluded"
        );
    }

    // An empty registry can never produce a missing phase.
    #[test]
    fn guard_empty_registry_is_silent() {
        assert!(missing_committed_funnel_phases(&[], &committed_set(&["PHASE-01"])).is_empty(),);
    }
}
