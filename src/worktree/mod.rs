// SPDX-License-Identifier: GPL-3.0-only
//! Worktree provisioning — the sole copy path into a fork (SL-029, design §3).
//!
//! ADR-001 leaf: the pure core (`WITHHELD`, `parse_allowlist`, `is_withheld`,
//! `select_copies`, `allowlist_violations`) takes paths/strings as inputs — no
//! disk, git, clock, or rng. The impure shell (`run_provision`,
//! `run_check_allowlist`) is the thin imperative seam: it reads
//! `.worktreeinclude`, drives `git ls-files`/`rev-parse` through the `git.rs`
//! runners, and copies via the `fsutil` safe-copy helper.
//!
//! Two-layer exclusion (OQ-3-B): `select_copies` is the *guarantee* — it drops
//! any file matching the coordination/runtime tier even under a broad `**`
//! allowlist, so the copy physically cannot leak the tier. `allowlist_violations`
//! is a static *smell test* — a green result is NOT completeness (F7);
//! `select_copies` remains the guarantee.

use std::path::PathBuf;

use clap::Subcommand;

mod shared;
pub(crate) use shared::is_linked_worktree;

mod allowlist;

mod marker;
#[cfg(test)]
pub(crate) use marker::{Cause, describe_mode};
pub(crate) use marker::{
    DISPATCH_WORKER_AGENT_TYPE, DUAL_CAUSE, env_worker_set, resolve_mode, run_marker_clear,
    run_status,
};

mod coordinate;
mod fork;
mod gc;
mod import;
mod land;
mod provision;
mod subagent;

pub(crate) use coordinate::{coordinate, run_branch_point_check, run_coordinate};
pub(crate) use fork::run_fork;
pub(crate) use gc::run_gc;
pub(crate) use import::run_import;
pub(crate) use land::run_land;
pub(crate) use provision::{run_check_allowlist, run_provision};
pub(crate) use subagent::{run_stamp_subagent, run_verify_worker};

#[cfg(test)]
pub(crate) use coordinate::{CoordAction, CoordRefusal, base_has_slice_plan, classify_coordinate};
#[cfg(test)]
pub(crate) use gc::{GcPlan, GcRefusal, GcState, GcVerdict, classify_gc};
#[cfg(test)]
pub(crate) use land::{ForkState, LandRefusal, Merge, classify_land};
#[cfg(test)]
pub(crate) use subagent::{
    Stamp, StampRefusal, WorkerVerify, WorkerVerifyRefusal, classify_stamp, classify_worker_verify,
};

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
pub(crate) use crate::globmatch::glob_matches;
#[cfg(test)]
pub(crate) use allowlist::{DERIVED_RUNTIME, WITHHELD};

// ---------------------------------------------------------------------------
// CLI enum & dispatch (PHASE-03 relocation from main.rs)
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub(crate) enum WorktreeCommand {
    /// Copy allowlisted files into a worktree fork.
    /// The sole copy path; the coordination/runtime tier is always excluded.
    Provision {
        /// The target sibling worktree to populate.
        fork: PathBuf,

        /// Explicit source project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Check `.worktreeinclude` for invalid patterns.
    /// Nonzero exit if any pattern names a withheld tier or uses unsupported
    /// syntax (`!`/anchoring).
    CheckAllowlist {
        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Check HEAD stationarity at batch-commit boundary.
    /// Exit 0 if coordination HEAD still equals the orchestrator's pre-spawn base,
    /// 1 otherwise (→ re-dispatch). Not a merge-base compute (C-V).
    BranchPointCheck {
        /// The orchestrator's pre-spawn captured base commit `B`.
        #[arg(long)]
        base: String,

        /// HEAD to compare against (default: `git rev-parse HEAD`).
        #[arg(long)]
        head: Option<String>,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create a worktree fork.
    /// Orchestrator-owned fork off `<base>` on a NEW branch, provisioned,
    /// optionally worker-stamped. Emits the per-worktree env contract on stdout.
    /// Orchestrator-classed — refused under worker-mode. Atomic via compensating
    /// rollback.
    Fork {
        /// The base commit `B` the fork is created from (the orchestrator's
        /// captured coordination HEAD).
        #[arg(long)]
        base: String,

        /// The NEW branch to create at `<base>` for the fork.
        #[arg(long)]
        branch: String,

        /// The fork worktree directory (must not already exist; unique per branch).
        #[arg(long)]
        dir: PathBuf,

        /// Stamp the worker-mode marker so the fork resolves to worker mode.
        #[arg(long)]
        worker: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Create or resume a coordination worktree.
    /// For a slice on branch `dispatch/<slice>` off the resolved trunk.
    /// MARKERLESS — the coordination tree IS the orchestrator. A live worktree
    /// already on `dispatch/<slice>` is refused (`coordination-live`); a branch
    /// with no live worktree resumes (reattach, never a second branch).
    /// Orchestrator-classed — refused under worker-mode.
    Coordinate {
        /// The slice id (bare number, e.g. `64`) whose `dispatch/<slice>`
        /// coordination worktree to create or resume.
        #[arg(long)]
        slice: u32,

        /// The coordination worktree directory (must not already exist).
        #[arg(long)]
        dir: PathBuf,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Import a worker's commit into the coordination index.
    /// NON-committing (ADR-006 D7: import ≠ commit). Stationary-head case only —
    /// fails closed on any precond/belt violation; never auto-merges.
    /// Orchestrator-classed — refused under worker-mode.
    Import {
        /// The orchestrator's pre-spawn captured base commit `B`.
        #[arg(long)]
        base: String,

        /// The fork branch carrying the single non-merge commit `S` (`S^ == B`).
        #[arg(long)]
        fork: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Land a worktree branch onto coordination.
    /// Merges a solo multi-commit TDD branch with ancestry PRESERVED via
    /// `git merge --no-ff` (NEVER `--squash`). Solo `/execute`'s analog of
    /// `import`. Fails closed on any precond/merge violation.
    /// Orchestrator-classed — refused under worker-mode.
    Land {
        /// The solo fork branch to merge onto the coordination branch.
        #[arg(long)]
        fork: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Reap a spent worktree fork.
    /// One idempotent act — deletes ONLY when the fork has provably landed via
    /// the durable-git oracle. `--superseded-head <SHA>` reaps iff the SHA equals
    /// the branch's current head (movement-guard). `--force` bypasses the oracle.
    /// `--dry-run` prints the verdict and destroys nothing. Orchestrator-classed
    /// — refused under worker-mode.
    Gc {
        /// The fork branch to reap.
        #[arg(long)]
        fork: String,

        /// Reap iff this SHA equals the branch's current head (the moved-HEAD
        /// re-dispatch case: a spent-yet-never-landed fork). A movement-guard, not a
        /// landing proof.
        #[arg(long)]
        superseded_head: Option<String>,

        /// Bypass the landed oracle and reap knowingly.
        #[arg(long)]
        force: bool,

        /// Compute and print the per-fork verdict, destroying nothing.
        #[arg(long)]
        dry_run: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the resolved worker-mode and cause.
    /// `--assert` derives a non-zero `stale-marker` exit. Read-classed — open
    /// to workers.
    Status {
        /// Gate exit: non-zero with a `stale-marker` token if a stray marker sits
        /// in this linked worktree (clean direct-writer entry ⇒ exit 0).
        #[arg(long)]
        assert: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Verify a worker's base commit.
    /// Post-spawn check: prove the worker worktree's HEAD descends from the
    /// base `B` it was meant to fork off. Diagnostic only — fail-loud, NEVER
    /// removes the fork. Read-classed (callable under worker-mode).
    VerifyWorker {
        /// The base commit `B` the worker was meant to fork off (the
        /// orchestrator's coordination HEAD at spawn).
        #[arg(long)]
        base: String,

        /// The worker worktree to verify — the git `-C` root for every probe.
        #[arg(long)]
        dir: PathBuf,

        /// The worker fork branch S — binds HEAD(--dir) == tip(S) (dir↔branch coherence).
        #[arg(long)]
        branch: Option<String>,
    },

    /// Manage the worker-mode disk marker (SL-056 §3). `--clear` removes it at the
    /// cwd tree root with a loud receipt — the self-brick cure; never refused by
    /// the marker conjunct itself.
    Marker {
        /// Remove the marker at the cwd tree root.
        #[arg(long)]
        clear: bool,

        /// Confirm a clear inside a linked worktree (the accident-fence).
        #[arg(long)]
        operator: bool,

        /// Provision + stamp the worker marker into the `SubagentStart` payload's
        /// worktree (SL-056 PHASE-10). Reads `{cwd, agent_type}` JSON on stdin;
        /// the claude harness spawn path's mark step.
        #[arg(long)]
        stamp_subagent: bool,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: WorktreeCommand) -> anyhow::Result<()> {
    match cmd {
        WorktreeCommand::Provision { fork, path } => run_provision(path, &fork),
        WorktreeCommand::CheckAllowlist { path } => run_check_allowlist(path),
        WorktreeCommand::BranchPointCheck { base, head, path } => {
            run_branch_point_check(path, &base, head)
        }
        WorktreeCommand::Fork {
            base,
            branch,
            dir,
            worker,
            path,
        } => run_fork(path, &base, &branch, &dir, worker),
        WorktreeCommand::Coordinate { slice, dir, path } => run_coordinate(path, slice, &dir),
        WorktreeCommand::Import { base, fork, path } => run_import(path, &base, &fork),
        WorktreeCommand::Land { fork, path } => run_land(path, &fork),
        WorktreeCommand::Gc {
            fork,
            superseded_head,
            force,
            dry_run,
            path,
        } => run_gc(path, &fork, superseded_head.as_deref(), force, dry_run),
        WorktreeCommand::Status { assert, path } => run_status(path, assert),
        WorktreeCommand::VerifyWorker { base, dir, branch } => {
            run_verify_worker(&base, &dir, branch.as_deref())
        }
        WorktreeCommand::Marker {
            clear,
            operator,
            stamp_subagent,
            path,
        } => {
            if stamp_subagent {
                run_stamp_subagent(path)
            } else if clear {
                run_marker_clear(path, operator)
            } else {
                anyhow::bail!("`worktree marker` requires `--clear` or `--stamp-subagent`")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::test_helpers::{git, init_repo};
    use super::*;
    use glob::Pattern;
    use std::fs;
    use std::path::Path;

    use super::fork::rollback_fork;

    // --- SL-056 PHASE-08: land pure classifier + refusal-token table (design §6) ---

    fn fork_state(exists: bool, has_live_worktree: bool, bears_marker: bool) -> ForkState {
        ForkState {
            exists,
            has_live_worktree,
            bears_marker,
        }
    }

    #[test]
    fn classify_land_precedence_and_ok() {
        // Happy: clean tree, fork exists, live worktree, no marker ⇒ Ok(Merge).
        assert_eq!(
            classify_land(true, "main", fork_state(true, true, false)),
            Ok(Merge::Ok)
        );
        // Precedence tree-unclean → no-such-fork → worktree-gone → dispatch-fork.
        // Dirty tree wins over every later fault.
        assert_eq!(
            classify_land(false, "main", fork_state(false, false, true)),
            Err(LandRefusal::TreeUnclean)
        );
        // Clean tree, missing fork wins over the worktree/marker checks.
        assert_eq!(
            classify_land(true, "main", fork_state(false, false, true)),
            Err(LandRefusal::NoSuchFork)
        );
        // worktree-gone GATES dispatch-fork: a worktree-less branch refuses
        // worktree-gone BEFORE the marker check can pass vacuously.
        assert_eq!(
            classify_land(true, "main", fork_state(true, false, false)),
            Err(LandRefusal::WorktreeGone)
        );
        // Live worktree that bears the marker ⇒ dispatch-fork.
        assert_eq!(
            classify_land(true, "main", fork_state(true, true, true)),
            Err(LandRefusal::DispatchFork)
        );
    }

    #[test]
    fn classify_land_ignores_head() {
        // `head` documents the contextual coordination-root precond; it gates NO
        // token, so the verdict is invariant under any HEAD value.
        let st = fork_state(true, true, false);
        assert_eq!(
            classify_land(true, "main", st),
            classify_land(true, "detached-xyz", st)
        );
    }

    #[test]
    fn land_refusal_tokens_are_distinct_and_exhaustive() {
        // The exhaustive 7-token set (design §6). wedged-merge's live abort-failure
        // path is not deterministically black-box reproducible (it needs `git merge
        // --abort` itself to fail); its token is pinned HERE per the worker
        // contract's fallback, alongside the other six.
        let all = [
            LandRefusal::TreeUnclean,
            LandRefusal::NoSuchFork,
            LandRefusal::WorktreeGone,
            LandRefusal::DispatchFork,
            LandRefusal::MergeConflict,
            LandRefusal::WedgedMerge,
            LandRefusal::InconsistentMergeState,
        ];
        let tokens: Vec<&str> = all.iter().map(|r| r.token()).collect();
        assert_eq!(tokens.len(), 7, "exactly seven refusal tokens");
        let unique: std::collections::BTreeSet<&str> = tokens.iter().copied().collect();
        assert_eq!(unique.len(), 7, "every token is distinct");
        assert_eq!(LandRefusal::WedgedMerge.token(), "wedged-merge");
        assert_eq!(LandRefusal::MergeConflict.token(), "merge-conflict");
        assert_eq!(
            LandRefusal::InconsistentMergeState.token(),
            "inconsistent-merge-state"
        );
    }

    // --- SL-056 PHASE-09: classify_gc pure verdict (design §8.2) ---

    fn gc_state(
        branch_exists: bool,
        worktree_present: bool,
        target_present: bool,
        landed_verdict: Option<bool>,
    ) -> GcState {
        GcState {
            branch_exists,
            worktree_present,
            target_present,
            landed_verdict,
        }
    }

    // --- SL-064 PHASE-02: coordination-create classifier (design §1/§2) ---

    #[test]
    fn classify_coordinate_create_resume_collide() {
        // Branch absent ⇒ create fresh (live-worktree fact is irrelevant).
        assert_eq!(classify_coordinate(false, false), Ok(CoordAction::Create));
        assert_eq!(classify_coordinate(false, true), Ok(CoordAction::Create));
        // Branch exists, NO live worktree ⇒ handover resume (reattach same branch).
        assert_eq!(classify_coordinate(true, false), Ok(CoordAction::Resume));
        // Branch exists WITH a live worktree ⇒ concurrent run; refuse.
        assert_eq!(
            classify_coordinate(true, true),
            Err(CoordRefusal::LiveWorktree)
        );
    }

    #[test]
    fn coord_refusal_token_distinct() {
        assert_eq!(CoordRefusal::LiveWorktree.token(), "coordination-live");
    }

    #[test]
    fn classify_gc_landed_reaps_present_things_in_order() {
        // Branch + worktree + target present, oracle positive ⇒ reap all three.
        let v = classify_gc(gc_state(true, true, true, Some(true)), false, false, false);
        assert_eq!(
            v,
            GcVerdict::Reap(GcPlan {
                remove_worktree: true,
                delete_branch: true,
                reap_target: true,
            })
        );
    }

    #[test]
    fn classify_gc_skips_absent_steps() {
        // Worktree already gone (crash mid-gc), target gone too ⇒ only branch -D.
        let v = classify_gc(
            gc_state(true, false, false, Some(true)),
            false,
            false,
            false,
        );
        assert_eq!(
            v,
            GcVerdict::Reap(GcPlan {
                remove_worktree: false,
                delete_branch: true,
                reap_target: false,
            })
        );
    }

    #[test]
    fn classify_gc_branch_gone_reaps_only_the_target() {
        // Branch-gone ⇒ already-certified; the ONLY residue is the target dir,
        // reaped from the branch NAME alone (landed_verdict is None — gate skipped).
        let v = classify_gc(gc_state(false, false, true, None), false, false, false);
        assert_eq!(
            v,
            GcVerdict::Reap(GcPlan {
                remove_worktree: false,
                delete_branch: false,
                reap_target: true,
            })
        );
        // Branch gone AND target gone ⇒ a fully-reaped no-op (idempotent rerun).
        let done = classify_gc(gc_state(false, false, false, None), false, false, false);
        assert_eq!(
            done,
            GcVerdict::Reap(GcPlan {
                remove_worktree: false,
                delete_branch: false,
                reap_target: false,
            })
        );
    }

    #[test]
    fn classify_gc_not_landed_refuses_unless_overridden() {
        // Non-ancestor tip with a `+` (oracle false; also the squash case — the two
        // are indistinguishable), no override ⇒ not-landed.
        let st = gc_state(true, true, true, Some(false));
        assert_eq!(
            classify_gc(st, false, false, false),
            GcVerdict::Refuse(GcRefusal::NotLanded)
        );
        // --force bypasses the oracle ⇒ reap.
        assert!(matches!(
            classify_gc(st, true, false, false),
            GcVerdict::Reap(_)
        ));
        // --superseded-head match (head == asserted SHA) ⇒ reap.
        assert!(matches!(
            classify_gc(st, false, true, false),
            GcVerdict::Reap(_)
        ));
    }

    #[test]
    fn classify_gc_dry_run_does_not_change_the_verdict() {
        // dry_run is honoured in the shell; the classifier returns the SAME verdict
        // a real run would act on (so the dry-run print is truthful).
        let landed = gc_state(true, true, true, Some(true));
        assert_eq!(
            classify_gc(landed, false, false, true),
            classify_gc(landed, false, false, false)
        );
        let refused = gc_state(true, true, true, Some(false));
        assert_eq!(
            classify_gc(refused, false, false, true),
            classify_gc(refused, false, false, false)
        );
    }

    #[test]
    fn gc_refusal_token_is_not_landed() {
        assert_eq!(GcRefusal::NotLanded.token(), "not-landed");
    }

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

    // --- T1: WITHHELD authority + .gitignore parity (VT-4) ---

    #[test]
    fn withheld_globs_all_compile() {
        for item in WITHHELD {
            Pattern::new(item.glob).unwrap();
        }
        for g in DERIVED_RUNTIME {
            Pattern::new(g).unwrap();
        }
    }

    /// A concrete sample path for a `.gitignore` runtime line: trailing-slash dirs
    /// gain a file; wildcards collapse to a literal segment.
    fn gitignore_representative(line: &str) -> String {
        let base = line
            .strip_suffix('/')
            .map_or_else(|| line.to_string(), |dir| format!("{dir}/f"));
        base.replace('*', "x")
    }

    fn classified(rep: &str) -> bool {
        WITHHELD
            .iter()
            .any(|item| glob_matches(&Pattern::new(item.glob).unwrap(), rep))
            || DERIVED_RUNTIME
                .iter()
                .any(|g| glob_matches(&Pattern::new(g).unwrap(), rep))
    }

    #[test]
    fn every_runtime_gitignore_glob_is_classified() {
        let gitignore = fs::read_to_string(".gitignore").unwrap();
        for raw in gitignore.lines() {
            let line = raw.trim();
            // Runtime-tier globs: `.doctrine/`-prefixed, non-negated, more specific
            // than the broad `.doctrine/*` exclude (the authored-tier negations are
            // `!`-prefixed and filtered here).
            if !line.starts_with(".doctrine/") || line == ".doctrine/*" {
                continue;
            }
            let rep = gitignore_representative(line);
            assert!(
                classified(&rep),
                "unclassified runtime gitignore glob `{line}` (rep `{rep}`) — \
                 add it to WITHHELD or DERIVED_RUNTIME"
            );
        }
    }

    // --- T1: is_linked_worktree self-detection (SL-032 PHASE-04, VT-1) ---

    #[test]
    fn is_linked_worktree_true_for_a_fork_false_for_the_primary_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("src"));
        let fork = tmp.path().join("fork");
        git(
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
        let fork = fs::canonicalize(&fork).unwrap();

        assert!(is_linked_worktree(&fork).unwrap(), "a linked worktree");
        assert!(!is_linked_worktree(&primary).unwrap(), "the primary tree");
    }

    #[test]
    fn primary_worktree_resolves_to_the_main_tree_from_a_fork_or_itself() {
        // VT-2 (SL-125): the R2 provision SOURCE. From a linked worktree it must
        // resolve to the PRIMARY tree (the Defect-C correction); from the primary
        // tree it is idempotent (resolves to itself).
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("src"));
        let fork = tmp.path().join("fork");
        git(
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
        let fork = fs::canonicalize(&fork).unwrap();

        assert_eq!(
            crate::git::primary_worktree(&fork).unwrap(),
            primary,
            "a linked worktree resolves to the main tree"
        );
        assert_eq!(
            crate::git::primary_worktree(&primary).unwrap(),
            primary,
            "the main tree resolves to itself"
        );
    }

    #[test]
    fn rollback_fork_retracts_stale_worktree_entry_after_fs_reap() {
        // F-8: when step-1 `git worktree remove` FAILS (the dir is not a
        // registered worktree) but step-3 fs-reaps the dir, the stale step-1
        // debris entry must be retracted so a fully-cleaned rollback reports NO
        // debris — else run_fork false-bails `fork-rollback-debris` over a tree
        // that was, in fact, fully cleaned.
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("src"));
        // A plain dir that is NOT a git worktree ⇒ `worktree remove --force`
        // errors, but `fs::remove_dir_all` reaps it.
        let dir = tmp.path().join("orphan");
        fs::create_dir_all(&dir).unwrap();

        let debris = rollback_fork(&repo, "no-such-branch", &dir);

        assert!(
            debris.is_empty(),
            "a fully fs-reaped rollback reports no debris; got: {debris:?}"
        );
        assert!(!dir.exists(), "the orphan dir was reaped");
    }

    // --- SL-127 PHASE-02: plan-presence refuse-gate at coordinate (Create) ---

    /// Commit `.doctrine/slice/<NNN>/plan.toml` onto the repo's current branch so
    /// the chosen trunk base carries the dispatched slice's plan.
    fn commit_slice_plan(repo: &Path, slice: u32) {
        let slice_dir = repo.join(format!(".doctrine/slice/{slice:03}"));
        fs::create_dir_all(&slice_dir).unwrap();
        fs::write(slice_dir.join("plan.toml"), "# plan\n").unwrap();
        git(repo, &["add", "."]);
        git(repo, &["commit", "-q", "-m", "add slice plan"]);
    }

    #[test]
    fn base_has_slice_plan_tracks_presence_on_the_trunk_tree() {
        // VT-1 (helper): absent on the base tree ⇒ false; once committed ⇒ true.
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("src"));

        assert!(
            !base_has_slice_plan(&repo, "main", 127).unwrap(),
            "base lacking the slice plan ⇒ absent"
        );

        commit_slice_plan(&repo, 127);

        assert!(
            base_has_slice_plan(&repo, "main", 127).unwrap(),
            "base carrying the slice plan ⇒ present"
        );
        // A different slice with no plan dir is still absent on the same base.
        assert!(
            !base_has_slice_plan(&repo, "main", 99).unwrap(),
            "an unrelated slice number stays absent"
        );
    }

    #[test]
    fn coordinate_refuses_create_when_base_lacks_the_slice_plan() {
        // VT-1 (coordinate): Create where trunk lacks the slice plan bails BEFORE
        // the fork — Err names DOCTRINE_TRUNK_REF and NO worktree dir is created
        // (the rollback path is never entered, F6).
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(&tmp.path().join("src"));
        let dir = tmp.path().join("coord");

        let Err(err) = coordinate(&repo, 127, &dir) else {
            panic!("must refuse: base predates plan");
        };
        let msg = format!("{err:#}");
        assert!(
            msg.contains("DOCTRINE_TRUNK_REF"),
            "refusal must hint DOCTRINE_TRUNK_REF; got: {msg}"
        );
        assert!(
            msg.contains(".doctrine/slice/127/plan.toml"),
            "refusal names the missing plan path; got: {msg}"
        );
        assert!(
            !dir.exists(),
            "no worktree dir is created on the early bail"
        );
    }

    // --- SL-056 PHASE-10: classify_stamp pure arms (T2) ---

    #[test]
    fn classify_stamp_ok_when_all_inputs_hold() {
        // Valid dir + agent-type + marker ABSENT (the first stamp) ⇒ Ok.
        assert_eq!(
            classify_stamp(DISPATCH_WORKER_AGENT_TYPE, true, true, false),
            Ok(Stamp::Ok)
        );
    }

    #[test]
    fn classify_stamp_missing_cwd_refuses() {
        // cwd absent ⇒ missing-cwd, regardless of the other inputs.
        assert_eq!(
            classify_stamp(DISPATCH_WORKER_AGENT_TYPE, false, false, false),
            Err(StampRefusal::MissingCwd)
        );
        assert_eq!(StampRefusal::MissingCwd.token(), "missing-cwd");
    }

    #[test]
    fn classify_stamp_bad_dir_refuses_when_cwd_present_but_invalid() {
        // cwd present but not under-repo-and-linked ⇒ bad-dir (checked before
        // agent-type, so even a wrong agent_type still names the dir problem).
        assert_eq!(
            classify_stamp(DISPATCH_WORKER_AGENT_TYPE, true, false, false),
            Err(StampRefusal::BadDir)
        );
        assert_eq!(
            classify_stamp("anything", true, false, false),
            Err(StampRefusal::BadDir)
        );
        assert_eq!(StampRefusal::BadDir.token(), "bad-dir");
    }

    #[test]
    fn classify_stamp_missing_agent_type_refuses() {
        // agent_type absent ("") OR present-but-wrong ⇒ missing-agent-type.
        assert_eq!(
            classify_stamp("", true, true, false),
            Err(StampRefusal::MissingAgentType)
        );
        assert_eq!(
            classify_stamp("some-other-agent", true, true, false),
            Err(StampRefusal::MissingAgentType)
        );
        assert_eq!(StampRefusal::MissingAgentType.token(), "missing-agent-type");
    }

    #[test]
    fn classify_stamp_already_marked_refuses() {
        // Valid dir + agent-type but the worktree ALREADY bears the marker ⇒ a
        // re-entrant stamp ⇒ already-marked (the marker check is LAST, F-9).
        assert_eq!(
            classify_stamp(DISPATCH_WORKER_AGENT_TYPE, true, true, true),
            Err(StampRefusal::AlreadyMarked)
        );
        assert_eq!(StampRefusal::AlreadyMarked.token(), "already-marked");
    }

    // --- SL-064 PHASE-08: worker-verify pure classifier + token table (design §8.4) ---
    // --- SL-123 PHASE-01: not-isolated + branch-mismatch belts (design §5.2) ---

    // VT-3 (updated): existing goldens with the 5-arg signature, verdicts UNCHANGED.
    #[test]
    fn classify_worker_verify_ok_when_all_preconds_hold() {
        // HEAD resolves, isolated, marker present, B is an ancestor, branch tip
        // matches ⇒ base==B holds.
        assert_eq!(
            classify_worker_verify(true, true, true, true, true),
            Ok(WorkerVerify::Ok)
        );
    }

    #[test]
    fn classify_worker_verify_no_worker_head_refuses_first() {
        // HEAD unresolved ⇒ no-worker-head, regardless of the other inputs (the
        // first precond — nothing to verify without a HEAD).
        assert_eq!(
            classify_worker_verify(false, true, true, true, true),
            Err(WorkerVerifyRefusal::NoWorkerHead)
        );
        assert_eq!(
            classify_worker_verify(false, false, false, false, false),
            Err(WorkerVerifyRefusal::NoWorkerHead)
        );
        assert_eq!(WorkerVerifyRefusal::NoWorkerHead.token(), "no-worker-head");
    }

    #[test]
    fn classify_worker_verify_unstamped_names_itself_before_base() {
        // HEAD resolves but marker absent ⇒ unstamped, EVEN WHEN the base is also
        // wrong — the marker check precedes the base check (precond order).
        assert_eq!(
            classify_worker_verify(true, true, false, false, true),
            Err(WorkerVerifyRefusal::Unstamped)
        );
        assert_eq!(WorkerVerifyRefusal::Unstamped.token(), "unstamped");
    }

    #[test]
    fn classify_worker_verify_wrong_base_refuses_last() {
        // Resolvable, stamped fork, but B is NOT an ancestor of the worker HEAD ⇒
        // wrong-base.
        assert_eq!(
            classify_worker_verify(true, true, true, false, true),
            Err(WorkerVerifyRefusal::WrongBase)
        );
        assert_eq!(WorkerVerifyRefusal::WrongBase.token(), "wrong-base");
    }

    // VT-1: not-isolated refuses after NoWorkerHead but before marker.
    #[test]
    fn classify_worker_verify_not_isolated_refuses_after_head_before_marker() {
        // HEAD resolves but is_isolated=false ⇒ NotIsolated, regardless of marker/base.
        assert_eq!(
            classify_worker_verify(true, false, true, true, true),
            Err(WorkerVerifyRefusal::NotIsolated)
        );
        assert_eq!(
            classify_worker_verify(true, false, false, false, false),
            Err(WorkerVerifyRefusal::NotIsolated)
        );
        assert_eq!(WorkerVerifyRefusal::NotIsolated.token(), "not-isolated");
    }

    // VT-2: branch-mismatch refuses last.
    #[test]
    fn classify_worker_verify_branch_mismatch_refuses_last() {
        // Everything ok except head_is_branch_tip=false ⇒ BranchMismatch.
        assert_eq!(
            classify_worker_verify(true, true, true, true, false),
            Err(WorkerVerifyRefusal::BranchMismatch)
        );
        // No --branch (head_is_branch_tip=true) with all-true ⇒ Ok.
        assert_eq!(
            classify_worker_verify(true, true, true, true, true),
            Ok(WorkerVerify::Ok)
        );
        assert_eq!(
            WorkerVerifyRefusal::BranchMismatch.token(),
            "branch-mismatch"
        );
    }

    // --- SL-056 PHASE-10 T6 / VT-4: agent-def `name` ↔ const drift gate ---
    //
    // Reds if `install/agents/claude/dispatch-worker.md` frontmatter `name:`
    // diverges from `DISPATCH_WORKER_AGENT_TYPE`. The SubagentStart matcher leg
    // is covered in `src/boot.rs`; the `/dispatch-agent` skill leg is below
    // (PHASE-13). Together they pin every replica of the literal to the const.
    #[test]
    fn dispatch_worker_agent_def_name_matches_const() {
        let manifest = crate::test_support::repo_root();
        let def = manifest.join("install/agents/claude/dispatch-worker.md");
        let text =
            fs::read_to_string(&def).unwrap_or_else(|e| panic!("read {}: {e}", def.display()));
        let name = text
            .lines()
            .find_map(|l| l.trim().strip_prefix("name:"))
            .map(str::trim)
            .unwrap_or_else(|| panic!("no `name:` frontmatter in {}", def.display()));
        assert_eq!(
            name, DISPATCH_WORKER_AGENT_TYPE,
            "agent-def name must equal DISPATCH_WORKER_AGENT_TYPE"
        );
    }

    // --- SL-056 PHASE-13 / VT-1 (τ): `/dispatch-agent` skill `subagent_type` leg ---
    //
    // Reds if the `/dispatch-agent` skill's `subagent_type:` literal — the value
    // the orchestrator passes to the `Agent` tool to spawn a worker — diverges
    // from `DISPATCH_WORKER_AGENT_TYPE`. A one-character drift fails OPEN (the
    // SubagentStart matcher never fires ⇒ no stamp ⇒ worker_mode false), so the
    // literal is PINNED here, not merely documented.
    #[test]
    fn dispatch_agent_skill_subagent_type_matches_const() {
        let manifest = crate::test_support::repo_root();
        let skill = manifest.join("plugins/doctrine/skills/dispatch-agent/SKILL.md");
        let text =
            fs::read_to_string(&skill).unwrap_or_else(|e| panic!("read {}: {e}", skill.display()));
        let pinned = text
            .lines()
            .find_map(|l| l.split_once("subagent_type:").map(|(_, rest)| rest))
            .map(|rest| {
                rest.trim()
                    .trim_start_matches('`')
                    .split([' ', '`', '#'])
                    .next()
                    .unwrap_or("")
                    .trim()
            })
            .unwrap_or_else(|| panic!("no `subagent_type:` line in {}", skill.display()));
        assert_eq!(
            pinned, DISPATCH_WORKER_AGENT_TYPE,
            "/dispatch-agent subagent_type must equal DISPATCH_WORKER_AGENT_TYPE"
        );
    }
}
