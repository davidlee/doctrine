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

- **DEVIATION for audit/reconcile:** catalog::dot renders the design §5.3
  named-slice-const style tables (`NODE_STYLES: &[..]` / `EDGE_COLORS: &[..]`) as
  match-lookup fns `node_style`/`edge_color` + `DEFAULT_*` consts instead.
  Functionally equivalent, values correct vs dot.ts, STD-001 intent met; but the
  VT-1 keyword literals then live only in test comments (vtgate raw-byte match
  still Passes). Adjudicate: keep, or follow-up refactor to the literal
  slice-const form. (Recorded in phase-03 sheet Findings — disposable — hence
  lifted here.)
- **VA-1 (PHASE-04):** DISCHARGEABLE — graphviz `dot` IS present in the jail.
  `graph SL-226 --depth 2 --format dot | dot -Tsvg` → valid 29KB SVG, empty
  graph-side stderr, NO shape/syntax warnings (D10). Lone dot stderr is an
  environmental `Fontconfig` "cannot load default config" (missing jail font
  config), NOT emitter-attributable. Audit to formally close VA-1 on this
  evidence.
- Deferred to reconcile: PRD-016 §2 interchange boundary sentence revisit.
  Follow-ons live in IDE-043 (render/mermaid/d2, coverage & actionability views).
