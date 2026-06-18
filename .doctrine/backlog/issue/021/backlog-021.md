# ISS-021: Semantic/graphviz view overflows bounding box — needs zoom/pan/crop

## Problem

The semantic entity graph (DOT/Graphviz-rendered `<svg>` in `graphPane()`) has no
zoom or pan controls. For large graphs (wide neighbourhood, deep depth), the
rendered SVG extends past its container bounding box, clipping content and making
navigation impractical.

The d3 actionability graph (IMP-092) already has `d3.zoom()`-based
scroll-to-zoom and drag-to-pan wired in. The DOT-rendered graph must match that
capability, plus crop/constrain the SVG to the container bounds.

## Expected behaviour

1. **Scroll-wheel zoom** on the semantic graph pane
2. **Drag to pan** (mousedown + move)
3. **Crop-on-bounds** — the SVG must not visually overflow its container

## Affected code

- `web/map/src/render.ts` — `graphPane()` (the SVG injection site, ~L669)
- `web/map/src/svg.ts` — may need zoom/pan helpers
- Possibly CSS for overflow/clip on the `.graph-pane` container

## Prior art

- IMP-092: d3 actionability graph has `d3.zoom()` with `.call(zoom.transform, …)`
  for fit-to-content + free pan/zoom. The DOT view needs equivalent vanilla-SVG
  (no d3 dependency for this path).
