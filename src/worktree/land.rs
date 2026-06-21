#![expect(unused, reason = "extraction; PHASE-03 prunes")]
// SPDX-License-Identifier: GPL-3.0-only
//! land machine — extracted from worktree/mod.rs (SL-116 PHASE-02).

use super::allowlist::{
    Allowlist, allowlist_violations, is_withheld, parse_allowlist, select_copies,
};
use super::marker::{DISPATCH_WORKER_AGENT_TYPE, marker_present, write_marker};
use super::shared::{
    gather_fork_worktree, gather_tree_clean, is_linked_worktree, matches, resolve_commit,
    resolve_common_dir, target_dir_for_branch,
};
use crate::fsutil::{self, CopyOutcome};
use crate::git;
use crate::root;
use anyhow::{Context, bail};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

/// Verdict of the PURE land classifier: the preconds hold ⇒ the shell may run the
/// `--no-ff` merge. Mirror of [`Apply`] for the import verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Merge {
    /// All four preconds hold ⇒ the shell drives `git merge --no-ff <fork>`.
    Ok,
}

/// The exhaustive `land` refusal set (design §6) — EXACTLY these 7, each a
/// distinct named token. The 4 PRECOND refusals are returned by the pure
/// [`classify_land`]; the 3 merge-time refusals are determined in the shell from
/// the `git merge` outcome + a `MERGE_HEAD` probe, but the enum carries all 7
/// variants so the shell can name them with one [`token`](LandRefusal::token)
/// table. Deliberately SEPARATE from [`Refusal`] — `land`'s beltless `--no-ff`
/// merge is a different verb from `import`'s belted apply; do NOT widen `Refusal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LandRefusal {
    /// Tracked tree dirty (`git status --porcelain --untracked-files=no` nonempty).
    TreeUnclean,
    /// `<fork>` branch does not exist.
    NoSuchFork,
    /// `<fork>` exists but has NO live linked worktree — its marker would be
    /// uncommitted/unreachable, so the dispatch-fork check would pass vacuously.
    WorktreeGone,
    /// `<fork>`'s live linked worktree bears the worker marker ⇒ it is a dispatch
    /// worker; its delta must funnel through the belted `import`, never `land`.
    DispatchFork,
    /// `git merge --no-ff <fork>` conflicted; the merge was aborted FIRST (tree
    /// restored clean), THEN refused.
    MergeConflict,
    /// `git merge --abort` itself FAILED — the tree is NOT clean; names `MERGE_HEAD`,
    /// the unmerged paths, and the manual remedy.
    WedgedMerge,
    /// Step 3 reached with NO merge in progress (`MERGE_HEAD` absent) — never a
    /// silent abort masquerading as a clean conflict.
    InconsistentMergeState,
}

impl LandRefusal {
    /// The distinct named token each refusal fails closed with (the property the
    /// VT goldens assert, not a proxy).
    pub(crate) fn token(self) -> &'static str {
        match self {
            LandRefusal::TreeUnclean => "tree-unclean",
            LandRefusal::NoSuchFork => "no-such-fork",
            LandRefusal::WorktreeGone => "worktree-gone",
            LandRefusal::DispatchFork => "dispatch-fork",
            LandRefusal::MergeConflict => "merge-conflict",
            LandRefusal::WedgedMerge => "wedged-merge",
            LandRefusal::InconsistentMergeState => "inconsistent-merge-state",
        }
    }
}

/// The gathered state of the `<fork>` branch the precond logic classifies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForkState {
    /// `<fork>` resolves to a commit (the branch exists).
    pub(crate) exists: bool,
    /// `<fork>` has a live linked worktree checked out (per `git worktree list`).
    pub(crate) has_live_worktree: bool,
    /// That live linked worktree bears the worker marker.
    pub(crate) bears_marker: bool,
}

/// PURE land classifier (no git / disk / env — ADR-001 leaf, CLAUDE.md
/// pure/imperative split). Mirror of [`classify_import`]: it takes the gathered
/// FACTS and returns the verdict or one of the 4 PRECOND refusals only.
///
/// * `tree_status_clean` — tracked tree clean (the SAME `--untracked-files=no`
///   scoping `import` uses, via [`gather_tree_clean`]).
/// * `_head` — documents the contextual "HEAD is the coordination branch" precond.
///   It is intentionally UNUSED by the 7-token logic (design §6: that precond
///   carries NO refusal token; the verb runs at the coordination root by contract).
///   Kept in the signature to preserve the design's `classify_land` shape.
/// * `fork_state` — `{exists, has_live_worktree, bears_marker}`.
///
/// Precond precedence (design §6): tree-unclean → no-such-fork → worktree-gone →
/// dispatch-fork. `worktree-gone` gates `dispatch-fork` — refuse the worktree-less
/// branch BEFORE the marker check can pass vacuously.
pub(crate) fn classify_land(
    tree_status_clean: bool,
    _head: &str,
    fork_state: ForkState,
) -> Result<Merge, LandRefusal> {
    if !tree_status_clean {
        return Err(LandRefusal::TreeUnclean);
    }
    if !fork_state.exists {
        return Err(LandRefusal::NoSuchFork);
    }
    if !fork_state.has_live_worktree {
        return Err(LandRefusal::WorktreeGone);
    }
    if fork_state.bears_marker {
        return Err(LandRefusal::DispatchFork);
    }
    Ok(Merge::Ok)
}

/// Gather the `<fork>` branch's live-linked-worktree path, if any, via the shared
/// `doctrine worktree land --fork <branch>` — solo `/execute`'s analog of
/// `import` (design §6, ADR-006). Lands a solo multi-commit isolated-worktree TDD
/// branch onto the coordination branch with ancestry PRESERVED via `git merge
/// --no-ff` (NEVER `--squash` — the verb cannot express a squash). Ancestry
/// preserved ⇒ fork commits reachable ⇒ gc's ancestry leg can later reap them;
/// squash is structurally uncertifiable by gc, so it is forbidden here.
///
/// Gather → pure-classify → act, patterned after [`run_import`]:
/// 1. gather the precond FACTS (tracked-tree cleanliness via the SHARED
///    [`gather_tree_clean`]; `<fork>` existence; its live-linked-worktree path via
///    [`gather_fork_worktree`]; the marker on that path via [`marker_present`]),
/// 2. [`classify_land`] returns `Ok(Merge)` or one of the 4 PRECOND refusals,
/// 3. on `Ok`, drive `git merge --no-ff <fork>`. On conflict → `git merge --abort`
///    FIRST (restore the clean tree), THEN refuse `merge-conflict`. The abort is
///    guarded to fire ONLY mid-merge (`MERGE_HEAD` present); step 3 with no merge
///    in progress → `inconsistent-merge-state`. Abort FAILURE → `wedged-merge`.
///
/// Orchestrator-classed; refused under worker-mode by `worker_guard`.
pub(crate) fn run_land(path: Option<PathBuf>, fork: &str) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    // --- gather: precond — tracked tree clean (the SHARED gather, untracked excluded) ---
    let tree_clean = gather_tree_clean(&root)?;

    // --- gather: precond — <fork> exists (resolves to a commit) ---
    let exists = git::git_opt(
        &root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{fork}^{{commit}}"),
        ],
    )?
    .is_some();

    // --- gather: precond — <fork>'s live linked worktree (path) + its marker ---
    let fork_wt = gather_fork_worktree(&root, fork)?;
    let has_live_worktree = fork_wt.is_some();
    let bears_marker = fork_wt.as_deref().is_some_and(marker_present);

    // --- gather: contextual — HEAD branch (documents the coordination-root precond) ---
    let head = git::git_text(&root, &["rev-parse", "--abbrev-ref", "HEAD"])?;

    // --- pure classify (the 4 PRECOND refusals) ---
    let fork_state = ForkState {
        exists,
        has_live_worktree,
        bears_marker,
    };
    match classify_land(tree_clean, &head, fork_state) {
        Err(refusal) => bail!("land-refused: {}", refusal.token()),
        Ok(Merge::Ok) => {}
    }

    // --- act: git merge --no-ff <fork> (NEVER --squash) ---
    let merged = git::git_opt(&root, &["merge", "--no-ff", "--no-edit", fork])?;
    if merged.is_some() {
        writeln!(
            io::stdout(),
            "landed {fork}: --no-ff merge onto coordination HEAD"
        )?;
        return Ok(());
    }

    // --- merge failed: classify the merge-time refusal from MERGE_HEAD + abort ---
    // Guard the abort to fire ONLY mid-merge: a failed merge with no MERGE_HEAD is
    // an inconsistent state, never a silent abort masquerading as a clean conflict.
    let mid_merge =
        git::git_opt(&root, &["rev-parse", "--verify", "--quiet", "MERGE_HEAD"])?.is_some();
    if !mid_merge {
        bail!(
            "land-refused: {}",
            LandRefusal::InconsistentMergeState.token()
        );
    }

    // Mid-merge: capture the unmerged paths, then abort FIRST to restore the tree.
    let unmerged = git::git_text(&root, &["diff", "--name-only", "--diff-filter=U"])?;
    let aborted = git::git_opt(&root, &["merge", "--abort"])?;
    if aborted.is_some() {
        // Abort SUCCESS ⇒ ordinary merge-conflict; the tree is guaranteed clean.
        bail!("land-refused: {}", LandRefusal::MergeConflict.token());
    }

    // Abort FAILURE ⇒ wedged: the tree is NOT clean. Name MERGE_HEAD, the unmerged
    // paths, and the manual remedy.
    bail!(
        "land-refused: {token} — `git merge --abort` failed; MERGE_HEAD is present and the tree is NOT clean. Unmerged paths:\n{unmerged}\nManual remedy: resolve in place and `git commit`, or `git merge --abort` / `git reset --hard {head}` from the coordination root.",
        token = LandRefusal::WedgedMerge.token(),
    )
}
