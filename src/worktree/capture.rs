#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-182 PHASE-05 T1 pure surface; the T2 SubagentStop shell + T3 command wire it"
    )
)]
// SPDX-License-Identifier: GPL-3.0-only
//! `SubagentStop` capture correlation — PURE path logic (SL-182 PHASE-05 T1).
//!
//! `SubagentStop` carries no `worktree_path` (RV-202), so the capture hook must
//! DERIVE which worktree to `git -C <wt> diff`. This leaf holds the pure pieces —
//! coord-root recovery, the captured-patch destination, and the correlator — with
//! every git/disk touch injected as input (CLAUDE.md pure/imperative split). The
//! T2 shell gathers the facts (the real `is_linked_worktree`, the diff) and acts.

use super::create::WORKTREES_SUBDIR;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Runtime-tier home for captured worker patches under the coord-tree root:
/// `<coord>/.doctrine/state/dispatch/capture/<name>.patch` — OUTSIDE every worktree
/// (gitignored, `rm -rf`-able), name-keyed. Sits beside [`JAIL_SUBPATH`] /
/// [`ARMING_SUBPATH`] (STD-001 named constant, design D-capture-path).
pub(crate) const CAPTURE_SUBPATH: &str = ".doctrine/state/dispatch/capture";

/// Captured-patch file extension (STD-001; the funnel's `--patch` import reads it).
const PATCH_EXT: &str = "patch";

/// Recover the coordination-tree root from a worker worktree path by stripping the
/// `.worktrees/<name>` layout. The git-common-dir points at the PRIMARY `.git`, NOT
/// the coord root where `.worktrees/` + the jail/capture dirs live
/// (`mem.fact.dispatch.coord-root-not-git-common-dir`), so recovery is by LAYOUT,
/// not git topology. Reuses [`WORKTREES_SUBDIR`] — one owner of the shape (design
/// §5.3, no re-spell). `None` if `worktree` is not `<coord>/.worktrees/<name>` shaped.
pub(crate) fn coord_root_from_worktree(worktree: &Path) -> Option<PathBuf> {
    let worktrees_dir = worktree.parent()?;
    if worktrees_dir.file_name()? != OsStr::new(WORKTREES_SUBDIR) {
        return None;
    }
    Some(worktrees_dir.parent()?.to_path_buf())
}

/// The captured-patch destination for a worktree `name` under `coord_root`
/// (design D-capture-path). Pure path join.
pub(crate) fn capture_patch_path(coord_root: &Path, name: &str) -> PathBuf {
    coord_root
        .join(CAPTURE_SUBPATH)
        .join(format!("{name}.{PATCH_EXT}"))
}

/// PURE correlator (`SubagentStop` carries no `worktree_path` — RV-202). Resolve the
/// worker worktree from the payload, validating each candidate with the injected
/// `is_worktree` predicate (the git touch rides in — CLAUDE.md pure/imperative
/// split), in design-§5.4 preference order:
///
/// * **(a)** `agent_id` present ⇒ `<coord>/.worktrees/agent-<id>` (the `create-fork`
///   name mint) — the primary; proven live (PHASE-01 F-T2).
/// * **(b)** the payload `cwd` (F-T2: `cwd` == the worktree) — the fallback, taken
///   when (a) is absent or does not validate.
///
/// First candidate the predicate accepts wins; `None` if neither validates (the
/// shell then logs loud + exits 0, D-capture-failmode — never deadlocks the stop).
pub(crate) fn correlate_worktree(
    agent_id: Option<&str>,
    payload_cwd: &Path,
    coord_root: &Path,
    is_worktree: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if let Some(id) = agent_id {
        let primary = coord_root
            .join(WORKTREES_SUBDIR)
            .join(format!("agent-{id}"));
        if is_worktree(&primary) {
            return Some(primary);
        }
    }
    is_worktree(payload_cwd).then(|| payload_cwd.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord() -> PathBuf {
        PathBuf::from("/coord")
    }

    #[test]
    fn coord_root_strips_worktrees_layout() {
        let wt = coord().join(".worktrees").join("agent-abc123");
        assert_eq!(coord_root_from_worktree(&wt), Some(coord()));
    }

    #[test]
    fn coord_root_rejects_non_worktree_shape() {
        // parent dir is not `.worktrees`
        assert_eq!(
            coord_root_from_worktree(Path::new("/coord/somewhere/agent-abc123")),
            None
        );
    }

    #[test]
    fn capture_patch_path_is_name_keyed_under_capture_subpath() {
        assert_eq!(
            capture_patch_path(&coord(), "agent-abc123"),
            coord()
                .join(".doctrine/state/dispatch/capture")
                .join("agent-abc123.patch")
        );
    }

    #[test]
    fn correlate_prefers_agent_id_primary_when_it_validates() {
        let primary = coord().join(".worktrees").join("agent-abc123");
        let cwd = PathBuf::from("/some/cwd");
        let got = correlate_worktree(Some("abc123"), &cwd, &coord(), |p| p == primary);
        assert_eq!(got, Some(primary));
    }

    #[test]
    fn correlate_falls_back_to_cwd_when_primary_invalid() {
        // agent_id present but its minted path does not validate; cwd does (F-T2).
        let cwd = PathBuf::from("/coord/.worktrees/moby-word-abc");
        let got = correlate_worktree(Some("abc123"), &cwd, &coord(), |p| p == cwd);
        assert_eq!(got, Some(cwd));
    }

    #[test]
    fn correlate_uses_cwd_when_agent_id_absent() {
        let cwd = PathBuf::from("/coord/.worktrees/agent-xyz");
        let got = correlate_worktree(None, &cwd, &coord(), |p| p == cwd);
        assert_eq!(got, Some(cwd));
    }

    #[test]
    fn correlate_none_when_neither_validates() {
        let cwd = PathBuf::from("/some/cwd");
        let got = correlate_worktree(Some("abc123"), &cwd, &coord(), |_| false);
        assert_eq!(got, None);
    }
}
