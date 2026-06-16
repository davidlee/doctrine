# Notes SL-076: Load concept maps into the Map Explorer and ship a web authoring surface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-06 (completed)

- **Diagnostic variant serialisation**: Rust `#[derive(Serialize)]` on
  `ConceptMapDiagnostic` produces tagged-enum JSON objects like
  `{"CanonicalNodeCollision": {"key":"foo","label":"Foo",...}}`. The JS
  `formatDiagnostic()` function dispatches on `Object.keys(d)[0]` to extract
  the variant name, then accesses nested fields.
- **MalformedLine/EmptyLabel now included in GET response**: The route was
  updated (during earlier phases) to merge parse-time diagnostics with
  `check()` diagnostics — the renderer handles all 8 variants, though
  DuplicateEdge only surfaces via parse-time (not from `check()`).
- **Window export for IIFE functions**: `renderCmDiagnostics()` lives inside
  app.js's `(function() { 'use strict'; ... })()` IIFE, so it must be
  explicitly exported via `window.renderCmDiagnostics = renderCmDiagnostics;`
  for the test harness to call it. This pattern applies to any app.js function
  that needs test coverage.
- **Cache eviction strategy**: On focus change away from a CM, the old CM's
  cache entry is deleted (`conceptMapCache.delete(prevFocusId)`). Combined with
  the full clear on refresh, this ensures stale CM data is never displayed.
- **Dark-mode diagnostics colours**: Chosen for legibility but not WCAG-tested.
  Acceptable for auxiliary content. Colours: bg `#2a2410`, border `#8b6914`,
  heading `#e0c060`, item `#c0a040`, item-border `#3a3410`.
- **Phase completed without Rust changes** — pure frontend (JS + CSS).
