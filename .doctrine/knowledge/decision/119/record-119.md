# DEC-119: REV-045 uses a PRD umbrella modification

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Context

REV-045 changes PRD-001 and introduces product requirements for the phase-plan
content model. The Revision CLI accepted `modify PRD-001` but refused an
`introduce` row whose `member_of` target was PRD-001: the command admits only a
`SPEC-NNN` target. IMP-297 already records that exact product-requirement gap.

## Decision

REV-045 uses one primary `modify PRD-001` row as the umbrella for both the PRD
prose amendment and its new requirement members. The requirements are created
through the canonical `spec req add PRD-001` seam and are named explicitly in
REV-045's rationale.

This is a local disposition for REV-045, not a decision that product
requirements generally fall outside Revision payloads. IMP-297 remains the
place to settle and implement the missing Revision shape.

## Alternatives considered

- Waiting for IMP-297 would preserve a fully structured introduction payload,
  but would block the already-decided product-altitude work on unrelated CLI
  machinery.
- Recasting the requirements as technical requirements would satisfy the current
  Revision grammar but put product obligations at the wrong altitude.
- Creating a sibling PRD would avoid amending PRD-001 but contradict IMP-381's
  ownership assessment and duplicate the slice capability boundary.

## Consequences

- REV-045's touched-entity set names PRD-001 rather than each newly allocated
  requirement.
- The revision rationale must enumerate REQ-439 through REQ-447 so review can
  reconcile the umbrella row with the landed product contract.
- IMP-297 remains open; this decision neither resolves nor broadens it.
