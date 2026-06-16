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
then markdown (the content layer), then the interactive UI surfaces (kind filter,
search, relationship table, depth selector), and finally integration/polish
(edge detail, refresh, --path flag, acceptance). The Rust `--path` flag change
(IMP-079) and JS unit tests are threaded through the appropriate phases rather
than siloed separately.

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

### PHASE-05: Interactive UI

Depends on PHASE-04 for a fully functional base (shell + model + SVG + markdown).
Wires the four main interactive surfaces: kind filter, search, relationship
table, and depth selector. These are individually small but collectively produce
the core explorer experience. They're grouped into one phase because they share
no internal dependencies — each is a consumer of PHASE-02/03/04 APIs wired to a
specific DOM region.

The kind filter's "List/table filter" label and semantics (sidebar + relationship
table only, SVG unchanged) are verified against the Hard Contracts. The search
Enter → findFocus null-on-miss behaviour is verified against the Hard Contracts
search navigation requirement.

### PHASE-06: Integration + acceptance

The final integration phase. Edge detail page (reachable from relationship
table), refresh wiring (clear caches, increment graphRenderSeq, re-fetch,
preserve focus), --path CLI flag (IMP-079), SVG sanitization edge cases, and
the full 21-item acceptance checklist.

This phase is deliberately last — it exercises the full stack and validates
that all the individually-built components compose correctly. The edge detail
page is placed here rather than PHASE-05 because it depends on relationship
table edge ID click wiring and URL-safe edge IDs, both delivered in PHASE-05.
The --path flag is a small, independent Rust diff that can be implemented at
any point; placing it in PHASE-06 keeps it from blocking frontend work.

## Notes

- The Rust `--path` flag change (IMP-079) is a small, independent diff delivered
  in PHASE-06 to avoid blocking frontend work.
- JS unit tests in `web/map/test.html` are additive — PHASE-02 writes the
  initial tests (model, router), PHASE-03 adds dotQuote tests, PHASE-05 adds
  searchFilter + kinds tests.
- Edge ID encoding (`encodePart`: non-`[A-Za-z0-9-]` → `_HH` hex) is
  implemented in PHASE-02 alongside `normalizeGraph` and verified in PHASE-06
  against URL-safe round-tripping.
- The `--open` flag already exists in SL-072's CLI. The new `--path` flag is
  additive and does not change existing behaviour.
- No RustEmbed changes needed — `web/map/` is already embedded by SL-072's
  `src/map_server/assets.rs`.
