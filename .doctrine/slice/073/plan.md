# Implementation Plan SL-073: Doctrine Map Frontend: interactive browser explorer

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five sequential phases builds the static frontend from bare shell through to
full acceptance. The plan is strictly bottom-up: each phase delivers a complete,
testable layer that the next phase consumes. No phase depends on a later phase's
output.

The phases follow the design's module contract boundaries — `api` + `model` +
`router` first (the data layer), then `dot` + SVG (the visualisation layer),
then markdown (the content layer), and finally the integration/polish work
(kind filter, search, relationship table, edge detail, refresh wiring). The
Rust `--path` flag change (IMP-079) and JS unit tests are threaded through the
appropriate phases rather than siloed into a separate phase.

## Sequencing & Rationale

### PHASE-01: Static shell

Must come first. Every subsequent phase injects content into DOM regions
defined by this shell. The semantic CSS Grid layout, light/dark theme custom
properties, and kind colour palette CSS variables are the foundation that all
rendering phases target. Without this, the app has no visual structure.

No JS logic yet — placeholder content confirms the layout works before any
data dependency exists. This keeps the feedback loop tight: verify the shell
renders correctly, then move on.

### PHASE-02: Model + routing

The data layer. Normalization, focus resolution, BFS neighbourhood, and hash
routing are pure functions testable without a browser. This phase delivers the
full mental model of the application — every rendering phase is a consumer of
these APIs.

The `graphRenderSeq` field is added to state here (even though its first
consumer is PHASE-03) to keep state definition in one place.

`findFocus` is implemented alongside `resolveFocus` per the Hard Contracts
search-miss requirement. The separation is verified by unit tests before any
rendering code depends on it.

JS unit tests (`web/map/test.html`) are authored during this phase. The pure
functions this phase delivers are the highest-risk logic in the SPA — BFS,
focus resolution, and edge ID generation have the most edge cases. Testing
them before they're embedded in DOM code keeps the feedback loop fast.

### PHASE-03: DOT/SVG rendering

Depends on PHASE-02 for `model.neighbourhood` (BFS) and `state.graph`. The
DOT generation is pure (graphToDot, dotQuote, nodeAttrs, edgeAttrs) but the
SVG pipeline is impure — fetch, sanitize, inject, wire handlers. The
stale-render guard (graphRenderSeq token) is critical here because DOT
rendering is async and the user can change focus during the render.

Node identity extraction from `<g class="node"><title>` is verified against
the Hard Contracts requirement: DOT node key is the canonical ref. The hover
detail pane is delivered here because it's the natural companion to SVG
node hover.

### PHASE-04: Markdown rendering

Depends on PHASE-03 only in that the app shell is fully functional by this
point. Markdown is the last content panel — it renders below the graph and
relationship table. The stale-request guard uses `state.focusId` comparison
(not graphRenderSeq) because markdown fetches are per-entity, not per-render.

Link policy post-processing (external → new tab + noopener, relative → strip,
anchor → preserve) is implemented here because it's a markdown-specific concern.
Error states (404, 501, 500) are handled with distinct UI per the design's
state table.

### PHASE-05: Polish + acceptance

The integration phase. All remaining UI surfaces (kind filter, search,
relationship table, edge detail page, depth selector, refresh) are wired here.
These are individually small but collectively produce the full interactive
experience. They're grouped into one phase because they share no internal
dependencies — each is a consumer of PHASE-02/03/04 APIs wired to a specific
DOM region.

The kind filter's "List/table filter" label and semantics (sidebar + relationship
table only, SVG unchanged) are verified against the Hard Contracts. The search
Enter → findFocus null-on-miss behaviour is verified against the Hard Contracts
search navigation requirement.

The Rust `--path` flag test is delivered here since it's a small CLI change
with no frontend dependency. Full 19-item acceptance checklist pass gates
phase exit.

## Notes

- The Rust `--path` flag change (IMP-079) is a small, independent diff. It can
  be implemented during any phase but is tracked in PHASE-05 to keep it from
  blocking frontend work.
- JS unit tests in `web/map/test.html` are additive — PHASE-02 writes the
  initial tests, PHASE-03 adds dotQuote tests, PHASE-05 adds searchFilter +
  kinds tests.
- The `--open` flag already exists in SL-072's CLI. The new `--path` flag is
  additive and does not change existing behaviour.
- No RustEmbed changes needed — `web/map/` is already embedded by SL-072's
  `src/map_server/assets.rs`.
