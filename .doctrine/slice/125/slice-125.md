# Stamp provision source from primary worktree

## Context

ISS-011 **Defect C**, proven by the IMP-046 fresh-session probe (2026-06-20). The
`dispatch-worker` `SubagentStart` auto-stamp never lands a marker, so claude-arm
dispatch workers come up unstamped and must be hand-stamped.

`run_stamp_subagent` (`src/worktree.rs:2099`) resolves the provision **SOURCE** via
`root::find` on the **hook process cwd** (`src/worktree.rs:2110-2118`), on the
assumption that the `SubagentStart` hook fires inside the orchestrator tree.
Empirically false: the Claude harness runs the hook with **process cwd = the
worker's own worktree** (`.claude/worktrees/agent-<id>`) — identical to the payload
`cwd`. So source==fork, and `verify_sibling_worktree` bails `fork path is the
source tree itself; refusing to provision` (`src/worktree.rs:417`) →
`run_provision` aborts → no marker.

Hand-stamping works only because it runs from the orchestrator cwd (source ≠ fork).
The auto-hook cannot, as written.

Harness finding: `mem.pattern.dispatch.subagentstart-hook-cwd-is-worker-worktree`.
Governed by ADR-006 (orchestrator-sole-writer dispatch); mechanism origin SL-056.

## Scope & Objectives

Resolve the provision SOURCE in `run_stamp_subagent` to the repo's **primary
worktree** (the main checkout), independent of the hook process cwd, so
provision copies FROM source TO the worker worktree with source ≠ fork.

- Derive the primary worktree from the payload `cwd`'s repo — e.g. the main
  worktree behind `--git-common-dir`, or the first entry of
  `git worktree list --porcelain`. The payload `cwd` already pins the right repo
  (validated via `cwd_shares_repo` / `is_linked_worktree`); the primary worktree
  is a deterministic function of that repo, not of where the hook fires.
- Preserve every existing refusal/fail-closed path: `missing-cwd`, `bad-dir`,
  wrong `agent_type`, `already_marked` re-entrancy, and the M3 no-rollback posture.
- End state: a `dispatch-worker` subagent spawned at `isolation: worktree` comes up
  **stamped** — marker present at `<worker>/.doctrine/state/dispatch/worker` before
  the worker's first command — with no hand-stamp.

## Non-Goals

- ISS-011 Defect A (matcher heal on reinstall) and Defect B (`(deleted)` path
  poison) — separate defects, separate fixes (SL-124 territory).
- Changing the `SubagentStart` wiring, matcher, or the `/dispatch-agent` skill leg
  (the probe proved those sound).
- The fail-closed marker-absent privilege rule (ADR-006 D2a) — unchanged; this only
  makes the happy path actually stamp.
- `verify-worker` self-stamp-on-first-use (explicitly rejected in ISS-011 — fix the
  writer, not the symptom).

## Summary

(to be completed at close)

## Follow-Ups

(none yet)
