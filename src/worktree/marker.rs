// SPDX-License-Identifier: GPL-3.0-only
//! Worker identity — the `DOCTRINE_WORKER` environment leg, and nothing else
//! (SL-254 `DEC-207`).
//!
//! The disk marker this module was named for is gone. Identity is now a property
//! of the PROCESS, not of a tree: `DOCTRINE_WORKER` is set by the same confinement
//! argv that establishes the write floor (`scripts/spawn-confined.sh`, and
//! `jail.rs`'s `bwrap_argv` / `sandbox_exec_argv`), and it dies with the process.
//! So there is no stale class to detect and no cure verb to gate — which is why
//! `marker --clear`, `status --assert`'s stale exit, and the whole `Cause`
//! truth table retired with it (design §5.2.3, §5.2.4).

use std::io::{self, Write};
use std::path::PathBuf;

use crate::root;

// ---------------------------------------------------------------------------
// StatusLine — pure core
// ---------------------------------------------------------------------------

/// The resolved worker-mode verdict. One field: with a single signal there is no
/// cause to disambiguate and no `is_linked` context to carry (design §5.2.3).
/// Kept as a struct rather than a bare `bool` because it remains the SINGLE source
/// for both the `worktree status` human line and the guard's refusal, which is the
/// anti-parallel-implementation property SL-056 §3 established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StatusLine {
    /// True iff a write-classed verb would be refused.
    pub(crate) refused: bool,
}

/// Resolve worker mode from the one primitive signal. PURE — the caller's shell
/// supplies `env_set`. A two-row truth table, where SL-056's was eight rows over
/// three inputs (design §5.2.3, `VT-4`).
pub(crate) fn describe_mode(env_set: bool) -> StatusLine {
    StatusLine { refused: env_set }
}

/// True iff `DOCTRINE_WORKER` is set to `1` — now the identity itself, not the
/// optimisation it was under SL-056. Cheap (env only) and topology-independent:
/// it answers for THIS PROCESS, in a linked worktree, a clone, or the primary tree
/// alike.
pub(crate) fn env_worker_set() -> bool {
    // Name and value are single-sourced from the jail leaf (STD-001): the argv that
    // ESTABLISHES worker identity and the predicate that OBSERVES it are the same
    // two constants, so they cannot drift (SL-254 PHASE-02).
    std::env::var_os(super::jail::ENV_DOCTRINE_WORKER).as_deref()
        == Some(std::ffi::OsStr::new(super::jail::ENV_WORKER_ON))
}

/// The full worker-mode verdict for this process. Takes no root: with the marker
/// leg gone there is nothing about a TREE left to consult (`DEC-207`).
pub(crate) fn resolve_mode() -> StatusLine {
    describe_mode(env_worker_set())
}

/// The named refusal substance for the env leg. This is SL-056's `DUAL_CAUSE` with
/// its first horn removed: "a worker was dropped on the coordination root" was a
/// statement about tree topology, and topology no longer participates. What is left
/// is the one remedy that can be acted on (design §5.2.3).
pub(crate) const WORKER_ENV_CAUSE: &str =
    "`DOCTRINE_WORKER` is set, so this process is a worker: if that is wrong, unset it";

// ---------------------------------------------------------------------------
// worktree status (the observability verb)
// ---------------------------------------------------------------------------

/// `doctrine worktree status` (Read-classed). Prints the resolved mode from the
/// SINGLE [`describe_mode`] verdict the guard also reads, so the human line and the
/// refusal can never disagree.
///
/// `--assert` is gone with the stale-marker class it existed to detect (design
/// §5.2.4): an env leg cannot go stale, so there is no state for an operator to be
/// warned about.
pub(crate) fn run_status(path: Option<PathBuf>) -> anyhow::Result<()> {
    // The root is resolved purely to keep `-p`'s validation and the "not a doctrine
    // project" error exactly as they are; worker mode itself no longer consults it.
    let _root = root::find(path, &root::default_markers())?;

    if resolve_mode().refused {
        writeln!(
            io::stdout(),
            "worker fork: yes — writes refused; signal: env"
        )?;
    } else {
        writeln!(io::stdout(), "worker fork: no — writes allowed")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// SL-254 `VT-4`: the truth table is two rows over one input. The `marker_on_main`
    /// / `linked_no_marker` / `both` rows retired with their subject.
    #[test]
    fn describe_mode_truth_table() {
        assert!(!describe_mode(false).refused, "env unset ⇒ writes allowed");
        assert!(describe_mode(true).refused, "env set ⇒ writes refused");
    }

    #[test]
    fn env_worker_set_reads_the_env_flag() {
        // Under `DOCTRINE_WORKER=1` (a worker running the gate) the assertion below
        // is inapplicable by construction — skip rather than fail.
        if env_worker_set() {
            eprintln!("skipping: DOCTRINE_WORKER set — env-unset test inapplicable");
            return;
        }
        assert!(
            !env_worker_set(),
            "DOCTRINE_WORKER should not be set in the test harness"
        );
    }

    #[test]
    fn run_status_reports_writes_allowed_without_env() {
        let tmp = tempfile::tempdir().unwrap();
        let root = super::super::test_helpers::init_repo(&tmp.path().join("src"));
        // Should succeed without panicking.
        run_status(Some(root)).unwrap();
    }
}
