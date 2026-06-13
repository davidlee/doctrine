# Eighth Inquisition — SL-056 design (`design.md`, clean-rewrite re-lock target)

Convened for `nihil obstat` upon the rewritten `design.md` (commit `24e8ed7`),
after the `WorktreeCreate`-hook pivot (`ef2e50d`). Seven prior passes lie in
`inquisition-1.md`…`-7.md`; the round-1→7 reasoning is preserved in
`design-history.md` (superseded). This pass interrogates whether the pivot — a
*foundational* change to the claude worker path — smuggled fresh mechanism past the
design's own gate.

Doctrine consulted: `doctrine slice show SL-056`; `design.md` (clean); `slice-056.md`
(scope); `inquisition-7.md`; `design-history.md` §"Mechanism admission rule" (the
ten-question standard, the design's own governing canon) and §"Invariants preserved";
the two empirical hook memories
`mem.pattern.dispatch.claude-subagentstart-worker-identity` and
`mem.pattern.dispatch.claude-agent-worktree-not-fork-provisioned`; ADR-006
(D2/D2b/D6a/D7/D9), ADR-008 (D-B1/D-B3); the boot storage/process rules. An external
adversarial reviewer (codex / GPT‑5.5) was put to the rack independently and confessed
the **same seven heresies, unprompted by my own count** — a convergence I record as
aggravating, not exculpating.

**`Nihil obstat` is DENIED.**

First, the absolution. The token-diff against `design-history.md` is clean: the only
refusal lexemes the rewrite shed — `already-armed`, `no-armed-sentinel`,
`stale-arm-cleared` — are the *deliberately dissolved* arm-sentinel apparatus (round‑7
ο, obviated by the concurrency probe). The surviving invariants the handover named —
belt scope (§7b), the `land` guard family (§6: `worktree-gone`, `wedged-merge`,
`inconsistent-merge-state`), the router cross-check (§11), altitude honesty (§10), the
SR‑4 payload-not-process-cwd rule (§4b/§12) — all carried forward. Round‑7 ν (§4a
`env -C "$D"`), ξ (§8.2 idempotent gc), π (§12 exhaustive `Orchestrator` list incl.
`land`) are genuinely answered. The topology stands. **The heresy is not in the bones;
it is in the seam of the new limb grafted on** — precisely where the design-history
warns it always festers.

---

## Charges

### Charge ρ (rho) — HIGH — `create-fork` mints a worktree by its own hand, then claims "no worktree" on failure, naming no cleanup, no leftover, no rollback owner

**Doctrine violated.** The design's own **Mechanism admission rule**
(`design-history.md` §"Mechanism admission rule") forbids relying on any git-mutating
verb or cleanup path that cannot answer all ten questions — explicitly **Q3** (creation
fails halfway → no corrupt/half-state), **Q4** (cleanup fails → name the leftover, exit
non‑zero — the Charge VIII standard), **Q8** (what refusal names its bad states). "A
mechanism that cannot is **rejected, not shipped with a gap.**" The §1 thesis: mechanism
lives in fail‑closed verbs, not faith.

**Evidence — confessed in the pseudocode itself.** §4b's `create-fork` runs, *in the
hook's own process*: `git worktree add -b <branch> <dir> <HEAD>` → `provision` →
`write_marker` → `print <dir>`, sealed with "`any failure → non-zero exit → creation
FAILS → no worktree → no worker (FAIL-CLOSED)`". But the hook **replaces** default git
creation — *the hook itself created the worktree on disk*. A provision or stamp failure
*after* `git worktree add` succeeds yields a non‑zero exit that tells Claude "no
worker" — yet the git-added directory and branch **remain on disk, an orphan Claude
never made and will not reap.** §4a `fork` is honest here: "compensating cleanup, not a
true transaction… any failure after step 1 triggers a best-effort rollback… a rollback
that itself fails **names the leftover and exits non-zero**." §4b's `create-fork`
carries **no such clause.** §11 mutters "reuses `run_fork`'s core" — a hint, not a
specification, and the normative §4b never names the rollback, the leftover, or a
refusal. §12 tests only the *wish*: "a forced provision/stamp failure → non-zero → no
worktree" (`design.md:525`) — asserting the outcome the mechanism does not deliver.

**Risk.** "Fail-closed by construction" is the load-bearing boast of the entire pivot
(§4b, §10, G3). It is **half-true**: fail-closed against the *worker spawning* (Claude
honours the non-zero exit), fail-**open** against *disk hygiene* (the orphan worktree +
branch survive). Worse, an orphaned linked worktree at the dispatch path is exactly the
substrate later verbs trip over — `land`'s `worktree-gone`, `gc`'s idempotence — now
seeded with debris no verb owns. The redesign reintroduces the round‑1→6 disease: a new
mechanism admitted without its lifecycle, leaking at the seam.

**Sentencing.** §4b must specify `create-fork`'s compensating cleanup as §4a's equal:
on any failure after `git worktree add`, roll back (`git worktree remove --force`,
`git branch -D`, reap the dir) **before** the non‑zero exit; a rollback that half-fails
names the leftover and exits non‑zero with a distinct token. Name the bad-state
refusals (Q8). §12 must test that the orphan is *gone*, not merely that the worker did
not spawn. Until create-fork answers Q3/Q4/Q8/Q9, by the design's own rule it is
**rejected, not shipped with a gap.**

### Charge σ (sigma) — HIGH — "replicate default creation" for every non-dispatch worktree is hand-waved, with a blast radius of *all* isolated subagents

**Doctrine violated.** §1 thesis (mechanism is specified, golden-testable, not prose);
the admission rule Q9 (a golden/spike proves the path). ADR-001 leaf discipline implies
a *specified* behaviour, not a verbal gesture.

**Evidence.** §4b confesses the hook "**replaces default creation for *every*
worktree** (incl. `--worktree` launches and benign isolation:worktree subagents)" and
therefore "must **replicate default creation for non-`dispatch-worker` agent_types**."
The else-branch then reads, in full: "`replicate default creation, print path, NO
marker`." **What** default creation *is* — Claude's branch-naming convention
(`worktree-agent-<agentId>`), the gitdir layout (`.git/worktrees/agent-<agentId>`), the
path (`.claude/worktrees/agent-<agentId>`), all confessed under cross-examination in
`mem.pattern.dispatch.claude-subagentstart-worker-identity` — is **nowhere specified**,
and no equivalence oracle pins the replica to Claude's native act. §12's create-fork
gate tests only "other agent_type → replicate-default creation, path printed, **no
marker**" — fidelity to the replaced default is unverified.

**Risk.** This hook now sits astride **every** `isolation: worktree` spawn in the repo,
not merely dispatch. A defect in the replicate-default path — a wrong branch name, a
malformed gitdir — breaks *all* isolated subagents and `--worktree` launches, a
blast radius vastly exceeding SL-056's stated surface, and an undocumented,
version-fragile reimplementation of Claude's internals. The matcher escape hatch (scope
the hook to `dispatch-worker`) is named as "a spike sub-question" — i.e. the only thing
that *removes* this hazard is itself unconfirmed.

**Sentencing.** Make the matcher question a **gating** spike outcome, not a sub-question:
if a WorktreeCreate matcher can scope the hook to `dispatch-worker`, the replicate path
is deleted and this charge evaporates. If it cannot, the else-branch must be specified as
a **serviceable** default — doctrine's own naming/path conventions, sufficient for
general isolated-subagent duty; it **need not mirror Claude's native layout** (User
adjudication: Claude's default is not the bar — "suffice for general duty" is) — with a
golden asserting the produced worktree is valid + usable (and bears no marker). §10/G3
must confess the hook now **intermediates** creation for **all** isolated subagents (the
blast radius is real even though the contract is small).

### Charge τ (tau) — HIGH — the `dispatch-worker` discriminator is replicated across three surfaces with no single source of truth; drift fails *open*

**Doctrine violated.** "Naming things well is VERY important"; obsess over coupling
(global standards). The admission rule Q7 (can a worker mutate/escape the guard) and the
§1 fail-closed thesis. A guard whose key is a free-floating string literal in three
files is a guard with three latent holes.

**Evidence.** The literal `"dispatch-worker"` is the load-bearing discriminator and is
written, independently, in: the `Agent`-tool `subagent_type` (§4b spawn contract); the
hook gate `if payload.agent_type == "dispatch-worker"` (§4b/§9); the installed agent
definition `install/agents/claude/dispatch-worker.md` (§9/§11); and the `/dispatch-agent`
router/skill surface (§11). The design names the coupling nowhere as a single constant
and ships **no cross-surface test** that the four uses agree.

**Risk — fail-open, the cardinal sin.** Drift any one literal and the hook's gate misses:
`create-fork` falls to the **else** branch, replicates default creation, **writes no
marker.** The worker is then born in a linked worktree with `marker_present == false` ⇒
`worker_mode(root) == false` ⇒ **writes are not refused.** A dispatch worker that should
be branded and fenced is instead an unbranded free-writer on a harness with no env leg
and no bwrap (§4c) — the guard inverts from fail-closed to fail-open on a one-character
typo, silently. This is the precise inversion the slice exists to *prevent*.

**Sentencing.** Declare one source of truth — a `const DISPATCH_WORKER_AGENT_TYPE` in
the binary — consumed by `classify_create`, and pin the installed agent def's `name` and
the skill's `subagent_type` to it with a golden/test asserting all surfaces resolve to
the same value. Drift must be a **red test**, not a silent fail-open.

### Charge υ (upsilon) — MEDIUM — "concurrent file-disjoint claude dispatch is first-class" is sold louder than the v1 funnel can land

**Doctrine violated.** Altitude honesty (§10 mandate; the slice's repeated insistence
that degradations are *confessed, not closed*). The admission rule Q10 (the governance
claim must be scoped honestly).

**Evidence.** §4b markets the redesign's headline: "**concurrent file-disjoint claude
dispatch is first-class**… the funnel-back is still serialized by `import`'s
stationary-head precond." §10/G3 brands claude "**concurrent-safe**." But §7a `import`
demands `HEAD == B` (else `head-moved`), and §7a step 4 has the orchestrator commit
**separately** after each import. Trace a concurrent batch: import worker A → orchestrator
commits → `HEAD` moves `B → B+1`. Import worker B, **also forked at B**, now reads
`HEAD(B+1) != B` → hard refusal `head-moved`. The in-verb re-anchor onto a moved HEAD is
**deferred** (§13 OQ‑1 / IMP‑043). So v1 does not *serialize a batch to completion* — it
lands the **first** worker and forces every sibling to re-dispatch from the bumped base.
§7c discusses this hazard only for **external** committers ("livelock"); it is silent
that the orchestrator's **own** sequential imports trigger the identical invalidation
within a single concurrent batch.

**Risk.** A reader provisions N concurrent file-disjoint workers expecting N landings and
gets one, plus N‑1 re-dispatches — the concurrency payoff the §4b headline promises is,
in v1, **execution-only**. "Serialized" connotes an orderly drain; the reality is
first-wins-rest-reanchor, and reanchor is not in scope. The mechanism is sound; the
*marketing* over-reaches — the exact altitude-honesty sin the slice swore off.

**Sentencing.** Scope the claim honestly in §4b and §10: concurrent *execution* is
first-class; v1 *funnels* one landing per base, every sibling re-dispatching onto the
bumped HEAD until the IMP‑043 in-verb re-anchor (or a single multi-fork import) lands.
State plainly that v1 concurrent dispatch buys parallel execution, **not** parallel
landing.

### Charge φ (phi) — MEDIUM — §10/G3 asserts claude "fail-closed, first-class" as settled governance while the property is admittedly O3-spike-contingent

**Doctrine violated.** Admission rule Q10 (claim scoped honestly); the boot storage rule
spirit (no claim asserted above its evidence); G3's own caveat that "env/spike claims
stay `proposed` until the O3 gate is green."

**Evidence.** §4b is candid: "**the decision hinges on one probe, and it conflicts with
our own**" — the prior probe saw `name`, **not** `agent_type`, on WorktreeCreate (albeit
on an *unnamed* subagent, so absence is expected, not refutation). If the named-subagent
`agent_type` propagation fails, the entire claude path drops to the **SubagentStart-stamp
fallback**, which §4b itself stamps with a "**fail-open created-but-unstamped window**."
Yet §10/G3 presents claude as "a **first-class** backend, not a degraded rung;
**fail-closed** — no worktree without a marker." Those are **post-spike** properties.
The same G3 paragraph's later clause concedes "O3-spike-contingent" and "claims stay
`proposed` until the O3 gate is green" — so the table's headline and its footnote
**contradict on altitude**.

**Risk.** A governance artifact (ADR-011 draft, G3) that states a fail-closed altitude
as *accepted* when it is *proposed-pending-spike* will be read as canon by the next
agent. If the spike reds, the achievable claude altitude is the fail-open SubagentStart
window — and the table never showed that row.

**Sentencing.** Make the §10/G3 claude altitude two-valued and explicit: **on O3 green**
— fail-closed via WorktreeCreate; **on O3 red** — fail-open SubagentStart window →
prompt-enforced. Mark the fail-closed cell `proposed` until the gate is green, matching
G3's own "claims stay `proposed`" clause. Do not let the headline outrun the footnote.

### Charge χ (chi) — MEDIUM — §12 verifies every verb except the `claude install` surface that makes the whole claude path real

**Doctrine violated.** §1 thesis (golden-testable mechanism); task-completion checklist
(decent test coverage). The handover names "the agents leg and the hook leg [as] the
load-bearing parts."

**Evidence.** §9 introduces a substantial install surface: rename `skills install →
claude install`; a **hidden deprecated alias**; the **agents** leg (symlink
`install/agents/claude/*.md` → `.claude/agents/`); the **WorktreeCreate hook merge** into
`.claude/settings.local.json` via the `HookSpec` merge core. §11 lists three install-
related code rows. §12 — the entire Verification section — contains **not one bullet**
for any of it: no proof the alias dispatches the same handler, no proof the agent file
lands, no proof the hook merges *without clobbering* existing hooks, no idempotent-
reinstall check, no golden on the renamed surface (despite §11 promising "update goldens").

**Risk.** If this surface is wrong, the claude path is **never wired** — the agent def is
absent, the hook never fires, and the marker is never stamped, silently. The
fail-closed/fail-open question is moot if the gun was never loaded. An unverified
load-bearing installer is a heresy of omission.

**Sentencing.** Add §12 bullets: alias→same-handler golden; agent-def symlink presence;
hook-merge into `settings.local.json` that **preserves** pre-existing hooks (reuse the
boot `HookSpec` merge tests as prior art); idempotent reinstall; the audit-label/golden
rename. The install surface is mechanism — it must be golden-pinned like the verbs.

### Charge ψ (psi) — MEDIUM — `create-fork` is a stdin-JSON trust boundary that names no malformed/missing-payload refusal

**Doctrine violated.** Admission rule Q8 (what refusal names its bad states); silent-
error-handling is a named mortal sin of `/canon`. The hook is harness-facing untrusted
input.

**Evidence.** §9/§4b/§12 specify `classify_create(payload) -> ForkWorker | PlainCreate`
and exercise only the **success** branches (`dispatch-worker` → stamp; else → replicate).
The hook's whole control flow parses stdin JSON and derives `<dir>` from
`payload.cwd` (§4b "create-fork privilege"; SR‑4). Nowhere is the **bad payload** named:
malformed JSON, missing `agent_type`, missing/empty `cwd`, a `<dir>` that fails to derive
or escapes the repo. A `classify_create` over a partial type that silently falls to
`PlainCreate` on a *missing* `agent_type` is the same fail-open as Charge τ, reached by a
different road.

**Risk.** A trust boundary that classifies malformed input as "benign, replicate
default, no marker" turns every payload defect into a silent unbranded worker. Silent
on the bad path is fail-open on the guard.

**Sentencing.** Name the refusals: malformed/missing payload, underivable/escaping
`<dir>` → distinct non‑zero exits that **fail creation** (fail-closed — a worker the hook
cannot classify must not be born), each with a golden. Missing `agent_type` must **not**
silently mean "benign"; it means "cannot classify → refuse."

---

## Questions

1. When `create-fork`'s `git worktree add` succeeds but `provision`/`write_marker`
   fails, what removes the orphan worktree and branch before the non‑zero exit — and what
   token names a rollback that itself fails? (Charge ρ)
2. Can a WorktreeCreate **matcher** scope the hook to `dispatch-worker`? If not, what is
   the exact replicate-default contract and its fidelity oracle? (Charge σ)
3. Is `"dispatch-worker"` a single binary constant the agent def and skill are pinned to,
   or four independent literals? Where is the cross-surface drift test? (Charge τ)
4. In v1, can a concurrent file-disjoint batch *land* more than one worker without
   IMP‑043 re-anchor, or does every sibling after the first re-dispatch? (Charge υ)
5. Does §10/G3 assert claude "fail-closed" as accepted or as `proposed`-pending-O3? Show
   the O3‑red altitude row. (Charge φ)
6. Where does §12 prove the agent-def symlink lands, the alias dispatches the same
   handler, and the WorktreeCreate hook merges without clobbering existing hooks? (Charge χ)
7. What refusal names a malformed/`agent_type`-missing/`cwd`-missing `create-fork`
   payload? (Charge ψ)

## Pronounce Judgement

**This is heresy** — not of the topology, which seven prior fires have tempered, but of
the **graft**. The `WorktreeCreate`-hook pivot is sound *as a direction* and faithfully
tracks the empirical hook memories. But the new load-bearing mechanism, `create-fork`,
was **admitted without answering the design's own ten-question admission rule** — the
very discipline minted to end the round‑1→6 cycle of "a remediation leaks at the seam the
next round." Q3/Q4/Q8/Q9 go unanswered for the orphan-worktree path (ρ); the discriminator
is a three-surface fail-open drift trap (τ); the replicate-default path is hand-waved over
a repo-wide blast radius (σ); the trust-boundary payload has no bad-state refusal (ψ); and
two governance claims — concurrent-first-class (υ) and fail-closed-first-class (φ) —
outrun the v1 funnel and the unrun O3 spike respectively. The install surface that makes
any of it real is unverified (χ).

Four HIGH, three MEDIUM, no CRITICAL. The bones are clean; the new limb bleeds at every
joint the admission rule warned of. **`Nihil obstat` is DENIED.** The pivot must answer
its own rule before `/plan`.

## Sentencing

In order, each with its verification and its historically fitting penance:

1. **Charge ρ first.** Specify `create-fork`'s compensating cleanup as §4a's equal —
   rollback on post-`add` failure, leftover named, distinct non‑zero token. Verify the
   orphan is reaped, not merely that the worker did not spawn. *Penance: the offending
   pseudocode shall be read aloud and then broken upon the wheel, joint by joint, until it
   confesses its rollback owner.*
2. **Charge τ.** One binary constant `DISPATCH_WORKER_AGENT_TYPE`; the agent def and skill
   pinned to it; a drift test that reds on mismatch. *Penance: the four scattered literals
   shall be gathered and burnt as a single faggot, that one truth replace the three.*
3. **Charge σ.** Promote the matcher to a gating spike outcome; absent it, specify and
   golden a **serviceable** default-creation else-branch (own conventions, *not* a
   Claude-fidelity mirror — User adjudication); confess the all-subagent intermediation
   in §10/G3.
   *Penance: the hand-wave shall be staked at the crossroads as warning to all who would
   write "replicate default creation" and specify nothing.*
4. **Charge ψ.** Name `create-fork`'s malformed/missing-payload refusals; missing
   `agent_type` refuses, never silently "benign." Golden each. *Penance: the silent path
   shall be racked until it screams its bad states aloud.*
5. **Charge υ.** Re-scope the concurrency claim: parallel *execution*, not parallel
   *landing*, in v1. *Penance: recant the over-loud boast in sackcloth.*
6. **Charge φ.** Two-value the §10/G3 claude altitude; mark fail-closed `proposed` until
   O3 greens. *Penance: the claim shall wear the `proposed` brand until the spike absolves
   it.*
7. **Charge χ.** Add §12 install-surface verification (alias, agent symlink, hook merge
   without clobber, idempotent reinstall). *Penance: the unverified installer shall be
   tested sevenfold, once for each fire it escaped.*

After remediation in `design.md`, re-lock and convene a **ninth** confirmatory inquisition
before `/plan` — the HIGH charges forbid passage. Let no `/plan` be uttered while the
admission rule stands unanswered.

> **HERESIS URITOR; DOCTRINA MANET**
