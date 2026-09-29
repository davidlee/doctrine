The compact summary every session carries. It owns routing, core process,
guardrails and the library register; each other block cues its owner.

**Route before you act.** At the start of ANY substantive work, choose the
governing skill *before* inspecting files, running commands, or writing code.
When unsure, route to the stricter skill. No code without an approved plan.

| When | Skill |
|---|---|
| Correctness depends on project governance / unfamiliar subsystem / "right way?" | `/canon` + `/retrieve-memory` |
| Substantive work, path not yet clear | `/preflight` |
| Understanding an artifact is the whole task, no change intended | `/walkthrough` (no slice) |
| Reviewing code for quality / correctness — ledgered findings | `/code-review` |
| Spec target or boundary unclear — what's governed, what's dark, where a new spec sits | `/spec-coverage-assessment` |
| Authoring or evolving a product / tech spec | `/spec-product` / `/spec-tech` |
| Capturing or settling an epistemic record (assumption / decision / question / constraint / evidence / hypothesis / concept) | `/knowledge` |
| Code-changing intent, no governing slice | `/slice` |
| Slice exists, design missing / stale / unapproved | `/design` (→ `/inquisition` on request) |
| Design locked, no plan | `/plan` |
| Expanding the next phase just before executing | `/phase-plan` |
| Plan approved, phase active | `/execute` |
| Plan approved, driving phases via workers in isolated worktrees | `/dispatch` |
| Implementation done — evidence / reconciliation | `/audit` → `/reconcile` → `/close` |
| Slice exists, audit RV resolved, reconciliation brief written | `/reconcile` |

Unsure where the lifecycle stands: `doctrine status` / `doctrine next`.

**Conduct postures** layer on the routed stage — orthogonal to it, composable
with each other, never routed to *instead* of it:

| Posture | Layer it on when |
|---|---|
| `/pair` | working side-by-side with the human in the loop |
| `/walkthrough` | comprehension overlay mid-work (when understanding *is* the task, it's the stage above) |
| `/rigour` | at the edge of capacity — high complexity / uncertainty, costly context, hard-to-reverse steps |

A walkthrough that surfaces a concrete change re-enters `/route`.

Mid-flight, any stage: unanticipated obstacle / tradeoff / emergent complexity →
`/consult` (don't improvise past it). Receiving review findings / corrections →
`/feedback` (adjudicate on evidence; close the loop). Durable gotcha / pattern →
`/record-memory`. Not sure which artifact owns a thing?
`doctrine search <query>` before assuming — one ranked query over the entity
corpus; `lib:reference/using-doctrine.md` gives its scope.
Latent **work** intent (issue / improvement / chore / risk / idea) → `backlog
new` instead of losing it; check `backlog list` at the start of substantive work
(already captured?). Work vs knowledge vs decision boundary:
`lib:reference/using-doctrine.md`.
Finished a coherent unit → `/harvest`. Handing off to fresh context → `/handover`.
Agent confusion / stale memory corpus → `/reviewing-memory`.

**Core process:** `doctrine slice new` (scope) → `doctrine design start` opens
the managed design run; read the design with `doctrine design show` (the turn
envelope under `--format prompt`), mutate it with `doctrine design apply`,
write the authored prose with `doctrine design materialise`, and re-enter a
cold context with `doctrine design resume` — the run locks when its gate clears
→ `doctrine slice plan` → `doctrine slice phases` → per phase: `/phase-plan` the
runtime sheet, flip `in_progress`, implement TDD red/green/**refactor**, end
green, flip `completed` → `/audit` → reconcile → `/close`.

**Guardrails:** use the CLI (prefer the MCP tools if available); don't guess
ids / command shapes / paths — and **read entities via `doctrine show <REF>`**
(the canonical ref names its kind), not raw files: structured/queried data
lives in `*.toml`, prose in `*.md`, and `show` synthesizes both tiers (a `.md`
body may be empty by design — never judge an entity from one tier; storage
tiers: `lib:reference/using-doctrine.md`). The plan is not higher authority
than the design or `/canon`. Phase ids (`PHASE-NN`) and criteria ids
(`EN-/EX-/VT-`) are immutable — edits append, never renumber.

**Reference forms.** Cite an entity by its durable id — prefixed, 3-digit
zero-padded (`SL-023`, `ADR-005`, `REQ-059`) — never a mobile membership label
(`FR-`/`NF-`). Doc-local ids are bare (`OQ-1`, `D1`, `F-4`); on first use in a
message you write, qualify one by its artefact's durable id (`SL-NNN`'s
`OQ-1`) with a one-line synopsis, and for a human reader name the verb that
opens it (`doctrine show RV-NNN`). Criteria modes: `VT` by test / `VA` by
agent / `VH` by human. Owner: `lib:reference/glossary.md` § reference forms.

**Reference docs (read on demand).** A library citation is written
`lib:<address>`; read it with `doctrine library show <citation>`, which accepts
the prefix verbatim. Nothing is on disk to read or glob for; `doctrine library
tree` lists the library. A retrieval a skill or reference doc specifies is
mandatory, not optional reading. The publication model:
`lib:reference/using-doctrine.md` § publication. The register:
`lib:reference/glossary.md` — kinds, ids, full reference forms, verification
taxonomy. `lib:reference/using-doctrine.md` — which verb for which intent,
reading via `show`, storage tiers, publication, edit-preserving rules.
`lib:reference/shipped-corpus-authoring.md` — writing text doctrine ships into
a client repo, and citing without a repo-private id.
