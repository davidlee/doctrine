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
    /// The landed-oracle verdict, computed in the shell ONLY when it is actually
    /// consulted. `None` ⇔ it was NOT read: either the branch is gone (the
    /// deletion of a fork branch IS the landing certificate, design §8.2), or
    /// `funnel_landed` already certified the fork, in which case the oracle is
    /// skipped entirely (SL-228 PHASE-08 EX-1 — "first" is literal).
    pub(crate) landed_verdict: Option<bool>,
    /// The FUNNEL RECORD certifies this fork landed (SL-228 PHASE-08, ISS-245) —
    /// INJECTED by the caller that owns funnel knowledge, never derived here: gc
    /// is ENGINE tier and may not import `crate::dispatch` (ADR-001 — `dispatch →
    /// worktree` already exists, so the reverse edge would close a same-tier
    /// cycle). The proof is `crate::dispatch::funnel::resolve_landing`'s
    /// three-check conjunction: EXACTLY one funnel row's `spawn.fork` names this
    /// branch, that row stands at `concluded` (or `reaped`, its replay), and the
    /// LIVE branch oid still equals that row's `import.fork_tip`. gc consumes the
    /// FACT and never learns who produced it (the `WorktreeCommand::Import`
    /// selector-resolution precedent) — but a reader here should not have to
    /// cross a module boundary to learn what the bool asserts.
    pub(crate) funnel_landed: bool,
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

    /// The operator REMEDY for this refusal, single-sourced (STD-001, SL-228 PHASE-08
    /// D-P8-6): the CLI's `gc-refused` message and the MCP `Refused{detail}` carry the
    /// SAME string, not two paraphrases that drift. Names the funnel route FIRST,
    /// because a funnel-managed fork's landing proof is the funnel record — the CLI gc
    /// alone cannot read it (D-P8-2), so `--force` is the wrong reflex there.
    pub(crate) fn remedy(self) -> &'static str {
        match self {
            GcRefusal::NotLanded => {
                "if this fork is funnel-managed, reap it with `dispatch_reap` (the funnel record is its landing proof); \
                 otherwise `--force` to reap knowingly, or `--superseded-head <SHA>` to assert it is spent-and-abandoned. \
                 A squash-merge cannot be certified — re-land via `worktree land` (--no-ff)."
            }
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
/// takes the gathered FACTS plus the operator's `force` / `superseded_match`
/// intents and returns the verdict (design §8.2).
///
/// The reap GATE (whether deletion is authorised) is decided here from:
/// * **`state.funnel_landed`** — the injected funnel-record proof, consulted FIRST
///   (SL-228 PHASE-08 EX-1): when it holds, the shell never even reads the git
///   oracle, because the record is the stronger authority (the atomic import makes
///   `git cherry` report every funnel-managed fork unlanded — ISS-245),
/// * OR a positive `state.landed_verdict` (the git oracle passed — only ever `Some`
///   when it was actually consulted),
/// * OR `superseded_match` (the operator asserted `--superseded-head` == the live
///   head: a TOCTOU movement-guard, not a landing proof),
/// * OR `force` (the operator knowingly bypassed the oracle),
/// * OR **branch-gone**: a fork branch is deleted only via `branch -D` AFTER the
///   gate passed, so a gone branch is ALREADY certified — and its worktree (with the
///   in-tree `target/` that lived inside it) is already gone too, so there is nothing
///   left to reap (an idempotent no-op, design §8.2).
///
/// `force`/`superseded_match` authorise the reap and skip the refusal (the operator
/// chose to). `--dry-run` is NOT a parameter: the classifier cannot see it, so the
/// dry-run print is the SAME verdict a real run acts on BY CONSTRUCTION (SL-228
/// PHASE-08 — the property the removed `_dry_run` argument used to be asserted for).
pub(crate) fn classify_gc(state: GcState, force: bool, superseded_match: bool) -> GcVerdict {
    // Branch-gone ⇒ already-certified ⇒ the ONLY residue is the target dir.
    // (A live linked worktree on a gone branch is git-impossible — `branch -D`
    // refuses a checked-out branch — so worktree_present is moot here.)
    if !state.branch_exists {
        return GcVerdict::Reap(GcPlan {
            remove_worktree: false,
            delete_branch: false,
        });
    }

    // Branch alive: decide the reap gate. The injected funnel proof is read FIRST,
    // then the operator overrides, then the git oracle.
    let authorised =
        state.funnel_landed || force || superseded_match || state.landed_verdict == Some(true);
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

/// The SHARED landed-oracle (design §8.1), gathered in the shell: true ONLY when
/// `<fork>`'s commit has provably landed against `target` (an arbitrary landing ref,
/// not just coordination HEAD — SL-190 PHASE-04 lift), tested against durable git
/// state — TWO LEGS, UNION:
/// * **ancestry leg** — `<fork-tip>` is an ancestor of `target` (the `land` route,
///   `merge-base --is-ancestor` exit 0) ⇒ landed;
/// * **patch-id leg** — `git cherry <target> <fork>` lists at least one commit
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
/// and IS correctly certified — its content is in `target`). There is no empty-`cherry`
/// squash signal, so the oracle returns plain `not-landed`; the refusal message
/// names the squash remedy. (See [`GcRefusal`] — design-faithful collapse.)
///
/// An EMPTY `git cherry` with a non-ancestor tip means no fork commit's patch is
/// reachable AND none is unmatched — i.e. nothing to certify ⇒ NOT landed (conservative:
/// never reap on a vacuous true). Impure (the two git reads).
///
/// The return stays a clean total `bool` over a VALID `target`: the missing-target →
/// unknown tri-state is the CALLER's concern (PHASE-05 inventory), never the oracle's,
/// so this shared machinery stays behaviour-preserving. gc passes `"HEAD"`.
pub(crate) fn landed_against(root: &Path, target: &str, fork: &str) -> anyhow::Result<bool> {
    // ancestry leg: <fork> is an ancestor of <target>.
    if git::git_status_ok(root, &["merge-base", "--is-ancestor", fork, target])? {
        return Ok(true);
    }
    // patch-id leg: a non-empty `git cherry <target> <fork>` whose every line is `-`.
    let cherry = git::git_cherry(root, target, fork)?;
    Ok(!cherry.is_empty() && cherry.iter().all(|line| line.starts_with('-')))
}

// ---------------------------------------------------------------------------
// ONE machine, THREE shells (SL-228 PHASE-08 D-P8-3): `gather` → `classify_gc` →
// `act` are shared; the typed [`reap_fork`] entry point and the CLI
// [`run_gc`]/[`run_gc_to`] sit over them. No parallel implementation — the CLI's
// byte-compatible bail texts are re-added in its own shell, so PHASE-01's
// behaviour-preservation suite (`tests/e2e_worktree_gc.rs`) holds by construction.
// ---------------------------------------------------------------------------

/// What a completed [`act`] FAILED to reap — a SET, not a first-failure, because the
/// steps are independent and the operator needs every member (the CLI folds the whole
/// set into ONE `gc-incomplete` message). Classified by the caller: `worktree`/`branch`
/// are CRITICAL residue (the fork is still there), a leftover `dispatch_record` is
/// ADMINISTRATIVE (the fork IS gone; only a stale record survives).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Residue {
    /// The fork's linked worktree dir survived the removal (+ `worktree prune`).
    pub(crate) worktree: bool,
    /// The fork branch survived `branch -D`.
    pub(crate) branch: bool,
    /// The per-worktree dispatch record could not be deleted; the payload is the
    /// already-formatted cause.
    pub(crate) dispatch_record: Option<String>,
}

impl Residue {
    /// Nothing was left behind.
    pub(crate) fn is_empty(&self) -> bool {
        !self.worktree && !self.branch && self.dispatch_record.is_none()
    }

    /// Every leftover this residue names, in the design §8 step order — THE one
    /// rendering both the CLI `gc-incomplete` bail and the MCP `Refused{detail}` read
    /// (STD-001). `worktree_dir` is the caller's own gathered path when it has one.
    pub(crate) fn members(&self, fork: &str, worktree_dir: Option<&Path>) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();
        if self.worktree {
            parts.push(match worktree_dir {
                Some(dir) => format!("worktree {}", dir.display()),
                None => format!("worktree of {fork} (see `doctrine worktree list`)"),
            });
        }
        if self.branch {
            parts.push(format!("branch {fork}"));
        }
        if let Some(cause) = self.dispatch_record.as_deref() {
            parts.push(format!("dispatch record: {cause}"));
        }
        parts
    }
}

/// What a [`reap_fork`] run actually DID — the typed outcome a programmatic caller
/// acts on (SL-228 PHASE-08, ISS-246). Every arm here is a NORMAL outcome: `Err` is
/// reserved for pre-act faults (root resolution, the gather reads), never for a
/// verdict the caller is expected to relay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GcOutcome {
    /// The fork was present and is now reaped.
    Reaped,
    /// There was nothing left to reap — idempotent completion, NOT an error
    /// (design §8.2; SL-228 PHASE-08 D-P8-4).
    AlreadyAbsent,
    /// An active claimant holds the fork's claim lock: nothing was classified and
    /// nothing deleted. RETURNED, never printed — a caller writing to `io::sink()`
    /// cannot observe a printed skip.
    Busy,
    /// The fork has not provably landed: neither the injected funnel proof nor the
    /// git oracle authorised the reap. Nothing was deleted.
    NotLanded,
    /// The act ran but left something behind.
    Residual(Residue),
}

impl GcOutcome {
    /// Whether the funnel position may advance to `reaped` on this outcome: the fork
    /// is gone (or was already), so recording the milestone tells the truth. CRITICAL
    /// residue outranks ADMINISTRATIVE — a surviving worktree/branch blocks the
    /// advance, a stale dispatch record alone does not.
    pub(crate) fn advances(&self) -> bool {
        match self {
            GcOutcome::Reaped | GcOutcome::AlreadyAbsent => true,
            GcOutcome::Busy | GcOutcome::NotLanded => false,
            GcOutcome::Residual(residue) => !residue.worktree && !residue.branch,
        }
    }
}

/// The per-name claim hold a gc verb runs UNDER, plus the claim name it resolved
/// (`None` for a fork outside the `dispatch/<name>` claim namespace — nothing to hold,
/// and the dispatch-record key falls back to the fork name). The lock field is bound,
/// never dropped early: it lives exactly as long as this value.
struct ClaimHold {
    name: Option<String>,
    _lock: Option<super::claim_lock::ClaimLock>,
}

/// Take the per-name claim lock BEFORE anything is classified or deleted (design §3 /
/// RV-304 F-2). `Ok(None)` ⇔ BUSY: an active claimant holds the name — a spawn mid
/// claim→bind→act looks byte-for-byte like crash residue, so sweeping it would let a
/// second spawn re-claim the name and cross-pair the bindings. Non-blocking on purpose:
/// gc never waits on a live claimant, it skips it this pass. A CRASHED spawn's lock is
/// already kernel-released, so its residue sweeps freely.
fn hold_claim(root: &Path, fork: &str) -> anyhow::Result<Option<ClaimHold>> {
    let name = fork
        .strip_prefix("dispatch/")
        .and_then(|n| super::create::sanitise_name(n).ok());
    let lock = match name.as_deref() {
        Some(name) => super::claim_lock::try_acquire(root, name)?,
        None => None,
    };
    if name.is_some() && lock.is_none() {
        return Ok(None);
    }
    Ok(Some(ClaimHold { name, _lock: lock }))
}

/// The gathered FACTS plus the two shell-only reads the classifier does not take: the
/// branch's current head (the `--superseded-head` movement-guard's right-hand side) and
/// the fork's live worktree dir (the removal target and the residue's name).
struct Gathered {
    state: GcState,
    branch_head: Option<String>,
    fork_wt: Option<PathBuf>,
}

/// GATHER (impure): branch existence, the fork's live linked worktree, and — ONLY when
/// the injected `funnel_landed` proof does NOT already hold and the branch still lives
/// — the git landed-oracle. Skipping the oracle under the funnel proof is EX-1's
/// "consulted first", literally: the `git cherry` read never happens.
fn gather(root: &Path, fork: &str, funnel_landed: bool) -> anyhow::Result<Gathered> {
    // --- branch existence (resolves to a commit) ---
    let branch_head = git::git_opt(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{fork}^{{commit}}"),
        ],
    )?;
    let branch_exists = branch_head.is_some();

    // --- the fork's live linked worktree (shared gather) ---
    // The fork's in-tree `target/` lives INSIDE this worktree dir (SL-156), so
    // removing the worktree reaps the target with it — no separate target gather.
    let fork_wt = gather_fork_worktree(root, fork)?;

    // --- the landed oracle (only while the branch lives AND nothing certified it) ---
    let landed_verdict = if branch_exists && !funnel_landed {
        // gc lands against coordination HEAD (the shared oracle's `target`).
        Some(landed_against(root, "HEAD", fork)?)
    } else {
        None
    };

    Ok(Gathered {
        state: GcState {
            branch_exists,
            worktree_present: fork_wt.is_some(),
            landed_verdict,
            funnel_landed,
        },
        branch_head,
        fork_wt,
    })
}

/// ACT (impure, TOTAL): execute `plan` in the design §8 forced order and report what
/// was left behind. Never bails and never prints — every failure becomes a [`Residue`]
/// member, so a caller can decide between "refuse" and "advance and name the leftover"
/// instead of having the decision made for it by a `?`.
fn act(
    root: &Path,
    plan: GcPlan,
    fork: &str,
    fork_wt: Option<&Path>,
    record_name: &str,
) -> Residue {
    let mut residue = Residue::default();

    // Step 1: remove the live linked worktree FIRST (it holds the marker, and
    // `branch -D` would refuse a checked-out branch). Fold a stale administrative
    // entry via `git worktree prune` before believing a removal failed.
    if let (true, Some(wt)) = (plan.remove_worktree, fork_wt) {
        let removed = git::git_opt(
            root,
            &["worktree", "remove", "--force", &wt.to_string_lossy()],
        )
        .ok()
        .flatten();
        if removed.is_none() {
            // Fold a stale admin entry, then re-check whether the dir survives.
            drop(git::git_opt(root, &["worktree", "prune"]));
            residue.worktree = wt.exists();
        }
    }

    // Step 2: delete the branch (never a git-ancestor on the import route, so `-d`
    // always refuses — the landed gate, not `-d`, is the safety; use `-D`).
    if plan.delete_branch {
        let deleted = git::git_opt(root, &["branch", "-D", fork]).ok().flatten();
        let branch_ref = format!("refs/heads/{fork}");
        if deleted.is_none() {
            residue.branch = git::git_opt(root, &["rev-parse", "--verify", "--quiet", &branch_ref])
                .ok()
                .flatten()
                .is_some();
        }
    }

    // The fork's in-tree `target/` needs no separate reap step — it lived inside the
    // worktree dir and died with the `git worktree remove` above (SL-156).

    // Step 3 (SL-198 PHASE-01): delete the per-worktree dispatch record, co-located
    // with the worktree+branch reap — no record survives a reaped worktree (closes the
    // stale-oracle, design §5.3 EX-2). Keyed off the SAME name the claim gate resolved
    // (one derivation, no re-spell); a no-op for a non-dispatch fork or a rerun. Runs
    // UNDER the held claim lock, so a residue can never acquire a worktree concurrently
    // with its sweep.
    if let Err(cause) = super::dispatch_record::delete_dispatch_record(root, record_name) {
        residue.dispatch_record = Some(format!("{cause:#}"));
    }

    residue
}

/// The `--dry-run` BASIS line: the truth the operator needs before a real run — the
/// actual landed authority and whether the reap is proof- or override-authorised.
/// Never a blanket `landed ✓` that lies on a forced or branch-gone reap (F-5). PURE.
fn dry_run_basis(state: GcState, force: bool) -> String {
    if !state.branch_exists {
        "already-certified (branch gone)".to_owned()
    } else if state.funnel_landed {
        "landed ✓ (funnel record)".to_owned()
    } else if state.landed_verdict == Some(true) {
        "landed ✓ (oracle)".to_owned()
    } else {
        let how = if force {
            "--force"
        } else {
            "--superseded-head"
        };
        format!("NOT landed — reap authorised by {how} (oracle override)")
    }
}

/// The TYPED reap entry point (SL-228 PHASE-08 T6, ISS-246) — the seam a programmatic
/// caller (`dispatch_reap`) drives: gather → classify → act, returning a [`GcOutcome`]
/// it can relay as a structured refusal instead of a flattened `-32603`.
///
/// The exclusions are BY SIGNATURE, not by call-site discipline: there is no `force`
/// (a funnel verb never bypasses its own proof), no `dry_run` (nothing to print), and
/// no `out` — this entry point writes NOTHING, on any stream. That last one is
/// load-bearing: the CLI's report goes to process stdout, which for an MCP server IS
/// the JSON-RPC wire (D-P8-7), and `Busy` is RETURNED rather than printed because a
/// caller passing `io::sink()` could never observe a printed skip.
///
/// `funnel_landed` is the injected landing proof (see [`GcState::funnel_landed`]).
/// `Err` is reserved for PRE-ACT faults: root resolution and the gather reads.
pub(crate) fn reap_fork(
    path: Option<PathBuf>,
    fork: &str,
    funnel_landed: bool,
) -> anyhow::Result<GcOutcome> {
    let root = root::find(path, &root::default_markers())?;
    let root =
        fs::canonicalize(&root).with_context(|| format!("canonicalize root {}", root.display()))?;

    // The claim gate outranks every authority below it: it runs BEFORE anything is
    // classified, so a live claimant is skipped rather than swept.
    let Some(claim) = hold_claim(&root, fork)? else {
        return Ok(GcOutcome::Busy);
    };

    let gathered = gather(&root, fork, funnel_landed)?;
    // No operator overrides on this path — the funnel's own proof is the only authority
    // beyond the shared oracle.
    let plan = match classify_gc(gathered.state, false, false) {
        GcVerdict::Refuse(GcRefusal::NotLanded) => return Ok(GcOutcome::NotLanded),
        GcVerdict::Reap(plan) => plan,
    };

    let record_name = claim.name.as_deref().unwrap_or(fork);
    let residue = act(&root, plan, fork, gathered.fork_wt.as_deref(), record_name);
    Ok(if residue.is_empty() {
        if gathered.state.branch_exists {
            GcOutcome::Reaped
        } else {
            // Nothing was there to delete: the run COMPLETES (D-P8-4), it does not err.
            GcOutcome::AlreadyAbsent
        }
    } else {
        GcOutcome::Residual(residue)
    })
}

/// `doctrine worktree gc --fork <branch> [--superseded-head <SHA>] [--force]
/// [--dry-run]` — reap a spent worktree fork in ONE idempotent act (design §8),
/// deleting ONLY when the fork has provably landed (design §8.1) and completing /
/// naming any leftover on a crash-rerun (design §8.2). Runs at the coordination
/// root. Orchestrator-classed; refused under worker-mode by `worker_guard`.
///
/// Gather → pure-classify → act, patterned after [`run_land`]:
/// 1. gather the FACTS — `<fork>` existence; its live linked worktree (via the
///    SHARED [`gather_fork_worktree`]); the landed oracle (via [`landed_against`]
///    with `"HEAD"`, ONLY while the branch lives); and the `--superseded-head == current-head`
///    movement-guard match,
/// 2. [`classify_gc`] returns `Reap(plan)` or `Refuse(token)`,
/// 3. on `--dry-run`, PRINT the verdict and destroy NOTHING; otherwise execute the
///    plan in the forced order (worktree → branch), each destructive step honest on
///    failure (names its leftover, exits non-zero), folding a stale admin worktree
///    entry via `git worktree prune`. The fork's in-tree `target/` dies with the
///    worktree dir (SL-156 — no separate reap). Finally stderr-WARN the
///    `CARGO_MANIFEST_DIR`-baked-test-binary recompile.
///
/// The CLI entry: the reap report goes to STDOUT. A programmatic caller that runs
/// over the MCP stdio JSON-RPC channel (fd 1) must use [`run_gc_to`] with a non-stdout
/// sink instead, or the report corrupts the wire (SL-199 F3).
pub(crate) fn run_gc(
    path: Option<PathBuf>,
    fork: &str,
    superseded_head: Option<&str>,
    force: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    run_gc_to(
        path,
        fork,
        superseded_head,
        force,
        dry_run,
        &mut io::stdout().lock(),
    )
}

/// The gc machine with an INJECTED report sink (SL-199 F3). Identical to [`run_gc`]
/// except the human-readable reap/dry-run report is written to `out` rather than the
/// process stdout — so an MCP tool (`dispatch_reap`) can pass [`io::sink`] and keep the
/// JSON-RPC channel clean. The stderr recompile warning is untouched (stderr is not the
/// MCP wire).
pub(crate) fn run_gc_to(
    path: Option<PathBuf>,
    fork: &str,
    superseded_head: Option<&str>,
    force: bool,
    dry_run: bool,
    out: &mut dyn io::Write,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let root =
        fs::canonicalize(&root).with_context(|| format!("canonicalize root {}", root.display()))?;

    // The claim gate (design §3 / RV-304 F-2) — held for the rest of the verb, so no
    // claimant can acquire the name between our classify and our delete. A BUSY name is
    // a SKIP, not an error; the CLI shell is the surface that PRINTS it.
    let Some(claim) = hold_claim(&root, fork)? else {
        writeln!(
            out,
            "{fork}: skipped — an active claimant holds the claim lock (a spawn is mid claim→bind→act); nothing classified, nothing deleted"
        )?;
        return Ok(());
    };

    // The CLI keeps the patch-id oracle as its SOLE landed authority (D-P8-2): `gc`
    // takes no `--slice` and derives none, so it has no funnel record to read and
    // injects no funnel fact. `dispatch_reap` is the funnel-aware route (the
    // `not-landed` remedy signposts it).
    let gathered = gather(&root, fork, false)?;
    let state = gathered.state;

    // --- gather: --superseded-head movement-guard match (SHA == CURRENT head) ---
    // A movement-guard, not a landing proof: reaps iff the asserted SHA equals the
    // branch's current head (TOCTOU guard — a stale SHA cannot match a live head).
    let superseded_match = match (superseded_head, &gathered.branch_head) {
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

    // --- pure classify ---
    let verdict = classify_gc(state, force, superseded_match);

    // --- dry-run: PRINT the verdict, destroy NOTHING (the operator never --forces blind) ---
    if dry_run {
        match verdict {
            GcVerdict::Reap(plan) => {
                writeln!(
                    out,
                    "{fork}: {} — would reap ({})",
                    dry_run_basis(state, force),
                    reap_targets(plan)
                )?;
            }
            GcVerdict::Refuse(GcRefusal::NotLanded) => {
                writeln!(
                    out,
                    "{fork}: {} — {}",
                    GcRefusal::NotLanded.token(),
                    GcRefusal::NotLanded.remedy()
                )?;
            }
        }
        return Ok(());
    }

    // --- act ---
    // The lone refusal NAMES every remedy — the funnel route, the overrides, and the
    // squash re-land (a squash-merge is indistinguishable from a never-landed fork —
    // see `GcRefusal`) — through the single-sourced [`GcRefusal::remedy`].
    let plan = match verdict {
        GcVerdict::Refuse(GcRefusal::NotLanded) => bail!(
            "gc-refused: {} — fork {fork} has not provably landed; {}",
            GcRefusal::NotLanded.token(),
            GcRefusal::NotLanded.remedy()
        ),
        GcVerdict::Reap(plan) => plan,
    };

    let record_name = claim.name.as_deref().unwrap_or(fork);
    let residue = act(&root, plan, fork, gathered.fork_wt.as_deref(), record_name);
    if !residue.is_empty() {
        bail!(
            "gc-incomplete: leftover(s) need manual cleanup: {}",
            residue
                .members(fork, gathered.fork_wt.as_deref())
                .join(", ")
        );
    }

    // WARN that env!(CARGO_MANIFEST_DIR)-baked test binaries now point at a deleted
    // fork path and must be recompiled (mem.pattern.dispatch.worktree-removal-stale-
    // manifest-dir-false-red).
    writeln!(
        io::stderr(),
        "warning: test binaries baked with the reaped fork's CARGO_MANIFEST_DIR are now stale — recompile before trusting a RED"
    )?;
    writeln!(out, "gc {fork}: reaped (worktree/branch as present)")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests — the GENERALIZED (non-HEAD `target`) contract of the shared
// [`landed_against`] oracle (SL-190 PHASE-04, EX-3 / VT-1). gc's own HEAD-target
// behaviour is proven UNCHANGED by `tests/e2e_worktree_gc.rs` (EX-2); here we
// exercise a real non-HEAD landing ref against each of the three verdicts:
// landed-via-ancestry, landed-via-patch-id, and not-landed. Missing-target →
// soft unknown is the inventory caller's concern (PHASE-05), not the oracle's.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::landed_against;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// A throwaway git repo with pinned identity — the fixture idiom shared with
    /// `git.rs`'s `ScratchRepo`, minimised for the oracle's needs.
    struct ScratchRepo {
        _dir: tempfile::TempDir,
        path: PathBuf,
    }

    impl ScratchRepo {
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().to_path_buf();
            let repo = Self { _dir: dir, path };
            repo.git(&["init", "-q", "-b", "main"]);
            repo.git(&["config", "user.email", "t@example.com"]);
            repo.git(&["config", "user.name", "Test"]);
            repo
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn git(&self, args: &[&str]) -> String {
            let out = Command::new("git")
                .arg("-C")
                .arg(&self.path)
                .args(args)
                .output()
                .expect("spawn git");
            assert!(
                out.status.success(),
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }

        /// Whether `branch` still resolves to a commit — a NON-asserting probe (the
        /// asserting [`ScratchRepo::git`] cannot ask about an absent ref, since
        /// `rev-parse --verify --quiet` exits 1 on a miss).
        fn branch_exists(&self, branch: &str) -> bool {
            crate::git::git_opt(
                &self.path,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{branch}"),
                ],
            )
            .expect("probe branch")
            .is_some()
        }

        fn commit(&self, rel: &str, contents: &str, message: &str) -> String {
            std::fs::write(self.path.join(rel), contents).expect("write file");
            self.git(&["add", rel]);
            self.git(&["commit", "-q", "-m", message]);
            self.git(&["rev-parse", "HEAD"])
        }
    }

    /// ANCESTRY leg against a NON-HEAD `target`: the fork tip is an ancestor of
    /// `release` (merged in via `--no-ff`) but NOT of HEAD (`main`), so the oracle
    /// must consult the passed `target`, not a hardcoded HEAD.
    #[test]
    fn landed_against_non_head_target_via_ancestry() {
        let repo = ScratchRepo::new();
        repo.commit("base.txt", "0", "C0");
        repo.git(&["checkout", "-q", "-b", "feature"]);
        let fork = "feature";
        repo.commit("feat.txt", "f", "feature work");
        // `release` is the non-HEAD landing target; merge the fork into it.
        repo.git(&["checkout", "-q", "main"]);
        repo.git(&["checkout", "-q", "-b", "release"]);
        repo.git(&["merge", "--no-ff", "-q", "-m", "land feature", fork]);
        // HEAD returns to `main`, which does NOT contain the fork.
        repo.git(&["checkout", "-q", "main"]);
        let target = "release";

        // ancestry leg: `merge-base --is-ancestor <fork> <target>` exit 0 ⇒ landed.
        assert!(landed_against(repo.path(), target, fork).expect("oracle"));
        // ...and NOT landed against HEAD (main), proving the `target` param is honoured.
        assert!(!landed_against(repo.path(), "HEAD", fork).expect("oracle"));
    }

    /// PATCH-ID leg against a NON-HEAD `target`: the fork's patch is cherry-picked
    /// onto `release` (ancestry severed, patch-id equal), so `git cherry <target>
    /// <fork>` yields an all-`-` list ⇒ landed.
    #[test]
    fn landed_against_non_head_target_via_patch_id_cherry() {
        let repo = ScratchRepo::new();
        repo.commit("base.txt", "0", "C0");
        repo.git(&["checkout", "-q", "-b", "feature"]);
        let fork = "feature";
        let fork_tip = repo.commit("feat.txt", "content", "add feat");
        // `release` gets an EQUIVALENT patch via cherry-pick — same patch-id, new SHA.
        repo.git(&["checkout", "-q", "main"]);
        repo.git(&["checkout", "-q", "-b", "release"]);
        repo.git(&["cherry-pick", &fork_tip]);
        repo.git(&["checkout", "-q", "main"]);
        let target = "release";

        // not an ancestor, but `git cherry <target> <fork>` is all `-` ⇒ landed.
        assert!(landed_against(repo.path(), target, fork).expect("oracle"));
    }

    /// NOT LANDED against a NON-HEAD `target`: the fork carries a commit whose patch
    /// is absent from `release` (a `+` in `git cherry`) and is not an ancestor.
    #[test]
    fn not_landed_against_non_head_target() {
        let repo = ScratchRepo::new();
        repo.commit("base.txt", "0", "C0");
        repo.git(&["checkout", "-q", "-b", "feature"]);
        let fork = "feature";
        repo.commit("only.txt", "only", "unlanded work");
        // `release` diverges with unrelated work — the fork's patch never lands.
        repo.git(&["checkout", "-q", "main"]);
        repo.git(&["checkout", "-q", "-b", "release"]);
        repo.commit("other.txt", "other", "unrelated release work");
        repo.git(&["checkout", "-q", "main"]);
        let target = "release";

        // neither `--is-ancestor` nor an all-`-` `cherry` ⇒ not landed.
        assert!(!landed_against(repo.path(), target, fork).expect("oracle"));
    }

    // --- SL-198 PHASE-01 (VT-2): reap deletes the per-worktree DispatchRecord -----
    // A reaped worker worktree must leave NO record behind (closes the stale-oracle):
    // after gc removes the worktree+branch, a `resolve_agent` of the same agent yields
    // `unknown-agent`.
    #[test]
    fn reap_deletes_dispatch_record_and_resolve_yields_unknown_agent() {
        use super::run_gc;
        use crate::worktree::dispatch_record::{
            ForkExpect, RECORD_SUBPATH, ResolveRefusal, provision_dispatch_record, resolve_agent,
        };

        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "0", "base");
        let coord = std::fs::canonicalize(repo.path()).unwrap();

        // Stand up a live worker worktree on dispatch/<name> + its trusted record.
        let name = "agent-cafe";
        let branch = format!("dispatch/{name}");
        let dir = coord.join(".worktrees").join(name);
        let dir_s = dir.to_string_lossy().to_string();
        repo.git(&["worktree", "add", "-b", &branch, &dir_s, &base]);
        provision_dispatch_record(&coord, name, &base, &dir, &branch, None).unwrap();

        let record_file = coord.join(RECORD_SUBPATH).join(format!("{name}.toml"));
        assert!(
            record_file.exists(),
            "the DispatchRecord is written for a live worker"
        );
        assert!(
            resolve_agent(&coord, name, ForkExpect::AtBase).is_ok(),
            "a live, consistent worker resolves pre-reap"
        );

        // Reap the worker fork (--force bypasses the landed oracle).
        run_gc(Some(coord.clone()), &branch, None, true, false).expect("gc reap");

        assert!(
            !record_file.exists(),
            "reap deleted the DispatchRecord — none survives a reaped worktree"
        );
        assert_eq!(
            resolve_agent(&coord, name, ForkExpect::AtBase),
            Err(ResolveRefusal::UnknownAgent),
            "post-reap resolve of the same agent yields unknown-agent"
        );
    }

    // --- SL-228 PHASE-08 (VT-1): the INJECTED funnel proof ------------------------
    //
    // ISS-245: the funnel's atomic one-commit import makes `git cherry` report every
    // funnel-managed fork unlanded, so reap was unreachable on its prescribed path and
    // the operator learned a `--force` reflex. The fix injects a proven FACT; the git
    // oracle is deliberately UNCHANGED and stays the authority for a fork with no
    // funnel row.

    fn state(
        branch_exists: bool,
        worktree_present: bool,
        landed_verdict: Option<bool>,
        funnel_landed: bool,
    ) -> super::GcState {
        super::GcState {
            branch_exists,
            worktree_present,
            landed_verdict,
            funnel_landed,
        }
    }

    /// (a) The injected proof authorises on its own — no `--force`, no
    /// `--superseded-head` — and (b) with the proof absent the git verdict still
    /// decides, unchanged.
    #[test]
    fn the_injected_funnel_proof_authorises_without_force_or_superseded() {
        use super::{GcPlan, GcRefusal, GcVerdict, classify_gc};

        // (a) The oracle was never consulted (`landed_verdict: None`) and the fork is
        // still reaped, on the funnel proof alone.
        assert_eq!(
            classify_gc(state(true, true, None, true), false, false),
            GcVerdict::Reap(GcPlan {
                remove_worktree: true,
                delete_branch: true,
            }),
            "the funnel record certifies the fork; no operator override is needed"
        );
        // Even a NEGATIVE git verdict is outranked — that is exactly the ISS-245 shape
        // (the atomic import defeats patch-id, the record proves the landing).
        assert!(matches!(
            classify_gc(state(true, true, Some(false), true), false, false),
            GcVerdict::Reap(_)
        ));
        // (b) Proof absent ⇒ the git verdict decides, unchanged in both directions.
        assert_eq!(
            classify_gc(state(true, true, Some(false), false), false, false),
            GcVerdict::Refuse(GcRefusal::NotLanded)
        );
        assert!(matches!(
            classify_gc(state(true, true, Some(true), false), false, false),
            GcVerdict::Reap(_)
        ));
    }

    /// (c) The ISS-245 REGRESSION WITNESS: the landing commit's patch is a strict
    /// SUPERSET of the fork's (the fork touches `a.rs`; the atomic import lands `a.rs`
    /// ⊕ `funnel.toml` in ONE commit), so no patch-id matches and `landed_against`
    /// stays `false`. Pinned deliberately — the git oracle is NOT the thing being
    /// fixed. The injected fact is what makes the reap reachable, and `gather` proves
    /// the oracle is skipped entirely when it holds (EX-1's "first" is literal).
    #[test]
    fn an_import_superset_patch_still_reads_not_landed_and_the_funnel_fact_reaps_it() {
        use super::{GcOutcome, GcVerdict, classify_gc, gather, landed_against, reap_fork};

        let repo = ScratchRepo::new();
        repo.commit("base.txt", "0", "C0");
        let base = repo.git(&["rev-parse", "HEAD"]);
        // The worker fork: ONE commit touching a.rs.
        let name = "agent-superset";
        let branch = format!("dispatch/{name}");
        repo.git(&["checkout", "-q", "-b", &branch]);
        repo.commit("a.rs", "fn a() {}\n", "worker work");
        // The coordination landing: a.rs ⊕ funnel.toml in ONE commit (the atomic import).
        repo.git(&["checkout", "-q", "main"]);
        std::fs::write(repo.path().join("a.rs"), "fn a() {}\n").expect("a.rs");
        std::fs::create_dir_all(repo.path().join(".doctrine/dispatch/228")).expect("dir");
        std::fs::write(
            repo.path().join(".doctrine/dispatch/228/funnel.toml"),
            "schema = 1\n",
        )
        .expect("funnel.toml");
        repo.git(&["add", "a.rs", ".doctrine"]);
        repo.git(&["commit", "-q", "-m", "import: delta ⊕ funnel row"]);
        let coord = std::fs::canonicalize(repo.path()).expect("canonicalize");

        // The oracle is UNCHANGED and still says "not landed" — the regression witness.
        assert!(
            !landed_against(&coord, "HEAD", &branch).expect("oracle"),
            "a superset landing commit matches no patch-id: `git cherry` reports `+`"
        );
        assert!(matches!(
            classify_gc(
                gather(&coord, &branch, false).expect("gather").state,
                false,
                false
            ),
            GcVerdict::Refuse(_)
        ));

        // With the proof injected, the oracle is never read AND the reap proceeds.
        let gathered = gather(&coord, &branch, true).expect("gather");
        assert_eq!(
            gathered.state.landed_verdict, None,
            "the `git cherry` read is skipped entirely under the funnel proof"
        );
        assert_eq!(
            reap_fork(Some(coord.clone()), &branch, true).expect("reap"),
            GcOutcome::Reaped,
            "no --force anywhere: the funnel record is the landing proof"
        );
        assert!(!repo.branch_exists(&branch), "the branch is gone");
        assert_ne!(
            base, "",
            "the base commit is real (the fork was a descendant, not an orphan)"
        );
    }

    /// ISS-246 at the gc seam: an unproven fork is a typed OUTCOME, not an `Err` — the
    /// caller relays a diagnosis instead of a flattened internal error, and the fork
    /// survives. A rerun on an already-reaped fork is `AlreadyAbsent`, not a failure.
    #[test]
    fn an_unproven_fork_is_a_typed_outcome_and_a_gone_fork_is_already_absent() {
        use super::{GcOutcome, reap_fork};

        let repo = ScratchRepo::new();
        repo.commit("a.txt", "0", "base");
        let name = "agent-unproven";
        let branch = format!("dispatch/{name}");
        // A genuinely diverged fork: a commit of its own, and coord work that never
        // carries its patch ⇒ the oracle reports `+` ⇒ not landed.
        repo.git(&["checkout", "-q", "-b", &branch]);
        repo.commit("only.txt", "unlanded", "fork work");
        repo.git(&["checkout", "-q", "main"]);
        repo.commit("other.txt", "other", "diverging coord work");
        let coord = std::fs::canonicalize(repo.path()).expect("canonicalize");

        assert_eq!(
            reap_fork(Some(coord.clone()), &branch, false).expect("no Err on a refusal"),
            GcOutcome::NotLanded
        );
        assert!(
            !reap_fork(Some(coord.clone()), &branch, false)
                .expect("outcome")
                .advances(),
            "a refusal never advances the funnel position"
        );
        assert!(
            repo.branch_exists(&branch),
            "the unproven fork survives the refusal"
        );

        // Now let the proof authorise it, then re-run: the second pass is idempotent
        // completion, not an error (D-P8-4).
        assert_eq!(
            reap_fork(Some(coord.clone()), &branch, true).expect("reap"),
            GcOutcome::Reaped
        );
        let replay = reap_fork(Some(coord), &branch, true).expect("replay");
        assert_eq!(replay, GcOutcome::AlreadyAbsent);
        assert!(replay.advances(), "already-absent completes the run");
    }

    /// A BUSY claim lock is RETURNED, never printed: the MCP caller passes no sink and
    /// could not observe a printed skip (the reason `reap_fork` takes no `out`).
    #[test]
    fn a_busy_claim_lock_is_a_returned_outcome_not_a_printed_skip() {
        use super::{GcOutcome, reap_fork};
        use crate::worktree::claim_lock;

        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "0", "base");
        let coord = std::fs::canonicalize(repo.path()).expect("canonicalize");
        let name = "agent-busy";
        let branch = format!("dispatch/{name}");
        repo.git(&["branch", &branch, &base]);

        let held = claim_lock::acquire(&coord, name).expect("the claimant takes the name");
        let outcome = reap_fork(Some(coord.clone()), &branch, true).expect("busy is not an error");
        assert_eq!(outcome, GcOutcome::Busy);
        assert!(
            !outcome.advances(),
            "a skipped name never advances position"
        );
        assert!(
            repo.branch_exists(&branch),
            "the live claimant's branch is NOT swept, funnel proof or not"
        );
        drop(held);
        assert_eq!(
            reap_fork(Some(coord), &branch, true).expect("reap"),
            GcOutcome::Reaped,
            "with the claim released the reap proceeds"
        );
    }

    /// The `--dry-run` basis names its AUTHORITY, never a blanket `landed ✓` (F-5),
    /// including the new funnel arm.
    #[test]
    fn the_dry_run_basis_names_the_authority_that_authorised_the_reap() {
        use super::dry_run_basis;

        assert_eq!(
            dry_run_basis(state(false, false, None, false), false),
            "already-certified (branch gone)"
        );
        assert_eq!(
            dry_run_basis(state(true, true, None, true), false),
            "landed ✓ (funnel record)"
        );
        assert_eq!(
            dry_run_basis(state(true, true, Some(true), false), false),
            "landed ✓ (oracle)"
        );
        assert!(
            dry_run_basis(state(true, true, Some(false), false), true).contains("--force"),
            "an override reap says so, and says it is NOT landed"
        );
    }

    /// The residue is a SET, and CRITICAL residue outranks ADMINISTRATIVE.
    #[test]
    fn residue_classification_puts_critical_above_administrative() {
        use super::{GcOutcome, Residue};

        let admin = Residue {
            worktree: false,
            branch: false,
            dispatch_record: Some("permission denied".to_owned()),
        };
        assert!(
            GcOutcome::Residual(admin.clone()).advances(),
            "the fork IS gone; only a stale record survives"
        );
        let critical = Residue {
            worktree: true,
            branch: true,
            dispatch_record: Some("permission denied".to_owned()),
        };
        assert!(!GcOutcome::Residual(critical.clone()).advances());
        // Every member is named — never a first-failure.
        let members = critical.members("dispatch/x", Some(Path::new("/w/x")));
        assert_eq!(
            members,
            vec![
                "worktree /w/x".to_owned(),
                "branch dispatch/x".to_owned(),
                "dispatch record: permission denied".to_owned(),
            ]
        );
        assert!(Residue::default().is_empty());
        assert!(!admin.is_empty());
    }

    // --- SL-199 F3: the reap report is redirectable off stdout -------------------
    // `run_gc_to` writes the human report to the injected sink (never the process
    // stdout), so an MCP tool driving over the stdio JSON-RPC channel keeps fd 1 clean.
    #[test]
    fn run_gc_to_writes_the_report_to_the_injected_sink() {
        use super::run_gc_to;

        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "0", "base");
        let coord = std::fs::canonicalize(repo.path()).unwrap();
        let name = "agent-feed";
        let branch = format!("dispatch/{name}");
        let dir = coord.join(".worktrees").join(name);
        repo.git(&[
            "worktree",
            "add",
            "-b",
            &branch,
            &dir.to_string_lossy(),
            &base,
        ]);

        let mut sink: Vec<u8> = Vec::new();
        run_gc_to(Some(coord.clone()), &branch, None, true, false, &mut sink).expect("gc reap");

        let report = String::from_utf8(sink).expect("utf8 report");
        assert!(
            report.contains(&format!("gc {branch}: reaped")),
            "the reap report lands in the injected sink, not stdout: {report:?}"
        );
        assert!(
            !repo.path().join(".worktrees").join(name).exists(),
            "the fork worktree is actually reaped (side effects unchanged)"
        );
    }

    // --- SL-228 PHASE-04 (VT-4): the lock-gated branch-residue sweep ----------------
    //
    // A BUSY claim lock IS the active-claimant signal: a spawn is mid claim→bind→act,
    // so its branch exists and its worktree does not YET — byte-for-byte what crash
    // residue looks like. gc must SKIP that name rather than sweep a live claim (which
    // would let a second spawn re-claim the name and cross-pair the bindings).
    //
    // Exercised against a REAL flock via the real `acquire`, not a mock.
    #[test]
    fn a_busy_claim_lock_makes_the_sweep_skip_the_name_and_destroy_nothing() {
        use super::run_gc_to;
        use crate::worktree::claim_lock;
        use crate::worktree::dispatch_record::{RECORD_SUBPATH, provision_dispatch_record};

        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "0", "base");
        let coord = std::fs::canonicalize(repo.path()).unwrap();
        let name = "agent-claimed";
        let branch = format!("dispatch/{name}");
        let dir = coord.join(".worktrees").join(name);

        // Exactly the mid-claim shape: branch ⊕ record, no worktree.
        repo.git(&["branch", &branch, &base]);
        provision_dispatch_record(&coord, name, &base, &dir, &branch, None).unwrap();
        let record_file = coord.join(RECORD_SUBPATH).join(format!("{name}.toml"));

        // An ACTIVE claimant holds the name.
        let held = claim_lock::acquire(&coord, name).expect("the claimant takes the name");

        let mut sink: Vec<u8> = Vec::new();
        // `--force` would otherwise reap unconditionally — the claim gate outranks it,
        // because it runs BEFORE anything is classified.
        run_gc_to(Some(coord.clone()), &branch, None, true, false, &mut sink)
            .expect("a busy name is a skip, not an error");

        let report = String::from_utf8(sink).expect("utf8 report");
        assert!(
            report.contains("skipped") && report.contains("claim lock"),
            "the skip names its reason: {report:?}"
        );
        assert!(
            repo.git(&["rev-parse", "--verify", "--quiet", &branch]) != *"",
            "the live claimant's branch is NOT swept"
        );
        assert!(
            record_file.exists(),
            "the live claimant's record is NOT deleted"
        );

        // Once the claimant is done, the same sweep proceeds normally.
        drop(held);
        let mut sink2: Vec<u8> = Vec::new();
        run_gc_to(Some(coord.clone()), &branch, None, true, false, &mut sink2).expect("gc reap");
        assert!(
            !record_file.exists(),
            "with the claim released the residue sweeps"
        );
        assert!(
            String::from_utf8(sink2).unwrap().contains("reaped"),
            "and reports a real reap"
        );
    }
}
