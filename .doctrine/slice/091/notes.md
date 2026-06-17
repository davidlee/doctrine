# Notes SL-091: Frontend dev server with TypeScript, HMR, and hot reload

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-10 — app.ts entry point (002de17d)

- Converted 171-line app.js IIFE to 925-line app.ts ES module.
- `setViewMode` renamed to `setPageMode` (render.ts conversion).
- `window.renderCmDiagnostics` → `export function renderCmDiagnostics()`.
- `priority.renderGraph` TS API differs from JS: `view` param (not `layout`),
  no hover callbacks, no zoom toggle (existing priority.ts shape).
- `CmDiagramOpts.cm` typed `ConceptMap` but receives `CmNeighbourhood` —
  `as unknown as ConceptMap` cast as workaround (no concept-map.ts changes).
- All innerHTML assignments avoided via `el()` + `replaceChildren()` to satisfy
  eslint `no-restricted-syntax`. Stale pipeline: `graphArea.innerHTML` →
  `replaceChildren(el(...))`.
- app.js still present, all .js files retained.
