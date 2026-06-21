// SPDX-License-Identifier: GPL-3.0-only
//! Cross-cutting shared helpers — worktree detection, commit resolution, branch-point
//! equality, and the lowest-level gathering primitives.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::git;

// ---------------------------------------------------------------------------
// Pure ref-equality compare
// ---------------------------------------------------------------------------

pub(crate) fn matches(base: &str, head: &str) -> bool {
    base == head
}

// ---------------------------------------------------------------------------
// shared gathering helpers
// ---------------------------------------------------------------------------

pub(super) fn resolve_common_dir(root: &Path, common: &str) -> anyhow::Result<PathBuf> {
    let raw = Path::new(common);
    let joined = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        root.join(raw)
    };
    fs::canonicalize(&joined)
        .with_context(|| format!("canonicalize git-common-dir {}", joined.display()))
}

/// True iff `root` sits on a *linked* worktree rather than the primary tree:
/// `git rev-parse --git-dir` (this tree's gitdir) differs from `--git-common-dir`
/// (the repo's shared gitdir). On the primary tree both resolve to the same
/// `.git`; on a linked worktree the gitdir is `.git/worktrees/<name>` (SL-032
/// PHASE-04, ADR-006 amendment). Shared, not memory-private — the provision path
/// may call it; `memory record` calls it to warn on squash-orphan risk.
pub(crate) fn is_linked_worktree(root: &Path) -> anyhow::Result<bool> {
    let git_dir = resolve_common_dir(root, &git::git_text(root, &["rev-parse", "--git-dir"])?)?;
    let common = resolve_common_dir(
        root,
        &git::git_text(root, &["rev-parse", "--git-common-dir"])?,
    )?;
    Ok(git_dir != common)
}

pub(super) fn resolve_commit(root: &Path, reference: &str) -> anyhow::Result<String> {
    Ok(git::git_text(
        root,
        &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
    )?)
}

pub(super) fn gather_tree_clean(root: &Path) -> anyhow::Result<bool> {
    Ok(git::tree_clean(root)?)
}

pub(super) fn gather_fork_worktree(root: &Path, fork: &str) -> anyhow::Result<Option<PathBuf>> {
    Ok(git::worktree_for_ref(root, &format!("refs/heads/{fork}"))?)
}

pub(crate) fn target_dir_for_branch(branch: &str) -> PathBuf {
    Path::new("wt").join(branch)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // --- branch-point-check pure compare (SL-031 PHASE-02, VT-1) ---

    #[test]
    fn matches_is_ref_equality() {
        assert!(matches("abc123", "abc123"), "equal shas ⇒ stationary");
        assert!(!matches("abc123", "def456"), "differing shas ⇒ moved");
        assert!(!matches("abc123", ""), "empty head ⇒ moved");
        assert!(
            matches("", ""),
            "degenerate equal ⇒ stationary (caller guards emptiness)"
        );
    }

    // --- SL-056 PHASE-06: target_dir_for_branch pure mapping (VT-3 unit half) ---

    #[test]
    fn target_dir_for_branch_maps_under_wt() {
        assert_eq!(
            target_dir_for_branch("sl056-p06"),
            PathBuf::from("wt/sl056-p06"),
            "branch maps to wt/<branch>"
        );
        assert_eq!(
            target_dir_for_branch("feature/x"),
            PathBuf::from("wt/feature/x"),
            "slashes in the branch survive as nested components"
        );
    }
}
