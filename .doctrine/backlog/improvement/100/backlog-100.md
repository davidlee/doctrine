# IMP-100: Port pan/zoom + fit-to-viewport from semantic graph to concept map browser

## Context

SL-094 shipped pan/zoom + fit-large-graphs-to-viewport on the semantic graph
(Graphviz) web viewer. The concept map browser renders identically (DOT → SVG →
`injectHitRects` → `wireHandlers`) but never got the niceties — its SVG is
injected at native size with no transform wrapper, no viewport, no zoom/pan.

## What's already reusable (no design needed)

- `web/map/src/viewport.ts` — **pure, DOM-free**: `fitViewport`,
  `applyFocusChange`, `parseTransform`, `clampViewport`, `readSvgDims`. Fit math
  is free.
- `web/map/graph.css` — grab/grabbing cursor, `touch-action:none`.
- The concept map render path (`concept-map.ts` `renderDiagram`, ~line 152) is
  structurally a twin of the semantic graph path (`render.ts` ~line 698), so the
  zoom/pan wrapper slots into the same spot.

## The real work (the only non-trivial part)

The pan/zoom **wiring** (`render.ts:705-780`: transform-layer wrapper div +
wheel-to-zoom + mousedown-to-pan, ~75 lines) is currently **inline in
`render.ts`**, not extracted. DRY / no-parallel-implementation forbid copy-paste
into `concept-map.ts`.

→ Extract a shared `wireZoomPan(container, svgEl, opts)` helper (alongside
`viewport.ts`), then call it from **both** viewers.

## Constraints / gates

- **Behaviour-preservation gate**: the extraction touches `render.ts` (the
  recently-shipped semantic graph viewer). Existing render/viewport suites must
  stay green **unchanged** — they are the proof.
- No test framework on `web/map` frontend yet (see IMP-088) — pure `viewport.ts`
  logic is the testable seam; DOM wiring verified by hand / `/run`.

## Open decisions for the sketch (both have obvious defaults)

1. **Extract vs duplicate** → extract. (Project DRY rules decide this; not a real
   open question.)
2. **Viewport persistence across focus change** → default: fit-on-load only, no
   cross-focus persistence. The concept map has editing-UI churn; persistence is
   the semantic viewer's complication — skip it unless the sketch says otherwise.
3. **Mousedown-pan vs editing interactions** → concept map has edge-add/edit UI
   the semantic viewer lacks. Verify pan's `mousedown` doesn't swallow those; the
   existing `.doctrine-node` closest-guard (`render.ts:759`) covers node clicks,
   may need an analogous guard for edit affordances.

## Scope

Single subsystem (`web/map/src`). No new governance, architecture, or subsystem.
Assessed as backlog → sketch → do (not slice-worthy): mechanism already designed
and tested in SL-094; this is reuse + apply.

## Lineage

Sibling of IMP-094 (SL-094 residual: double-click-to-reset-zoom affordance) —
both descend from the SL-094 pan/zoom work. Related: IMP-085 (web frontend
code-quality), IMP-088 (web/map test framework).
