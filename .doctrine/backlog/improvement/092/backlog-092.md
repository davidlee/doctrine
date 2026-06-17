# IMP-092: Actionability graph: wire zoom-to-selected on click + fit-to-content viewBox and free pan/zoom

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The actionability (priority) graph in the web explorer (`web/map`, SL-072/073
frontend; nodes from `/api/survey` → `src/priority/view.rs`) renders a single
oversized static DAG. It loads at a fixed window, never reframes on interaction,
and overflows/clips off-screen — unusable for navigation. Surfaced while tackling
the SL-081 graph UI after the Vite/TypeScript port.

Renderer: `web/map/src/priority.ts` (d3-dag `sugiyama` layout → hand-built SVG).

## Root cause (spike)

1. **Zoom-to-selected is dead wiring.** The renderer already has the machinery —
   `priorityZoomId` drives a `zoomLayer` `translate(...) scale(5)` that centres the
   target (`priority.ts:137-144`), plus a `priority-node--zoom` highlight class. But
   `state.priorityZoomId` is **only ever assigned `null`**: init `null`
   (`state.ts:43`), reset on view-switch (`app.ts:165`). Nothing sets it to a node
   id, so the `if (zoomId !== null)` branch never fires. Node click →
   `onNodeClick: goto` → `goto(id)` → `setFocus(id, depth)` (`app.ts:40-42`): moves
   the focus/detail pane, never the zoom target, never a zoomed re-render.

2. **Fixed viewBox, no fit.** `svg viewBox="0 0 960 600"` is hardcoded
   (`priority.ts:110`) with `width/height=100%`. Sugiyama lays the full DAG far
   beyond 960×600, so only the top-left window shows; the rest overflows/clips.
   No fit-to-content, no global pan/zoom. (`d3-zoom`/`d3-brush` already vendored
   in `node_modules`.)

3. **Off-path completed nodes already filtered server-side** (`view.rs`): terminal
   nodes are excluded unless they are a prerequisite on a live path. No change
   needed there.

## Scope

- **A — wire zoom-on-click (the core ask):** in actionability view, node click sets
  `state.priorityZoomId = id` + re-renders; clicking empty space clears it.
- **B — fit + free pan/zoom:** compute the layout bounding box → set the viewBox to
  fit the graph on load; add `d3-zoom` drag-pan + wheel-zoom to roam. Kills the
  overflow.

Renderer scaffolding for A largely exists; B leans on the already-vendored d3-zoom.

## Governance

Lightweight path approved by user (2026-06-18): backlog item + quick design sketch
→ accept → implement (TDD) → close. Not sliced.

## Pointers

- `web/map/src/priority.ts` — layout + SVG render, the zoom transform branch.
- `web/map/src/app.ts` — `goto`, `renderGraph` call site (`onNodeClick`), view wiring.
- `web/map/src/state.ts` / `types.ts` — `priorityZoomId` state field.
- `src/priority/view.rs` — server survey/actionability view (node inclusion).

Related web/map items: IMP-085 (code-quality hardening), IMP-088 (test framework),
IMP-090 (style.css), IMP-086 (vendor pinning).
