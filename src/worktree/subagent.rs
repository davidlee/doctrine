#![expect(unused, reason = "extraction; PHASE-03 prunes")]
#![expect(
    clippy::fn_params_excessive_bools,
    reason = "classifier gathers multiple facts; refactoring out of scope for SL-116"
)]
// SPDX-License-Identifier: GPL-3.0-only
//! subagent machine — extracted from worktree/mod.rs (SL-116 PHASE-02).

use super::allowlist::{
    Allowlist, allowlist_violations, is_withheld, parse_allowlist, select_copies,
};
use super::jail::is_privileged_agent_type;
use super::marker::{DISPATCH_WORKER_AGENT_TYPE, marker_present, write_marker};
use super::provision::run_provision;
use super::shared::{
    gather_fork_worktree, gather_tree_clean, is_linked_worktree, matches, project_anchor,
    resolve_commit, resolve_common_dir, target_dir_for_branch,
};
use crate::fsutil::{self, CopyOutcome};
use crate::git;
use crate::root;
use anyhow::{Context, bail};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

/// Verdict of the PURE stamp classifier: the resolved inputs hold ⇒ the shell may
/// provision + mark the already-created worktree. Mirror of [`Apply`]/[`Merge`] —
/// the pure core decides, the shell ([`run_stamp_subagent`]) acts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stamp {
    /// All preconds hold ⇒ the shell runs `run_provision` then `write_marker`.
    Ok,
}

/// Why a `marker --stamp-subagent` refuses (SL-056 PHASE-10, design — the claude
/// spawn path's mark step). Two-valued classifier (Stamp vs Refuse): there is NO
/// `PlainCreate` / else-branch — the `SubagentStart` matcher scopes the hook to
/// dispatch workers, so a benign subagent never reaches this verb. Each variant
/// fails closed with a distinct named token (the property the goldens assert).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StampRefusal {
    /// The payload `cwd` is absent/empty (also the malformed-JSON fold target).
    MissingCwd,
    /// `cwd` is not under the repo, OR is not a linked worktree.
    BadDir,
    /// `agent_type` is absent, OR present but != [`DISPATCH_WORKER_AGENT_TYPE`].
    MissingAgentType,
    /// The payload worktree ALREADY bears the worker marker — a re-entrant stamp
    /// (design §5 Hook-mint: only the first, marker-absent stamp is exempt).
    /// Re-provisioning would overwrite live worker state on a resume.
    AlreadyMarked,
}

impl StampRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            StampRefusal::MissingCwd => "missing-cwd",
            StampRefusal::BadDir => "bad-dir",
            StampRefusal::MissingAgentType => "missing-agent-type",
            StampRefusal::AlreadyMarked => "already-marked",
        }
    }
}

/// PURE stamp classifier (no git / disk / env / clock — ADR-001 leaf, CLAUDE.md
/// pure/imperative split). Mirror of [`classify_import`]/[`classify_land`]/
/// [`classify_gc`]: it takes the gathered, already-resolved FACTS and returns the
/// verdict. The shell resolves cwd-presence and the under-repo + linked-worktree
/// probes (impure git/disk), then calls this.
///
/// * `agent_type` — the payload `agent_type` ("" if absent); must equal
///   [`DISPATCH_WORKER_AGENT_TYPE`].
/// * `cwd_present` — the payload carried a non-empty `cwd`.
/// * `cwd_is_under_repo_linked_worktree` — the resolved cwd is under the repo AND
///   a live linked worktree (both probes folded by the shell into one bool).
///
/// * `already_marked` — the resolved payload worktree already bears the worker
///   marker (a prior stamp). Only the FIRST, marker-absent stamp is exempt.
///
/// Precond order: cwd-presence → dir-validity → agent-type → already-marked.
/// (Agent-type before the marker so a wrong agent-type names itself first; the
/// marker check is LAST — it only matters once the dir is a valid worker worktree.)
pub(crate) fn classify_stamp(
    agent_type: &str,
    cwd_present: bool,
    cwd_is_under_repo_linked_worktree: bool,
    already_marked: bool,
) -> Result<Stamp, StampRefusal> {
    if !cwd_present {
        return Err(StampRefusal::MissingCwd);
    }
    if !cwd_is_under_repo_linked_worktree {
        return Err(StampRefusal::BadDir);
    }
    if agent_type != DISPATCH_WORKER_AGENT_TYPE {
        return Err(StampRefusal::MissingAgentType);
    }
    if already_marked {
        return Err(StampRefusal::AlreadyMarked);
    }
    Ok(Stamp::Ok)
}

/// The `SubagentStart` payload subset we read (tolerate extra fields). JSON on
/// stdin: `{ "cwd": "<worktree path>", "agent_type": "<e.g. dispatch-worker>" }`.
#[derive(Debug, Default, serde::Deserialize)]
struct SubagentPayload {
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    agent_type: Option<String>,
}

/// True iff `cwd` belongs to the SAME repo as `repo` — they resolve to the same
/// `git-common-dir`. This is the worktree notion of "under the repo": a linked
/// worktree lives in a SEPARATE directory (a sibling, not a path-prefix child of
/// the source), so a path-`starts_with` test would wrongly reject every real fork.
/// Shared-common-dir membership is exactly what [`verify_sibling_worktree`] (inside
/// [`run_provision`]) re-checks before copying. A git failure on either side ⇒ not
/// the same repo (fail-closed). Impure (the git reads).
fn cwd_shares_repo(repo: &Path, cwd: &Path) -> bool {
    let repo_common = git::git_text(repo, &["rev-parse", "--git-common-dir"])
        .ok()
        .and_then(|c| resolve_common_dir(repo, &c).ok());
    let cwd_common = git::git_text(cwd, &["rev-parse", "--git-common-dir"])
        .ok()
        .and_then(|c| resolve_common_dir(cwd, &c).ok());
    match (repo_common, cwd_common) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// `doctrine worktree marker --stamp-subagent` — the claude harness spawn path's
/// post-hoc mark step (SL-056 PHASE-10). Historically Claude created the worker's
/// worktree and this verb stamped it afterwards from the matcher-scoped
/// `SubagentStart` hook (discriminating by the payload `agent_type`). SL-152 REVIVES
/// `create-fork` — the `WorktreeCreate` hook that create+provision+marks the worktree
/// ATOMICALLY, discriminating POSITIONALLY by cwd (the thin payload carries no
/// `agent_type`/path); SL-152 PHASE-04 retires THIS verb's install wiring. While both
/// coexist, this verb still runs to **provision + stamp** the already-created worktree
/// named by the payload `cwd`.
///
/// `SubagentStart` is a READ-ONLY hook event — a non-zero exit does NOT abort the
/// subagent. So this verb only stamps-or-refuses and exits honestly; it cannot and
/// must not try to block an unstamped worker (fenced elsewhere: the import belt,
/// the worker-mode guard, and the orchestrator's post-spawn check).
///
/// Shell flow (gather → pure-classify → act):
/// 1. read stdin → parse JSON (malformed ⇒ empty payload ⇒ `missing-cwd`);
/// 2. resolve cwd: a [`is_linked_worktree`] of the SAME repo as the source (shared
///    git-common-dir — [`cwd_shares_repo`], the worktree notion of "under the repo");
/// 3. [`classify_stamp`]; on Refuse print the token to stderr + exit non-zero;
/// 4. on Stamp: [`run_provision`] (the SOLE copier, source = the orchestrator tree,
///    destination = the worker worktree `cwd`) THEN [`write_marker`].
///
/// M3 failure posture: if provision/mark fails, print a LOUD stderr diagnostic and
/// exit non-zero — and do NOT `git worktree remove` (we added no worktree; Claude
/// owns it, the worker is already cleared to run). There is NO compensating
/// rollback here (unlike `create-fork`, which owns its creation and DOES compensate);
/// the half-stamped fork is left for the orchestrator's post-spawn check.
///
/// NOTE: the `SubagentStart`-matcher wiring and the `/dispatch-agent` skill leg are
/// LATER phases (out of scope here).
pub(crate) fn run_stamp_subagent(path: Option<PathBuf>) -> anyhow::Result<()> {
    let mut raw = String::new();
    io::Read::read_to_string(&mut io::stdin(), &mut raw).context("read SubagentStart payload")?;
    // Malformed JSON folds to an empty payload ⇒ classified as `missing-cwd`
    // (fail-closed on the stamp decision; we never block the worker either way).
    let payload: SubagentPayload = serde_json::from_str(&raw).unwrap_or_default();

    let agent_type = payload.agent_type.unwrap_or_default();
    let cwd_str = payload.cwd.unwrap_or_default();
    let cwd_present = !cwd_str.is_empty();

    // R1 binding ANCHOR ONLY: `root::find` on the PROCESS cwd resolves a doctrine
    // root used to VALIDATE the payload cwd (via `cwd_shares_repo` / `is_linked_worktree`
    // below) — it is NOT the provision SOURCE. The `SubagentStart` hook fires INSIDE the
    // worker worktree, so the process cwd is the FORK, not the orchestrator tree; using it
    // as the copy source would make source == fork and trip the sibling-worktree guard
    // (ISS-011 Defect C). The R2 provision SOURCE is derived separately as the repo's
    // primary worktree. `None` ⇒ no doctrine root above the process cwd ⇒ the cwd cannot
    // be validated against a repo ⇒ bad-dir.
    let repo = root::find(path, &root::default_markers())
        .ok()
        .and_then(|r| fs::canonicalize(&r).ok());
    let cwd_canon = if cwd_present {
        fs::canonicalize(&cwd_str).ok()
    } else {
        None
    };
    // Valid iff the payload cwd is a linked worktree of the SAME repo as the source
    // (shared git-common-dir) — the worktree notion of "under the repo". A path
    // prefix-check is WRONG: a linked worktree is a sibling dir, not a child.
    let cwd_valid = match (repo.as_deref(), cwd_canon.as_deref()) {
        (Some(repo), Some(cwd)) => {
            is_linked_worktree(cwd).unwrap_or(false) && cwd_shares_repo(repo, cwd)
        }
        _ => false,
    };
    // Re-entrant guard: a payload worktree already bearing the marker must NOT be
    // re-provisioned (it would overwrite live worker state on a resume) — only the
    // first, marker-absent stamp is exempt (design §5 Hook-mint, F-9).
    let already_marked = cwd_canon.as_deref().is_some_and(marker_present);

    match classify_stamp(&agent_type, cwd_present, cwd_valid, already_marked) {
        Ok(Stamp::Ok) => {}
        Err(refusal) => {
            writeln!(io::stderr(), "stamp-refused: {}", refusal.token())?;
            bail!("stamp-refused: {}", refusal.token());
        }
    }
    // Stamp passed ⇒ both the source repo and the canonical cwd resolved (cwd_valid
    // required Some of each). Source = the orchestrator tree (provision copies FROM
    // it); the worker worktree `cwd` is the destination. Fail closed if either is
    // somehow absent — never panic on a hook input.
    let (Some(_anchor), Some(cwd)) = (repo, cwd_canon) else {
        let token = StampRefusal::BadDir.token();
        writeln!(io::stderr(), "stamp-refused: {token}")?;
        bail!("stamp-refused: {token}");
    };

    // --- act: provision (SOLE copier) THEN mark. M3: NO rollback on failure. ---
    // R2: provision SOURCE is the PRIMARY worktree, NOT the binding anchor — which is
    // the fork itself when the hook fires inside the worker worktree (ISS-011 Defect C).
    let act = git::primary_worktree(&cwd)
        .and_then(|source| run_provision(Some(source), &cwd))
        .and_then(|()| write_marker(&cwd));
    if let Err(cause) = act {
        // LOUD diagnostic; the worktree is LEFT in place (Claude owns it). No
        // `git worktree remove` — there is no compensating rollback for a stamp.
        writeln!(
            io::stderr(),
            "STAMP FAILED for {} — worktree LEFT in place (not removed); orchestrator post-spawn check will catch the unstamped worker: {cause:#}",
            cwd.display()
        )?;
        return Err(cause.context(format!("stamp worker worktree {}", cwd.display())));
    }

    writeln!(io::stderr(), "stamped worker worktree {}", cwd.display())?;
    Ok(())
}

/// Verdict of the PURE worker-verify classifier: the spawned claude worker is a
/// stamped fork whose HEAD descends from the orchestrator's base `B`. The shell
/// ([`run_verify_worker`]) gathers the FACTS and acts on this verdict (ADR-001
/// leaf, gather → pure-classify → act).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerVerify {
    /// HEAD resolves, the worker marker is present, and `B` is an ancestor of the
    /// worker HEAD ⇒ base==B by placement holds (design §8.4).
    Ok,
}

/// Why a post-spawn `verify-worker` refuses (design §8.4 / DD-12). Each variant
/// fails closed with a distinct named token (the property the goldens assert, not
/// a proxy). The verb is fail-LOUD and diagnostic only: it never removes the fork.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerVerifyRefusal {
    /// The worker worktree HEAD does not resolve (`git -C <dir> rev-parse HEAD`
    /// failed) — no fork to verify. The explicit unresolved-HEAD verdict.
    NoWorkerHead,
    /// The worktree is NOT a linked worktree (git-dir == git-common-dir ⇒ primary
    /// tree) — the fork exists but it is the main tree, not an isolated worker.
    NotIsolated,
    /// HEAD resolves but the worker marker is absent — the `SubagentStart` stamp
    /// never landed (a non-fail-closable hook), so this is not a trusted worker.
    Unstamped,
    /// Stamped fork, but `B` is NOT an ancestor of the worker HEAD — the worker
    /// forked off the wrong base (`baseRef` misconfigured or placement wrong).
    WrongBase,
    /// The worker HEAD does NOT equal the branch tip `S` the funnel imports as —
    /// the footer dir and branch are incoherent (a `--branch` coherence belt).
    BranchMismatch,
}

impl WorkerVerifyRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            WorkerVerifyRefusal::NoWorkerHead => "no-worker-head",
            WorkerVerifyRefusal::NotIsolated => "not-isolated",
            WorkerVerifyRefusal::Unstamped => "unstamped",
            WorkerVerifyRefusal::WrongBase => "wrong-base",
            WorkerVerifyRefusal::BranchMismatch => "branch-mismatch",
        }
    }
}

pub(crate) fn classify_worker_verify(
    head_resolved: bool,
    is_isolated: bool,
    marker_present: bool,
    base_is_ancestor: bool,
    head_is_branch_tip: bool,
) -> Result<WorkerVerify, WorkerVerifyRefusal> {
    if !head_resolved {
        return Err(WorkerVerifyRefusal::NoWorkerHead);
    }
    if !is_isolated {
        return Err(WorkerVerifyRefusal::NotIsolated);
    }
    if !marker_present {
        return Err(WorkerVerifyRefusal::Unstamped);
    }
    if !base_is_ancestor {
        return Err(WorkerVerifyRefusal::WrongBase);
    }
    if !head_is_branch_tip {
        return Err(WorkerVerifyRefusal::BranchMismatch);
    }
    Ok(WorkerVerify::Ok)
}

/// `doctrine worktree verify-worker --base <B> --dir <worktree> [--branch <S>]` —
/// the claude `/dispatch` arm's post-spawn base==B check (design §8.4 / DD-12).
/// After a claude worker returns, the orchestrator runs this against the worker
/// worktree to PROVE its HEAD descends from the base `B` it was meant to fork off
/// (option Y: base is orchestrator-controlled by placement, verified here rather
/// than ref-redirected). Fail-LOUD and diagnostic only — it NEVER removes the fork;
/// the orchestrator decides what to do with a refused worker.
///
/// `--dir` fully locates the worker worktree (it is the git `-C` root for every
/// probe), so no `-p` root override is needed — unlike the funnel verbs that run
/// at the coordination root, this verb's operand IS the subject worktree.
///
/// When `--branch <S>` is given, an additional coherence belt fires: the worktree
/// HEAD must equal the tip of `S` (dir↔branch coherence).
///
/// Refusal tokens: `no-worker-head` / `not-isolated` / `unstamped` /
/// `wrong-base` / `branch-mismatch`.
///
/// Read-classed (no writes; mirrors `branch-point-check`/`status`) — harmless
/// under worker-mode, and design §8.6 lists no impersonation test for it.
///
/// Gather → pure-classify → act:
/// 1. gather the FACTS — worker HEAD resolves (`rev-parse --verify HEAD`, run -C
///    the worker dir); the worktree is isolated (git-dir ≠ git-common-dir); the
///    worker marker is present at `<dir>`; `B` is an ancestor of the worker HEAD
///    (`merge-base --is-ancestor <B> HEAD`, the SHARED [`git::git_status_ok`]
///    is-ancestor primitive — NO new git.rs plumbing); when `--branch <S>`, the
///    worker HEAD equals the tip of `S` (dir↔branch coherence belt);
/// 2. [`classify_worker_verify`] returns the verdict;
/// 3. on Refuse print the distinct token to stderr + exit non-zero, fork PRESERVED;
///    on Ok exit 0.
pub(crate) fn run_verify_worker(
    base: &str,
    dir: &Path,
    branch: Option<&str>,
) -> anyhow::Result<()> {
    // --- gather (all impure git/disk reads, fail-closed) ---
    // Worker HEAD must resolve in the worker WORKTREE (-C <dir>), not the
    // orchestrator root — a non-resolving HEAD ⇒ `no-worker-head`.
    let head_resolved = git::git_opt(dir, &["rev-parse", "--verify", "HEAD"])?.is_some();
    // Isolation check: git-dir must differ from git-common-dir (linked worktree).
    // Missing dir ⇒ NoWorkerHead wins (head_resolved gates is_isolated).
    let is_isolated = head_resolved && is_linked_worktree(dir)?;
    let marker = marker_present(dir);
    // is-ancestor signals purely via exit code; unresolvable refs ⇒ Ok(false)
    // (fail-closed, never a panic). `git_status_ok` errors only on a spawn failure.
    let base_is_ancestor = git::git_status_ok(dir, &["merge-base", "--is-ancestor", base, "HEAD"])?;
    // Coherence: worktree HEAD must equal the branch tip the funnel imports as S.
    let head_is_branch_tip = match branch {
        Some(s) => {
            let head = git::git_opt(dir, &["rev-parse", "--verify", "HEAD"])?;
            let tip = git::git_opt(dir, &["rev-parse", "--verify", &format!("{s}^{{commit}}")])?;
            matches!((head, tip), (Some(h), Some(t)) if h == t)
        }
        None => true,
    };

    // --- pure classify ---
    match classify_worker_verify(
        head_resolved,
        is_isolated,
        marker,
        base_is_ancestor,
        head_is_branch_tip,
    ) {
        Ok(WorkerVerify::Ok) => {
            writeln!(
                io::stderr(),
                "verify-worker: base==B holds for {}",
                dir.display()
            )?;
            Ok(())
        }
        Err(refusal) => {
            // Fail-loud; the fork is LEFT in place (the orchestrator owns the
            // disposition of a refused worker — this verb never removes a worktree).
            writeln!(
                io::stderr(),
                "verify-worker-refused: {} ({})",
                refusal.token(),
                dir.display()
            )?;
            bail!("verify-worker-refused: {}", refusal.token());
        }
    }
}

// ---------------------------------------------------------------------------
// Nomination + spawn-gate hooks (SL-206 PHASE-11, design §5.6) — the
// `SubagentStart(dispatch-orchestrator)` nominate handler and the
// `SubagentStop` denominate hygiene handler. Closed loop with `pretooluse`'s
// `Agent`/`Workflow` gate legs: nominate WRITES the allowlist entry (I2 — the
// FIXED, out-of-jail path is writable only by a process that resolves
// `CLAUDE_PROJECT_DIR`; jailed callers cannot spawn at all, so a worker cannot
// nominate anything); denominate REMOVES it on `SubagentStop` (hygiene, so a
// stale `agent_id` cannot be reused); [`is_nominated`] is the READ side
// `pretooluse`'s gate + jail PassThrough leg both consume.
// ---------------------------------------------------------------------------

/// The allowlist's FIXED, out-of-jail location relative to `project_anchor()`
/// (design §5.6 I2 — resolved from `CLAUDE_PROJECT_DIR`, NEVER cwd-relative;
/// a `SubagentStart` hook's cwd is the SPAWNED tree, not the project root,
/// `mem_019ee3a0`). Presence-plus-membership, one `agent_id` per line — mirrors
/// `marker.rs`'s `.doctrine/state/dispatch/worker` shape (a runtime artifact,
/// not an authored entity).
const NOMINATION_ALLOWLIST_REL: &str = ".doctrine/state/orch-allowlist.txt";

fn allowlist_path(project_dir: &Path) -> PathBuf {
    project_dir.join(NOMINATION_ALLOWLIST_REL)
}

/// Read the allowlist into trimmed, non-empty lines. Fail-safe (design §5.6):
/// an absent or unreadable file resolves to EMPTY — nobody nominated, never a
/// read error that could somehow imply membership.
fn read_allowlist(project_dir: &Path) -> Vec<String> {
    fs::read_to_string(allowlist_path(project_dir))
        .ok()
        .map(|body| {
            body.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Write the allowlist body (one `agent_id` per line), creating the parent dir
/// as needed. The SOLE writer-shell for both [`act_nominate`] and
/// [`act_denominate`] — never two write paths that could diverge on format.
fn write_allowlist(project_dir: &Path, lines: &[String]) -> anyhow::Result<()> {
    let path = allowlist_path(project_dir);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .with_context(|| format!("create nomination-allowlist dir {}", dir.display()))?;
    }
    let mut body = String::new();
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "runtime nomination-allowlist state"
    )]
    fs::write(&path, body)
        .with_context(|| format!("write nomination allowlist {}", path.display()))?;
    Ok(())
}

/// `agent_id ∈ existing` — the ONE membership test both the gate (T4) and the
/// jail `PassThrough` leg (T5) rely on via [`is_nominated`]. PURE.
fn is_nominated_among(existing: &[String], agent_id: &str) -> bool {
    existing.iter().any(|line| line == agent_id)
}

/// Idempotent append (T2): `agent_id` appended to `existing` iff not already
/// present (exact-line match) — a repeated nomination (retry, resume) never
/// duplicates the entry. PURE.
pub(crate) fn append_nomination(existing: &[String], agent_id: &str) -> Vec<String> {
    if is_nominated_among(existing, agent_id) {
        existing.to_vec()
    } else {
        let mut out = existing.to_vec();
        out.push(agent_id.to_string());
        out
    }
}

/// Remove every line exactly equal to `agent_id` (T3, denominate hygiene).
/// Absent id ⇒ `existing` returned content-unchanged. PURE.
pub(crate) fn remove_nomination(existing: &[String], agent_id: &str) -> Vec<String> {
    existing
        .iter()
        .filter(|line| line.as_str() != agent_id)
        .cloned()
        .collect()
}

/// `agent_id` is currently nominated at `project_dir` (design §5.6). The
/// `pretooluse` Bash/Edit/Write `PassThrough` leg (T5) and the `Agent` spawn-gate
/// (T4) both call this with the SAME `project_dir` they resolved
/// (`project_anchor()`), so the membership read is single-sourced. Fail-safe:
/// nomination NEVER defaults to granted — an absent/unreadable allowlist file
/// folds to [`read_allowlist`]'s empty default.
pub(crate) fn is_nominated(project_dir: &Path, agent_id: &str) -> bool {
    is_nominated_among(&read_allowlist(project_dir), agent_id)
}

/// Why `nominate` refuses (design §5.6). `SubagentStart` cannot fail-closed —
/// exit 2 does not abort the spawn — so a refusal here is simply "don't write",
/// never a block. Distinct named tokens (STD-001; the security-boundary
/// property tests assert on VARIANTS, not prose).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NominateRefusal {
    /// `CLAUDE_PROJECT_DIR` is unset/uncanonicalizable — the FIXED allowlist
    /// path is unresolvable. An environment/setup error, not a benign refusal.
    MissingProjectDir,
    /// `agent_type` is absent or not in [`super::jail::PRIVILEGED_AGENT_TYPES`]
    /// — a benign (non-orchestrator) spawn. `SubagentStart` fires for EVERY
    /// subagent type, so this is the expected common case, not a failure.
    Ineligible,
    /// The payload carried no (or an empty) `agent_id` — nothing to write.
    MissingAgentId,
}

impl NominateRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            NominateRefusal::MissingProjectDir => "missing-project-dir",
            NominateRefusal::Ineligible => "ineligible",
            NominateRefusal::MissingAgentId => "missing-agent-id",
        }
    }
}

/// PURE nominate classifier (T2). Precondition order: project-dir resolvability
/// (an environment/setup problem, checked first — without it nothing else
/// matters) → eligibility (the ordinary benign-spawn case, named next so a
/// benign spawn's refusal reads as "ineligible", not a missing-id red herring)
/// → `agent_id` presence (a malformed/partial payload, checked last).
pub(crate) fn classify_nominate(
    agent_type: &str,
    project_dir_present: bool,
    agent_id_present: bool,
) -> Result<(), NominateRefusal> {
    if !project_dir_present {
        return Err(NominateRefusal::MissingProjectDir);
    }
    if !is_privileged_agent_type(agent_type) {
        return Err(NominateRefusal::Ineligible);
    }
    if !agent_id_present {
        return Err(NominateRefusal::MissingAgentId);
    }
    Ok(())
}

/// The `SubagentStart` nominate payload subset read (tolerates extra fields,
/// mirroring [`SubagentPayload`]). JSON on stdin: `{ "agent_id": "...",
/// "agent_type": "dispatch-orchestrator" }`.
#[derive(Debug, Default, serde::Deserialize)]
struct NominatePayload {
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    agent_type: Option<String>,
}

/// The already-resolved-inputs nominate act (gather → classify → act):
/// testable with an injected `project_dir` — no env mutation in tests, since
/// `CLAUDE_PROJECT_DIR` is a process-global env var multiple test threads would
/// race on (design §5.6 I2's real resolution stays in [`run_nominate`] alone).
/// `pub(crate)` for that testability (mod.rs's test module, T2's golden suite).
pub(crate) fn act_nominate(
    project_dir: Option<&Path>,
    agent_id: Option<&str>,
    agent_type: &str,
) -> anyhow::Result<()> {
    let agent_id = agent_id.filter(|s| !s.is_empty());
    if let Err(refusal) = classify_nominate(agent_type, project_dir.is_some(), agent_id.is_some()) {
        writeln!(io::stderr(), "nominate-refused: {}", refusal.token())?;
        bail!("nominate-refused: {}", refusal.token());
    }
    // classify_nominate's Ok arm guarantees both are Some — the precondition
    // order above checked exactly these two facts. Never a panic on the
    // structurally-unreachable else (`unreachable!`/`unwrap` are denied,
    // Cargo.toml): a bail is the fail-loud, non-panicking equivalent.
    let (Some(project_dir), Some(agent_id)) = (project_dir, agent_id) else {
        bail!("nominate: internal error — classify_nominate(Ok) without both inputs present");
    };
    let existing = read_allowlist(project_dir);
    let updated = append_nomination(&existing, agent_id);
    write_allowlist(project_dir, &updated)?;
    writeln!(io::stderr(), "nominated {agent_id}")?;
    Ok(())
}

/// `doctrine worktree nominate` — the `SubagentStart(dispatch-orchestrator)`
/// hook handler (design §5.6). Reads `{agent_id, agent_type}` JSON on stdin; an
/// ELIGIBLE spawn (`agent_type ∈ PRIVILEGED_AGENT_TYPES`) appends `agent_id` to
/// the FIXED allowlist at `$CLAUDE_PROJECT_DIR`-resolved
/// `.doctrine/state/orch-allowlist.txt` (I2 — out of every worker jail).
///
/// `SubagentStart` is READ-ONLY (a non-zero exit does NOT abort the spawn — the
/// same non-fail-closable contract as [`run_stamp_subagent`]), so a refusal
/// here is simply "don't write"; the fail-safe direction is a MISSED
/// nomination leaves the orchestrator jailed (a visible functional failure),
/// NEVER an escape (design §5.6 "fail-safe" — absence of an allowlist entry
/// never grants `PassThrough`).
pub(crate) fn run_nominate() -> anyhow::Result<()> {
    let mut raw = String::new();
    io::Read::read_to_string(&mut io::stdin(), &mut raw)
        .context("read SubagentStart nominate payload")?;
    // Malformed JSON folds to an empty payload ⇒ classified `missing-agent-id`
    // (an empty `agent_type` also refuses `ineligible`) — never a panic.
    let payload: NominatePayload = serde_json::from_str(&raw).unwrap_or_default();
    let agent_type = payload.agent_type.unwrap_or_default();
    act_nominate(
        project_anchor().as_deref(),
        payload.agent_id.as_deref(),
        &agent_type,
    )
}

/// The `SubagentStop` denominate payload subset read. JSON on stdin:
/// `{ "agent_id": "..." }` (tolerates extra fields, same subset philosophy as
/// [`SubagentPayload`]/[`NominatePayload`]).
#[derive(Debug, Default, serde::Deserialize)]
struct DenominatePayload {
    #[serde(default)]
    agent_id: Option<String>,
}

/// The already-resolved-inputs denominate act (T3) — see [`act_nominate`] on
/// why `project_dir` is injected rather than env-resolved here. A missing
/// `agent_id`, an unresolvable `project_dir`, or an entry not currently present
/// is a CLEAN no-op: NO write at all (never even touches the file), so a
/// benign `SubagentStop` for an un-nominated agent leaves nothing behind.
/// `pub(crate)` for that testability (mod.rs's test module, T3's golden suite).
pub(crate) fn act_denominate(
    project_dir: Option<&Path>,
    agent_id: Option<&str>,
) -> anyhow::Result<()> {
    let Some(agent_id) = agent_id.filter(|s| !s.is_empty()) else {
        return Ok(()); // no agent_id ⇒ nothing to remove.
    };
    let Some(project_dir) = project_dir else {
        return Ok(()); // no resolvable anchor ⇒ nothing to remove.
    };
    let existing = read_allowlist(project_dir);
    if !is_nominated_among(&existing, agent_id) {
        return Ok(()); // absent entry ⇒ clean no-op, no write.
    }
    let updated = remove_nomination(&existing, agent_id);
    write_allowlist(project_dir, &updated)?;
    writeln!(io::stderr(), "denominated {agent_id}")?;
    Ok(())
}

/// `doctrine worktree denominate` — the `SubagentStop` hygiene hook handler
/// (design §5.6). Reads `{agent_id}` JSON on stdin; removes exactly that entry
/// from the allowlist so a stale `agent_id` cannot be reused once the
/// orchestrator's session ends. Absent file / absent entry / absent env is a
/// CLEAN no-op, exit 0 — never an error: this is best-effort hygiene, not the
/// security-load-bearing gate itself (the gate is the allowlist READ,
/// [`is_nominated`], which is fail-safe regardless of whether hygiene ran).
pub(crate) fn run_denominate() -> anyhow::Result<()> {
    let mut raw = String::new();
    io::Read::read_to_string(&mut io::stdin(), &mut raw)
        .context("read SubagentStop denominate payload")?;
    let payload: DenominatePayload = serde_json::from_str(&raw).unwrap_or_default();
    act_denominate(project_anchor().as_deref(), payload.agent_id.as_deref())
}
