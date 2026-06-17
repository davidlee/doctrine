<!-- Canonical doctrine digest — embedded, read by `doctrine boot`. Edit the
     source in `install/routing-process.md`; the installed copy is inert. -->

**Route before you act.** At the start of ANY substantive work, choose the
governing skill *before* inspecting files, running commands, or writing code.
When unsure, route to the stricter skill. No code without an approved plan.

| When | Skill |
|---|---|
| Correctness depends on project governance / unfamiliar subsystem / "right way?" | `/canon` + `/retrieve-memory` |
| Substantive work, path not yet clear | `/preflight` |
| Understand / audit an existing artifact, no change intended | `/walkthrough` (no slice) |
| Code-changing intent, no governing slice | `/slice` |
| Slice exists, design missing / stale / unapproved | `/design` → `/inquisition` |
| Design locked, no plan | `/plan` |
| Expanding the next phase just before executing | `/phase-plan` |
| Plan approved, phase active | `/execute` |
| Implementation done — evidence / reconciliation | `/audit` → `/reconcile` → `/close` |
| Slice exists, audit RV resolved, reconciliation brief written | `/reconcile` |

Mid-flight, any stage: unanticipated obstacle / tradeoff / emergent complexity →
`/consult` (don't improvise past it). Durable gotcha / pattern → `/record-memory`.
Latent **work** intent (issue / improvement / chore / risk / idea) → `backlog
new` instead of losing it; check `backlog list` at the start of substantive work
(already captured?). Work vs knowledge vs decision boundary: `using-doctrine.md`.
Finished a coherent unit → `/notes`. Handing off to fresh context → `/next`.
**Pairing / walkthrough are conduct postures**, orthogonal to the stage — layer
them on the routed stage, don't route to them *instead* of it. A walkthrough that
surfaces a concrete change re-enters `/route`.

**Core process:** `slice new` (scope) → `slice design` (author + adversarial
review until locked) → `slice plan` → `slice phases` → per phase: `phase-plan`
the runtime sheet, flip `in_progress`, implement TDD red/green/**refactor**, end
green, flip `completed` → `/audit` → reconcile → `/close`.

**Guardrails:** use the CLI, don't guess ids / command shapes / paths — and **read
entities via `doctrine <kind> show <ID>`, not raw files**: structured/queried data
lives in `*.toml`, prose in `*.md`, and `show` synthesizes both tiers (a `.md` body
may be empty by design — never judge an entity from one tier). The plan is not
higher authority than the design or `/canon`. Phase ids (`PHASE-NN`) and criteria
ids (`EN-/EX-/VT-`) are immutable — edits append, never renumber.

**Reference forms.** Entity ids — prefixed, 3-digit zero-padded (`SL-023`,
`ADR-005`, `REQ-059`); cite the durable id, never a mobile membership label
(`FR-`/`NF-`). Doc-local enumerations — bare (`OQ-1`, `D1`, `R1`, `Q1`, `C1`).
Criteria modes — `VT` by test / `VA` by agent / `VH` by human.

**Reference docs (read on demand).** `glossary.md` — kinds, ids, full reference
forms, verification taxonomy. `using-doctrine.md` — which verb for which intent,
reading via `show`, storage tiers, and hand-editing / edit-preserving rules.
