// SPDX-License-Identifier: GPL-3.0-only
//! `worker_commit` — the gated, server-side self-commit for a jailed dispatch worker
//! (SL-198 PHASE-02, IMP-253; design §5.2/§5.4/§5.5).
//!
//! A dispatch worker runs in a linked worktree with the shared `.git` **read-only**
//! (SL-182 jail), so its raw `git commit` is walled. This MCP tool lets it self-commit
//! ONE gated commit on its own `dispatch/<agent>` branch entirely server-side: the
//! worker passes only an OPAQUE `agent` id (its worktree name) — never a path — and the
//! UNCONFINED MCP server resolves the target, runs the belts, and commits. This is the
//! deliberate bypass of the jail wall (RSK-225), so the belts ARE the security boundary
//! (INV-2: every write path is belt-gated server-side; a worker cannot skip a belt).
//!
//! **Belt order — cheap-first (design §10 X4, authoritative over §5.2's numbering):**
//! resolve → non-empty PRE-fmt delta → scope belt (two-tier) → HEAD==B → the mutating
//! `check commit` gate → stage the pre-fmt classified paths by name (post-fmt CONTENT,
//! PIN-3) → exactly one non-merge commit `C`, `C^==B`. Scoping the PRE-fmt delta before
//! the gate is why cheap-first is mandatory: the gate's `fmt` can widen the touched-path
//! set, so the belt must classify the intended delta first, and the commit then stages
//! exactly that classified set (INV-5 / F2 guard).
//!
//! **DRY (design §8 R3):** the scope hard tier reuses the SAME `.doctrine/`/`.claude/`
//! prefix consts as `classify_import` and the SAME `undeclared_paths` soft predicate, so
//! the two belt callers cannot diverge (VT-3). The resolver, gate, and delta-gather are
//! all reused PHASE-01 / existing seams — no forked copies.

use crate::dispatch::{Provenance, dispatch_identity, dispatch_ref, funnel};
use crate::dispatch_config::ForbiddenWrites;
use crate::funnel_machine::{
    self, IllegalTransition, PhaseRow, Position, Transition, TransitionKind,
};
use crate::git;
use crate::slice::SelectorIntent;
use crate::verify::{CheckKind, CheckPlan, VerificationConfig, resolve_check};
use crate::worktree::{
    CLAUDE_PREFIX, DOCTRINE_PREFIX, DispatchRecord, ForkBinding, ForkExpect, ResolveRefusal,
    gather_worktree_delta_paths, require_binding, resolve_agent,
};
use anyhow::{Context, bail};
use serde::Serialize;
use std::io::Write as _;
use std::path::Path;

/// Refuse `worker_commit` with no in-scope change — never mint an empty commit (§5.5).
const EMPTY_DELTA: &str = "empty-delta";
/// Refuse: a delta path hit the HARD scope tier (`.doctrine/`/`.claude/` code floor OR
/// the `[dispatch].worker-forbidden-writes` config matcher). The offending path rides in
/// `detail` (§5.2 step 3, EX-6).
const FORBIDDEN_ZONE: &str = "forbidden-zone";
/// Refuse: the worktree `HEAD != B` (resumed / stacked) — no fast-forward past B (§5.5
/// INV-1). Normally subsumed by the resolver's `stale-record` consistency check; kept as
/// a distinct belt-and-suspenders assertion at the commit boundary (§5.2 step 4).
const NOT_AT_BASE: &str = "not-at-base";
/// Refuse: the `check commit` gate (fmt + lint/test/build) exited red; the captured
/// output rides in `detail` (§5.2 step 2).
const COMMIT_GATE_RED: &str = "commit-gate-red";
/// Refuse: the fork has advanced past `B` and the worktree is DIRTY (SL-228 design §3).
/// That is not the retry signature — it is a LATE RE-COMMIT (a second commit attempt on
/// an already-committed fork), and the one-commit invariant `C^ == B` forbids it. The
/// worker's lost-response ambiguity always ends in a truthful terminal answer, so this
/// names itself rather than hiding behind `stale-record`.
const LATE_RECOMMIT: &str = "late-recommit";

/// The tool result: a landed commit, or a typed refusal (design §5.2). Serialised
/// externally-tagged (`{"Committed": {…}}` / `{"Refused": {…}}`), matching the other MCP
/// verbs' `{"Verb": {…}}` shape. A refusal is a normal `Ok` result carrying its token —
/// NOT a JSON-RPC error — so the worker/orchestrator reads the reason structurally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum WorkerCommitOutput {
    /// One non-merge commit `C` landed on `dispatch/<agent>` with `C^ == B`. `undeclared`
    /// carries the SOFT-tier src paths outside the design-target selectors (non-blocking —
    /// the orchestrator amends the selectors or rejects at import; §5.2 step 3).
    Committed {
        oid: String,
        base: String,
        undeclared: Vec<String>,
    },
    /// A belt refused; `reason` is the distinct token, `detail` the offending path /
    /// captured output (empty when the token is self-describing).
    Refused { reason: String, detail: String },
}

fn refused(reason: &str, detail: String) -> WorkerCommitOutput {
    WorkerCommitOutput::Refused {
        reason: reason.to_owned(),
        detail,
    }
}

/// The HARD scope tier: a path is a forbidden zone iff it hits the `.doctrine/` OR
/// `.claude/` code floor (the SAME consts `classify_import` rejects on — VT-3 agreement)
/// OR the `[dispatch].worker-forbidden-writes` config matcher (which additionally
/// re-applies the `.doctrine/` floor fail-closed, PIN-2). PURE.
fn is_forbidden_zone(path: &str, forbidden: &ForbiddenWrites) -> bool {
    path.starts_with(DOCTRINE_PREFIX)
        || path.starts_with(CLAUDE_PREFIX)
        || forbidden.is_forbidden(path)
}

/// PURE two-tier scope classification over the PRE-fmt delta (design §5.2 step 3, EX-6).
/// HARD tier: any forbidden-zone path fails closed with `Err(path)` → `forbidden-zone`.
/// SOFT tier: `undeclared_paths` (the SAME predicate `classify_import` uses) returns the
/// src paths outside the `design-target` selectors — these DO NOT block; they ride back
/// as `undeclared` for the orchestrator to amend/reject downstream (bounded by import
/// staying strict, PIN-4). Takes the gathered facts (delta, matcher, selectors) — no git
/// / disk here.
fn classify_scope(
    delta_paths: &[String],
    forbidden: &ForbiddenWrites,
    selectors: &[String],
) -> Result<Vec<String>, String> {
    if let Some(hit) = delta_paths.iter().find(|p| is_forbidden_zone(p, forbidden)) {
        return Err(hit.clone());
    }
    let paths: Vec<&str> = delta_paths.iter().map(String::as_str).collect();
    Ok(crate::conformance::undeclared_paths(selectors, &paths))
}

/// The commit-invariant base gate (§5.2 step 4, INV-1): the worktree `HEAD` must equal the
/// snapshotted base `B` pre-commit, so the one commit `C` satisfies `C^ == B`. PURE.
fn head_at_base(head: &str, base: &str) -> bool {
    head == base
}

/// The `check commit` gate outcome — green, or red with the captured child output.
enum GateOutcome {
    Green,
    Red(String),
}

/// Run the resolved `commit` tier (`resolve_check(Commit)`) in the worker worktree `dir`
/// (design §5.2 step 2). fmt mutates the tree FIRST, then lint/validate/test/build; a
/// red exit captures stdout+stderr for the refusal detail. The gate argv is resolved from
/// the tamper-proof PRIMARY config (`cfg`), never the worker-writable fork copy.
fn run_commit_gate(dir: &Path, cfg: &VerificationConfig) -> anyhow::Result<GateOutcome> {
    let argv = match resolve_check(cfg, CheckKind::Commit) {
        CheckPlan::Run(argv) => argv,
        CheckPlan::Empty(kind) => {
            bail!(
                "worker_commit: [verification].{} is empty — cannot run the commit gate",
                kind.key()
            )
        }
        CheckPlan::Noop(_) => {
            bail!("worker_commit: the commit gate resolved to a no-op — cannot gate the commit")
        }
    };
    let (program, rest) = argv
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("worker_commit: resolved commit-gate argv is empty"))?;
    // The gate runs the TRUSTED verification suite (`cargo test`) in the fork. The fork
    // carries the worker marker (`create-fork` stamps it, create.rs), so every e2e test
    // that spawns an authored-write `doctrine` fixture (`backlog new`, `install`, …)
    // resolves its root to the fork cwd and is REFUSED by the worker-mode guard's marker
    // leg — collateral damage, not the worker agent writing authored state (the agent is
    // blocked awaiting this MCP call, so no write can race the cleared window). Clear the
    // marker for the gate so the suite runs as on a normal tree, and do NOT export
    // DOCTRINE_WORKER (its env leg would re-trip the same guard). Restore the marker
    // afterwards so worker-agent protection survives past this call. See SL-199 F2.
    let had_marker = crate::worktree::marker_present(dir);
    if had_marker {
        crate::worktree::remove_marker(dir).context("clear worker marker for the commit gate")?;
    }
    let spawned = std::process::Command::new(program)
        .args(rest)
        .current_dir(dir)
        .env_remove("DOCTRINE_WORKER")
        // SL-225 #1: mark the gate window so `just validate` skips its governance self-checks
        // (which read coord's authored state, inert in a fork — ISS-218). This is the only
        // visible worker-context signal here: the marker is cleared and DOCTRINE_WORKER unset
        // above. Neutral context flag — no path/binary/cargo policy (POL-002, DEC-003).
        .env("DOCTRINE_DISPATCH_GATE", "1")
        .output()
        .with_context(|| format!("spawning the worker commit gate: {}", argv.join(" ")));
    if had_marker {
        crate::worktree::write_marker(dir)
            .context("restore worker marker after the commit gate")?;
    }
    let output = spawned?;
    if output.status.success() {
        Ok(GateOutcome::Green)
    } else {
        let mut detail = String::from_utf8_lossy(&output.stdout).into_owned();
        detail.push_str(&String::from_utf8_lossy(&output.stderr));
        Ok(GateOutcome::Red(detail))
    }
}

/// Stage the PRE-fmt classified `paths` by name, then create exactly ONE non-merge
/// commit (design §5.2 step 6, INV-5 / PIN-3). Staging + committing BY PATH (not the
/// post-fmt working-tree diff) means a repo-wide `fmt` that normalised a pre-existing /
/// out-of-scope file cannot ride into the commit — yet the staged CONTENT is the
/// post-fmt content of these paths (the gate already ran), never a pre-fmt blob snapshot.
fn stage_and_commit(dir: &Path, message: &str, paths: &[String]) -> anyhow::Result<()> {
    let mut add_args: Vec<&str> = vec!["add", "--"];
    add_args.extend(paths.iter().map(String::as_str));
    git::git_text(dir, &add_args).context("stage the classified worker delta")?;

    let mut commit_args: Vec<&str> = vec!["commit", "-q", "-m", message, "--"];
    commit_args.extend(paths.iter().map(String::as_str));
    git::git_text(dir, &commit_args).context("create the gated worker commit")?;
    Ok(())
}

/// Resolve the slice's `design-target` selectors for the SOFT tier (design §5.2 step 3).
/// The slice comes from the DURABLE FORK BINDING (SL-228 PHASE-04) — the very record the
/// verb already read for its base guard — not from re-parsing the coord's branch: one
/// read, always available, and no second fork→slice derivation to drift. Best-effort by
/// design: the SOFT tier never blocks, so a resolution failure degrades to no selectors
/// (every src path reported `undeclared`) rather than refusing a legitimate commit.
fn design_target_selectors(coord: &Path, slice: u32) -> anyhow::Result<Vec<String>> {
    crate::slice::selectors(coord, slice, Some(SelectorIntent::DesignTarget))
}

// --------------------------------------------------------------------------------------
// The funnel half (SL-228 PHASE-04, design §3/§4): the pre-act gate and the Class-2
// `RecordWorkerCommit` record this verb is the ORIGINAL RECORDER for (D8).
// --------------------------------------------------------------------------------------

/// The coordination-side funnel context for one `worker_commit` call: which row this
/// fork's commits belong to (from the durable binding), and where that row stands.
///
/// Gathered ONCE per call from the coord ref's committed record — an object-db read; the
/// coord index/worktree are never touched. Position is re-derived from a freshly resolved
/// tip on each landing attempt, so a lost-ref race re-reads rather than reusing this.
struct FunnelCtx {
    /// The coordination root (a git dir the funnel writer runs `-C` in).
    coord: std::path::PathBuf,
    /// `refs/heads/dispatch/<NNN>` — the CAS target.
    coord_ref: String,
    slice: u32,
    phase: String,
    /// The fork branch and the base it forked from — the `Spawn` provenance this
    /// verb heals when `create-fork`'s non-fatal row never landed (SL-228 PHASE-05).
    fork: String,
    base_oid: String,
    /// The coord tip this context's `position`/`row` were read at.
    tip: String,
    row: Option<PhaseRow>,
}

impl FunnelCtx {
    /// Open the context for a bound fork. Fails (rather than refusing) on a plumbing
    /// problem — an unresolvable coord ref is a broken coordination tree, not a
    /// worker-visible refusal.
    fn open(record: &DispatchRecord, binding: &ForkBinding) -> anyhow::Result<Self> {
        let coord_ref = dispatch_ref(binding.slice);
        let tip = commit_oid(&record.coord, &coord_ref)
            .with_context(|| format!("resolve the coordination ref {coord_ref}"))?;
        let funnel_record = funnel::read_funnel_at(&record.coord, &tip, binding.slice)?;
        Ok(Self {
            coord: record.coord.clone(),
            coord_ref,
            slice: binding.slice,
            phase: binding.phase.clone(),
            fork: record.branch.clone(),
            base_oid: record.base.clone(),
            row: funnel_record.row(&binding.phase).cloned(),
            tip,
        })
    }

    fn position(&self) -> Option<Position> {
        self.row.as_ref().map(|r| r.position)
    }

    /// The ladder this verb's Class-2 record lands (SL-228 PHASE-05, T10). With NO row
    /// at all, `create-fork`'s `Spawn` row is simply MISSING — that landing is
    /// deliberately non-fatal (D4), so a live, bound fork can legitimately reach here
    /// pre-spawn. Rather than refuse a real commit over a bookkeeping gap, this verb
    /// heals the `Spawn` rung in the SAME splice as its own row: one commit, all-or-
    /// none, under exactly the belts that already gate the record.
    ///
    /// Only the pre-spawn case grows a rung. From `spawned` the ladder is the single
    /// `RecordWorkerCommit` PHASE-04 landed, so a create-fork-written `Spawn` row is
    /// never re-stamped (and its `base_oid` spelling never has to match this one).
    fn ladder(&self, fork_tip: &str) -> Vec<Transition> {
        let record = Transition::RecordWorkerCommit {
            fork_tip: fork_tip.to_owned(),
        };
        match self.position() {
            None => vec![
                Transition::Spawn {
                    fork: self.fork.clone(),
                    base_oid: self.base_oid.clone(),
                },
                record,
            ],
            Some(_) => vec![record],
        }
    }

    /// The PRE-ACT gate (design §4): kind-level legality for the FIRST-COMMIT leg. It is
    /// `preflight`, not `attempt`, because the fork tip this transition records does not
    /// EXIST yet — the commit has not been made (RV-304 F-3, the same reason `verify`
    /// preflights). Advisory against races by design; the post-act CAS record below is
    /// the authority. It is what refuses a late re-commit at `imported`+ with
    /// `already-<position>`.
    ///
    /// The kind asked about is the FIRST rung of the ladder this call would land, so a
    /// pre-spawn fork is gated on `Spawn` (legal) rather than mis-refused `not-spawned`.
    fn preflight(&self) -> Result<(), IllegalTransition> {
        let kind = match self.position() {
            None => TransitionKind::Spawn,
            Some(_) => TransitionKind::RecordWorkerCommit,
        };
        funnel_machine::preflight(self.position(), kind)
    }

    /// Land the Class-2 `RecordWorkerCommit` row for `fork_tip` — plus the `Spawn` rung
    /// when the row is missing entirely — re-reading the tip on a lost-ref race (the
    /// replay rule makes the retry safe — design §3 CAS contract). The whole ladder is
    /// ONE splice and ONE commit, so a kill mid-heal lands nothing at all.
    fn record_worker_commit(&self, fork_tip: &str) -> anyhow::Result<()> {
        let ts = self.ladder(fork_tip);
        let at = crate::clock::now_timestamp()?;
        let prov = Provenance::Conclude {
            who: dispatch_identity(),
        };
        // Bounded: each iteration re-reads the tip a competing transition advanced to.
        let mut tip = self.tip.clone();
        for _ in 0..LAND_ATTEMPTS {
            match funnel::land_funnel_transitions(
                &self.coord,
                &self.coord_ref,
                &tip,
                self.slice,
                &self.phase,
                &ts,
                None,
                None,
                &crate::dispatch::funnel_message(self.slice, &self.phase),
                &at,
                &prov,
            )? {
                funnel::FunnelLanding::Landed { .. } | funnel::FunnelLanding::Replayed { .. } => {
                    return Ok(());
                }
                funnel::FunnelLanding::Illegal(illegal) => bail!("{illegal}"),
                funnel::FunnelLanding::CommitRefused(refusal) => {
                    if refusal != crate::dispatch::CommitRefusal::LostRefRace {
                        bail!("record worker-commit: {}", refusal.token());
                    }
                    tip = commit_oid(&self.coord, &self.coord_ref)?;
                }
            }
        }
        bail!("record worker-commit: lost the ref race {LAND_ATTEMPTS} times running")
    }
}

/// How many times a CAS landing re-reads the tip before giving up.
const LAND_ATTEMPTS: u32 = 3;

/// Turn a machine refusal into the verb's refusal: the token is the machine's own reason
/// (`already-imported`, `not-spawned`, …) and the detail is the FULL payload, which IS
/// the recovery procedure (FR-009 — surfaced verbatim, never paraphrased).
fn refused_illegal(illegal: &IllegalTransition) -> WorkerCommitOutput {
    refused(illegal.reason, illegal.to_string())
}

/// Resolve a commit oid in `dir` (`rev-parse <rev>^{commit}`).
fn commit_oid(dir: &Path, rev: &str) -> anyhow::Result<String> {
    git::git_text(dir, &["rev-parse", &format!("{rev}^{{commit}}")])
        .with_context(|| format!("rev-parse {rev} in {}", dir.display()))
}

/// The gated server-side worker self-commit (design §5.2; belt order §10 X4). Resolves
/// the opaque `agent` to its per-worktree record, runs the cheap-first belts over the
/// worker's live PRE-fmt delta, gates with `check commit`, and lands exactly one
/// non-merge commit on `dispatch/<agent>`. `root` is the PRIMARY (unconfined) MCP root —
/// the tamper-proof source of the forbidden-write config + the gate argv (both under the
/// worker-unwritable `.doctrine/` floor). A belt refusal is a normal `Ok(Refused{…})`.
///
/// # Two legs (SL-228 PHASE-04, design §3 / RV-305 F-1)
///
/// Its own crash window is healed by its own re-drive. A retry after a lost response, or
/// after a kill between the fork commit and the coord record, arrives wearing the
/// **retry signature** — worktree CLEAN, fork advanced by exactly one commit `C` with
/// `C^ == B`. Before this existed the resolver folded any `HEAD != base` to
/// `stale-record` and the worker's ambiguity ended in a misdiagnosis. Now:
///
/// * `AtBase` resolves ⇒ the **first-commit leg** (belts → commit → record);
/// * `AtBase` refuses `stale-record` ⇒ re-resolve declaring `Advanced`, and if the retry
///   signature holds, the **retry legs** (recorded no-op replay, or belt-gated
///   self-record). Anything else keeps the original `stale-record` diagnosis.
pub(crate) fn run_worker_commit(
    root: &Path,
    agent: &str,
    message: &str,
) -> anyhow::Result<WorkerCommitOutput> {
    // 1. Resolve the opaque agent-id → its per-worktree record (PHASE-01). No worker path
    //    enters resolution; a typed resolver refusal maps 1:1 to our refusal token.
    match resolve_agent(root, agent, ForkExpect::AtBase) {
        Ok(record) => first_commit_leg(root, &record, message),
        // The ONE ambiguous refusal: `HEAD != base` is either a genuinely stale record OR
        // the retry signature. Ask the resolver again, declaring what a retry looks like.
        Err(ResolveRefusal::StaleRecord) => {
            match resolve_agent(root, agent, ForkExpect::Advanced) {
                Ok(record) => retry_leg(root, &record),
                // Not one-commit-past-base either ⇒ the original diagnosis stands.
                Err(_) => Ok(refused(ResolveRefusal::StaleRecord.token(), String::new())),
            }
        }
        Err(refusal) => Ok(refused(refusal.token(), String::new())),
    }
}

/// The gathered inputs both acting legs share: the tamper-proof config, the scope
/// selectors, and the forbidden-write matcher. `root` is the PRIMARY MCP root.
struct Belts {
    cfg: crate::dtoml::DoctrineToml,
    forbidden: ForbiddenWrites,
    selectors: Vec<String>,
}

impl Belts {
    fn gather(root: &Path, record: &DispatchRecord, slice: u32) -> anyhow::Result<Self> {
        let cfg = crate::dtoml::load_doctrine_toml(root)?;
        Ok(Self {
            forbidden: cfg.dispatch.forbidden_writes(),
            selectors: design_target_selectors(&record.coord, slice).unwrap_or_default(),
            cfg,
        })
    }
}

/// The FIRST-COMMIT leg: the fork is at `B` and the worker's change is the live
/// working-tree delta. Belt order is unchanged (cheap-first, §10 X4); the funnel
/// pre-gate is inserted BEFORE the belts (design §4 — one step after resolve), and the
/// Class-2 record lands strictly AFTER the commit (D8).
fn first_commit_leg(
    root: &Path,
    record: &DispatchRecord,
    message: &str,
) -> anyhow::Result<WorkerCommitOutput> {
    // The durable fork binding — WHICH funnel row this fork's commit belongs to. An
    // unbound fork is not provable: refuse outright rather than commit into a row nobody
    // can name. There is no ungated fallback and no "skip and let import heal" arm —
    // import refuses the same unbound fork, so a skip would promise a heal that never
    // comes (design §4, RV-303 F-3 plus the F-3/F-4 contests).
    let binding = match require_binding(record) {
        Ok(binding) => binding,
        Err(refusal) => return Ok(refused(refusal.token(), record.branch.clone())),
    };
    let ctx = FunnelCtx::open(record, &binding)?;
    if let Err(illegal) = ctx.preflight() {
        return Ok(refused_illegal(&illegal));
    }

    // 2. Capture the PRE-fmt in-scope delta (tracked diff vs HEAD + untracked adds) — the
    //    worker cannot self-commit, so its change is the live working-tree delta at B.
    let delta = gather_worktree_delta_paths(&record.dir)?;
    if delta.is_empty() {
        return Ok(refused(EMPTY_DELTA, String::new()));
    }

    let belts = Belts::gather(root, record, binding.slice)?;

    // 3. Scope belt (cheap-first, over the PRE-fmt delta): HARD refuses, SOFT rides back.
    let undeclared = match classify_scope(&delta, &belts.forbidden, &belts.selectors) {
        Ok(undeclared) => undeclared,
        Err(path) => return Ok(refused(FORBIDDEN_ZONE, path)),
    };

    // 4. Commit invariant — HEAD == B (pre-commit). Belt-and-suspenders against the
    //    resolver's `AtBase` check, at the commit boundary itself.
    let base = commit_oid(&record.dir, &record.base)?;
    let head = commit_oid(&record.dir, "HEAD")?;
    if !head_at_base(&head, &base) {
        return Ok(refused(NOT_AT_BASE, String::new()));
    }

    // 5. Gate — the mutating `check commit` (fmt first) in the worker worktree.
    match run_commit_gate(&record.dir, &belts.cfg.verification)? {
        GateOutcome::Green => {}
        GateOutcome::Red(detail) => return Ok(refused(COMMIT_GATE_RED, detail)),
    }

    // 6. Stage the PRE-fmt classified paths (post-fmt content) + one non-merge commit.
    stage_and_commit(&record.dir, message, &delta)?;

    let oid = commit_oid(&record.dir, "HEAD")?;
    assert_one_commit_past_base(&record.dir, &oid, &base)?;

    // 7. THE ACT IS DONE. Record it (Class 2, D8: strictly after). A failed record is
    //    NOT a failed commit — the fork ref has advanced and saying otherwise would be a
    //    lie. Leave the lag; this verb's own re-drive heals it (retry signature), and
    //    import's heal-forward is the backstop when no retry ever comes.
    warn_if_unrecorded(&ctx, &oid);

    Ok(WorkerCommitOutput::Committed {
        oid,
        base,
        undeclared,
    })
}

/// The RETRY legs (design §3, RV-305 F-1). The resolver has already PROVEN the retry
/// signature's git half — the fork is exactly one non-merge commit `C` past `B`. What is
/// left is to decide which truthful terminal answer this is:
///
/// * a DIRTY tree is not a retry at all — it is a late re-commit, and `C^ == B` forbids
///   a second commit, so it refuses naming itself;
/// * a row already at `worker-committed` for this very `C` ⇒ a **recorded no-op replay**
///   naming the landed tip. No belts re-run: a row resting at exactly `worker-committed`
///   can only have been landed by a belt-running leg (this leg or the first-commit leg —
///   import's heal lands that row only in the SAME commit as `Import`, so position never
///   durably rests there via heal), so the stored row IS the belt proof, by induction;
/// * otherwise ⇒ the **belt-gated self-record**. This verb is the ORIGINAL recorder for
///   `RecordWorkerCommit` (D8), so it lands its own lagging row now — but only after
///   re-running the SAME content belts the first-commit leg enforces, against `C`'s
///   delta. The gate is PROVENANCE-INDIFFERENT: an ungated one-commit fork (a raw-git
///   bypass wearing the retry signature) either fails a belt and refuses with that belt
///   NAMED — never adopted, never recorded — or passes them all, in which case adoption
///   is sound by construction. The belts, not authorship, are the content authority, and
///   a commit they cannot distinguish from a legitimate one IS legitimate.
fn retry_leg(root: &Path, record: &DispatchRecord) -> anyhow::Result<WorkerCommitOutput> {
    let binding = match require_binding(record) {
        Ok(binding) => binding,
        Err(refusal) => return Ok(refused(refusal.token(), record.branch.clone())),
    };

    // A dirty tree on an advanced fork is a LATE RE-COMMIT, not a retry.
    let dirt = gather_worktree_delta_paths(&record.dir)?;
    if !dirt.is_empty() {
        return Ok(refused(LATE_RECOMMIT, dirt.join(", ")));
    }

    let base = commit_oid(&record.dir, &record.base)?;
    let c = commit_oid(&record.dir, "HEAD")?;
    let ctx = FunnelCtx::open(record, &binding)?;

    // The SAME ladder the record lands (SL-228 PHASE-05, T10), asked of the machine as
    // one all-or-none fold: a pre-spawn row heals its `Spawn` rung here too, rather
    // than mis-refusing a legitimately-committed fork `not-spawned`.
    let fold = match funnel_machine::fold_transitions(
        ctx.row.as_ref(),
        &ctx.phase,
        &ctx.ladder(&c),
        &ctx.tip,
        None,
        &crate::clock::now_timestamp()?,
    ) {
        Ok(Some(fold)) => fold,
        // `ladder` always yields at least `RecordWorkerCommit`.
        Ok(None) => return Ok(refused(EMPTY_DELTA, String::new())),
        // `already-imported` / `already-verified` / … — the imported tip is already
        // downstream, so there is no late re-commit to make. A truthful terminal answer.
        Err(illegal) => return Ok(refused_illegal(&illegal)),
    };

    if fold.replay {
        // Recorded no-op replay: nothing written, the landed tip NAMED. This is exactly
        // the response the worker lost. `undeclared` is empty because a replay
        // re-classifies nothing — the stored row already carries the belt proof.
        return Ok(WorkerCommitOutput::Committed {
            oid: c,
            base,
            undeclared: Vec::new(),
        });
    }

    // Belt-gated self-record. The delta under scrutiny is `C`'s OWN delta (`B..C`), not a
    // working-tree diff — the tree is clean by the check above.
    let delta = commit_delta_paths(&record.dir, &base, &c)?;
    if delta.is_empty() {
        return Ok(refused(EMPTY_DELTA, String::new()));
    }
    let belts = Belts::gather(root, record, binding.slice)?;
    let undeclared = match classify_scope(&delta, &belts.forbidden, &belts.selectors) {
        Ok(undeclared) => undeclared,
        Err(path) => return Ok(refused(FORBIDDEN_ZONE, path)),
    };
    match run_commit_gate(&record.dir, &belts.cfg.verification)? {
        GateOutcome::Green => {}
        GateOutcome::Red(detail) => return Ok(refused(COMMIT_GATE_RED, detail)),
    }

    // Every belt passed ⇒ adoption is sound. Land the lagging Class-2 row now.
    assert_one_commit_past_base(&record.dir, &c, &base)?;
    ctx.record_worker_commit(&c)?;

    Ok(WorkerCommitOutput::Committed {
        oid: c,
        base,
        undeclared,
    })
}

/// The paths `commit` changed against `base` — the SELF-RECORD leg's belt input, read
/// from committed trees (the working tree is clean here, so a working-tree diff would
/// report nothing at all).
fn commit_delta_paths(dir: &Path, base: &str, commit: &str) -> anyhow::Result<Vec<String>> {
    let out = git::git_text(dir, &["diff", "--name-only", base, commit])
        .with_context(|| format!("diff {base}..{commit} in {}", dir.display()))?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect())
}

/// INV-1 at the commit boundary: exactly one non-merge commit with `C^ == B`.
fn assert_one_commit_past_base(dir: &Path, oid: &str, base: &str) -> anyhow::Result<()> {
    // No indexing (clippy floor).
    match git::parents(dir, oid)?.as_slice() {
        [parent] if parent == base => Ok(()),
        [parent] => bail!("worker_commit parent {parent} != base {base} (C^ != B)"),
        other => bail!(
            "worker_commit produced a commit with {} parents (expected exactly 1)",
            other.len()
        ),
    }
}

/// Land the Class-2 record, warning (never failing) if it does not land — the commit is
/// already durable, so a failed record lags reality rather than undoing it (D8).
fn warn_if_unrecorded(ctx: &FunnelCtx, oid: &str) {
    if let Err(cause) = ctx.record_worker_commit(oid) {
        // stderr, never stdout: stdout is the MCP JSON-RPC wire.
        drop(writeln!(
            std::io::stderr(),
            "warning: worker-commit row not landed for {} {} ({cause:#}) — the commit {oid} IS durable; a re-drive or import heals the row forward",
            ctx.slice,
            ctx.phase
        ));
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on fixture setup is idiomatic"
)]
mod tests {
    use super::*;
    use crate::dispatch_config::DispatchConfig;
    use crate::worktree::{Apply, Refusal, classify_import, provision_dispatch_record};
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn forbidden_from(lines: &[&str]) -> ForbiddenWrites {
        let cfg = DispatchConfig {
            worker_forbidden_writes: lines.iter().map(|s| (*s).to_string()).collect(),
            ..DispatchConfig::default()
        };
        cfg.forbidden_writes()
    }

    // --- VT-1 / VT-3 / VT-5: the PURE scope belt --------------------------------------

    #[test]
    fn classify_scope_hard_refuses_the_doctrine_and_claude_floors() {
        // The `.doctrine/`/`.claude/` code floor fails closed even with an EMPTY config —
        // it is not sourced from the matcher (VT-1 forbidden-zone).
        let fw = forbidden_from(&[]);
        assert_eq!(
            classify_scope(&[".doctrine/state/x".to_string()], &fw, &[]),
            Err(".doctrine/state/x".to_string())
        );
        assert_eq!(
            classify_scope(&[".claude/agents/w.md".to_string()], &fw, &[]),
            Err(".claude/agents/w.md".to_string())
        );
    }

    #[test]
    fn classify_scope_hard_refuses_a_config_forbidden_agent_def_or_flake() {
        // The config matcher hard-fences agent-defs + flake.nix (the template defaults).
        let fw = forbidden_from(&["flake.nix", "install/agents/**"]);
        assert_eq!(
            classify_scope(&["flake.nix".to_string()], &fw, &[]),
            Err("flake.nix".to_string())
        );
        assert_eq!(
            classify_scope(
                &["install/agents/claude/dispatch-worker.md".to_string()],
                &fw,
                &[]
            ),
            Err("install/agents/claude/dispatch-worker.md".to_string())
        );
    }

    #[test]
    fn classify_scope_soft_reports_undeclared_without_blocking() {
        // VT-5 (soft): a src write outside the design-target selectors COMMITS (Ok) and
        // rides back as `undeclared`; a declared path does not.
        let fw = forbidden_from(&[]);
        let selectors = vec!["src/allowed.rs".to_string()];
        let delta = vec!["src/allowed.rs".to_string(), "src/other.rs".to_string()];
        assert_eq!(
            classify_scope(&delta, &fw, &selectors),
            Ok(vec!["src/other.rs".to_string()])
        );
    }

    #[test]
    fn classify_scope_forbidden_wins_even_alongside_an_undeclared_path() {
        // VT-5 (hard still wins): a forbidden-zone write in the same call hard-refuses,
        // regardless of a co-present undeclared src path.
        let fw = forbidden_from(&[]);
        let selectors = vec!["src/allowed.rs".to_string()];
        let delta = vec!["src/other.rs".to_string(), ".doctrine/x".to_string()];
        assert_eq!(
            classify_scope(&delta, &fw, &selectors),
            Err(".doctrine/x".to_string())
        );
    }

    #[test]
    fn worker_commit_and_classify_import_agree_on_the_hard_verdict() {
        // VT-3: both belts REJECT the same `.doctrine/`/`.claude/` change-set and ACCEPT
        // the same benign src set — the shared consts + `undeclared_paths` predicate, not
        // a forked copy. `classify_import`'s stationary-head preconds are all true here so
        // only the scope verdict differs.
        let fw = forbidden_from(&[]);
        for hard in [".doctrine/state/x", ".claude/agents/w.md"] {
            let delta = vec![hard.to_string()];
            assert!(
                classify_scope(&delta, &fw, &[]).is_err(),
                "worker_commit rejects {hard}"
            );
            assert!(
                classify_import(true, true, true, &delta, &[]).is_err(),
                "classify_import rejects {hard}"
            );
        }
        // A benign src path: both accept.
        let benign = vec!["src/lib.rs".to_string()];
        assert!(classify_scope(&benign, &fw, &[]).is_ok());
        assert_eq!(
            classify_import(true, true, true, &benign, &[]),
            Ok(Apply::Ok)
        );
        // And the specific import token for a `.doctrine/` path is DoctrineTouch — the
        // hard verdict worker_commit mirrors as `forbidden-zone`.
        assert_eq!(
            classify_import(true, true, true, &[".doctrine/x".to_string()], &[]),
            Err(Refusal::DoctrineTouch)
        );
    }

    #[test]
    fn head_at_base_is_exact_equality() {
        assert!(head_at_base("abc123", "abc123"));
        assert!(!head_at_base("abc123", "def456"));
    }

    // --- integration: a live per-worktree record + a real linked worktree -------------

    fn git_run(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// The slice + phase every fixture fork is BOUND to (SL-228 PHASE-04).
    const SLICE: u32 = 199;
    const PHASE: &str = "PHASE-01";

    /// The fixture's durable fork binding.
    fn fixture_binding() -> ForkBinding {
        ForkBinding {
            slice: SLICE,
            phase: PHASE.to_owned(),
        }
    }

    /// Stand up a primary repo at base B, a COORDINATION worktree on `dispatch/<SLICE>`,
    /// and a linked worker worktree on `dispatch/<name>` at `<coord>/.worktrees/<name>`,
    /// plus its BOUND per-worktree record and a funnel row at `spawned`.
    /// `commit_override` is written into the primary `[verification].commit` (the gate).
    ///
    /// The coord tree and the funnel row are net-new here (SL-228 PHASE-04): the verb now
    /// gates on the funnel machine and records a Class-2 row on the coord ref, so a fork
    /// with no coordination tree and no binding is no longer a legitimate input — it is
    /// exactly the `unprovable-fork` case the design refuses. Returns
    /// `(tmp, primary, wt, agent, base)`.
    fn worker_fixture(
        commit_override: &str,
        seed_files: &[(&str, &str)],
    ) -> (tempfile::TempDir, PathBuf, PathBuf, String, String) {
        let tmp = tempfile::tempdir().unwrap();
        let primary = fs::canonicalize(tmp.path()).unwrap().join("primary");
        fs::create_dir_all(&primary).unwrap();
        git_run(&primary, &["init", "-q", "-b", "main"]);
        git_run(&primary, &["config", "user.email", "t@t"]);
        git_run(&primary, &["config", "user.name", "t"]);
        // Base tree.
        for (rel, body) in seed_files {
            let path = primary.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, body).unwrap();
        }
        fs::write(primary.join("seed"), "base\n").unwrap();
        // Mirror production: `.doctrine/state/**` is runtime/gitignored, so a stamped
        // worker marker never enters the worker's tracked/untracked commit delta.
        fs::write(primary.join(".gitignore"), ".doctrine/state/\n").unwrap();
        git_run(&primary, &["add", "-A"]);
        git_run(&primary, &["commit", "-q", "-m", "base"]);
        let base = git_run(&primary, &["rev-parse", "HEAD^{commit}"]);

        // The tamper-proof gate config lives on the PRIMARY (the MCP root).
        fs::create_dir_all(primary.join(".doctrine")).unwrap();
        fs::write(
            primary.join(".doctrine/doctrine.toml"),
            format!("[verification]\ncommit = {commit_override}\n"),
        )
        .unwrap();

        // The COORDINATION worktree on `dispatch/<SLICE>` — the tree whose ref carries
        // the funnel record the verb gates on and records to.
        let coord = fs::canonicalize(tmp.path()).unwrap().join("coord");
        git_run(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                &format!("dispatch/{SLICE:03}"),
                coord.to_str().unwrap(),
                &base,
            ],
        );
        let coord = fs::canonicalize(&coord).unwrap();

        // The worker worktree + its BOUND per-worktree record.
        let agent = "wk1".to_string();
        let wt = coord.join(".worktrees").join(&agent);
        fs::create_dir_all(coord.join(".worktrees")).unwrap();
        git_run(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                &format!("dispatch/{agent}"),
                wt.to_str().unwrap(),
                &base,
            ],
        );
        let wt = fs::canonicalize(&wt).unwrap();
        provision_dispatch_record(
            &coord,
            &agent,
            &base,
            &wt,
            &format!("dispatch/{agent}"),
            Some(&fixture_binding()),
        )
        .unwrap();

        // The phase stands at `spawned` — what `create-fork` lands post-act (D8).
        land(
            &coord,
            &Transition::Spawn {
                fork: format!("dispatch/{agent}"),
                base_oid: base.clone(),
            },
        );

        (tmp, primary, wt, agent, base)
    }

    /// Land one funnel transition on the fixture's coord ref, through the sole writer.
    fn land(coord: &Path, t: &Transition) {
        let coord_ref = dispatch_ref(SLICE);
        let tip = git_run(coord, &["rev-parse", &format!("{coord_ref}^{{commit}}")]);
        match funnel::land_funnel_transition(
            coord,
            &coord_ref,
            &tip,
            SLICE,
            PHASE,
            t,
            None,
            "2026-07-26T00:00:00Z",
            &Provenance::Conclude {
                who: dispatch_identity(),
            },
        )
        .unwrap()
        {
            funnel::FunnelLanding::Landed { .. } => {}
            other => panic!("fixture: expected Landed, got {other:?}"),
        }
    }

    /// The coordination root for a fixture worker worktree.
    fn coord_of(wt: &Path) -> PathBuf {
        wt.parent()
            .expect("wt has a .worktrees parent")
            .parent()
            .expect(".worktrees has a coord parent")
            .to_path_buf()
    }

    /// The phase's landed funnel row (read at the coord tip, object-db only).
    fn funnel_row(coord: &Path) -> Option<PhaseRow> {
        let coord_ref = dispatch_ref(SLICE);
        let tip = git_run(coord, &["rev-parse", &format!("{coord_ref}^{{commit}}")]);
        funnel::read_funnel_at(coord, &tip, SLICE)
            .unwrap()
            .row(PHASE)
            .cloned()
    }

    #[test]
    fn worker_commit_unknown_agent_refuses() {
        // VT-1: the resolver refusal token surfaces as our refusal.
        let (_tmp, primary, _wt, _agent, _base) = worker_fixture("[\"true\"]", &[]);
        let out = run_worker_commit(&primary, "nosuchagent", "msg").unwrap();
        assert_eq!(
            out,
            refused("unknown-agent", String::new()),
            "an unresolvable agent refuses unknown-agent"
        );
    }

    #[test]
    fn worker_commit_empty_delta_refuses() {
        // VT-1: no in-scope change ⇒ empty-delta (no empty commit).
        let (_tmp, primary, _wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        assert_eq!(out, refused(EMPTY_DELTA, String::new()));
    }

    #[test]
    fn worker_commit_gate_red_refuses_and_leaves_head_at_base() {
        // VT-1: a red gate refuses commit-gate-red; the branch stays at B (no commit).
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"false\"]", &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Refused { reason, .. } => assert_eq!(reason, COMMIT_GATE_RED),
            other => panic!("expected commit-gate-red, got {other:?}"),
        }
        assert_eq!(git_run(&wt, &["rev-parse", "HEAD^{commit}"]), base);
    }

    #[test]
    fn worker_commit_gate_clears_marker_and_unsets_env_then_restores() {
        // SL-199 F2: the gate runs the trusted suite in the MARKED fork. It must clear the
        // worker marker AND leave DOCTRINE_WORKER unset so authored-write test fixtures are
        // not refused by the worker-mode guard. A gate that passes IFF the marker file is
        // absent AND the env is unset lands the commit only when both hold; the marker is
        // then restored past the gate.
        let gate = r#"["sh", "-c", "test ! -f .doctrine/state/dispatch/worker && test -z \"$DOCTRINE_WORKER\""]"#;
        let (_tmp, primary, wt, agent, base) = worker_fixture(gate, &[]);
        // Stamp the marker exactly as `create-fork` does on the real fork.
        crate::worktree::write_marker(&wt).unwrap();
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Committed { base: out_base, .. } => assert_eq!(out_base, base),
            other => panic!("gate must see a cleared marker + unset env and pass; got {other:?}"),
        }
        // Protection restored: the marker is back after the gate window.
        assert!(
            crate::worktree::marker_present(&wt),
            "the worker marker must be restored after the gate"
        );
    }

    #[test]
    fn worker_commit_gate_carries_dispatch_gate_signal() {
        // VT-1e (SL-225 #1): the gate spawn sets DOCTRINE_DISPATCH_GATE=1 so `just validate`
        // knows it runs inside the worker_commit gate window — needed because the gate clears
        // the marker AND unsets DOCTRINE_WORKER (SL-199 F2), leaving this the only visible
        // worker-context signal. A gate that passes IFF the var == "1" commits only when the
        // engine sets it.
        let gate = r#"["sh", "-c", "test \"$DOCTRINE_DISPATCH_GATE\" = 1"]"#;
        let (_tmp, primary, wt, agent, base) = worker_fixture(gate, &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Committed { base: out_base, .. } => assert_eq!(out_base, base),
            other => panic!("gate must see DOCTRINE_DISPATCH_GATE=1 and pass; got {other:?}"),
        }
    }

    #[test]
    fn worker_commit_forbidden_zone_refuses() {
        // VT-1: an AUTHORED `.doctrine/` write hard-refuses forbidden-zone (before the gate).
        // Runtime `.doctrine/state/**` is gitignored (never in the delta); the belt guards
        // authored state (`adr`/`slice`/…), which IS tracked — use one of those paths.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        fs::create_dir_all(wt.join(".doctrine/adr")).unwrap();
        fs::write(wt.join(".doctrine/adr/x"), "sneaky\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Refused { reason, detail } => {
                assert_eq!(reason, FORBIDDEN_ZONE);
                assert!(
                    detail.contains(".doctrine/adr/x"),
                    "names the path: {detail}"
                );
            }
            other => panic!("expected forbidden-zone, got {other:?}"),
        }
        assert_eq!(git_run(&wt, &["rev-parse", "HEAD^{commit}"]), base);
    }

    #[test]
    fn worker_commit_happy_path_lands_one_non_merge_commit() {
        // VT-2: an in-scope, gate-green tree → one non-merge commit, C^==B, parents==1.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "the message").unwrap();
        let (oid, out_base) = match out {
            WorkerCommitOutput::Committed { oid, base, .. } => (oid, base),
            other => panic!("expected Committed, got {other:?}"),
        };
        assert_eq!(out_base, base, "returned base == B");
        // The branch advanced B → C, exactly one non-merge commit, C^ == B.
        assert_eq!(git_run(&wt, &["rev-parse", "HEAD^{commit}"]), oid);
        let parents = git_run(&wt, &["rev-list", "--parents", "-n", "1", &oid]);
        let cols: Vec<&str> = parents.split_whitespace().collect();
        assert_eq!(cols.len(), 2, "exactly one parent: {parents}");
        assert_eq!(cols[1], base, "C^ == B");
        assert_eq!(git_run(&wt, &["log", "-1", "--format=%s"]), "the message");
    }

    #[test]
    fn worker_commit_stages_only_the_in_scope_path_after_the_gate_fmt() {
        // VT-4 (F2 guard): the gate reformats a pre-existing out-of-scope file; the
        // commit contains ONLY the in-scope path the worker touched, not the file the
        // gate rewrote. The "fmt" is simulated by a gate that mutates outofscope.txt.
        let (_tmp, primary, wt, agent, _base) = worker_fixture(
            "[\"sh\", \"-c\", \"printf reformatted > outofscope.txt\"]",
            &[
                ("outofscope.txt", "misformatted\n"),
                ("src/feature.rs", "fn f(){}\n"),
            ],
        );
        // Worker touches only the in-scope file.
        fs::write(wt.join("src/feature.rs"), "fn f() {}\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "in-scope only").unwrap();
        let oid = match out {
            WorkerCommitOutput::Committed { oid, .. } => oid,
            other => panic!("expected Committed, got {other:?}"),
        };
        let touched = git_run(&wt, &["show", "--name-only", "--format=", &oid]);
        let names: Vec<&str> = touched.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(
            names,
            vec!["src/feature.rs"],
            "commit contains ONLY the in-scope path, not the gate-reformatted file"
        );
    }

    #[test]
    fn worker_commit_soft_undeclared_still_commits() {
        // VT-5 (shell): a src write outside the (here empty) design-target selectors
        // COMMITS and returns the path as `undeclared` — never dropped, never blocked.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Committed { undeclared, .. } => {
                assert!(
                    undeclared.contains(&"seed".to_string()),
                    "the out-of-selector src path is reported undeclared: {undeclared:?}"
                );
            }
            other => panic!("expected Committed, got {other:?}"),
        }
    }

    // --- VT-2: wrong-base revive rejection (design §5.5 mechanism 2, RV-258 F-5) -------

    #[test]
    fn worker_commit_wrong_base_revive_refuses_before_any_tip_moves() {
        // Design §5.5: a fixup revive's `DispatchRecord` `base` is re-stamped to
        // `fork_tip` — the prior worker's durable committed delta — precisely so
        // `worker_commit` mechanically rejects a revive whose worktree HEAD does NOT
        // actually descend from `fork_tip` (a botched/omitted reset), BEFORE any tip
        // moves. RV-258 F-5: a raw `git reset --hard` + prompt-obedience is NO
        // guarantee — the guarantee is this base-check, the SAME belt
        // ([`head_at_base`] / the resolver's stale-record consistency check) that
        // guards every initial commit. No test step here relies on the revive worker
        // "obeying" a fixup prompt; the mismatch alone must refuse.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);

        // The PRIOR worker's landed commit — the durable `fork_tip` a revive must
        // resume from.
        fs::write(wt.join("seed"), "prior worker work\n").unwrap();
        git_run(&wt, &["commit", "-aq", "-m", "prior worker commit"]);
        let fork_tip = git_run(&wt, &["rev-parse", "HEAD^{commit}"]);
        assert_ne!(
            fork_tip, base,
            "fork_tip must have advanced past the original base"
        );

        // Simulate a BOTCHED revive: the fresh worktree's fixup edit lands WITHOUT the
        // worker ever actually resetting to `fork_tip` — HEAD stays at the stale `base`
        // instead (the un-enforced reset step RV-258 F-5 flags as no guarantee).
        git_run(&wt, &["reset", "--hard", &base]);
        fs::write(wt.join("seed"), "revive edit at the wrong base\n").unwrap();

        // The revive's DispatchRecord `base` IS re-stamped to `fork_tip` (what design
        // §5.5 requires) — the enforcement lives entirely in the base-check below, not
        // in trusting that the reset happened.
        provision_dispatch_record(
            &coord,
            &agent,
            &fork_tip,
            &wt,
            &format!("dispatch/{agent}"),
            Some(&fixture_binding()),
        )
        .unwrap();

        let out = run_worker_commit(&primary, &agent, "revive commit").unwrap();
        match out {
            WorkerCommitOutput::Refused { reason, .. } => {
                assert!(
                    reason == NOT_AT_BASE || reason == "stale-record",
                    "a wrong-base revive must refuse via the base-check family \
                     (head_at_base / the resolver's HEAD==base consistency check), \
                     never commit: got reason {reason}"
                );
            }
            other => panic!(
                "wrong-base revive MUST be refused before any tip moves — there is no \
                 prompt-obedience substitute for the base-check; got {other:?}"
            ),
        }
        // No tip moved: the worktree HEAD sits exactly where the botched reset left it,
        // never having advanced past `base` (and never onto a new commit).
        assert_eq!(
            git_run(&wt, &["rev-parse", "HEAD^{commit}"]),
            base,
            "no commit landed — HEAD did not advance past the wrong base"
        );
    }

    // ==================================================================================
    // SL-228 PHASE-04 — the funnel gate, the Class-2 record, and the retry legs
    // (VT-3 / VT-8; design §3 RV-305 F-1, §4).
    // ==================================================================================

    /// Commit `C` on the fork with RAW git — no belt ever ran. This is BOTH the
    /// "killed between the fork commit and the coord record" state AND the ungated
    /// raw-git bypass wearing the retry signature; the two are indistinguishable by
    /// construction, which is exactly the point of a provenance-INDIFFERENT gate.
    fn raw_commit_on_fork(wt: &Path, files: &[(&str, &str)]) -> String {
        for (rel, body) in files {
            let path = wt.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, body).unwrap();
        }
        git_run(wt, &["add", "-A"]);
        git_run(wt, &["commit", "-q", "-m", "raw"]);
        git_run(wt, &["rev-parse", "HEAD^{commit}"])
    }

    #[test]
    fn first_commit_leg_lands_the_class_2_worker_commit_row_after_the_act() {
        // VT-3 / D8: the record lands STRICTLY AFTER the act, and names the tip that
        // actually landed. Before this the fork ref advanced with no coord row at all.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        assert_eq!(
            funnel_row(&coord).map(|r| r.position),
            Some(Position::Spawned),
            "the phase stands at spawned before the worker commits"
        );

        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let oid = match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Committed { oid, .. } => oid,
            other => panic!("expected Committed, got {other:?}"),
        };

        let row = funnel_row(&coord).expect("a row was landed");
        assert_eq!(row.position, Position::WorkerCommitted);
        assert_eq!(
            row.worker_commit.map(|w| w.fork_tip),
            Some(oid),
            "the row names the tip that actually landed"
        );
    }

    // --- SL-228 PHASE-05 (T10): the pre-spawn heal ------------------------------------

    /// Rewind the coord ref past `create-fork`'s `Spawn` row — the landing is
    /// deliberately NON-FATAL (D4), so a live, bound fork can genuinely reach
    /// `worker_commit` with no funnel row at all.
    fn drop_the_spawn_row(coord: &Path, base: &str) {
        git_run(coord, &["update-ref", &dispatch_ref(SLICE), base]);
        assert!(funnel_row(coord).is_none(), "fixture: the row is gone");
    }

    #[test]
    fn a_pre_spawn_row_is_healed_in_the_same_splice_as_the_worker_commit_row() {
        // Before PHASE-05 this refused `not-spawned`: the writer took ONE transition, so
        // a missing (non-fatal) `Spawn` row blocked a perfectly legitimate commit. Now
        // the verb heals the rung it is already the recorder for — in ONE splice.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        drop_the_spawn_row(&coord, &base);

        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let oid = match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Committed { oid, .. } => oid,
            other => panic!("expected Committed, got {other:?}"),
        };
        let row = funnel_row(&coord).expect("the healed row");
        assert_eq!(row.position, Position::WorkerCommitted);
        assert_eq!(
            row.spawn.expect("the Spawn rung healed").fork,
            format!("dispatch/{agent}"),
            "the missing create-fork row is filled from the durable record"
        );
        assert_eq!(row.worker_commit.map(|w| w.fork_tip), Some(oid));
        // BOTH rungs in ONE commit — all-or-none, no intermediate position to be
        // killed at.
        let tip = git_run(&coord, &["rev-parse", &dispatch_ref(SLICE)]);
        assert_eq!(
            git_run(&coord, &["rev-list", "--count", &format!("{base}..{tip}")]),
            "1"
        );
    }

    #[test]
    fn the_retry_leg_also_heals_a_pre_spawn_row_under_the_same_belts() {
        // The adopt arm meets the same gap: the fork already carries its commit and the
        // row is missing entirely. The belts still run (this is the self-record leg),
        // and the heal rides the same splice.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        let c = raw_commit_on_fork(&wt, &[("seed", "ungated but benign\n")]);
        drop_the_spawn_row(&coord, &base);

        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Committed { oid, .. } => {
                assert_eq!(oid, c, "the landed commit is adopted, not re-made");
            }
            other => panic!("expected Committed, got {other:?}"),
        }
        let row = funnel_row(&coord).expect("the healed row");
        assert_eq!(row.position, Position::WorkerCommitted);
        assert!(
            row.spawn.is_some(),
            "the Spawn rung healed on the retry leg too"
        );
    }

    #[test]
    fn an_unbound_fork_refuses_unprovable_fork_and_commits_nothing() {
        // EX-2 / design §4: no ungated fallback and no "skip and let import heal" arm —
        // import refuses the same unbound fork, so a skip would promise a heal that
        // never comes. The refusal NAMES the fork.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        // Strip the binding: a live, perfectly consistent fork that nothing can place.
        provision_dispatch_record(
            &coord,
            &agent,
            &base,
            &wt,
            &format!("dispatch/{agent}"),
            None,
        )
        .unwrap();

        fs::write(wt.join("seed"), "worker change\n").unwrap();
        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Refused { reason, detail } => {
                assert_eq!(reason, "unprovable-fork");
                assert_eq!(detail, format!("dispatch/{agent}"), "names the fork");
            }
            other => panic!("expected unprovable-fork, got {other:?}"),
        }
        assert_eq!(
            git_run(&wt, &["rev-parse", "HEAD^{commit}"]),
            base,
            "nothing committed"
        );
    }

    #[test]
    fn a_lost_response_retry_is_a_recorded_no_op_replay_naming_the_landed_tip() {
        // VT-3: the response was lost, not the commit. Re-driving must NOT mint a second
        // commit and must NOT misdiagnose `stale-record` — it answers with the tip that
        // is already landed and recorded.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let first = run_worker_commit(&primary, &agent, "msg").unwrap();
        let oid = match first {
            WorkerCommitOutput::Committed { ref oid, .. } => oid.clone(),
            other => panic!("expected Committed, got {other:?}"),
        };
        let coord_tip_before = git_run(&coord, &["rev-parse", &dispatch_ref(SLICE)]);

        // The worker never saw that reply and re-drives the identical call.
        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Committed {
                oid: again,
                base: again_base,
                undeclared,
            } => {
                assert_eq!(again, oid, "the replay NAMES the already-landed tip");
                assert_eq!(again_base, base);
                assert!(undeclared.is_empty(), "a replay re-classifies nothing");
            }
            other => panic!("expected a no-op replay, got {other:?}"),
        }
        assert_eq!(
            git_run(&wt, &["rev-parse", "HEAD^{commit}"]),
            oid,
            "no second commit was minted"
        );
        assert_eq!(
            git_run(&coord, &["rev-parse", &dispatch_ref(SLICE)]),
            coord_tip_before,
            "a replay writes NOTHING — the coord ref did not move"
        );
    }

    #[test]
    fn a_kill_between_the_fork_commit_and_the_coord_record_self_records_on_re_drive() {
        // VT-3 / VT-8 (adopt arm): the fork carries exactly one commit whose delta passes
        // every content belt, but the coord row still says `spawned`. This verb IS the
        // original recorder (D8), so the re-drive lands its own lagging row — and does so
        // regardless of authorship, because the belts, not provenance, are the authority.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        let c = raw_commit_on_fork(&wt, &[("seed", "ungated but benign\n")]);
        assert_eq!(
            funnel_row(&coord).map(|r| r.position),
            Some(Position::Spawned),
            "the row lags the act — exactly the crash window"
        );

        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Committed { oid, .. } => {
                assert_eq!(oid, c, "the landed commit is adopted, not re-made");
            }
            other => panic!("expected the belt-gated self-record, got {other:?}"),
        }
        let row = funnel_row(&coord).expect("the lagging row was landed");
        assert_eq!(row.position, Position::WorkerCommitted);
        assert_eq!(row.worker_commit.map(|w| w.fork_tip), Some(c));
    }

    #[test]
    fn the_self_record_leg_refuses_naming_the_belt_an_ungated_commit_violates() {
        // VT-8 (refuse arm): an ungated one-commit fork WEARING the retry signature whose
        // delta violates a content belt is never adopted and never recorded — it refuses
        // NAMING that belt. This is the whole security claim of belt-gated adoption.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        let c = raw_commit_on_fork(&wt, &[(".doctrine/adr/x", "sneaky\n")]);

        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Refused { reason, detail } => {
                assert_eq!(reason, FORBIDDEN_ZONE, "the SCOPE belt is named");
                assert!(
                    detail.contains(".doctrine/adr/x"),
                    "names the path: {detail}"
                );
            }
            other => panic!("expected forbidden-zone, got {other:?}"),
        }
        assert_eq!(
            funnel_row(&coord).map(|r| r.position),
            Some(Position::Spawned),
            "never adopted, never recorded — the row still lags"
        );
        assert_eq!(
            git_run(&wt, &["rev-parse", "HEAD^{commit}"]),
            c,
            "the fork is left exactly as found (a triage beat, not a rescue)"
        );
    }

    #[test]
    fn the_self_record_leg_refuses_a_gate_red_ungated_commit() {
        // VT-8: the GATE belt is re-run against `C` too, not just the scope belt — a
        // one-commit fork that cannot pass `check commit` is not adopted.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"false\"]", &[]);
        let coord = coord_of(&wt);
        raw_commit_on_fork(&wt, &[("seed", "ungated\n")]);

        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Refused { reason, .. } => assert_eq!(reason, COMMIT_GATE_RED),
            other => panic!("expected commit-gate-red, got {other:?}"),
        }
        assert_eq!(
            funnel_row(&coord).map(|r| r.position),
            Some(Position::Spawned),
            "a red gate adopts nothing"
        );
    }

    #[test]
    fn a_dirty_advanced_fork_is_a_late_recommit_not_a_retry() {
        // VT-3: the retry signature requires a CLEAN tree. Dirt on an already-committed
        // fork is a second commit attempt, which `C^ == B` forbids — refuse, naming it,
        // rather than misdiagnosing `stale-record`.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        let c = raw_commit_on_fork(&wt, &[("seed", "first\n")]);
        fs::write(wt.join("seed"), "a second, later edit\n").unwrap();

        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Refused { reason, detail } => {
                assert_eq!(reason, LATE_RECOMMIT);
                assert!(detail.contains("seed"), "names the dirt: {detail}");
            }
            other => panic!("expected late-recommit, got {other:?}"),
        }
        assert_eq!(
            git_run(&wt, &["rev-parse", "HEAD^{commit}"]),
            c,
            "no second commit"
        );
    }

    #[test]
    fn an_imported_phase_refuses_already_imported_on_both_legs() {
        // VT-3: past `worker-committed` there is no late re-commit to make. Both the
        // pre-act gate (clean fork at base) and the retry leg (advanced fork) answer with
        // the machine's own `already-<position>` payload — a truthful terminal answer,
        // never `stale-record`.
        // (a) the PRE-ACT gate on a fork still at base.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        let coord = coord_of(&wt);
        let fork = format!("dispatch/{agent}");
        land(
            &coord,
            &Transition::RecordWorkerCommit {
                fork_tip: "deadbeef".to_owned(),
            },
        );
        land(
            &coord,
            &Transition::Import {
                fork_tip: "deadbeef".to_owned(),
                onto: "cafe".to_owned(),
            },
        );
        fs::write(wt.join("seed"), "too late\n").unwrap();
        match run_worker_commit(&primary, &agent, "msg").unwrap() {
            WorkerCommitOutput::Refused { reason, detail } => {
                assert_eq!(reason, "already-imported");
                assert!(
                    detail.contains("imported"),
                    "the payload IS the recovery procedure: {detail}"
                );
            }
            other => panic!("expected already-imported, got {other:?}"),
        }

        // (b) the RETRY leg on an advanced fork at the same position.
        let (_tmp2, primary2, wt2, agent2, _base2) = worker_fixture("[\"true\"]", &[]);
        let coord2 = coord_of(&wt2);
        let c = raw_commit_on_fork(&wt2, &[("seed", "landed\n")]);
        land(
            &coord2,
            &Transition::RecordWorkerCommit {
                fork_tip: c.clone(),
            },
        );
        land(
            &coord2,
            &Transition::Import {
                fork_tip: c,
                onto: "cafe".to_owned(),
            },
        );
        match run_worker_commit(&primary2, &agent2, "msg").unwrap() {
            WorkerCommitOutput::Refused { reason, .. } => assert_eq!(reason, "already-imported"),
            other => panic!("expected already-imported, got {other:?}"),
        }
        drop(fork);
    }
}
