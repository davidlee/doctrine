// SPDX-License-Identifier: GPL-3.0-only
//! §B transactional core + the dispatch funnel's WRITE SURFACE (SL-199 PHASE-02/03,
//! design §B). PHASE-02 built the two INTERNAL primitives; PHASE-03 wires the three
//! MCP tools (`dispatch_import` / `dispatch_conclude_phase` / `dispatch_reap`) onto
//! them, each resolving the coord tree server-side by slice-id and composing an
//! EXISTING belt (`classify_import` / the gc machine) with the primitives below — one seam,
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
use crate::dispatch::funnel::LandingVerdict;
use crate::dispatch::{
    CommitOutcome, Identity, Provenance, VerifyOutcome, dispatch_identity, dispatch_ref, funnel,
    funnel_message, land_boundary_row, land_conclude, verify_phase,
};
use crate::funnel_machine::{self, PhaseRow, Position, Transition, TransitionFacts};
use crate::git::{self, WorktreeRecord};
use crate::ledger::Boundaries;
use crate::worktree::{
    Apply, ForkExpect, GcOutcome, GcRefusal, classify_import, reap_fork, require_binding,
    resolve_agent,
};
use anyhow::Context;
use serde::Serialize;
use std::io::Write as _;
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
//   * `dispatch_reap`           — [`reap_fork`] (the shared gc machine's typed entry
//                                 point) over [`funnel::resolve_landing`]'s proof

/// The three write-surface tool names (STD-001 single-source — PHASE-04's lint
/// references these, never a bare literal). They are DEFINED in `crate::dispatch` (the
/// engine, the funnel authority home) because the SL-228 PHASE-06 `next` oracle renders
/// them into runnable `command` literals and an engine module may not import the command
/// tier (ADR-001 — the layering gate flags the upward edge). Re-exported here so the
/// registry, the lint, and the oracle all spell each tool exactly once.
pub(crate) use crate::dispatch::{
    TOOL_DISPATCH_CONCLUDE_PHASE, TOOL_DISPATCH_IMPORT, TOOL_DISPATCH_REAP,
};
/// The tool name the MCP registry keys `dispatch_verify` on (STD-001) — SL-228
/// PHASE-05, the funnel's evidence-producing write verb.
pub(crate) const TOOL_DISPATCH_VERIFY: &str = "dispatch_verify";

/// Refuse: `dispatch_import`'s working-tree-free [`git::merge_tree`] compose of
/// coord-tip ⊕ worker-tip hit a content conflict. File-disjoint dispatch makes this
/// unreachable in the happy topology; kept as a fail-closed token rather than a silent
/// bad tree (design D-B4).
const MERGE_CONFLICT: &str = "merge-conflict";

/// Refuse: the gc act ran but left CRITICAL residue behind (the fork's worktree and/or
/// branch survived), so the fork is NOT gone and `reaped` would be a lie. Mirrors the
/// CLI gc's own `gc-incomplete` bail token (SL-228 PHASE-08, ISS-246).
const GC_INCOMPLETE: &str = "gc-incomplete";

/// Refuse: an active claimant holds the fork's claim lock (a spawn is mid
/// claim→bind→act), so nothing was classified and nothing deleted. Retry after the
/// claimant finishes — the reap is idempotent.
const CLAIM_BUSY: &str = "claim-busy";

/// Refuse: TWO OR MORE funnel rows record this same fork branch, so no single
/// `import.fork_tip` speaks for it and no landing can be proven. Fails closed BEFORE
/// any act rather than binding the first row.
const AMBIGUOUS_FORK_ROW: &str = "ambiguous-fork-row";

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
    /// `dispatch_reap` reaped the (landed) fork worktree + branch AND landed the
    /// `Reap` row — position stands at `reaped`.
    Reaped { fork: String },
    /// `dispatch_reap` reaped the fork, but the Class-2 `Reap` row did NOT land (a lost
    /// CAS race / a write fault). BOTH halves of that are true and both are said: the
    /// fork IS gone (claiming otherwise would lie) and the row is PENDING, so
    /// re-driving `dispatch_reap` completes it (the replay rule makes that safe).
    /// A success outcome may never imply a landed row — that is why this variant
    /// exists rather than a bare `Reaped` with a stderr warning the driver never reads.
    ReapedRowPending { fork: String, detail: String },
    /// `dispatch_verify` ran the suite, it PASSED, and pass evidence landed;
    /// `coord_tip` is the resulting tip (unchanged on an idempotent replay).
    Verified { coord_tip: String, suite: String },
    /// `dispatch_verify` ran the suite and it did NOT pass (or it mutated the tree).
    /// FAIL evidence IS landed — the phase keeps its position and prescribes triage.
    VerifyFailed { suite: String, detail: String },
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
/// # The one-commit HEAL-FORWARD (SL-228 PHASE-05, T6/D6)
///
/// Import is the funnel's catch-up point. `worker_commit` records its own row Class-2
/// (strictly AFTER the fork ref advanced), and `create-fork`'s `Spawn` row is
/// non-fatal, so the row can legitimately LAG the git reality by up to two milestones.
/// Import heals the whole provable prefix — `[Spawn, RecordWorkerCommit, Import]` —
/// in ONE splice over its own composed merge tree and ONE commit, so a kill mid-heal
/// lands NOTHING and position can never durably rest at `worker-committed` via any
/// import path.
///
/// The phase comes from the DURABLE FORK BINDING (`resolve_agent` → `require_binding`
/// — the very record PHASE-04 added `slice`/`phase` for), never from a caller arg:
/// `dispatch_import{slice, name}` carries no phase and the funnel row is phase-keyed.
/// An unbound fork cannot prove which row its commit belongs to, so it refuses
/// (`unprovable-fork`) exactly as `worker_commit` does — there is no guess arm.
pub(crate) fn dispatch_import(
    root: &Path,
    slice: u32,
    name: &str,
) -> anyhow::Result<FunnelOutcome> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
    };
    // D6 — the durable fork binding names the funnel row. One record read, the same
    // seam `worker_commit` uses; no second fork→phase derivation to drift.
    let fork = match resolve_agent(&coord.root, fork_agent(name), ForkExpect::Advanced)
        .and_then(|record| require_binding(&record).map(|binding| (record, binding)))
    {
        Ok((record, binding)) if binding.slice == slice => (record, binding),
        Ok((_, binding)) => {
            return Ok(funnel_refused(
                crate::worktree::ResolveRefusal::UnprovableFork.token(),
                format!(
                    "{name} is bound to SL-{:03}, not the SL-{slice:03} funnel",
                    binding.slice
                ),
            ));
        }
        Err(refusal) => return Ok(funnel_refused(refusal.token(), name.to_owned())),
    };
    let (record, binding) = fork;

    let fork_tip = git::git_text(&coord.root, &["rev-parse", &format!("{name}^{{commit}}")])
        .with_context(|| format!("resolve fork tip {name}"))?;
    let funnel_record = funnel::read_funnel_at(&coord.root, &coord.tip, slice)?;
    let stored = funnel_record.row(&binding.phase).cloned();
    let ts = import_ladder(
        stored.as_ref().map(|r| r.position),
        &record.branch,
        &record.base,
        &fork_tip,
        &coord.tip,
    );
    let at = crate::clock::now_timestamp()?;

    // The GATE, one step after `resolve_coord` and BEFORE the belts (design §4). The
    // pure fold is the same authority the sole writer applies at landing — asking it
    // here just moves the verdict ahead of the compose, so an illegal import composes
    // nothing at all.
    let fold = match funnel_machine::fold_transitions(
        stored.as_ref(),
        &binding.phase,
        &ts,
        &coord.tip,
        None,
        &at,
    ) {
        Ok(Some(fold)) => fold,
        // `ts` is never empty by construction (`import_ladder` always yields Import).
        Ok(None) => return Ok(funnel_refused(MISSING_LADDER, name.to_owned())),
        Err(illegal) => return Ok(funnel_refused(illegal.reason, illegal.to_string())),
    };
    if fold.replay {
        // The LOST-RESPONSE retry: this exact fork tip is already imported. Nothing to
        // compose, nothing to land — just name where the coord stands. `onto` is
        // excluded from replay identity, so a freshly-resolved tip cannot defeat this.
        return Ok(FunnelOutcome::Imported {
            coord_tip: coord.tip,
        });
    }

    // The slice's design-target selectors gate the scope belt (strict, HARD — a read
    // failure propagates rather than silently disabling the gate).
    let selectors = crate::slice::selectors(
        &coord.root,
        slice,
        Some(crate::slice::SelectorIntent::DesignTarget),
    )?;
    let plan = match import_plan(&coord, slice, name, &selectors)? {
        Composed::Refused(outcome) => return Ok(outcome),
        Composed::Ready(plan) => plan,
    };

    // ONE splice over the composed merge tree, ONE commit: the worker delta ⊕ the
    // healed funnel prefix. IMPORT provenance keeps the worker author.
    match funnel::land_funnel_transitions(
        &coord.root,
        &dispatch_ref(slice),
        &coord.tip,
        slice,
        &binding.phase,
        &ts,
        None,
        Some(&plan.tree),
        &import_message(slice, name),
        &at,
        &plan.prov,
    )? {
        funnel::FunnelLanding::Landed { oid, .. } => Ok(FunnelOutcome::Imported { coord_tip: oid }),
        funnel::FunnelLanding::Replayed { .. } => Ok(FunnelOutcome::Imported {
            coord_tip: coord.tip.clone(),
        }),
        funnel::FunnelLanding::Illegal(illegal) => {
            Ok(funnel_refused(illegal.reason, illegal.to_string()))
        }
        funnel::FunnelLanding::CommitRefused(refusal) => {
            Ok(funnel_refused(refusal.token(), String::new()))
        }
    }
}

/// Refuse (defensive): a transition ladder came back empty, which `import_ladder`
/// cannot produce. Named rather than silently ignored (STD-001).
const MISSING_LADDER: &str = "missing-ladder";

/// The bare agent id inside a fork branch name (`dispatch/<agent>` → `<agent>`).
/// `resolve_agent` takes the OPAQUE id and re-derives the branch itself.
fn fork_agent(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

/// The commit subject an import lands under — names the fork, so the history stays
/// greppable per worker (STD-001: one spelling, shared by the gated path and the
/// compose driver).
fn import_message(slice: u32, name: &str) -> String {
    funnel_message(slice, &format!("import {name}"))
}

/// The provable heal-forward prefix for a fork at `position` (T6). Position-KEYED,
/// not "always the full prefix": once a milestone is passed, re-attempting it is not
/// a replay but an `already-<position>` refusal, so the ladder must start where the
/// row actually stands. `Imported`+ yields the bare `[Import]`, which the machine
/// resolves to either a replay (matching `fork_tip`) or `already-imported`.
fn import_ladder(
    position: Option<Position>,
    fork: &str,
    base_oid: &str,
    fork_tip: &str,
    onto: &str,
) -> Vec<Transition> {
    let worker_commit = Transition::RecordWorkerCommit {
        fork_tip: fork_tip.to_owned(),
    };
    let import = Transition::Import {
        fork_tip: fork_tip.to_owned(),
        onto: onto.to_owned(),
    };
    match position {
        None => vec![
            Transition::Spawn {
                fork: fork.to_owned(),
                base_oid: base_oid.to_owned(),
            },
            worker_commit,
            import,
        ],
        Some(Position::Spawned) => vec![worker_commit, import],
        Some(_) => vec![import],
    }
}

/// The working-tree-free import payload: the composed union tree and the provenance
/// the commit carries.
struct ImportPlan {
    tree: String,
    prov: Provenance,
}

/// [`import_plan`]'s two arms — a composed payload, or a belt refusal to surface
/// verbatim.
enum Composed {
    Ready(ImportPlan),
    Refused(FunnelOutcome),
}

/// The compose core of [`dispatch_import`] (selectors already resolved) — the pure belt
/// then a working-tree-free compose, factored so a unit test drives it with
/// explicit selectors without standing up a full authored slice, and so the gated
/// path can splice the SAME tree under the funnel record (D1) instead of committing
/// it directly.
fn import_plan(
    coord: &CoordTarget,
    slice: u32,
    name: &str,
    selectors: &[String],
) -> anyhow::Result<Composed> {
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
        // The scope refusal alone carries a runnable `Refused.detail` naming the offending
        // paths (SL-224 PHASE-01); every other refusal yields an empty detail. Both the
        // token and the detail are computed on the refusal VALUE (`Refusal` is a
        // `#[cfg(test)]`-only re-export, never named in prod) — `scope_detail` recomputes
        // the undeclared set from the SAME pure predicate the belt used.
        Err(refusal) => {
            return Ok(Composed::Refused(funnel_refused(
                refusal.token(),
                refusal.scope_detail(slice, selectors, &delta_paths),
            )));
        }
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
            git::MergeTree::Conflict { .. } => {
                return Ok(Composed::Refused(funnel_refused(
                    MERGE_CONFLICT,
                    String::new(),
                )));
            }
        }
    };

    // IMPORT provenance keeps the worker author, stamps the dispatch committer.
    Ok(Composed::Ready(ImportPlan {
        tree,
        prov: Provenance::Import {
            author: commit_author(&coord.root, &fork_tip)?,
            committer: dispatch_identity(),
        },
    }))
}

/// The UNGATED compose-and-land driver: [`import_plan`] plus ONE `commit_on_behalf`
/// onto the coord tip, with NO funnel record. This is the pre-funnel landing shape —
/// production now routes every import through [`dispatch_import`]'s gated,
/// heal-forward, one-commit path (D6), so the only remaining driver is the belt /
/// compose / provenance test bed, which exercises `import_plan` (the real belt and
/// the real compose) without standing up a fork worktree, a dispatch record, and a
/// funnel row for each case.
#[cfg(test)]
fn import_compose(
    coord: &CoordTarget,
    slice: u32,
    name: &str,
    selectors: &[String],
) -> anyhow::Result<FunnelOutcome> {
    let plan = match import_plan(coord, slice, name, selectors)? {
        Composed::Refused(outcome) => return Ok(outcome),
        Composed::Ready(plan) => plan,
    };
    match crate::dispatch::commit_on_behalf(
        &coord.root,
        &dispatch_ref(slice),
        &coord.tip,
        &plan.tree,
        &import_message(slice, name),
        &plan.prov,
    )? {
        CommitOutcome::Landed { oid } => Ok(FunnelOutcome::Imported { coord_tip: oid }),
        CommitOutcome::Refused(refusal) => Ok(funnel_refused(refusal.token(), String::new())),
    }
}

// --- T2: dispatch_conclude_phase -------------------------------------------------------

/// `dispatch_conclude_phase{slice, phase, code_start, code_end, note?}` — conclude a
/// phase across the durable and disposable tiers, in THAT ORDER (SL-228 PHASE-05, T3):
///
/// 1. the DURABLE commit — see the two routes below;
/// 2. THEN [`crate::state::set_phase_status`] flips the GITIGNORED phase sheet to
///    `completed`, as a trailing PROJECTION of what already landed.
///
/// **Why the commit goes first.** A crash between the two now leaves the sheet BEHIND
/// the durable authority, and the §9 receipt matrix reads position ⊕ boundary as the
/// truth — so that window still reports `Completed`. The old order left the sheet
/// AHEAD of the authority, which is exactly the `conclude-incomplete` alarm. The
/// disposable tier may lag the committed one; it must never lead it.
///
/// **Two routes, one authority (D3's rule, generalised).** A phase with a FUNNEL ROW
/// is funnel-managed: the `Conclude` transition is gated by the machine and lands
/// `boundaries.toml ⊕ funnel.toml` in ONE CAS commit, so the boundary and the position
/// that authorises it can never disagree. A phase with NO row is solo / pre-funnel /
/// legacy: routing it through the machine could only refuse it `not-spawned`, so it
/// keeps today's boundary-only commit, unchanged. Neither route bypasses anything —
/// the second has no funnel authority to bypass.
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
    let now = crate::clock::now_timestamp()?;
    let funnel_record = funnel::read_funnel_at(&coord.root, &coord.tip, slice)?;
    let landed = match funnel_record.row(phase) {
        Some(row) => conclude_funnel_commit(&coord, slice, phase, code_start, code_end, row, &now)?,
        None => {
            conclude_boundary_commit(&coord.root, &coord.tip, slice, phase, code_start, code_end)?
        }
    };
    // The trailing projection — only for a conclusion that actually landed, so a
    // refusal can never leave the sheet claiming a completion nothing backs. The coord
    // flip auto-mirrors into the PRIMARY tree (ISS-212): `set_phase_status` is the
    // single writer and owns the mirror, so `prepare-review`'s completeness gate —
    // which reads the completed set from the primary sheet — sees this phase without a
    // per-call-site block here. Called directly (not via `run_phase`) so no CLI-shell
    // `stdout` print pollutes the MCP JSON-RPC channel.
    if matches!(landed, FunnelOutcome::Concluded { .. }) {
        crate::state::set_phase_status(
            &coord.root,
            slice,
            phase,
            crate::state::PhaseStatus::Completed,
            note,
            &now,
        )?;
    }
    Ok(landed)
}

/// The FUNNEL-MANAGED conclude route (T3/T4): gate the `Conclude` transition against
/// the stored row, then land the boundary row and the position in ONE CAS commit by
/// composing `boundaries.toml` into the tip's tree and handing THAT tree to the sole
/// writer as its splice base (D1).
///
/// The gate's `paths_since_verify` is the T4 supply: the paths changed between the
/// stored `verified_oid` and the live tip. With NO stored evidence there is nothing to
/// diff, so `None` is passed and the machine fails closed (`conclude-unverified` /
/// `conclude-verify-stale`) rather than assuming identity.
fn conclude_funnel_commit(
    coord: &CoordTarget,
    slice: u32,
    phase: &str,
    code_start: &str,
    code_end: &str,
    row: &PhaseRow,
    now: &str,
) -> anyhow::Result<FunnelOutcome> {
    match land_conclude(
        &coord.root,
        &coord.tip,
        slice,
        row,
        boundary_row(phase, code_start, code_end),
        now,
    )? {
        funnel::FunnelLanding::Landed { oid, .. } => {
            Ok(FunnelOutcome::Concluded { coord_tip: oid })
        }
        funnel::FunnelLanding::Replayed { .. } => Ok(FunnelOutcome::Concluded {
            coord_tip: coord.tip.clone(),
        }),
        funnel::FunnelLanding::Illegal(illegal) => {
            Ok(funnel_refused(illegal.reason, illegal.to_string()))
        }
        funnel::FunnelLanding::CommitRefused(refusal) => {
            Ok(funnel_refused(refusal.token(), String::new()))
        }
    }
}

/// The `(code_start, code_end)` boundary row a conclude records — ONE spelling shared
/// by both conclude routes.
fn boundary_row(phase: &str, code_start: &str, code_end: &str) -> BoundaryRow {
    BoundaryRow {
        phase: phase.to_owned(),
        code_start_oid: code_start.to_owned(),
        code_end_oid: code_end.to_owned(),
        provenance: BoundaryProvenance::Funnel,
    }
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
    let row = boundary_row(phase, code_start, code_end);
    let prov = Provenance::Conclude {
        who: dispatch_identity(),
    };
    match land_boundary_row(coord_root, &dispatch_ref(slice), tip, slice, row, &prov)? {
        CommitOutcome::Landed { oid } => Ok(FunnelOutcome::Concluded { coord_tip: oid }),
        CommitOutcome::Refused(refusal) => Ok(funnel_refused(refusal.token(), String::new())),
    }
}

// --- T3: dispatch_reap -----------------------------------------------------------------

/// `dispatch_reap{slice, name}` — reap a spent worker fork through the shared gc
/// machine's TYPED entry point [`reap_fork`], and record the `Reap` milestone.
///
/// # The landing proof is the funnel record, not git archaeology (ISS-245)
///
/// The import lands the worker delta ⊕ the funnel row in ONE commit, so the landing
/// commit's patch is a strict SUPERSET of the fork's and `git cherry` matches no
/// patch-id: the shared oracle reports EVERY funnel-managed fork unlanded. Reap was
/// therefore unreachable on its own prescribed path, and the operator learned a
/// `--force` reflex that defeats the gate wholesale.
///
/// So for a funnel-managed fork the proof is [`funnel::resolve_landing`]'s three-check
/// conjunction (exactly one row names the branch ⊕ that row is at `concluded`/`reaped`
/// ⊕ the live branch OID still equals its `import.fork_tip`), injected into gc as an
/// already-proven FACT. It is NOT "the transition gate passed" and NOT `row.position`
/// alone: a branch advanced past the imported commit carries work nothing certified,
/// and `Ambiguous` refuses before any act. Any check failing injects NO fact and the
/// `git cherry` oracle decides, unchanged — fail-closed. A fork with NO funnel row is
/// not funnel-managed (solo / pre-funnel / legacy): there is no funnel authority to
/// read, so the oracle is its only gate, exactly as before.
///
/// NOTE (SL-228 PHASE-08, R3): after this change the CLI `worktree gc` alone can
/// DISAGREE with this verb on a funnel-managed fork — it takes no `--slice` and derives
/// none, so it cannot read the record (D-P8-2). Deliberate; the shared `not-landed`
/// remedy signposts this verb.
///
/// # Every actionable verdict is a structured refusal (ISS-246)
///
/// [`reap_fork`] returns a typed [`GcOutcome`] and writes NOTHING (D-P8-7: the CLI gc's
/// report goes to process stdout, which for this server IS the JSON-RPC wire), so an
/// unlanded fork, a busy claim and leftover residue each surface as
/// `Refused{reason, detail}` carrying the single-sourced remedy. `Err` is left for
/// internal faults.
///
/// # The Class-2 `Reap` row
///
/// The row is found by the fork the funnel itself RECORDED (`spawn.fork == name`), not
/// by re-resolving the worker worktree — a half-reaped fork (worktree gone, branch
/// left) must still be completable, and the resolver cannot see one.
///
/// IDEMPOTENT COMPLETION (D-P8-4): reaching `reaped` is contingent on the gate passing,
/// the gc outcome being advance-eligible, and the `Reap` row landing — NOT on gc finding
/// the fork present. A fork already gone is `AlreadyAbsent`, which advances. The record
/// is Class-2 (strictly after the act); a row that does not land yields
/// [`FunnelOutcome::ReapedRowPending`], never a bare `Reaped`.
pub(crate) fn dispatch_reap(root: &Path, slice: u32, name: &str) -> anyhow::Result<FunnelOutcome> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
    };
    let reaped = FunnelOutcome::Reaped {
        fork: name.to_owned(),
    };
    // THE landing authority — one named resolver, shared with `worktree list` (T10).
    let funnel_landed = match funnel::resolve_landing(&coord.root, &coord.tip, slice, name)? {
        LandingVerdict::Landed => true,
        // Neither injects a fact: `git cherry` decides, unchanged (fail-closed).
        LandingVerdict::NotProven | LandingVerdict::NoRow => false,
        LandingVerdict::Ambiguous => {
            return Ok(funnel_refused(
                AMBIGUOUS_FORK_ROW,
                format!(
                    "two or more SL-{slice:03} funnel rows record fork {name}: no single import certifies it — repair the funnel record before reaping"
                ),
            ));
        }
    };

    let funnel_record = funnel::read_funnel_at(&coord.root, &coord.tip, slice)?;
    let Some((phase, row)) = row_for_fork(&funnel_record, name) else {
        // Not funnel-managed: no row to gate and no row to advance — the gc machine's
        // own oracle IS the whole decision, now relayed as a structured refusal.
        let outcome = reap_fork(Some(coord.root), name, funnel_landed)?;
        return Ok(match reap_verdict(outcome, name) {
            Ok(_) => reaped,
            Err(refusal) => refusal,
        });
    };

    // The GATE, one step after `resolve_coord` and BEFORE the gc belt.
    let facts = TransitionFacts {
        row: Some(row),
        coord_tip: &coord.tip,
        paths_since_verify: None,
    };
    if let Err(illegal) =
        funnel_machine::attempt_advance(Some(row.position), &Transition::Reap, &facts)
    {
        return Ok(funnel_refused(illegal.reason, illegal.to_string()));
    }

    // The act. No presence probe: `reap_fork` IS idempotent — an already-gone fork is
    // `AlreadyAbsent` (which advances), so there is no second existence derivation to
    // drift from the gather's own.
    let outcome = reap_fork(Some(coord.root.clone()), name, funnel_landed)?;
    let leftover = match reap_verdict(outcome, name) {
        Ok(leftover) => leftover,
        Err(refusal) => return Ok(refusal),
    };

    // Class-2: record STRICTLY after the act.
    if let Some(pending) = land_reap_row(&coord.root, &coord.tip, slice, &phase, name)? {
        return Ok(pending);
    }
    if !leftover.is_empty() {
        // ADMINISTRATIVE residue only (the fork IS gone and the row DID land, so the
        // outcome is `Reaped`): named on stderr, never on stdout — stdout is the wire.
        drop(writeln!(
            std::io::stderr(),
            "warning: reap of {name} left administrative residue: {}",
            leftover.join(", ")
        ));
    }
    Ok(reaped)
}

/// Land the Class-2 `Reap` row against `tip`, STRICTLY after the act. `Ok(None)` ⇒ the
/// row landed (or replayed) and the position stands at `reaped`; `Ok(Some(outcome))` ⇒ it
/// did NOT, so the caller must return [`FunnelOutcome::ReapedRowPending`] rather than a
/// bare `Reaped` — the fork IS gone and the row is pending, and a success outcome may
/// never imply a landed row (SL-228 PHASE-08, ISS-246).
///
/// Factored out so a unit test can inject a fault at the ref-update step (a STALE `tip`
/// whose CAS loses to a sibling advance) and prove the distinct outcome — the
/// [`conclude_boundary_commit`] precedent.
fn land_reap_row(
    coord_root: &Path,
    tip: &str,
    slice: u32,
    phase: &str,
    fork: &str,
) -> anyhow::Result<Option<FunnelOutcome>> {
    let at = crate::clock::now_timestamp()?;
    let landing = funnel::land_funnel_transition(
        coord_root,
        &dispatch_ref(slice),
        tip,
        slice,
        phase,
        &Transition::Reap,
        None,
        &at,
        &Provenance::Conclude {
            who: dispatch_identity(),
        },
    );
    // The row lost its CAS (or the machine refused it): the fork IS gone, the row is NOT
    // landed. BOTH facts travel to the caller — a re-drive completes the row.
    let detail = match landing {
        Ok(funnel::FunnelLanding::Landed { .. } | funnel::FunnelLanding::Replayed { .. }) => {
            return Ok(None);
        }
        Ok(other) => format!("{other:?}"),
        Err(cause) => format!("{cause:#}"),
    };
    Ok(Some(FunnelOutcome::ReapedRowPending {
        fork: fork.to_owned(),
        detail: format!(
            "the fork {fork} IS reaped; the SL-{slice:03} {phase} reap row did not land ({detail}) — re-drive `dispatch_reap` to complete it"
        ),
    }))
}

/// The [`GcOutcome`] ⇄ [`FunnelOutcome`] map (ISS-246). `Ok(leftover)` ⇒ the fork is
/// gone, so the position MAY advance; `leftover` names any ADMINISTRATIVE residue worth
/// reporting alongside the success. `Err(refusal)` ⇒ a structured refusal to relay: the
/// caller gets the diagnosis AND the single-sourced remedy instead of a `-32603` that
/// drops both. PURE — its home is here, beside [`FunnelOutcome`], because it is the
/// translation between an engine-tier vocabulary and this module's own wire shape.
fn reap_verdict(outcome: GcOutcome, fork: &str) -> Result<Vec<String>, FunnelOutcome> {
    match outcome {
        GcOutcome::Reaped | GcOutcome::AlreadyAbsent => Ok(Vec::new()),
        GcOutcome::NotLanded => Err(funnel_refused(
            GcRefusal::NotLanded.token(),
            format!(
                "fork {fork} has not provably landed: {}",
                GcRefusal::NotLanded.remedy()
            ),
        )),
        GcOutcome::Busy => Err(funnel_refused(
            CLAIM_BUSY,
            format!(
                "an active claimant holds the claim lock for {fork} (a spawn is mid claim→bind→act); nothing was classified and nothing deleted — retry once it finishes"
            ),
        )),
        // CRITICAL residue outranks ADMINISTRATIVE: a surviving worktree/branch means
        // the fork is NOT gone, so `reaped` would be a lie.
        GcOutcome::Residual(residue) => {
            let members = residue.members(fork, None);
            if GcOutcome::Residual(residue).advances() {
                Ok(members)
            } else {
                Err(funnel_refused(
                    GC_INCOMPLETE,
                    format!(
                        "reap of {fork} left leftover(s) needing manual cleanup: {}",
                        members.join(", ")
                    ),
                ))
            }
        }
    }
}

/// The funnel row a fork branch belongs to — matched on the `spawn.fork` the funnel
/// itself recorded, so it survives a half-reaped fork the agent resolver cannot see.
/// Yields the phase id alongside the row (the row's `id` IS the phase).
///
/// Delegates to `FunnelRecord::rows_for_fork` — THE one fork⇄row predicate (SL-228
/// PHASE-08 T3), shared with [`funnel::resolve_landing`], which needs the match COUNT.
/// A single row is all this call site needs; ambiguity is refused upstream, before any
/// act, so taking the first here can never authorise a deletion on its own.
fn row_for_fork<'a>(
    record: &'a funnel::FunnelRecord,
    fork: &'a str,
) -> Option<(String, &'a PhaseRow)> {
    record.rows_for_fork(fork).next().map(|r| (r.id.clone(), r))
}

// --- T4: dispatch_verify ---------------------------------------------------------------

/// `dispatch_verify{slice, phase}` — resolve the coord SERVER-SIDE by slice, then run
/// the SHARED [`verify_phase`] engine (design §5): pre-act gate → proof-gated
/// forward-sync → the configured suite → re-prove the tree → land the verdict in ONE
/// CAS commit. The CLI `dispatch verify` runs the SAME engine against the invoking
/// coord tree — no forked logic.
pub(crate) fn dispatch_verify(
    root: &Path,
    slice: u32,
    phase: &str,
) -> anyhow::Result<FunnelOutcome> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
    };
    Ok(match verify_phase(&coord.root, &coord.tip, slice, phase)? {
        VerifyOutcome::Verified { coord_tip, suite } => {
            FunnelOutcome::Verified { coord_tip, suite }
        }
        VerifyOutcome::VerifyFailed { suite, detail } => {
            FunnelOutcome::VerifyFailed { suite, detail }
        }
        VerifyOutcome::Refused { reason, detail } => funnel_refused(&reason, detail),
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
/// The tool name the MCP registry keys `dispatch_tree_state` on (STD-001) — SL-228
/// PHASE-01, the only Move-E read verb needing an MCP tool this phase.
pub(crate) const TOOL_DISPATCH_TREE_STATE: &str = "dispatch_tree_state";
/// The tool name the MCP registry keys `dispatch_next` on (STD-001) — SL-228 PHASE-06,
/// the funnel's single-prescription oracle. DISTINCT from [`TOOL_DISPATCH_NEXT_READY`],
/// which answers the READINESS question (which phase may start); `dispatch_next` answers
/// the FUNNEL question (what ONE thing to do now). Neither replaces the other.
pub(crate) const TOOL_DISPATCH_NEXT: &str = "dispatch_next";

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
    /// The phase's durable FUNNEL POSITION at the tip (SL-228 PHASE-05, D4), absent
    /// when the phase has no funnel row — which is exactly the input that selects the
    /// pre-funnel receipt derivation. Carried so a consumer can see WHY the status
    /// reads as it does (position ⊕ boundary outrank the disposable sheet).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) position: Option<String>,
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
    // The committed funnel record at the SAME tip — the durable position tier (D4).
    // This is the ONLY caller that supplies a position: `phase_projection` /
    // `run_status` pass `None`, so the rollup's rendering is untouched (R1).
    let position = funnel::read_funnel_at(&coord.root, &coord.tip, slice)?.position(phase);
    // The disposable runtime sheet lives under the coord worktree's gitignored state.
    let state_dir = crate::state::phases_dir(&coord.root, slice);
    let status = crate::dispatch::phase_receipt_status(&state_dir, phase, has_boundary, position);
    Ok(ReadOutcome::Resolved(PhaseReceiptCore {
        slice,
        phase: phase.to_owned(),
        status: status.as_str().to_owned(),
        dispatch_tip: coord.tip,
        code_start: boundary.map(|b| b.code_start_oid.clone()),
        code_end: boundary.map(|b| b.code_end_oid.clone()),
        position: position.map(|p| p.as_str().to_owned()),
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

/// `dispatch_tree_state` outcome — the tree-state readout, or a coord refusal. The
/// payload [`crate::dispatch::TreeStateCore`] is single-sourced in `dispatch.rs`
/// (the funnel authority home, design D6) and shared with the CLI verb — reached
/// along the existing `mcp_server → dispatch` edge, so no reverse edge is minted.
pub(crate) type TreeStateResult = ReadOutcome<crate::dispatch::TreeStateCore>;

/// `dispatch_tree_state{slice}` — resolve the coord SERVER-SIDE, then report its
/// UNTRACKED-AWARE tree state via [`git::tree_clean_untracked`] (SL-228 PHASE-01,
/// design §8 gap 1 — the funnel read that replaces a raw post-write `git status`).
/// Read-only: the object-db resolve plus a `git status` read, no coord mutation. A
/// [`resolve_coord`] refusal short-circuits to `CoordRefused(reason)`. The payload
/// is built by the SHARED [`crate::dispatch::tree_state_core`] the CLI verb also uses.
pub(crate) fn dispatch_tree_state(root: &Path, slice: u32) -> anyhow::Result<TreeStateResult> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(coord_refused(refusal)),
    };
    let core = crate::dispatch::tree_state_core(slice, git::tree_clean_untracked(&coord.root)?);
    Ok(ReadOutcome::Resolved(core))
}

/// `dispatch_next` outcome — the single prescription, or a coord refusal. The payload
/// [`crate::dispatch::NextCore`] is single-sourced in `dispatch.rs` (the funnel authority
/// home, design D6) and shared with the CLI verb — reached along the existing
/// `mcp_server → dispatch` edge, so no reverse edge is minted.
pub(crate) type NextResult = ReadOutcome<crate::dispatch::NextCore>;

/// `dispatch_next{slice}` — resolve the coord SERVER-SIDE, then return the ONE action
/// the funnel prescribes (SL-228 PHASE-06, design §6), via the SHARED
/// [`crate::dispatch::next_core`] oracle the CLI verb also uses. Strictly read-only: the
/// committed funnel record at the tip, a changed-path set, and the readiness authority's
/// own plan reads — it heals nothing and lands nothing. A [`resolve_coord`] refusal
/// short-circuits to `CoordRefused(reason)`.
///
/// DISTINCT from [`dispatch_next_ready`]: that tool answers "which phase may start",
/// this one answers "what is the single next thing to do".
pub(crate) fn dispatch_next(root: &Path, slice: u32) -> anyhow::Result<NextResult> {
    let coord = match resolve_coord(root, slice) {
        Ok(coord) => coord,
        Err(refusal) => return Ok(coord_refused(refusal)),
    };
    let core = crate::dispatch::next_core(&coord.root, &coord.tip, slice)?;
    Ok(ReadOutcome::Resolved(core))
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

    // ==================================================================================
    // SL-228 PHASE-05 — the funnel GATES on the write verbs (T2/T3/T6), the one-commit
    // heal-forward, and the conclude reorder.
    // ==================================================================================

    mod funnel_gates {
        use super::super::{
            dispatch_conclude_phase, dispatch_import, dispatch_phase_receipt, dispatch_reap,
        };
        use super::{add_fork, git_run, primary_with_coord};
        use crate::dispatch::{
            Identity, Provenance, dispatch_ref, funnel, funnel_message, verify_phase,
        };
        use crate::funnel_machine::{
            Position, Transition, VerifyEvidence, VerifyStatus, funnel_record_path,
        };
        use crate::mcp_server::dispatch::{FunnelOutcome, ReadOutcome};
        use crate::worktree::{ForkBinding, provision_dispatch_record};
        use std::fs;
        use std::path::{Path, PathBuf};

        const SLICE: u32 = 199;
        const PHASE: &str = "PHASE-01";
        const FORK: &str = "dispatch/wk";
        const AGENT: &str = "wk";
        const AT: &str = "2026-07-25T09:00:00Z";

        fn disp() -> Provenance {
            Provenance::Conclude {
                who: Identity {
                    name: "dispatch".to_string(),
                    email: "dispatch@doctrine".to_string(),
                },
            }
        }

        /// Land `ts` through the sole writer against `tip`; return the new tip.
        /// `paths` is conclude's tree-identity supply (`None` for every other kind).
        fn land(coord: &Path, tip: &str, ts: &[Transition], paths: Option<&[String]>) -> String {
            match funnel::land_funnel_transitions(
                coord,
                &dispatch_ref(SLICE),
                tip,
                SLICE,
                PHASE,
                ts,
                paths,
                None,
                &funnel_message(SLICE, PHASE),
                AT,
                &disp(),
            )
            .unwrap()
            {
                funnel::FunnelLanding::Landed { oid, .. } => oid,
                other => panic!("expected Landed, got {other:?}"),
            }
        }

        fn tip(coord: &Path) -> String {
            git_run(coord, &["rev-parse", &dispatch_ref(SLICE)])
        }

        fn position(coord: &Path, at: &str) -> Option<Position> {
            funnel::read_funnel_at(coord, at, SLICE)
                .unwrap()
                .position(PHASE)
        }

        /// A LIVE worker fork the resolver can see: a linked worktree at
        /// `<coord>/.worktrees/<AGENT>` on `dispatch/<AGENT>`, advanced by exactly one
        /// commit past `base`, plus the durable per-worktree record. `binding` absent ⇒
        /// an UNBOUND fork (the `unprovable-fork` fixture).
        fn live_fork(coord: &Path, base: &str, binding: Option<&ForkBinding>) -> PathBuf {
            let dir = coord.join(".worktrees").join(AGENT);
            git_run(
                coord,
                &[
                    "worktree",
                    "add",
                    "-q",
                    "-b",
                    FORK,
                    dir.to_str().unwrap(),
                    base,
                ],
            );
            add_fork(coord, base, FORK, "src/f.rs", "fn f() {}\n");
            provision_dispatch_record(coord, AGENT, base, &dir, FORK, binding).unwrap();
            dir
        }

        fn bound() -> ForkBinding {
            ForkBinding {
                slice: SLICE,
                phase: PHASE.to_owned(),
            }
        }

        /// Author a minimal slice in the coord tree declaring `design-target`
        /// selectors — `dispatch_import`'s scope belt reads them as a HARD gate, so a
        /// slice-less fixture is not a valid import target.
        fn seed_slice(coord: &Path, selectors: &[&str]) {
            let dir = coord.join(format!(".doctrine/slice/{SLICE:03}"));
            fs::create_dir_all(&dir).unwrap();
            let rows: String = selectors
                .iter()
                .map(|s| {
                    format!("\n[[selector]]\nselector = \"{s}\"\nintent = \"design-target\"\n")
                })
                .collect();
            fs::write(
                dir.join(format!("slice-{SLICE:03}.toml")),
                format!(
                    "id = {SLICE}\nslug = \"fixture\"\ntitle = \"Fixture\"\n\
                     status = \"proposed\"\ncreated = \"2026-07-25\"\nupdated = \"2026-07-25\"\n\
                     \n[relationships]\nneeds = []\nafter = []\n{rows}"
                ),
            )
            .unwrap();
            fs::write(
                dir.join(format!("slice-{SLICE:03}.md")),
                "# Fixture\n\n## Context\n\n## Scope & Objectives\n",
            )
            .unwrap();
        }

        // --- T6: the one-commit heal-forward ------------------------------------------

        #[test]
        fn import_heals_the_whole_prefix_in_exactly_one_commit() {
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            seed_slice(&coord, &["src/**"]);
            live_fork(&coord, &base, Some(&bound()));

            let out = dispatch_import(&primary, SLICE, FORK).unwrap();
            let landed = match out {
                FunnelOutcome::Imported { coord_tip } => coord_tip,
                other => panic!("expected Imported, got {other:?}"),
            };
            // ONE commit for the worker delta AND the whole healed funnel prefix.
            assert_eq!(
                git_run(
                    &coord,
                    &["rev-list", "--count", &format!("{base}..{landed}")]
                ),
                "1",
                "delta ⊕ healed prefix in a single commit"
            );
            let row = funnel::read_funnel_at(&coord, &landed, SLICE)
                .unwrap()
                .row(PHASE)
                .cloned()
                .expect("the healed row");
            assert_eq!(row.position, Position::Imported);
            assert_eq!(row.spawn.expect("spawn").fork, FORK);
            assert!(
                row.worker_commit.is_some(),
                "the lagging Class-2 row healed"
            );
            assert!(row.import.is_some());
            // The worker delta rode the same commit, with the worker AUTHOR preserved.
            let names = git_run(&coord, &["ls-tree", "-r", "--name-only", &landed]);
            assert!(names.lines().any(|l| l == "src/f.rs"), "{names}");
            assert!(names.contains(&funnel_record_path(SLICE)), "{names}");
            assert_eq!(
                git_run(&coord, &["log", "-1", "--format=%an", &landed]),
                "worker-x",
                "IMPORT provenance keeps the worker author"
            );
        }

        #[test]
        fn position_never_durably_rests_at_worker_committed_via_import() {
            // The reason the heal is ONE commit: there is no intermediate commit a kill
            // could freeze the run at. Every commit the import produced is inspected.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            seed_slice(&coord, &["src/**"]);
            live_fork(&coord, &base, Some(&bound()));
            dispatch_import(&primary, SLICE, FORK).unwrap();

            let landed = tip(&coord);
            let walk = git_run(&coord, &["rev-list", &format!("{base}..{landed}")]);
            for commit in walk.lines() {
                assert_ne!(
                    position(&coord, commit),
                    Some(Position::WorkerCommitted),
                    "no commit in the import run rests at worker-committed"
                );
            }
        }

        #[test]
        fn a_lost_response_retry_is_a_no_op_replay_despite_a_fresh_onto() {
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            seed_slice(&coord, &["src/**"]);
            live_fork(&coord, &base, Some(&bound()));
            let first = match dispatch_import(&primary, SLICE, FORK).unwrap() {
                FunnelOutcome::Imported { coord_tip } => coord_tip,
                other => panic!("{other:?}"),
            };
            // The retry re-resolves a FRESH `onto` (the coord tip has moved to `first`),
            // which is deliberately EXCLUDED from replay identity — only `fork_tip`
            // keys it, so the retry is a no-op that names where the coord stands.
            assert_eq!(
                dispatch_import(&primary, SLICE, FORK).unwrap(),
                FunnelOutcome::Imported {
                    coord_tip: first.clone()
                }
            );
            assert_eq!(tip(&coord), first, "the retry landed nothing");
            assert_eq!(
                git_run(
                    &coord,
                    &["rev-list", "--count", &format!("{base}..{first}")]
                ),
                "1",
                "still exactly one import commit"
            );
        }

        #[test]
        fn an_unbound_fork_refuses_unprovable_fork_and_lands_nothing() {
            // D6 / zero-rescue: nothing can prove which funnel row this fork's commit
            // belongs to, and import refuses rather than guessing — exactly as
            // `worker_commit` does, so there is no "the other one will heal it" arm.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            seed_slice(&coord, &["src/**"]);
            live_fork(&coord, &base, None);
            match dispatch_import(&primary, SLICE, FORK).unwrap() {
                FunnelOutcome::Refused { reason, detail } => {
                    assert_eq!(reason, "unprovable-fork");
                    assert_eq!(detail, FORK);
                }
                other => panic!("expected Refused, got {other:?}"),
            }
            assert_eq!(tip(&coord), base, "nothing landed");
        }

        #[test]
        fn a_fork_bound_to_another_slice_is_refused() {
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            seed_slice(&coord, &["src/**"]);
            live_fork(
                &coord,
                &base,
                Some(&ForkBinding {
                    slice: SLICE + 1,
                    phase: PHASE.to_owned(),
                }),
            );
            match dispatch_import(&primary, SLICE, FORK).unwrap() {
                FunnelOutcome::Refused { reason, detail } => {
                    assert_eq!(reason, "unprovable-fork");
                    assert!(detail.contains("SL-200"), "names the other slice: {detail}");
                }
                other => panic!("expected Refused, got {other:?}"),
            }
            assert_eq!(tip(&coord), base, "nothing landed");
        }

        // --- T3/T4: the conclude gate, the one-commit boundary ⊕ position, the reorder -

        /// Drive a phase to `verified` with PASS evidence pinned to the pre-evidence
        /// tip — the state conclude legitimately follows. Returns the coord tip.
        fn verified_phase(coord: &Path, base: &str) -> String {
            let imported = land(
                coord,
                base,
                &[
                    Transition::Spawn {
                        fork: FORK.to_owned(),
                        base_oid: base.to_owned(),
                    },
                    Transition::RecordWorkerCommit {
                        fork_tip: "forktip".to_owned(),
                    },
                    Transition::Import {
                        fork_tip: "forktip".to_owned(),
                        onto: base.to_owned(),
                    },
                ],
                None,
            );
            land(
                coord,
                &imported,
                &[Transition::Verify {
                    evidence: VerifyEvidence {
                        status: VerifyStatus::Pass,
                        verified_oid: imported.clone(),
                        suite: "gate".to_owned(),
                        at: AT.to_owned(),
                    },
                }],
                None,
            )
        }

        /// Seed the disposable phase sheet in both trees at `status`.
        fn seed_sheets(coord: &Path, primary: &Path, status: &str) {
            for root in [coord, primary] {
                let dir = crate::state::phases_dir(root, SLICE);
                fs::create_dir_all(&dir).unwrap();
                fs::write(
                    dir.join("phase-01.toml"),
                    format!("status = \"{status}\"\n"),
                )
                .unwrap();
            }
        }

        #[test]
        fn a_funnel_managed_conclude_lands_boundary_and_position_in_one_commit() {
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            let verified = verified_phase(&coord, &base);
            seed_sheets(&coord, &primary, "in_progress");

            let out =
                dispatch_conclude_phase(&primary, SLICE, PHASE, &base, &verified, Some("done"))
                    .unwrap();
            let landed = match out {
                FunnelOutcome::Concluded { coord_tip } => coord_tip,
                other => panic!("expected Concluded, got {other:?}"),
            };
            assert_eq!(
                git_run(
                    &coord,
                    &["rev-list", "--count", &format!("{verified}..{landed}")]
                ),
                "1",
                "boundary ⊕ position in ONE commit"
            );
            let names = git_run(&coord, &["ls-tree", "-r", "--name-only", &landed]);
            assert!(names.contains("boundaries.toml"), "{names}");
            assert_eq!(position(&coord, &landed), Some(Position::Concluded));
            // The trailing projection ran too.
            assert_eq!(
                crate::state::read_phase_status(
                    &crate::state::phases_dir(&coord, SLICE),
                    "phase-01"
                )
                .unwrap(),
                Some("completed".to_string())
            );
        }

        #[test]
        fn the_conclude_kill_window_still_reads_as_completed() {
            // VT-4. The CAS lands boundary ⊕ position; the sheet flip is a TRAILING
            // projection. Kill in between (drive the durable half alone) and the §9
            // matrix must still report `completed` — the disposable tier may lag the
            // committed one, never lead it.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            let verified = verified_phase(&coord, &base);
            seed_sheets(&coord, &primary, "in_progress");

            let row = funnel::read_funnel_at(&coord, &verified, SLICE)
                .unwrap()
                .row(PHASE)
                .cloned()
                .expect("the verified row");
            // The durable half ONLY — exactly what a kill before the flip leaves.
            let landed = match super::super::conclude_funnel_commit(
                &super::ct(&coord, &verified),
                SLICE,
                PHASE,
                &base,
                &verified,
                &row,
                AT,
            )
            .unwrap()
            {
                FunnelOutcome::Concluded { coord_tip } => coord_tip,
                other => panic!("expected Concluded, got {other:?}"),
            };
            assert_eq!(position(&coord, &landed), Some(Position::Concluded));
            assert_eq!(
                crate::state::read_phase_status(
                    &crate::state::phases_dir(&coord, SLICE),
                    "phase-01"
                )
                .unwrap(),
                Some("in_progress".to_string()),
                "the sheet is deliberately BEHIND — the kill window"
            );

            match dispatch_phase_receipt(&primary, SLICE, PHASE).unwrap() {
                ReadOutcome::Resolved(core) => {
                    assert_eq!(
                        core.status, "completed",
                        "a lagging sheet must not mask durable completion"
                    );
                    assert_eq!(core.position.as_deref(), Some("concluded"));
                }
                other => panic!("expected Resolved, got {other:?}"),
            }
        }

        #[test]
        fn conclude_refuses_when_a_non_record_path_changed_since_the_verified_tip() {
            // T4: the gate's `paths_since_verify` is the real B..tip path set, so an
            // unrelated commit landed after the evidence makes it STALE.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            let verified = verified_phase(&coord, &base);
            let drifted_tree = crate::git::tree_with_file(
                &coord,
                &format!("{verified}^{{tree}}"),
                "seed",
                "drift\n",
            )
            .unwrap();
            let drifted = git_run(
                &coord,
                &["commit-tree", &drifted_tree, "-p", &verified, "-m", "drift"],
            );
            git_run(&coord, &["update-ref", &dispatch_ref(SLICE), &drifted]);
            seed_sheets(&coord, &primary, "in_progress");

            match dispatch_conclude_phase(&primary, SLICE, PHASE, &base, &drifted, None).unwrap() {
                FunnelOutcome::Refused { reason, .. } => {
                    assert_eq!(reason, "conclude-verify-stale");
                }
                other => panic!("expected Refused, got {other:?}"),
            }
            assert_eq!(tip(&coord), drifted, "nothing landed");
            assert_eq!(
                crate::state::read_phase_status(
                    &crate::state::phases_dir(&coord, SLICE),
                    "phase-01"
                )
                .unwrap(),
                Some("in_progress".to_string()),
                "a refused conclude never flips the sheet"
            );
        }

        #[test]
        fn a_fresh_pass_can_always_conclude_because_evidence_never_self_stales() {
            // The self-consistency property the whole design rests on: the evidence
            // commit is a funnel-record-ONLY child of the tested tree, and conclude's
            // gate is "identical modulo the funnel record" — so a real `verify` run is
            // immediately concludable.
            let (_tmp, primary, coord, _base, _bt) = primary_with_coord(SLICE);
            fs::create_dir_all(coord.join(".doctrine")).unwrap();
            fs::write(
                coord.join(".doctrine/doctrine.toml"),
                "[verification]\ngate = [\"true\"]\n\n[dispatch]\nverify-suite = \"gate\"\n",
            )
            .unwrap();
            git_run(&coord, &["add", "."]);
            git_run(&coord, &["commit", "-q", "-m", "config"]);
            let configured = git_run(&coord, &["rev-parse", "HEAD"]);
            let imported = land(
                &coord,
                &configured,
                &[
                    Transition::Spawn {
                        fork: FORK.to_owned(),
                        base_oid: configured.clone(),
                    },
                    Transition::RecordWorkerCommit {
                        fork_tip: "forktip".to_owned(),
                    },
                    Transition::Import {
                        fork_tip: "forktip".to_owned(),
                        onto: configured.clone(),
                    },
                ],
                None,
            );
            verify_phase(&coord, &imported, SLICE, PHASE).unwrap();
            let verified = tip(&coord);
            seed_sheets(&coord, &primary, "in_progress");

            match dispatch_conclude_phase(&primary, SLICE, PHASE, &configured, &verified, None)
                .unwrap()
            {
                FunnelOutcome::Concluded { .. } => {}
                other => panic!("a fresh pass must conclude, got {other:?}"),
            }
            assert_eq!(position(&coord, &tip(&coord)), Some(Position::Concluded));
        }

        // --- T2: the reap gate + idempotent completion --------------------------------

        #[test]
        fn reap_refuses_before_conclude_when_the_phase_is_funnel_managed() {
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            let dir = live_fork(&coord, &base, Some(&bound()));
            land(
                &coord,
                &base,
                &[Transition::Spawn {
                    fork: FORK.to_owned(),
                    base_oid: base.clone(),
                }],
                None,
            );
            match dispatch_reap(&primary, SLICE, FORK).unwrap() {
                FunnelOutcome::Refused { reason, .. } => assert_eq!(reason, "worker-not-committed"),
                other => panic!("expected Refused, got {other:?}"),
            }
            assert!(dir.exists(), "the gate refused BEFORE the gc belt ran");
            assert!(
                crate::git::git_opt(&coord, &["rev-parse", "--verify", "--quiet", FORK])
                    .unwrap()
                    .is_some(),
                "the fork branch survives the refusal"
            );
        }

        #[test]
        fn reap_at_concluded_with_the_fork_already_absent_completes_idempotently() {
            // The crash window between deleting the fork and recording the row: a
            // re-drive must COMPLETE the run, not error on a fork that is already gone.
            //
            // EX-2 as a CLASS (SL-228 PHASE-08 D-P8-4), not as this one case: the funnel
            // position is advanced by the funnel verb, and reaching `reaped` is contingent
            // on exactly three things — the transition gate passing, the gc outcome being
            // advance-eligible, and the `Reap` row landing. It is NOT contingent on gc
            // finding the fork present, which is why the gc leg reports `AlreadyAbsent`
            // (an advancing outcome) rather than an error.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            let verified = verified_phase(&coord, &base);
            // Conclude's tree-identity supply: the ONLY thing that changed since the
            // verified tip is the evidence commit's own funnel record.
            let since = vec![funnel_record_path(SLICE)];
            let concluded = land(&coord, &verified, &[Transition::Conclude], Some(&since));
            assert_eq!(position(&coord, &concluded), Some(Position::Concluded));
            // No fork branch exists at all — the delete already happened.
            let out = dispatch_reap(&primary, SLICE, FORK).unwrap();
            assert_eq!(
                out,
                FunnelOutcome::Reaped {
                    fork: FORK.to_owned()
                }
            );
            assert_eq!(
                position(&coord, &tip(&coord)),
                Some(Position::Reaped),
                "the run completed: the Reap row landed without a gc"
            );
        }

        #[test]
        fn a_fork_with_no_funnel_row_keeps_the_legacy_ungated_reap() {
            // Solo / pre-funnel / legacy: there is no funnel authority to bypass, so
            // the shared git oracle stays the only gate — the VERDICT is unchanged.
            //
            // SL-228 PHASE-08 (ISS-246): what changed is its SHAPE. The refusal was a
            // hard `Err` that MCP flattened to `-32603`, dropping both the diagnosis and
            // the remedy; it is now a structured `Refused{reason, detail}` the caller can
            // act on. `Err` is reserved for internal faults.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            seed_slice(&coord, &["src/**"]);
            live_fork(&coord, &base, Some(&bound()));
            dispatch_import(&primary, SLICE, FORK).unwrap();
            // A DIFFERENT fork the funnel never recorded.
            let other = "dispatch/legacy";
            add_fork(&coord, &base, other, "src/legacy.rs", "l\n");
            match dispatch_reap(&primary, SLICE, other).unwrap() {
                FunnelOutcome::Refused { reason, detail } => {
                    assert_eq!(reason, "not-landed");
                    assert!(detail.contains(other), "names the fork: {detail}");
                    assert!(
                        detail.contains("dispatch_reap"),
                        "carries the single-sourced remedy: {detail}"
                    );
                }
                other => panic!("expected Refused, got {other:?}"),
            }
            assert!(
                crate::git::git_opt(&coord, &["rev-parse", "--verify", "--quiet", other])
                    .unwrap()
                    .is_some(),
                "the unlanded fork survives the refusal"
            );
        }

        // --- SL-228 PHASE-08 (T8/VT-2, EX-2, EX-4): the funnel-managed reap ------------

        /// Drive a LIVE fork all the way to `concluded` through the ATOMIC one-commit
        /// import — the exact topology ISS-245 broke, because the import commit's patch
        /// is a strict SUPERSET of the fork's (delta ⊕ `funnel.toml`) and so matches no
        /// patch-id. Returns the fork's live worktree dir (R4: without it present the gc
        /// leg is skipped and the regression is not exercised).
        fn concluded_with_live_fork(primary: &Path, coord: &Path, base: &str) -> PathBuf {
            seed_slice(coord, &["src/**"]);
            let dir = live_fork(coord, base, Some(&bound()));
            let imported = match dispatch_import(primary, SLICE, FORK).unwrap() {
                FunnelOutcome::Imported { coord_tip } => coord_tip,
                other => panic!("expected Imported, got {other:?}"),
            };
            let verified = land(
                coord,
                &imported,
                &[Transition::Verify {
                    evidence: VerifyEvidence {
                        status: VerifyStatus::Pass,
                        verified_oid: imported.clone(),
                        suite: "gate".to_owned(),
                        at: AT.to_owned(),
                    },
                }],
                None,
            );
            let since = vec![funnel_record_path(SLICE)];
            land(coord, &verified, &[Transition::Conclude], Some(&since));
            dir
        }

        #[test]
        fn a_funnel_managed_fork_is_reaped_without_force_despite_the_patch_id_oracle() {
            // ISS-245 end-to-end + EX-4: the fork's worktree AND branch are still there,
            // `git cherry` reports it unlanded, and the reap SUCCEEDS on the funnel
            // record's proof alone — no `--force` anywhere in the path.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            let dir = concluded_with_live_fork(&primary, &coord, &base);
            assert!(dir.exists(), "precondition: the fork worktree is live");
            let cherry = git_run(&coord, &["cherry", "HEAD", FORK]);
            assert!(
                cherry.lines().any(|l| l.starts_with('+')),
                "precondition (ISS-245): the atomic import defeats patch-id; got {cherry:?}"
            );

            assert_eq!(
                dispatch_reap(&primary, SLICE, FORK).unwrap(),
                FunnelOutcome::Reaped {
                    fork: FORK.to_owned()
                }
            );
            assert!(
                crate::git::git_opt(&coord, &["rev-parse", "--verify", "--quiet", FORK])
                    .unwrap()
                    .is_none(),
                "the fork branch is reaped"
            );
            assert!(!dir.exists(), "the fork worktree is reaped");
            assert_eq!(
                position(&coord, &tip(&coord)),
                Some(Position::Reaped),
                "the funnel position advanced"
            );
        }

        #[test]
        fn a_branch_advanced_past_the_imported_tip_refuses_and_keeps_the_branch() {
            // The deletion-of-unimported-work regression: the row says `concluded`, but
            // the branch carries a commit the funnel never imported, so the record's
            // proof does NOT cover the live tip. The conjunction fails ⇒ no fact is
            // injected ⇒ `git cherry` decides ⇒ `not-landed`, branch intact.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            concluded_with_live_fork(&primary, &coord, &base);
            let fork_tip = git_run(&coord, &["rev-parse", FORK]);
            let advanced = git_run(
                &coord,
                &[
                    "commit-tree",
                    &format!("{fork_tip}^{{tree}}"),
                    "-p",
                    &fork_tip,
                    "-m",
                    "work the funnel never imported",
                ],
            );
            git_run(
                &coord,
                &["update-ref", &format!("refs/heads/{FORK}"), &advanced],
            );

            match dispatch_reap(&primary, SLICE, FORK).unwrap() {
                FunnelOutcome::Refused { reason, .. } => assert_eq!(reason, "not-landed"),
                other => panic!("expected Refused, got {other:?}"),
            }
            assert_eq!(
                crate::git::git_opt(&coord, &["rev-parse", "--verify", "--quiet", FORK])
                    .unwrap()
                    .as_deref(),
                Some(advanced.as_str()),
                "the branch — and the uncertified work on it — survives"
            );
            assert_eq!(
                position(&coord, &tip(&coord)),
                Some(Position::Concluded),
                "position did not advance"
            );
        }

        #[test]
        fn a_lost_reap_row_cas_is_row_pending_never_a_bare_reaped() {
            // The Class-2 window: the deletes succeeded, then the row's CAS lost to a
            // sibling that advanced the coord ref. Saying `Reaped` would imply a landed
            // row; the caller is told BOTH facts and re-drives to completion.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            concluded_with_live_fork(&primary, &coord, &base);
            let concluded = tip(&coord);
            // A sibling advances the ref after our tip was resolved.
            let sibling = git_run(
                &coord,
                &[
                    "commit-tree",
                    &format!("{concluded}^{{tree}}"),
                    "-p",
                    &concluded,
                    "-m",
                    "a sibling advance",
                ],
            );
            git_run(&coord, &["update-ref", &dispatch_ref(SLICE), &sibling]);
            assert_ne!(tip(&coord), concluded, "precondition: the ref moved");

            match super::super::land_reap_row(&coord, &concluded, SLICE, PHASE, FORK).unwrap() {
                Some(FunnelOutcome::ReapedRowPending { fork, detail }) => {
                    assert_eq!(fork, FORK);
                    assert!(detail.contains("IS reaped"), "{detail}");
                    assert!(detail.contains("re-drive"), "{detail}");
                }
                other => panic!("expected ReapedRowPending, got {other:?}"),
            }
            assert_eq!(
                position(&coord, &tip(&coord)),
                Some(Position::Concluded),
                "the reap row did NOT land"
            );
            assert_eq!(tip(&coord), sibling, "and nothing was committed");
            // Contrast: against the LIVE tip the same row lands and reports nothing.
            assert_eq!(
                super::super::land_reap_row(&coord, &sibling, SLICE, PHASE, FORK).unwrap(),
                None
            );
            assert_eq!(position(&coord, &tip(&coord)), Some(Position::Reaped));
        }

        #[test]
        fn two_rows_naming_one_fork_refuse_before_any_act() {
            // Fail-closed on ambiguity: no single `import.fork_tip` speaks for the
            // branch, so nothing is deleted and the first row is never bound.
            let (_tmp, primary, coord, base, _bt) = primary_with_coord(SLICE);
            let dir = concluded_with_live_fork(&primary, &coord, &base);
            // A SECOND phase row recording the very same fork.
            let concluded = tip(&coord);
            let second = crate::dispatch::funnel::land_funnel_transition(
                &coord,
                &dispatch_ref(SLICE),
                &concluded,
                SLICE,
                "PHASE-02",
                &Transition::Spawn {
                    fork: FORK.to_owned(),
                    base_oid: base.clone(),
                },
                None,
                AT,
                &disp(),
            )
            .unwrap();
            assert!(matches!(
                second,
                crate::dispatch::funnel::FunnelLanding::Landed { .. }
            ));

            match dispatch_reap(&primary, SLICE, FORK).unwrap() {
                FunnelOutcome::Refused { reason, detail } => {
                    assert_eq!(reason, "ambiguous-fork-row");
                    assert!(detail.contains(FORK), "names the fork: {detail}");
                }
                other => panic!("expected Refused, got {other:?}"),
            }
            assert!(dir.exists(), "nothing was deleted");
            assert!(
                crate::git::git_opt(&coord, &["rev-parse", "--verify", "--quiet", FORK])
                    .unwrap()
                    .is_some(),
                "the branch survives an ambiguous record"
            );
        }
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
        // HARD refuse with the SAME token the CLI import's `Refusal::UndeclaredScope`
        // uses, AND a non-empty `detail` that NAMES the offending path via the shared
        // `undeclared_detail` formatter (SL-224 PHASE-01) — no more empty detail.
        match out {
            FunnelOutcome::Refused { reason, detail } => {
                assert_eq!(reason, Refusal::UndeclaredScope.token());
                assert!(
                    detail.contains("docs/readme.md"),
                    "detail names the offending path: {detail}"
                );
                assert!(
                    detail.contains("doctrine slice selector add SL-199 docs/readme.md"),
                    "detail carries a runnable remediation: {detail}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
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

    // --- VT-3: dispatch_reap (the shared landed-oracle for a fork with no funnel row) --

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
        // the shared oracle REFUSES and the branch survives.
        //
        // SL-228 PHASE-08 (ISS-246): the refusal is now a STRUCTURED `Refused{reason,
        // detail}` carrying the diagnosis and the single-sourced remedy, not an `Err` that
        // MCP flattens to `-32603` and drops both. The verdict is identical; only its
        // shape changed.
        let unlanded_branch = "dispatch/unlanded";
        add_fork(&coord, &base, unlanded_branch, "src/unlanded.rs", "u\n");
        match dispatch_reap(&primary, 199, unlanded_branch).unwrap() {
            FunnelOutcome::Refused { reason, detail } => {
                assert_eq!(reason, "not-landed", "gc refuses an unlanded fork");
                assert!(
                    detail.contains(unlanded_branch) && detail.contains("--force"),
                    "the detail names the fork and the remedy: {detail}"
                );
            }
            other => panic!("expected Refused, got {other:?}"),
        }
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
