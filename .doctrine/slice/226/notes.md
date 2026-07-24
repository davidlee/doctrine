# Notes SL-226: CLI graph emitter and ego-view

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-25 · all 4 phases landed on dispatch/226 · 08b8d643

### Produced

- SL-226 (scope, design.md locked, plan.toml/plan.md, 4 phase sheets)
- DEC-007, DEC-008, DEC-009 (foundational design decisions; bodies in
  record-NNN.md)
- RV-298 (design review, codex, done — 14/14 verified)
- IDE-043 (parent idea; fulfils edge), ISS-235 (link error UX), ISS-237
  (web dot.ts shape defect, found via RV-298 F-10)
- RFC-011 case notes ×5 (sess-graph-cli + fable-sl226-a dispatch drive)
- IMPLEMENTATION (dispatch/226, pi arm, serial P01→P04):
  - PHASE-01 fcbacfd4 — CatalogGraph filter pipeline (filter_kinds/filter_label/
    exclude_memory/drop_isolated/contains + filter_nodes DRY helper)
  - PHASE-02 7ecfa8f5 — neighbourhood(focus,depth) undirected BFS
  - PHASE-03 f5a7c075 (+layering gov 0f93f630) — catalog::dot emitter + dot_escape
    lift from concept_map
  - PHASE-04 08b8d643 — `doctrine graph` CLI verb (build_graph_output seam)
  - dispatch base refreshed once mid-drive (f39588f) to absorb SL-204's landing.

### Learned

- Design D1–D15 + R1–R4 in design.md §7/§8 are canon; D9/D11/D13 came from
  the external pass and reverse earlier internal choices — trust design.md,
  not intermediate session prose.
- dot.ts `shape="box,rounded"` is invalid Graphviz (ISS-237); Rust tables
  deliberately diverge (D10). Confirmed at VA-1: real-corpus render to SVG has
  no unknown-shape warnings.
- Adding a sub-file under a mixed layering umbrella needs a `"module::file"` row
  (catalog::dot=engine) — gate-critical (mem.pattern.lint.module-split-needs-
  layering-entry). A new command-tier file under the uniform `commands` umbrella
  needs NO row (verified empirically for commands::graph).

### Open

- Audited on **RV-301** (reconciliation, done · 4 findings · 0 blockers). All
  acceptance re-verified green on a fresh binary (VTs 7/7, VA-1 reproduced,
  clippy zero). Reconciliation Brief + Synthesis on RV-301. Remaining lifecycle:
  /reconcile → /close.

Findings settled (were the two Open items above + two conformance/governance):

- **F-1 style-table DEVIATION** (match fns vs design §5.2/§5.3 slice-consts):
  adjudicated (user) **accept the code, reconcile canon** → reconcile edits
  design.md §5.2/§5.3 prose to the as-built `node_style()`/`edge_color()` +
  `DEFAULT_*` form. No code churn.
- **F-2 VT-1 keyword-provenance** (NODE_STYLES/EDGE_COLORS now comment-only):
  tolerated — behavioural assertions strong, plan VT-1 immutable, no
  POL-002-compliant vtgate fix (IMP-228 blind spot for touched files).
- **VA-1 (PHASE-04):** DISCHARGED — reproduced on the fresh audit binary:
  `graph SL-226 --depth 2 --format dot | dot -Tsvg` → 29,118-byte SVG, empty
  graph-side stderr, zero `shape="box,rounded"` (D10). Lone dot stderr is the
  env `Fontconfig` artifact, not emitter-attributable.
- **F-3 conformance undelivered layering.toml:** pi-arm topology artifact (row
  rode gov commit 0f93f630, outside worker source-deltas); delivered in the
  bundle, integrates to main. Close-time check: confirm catalog::dot=engine on
  main post-integrate (gate-critical).
- **F-4 PRD-016 §2 boundary:** routed to reconcile as a governance REV
  (pre-declared deferral). Follow-ons live in IDE-043 (render/mermaid/d2,
  coverage & actionability views).
