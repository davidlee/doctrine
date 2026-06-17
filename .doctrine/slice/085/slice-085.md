# Push dispatch drive loop into CLI — shrink skill to a thin CLI-calling wrapper

## Context

The dispatch subsystem spans ~930 lines across 4 skill files (dispatch, dispatch-agent,
dispatch-subprocess, worktree). Most of the drive loop is deterministic state-machine
logic that could live in the CLI — harness detection, coordination worktree setup,
phase ordering from plan.toml, the import funnel, branch-point guards, candidate
lifecycle, GC. The agent's job is judgment (batching, inline fallback, worker reports),
not mechanical orchestration.

Doctrine already follows this pattern: `slice status` encodes the closure seam,
`worktree gc` encodes the two-leg landed oracle, `review raise/dispose/verify` encodes
the adversarial protocol. Dispatch is the same class of problem.

Spawning workers crosses the agent boundary (CLI can't call `Agent` tool), but the CLI
can emit the structured next-action instruction the agent mechanically executes.

**Origin:** IMP-096, UX audit of agent-facing CLI surface.

## Scope & Objectives

- Move deterministic dispatch logic into `doctrine dispatch` subcommands:
  - `dispatch setup <SL-ID>` — create/resume coordination worktree, print env contract
  - `dispatch plan-next <SL-ID>` — read plan.toml → next phase(s), file-disjointness info
  - `dispatch status <SL-ID>` — phase rollup, what's done/blocked/next
  - `dispatch import <SL-ID> <PHASE-ID>` — alias/thin-wrapper over `worktree import`
- Shrink `dispatch/SKILL.md` to ~40 lines: when to use, `dispatch setup` → spawn worker
  → `dispatch import` loop, close handoff
- Shrink `dispatch-agent` / `dispatch-subprocess` to harness-specific spawn templates only
- CLI verbs are independently testable (unit + integration)

## Non-Goals

- `doctrine dispatch --drive` that fully automates the loop (crosses agent boundary)
- Changing the funnel cadence semantics (import → verify → branch-point → commit → record)
- Generalizing to other skills — dispatch is the exemplar; further skills follow later

## Summary

The dispatch skill file is the heaviest in the system. Its deterministic machinery
belongs in the CLI, following the existing pattern of `slice status`, `worktree gc`,
and `review` verbs. The agent becomes a thin loop: ask the CLI what's next, spawn a
worker, feed the result back.

## Follow-Ups

- Apply the same pattern to other large skills (triage after dispatch ships)
