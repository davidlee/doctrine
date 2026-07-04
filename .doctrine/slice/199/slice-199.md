# Confined subagent orchestrator drive-loop (Mode B)

## Context

The serial-dependent **capstone** of Mode B (RFC-005). SL-198 lays the foundation —
the gated `mcp__doctrine__worker_commit` tool + the worker tool-surface conformance
lint — with the **existing main-thread** orchestrator still driving the funnel. This
slice moves orchestration **off the main thread** onto a **fully-confined subagent
orchestrator**, unlocking off-main-thread and parallel/hierarchical dispatch (the
RFC-011 lever: main context stays a thin cache-stable router; the ~40–49k/phase funnel
ceremony burns a disposable subagent context).

`needs` SL-198 (the `worker_commit` keystone it builds on). No exemption, no
un-jailing — the orchestrator runs in the coordination worktree under `Jail(coord-cwd)`
and performs every boundary-crossing write through doctrine MCP tools.

## Scope & Objectives (provisional — shaped in parallel while SL-198 builds)

1. **Orchestrator agent type.** A `dispatch-orchestrator` subagent type that lists
   `Agent` in `tools` (so it can spawn nested workers — harness allows nesting to
   depth-5 since v2.1.172) and holds the dispatch MCP funnel tools, nothing else
   writable (same tool-surface invariant + conformance lint as the worker). Placement:
   spawned with cwd inside the coordination worktree → `Jail(coord-cwd)` (a
   primary-tree cwd would be `Reject`-ed outright — probe finding).

2. **Dispatch MCP funnel surface** (the tools a *confined* orchestrator needs, that
   SL-198 deliberately deferred): one-shot commit-**import**, **reap** sibling
   worktrees, **record-boundary** (ledger splice — tree-read from the branch tip, not
   the worktree fs, [[mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree]]), and
   **lifecycle-flip**. Each rides the same belt/logic its CLI verb already runs (one
   seam, two doors — no forked implementation).

3. **The drive-loop.** The confined orchestrator runs the funnel cadence:
   `arm-spawn` → spawn `dispatch-worker` (nested) → worker self-commits via
   `worker_commit` → orchestrator imports the commit, records the boundary, flips
   lifecycle — all via MCP, its raw `Edit`/`Write` confined to the coord tree, raw
   Bash to shared `.git` walled (proving no un-jailing). Conflict-judgement ops
   (`refresh-base`, `candidate create/admit`, `integrate`) stay **report-and-halt** to
   the human/main-thread, unchanged.

## Non-Goals

- Everything in SL-198 (keystone + lint) — consumed, not rebuilt.
- Mode A coordinator exemption — separate follow-on; this slice un-jails nothing.
- Persistent (http/sse) MCP for claude/subprocess arm unification — noted, not built.
- Hierarchical (N-deep) orchestration beyond confirming the depth-5 budget headroom.

## Affected surface (coarse — `/design` sets the exact touch-set)

- `.claude/agents/**` — the `dispatch-orchestrator` agent-def.
- `src/mcp_server/**` — the import / reap / record-boundary / lifecycle-flip tools.
- `src/dispatch.rs`, `src/worktree/**` — belt/logic reuse; `arm-spawn` under a
  subagent spawner.
- `.claude/skills/dispatch-agent/**` — the drive-loop cadence.

## Risks / assumptions / open questions (from RFC-005 OQ set)

- **OQ-1 — `arm-spawn` under a subagent spawner + `Jail`.** Closed "yes mechanically"
  by codex-2 (writes under the coord worktree, bwrap grants it rw, `Agent` passes);
  one live confirmation at execute-time.
- **OQ-2 — belt survival under relocation** into orchestrator-invokable MCP tools;
  which funnel ops lack a tool today.
- **The blocking-`Agent` rendezvous** ([[mem.fact.dispatch.single-slot-arming-rendezvous]]):
  the orchestrator gets no turn between spawn and worker completion — informs the
  serial vs parallel drive shape and per-worker state.
- **Worktree lifecycle** on the claude arm: the orchestrator (not the harness) reaps;
  `WorktreeCreate` present, no `WorktreeRemove`, tree left on disk with diff intact
  ([[mem.fact.claude.worktree-remove-auto-teardown]]) — with `worker_commit` the
  import is of a commit and reap is an MCP op.
- **OQ-5 — depth-5 ceiling** vs hierarchical ambitions.
- **MCP soft-dependency** becomes load-bearing for this arm (no raw-git fallback for a
  confined orchestrator) — MCP-server health is a dispatch-stopper; document.

## Verification / closure intent

- A `dispatch-orchestrator` subagent drives a real slice phase end-to-end on the claude
  arm: spawns a worker, worker self-commits, orchestrator imports the commit, records
  the boundary, flips lifecycle — while its own raw Bash to shared `.git` stays walled.
- Nested worker confinement still holds (`Jail(cwd)`, depth-agnostic).
- Main-thread dispatch remains available as the MCP-down fallback; subprocess arm green.

## Summary

Move Mode B orchestration off the main thread onto a confined `dispatch-orchestrator`
subagent that drives the funnel via MCP tools (built here) atop SL-198's
`worker_commit`. Unlocks off-main-thread/parallel dispatch with no un-jailing.

## Follow-Ups

- Mode A coordinator-exemption knob (raw-git orchestration alternative).
- Governance ratified at reconcile: ADR-012 REV + ADR-011 D6 amendment + SL-182 note.
- Persistent http/sse MCP for arm unification; exfil tightening for downstream repos.
