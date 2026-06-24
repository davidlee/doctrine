# References role grammar

## Context

RFC-003 triaged CHR-024's holistic relation-model review into four axes. **Axis B —
"overloaded edge intent"** is the RFC's core: `specs`/`slices`/`related` conflate
distinct intents (F-3, IMP-149). Axis A shipped as SL-145 (backlog source parity);
this slice is B.

P1's exhaustive census (RFC-003 § Empirical grounding) found **~25 of ~113**
entity→entity reference edges assert an intent their label gets wrong — a risk cannot
*implement* a spec; an improvement only *bears on* one; `related` is a degenerate
catch-all whose peer reading is the minority. The defect is a **noun verbing**: `specs`
names the target kind, never the verb. The missing verb *is* the role.

The fix RFC-003 proposes: separate **durable structural relation shape** (the label)
from **contextual role-intent** (a closed role enum), collapsing the work→canon family
into one `references` label refined by `{implements, reviews, scoped_from, bears_on,
related}`, with target validation re-keyed from `(source, label)` to
`(source, label, role)`. Type safety is preserved — the target gate relocates from
label to role.

**Governance gate.** RFC-003 asserts no canon; adoption requires a ratifying
ADR/Revision (RFC § Outcome). Per the routing decision for this slice, **the ratifying
ADR is folded into this slice's `/design`** — the structure/intent split, the
*derivable-not-relational* law, and graph-effect-in-consumer are the decisions `/design`
must lock and emit as an ADR before the plan. No code precedes that ADR.

## Scope & Objectives

In scope (the B core, per RFC-003 § Proposed slice decomposition row B):

- **Closed `role` enum** — `{implements, scoped_from, concerns}` as a code-level closed
  enum, all directional (each new intent a code change, cost #1). Decisions vs RFC-003:
  `reviews` **dropped** — folds into `concerns` (no structural distinction; heavyweight
  review stays the RV `reviews` label); `bears_on` **renamed `concerns`** (jargony/weak);
  `related` **not** a role — stays its own symmetric-neutral label (symmetry is structural).
- **Two-level closed grammar** in the relation contract:
  - `(source_kind, label) → legal roles` (e.g. `references` from SL admits
    `{implements, scoped_from, concerns}`).
  - `(source_kind, label, role) → TargetSpec` (e.g. `references(implements) →
    {SPEC,PRD,REQ}`; `references(concerns) → AnyNumbered`; `references(scoped_from) →
    {backlog kinds}`).
- **`references` label** replacing the work→canon family: `specs` and the standalone
  `requirements` label (SL→REQ) fold into `references` + role. `governed_by`, `related`,
  `part_of`, `supersedes`, `exclusive_with` stay distinct labels. (`related` does **not**
  fold — only its mismapped rows migrate to `references(concerns|scoped_from)`; true peers
  stay.)
- **Seam threading** (RFC cost #3 — this is the bulk): the `role` column threads
  `RELATION_RULES`, `lookup(source,label)` → role-aware, `RelationEdge`/`RelationRow {
   label, target }` → carries role, `validate_link`, and the surfaces
  (`CatalogEdgeLabel`, `inspect`, `relation list`, web graph).
- **Migration** (cost #2): out-of-band deterministic one-time rewrite (**no shipped
  `migrate` verb** — SPEC-018 dogfood precedent), mapping existing `specs`/`requirements`/
  mismapped-`related` edges per a `(source-kind, label, target-kind)` map, re-censused
  live. Ambiguous rows (SL→SPEC implements-vs-concerns; `related` peer-vs-concerns)
  hand-triaged pre-commit. **No persisted `unspecified`** — every landed row carries a
  real role. Hard-cut, atomic with the code.
- **Role-derived inbound reciprocal** (cost #4, leaning role-derived per RFC): `inspect`
  renders "implemented by / reviewed by / scoped from / bears on / related to" rather
  than a flat label echo; `inbound_name` re-keyed from label to `(label, role)`, coexisting
  with the ADR-004 `superseded_by` reverse carve-out.
- **CLI end-to-end**: `doctrine link <src> references --role <role> <target>` authors →
  validates against the two-level grammar → persists → reads back on `inspect`/`show`
  (outbound) → renders the derived inbound on the target. `unlink` round-trips.

## Non-Goals

- **Axis A** (backlog source parity) — shipped, SL-145.
- **Axis C** (coverage / close-gate) — `validate`/`/close`/SPEC-002, not vocabulary
  (RFC § Layer 2 design law). Separate slice.
- **Axis D** (decomposition `part_of` + altitude facets + concept-map lattice) — sibling
  spec, sequenced separately. `part_of` is **kept strictly separate** from
  `references(scoped_from)` (RFC § Decomposition).
- **Non-entity-target edge** (memory / file / glob / vec) — the boundary the role grammar
  cannot absorb (IMP-012, IDE-015). Named, deferred. `drift` is **not** fully retired by
  this slice (only its entity→entity rows).
- **Temporal projection** (`slices` planned-vs-done) — derive from lifecycle status, not
  a label. No `planned_by`/`completed_by`. The mapping to closure-without-delivery states
  is deferred to lifecycle semantics.
- **Relation planes / `influences` family / `related` symmetry** — emergent, explicitly
  not locked (RFC § Open). Out of B.
- **Can work `implements` an ADR?** — ADR excluded from `implements` target set;
  `governed_by` stays the ADR relation. Filed, not resolved here.

## Open Questions — RESOLVED in /design

- **OQ-1 `related` symmetry tension — RESOLVED:** `related` stays its own
  symmetric-neutral label; symmetry changes inbound semantics → structural → label, not
  role (RFC's own deciding principle). `reviews` likewise dropped (folds into `concerns`).
- **OQ-2 inbound rendering — RESOLVED:** role-derived inbound; `inbound_name` re-keyed to
  `(label, role)`; VT-3 re-keyed; `supersedes`/`governed_by` carve-out untouched.
- **OQ-3 migration default — RESOLVED:** no blanket default; deterministic
  `(source-kind, label, target-kind)` map re-censused live + hand-triage of the ambiguous
  residue; no persisted `unspecified`.

See `design.md` decision ledger (D1–D6) and the integrated adversarial review (AR-1–AR-6).

## Summary

Shipped the structure/intent split (ADR-016): the work→canon label family — the
noun-named `specs` (SL→`{SPEC,PRD}`) and the standalone `requirements` label (SL→`REQ`)
— collapsed onto a single structural `references` label refined by a closed `Role`
`{implements, scoped_from, concerns}`. Target validation re-keyed from `(source, label)`
to `(source, label, role)`: the gate relocated from label to role, so type safety is
preserved while the missing *verb* becomes first-class. `related` did **not** fold
(symmetry is structural — it stays its own label); only its mismapped rows migrated to
`references(scoped_from|concerns)`. RFC-003's proposed `reviews`/`bears_on` roles
resolved to `concerns` (the lightweight `reviews` folds in; heavyweight review keeps the
RV `reviews` label).

Delivered across six phases: ADR-016 ratified (P1); the closed `Role` enum + two-level
grammar `legal_roles`/`targets_for_role` (P2); role-bearing `[[relation]]` storage with
no-dual-read (P3); every surface threaded — reader inbound, `inspect`, `relation
list`/`census`, web-graph backend data, `show`/`show --json` references-by-role object,
and `link`/`unlink --role` (P4); the out-of-band corpus migration — 195 edges
(implements 93 · concerns 76 · scoped_from 14 · related-kept 12), a full in-memory
transform applied as one atomic swap, parser + rewritten corpus in the same commit,
gated by the role-assignment oracle and a committed disposition artifact
(`migration-dispositions.md`, VH-1-confirmed) (P5); and SPEC-018 + relation-vocabulary.md
rewritten to the references+role contract (P6). `validate` clean; `just gate` green.

Carried-opens confirmed against the shipped state: the web-graph **TS frontend**
(`web/map/`) renders the backend `role` field but is out of the design's named seam
(§2.7 points at `catalog/graph.rs`) — filed as a follow-up, not a regression.

## Follow-Ups

- Axis C slice (coverage/close-gate).
- Axis D sibling spec (`part_of` + altitude).
- Non-entity-target edge (IMP-012, IDE-015) — retires remaining `drift` rows.
- Prose-hunt pass for absent relations expressed as prose (F-1/F-5/F-7).
- Web-graph TS frontend (`web/map/`) — read `edge.role` to render `references(<role>)`
  in the dot label; backend already serialises it (P4a). File as backlog at reconcile.
- SPEC-005/006/016 rewire to *reference* SPEC-018 rather than re-tell the relation story
  (now the contract is proven in code).
