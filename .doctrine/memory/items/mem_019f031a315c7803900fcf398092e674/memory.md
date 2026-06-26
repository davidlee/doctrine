# Conformance undeclared noise from boundary start-oid pollution

Audit: huge undeclared conformance noise often means polluted boundary start-oids, not scope creep — fix with record-delta(parent..feat).

## Why

The solo phase-binding (`src/state.rs` `capture_phase_boundary`, SL-147) stamps a
phase's `code_start_oid` at the **in_progress** flip and `code_end_oid` at
**completed**. If `edge` advances — other slices land — between a phase starting
and its (rebased) land, those foreign commits fall inside `start..end`, and
`slice conformance` attributes them to the phase. A slice developed across a busy
period can show dozens of undeclared paths that are nothing to do with it.

## How to apply

When `slice conformance <id>` reports a large `undeclared` set during audit, do
**not** read it as scope creep until you check the boundary registry
(`.doctrine/state/slice/<id>/boundaries.toml`). For each phase, compare the
recorded range against the actual feat commit:

- `git log --oneline <start>..<end>` — if it lists commits from other slices, the
  range is polluted.
- A funnel/solo phase usually lands as exactly **one non-merge feat commit**; the
  true delta is `feat^..feat`.
- Correct it: `doctrine slice record-delta <id> PHASE-NN --start <feat-parent>
  --end <feat>` (the sanctioned escape hatch — "correct a recorded range"), then
  re-run conformance. Legitimate residual undeclared = test files + doctrine
  authored state, since design-target selectors typically name only source paths.

Root-cause work tracked in IMP-175. Related: [[mem_019f025ee2027bf281f7d3a013bc9415]]
(stale binary strips BoundaryRow fields).
