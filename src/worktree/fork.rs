// SPDX-License-Identifier: GPL-3.0-only
//! fork machine — extracted from worktree/mod.rs (SL-116 PHASE-02).

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

/// The byte-identical creation CORE — *add + provision + mark* with compensating
/// rollback, SHARED by [`run_fork`] (the CLI verb: status to stderr, empty stdout)
/// and [`super::create::act_on_create`] (the `WorktreeCreate`
/// hook: prints the created path alone). The D11 split: this core is SILENT — it
/// writes NOTHING to stdout/stderr; refusals and rollback debris surface only as
/// `Err`. The `repo` (provision source + git `-C` root) is passed in EXPLICITLY,
/// never `root::find`-resolved here, so each caller controls the source tree (D1/D11).
///
/// Atomic via COMPENSATING ROLLBACK (not a git transaction): any failure AFTER the
/// `git worktree add` reverses every leg via [`rollback_fork`]. A pre-`add` refusal
/// (dir/branch exists, `B` not a commit) leaves no fork.
pub(super) fn fork_core(
    repo: &Path,
    base: &str,
    branch: &str,
    dir: &Path,
    worker: bool,
) -> anyhow::Result<()> {
    // --- Step 1 refusals (pre-`add`: leave NO fork) ---
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

    // --- Step 1: create the worktree on a NEW branch at B ---
    git::git_text(
        repo,
        &[
            "worktree",
            "add",
            "-b",
            branch,
            &dir.to_string_lossy(),
            base,
        ],
    )
    .with_context(|| format!("git worktree add -b {branch} {} {base}", dir.display()))?;

    // From here on, any failure compensates (rollback every leg).
    let finish = (|| -> anyhow::Result<()> {
        // --- Step 2: provision via the sole copier (do NOT reimplement copying) ---
        run_provision(Some(repo.to_path_buf()), dir).context("provision fork")?;

        // --- Step 3: stamp the worker marker BEFORE returning / any spawn window ---
        if worker {
            write_marker(dir).context("stamp worker marker")?;
        }
        Ok(())
    })();

    if let Err(cause) = finish {
        let debris = rollback_fork(repo, branch, dir);
        if debris.is_empty() {
            return Err(cause.context(format!(
                "fork failed after add; rolled back cleanly (dir {} + branch {branch} removed)",
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

/// `doctrine worktree fork --base <B> --branch <name> --dir <path> [--worker]` —
/// create an orchestrator-owned worktree fork off `B`, provision it, and optionally
/// stamp the worker marker (design §5). The creation work is [`fork_core`]; this CLI
/// shell adds only the human status line. The fork compiles into its own in-tree
/// `<dir>/target` — no env contract is emitted (SL-156: platform exited the
/// build-env business).
///
/// - **stdout**: empty (machine-clean).
/// - **stderr**: human status (what it did).
pub(crate) fn run_fork(
    path: Option<PathBuf>,
    base: &str,
    branch: &str,
    dir: &Path,
    worker: bool,
) -> anyhow::Result<()> {
    let repo = root::find(path, &root::default_markers())?;
    fork_core(&repo, base, branch, dir, worker)?;

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
