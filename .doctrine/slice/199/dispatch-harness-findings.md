# SL-199 dispatch — empirical harness findings (session 2026-07-04)

Session notes from attempting to drive SL-199 PHASE-01 through the **claude arm**
(`/dispatch` → `/dispatch-agent`). Captured before compaction. Companion to the
RFC-011 case-note entry `[dispatch; SL-199-armswitch-a7]`.

## TL;DR

**The claude arm works. Nothing in the stack was broken.** The blocker was
**operator error**: the two *real* `dispatch-worker` Agent spawns were issued
**without the `isolation: worktree` parameter**. Without that flag the Agent tool
runs the subagent **in-place in the orchestrator's current worktree** (the coord
tree) — the `WorktreeCreate` hook never fires, no fork is minted, and the doctrine
pretooluse confinement jails the subagent to its cwd (the arming dir) with `src/`
read-only → the worker hits base-guard `src-readonly` and correctly refuses.

This was chased through a long false-hypothesis chain before the cause was found.
The lesson is a cheap up-front invariant (below).

## The one invariant that would have saved the session

> On any **"worker landed in the coord tree / `src/` read-only / no fork"** symptom:
> FIRST re-read the `Agent` call and confirm **`isolation: worktree` is present.**
> It is the single necessary condition for the fork. The `arm-spawn` + `cd`-into-
> spawn-dir ritual is **inert** without it. A worker in the coord tree almost always
> means the flag was omitted — not that the hook, jail, binary, or agent-def is broken.

## Definitive spawn recipe (claude arm) — proven this session

```
# orchestrator, cwd = coord root (.dispatch/SL-<n>)
doctrine dispatch arm-spawn --base <B> --slice <n>     # writes <coord>/.doctrine/state/dispatch/spawn/base
cd <coord>/.doctrine/state/dispatch/spawn              # park Bash cwd at the arming dir
# then Agent tool:
#   subagent_type: dispatch-worker
#   isolation: worktree          <-- MANDATORY. omitting it is the whole bug below.
#   prompt: <distilled phase prompt incl. base-guard>
```

Result when correct: worker forks to `<coord>/.worktrees/agent-<hex>`, HEAD == B,
branch `dispatch/agent-<hex>`, footer carries `worktreePath`.

## Probe matrix (empirical, this session, CC 2.1.198, in-jail)

| # | agent_type       | `isolation:worktree` | Bash cwd at spawn | outcome |
|---|------------------|:--------------------:|-------------------|---------|
| 1 | dispatch-worker  | **omitted**          | arming dir        | ran in **coord tree**, jailed to arming dir, `src/` RO → `src-readonly`. NO fork. |
| 2 | dispatch-worker  | **omitted**          | arming dir        | identical to #1 (post session-restart — ruled out "stale hooks"). |
| 3 | general-purpose  | yes                  | primary root      | **Fork/Passthrough** → `/workspace/doctrine/.worktrees/agent-<hex>`, detached @ primary HEAD `0fceaf72`. |
| 4 | general-purpose  | yes                  | arming dir        | **Fork @ B** → `<coord>/.worktrees/agent-<hex>` on `dispatch/agent-<hex>`, HEAD `bbd53972`. |
| 5 | dispatch-worker  | yes                  | arming dir        | **Fork @ B** → `<coord>/.worktrees/agent-<hex>` on `dispatch/agent-<hex>`, HEAD `bbd53972`. (trivial prompt) |

Rows 3–5 (flag present) all fork correctly, incl. the dispatch-worker. Rows 1–2
(flag absent) both fail. The isolated variable is the flag — not agent-type, model,
prompt, hook, jail, or version.

## Hypotheses raised and RULED OUT (the wrong turns, for the record)

1. **"Claude arm architecturally unavailable in a git repo"** — WRONG. Based on
   misreading the `EnterWorktree` doc; the hook fires for `isolation:worktree`
   Agent spawns in a git repo (rows 3–5).
2. **"Stale plugin hooks (session predates v0.15.2 rebuild)"** — WRONG. Survived a
   full session restart; failure #2 identical to #1.
3. **"Jail breaks WorktreeCreate"** — WRONG. Rows 4–5 fork fine inside the jail.
4. **"doctrine enabled project-level only; absent from user `enabledPlugins`"** —
   red herring (user: per-project hooks don't need a user-settings entry; `/hooks`
   UI is unreliable and shows the same "no hooks configured" even in a working
   setup).
5. **"dispatch-worker agent-def has `model: deepseek/deepseek-v4-pro`"** — WRONG for
   the claude arm. The active `.claude/agents/dispatch-worker.md` is the canonical
   **Claude** def (no model override). Deepseek is the **pi/codex/universal** arm
   variant (see below).
6. **"agent-type determines fork"** — WRONG. Row 5 (dispatch-worker + flag) forks.

## Confirmed environment facts (all GREEN — none was the cause)

- **CC version:** 2.1.198 — the version a prior memory (`mem.fact.claude.worktreecreate-hook-fires`, high trust) *proved* the hook fires on.
- **`WorktreeCreate` hook:** registered + fires. Loaded plugin `hooks.json`
  (`~/.claude/plugins/cache/doctrine/doctrine/0.1.0/hooks/hooks.json`) is valid and
  declares `SessionStart`, `WorktreeCreate` (`doctrine worktree create-fork`),
  `PreToolUse` (`doctrine worktree pretooluse`). No `SubagentStart` hook present.
- **`create-fork` standalone:** feeding `{"cwd":"<arming-dir>","name":"probe"}` to
  `doctrine worktree create-fork` forks correctly via BOTH `./target/debug/doctrine`
  and PATH `~/.cargo/bin/doctrine` — both 0.15.2.
- **`/hooks` UI:** unreliable — shows "no hooks configured for WorktreeCreate" even
  when the hook is live. Do NOT use it as a firing oracle. Use the probe (does an
  `isolation:worktree` spawn land in `.worktrees/agent-<id>`?) instead.

## dispatch-worker agent-def landscape (for future confusion)

Many `dispatch-worker.md` exist; they are **per-arm** variants, not conflicting
overrides:

- **claude arm** (`.claude/agents/dispatch-worker.md` → `.doctrine/agents/dispatch-worker.md`;
  also `.doctrine/agents/claude/`, `install/agents/claude/`): `tools: Read, Edit,
  Write, Bash, Grep, Glob[, mcp__doctrine__worker_commit]`, **no model override**
  (inherits the session Claude model).
- **pi / codex / universal arms** (`.pi/agents/`, `.doctrine/agents/{pi,codex,universal}/`,
  `install/agents/pi/`): `tools: read, edit, write, bash`, `model:
  deepseek/deepseek-v4-pro`, `traits: ["adherence/low"]`.

`.doctrine/agents/` is **gitignored** (install-materialized from the plugin/`install/`
trees). The claude arm spawns the Claude variant.

## Funnel-mechanics note (separate, still valid)

The **cached** `/dispatch-agent` SKILL (plugin v0.1.0) describes the *live-worktree
import* funnel (`import --from-worktree`) and "ABORT if no `worktreePath` footer."
That is **stale** for the claude arm. Per the shipped `install/dispatch-mechanics.md`
(authoritative) + the boot CASE-NOTES: the **claude arm now self-commits via the
gated `worker_commit` MCP tool**; the orchestrator then imports the **commit**
(`import --fork <C> --branch dispatch/<agent>`), non-committing, and commits
separately on the coord branch. The live-worktree-diff import is the **pi/subprocess**
path (or the MCP-down fallback). Drive funnel steps off the CLI, not the cached skill.

## Current state — fully resumable, nothing lost

- Coord worktree `dispatch/199` at base **`B = bbd53972`** (`.dispatch/SL-199/`).
- **S1 regression baseline captured at B: 0 failures** (`check regression capture`).
- Arm **primed** (`.doctrine/state/dispatch/spawn/base` = bbd53972).
- **Nothing committed.** No worker forks, no jail residue, no debris (all 5 probe
  forks cleaned up). Coord tree clean.
- SL-199 phases: 0/5, all `planned`. PHASE-01 not started.

## Next step (resume here)

1. Confirm arm primed + cwd = arming dir (`.dispatch/SL-199/.doctrine/state/dispatch/spawn`).
2. Spawn the PHASE-01 worker: `subagent_type: dispatch-worker`, **`isolation: worktree`**,
   with the distilled PHASE-01 prompt (create-fork confined discriminator + one-shot
   arm; touch `src/worktree/create.rs` only; base-guard block; self-commit via
   `worker_commit`). The distilled prompt is in this session's transcript.
3. Funnel per the router: read footer `worktreePath` → derive `name`/`branch` →
   `verify-worker` → `worktree import --fork <C> --branch dispatch/<name>` →
   `check regression diff --base B` → commit on coord branch → `dispatch record-boundary`
   → reap. Then PHASE-02.
