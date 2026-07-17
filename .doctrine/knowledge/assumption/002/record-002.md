# ASM-002: Capture touchpoints in design/consult/preflight drive record population

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Assumption

Light capture pointers in the design, consult, and preflight skills are
sufficient to drive knowledge-record population — records get authored at the
moments they arise, without a heavier enforcement mechanism.

## Validation plan

Re-census after SL-215 (harvest surface) lands: compare record count and
`shapes` edge count against the baseline at SL-214 close. If in-flow capture
contributed ~nothing beyond harvest-time records, the pointers are inert and
this assumption is invalidated — the lever moves to gating or lint (IDE-009
territory).
