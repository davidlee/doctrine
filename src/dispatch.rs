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

use crate::slice::parse_cli_id;
use crate::worktree::JailPolicy;

use crate::boundary::{BoundaryRow, Provenance as BoundaryProvenance};
use crate::corpus_guard;
use crate::git::{self, MergeTree, RefCas, ZERO_OID};
use crate::kinds::{
    CANDIDATE_REF_PREFIX, DISPATCH_REF_PREFIX, PHASE_REF_PREFIX, REVIEW_REF_PREFIX,
};
use crate::ledger::{
    Admission, Boundaries, CandidateKind, CandidatePayload, CandidateRole, CandidateRow,
    CandidateStatus, Candidates, Journal, JournalRow, LedgerStatus, Orthogonal, read_candidates,
};
use crate::listing::render_table;
use crate::root;
use crate::worktree::run_provision;

#[derive(Subcommand)]
pub(crate) enum DispatchCommand {
    /// Sync reviewable refs from the dispatch branch.
    /// Stage selector required; `--prepare-review` creates `review/<slice>` +
    /// `phase/<slice>-NN` under CAS (never writing trunk). `--integrate` replays
    /// the journal. Orchestrator-classed — refused under worker-mode.
    Sync {
        /// The slice id (bare number, e.g. `64`) whose `dispatch/<slice>`
        /// coordination branch to project.
        #[arg(long, value_parser = parse_cli_id)]
        slice: u32,

        /// Stage-1: create the reviewable `review/<slice>` and `phase/<slice>-NN`
        /// refs from the dispatch tip; never writes trunk.
        #[arg(long, group = "stage", required = true)]
        prepare_review: bool,

        /// Stage-2: replay the prepared journal idempotently and project the
        /// audited code units. Runs from parent/root after the coordination
        /// worktree is removed. Trunk is only advanced when `--trunk` is given —
        /// without it, trunk is left untouched (effectively a dry-run for the
        /// trunk leg). `--edge` advances an aggregate ref independently.
        /// Fails rather than auto-resolving conflicts.
        #[arg(long, group = "stage", required = true)]
        integrate: bool,

        /// Read-only (SL-121 §3(b)): print the committed journal's trunk-row
        /// `planned_new_oid` — the row whose target is `--trunk` — to stdout and
        /// exit; the close step-3a verify read surface. Tree-reads `dispatch/<slice>`,
        /// writes nothing.
        #[arg(long, group = "stage", required = true)]
        show_journal_trunk_oid: bool,

        /// Stage: record an out-of-band trunk integration (split-lineage close
        /// recovery, SL-211). The trunk payload (phase-chain tip / admitted
        /// `close_target`) must ALREADY be an ancestor of `--trunk`; commits a
        /// Verified trunk row to `dispatch/<slice>` and advances NO external ref.
        /// `--trunk` required.
        #[arg(long, group = "stage", required = true)]
        record_integration: bool,

        /// Project the cumulative code units onto this trunk ref, fast-forward-only +
        /// expected-tip CAS (e.g. `refs/heads/main`). Also names the row to read
        /// under `--show-journal-trunk-oid`. Omit to leave trunk untouched — useful
        /// as a safety check or when only `--edge` is desired. Required for
        /// `--record-integration` (the recorded row must name the gate's ref).
        #[arg(
            long,
            conflicts_with = "prepare_review",
            required_if_eq("record_integration", "true")
        )]
        trunk: Option<String>,

        /// Stage-2 only: advance this standing aggregate ref to the `review/<slice>`
        /// bundle (e.g. `refs/heads/edge`). Absent ⇒ no aggregate written.
        #[arg(long, requires = "integrate")]
        edge: Option<String>,

        /// Stage-2 g3 escape (SL-166): allow the advance to delete/revert this
        /// authored `.doctrine/**` path even though the slice did not author the
        /// change. Repeatable; the allowlist is global across BOTH the
        /// `--trunk` and `--edge` legs of a single integrate call (design §10) —
        /// one named path is permitted on either ref it would clobber.
        /// Absent for a clobbered path ⇒ the advance is refused (fail-closed).
        #[arg(long = "allow-corpus-clobber", requires = "integrate")]
        allow_corpus_clobber: Vec<String>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Record a phase code boundary.
    /// Appends a per-phase boundary to `.doctrine/dispatch/<slice>/boundaries.toml`.
    /// Orchestrator-classed — refused under worker-mode.
    RecordBoundary {
        /// The slice id (bare number, e.g. `64`) whose ledger to append.
        #[arg(long, value_parser = parse_cli_id)]
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
        #[arg(long, value_parser = parse_cli_id)]
        slice: u32,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create or resume dispatch coordination.
    /// Emits the dispatch env contract on stdout. Orchestrator-classed — refused
    /// under worker-mode.
    Setup {
        /// The slice id (bare number, e.g. `85`).
        #[arg(long, value_parser = parse_cli_id)]
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
        #[arg(long, value_parser = parse_cli_id)]
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
        #[arg(long, value_parser = parse_cli_id)]
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
        /// Optional: omitted ⇒ defaults to the coord-root `HEAD` (A1).
        #[arg(long)]
        base: Option<String>,

        /// The slice being dispatched (bare number) — diagnostic only; the arming dir
        /// is per-coord-tree, not per-slice (cross-slice partition is by coord tree).
        #[arg(long, value_parser = parse_cli_id)]
        slice: Option<u32>,

        /// Per-arming jail widening (objective 3): absolute paths granted rw inside the
        /// worker jail beyond its worktree. Repeatable. Empty (+ default network) ⇒ no
        /// declaration ⇒ the pretooluse Default floor (design §5.3, D2).
        #[arg(long = "extra-rw")]
        extra_rw: Vec<PathBuf>,

        /// Deny the worker jail network access (`--unshare-net`). Default keeps
        /// today's network behaviour. Setting it declares a non-Default policy.
        #[arg(long = "no-network")]
        no_network: bool,

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
        #[arg(long, value_parser = parse_cli_id)]
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
        #[arg(long, value_parser = parse_cli_id)]
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
        #[arg(long, value_parser = parse_cli_id)]
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

    /// Ingest (SL-212): adopt an operator's hand-resolved trunk merge into a
    /// Conflicted candidate row. Run from the COORDINATION tree (refuses a
    /// candidate-worktree cwd) after resolving the markers and `git commit`ting
    /// inside the candidate worktree. Validates that the commit is a faithful
    /// `(base, source)` 3-way — never an arbitrary tree — then write-once fills the
    /// row (`merge_provenance=OperatorIngest`); `admit`/`integrate` follow the
    /// existing FF-only contract. `base`/`source` come from the row, not flags.
    /// Orchestrator-classed.
    Ingest {
        /// The slice id (bare number, e.g. `68`).
        #[arg(long, value_parser = parse_cli_id)]
        slice: u32,

        /// The candidate label (e.g. `review-001`) — selects the exactly-one
        /// Conflicted, un-ingested row to fill.
        #[arg(long, visible_alias = "target")]
        label: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: DispatchCommand, _color: bool) -> anyhow::Result<()> {
    match cmd {
        DispatchCommand::Sync {
            slice,
            record_integration,
            integrate,
            show_journal_trunk_oid,
            trunk,
            edge,
            allow_corpus_clobber,
            path,
            ..
        } => {
            // The `stage` group is `required = true` single-choice: exactly one
            // of `--prepare-review` / `--integrate` / `--show-journal-trunk-oid` /
            // `--record-integration` is set, so the booleans select the stage in
            // order (no unreachable arm).
            if show_journal_trunk_oid {
                // SL-128 D3: absent `--trunk` defaults from `[dispatch] deliver_to`;
                // explicit `--trunk` still wins. `--integrate` is unchanged.
                run_show_journal_trunk_oid(path, slice, trunk.as_deref())
            } else if record_integration {
                // SL-211: record an out-of-band trunk land; advances no external
                // ref (INV-1). `--trunk` is `required_if_eq` at the CLI.
                run_record_integration(path, slice, trunk.as_deref())
            } else if integrate {
                let allow: BTreeSet<String> = allow_corpus_clobber.into_iter().collect();
                run_integrate(path, slice, trunk.as_deref(), edge.as_deref(), &allow)
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
            CandidateCommand::Ingest { slice, label, path } => {
                let req = IngestRequest {
                    slice,
                    label,
                    ingested_at: crate::clock::today(),
                };
                run_candidate_ingest(path, &req)
            }
        },
        DispatchCommand::PlanNext { slice, json, path } => run_plan_next(path, slice, json),
        DispatchCommand::Status { slice, json, path } => run_status(path, slice, json),
        DispatchCommand::DeliverTo { path } => run_deliver_to(path),
        DispatchCommand::ArmSpawn {
            base,
            slice,
            extra_rw,
            no_network,
            path,
        } => run_arm_spawn(path, base.as_deref(), slice, extra_rw, no_network),
    }
}

/// `dispatch arm-spawn` — write the arming `base` file (and the paired `jail.toml`
/// policy DECLARATION) and print the spawn dir (SL-152 PHASE-03; design §5.2/§5.3).
/// The arming dir is in the coord tree's own runtime state (gitignored, withheld
/// `Tier::State` ⇒ never provisioned into a worker fork). The path const is SHARED
/// with the `worktree create-fork` reader ([`crate::worktree::ARMING_SUBPATH`]) — one
/// contract anchor, no re-spelling.
///
/// F-4 pairing: `base` and `jail.toml` are written in this SINGLE arming step, and a
/// re-arm rewrites both — there is no separate "update jail.toml" path. When the
/// declared policy equals the Default floor (no `--extra-rw`, network on), NO
/// `jail.toml` is written and any STALE one is removed, so a leftover declaration can
/// never pair with a fresh `base` (design §5.3).
fn run_arm_spawn(
    path: Option<PathBuf>,
    base: Option<&str>,
    slice: Option<u32>,
    extra_rw: Vec<PathBuf>,
    no_network: bool,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    // A1: an omitted `--base` defaults to the coord-root HEAD; an explicit base is
    // honored unchanged. Resolve through the crate's shared git shell (same helper
    // `run_refresh_base` uses), never a hand-rolled Command.
    //
    // A7 caveat: the DEFAULTED base is NOT self-catching on a confined arm. This
    // resolves against the root this process sees; if that is the wrong-but-consistent
    // tree, a wrong base can land silently — there is no cross-check here. The deferred
    // spawn-time guard (IMP-268) is the real net for that, not this default; do not
    // mistake this resolution for one.
    let resolved = match base {
        Some(b) => b.trim().to_string(),
        None => git::git_text(&root, &["rev-parse", "HEAD"])?,
    };

    // Fail closed on a base outside the reader's accepted envelope (4..=64 hex), so a
    // bad base surfaces at arm time, not silently as a no-fork at spawn time. Applied to
    // the RESOLVED base, so an explicit bad `--base` still fails exactly as before.
    let b = resolved.as_str();
    if !(4..=64).contains(&b.len()) || !b.bytes().all(|c| c.is_ascii_hexdigit()) {
        bail!("bad-base: `{b}` is not a 4..=64-char hex oid");
    }

    let spawn = root.join(crate::worktree::ARMING_SUBPATH);
    std::fs::create_dir_all(&spawn)
        .with_context(|| format!("create arming dir {}", spawn.display()))?;
    crate::fsutil::write_atomic(&spawn.join("base"), format!("{b}\n").as_bytes())
        .with_context(|| format!("write arming base in {}", spawn.display()))?;
    write_arming_jail_policy(&spawn, extra_rw, no_network)?;

    let spawn_canon = std::fs::canonicalize(&spawn)
        .with_context(|| format!("canonicalize arming dir {}", spawn.display()))?;
    if let Some(slice) = slice {
        writeln!(io::stderr(), "armed SL-{slice:03} at base {b}")?;
    }
    writeln!(io::stdout(), "{}", spawn_canon.display())?;
    Ok(())
}

/// Write (or clear) the arming jail-policy DECLARATION beside `base` in `spawn`
/// (F-4 pairing). The declared policy is `{ extra_rw, network: !no_network }`. When it
/// equals the Default floor, absence and an explicit Default are indistinguishable to
/// the reader (both ⇒ pretooluse Default), so we write NOTHING and remove any stale
/// `jail.toml` — the pairing-hygiene guard that stops a leftover declaration riding a
/// fresh `base`. Otherwise the policy is serialised through the single `JailPolicy`
/// schema (round-trips to `from_toml_str` on read).
fn write_arming_jail_policy(
    spawn: &Path,
    extra_rw: Vec<PathBuf>,
    no_network: bool,
) -> anyhow::Result<()> {
    let policy = JailPolicy {
        extra_rw,
        network: !no_network,
    };
    let decl = spawn.join(crate::worktree::ARMING_JAIL_FILE);
    if policy == JailPolicy::default() {
        match std::fs::remove_file(&decl) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => {
                Err(e).with_context(|| format!("clear stale jail declaration {}", decl.display()))
            }
        }
    } else {
        let body = toml::to_string(&policy).context("serialize arming jail policy")?;
        crate::fsutil::write_atomic(&decl, body.as_bytes())
            .with_context(|| format!("write jail declaration {}", decl.display()))
    }
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

    // Delegate to the extracted pure-ish core; thread the resolved g2
    // authoring-branch VALUE (ADR-001/VA-1: value, not the loader).
    let authoring = crate::dtoml::load_doctrine_toml(&root)?
        .dispatch
        .authoring_branch;
    let outcome = crate::worktree::coordinate(&root, slice, dir, authoring.as_deref())?;

    // Emit the dispatch env contract on stdout (4 KEY=value lines).
    let dispatch_ref = format!("{DISPATCH_REF_PREFIX}{slice:03}");
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

/// SL-211 — the `--record-integration` stage handler (split-lineage close
/// recovery). Records that the trunk payload (phase-chain tip / admitted
/// `close_target`, via the shared [`resolve_trunk_payload`] seam) has ALREADY
/// landed out-of-band as an ancestor of `trunk`, committing a single Verified
/// trunk row to `dispatch/<slice>`. It advances **no external ref** (INV-1) — the
/// unchanged close gate reads the row and passes the slice to `done`.
///
/// Guards (design §5.4/§5.5): F-4 `--trunk` must equal `[dispatch] deliver_to`
/// (a row targeting a ref the gate does not read would still block `done`) —
/// absent `--trunk` defaults to it, an explicit mismatch is refused. F-2 an
/// existing trunk row: a `Verified` row is a real prior integration ⇒ idempotent
/// no-op; a `Pending`/`Failed` row carried zero external effect ⇒ replaced by the
/// earned row (D7). The earned `is_ancestor` check lives in
/// [`plan_recorded_trunk_row`] (R1). Unlike integrate the journal may legitimately
/// be empty here (prepare-review need not have run for this recovery), so there is
/// no empty-journal bail.
pub(crate) fn run_record_integration(
    path: Option<PathBuf>,
    slice: u32,
    trunk: Option<&str>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    // F-4: the recorded row must target the gate's delivery ref, else `done` still
    // blocks. Absent `--trunk` defaults to `deliver_to` (as show-journal does); an
    // explicit mismatch is a hard refusal naming both refs.
    let deliver_to = crate::dtoml::load_doctrine_toml(&root)?.dispatch.deliver_to;
    let trunk: String = match trunk {
        None => deliver_to,
        Some(t) if t == deliver_to => t.to_string(),
        Some(t) => bail!(
            "record-integration: --trunk {t} does not match the close gate's delivery ref \
             {deliver_to} ([dispatch] deliver_to) — a row targeting {t} would not be read at \
             `slice status … done`; record onto {deliver_to} (or omit --trunk to default)"
        ),
    };

    let slice3 = format!("{slice:03}");
    let coord_ref = format!("{DISPATCH_REF_PREFIX}{slice3}");
    let tip = resolve_commit(&root, &coord_ref)?
        .with_context(|| format!("record-integration: dispatch/{slice3} does not exist"))?;
    let tip_tree = tree_of(&root, &tip)?;
    let journal_path = format!(".doctrine/dispatch/{slice3}/journal.toml");

    // Tree-read the ledgers from the coordination tip (never the filesystem). An
    // empty journal is legitimate here — recovery may run without a prior
    // prepare-review — so, unlike integrate, we do NOT bail on an empty row set.
    let mut journal = read_ledger::<Journal>(&root, &coord_ref, &slice3, "journal.toml")?;
    let candidates = read_candidates(&root, slice)?;

    // F-2 existing-trunk-row guard (D7): a prior Verified row is a real prior
    // integration ⇒ the gate already passes ⇒ idempotent no-op (never a duplicate).
    if journal
        .rows
        .iter()
        .any(|r| r.target_ref == trunk && r.status == LedgerStatus::Verified)
    {
        writeln!(
            io::stderr(),
            "record-integration: already integrated — dispatch/{slice3} carries a Verified \
             trunk row for {trunk}; no-op"
        )?;
        return Ok(());
    }

    // Plan the earned row (R1: refuses an un-landed payload / unresolved trunk).
    let row = plan_recorded_trunk_row(&root, &slice3, &journal, &candidates, &trunk)?;
    let payload = row.planned_new_oid.clone();
    // A non-applied Pending/Failed row (the only remaining trunk-targeting shape —
    // Verified returned above) carried zero external effect ⇒ replace it in place
    // with the earned Verified row (the recovery, not a dead end); else append.
    if let Some(slot) = journal.rows.iter_mut().find(|r| r.target_ref == trunk) {
        *slot = row;
    } else {
        journal.rows.push(row);
    }

    // Commit the journal onto dispatch/<slice> — NO `with_journaled_projection`:
    // the recorder advances no external ref (INV-1).
    commit_journal(
        &root,
        &tip_tree,
        &tip,
        &journal_path,
        &coord_ref,
        &journal,
        "journal: record-integration",
    )?;
    writeln!(
        io::stderr(),
        "record-integration: recorded Verified trunk row for {trunk} on dispatch/{slice3} \
         (payload {payload})"
    )?;
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
    allow: &BTreeSet<String>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    // g1 (SL-166 design §5.4): refuse at the verb entry — the earliest, cheapest
    // point, before any ref work — when HEAD sits on the integration buffer under
    // the buffered-trunk posture. This single call covers BOTH the `--trunk`/
    // `--edge` and candidate-active legs, which all land in `integrate()`.
    let cfg = crate::dtoml::load_doctrine_toml(&root)?.dispatch;
    guard_not_on_integration_ref(&root, &cfg)?;
    integrate(&root, slice, trunk, edge, allow)
}

/// g1 (SL-166 design §5.2, EX-1..3) — refuse a trunk-mutating dispatch verb
/// invoked while HEAD sits on the integration buffer (`deliver_to`). The HEAD read
/// is worktree-local (`symbolic-ref --short HEAD` via [`current_branch`], the
/// invoking cwd worktree's branch — EX-2), the same seam the raw-evidence-ref
/// guard uses. Inert unless the buffered-trunk posture is on (`authoring-branch`
/// set and ≠ `deliver_to`, EX-3); the decision is the pure
/// [`corpus_guard::on_integration_buffer`] predicate. The refusal names the buffer
/// ref and the `fetch`-not-`checkout` promotion recovery.
fn guard_not_on_integration_ref(
    root: &Path,
    cfg: &crate::dispatch_config::DispatchConfig,
) -> anyhow::Result<()> {
    let current = current_branch(root)?;
    if corpus_guard::on_integration_buffer(
        current.as_deref(),
        cfg.authoring_branch.as_deref(),
        &cfg.deliver_to,
    ) {
        // Posture is on here (predicate true ⇒ authoring is Some), so unwrap is
        // total; default only as a belt-and-braces non-panic.
        let authoring = cfg.authoring_branch.as_deref().unwrap_or_default();
        let buffer = corpus_guard::short_branch_name(&cfg.deliver_to);
        bail!(
            "{} `{}` — the primary must stay on `{authoring}`. Restore \
             (`git checkout {authoring}`) and promote via \
             `git fetch . {authoring}:{buffer}`, never `checkout {buffer}`.",
            corpus_guard::REFUSE_ON_TRUNK,
            cfg.deliver_to,
        );
    }
    Ok(())
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
    // — the phase-cut input prepare-review tree-reads. SL-221 PHASE-03 (ISS-225):
    // land it WORKING-TREE-FREE on the `dispatch/<slice>` ref, exactly as the
    // conclude path does (`conclude_boundary_commit`), collapsing the split write
    // seam. The row keeps its `Funnel` attribution; the commit identity is the
    // shared dispatch identity under a `Conclude`-shaped provenance (OQ-1 default).
    let dref = dispatch_ref(slice);
    let tip = resolve_commit(&root, &dref)?
        .with_context(|| format!("record-boundary: {dref} does not resolve to a commit"))?;
    match land_boundary_row(
        &root,
        &dref,
        &tip,
        slice,
        row.clone(),
        &Provenance::Conclude {
            who: dispatch_identity(),
        },
    )? {
        CommitOutcome::Landed { .. } => {}
        CommitOutcome::Refused(refusal) => bail!("record-boundary: {}", refusal.token()),
    }
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
    let dispatch_ref = format!("{DISPATCH_REF_PREFIX}{slice3}");

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
        CandidateRole::ReviewSurface => Ok(format!("{REVIEW_REF_PREFIX}{slice3}")),
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

/// Single classifier for a journaled-evidence source ref: `review/<slice>` or
/// `phase/<slice>-NN`. The one source of truth for "is this source a journaled
/// evidence ref", so the provenance base-case (and, later, the recursion step)
/// agree — no inline ref-shape duplication.
fn is_journaled_evidence_ref(source_ref: &str, slice3: &str) -> bool {
    source_ref == format!("{REVIEW_REF_PREFIX}{slice3}")
        || source_ref
            .strip_prefix(&format!("{PHASE_REF_PREFIX}{slice3}-"))
            .and_then(|nn| nn.parse::<u32>().ok())
            .is_some()
}

/// Depth budget for candidate-provenance chain tracing (INV-4, OQ-1).
/// Named constant per STD-001 — never a literal at the call site.
const CANDIDATE_PROVENANCE_DEPTH_BUDGET: u32 = 16;

/// The gitignored runtime subpath (relative to the coordination root) that parents
/// every candidate linked worktree: `<root>/.doctrine/state/dispatch/candidate/<id>`.
/// Named constant per STD-001 — the single source for the two `add`/`rollback`
/// worktree-path joins AND the ingest coordination-root guard (design §5.3, RV-289
/// F-1: a cwd resolved *under* this subpath is a candidate checkout, refused).
const CANDIDATE_WORKTREE_SUBPATH: &str = ".doctrine/state/dispatch/candidate";

/// Single classifier for a candidate ref: `refs/heads/candidate/<N>/<label>`.
fn is_candidate_ref(source_ref: &str) -> bool {
    source_ref.starts_with(CANDIDATE_REF_PREFIX)
}

/// Walk the recorded-candidate chain from `ref_name` to a Verified journaled
/// evidence root (design §5.1, INV-1..5). Count-exact `target_ref` match (fail-
/// closed on duplicates, mirrors `ledger.rs:464`); status gate `Created` only;
/// role/kind gate `review_surface | close_target` + `audit`; recurse on a
/// candidate `source_ref`; terminate on a journaled `source_ref` by routing the
/// FULL existing journaled gate (Verified + phase-hole, F3). Bounded by
/// `budget` (INV-4). Returns the matched candidate row so the caller can bind
/// lineage (INV-6) without a second lookup.
fn trace_candidate_provenance<'a>(
    candidates: &'a Candidates,
    journal: &Journal,
    slice3: &str,
    ref_name: &str,
    budget: u32,
) -> anyhow::Result<&'a CandidateRow> {
    if budget == 0 {
        bail!(
            "candidate create: provenance chain too deep or cyclic — \
             budget exhausted at {ref_name}"
        );
    }
    // Count-exact match — fail-closed on duplicates (INV-5).
    let mut rows = candidates.rows.iter().filter(|r| r.target_ref == ref_name);
    let row = rows.next().with_context(|| {
        format!("candidate create: no recorded candidate row for source {ref_name}")
    })?;
    if rows.next().is_some() {
        bail!(
            "candidate create: ambiguous candidate row for {ref_name} — \
             multiple rows share the same target_ref"
        );
    }
    anyhow::ensure!(
        row.status == CandidateStatus::Created,
        "candidate create: source candidate {ref_name} is {:?}, not clean (must be Created)",
        row.status
    );
    anyhow::ensure!(
        matches!(
            row.role,
            CandidateRole::ReviewSurface | CandidateRole::CloseTarget
        ) && row.kind == CandidateKind::Audit,
        "candidate create: source candidate {ref_name} is role={:?}/kind={:?} — \
         only an audit review_surface (or chained close_target) may source a close_target",
        row.role,
        row.kind
    );
    let next = &row.source_ref;
    if is_journaled_evidence_ref(next, slice3) {
        // Terminate at journaled evidence — run the FULL existing gate
        // (Verified + phase-hole, F3 — not a weakened subset).
        let jrow = journal
            .rows
            .iter()
            .find(|r| r.target_ref == *next)
            .with_context(|| {
                format!(
                    "candidate create: no prepare-review journal row for source {next} — \
                     run `dispatch sync --prepare-review` first"
                )
            })?;
        anyhow::ensure!(
            jrow.status == LedgerStatus::Verified,
            "candidate create: source {next} is not verified (status {:?}) — \
             no verified evidence to build a candidate from",
            jrow.status
        );
        // Phase-chain integrity: a close target built off phase/<slice>-NN must
        // have no earlier failed phase row.
        let prefix = format!("{PHASE_REF_PREFIX}{slice3}-");
        if let Some(nn) = next
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
                         below {next} has an unresolved hole",
                        r.target_ref
                    );
                }
            }
        }
        Ok(row)
    } else if is_candidate_ref(next) {
        trace_candidate_provenance(candidates, journal, slice3, next, budget - 1)
    } else {
        bail!(
            "candidate create: source candidate built from non-evidence {next} — \
             the recorded chain must terminate at a journaled evidence ref"
        )
    }
}

/// EX-1 provenance: the candidate's source ref must correspond to a journal
/// prepare-review row whose `status == Verified`. For a `phase/<slice>-NN` source
/// (a `code` close target) additionally refuse when an EARLIER non-empty
/// phase-chain row `failed` — a hole in the chain means the selected phase does
/// not actually carry verified prior code. Reads the journal from the
/// coordination branch tip (object db). Refuses (no writes) before any verified
/// evidence exists.
fn check_provenance<'a>(
    journal: &Journal,
    candidates: &'a Candidates,
    slice3: &str,
    role: CandidateRole,
    source_ref: &str,
) -> anyhow::Result<Option<&'a CandidateRow>> {
    if is_journaled_evidence_ref(source_ref, slice3) {
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
        let prefix = format!("{PHASE_REF_PREFIX}{slice3}-");
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
        Ok(None)
    } else if role == CandidateRole::CloseTarget && is_candidate_ref(source_ref) {
        let row = trace_candidate_provenance(
            candidates,
            journal,
            slice3,
            source_ref,
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        )?;
        Ok(Some(row))
    } else {
        bail!(
            "candidate create: no prepare-review journal row for source {source_ref} — \
             run `dispatch sync --prepare-review` first"
        )
    }
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
    let coord_ref = format!("{DISPATCH_REF_PREFIX}{slice3}");
    let target_ref = format!("{CANDIDATE_REF_PREFIX}{slice3}/{}", req.label);
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
    let mut ledger = read_candidates(root, req.slice)?;
    let matched_row = check_provenance(&journal, &ledger, &slice3, req.role, &source_ref)?;

    // --- resolve source + base oids (the journal proved the source verified) -
    let source_oid = resolve_commit(root, &source_ref)?
        .with_context(|| format!("candidate create: source {source_ref} does not resolve"))?;
    let base_oid = resolve_commit(root, &req.base)?
        .with_context(|| format!("candidate create: base {} does not resolve", req.base))?;

    // --- INV-6 lineage binding (RV-175 F-1): for a candidate source, the
    //     live source_oid MUST descend from the recorded merge_oid — binding
    //     resolved CONTENT (not just the ref name) to the verified-traced
    //     provenance. Source-side analog of admit's I3. -------------------------
    if let Some(row) = matched_row {
        anyhow::ensure!(
            !row.merge_oid.is_empty(),
            "candidate create: source candidate {source_ref} has an empty merge_oid — cannot verify lineage"
        );
        anyhow::ensure!(
            git::is_ancestor(root, &row.merge_oid, &source_oid)?,
            "candidate create: source candidate {} tip {} does not descend from its recorded \
             merge {} — the ref moved off its provenance lineage",
            source_ref,
            source_oid,
            row.merge_oid
        );
    }

    // --- EX-2 supersession: a fresh row links to a prior candidate id --------
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
            MergeTree::Conflict { .. } if !req.worktree => {
                // SL-127 EX-1 (§3.3): diagnostic-only base-divergence hint. The
                // drift count is resolved here in the shell; the (pure) message
                // builder appends a non-asserting hint when trunk has advanced past
                // the source, and renders BYTE-IDENTICAL legacy text when it has not.
                let ahead = trunk_drift(root, &source_oid)?.map_or(0, |d| d.ahead);
                bail!(candidate_conflict_message(&source_ref, &req.base, ahead))
            }
            // Conflicted + --worktree (SL-212 PHASE-03, design §5.4/D2): guards, an
            // atomic Conflicted row written BEFORE the worktree, an ON-BRANCH checkout,
            // then materialise merge-tree's `T_c` + unmerged stages so the operator
            // resolves the markers and `git commit`s a genuine 2-parent merge. Fully
            // self-contained — returns without the shared clean-create tail below.
            MergeTree::Conflict { tree, stages } => {
                return create_conflict_worktree(
                    root,
                    req,
                    &slice3,
                    &id,
                    &target_ref,
                    &base_oid,
                    &source_oid,
                    &source_ref,
                    &tree,
                    &stages,
                    ledger,
                    supersedes,
                );
            }
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
        // Clean --worktree: detached checkout (unchanged; the conflict arm returns
        // earlier with an on-branch checkout so the operator's commit advances the ref).
        match add_candidate_worktree(root, &id, &target_ref, false) {
            Ok(path) => {
                // CHR-030: provision gitignored embed assets (web/map/dist/)
                // so the candidate compiles out of the box. run_provision
                // reads .worktreeinclude and copies allowlisted files — the
                // same machinery used by `worktree fork`.
                if let Err(e) = run_provision(Some(root.to_path_buf()), &path) {
                    rollback_ref(root, &target_ref, &branch_oid);
                    return Err(e.context("provision candidate worktree"));
                }
                Some(path)
            }
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
        ingested_at: String::new(),
        merge_provenance: crate::ledger::MergeProvenance::Doctrine,
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
///
/// `on_branch` picks the checkout mode. `false` passes the FULL refname → git
/// detaches HEAD (the clean-arm behaviour). `true` passes the branch SHORTNAME
/// (`refs/heads/` stripped) → git checks out ON the branch, so an operator commit
/// **advances** `target_ref` — required by SL-212 so the ingest verb's
/// `resolve_commit(target_ref)` reads the resolved merge `R` (OQ-1 probe; design §5.4).
fn add_candidate_worktree(
    root: &Path,
    id: &str,
    target_ref: &str,
    on_branch: bool,
) -> anyhow::Result<PathBuf> {
    let wt_path = root.join(CANDIDATE_WORKTREE_SUBPATH).join(id);
    if let Some(parent) = wt_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let wt_str = wt_path
        .to_str()
        .context("candidate create: worktree path is not valid UTF-8")?;
    let checkout = if on_branch {
        target_ref.strip_prefix("refs/heads/").unwrap_or(target_ref)
    } else {
        target_ref
    };
    git::git_text(root, &["worktree", "add", "--quiet", wt_str, checkout])?;
    Ok(wt_path)
}

/// Best-effort CAS rollback of a ref this create just created — used when a later
/// step fails after the branch was written (EX-3: no partial durable state). A
/// failed delete is swallowed: the caller is already returning the primary error.
fn rollback_ref(root: &Path, target_ref: &str, expected: &str) {
    let _ignored = git::git_opt(root, &["update-ref", "-d", target_ref, expected]);
}

/// The two conflict-reproducibility guards SL-212 runs before trusting a mechanical
/// `merge-tree` conflict set — shared by create's conflict arm and the ingest verb
/// (design §5.4 step 1/4, D8). Refuses (a) more than one merge base (criss-cross —
/// an ambiguous 3-way a hand-resolution cannot be bound to) and (b) any custom
/// (non-built-in) `merge` driver on a path in `tc` (nondeterministic `C` — the
/// conflict set is not reproducible). Returns the single merge-base oid on success;
/// `verb` names the caller for the refusal ("candidate create" | "candidate
/// ingest"). Impure shell (git). One home for the guard pair — no third inline copy.
fn conflict_reproducibility_guards(
    root: &Path,
    base_oid: &str,
    source_oid: &str,
    tc: &str,
    verb: &str,
) -> anyhow::Result<String> {
    let bases = git::merge_base_all(root, base_oid, source_oid)?;
    anyhow::ensure!(
        bases.len() == 1,
        "{verb}: base {base_oid} and source {source_oid} have {} merge bases \
         (criss-cross) — a hand-resolved ingest needs a single 3-way base; the conflict \
         set would be ambiguous. Resolve via a different route.",
        bases.len()
    );
    let custom = git::custom_merge_driver_paths(root, tc)?;
    if let Some(path) = custom.first() {
        bail!(
            "{verb}: path {} carries a custom (non-built-in) merge driver — the \
             conflict set is not reproducible, so a hand-resolved ingest cannot be validated \
             (design §5.4/D8). Only built-in drivers (union/binary/…) are allowed.",
            String::from_utf8_lossy(path)
        );
    }
    // `len() == 1` ensured above ⇒ exactly one element; `next()` cannot be None.
    bases.into_iter().next().ok_or_else(|| {
        anyhow::anyhow!("{verb}: merge-base --all returned no base after a len==1 check")
    })
}

/// The conflict + `--worktree` create arm (SL-212 PHASE-03, design §5.4/D2). Runs
/// only when `merge-tree` reported a conflict AND `--worktree` was requested; every
/// other arm keeps the shared clean-create path (EX-5). Steps, in order:
///
/// 1. **Guards** (before any durable write): a single merge base (else criss-cross,
///    refuse) and no custom (non-built-in) merge driver on any merged path (else the
///    conflict set is not reproducible, refuse — D8). `T_c` is the superset checked.
/// 2. **CAS the branch** at `base_oid` (zero-oid create — refuses an existing ref).
/// 3. **Write the Conflicted row atomically & durably** (`merge_oid=""`) — BEFORE the
///    worktree (§3 / R-4 / EX-4), so a crash never leaves a worktree the ledger does
///    not know about. Rolls the ref back if the store fails.
/// 4. **Provision the worktree ON the branch** + **materialise** merge-tree's `T_c`
///    (no `git merge`, D2). Any failure here rolls back worktree + row + ref (§6).
#[expect(
    clippy::too_many_arguments,
    reason = "the resolved conflict-create inputs (req + computed slice3/id/target_ref/oids + \
              ledger); a single-use params struct would only relocate the same fields"
)]
fn create_conflict_worktree(
    root: &Path,
    req: &CreateRequest,
    slice3: &str,
    id: &str,
    target_ref: &str,
    base_oid: &str,
    source_oid: &str,
    source_ref: &str,
    tc: &str,
    stages: &[git::ConflictStage],
    mut ledger: Candidates,
    supersedes: String,
) -> anyhow::Result<()> {
    // --- 1. Guards — refuse before any durable state (design §5.4 step 1). The
    //        returned single base is unused here (the caller already merged with it
    //        to produce `tc`); ingest uses it to feed its own `merge_tree`. --------
    let _base =
        conflict_reproducibility_guards(root, base_oid, source_oid, tc, "candidate create")?;

    // --- 2. CAS-create the branch at base_oid (precedes the row write). -----------
    match git::update_ref_cas(root, target_ref, base_oid, ZERO_OID)? {
        RefCas::Updated => {}
        RefCas::Moved { actual } => bail!(
            "candidate create: {target_ref} already exists (at {}) — \
             supersede creates a fresh label, never rewrites a branch",
            actual.as_deref().unwrap_or("?")
        ),
    }

    // --- 3. Conflicted row (empty merge_oid) — atomic & durable BEFORE the worktree.
    let row = CandidateRow {
        id: id.to_owned(),
        label: req.label.clone(),
        kind: req.kind,
        role: req.role,
        payload: req.payload,
        target_ref: target_ref.to_owned(),
        source_ref: source_ref.to_owned(),
        source_oid: source_oid.to_owned(),
        base_ref: req.base.clone(),
        base_oid: base_oid.to_owned(),
        merge_oid: String::new(),
        status: CandidateStatus::Conflicted,
        supersedes,
        reason: String::new(),
        created_by: "dispatch candidate create".to_owned(),
        created_at: req.created_at.clone(),
        ingested_at: String::new(),
        merge_provenance: crate::ledger::MergeProvenance::Doctrine,
    };
    ledger.rows.push(row);
    if let Err(e) = crate::ledger::write_candidates(root, req.slice, &ledger) {
        rollback_ref(root, target_ref, base_oid);
        return Err(e.context("candidate create: record conflicted candidate row"));
    }

    // --- 4. Provision the worktree ON the branch, then materialise T_c. On any
    //        failure, roll back worktree + row + ref (design §5.4 step 6). ---------
    let merge_msg = format!("candidate({slice3}/{}): merge {source_ref}\n", req.label);
    let provision_and_materialise = || -> anyhow::Result<PathBuf> {
        let path = add_candidate_worktree(root, id, target_ref, true)?;
        // CHR-030 parity with the clean arm: provision gitignored embed assets so
        // the candidate compiles out of the box.
        run_provision(Some(root.to_path_buf()), &path).context("provision candidate worktree")?;
        git::materialise_conflict_worktree(&path, tc, stages, source_oid, &merge_msg)?;
        Ok(path)
    };
    let worktree_path = match provision_and_materialise() {
        Ok(path) => path,
        Err(e) => {
            rollback_conflict_worktree(root, id, req.slice, target_ref, base_oid, &ledger);
            return Err(e);
        }
    };

    writeln!(io::stdout(), "{target_ref}")?;
    writeln!(io::stdout(), "{}", worktree_path.display())?;
    writeln!(
        io::stderr(),
        "candidate create: {id} conflicted — resolve the markers and `git commit` in {}, \
         then `dispatch candidate ingest` from the coordination tree",
        worktree_path.display()
    )?;
    Ok(())
}

/// Roll back a partly-built conflict candidate (design §5.4 step 6): remove the
/// worktree, drop its row from the ledger (re-store atomically), then CAS-delete the
/// ref at `base_oid` (valid — nothing moved it). Best-effort; the caller returns the
/// primary error. `ledger` still holds the just-pushed row, so filtering by `id`
/// yields the pre-row manifest.
fn rollback_conflict_worktree(
    root: &Path,
    id: &str,
    slice: u32,
    target_ref: &str,
    base_oid: &str,
    ledger: &Candidates,
) {
    let wt_path = root.join(CANDIDATE_WORKTREE_SUBPATH).join(id);
    if let Some(wt) = wt_path.to_str() {
        let _ignored = git::git_opt(root, &["worktree", "remove", "--force", wt]);
    }
    let mut without_row = ledger.clone();
    without_row.rows.retain(|r| r.id != id);
    let _ignored = crate::ledger::write_candidates(root, slice, &without_row);
    rollback_ref(root, target_ref, base_oid);
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

/// A candidate-ingest request (SL-212 §5.2). `base`/`source` are read from the
/// recorded row, never flags — this carries only what the CLI supplies. Consumed
/// by `run_candidate_ingest` (PHASE-04).
pub(crate) struct IngestRequest {
    pub slice: u32,
    pub label: String,
    pub ingested_at: String,
}

/// A structured ingest-provenance rejection (SL-212 §5.2). Consumed by
/// `run_candidate_ingest` (PHASE-04).
pub(crate) struct IngestReject {
    pub reason: String,
}

/// Pure ingest-provenance validator (SL-212 §5.2, D1/D9) — no git/clock/disk.
///
/// Given the resolved merge commit `R`'s ordered parents and the byte-path sets
/// `D` (`= changed_paths(R^tree, T_c)`, `--no-renames`) and `C` (the conflict
/// paths), decide whether `R` is a faithful adoption of the mechanical
/// `(base, source)` 3-way — not an arbitrary tree the operator hand-built:
///
///   (i)   `parents == [base_oid, source_oid]`     — ordered; covers single/reversed/≠2
///   (ii)  `diff_from_mechanical ⊆ conflict_paths` — "never an arbitrary tree" (byte-wise)
///   (iii) `marker_paths.is_empty()`               — ADVISORY (caller fails open when it
///                                                    cannot read blobs → passes `&[]`)
///
/// Path compares are byte-wise (F8, D9); a rejection renders the offending path
/// lossily for the human message only — never for the comparison. Consumed by
/// `run_candidate_ingest` (PHASE-04).
pub(crate) fn validate_ingest_provenance(
    parents: &[String],
    base_oid: &str,
    source_oid: &str,
    diff_from_mechanical: &BTreeSet<Vec<u8>>,
    conflict_paths: &BTreeSet<Vec<u8>>,
    marker_paths: &[Vec<u8>],
) -> Result<(), IngestReject> {
    // (i) ordered parents — one match covers single / reversed / ≠2.
    if !matches!(parents, [p0, p1] if p0.as_str() == base_oid && p1.as_str() == source_oid) {
        return Err(IngestReject {
            reason: format!(
                "parents must be [base, source] = [{base_oid}, {source_oid}] in order, got {parents:?}"
            ),
        });
    }

    // (ii) D ⊆ C, byte-wise — an edit outside the conflict set is an arbitrary tree.
    if let Some(stray) = diff_from_mechanical
        .iter()
        .find(|p| !conflict_paths.contains(*p))
    {
        return Err(IngestReject {
            reason: format!(
                "resolved tree edits a non-conflict path {} — not a faithful (base, source) merge",
                String::from_utf8_lossy(stray)
            ),
        });
    }

    // (iii) surviving conflict markers — ADVISORY (caller passes &[] when unreadable).
    if let Some(marked) = marker_paths.first() {
        return Err(IngestReject {
            reason: format!(
                "conflict markers still present at {} — resolve before ingest",
                String::from_utf8_lossy(marked)
            ),
        });
    }

    Ok(())
}

/// CLI entry — resolve the root and ingest the operator's hand-resolved merge.
pub(crate) fn run_candidate_ingest(
    path: Option<PathBuf>,
    req: &IngestRequest,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_ingest(&root, req)
}

/// Core `candidate ingest` (design §5.4 Ingest, §5.3). Adopts the operator's
/// hand-resolved merge `R` (committed on `target_ref` inside the candidate worktree)
/// into the recorded Conflicted row after proving `R` is a FAITHFUL adoption of the
/// mechanical `(base, source)` 3-way — never an arbitrary hand-built tree. The
/// write-once fill is the sole durable effect; `admit → integrate` then proceed by
/// the existing FF-only contract. Refuses (fail-closed) at every gate before the fill.
fn candidate_ingest(root: &Path, req: &IngestRequest) -> anyhow::Result<()> {
    // --- 1. Coordination-root guard (design §5.3, RV-289 F-1): refuse a
    //     candidate-worktree cwd — a resolved root UNDER the candidate subpath. The
    //     linked-worktree test is unusable (the coord tree is itself linked). ------
    let resolved = std::fs::canonicalize(root).unwrap_or_else(|_ignored| root.to_path_buf());
    if resolved
        .to_string_lossy()
        .contains(CANDIDATE_WORKTREE_SUBPATH)
    {
        bail!(
            "candidate ingest: the current worktree is a candidate checkout ({}) — \
             resolve the conflicts and `git commit` here, then run `dispatch candidate ingest` \
             from the coordination tree (the ledger resolves at the coordination root, never \
             this candidate checkout's stale tree)",
            resolved.display()
        );
    }

    let slice3 = format!("{:03}", req.slice);
    let target_ref = format!("{CANDIDATE_REF_PREFIX}{slice3}/{}", req.label);

    // --- 2. Select the exactly-one Conflicted ∧ merge_oid=="" row for `label` — the
    //     fail-closed write-once gate (D5). Zero/many/Created all refuse; a Created
    //     row (merge_oid filled) is not matched, so it cannot be re-ingested. -------
    let mut ledger = read_candidates(root, req.slice)?;
    let matched: Vec<usize> = ledger
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.label == req.label
                && r.status == CandidateStatus::Conflicted
                && r.merge_oid.is_empty()
        })
        .map(|(i, _)| i)
        .collect();
    let idx = match matched.as_slice() {
        [only] => *only,
        [] => bail!(
            "candidate ingest: no un-ingested conflicted candidate for label {} — an already \
             ingested (Created) row cannot be re-ingested (write-once)",
            req.label
        ),
        many => bail!(
            "candidate ingest: {} candidates match label {} in the conflicted pre-state — \
             ambiguous; the write-once gate needs exactly one",
            many.len(),
            req.label
        ),
    };
    let row = ledger
        .rows
        .get(idx)
        .cloned()
        .with_context(|| "candidate ingest: internal — selected row index out of range")?;
    let base_oid = row.base_oid.clone();
    let source_oid = row.source_oid.clone();

    // --- 3. Resolve R = target_ref; refuse R == base (nothing committed yet). ------
    let r = resolve_commit(root, &target_ref)?.with_context(|| {
        format!("candidate ingest: {target_ref} does not resolve to a committed tip")
    })?;
    anyhow::ensure!(
        r != base_oid,
        "candidate ingest: {target_ref} still points at base {base_oid} — resolve the \
         conflicts and `git commit` in the candidate worktree before ingesting"
    );

    // --- 4/5. Recompute the mechanical merge → T_c/C, then the shared guards (single
    //     merge-base + no custom driver on T_c). A clean/empty-C result means the
    //     recorded conflict no longer reproduces — corruption, bail (design §5.4). --
    let mb = git::merge_base(root, &base_oid, &source_oid)?.with_context(|| {
        format!("candidate ingest: base {base_oid} and source {source_oid} share no ancestor")
    })?;
    let (tc, stages) = match git::merge_tree(root, &mb, &base_oid, &source_oid)? {
        git::MergeTree::Conflict { tree, stages } => (tree, stages),
        git::MergeTree::Clean { .. } => bail!(
            "candidate ingest: the recorded conflict no longer reproduces (merge is now clean) — \
             the base/source refs moved under the row; refusing to ingest against a changed merge"
        ),
    };
    anyhow::ensure!(
        !stages.is_empty(),
        "candidate ingest: merge-tree reported a conflict with an empty conflict set — corruption"
    );
    let _mb =
        conflict_reproducibility_guards(root, &base_oid, &source_oid, &tc, "candidate ingest")?;
    let conflict_paths: BTreeSet<Vec<u8>> = stages.iter().map(|s| s.path.clone()).collect();

    // --- 6. D = changed_paths(R^tree, T_c); advisory marker scan of R's blobs at C. -
    let r_tree = tree_of(root, &r)?;
    let diff_from_mechanical = git::changed_paths(root, &r_tree, &tc)?;
    let marker_paths = git::surviving_marker_paths(root, &r_tree, &conflict_paths);

    // --- 7. Pure provenance validation (ordered parents; D ⊆ C; advisory markers). --
    if let Err(reject) = validate_ingest_provenance(
        &git::parents(root, &r)?,
        &base_oid,
        &source_oid,
        &diff_from_mechanical,
        &conflict_paths,
        &marker_paths,
    ) {
        bail!("candidate ingest: {}", reject.reason);
    }

    // --- 8. Best-effort re-read — a ref moved mid-ingest is refused (D6, admit parity).
    let r2 = resolve_commit(root, &target_ref)?;
    anyhow::ensure!(
        r2.as_deref() == Some(r.as_str()),
        "candidate ingest: {target_ref} moved during ingest (was {r}, now {}) — re-run",
        r2.as_deref().unwrap_or("absent")
    );

    // --- 9. Write-once atomic fill (rides the existing temp+rename `store`). --------
    let filled = ledger
        .rows
        .get_mut(idx)
        .with_context(|| "candidate ingest: internal — selected row index out of range")?;
    filled.merge_oid.clone_from(&r);
    filled.status = CandidateStatus::Created;
    filled.ingested_at.clone_from(&req.ingested_at);
    filled.merge_provenance = crate::ledger::MergeProvenance::OperatorIngest;
    crate::ledger::write_candidates(root, req.slice, &ledger)?;

    writeln!(io::stdout(), "{r}")?;
    writeln!(
        io::stderr(),
        "candidate ingest: {} ingested at {r} (operator merge) — admit then integrate",
        row.id
    )?;
    Ok(())
}

/// CLI entry — resolve the root and admit the candidate for `req`.
pub(crate) fn run_candidate_admit(path: Option<PathBuf>, req: &AdmitRequest) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_admit(&root, req)
}

/// Core `candidate admit` (design §5.2 + §5.5 invariants). Pins a recorded
/// candidate's committed tip as the immutable `admitted_oid` a downstream verb
/// targets, after validating provenance (I3, R7): the recorded `merge_oid` is a
/// genuine candidate merge — its parents are exactly base+source (provenance, not
/// authorship — REV-030: Doctrine's 3-way OR an operator-ingested resolution) AND
/// an ancestor of the admitted tip. Re-reads the candidate ref before recording so a
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

    // --- a conflicted/unresolved row has no merge commit to validate -----------
    anyhow::ensure!(
        !row.merge_oid.is_empty(),
        "candidate admit: candidate {} has no merge to validate \
         (conflicted/unresolved) — resolve by hand and `dispatch candidate ingest` \
         (or re-create) before admitting",
        row.id
    );

    // --- provenance (EX-2, I3, R7): merge_oid is a genuine candidate merge -----
    //     (parents == base+source — provenance, not authorship; REV-030) --------
    let merge_parents: std::collections::BTreeSet<String> =
        git::parents(root, &row.merge_oid)?.into_iter().collect();
    let expected_parents: std::collections::BTreeSet<String> =
        [row.base_oid.clone(), row.source_oid.clone()]
            .into_iter()
            .collect();
    anyhow::ensure!(
        merge_parents == expected_parents,
        "candidate admit: merge_oid {} is not a genuine candidate merge \
         (parents are not {{base, source}})",
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
        (format!("{DISPATCH_REF_PREFIX}{slice3}"), "coordination"),
        (format!("{REVIEW_REF_PREFIX}{slice3}"), "impl-bundle"),
    ] {
        let tip = resolve_commit(root, &refname)?.unwrap_or_else(|| "—".to_owned());
        rows.push(EvidenceRow {
            refname,
            group,
            tip,
        });
    }
    for refname in for_each_ref(root, &format!("{PHASE_REF_PREFIX}{slice3}-*"))? {
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

/// EX-3: the safe NEXT command lines — concrete verbs the user runs, not "inspect
/// the raw refs". Pure so it is unit-testable (the impure shell just prints them).
/// Guidance branches on ledger state: no candidates ⇒ create; candidates present ⇒
/// admit/close guidance; a conflicted (parked, empty-`merge_oid`) row ⇒ the
/// sanctioned hand-resolve path `candidate ingest` (SL-212 — adopt a hand-resolved
/// merge, not just re-create); any drift ⇒ a re-admit note (the admitted oid is
/// immutable; a moved tip needs a fresh candidate).
fn next_command_lines(slice3: &str, ledger: &Candidates, any_drift: bool) -> Vec<String> {
    let slice = slice3.trim_start_matches('0');
    let slice = if slice.is_empty() { "0" } else { slice };
    if ledger.rows.is_empty() {
        return vec![format!(
            "dispatch candidate create --slice {slice} --role review_surface \
             --payload impl_bundle --base refs/heads/main --label review-001 --worktree"
        )];
    }
    let mut lines = vec![
        format!("dispatch candidate create --slice {slice} ...   # publish a fresh candidate"),
        format!(
            "dispatch candidate admit --slice {slice} --id <candidate-id> --review RV-NNN   \
             # pin a candidate for review/close"
        ),
    ];
    // SL-212: a conflicted row is parked at base with an empty merge_oid — the
    // sanctioned recovery is a hand-resolve adopted via `candidate ingest`, not a
    // re-create (which recomputes the same conflict). One line per conflicted row.
    for row in &ledger.rows {
        if row.status == CandidateStatus::Conflicted && row.merge_oid.is_empty() {
            lines.push(format!(
                "dispatch candidate ingest --slice {slice} --label {}   \
                 # adopt a hand-resolved merge for this conflicted candidate",
                row.label
            ));
        }
    }
    if any_drift {
        lines.push(
            "note: a DRIFTED candidate's live tip moved off its recorded/admitted oid \
             (immutable) — supersede with a fresh candidate rather than editing in place"
                .to_owned(),
        );
    }
    lines
}

/// Print the EX-3 next-command block ([`next_command_lines`]) under a `next:`
/// header, two-space indented. Impure shell.
fn write_next_commands(slice3: &str, ledger: &Candidates, any_drift: bool) -> anyhow::Result<()> {
    writeln!(io::stdout(), "\nnext:")?;
    for line in next_command_lines(slice3, ledger, any_drift) {
        writeln!(io::stdout(), "  {line}")?;
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
        .filter(|r| {
            matches!(
                r.provenance,
                BoundaryProvenance::Funnel | BoundaryProvenance::Unknown
            )
        })
        .map(|r| r.phase.as_str())
        .filter(|p| !committed.contains(p))
        .collect()
}

/// Stage-1 prepare-review (design §4.2 B + §4.3 C).
fn prepare_review(root: &Path, slice: u32) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");
    let coord_ref = format!("{DISPATCH_REF_PREFIX}{slice3}");
    let journal_path = format!(".doctrine/dispatch/{slice3}/journal.toml");

    let tip0 = resolve_commit(root, &coord_ref)?
        .with_context(|| format!("prepare-review: dispatch/{slice3} does not exist"))?;
    // SL-221 PHASE-05: the run ledger is sourced EXCLUSIVELY from the dispatch ref
    // (object db). `conclude`/`land_boundary_row` (P03) commit every boundary row
    // straight onto `dispatch/<slice>`, so there is no uncommitted working-tree
    // ledger left to splice — the working-tree path is retired (ISS-225: a stale
    // working ledger can no longer clobber the concluded ref rows). `read_ledger`/
    // `plan_phases` read the ref directly below.
    let tip = tip0;
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
    allow: &BTreeSet<String>,
) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");
    let coord_ref = format!("{DISPATCH_REF_PREFIX}{slice3}");
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
            plan_candidate_trunk_row(root, &slice3, &journal, &candidates, trunk_ref)?
        } else {
            plan_trunk_row(root, &slice3, &journal, &candidates, trunk_ref)?
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

    // --- record the operator g3 corpus-clobber allowlist on the committed journal
    //     (SL-166 EX-4): call-global across both legs (§10), an audit trail of
    //     what the orchestrator waved through. Empty by default ⇒ g3 fail-closed.
    journal.allowed_clobbers = allow.iter().cloned().collect();

    // --- journal the (possibly extended) intent onto the branch BEFORE any
    //     external ref mutation (EX-5, ADR-012 D4); advance every row idempotently
    //     — exact-CAS classification, worktree-aware mechanism (§2.2, EX-2..EX-5);
    //     g3 (always-on) refuses a corpus-clobbering advance before the mutation;
    //     record applied status back. ---------------------------------------------
    let outcomes = with_journaled_projection(
        root,
        &tip,
        &tip_tree,
        &journal_path,
        &coord_ref,
        &mut journal,
        "journal: integrate",
        |root, row| advance_row(root, row, allow),
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
fn advance_row(
    root: &Path,
    row: &mut JournalRow,
    allow: &BTreeSet<String>,
) -> anyhow::Result<RowOutcome> {
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

    // g3 — the always-on 3-way corpus-clobber gate (SL-166 design §5.2/§5.5).
    // Before the mutation, refuse an advance that would delete or revert authored
    // `.doctrine/**` paths the live target holds. Inert when the target is absent
    // (a creation holds nothing to clobber) and on a true fast-forward (planned
    // descends current ⇒ base == current ⇒ empty changed-set). Load-bearing now on
    // the un-gated `--edge` leg (RV-176 F-2); forward-insurance for RFC-006.
    if current != ZERO_OID
        && let Some(token) = corpus_clobber_refusal(root, &planned, current, allow)?
    {
        row.status = LedgerStatus::Failed;
        return Ok(RowOutcome::Refused { token });
    }

    // current == expected_old → a real advance. The ONLY place the mechanism
    // branches on checkout state.
    match git::worktree_for_ref(root, &row.target_ref)? {
        None => advance_pure_ref(root, row, &planned, &expected_old),
        Some(wt) => advance_checked_out(root, row, &wt, &planned, &expected_old),
    }
}

/// g3 shell (SL-166 design §5.2): read the three trees and run the pure
/// [`corpus_guard::corpus_clobber_check`] predicate. Returns the refusal token (a
/// capped path list) when advancing the target from `cur` to `new` would clobber
/// an unallowed authored `.doctrine` path, else `None`. `base = merge-base(new,
/// cur)`, falling back to the empty tree when their histories are unrelated. All
/// git I/O lives here (the impure shell); the predicate stays a pure leaf.
fn corpus_clobber_refusal(
    root: &Path,
    new: &str,
    cur: &str,
    allow: &BTreeSet<String>,
) -> anyhow::Result<Option<String>> {
    let base = git::merge_base(root, new, cur)?.unwrap_or_else(|| git::EMPTY_TREE_OID.to_owned());
    let changed = git::diff_doctrine_paths(root, &base, cur, corpus_guard::DOCTRINE_PATHSPEC)?;
    if changed.is_empty() {
        return Ok(None);
    }
    let readings = changed
        .into_iter()
        .map(|path| -> anyhow::Result<corpus_guard::ClobberReading> {
            let base_oid = git::blob_oid_at(root, &base, &path)?;
            let new_oid = git::blob_oid_at(root, new, &path)?;
            Ok(corpus_guard::ClobberReading {
                path,
                base_oid,
                new_oid,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let clobbers = corpus_guard::corpus_clobber_check(&readings, allow);
    if clobbers.is_empty() {
        Ok(None)
    } else {
        Ok(Some(format!(
            "{} ({})",
            corpus_guard::CORPUS_CLOBBER,
            corpus_guard::render_clobbers(&clobbers, corpus_guard::CLOBBER_RENDER_CAP),
        )))
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
    let prefix = format!("{PHASE_REF_PREFIX}{slice3}-");
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

/// Shared refusal (SL-068 I6/R4) when a candidate workflow is active but no
/// `close_target` admission exists — integration will NOT fall back to a raw
/// phase ref. Verb-neutral: the single source string every trunk consumer
/// reaches via [`resolve_trunk_payload`] (STD-001).
const NO_CLOSE_TARGET_ADMISSION: &str = "a candidate workflow is active but no close_target admission exists — run \
     `dispatch candidate admit --role close_target` first; integration will not fall back \
     to a raw phase ref";

/// The SOLE home of the candidate-vs-legacy trunk *source* decision (SL-211
/// EX-1). An empty candidate ledger ⇒ legacy: the verified phase-chain tip (NOT
/// `review/<N>`). A recorded candidate workflow ⇒ the admitted `close_target`
/// OID, REFUSING (no raw-evidence fallback, EX-2) when no such admission exists.
/// Returns the resolved source oid; `integrate()` and both trunk planners route
/// their `planned` through here — no duplicated source resolution remains.
fn resolve_trunk_payload(
    root: &Path,
    slice3: &str,
    journal: &Journal,
    candidates: &Candidates,
) -> anyhow::Result<String> {
    if candidates.rows.is_empty() {
        // legacy: the phase-chain tip (NOT review/<N>).
        let phase_ref = phase_chain_tip(journal, slice3)
            .with_context(|| format!("no phase/{slice3}-NN code units to integrate"))?;
        resolve_commit(root, &phase_ref)?.with_context(|| format!("{phase_ref} does not resolve"))
    } else {
        // candidate-active: the admitted close_target — REFUSE if none.
        candidates
            .current_admission
            .close_target
            .as_ref()
            .context(NO_CLOSE_TARGET_ADMISSION)
            .map(|a| a.admitted_oid.clone())
    }
}

/// A TERMINAL, already-applied trunk-record row: a *statement of fact* that
/// `payload` sits on `trunk_ref`, not an intent to move it (contrast
/// [`projection_row`], a Pending CAS advance). All four oid fields are the
/// payload and the status is `Verified`. The replay-safety differences (D2):
/// `expected_old == planned == payload` (NOT the trunk tip), so a stray later
/// `--integrate` re-checks `is_ancestor(payload, trunk)` and converges to a
/// non-destructive Refused/no-op — never a backward advance; `applied == payload`
/// (already applied, not an empty pending slot). Consumed by
/// [`plan_recorded_trunk_row`] (the record-integration handler, SL-211 PHASE-03).
fn recorded_row(trunk_ref: &str, payload: String) -> JournalRow {
    JournalRow {
        source_oid: payload.clone(),
        target_ref: trunk_ref.to_owned(),
        expected_old_oid: payload.clone(), // = planned, NOT the trunk tip (D2)
        planned_new_oid: payload.clone(),  // the gate re-checks is_ancestor(this, trunk)
        applied_new_oid: payload,          // already applied — terminal, not pending
        status: LedgerStatus::Verified,    // statement of fact, not intent
    }
}

/// Plan the recorder row for `record-integration`: resolve the trunk payload via
/// the shared [`resolve_trunk_payload`] seam, then hold it to the **same standard
/// the integrate gate holds** — the payload must ALREADY be an ancestor of the
/// trunk tip (R1 negative: recording is a statement of fact, not a way to force
/// one). Returns the earned terminal row on success; REFUSES (named error) when
/// the trunk ref does not resolve or the payload is not on trunk. Pure planner
/// posture (A1): git lives in the shell (`resolve_commit`, `git::is_ancestor`
/// take `root`); the row construction takes OIDs only. Consumed by
/// [`run_record_integration`] (SL-211 PHASE-03).
fn plan_recorded_trunk_row(
    root: &Path,
    slice3: &str,
    journal: &Journal,
    candidates: &Candidates,
    trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let payload = resolve_trunk_payload(root, slice3, journal, candidates)?;
    let tip = resolve_commit(root, trunk_ref)?.with_context(|| {
        format!("record-integration: {trunk_ref} does not resolve — no trunk to record onto")
    })?;
    // EARNED CHECK (R1 negative): the payload must already be on trunk — the same
    // standard the gate holds (is_ancestor proves integration occurred).
    anyhow::ensure!(
        git::is_ancestor(root, &payload, &tip)?,
        "record-integration: trunk payload {payload} is not an ancestor of {trunk_ref} \
         (at {tip}) — land it (`git merge --no-ff phase/{slice3}-NN` or the admitted \
         candidate) before recording"
    );
    Ok(recorded_row(trunk_ref, payload))
}

/// Plan the trunk projection row (EX-3): the cumulative code tip advances
/// `trunk_ref` **fast-forward-only**. `expected_old` is the trunk tip (zero if the
/// ref is absent); a planned commit that does not descend from it ⇒ the trunk
/// moved ⇒ refuse (re-anchor is reported, never auto-resolved).
fn plan_trunk_row(
    root: &Path,
    slice3: &str,
    journal: &Journal,
    candidates: &Candidates,
    trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let planned = resolve_trunk_payload(root, slice3, journal, candidates)?;
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
    let review_ref = format!("{REVIEW_REF_PREFIX}{slice3}");
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
    slice3: &str,
    journal: &Journal,
    candidates: &Candidates,
    trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let planned = resolve_trunk_payload(root, slice3, journal, candidates)?;
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
        target_ref: format!("{REVIEW_REF_PREFIX}{slice3}"),
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
    // Chain by ascending PHASE ordinal, NOT on-disk row order: the escape hatch
    // (PHASE-03, D-B4) can tail-append an out-of-order row onto the boundaries
    // ref, so row order ≠ phase order. Normalise HERE, at the sole order-sensitive
    // consumer — a LOCAL sorted view; the on-disk ledger order is left untouched.
    // Parse the ordinal from the same `strip_prefix("PHASE-")` the walk uses;
    // malformed / non-`PHASE-NN` labels (`None`) sort LAST, stably (stable sort
    // preserves input order among equal keys).
    let ordinal = |row: &BoundaryRow| -> Option<u32> {
        row.phase.strip_prefix("PHASE-")?.parse::<u32>().ok()
    };
    let mut ordered: Vec<&BoundaryRow> = boundaries.rows.iter().collect();
    ordered.sort_by_key(|row| {
        let ord = ordinal(row);
        (ord.is_none(), ord.unwrap_or(0))
    });

    let mut parent = trunk_base.to_owned();
    for boundary in ordered {
        if boundary.code_start_oid == boundary.code_end_oid {
            continue; // empty-code phase — no branch cut (design §4.3)
        }
        let nn = boundary
            .phase
            .strip_prefix("PHASE-")
            .unwrap_or(&boundary.phase);
        let code_tree = tree_of(root, &boundary.code_end_oid)?;
        let phase_tree =
            git::filter_tree(root, &code_tree, &[crate::corpus_guard::DOCTRINE_PATHSPEC])?;
        let phase_commit =
            git::commit_tree(root, &phase_tree, &parent, &format!("phase({slice3}-{nn})"))?;
        planned.push(Planned {
            target_ref: format!("{PHASE_REF_PREFIX}{slice3}-{nn}"),
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
        allowed_clobbers: Vec::new(),
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

    // Read the plan + runtime sheets into ordered `(id, status, name)` rows via
    // the shared readiness seam (EX-7) — the SAME value `dispatch_next_ready`
    // (MCP) consumes, so the two never re-derive. `next` rides the shared
    // `compute_next_phases` authority, byte-identical to the pre-refactor inline
    // scan (the same guarantee `run_status` already relies on).
    let rows = plan_next_rows(&root, slice)?;
    let next = compute_next_phases(&rows);
    // `saw_blocked` only gates the all-blocked message, and is only read when
    // `next` is empty — where it equals "any row is blocked" (the pre-refactor
    // scan, unable to find an actionable phase, walked every row and set the flag
    // on each blocked one).
    let saw_blocked = rows.iter().any(|(_, status, _)| status == "blocked");

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

/// A planned phase's receipt status, sourced across three tiers: the plan (the
/// phase set), the disposable runtime sheet, and the COMMITTED boundaries ledger
/// (SL-206 PHASE-02, EX-5). Unlike the sheet-only status string it enriches,
/// `Completed` is sourced from a committed boundary row — a sheet that merely
/// claims "completed" with no committed boundary is surfaced distinctly as
/// `ConcludeIncomplete` (the gap the sheet alone hides). A genuinely unreadable
/// sheet is `Unknown` (fail-loud), never silently collapsed to `NotStarted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReceiptStatus {
    /// No runtime sheet row — the phase has no recorded progress.
    NotStarted,
    /// The sheet records the phase in progress.
    InProgress,
    /// The sheet records the phase blocked.
    Blocked,
    /// A committed boundary row backs the phase — the authority for done.
    Completed,
    /// The sheet says "completed" but NO committed boundary row backs it.
    ConcludeIncomplete,
    /// The sheet is unreadable (an IO error, not merely absent) — fail-loud.
    Unknown,
}

impl ReceiptStatus {
    /// The distinct kebab-case token each status surfaces as over the read-only
    /// funnel tools (SL-206 PHASE-03) — richer than the legacy phase-row string
    /// ([`receipt_status_legacy_str`]): `ConcludeIncomplete` stays SEPARATE from
    /// `Completed` (the sheet-completed-but-uncommitted gap the enum exists to
    /// surface). STD-001 single-source — no bare literal at the tool call sites.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ReceiptStatus::NotStarted => "not-started",
            ReceiptStatus::InProgress => "in-progress",
            ReceiptStatus::Blocked => "blocked",
            ReceiptStatus::Completed => "completed",
            ReceiptStatus::ConcludeIncomplete => "conclude-incomplete",
            ReceiptStatus::Unknown => "unknown",
        }
    }
}

/// One projected row per planned phase: its id, name, the durable
/// [`ReceiptStatus`] classification, and the verbatim legacy status string
/// (SL-206 PHASE-02). `run_status` folds this into its legacy
/// `(id, status_string, name)` tuple using `legacy_status` — NOT the enum — so
/// `dispatch status` output stays byte-identical to the pre-refactor code; the
/// enum enriches for PHASE-03 consumers but never alters the rendered string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhaseProjection {
    pub id: String,
    pub name: String,
    pub status: ReceiptStatus,
    /// The verbatim sheet-derived status string under the ORIGINAL `run_status`
    /// mapping — `Ok(Some(s)) ⇒ s` (so a `planned` skeleton stays "planned"),
    /// `Ok(None) ⇒ "pending"`, `Err ⇒ "unknown"`. The behaviour-preserving
    /// source for the rendered phase row (EX-4).
    pub legacy_status: String,
}

/// Derive a phase's [`ReceiptStatus`] from its runtime-sheet read and whether a
/// committed boundary row backs it. `Completed` is boundary-sourced; a
/// sheet-"completed" with no boundary is `ConcludeIncomplete`. A read *error* is
/// `Unknown` (fail-loud), never `NotStarted`. The `planned` skeleton (a phase
/// materialised but not yet started) has no recorded progress ⇒ `NotStarted`; an
/// unrecognised status string ⇒ `Unknown`. Pure over its two inputs.
fn derive_receipt_status(
    sheet: anyhow::Result<Option<String>>,
    has_boundary: bool,
) -> ReceiptStatus {
    use crate::state::PhaseStatus;
    match sheet {
        Err(_) => ReceiptStatus::Unknown,
        Ok(None) => ReceiptStatus::NotStarted,
        Ok(Some(s)) => {
            if s == PhaseStatus::InProgress.as_str() {
                ReceiptStatus::InProgress
            } else if s == PhaseStatus::Blocked.as_str() {
                ReceiptStatus::Blocked
            } else if s == PhaseStatus::Completed.as_str() {
                if has_boundary {
                    ReceiptStatus::Completed
                } else {
                    ReceiptStatus::ConcludeIncomplete
                }
            } else if s == PhaseStatus::Planned.as_str() {
                ReceiptStatus::NotStarted
            } else {
                ReceiptStatus::Unknown
            }
        }
    }
}

/// Map a [`ReceiptStatus`] back to a legacy phase-row status string —
/// `Completed`/`ConcludeIncomplete` both render "completed". A durable
/// classification→string projection for future [`ReceiptStatus`] consumers
/// (PHASE-03). NOTE: `run_status` does NOT use this — it renders the verbatim
/// `PhaseProjection::legacy_status` to stay byte-identical to the pre-refactor
/// output (a `planned` skeleton must survive as "planned", which the enum cannot
/// carry). Only the projection tests exercise it today.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "PHASE-03 consumers are the first non-test callers; run_status renders the verbatim legacy string for behaviour-preservation"
    )
)]
fn receipt_status_legacy_str(status: ReceiptStatus) -> &'static str {
    match status {
        ReceiptStatus::Completed | ReceiptStatus::ConcludeIncomplete => "completed",
        ReceiptStatus::InProgress => "in_progress",
        ReceiptStatus::Blocked => "blocked",
        ReceiptStatus::NotStarted => "pending",
        ReceiptStatus::Unknown => "unknown",
    }
}

/// Read-only projection of a slice's planned phases over three tiers — the plan
/// (`plan`, the phase set + names), the disposable runtime sheet (per-phase
/// progress under `state_dir`), and the committed boundaries ledger (`committed`,
/// the phase-id set with a committed boundary row — the authority for
/// `Completed`). Yields one [`PhaseProjection`] per planned phase, in plan order
/// (SL-206 PHASE-02, EX-5). The git read of the ledger stays in the caller so
/// this reader touches only the sheet.
fn phase_projection(
    plan: &crate::plan::Plan,
    state_dir: &Path,
    committed: &BTreeSet<&str>,
) -> Vec<PhaseProjection> {
    plan.phases
        .iter()
        .map(|ph| {
            let stem = ph.id.to_lowercase();
            let sheet = crate::state::read_phase_status(state_dir, &stem);
            let has_boundary = committed.contains(ph.id.as_str());
            // The verbatim legacy string, under the ORIGINAL run_status mapping —
            // computed off the same read, before the read is consumed by the
            // ReceiptStatus derivation. This is what run_status renders (EX-4).
            let legacy_status = match &sheet {
                Ok(Some(s)) => s.clone(),
                Ok(None) => "pending".to_string(),
                Err(_) => "unknown".to_string(),
            };
            PhaseProjection {
                id: ph.id.clone(),
                name: ph.name.clone(),
                status: derive_receipt_status(sheet, has_boundary),
                legacy_status,
            }
        })
        .collect()
}

/// A single planned phase's [`ReceiptStatus`], over the SAME PHASE-02 projection
/// authority [`phase_projection`] applies per row (SL-206 PHASE-03): the phase's
/// disposable runtime sheet read under `state_dir`, folded with `has_boundary`
/// (whether a committed boundary row backs it) through [`derive_receipt_status`].
/// The single-phase seam the `dispatch_phase_receipt` MCP tool consumes — no
/// parallel status derivation. The caller supplies `has_boundary` from the
/// committed ledger read (the git touch stays in the caller, as in `run_status`).
pub(crate) fn phase_receipt_status(
    state_dir: &Path,
    phase: &str,
    has_boundary: bool,
) -> ReceiptStatus {
    let stem = phase.to_lowercase();
    let sheet = crate::state::read_phase_status(state_dir, &stem);
    derive_receipt_status(sheet, has_boundary)
}

/// `doctrine dispatch status` — read-only full dispatch rollup: coordination
/// state, phase table, trunk drift, sync state, candidate summary, next-step
/// guidance. Read-only — callable from anywhere.
pub(crate) fn run_status(path: Option<PathBuf>, slice: u32, json: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice3 = format!("{slice:03}");
    let dispatch_ref = format!("{DISPATCH_REF_PREFIX}{slice3}");

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
    // The committed boundaries ledger (the authority for `Completed`), read from
    // the dispatch tip's object db. Absent/empty ⇒ empty set, so a sheet-
    // "completed" phase projects `ConcludeIncomplete` — which still renders
    // "completed" (EX-4), preserving today's output while surfacing the gap to
    // ReceiptStatus consumers.
    let boundaries = read_ledger::<Boundaries>(&root, &dispatch_ref, &slice3, "boundaries.toml")?;
    let committed: BTreeSet<&str> = boundaries.rows.iter().map(|r| r.phase.as_str()).collect();
    // Delegate per-phase row construction to the projection, then fold each row
    // back to the legacy `(id, status_string, name)` tuple. The status string is
    // the VERBATIM `legacy_status` (the original Ok(Some(s))→s / None→"pending" /
    // Err→"unknown" mapping), NOT derived from ReceiptStatus — so ALL downstream
    // (all_completed, compute_next_phases on the literal "pending", the sheet's
    // "planned", render_phase_table, JSON) is byte-identical (SL-206 PHASE-02,
    // EX-4). ReceiptStatus rides alongside for PHASE-03 consumers only.
    let phase_rows: Vec<(String, String, String)> = phase_projection(&plan, &state_dir, &committed)
        .into_iter()
        .map(|p| (p.id, p.legacy_status, p.name))
        .collect();

    // --- Sync state ------------------------------------------------------------
    let review_ref = format!("{REVIEW_REF_PREFIX}{slice3}");
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
    let target_branch = format!("{DISPATCH_REF_PREFIX}{slice3}");
    match git::worktree_for_ref(root, &target_branch) {
        Ok(Some(path)) => path.to_string_lossy().into_owned(),
        Ok(None) | Err(_) => "(removed)".to_string(),
    }
}

/// Count `refs/heads/phase/{slice3}-*` refs via `git for-each-ref`.
fn count_phase_refs(root: &Path, slice3: &str) -> usize {
    let pattern = format!("{PHASE_REF_PREFIX}{slice3}-*");
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
/// Read a slice's plan + disposable runtime phase sheets into ordered
/// `(id, legacy_status, name)` rows — the readiness input BOTH `dispatch
/// plan-next` (CLI, [`run_plan_next`]) and `dispatch_next_ready` (the MCP funnel
/// tool) consume, so neither re-derives it (SL-206 PHASE-03, EX-7). Rides the
/// PHASE-02 [`phase_projection`] authority with an EMPTY committed set:
/// `plan-next` never consulted the committed boundaries ledger, and
/// `legacy_status` is boundary-independent, so the rows stay byte-identical to
/// the pre-refactor inline read.
pub(crate) fn plan_next_rows(
    root: &Path,
    slice: u32,
) -> anyhow::Result<Vec<(String, String, String)>> {
    let plan = crate::slice::read_plan(&root.join(".doctrine/slice"), slice)?;
    let state_dir = crate::state::phases_dir(root, slice);
    let committed: BTreeSet<&str> = BTreeSet::new();
    Ok(phase_projection(&plan, &state_dir, &committed)
        .into_iter()
        .map(|p| (p.id, p.legacy_status, p.name))
        .collect())
}

/// The next actionable phase(s) over the readiness rows — the SOLE readiness
/// authority (SL-206 PHASE-03, EX-2): scan in plan order, skip completed; the
/// first actionable `in_progress` gates alone; the first actionable pending runs
/// with its consecutive pending followers. Both `dispatch plan-next` (via
/// [`run_plan_next`]), `run_status`, and the `dispatch_next_ready` MCP tool
/// consume this — no parallel readiness logic exists.
pub(crate) fn compute_next_phases(rows: &[(String, String, String)]) -> Vec<String> {
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

// ======================================================================================
// The object-db commit engine (SL-221 PHASE-01 — relocated DOWN from the command tier,
// ADR-001: `dispatch.rs` is engine, `mcp_server` is command). Composes ONE non-merge
// commit working-tree-free onto an EXPLICIT `target_ref` and lands it with a
// compare-and-swap; on ANY fault the live index + worktree are BYTE-UNCHANGED.
// ======================================================================================

/// The stable funnel marker every landed dispatch commit is grepped by (STD-001
/// single-source — no bare literal at the call sites). Naming the slice/phase keeps the
/// history greppable across a run.
const FUNNEL_MARKER: &str = "dispatch-funnel";

/// The grep-stable commit message the funnel lands: `<marker>: SL-<NNN> <phase>`
/// (provenance contract — the orchestrator recovers slice/phase from the subject).
pub(crate) fn funnel_message(slice: u32, phase: &str) -> String {
    format!("{FUNNEL_MARKER}: SL-{slice:03} {phase}")
}

/// The canonical coordination branch ref for a slice, `refs/heads/dispatch/<NNN>` (STD-001
/// single-source — the CAS target [`commit_on_behalf`] advances). The one seam both funnel
/// production call sites derive the explicit `target_ref` from.
pub(crate) fn dispatch_ref(slice: u32) -> String {
    format!("{DISPATCH_REF_PREFIX}{slice:03}")
}

/// A git author/committer identity (`<name> <email>`), form `<id> <id@doctrine>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Identity {
    pub(crate) name: String,
    pub(crate) email: String,
}

/// Which identities a landed commit carries (provenance contract, design §B):
/// IMPORT preserves the worker AUTHOR + dispatch COMMITTER; CONCLUDE sets
/// author == committer == the dispatch id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Provenance {
    /// Import a worker delta: keep the worker's authorship, stamp the dispatch committer.
    Import {
        author: Identity,
        committer: Identity,
    },
    /// Conclude on the funnel's own behalf: author == committer == the dispatch id.
    Conclude { who: Identity },
}

impl Provenance {
    fn author(&self) -> &Identity {
        match self {
            Provenance::Import { author, .. } => author,
            Provenance::Conclude { who } => who,
        }
    }

    fn committer(&self) -> &Identity {
        match self {
            Provenance::Import { committer, .. } => committer,
            Provenance::Conclude { who } => who,
        }
    }
}

/// Why [`commit_on_behalf`] refuses (design §B) — a semantic refusal, distinct from a
/// plumbing error (which stays an `anyhow` `Err`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitRefusal {
    /// The composed tree equals the parent's tree — no change; never mint an empty commit.
    EmptyDelta,
    /// The CAS found the tip moved off `expected_old` (a lost-ref race). Nothing written;
    /// the composed commit is left dangling, the ref untouched.
    LostRefRace,
}

impl CommitRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            CommitRefusal::EmptyDelta => "empty-delta",
            CommitRefusal::LostRefRace => "lost-ref-race",
        }
    }
}

/// The outcome of [`commit_on_behalf`]: a landed commit oid, or a typed refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommitOutcome {
    /// One non-merge commit `C` landed on the coord branch, `C^ == expected_old`.
    Landed { oid: String },
    /// A belt refused; the ref + live index + worktree are byte-unchanged.
    Refused(CommitRefusal),
}

/// The dispatch committer identity every funnel commit carries (STD-001 — the same
/// `<name> <email>` the §B provenance tests pin).
const DISPATCH_NAME: &str = "dispatch";
const DISPATCH_EMAIL: &str = "dispatch@doctrine";

/// The dispatch committer/author identity (`Conclude` author==committer; `Import`
/// committer).
pub(crate) fn dispatch_identity() -> Identity {
    Identity {
        name: DISPATCH_NAME.to_owned(),
        email: DISPATCH_EMAIL.to_owned(),
    }
}

/// Compose a commit of `tree` onto `parent` with EXPLICIT author/committer identities,
/// working-tree-free (mirror of `git::commit_empty_tree_as`, but on a NAMED tree with a
/// two-identity provenance — the one variant the shipped `commit_tree` lacks). Reads no
/// index and no working tree; `git commit-tree` operates on object-db oids only.
fn commit_tree_as(
    root: &Path,
    tree: &str,
    parent: &str,
    message: &str,
    prov: &Provenance,
) -> anyhow::Result<String> {
    let author = prov.author();
    let committer = prov.committer();
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["commit-tree", tree, "-p", parent, "-m", message])
        .env("GIT_AUTHOR_NAME", &author.name)
        .env("GIT_AUTHOR_EMAIL", &author.email)
        .env("GIT_COMMITTER_NAME", &committer.name)
        .env("GIT_COMMITTER_EMAIL", &committer.email)
        .output()
        .with_context(|| format!("spawn git commit-tree in {}", root.display()))?;
    if !output.status.success() {
        bail!(
            "commit-tree {tree} -p {parent}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Compose ONE non-merge commit of `tree` onto `target_ref`'s tip and land it with a
/// compare-and-swap against `expected_old` (design §B). `git_root` is the git tree the
/// object-db reads/writes route through; `target_ref` is the EXPLICIT branch the CAS
/// advances (no longer derived from the coord worktree's `HEAD`). The commit is built
/// object-db-only (`commit-tree` — VT-4: a pre-existing staged/dirty tree is NEVER swept
/// in, because the compose reads the NAMED `tree`, not the live index).
///
/// Refuses (typed, not an error): `empty-delta` when `tree` matches the parent's tree
/// (no change); `lost-ref-race` when the CAS finds the tip moved off `expected_old`. On
/// EITHER refusal — and on any fault before the ref advance — the ref, index, and
/// worktree are BYTE-UNCHANGED (the primitive touches none of them; a dangling commit
/// object is inert). A genuine plumbing failure stays an `anyhow` `Err`.
pub(crate) fn commit_on_behalf(
    git_root: &Path,
    target_ref: &str,
    expected_old: &str,
    tree: &str,
    message: &str,
    prov: &Provenance,
) -> anyhow::Result<CommitOutcome> {
    // empty-delta: the composed tree equals the parent's tree ⇒ no change (never mint an
    // empty commit). Peeling `^{tree}` accepts either a tree or a commit as `tree`.
    let parent_tree = git::git_text(
        git_root,
        &["rev-parse", &format!("{expected_old}^{{tree}}")],
    )
    .context("resolve parent tree")?;
    let new_tree = git::git_text(git_root, &["rev-parse", &format!("{tree}^{{tree}}")])
        .context("resolve composed tree")?;
    if new_tree == parent_tree {
        return Ok(CommitOutcome::Refused(CommitRefusal::EmptyDelta));
    }

    // Compose the single non-merge commit (one `-p`) — object-db only, no working tree.
    let oid = commit_tree_as(git_root, tree, expected_old, message, prov)?;

    // Lost-ref-race guard: advance the EXPLICIT target ref ONLY if it still equals
    // `expected_old`.
    match git::update_ref_cas(git_root, target_ref, &oid, expected_old)? {
        git::RefCas::Updated => Ok(CommitOutcome::Landed { oid }),
        git::RefCas::Moved { .. } => Ok(CommitOutcome::Refused(CommitRefusal::LostRefRace)),
    }
}

/// UPSERT `row` (by phase) into the committed boundaries at `tip` and land it on
/// `coord_ref` with one working-tree-free commit. The single boundary writer for
/// both the funnel (conclude) and the CLI escape hatch (record-boundary).
pub(crate) fn land_boundary_row(
    git_root: &Path,
    coord_ref: &str,
    tip: &str,
    slice: u32,
    row: BoundaryRow,
    prov: &Provenance,
) -> anyhow::Result<CommitOutcome> {
    let path = format!(".doctrine/dispatch/{slice:03}/boundaries.toml");
    let phase = row.phase.clone(); // captured before the UPSERT consumes `row`
    let mut b =
        read_ledger::<Boundaries>(git_root, tip, &format!("{slice:03}"), "boundaries.toml")?;
    match b.rows.iter_mut().find(|r| r.phase == row.phase) {
        Some(existing) => *existing = row, // UPSERT by phase (funnel + escape hatch alike)
        None => b.rows.push(row),
    }
    let tree = git::tree_with_file(git_root, &tree_of(git_root, tip)?, &path, &b.to_toml()?)?;
    commit_on_behalf(
        git_root,
        coord_ref,
        tip,
        &tree,
        &funnel_message(slice, &phase),
        prov,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::SCHEMA_PLAN_OVERVIEW;
    use std::fs;
    use std::path::Path;

    // ==================================================================================
    // The object-db commit engine — relocated unit tests (SL-221 PHASE-01, VT-1/VT-2).
    // ==================================================================================

    fn git_run(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn disp() -> Provenance {
        Provenance::Conclude {
            who: Identity {
                name: "dispatch".to_string(),
                email: "dispatch@doctrine".to_string(),
            },
        }
    }

    /// Stand up a primary repo at base B with a linked coordination worktree on
    /// `dispatch/<NNN>` at `<tmp>/coord`. Returns `(tmp, primary, coord, base, base_tree)`.
    fn primary_with_coord(slice: u32) -> (tempfile::TempDir, PathBuf, PathBuf, String, String) {
        let tmp = tempfile::tempdir().unwrap();
        let primary = fs::canonicalize(tmp.path()).unwrap().join("primary");
        fs::create_dir_all(&primary).unwrap();
        git_run(&primary, &["init", "-q", "-b", "main"]);
        git_run(&primary, &["config", "user.email", "t@t"]);
        git_run(&primary, &["config", "user.name", "t"]);
        fs::write(primary.join("seed"), "base\n").unwrap();
        git_run(&primary, &["add", "-A"]);
        git_run(&primary, &["commit", "-q", "-m", "base"]);
        let base = git_run(&primary, &["rev-parse", "HEAD^{commit}"]);
        let base_tree = git_run(&primary, &["rev-parse", "HEAD^{tree}"]);

        let coord = fs::canonicalize(tmp.path()).unwrap().join("coord");
        git_run(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                &format!("dispatch/{slice:03}"),
                coord.to_str().unwrap(),
                &base,
            ],
        );
        let coord = fs::canonicalize(&coord).unwrap();
        (tmp, primary, coord, base, base_tree)
    }

    impl CommitOutcome {
        fn token_is(&self, want: &str) -> bool {
            matches!(self, CommitOutcome::Refused(r) if r.token() == want)
        }
    }

    // --- VT-1: land_boundary_row (the single UPSERT-by-phase boundary writer) ----------

    /// Read the COMMITTED `boundaries.toml` at `oid` (object-db, never the working tree).
    fn boundaries_at(coord: &Path, oid: &str) -> Boundaries {
        let text = git_run(
            coord,
            &[
                "show",
                &format!("{oid}:.doctrine/dispatch/199/boundaries.toml"),
            ],
        );
        Boundaries::parse(&text).unwrap()
    }

    #[test]
    fn land_boundary_row_upserts_by_phase() {
        let (_tmp, _primary, coord, base, _bt) = primary_with_coord(199);
        let coord_ref = dispatch_ref(199);
        let mk = |phase: &str, s: &str, e: &str| BoundaryRow {
            phase: phase.to_owned(),
            code_start_oid: s.to_owned(),
            code_end_oid: e.to_owned(),
            provenance: BoundaryProvenance::Funnel,
        };
        let land = |tip: &str, row: BoundaryRow| match land_boundary_row(
            &coord,
            &coord_ref,
            tip,
            199,
            row,
            &disp(),
        )
        .unwrap()
        {
            CommitOutcome::Landed { oid } => oid,
            other => panic!("expected Landed, got {other:?}"),
        };

        // Seed committed boundaries with an existing phase row.
        let tip1 = land(&base, mk("PHASE-01", "aaa", "bbb"));

        // (i) a NEW phase APPENDS at the tail; the existing row is preserved untouched.
        let tip2 = land(&tip1, mk("PHASE-02", "ccc", "ddd"));
        let rows2 = boundaries_at(&coord, &tip2).rows;
        assert_eq!(rows2.len(), 2, "append, never duplicate");
        assert_eq!(rows2[0].phase, "PHASE-01");
        assert_eq!(rows2[0].code_end_oid, "bbb", "existing row preserved");
        assert_eq!(rows2[1].phase, "PHASE-02", "new row lands at the tail");
        assert_eq!(rows2[1].code_end_oid, "ddd");

        // (ii) an EXISTING phase REPLACES its row IN PLACE (oids updated); no duplicate,
        //      order preserved, the sibling row untouched.
        let tip3 = land(&tip2, mk("PHASE-01", "aaa", "zzz"));
        let rows3 = boundaries_at(&coord, &tip3).rows;
        assert_eq!(rows3.len(), 2, "replace in place, never a second PHASE-01");
        assert_eq!(rows3[0].phase, "PHASE-01");
        assert_eq!(
            rows3[0].code_end_oid, "zzz",
            "PHASE-01 row updated in place"
        );
        assert_eq!(rows3[1].phase, "PHASE-02", "sibling preserved, order kept");
        assert_eq!(rows3[1].code_end_oid, "ddd");
    }

    // --- plan_phases ordering (SL-221 PHASE-04, D-B4) ---------------------------------

    /// `plan_phases` chains cuts in ascending PHASE ordinal, independent of the
    /// on-disk ledger row order. The escape hatch (PHASE-03) can tail-append an
    /// out-of-order row onto the boundaries ref, so row order ≠ phase order; the
    /// consumer must normalise. Fixture: rows for PHASE-01..05 with PHASE-03
    /// appended AFTER PHASE-05 (row order 01,02,04,05,03), each a distinct
    /// non-empty code tree. The emitted chain must still be 01←02←03←04←05.
    #[test]
    fn plan_phases_orders_by_phase_ordinal_not_row_order() {
        let (_tmp, primary, _coord, base, _bt) = primary_with_coord(85);
        let trunk_base = base.clone();
        // Five distinct non-empty code tips (each adds a file ⇒ a distinct tree).
        let commit = |file: &str, content: &str, msg: &str| -> String {
            fs::write(primary.join(file), content).unwrap();
            git_run(&primary, &["add", "-A"]);
            git_run(&primary, &["commit", "-q", "-m", msg]);
            git_run(&primary, &["rev-parse", "HEAD^{commit}"])
        };
        let c1 = commit("p1.txt", "one", "c1");
        let c2 = commit("p2.txt", "two", "c2");
        let c3 = commit("p3.txt", "three", "c3");
        let c4 = commit("p4.txt", "four", "c4");
        let c5 = commit("p5.txt", "five", "c5");
        let row = |phase: &str, end: &str| BoundaryRow {
            phase: phase.to_owned(),
            // start ≠ end ⇒ non-empty phase, a cut is emitted.
            code_start_oid: trunk_base.clone(),
            code_end_oid: end.to_owned(),
            provenance: BoundaryProvenance::Funnel,
        };
        // SCRAMBLED: PHASE-03 tail-appended after PHASE-05 (out-of-order landing).
        let scrambled = vec![
            row("PHASE-01", &c1),
            row("PHASE-02", &c2),
            row("PHASE-04", &c4),
            row("PHASE-05", &c5),
            row("PHASE-03", &c3),
        ];
        let boundaries = Boundaries {
            rows: scrambled.clone(),
        };

        let mut planned: Vec<Planned> = Vec::new();
        plan_phases(&primary, "085", &trunk_base, &boundaries, &mut planned)
            .expect("plan_phases succeeds on the scrambled ledger");

        // The commit emitted for a given ordinal (by its `phase/085-NN` ref).
        let commit_for = |nn: &str| -> String {
            let target = format!("{PHASE_REF_PREFIX}085-{nn}");
            planned
                .iter()
                .find(|p| p.target_ref == target)
                .unwrap_or_else(|| panic!("no planned cut for {nn}"))
                .commit_oid
                .clone()
        };
        let parent_of = |oid: &str| git_run(&primary, &["rev-parse", &format!("{oid}^")]);

        // Ascending-ordinal chain: NN parents off (NN-1); 01 parents off trunk_base.
        // Under a row-order walk 03 would parent off 05 and 04 off 02 — this fails.
        assert_eq!(parent_of(&commit_for("01")), trunk_base, "01 ← trunk_base");
        assert_eq!(parent_of(&commit_for("02")), commit_for("01"), "02 ← 01");
        assert_eq!(
            parent_of(&commit_for("03")),
            commit_for("02"),
            "03 ← 02 (not the scrambled predecessor 05)"
        );
        assert_eq!(
            parent_of(&commit_for("04")),
            commit_for("03"),
            "04 ← 03 (not the scrambled predecessor 02)"
        );
        assert_eq!(parent_of(&commit_for("05")), commit_for("04"), "05 ← 04");

        // The on-disk ledger order is NOT mutated — no second normalisation seam.
        assert_eq!(
            boundaries.rows, scrambled,
            "plan_phases must not reorder the input ledger rows"
        );
    }

    // --- VT-2: commit_on_behalf (compose + empty-delta + lost-ref-race) ---------------

    #[test]
    fn commit_on_behalf_happy_lands_one_non_merge_commit() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        let tree = git::tree_with_file(&coord, &base_tree, "added.txt", "hi\n").unwrap();
        let out = commit_on_behalf(
            &coord,
            "refs/heads/dispatch/199",
            &base,
            &tree,
            "m",
            &disp(),
        )
        .unwrap();
        let oid = match out {
            CommitOutcome::Landed { oid } => oid,
            other => panic!("expected Landed, got {other:?}"),
        };
        // The branch advanced base → oid, exactly one parent == base, tree == composed.
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            oid
        );
        let parents = git_run(&coord, &["rev-list", "--parents", "-n", "1", &oid]);
        let cols: Vec<&str> = parents.split_whitespace().collect();
        assert_eq!(cols.len(), 2, "exactly one parent: {parents}");
        assert_eq!(cols[1], base, "C^ == expected_old");
        assert_eq!(
            git_run(&coord, &["rev-parse", &format!("{oid}^{{tree}}")]),
            tree
        );
    }

    #[test]
    fn commit_on_behalf_empty_delta_refuses_and_leaves_the_tip() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        // Composing the parent's own tree ⇒ no change.
        let out = commit_on_behalf(
            &coord,
            "refs/heads/dispatch/199",
            &base,
            &base_tree,
            "m",
            &disp(),
        )
        .unwrap();
        assert_eq!(out, CommitOutcome::Refused(CommitRefusal::EmptyDelta));
        assert_eq!(out.token_is("empty-delta"), true);
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            base
        );
    }

    #[test]
    fn commit_on_behalf_lost_ref_race_refuses_and_leaves_the_tip() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        // Move the ref to a same-tree dangling commit so expected_old=base is now STALE,
        // without touching the index / worktree (status stays identical).
        let other = git_run(
            &coord,
            &["commit-tree", &base_tree, "-p", &base, "-m", "other"],
        );
        git_run(&coord, &["update-ref", "refs/heads/dispatch/199", &other]);
        let tree = git::tree_with_file(&coord, &base_tree, "added.txt", "hi\n").unwrap();
        let out = commit_on_behalf(
            &coord,
            "refs/heads/dispatch/199",
            &base,
            &tree,
            "m",
            &disp(),
        )
        .unwrap();
        assert_eq!(out, CommitOutcome::Refused(CommitRefusal::LostRefRace));
        // The ref is untouched — still at the racing commit, never clobbered.
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            other
        );
    }

    // --- VT-3: provenance (author/committer/message on the produced commit object) ----

    fn commit_idents(dir: &Path, oid: &str) -> (String, String, String, String, String) {
        let raw = git_run(dir, &["log", "-1", "--format=%an%n%ae%n%cn%n%ce%n%s", oid]);
        let mut it = raw.lines();
        (
            it.next().unwrap().to_string(),
            it.next().unwrap().to_string(),
            it.next().unwrap().to_string(),
            it.next().unwrap().to_string(),
            it.next().unwrap().to_string(),
        )
    }

    #[test]
    fn commit_on_behalf_import_mode_preserves_worker_author_and_dispatch_committer() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        let tree = git::tree_with_file(&coord, &base_tree, "added.txt", "hi\n").unwrap();
        let prov = Provenance::Import {
            author: Identity {
                name: "worker-x".to_string(),
                email: "worker-x@doctrine".to_string(),
            },
            committer: Identity {
                name: "dispatch".to_string(),
                email: "dispatch@doctrine".to_string(),
            },
        };
        let msg = funnel_message(199, "PHASE-02");
        let oid =
            match commit_on_behalf(&coord, "refs/heads/dispatch/199", &base, &tree, &msg, &prov)
                .unwrap()
            {
                CommitOutcome::Landed { oid } => oid,
                other => panic!("expected Landed, got {other:?}"),
            };
        let (an, ae, cn, ce, subject) = commit_idents(&coord, &oid);
        assert_eq!(
            (an.as_str(), ae.as_str()),
            ("worker-x", "worker-x@doctrine")
        );
        assert_eq!(
            (cn.as_str(), ce.as_str()),
            ("dispatch", "dispatch@doctrine")
        );
        assert_eq!(subject, msg);
        assert!(
            subject.starts_with(FUNNEL_MARKER),
            "grep-stable marker: {subject}"
        );
        assert!(subject.contains("SL-199") && subject.contains("PHASE-02"));
    }

    #[test]
    fn commit_on_behalf_conclude_mode_sets_author_equal_committer() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        let tree = git::tree_with_file(&coord, &base_tree, "added.txt", "hi\n").unwrap();
        let msg = funnel_message(199, "PHASE-02");
        let oid = match commit_on_behalf(
            &coord,
            "refs/heads/dispatch/199",
            &base,
            &tree,
            &msg,
            &disp(),
        )
        .unwrap()
        {
            CommitOutcome::Landed { oid } => oid,
            other => panic!("expected Landed, got {other:?}"),
        };
        let (an, ae, cn, ce, _s) = commit_idents(&coord, &oid);
        assert_eq!(an, "dispatch");
        assert_eq!(ae, "dispatch@doctrine");
        assert_eq!((an, ae), (cn, ce), "author == committer == dispatch id");
    }

    // --- VT-4: no-residue atomicity ---------------------------------------------------

    fn dirty_coord(coord: &Path) {
        fs::write(coord.join("seed"), "dirtied\n").unwrap(); // unstaged tracked mod
        fs::write(coord.join("staged.txt"), "s\n").unwrap();
        git_run(coord, &["add", "staged.txt"]); // staged add
    }

    #[test]
    fn commit_on_behalf_composes_the_named_tree_not_the_live_index() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        dirty_coord(&coord);
        // The index state (HEAD-independent — `status` is HEAD-relative and the ref moves
        // on a happy land, so it is the wrong probe here).
        let index_before = git_run(&coord, &["ls-files", "--stage"]);
        // Compose from the base tree — the staged/dirty entries live only in the index /
        // worktree, never in `base_tree`.
        let tree = git::tree_with_file(&coord, &base_tree, "committed.txt", "c\n").unwrap();
        let oid = match commit_on_behalf(
            &coord,
            "refs/heads/dispatch/199",
            &base,
            &tree,
            "m",
            &disp(),
        )
        .unwrap()
        {
            CommitOutcome::Landed { oid } => oid,
            other => panic!("expected Landed, got {other:?}"),
        };
        // The commit carries ONLY the named-tree content — not the staged/dirty files.
        let names = git_run(&coord, &["ls-tree", "-r", "--name-only", &oid]);
        let names: Vec<&str> = names.lines().collect();
        assert!(
            names.contains(&"committed.txt"),
            "named tree landed: {names:?}"
        );
        assert!(
            !names.contains(&"staged.txt"),
            "index NOT swept in: {names:?}"
        );
        assert_eq!(
            git_run(&coord, &["cat-file", "-p", &format!("{oid}:seed")]),
            "base",
            "the dirty worktree seed is NOT in the commit"
        );
        // The live index + worktree are byte-unchanged (the primitive never touched them).
        assert_eq!(
            git_run(&coord, &["ls-files", "--stage"]),
            index_before,
            "index untouched"
        );
        assert_eq!(fs::read_to_string(coord.join("seed")).unwrap(), "dirtied\n");
        assert_eq!(fs::read_to_string(coord.join("staged.txt")).unwrap(), "s\n");
    }

    #[test]
    fn commit_on_behalf_fault_leaves_ref_index_and_worktree_byte_unchanged() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        // Move the ref to a same-tree dangling commit so the CAS will fault (expected_old
        // stale) — a fault injected exactly at the ref-update step.
        let other = git_run(
            &coord,
            &["commit-tree", &base_tree, "-p", &base, "-m", "other"],
        );
        git_run(&coord, &["update-ref", "refs/heads/dispatch/199", &other]);
        dirty_coord(&coord);
        let ref_before = git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]);
        let status_before = git_run(&coord, &["status", "--porcelain"]);
        let tree = git::tree_with_file(&coord, &base_tree, "committed.txt", "c\n").unwrap();
        let out = commit_on_behalf(
            &coord,
            "refs/heads/dispatch/199",
            &base,
            &tree,
            "m",
            &disp(),
        )
        .unwrap();
        assert_eq!(out, CommitOutcome::Refused(CommitRefusal::LostRefRace));
        // Ref, index, and worktree are all byte-unchanged.
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            ref_before
        );
        assert_eq!(git_run(&coord, &["status", "--porcelain"]), status_before);
    }

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

    // ---- VT-2: arm-spawn writes the jail declaration beside base (PHASE-04) --------

    #[test]
    fn arm_spawn_writes_jail_declaration_beside_base() {
        let repo = tempfile::tempdir().unwrap();
        init_repo(repo.path());

        run_arm_spawn(
            Some(repo.path().to_path_buf()),
            Some("68250bcd"),
            None,
            vec![PathBuf::from("/nix/store")],
            true, // --no-network
        )
        .unwrap();

        let spawn = repo.path().join(crate::worktree::ARMING_SUBPATH);
        assert!(spawn.join("base").exists(), "base written");
        let decl = spawn.join(crate::worktree::ARMING_JAIL_FILE);
        assert!(decl.exists(), "jail.toml written beside base (F-4 pairing)");
        // The declaration round-trips through the single JailPolicy schema.
        let policy = JailPolicy::from_toml_str(&std::fs::read_to_string(&decl).unwrap()).unwrap();
        assert_eq!(policy.extra_rw, vec![PathBuf::from("/nix/store")]);
        assert!(!policy.network, "--no-network declared");
    }

    #[test]
    fn arm_spawn_default_policy_writes_no_declaration_and_clears_stale() {
        let repo = tempfile::tempdir().unwrap();
        init_repo(repo.path());
        let spawn = repo.path().join(crate::worktree::ARMING_SUBPATH);
        let decl = spawn.join(crate::worktree::ARMING_JAIL_FILE);

        // First arm declares a policy ⇒ jail.toml present.
        run_arm_spawn(
            Some(repo.path().to_path_buf()),
            Some("68250bcd"),
            None,
            vec![PathBuf::from("/nix/store")],
            false,
        )
        .unwrap();
        assert!(decl.exists(), "declaration present after a non-Default arm");

        // Re-arm with the Default floor (no flags) ⇒ no declaration; the stale one is
        // cleared so it cannot pair with the fresh base (F-4 hygiene).
        run_arm_spawn(
            Some(repo.path().to_path_buf()),
            Some("abcd1234"),
            None,
            vec![],
            false,
        )
        .unwrap();
        assert!(spawn.join("base").exists(), "base rewritten");
        assert!(
            !decl.exists(),
            "stale jail.toml cleared on a Default re-arm"
        );
    }

    // ---- A1 / VT-2: an omitted --base defaults to the coord-root HEAD ---------------

    #[test]
    fn arm_spawn_defaults_base_to_head_when_omitted() {
        let repo = tempfile::tempdir().unwrap();
        init_repo(repo.path());
        let spawn = repo.path().join(crate::worktree::ARMING_SUBPATH);

        // base = None ⇒ the arming `base` file equals the repo's `git rev-parse HEAD`.
        run_arm_spawn(Some(repo.path().to_path_buf()), None, None, vec![], false).unwrap();
        let head = git(repo.path(), &["rev-parse", "HEAD"]);
        let written = std::fs::read_to_string(spawn.join("base")).unwrap();
        assert_eq!(
            written.trim(),
            head,
            "defaulted base is the coord-root HEAD"
        );

        // base = Some(<explicit sha>) ⇒ that sha is written unchanged.
        run_arm_spawn(
            Some(repo.path().to_path_buf()),
            Some("abcd1234"),
            None,
            vec![],
            false,
        )
        .unwrap();
        let explicit = std::fs::read_to_string(spawn.join("base")).unwrap();
        assert_eq!(
            explicit.trim(),
            "abcd1234",
            "explicit base honored unchanged"
        );
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
            &format!("schema = \"{SCHEMA_PLAN_OVERVIEW}\"\nversion = 1\nslice = \"SL-085\"\n"),
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
            &format!(
                "schema = \"{SCHEMA_PLAN_OVERVIEW}\"\nversion = 1\nslice = \"SL-085\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n"
            ),
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
            &format!(
                "schema = \"{SCHEMA_PLAN_OVERVIEW}\"\nversion = 1\nslice = \"SL-085\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n"
            ),
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
            &format!(
                "schema = \"{SCHEMA_PLAN_OVERVIEW}\"\nversion = 1\nslice = \"SL-085\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n"
            ),
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
            format!("schema = \"{SCHEMA_PLAN_OVERVIEW}\"\nversion = 1\nslice = \"SL-085\"\n");
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
                &format!("{DISPATCH_REF_PREFIX}{slice:03}"),
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
                &format!("{REVIEW_REF_PREFIX}{slice:03}"),
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

    // --- SL-206 PHASE-02: phase_projection + ReceiptStatus ---------------------

    /// VT-1 (SL-206 PHASE-02): the `phase_projection` branch table — each tier
    /// combination derives the right `ReceiptStatus`. Boundary-present ⇒
    /// Completed; sheet-"completed" with no boundary ⇒ ConcludeIncomplete;
    /// blocked ⇒ Blocked; an unreadable sheet ⇒ Unknown (fail-loud, never
    /// NotStarted); in_progress ⇒ InProgress; no sheet ⇒ NotStarted. Hermetic: a
    /// seeded phases dir + a committed set parsed from a real boundaries body.
    #[test]
    fn phase_projection_derives_receipt_status_per_tier() {
        let tmp = tempfile::tempdir().unwrap();
        let state_dir = tmp.path().join("phases");
        std::fs::create_dir_all(&state_dir).unwrap();
        let write = |stem: &str, status: &str| {
            std::fs::write(
                state_dir.join(format!("{stem}.toml")),
                format!("status = \"{status}\"\n"),
            )
            .unwrap();
        };
        write("phase-01", "completed"); // + committed boundary → Completed
        write("phase-02", "completed"); // no boundary          → ConcludeIncomplete
        write("phase-03", "blocked"); //                        → Blocked
        // phase-04: a DIRECTORY at the `.toml` path forces read_phase_status to
        // return Err (an IO error that is not NotFound) → Unknown.
        std::fs::create_dir_all(state_dir.join("phase-04.toml")).unwrap();
        write("phase-05", "in_progress"); //                    → InProgress
        // phase-06: no sheet at all →                             NotStarted

        let plan = crate::plan::Plan::parse(&plan_body(&[
            ("PHASE-01", "done-bounded"),
            ("PHASE-02", "conclude-incomplete"),
            ("PHASE-03", "blocked"),
            ("PHASE-04", "malformed"),
            ("PHASE-05", "in-progress"),
            ("PHASE-06", "not-started"),
        ]))
        .unwrap();

        // Committed ledger backs only PHASE-01 — parsed from a real boundaries
        // body, then reduced to the phase-id set exactly as run_status does.
        let boundaries = Boundaries::parse(
            "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"s\"\ncode_end_oid = \"e\"\n",
        )
        .unwrap();
        let committed: BTreeSet<&str> = boundaries.rows.iter().map(|r| r.phase.as_str()).collect();

        let projected = phase_projection(&plan, &state_dir, &committed);
        let got: Vec<(&str, ReceiptStatus)> = projected
            .iter()
            .map(|r| (r.id.as_str(), r.status))
            .collect();
        assert_eq!(
            got,
            vec![
                ("PHASE-01", ReceiptStatus::Completed),
                ("PHASE-02", ReceiptStatus::ConcludeIncomplete),
                ("PHASE-03", ReceiptStatus::Blocked),
                ("PHASE-04", ReceiptStatus::Unknown),
                ("PHASE-05", ReceiptStatus::InProgress),
                ("PHASE-06", ReceiptStatus::NotStarted),
            ]
        );
    }

    /// VT-2 (SL-206 PHASE-02): `run_status` delegates its per-phase rows to
    /// `phase_projection`, and the legacy status string is unchanged. On a full
    /// fixture (dispatch ref + committed boundary for PHASE-01 + sheets) the
    /// command succeeds, and the same ingredients fed through
    /// `phase_projection` → `receipt_status_legacy_str` reproduce today's
    /// `(id, status)` rows — Completed and InProgress both round-trip to their
    /// legacy strings, proving the delegation path.
    #[test]
    fn run_status_delegates_phase_rows_to_phase_projection() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[("PHASE-01", "setup"), ("PHASE-02", "build")]),
        );
        // Commit a boundaries ledger (PHASE-01 only) onto the tree the dispatch
        // ref will point at, so run_status reads it from the object db.
        let bdir = src.path().join(".doctrine/dispatch/085");
        std::fs::create_dir_all(&bdir).unwrap();
        std::fs::write(
            bdir.join("boundaries.toml"),
            "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"s\"\ncode_end_oid = \"e\"\n",
        )
        .unwrap();
        git(src.path(), &["add", "-A"]);
        git(src.path(), &["commit", "-q", "-m", "seed boundaries"]);
        create_dispatch_ref(src.path(), 85);
        seed_phase_tracking(src.path(), 85, 1, "completed");
        seed_phase_tracking(src.path(), 85, 2, "in_progress");

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");

        // Reconstruct run_status' ingredients and assert the delegated projection
        // reproduces the legacy rows (behaviour-preservation, EX-4).
        let dispatch_ref = format!("{DISPATCH_REF_PREFIX}085");
        let boundaries =
            read_ledger::<Boundaries>(src.path(), &dispatch_ref, "085", "boundaries.toml").unwrap();
        let committed: BTreeSet<&str> = boundaries.rows.iter().map(|r| r.phase.as_str()).collect();
        let plan = crate::slice::read_plan(&src.path().join(".doctrine/slice"), 85).unwrap();
        let state_dir = crate::state::phases_dir(src.path(), 85);

        let projected = phase_projection(&plan, &state_dir, &committed);
        assert_eq!(
            projected.iter().map(|r| r.status).collect::<Vec<_>>(),
            vec![ReceiptStatus::Completed, ReceiptStatus::InProgress],
            "PHASE-01 is boundary-backed Completed; PHASE-02 InProgress",
        );
        let legacy: Vec<(String, String)> = projected
            .iter()
            .map(|r| {
                (
                    r.id.clone(),
                    receipt_status_legacy_str(r.status).to_string(),
                )
            })
            .collect();
        assert_eq!(
            legacy,
            vec![
                ("PHASE-01".to_string(), "completed".to_string()),
                ("PHASE-02".to_string(), "in_progress".to_string()),
            ]
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

    fn reg_row(phase: &str, provenance: BoundaryProvenance) -> BoundaryRow {
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
            reg_row("PHASE-01", BoundaryProvenance::Funnel),
            reg_row("PHASE-02", BoundaryProvenance::Funnel),
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
            reg_row("PHASE-01", BoundaryProvenance::Funnel),
            reg_row("PHASE-02", BoundaryProvenance::Funnel),
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
            reg_row("PHASE-01", BoundaryProvenance::Unknown),
            reg_row("PHASE-02", BoundaryProvenance::Solo),
            reg_row("PHASE-03", BoundaryProvenance::Manual),
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

    // --- SL-165 PHASE-02: trace_candidate_provenance refuse matrix (INV-2..5) --
    //
    // The git-dependent INV-6 lineage binding (is_ancestor of the live tip onto
    // the recorded merge) is exercised end-to-end in
    // `tests/e2e_dispatch_candidate.rs::close_target_from_moved_candidate_ref_refuses`.
    // The structural refuse branches below are pure over (Candidates, Journal):
    // hand-crafting an ambiguous / cyclic / non-evidence ledger through the CLI
    // is impractical, so they are unit-covered against the design's bail classes.

    /// Build a candidate row with the fields the trace reads; the rest are inert.
    fn cand_row(
        target_ref: &str,
        source_ref: &str,
        role: CandidateRole,
        kind: CandidateKind,
        status: CandidateStatus,
    ) -> CandidateRow {
        CandidateRow {
            id: format!("cand-{target_ref}"),
            label: "l".into(),
            kind,
            role,
            payload: CandidatePayload::ImplBundle,
            target_ref: target_ref.into(),
            source_ref: source_ref.into(),
            source_oid: "0".repeat(40),
            base_ref: "refs/heads/main".into(),
            base_oid: "0".repeat(40),
            merge_oid: "0".repeat(40),
            status,
            supersedes: String::new(),
            reason: String::new(),
            created_by: "test".into(),
            created_at: "2026-01-01".into(),
            ingested_at: String::new(),
            merge_provenance: crate::ledger::MergeProvenance::Doctrine,
        }
    }

    fn jrow(target_ref: &str, status: LedgerStatus) -> JournalRow {
        JournalRow {
            source_oid: "0".repeat(40),
            target_ref: target_ref.into(),
            expected_old_oid: "0".repeat(40),
            planned_new_oid: "0".repeat(40),
            applied_new_oid: String::new(),
            status,
        }
    }

    /// A clean review_surface candidate sourced from a Verified `review/200`.
    fn audit_surface_row() -> CandidateRow {
        cand_row(
            "refs/heads/candidate/200/review-001",
            "refs/heads/review/200",
            CandidateRole::ReviewSurface,
            CandidateKind::Audit,
            CandidateStatus::Created,
        )
    }

    fn verified_review_journal() -> Journal {
        Journal {
            rows: vec![jrow("refs/heads/review/200", LedgerStatus::Verified)],
            ..Default::default()
        }
    }

    fn trace_err(
        candidates: &Candidates,
        journal: &Journal,
        ref_name: &str,
        budget: u32,
    ) -> String {
        let err = trace_candidate_provenance(candidates, journal, "200", ref_name, budget)
            .expect_err("trace should refuse");
        format!("{err}")
    }

    // SL-211 EX-2 / VT-2: candidate workflow active (≥1 recorded row) but NO
    // close_target admission ⇒ resolve_trunk_payload REFUSES (no raw-evidence
    // fallback). Pure over (Candidates, Journal): the candidate branch errors
    // before any git read, so the root path and journal are inert.
    #[test]
    fn resolve_trunk_payload_candidate_without_admission_refuses() {
        let candidates = Candidates {
            rows: vec![audit_surface_row()],
            ..Default::default()
        };
        let err = resolve_trunk_payload(
            std::path::Path::new("/nonexistent"),
            "200",
            &Journal::default(),
            &candidates,
        )
        .expect_err("candidate-active without a close_target admission must refuse");
        assert!(
            format!("{err}").contains("close_target"),
            "refusal names the missing close_target admission: {err}"
        );
    }

    // INV-1: a clean audit review_surface tracing to Verified journaled evidence
    // is accepted — the recursion's terminal base case.
    #[test]
    fn trace_accepts_audit_surface_to_verified_root() {
        let candidates = Candidates {
            rows: vec![audit_surface_row()],
            ..Default::default()
        };
        let row = trace_candidate_provenance(
            &candidates,
            &verified_review_journal(),
            "200",
            "refs/heads/candidate/200/review-001",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        )
        .expect("clean audit surface should trace to the verified root");
        assert_eq!(row.target_ref, "refs/heads/candidate/200/review-001");
    }

    // INV-4: an exhausted budget refuses (the over-deep / cyclic guard).
    #[test]
    fn trace_over_budget_refuses() {
        let candidates = Candidates {
            rows: vec![audit_surface_row()],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &verified_review_journal(),
            "refs/heads/candidate/200/review-001",
            0,
        );
        assert!(err.contains("too deep or cyclic"), "got: {err}");
    }

    // INV-4: a cyclic chain (A→B→A) exhausts the budget rather than looping.
    #[test]
    fn trace_cyclic_chain_refuses() {
        let a = cand_row(
            "refs/heads/candidate/200/a",
            "refs/heads/candidate/200/b",
            CandidateRole::CloseTarget,
            CandidateKind::Audit,
            CandidateStatus::Created,
        );
        let b = cand_row(
            "refs/heads/candidate/200/b",
            "refs/heads/candidate/200/a",
            CandidateRole::CloseTarget,
            CandidateKind::Audit,
            CandidateStatus::Created,
        );
        let candidates = Candidates {
            rows: vec![a, b],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &Journal::default(),
            "refs/heads/candidate/200/a",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(err.contains("too deep or cyclic"), "got: {err}");
    }

    // INV-5: two rows sharing a target_ref are fail-closed, never first-match.
    #[test]
    fn trace_ambiguous_row_refuses() {
        let candidates = Candidates {
            rows: vec![audit_surface_row(), audit_surface_row()],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &verified_review_journal(),
            "refs/heads/candidate/200/review-001",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(err.contains("ambiguous candidate row"), "got: {err}");
    }

    // A source ref naming no recorded row is refused.
    #[test]
    fn trace_missing_row_refuses() {
        let candidates = Candidates::default();
        let err = trace_err(
            &candidates,
            &Journal::default(),
            "refs/heads/candidate/200/ghost",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(err.contains("no recorded candidate row"), "got: {err}");
    }

    // INV-3: only a `Created` source candidate qualifies — a `Conflicted`
    // (parked-at-base) candidate is refused as not clean.
    #[test]
    fn trace_conflicted_status_refuses() {
        let mut row = audit_surface_row();
        row.status = CandidateStatus::Conflicted;
        let candidates = Candidates {
            rows: vec![row],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &verified_review_journal(),
            "refs/heads/candidate/200/review-001",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(err.contains("not clean"), "got: {err}");
    }

    // INV-2: an `experiment`-kind source is refused even when its role would
    // otherwise pass — only `audit` content may source a close_target.
    #[test]
    fn trace_experiment_kind_refuses() {
        let mut row = audit_surface_row();
        row.kind = CandidateKind::Experiment;
        let candidates = Candidates {
            rows: vec![row],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &verified_review_journal(),
            "refs/heads/candidate/200/review-001",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(
            err.contains("kind=Experiment") || err.contains("only an audit review_surface"),
            "got: {err}"
        );
    }

    // INV-2: a `scratch`-role source is refused (mirrors the e2e scratch case at
    // the pure layer, asserting the bail class).
    #[test]
    fn trace_scratch_role_refuses() {
        let mut row = audit_surface_row();
        row.role = CandidateRole::Scratch;
        let candidates = Candidates {
            rows: vec![row],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &verified_review_journal(),
            "refs/heads/candidate/200/review-001",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(
            err.contains("role=Scratch") || err.contains("only an audit review_surface"),
            "got: {err}"
        );
    }

    // The recorded chain must terminate at journaled evidence: a hop to a ref
    // that is neither a candidate nor journaled evidence is refused.
    #[test]
    fn trace_non_evidence_hop_refuses() {
        let row = cand_row(
            "refs/heads/candidate/200/review-001",
            "refs/heads/feature/random",
            CandidateRole::ReviewSurface,
            CandidateKind::Audit,
            CandidateStatus::Created,
        );
        let candidates = Candidates {
            rows: vec![row],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &Journal::default(),
            "refs/heads/candidate/200/review-001",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(err.contains("non-evidence"), "got: {err}");
    }

    // F3: the journaled base case runs the FULL existing gate — an UNVERIFIED
    // journal row at the chain root is refused, not silently accepted.
    #[test]
    fn trace_unverified_journal_root_refuses() {
        let candidates = Candidates {
            rows: vec![audit_surface_row()],
            ..Default::default()
        };
        let journal = Journal {
            rows: vec![jrow("refs/heads/review/200", LedgerStatus::Pending)],
            ..Default::default()
        };
        let err = trace_err(
            &candidates,
            &journal,
            "refs/heads/candidate/200/review-001",
            CANDIDATE_PROVENANCE_DEPTH_BUDGET,
        );
        assert!(err.contains("not verified"), "got: {err}");
    }

    // The journaled/candidate classifiers are the single source of truth for the
    // base-case vs recursion-step split (design §5.2).
    #[test]
    fn ref_classifiers_agree() {
        assert!(is_journaled_evidence_ref("refs/heads/review/200", "200"));
        assert!(is_journaled_evidence_ref("refs/heads/phase/200-03", "200"));
        assert!(!is_journaled_evidence_ref("refs/heads/phase/200-xx", "200"));
        assert!(!is_journaled_evidence_ref(
            "refs/heads/candidate/200/x",
            "200"
        ));
        assert!(is_candidate_ref("refs/heads/candidate/200/review-001"));
        assert!(!is_candidate_ref("refs/heads/review/200"));
    }

    // --- g1: guard_not_on_integration_ref (PHASE-04) -----------------------

    /// Build a posture config with the given authoring branch (deliver_to keeps
    /// its `refs/heads/main` default — the buffer in these fixtures).
    fn posture_cfg(authoring: Option<&str>) -> crate::dispatch_config::DispatchConfig {
        crate::dispatch_config::DispatchConfig {
            authoring_branch: authoring.map(str::to_owned),
            ..Default::default()
        }
    }

    #[test]
    fn integrate_refused_when_head_on_buffer() {
        // VT-1: posture on (authoring=edge), HEAD on the buffer `main` ⇒ refuse,
        // naming the buffer ref and the fetch-not-checkout recovery.
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path()); // HEAD on `main` (== deliver_to short name)
        let cfg = posture_cfg(Some("refs/heads/edge"));
        let err = guard_not_on_integration_ref(src.path(), &cfg).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains(corpus_guard::REFUSE_ON_TRUNK),
            "names the g1 token: {msg}"
        );
        assert!(
            msg.contains("refs/heads/main"),
            "names the buffer ref: {msg}"
        );
        assert!(
            msg.contains("git fetch . refs/heads/edge:main")
                && msg.contains("never `checkout main`"),
            "names the fetch-not-checkout recovery: {msg}"
        );
    }

    #[test]
    fn integrate_allowed_on_authoring_branch() {
        // VT-2: posture on, HEAD on the authoring branch `edge` ⇒ the safe leg, Ok.
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        git(src.path(), &["checkout", "-q", "-b", "edge"]);
        let cfg = posture_cfg(Some("refs/heads/edge"));
        assert!(guard_not_on_integration_ref(src.path(), &cfg).is_ok());
    }

    #[test]
    fn g1_inert_when_posture_unset() {
        // VT-2: single-branch parity (INV-2) — authoring-branch unset ⇒ inert even
        // with HEAD on the buffer `main`.
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let cfg = posture_cfg(None);
        assert!(guard_not_on_integration_ref(src.path(), &cfg).is_ok());
    }

    #[test]
    fn g1_guards_only_the_integrate_verb_entry() {
        // VA-1 verb-set audit (design F-4 / OQ-3): g1 has exactly ONE call site,
        // at the `run_integrate` verb entry — the sole landing for the
        // `--trunk`/`--edge` and candidate-active legs that advance an integration
        // ref. `candidate create` / `candidate admit` advance no integration ref
        // and MUST stay unguarded. This pins the enumeration so no future edit can
        // silently widen or narrow it without tripping the test.
        // Scope to PRODUCTION source only — exclude this test module, which names
        // the symbol in its own assertions.
        let this = include_str!("dispatch.rs");
        let prod = this
            .split("#[cfg(test)]")
            .next()
            .expect("production source before the test module");
        let call_sites = prod
            .lines()
            .filter(|l| {
                l.contains("guard_not_on_integration_ref(")
                    && !l.contains("fn guard_not_on_integration_ref")
                    && !l.trim_start().starts_with("//")
                    && !l.trim_start().starts_with("///")
            })
            .count();
        assert_eq!(
            call_sites, 1,
            "exactly one g1 call site (the integrate verb)"
        );
        // The one call site is in run_integrate, right after the config load.
        assert!(
            prod.contains(
                "let cfg = crate::dtoml::load_doctrine_toml(&root)?.dispatch;\n    guard_not_on_integration_ref(&root, &cfg)?;"
            ),
            "the g1 call site is run_integrate's verb entry"
        );
        // And neither candidate path references it (create/admit are excluded).
        for fn_name in ["fn candidate_create", "fn run_candidate_admit"] {
            let body_start = prod.find(fn_name).expect("candidate fn present");
            let after = &prod[body_start..];
            // Scope to a generous window covering the fn body.
            let window = &after[..after.len().min(6000)];
            assert!(
                !window.contains("guard_not_on_integration_ref("),
                "{fn_name} must not call g1 (advances no integration ref)"
            );
        }
    }

    // ---- SL-211 PHASE-02: recorder planner + R1 earned check -----------------

    /// VT-1: `recorded_row` row shape — a TERMINAL Verified row whose four oid
    /// fields ALL equal the payload (`expected_old == planned == applied == source
    /// == payload`), status `Verified`. This pins the replay-safety contract (D2):
    /// `expected_old` is the payload, NOT the trunk tip.
    #[test]
    fn recorded_row_is_terminal_verified_all_oids_payload() {
        let payload = "a".repeat(40);
        let row = recorded_row("refs/heads/main", payload.clone());
        assert_eq!(row.target_ref, "refs/heads/main");
        assert_eq!(row.source_oid, payload, "source == payload");
        assert_eq!(
            row.expected_old_oid, payload,
            "expected_old == payload (= planned, NOT tip)"
        );
        assert_eq!(row.planned_new_oid, payload, "planned == payload");
        assert_eq!(
            row.applied_new_oid, payload,
            "applied == payload (terminal, not empty)"
        );
        assert_eq!(row.status, LedgerStatus::Verified, "statement of fact");
    }

    /// VT-2: earned PASS — the resolved trunk payload is an ancestor of the trunk
    /// tip ⇒ `plan_recorded_trunk_row` returns the earned terminal row with
    /// `planned_new_oid == payload`. Legacy (candidate-free) path: the payload is
    /// the phase-chain tip `phase/211-01`, and the trunk `main` has advanced past
    /// it (payload strictly an ancestor).
    #[test]
    fn plan_recorded_trunk_row_returns_earned_row_when_payload_on_trunk() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path()); // c0 on main
        let payload = commit_file(src.path(), "p.txt", "phase work\n", "phase 01 code");
        git(src.path(), &["branch", "phase/211-01", &payload]);
        // Trunk advances PAST the payload ⇒ payload is a strict ancestor of main.
        commit_file(
            src.path(),
            "t.txt",
            "trunk ahead\n",
            "trunk advances past payload",
        );

        let journal = Journal {
            rows: vec![jrow("refs/heads/phase/211-01", LedgerStatus::Verified)],
            ..Default::default()
        };
        let candidates = Candidates::default();
        let row =
            plan_recorded_trunk_row(src.path(), "211", &journal, &candidates, "refs/heads/main")
                .expect("payload on trunk ⇒ earned row");
        assert_eq!(
            row.planned_new_oid, payload,
            "the earned row records the payload"
        );
        assert_eq!(row.status, LedgerStatus::Verified);
    }

    /// VT-3: R1 NEGATIVE — the payload is NOT an ancestor of the trunk tip (a
    /// divergent phase branch never landed) ⇒ `plan_recorded_trunk_row` REFUSES
    /// with an error naming `is not an ancestor`; no row is produced.
    #[test]
    fn plan_recorded_trunk_row_refuses_when_payload_not_on_trunk() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path()); // c0 on main
        // Phase branch diverges from c0 with its own commit — never merged to main.
        git(src.path(), &["checkout", "-q", "-b", "phase/211-01"]);
        let payload = commit_file(src.path(), "p.txt", "unlanded phase\n", "phase 01 code");
        // Advance main divergently so the payload is NOT its ancestor.
        git(src.path(), &["checkout", "-q", "main"]);
        commit_file(
            src.path(),
            "t.txt",
            "trunk only\n",
            "trunk advances without the phase",
        );

        let journal = Journal {
            rows: vec![jrow("refs/heads/phase/211-01", LedgerStatus::Verified)],
            ..Default::default()
        };
        let candidates = Candidates::default();
        let err =
            plan_recorded_trunk_row(src.path(), "211", &journal, &candidates, "refs/heads/main")
                .expect_err("unlanded payload must refuse");
        let msg = format!("{err}");
        assert!(
            msg.contains("is not an ancestor"),
            "R1 refusal names the earned-check failure; got: {msg}"
        );
        assert!(
            msg.contains(&payload),
            "refusal names the payload oid; got: {msg}"
        );
    }

    // ---- SL-211 PHASE-03: run_record_integration handler guards (VT-2) --------

    /// Seed `refs/heads/dispatch/{slice:03}` at a commit whose tree carries
    /// `.doctrine/dispatch/{slice:03}/journal.toml` == `journal` (the handler
    /// tree-reads it from the coordination tip). Returns the seeded dispatch tip.
    fn seed_dispatch_journal(dir: &Path, slice: u32, journal: &Journal) -> String {
        let rel = format!(".doctrine/dispatch/{slice:03}/journal.toml");
        let full = dir.join(&rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, journal.to_toml().unwrap()).unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "seed journal"]);
        let head = git(dir, &["rev-parse", "HEAD"]);
        git(
            dir,
            &[
                "update-ref",
                &format!("{DISPATCH_REF_PREFIX}{slice:03}"),
                &head,
            ],
        );
        head
    }

    /// Re-read the committed journal from the (possibly advanced) dispatch tip.
    fn read_dispatch_journal(dir: &Path, slice: u32) -> Journal {
        let coord = format!("{DISPATCH_REF_PREFIX}{slice:03}");
        read_ledger::<Journal>(dir, &coord, &format!("{slice:03}"), "journal.toml").unwrap()
    }

    /// F-4: an explicit `--trunk` ≠ `[dispatch] deliver_to` (default
    /// `refs/heads/main`) is REFUSED before any ref work — a row targeting a ref the
    /// close gate never reads would leave `done` permanently blocked.
    #[test]
    fn run_record_integration_refuses_trunk_deliver_to_mismatch() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let err = run_record_integration(
            Some(src.path().to_path_buf()),
            211,
            Some("refs/heads/other"),
        )
        .expect_err("mismatched --trunk must refuse");
        let msg = format!("{err}");
        assert!(msg.contains("does not match"), "names the mismatch: {msg}");
        assert!(
            msg.contains("refs/heads/other"),
            "names the given ref: {msg}"
        );
        assert!(msg.contains("refs/heads/main"), "names deliver_to: {msg}");
    }

    /// F-2/D7: a pre-existing *Verified* trunk row is a real prior integration ⇒
    /// idempotent no-op: `Ok`, and `dispatch/<slice>` is NOT advanced (no duplicate
    /// row committed).
    #[test]
    fn run_record_integration_verified_row_is_idempotent_noop() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        let journal = Journal {
            rows: vec![jrow("refs/heads/main", LedgerStatus::Verified)],
            ..Default::default()
        };
        let tip_before = seed_dispatch_journal(src.path(), 211, &journal);
        run_record_integration(Some(src.path().to_path_buf()), 211, Some("refs/heads/main"))
            .expect("a Verified row ⇒ no-op Ok");
        let tip_after = git(
            src.path(),
            &["rev-parse", &format!("{DISPATCH_REF_PREFIX}211")],
        );
        assert_eq!(
            tip_before, tip_after,
            "no-op leaves dispatch/<slice> unadvanced"
        );
    }

    /// F-2/D7: a pre-existing non-applied (Failed) trunk row carried zero external
    /// effect ⇒ REPLACED in place by the earned Verified row (the recovery), not
    /// duplicated, not refused. The payload (phase-chain tip) is a strict ancestor
    /// of the trunk tip, so the earned check passes.
    #[test]
    fn run_record_integration_replaces_failed_trunk_row() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        // Landed payload: a commit that becomes a strict ancestor of main.
        let payload = commit_file(src.path(), "p.txt", "phase work\n", "phase 01 code");
        git(src.path(), &["branch", "phase/211-01", &payload]);
        // Advance main PAST the payload so it is a strict ancestor.
        commit_file(
            src.path(),
            "t.txt",
            "trunk ahead\n",
            "advance main past payload",
        );

        let journal = Journal {
            rows: vec![
                jrow("refs/heads/phase/211-01", LedgerStatus::Verified),
                jrow("refs/heads/main", LedgerStatus::Failed),
            ],
            ..Default::default()
        };
        seed_dispatch_journal(src.path(), 211, &journal);

        run_record_integration(Some(src.path().to_path_buf()), 211, Some("refs/heads/main"))
            .expect("a Failed row is replaced with the earned Verified row");

        let reloaded = read_dispatch_journal(src.path(), 211);
        let main_rows: Vec<_> = reloaded
            .rows
            .iter()
            .filter(|r| r.target_ref == "refs/heads/main")
            .collect();
        assert_eq!(main_rows.len(), 1, "replaced in place, not duplicated");
        assert_eq!(
            main_rows[0].status,
            LedgerStatus::Verified,
            "the earned row is Verified"
        );
        assert_eq!(
            main_rows[0].planned_new_oid, payload,
            "the earned row records the payload"
        );
    }

    // ==================================================================================
    // SL-212 PHASE-02 — pure ingest-provenance validator (no git, byte paths, D1/D9).
    // ==================================================================================

    /// Build a byte-path set from literal path bytes.
    fn pset(paths: &[&[u8]]) -> BTreeSet<Vec<u8>> {
        paths.iter().map(|p| p.to_vec()).collect()
    }

    /// VT-1 — the ordered-parent gate covers reversed / single / ≠2 in one check.
    #[test]
    fn ingest_validate_parent_gate() {
        let base = "aaa".to_owned();
        let source = "bbb".to_owned();
        let d = pset(&[]); // D ⊆ C trivially — isolate the parent gate
        let c = pset(&[b"x"]);

        // Ordered [base, source] passes the parent gate.
        let ordered = vec![base.clone(), source.clone()];
        assert!(validate_ingest_provenance(&ordered, &base, &source, &d, &c, &[]).is_ok());

        // Reversed rejects.
        let reversed = vec![source.clone(), base.clone()];
        assert!(validate_ingest_provenance(&reversed, &base, &source, &d, &c, &[]).is_err());

        // Single parent rejects.
        let single = vec![base.clone()];
        assert!(validate_ingest_provenance(&single, &base, &source, &d, &c, &[]).is_err());

        // Three parents reject.
        let three = vec![base.clone(), source.clone(), "ccc".to_owned()];
        assert!(validate_ingest_provenance(&three, &base, &source, &d, &c, &[]).is_err());
    }

    /// VT-1 — IngestRequest carries slice / label / ingested_at.
    #[test]
    fn ingest_request_carries_fields() {
        let req = IngestRequest {
            slice: 212,
            label: "cand-x".to_owned(),
            ingested_at: "2026-07-22T00:00:00Z".to_owned(),
        };
        assert_eq!(req.slice, 212);
        assert_eq!(req.label, "cand-x");
        assert!(!req.ingested_at.is_empty());
    }

    /// VT-2 — D ⊄ C (an arbitrary-tree edit) rejects and the reason names the
    /// offending path; D ⊆ C passes.
    #[test]
    fn ingest_validate_subset_gate_names_offending_path() {
        let base = "aaa".to_owned();
        let source = "bbb".to_owned();
        let parents = vec![base.clone(), source.clone()];
        let conflict_paths = pset(&[b"conflict.rs"]);

        // D touches a non-conflict path — "never an arbitrary tree".
        let diff_from_mechanical = pset(&[b"conflict.rs", b"arbitrary.rs"]);
        let rej = validate_ingest_provenance(
            &parents,
            &base,
            &source,
            &diff_from_mechanical,
            &conflict_paths,
            &[],
        )
        .unwrap_err();
        assert!(
            rej.reason.contains("arbitrary.rs"),
            "reason names the offending path: {}",
            rej.reason
        );

        // D ⊆ C accepts.
        let ok = pset(&[b"conflict.rs"]);
        assert!(
            validate_ingest_provenance(&parents, &base, &source, &ok, &conflict_paths, &[]).is_ok()
        );
    }

    /// VT-2 (F8/D9) — the subset compare is byte-wise; a non-UTF-8 offending path
    /// still rejects, and the lossy render in the reason does not panic.
    #[test]
    fn ingest_validate_subset_gate_is_byte_wise() {
        let base = "aaa".to_owned();
        let source = "bbb".to_owned();
        let parents = vec![base.clone(), source.clone()];
        let conflict_paths = pset(&[b"\xff\xfe.bin"]);
        // Same first byte, different tail — a lossy UTF-8 compare could conflate these.
        let diff_from_mechanical = pset(&[b"\xff\xfe.bin", b"\xff\x01arb"]);
        let rej = validate_ingest_provenance(
            &parents,
            &base,
            &source,
            &diff_from_mechanical,
            &conflict_paths,
            &[],
        )
        .unwrap_err();
        assert!(!rej.reason.is_empty());
    }

    /// VT-3 — a surviving marker rejects (advisory); the fully-resolved happy case
    /// accepts.
    #[test]
    fn ingest_validate_marker_gate_advisory() {
        let base = "aaa".to_owned();
        let source = "bbb".to_owned();
        let parents = vec![base.clone(), source.clone()];
        let conflict_paths = pset(&[b"conflict.rs"]);
        let diff_from_mechanical = pset(&[b"conflict.rs"]);

        // A marker still present at a conflict path rejects (advisory gate).
        let marker_paths = vec![b"conflict.rs".to_vec()];
        let rej = validate_ingest_provenance(
            &parents,
            &base,
            &source,
            &diff_from_mechanical,
            &conflict_paths,
            &marker_paths,
        )
        .unwrap_err();
        assert!(!rej.reason.is_empty());

        // No markers, ordered parents, D ⊆ C — the genuine 3-way accepts.
        assert!(
            validate_ingest_provenance(
                &parents,
                &base,
                &source,
                &diff_from_mechanical,
                &conflict_paths,
                &[]
            )
            .is_ok()
        );
    }

    /// A minimal in-memory candidate row for the pure status-surface tests —
    /// only the fields the next-command guidance reads (`label`, `status`,
    /// `merge_oid`) are meaningful; the rest are inert placeholders.
    fn guidance_row(label: &str, status: CandidateStatus, merge_oid: &str) -> CandidateRow {
        CandidateRow {
            id: format!("cand-212-{label}"),
            label: label.to_owned(),
            kind: CandidateKind::Audit,
            role: CandidateRole::CloseTarget,
            payload: CandidatePayload::Code,
            target_ref: format!("refs/heads/candidate/212/{label}"),
            source_ref: "refs/heads/source".to_owned(),
            source_oid: "src".to_owned(),
            base_ref: "refs/heads/main".to_owned(),
            base_oid: "base".to_owned(),
            merge_oid: merge_oid.to_owned(),
            status,
            supersedes: String::new(),
            reason: String::new(),
            created_by: "test".to_owned(),
            created_at: "2026-01-01".to_owned(),
            ingested_at: String::new(),
            merge_provenance: crate::ledger::MergeProvenance::Doctrine,
        }
    }

    /// VT-2 — the status surface prescribes `candidate ingest` for a conflicted,
    /// un-ingested row (parked at base, empty `merge_oid`), naming that row's
    /// label, and does NOT prescribe it for a cleanly-created row.
    #[test]
    fn candidate_status_prescribes_ingest_for_conflicted_row() {
        let mut ledger = Candidates::default();
        ledger.rows.push(guidance_row(
            "review-001",
            CandidateStatus::Created,
            "mergeoid",
        ));
        ledger
            .rows
            .push(guidance_row("close-002", CandidateStatus::Conflicted, ""));

        let lines = next_command_lines("212", &ledger, false);
        let ingest: Vec<&String> = lines
            .iter()
            .filter(|l| l.contains("candidate ingest"))
            .collect();
        assert_eq!(
            ingest.len(),
            1,
            "one ingest line for the one conflicted row"
        );
        assert!(
            ingest[0].contains("--label close-002"),
            "prescription names the conflicted row's label: {}",
            ingest[0]
        );
        assert!(
            ingest[0].contains("--slice 212"),
            "prescription carries the slice: {}",
            ingest[0]
        );
    }

    /// VT-2 — a Created row alone yields no ingest prescription (the guidance is
    /// specific to a conflicted parked row, not blanket noise).
    #[test]
    fn candidate_status_omits_ingest_when_no_conflict() {
        let mut ledger = Candidates::default();
        ledger.rows.push(guidance_row(
            "review-001",
            CandidateStatus::Created,
            "mergeoid",
        ));
        let lines = next_command_lines("212", &ledger, false);
        assert!(
            !lines.iter().any(|l| l.contains("candidate ingest")),
            "no ingest line without a conflicted row"
        );
    }

    // --- SL-212 PHASE-04: `run_candidate_ingest` verb (design §5.3/§5.4) ----------

    /// Build a repo whose `main` (base) and `source` branches conflict ONLY on
    /// `trunk.txt` (so `C == {trunk.txt}` and a base-tree-with-resolved-trunk is a
    /// faithful `R`), seed a Conflicted, un-ingested candidate row for `label`, and
    /// park the candidate branch at base. Returns `(base_oid, source_oid, target_ref)`.
    /// Init a git repo on `main` with an identity and a `.doctrine/state/`
    /// gitignore, then commit a first `root`. The common floor every taxonomy
    /// fixture builds its conflict on top of.
    fn init_test_repo(root: &Path) {
        git(root, &["init", "-q", "-b", "main"]);
        git(root, &["config", "user.email", "t@doctrine.invalid"]);
        git(root, &["config", "user.name", "Doctrine Test"]);
        // Ignore ALL of `.doctrine/` (not just state/): the candidate ledger is
        // runtime here, so a broad `git add -A` in a taxonomy fixture must not
        // sweep candidates.toml into an operator merge tree (it would pollute D)
        // nor let `reset --hard` delete it out from under `candidate_ingest`.
        fs::write(root.join(".gitignore"), ".doctrine/\n").unwrap();
        git(root, &["add", ".gitignore"]);
        git(root, &["commit", "-q", "-m", "root"]);
    }

    /// Seed a Conflicted, un-ingested candidate row for `label` from the given
    /// branch oids and park the candidate branch at `base_oid` (the conflicted-
    /// create end-state). Factored out so taxonomy fixtures supply their own
    /// conflict shapes (rename/modify/delete) while sharing the ledger seam.
    /// Returns `target_ref`.
    fn seed_conflicted_row(
        root: &Path,
        slice: u32,
        label: &str,
        base_oid: &str,
        source_oid: &str,
    ) -> String {
        let slice3 = format!("{slice:03}");
        let target_ref = format!("refs/heads/candidate/{slice3}/{label}");
        git(
            root,
            &["branch", &format!("candidate/{slice3}/{label}"), base_oid],
        );
        let mut ledger = Candidates::default();
        ledger.rows.push(CandidateRow {
            id: format!("cand-{slice3}-{label}"),
            label: label.to_owned(),
            kind: CandidateKind::Audit,
            role: CandidateRole::CloseTarget,
            payload: CandidatePayload::Code,
            target_ref: target_ref.clone(),
            source_ref: "refs/heads/source".to_owned(),
            source_oid: source_oid.to_owned(),
            base_ref: "refs/heads/main".to_owned(),
            base_oid: base_oid.to_owned(),
            merge_oid: String::new(),
            status: CandidateStatus::Conflicted,
            supersedes: String::new(),
            reason: String::new(),
            created_by: "test".to_owned(),
            created_at: "2026-01-01".to_owned(),
            ingested_at: String::new(),
            merge_provenance: crate::ledger::MergeProvenance::Doctrine,
        });
        crate::ledger::write_candidates(root, slice, &ledger).unwrap();
        target_ref
    }

    fn seed_conflicted_candidate(root: &Path, slice: u32, label: &str) -> (String, String, String) {
        init_test_repo(root);
        fs::write(root.join("trunk.txt"), "COMMON\n").unwrap();
        git(root, &["add", "trunk.txt"]);
        git(root, &["commit", "-q", "-m", "common trunk"]);
        // source: edit the conflicting path only.
        git(root, &["checkout", "-q", "-b", "source"]);
        fs::write(root.join("trunk.txt"), "SOURCE\n").unwrap();
        git(root, &["add", "trunk.txt"]);
        git(root, &["commit", "-q", "-m", "source edit"]);
        let source_oid = git(root, &["rev-parse", "HEAD"]);
        // base (main): diverging edit to the same path ⇒ conflict.
        git(root, &["checkout", "-q", "main"]);
        fs::write(root.join("trunk.txt"), "MAIN\n").unwrap();
        git(root, &["add", "trunk.txt"]);
        git(root, &["commit", "-q", "-m", "base edit"]);
        let base_oid = git(root, &["rev-parse", "HEAD"]);
        let target_ref = seed_conflicted_row(root, slice, label, &base_oid, &source_oid);
        (base_oid, source_oid, target_ref)
    }

    /// Commit the CURRENT staged index tree as a genuine 2-parent `[base, source]`
    /// operator merge on `target_ref` (advancing the ref, as an on-branch candidate
    /// checkout would), then restore a clean tree. The caller stages the resolution
    /// first (`git add`/`git rm`). Returns the resolved merge oid `R`.
    fn commit_operator_merge(
        root: &Path,
        base_oid: &str,
        source_oid: &str,
        target_ref: &str,
    ) -> String {
        let tree = git(root, &["write-tree"]);
        let r = git(
            root,
            &[
                "commit-tree",
                &tree,
                "-p",
                base_oid,
                "-p",
                source_oid,
                "-m",
                "operator merge",
            ],
        );
        git(root, &["update-ref", target_ref, &r]);
        git(root, &["reset", "-q", "--hard", "HEAD"]);
        r
    }

    /// The operator resolves the sole conflict path and commits a genuine 2-parent
    /// `[base, source]` merge on `target_ref` (advancing the ref, as an on-branch
    /// candidate checkout would). Returns the resolved merge oid `R`.
    fn operator_resolve(
        root: &Path,
        base_oid: &str,
        source_oid: &str,
        target_ref: &str,
        resolved: &str,
    ) -> String {
        git(root, &["checkout", "-q", "main"]);
        fs::write(root.join("trunk.txt"), resolved).unwrap();
        git(root, &["add", "trunk.txt"]);
        commit_operator_merge(root, base_oid, source_oid, target_ref)
    }

    /// VT-1: a faithful operator merge is ingested — the row flips Created with
    /// `merge_provenance=OperatorIngest`, and `admit` then pins it by the existing
    /// contract (the FF integrate is proven end-to-end in the binary e2e).
    #[test]
    fn run_candidate_ingest_fills_row_with_operator_provenance_and_admits() {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        let (base, source, target_ref) = seed_conflicted_candidate(root, 212, "review-001");
        let r = operator_resolve(root, &base, &source, &target_ref, "RESOLVED\n");

        let req = IngestRequest {
            slice: 212,
            label: "review-001".to_owned(),
            ingested_at: "2026-02-02".to_owned(),
        };
        candidate_ingest(root, &req).expect("ingest accepts a faithful operator merge");

        let ledger = read_candidates(root, 212).unwrap();
        let row = ledger
            .rows
            .iter()
            .find(|r| r.label == "review-001")
            .expect("row present");
        assert_eq!(row.status, CandidateStatus::Created, "row flips to Created");
        assert_eq!(row.merge_oid, r, "merge_oid pinned to the resolved R");
        assert_eq!(
            row.merge_provenance,
            crate::ledger::MergeProvenance::OperatorIngest,
            "provenance records the operator ingest"
        );
        assert!(!row.ingested_at.is_empty(), "ingested_at stamped");

        // admit pins the operator-ingested merge by the existing provenance contract.
        let admit_req = AdmitRequest {
            slice: 212,
            role: CandidateRole::CloseTarget,
            candidate: target_ref.clone(),
            review: None,
            admitted_at: "2026-02-03".to_owned(),
        };
        candidate_admit(root, &admit_req).expect("admit pins the operator-ingested candidate");
        let admitted = read_candidates(root, 212).unwrap();
        assert_eq!(
            admitted
                .current_admission
                .close_target
                .as_ref()
                .expect("close-target admission")
                .admitted_oid,
            r,
            "admit pins the resolved merge R"
        );
    }

    /// Commit an operator merge on `target_ref` with the given ordered `parents`
    /// and a tree that writes each `(path, content)` over a clean base checkout —
    /// the governance-negative builder (reversed parents, an arbitrary-tree edit).
    /// Returns the merge oid. Mirrors `operator_resolve`'s ref-advance + restore.
    fn operator_merge(
        root: &Path,
        target_ref: &str,
        parents: &[&str],
        writes: &[(&str, &str)],
    ) -> String {
        git(root, &["checkout", "-q", "main"]);
        for (path, content) in writes {
            fs::write(root.join(path), content).unwrap();
            git(root, &["add", path]);
        }
        let tree = git(root, &["write-tree"]);
        let mut args = vec!["commit-tree", &tree];
        for p in parents {
            args.push("-p");
            args.push(p);
        }
        args.push("-m");
        args.push("operator merge");
        let r = git(root, &args);
        git(root, &["update-ref", target_ref, &r]);
        git(root, &["reset", "-q", "--hard", "HEAD"]);
        r
    }

    /// VT-1 (EX-2) — the operator-ingest governance set, realised through the
    /// `candidate_ingest` VERB (not just the pure validator): (a) an arbitrary-tree
    /// edit (`D ⊄ C`) is rejected, (b) reversed parents `[source, base]` are
    /// rejected, (c) the genuine ordered 3-way is accepted and records
    /// `OperatorIngest`. The FF-integrate-by-the-same-contract cell (d) is proven
    /// end-to-end in `tests/e2e_dispatch_candidate.rs`.
    #[test]
    fn ingest_governance_arbitrary_and_reversed_reject_genuine_accepts() {
        let req = |slice| IngestRequest {
            slice,
            label: "review-001".to_owned(),
            ingested_at: "2026-02-02".to_owned(),
        };

        // (a) arbitrary tree: the operator merge edits a NON-conflict path — the
        //     D ⊆ C subset gate refuses (never an arbitrary tree).
        {
            let repo = tempfile::tempdir().unwrap();
            let root = repo.path();
            let (base, source, target_ref) = seed_conflicted_candidate(root, 212, "review-001");
            operator_merge(
                root,
                &target_ref,
                &[&base, &source],
                &[("trunk.txt", "RESOLVED\n"), ("arbitrary.txt", "sneaked\n")],
            );
            let err = candidate_ingest(root, &req(212))
                .expect_err("an arbitrary-tree edit is rejected")
                .to_string();
            assert!(
                err.contains("arbitrary.txt"),
                "the refusal names the arbitrary path: {err}"
            );
        }

        // (b) reversed parents [source, base]: the ordered-parent gate refuses.
        {
            let repo = tempfile::tempdir().unwrap();
            let root = repo.path();
            let (base, source, target_ref) = seed_conflicted_candidate(root, 212, "review-001");
            operator_merge(
                root,
                &target_ref,
                &[&source, &base],
                &[("trunk.txt", "RESOLVED\n")],
            );
            assert!(
                candidate_ingest(root, &req(212)).is_err(),
                "reversed parents [source, base] are rejected"
            );
        }

        // (c) genuine ordered 3-way: accepted, row flips Created + OperatorIngest.
        {
            let repo = tempfile::tempdir().unwrap();
            let root = repo.path();
            let (base, source, target_ref) = seed_conflicted_candidate(root, 212, "review-001");
            let r = operator_resolve(root, &base, &source, &target_ref, "RESOLVED\n");
            candidate_ingest(root, &req(212)).expect("the genuine 3-way is accepted");
            let ledger = read_candidates(root, 212).unwrap();
            let row = ledger
                .rows
                .iter()
                .find(|r| r.label == "review-001")
                .unwrap();
            assert_eq!(row.status, CandidateStatus::Created);
            assert_eq!(row.merge_oid, r);
            assert_eq!(
                row.merge_provenance,
                crate::ledger::MergeProvenance::OperatorIngest
            );
        }
    }

    /// The ingest request for slice 212 / `review-001` used across the taxonomy
    /// cells. Distinct `ingested_at` values keep the goldens legible.
    fn taxonomy_req() -> IngestRequest {
        IngestRequest {
            slice: 212,
            label: "review-001".to_owned(),
            ingested_at: "2026-02-02".to_owned(),
        }
    }

    /// VT-3 (D9, EX-3) — taxonomy cell 1: a conflict on a path whose name carries
    /// a **non-UTF-8 byte** (0xFF — deliberately NOT LF/TAB, which the index-info
    /// rewrite cannot represent; the `-z` diff path here has no such limit) round-
    /// trips byte-safe through create→resolve→ingest. C (merge-tree stages) and D
    /// (`changed_paths`) are BOTH `BTreeSet<Vec<u8>>`, so the exact bytes survive
    /// with no lossy UTF-8 compare and ingest ACCEPTS.
    #[test]
    fn ingest_taxonomy_non_utf8_path_round_trips_byte_safe() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let name: &[u8] = b"payload\xff.txt"; // 0xFF: invalid UTF-8, not LF/TAB
        let path = |root: &Path| root.join(OsStr::from_bytes(name));

        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        init_test_repo(root);
        fs::write(path(root), "COMMON\n").unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", "common"]);
        git(root, &["checkout", "-q", "-b", "source"]);
        fs::write(path(root), "SOURCE\n").unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", "source"]);
        let source = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-q", "main"]);
        fs::write(path(root), "MAIN\n").unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", "base"]);
        let base = git(root, &["rev-parse", "HEAD"]);
        let target_ref = seed_conflicted_row(root, 212, "review-001", &base, &source);

        // The mechanical merge conflicts and C carries the EXACT non-UTF-8 bytes.
        let mb = git::merge_base(root, &base, &source).unwrap().unwrap();
        let (tc, stages) = match git::merge_tree(root, &mb, &base, &source).unwrap() {
            git::MergeTree::Conflict { tree, stages } => (tree, stages),
            git::MergeTree::Clean { .. } => panic!("a non-UTF-8 modify/modify must conflict"),
        };
        let c: BTreeSet<Vec<u8>> = stages.iter().map(|s| s.path.clone()).collect();
        assert!(
            c.contains(&name.to_vec()),
            "C carries the exact non-UTF-8 path bytes (no lossy compare)"
        );

        // Operator resolves that path only; R differs from T_c within C ⇒ D ⊆ C.
        git(root, &["checkout", "-q", "main"]);
        fs::write(path(root), "RESOLVED\n").unwrap();
        git(root, &["add", "-A"]);
        let r = commit_operator_merge(root, &base, &source, &target_ref);
        let r_tree = tree_of(root, &r).unwrap();
        let d: BTreeSet<Vec<u8>> = git::changed_paths(root, &r_tree, &tc).unwrap();
        assert!(
            !d.is_empty() && d.iter().all(|p| c.contains(p)),
            "D ⊆ C over raw bytes (non-empty, no lossy compare): D={d:?} C={c:?}"
        );

        candidate_ingest(root, &taxonomy_req())
            .expect("the non-UTF-8 path round-trips byte-safe and ingest accepts");
        let ledger = read_candidates(root, 212).unwrap();
        let row = ledger
            .rows
            .iter()
            .find(|r| r.label == "review-001")
            .unwrap();
        assert_eq!(row.status, CandidateStatus::Created);
        assert_eq!(row.merge_oid, r);
    }

    /// VT-3 — taxonomy cell 2: rename/rename to a THIRD name. base renames X→Y,
    /// source renames X→Z; the operator resolves to a brand-new name W ∉ C. W
    /// lands in D but not C ⇒ D ⊄ C ⇒ ingest REFUSES. This is the §5.5 documented
    /// v1 limitation — no special code, the refusal teaches it (resolve to a
    /// recorded path, or supersede).
    #[test]
    fn ingest_taxonomy_rename_rename_third_name_refuses() {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        init_test_repo(root);
        fs::write(root.join("orig.txt"), "COMMON\n").unwrap();
        git(root, &["add", "orig.txt"]);
        git(root, &["commit", "-q", "-m", "common"]);
        git(root, &["checkout", "-q", "-b", "source"]);
        git(root, &["mv", "orig.txt", "z.txt"]);
        git(root, &["commit", "-q", "-m", "source renames orig->z"]);
        let source = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-q", "main"]);
        git(root, &["mv", "orig.txt", "y.txt"]);
        git(root, &["commit", "-q", "-m", "base renames orig->y"]);
        let base = git(root, &["rev-parse", "HEAD"]);
        let target_ref = seed_conflicted_row(root, 212, "review-001", &base, &source);

        // Operator resolves to a THIRD name w.txt (∉ C).
        git(root, &["checkout", "-q", "main"]);
        git(root, &["rm", "-q", "y.txt"]);
        fs::write(root.join("w.txt"), "RESOLVED\n").unwrap();
        git(root, &["add", "-A"]);
        commit_operator_merge(root, &base, &source, &target_ref);

        assert!(
            candidate_ingest(root, &taxonomy_req()).is_err(),
            "resolving a rename/rename to a third name ∉ C is refused (§5.5 limitation)"
        );
    }

    /// VT-3 — taxonomy cell 3: modify/delete. base modifies X, source deletes X;
    /// the operator resolves AT X (the conflict locus) and ingest ACCEPTS (D ⊆ C).
    #[test]
    fn ingest_taxonomy_modify_delete_accepts() {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        init_test_repo(root);
        fs::write(root.join("keep.txt"), "COMMON\n").unwrap();
        fs::write(root.join("clean.txt"), "CLEAN\n").unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", "common"]);
        git(root, &["checkout", "-q", "-b", "source"]);
        git(root, &["rm", "-q", "keep.txt"]);
        git(root, &["commit", "-q", "-m", "source deletes keep"]);
        let source = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-q", "main"]);
        fs::write(root.join("keep.txt"), "MAIN\n").unwrap();
        git(root, &["add", "keep.txt"]);
        git(root, &["commit", "-q", "-m", "base modifies keep"]);
        let base = git(root, &["rev-parse", "HEAD"]);
        let target_ref = seed_conflicted_row(root, 212, "review-001", &base, &source);

        // Operator keeps the modification at the conflict locus keep.txt.
        git(root, &["checkout", "-q", "main"]);
        fs::write(root.join("keep.txt"), "RESOLVED\n").unwrap();
        git(root, &["add", "keep.txt"]);
        let r = commit_operator_merge(root, &base, &source, &target_ref);

        candidate_ingest(root, &taxonomy_req()).expect("modify/delete resolved within C accepts");
        let ledger = read_candidates(root, 212).unwrap();
        let row = ledger
            .rows
            .iter()
            .find(|r| r.label == "review-001")
            .unwrap();
        assert_eq!(row.status, CandidateStatus::Created);
        assert_eq!(row.merge_oid, r);
    }

    /// VT-3 — taxonomy cell 4: rename/delete (a real close-time class — a sibling
    /// slice deletes a file the base moved). base renames X→Y, source deletes X;
    /// the operator resolves within C and ingest ACCEPTS.
    #[test]
    fn ingest_taxonomy_rename_delete_accepts() {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        init_test_repo(root);
        fs::write(root.join("orig.txt"), "COMMON\n").unwrap();
        fs::write(root.join("clean.txt"), "CLEAN\n").unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", "common"]);
        git(root, &["checkout", "-q", "-b", "source"]);
        git(root, &["rm", "-q", "orig.txt"]);
        git(root, &["commit", "-q", "-m", "source deletes orig"]);
        let source = git(root, &["rev-parse", "HEAD"]);
        git(root, &["checkout", "-q", "main"]);
        git(root, &["mv", "orig.txt", "renamed.txt"]);
        git(root, &["commit", "-q", "-m", "base renames orig->renamed"]);
        let base = git(root, &["rev-parse", "HEAD"]);
        let target_ref = seed_conflicted_row(root, 212, "review-001", &base, &source);

        // Recompute the mechanical conflict and resolve strictly within C: start
        // from T_c and overwrite only the conflict paths with resolved content.
        let mb = git::merge_base(root, &base, &source).unwrap().unwrap();
        let (tc, stages) = match git::merge_tree(root, &mb, &base, &source).unwrap() {
            git::MergeTree::Conflict { tree, stages } => (tree, stages),
            git::MergeTree::Clean { .. } => panic!("rename/delete must conflict"),
        };
        git(root, &["read-tree", "-u", "--reset", &tc]);
        for stage in &stages {
            let p = std::path::Path::new(std::ffi::OsStr::new(
                std::str::from_utf8(&stage.path).expect("ascii conflict path in this fixture"),
            ));
            fs::write(root.join(p), "RESOLVED\n").unwrap();
        }
        git(root, &["add", "-A"]);
        let r = commit_operator_merge(root, &base, &source, &target_ref);

        candidate_ingest(root, &taxonomy_req()).expect("rename/delete resolved within C accepts");
        let ledger = read_candidates(root, 212).unwrap();
        let row = ledger
            .rows
            .iter()
            .find(|r| r.label == "review-001")
            .unwrap();
        assert_eq!(row.status, CandidateStatus::Created);
        assert_eq!(row.merge_oid, r);
    }

    /// VT-2 (RV-289 F-1): ingest from a candidate-worktree cwd (a root resolved
    /// UNDER `.doctrine/state/dispatch/candidate/`) is refused with a message
    /// directing to the coordination tree. The coord-root acceptance is proven by
    /// the happy path above (which runs from the coordination root).
    #[test]
    fn run_candidate_ingest_refuses_a_candidate_worktree_cwd() {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        seed_conflicted_candidate(root, 212, "review-001");
        let candidate_cwd = root
            .join(CANDIDATE_WORKTREE_SUBPATH)
            .join("cand-212-review-001");
        fs::create_dir_all(&candidate_cwd).unwrap();
        let req = IngestRequest {
            slice: 212,
            label: "review-001".to_owned(),
            ingested_at: "2026-02-02".to_owned(),
        };
        let err = candidate_ingest(&candidate_cwd, &req)
            .expect_err("a candidate-worktree cwd is refused")
            .to_string();
        assert!(
            err.contains("candidate checkout") && err.contains("coordination tree"),
            "refusal directs to the coordination tree: {err}"
        );
    }

    /// VT-3: the write-once pre-state gate — an un-committed ref (`R==base`), an
    /// already-ingested (Created) row, and an ambiguous label all refuse.
    #[test]
    fn run_candidate_ingest_write_once_pre_state_refuses() {
        // (a) R == base: the operator has not committed — refuse.
        {
            let repo = tempfile::tempdir().unwrap();
            let root = repo.path();
            seed_conflicted_candidate(root, 212, "review-001");
            let req = IngestRequest {
                slice: 212,
                label: "review-001".to_owned(),
                ingested_at: "d".to_owned(),
            };
            let err = candidate_ingest(root, &req)
                .expect_err("an un-committed ref refuses")
                .to_string();
            assert!(err.contains("still points at base"), "{err}");
        }
        // (b) already Created: write-once — a second ingest finds no conflicted row.
        {
            let repo = tempfile::tempdir().unwrap();
            let root = repo.path();
            let (base, source, target_ref) = seed_conflicted_candidate(root, 212, "review-001");
            operator_resolve(root, &base, &source, &target_ref, "RESOLVED\n");
            let req = IngestRequest {
                slice: 212,
                label: "review-001".to_owned(),
                ingested_at: "d".to_owned(),
            };
            candidate_ingest(root, &req).expect("first ingest succeeds");
            let err = candidate_ingest(root, &req)
                .expect_err("a Created row cannot be re-ingested")
                .to_string();
            assert!(err.contains("no un-ingested conflicted"), "{err}");
        }
        // (c) ambiguous label: two Conflicted rows share the label — refuse.
        {
            let repo = tempfile::tempdir().unwrap();
            let root = repo.path();
            seed_conflicted_candidate(root, 212, "review-001");
            let mut ledger = read_candidates(root, 212).unwrap();
            let mut dup = ledger.rows.first().expect("seed row").clone();
            dup.id = "cand-212-review-001-dup".to_owned();
            ledger.rows.push(dup);
            crate::ledger::write_candidates(root, 212, &ledger).unwrap();
            let req = IngestRequest {
                slice: 212,
                label: "review-001".to_owned(),
                ingested_at: "d".to_owned(),
            };
            let err = candidate_ingest(root, &req)
                .expect_err("an ambiguous label refuses")
                .to_string();
            assert!(err.contains("ambiguous"), "{err}");
        }
    }
}
