# Notes SL-226: CLI graph emitter and ego-view

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-24 · ready (pre-PHASE-01) · 53548814

### Produced

- SL-226 (scope, design.md locked, plan.toml/plan.md, 4 phase sheets)
- DEC-007, DEC-008, DEC-009 (foundational design decisions; bodies in
  record-NNN.md)
- RV-298 (design review, codex, done — 14/14 verified)
- IDE-043 (parent idea; fulfils edge), ISS-235 (link error UX), ISS-237
  (web dot.ts shape defect, found via RV-298 F-10)
- RFC-011 case notes ×4 (sess-graph-cli)

### Learned

- Design D1–D15 + R1–R4 in design.md §7/§8 are canon; D9/D11/D13 came from
  the external pass and reverse earlier internal choices — trust design.md,
  not intermediate session prose.
- dot.ts `shape="box,rounded"` is invalid Graphviz (ISS-237); Rust tables
  deliberately diverge (D10).

### Open

- None blocking (design §6). Deferred to reconcile: PRD-016 §2 interchange
  boundary sentence revisit. Follow-ons live in IDE-043 (render/mermaid/d2,
  coverage & actionability views).
