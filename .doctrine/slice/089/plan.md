# Implementation Plan SL-089: Backlog filtering, actionability graph, and prioritization views in the Map Explorer

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-089 delivers three capabilities to the Map Explorer: individual backlog kind
filtering, an actionability graph view (D3 sugiyama, server-computed ranks,
needs/after edges), and a semantic ↔ actionability view toggle. The priority
engine owns truth — the frontend is a render-only consumer.

Five phases, ordered by dependency.

## Sequencing & Rationale

**PHASE-01** (backend types + surface) comes first because it defines the data
types every other layer depends on. The `ActionabilityView` / `ActionabilityNode` /
`ActionabilityEdge` types are the API contract. `survey_view_for_map` computes
topological rank over the dep overlay — the server-side computation the frontend
consumes without reimplementing. `DataStores` is a prerequisite struct for the
state revision in PHASE-02. All pure, testable without a server.

**PHASE-02** (server endpoint + state wiring) depends on PHASE-01's types and
surface functions. It revises `AppState` to use the single-`RwLock` `DataStores`
pattern (D9/D10), adds the `GET /api/survey` endpoint, and atomically refreshes
all three stores. This is the backend substrate PHASE-04's renderer calls.

**PHASE-03** (frontend infrastructure) is file-disjoint from PHASE-01/02 — it
touches only `web/map/`. It can execute in parallel with backend phases if
desired. It vendors D3 + d3-dag, splits the kind checkboxes, adds the view
toggle control, and extends `model.js` + `api.js` with the new state and
fetch path. No rendering logic yet.

**PHASE-04** (actionability renderer + toggle) depends on PHASE-02 (API shape
available) and PHASE-03 (vendor scripts loaded, model/api wired). It implements
`priority.js` — the D3 sugiyama renderer — and wires the view toggle in `app.js`
to dispatch between semantic (DOT/Graphviz) and actionability (D3) rendering.
Updates `render.js` relationship table columns and `search.js` edge legend.

**PHASE-05** (styles + integration) is the final polish pass. CSS variables for
colours (OQ-3), toggle button styles, dark theme verification, and kind filter
contract enforcement (checkboxes affect entity list only, not the actionability
graph). Runs the full gate for commit-readiness.

## Dependencies

```
PHASE-01 ──→ PHASE-02 ──→ PHASE-04 ──→ PHASE-05
                │            │
PHASE-03 ───────┘            │
      (file-disjoint,         │
       parallel candidate)    │
                              │
                   PHASE-03 ──┘
```

PHASE-03 can run in parallel with PHASE-01/02 because they touch different
directory trees (`src/` vs `web/map/`). The contract PHASE-03 depends on is
the API shape documented in `design.md` — it doesn't need a running server
to add model state, vendor files, or HTML controls.

## Key invariants to preserve

- Priority engine owns truth (D4, D5): `survey_view_for_map` computes rank
  server-side — frontend `priority.js` only calls d3-dag layout + SVG rendering
- `DataStores` under single `RwLock` (D9/D10): atomic refresh, no torn-read window
- `survey_for_map` extraction: zero behavioural divergence from existing `survey()`
- Kind filters affect entity list only — actionability graph always shows all
  work entities regardless of checkbox state
- CSS variables for colours (OQ-3): `--priority-actionable-bg`, etc., no hardcoded
  values in JS

## Notes

- The design has 12 locked decisions (D1–D12), zero unresolved open questions.
- ADR-001 governs the layering: web frontend is leaf tier, map server + priority
  subsystem are engine tier.
- IMP-047 (trinary actionability) is a dependency only in the sense that its
  data model is what we render; this slice does not implement IMP-047.
- `d3-dag` UMD bundle extends the `d3` global — `d3.sugiyama()`, `d3.graphStratify()`.
  Load order: `d3.v7.min.js` before `d3-dag.min.js`.
