<!-- Canonical doctrine digest — embedded, read by `doctrine boot`. Edit the
     source in `install/routing-process.md`; the installed copy is inert. -->

**Route before you act.** At the start of ANY substantive work, choose the
governing skill *before* inspecting files, running commands, or writing code.
When unsure, route to the stricter skill. No code without an approved plan.

| When | Skill |
|---|---|
| Correctness depends on project governance / unfamiliar subsystem / "right way?" | `/canon` + `/retrieve-memory` |
| Substantive work, path not yet clear | `/preflight` |
| Code-changing intent, no governing slice | `/slice` |
| Slice exists, design missing / stale / unapproved | `/design` → `/inquisition` |
| Design locked, no plan | `/plan` |
| Expanding the next phase just before executing | `/phase-plan` |
| Plan approved, phase active | `/execute` |
| Implementation done — evidence / reconciliation | `/audit` → `/close` |

Mid-flight, any stage: unanticipated obstacle / tradeoff / emergent complexity →
`/consult` (don't improvise past it). Durable gotcha / pattern → `/record-memory`.
Finished a coherent unit → `/notes`. Handing off to fresh context → `/next`.

**Core process:** `slice new` (scope) → `slice design` (author + adversarial
review until locked) → `slice plan` → `slice phases` → per phase: `phase-plan`
the runtime sheet, flip `in_progress`, implement TDD red/green/**refactor**, end
green, flip `completed` → `/audit` → `/close`.

**Guardrails:** use the CLI, don't guess ids / command shapes / paths. The plan is
not higher authority than the design or `/canon`. Phase ids (`PHASE-NN`) and
criteria ids (`EN-/EX-/VT-`) are immutable — edits append, never renumber.
