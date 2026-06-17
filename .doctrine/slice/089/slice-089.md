# Backlog filtering, actionability graph, and prioritization views in the Map Explorer

## Context

SL-073 (Map Frontend) and SL-083 (modular decomposition) delivered the interactive
browser explorer. It shows entity relationship graphs (semantic edges: governed_by,
specs, slices, etc.) and provides kind-level filtering. The CLI has a rich
prioritization surface (`survey`, `next`, `explain`, `blockers`) that computes
actionability — which work items are eligible, blocked, and in what order — but
none of this is visible in the web UI.

The user wants to close that gap: make the Map Explorer the visualisation surface
for *why* work is ordered the way it is, not just *what* relates to what.

IMP-047 (trinary actionability) is a foundational backend item that splits
"blocked" from "not eligible" — the concepts this slice needs to visualise.
It is a dependency only in the sense that the data model it defines is what we
render; this slice does not implement IMP-047.

## Scope & Objectives

1. **Backlog kind filtering** — Split the monolithic `ISS/IMP/CHR/RSK` checkbox
   into individual kind checkboxes (ISS, IMP, CHR, RSK), so users can isolate,
   e.g., only improvements or only risks.

2. **Actionability graph layer** — The `GET /api/survey` endpoint returns the
   canonical actionability graph (nodes with server-computed ranks + `needs`/`after`
   edges). An entity is:
   - **actionable** = eligible (open/ready status) AND unblocked (all `needs`
     targets are closed/completed)
   - **blocked** = has at least one unsatisfied `needs` target
   - **terminal** = closed, done, resolved, superseded, archived (excluded from
     ordering)

   Render the actionability view as a D3 graph with:
   - `needs` edges (hard prerequisites) — solid red arrows
   - `after` edges (soft sequence) — dashed amber arrows
   - Consequence (inbound dep count) as node badge
   - Topological ordering: nodes laid out top-to-bottom by server-computed rank

3. **Prioritization graph** — Extend the actionability view beyond backlog kinds
   to include slices (SL). This is the "whole work graph" — all work-like entities
   with their dep/seq edges and ordering. Same rendering rules as (2).

4. **View toggle** — Add a toggle control (radio or segmented button) in the graph
   area header that switches between:
   - **Semantic** — all relation edges (current behaviour; default)
   - **Actionability** — only `needs`/`after` edges, with dep-rank layout and
     blocking/actionable status indicators

   The toggle is per-session (not persisted). When switching to actionability view
   for a non-work entity (ADR, SPEC, REQ, etc.), show a message: "This entity has
   no dep/seq edges — switch to a work entity (SL/backlog) or use Semantic view."

## Non-Goals

- IMP-047 trinary actionability backend implementation
- IMP-026 backlog triggers actionability mask
- IMP-021 filter by risk facet axes (likelihood/impact)
- Changing CLI `survey`/`next`/`explain` commands
- Adding new relation types or labels
- Persisting the view toggle preference across sessions

## Affected Surface

- `web/map/index.html` — kind checkboxes, view toggle control
- `web/map/model.js` — `viewMode`, `actionabilityView` state, `setActionabilityView()`
- `web/map/app.js` — view toggle wiring, actionability graph rendering dispatch
- `web/map/render.js` — actionability node styling, view toggle `--active` class
- `web/map/style.css` — view toggle styles, actionability indicators
- `src/map_server/routes.rs` — `GET /api/survey` handler (new), refresh via `DataStores`
- `src/map_server/state.rs` — `DataStores` wrapper under single `RwLock`
- `src/map_server/mod.rs` — build `DataStores` at startup (catalog + priority_graph + graph)
- `src/priority/surface.rs` — extract `survey_for_map`, add `survey_view_for_map`
- `src/priority/view.rs` — add `ActionabilityView`, `ActionabilityNode`, `ActionabilityEdge` types

## Summary

A single-view toggle and kind-filter refinement in the SPA. The `GET /api/survey`
endpoint (new) returns the full actionability graph shape: nodes with server-computed
ranks plus `needs` and `after` edges. The priority engine owns truth — the frontend is
a render-only consumer. D3 sugiyama renders the actionability graph; Graphviz DOT
remains for the semantic view.

## Follow-Ups

- IMP-047: once trinary actionability ships, update the client-side computation
  to use `ineligible` vs `blocked` vs `actionable` instead of the current binary
  model
- Persist view toggle preference in localStorage
