// SPDX-License-Identifier: GPL-3.0-only
//! coordination machine — extracted from worktree/mod.rs (SL-116 PHASE-03).

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::git;
use crate::root;
use anyhow::{Context, bail};

use super::fork::{remove_worktree_dir, rollback_fork};
use super::provision::run_provision;
use super::shared::{gather_fork_worktree, matches, resolve_commit};

/// The outcome of a successful coordination setup (SL-085, design D9).
pub(crate) struct CoordOutcome {
    /// Abbreviated commit hash of the dispatch branch tip after setup.
    pub dispatch_tip: String,
}

/// Peel a base/head ref to its canonical commit sha for the stationarity compare.
/// `rev-parse --verify <ref>^{commit}` resolves a sha, `HEAD`, a branch, or a
/// (lightweight/annotated) tag down to the commit it names; an unresolvable ref
/// `doctrine worktree branch-point-check --base <REF> [--head <REF>]` — the
/// funnel's one tested seam (SL-031 §5.2). Asserts coordination HEAD has not moved
/// off the orchestrator's pre-spawn base before the batch commit.
///
/// **Both** ends are resolved to a commit sha in the shell via [`resolve_commit`]
/// before the compare (`--head` absent ⇒ `HEAD`); a symbolic ref is never trusted
/// verbatim, and an unresolvable ref makes the verb bail (ISS-002 / SL-041). Exit
/// **0** on stationarity (resolved `base == head`), **1** otherwise (the
/// orchestrator re-dispatches the batch onto the moved HEAD — never commits on a
/// moved base). Read-classed (no authored write): callable under worker-mode,
/// though only the orchestrator drives it. C-V: ref-equality, not a merge-base —
/// see [`matches`].
pub(crate) fn run_branch_point_check(
    path: Option<PathBuf>,
    base: &str,
    head: Option<String>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let head = head.unwrap_or_else(|| "HEAD".to_owned());
    let base_sha = resolve_commit(&root, base)?;
    let head_sha = resolve_commit(&root, &head)?;
    if matches(&base_sha, &head_sha) {
        writeln!(io::stdout(), "stationary: HEAD == base {base_sha}")?;
        Ok(())
    } else {
        bail!("HEAD moved: base {base_sha} != HEAD {head_sha}");
    }
}

/// The coordination-create action the pure classifier selects (SL-064 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoordAction {
    /// `dispatch/<slice>` does not exist ⇒ create it fresh on a new branch off
    /// the integration base (trunk).
    Create,
    /// `dispatch/<slice>` exists with NO live linked worktree ⇒ a handover
    /// resume: reattach a worktree to the SAME branch (design §1 resume
    /// stability), never fork a second coordination branch.
    Resume,
}

/// The coordination-create refusal set. Distinct token; fails closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoordRefusal {
    /// `dispatch/<slice>` already has a LIVE linked worktree ⇒ a concurrent
    /// same-slice dispatch is live; refuse before mutating refs/dirs (never
    /// silently create a second coordination branch — EX-3).
    LiveWorktree,
}

impl CoordRefusal {
    /// The distinct named token this refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            CoordRefusal::LiveWorktree => "coordination-live",
        }
    }
}

/// PURE coordination-create classifier (no git/disk/env — ADR-001 leaf, CLAUDE.md
/// pure/imperative split). The branch-existence vs live-worktree discriminator
/// (design §1/§2): a mere branch is a resumable handover; a LIVE worktree is a
/// concurrent run. The worker marker is irrelevant HERE — a coordination tree
/// never bears it, and the worker-mode refusal (EX-4) is the Orchestrator-class
/// guard at the invocation site, not this classifier.
pub(crate) fn classify_coordinate(
    exists: bool,
    has_live_worktree: bool,
) -> Result<CoordAction, CoordRefusal> {
    match (exists, has_live_worktree) {
        (false, _) => Ok(CoordAction::Create),
        (true, true) => Err(CoordRefusal::LiveWorktree),
        (true, false) => Ok(CoordAction::Resume),
    }
}

/// Does the dispatched slice's `plan.toml` exist on the chosen trunk `base`'s
/// tree? Probes `git ls-tree <base> -- .doctrine/slice/<NNN>/plan.toml`: a path
/// the tree carries lists itself, an absent path lists nothing. `git_opt` yields
/// `None` only on non-zero exit, so an absent file (exit 0, empty stdout) arrives
/// as `Some("")` — both empty arms mean "absent" (F6).
pub(crate) fn base_has_slice_plan(root: &Path, base: &str, slice: u32) -> anyhow::Result<bool> {
    let pathspec = format!(".doctrine/slice/{slice:03}/plan.toml");
    let listing = git::git_opt(root, &["ls-tree", base, "--", &pathspec])?;
    Ok(listing.is_some_and(|out| !out.is_empty()))
}

/// Pure-ish core: form the dispatch coordination worktree, provision it, and
/// regenerate the runtime phase sheets. Returns the abbreviated dispatch tip.
/// No stdout/stderr — I/O lives in [`run_coordinate`].
pub(crate) fn coordinate(root: &Path, slice: u32, dir: &Path) -> anyhow::Result<CoordOutcome> {
    let branch = format!("dispatch/{slice:03}");

    // --- Step 1 refusal (pre-add: leave NO worktree) ---
    if dir.exists() {
        bail!("coordinate-refused: dir {} already exists", dir.display());
    }

    // --- gather: branch existence + its live linked worktree (if any) ---
    let exists = git::git_opt(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}^{{commit}}"),
        ],
    )?
    .is_some();
    let live_worktree = gather_fork_worktree(root, &branch)?;

    // --- pure classify (create / resume / refuse) ---
    let action = match classify_coordinate(exists, live_worktree.is_some()) {
        Ok(action) => action,
        Err(refusal) => {
            let at = live_worktree
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            bail!(
                "coordinate-refused: {} — {branch} has a live worktree at {at}",
                refusal.token()
            );
        }
    };

    // --- act: add the worktree (create off trunk vs resume the same branch) ---
    match action {
        CoordAction::Create => {
            let trunk = git::trunk_commit(root)?.ok_or_else(|| {
                anyhow::anyhow!(
                    "coordinate-refused: no trunk ref resolves (set DOCTRINE_TRUNK_REF)"
                )
            })?;
            // plan.toml for the dispatched slice must exist on the chosen base,
            // else the off-trunk fork would regen phase sheets against a tree
            // that predates the slice's own plan (ISS-036). Gate BEFORE the fork
            // so no worktree is created and the rollback path is never entered.
            if !base_has_slice_plan(root, &trunk, slice)? {
                bail!(
                    "coordinate-refused: base {trunk} lacks .doctrine/slice/{slice:03}/plan.toml \
                     — the trunk base predates this slice's plan; set DOCTRINE_TRUNK_REF to a base \
                     that carries it (e.g. DOCTRINE_TRUNK_REF=main)"
                );
            }
            git::git_text(
                root,
                &[
                    "worktree",
                    "add",
                    "-b",
                    &branch,
                    &dir.to_string_lossy(),
                    &trunk,
                ],
            )
            .with_context(|| format!("git worktree add -b {branch} {} {trunk}", dir.display()))?;
        }
        CoordAction::Resume => {
            git::git_text(root, &["worktree", "add", &dir.to_string_lossy(), &branch])
                .with_context(|| format!("git worktree add {} {branch}", dir.display()))?;
        }
    }

    // From here on, any failure compensates. Create rolls back the branch it
    // minted; Resume KEEPS the pre-existing branch (only its worktree is removed).
    let finish = (|| -> anyhow::Result<()> {
        run_provision(Some(root.to_path_buf()), dir).context("provision coordination worktree")?;
        crate::slice::run_phases(Some(dir.to_path_buf()), slice, false)
            .context("regenerate runtime phase sheets")?;
        Ok(())
    })();

    if let Err(cause) = finish {
        let debris = match action {
            CoordAction::Create => rollback_fork(root, &branch, dir),
            CoordAction::Resume => remove_worktree_dir(root, dir),
        };
        if debris.is_empty() {
            return Err(cause.context(format!(
                "coordinate failed after add; rolled back cleanly (worktree {} removed)",
                dir.display()
            )));
        }
        bail!(
            "coordinate-rollback-debris: {} (original cause: {cause:#})",
            debris.join(", ")
        );
    }

    // Resolve the dispatch branch tip (abbreviated commit hash).
    let dispatch_tip = git::git_text(
        root,
        &["rev-parse", "--short", &format!("refs/heads/{branch}")],
    )?;

    Ok(CoordOutcome { dispatch_tip })
}

/// `doctrine worktree coordinate --slice <n> --dir <path>` — create or resume the
/// dispatch coordination worktree for a slice (SL-064 §2). MARKERLESS: the
/// coordination tree IS the orchestrator (worker-mode OFF, must write), so it
/// stamps NO worker marker — its write permission rests on marker-absence (D2a),
/// never on a positive coordination marker (that is OQ-D / IMP-065, deferred).
///
/// Thin wrapper over [`coordinate`]: resolves the repo root, calls into the
/// pure-ish core, then reports human status on stderr (the fork builds into its own
/// in-tree `target/` — no env contract on stdout; SL-156). The existing integration
/// tests (`e2e_worktree_coordinate`) must stay green.
///
/// Orchestrator-classed; refused under worker-mode by `worker_guard` (EX-4) — the
/// marker-present / `DOCTRINE_WORKER` refusals ride the SAME guard as `fork`.
pub(crate) fn run_coordinate(path: Option<PathBuf>, slice: u32, dir: &Path) -> anyhow::Result<()> {
    let repo = root::find(path, &root::default_markers())?;
    let branch = format!("dispatch/{slice:03}");

    // Pre-probe: was the branch already there? Drives the stderr verb (create vs
    // resume) without leaking classification into the extracted core.
    let branch_existed = git::git_opt(
        &repo,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}^{{commit}}"),
        ],
    )?
    .is_some();

    let _outcome = coordinate(&repo, slice, dir)?;

    // --- human status on stderr; stdout stays empty (machine-clean, mirrors `fork`) ---
    let verb = if branch_existed { "resumed" } else { "created" };
    writeln!(
        io::stderr(),
        "coordination worktree {verb}: {branch} → {} (markerless)",
        dir.display()
    )?;
    Ok(())
}
