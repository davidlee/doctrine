# ISS-029: dispatch claude arm: missing cd-into-coord-tree instruction — workers fork off session main, not coordination base B

Discovered during SL-111 dispatch session (2026-06-19), where concurrent agents
were committing to `main` throughout.

## Root cause

**The mechanism works; the skill prose was missing the instruction.**

The Agent tool's `isolation: worktree` forks off the **Bash tool's cwd HEAD**,
not the session root (`mem_019ec65ecbc7`, controlled SL-067 probe). With
`worktree.baseRef='head'` in `.claude/settings.local.json`, base==B is achieved
by `cd`-ing into the coordination worktree before spawn — Bash cwd HEAD == B,
so the worker forks B. This was known and empirically confirmed (`mem_019ec6142d3b`).

But the dispatch-agent skill **never said to cd**. `dispatch setup` emits the
coordination dir path but doesn't cd; the skill prose listed `isolation: worktree`
with no pre-spawn placement instruction. The orchestrator stayed on its session's
`main`, so the worker forked `main` — not B.

## Two failure modes

1. **Mechanical (import refusal):** worker commit has `S^ = main`, not `S^ = B`.
   `worktree import` demands `S^==B` and `verify-worker` demands a stamped
   worker marker (harness worktrees are unstamped). PHASE-01 needed a manual
   `git rebase --onto B` to replant.

2. **Fatal for dependent phases:** PHASE-02 needs PHASE-01's `kinds.rs`, which
   exists only on `dispatch/111` — never on `main`. A worktree forked off
   `main` literally can't compile the prior phase's code.

## Fix

Added a **Pre-spawn — cd into the coordination tree** section to
`dispatch-agent/SKILL.md`, and step 2 in the dispatch router's outer loop:
cd into the coord tree after `dispatch setup`, park Bash cwd there for the
full drive loop. Step out only for authored writes.

## Related

- `mem_019ec65ecbc7`: controlled probe — Agent `isolation: worktree` forks Bash
  cwd HEAD; cd into coord tree ⇒ base == B
- `mem_019ec6142d3b`: `baseRef=head` is honoured, base controllable by placement
- `mem_019eb7263a90` (HIGH trust): rung-3 fork rule — workers must fork from
  explicit base B. The cd-into-coord-tree pattern achieves this on the claude arm.
- IMP-043: import `--allow-reanchor` for moved-HEAD recovery
- IMP-072: WorktreeCreate hook for claude-arm fail-closability (deferred)
