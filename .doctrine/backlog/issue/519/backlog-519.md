# ISS-519: capsule-phase-planner and /phase-plan disagree on who flips phase status

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced in ~/flakes SL-006 PHASE-01 (observation `01a13ffa` there).

## Conflict

Three sources give the `in_progress` flip to two owners:

| source | says |
|---|---|
| `install/agents/claude/capsule-orchestrator.md:20` | orchestrator flips the phase to `in_progress` (loop step 2) |
| `install/agents/claude/capsule-phase-planner.md:22` | planner writes the runtime phase sheet "and nothing else" |
| `plugins/doctrine/skills/phase-plan/SKILL.md:45` | step 8: flip the phase to `in_progress` once the sheet is coherent |

A planner that follows `/phase-plan` either violates its own contract or flips
the phase before the orchestrator does, leaving the orchestrator's step 2 a
redundant or refused transition.

## Fix direction

Keep the flip with the orchestrator (it owns lifecycle in the capsule tier), and
have the planner def state explicitly that it skips `/phase-plan` step 8. Or
make `/phase-plan` step 8 conditional on the caller owning lifecycle. Relates to
ISS-470 (capsule tier never moves slice lifecycle).
