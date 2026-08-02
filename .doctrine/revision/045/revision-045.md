# REV REV-045 — Govern phase-plan content

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-027 Stage 1 and IMP-381 established that Doctrine's phase-plan content model
has no product owner. PRD-001 owns slices but presently governs only the slice
contract and lifecycle; PRD-015 delegates slice and phase planning semantics to
PRD-001; SPEC-014 owns the slice surface and the existence of the plan scaffold,
not the plan's contents. Shipped reference prose is therefore carrying normative
rules that belong in the spec corpus.

### Product-altitude decision

Amend PRD-001 rather than create a sibling PRD. A phase plan is the durable,
ordered execution contract of an accepted slice, not a separate evergreen
capability. The amendment makes that ownership explicit while preserving the
boundary to the later component specification: PRD-001 states the required
behaviour and proof obligations; the planned "Phase plan surface" technical spec
will own representation and mechanism, descend from PRD-001, and be parented by
SPEC-004.

### Product contract added

PRD-001 gains product-level intent, scope, principles, constraints, invariants,
success signals, flows, and verification obligations for:

- a durable ordered phase plan whose progress remains outside the authored tier;
- entry, exit, and VT/VA/VH verification semantics;
- immutable phase and criterion identity and stable order;
- opt-in criterion evolution with explicit predecessor/successor disposition;
- deterministic active-criterion derivation across replacement, withdrawal,
  split, merge, and cross-phase relocation;
- validation before execution and distinct current-versus-history rendering;
- deterministic compatibility and migration for existing plans; and
- unambiguous human/agent reads plus single ownership across authored and
  observed truth.

The binding obligations are requirement entities, not prose rows:

- REQ-439 — durable ordered phase plans;
- REQ-440 — entry, exit, and verification modes;
- REQ-441 — phase and criterion identity and order;
- REQ-442 — opt-in criterion lineage and active meaning;
- REQ-443 — validation before execution;
- REQ-444 — governing versus historical rendering;
- REQ-445 — compatibility and migration;
- REQ-446 — unambiguous human and agent readability; and
- REQ-447 — single ownership across authored and observed tiers.

### Boundaries preserved

This revision does not settle a lineage schema or implementation design. It does
not revive RFC-027's rejected phase-local change claim, alter selector authority,
introduce a second executable-command or coverage contract, persist criterion
evidence, or design worker discovery and reconciliation. SPEC-002 remains the
owner of executable verification and observed coverage; selector-conformance work
remains with IMP-310 and ISS-251; criterion evidence remains a later programme.

### Revision payload limitation

The CLI refused `introduce --member-of PRD-001`, the known gap recorded by
IMP-297. Under DEC-119, the primary `modify PRD-001` row is the umbrella for this
prose amendment and REQ-439 through REQ-447, which are allocated through
`spec req add`. This local workaround does not settle IMP-297.
