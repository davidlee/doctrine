// SPDX-License-Identifier: GPL-3.0-only
//! `SubagentStop` capture correlation — PURE path logic (SL-182 PHASE-05 T1).
//!
//! `SubagentStop` carries no `worktree_path` (RV-202), so the capture hook must
//! DERIVE which worktree to `git -C <wt> diff`. This leaf holds the pure pieces —
//! coord-root recovery, the captured-patch destination, and the correlator — with
//! every git/disk touch injected as input (CLAUDE.md pure/imperative split). The
//! T2 shell gathers the facts (the real `is_linked_worktree`, the diff) and acts.

use super::create::WORKTREES_SUBDIR;
use super::shared::is_linked_worktree;
use crate::{fsutil, git};
use anyhow::Context;
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
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

// ---- git belt-hardening flags (STD-001; mirrors import.rs SL-056 §7) ------------
// The captured patch feeds the T5 `--patch` import belt (`classify_import` prefix-
// matches `.doctrine/`/`.claude/`). So the SAME hardening the `--fork` gather uses
// must ride HERE, at capture time — the belt cannot un-mangle a path the diff already
// C-quoted or a rename it already collapsed (preflight T2↔T5 coupling):
//   * quotePath off — a non-ASCII `.doctrine/` path is emitted verbatim, not C-quoted
//     past the `starts_with(".doctrine/")` belt;
//   * `--no-renames` — a governance-file rename shows BOTH legs (delete + add), so the
//     `.doctrine/` SOURCE cannot hide behind a same-content destination.
const QUOTE_PATH_OFF: [&str; 2] = ["-c", "core.quotePath=false"];
const NO_RENAMES: &str = "--no-renames";
const DEV_NULL: &str = "/dev/null";

/// The `SubagentStop` stdin subset consumed (design §5.4 / PHASE-01 F-T2): the payload
/// carries `{agent_id, agent_transcript_path, cwd}` with `cwd` == the worktree and NO
/// `worktree_path` (RV-202). Every field optional ⇒ a malformed payload folds to
/// `Default`; the capture then simply no-ops and exits 0 (D-capture-failmode).
#[derive(Debug, Default, Deserialize)]
struct SubagentStopInput {
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
}

/// Capture the worker's full working-tree delta as one applyable patch. The
/// **tracked** leg is `git -C <wt> diff HEAD` (staged + unstaged in one stream),
/// belt-hardened, RAW bytes (trailing newline preserved for `git apply`). The
/// **untracked** leg synthesizes an index-free `new file` hunk per untracked path via
/// `git diff --no-index /dev/null <f>` (D-untracked: the ro `.git` index is NEVER
/// written — no `git add`/`-N`) and concatenates it onto the same stream. Impure (git
/// reads only, all `-C <wt>`); nothing under `.git` is mutated.
fn gather_worktree_patch(wt: &Path) -> anyhow::Result<Vec<u8>> {
    let mut patch = git::git_bytes(
        wt,
        &[
            QUOTE_PATH_OFF[0],
            QUOTE_PATH_OFF[1],
            "diff",
            NO_RENAMES,
            "HEAD",
        ],
    )
    .with_context(|| format!("git diff HEAD in {}", wt.display()))?;

    let untracked = git::git_text(wt, &["ls-files", "--others", "--exclude-standard"])
        .with_context(|| format!("list untracked in {}", wt.display()))?;
    for rel in untracked.lines().filter(|l| !l.is_empty()) {
        // `--no-index` exits 1 whenever the inputs differ (always, vs /dev/null) —
        // lenient runner keeps the stdout that carries the new-file hunk.
        let hunk = git::git_bytes_lenient(
            wt,
            &[
                QUOTE_PATH_OFF[0],
                QUOTE_PATH_OFF[1],
                "diff",
                NO_RENAMES,
                "--no-index",
                "--",
                DEV_NULL,
                rel,
            ],
        )
        .with_context(|| format!("synthesize untracked hunk for {rel} in {}", wt.display()))?;
        patch.extend_from_slice(&hunk);
    }
    Ok(patch)
}

/// `doctrine worktree subagent-stop` — the claude-arm `SubagentStop` capture hook
/// (design §5.4, RV-201 F-2). Reads the payload on stdin, correlates the worker
/// worktree (no `worktree_path` — RV-202), captures its delta to a patch OUTSIDE the
/// worktree under the coord runtime tier, and **always exits 0** so the harness stop
/// is never deadlocked (**D-capture-failmode**: a capture failure is logged loud and
/// swallowed — the loss is caught downstream by the `--patch` import's report-and-halt,
/// R-capture-lossy; this deliberately diverges from `pretooluse`'s fail-closed posture,
/// which is scoped to the WRITE WALLS, not this capture). Runs UNJAILED (a harness hook,
/// not a Bash tool call — PHASE-01 proven), so it can `git -C <wt> diff` and write
/// outside the tree.
pub(crate) fn run_subagent_stop() {
    if let Err(e) = capture() {
        writeln!(
            io::stderr(),
            "doctrine subagent-stop: capture failed — worker delta NOT captured, \
             the orchestrator's --patch import will report-and-halt: {e:#}"
        )
        .ok();
    }
}

/// The fallible capture body (errors swallowed by [`run_subagent_stop`]).
fn capture() -> anyhow::Result<()> {
    let mut raw = String::new();
    let _read = io::stdin().read_to_string(&mut raw);
    let input: SubagentStopInput = serde_json::from_str(&raw).unwrap_or_default();

    let cwd = input
        .cwd
        .as_deref()
        .and_then(|c| fs::canonicalize(c).ok())
        .context("SubagentStop payload carried no resolvable cwd")?;
    let coord_root = coord_root_from_worktree(&cwd).with_context(|| {
        format!(
            "cwd {} is not a <coord>/.worktrees/<name> worktree",
            cwd.display()
        )
    })?;
    let worktree = correlate_worktree(input.agent_id.as_deref(), &cwd, &coord_root, |p| {
        is_linked_worktree(p).unwrap_or(false)
    })
    .context("could not correlate a worktree from the SubagentStop payload (RV-202)")?;
    let name = worktree
        .file_name()
        .and_then(OsStr::to_str)
        .context("correlated worktree path has no basename")?;

    let patch = gather_worktree_patch(&worktree)?;
    let dest = capture_patch_path(&coord_root, name);
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).with_context(|| format!("create capture dir {}", dir.display()))?;
    }
    fsutil::write_atomic(&dest, &patch)?;
    writeln!(
        io::stderr(),
        "doctrine subagent-stop: captured {} bytes for {name} → {}",
        patch.len(),
        dest.display()
    )
    .ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktree::test_helpers::{git, init_repo};

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

    /// VT-1 round-trip: the captured patch carries BOTH a tracked change and an
    /// untracked add, and re-applies cleanly onto the base tree it was cut from —
    /// proving the index-free untracked synthesis (D-untracked) is applyable, not
    /// just gatherable.
    #[test]
    fn gather_captures_tracked_and_untracked_and_reapplies() {
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("primary"));
        // A linked worktree off HEAD (the worker's tree).
        let wt = tmp.path().join("wt");
        git(
            &primary,
            &["worktree", "add", "-q", wt.to_str().unwrap(), "HEAD"],
        );
        let wt = fs::canonicalize(&wt).unwrap();

        // Worker mutates a tracked file and drops an untracked one.
        fs::write(wt.join("seed"), "mutated\n").unwrap();
        fs::write(wt.join("newfile"), "brand new\n").unwrap();

        let patch = gather_worktree_patch(&wt).unwrap();
        assert!(!patch.is_empty(), "patch must be non-empty");
        let text = String::from_utf8_lossy(&patch);
        assert!(text.contains("seed"), "tracked change captured");
        assert!(text.contains("newfile"), "untracked add captured");

        // Re-apply onto a fresh checkout of the SAME base ⇒ both deltas reconstruct.
        let target = tmp.path().join("apply");
        git(
            &primary,
            &["worktree", "add", "-q", target.to_str().unwrap(), "HEAD"],
        );
        let patch_file = tmp.path().join("captured.patch");
        fsutil::write_atomic(&patch_file, &patch).unwrap();
        git(&target, &["apply", patch_file.to_str().unwrap()]);

        assert_eq!(
            fs::read_to_string(target.join("seed")).unwrap(),
            "mutated\n"
        );
        assert_eq!(
            fs::read_to_string(target.join("newfile")).unwrap(),
            "brand new\n"
        );
    }

    #[test]
    fn capture_patch_path_uses_the_capture_subpath_constant() {
        assert!(
            capture_patch_path(&coord(), "n")
                .to_string_lossy()
                .contains(CAPTURE_SUBPATH)
        );
    }
}
