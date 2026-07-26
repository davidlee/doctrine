// SPDX-License-Identifier: GPL-3.0-only
//! fork machine — extracted from worktree/mod.rs (SL-116 PHASE-02).

use super::dispatch_record::{ForkBinding, coord_and_name};
use super::marker::write_marker;
use super::provision::run_provision;
use crate::git;
use crate::root;
use anyhow::{Context, bail};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Best-effort compensating cleanup for a partially-created fork (design §5 — git
/// mutations are NOT a transaction, so there is no rollback verb to lean on; we
/// reverse each leg ourselves). SHARED by [`run_fork`]/[`fork_core`] and SL-152's
/// `create-fork` (one cleanup impl, several callers — a hard reuse requirement).
///
/// Each leg is best-effort and independent: a failing leg must not mask the
/// original cause or abort the others. Removing a never-added worktree / dir / a
/// non-existent branch is a no-op, not an error. Returns the list of legs that
/// FAILED to reverse (debris descriptions) so the caller can decide whether the
/// rollback left leftovers needing a distinct, naming exit.
/// Remove a linked worktree registration and reap any leftover dir (best-effort).
/// The branch-agnostic half of [`rollback_fork`], shared with the SL-064
/// coordinate Resume rollback (which must KEEP the pre-existing branch). Returns
/// surviving debris.
pub(super) fn remove_worktree_dir(repo: &Path, dir: &Path) -> Vec<String> {
    let mut debris = Vec::new();

    // Remove the linked worktree registration (force: drop dirty/locked).
    if git::git_text(
        repo,
        &["worktree", "remove", "--force", &dir.to_string_lossy()],
    )
    .is_err()
        && dir.exists()
    {
        // Only debris if the dir actually survives — a "not a worktree" error on a
        // never-added dir is the expected no-op.
        debris.push(format!("worktree dir {}", dir.display()));
    }

    // Reap any leftover dir the worktree-remove could not (best-effort). On a
    // SUCCESSFUL reap, RETRACT any stale `worktree dir {dir}` entry — the dir is
    // gone, so a fully-cleaned rollback must report empty debris, never false-bail
    // over a tree it did clean (F-8).
    if dir.exists() {
        drop(fs::remove_dir_all(dir));
        let dir_str = dir.display().to_string();
        if dir.exists() {
            if !debris.iter().any(|d| d.contains(&dir_str)) {
                debris.push(format!("dir {dir_str}"));
            }
        } else {
            debris.retain(|d| !d.contains(&dir_str));
        }
    }

    debris
}

pub(super) fn rollback_fork(repo: &Path, branch: &str, dir: &Path) -> Vec<String> {
    // 1+3. Remove the worktree registration + reap the dir (shared half).
    let mut debris = remove_worktree_dir(repo, dir);

    // 1b (SL-228 PHASE-04). Reverse the BIND too: with the record now written BEFORE
    // the worktree act, a failure after the bind would otherwise strand a record with
    // no worktree. Keyed off the fork NAME exactly as gc's reap is (one shape, no
    // re-spell); absent ⇒ no-op, so a pre-bind failure and a non-dispatch branch both
    // cost nothing. Best-effort: rollback never masks the original cause.
    if let Some(name) = branch.strip_prefix("dispatch/") {
        drop(super::dispatch_record::delete_dispatch_record(repo, name));
    }

    // 2. Delete the branch (no-op if it was never created).
    if git::git_opt(repo, &["rev-parse", "--verify", "--quiet", branch])
        .ok()
        .flatten()
        .is_some()
    {
        // Best-effort delete; the re-probe below is what decides debris, so a
        // failed delete here is intentionally not propagated.
        drop(git::git_text(repo, &["branch", "-D", branch]));
        if git::git_opt(repo, &["rev-parse", "--verify", "--quiet", branch])
            .ok()
            .flatten()
            .is_some()
        {
            debris.push(format!("branch {branch}"));
        }
    }

    debris
}

/// The byte-identical creation CORE, SHARED by [`run_fork`] (the CLI verb: status to
/// stderr, empty stdout) and [`super::create::act_on_create`] (the `WorktreeCreate`
/// hook: prints the created path alone). The D11 split: this core is SILENT — it
/// writes NOTHING to stdout/stderr; refusals and rollback debris surface only as
/// `Err`. The `repo` (provision source + git `-C` root) is passed in EXPLICITLY,
/// never `root::find`-resolved here, so each caller controls the source tree (D1/D11).
///
/// # `claim → bind → act` (SL-228 PHASE-04, design §3 / RV-304 F-2)
///
/// The sequence is ORDERED so that **a live fork implies its binding by construction**:
///
/// 1. **claim** — create the branch ref at `B`. Git ref creation is ATOMIC, so the ref
///    IS the claim: of two concurrent same-name spawns exactly one wins and the loser
///    refuses `fork-refused: branch … already exists`. (A spawn never ADOPTS an existing
///    branch, which is what makes the claim exclusive rather than advisory.)
/// 2. **bind** — `bind()` writes the coord-side per-worktree state (the dispatch record
///    and jail policy) UNDER the held claim. No concurrent writer for the name can
///    exist here, and the caller's bind keeps its own no-clobber belt regardless.
/// 3. **act** — the worktree is created LAST, so there is no observable window in which
///    a live worktree exists without its binding. The pre-reorder order (act, then bind)
///    left exactly that window: a crash inside it stranded a live fork as permanently
///    `unprovable-fork`.
///
/// The window is additionally held under the per-name CLAIM LOCK by the dispatch caller
/// (see `super::claim_lock`), which closes the spawn–gc race the branch claim alone
/// cannot: gc must not mistake an active claimant's branch for crash residue.
///
/// Atomic via COMPENSATING ROLLBACK (not a git transaction): any failure AFTER the
/// claim reverses EVERY leg via [`rollback_fork`] — including the claim itself, so a
/// rolled-back fork leaves no branch residue. A pre-claim refusal (dir/branch exists,
/// `B` not a commit) leaves no fork at all.
pub(super) fn fork_core(
    repo: &Path,
    base: &str,
    branch: &str,
    dir: &Path,
    worker: bool,
    bind: &mut dyn FnMut() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    // --- Step 0 refusals (pre-claim: leave NO fork) ---
    if dir.exists() {
        bail!("fork-refused: dir {} already exists", dir.display());
    }
    if git::git_opt(repo, &["rev-parse", "--verify", "--quiet", branch])
        .ok()
        .flatten()
        .is_some()
    {
        bail!("fork-refused: branch {branch} already exists");
    }
    if git::git_opt(
        repo,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{base}^{{commit}}"),
        ],
    )?
    .is_none()
    {
        bail!("fork-refused: base {base} is not a commit");
    }

    // --- Step 1: CLAIM — create the branch ref at B (atomic) ---
    // The pre-check above only makes the refusal FRIENDLY; the ref creation is the real
    // gate, so a same-name racer that slipped between the two lands here and refuses
    // with the identical token (no TOCTOU hole between check and claim).
    if let Err(cause) = git::git_text(repo, &["branch", branch, base]) {
        if git::git_opt(repo, &["rev-parse", "--verify", "--quiet", branch])
            .ok()
            .flatten()
            .is_some()
        {
            bail!("fork-refused: branch {branch} already exists");
        }
        return Err(anyhow::Error::new(cause).context(format!(
            "claim branch {branch} at {base}: git branch failed"
        )));
    }

    // From here on, any failure compensates (rollback every leg, claim included).
    let finish = (|| -> anyhow::Result<()> {
        // --- Step 2: BIND — the coord-side binding, written under the held claim ---
        bind().context("bind the fork")?;

        // --- Step 3: ACT — create the worktree LAST, on the already-claimed branch ---
        git::git_text(repo, &["worktree", "add", &dir.to_string_lossy(), branch])
            .with_context(|| format!("git worktree add {} {branch}", dir.display()))?;

        // --- Step 4: provision via the sole copier (do NOT reimplement copying) ---
        run_provision(Some(repo.to_path_buf()), dir).context("provision fork")?;

        // --- Step 5: stamp the worker marker BEFORE returning / any spawn window ---
        if worker {
            write_marker(dir).context("stamp worker marker")?;
        }
        Ok(())
    })();

    if let Err(cause) = finish {
        let debris = rollback_fork(repo, branch, dir);
        if debris.is_empty() {
            return Err(cause.context(format!(
                "fork failed after the claim; rolled back cleanly (dir {} + branch {branch} removed)",
                dir.display()
            )));
        }
        // Rollback itself left leftovers — distinct token NAMING the debris.
        bail!(
            "fork-rollback-debris: {} (original cause: {cause:#})",
            debris.join(", ")
        );
    }

    Ok(())
}

/// `doctrine worktree fork --base <B> --branch <name> --dir <path> [--worker]
/// [--slice N --phase PHASE-NN]` — create an orchestrator-owned worktree fork off `B`,
/// provision it, and optionally stamp the worker marker (design §5). The creation work
/// is [`fork_core`]; this CLI shell adds only the human status line. The fork compiles
/// into its own in-tree `<dir>/target` — no env contract is emitted (SL-156: platform
/// exited the build-env business).
///
/// **The durable fork binding (SL-228 PHASE-04, design §3).** This is the SUBPROCESS
/// arm's fork verb, the sibling of the claude arm's `create-fork`; like it, it binds
/// the fork to its `(slice, phase)` when it can. It binds when ALL of: `--worker`,
/// both `--slice` and `--phase` supplied, and `dir` matching the `<coord>/.worktrees/
/// <name>` layout the resolver strips (a fork outside that layout can never be
/// resolved, so a record for it would be unreachable state, not a binding). Otherwise
/// the bind is a NO-OP and the fork is simply unbound — which downstream verbs report
/// honestly as `unprovable-fork`, never guess around.
///
/// - **stdout**: empty (machine-clean).
/// - **stderr**: human status (what it did).
pub(crate) fn run_fork(
    path: Option<PathBuf>,
    base: &str,
    branch: &str,
    dir: &Path,
    worker: bool,
    binding: Option<&ForkBinding>,
) -> anyhow::Result<()> {
    let repo = root::find(path, &root::default_markers())?;

    // Resolve the coord/name the resolver would recover from `dir`, so a bind only ever
    // writes a record the resolver can actually reach. `None` ⇒ no bind (see above).
    let anchor = worker
        .then(|| binding.zip(coord_and_name(dir)))
        .flatten()
        .map(|(bound, (coord, name))| (bound, coord, name));

    let mut bind = || -> anyhow::Result<()> {
        let Some((bound, coord, name)) = anchor.as_ref() else {
            return Ok(());
        };
        super::dispatch_record::bind_dispatch_record(coord, name, base, dir, branch, Some(bound))
    };
    fork_core(&repo, base, branch, dir, worker, &mut bind)?;

    // --- human status on stderr; stdout stays empty (machine-clean) ---
    writeln!(
        io::stderr(),
        "forked {branch} at {base} → {}{}",
        dir.display(),
        if worker {
            " (worker: marker stamped)"
        } else {
            ""
        }
    )?;
    Ok(())
}
