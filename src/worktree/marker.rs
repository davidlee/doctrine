// SPDX-License-Identifier: GPL-3.0-only
//! Worker identity — disk marker primary (SL-056 §3, pure core).
//! `Cause`, `StatusLine`, `describe_mode`, marker file ops, `run_status`, `run_marker_clear`.

use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::root;

use super::shared::is_linked_worktree;

// ---------------------------------------------------------------------------
// Cause & StatusLine — pure core
// ---------------------------------------------------------------------------

/// Which signal(s) put the process in worker mode, if any. The single source for
/// BOTH the `worktree status` human line AND the `--assert` exit — no
/// `classify_writable` twin (design §3, anti-parallel-implementation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cause {
    /// Neither signal — writes allowed (direct/solo writer).
    None,
    /// Marker present in a linked worktree (the PRIMARY, harness-agnostic signal).
    Marker,
    /// `DOCTRINE_WORKER` env set (the codex/pi worker-on-main optimisation).
    Env,
    /// Both legs trip at once.
    Both,
}

impl Cause {
    /// The `signal: <token>` word for the human status line / refusals.
    fn token(self) -> &'static str {
        match self {
            Cause::None => "none",
            Cause::Marker => "marker",
            Cause::Env => "env",
            Cause::Both => "both",
        }
    }
}

/// The resolved worker-mode verdict: whether writes are refused, the cause, and
/// the `is_linked` context the dual-cause message needs. Minimal pure data —
/// derived by [`describe_mode`] and consumed by both the human line and the
/// `--assert` exit so the two can never disagree (design §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StatusLine {
    /// True iff a write-classed verb would be refused.
    pub(crate) refused: bool,
    /// Which signal(s) caused the refusal (`None` when allowed).
    pub(crate) cause: Cause,
    /// Whether the resolved root is a linked worktree (for the dual-cause split).
    pub(crate) is_linked: bool,
}

impl StatusLine {
    /// A stale/stray marker: the env leg is NOT involved, the marker is present,
    /// but it sits in a linked worktree without env — the `--assert` stale-marker
    /// case the operator must clear. Derived from the SAME state the human line
    /// reads (design §3): `cause == Marker` already encodes "marker-only, linked".
    pub(crate) fn is_stale_marker(self) -> bool {
        self.cause == Cause::Marker
    }

    /// The env leg tripped on a tree that is NOT a linked worktree — the
    /// dual-cause hazard (a worker dropped on the coordination root, or a leaked
    /// env). Distinct from a marker fork; carries the named dual-cause message.
    pub(crate) fn is_env_on_nonlinked(self) -> bool {
        matches!(self.cause, Cause::Env | Cause::Both) && !self.is_linked
    }

    /// The `signal: <token>` word for the human status line and refusals.
    pub(crate) fn cause_token(self) -> &'static str {
        self.cause.token()
    }
}

/// Resolve worker mode from the three primitive signals (design §3 truth table).
/// PURE — the caller's shell supplies `is_linked` (git), `marker_present` (disk),
/// and `env_set` (env). The marker leg trips ONLY in a linked worktree (a marker
/// on the primary tree is inert — mode, not location, decides, but the marker's
/// reach is the linked fork). The env leg trips anywhere (the worker-on-main
/// catch).
pub(crate) fn describe_mode(is_linked: bool, marker_present: bool, env_set: bool) -> StatusLine {
    let marker_leg = is_linked && marker_present;
    let cause = match (marker_leg, env_set) {
        (true, true) => Cause::Both,
        (true, false) => Cause::Marker,
        (false, true) => Cause::Env,
        (false, false) => Cause::None,
    };
    StatusLine {
        refused: marker_leg || env_set,
        cause,
        is_linked,
    }
}

// ---------------------------------------------------------------------------
// Constants & marker file ops
// ---------------------------------------------------------------------------

pub(crate) const DISPATCH_WORKER_AGENT_TYPE: &str = "dispatch-worker";

/// The withheld-tier marker the trusted orchestrator stamps before a worker runs.
/// Presence-only (no contents). Sits under `.doctrine/state/**`, so it inherits
/// every gitignore / provision-drop / import-exclude rule with zero new tier
/// logic (design §3; the `is_withheld` test pins it to [`Tier::State`]).
pub(crate) fn marker_path(root: &Path) -> PathBuf {
    root.join(".doctrine/state/dispatch/worker")
}

/// True iff the worker marker file exists at `root`. Disk read (shell).
pub(crate) fn marker_present(root: &Path) -> bool {
    marker_path(root).exists()
}

/// True iff `DOCTRINE_WORKER` is set to `1` — the codex/pi worker-on-main
/// OPTIMISATION (design §3), not the identity. Cheap (env only), evaluated before
/// the marker leg so a Read verb in a non-doctrine cwd never gains a git/disk
/// failure path. `pub(crate)` so the rootless-cwd guard fallback can consult the
/// env leg alone when `root::find` errors (no marker leg without a root).
pub(crate) fn env_worker_set() -> bool {
    std::env::var_os("DOCTRINE_WORKER").as_deref() == Some(std::ffi::OsStr::new("1"))
}

/// Stamp the worker marker at `root` (mkdir-p the dispatch dir). Shell.
/// The non-test consumer is [`run_fork`] under `--worker` (SL-056 PHASE-06).
pub(crate) fn write_marker(root: &Path) -> anyhow::Result<()> {
    let path = marker_path(root);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .with_context(|| format!("create dispatch marker dir {}", dir.display()))?;
    }
    #[expect(clippy::disallowed_methods, reason = "runtime worker marker")]
    fs::write(&path, b"").with_context(|| format!("write worker marker {}", path.display()))?;
    Ok(())
}

/// Remove the worker marker at `root` (idempotent — absent ⇒ Ok). Shell.
pub(crate) fn remove_marker(root: &Path) -> anyhow::Result<()> {
    let path = marker_path(root);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("remove worker marker {}", path.display())),
    }
}

/// Resolve the full worker-mode verdict at `root` through the single pure
/// [`describe_mode`] — the design's `worker_mode(root) = (is_linked_worktree(root)
/// && marker_present(root)) OR env` predicate (its `.refused` field). The env leg
/// is checked first (cheap); the marker leg (`is_linked_worktree` +
/// [`marker_present`]) only matters in a linked worktree, and a git failure there
/// is treated as not-linked (the verdict degrades to the env leg, never a new
/// error path — design §3 lazy-marker note).
pub(crate) fn resolve_mode(root: &Path) -> StatusLine {
    let env_set = env_worker_set();
    let is_linked = is_linked_worktree(root).unwrap_or(false);
    let marker = is_linked && marker_present(root);
    describe_mode(is_linked, marker, env_set)
}

/// The named dual-cause refusal substance for the env leg on a NON-linked tree
/// (design §3). Stable tokens — goldens assert this. Never a bare "worker
/// refused"; the caller also names the verb.
pub(crate) const DUAL_CAUSE: &str = "`DOCTRINE_WORKER` set outside a worker worktree: a worker was dropped on the coordination root → re-dispatch isolated; or the env leaked into this process → unset it";

// ---------------------------------------------------------------------------
// worktree status / marker --clear (SL-056 §3, the observability + cure verbs)
// ---------------------------------------------------------------------------

/// `doctrine worktree status [--assert]` (Read-classed). Prints the resolved
/// mode and cause from the SINGLE [`describe_mode`] verdict; `--assert` derives a
/// non-zero `stale-marker` exit from the SAME state (design §3 — the human line
/// and the `--assert` exit can never disagree).
pub(crate) fn run_status(path: Option<PathBuf>, assert: bool) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let mode = resolve_mode(&root);

    if mode.refused {
        writeln!(
            io::stdout(),
            "worker fork: yes — writes refused; signal: {}",
            mode.cause_token()
        )?;
    } else {
        writeln!(io::stdout(), "worker fork: no — writes allowed")?;
    }

    if assert && mode.is_stale_marker() {
        bail!(
            "stale-marker: a worker marker is present in this linked worktree but no dispatch is active — clear it with `doctrine worktree marker --clear --operator`"
        );
    }
    Ok(())
}

/// `doctrine worktree marker --clear [--operator]` (bespoke `MarkerClear` class —
/// never refused by the marker conjunct itself; design §3 §5). Removes the marker
/// at the cwd tree root with a loud receipt. Bespoke refusals:
/// - `DOCTRINE_WORKER` set (clear it from a process without the env leg);
/// - cwd is NOT the marker's own tree root (refuse a remote clear);
/// - cwd tree is a LINKED worktree and `--operator` is absent (the accident-fence).
pub(crate) fn run_marker_clear(path: Option<PathBuf>, operator: bool) -> anyhow::Result<()> {
    if env_worker_set() {
        bail!(
            "refusing `marker --clear` while `DOCTRINE_WORKER` is set — run it from a process without the env leg (unset DOCTRINE_WORKER)"
        );
    }

    let root = root::find(path, &root::default_markers())?;
    let root =
        fs::canonicalize(&root).with_context(|| format!("canonicalize root {}", root.display()))?;
    let cwd = std::env::current_dir().context("current dir")?;
    let cwd =
        fs::canonicalize(&cwd).with_context(|| format!("canonicalize cwd {}", cwd.display()))?;
    if cwd != root {
        bail!(
            "refusing `marker --clear`: cwd {} is not the marker's tree root {} — run it from the tree root",
            cwd.display(),
            root.display()
        );
    }

    if is_linked_worktree(&root).unwrap_or(false) && !operator {
        bail!(
            "refusing `marker --clear` in a linked worktree without `--operator` — this is the accident-fence; pass `--operator` to confirm you are the trusted orchestrator"
        );
    }

    let existed = marker_present(&root);
    remove_marker(&root)?;
    if existed {
        writeln!(
            io::stdout(),
            "CLEARED worker marker at {} — writes restored",
            marker_path(&root).display()
        )?;
    } else {
        writeln!(
            io::stdout(),
            "no worker marker at {} — nothing to clear",
            marker_path(&root).display()
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- SL-056 PHASE-05 T1: describe_mode truth table (the single source) ---

    #[test]
    fn describe_mode_truth_table() {
        // Solo: neither signal, in or out of a linked worktree ⇒ allowed.
        let solo_plain = describe_mode(false, false, false);
        assert!(!solo_plain.refused, "no signal ⇒ writes allowed");
        assert_eq!(solo_plain.cause, Cause::None);

        // A marker on the PRIMARY tree is inert (mode needs a linked fork).
        let marker_on_main = describe_mode(false, true, false);
        assert!(
            !marker_on_main.refused,
            "marker without a linked worktree is inert ⇒ allowed"
        );
        assert_eq!(marker_on_main.cause, Cause::None);

        // A linked worktree WITHOUT a marker (the clean direct-writer entry).
        let linked_no_marker = describe_mode(true, false, false);
        assert!(!linked_no_marker.refused, "linked, no marker ⇒ allowed");
        assert_eq!(linked_no_marker.cause, Cause::None);

        // PRIMARY signal: marker in a linked worktree, no env ⇒ refused: marker.
        let marker = describe_mode(true, true, false);
        assert!(marker.refused);
        assert_eq!(marker.cause, Cause::Marker);
        assert!(
            marker.is_stale_marker(),
            "marker-only in a fork is the stale-marker case"
        );
        assert!(!marker.is_env_on_nonlinked());

        // Env on a NON-linked tree ⇒ refused: env, dual-cause hazard.
        let env_main = describe_mode(false, false, true);
        assert!(env_main.refused);
        assert_eq!(env_main.cause, Cause::Env);
        assert!(env_main.is_env_on_nonlinked(), "env on main ⇒ dual-cause");
        assert!(!env_main.is_stale_marker());

        // Env inside a linked worktree (no marker) ⇒ env, but NOT the dual-cause
        // (it is genuinely a worker fork via the env optimisation).
        let env_linked = describe_mode(true, false, true);
        assert!(env_linked.refused);
        assert_eq!(env_linked.cause, Cause::Env);
        assert!(!env_linked.is_env_on_nonlinked());

        // Both legs ⇒ signal: both.
        let both = describe_mode(true, true, true);
        assert!(both.refused);
        assert_eq!(both.cause, Cause::Both);
        assert!(
            !both.is_stale_marker(),
            "both is not the marker-only stale case"
        );

        assert_eq!(solo_plain.cause_token(), "none");
        assert_eq!(marker.cause_token(), "marker");
        assert_eq!(env_main.cause_token(), "env");
        assert_eq!(both.cause_token(), "both");
    }

    // SL-056 §3 T2: write_marker / remove_marker / marker_present round-trip.
    #[test]
    fn marker_write_present_remove_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert!(!marker_present(root), "no marker initially");
        write_marker(root).unwrap();
        assert!(marker_present(root), "marker present after write");
        assert!(
            marker_path(root).exists(),
            "marker file exists under .doctrine/state/dispatch/worker"
        );
        remove_marker(root).unwrap();
        assert!(!marker_present(root), "marker gone after remove");
        // Idempotent: removing an absent marker is Ok.
        remove_marker(root).unwrap();
    }

    #[test]
    fn env_worker_set_reads_the_env_flag() {
        assert!(
            !env_worker_set(),
            "DOCTRINE_WORKER should not be set in the test harness"
        );
    }

    // --- SL-056 §3 T3: run_marker_clear goldens (VT-3) ---

    #[test]
    fn run_marker_clear_refuses_without_operator_in_linked_worktree() {
        // Create a linked worktree, mark it, try to clear without `--operator`.
        let tmp = tempfile::tempdir().unwrap();
        let primary = super::super::test_helpers::init_repo(&tmp.path().join("src"));
        let fork = tmp.path().join("fork");
        super::super::test_helpers::git(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "feat",
                fork.to_str().unwrap(),
            ],
        );
        let fork = std::fs::canonicalize(&fork).unwrap();
        write_marker(&fork).unwrap();
        let err = run_marker_clear(Some(fork.clone()), false).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("--operator"),
            "should refuse without --operator: {msg}"
        );
    }

    #[test]
    fn run_marker_clear_with_operator_clears_in_linked_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let primary = super::super::test_helpers::init_repo(&tmp.path().join("src"));
        let fork = tmp.path().join("fork");
        super::super::test_helpers::git(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "feat",
                fork.to_str().unwrap(),
            ],
        );
        let fork = std::fs::canonicalize(&fork).unwrap();
        write_marker(&fork).unwrap();
        run_marker_clear(Some(fork.clone()), true).unwrap();
        assert!(!marker_present(&fork));
    }

    // --- SL-056 §3 T4: run_status goldens (VT-4) ---

    #[test]
    fn run_status_no_marker_no_env_reports_writes_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = super::super::test_helpers::init_repo(&tmp.path().join("src"));
        // Should succeed without panicking.
        run_status(Some(root), false).unwrap();
    }
}
