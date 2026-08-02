# Notes SL-243: Spec anchor map

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-02 · stage:design (no run started) · 946d7803

### Produced

- SL-243 scope (this slice) — design commitments settled pre-slice, see `slice-243.md`
- CHR-052 — SPEC-002's nine `[[source]]` anchors + its `## Source anchors` prose
- IMP-381 — spec-coverage census; map at `.doctrine/state/imp-381-coverage-map-criterion-lineage.md`

### Learned

- mem.pattern.spec.read-anchors-via-json-not-grep — the read seam this slice
  rides, and the two ways a raw TOML grep inflates the count
- Baseline figures, via that seam: 48 specs · 81 anchors · 0 non-resolving ·
  29,310 non-test `src/` loc (27%) dark · largest dark `src/review.rs` @ 2,824

### Open

No DEC / QUE / ASM minted — the open items are design questions held in
`slice-243.md`, not knowledge records (one fact, one artefact):

1. Where the engine spec sits — new component under SPEC-006/PRD-012 vs amending
   SPEC-017, which owns the anchor field model.
2. New PRD-012 requirements for the report (intent present via REQ-085/REQ-088;
   no requirement covers the report itself).
3. Which spec owns the identifier-form convention, where IMP-316 leg 2 overlaps
   this slice's `qualified_name` corroboration. Requirement membership is not
   exclusive, so this is a preference for clean boundaries, not a blocker.
