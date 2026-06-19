---
name: dispatch-agent
description: The claude arm of `/dispatch` — spawn a worker via the `Agent` tool using the dispatch-worker subagent type with worktree isolation. Base==B by placement (cd into the coordination tree before spawn). Reached only from the `/dispatch` router on a claude↔env-marker agreement; do not invoke directly.
---

# Dispatch — claude arm

Spawn a worker via the `Agent` tool. The harness-identical funnel and drive loop
live in the [`/dispatch` router](../dispatch/SKILL.md) — this skill is only the
spawn template.

## Pre-spawn — cd into the coordination tree

The Agent tool's `isolation: worktree` forks off the **Bash tool's cwd HEAD**,
not the session root (`mem_019ec65ecbc7`, controlled probe). With
`worktree.baseRef='head'` in `.claude/settings.local.json`, the worker's
worktree HEAD equals the Bash cwd's HEAD.

**Before every spawn:** `cd` into the coordination worktree directory (emitted
by `dispatch setup` as `coordination_dir=`). This makes `Bash cwd HEAD == B`,
so the worker forks exactly the intended base.

**Placement precondition (ISS-031):** the coordination worktree MUST live inside
the project root — convention `.dispatch/SL-<n>`. Under a cwd-confining jail
(bubblewrap rooted at `/workspace/<repo>`), a `cd` to an outside sibling
(`/workspace/<repo>-dispatch-N`) silently reverts to the project root on the next
Bash call — the `cd` never sticks, the session stays on `main`, and the worker
forks `main` instead of B. `dispatch setup` now fails closed on the claude arm
when `--dir` resolves outside the root; pass `--dir .dispatch/SL-<n>`. Confirm
placement with a bare `cd <coord> ; pwd` in a separate Bash call before trusting it.

Keep the Bash cwd parked in the coord tree across the whole drive loop — serial
dependent phases self-base: after a funnel commit advances the coord tree HEAD,
the next spawn (still cd'd there) forks the new tip, carrying prior phases' code.
Step out to the session root only for authored writes (slice status, audit,
memory).

## Spawn

```
subagent_type: dispatch-worker
isolation: worktree
prompt: <pre-distilled worker prompt>
```

## Post-spawn
After the worker returns: `doctrine worktree verify-worker --base <B> --dir <worktree>`.
Abort the funnel on any refusal.

## Boundary recording
After the batch's code commit and before the knowledge commit:
`doctrine dispatch record-boundary --slice <N> --phase PHASE-NN --code-start <B> --code-end <B+1>`.
Claude-arm-only (no fork branch); skip on codex/pi.

## Red Flags
**Never:** point `dispatch setup --dir` at an outside-root sibling on the claude
arm (the `cd` silently reverts under a jail → worker forks `main`; the CLI now
refuses this, but use `.dispatch/SL-<n>`); spawn without cd'ing into the
coordination tree first (a worker forked off `main` is a wrong-base verdict at
`verify-worker`); spawn with a
`subagent_type` other than `dispatch-worker`; run `fork` or bwrap here (that's
`/dispatch-subprocess`); claim parallel landing (v1 lands one per base).
**Always:** cd into the coord tree before every spawn; pin `subagent_type` to
`dispatch-worker`; run `verify-worker` before `import`; return to the router for
the funnel cadence.
