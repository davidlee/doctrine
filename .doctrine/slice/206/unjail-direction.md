# SL-206 — Direction: Agent-orchestrated driver + orchestrator-unjail nomination

> **Status: research direction, PRE-DESIGN, POC-gated.** Authored after PHASE-07
> proved the shipped workflow-spawn model (`design.md` §5.1/§5.4) structurally
> cannot drive committing dispatch in the jail. This doc records the reasoning
> chain + evidence so a POC can validate the direction before `design.md` is
> re-opened. It does **not** re-lock the design — that waits on the POC and a
> `/design` + adversarial pass (§7 below). Companion to `notes.md` FINDING 4/5.

---

## 1. What PHASE-07 killed

`design.md` §5.1 has the **Workflow script spawn a confined `dispatch-orchestrator`
leaf**, which arms a base, spawns a worker, imports, concludes. PHASE-07 drove it
live and it never completes a phase. Root cause is **not** placement (PHASE-06's
fix) or git permission — it is that a **Workflow `agent()` leaf hits three
independent structural walls**, any one fatal:

| # | Wall | Evidence |
|---|---|---|
| 1 | **No `Agent` tool** — a workflow leaf cannot nest-spawn a child, *regardless of its def's `tools:`* (the runtime strips it unconditionally; even a `Tools:*` def loses it). So it cannot be an orchestrator that spawns workers. | `mem.fact.claude.workflow-strips-agent-tool` (mem_019f36028bca); probes `wf_40e8c2c3-3fe` (workflow orch: no Agent) vs main-thread `ae3a5dd36e3ea8c41` (Agent present) |
| 2 | **Cannot commit** — the `isolation:worktree` fork is a *linked worktree on the RO shared jail `.git`*; plain `git add`/`commit` fails `index.lock: Read-only file system`. And `worker_commit` refuses it: `run_worker_commit` → `resolve_agent` needs a registered `DispatchRecord` (provisioned by `dispatch setup`/`create-fork`), which a harness-minted fork is **not** → `unknown-agent`. | `mem.fact.claude.workflow-leaf-cannot-commit` (mem_019f361983); probe `wf_01ed2bf2-ff8`; `src/mcp_server/worker_commit.rs:219-224`, test `worker_commit_unknown_agent_refuses` |
| 3 | **No-isolation is worse** — a no-iso leaf inherits the *primary* cwd; the `worktree-jail` PreToolUse hook denies **every** Bash: `cwd-not-a-worktree`. The gate keys on `agent_id` (`decide()`, `src/worktree/pretooluse.rs:129`): **no `agent_id`** (main thread) → `PassThrough`; **`agent_id` present** (any subagent) → confined. | mem_019f361983 (addendum); probe `wf_93413e70-159`; `src/worktree/jail.rs:94` |

**The one unconfined actor is the main thread** (no `agent_id` → `PassThrough`).
A **Workflow is never the main thread**; its leaves are always confined subagents.
So the "workflow-templated" deliverable premise is incompatible with driving
committing dispatch in this jail. `design.md` §1's RFC-011 lever (live `budget`
metering across the spawned tree) was the motivation — it does not survive contact
with the confinement walls.

## 2. The proven committing path (what the claude arm actually does)

The shipped `/dispatch` claude arm commits because it uses exactly what a workflow
denies:
- a **main-thread** (PassThrough, unconfined) orchestrator — sole writer, RW `.git`;
- **worker subagents** spawned via the `Agent` tool at `isolation:worktree`
  (mem_019f2d4d — omitting isolation runs *in* the coord tree, no fork);
- workers commit via the **server-side, unconfined `worker_commit`** bypass into a
  **registered `DispatchRecord`** (marker stamped on the coord tree, mem_019ef99b).

## 3. The direction — A2: Agent-orchestrated driver + orchestrator-unjail nomination

Reframe the deliverable from *workflow-templated* to **Agent-orchestrated**. The
orchestrator is judgement (prep context, check delta, adapt-and-halt, land) — an
**agent**, not a JS script. Two shapes were on the table:

- **A1** — `/drive-slice` is a *skill* the main thread runs = essentially `/dispatch`
  packaged with typed `PhaseReceipt` + one-command drive. Zero new machinery; the
  delta over `/dispatch` is ergonomic, not mechanical.
- **A2** — the main thread spawns **one `dispatch-orchestrator` subagent** that
  drives the whole slice (the original "one spawn drives the slice" intent). This
  is the richer deliverable, but the orchestrator-as-subagent is **confined** by
  `worktree-jail` (wall #3 applies to it too — it has an `agent_id`).

**The unlock (operator's idea): nominate the orchestrator subagent as UNJAILED.**
Doctrine already runs a `SubagentStart` matcher scoped to the *worker* agent-type
that provisions+stamps (`install/dispatch-mechanics.md:204`, `dispatch-worker.md:30`,
`src/worktree.rs` `DISPATCH_WORKER_AGENT_TYPE`). The same seam, a different marker,
grants the orchestrator `PassThrough`:

```
main thread → Agent-tool spawn: dispatch-orchestrator subagent
  └─ SubagentStart(matcher = dispatch-orchestrator) FIRES  [top-level spawn]
       → write agent_id → orchestrator-allowlist  (FIXED path, see §4)
  └─ pretooluse decide(): agent_id ∈ allowlist → PassThrough  [NEW leg]
       → orchestrator UNJAILED: RW .git, retains Agent tool
  └─ orchestrator nest-spawns dispatch-worker(isolation:worktree) per phase
       → worker SubagentStart may NO-FIRE (nested session, §4.3) → unstamped
       → but orchestrator is UNJAILED → it hand-stamps / imports the worker's
         working-tree diff itself (pi-arm model) → commits → concludes → reports
```

Every step is a proven component. The nomination's payoff: a **subagent**
orchestrator behaves like the **main-thread** one (RW `.git`, full git, hand-stamp,
direct diff-import) — dropping SL-199 Mode B's "route every coord write through
server-side MCP" constraint. It also closes wall #2 for workers: an unjailed
orchestrator either post-hoc stamps them into `DispatchRecord`s (so `worker_commit`
resolves) **or** imports their uncommitted working-tree diffs directly and commits
them itself (no `worker_commit` needed — the shipped **pi-arm** model).

## 4. Corrected harness facts (the docs lie — verify empirically)

The published hook docs (`docs/claude/hooks.md`) misstate three things the memories
correct. The nomination rests on the corrected facts, not the docs.

### 4.1 SubagentStart FIRES and is matchable — CONFIRMED
For an **Agent-tool** `isolation:worktree` spawn: the hook fires, the
`matcher: <agent_type>` matches on the payload, payload carries `agent_id` +
`agent_type` + `cwd`. Proven — IMP-046 fresh-session probe. `mem_019ee3a0`.

### 4.2 It is SYNC-BLOCKING — the write lands before the first command
A `SubagentStart` **command** hook gates the subagent until the hook process exits
(proven by sleep-scaling: worker-start lag tracks hook duration 1:1). So the
allowlist entry is **mechanically guaranteed present before the orchestrator's
first `PreToolUse`** — on hook success. `mem_019ec0a5` (verified).

### 4.3 The three lies
1. **Hook process `cwd` = the SPAWNED agent's own worktree**, not the orchestrator
   tree. Silently broke doctrine's auto-stamp (`run_stamp_subagent` resolved the
   source from process cwd). ⇒ **the allowlist write MUST target a FIXED path**
   (`$CLAUDE_PROJECT_DIR`-resolved — present in hook env, `mem_019f1b76`), never
   cwd-relative. `mem_019ee3a0`.
2. **`WorktreeCreate` payload has NO `agent_type`, NO path** — cannot discriminate
   an orchestrator there; `SubagentStart` is the only type-scopable seam.
   `mem_019ec093`.
3. **SubagentStart SILENTLY NO-FIRES when the spawning session is itself a
   nested/child Claude session** (`CLAUDE_CODE_CHILD_SESSION=1`). Undocumented —
   the load-bearing lie. `mem_019ec84b`.

### 4.4 Why the lies don't break A2 (but do bury D')
- Nomination targets the **orchestrator**, spawned by the **top-level main thread**
  (not nested) → SubagentStart **fires** (4.1) → nomination works. ✓
- The nested-session no-fire (4.3-3) hits the **workers** the orchestrator then
  spawns — but workers **don't need nomination** (they stay jailed, correct); they
  need *stamping*, and the **unjailed orchestrator** hand-stamps them post-spawn
  (the proven nested-session workaround, `mem_019ec84b` "how to apply"). ✓
- **Fail direction is SAFE.** SubagentStart cannot be made fail-closed (exit 2
  doesn't abort — `mem_019ec0a5`). For *unjailing*, a hook miss means the
  orchestrator **stays jailed** — a visible functional failure, never an escape.
  Absence of the allowlist entry never grants `PassThrough`.

## 5. Security analysis (this is a confinement-boundary change — ADR-008)

Adding an unjail-allowlist to `pretooluse`/`jail` modifies the **security boundary**
SL-182 built. It holds, but deserves real rigor (see §7):

- **Authorization = three gates.** (a) the `agent_type` matcher; (b) `agent_type` ==
  the def's committed frontmatter `name`, and `.claude/agents/` is worker-unwritable
  (jailed + import R-5 belt) — the name cannot be forged; (c) only `Agent`-tool
  holders spawn (main thread + orchestrators) — **workers cannot spawn at all**, so
  a worker cannot nominate anything.
- **Consistent with CHR-039 / ADR-006.** The orchestrator is unconfined *by design*
  (sole writer). Nomination relocates the main thread's existing unjailed status to
  a trusted-def subagent; it does not invent a new trust.
- **Hygiene.** A `SubagentStop` matcher must remove the allowlist entry so a stale
  `agent_id` cannot be reused. `SubagentStop` fires awaited + tree-intact
  (`mem_019f1acb`).
- **Fail-safe** as in §4.4 — every failure mode lands on *more* confinement.

## 6. The POC (operator to run) — settles the direction empirically

Two probes. **P1 is the gate for A2**; P2 is the optional check on whether the
workflow form (D') can be revived after all.

### P1 — orchestrator-unjail round-trip (Agent-tool path)
1. Add a temporary `SubagentStart` matcher (`dispatch-orchestrator`) whose command
   writes `agent_id` to a fixed allowlist path under `$CLAUDE_PROJECT_DIR/.doctrine/state/`.
2. Add the `pretooluse` `PassThrough`-if-`agent_id ∈ allowlist` leg (behind the
   temporary allowlist; do NOT ship yet).
3. Main-thread Agent-tool-spawn a `dispatch-orchestrator` subagent; have it run
   `git rev-parse HEAD` + a throwaway commit in the coord tree.
4. **PASS** = the orchestrator's Bash is NOT denied (PassThrough), `.git` is RW, the
   commit lands. **FAIL** = still `worktree-jail: cwd-not-a-worktree` / RO `.git`.
   Confirms (or refutes) that nomination flips a subagent to unjailed.

> **▶ P1 RESULT — RAN 2026-07-06, PASS (both halves).** The nomination + unjail
> mechanism works end-to-end. Rig: POC leg in `pretooluse.rs` (`agent_id ∈
> $CLAUDE_PROJECT_DIR/.doctrine/state/orch-allowlist.txt ⇒ PassThrough`, fail-safe),
> a `SubagentStart(dispatch-orchestrator)` hook writing the allowlist, both built
> into the live `DOCTRINE_BIN` (SL-206-bin) and torn down after. Preserved:
> `scratchpad/p1-pretooluse.patch`, `p1-orch-nominate.sh`, `p1-evidence-*.log`.
> - **P1a (hook boundary, deterministic).** Synthetic jailed-subagent payload through
>   the real binary: allowlist absent → `deny: cwd-not-a-worktree`; same `agent_id`
>   listed → **empty = PassThrough**; a *different* `agent_id`, same cwd → `deny`.
>   Membership flips confinement, and only for the listed id.
> - **P1b (live orchestrator, end-to-end).** Main-thread `Agent`-tool spawn of a
>   `dispatch-orchestrator` (agent `a418b827a5f402ec4`). `SubagentStart` **fired**,
>   matched, wrote the allowlist **before** the first Bash (sync-blocking confirmed:
>   3× `NOMINATED-passthrough`). The orchestrator's Bash ran **unwrapped**:
>   `git rev-parse HEAD`✓, coord-tree read `git -C .dispatch/SL-206 rev-parse` →
>   `9ba5c0a6` (**a jailed agent cannot run this**), empty commit **landed** `98835cc`
>   in a scratch repo (RW `.git` under PassThrough). `RESULT: ALLOWED-and-committed`.
>
> **Mechanism confirmed.** A side-probe showed every `.git` (primary, coord
> `.dispatch/SL-206`, and a `.worktrees/` worker gitdir) is **RW from the main
> thread**. So the "RO shared `.git`" of §1 wall #2 is **not an fs permission** —
> it is the **bwrap wrap** the jail imposes on a *confined* subagent's Bash.
> `PassThrough` (main thread, or a nominated orchestrator) runs Bash **unwrapped**
> ⇒ RW `.git` everywhere. This is *why* nomination yields a committing orchestrator:
> flip to PassThrough ⇒ no wrap ⇒ RW `.git`. (Rig preserved in the session
> scratchpad — `p1-pretooluse.patch` / `p1-orch-nominate.sh` — reconstructable from
> this block if the scratchpad is gone.)
>
> **⚠ Correction to §4.3 lie #3.** This POC ran in a session with
> `CLAUDE_CODE_CHILD_SESSION=1`, yet `SubagentStart` **fired** for the orchestrator
> spawn. `mem_019ec84b`'s "silently no-fires in a child session" is therefore either
> stale or narrower than stated — the true condition is **not** "spawner is a child
> session." A child session's own main thread spawning via the `Agent` tool DOES get
> `SubagentStart`. (The no-fire may bite only spawns nested *below* that — untested
> here.) Also observed: the `settings.local.json` hook edit **hot-reloaded** (no
> restart). Net: A2's nomination is *more* robust than §4.4 assumed — the lie it was
> hedging against did not materialise for the orchestrator spawn. See `mem_019f3b…`
> (new finding). **This strengthens A2 and further weakens the need for D'.**

### P2 — is SubagentStart "less full of shit" inside a Workflow? (D' revival)
Replace the hook with a **logger shim** and run it under a **Workflow** `agent()`
spawn (both `isolation:worktree` and no-iso). In the *same* run, log **every** hook
payload the harness delivers for a workflow leaf — `SubagentStart`, `WorktreeCreate`,
`PreToolUse`, `SubagentStop`, `PostToolUse` — dumping full stdin JSON + `pwd` + a
real epoch timestamp.
- **Key question:** does `SubagentStart` fire for a workflow `agent()` leaf, carrying
  `agent_id` + `agent_type`? (Prior: **likely NO** — a workflow leaf is
  child-session-like, and §4.3-3 says nested sessions silently no-fire.)
- **If it FIRES** → D' (workflow script spawns a nominated-unjailed orchestrator-leaf
  + jailed worker-leaves; orchestrator-leaf imports worker diffs since it can't
  nest-spawn) becomes viable — revisit whether the workflow form is worth reviving
  for the RFC-011 `budget` lever.
- **If it NO-FIRES** → D' is dead for good; A2 is the sole path. Either outcome is
  decisive.

### ▶ P0 RESULT — nested-spawn probe — RAN 2026-07-06 (settles two questions)

Probe: main thread parked at coord `.dispatch/SL-209` (dispatch/209 @ `fa951846`)
`Agent`-spawned a **confined** `dispatch-orchestrator` (agent `add711b568b9d372e`,
no-iso ⇒ jailed at coord root); it nest-spawned one `dispatch-worker`
(`isolation:worktree`, `afd0c43fe5815021e`). Observer shims logged `SubagentStart`
+ `WorktreeCreate`. Rig preserved: `scratchpad/p0-*.log`, `p0-*.sh`. **No** allowlist /
pretooluse edit — natural confinement only.

- **Q1 — escalation reading: CONFIRMED LIVE.** `SubagentStart` **fired for the
  subagent-initiated** worker spawn (2 log entries, one `prompt_id`: orchestrator
  *and* worker). Payload carries **no spawner id** — a hook cannot tell main- from
  subagent-initiated. ⇒ a type-match-only nomination allowlist is escalatable
  (jailed `Agent`-holder → `Agent(subagent_type:"dispatch-orchestrator")` →
  auto-nominated grandchild). **The §5 arming token is now a ship-blocker for the
  unjail leg, not optional hardening.** ([[mem.fact.claude.subagentstart-fires-nested-no-parent-discriminator]].)
- **Q2 — confined-A2 viability: STRONGLY SUPPORTED.** The confined orchestrator:
  sealed out of shared `.git` (RO, `index.lock` EROFS) but **wrote its own coord
  `.doctrine/state/`** (arming feasible); its nested `isolation:worktree` spawn
  **fired `WorktreeCreate`** (provisioner runs) and **forked at the correct base**
  (`fa951846`); the worker's `worker_commit` **resolved + ran the full gate**,
  refusing only at `commit-gate-red` — a **pre-existing red `test` recipe on the
  SL-209 base** (exit 101), not a confinement failure. ⇒ the shipped worker
  machinery works from a *confined* orchestrator. ([[mem.fact.dispatch.confined-orchestrator-nested-worker-machinery-works]].)

**Net.** Confined-A2 looks viable → the unjail/nomination boundary-change may be
**unnecessary** (cheaper path, no ADR-008 amendment). If unjail is pursued anyway
(harness-surface minimization), the arming token is mandatory. Remaining confined-A2
unknown: the full funnel (`dispatch_import`→verify→conclude) via server-side MCP,
untested end-to-end but Mode-B-proven. Base-health aside: **dispatch/209 is red**
(test exit 101) — the SL-209 rig base needs a green before any real drive.

### ▶ P3 RESULT — PreToolUse(Agent) spawner-id probe — RAN 2026-07-06 (PASS)

Probe: observer matcher `Agent` on `PreToolUse`; nested spawn
`main → spawner(general-purpose) → leaf`. Rig preserved:
`scratchpad/p3-pretooluse-agent.{log,sh}`; torn down after (matcher + hook removed).

- **Main-thread Agent call → `agent_id="<NONE>"`** (outer entry), and
  **subagent-initiated call → `agent_id="a1904cf1da408e13e"`** (= the spawner's own
  id, inner entry). `tool_input.subagent_type` present in **both**. So the spawner
  identity that `SubagentStart` lacks (P0 Q1) **is present one seam earlier**, at
  `PreToolUse(Agent)`.
- **Held under `CLAUDE_CODE_CHILD_SESSION=1`** — the child-session env did NOT leak
  an `agent_id` onto the main-thread call. New `Agent` matcher **hot-reloaded**.
  ([[mem.fact.claude.pretooluse-agent-carries-spawner-id]].)

**Net — the arming token is no longer required.** §5's escalation hole (P0 Q1)
closes with a **PreToolUse(Agent) spawn-gate** ("Option A"): deny the spawn iff
`agent_id present ∧ caller ∉ allowlist ∧ subagent_type ∈ privileged-set`. This is
cheaper and stronger than the single-slot arming token (which only compensated for
`SubagentStart`'s missing parentage) — and it closes escalation at the source, so
`SubagentStart` type-match becomes trustworthy again. Same gate hardens the shipped
`dispatch-worker` provisioning matcher (escalatable today, bounded) and gives
IMP-269 `/fork` its discriminator. **Q2 (/fork PreToolUse behavior) un-probed —
informational, not load-bearing; Option A holds either way.**

### ▶ P4 RESULT — spawn-gate ACTIVE DENY, E2E — RAN 2026-07-06 (PASS)

P3 was observer-only; P4 ran an **active** deny gate (deny iff
`agent_id present ∧ subagent_type == dispatch-orchestrator`). Main thread spawned a
`general-purpose` attacker; it attempted two nested spawns. Gate log:

```
{agent_id:<NONE>,             target:general-purpose,      allow}   ← my spawn of attacker
{agent_id:a551f6df03874e447,  target:dispatch-orchestrator, DENY}   ← escalation attempt — BLOCKED
{agent_id:a551f6df03874e447,  target:general-purpose,       allow}  ← benign nested spawn — passed
```

Attacker self-report: `ATTEMPT-1: BLOCKED … escalation guard` / `ATTEMPT-2: SUCCESS: BENIGN-OK`.
The denied `dispatch-orchestrator` **never spawned**. Proves: (1) the deny
**prevents** the spawn (not a warning); (2) the gate is **scoped** — benign nested
spawns pass; (3) the legit main-thread nomination path is **not over-blocked**.
Rig preserved: `scratchpad/p4-spawn-gate.{log,sh}`; torn down after.
([[mem.fact.claude.pretooluse-agent-carries-spawner-id]].)

**Net — full-commit unjail is provably safe.** Closed loop, no token:
`SubagentStart(dispatch-orchestrator)` nominates PassThrough (P1) ∧
`PreToolUse(Agent)` deny-gate blocks privileged spawns from subagent callers (P4).
Only the main thread (no `agent_id`) or an allowlisted orchestrator reaches an
unjailed privileged spawn. The remaining unjail work is now ADR-008 amendment +
`/inquisition` + the (independent, composable) integration — not a novel safety
unknown.

## 7. Open, and what re-opens `design.md`

- **OQ-A2a — A1 vs A2.** Is the one-spawn subagent orchestrator (A2) worth the
  confinement-boundary change over the simpler skill form (A1 ≈ `/dispatch` packaged)?
  **P0 shifts this: a *confined* orchestrator drives the shipped worker machinery
  (Q2), so a confined-A2 needs NO boundary change at all — the unjail is only worth
  it for harness-surface minimization. **P3 update:** the unjail leg's escalation
  cost is a PreToolUse(Agent) spawn-gate, NOT an arming token (cheaper than thought,
  but still a boundary change confined-A2 avoids entirely).
  Leaning: confined-A2 first; unjail as banked fallback.**
- **OQ-A2b — ADR-008 amendment.** An unjail-allowlist is a new exception to
  orchestrator confinement. **P0 Q2 makes it likely avoidable** — confined-A2 keeps
  the boundary intact (no amendment). Amendment (+ `/inquisition`) is required ONLY
  if the unjail path is chosen; and that path additionally requires the §5
  escalation fix — a **PreToolUse(Agent) spawn-gate** (P3), superseding the arming
  token (P0 Q1: type-match alone is escalatable; P3: the spawner id is available at
  the Agent PreToolUse seam).
- **IMP-275** — in-workflow / in-orchestrator landing (audit/reconcile/close) stays
  deferred (reading (ii)); this direction is scoped to drive-to-Completed (reading (i)).
- **Re-open trigger:** P1 PASS → `/design` on A2 §5 (spawn model, nomination seam,
  security) with an adversarial pass; P1 FAIL → fall back to A1 and re-scope SL-206.
  **STATUS: P1 + P0 + P3 all RAN & PASSED (2026-07-06, §6 result blocks). Board:
  confined-A2 viable (P0 Q2) → the design fork is "confined-A2 (no boundary change)
  vs unjail (surface-min)". P3 retired the arming token — the unjail leg's
  escalation fix is a PreToolUse(Agent) spawn-gate. Operator chose (ii): run P3
  first (done), then `/design` re-open on §5 speccing the fallback security once
  (spawn-gate) + IMP-269. `/design` re-open now unblocked — proceeding.**

## 8. Evidence index

**Memories** — workflow walls: `mem_019f36028bca` (no Agent), `mem_019f361983`
(no commit + no-iso deny). SubagentStart/harness: `mem_019ee3a0` (fires; cwd=spawned
tree), `mem_019ec0a5` (sync-blocking; not fail-closeable), `mem_019ec84b` (nested
no-fire — the lie), `mem_019ec093` (WorktreeCreate no type/path), `mem_019f1b76`
(`CLAUDE_PROJECT_DIR` in hook env), `mem_019f1acb` (SubagentStop awaited).
Dispatch: `mem_019f2d4d` (worker needs isolation:worktree), `mem_019ef99b` (worker
marker on coord tree), `mem_019f331005` (fork arms at base), `mem_019f328b` (fork
reaches MCP).

**Probe run IDs** — `ae3a5dd36e3ea8c41` (main-thread orch HAS Agent), `wf_40e8c2c3-3fe`
(workflow orch no Agent), `wf_01ed2bf2-ff8` (iso leaf RO `.git`), `wf_93413e70-159`
(no-iso leaf worktree-jail deny).

**Source** — `src/mcp_server/worker_commit.rs:219-224` (`resolve_agent` needs a
`DispatchRecord`), `src/worktree/pretooluse.rs:129` (`decide` keys on `agent_id`),
`src/worktree/jail.rs:94` (`cwd-not-a-worktree`), `src/worktree.rs`
(`DISPATCH_WORKER_AGENT_TYPE`, `run_stamp_subagent`).

**Entities** — supersedes `design.md` §5.1/§5.4 workflow-spawn model (pending POC);
`notes.md` FINDING 4/5; deferred work `IMP-275`.
