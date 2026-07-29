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

/// The repo's shared gitdir (`git rev-parse --git-common-dir`), canonicalized.
/// Identifies the repository behind `root`: two worktrees of the SAME repo share
/// one common-dir, whereas a sibling repo's worktree resolves to a different one.
/// SL-182 PHASE-03 uses it to confirm a subagent `cwd` belongs to THIS project
/// (matched against `CLAUDE_PROJECT_DIR`'s common-dir), the git-topology check
/// that replaces the probe's hard-coded `.worktrees/` path prefix (A1).
pub(super) fn common_git_dir(root: &Path) -> anyhow::Result<PathBuf> {
    resolve_common_dir(
        root,
        &git::git_text(root, &["rev-parse", "--git-common-dir"])?,
    )
}

/// True iff `root` sits on a *linked* worktree rather than the primary tree:
/// `git rev-parse --git-dir` (this tree's gitdir) differs from `--git-common-dir`
/// (the repo's shared gitdir). On the primary tree both resolve to the same
/// `.git`; on a linked worktree the gitdir is `.git/worktrees/<name>` (SL-032
/// PHASE-04, ADR-006 amendment). Shared, not memory-private — the provision path
/// may call it; `memory record` calls it to warn on squash-orphan risk.
pub(crate) fn is_linked_worktree(root: &Path) -> anyhow::Result<bool> {
    let git_dir = resolve_common_dir(root, &git::git_text(root, &["rev-parse", "--git-dir"])?)?;
    let common = common_git_dir(root)?;
    Ok(git_dir != common)
}

/// The coordination branch's SHORT-ref prefix (`dispatch/<NNN>`, as
/// [`crate::git::current_branch`] returns it) — `DISPATCH_REF_PREFIX` sans the
/// `refs/heads/` qualifier. Single-sourced (STD-001) for the role classifier.
const COORD_BRANCH_SHORT_PREFIX: &str = "dispatch/";

/// PURE classification of a worktree into the funnel's three roles from its branch +
/// isolation (SL-228 design §8): the primary tree ⇒ `primary`; a linked worktree on a
/// `dispatch/<NNN>` coordination branch (NUMERIC slice suffix) ⇒ `coord`; any other
/// linked worktree — including a worker fork's `dispatch/<agent>` (non-numeric
/// suffix), a `review/*`, or a detached HEAD ⇒ `fork`. The numeric-suffix test is
/// load-bearing: coord and worker-fork branches SHARE the `dispatch/` prefix, so a
/// bare prefix match would misread every worker fork as a coord. No git/disk — the
/// shell gathers `branch`/`linked` and hands them in (pure/imperative split).
///
/// Lives here beside [`is_linked_worktree`] rather than in `dispatch` because two
/// command-tier callers need it: `dispatch whereami` reports it, and `review`'s
/// root guard admits `primary`/`coord` while refusing `fork` (ISS-275).
pub(crate) fn classify_worktree_role(branch: Option<&str>, linked: bool) -> &'static str {
    if !linked {
        return "primary";
    }
    if coord_branch_suffix(branch).is_some() {
        "coord"
    } else {
        "fork"
    }
}

/// The numeric suffix of a COORD-SHAPED branch (`dispatch/<NNN>`) — the ONE place the
/// `dispatch/` prefix and the load-bearing all-digits test live (STD-001). `None` for a
/// worker fork's non-numeric `dispatch/<agent>`, any other branch, or a detached HEAD.
/// Isolation is not consulted; shape only.
fn coord_branch_suffix(branch: Option<&str>) -> Option<&str> {
    branch
        .and_then(|b| b.strip_prefix(COORD_BRANCH_SHORT_PREFIX))
        .filter(|suffix| !suffix.is_empty() && suffix.bytes().all(|c| c.is_ascii_digit()))
}

/// PURE — the slice id a coordination branch names, iff `branch` is coord-shaped
/// (IMP-268). `None` for every non-coord branch, and for a numeric suffix too large to
/// be a slice id. Shares [`coord_branch_suffix`] with [`classify_worktree_role`], so the
/// prefix rule is stated once: a caller pairing this with `linked` gets the same coord
/// verdict the role classifier gives.
pub(crate) fn coord_branch_slice(branch: Option<&str>) -> Option<u32> {
    coord_branch_suffix(branch)?.parse().ok()
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
// project anchor (CLAUDE_PROJECT_DIR) — SL-182 PHASE-03 / SL-206 PHASE-11
// ---------------------------------------------------------------------------

/// Harness-supplied project-root anchor (`docs/claude/hooks.md:462`). SINGLE
/// SOURCE (STD-001): every hook handler that needs the FIXED, cwd-independent
/// project root reads this ONE env var — `pretooluse`'s topology check, and
/// SL-206's `nominate`/`denominate`/spawn-gate, which all fire in a hook
/// process whose `cwd` may be the SPAWNED subagent's own tree, never the
/// project root (design §5.6 / §4.3-1).
pub(crate) const ENV_PROJECT_DIR: &str = "CLAUDE_PROJECT_DIR";

/// The harness-supplied project-root anchor, realpath'd. `None` ⇒ absent or
/// uncanonicalizable ⇒ every consumer fails closed (never falls back to `cwd`,
/// which a hook cannot trust — design §4.3-1).
pub(crate) fn project_anchor() -> Option<PathBuf> {
    let raw = std::env::var_os(ENV_PROJECT_DIR)?;
    fs::canonicalize(PathBuf::from(raw)).ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // --- SL-228 PHASE-01: whereami role classifier (pure — design §8).
    //     Relocated here from `dispatch` with the function (ISS-275). ---

    #[test]
    fn classify_worktree_role_maps_branch_and_isolation() {
        // The primary tree is `primary` regardless of branch.
        assert_eq!(classify_worktree_role(Some("main"), false), "primary");
        assert_eq!(classify_worktree_role(None, false), "primary");
        assert_eq!(
            classify_worktree_role(Some("dispatch/228"), false),
            "primary"
        );
        // A linked worktree on a NUMERIC-suffix coordination branch is `coord`.
        assert_eq!(classify_worktree_role(Some("dispatch/228"), true), "coord");
        // A worker fork shares the `dispatch/` prefix but has a NON-numeric agent
        // suffix — it must classify as `fork`, not `coord` (the load-bearing case).
        assert_eq!(
            classify_worktree_role(Some("dispatch/agent-abc"), true),
            "fork"
        );
        // Any other linked worktree is a `fork` — a `review/*` ref or a detached HEAD.
        assert_eq!(classify_worktree_role(Some("review/064"), true), "fork");
        assert_eq!(classify_worktree_role(None, true), "fork");
    }

    // --- IMP-268: the coord branch's slice id, over the SAME shape test ---

    #[test]
    fn coord_branch_slice_reads_the_id_only_from_a_coord_shaped_branch() {
        // Zero-padded and bare forms both parse to the numeric id, so a caller can
        // compare against a bare `--slice N` without re-formatting.
        assert_eq!(coord_branch_slice(Some("dispatch/228")), Some(228));
        assert_eq!(coord_branch_slice(Some("dispatch/007")), Some(7));
        assert_eq!(coord_branch_slice(Some("dispatch/7")), Some(7));

        // Every non-coord shape yields None — including the worker fork that shares the
        // prefix, which is exactly the case a bare prefix match would misread.
        for branch in [
            Some("dispatch/agent-abc"),
            Some("dispatch/"),
            Some("main"),
            Some("edge"),
            Some("review/064"),
            None,
        ] {
            assert_eq!(coord_branch_slice(branch), None, "branch: {branch:?}");
        }

        // A digits-only suffix too large for a u32 keeps the coord ROLE (the shape test
        // passes) but yields no id — so a caller cross-checking the slice must treat
        // `None` as "cannot compare", never as a mismatch.
        assert_eq!(coord_branch_slice(Some("dispatch/99999999999999")), None);
        assert_eq!(
            classify_worktree_role(Some("dispatch/99999999999999"), true),
            "coord",
            "shape and parse disagree only here — documented, not a mismatch"
        );
    }

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
