# ISS-020: Semantic/Actionability view toggle no-ops until next focus/depth change — viewMode change absent from renderView guard

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

In the web explorer (`web/map`), clicking the **Semantic** view button while in
Actionability view appears to do nothing — the graph stays on the actionability
render until some *other* event (focus or depth change) fires.

## Root cause

`renderView` (`web/map/src/app.ts`) renders the semantic entity-graph branch only
when one of `focusChanged || depthChanged || graphMissing || cmFocusChanged ||
cmCacheChanged` holds. A pure view-mode toggle changes none of these:
`graphMissing` is false because the actionability view already drew an `<svg>` into
`.graph-area`. So the guard skips, the semantic branch never runs, and the stale
actionability SVG remains. The actionability branch is unguarded (always renders),
which is why the *reverse* toggle (→ Actionability) works.

Surfaced 2026-06-18 during IMP-092 manual testing; pre-existing, most likely from
the SL-091 Vite/TypeScript port.

## Fix

Track the last-rendered view mode (`state.renderedViewMode`, mirroring the existing
`renderedCmFocus` idiom) and include `viewModeChanged` in the semantic-branch guard.
Reset the tracker at the end of `renderView`.

## Pointers

- `web/map/src/app.ts` — `renderView` change-detection (~L347) + semantic guard (~L429).
- `web/map/src/types.ts` / `state.ts` — AppState shape + init.
