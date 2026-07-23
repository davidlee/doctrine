# Notes SL-215: Unified harvest surface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-24 · PHASE-05 · 2a1dc02d

### Produced
- PHASES 01–05 done — unified harvest surface: one owner doc + `/notes` entry, every other tail a citation (commits ded011695..2a1dc02d)
- shipped: `install/harvest.md` (harvest owner) · `install/templates/notes.md` (`## Harvest` stub) · 11 skill edits (code-review Cadence + 6 tail shrinks · notes entry point · handover/next consumers · dispatch/execute touchpoints)

### Learned
- dispatch-mechanics gotchas → `.doctrine/rfc/011/case-notes.md` (working-tree-free coord sync; prose-slice regression vacuity) — candidate memories at close

### Open
- DEC-004 — harvest shape: shared owner doc + `/notes` entry point
- DEC-005 — harvest output: single maintained freshness-stamped section
- DEC-006 — code-review cadence: model-tier gate + tripwires, arm-agnostic
- residual — adherence bar is a qualitative heuristic, no model-tier registry (design §6)
- residual — `## Harvest` staleness is honour-model, no enforcing verb (design §6)
