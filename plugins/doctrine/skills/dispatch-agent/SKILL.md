---
name: dispatch-agent
description: The claude arm of `/dispatch` — spawn a worker via the `Agent` tool using the dispatch-worker subagent type with worktree isolation. Reached only from the `/dispatch` router on a claude↔env-marker agreement; do not invoke directly.
---

# Dispatch — claude arm

Spawn a worker via the `Agent` tool. The harness-identical funnel and drive loop
live in the [`/dispatch` router](../dispatch/SKILL.md) — this skill is only the
spawn template.

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
**Never:** spawn with a `subagent_type` other than `dispatch-worker`; run `fork` or
bwrap here (that's `/dispatch-subprocess`); claim parallel landing (v1 lands one per base).
**Always:** pin `subagent_type` to `dispatch-worker`; run `verify-worker` before
`import`; return to the router for the funnel cadence.
