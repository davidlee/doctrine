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

- **Closed `role` enum** — `{implements, reviews, scoped_from, bears_on, related}` as a
  code-level closed enum (each new intent a code change, cost #1).
- **Two-level closed grammar** in the relation contract:
  - `(source_kind, label) → legal roles` (e.g. `references` from SL admits
    `{implements, reviews, scoped_from, bears_on}`; from RV admits `{reviews}`).
  - `(source_kind, label, role) → TargetSpec` (e.g. `references(implements) →
    {SPEC,PRD,REQ}`; `references(reviews) → AnyNumbered`).
- **`references` label** replacing the work→canon family: `specs`, `related`, and the
  standalone `requirements` label (SL→REQ) fold into `references` + role. `governed_by`,
  `part_of`, `supersedes`, `exclusive_with` stay distinct labels.
- **Seam threading** (RFC cost #3 — this is the bulk): the `role` column threads
  `RELATION_RULES`, `lookup(source,label)` → role-aware, `RelationEdge`/`RelationRow {
   label, target }` → carries role, `validate_link`, and the surfaces
  (`CatalogEdgeLabel`, `inspect`, `relation list`, web graph).
- **Migration** (cost #2): backfill a role onto existing `specs`/`slices`/`related`
  edges; `migrate` stamps a default (`implements`) or explicit `unspecified` to force
  triage. `unspecified` is **migration-transient only** — `validate` flags it as
  unresolved; `role` is mandatory on every persisted `references` edge in steady state.
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

## Open Questions (for /design)

- **OQ-1 `related` symmetry tension** — RFC's late finding: `related` is symmetric +
  neutral, the odd man out among directional `references` roles. Does `related` keep its
  own symmetric-neutral label rather than collapse to a `references` role? (RFC § Open,
  "relation planes".) Design must settle or explicitly defer.
- **OQ-2 inbound rendering** — role-derived vs label-flat inbound (cost #4). RFC leans
  role-derived; confirm and scope the `(label, role)` `inbound_name` re-key.
- **OQ-3 migration default** — `implements` blanket default vs `unspecified`-forces-triage
  per source/target shape. P1 shows `SL→spec` = implements (~44) but `IMP→`/`RSK→` are
  mismapped; a blanket `implements` would re-mismap them. Design the per-shape default.

## Summary

(to be written at close)

## Follow-Ups

- Axis C slice (coverage/close-gate).
- Axis D sibling spec (`part_of` + altitude).
- Non-entity-target edge (IMP-012, IDE-015) — retires remaining `drift` rows.
- Prose-hunt pass for absent relations expressed as prose (F-1/F-5/F-7).
