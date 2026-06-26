#![expect(unused, reason = "extraction; PHASE-03 prunes")]
// SPDX-License-Identifier: GPL-3.0-only
//! gc machine — extracted from worktree/mod.rs (SL-116 PHASE-02).

use super::allowlist::{
    Allowlist, allowlist_violations, is_withheld, parse_allowlist, select_copies,
};
use super::marker::{DISPATCH_WORKER_AGENT_TYPE, marker_present, write_marker};
use super::shared::{
    gather_fork_worktree, gather_tree_clean, is_linked_worktree, matches, resolve_commit,
    resolve_common_dir,
};
use crate::fsutil::{self, CopyOutcome};
use crate::git;
use crate::root;
use anyhow::{Context, bail};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

/// The gathered, impure-read state of a `<fork>` the gc classifier reasons over
/// (design §8.2). Every field is a FACT gathered in the shell — the pure
/// [`classify_gc`] never reads git/disk/env.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GcState {
    /// `<fork>` branch resolves to a commit (the branch exists).
    pub(crate) branch_exists: bool,
    /// `<fork>` has a live linked worktree checked out.
    pub(crate) worktree_present: bool,
    /// The landed-oracle verdict, computed in the shell ONLY while the branch
    /// lives (`None` when the branch is gone — the gate is skipped because the
    /// deletion of a fork branch IS the landing certificate, design §8.2).
    pub(crate) landed_verdict: Option<bool>,
}

/// The destructive steps a positive-verdict gc will take, in the design §8 forced
/// order (worktree before branch, because `git branch -D` refuses a checked-out
/// branch). A step is only set when its target is actually present — reaping an
/// absent thing is a no-op, so completed steps are simply skipped on a rerun
/// (design §8.2 idempotence).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GcPlan {
    /// `git worktree remove` the fork's live linked worktree (removes its marker).
    pub(crate) remove_worktree: bool,
    /// `git branch -D` the fork branch (never a git-ancestor on the import route).
    pub(crate) delete_branch: bool,
}

/// Why a gc refuses to reap (design §8.1). Fails closed with a named token.
/// SEPARATE from [`Refusal`]/[`LandRefusal`] — gc's reap-vs-refuse decision is its
/// own verb; do NOT widen the import/land enums.
///
/// **One refusal, not two (design-faithful collapse — orchestrator to confirm).**
/// The design names a "squash-uncertifiable" case, but a manually squash-merged
/// fork is STRUCTURALLY INDISTINGUISHABLE from a never-landed fork: a multi-commit
/// `git merge --squash` yields `git cherry HEAD <fork>` = `+` lines, exactly like a
/// never-landed fork (verified empirically; a *single*-commit squash yields `-` and
/// is correctly certified as landed). There is no empty-`cherry` squash signal, so
/// the oracle cannot split the two states. The design's "named message" is therefore
/// realised as the `not-landed` refusal message NAMING the squash remedy — the user
/// gets the `worktree land --no-ff` / `--force` guidance whether they squashed or
/// never landed, which is the right action either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GcRefusal {
    /// The fork has NOT provably landed (non-ancestor tip with a `+` in `git
    /// cherry` — a never-landed fork OR a manual squash-merge) and neither
    /// `--superseded-head <head>` nor `--force` was given.
    NotLanded,
}

impl GcRefusal {
    /// The distinct named token each refusal fails closed with (the property the
    /// VT goldens assert, not a proxy).
    pub(crate) fn token(self) -> &'static str {
        match self {
            GcRefusal::NotLanded => "not-landed",
        }
    }
}

/// The verdict of the pure gc classifier: a [`GcPlan`] of steps to take, or a named
/// [`GcRefusal`]. `--dry-run` short-circuits to a plan-less verdict in the shell
/// (it never reaches the destructive plan), so the classifier only ever describes
/// what WOULD happen — the shell decides whether to execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GcVerdict {
    /// Reap per this plan (the operator authorised it: positive oracle / matching
    /// `--superseded-head` / `--force`).
    Reap(GcPlan),
    /// Fail closed with this named refusal — destroy nothing.
    Refuse(GcRefusal),
}

/// PURE gc classifier (no git / disk / env — ADR-001 leaf, CLAUDE.md
/// pure/imperative split). Mirror of [`classify_import`]/[`classify_land`]: it
/// takes the gathered FACTS plus the operator's `force` / `superseded_match` /
/// `dry_run` intents and returns the verdict (design §8.2).
///
/// The reap GATE (whether deletion is authorised) is decided here from:
/// * a positive `state.landed_verdict` (the oracle passed — only ever `Some` while
///   the branch lives, since the gate requires the branch),
/// * OR `superseded_match` (the operator asserted `--superseded-head` == the live
///   head: a TOCTOU movement-guard, not a landing proof),
/// * OR `force` (the operator knowingly bypassed the oracle),
/// * OR **branch-gone**: a fork branch is deleted only via `branch -D` AFTER the
///   gate passed, so a gone branch is ALREADY certified — and its worktree (with the
///   in-tree `target/` that lived inside it) is already gone too, so there is nothing
///   left to reap (an idempotent no-op, design §8.2).
///
/// `force`/`superseded_match` authorise the reap and skip the refusal (the operator
/// chose to). `dry_run` does NOT change the verdict — it is honoured in the shell
/// (compute + print, act on nothing); the classifier still reports the would-be
/// plan/refusal so the dry-run print is the SAME verdict a real run would act on.
pub(crate) fn classify_gc(
    state: GcState,
    force: bool,
    superseded_match: bool,
    _dry_run: bool,
) -> GcVerdict {
    // Branch-gone ⇒ already-certified ⇒ the ONLY residue is the target dir.
    // (A live linked worktree on a gone branch is git-impossible — `branch -D`
    // refuses a checked-out branch — so worktree_present is moot here.)
    if !state.branch_exists {
        return GcVerdict::Reap(GcPlan {
            remove_worktree: false,
            delete_branch: false,
        });
    }

    // Branch alive: decide the reap gate. Operator overrides skip the oracle.
    let authorised = force || superseded_match || state.landed_verdict == Some(true);
    if !authorised {
        // Not provably landed (a `+` in `git cherry` — never-landed OR a manual
        // squash-merge; the two are indistinguishable). The message names the
        // squash remedy regardless, so the operator gets the right guidance.
        return GcVerdict::Refuse(GcRefusal::NotLanded);
    }

    // Authorised: reap the present things in the forced order (skip absent ones).
    GcVerdict::Reap(GcPlan {
        remove_worktree: state.worktree_present,
        delete_branch: true,
    })
}

/// The reap set a [`GcPlan`] would act on, as a `/`-joined token list for the
/// dry-run print — the ACTUAL legs, never a blanket `worktree/branch` (a branch-gone
/// plan reaps nothing — the in-tree `target/` died with the worktree dir, F-5).
fn reap_targets(plan: GcPlan) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if plan.remove_worktree {
        parts.push("worktree");
    }
    if plan.delete_branch {
        parts.push("branch");
    }
    if parts.is_empty() {
        "nothing".to_owned()
    } else {
        parts.join("/")
    }
}

/// The landed-oracle (design §8.1), gathered in the shell: true ONLY when the
/// fork's commit has provably landed, tested against durable git state — TWO LEGS,
/// UNION:
/// * **ancestry leg** — `<fork-tip>` is an ancestor of coordination HEAD (the
///   `land` route, `merge-base --is-ancestor` exit 0) ⇒ landed;
/// * **patch-id leg** — `git cherry <coord-HEAD> <fork>` lists at least one commit
///   and EVERY listed commit is `-` prefixed (the `import` route: ancestry severed,
///   but each patch landed) ⇒ landed. A `+` prefix = a commit whose patch is NOT
///   upstream ⇒ not landed.
///
/// **Crash-proof:** a crash between apply and commit leaves no commit ⇒ `git
/// cherry` reports `+` ⇒ NOT landed ⇒ gc refuses (a receipt would have lied
/// "landed" and reaped the only copy).
///
/// **Squash:** a multi-commit `git merge --squash` yields `+` lines (each fork
/// commit's patch-id is unmatched by the combined squash commit) — STRUCTURALLY
/// INDISTINGUISHABLE from a never-landed fork (a *single*-commit squash yields `-`
/// and IS correctly certified — its content is in HEAD). There is no empty-`cherry`
/// squash signal, so the oracle returns plain `not-landed`; the refusal message
/// names the squash remedy. (See [`GcRefusal`] — design-faithful collapse.)
///
/// An EMPTY `git cherry` with a non-ancestor tip means no fork commit's patch is
/// reachable AND none is unmatched — i.e. nothing to certify ⇒ NOT landed (conservative:
/// never reap on a vacuous true). Impure (the two git reads).
fn gather_landed(root: &Path, fork: &str) -> anyhow::Result<bool> {
    // ancestry leg: <fork> is an ancestor of HEAD.
    if git::git_status_ok(root, &["merge-base", "--is-ancestor", fork, "HEAD"])? {
        return Ok(true);
    }
    // patch-id leg: a non-empty `git cherry HEAD <fork>` whose every line is `-`.
    let cherry = git::git_cherry(root, "HEAD", fork)?;
    Ok(!cherry.is_empty() && cherry.iter().all(|line| line.starts_with('-')))
}

/// `doctrine worktree gc --fork <branch> [--superseded-head <SHA>] [--force]
/// [--dry-run]` — reap a spent worktree fork in ONE idempotent act (design §8),
/// deleting ONLY when the fork has provably landed (design §8.1) and completing /
/// naming any leftover on a crash-rerun (design §8.2). Runs at the coordination
/// root. Orchestrator-classed; refused under worker-mode by `worker_guard`.
///
/// Gather → pure-classify → act, patterned after [`run_land`]:
/// 1. gather the FACTS — `<fork>` existence; its live linked worktree (via the
///    SHARED [`gather_fork_worktree`]); the landed oracle (via [`gather_landed`],
///    ONLY while the branch lives); and the `--superseded-head == current-head`
///    movement-guard match,
/// 2. [`classify_gc`] returns `Reap(plan)` or `Refuse(token)`,
/// 3. on `--dry-run`, PRINT the verdict and destroy NOTHING; otherwise execute the
///    plan in the forced order (worktree → branch), each destructive step honest on
///    failure (names its leftover, exits non-zero), folding a stale admin worktree
///    entry via `git worktree prune`. The fork's in-tree `target/` dies with the
///    worktree dir (SL-156 — no separate reap). Finally stderr-WARN the
///    `CARGO_MANIFEST_DIR`-baked-test-binary recompile.
pub(crate) fn run_gc(
    path: Option<PathBuf>,
    fork: &str,
    superseded_head: Option<&str>,
    force: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let root =
        fs::canonicalize(&root).with_context(|| format!("canonicalize root {}", root.display()))?;

    // --- gather: branch existence (resolves to a commit) ---
    let branch_ref = format!("refs/heads/{fork}");
    let branch_head = git::git_opt(
        &root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{branch_ref}^{{commit}}"),
        ],
    )?;
    let branch_exists = branch_head.is_some();

    // --- gather: the fork's live linked worktree (shared gather) ---
    // The fork's in-tree `target/` lives INSIDE this worktree dir (SL-156), so
    // removing the worktree reaps the target with it — no separate target gather.
    let fork_wt = gather_fork_worktree(&root, fork)?;
    let worktree_present = fork_wt.is_some();

    // --- gather: the landed oracle (ONLY while the branch lives — design §8.2) ---
    let landed_verdict = if branch_exists {
        Some(gather_landed(&root, fork)?)
    } else {
        None
    };

    // --- gather: --superseded-head movement-guard match (SHA == CURRENT head) ---
    // A movement-guard, not a landing proof: reaps iff the asserted SHA equals the
    // branch's current head (TOCTOU guard — a stale SHA cannot match a live head).
    let superseded_match = match (superseded_head, &branch_head) {
        (Some(sha), Some(head)) => {
            // Resolve the operator's SHA to a commit before comparing (never trust a
            // symbolic ref verbatim); an unresolvable SHA simply cannot match.
            match git::git_opt(
                &root,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("{sha}^{{commit}}"),
                ],
            )? {
                Some(resolved) => matches(&resolved, head),
                None => false,
            }
        }
        _ => false,
    };

    let state = GcState {
        branch_exists,
        worktree_present,
        landed_verdict,
    };

    // --- pure classify ---
    let verdict = classify_gc(state, force, superseded_match, dry_run);

    // --- dry-run: PRINT the verdict, destroy NOTHING (the operator never --forces blind) ---
    if dry_run {
        match verdict {
            GcVerdict::Reap(plan) => {
                // Report the TRUTH the operator needs before a real run: the actual
                // landed verdict + whether the reap is oracle- or override-authorised,
                // and the ACTUAL reap set — never a blanket `landed ✓ (worktree/
                // branch)` that lies on a forced or branch-gone reap (F-5).
                let basis = if !branch_exists {
                    "already-certified (branch gone)".to_owned()
                } else if landed_verdict == Some(true) {
                    "landed ✓ (oracle)".to_owned()
                } else {
                    let how = if force {
                        "--force"
                    } else {
                        "--superseded-head"
                    };
                    format!("NOT landed — reap authorised by {how} (oracle override)")
                };
                writeln!(
                    io::stdout(),
                    "{fork}: {basis} — would reap ({})",
                    reap_targets(plan)
                )?;
            }
            GcVerdict::Refuse(GcRefusal::NotLanded) => {
                writeln!(
                    io::stdout(),
                    "{fork}: not-landed — `--force` to reap, or `--superseded-head <SHA>` if spent-and-abandoned. If you squash-merged, re-land via `worktree land` (--no-ff)."
                )?;
            }
        }
        return Ok(());
    }

    // --- act ---
    // The lone refusal NAMES the squash remedy too (a squash-merge is
    // indistinguishable from a never-landed fork — see `GcRefusal`).
    let plan = match verdict {
        GcVerdict::Refuse(GcRefusal::NotLanded) => bail!(
            "gc-refused: {} — fork {fork} has not provably landed; `--force` to reap, or `--superseded-head <SHA>` to assert it is spent-and-abandoned. Cannot certify a squash-merge — re-land via `worktree land` (--no-ff), or `--force` knowingly.",
            GcRefusal::NotLanded.token()
        ),
        GcVerdict::Reap(plan) => plan,
    };

    let mut leftovers: Vec<String> = Vec::new();

    // Step 1: remove the live linked worktree FIRST (it holds the marker, and
    // `branch -D` would refuse a checked-out branch). Fold a stale administrative
    // entry via `git worktree prune` before believing a removal failed.
    if let (true, Some(wt)) = (plan.remove_worktree, fork_wt.as_deref()) {
        let removed = git::git_opt(
            &root,
            &["worktree", "remove", "--force", &wt.to_string_lossy()],
        )?;
        if removed.is_none() {
            // Fold a stale admin entry, then re-check whether the dir survives.
            drop(git::git_opt(&root, &["worktree", "prune"]));
            if wt.exists() {
                leftovers.push(format!("worktree {}", wt.display()));
            }
        }
    }

    // Step 2: delete the branch (never a git-ancestor on the import route, so `-d`
    // always refuses — the patch-id gate, not `-d`, is the safety; use `-D`).
    if plan.delete_branch {
        let deleted = git::git_opt(&root, &["branch", "-D", fork])?;
        if deleted.is_none()
            && git::git_opt(&root, &["rev-parse", "--verify", "--quiet", &branch_ref])?.is_some()
        {
            leftovers.push(format!("branch {fork}"));
        }
    }

    // The fork's in-tree `target/` needs no separate reap step — it lived inside the
    // worktree dir and died with the `git worktree remove` above (SL-156).

    if !leftovers.is_empty() {
        bail!(
            "gc-incomplete: leftover(s) need manual cleanup: {}",
            leftovers.join(", ")
        );
    }

    // Step 3: WARN that env!(CARGO_MANIFEST_DIR)-baked test binaries now point at a
    // deleted fork path and must be recompiled (mem.pattern.dispatch.worktree-
    // removal-stale-manifest-dir-false-red).
    writeln!(
        io::stderr(),
        "warning: test binaries baked with the reaped fork's CARGO_MANIFEST_DIR are now stale — recompile before trusting a RED"
    )?;
    writeln!(
        io::stdout(),
        "gc {fork}: reaped (worktree/branch as present)"
    )?;
    Ok(())
}
