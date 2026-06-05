---
name: route
description: Use at the very start of ANY substantive work in a Doctrine repo — before inspecting files, running commands, or writing code. The mandatory gate that chooses the governing skill. Skip it only when the user has explicitly told you to.
---

# Route

The mandatory routing layer for Doctrine. Follow it rigorously unless the user
has explicitly instructed otherwise.

Do not respond, explore, inspect files, run commands, or start implementation
until you have chosen the skill that governs the task. If there is a reasonable
chance another Doctrine skill governs it, route through that skill first.

Do not rationalize around this. If you skip routing because the task feels
familiar, simple, urgent, or "probably fine", you are doing it wrong:

- "I'll just inspect files first." → No. Routing decides *how* you inspect.
- "I already know the command shape." → No. Use the CLI, don't guess.
- "Small enough to skip routing." → Small tasks still need the right skill.
- "I'll gather context first and decide later." → Decide first.

When unsure, route to the stricter skill, not the looser one.

## CLI

`doctrine --help` (dev: `./target/debug/doctrine --help`). If `doctrine` is
unavailable, STOP and alert the user.

## Choose the governing skill

1. Correctness depends on project governance, an unfamiliar subsystem, or "what
   is the right way here?" → `/canon` and `/retrieve-memory` first.
2. Substantive new work and the path is not yet clear → `/preflight`.
3. Code-changing intent with no governing slice → `/slice`.
4. Slice exists, design missing / stale / unapproved → `/design`
   (then `/inquisition` for an adversarial pass before locking).
5. Design locked, no plan → `/plan`; expand the next phase just before
   executing it → `/phase-plan`.
6. Plan approved and a phase is active → `/execute`.
7. Implementation done, now evidence / reconciliation → `/audit` → `/close`.

Mid-flight, regardless of stage:

- Unanticipated obstacle, decision, or emergent complexity → `/consult`.
- Durable fact / gotcha / pattern worth keeping → `/record-memory`.
- Finished a coherent unit → `/notes`; handing off to fresh context → `/next`.
- Authoring evergreen specs under `doc/*` → `/spec-product`, `/spec-tech`.

## Priority order

1. `/canon` + `/retrieve-memory` — when correctness depends on project truth.
2. `/preflight` — when the path is not clear.
3. shaping — `/slice` `/design` `/plan` `/phase-plan`.
4. execution — `/execute`.
5. close-out — `/audit` `/inquisition` `/close`.

## Guardrails

- **No code without an approved plan** (the gate). Do not jump "there is a
  slice" → `/execute`; the design, plan, and phase sheet must exist first.
- Do not guess slice ids, command shapes, or file locations — use the CLI.
- Do not treat the plan as higher authority than the design or `/canon`.
- Do not import stricter ceremony than the project has adopted; surface a
  conflict between local doctrine and a routing default rather than improvising.
