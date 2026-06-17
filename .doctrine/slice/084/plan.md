# Implementation Plan SL-084: Dispatch harness routing decomposition

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Four phases, strictly sequential. No parallelism opportunity — each phase
builds on the previous. All changes are skill prose (SKILL.md files) plus one
new agent-definition file. No Rust code changes, no CLI verb changes, no
installer changes (deferred).

## Sequencing & Rationale

**PHASE-01 → PHASE-02 → PHASE-03 → PHASE-04**

### PHASE-01: Agent definition first

The pi dispatch-worker agent definition (`.pi/agents/dispatch-worker.md`) is
authored first because it's referenced by name in PHASE-03's harness table
(`agent="dispatch-worker"`). It has no dependencies on other skill files — a
self-contained artifact. Authoring it first also confirms pi tool names
(`read, edit, write, bash`) before the spawn template references them.

### PHASE-02: Router detection second

The dispatch router (`dispatch/SKILL.md`) is updated next because it's the
entry point — the detection logic that routes to the right arm. It references
the arm skills but doesn't depend on their internal prose. Updating it second
means PHASE-03 can verify that the router's detection prose and the
subprocess arm's harness table are consistent.

### PHASE-03: Spawn table third

The dispatch-subprocess skill (`dispatch-subprocess/SKILL.md`) is the largest
change: harness→spawn table with pi and codex rows, D3 extension detection,
gc residual notes. Depends on:
- PHASE-01: dispatcher-worker agent name is referenced in the pi spawn row
- PHASE-02: harness detection prose in router should match notes in spawn table

### PHASE-04: Verification last

Install, lint, cross-reference check. Validates the whole chain: agent-def
loads, skills install, references resolve, no stale language remnants.

## Notes

- **No `dispatch-agent/SKILL.md` change.** The Claude arm is unchanged.
  Model selection surface is documented in the agent-def YAML (direct `model:`
  field), not in the skill.
- **No `.doctrine/agent-models.toml`.** Deferred per design D4.
- **No installer changes.** Unified `doctrine install` verb is a backlog item.
- **PHASE-02 and PHASE-03 are file-disjoint** (different files). If this plan
  is dispatched, they could run concurrently — but serial is simpler for a
  four-phase skill-prose slice.
- **Cross-references to verify post-install:**
  - `dispatch/SKILL.md` → `../dispatch-subprocess/SKILL.md` (pi, codex rows)
  - `dispatch/SKILL.md` → `../dispatch-agent/SKILL.md` (claude row)
  - `dispatch-subprocess/SKILL.md` → `../dispatch/SKILL.md` (funnel cadence)
  - `dispatch-subprocess/SKILL.md` → `../worktree/SKILL.md` (fork verb)
