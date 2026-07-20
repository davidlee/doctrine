// SPDX-License-Identifier: GPL-3.0-only
//! §B transactional core + the dispatch funnel's WRITE SURFACE (SL-199 PHASE-02/03,
//! design §B). PHASE-02 built the two INTERNAL primitives; PHASE-03 wires the three
//! MCP tools (`dispatch_import` / `dispatch_conclude_phase` / `dispatch_reap`) onto
//! them, each resolving the coord tree server-side by slice-id and composing an
//! EXISTING belt (`classify_import` / `run_gc`) with the primitives below — one seam,
//! no forked logic.
//!
//! 1. [`resolve_coord`] — coord-by-slice resolver. Enumerates worktrees (the SHARED
//!    [`git::list_worktrees`] multi-hit seam — NOT the single-hit `worktree_for_ref`,
//!    which cannot raise the defensive `ambiguous` arm) and filters to the ONE live
//!    worktree on `dispatch/<NNN>`. No path parameter enters resolution.
//! 2. [`commit_on_behalf`] — composes ONE non-merge commit working-tree-free onto the
//!    coord branch tip (`commit-tree` object-db-only, never the live index) and lands
//!    it with a compare-and-swap ([`git::update_ref_cas`]) so a moved tip is a typed
//!    refusal, never a clobber. On ANY fault the live coord index + worktree are
//!    BYTE-UNCHANGED (the primitive never reads or writes them).
//!
//! ADR-001 leaf/engine split (mirror of `classify_resolve` / `resolve_agent`): a PURE
//! classifier ([`classify_coord`]) reasons over already-gathered FACTS; the impure
//! [`resolve_coord`] gathers them (the `git worktree list` enumerate + the branch-tip
//! rev-parse) then classifies. Provenance is set EXPLICITLY via `GIT_AUTHOR_*`/
//! `GIT_COMMITTER_*` (mirror of `git::commit_empty_tree_as`), the one seam the shipped
//! `commit_tree` lacks — a named-tree, two-identity commit primitive whose natural home
//! is the git leaf, promoted there in a follow-up (out of this phase's declared set).

use crate::boundary::{BoundaryRow, Provenance as BoundaryProvenance};
use crate::dispatch::{
    CommitOutcome, Identity, Provenance, commit_on_behalf, dispatch_identity, dispatch_ref,
    funnel_message, land_boundary_row,
};
use crate::git::{self, WorktreeRecord};
use crate::ledger::Boundaries;
use crate::worktree::{Apply, classify_import, run_gc};
use anyhow::Context;
use serde::Serialize;
use std::path::{Path, PathBuf};

// --- resolve_coord: coord-by-slice ----------------------------------------------------

/// Why [`resolve_coord`] refuses (design §B). Fails closed with a distinct named token —
/// the property the goldens assert, never a proxy. A SIBLING of the agent resolver's
/// [`crate::worktree::DispatchRecord`] refusals: coord-by-slice is its own verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoordRefusal {
    /// Zero live worktrees on `dispatch/<NNN>` — no such live coord for this slice.
    UnknownSlice,
    /// More than one hit. git guarantees ≤1 worktree per branch, so this is normally
    /// unreachable; kept as a DEFENSIVE refusal (mirror of `AmbiguousAgent`).
    Ambiguous,
    /// A single hit that is inconsistent: the dir is gone, git annotated it `prunable`
    /// (a stale gitdir no live checkout backs), or the branch tip is unresolved.
    Stale,
}

impl CoordRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            CoordRefusal::UnknownSlice => "unknown-slice",
            CoordRefusal::Ambiguous => "ambiguous",
            CoordRefusal::Stale => "stale",
        }
    }
}

/// The gathered, impure-read facts the PURE [`classify_coord`] reasons over (mirror of
/// `ResolveFacts`). Every field is a FACT gathered in [`resolve_coord`]'s shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CoordFacts {
    /// Worktrees checked out on `dispatch/<NNN>` (git guarantees ≤1; `> 1` is defended).
    pub(crate) branch_hits: usize,
    /// The single hit's worktree path (verbatim from the porcelain), `None` on 0 hits.
    pub(crate) path: Option<PathBuf>,
    /// git annotated the single hit `prunable` — a stale gitdir no live checkout backs.
    pub(crate) prunable: bool,
    /// The single hit's `path` still exists on disk.
    pub(crate) dir_exists: bool,
    /// The resolved tip of `dispatch/<NNN>`; `None` ⇒ branch unresolved.
    pub(crate) tip: Option<String>,
}

/// The resolved live coordination worktree for a slice — its root and current branch tip
/// (the CAS `expected_old` a subsequent [`commit_on_behalf`] guards against).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CoordTarget {
    /// The coordination worktree root (the `dispatch/<NNN>` checkout).
    pub(crate) root: PathBuf,
    /// The branch tip at resolution time.
    pub(crate) tip: String,
}

/// PURE coord classifier (no git / disk — ADR-001 leaf). Mirror of `classify_resolve`:
/// 0 hits ⇒ `unknown-slice`; `> 1` ⇒ `ambiguous`; a single hit whose dir is gone, that
/// git annotated `prunable`, or whose branch tip is unresolved ⇒ `stale`; otherwise the
/// live coord target (design §B).
pub(crate) fn classify_coord(facts: CoordFacts) -> Result<CoordTarget, CoordRefusal> {
    if facts.branch_hits == 0 {
        return Err(CoordRefusal::UnknownSlice);
    }
    if facts.branch_hits > 1 {
        return Err(CoordRefusal::Ambiguous);
    }
    let Some(root) = facts.path else {
        return Err(CoordRefusal::Stale);
    };
    if facts.prunable || !facts.dir_exists {
        return Err(CoordRefusal::Stale);
    }
    let Some(tip) = facts.tip else {
        return Err(CoordRefusal::Stale);
    };
    Ok(CoordTarget { root, tip })
}

/// Resolve a slice to its live coordination worktree (design §B), else a typed
/// [`CoordRefusal`]. NO path parameter: the slice keys the canonical coord branch
/// `dispatch/<NNN>`, which is matched against the SHARED [`git::list_worktrees`]
/// enumerate (the multi-hit seam — so a defensive `> 1` is *seen*, not first-match-
/// masked). A git-read failure folds to empty (⇒ `unknown-slice`), mirroring how
/// `resolve_agent` folds its seam error to `None`.
///
/// Gather (impure) → [`classify_coord`] (pure):
/// 1. build the canonical branch ref `refs/heads/dispatch/<NNN>`,
/// 2. enumerate worktrees; count/keep the ones on that branch,
/// 3. gather the single hit's `path` existence + `prunable`, and the branch tip,
/// 4. classify.
pub(crate) fn resolve_coord(root: &Path, slice: u32) -> Result<CoordTarget, CoordRefusal> {
    let branch_ref = format!("refs/heads/dispatch/{slice:03}");
    let records = git::list_worktrees(root).unwrap_or_default();
    let hits: Vec<&WorktreeRecord> = records
        .iter()
        .filter(|r| r.branch.as_deref() == Some(branch_ref.as_str()))
        .collect();
    let branch_hits = hits.len();
    let single = hits.first().copied();
    let path = single.map(|r| r.path.clone());
    let prunable = single.is_some_and(|r| r.prunable);
    let dir_exists = single.is_some_and(|r| r.path.exists());
    let tip = git::git_opt(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{branch_ref}^{{commit}}"),
        ],
    )
    .ok()
    .flatten();

    classify_coord(CoordFacts {
        branch_hits,
        path,
        prunable,
        dir_exists,
        tip,
    })
}

// ======================================================================================
// §C the funnel WRITE SURFACE — three MCP tools over the §B primitives (PHASE-03)
// ======================================================================================
//
// Each tool resolves the coord tree SERVER-SIDE by slice-id ([`resolve_coord`], no
// caller path) and composes an EXISTING belt with the §B primitives — one seam, no
// forked logic:
//   * `dispatch_import`         — [`classify_import`] scope belt (pre-compose) →
//                                 [`git::merge_tree`] compose → [`commit_on_behalf`]
//   * `dispatch_conclude_phase` — [`crate::state::set_phase_status`] gitignored sheet
//                                 flip + [`commit_on_behalf`] of the boundary row
//   * `dispatch_reap`           — [`run_gc`] (the CLI gc's landed-oracle, UNCHANGED)

/// The tool name the MCP registry keys `dispatch_import` on (STD-001 single-source —
/// PHASE-04's lint references these, never a bare literal).
pub(crate) const TOOL_DISPATCH_IMPORT: &str = "dispatch_import";
/// The tool name the MCP registry keys `dispatch_conclude_phase` on (STD-001).
pub(crate) const TOOL_DISPATCH_CONCLUDE_PHASE: &str = "dispatch_conclude_phase";
/// The tool name the MCP registry keys `dispatch_reap` on (STD-001).
pub(crate) const TOOL_DISPATCH_REAP: &str = "dispatch_reap";

/// Refuse: `dispatch_import`'s working-tree-free [`git::merge_tree`] compose of
/// coord-tip ⊕ worker-tip hit a content conflict. File-disjoint dispatch makes this
/// unreachable in the happy topology; kept as a fail-closed token rather than a silent
/// bad tree (design D-B4).
const MERGE_CONFLICT: &str = "merge-conflict";

/// The outcome every funnel tool returns — a landed result or a typed refusal.
/// Serialised externally-tagged (`{"Imported": {…}}` / `{"Refused": {…}}`), matching
/// the `worker_commit` shape: a belt refusal is a normal `Ok` carrying its token, never
/// a JSON-RPC error, so the orchestrator reads the reason structurally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum FunnelOutcome {
    /// `dispatch_import` landed the worker delta; `coord_tip` is the advanced tip.
    Imported { coord_tip: String },
    /// `dispatch_conclude_phase` landed the boundary commit; `coord_tip` is the new tip.
    Concluded { coord_tip: String },
    /// `dispatch_reap` reaped the (landed) fork worktree + branch.
    Reaped { fork: String },
    /// A belt/primitive refused; `reason` is the distinct token, `detail` the offending
    /// path / context (empty when the token is self-describing).
    Refused { reason: String, detail: String },
}

fn funnel_refused(reason: &str, detail: String) -> FunnelOutcome {
    FunnelOutcome::Refused {
        reason: reason.to_owned(),
        detail,
    }
}

/// Read a commit's AUTHOR identity (`%an`/`%ae`) — the worker author `dispatch_import`
/// preserves under IMPORT provenance.
fn commit_author(root: &Path, rev: &str) -> anyhow::Result<Identity> {
    let raw = git::git_text(root, &["log", "-1", "--format=%an%n%ae", rev])
        .with_context(|| format!("read author of {rev}"))?;
    let mut lines = raw.lines();
    Ok(Identity {
        name: lines.next().unwrap_or_default().to_owned(),
        email: lines.next().unwrap_or_default().to_owned(),
    })
}

// --- T1: dispatch_import ---------------------------------------------------------------

/// `dispatch_import{slice, name}` — import a worker's committed fork branch `name` onto
/// the live coord tip, working-tree-free (design §B / D-B4). Resolve the coord by slice,
/// run the [`classify_import`] scope belt as a HARD pre-compose gate (an undeclared path
/// lands NOTHING — coord tip unchanged), compose coord-tip ⊕ worker-tip with
/// [`git::merge_tree`] (the trivial `coord-tip == B` case commits the worker tree
/// directly), and land ONE non-merge commit via [`commit_on_behalf`] under IMPORT
/// provenance (worker AUTHOR preserved + dispatch COMMITTER). Returns the advanced
/// `coord_tip`, or a typed refusal (coord / scope / commit). The live coord index +
/// worktree are NEVER touched — the compose is object-db only.
pub(crate) fn dispatch_import(
    root: &Path,
    slice: u32,
    name: &str,
) -> anyhow::Result<FunnelOutcome> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
    };
    // The slice's design-target selectors gate the scope belt (strict, HARD — a read
    // failure propagates rather than silently disabling the gate).
    let selectors = crate::slice::selectors(
        &coord.root,
        slice,
        Some(crate::slice::SelectorIntent::DesignTarget),
    )?;
    import_compose(&coord, slice, name, &selectors)
}

/// The compose core of [`dispatch_import`] (selectors already resolved) — the pure belt
/// then a working-tree-free compose and commit, factored so a unit test drives it with
/// explicit selectors without standing up a full authored slice.
fn import_compose(
    coord: &CoordTarget,
    slice: u32,
    name: &str,
    selectors: &[String],
) -> anyhow::Result<FunnelOutcome> {
    let fork_tip = git::git_text(&coord.root, &["rev-parse", &format!("{name}^{{commit}}")])
        .with_context(|| format!("resolve fork tip {name}"))?;
    // B = the common ancestor of the coord tip and the worker fork (the dispatch base
    // both descend from); when the coord tip has not advanced yet, B == coord tip.
    let base = git::git_text(&coord.root, &["merge-base", &coord.tip, &fork_tip])
        .with_context(|| format!("merge-base {} {fork_tip}", coord.tip))?;

    // single_commit fact — `<fork>^ == B` (exactly one non-merge commit on the fork).
    let fork_parent = git::git_opt(
        &coord.root,
        &["rev-parse", "--verify", &format!("{name}^^{{commit}}")],
    )?;
    let single_commit = fork_parent.as_deref() == Some(base.as_str());

    // Belt input — the B..fork name-only TRACKED diff, belt-hardened (quotePath off so a
    // non-ASCII governance path is verbatim; --no-renames so a `.doctrine/` source leg
    // cannot hide behind a same-content destination) — identical to the CLI import gather.
    let diff = git::git_text(
        &coord.root,
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            "--name-only",
            "--no-renames",
            &format!("{base}..{fork_tip}"),
        ],
    )?;
    let delta_paths: Vec<String> = diff.lines().map(str::to_owned).collect();

    // HARD pre-compose gate — the SHARED pure belt. `head_at_base`/`tree_clean` are
    // vacuously true here: the compose is working-tree-free onto the coord tip, so
    // neither a coord HEAD position nor a dirty coord tree is a precondition (the CLI
    // import needs them because it applies into the live index; the funnel never does).
    // The load-bearing legs are the `.doctrine/`/`.claude/` prefix reject + the scope
    // leg — the SAME verdict the CLI import reaches (VA-1 agreement).
    match classify_import(true, true, single_commit, &delta_paths, selectors) {
        Ok(Apply::Ok) => {}
        Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
    }

    // Compose coord-tip ⊕ worker-tip working-tree-free (object-db only, EX-1). When the
    // coord tip is still B, the 3-way is trivial (merge(B, B, fork) == fork's tree), so
    // commit the fork tip's tree directly; otherwise a 3-way union against B.
    let tree = if coord.tip == base {
        git::git_text(&coord.root, &["rev-parse", &format!("{fork_tip}^{{tree}}")])
            .with_context(|| format!("resolve fork tree {fork_tip}"))?
    } else {
        match git::merge_tree(&coord.root, &base, &coord.tip, &fork_tip)? {
            git::MergeTree::Clean { tree } => tree,
            git::MergeTree::Conflict => return Ok(funnel_refused(MERGE_CONFLICT, String::new())),
        }
    };

    // Land ONE non-merge commit — IMPORT provenance keeps the worker author, stamps the
    // dispatch committer. The CAS parent is the coord tip (a moved tip ⇒ lost-ref-race).
    let prov = Provenance::Import {
        author: commit_author(&coord.root, &fork_tip)?,
        committer: dispatch_identity(),
    };
    let message = funnel_message(slice, &format!("import {name}"));
    match commit_on_behalf(
        &coord.root,
        &dispatch_ref(slice),
        &coord.tip,
        &tree,
        &message,
        &prov,
    )? {
        CommitOutcome::Landed { oid } => Ok(FunnelOutcome::Imported { coord_tip: oid }),
        CommitOutcome::Refused(refusal) => Ok(funnel_refused(refusal.token(), String::new())),
    }
}

// --- T2: dispatch_conclude_phase -------------------------------------------------------

/// `dispatch_conclude_phase{slice, phase, code_start, code_end, note?}` — conclude a
/// phase in TWO kept-separate tiers (design §B):
/// * (a) [`crate::state::set_phase_status`] flips the GITIGNORED phase sheet to
///   `completed` — disposable runtime, idempotent on retry, NEVER in committed history;
/// * (b) ONE working-tree-free [`commit_on_behalf`] lands the `(code_start, code_end)`
///   boundary row (CONCLUDE provenance: author == committer == dispatch), UPSERT-by-phase.
///
/// Atomicity by construction: the boundary commit is all-or-nothing (§B). The only fault
/// outcome is a completed sheet with NO committed boundary — self-healing (the sheet is
/// disposable; a retry re-composes the same boundary). The sheet write is DELIBERATELY
/// NOT folded into the commit.
pub(crate) fn dispatch_conclude_phase(
    root: &Path,
    slice: u32,
    phase: &str,
    code_start: &str,
    code_end: &str,
    note: Option<&str>,
) -> anyhow::Result<FunnelOutcome> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
    };
    // (a) gitignored sheet flip — the disposable runtime tier. `set_phase_status` is the
    // exact `fs::write` sheet writer `run_phase` wraps; called directly (not via
    // `run_phase`) so no CLI-shell `stdout` print pollutes the MCP JSON-RPC channel.
    let now = crate::clock::now_timestamp()?;
    crate::state::set_phase_status(
        &coord.root,
        slice,
        phase,
        crate::state::PhaseStatus::Completed,
        note,
        &now,
    )?;
    // (a') The coord flip above auto-mirrors its `completed` into the PRIMARY tree
    // (ISS-212): `set_phase_status` is the single writer and now owns the mirror, so
    // `prepare-review`'s completeness gate — which reads the completed set from the
    // primary sheet — sees this phase without a per-call-site block here (IMP-272's
    // narrow mirror, now generalised down into the writer).
    // (b) the working-tree-free boundary commit — the durable, committed tier.
    conclude_boundary_commit(&coord.root, &coord.tip, slice, phase, code_start, code_end)
}

/// Tier (b) of [`dispatch_conclude_phase`]: compose the `(code_start, code_end)`
/// [`BoundaryRow`] into `boundaries.toml` (UPSERT-by-phase — any existing row for
/// `phase` is replaced, never duplicated) and land it with ONE working-tree-free
/// [`commit_on_behalf`] onto `tip`. Factored out so a unit test can inject a fault at the ref-update step (a
/// STALE `tip`) and prove the boundary is ABSENT + the live index/worktree byte-unchanged.
fn conclude_boundary_commit(
    coord_root: &Path,
    tip: &str,
    slice: u32,
    phase: &str,
    code_start: &str,
    code_end: &str,
) -> anyhow::Result<FunnelOutcome> {
    // The `(code_start, code_end)` row this phase concludes; the UPSERT-by-phase fold,
    // splice, and working-tree-free commit are the shared `land_boundary_row` writer.
    let row = BoundaryRow {
        phase: phase.to_owned(),
        code_start_oid: code_start.to_owned(),
        code_end_oid: code_end.to_owned(),
        provenance: BoundaryProvenance::Funnel,
    };
    let prov = Provenance::Conclude {
        who: dispatch_identity(),
    };
    match land_boundary_row(coord_root, &dispatch_ref(slice), tip, slice, row, &prov)? {
        CommitOutcome::Landed { oid } => Ok(FunnelOutcome::Concluded { coord_tip: oid }),
        CommitOutcome::Refused(refusal) => Ok(funnel_refused(refusal.token(), String::new())),
    }
}

// --- T3: dispatch_reap -----------------------------------------------------------------

/// `dispatch_reap{slice, name}` — reap the worker fork branch `name` via the CLI gc's
/// [`run_gc`], UNCHANGED (design §B): the shared patch-id landed-oracle (`git cherry`)
/// REFUSES deleting a fork whose patch is not yet in coord history; a landed fork's
/// worktree + branch are removed. Resolves the coord by slice, then delegates — no forked
/// scope/landed logic (VA-1: `dispatch_reap` and the CLI gc agree on the landed verdict).
pub(crate) fn dispatch_reap(root: &Path, slice: u32, name: &str) -> anyhow::Result<FunnelOutcome> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
    };
    // Delegate to the CLI gc UNCHANGED (lands against coord HEAD; a not-landed fork is a
    // hard refusal from `run_gc` — propagated as an Err, never a silent reap).
    run_gc(Some(coord.root), name, None, false, false)?;
    Ok(FunnelOutcome::Reaped {
        fork: name.to_owned(),
    })
}

// ======================================================================================
// §D the funnel READ SURFACE — three read-only MCP tools over the coord (SL-206 PHASE-03)
// ======================================================================================
//
// Each tool resolves the coord tree SERVER-SIDE by slice-id ([`resolve_coord`], no caller
// path) and reads — NEVER mutates — the coordination state, composing an EXISTING
// authority, no forked logic:
//   * `dispatch_phase_receipt`       — the PHASE-02 per-phase projection
//                                      ([`crate::dispatch::phase_receipt_status`]) + the
//                                      committed boundary row.
//   * `dispatch_next_ready`          — the readiness authority verbatim
//                                      ([`crate::dispatch::compute_next_phases`] over the
//                                      shared [`crate::dispatch::plan_next_rows`] seam).
//   * `dispatch_authored_divergence` — `.doctrine/**` divergence over
//                                      trunk-authority..dispatch-tip
//                                      ([`git::trunk_commit`] + [`git::diff_doctrine_paths`]).

/// The tool name the MCP registry keys `dispatch_phase_receipt` on (STD-001 single-source).
pub(crate) const TOOL_DISPATCH_PHASE_RECEIPT: &str = "dispatch_phase_receipt";
/// The tool name the MCP registry keys `dispatch_next_ready` on (STD-001).
pub(crate) const TOOL_DISPATCH_NEXT_READY: &str = "dispatch_next_ready";
/// The tool name the MCP registry keys `dispatch_authored_divergence` on (STD-001).
pub(crate) const TOOL_DISPATCH_AUTHORED_DIVERGENCE: &str = "dispatch_authored_divergence";

/// The outcome shape shared by every §D read tool: a `Resolved(core)` payload, or a
/// `CoordRefused { reason }` carrying the [`CoordRefusal`] token VERBATIM — with NO
/// fabricated tip or payload on the refusal path (EX-1). Serialised externally-tagged
/// (`{"Resolved": {…}}` / `{"CoordRefused": { "reason": … }}`), matching the write
/// surface's `Ok`-carries-refusal convention: a coord refusal is a normal result, never a
/// JSON-RPC error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum ReadOutcome<T> {
    /// The coord resolved; `T` is the read payload.
    Resolved(T),
    /// [`resolve_coord`] refused (unknown-slice | ambiguous | stale); `reason` is the
    /// distinct token, and NO payload/tip is fabricated.
    CoordRefused { reason: String },
}

/// `dispatch_phase_receipt` outcome — a per-phase receipt, or a coord refusal.
pub(crate) type PhaseReceiptResult = ReadOutcome<PhaseReceiptCore>;
/// `dispatch_next_ready` outcome — the ready-phase readout, or a coord refusal.
pub(crate) type NextReadyResult = ReadOutcome<NextReadyCore>;
/// `dispatch_authored_divergence` outcome — the divergence readout, or a coord refusal.
pub(crate) type DivergenceResult = ReadOutcome<DivergenceCore>;

/// Fold a [`CoordRefusal`] to the shared `CoordRefused` arm (token VERBATIM, no tip).
fn coord_refused<T>(refusal: CoordRefusal) -> ReadOutcome<T> {
    ReadOutcome::CoordRefused {
        reason: refusal.token().to_owned(),
    }
}

/// The `Resolved` payload of [`dispatch_phase_receipt`]: a phase's receipt over three
/// tiers. `dispatch_tip` is the LIVE coord branch tip (real, from [`resolve_coord`]) —
/// DISTINCT from `code_end`, the phase's committed boundary end oid (EX-1); both are
/// carried as separate fields so a consumer can see the tip has advanced past the recorded
/// boundary. `code_start`/`code_end` are absent when no boundary row backs the phase yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PhaseReceiptCore {
    pub(crate) slice: u32,
    pub(crate) phase: String,
    /// The rich [`crate::dispatch::ReceiptStatus`] token (kebab-case) — `conclude-incomplete`
    /// stays distinct from `completed`.
    pub(crate) status: String,
    /// The live coordination branch tip (real, distinct from `code_end`).
    pub(crate) dispatch_tip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code_end: Option<String>,
}

/// `dispatch_phase_receipt{slice, phase}` — resolve the coord SERVER-SIDE, then project a
/// SINGLE phase's receipt over the PHASE-02 authority (design §A / EX-1). Read-only: no
/// coord mutation. EVERY [`resolve_coord`] refusal (unknown-slice | ambiguous | stale)
/// short-circuits to `CoordRefused(reason)` with NO fabricated tip. On resolution the core
/// carries the LIVE `dispatch_tip` (the coord branch tip) and — when a committed boundary
/// row backs the phase — the `(code_start, code_end)` oids, `code_end` being DISTINCT from
/// the live tip. The receipt status rides [`crate::dispatch::phase_receipt_status`]
/// (the same per-row derivation `run_status` uses), with `has_boundary` sourced from the
/// committed boundaries ledger at the tip.
pub(crate) fn dispatch_phase_receipt(
    root: &Path,
    slice: u32,
    phase: &str,
) -> anyhow::Result<PhaseReceiptResult> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(coord_refused(refusal)),
    };
    // The committed boundaries ledger at the LIVE tip (object-db read; the live coord
    // index/worktree are never touched). Absent ⇒ empty ⇒ no boundary backs the phase.
    let boundaries = read_boundaries_at(&coord.root, &coord.tip, slice)?;
    let boundary = boundaries.rows.iter().find(|r| r.phase == phase);
    let has_boundary = boundary.is_some();
    // The disposable runtime sheet lives under the coord worktree's gitignored state.
    let state_dir = crate::state::phases_dir(&coord.root, slice);
    let status = crate::dispatch::phase_receipt_status(&state_dir, phase, has_boundary);
    Ok(ReadOutcome::Resolved(PhaseReceiptCore {
        slice,
        phase: phase.to_owned(),
        status: status.as_str().to_owned(),
        dispatch_tip: coord.tip,
        code_start: boundary.map(|b| b.code_start_oid.clone()),
        code_end: boundary.map(|b| b.code_end_oid.clone()),
    }))
}

/// One phase row in a [`NextReadyCore`] readout — the plan id, its legacy status string,
/// and its name (the same tuple the readiness seam yields).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct NextPhaseRow {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) name: String,
}

/// The `Resolved` payload of [`dispatch_next_ready`]: the `next` actionable phase id(s)
/// — the [`crate::dispatch::compute_next_phases`] output VERBATIM (EX-2) — alongside the
/// full ordered `phases` readout the readiness was computed from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct NextReadyCore {
    pub(crate) next: Vec<String>,
    pub(crate) phases: Vec<NextPhaseRow>,
}

/// `dispatch_next_ready{slice}` — resolve the coord SERVER-SIDE, then return the next
/// actionable phase(s) as computed by the EXISTING readiness authority
/// [`crate::dispatch::compute_next_phases`] over the SHARED
/// [`crate::dispatch::plan_next_rows`] seam (design §A / EX-2) — the SAME value `dispatch
/// plan-next` renders, no parallel readiness logic. Read-only. A [`resolve_coord`] refusal
/// short-circuits to `CoordRefused(reason)`.
pub(crate) fn dispatch_next_ready(root: &Path, slice: u32) -> anyhow::Result<NextReadyResult> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(coord_refused(refusal)),
    };
    let rows = crate::dispatch::plan_next_rows(&coord.root, slice)?;
    let next = crate::dispatch::compute_next_phases(&rows);
    let phases = rows
        .into_iter()
        .map(|(id, status, name)| NextPhaseRow { id, status, name })
        .collect();
    Ok(ReadOutcome::Resolved(NextReadyCore { next, phases }))
}

/// The `Resolved` payload of [`dispatch_authored_divergence`]: whether the coord's
/// `.doctrine/**` authored tree has `diverged` from the trunk authority, the resolved
/// `compared_ref` (the trunk commit the diff was taken against), and the `drifted_paths`
/// (present only when non-empty).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DivergenceCore {
    pub(crate) diverged: bool,
    pub(crate) compared_ref: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) drifted_paths: Vec<String>,
}

/// `dispatch_authored_divergence{slice}` — resolve the coord SERVER-SIDE, then report
/// whether its `.doctrine/**` authored tree has diverged from the trunk over
/// `trunk_ref..dispatch_tip` (design §A / EX-3, EX-8). The trunk `compared_ref` is
/// resolved from the REAL trunk authority [`git::trunk_commit`] (the peeled ladder —
/// `DOCTRINE_TRUNK_REF` / `origin/HEAD` / `main` / `master`), NEVER hardcoded `edge` and
/// NEVER the journal-row printer. `dispatch_tip` is the live coord tip. Read-only: no
/// coord mutation — a name-only diff over the [`crate::corpus_guard::DOCTRINE_PATHSPEC`]
/// authored subtree. A [`resolve_coord`] refusal short-circuits to `CoordRefused(reason)`.
pub(crate) fn dispatch_authored_divergence(
    root: &Path,
    slice: u32,
) -> anyhow::Result<DivergenceResult> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(coord_refused(refusal)),
    };
    let compared_ref = git::trunk_commit(&coord.root)?.context(
        "trunk ref not found (no DOCTRINE_TRUNK_REF / origin/HEAD / main / master resolves)",
    )?;
    let drifted_paths = git::diff_doctrine_paths(
        &coord.root,
        &compared_ref,
        &coord.tip,
        crate::corpus_guard::DOCTRINE_PATHSPEC,
    )?;
    Ok(ReadOutcome::Resolved(DivergenceCore {
        diverged: !drifted_paths.is_empty(),
        compared_ref,
        drifted_paths,
    }))
}

/// Read the committed boundaries ledger at `tip` (`.doctrine/dispatch/<NNN>/boundaries.toml`)
/// — an object-db read (the live coord index/worktree are never touched). Absent ⇒ the
/// empty ledger. Mirrors `read_ledger` in the CLI, but keyed on a resolved tip.
fn read_boundaries_at(root: &Path, tip: &str, slice: u32) -> anyhow::Result<Boundaries> {
    let path = format!(".doctrine/dispatch/{slice:03}/boundaries.toml");
    match git::read_path_at(root, tip, &path)? {
        Some(text) => Boundaries::parse(&text)
            .with_context(|| format!("parse committed boundaries.toml at {tip}")),
        None => Ok(Boundaries::default()),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on fixture setup is idiomatic"
)]
mod tests {
    use super::*;
    use crate::worktree::Refusal;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn git_run(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
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

    // --- VT-1: resolve_coord (pure classify + one shell round-trip) -------------------

    #[test]
    fn classify_coord_happy_returns_the_live_target() {
        let facts = CoordFacts {
            branch_hits: 1,
            path: Some(PathBuf::from("/x/coord")),
            prunable: false,
            dir_exists: true,
            tip: Some("abc".to_string()),
        };
        assert_eq!(
            classify_coord(facts),
            Ok(CoordTarget {
                root: PathBuf::from("/x/coord"),
                tip: "abc".to_string(),
            })
        );
    }

    #[test]
    fn classify_coord_zero_hits_is_unknown_slice() {
        let facts = CoordFacts {
            branch_hits: 0,
            path: None,
            prunable: false,
            dir_exists: false,
            tip: None,
        };
        assert_eq!(classify_coord(facts), Err(CoordRefusal::UnknownSlice));
        assert_eq!(CoordRefusal::UnknownSlice.token(), "unknown-slice");
    }

    #[test]
    fn classify_coord_multi_hit_is_ambiguous() {
        // A synthetic > 1 listing — the defensive arm the multi-hit enumerate raises.
        let facts = CoordFacts {
            branch_hits: 2,
            path: Some(PathBuf::from("/x/coord")),
            prunable: false,
            dir_exists: true,
            tip: Some("abc".to_string()),
        };
        assert_eq!(classify_coord(facts), Err(CoordRefusal::Ambiguous));
        assert_eq!(CoordRefusal::Ambiguous.token(), "ambiguous");
    }

    #[test]
    fn classify_coord_single_hit_inconsistencies_are_stale() {
        let stale_variants = [
            // dir gone
            CoordFacts {
                branch_hits: 1,
                path: Some(PathBuf::from("/x/coord")),
                prunable: false,
                dir_exists: false,
                tip: Some("abc".to_string()),
            },
            // prunable gitdir
            CoordFacts {
                branch_hits: 1,
                path: Some(PathBuf::from("/x/coord")),
                prunable: true,
                dir_exists: true,
                tip: Some("abc".to_string()),
            },
            // branch tip unresolved
            CoordFacts {
                branch_hits: 1,
                path: Some(PathBuf::from("/x/coord")),
                prunable: false,
                dir_exists: true,
                tip: None,
            },
        ];
        for facts in stale_variants {
            assert_eq!(classify_coord(facts), Err(CoordRefusal::Stale));
        }
        assert_eq!(CoordRefusal::Stale.token(), "stale");
    }

    #[test]
    fn resolve_coord_finds_the_live_coord_and_refuses_an_absent_slice() {
        let (_tmp, primary, coord, base, _bt) = primary_with_coord(199);
        // Happy: the slice resolves to its live coord worktree at the branch tip.
        assert_eq!(
            resolve_coord(&primary, 199),
            Ok(CoordTarget {
                root: coord,
                tip: base,
            })
        );
        // A slice with no worktree is unknown-slice.
        assert_eq!(
            resolve_coord(&primary, 200),
            Err(CoordRefusal::UnknownSlice)
        );
    }

    // ==================================================================================
    // §C the funnel WRITE SURFACE (PHASE-03)
    // ==================================================================================

    /// A `CoordTarget` for the funnel-tool compose cores (bypasses `resolve_coord` so a
    /// unit test can drive the compose against an explicit tip).
    fn ct(coord: &Path, tip: &str) -> CoordTarget {
        CoordTarget {
            root: coord.to_path_buf(),
            tip: tip.to_string(),
        }
    }

    /// Create a committed worker fork branch `branch` = ONE commit on top of `base`
    /// (worker author preserved), adding `file`. Working-tree-free (commit-tree over a
    /// spliced tree); the fork's objects + ref live in the SAME store the coord reads.
    fn add_fork(dir: &Path, base: &str, branch: &str, file: &str, content: &str) -> String {
        let base_tree = git_run(dir, &["rev-parse", &format!("{base}^{{tree}}")]);
        let tree = git::tree_with_file(dir, &base_tree, file, content).unwrap();
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["commit-tree", &tree, "-p", base, "-m", "worker work"])
            .env("GIT_AUTHOR_NAME", "worker-x")
            .env("GIT_AUTHOR_EMAIL", "worker-x@doctrine")
            .env("GIT_COMMITTER_NAME", "worker-x")
            .env("GIT_COMMITTER_EMAIL", "worker-x@doctrine")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "commit-tree: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let commit = String::from_utf8_lossy(&out.stdout).trim().to_string();
        git_run(
            dir,
            &["update-ref", &format!("refs/heads/{branch}"), &commit],
        );
        commit
    }

    // --- VT-1: dispatch_import (classify_import gate + working-tree-free compose) ------

    #[test]
    fn dispatch_import_happy_advances_the_coord_tip_by_one() {
        let (_tmp, _p, coord, base, _bt) = primary_with_coord(199);
        add_fork(&coord, &base, "dispatch/wk", "src/f.rs", "fn f() {}\n");
        // coord tip == base (no prior imports) ⇒ trivial compose of the worker tree.
        let out = import_compose(&ct(&coord, &base), 199, "dispatch/wk", &[]).unwrap();
        let coord_tip = match out {
            FunnelOutcome::Imported { coord_tip } => coord_tip,
            other => panic!("expected Imported, got {other:?}"),
        };
        // The coord branch advanced base → coord_tip (exactly one parent == B).
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            coord_tip
        );
        assert_ne!(coord_tip, base, "coord tip advanced");
        let parents = git_run(&coord, &["rev-list", "--parents", "-n", "1", &coord_tip]);
        let cols: Vec<&str> = parents.split_whitespace().collect();
        assert_eq!(cols.len(), 2, "exactly one parent: {parents}");
        assert_eq!(cols[1], base, "parent == coord tip (B)");
        // The worker file landed, and the worker AUTHOR is preserved (IMPORT provenance).
        let names = git_run(&coord, &["ls-tree", "-r", "--name-only", &coord_tip]);
        assert!(
            names.lines().any(|l| l == "src/f.rs"),
            "worker file imported: {names}"
        );
        let raw = git_run(&coord, &["log", "-1", "--format=%an%n%ae", &coord_tip]);
        let mut it = raw.lines();
        assert_eq!(it.next().unwrap(), "worker-x", "worker author preserved");
        assert_eq!(it.next().unwrap(), "worker-x@doctrine");
    }

    #[test]
    fn dispatch_import_undeclared_scope_refuses_before_compose_and_leaves_the_tip() {
        let (_tmp, _p, coord, base, _bt) = primary_with_coord(199);
        add_fork(&coord, &base, "dispatch/wk", "docs/readme.md", "hi\n");
        // A design-target selector that does NOT declare `docs/readme.md`.
        let selectors = vec!["src/**".to_string()];
        let out = import_compose(&ct(&coord, &base), 199, "dispatch/wk", &selectors).unwrap();
        // HARD refuse — the SAME token the CLI import's `Refusal::UndeclaredScope` uses.
        assert_eq!(
            out,
            funnel_refused(Refusal::UndeclaredScope.token(), String::new())
        );
        // Coord tip UNCHANGED — nothing landed (report-and-halt before any compose).
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            base
        );
    }

    #[test]
    fn dispatch_import_composes_a_second_disjoint_fork_via_merge_tree() {
        // coord tip past B ⇒ the 3-way merge_tree compose path (EX-1), file-disjoint forks.
        let (_tmp, _p, coord, base, _bt) = primary_with_coord(199);
        add_fork(&coord, &base, "dispatch/wk1", "src/a.rs", "a\n");
        let tip1 = match import_compose(&ct(&coord, &base), 199, "dispatch/wk1", &[]).unwrap() {
            FunnelOutcome::Imported { coord_tip } => coord_tip,
            other => panic!("expected Imported, got {other:?}"),
        };
        // A second worker forked at the SAME base B; the coord tip is now tip1 != B.
        add_fork(&coord, &base, "dispatch/wk2", "src/b.rs", "b\n");
        let tip2 = match import_compose(&ct(&coord, &tip1), 199, "dispatch/wk2", &[]).unwrap() {
            FunnelOutcome::Imported { coord_tip } => coord_tip,
            other => panic!("expected Imported, got {other:?}"),
        };
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            tip2
        );
        // BOTH disjoint deltas are present in the composed union tree.
        let names = git_run(&coord, &["ls-tree", "-r", "--name-only", &tip2]);
        assert!(
            names.lines().any(|l| l == "src/a.rs"),
            "first delta: {names}"
        );
        assert!(
            names.lines().any(|l| l == "src/b.rs"),
            "second delta: {names}"
        );
    }

    #[test]
    fn dispatch_import_unknown_slice_refuses_via_resolve_coord() {
        let (_tmp, primary, _coord, _base, _bt) = primary_with_coord(199);
        // No worktree on dispatch/200 ⇒ resolve_coord refuses unknown-slice (the public
        // tool short-circuits before touching selectors / compose).
        let out = dispatch_import(&primary, 200, "dispatch/wk").unwrap();
        assert_eq!(out, funnel_refused("unknown-slice", String::new()));
    }

    // --- A12 / VT-4: the funnel tools are transport-agnostic (design D-B5) -------------

    /// A12 / VT-4 — the funnel MCP tools are TRANSPORT-AGNOSTIC (design §B, D-B5). The
    /// public `dispatch_import(root, slice, name)` is driven by its DECLARED args ONLY:
    /// it resolves the coordination tree through `resolve_coord(root, slice)` and reads
    /// NOTHING from the process cwd, an agent_id, or a WorktreeCreate hook payload — the
    /// arm split (claude vs subprocess) lives entirely in the SPAWN seam, so a future
    /// out-of-jail transport reuses these tools unchanged.
    ///
    /// Proven BEHAVIOURALLY by triangulation: the process cwd is HELD CONSTANT (wherever
    /// the test runner runs — never any of these TempDir fixtures) across three calls
    /// that vary exactly ONE declared arg at a time, and the `resolve_coord` verdict
    /// tracks the ARGS, not the cwd. If `dispatch_import` grew a cwd / agent_id / payload
    /// dependency instead of routing on `(root, slice)`, these calls could not be told
    /// apart and one of the assertions below would fail.
    #[test]
    fn dispatch_import_is_transport_agnostic_routed_only_by_declared_args() {
        // Two independent fixtures, each under its OWN TempDir (neither is the process
        // cwd): `primary_a` carries a `dispatch/199` coord, `primary_b` a `dispatch/200`.
        let (_tmp_a, primary_a, _coord_a, _base_a, _bt_a) = primary_with_coord(199);
        let (_tmp_b, primary_b, _coord_b, base_b, _bt_b) = primary_with_coord(200);

        // (1) cwd-independence: routed only by (primary_a, 199), `resolve_coord` matches
        // the live `dispatch/199` coord at the DECLARED root and `dispatch_import`
        // advances PAST the coord seam — so it does NOT emit the `unknown-slice` refusal.
        // The process cwd holds no worktrees at all; a cwd-keyed tool would refuse here.
        let found = dispatch_import(&primary_a, 199, "dispatch/wk");
        assert!(
            !matches!(&found, Ok(o) if *o == funnel_refused("unknown-slice", String::new())),
            "resolve_coord matched the live dispatch/199 coord at the declared root, \
             not the process cwd — must not short-circuit unknown-slice: {found:?}",
        );

        // (2) root-driven routing: hold slice + name + cwd fixed, vary ONLY the `root`
        // arg. `primary_b` has no `dispatch/199` coord, so `resolve_coord(primary_b, 199)`
        // refuses `unknown-slice` — the verdict flips on the declared ROOT alone.
        assert_eq!(
            dispatch_import(&primary_b, 199, "dispatch/wk").unwrap(),
            funnel_refused("unknown-slice", String::new()),
            "the root arg keys resolve_coord — not cwd/agent_id/payload",
        );

        // (3) slice-driven routing: hold root + name + cwd fixed, vary ONLY the `slice`
        // arg. `primary_a` has no `dispatch/200` coord, so `resolve_coord(primary_a, 200)`
        // refuses `unknown-slice` — the verdict flips on the declared SLICE alone.
        assert_eq!(
            dispatch_import(&primary_a, 200, "dispatch/wk").unwrap(),
            funnel_refused("unknown-slice", String::new()),
            "the slice arg keys resolve_coord — not cwd/agent_id/payload",
        );

        // (4) no ambient cross-routing leaked: `primary_b`'s coord tip is byte-unchanged.
        assert_eq!(
            git_run(&primary_b, &["rev-parse", "refs/heads/dispatch/200"]),
            base_b,
            "sibling fixture untouched — resolution never keyed off ambient state",
        );
    }

    // --- VT-2: dispatch_conclude_phase (sheet flip + boundary commit, atomic) ---------

    #[test]
    fn conclude_boundary_commit_happy_lands_the_true_range_row() {
        let (_tmp, _p, coord, base, _bt) = primary_with_coord(199);
        // A real code range (B, code_end): advance the coord tip by importing a commit.
        add_fork(&coord, &base, "dispatch/wk", "src/f.rs", "x\n");
        let code_end = match import_compose(&ct(&coord, &base), 199, "dispatch/wk", &[]).unwrap() {
            FunnelOutcome::Imported { coord_tip } => coord_tip,
            other => panic!("{other:?}"),
        };
        let out =
            conclude_boundary_commit(&coord, &code_end, 199, "PHASE-01", &base, &code_end).unwrap();
        let tip = match out {
            FunnelOutcome::Concluded { coord_tip } => coord_tip,
            other => panic!("expected Concluded, got {other:?}"),
        };
        // Coord tip advanced by one; the boundary row is in the COMMITTED tree carrying
        // the TRUE (B, coord_tip) range.
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            tip
        );
        let committed = git_run(
            &coord,
            &[
                "show",
                &format!("{tip}:.doctrine/dispatch/199/boundaries.toml"),
            ],
        );
        let parsed = Boundaries::parse(&committed).unwrap();
        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(parsed.rows[0].phase, "PHASE-01");
        assert_eq!(parsed.rows[0].code_start_oid, base, "code_start == B");
        assert_eq!(
            parsed.rows[0].code_end_oid, code_end,
            "code_end == coord_tip"
        );
    }

    #[test]
    fn conclude_boundary_commit_upserts_by_phase_never_duplicates() {
        let (_tmp, _p, coord, base, _bt) = primary_with_coord(199);
        // First conclude of PHASE-01 (an empty-code range B..B is still a real row).
        let tip1 =
            match conclude_boundary_commit(&coord, &base, 199, "PHASE-01", &base, &base).unwrap() {
                FunnelOutcome::Concluded { coord_tip } => coord_tip,
                other => panic!("{other:?}"),
            };
        // Re-conclude the SAME phase with a wider range ⇒ UPSERT, never a duplicate row.
        let tip2 =
            match conclude_boundary_commit(&coord, &tip1, 199, "PHASE-01", &base, &tip1).unwrap() {
                FunnelOutcome::Concluded { coord_tip } => coord_tip,
                other => panic!("{other:?}"),
            };
        let committed = git_run(
            &coord,
            &[
                "show",
                &format!("{tip2}:.doctrine/dispatch/199/boundaries.toml"),
            ],
        );
        let parsed = Boundaries::parse(&committed).unwrap();
        assert_eq!(parsed.rows.len(), 1, "upsert replaces, never appends");
        assert_eq!(parsed.rows[0].code_end_oid, tip1, "the row was updated");
    }

    #[test]
    fn conclude_boundary_commit_fault_lands_no_boundary_and_leaves_porcelain_clean() {
        let (_tmp, _p, coord, base, base_tree) = primary_with_coord(199);
        // Inject a fault at the ref-update step: move the coord branch to a same-tree
        // dangling sibling so a conclude with the STALE tip (=base) CAS-faults AFTER the
        // working-tree-free write-tree/commit_tree — the boundary is composed, never landed.
        let other = git_run(
            &coord,
            &["commit-tree", &base_tree, "-p", &base, "-m", "other"],
        );
        git_run(&coord, &["update-ref", "refs/heads/dispatch/199", &other]);
        let status_before = git_run(&coord, &["status", "--porcelain"]);
        let out = conclude_boundary_commit(&coord, &base, 199, "PHASE-01", &base, &base).unwrap();
        assert_eq!(out, funnel_refused("lost-ref-race", String::new()));
        // The ref is untouched (still the racing commit); NO boundary landed there; and
        // `git status --porcelain` is byte-identical (live index + worktree untouched).
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            other
        );
        assert!(
            git::read_path_at(&coord, &other, ".doctrine/dispatch/199/boundaries.toml")
                .unwrap()
                .is_none(),
            "no boundary committed on the fault"
        );
        assert_eq!(
            git_run(&coord, &["status", "--porcelain"]),
            status_before,
            "porcelain byte-unchanged"
        );
    }

    #[test]
    fn dispatch_conclude_phase_flips_the_gitignored_sheet_and_lands_the_boundary() {
        let (_tmp, primary, coord, base, _bt) = primary_with_coord(199);
        // Pre-create the phase sheet (runtime tier) `set_phase_status` reopens+flips.
        let phases_dir = crate::state::phases_dir(&coord, 199);
        fs::create_dir_all(&phases_dir).unwrap();
        fs::write(
            phases_dir.join("phase-01.toml"),
            "status = \"in_progress\"\n",
        )
        .unwrap();
        // A real (B, code_end) range from an imported code commit.
        add_fork(&coord, &base, "dispatch/wk", "src/f.rs", "x\n");
        let code_end = match import_compose(&ct(&coord, &base), 199, "dispatch/wk", &[]).unwrap() {
            FunnelOutcome::Imported { coord_tip } => coord_tip,
            other => panic!("{other:?}"),
        };
        let out =
            dispatch_conclude_phase(&primary, 199, "PHASE-01", &base, &code_end, Some("done"))
                .unwrap();
        let tip = match out {
            FunnelOutcome::Concluded { coord_tip } => coord_tip,
            other => panic!("expected Concluded, got {other:?}"),
        };
        // (b) durable tier: the boundary landed in COMMITTED history.
        let committed = git_run(
            &coord,
            &[
                "show",
                &format!("{tip}:.doctrine/dispatch/199/boundaries.toml"),
            ],
        );
        assert!(
            committed.contains("PHASE-01"),
            "boundary committed: {committed}"
        );
        // (a) runtime tier: the GITIGNORED sheet flipped to completed, and it is NOT in
        // committed history (the flip is disposable, never folded into the commit).
        assert_eq!(
            crate::state::read_phase_status(&phases_dir, "phase-01").unwrap(),
            Some("completed".to_string()),
            "sheet flipped to completed"
        );
        let committed_names = git_run(&coord, &["ls-tree", "-r", "--name-only", &tip]);
        assert!(
            !committed_names
                .lines()
                .any(|l| l.starts_with(".doctrine/state/")),
            "the phase sheet never enters committed history"
        );
    }

    #[test]
    fn dispatch_conclude_phase_mirrors_the_completed_flip_into_the_primary_tree() {
        // IMP-272: prepare-review's completeness gate reads the completed-phase set
        // from the PRIMARY tree, but the claude-arm orchestrator drives from the coord
        // tree — so the flip must reach BOTH trees or the gate refuses a landed row as
        // "not a completed phase". The coord flip stays (load-bearing for next-ready).
        let (_tmp, primary, coord, base, _bt) = primary_with_coord(199);
        // Sheets materialised in BOTH trees: plan materialises the primary; dispatch
        // setup materialises the coord.
        for root in [&coord, &primary] {
            let dir = crate::state::phases_dir(root, 199);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("phase-01.toml"), "status = \"in_progress\"\n").unwrap();
        }
        // Drive from the PRIMARY root, as the MCP tool is invoked; the coord is resolved
        // server-side. An empty (B, B) range is a real row (upsert test proves it).
        dispatch_conclude_phase(&primary, 199, "PHASE-01", &base, &base, Some("done")).unwrap();
        // BOTH trees read completed — coord (next-ready authority) AND primary (the
        // completeness gate's completed-set source).
        assert_eq!(
            crate::state::read_phase_status(&crate::state::phases_dir(&coord, 199), "phase-01")
                .unwrap(),
            Some("completed".to_string()),
            "coord sheet flipped",
        );
        assert_eq!(
            crate::state::read_phase_status(&crate::state::phases_dir(&primary, 199), "phase-01")
                .unwrap(),
            Some("completed".to_string()),
            "primary sheet mirrored (IMP-272)",
        );
    }

    // --- VT-3: dispatch_reap (run_gc landed-oracle, UNCHANGED) -------------------------

    #[test]
    fn dispatch_reap_reaps_a_landed_fork_and_refuses_an_unlanded_one() {
        let (_tmp, primary, coord, base, _bt) = primary_with_coord(199);
        // A LANDED fork: its patch is in coord history (imported), so `git cherry` is
        // all-`-` ⇒ the oracle certifies it landed ⇒ reap the worktree + branch.
        let landed_branch = "dispatch/landed";
        add_fork(&coord, &base, landed_branch, "src/landed.rs", "l\n");
        // Give the fork a live linked worktree so there is a worktree to reap.
        let landed_wt = coord.join(".worktrees").join("landed");
        git_run(
            &coord,
            &[
                "worktree",
                "add",
                "-q",
                landed_wt.to_str().unwrap(),
                landed_branch,
            ],
        );
        // Land the fork's patch onto the coord branch (import advances the coord tip).
        import_compose(&ct(&coord, &base), 199, landed_branch, &[]).unwrap();

        let out = dispatch_reap(&primary, 199, landed_branch).unwrap();
        assert_eq!(
            out,
            FunnelOutcome::Reaped {
                fork: landed_branch.to_string()
            }
        );
        assert!(
            git::git_opt(&coord, &["rev-parse", "--verify", "--quiet", landed_branch])
                .unwrap()
                .is_none(),
            "the landed fork branch was reaped"
        );

        // An UNLANDED fork: its patch is NOT in coord history (`git cherry` has a `+`) ⇒
        // the shared oracle REFUSES (run_gc bails); dispatch_reap propagates the Err and
        // the branch survives.
        let unlanded_branch = "dispatch/unlanded";
        add_fork(&coord, &base, unlanded_branch, "src/unlanded.rs", "u\n");
        let err = dispatch_reap(&primary, 199, unlanded_branch).unwrap_err();
        assert!(
            err.to_string().contains("not-landed"),
            "gc refuses an unlanded fork: {err}"
        );
        assert!(
            git::git_opt(
                &coord,
                &["rev-parse", "--verify", "--quiet", unlanded_branch]
            )
            .unwrap()
            .is_some(),
            "the unlanded fork branch survives the refusal"
        );
    }

    #[test]
    fn dispatch_reap_unknown_slice_refuses_via_resolve_coord() {
        let (_tmp, primary, _coord, _base, _bt) = primary_with_coord(199);
        let out = dispatch_reap(&primary, 200, "dispatch/wk").unwrap();
        assert_eq!(out, funnel_refused("unknown-slice", String::new()));
    }

    // ==================================================================================
    // §D the funnel READ SURFACE (SL-206 PHASE-03)
    // ==================================================================================

    /// Write a disposable runtime phase sheet (`status = "<status>"`) for `phase` under
    /// the coord's gitignored state tree — the tier `read_phase_status` reads.
    fn write_phase_sheet(coord: &Path, slice: u32, phase: &str, status: &str) {
        let dir = crate::state::phases_dir(coord, slice);
        fs::create_dir_all(&dir).unwrap();
        let stem = phase.to_lowercase();
        fs::write(
            dir.join(format!("{stem}.toml")),
            format!("status = \"{status}\"\n"),
        )
        .unwrap();
    }

    // --- VT-1: dispatch_phase_receipt --------------------------------------------------

    #[test]
    fn dispatch_phase_receipt_resolved_carries_real_tip_distinct_from_code_end() {
        let (_tmp, primary, coord, base, _bt) = primary_with_coord(206);
        // A committed boundary row for PHASE-01 (code_end = base, DISTINCT from the tip
        // we advance to by committing the ledger).
        fs::create_dir_all(coord.join(".doctrine/dispatch/206")).unwrap();
        fs::write(
            coord.join(".doctrine/dispatch/206/boundaries.toml"),
            format!(
                "[[boundary]]\nphase = \"PHASE-01\"\ncode_start_oid = \"{base}\"\ncode_end_oid = \"{base}\"\n"
            ),
        )
        .unwrap();
        git_run(&coord, &["add", ".doctrine/dispatch/206/boundaries.toml"]);
        git_run(&coord, &["commit", "-q", "-m", "boundary"]);
        let tip = git_run(&coord, &["rev-parse", "refs/heads/dispatch/206"]);
        // A completed phase sheet in the coord's disposable runtime state (written after
        // the commit — it is not part of committed history).
        write_phase_sheet(&coord, 206, "PHASE-01", "completed");

        match dispatch_phase_receipt(&primary, 206, "PHASE-01").unwrap() {
            ReadOutcome::Resolved(core) => {
                assert_eq!(core.dispatch_tip, tip, "carries the REAL live coord tip");
                assert_eq!(
                    core.code_end.as_deref(),
                    Some(base.as_str()),
                    "the committed boundary's code_end"
                );
                assert_ne!(
                    core.dispatch_tip,
                    core.code_end.clone().unwrap(),
                    "the live dispatch_tip is DISTINCT from the boundary code_end (EX-1)"
                );
                // Sheet "completed" + a committed boundary ⇒ boundary-backed Completed.
                assert_eq!(core.status, "completed");
            }
            other => panic!("expected Resolved, got {other:?}"),
        }
    }

    #[test]
    fn dispatch_phase_receipt_no_boundary_omits_oids_and_reflects_sheet() {
        let (_tmp, primary, coord, _base, _bt) = primary_with_coord(206);
        // Sheet says completed, but NO committed boundary backs it ⇒ conclude-incomplete
        // (the gap the ReceiptStatus enum surfaces), and no code oids are fabricated.
        write_phase_sheet(&coord, 206, "PHASE-02", "completed");
        match dispatch_phase_receipt(&primary, 206, "PHASE-02").unwrap() {
            ReadOutcome::Resolved(core) => {
                assert_eq!(core.status, "conclude-incomplete");
                assert!(core.code_start.is_none() && core.code_end.is_none());
            }
            other => panic!("expected Resolved, got {other:?}"),
        }
    }

    #[test]
    fn read_tools_refuse_unknown_slice_with_no_fabricated_tip() {
        let (_tmp, primary, _coord, _base, _bt) = primary_with_coord(206);
        // slice 207 has no live coord ⇒ EVERY read tool short-circuits to CoordRefused
        // carrying the resolve_coord token VERBATIM, with no Resolved payload / tip.
        assert_eq!(
            dispatch_phase_receipt(&primary, 207, "PHASE-01").unwrap(),
            ReadOutcome::CoordRefused {
                reason: "unknown-slice".to_string()
            }
        );
        assert_eq!(
            dispatch_next_ready(&primary, 207).unwrap(),
            ReadOutcome::CoordRefused {
                reason: "unknown-slice".to_string()
            }
        );
        assert_eq!(
            dispatch_authored_divergence(&primary, 207).unwrap(),
            ReadOutcome::CoordRefused {
                reason: "unknown-slice".to_string()
            }
        );
    }

    // --- VT-2: dispatch_next_ready wraps compute_next_phases verbatim ------------------

    #[test]
    fn dispatch_next_ready_agrees_with_compute_next_phases() {
        let (_tmp, primary, coord, _base, _bt) = primary_with_coord(206);
        // A three-phase plan on disk in the coord (read_plan reads the working tree).
        fs::create_dir_all(coord.join(".doctrine/slice/206")).unwrap();
        fs::write(
            coord.join(".doctrine/slice/206/plan.toml"),
            "[[phase]]\nid = \"PHASE-01\"\nname = \"one\"\n\n\
             [[phase]]\nid = \"PHASE-02\"\nname = \"two\"\n\n\
             [[phase]]\nid = \"PHASE-03\"\nname = \"three\"\n",
        )
        .unwrap();
        // PHASE-01 completed; 02/03 pending (absent sheet ⇒ pending).
        write_phase_sheet(&coord, 206, "PHASE-01", "completed");

        // The shared readiness authority, computed directly over the same rows.
        let rows = crate::dispatch::plan_next_rows(&coord, 206).unwrap();
        let expected = crate::dispatch::compute_next_phases(&rows);
        assert_eq!(expected, vec!["PHASE-02", "PHASE-03"], "fixture sanity");

        match dispatch_next_ready(&primary, 206).unwrap() {
            ReadOutcome::Resolved(core) => {
                assert_eq!(core.next, expected, "next == compute_next_phases VERBATIM");
                let ids: Vec<&str> = core.phases.iter().map(|p| p.id.as_str()).collect();
                assert_eq!(ids, vec!["PHASE-01", "PHASE-02", "PHASE-03"]);
                assert_eq!(core.phases[0].status, "completed");
                assert_eq!(core.phases[1].status, "pending");
            }
            other => panic!("expected Resolved, got {other:?}"),
        }
    }

    // --- VT-3: dispatch_authored_divergence -------------------------------------------

    #[test]
    fn dispatch_authored_divergence_true_iff_doctrine_differs_trunk_to_tip() {
        let (_tmp, primary, coord, base, _bt) = primary_with_coord(206);
        // Clean coord at base: trunk (main) == coord tip == base ⇒ no `.doctrine/**`
        // divergence, and compared_ref is the RESOLVED trunk (main tip), not a hardcode.
        match dispatch_authored_divergence(&primary, 206).unwrap() {
            ReadOutcome::Resolved(core) => {
                assert!(!core.diverged, "clean coord has no divergence: {core:?}");
                assert_eq!(
                    core.compared_ref, base,
                    "compared_ref = git::trunk_commit (the resolved main tip)"
                );
                assert!(core.drifted_paths.is_empty());
            }
            other => panic!("expected Resolved, got {other:?}"),
        }

        // Commit a `.doctrine/**` change on the dispatch branch — the coord tip advances
        // past trunk; the authored subtree now diverges over trunk_ref..dispatch_tip.
        fs::create_dir_all(coord.join(".doctrine/slice/206")).unwrap();
        fs::write(coord.join(".doctrine/slice/206/notes.md"), "drift\n").unwrap();
        git_run(&coord, &["add", ".doctrine/slice/206/notes.md"]);
        git_run(&coord, &["commit", "-q", "-m", "authored drift"]);

        match dispatch_authored_divergence(&primary, 206).unwrap() {
            ReadOutcome::Resolved(core) => {
                assert!(
                    core.diverged,
                    "authored `.doctrine` change diverges: {core:?}"
                );
                assert_eq!(
                    core.compared_ref, base,
                    "still compared to the RESOLVED trunk, never edge/hardcode"
                );
                assert!(
                    core.drifted_paths
                        .iter()
                        .any(|p| p == ".doctrine/slice/206/notes.md"),
                    "the drifted authored path is reported: {:?}",
                    core.drifted_paths
                );
            }
            other => panic!("expected Resolved, got {other:?}"),
        }
    }
}
