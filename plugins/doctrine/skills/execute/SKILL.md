---
name: execute
description: Use to implement one planned, approved phase of a slice — once its design and plan exist and its runtime sheet is ready. Move the phase to in_progress, build it TDD red/green/refactor, keep notes current, end green, and surface blockers early.
---

# Execute

You are executing one phase of planned work.

Inputs:

- the active runtime phase sheet (`state/.../phases/phase-NN.md`)
- `plan.toml` / `plan.md`
- `design.md` (canonical design reference)
- `slice-nnn.md` (scope)

## Process

1. Confirm the phase's entrance criteria (`EN-`) are met for the active phase.
2. Read `design.md` + `plan.toml` + the phase sheet before coding. If the sheet
   is not yet filled, run `/phase-plan` first; use `/preflight` if confirmed
   inputs, assumptions, and tensions are not yet surfaced.
3. Identify the concrete files or components you expect to touch first and run
   `/retrieve-memory` against those paths before deep reading or editing, so any
   scope-bound gotchas or patterns surface early.
4. Ensure the phase is `in_progress` before implementation proceeds:
   `doctrine slice phase <ID> PHASE-NN --status in_progress`.
5. Implement phase tasks in small coherent units, **TDD red/green/refactor**:
   write a failing test, make it pass, then refactor. Test behaviour, not
   trivial implementation. Build and improve test helpers and fixtures as you go.
6. After each meaningful unit, run `/notes`.
7. If that unit produced a durable gotcha, pattern, or subsystem fact worth
   future retrieval, run `/record-memory` before moving on.
8. Lint as you go (`cargo clippy`, zero warnings) and keep the tree buildable.
9. Follow the repo's commit policy: frequent, small conventional commits scoped
   with the slice id (e.g. `feat(SL-009): …`). Bias toward a clean tree; don't
   let `.doctrine/**` workflow edits drift in a stale uncommitted pile.
10. If `/preflight` or implementation reveals unresolved design ambiguity,
    unexpected obstacles, tradeoffs, or policy ambiguity, stop and `/consult`
    before improvising past it.
11. Keep the runtime sheet current as work progresses — never record progress in
    authored `plan.toml` / `plan.md` (the storage rule).
12. Before declaring the phase ready, run the verification gate — `just check`
    (lint + test + format) — and review the touched subsystems and notes once
    more for missed memory-capture candidates.
13. When exit criteria (`EX-`) and verification (`VT-`) are satisfied, mark the
    phase `doctrine slice phase <ID> PHASE-NN --status completed` and hand off:
    `/phase-plan` for the next phase, or `/audit` when the slice's phases are done.

## Outcomes

- Phase objectives are implemented with traceable evidence, ending green.
- Phase status matches reality during implementation, not only at closure.
- Notes and durable memory stay current throughout execution.
