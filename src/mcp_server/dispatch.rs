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
use crate::git::{self, WorktreeRecord};
use crate::ledger::Boundaries;
use crate::worktree::{Apply, classify_import, run_gc};
use anyhow::{Context, bail};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The stable funnel marker every landed dispatch commit is grepped by (STD-001
/// single-source — no bare literal at the call sites). Naming the slice/phase keeps the
/// history greppable across a run.
const FUNNEL_MARKER: &str = "dispatch-funnel";

/// The grep-stable commit message the funnel lands: `<marker>: SL-<NNN> <phase>`
/// (provenance contract — the orchestrator recovers slice/phase from the subject).
pub(crate) fn funnel_message(slice: u32, phase: &str) -> String {
    format!("{FUNNEL_MARKER}: SL-{slice:03} {phase}")
}

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

// --- commit_on_behalf: working-tree-free compose --------------------------------------

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

/// Compose ONE non-merge commit of `tree` onto the coord branch tip and land it with a
/// compare-and-swap against `expected_old` (design §B). `coord_root` is the live
/// `dispatch/<NNN>` checkout; the CAS ref is its checked-out branch (`HEAD`). The commit
/// is built object-db-only (`commit-tree` — VT-4: a pre-existing staged/dirty coord tree
/// is NEVER swept in, because the compose reads the NAMED `tree`, not the live index).
///
/// Refuses (typed, not an error): `empty-delta` when `tree` matches the parent's tree
/// (no change); `lost-ref-race` when the CAS finds the tip moved off `expected_old`. On
/// EITHER refusal — and on any fault before the ref advance — the coord ref, index, and
/// worktree are BYTE-UNCHANGED (the primitive touches none of them; a dangling commit
/// object is inert). A genuine plumbing failure stays an `anyhow` `Err`.
pub(crate) fn commit_on_behalf(
    coord_root: &Path,
    expected_old: &str,
    tree: &str,
    message: &str,
    prov: &Provenance,
) -> anyhow::Result<CommitOutcome> {
    // empty-delta: the composed tree equals the parent's tree ⇒ no change (never mint an
    // empty commit). Peeling `^{tree}` accepts either a tree or a commit as `tree`.
    let parent_tree = git::git_text(
        coord_root,
        &["rev-parse", &format!("{expected_old}^{{tree}}")],
    )
    .context("resolve parent tree")?;
    let new_tree = git::git_text(coord_root, &["rev-parse", &format!("{tree}^{{tree}}")])
        .context("resolve composed tree")?;
    if new_tree == parent_tree {
        return Ok(CommitOutcome::Refused(CommitRefusal::EmptyDelta));
    }

    // Compose the single non-merge commit (one `-p`) — object-db only, no working tree.
    let oid = commit_tree_as(coord_root, tree, expected_old, message, prov)?;

    // The CAS target: the coord branch this worktree has checked out.
    let branch_ref = git::git_text(coord_root, &["rev-parse", "--symbolic-full-name", "HEAD"])
        .context("resolve coord branch ref")?;

    // Lost-ref-race guard: advance the ref ONLY if it still equals `expected_old`.
    match git::update_ref_cas(coord_root, &branch_ref, &oid, expected_old)? {
        git::RefCas::Updated => Ok(CommitOutcome::Landed { oid }),
        git::RefCas::Moved { .. } => Ok(CommitOutcome::Refused(CommitRefusal::LostRefRace)),
    }
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

/// The dispatch committer identity every funnel commit carries (STD-001 — the same
/// `<name> <email>` the §B provenance tests pin).
const DISPATCH_NAME: &str = "dispatch";
const DISPATCH_EMAIL: &str = "dispatch@doctrine";

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

/// The dispatch committer/author identity (`Conclude` author==committer; `Import`
/// committer).
fn dispatch_identity() -> Identity {
    Identity {
        name: DISPATCH_NAME.to_owned(),
        email: DISPATCH_EMAIL.to_owned(),
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
    match commit_on_behalf(&coord.root, &coord.tip, &tree, &message, &prov)? {
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
    // (b) the working-tree-free boundary commit — the durable, committed tier.
    conclude_boundary_commit(&coord.root, &coord.tip, slice, phase, code_start, code_end)
}

/// Tier (b) of [`dispatch_conclude_phase`]: compose the `(code_start, code_end)`
/// [`BoundaryRow`] into `boundaries.toml` (UPSERT-by-phase, mirroring
/// `ledger::record_boundary`) and land it with ONE working-tree-free [`commit_on_behalf`]
/// onto `tip`. Factored out so a unit test can inject a fault at the ref-update step (a
/// STALE `tip`) and prove the boundary is ABSENT + the live index/worktree byte-unchanged.
fn conclude_boundary_commit(
    coord_root: &Path,
    tip: &str,
    slice: u32,
    phase: &str,
    code_start: &str,
    code_end: &str,
) -> anyhow::Result<FunnelOutcome> {
    let path = format!(".doctrine/dispatch/{slice:03}/boundaries.toml");
    // Read-modify-write over the COMMITTED tip tree (object-db, working-tree-free) — the
    // live coord `boundaries.toml` is never touched.
    let mut boundaries = match git::read_path_at(coord_root, tip, &path)? {
        Some(text) => Boundaries::parse(&text)
            .with_context(|| format!("parse committed boundaries.toml at {tip}"))?,
        None => Boundaries::default(),
    };
    let row = BoundaryRow {
        phase: phase.to_owned(),
        code_start_oid: code_start.to_owned(),
        code_end_oid: code_end.to_owned(),
        provenance: BoundaryProvenance::Funnel,
    };
    // UPSERT by phase — a funnel retry REPLACES the same phase's row, never duplicates
    // (the SAME fold `ledger::record_boundary` applies).
    match boundaries.rows.iter_mut().find(|r| r.phase == row.phase) {
        Some(existing) => *existing = row,
        None => boundaries.rows.push(row),
    }
    let canonical = boundaries.to_toml()?;
    let tip_tree = git::git_text(coord_root, &["rev-parse", &format!("{tip}^{{tree}}")])
        .with_context(|| format!("resolve tip tree {tip}"))?;
    let tree = git::tree_with_file(coord_root, &tip_tree, &path, &canonical)?;
    let prov = Provenance::Conclude {
        who: dispatch_identity(),
    };
    match commit_on_behalf(coord_root, tip, &tree, &funnel_message(slice, phase), &prov)? {
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

    // --- VT-2: commit_on_behalf (compose + empty-delta + lost-ref-race) ---------------

    #[test]
    fn commit_on_behalf_happy_lands_one_non_merge_commit() {
        let (_tmp, _primary, coord, base, base_tree) = primary_with_coord(199);
        let tree = git::tree_with_file(&coord, &base_tree, "added.txt", "hi\n").unwrap();
        let out = commit_on_behalf(&coord, &base, &tree, "m", &disp()).unwrap();
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
        let out = commit_on_behalf(&coord, &base, &base_tree, "m", &disp()).unwrap();
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
        let out = commit_on_behalf(&coord, &base, &tree, "m", &disp()).unwrap();
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
        let oid = match commit_on_behalf(&coord, &base, &tree, &msg, &prov).unwrap() {
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
        let oid = match commit_on_behalf(&coord, &base, &tree, &msg, &disp()).unwrap() {
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
        let oid = match commit_on_behalf(&coord, &base, &tree, "m", &disp()).unwrap() {
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
        let out = commit_on_behalf(&coord, &base, &tree, "m", &disp()).unwrap();
        assert_eq!(out, CommitOutcome::Refused(CommitRefusal::LostRefRace));
        // Ref, index, and worktree are all byte-unchanged.
        assert_eq!(
            git_run(&coord, &["rev-parse", "refs/heads/dispatch/199"]),
            ref_before
        );
        assert_eq!(git_run(&coord, &["status", "--porcelain"]), status_before);
    }

    impl CommitOutcome {
        fn token_is(&self, want: &str) -> bool {
            matches!(self, CommitOutcome::Refused(r) if r.token() == want)
        }
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
}
