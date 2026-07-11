# Notes SL-216: Per-component gauge scope

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — per-component placement (2026-07-12, 89066bd4)

- Baseline (EN-2) at a36b2e79: comparison 100 / priority 168 green.
- TDD receipt: six RED against the shipped global trigger — s2/s8 re-pins,
  mixed_corpus_island (f got 1.2500 want 1.3333), singleton_island (s1 got
  0.6667 want 1.0000), p12 first-anchor flip (P moved), disconnect membership
  (["w"] not ["w","z"]). p12 cases 2/3 (anchored→island, island→anchored
  freeze) were already green pre-impl, as predicted — shipped P7+P5 island
  values don't depend on the neighbour component's shape; they are
  strengthening, not movers.
- Implementation: `components()` (BFS over out ∪ inn; `unvisited.pop_first()`
  seed is provably each component's min member, so the vec is min-ordered for
  free) + `place()` outer loop; old body became `place_component()` verbatim.
  Whole-graph adjacencies passed per component — safe by weak-connectivity
  (every neighbour of a member is a member).
- Singleton-island fixture: `Response::Equal` judgement pair merges to an
  edge-free class through the real compile seam (compile.rs:192) — no
  hand-built ConstraintSet needed.
- findings.rs membership test rides `crate::comparison::{compile, project}`
  (all pub(crate) re-exports) with a local Judgement builder — findings' own
  fixtures are disk-seeding, wrong shape for this.
- Golden audit sweep: s2/s8 confirmed the only value-pinned multi-component
  scenarios. p10's generated multi-component anchor-free ledgers assert
  intra-edge order + no-NaN only — invariant under the change (stayed green).
  p12's original anchored↔anchored witness retained verbatim (RV-267 F-4).
- e2e_compare_inference: green UNEDITED (asserts explain shape for the island
  SINK + finding presence, no island values). Its ISS-050 "Projected, not
  Gauge" comment (tests/e2e_compare_inference.rs:458-463) is now stale —
  PHASE-02 sweep target, deliberately untouched here.
- Module note: only the mechanics line (P8 "anchor-free component") moved;
  the "Gauge scope" adjudication ¶ (project.rs:24-37) still carries the
  follows-the-prototype framing — PHASE-02 EX-2 rewrites it.
- Verification: 4043 tests pass / 0 fail, clippy zero warnings, fmt clean.
  `just gate` / `doctrine check gate` blocked by PRE-EXISTING jail env
  breakage in lint-js (node_modules eslint shebang `/usr/bin/env` absent in
  jail) — web/map untouched by this slice; rust gate fully clean. Flag for
  host-side gate run or jail fix.
