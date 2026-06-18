# Semantic graph: zoom, pan, and crop-on-bounds for DOT/Graphviz SVG view

## Context

ISS-021 — the DOT/Graphviz-rendered `<svg>` in `graphPane()` (`render.ts`) has no
zoom or pan controls. For large graphs (wide neighbourhood, deep depth), the
rendered SVG extends past its container bounding box, clipping content and making
navigation impractical.

IMP-092 already delivered `d3.zoom()`-based scroll-to-zoom, drag-to-pan, and
zoom-to-selected for the actionability (priority) graph in `priority.ts`. The DOT
view needs equivalent capability, but vanilla-SVG (no d3 dependency on this path).

## Scope & Objectives

1. **Scroll-wheel zoom** on the `.graph-area` container housing the DOT SVG.
2. **Drag-to-pan** (mousedown + move) over the same area.
3. **Crop-on-bounds** — the SVG must not visually overflow the `.graph-area`
   container. Overlaid `overflow: hidden` (or `clip`) on the container, plus the
   SVG having its own `viewBox`-preserving transform layer.

Where IMP-092 uses a `zoomLayer` `<g>` inside the SVG that `d3.zoom()` controls,
this will implement the equivalent with direct DOM event handlers on the
`.graph-area` container, modifying a CSS `transform` on a wrapper `<div>` inside
the container.

## Non-Goals

- Not touching the actionability/priority graph (`priority.ts`) — already done by
  IMP-092.
- No d3 dependency addition on the DOT rendering path.
- No coordinate system changes to `graphToDot()` or the DOT generation pipeline.
- No SVG hit-test or interaction changes beyond the zoom/pan layer.

## Summary

Vanilla zoom/pan on the graph area:
- Track `GraphViewport` (x, y, k) in app state.
- CSS transform wrapper `<div class="graph-transform-layer">` inside `.graph-area`.
- On `wheel`: adjust k (scale) around cursor position.
- On `mousedown` + `mousemove` on `.graph-area` (not on nodes): translate x, y.
- Container `overflow: hidden` clips overflow.
- Viewport persists across re-renders; focus-change centres with fit-floor clamp.
- Pinch-to-zoom (touch) is a nice-to-have but not required for close.

## Affected Code

- `web/map/src/render.ts` — `graphPane()` (L640+), wrapper creation, event wiring.
- `web/map/src/graph.css` — `.graph-area` restyle, `.graph-transform-layer`.
- `web/map/src/app.ts` — state: `graphViewport`, `lastRenderedFocusId`.
- `web/map/src/svg.ts` — may house pure helpers (`fitViewport`, etc) or new `viewport.ts`.

## Verification

- VT: unit test(s) for zoom/pan transform math (if factored to pure functions).
- VT: integration — render a large graph, scroll + drag, verify content moves
  within bounds and does not spill past the container edge.
- VA: dark-theme rendering (memory: edge contrast bump for DOT graphs on dark
  backgrounds).

## Follow-Ups

- Pinch-to-zoom (touch).
- Resize/reflow handling (re-fit when browser window size changes).
- Reset-to-fit affordance (double-click background to reset zoom).
- Click-vs-drag disambiguation on `.doctrine-node`.
- Zoom-to-selected (animate to a clicked node's centre) — parity with IMP-092's
  `zoomToNode`.
