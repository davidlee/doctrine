# SL-094 Design: Semantic graph zoom, pan, and crop-on-bounds

## Summary

Add vanilla-SVG zoom/pan to the DOT/Graphviz `graphPane()` view. A CSS-transform
wrapper inside `.graph-area` applies `translate()` + `scale()` driven by wheel
(zooom) and mousedown/mousemove (pan) events. `.graph-area` gets `overflow:
hidden` and `position: relative` — the clip boundary.

Viewport is persisted across re-renders. On focus change, the viewport
centre-adjusts with a fit-floor clamp (k never zooms further out than the
fit-to-container scale for the new graph).

## Container mechanics

### `.graph-area` (target CSS)

```css
.graph-area {
  overflow: hidden;
  position: relative;
  border: 1px solid var(--border);
  height: 70vh;
  min-height: 400px;
  margin-bottom: 1rem;
  cursor: grab;
  touch-action: none;   /* prevent browser scroll-claim so drag-to-pan works on touch */
}
.graph-area.grabbing {
  cursor: grabbing;
}
```

Flex centering is removed — the transform wrapper controls position.

### Wrapper

```html
<div class="graph-area">
  <div class="graph-transform-layer" style="transform: translate(x px, y px) scale(k); transform-origin: 0 0;">
    <svg>…</svg>
  </div>
</div>
```

`transform-origin: 0 0` so `translate(x, y)` is relative to the container's
top-left corner — the same convention as the `{x, y}` state fields.

### Why CSS transform (not SVG transform)

The DOT SVG arrives as an opaque string from Graphviz. Parsing it to wrap
content in a `<g>` or modify root attributes is fragile and coupling-heavy. A
CSS transform on a parent wrapper is SVG-agnostic: the SVG internals (hit-rects
in `svg.ts`, focus/hover classes, `dimLegend`) are unaffected.

## State model

```ts
interface GraphViewport {
  x: number; // translate-x in px, relative to container top-left
  y: number; // translate-y in px
  k: number; // scale (1 = 100%, clamped to [minK, 10])
}
```

`app.ts` state gains:
- `graphViewport: GraphViewport | null` — null = first render (fit-to-content)
- `lastRenderedFocusId: string | null`

## Viewport rules

| Scenario | `k` | `x, y` |
|---|---|---|
| First render (viewport null) | `minK` = fit-to-container | centred: `(cw - sw*k)/2`, `(ch - sh*k)/2` |
| Same focus, re-render | restore prior `k` | restore prior `x, y` |
| Focus changed, `priorK >= newMinK` | `priorK` (unchanged) | centre new graph at `priorK` |
| Focus changed, `priorK < newMinK` | `newMinK` (clamped up) | centre new graph at `newMinK` |

`minK` = `min(containerWidth / svgWidth, containerHeight / svgHeight)` —
computed from the injected SVG's dimensions after render. Upper bound fixed at
`10`.

## SVG dimension reading

After injection, read the SVG's rendered dimensions via
`svgEl.getBoundingClientRect()` — not `width`/`height` attributes.
Graphviz outputs in points (pt), not pixels; `baseVal` would give wrong values.
Guard against zero-size SVG: if `rect.width <= 0 || rect.height <= 0`, use
`minK = 1`.

```ts
function readSvgDims(svgEl: SVGSVGElement): { w: number; h: number } {
  const r = svgEl.getBoundingClientRect();
  return { w: r.width, h: r.height };
}
```

## Pure helpers (testable)

Extracted to `web/map/src/svg.ts` (or a new `viewport.ts` if `svg.ts` becomes
unwieldy):

```ts
function fitViewport(svgW: number, svgH: number, cw: number, ch: number): GraphViewport
function applyFocusChange(vp: GraphViewport | null, minK: number, svgW: number, svgH: number, cw: number, ch: number): GraphViewport
function clampViewport(vp: GraphViewport, minK: number, maxK: number): GraphViewport
```

## Event handling

### Wheel (zoom)

- `wheel` event on `.graph-area`
- `event.preventDefault()` to suppress page scroll
- **Normalize `deltaY` via `deltaMode`** before applying scale factor:
  `let delta = event.deltaMode === 1 ? event.deltaY * 16 : event.deltaY;`
  (Chrome emits pixels, Firefox emits lines; this normalizes to ~pixels.)
  Cap `|delta|` to 40 to guard against inertial-scroll spikes from trackpads.
- Scale delta: `newK = clamp(k * (1 - delta * 0.002), minK, maxK)`
- Cursor-relative: `newX = cx - (newK/k)*(cx - x)`, same for `y`
- `.ctrlKey` — potential future pinch-to-zoom hook, not wired now

### Drag (pan)

- `mousedown` on `.graph-area` (NOT on `.doctrine-node` — nodes handled by
  `svg.ts`): record origin, add `mousemove`/`mouseup` on `document`
- `mouseup` listener uses `{ once: true }` so even if the mouse is released
  outside the browser viewport, the next mousedown starts fresh (no stale
  listener accumulation). On mouseup: remove `mousemove` listener, remove
  `grabbing` class
- Add class `grabbing` to `.graph-area` → `cursor: grabbing`
- `mousemove`: `vp.x += (e.clientX - origin.x) / vp.k` (undo scale for 1:1
  track), update origin

### Cross-mode guard

`.graph-area` is shared across three renderers (DOT graph, actionability d3.zoom,
concept-map). The zoom/pan handlers must not interfere with other view modes.
Each handler gates on the DOM presence of `.graph-transform-layer`:
`if (!container.querySelector('.graph-transform-layer')) return;` — so wheel
and drag are no-ops when another view is active.

### Node click vs drag

Existing `.doctrine-node` click handlers (`svg.ts` → `wireHandlers`) fire on
`click`. Mousedown on a node does NOT start a pan (the mousedown handler checks
`e.target.closest('.doctrine-node')`). A small drag on a node still fires the
node's click — this is existing behaviour, not regressed. Explicit
click-vs-drag disambiguation (suppressing node click after a drag) is deferred
to a follow-up.

## Integration: `graphPane()`

### `GraphPaneOpts` additions

```ts
initialViewport?: GraphViewport | null;
focusChanged: boolean;
onViewportChange?: (vp: GraphViewport) => void;
```

### Flow

1. Inject SVG via `renderDot()` + DOMPurify as before
2. Read SVG dimensions → compute `minK`
3. Determine target viewport:
   - `initialViewport` is null → `fitViewport()`
   - `focusChanged` → `applyFocusChange(initialViewport, minK, …)`
   - else → restore `initialViewport` as-is
4. Create `.graph-transform-layer` wrapper (fresh each render — `container.innerHTML = ''` clears the prior one), apply CSS transform
5. Wire wheel + mousedown handlers on `.graph-area` once, guarded with `container.dataset.zoomWired` to avoid duplicate listeners on re-render
6. On every viewport mutation → call `onViewportChange` for persistence

### `app.ts` integration

```ts
// state additions
graphViewport: GraphViewport | null = null
lastRenderedFocusId: string | null = null

// in render(), before graphPane():
const focusChanged = state.focusId !== state.lastRenderedFocusId
graphPane({
  // …existing…
  initialViewport: state.graphViewport,
  focusChanged,
  onViewportChange: (newVp) => { state.graphViewport = newVp },
})
state.lastRenderedFocusId = state.focusId
```

## Affected files

| File | Change |
|---|---|
| `web/map/src/render.ts` | `graphPane()`: create wrapper, wire events, viewport logic. New helpers or imports. `GraphPaneOpts` extended. |
| `web/map/src/graph.css` | `.graph-area` restyle, `.graph-transform-layer`, `.grabbing` |
| `web/map/src/app.ts` | State: `graphViewport`, `lastRenderedFocusId`. Pass new opts to `graphPane()`. |
| `web/map/src/svg.ts` | Possibly house `GraphViewport` type + pure helpers; or new `viewport.ts`. |
| `web/map/src/priority.ts` | No change — this is the IMP-092 d3 path |
| `web/map/src/dot.ts` | No change |
| `web/map/src/model.ts` | No change |
| `web/map/src/api.ts` | No change |

## Verification

| ID | Kind | Test |
|---|---|---|
| VT-1 | `fitViewport` | Unit: SVG smaller → `{k:1}`, SVG wider → scaled+centered |
| VT-2 | `applyFocusChange` | Unit: prior k≥minK → preserved, k<minK → clamped, null → fit |
| VT-3 | `clampViewport` | Unit: below minK → minK, above maxK → maxK |
| VT-4 | zoom/pan render | Integration: render large graph, scroll → zoom, drag → pan, no overflow |
| VT-5 | viewport persistence | Integration: zoom in, change depth → viewport restored |
| VT-6 | focus-change centre | Integration: zoomed in on node A, click node B → centred on B at same scale |
| VA-1 | dark theme | Agent: DOT edge contrast still legible after zoom |

## Design decisions

| ID | Decision | Rationale |
|---|---|---|
| D1 | CSS transform wrapper, not SVG transform | DOT SVG is an opaque string; CSS transform avoids fragile parsing |
| D2 | Viewport in app state, not graphPane local | Survives re-renders; matches priority.ts pattern |
| D3 | minK = fit-to-container, not 0.1 | Prevents zooming so far out you lose the graph entirely |
| D4 | Focus change centres new graph | User navigated to a different entity; they want to see its neighbourhood |
| D5 | Same-focus restores exact viewport | User adjusted zoom/pan for a reason; depth changes shouldn't discard it. Edge case: depth increase changes graph topology + coordinate space, so the prior (x,y,k) may point to empty space. User can pan back or change focus (which triggers centre-on-focus). Re-centre-on-depth-change is a plausible follow-up but not required for close. |

## Open questions

- Pinch-to-zoom (touch) — deferred to follow-up. CSS cursor should switch to
  `grab` on touch devices too.
- Reset-to-fit affordance (double-click background?) — follow-up.
- Click-vs-drag disambiguation on `.doctrine-node` — follow-up.
- Resize/reflow: `minK` changes when the browser window resizes. A `resize`
  handler that re-fits if current `k` is below (or far above) the new `minK` is
  deferred to a follow-up.
