# Claude dispatch-worker Agent spawn requires isolation:worktree — omission runs in-place in the coord tree, no fork

On the claude arm, the `Agent`-tool spawn of a `dispatch-worker` **MUST** carry
`isolation: worktree`. That flag is the single necessary condition for the
`WorktreeCreate` hook (`doctrine worktree create-fork`) to fire and mint the
isolated fork. The `dispatch arm-spawn` + `cd`-into-the-arming-dir ritual is
**completely inert without it** — it writes `base` and positions cwd, but no fork
happens unless the spawn itself requests worktree isolation.

## Symptom of omission (memorise this → cheap diagnosis)

Worker returns reporting: it is in the **coordination tree** (`git rev-parse
--git-dir` = `.git/worktrees/SL-<n>`, NOT `…/agent-<hex>`), jailed to its cwd (the
arming dir), `src/` **read-only**, and it cannot author → base-guard fails
`src-readonly` (or the delta lands as loose changes in the coord tree). **No**
`.worktrees/agent-<hex>` fork, **no** footer `worktreePath`, `base` unconsumed.

## The invariant

> On ANY "worker landed in the coord tree / `src/` read-only / no fork" symptom,
> FIRST re-read the `Agent` call and confirm `isolation: worktree` is present.
> A worker in the coord tree almost always means the flag was omitted — **not** that
> the hook, jail, binary, agent-def, plugin registration, or CC version is broken.

## Proof (SL-199, 2026-07-04, CC 2.1.198, in-jail)

Five spawns, one isolated variable — the flag:

| agent_type | `isolation:worktree` | cwd | outcome |
|---|:---:|---|---|
| dispatch-worker | omitted | arming dir | coord tree, `src/` RO, NO fork |
| dispatch-worker | omitted | arming dir | identical (post-restart) |
| general-purpose | yes | primary root | Fork/Passthrough → `.worktrees/agent-<hex>` |
| general-purpose | yes | arming dir | Fork @ B → `dispatch/agent-<hex>` |
| dispatch-worker | yes | arming dir | Fork @ B → `dispatch/agent-<hex>` |

Rows with the flag all fork (incl. dispatch-worker). Cost of NOT knowing this: a
multi-hour false-hypothesis chase (claude-arm-unavailable → stale-hooks → jail →
enabledPlugins → agent-def model override → agent-type) — every one wrong.

## Also confirmed GREEN (none was the cause)

- `WorktreeCreate` hook fires; `create-fork` forks standalone via both
  `./target/debug` and PATH `~/.cargo/bin` (0.15.2).
- The `/hooks` TUI is an **unreliable** firing oracle (shows "no hooks configured
  for WorktreeCreate" even when live). Use the probe — does an `isolation:worktree`
  spawn land in `.worktrees/agent-<id>`? — as the real test.
- Per-project plugin enablement does NOT require a `~/.claude/settings.json`
  `enabledPlugins` entry.

Refines [[mem.pattern.dispatch.unarmed-agent-worker-runs-in-coord-tree]] (same
coord-tree symptom, there attributed to skipping `arm-spawn`; the deeper necessary
condition is the flag). See also [[mem.fact.claude.worktreecreate-hook-fires]],
[[mem.pattern.dispatch.worktreecreate-replace-base-control]].
Fuller writeup: `.doctrine/slice/199/dispatch-harness-findings.md`.
