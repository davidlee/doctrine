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

---

# Sketch (locked 2026-06-19)

## Key discovery — they share the container

Both viewers render into the **same `.graph-area` element** (`index.html:88`;
`app.ts` dispatches `graphPane({container: graphArea})` for entities,
`renderCmGraph(graphArea) → renderDiagram({container: graphArea})` for concept
maps). Consequences:

- **No new CSS.** `.graph-area` (cursor grab, `touch-action:none`),
  `.graph-area.grabbing`, `.graph-transform-layer` already exist in `graph.css`
  and apply to whatever SVG sits in that element.
- The wheel/mousedown handlers wired in `graphPane` are wired **on that same
  element**, guarded by `container.dataset.zoomWired` (idempotent) and a
  `.graph-transform-layer`-null cross-mode guard. They already no-op for any
  non-wrapped SVG.
- Gap: a user who lands directly on a concept map (never viewed an entity graph)
  never triggered the wiring → `zoomWired` unset → no handlers. So the concept
  map path must also drive the wiring. This is exactly why extraction (not
  copy-paste) is right.

## Plan

1. **New DOM-seam module `web/map/src/zoompan.ts`** (over the existing pure
   `viewport.ts` — do **not** pollute `viewport.ts`, it stays DOM-free):

   ```ts
   export interface MountZoomPanOpts {
     initialViewport?: GraphViewport | null;
     focusChanged?: boolean;
     onViewportChange?: (vp: GraphViewport) => void;
   }
   export function mountZoomPan(
     container: HTMLElement, svgEl: SVGSVGElement, opts: MountZoomPanOpts,
   ): void
   ```

   Body = lift of `render.ts:705-780` verbatim-ish: `readSvgDims` → `minK` via
   `fitViewport` → target vp (`fitViewport` / `applyFocusChange` / preserve) →
   wrap SVG in `.graph-transform-layer` + `dataset.minK` → wire wheel + mousedown
   idempotently.

2. **`graphPane`** (`render.ts`): replace the `705-780` block with one
   `mountZoomPan(container, svgEl, { initialViewport, focusChanged,
   onViewportChange })` call. Net behaviour unchanged.

3. **`renderDiagram`** (`concept-map.ts`, after `wireHandlers` ~line 172): add
   `mountZoomPan(container, svgEl, {})` → fit-on-load, no persistence,
   `onViewportChange` omitted. No new `CmDiagramOpts` fields.

## Decisions resolved

1. **Extract vs duplicate** → extract to `zoompan.ts`. (DRY / no-parallel-impl.)
2. **Persistence across concept-map focus change** → **fit-on-load only**, no
   persistence. Concept map omits `onViewportChange`; semantic graph keeps its
   `state.graphViewport` persistence unchanged.
   - **Refinement required**: the handlers currently close over the *first*
     `onViewportChange` at wire time (`render.ts:751,771`). Since the handler is
     wired once but serves both modes through the shared container, a stale
     closure would write concept-map pans into `state.graphViewport`. Fix in the
     extraction: store the active callback on a `container` property, set per
     `mountZoomPan` call, read live in the handlers. Removes the latent
     cross-mode bug **and** lets concept map pass a no-op cleanly.
3. **Mousedown-pan vs editing UI** → **no conflict, no extra guard.** Edge
   add/remove live in separate containers (`.cm-edge-table`, `.cm-add-edge-form`,
   `index.html:130-131`) outside `.graph-area`. The only in-SVG edit interaction
   is node click → `startRenameNode`, and the existing `.doctrine-node`
   closest-guard (`render.ts:759`) already excludes node clicks from pan.

## Gate / verification

- **Behaviour-preservation gate**: `viewport.ts` is untouched → `viewport.test.ts`
  (vitest) stays green by construction = the proof for the moved math. The DOM
  wiring has no unit harness (no `render.test.ts`; see IMP-088) → verify by hand /
  `/run`:
  1. entity graph — zoom/pan/fit still work, persistence across focus survives;
  2. concept map — fit-on-load, zoom + pan work;
  3. **direct-land on a concept map** (no prior entity view) — handlers wire and
     work (the `zoomWired` gap above);
  4. toggle concept↔entity in one session — no stale-state cross-write.

## Effort

~1 new file (~60 lines lifted, not net-new), 2 call-site edits, **0 CSS, 0 new
test files** (gate rides existing `viewport.test.ts`). Half-day. Confirms the
backlog → sketch → do call — no slice.
