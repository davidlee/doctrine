# Subagent-as-orchestrator: feasibility probe + design finding

<!-- Authored 2026-07-04. Empirical probe (Claude Code 2.1.198) answering "can a
     Claude subagent drive the dispatch funnel?" Companion to RFC-005 (§1, the
     SL-181 positive-coordination-marker seam) and RFC-011 (orchestration cost).
     Method + evidence are live-witnessed, not doc-trusted (H0 discipline). -->

**Status:** research note. **Verdict:** the harness no longer blocks it; **our own
SL-182 confinement wall does**, and the wall *cannot currently tell an orchestrator
subagent from a worker*. One targeted change — a forge-proof coordinator exemption
(the SL-181 seam) — unlocks it.

## The question

Historically the dispatch orchestrator must be the **main thread**: it owns the
Agent-tool spawn, imports worker deltas, and is the sole writer of `.doctrine/` +
trunk. "Subagents can't orchestrate" was taken as a harness fact (ADR-011 D3: *"for
claude the only viable backend is the in-session Agent tool"* — read as main-thread-
only). If a **subagent** could orchestrate, orchestration context moves off the main
thread and hierarchical/parallel dispatch becomes possible (RFC-011 lever).

## What actually blocks it (two facts, both re-tested live)

### Fact 1 — the harness stopped blocking nesting (v2.1.172)

`docs/claude/subagents.md:786`: *"As of Claude Code v2.1.172, a subagent can spawn
its own subagents."* Depth ceiling **5** (`:790`). A subagent spawns nested
subagents **iff its agent definition lists `Agent` in `tools`** (`:374`/`:376`; the
type-allowlist in parens is ignored at subagent level). `dispatch-worker`'s tools
are `Read, Edit, Write, Bash, Grep, Glob` — **no `Agent`**; that omission (not the
harness) is why *that* subagent can't orchestrate. The orchestrator must be a
**different** subagent type that lists `Agent`.

### Fact 2 — the SL-182 wall keys on "is-a-subagent", not "is-a-worker" (the real blocker)

`src/worktree/jail.rs::resolve_target` (the `PreToolUse` jail core):

```
agent_id = None                         → Orchestrator (unconfined)
agent_id = Some, cwd is a linked wt     → Jail(cwd)      (rw ONLY cwd; shared .git RO)
agent_id = Some, cwd not a project wt   → Reject         (deny every tool)
```

Every subagent carries an `agent_id`, so **every** subagent is either jailed-to-cwd
or rejected. The orchestrator role needs unconfined rw to the **shared `.git`**
(commit / integrate / `worktree remove`) and to **`.doctrine/`** state — both live
*outside* any single cwd. So a subagent orchestrator is blocked in both branches.
This is deliberate (SL-182 defence-in-depth: the worker marker can fail-stamp — D6,
`SubagentStart` is read-only — so the wall trusts *nothing* and confines every
subagent-in-a-worktree).

## Probe method (Claude Code 2.1.198, primary tree on `edge`)

Installed throwaway observability hooks (`SubagentStart`/`SubagentStop`/
`WorktreeCreate`/`PreToolUse`, pass-through: log stdin, `exit 0`, emit nothing) in
`.claude/settings.local.json` — **confirmed they hot-reload** via a main-thread
canary. Then spawned a depth-1 `general-purpose` subagent (has `Agent`+`Bash`) that
self-reported and spawned a depth-2 `isolation: worktree` subagent, which spawned a
depth-3. Read back the captured hook payloads + the agents' self-reports. Worktrees
+ hooks torn down after.

## Evidence

**Wall keying (from captured `PreToolUse` payloads + live denials):**

| Actor | `agent_id` | cwd | verdict (witnessed) |
|---|---|---|---|
| main thread | *absent* | primary | unconfined (canary Bash ran) |
| depth-1 orchestrator | present | **primary** (inherited from spawner) | **`worktree-jail: cwd-not-a-worktree`** — Bash **and** Write both dead |
| depth-2 worker | present | linked wt | **`Jail(cwd)`** — own cwd rw; **primary + shared `.git` = RO** (`D2_PRIMARY=FAIL`, `D2_GIT=FAIL`, "Read-only file system") |

- **Nesting is live to at least depth-3.** The `Agent` tool is present at every
  level (`general-purpose` = `*`).
- **`isolation: worktree` works at depth-2**: landed a linked worktree
  (`.worktrees/agent-…`, git-dir ≠ common-dir), **detached HEAD** at the spawner's
  HEAD (`baseRef:"head"` → forked primary HEAD `12ca92ad`). Return footer:
  `worktreePath: …`, `worktreeBranch: undefined` (matches the dispatch-agent skill).
- **`SubagentStart` fired once per spawn at every depth (3×).** Payload carries
  `agent_id`, `agent_type`, `cwd` — but **`parent_agent_id: null`, `depth: null`**
  (harness exposes neither parent linkage nor depth).
- **`WorktreeCreate` fired for the nested isolated spawn**, payload **thin**:
  `{session_id, transcript_path, cwd, prompt_id, hook_event_name, name}` — **no
  `agent_type`, no base, no path, no `agent_id`** (the ADR-011 D6 σ blast-radius,
  now witnessed: it fires globally and can neither classify nor base-pin a worker).
- **`PreToolUse` payload has `agent_id` + `cwd` + `tool_name` but NOT `agent_type`.**
  So the wall physically *cannot* distinguish an orchestrator subagent from a worker
  at deny time — the root of Fact 2.
- **Confinement is a hook deny, un-widenable by permission mode:**
  `dangerouslyDisableSandbox: true` did **not** bypass the `worktree-jail` deny, and
  `cd` did not move it (it keys off the agent's assigned cwd, not the shell's).
  Confirms the expectation "permissions cascade, cannot widen" — but note the
  mechanism is the *hook*, orthogonal to permission mode, not the permission cascade
  itself.

**cwd nuance (as predicted).** A subagent's cwd is **inherited from the spawner's
cwd at spawn time** (depth-2/3 both got depth-2's worktree cwd; depth-1 got primary).
`isolation: worktree` *overrides* it to a fresh linked worktree. So an orchestrator
subagent spawned from the primary tree is **rejected outright** ("cwd-not-a-worktree")
— it must be spawned with its cwd inside a linked worktree (the coordination tree) to
even reach the `Jail` branch. And `Jail` still RO-mounts shared `.git`, so it still
can't commit. Both branches dead without an exemption.

## The unlock: a forge-proof coordinator exemption (SL-181 seam)

Add one branch to `resolve_target`:

```
agent_id = Some, coordinator-marker present  → Orchestrator (unconfined)
... else as today (Jail / Reject)             ← default stays fail-closed
```

**Forge constraint (load-bearing).** A jailed worker *can* write inside its own cwd
(witnessed: `D2_OWN=OK`), so the coordinator marker must live **where a jailed
subagent cannot write it** — the shared `.git`-adjacent store / primary tree, not the
worktree-internal `.doctrine/state/dispatch/worker` path. Only an already-unconfined
actor (main thread, or the `SubagentStart` hook, which runs unconfined) can stamp it
there → unforgeable. Natural implementation: a **matcher-scoped `SubagentStart`
hook** stamps `coordinator` when `agent_type` is the orchestrator type, `worker`
otherwise (SubagentStart *does* carry `agent_type`; PreToolUse does not — so role
resolution must happen at SubagentStart and be recorded for PreToolUse to read).
Default remains confine-all-subagents; only the explicit stamp lifts it, preserving
D6's fail-closed invariant.

This is exactly **SL-181** ("the positive coordination marker", RFC-005 §1,
`design`). This probe supplies its motivating case and its keying constraints.

## What each arm then looks like (both need the exemption)

- **Agent arm** — orchestrator subagent (with `Agent` in tools, unconfined via the
  marker) runs `dispatch arm-spawn` + spawns `dispatch-worker` subagents nested.
  Depth budget: main(0) → orch(1) → worker(2), ceiling 5 (room for hierarchical
  orchestration). All in-harness, subscription-billed.
- **Subprocess arm** — orchestrator subagent (Bash, unconfined) spawns pi/codex
  subprocess workers via `pi-spawn-confined.sh`. Note: even here the exemption is
  required — the orchestrator subagent's *own* `git commit` into the coord tree is
  walled without it.

## Value + honest caveat (RFC-011)

Value: orchestration context leaves the main thread → main stays a thin, cache-stable
router; the ~40–49k/phase funnel ceremony burns a **disposable** subagent context
(RFC-011's cold-small-context thesis applied to the orchestrator itself). Unlocks
**parallel orchestration** — N orchestrator subagents driving N slices, each
returning a compact summary. Caveat: per RFC-011's corrected cost model you do **not
save** orchestration tokens — you **relocate** them and buy parallelism +
main-context preservation. That is the lever, not a raw cut.

## Residual / open design questions

1. **Where the resolved role is recorded** so `PreToolUse` (no `agent_type` in
   payload) can read it — a per-agent_id file the `SubagentStart` hook writes to the
   trusted store, keyed by `agent_id` (which PreToolUse *does* carry).
2. **Marker lifecycle / teardown** — no `WorktreeRemove`/reliable end signal; the
   coordinator stamp needs a GC story (stale-coordinator-marker risk mirrors the
   worker-marker one).
3. **Nested worker confinement still holds** — a worker spawned *by* an orchestrator
   subagent must still hit `Jail(cwd)`. Witnessed true at depth-2 (wall keys on
   agent_id+cwd, depth-agnostic), so the exemption must be *narrow* (coordinator
   marker only) or a nested worker would inherit the exemption. This is why keying on
   a stamped role, not on "spawned-by-a-coordinator", matters.
4. **`WorktreeCreate` cannot base-pin** (thin payload) — the current claude arm's
   positional `arm-spawn` discriminator is unaffected (it keys on cwd == spawn dir),
   and an orchestrator subagent can run `arm-spawn` once unconfined; confirm the
   discriminator still fires when the spawner is a subagent (untested — the probe used
   raw `isolation:worktree`, not the doctrine `create-fork` hook, which is deferred
   per ADR-011 D6).
