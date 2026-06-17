# Decompose web/map/app.js God Object into modular frontend components

## Context

SL-073 (Doctrine Map Frontend) shipped the interactive browser explorer. Its
design (§1 Architecture & Module Layout) specified a `render.js` module alongside
`api.js`, `model.js`, `dot.js`, and `router.js`. That module was never extracted —
every rendering concern, interaction handler, concept-map editing surface, error
display, and bootstrap orchestration collapsed into `app.js` (1,406 lines, single
IIFE). The design also specified `renderShell` as a DOM-construction function; the
implementation instead builds HTML via string concatenation inline.

RV-049 (code-review of IMP-085) surfaced 16 findings. The four major ones (F-1
through F-4) are the decomposition itself: monolithic app.js, global mutable
state, inline HTML concatenation, and no module system. The 12 minor/nit findings
are cleanup items within the same decomposition.

SL-081 (memory-in-catalog) is currently `ready` and targets the same frontend
surface. This slice should land before SL-081 enters execution so the memory
surfacing work builds against a modular app shell, not the God Object.

## Scope & Objectives

Decompose `web/map/app.js` into the module structure the SL-073 design intended,
plus the concept-map module that SL-076 added:

| Module | Concern | ~lines |
|--------|---------|--------|
| `app.js` | Bootstrap, render loop, orchestration | ~100 |
| `render.js` | DOM construction (`el()`), entity list, focus header, relationship table, markdown pane, hover detail, graph pane | ~350 |
| `search.js` | Search input wiring, keyboard nav, filter checkboxes, depth buttons | ~150 |
| `concept-map.js` | CM rendering (diagram, edge table, diagnostics, add-edge form, edit toggle) — pure renderer; mutation handlers stay in `app.js` | ~350 |
| `svg.js` | SVG post-processing: hit-area injection, click/hover handlers, focus highlight, legend dimming (shared by entity graph and CM) | ~100 |

Remaining lines in `app.js` become pure orchestration: bootstrap, `render()`
dispatch, refresh coordination. No module exceeds ~400 lines.

Each module exposes its API on a namespace object (e.g. `window.render`,
`window.search`, `window.cm`, `window.svg`) — same global-namespace pattern, no
build step, consistent with the SL-073 design decision against ES modules. The
`/* global */` comments update accordingly.

### Cleanup items from RV-049 (included in scope)

- **F-5**: Remove dead `encodeAttr` function; keep `escapeHtml` + `escapeAttr`.
- **F-6**: Replace `dot.nodeAttrs` switch with a data-driven lookup table.
- **F-7**: Replace CSS `[style*="--kind-PRD"]` selectors with `data-kind`
  attributes on kind pills. Add `data-kind` to `buildEntityItem` and focus header.
- **F-8**: Extract a shared BFS traversal used by both `model.neighbourhood` and
  `model.cmNeighbourhood`.
- **F-9**: Cache DOM element references in module-level variables (e.g.
  `render.elements.graphArea`, `render.elements.focusHeader`) populated once at
  bootstrap.
- **F-14**: Extract `bindNodeHandler(g, event, handlerFactory)` shared between
  `wireSvgHandlers` and `wireCmSvgHandlers`.
- **F-15**: Rewrite `api.mutateConceptMap` body construction declaratively.
- **F-16**: Extract `safeStorage.get(key)` / `safeStorage.set(key, value)` helper.

### Not in scope

- ES module migration (import/export, bundler). The SL-073 design explicitly chose
  global-namespace script loading; reversing that choice is an ADR-level decision.
- New test infrastructure (framework, runner, CI). Tests stay in `test.html` with
  hand-rolled assert. Test coverage for rendering paths is out of scope.
- Dark mode toggle (F-11). Feature addition, not decomposition.
- Vendor file versioning/hashing (F-10). Supply-chain hygiene is a separate work
  item.
- Semantic HTML improvements (F-13). Can be tackled opportunistically if adjacent
  to a module boundary change, but not a goal.
- Any behaviour change. This is a pure refactor: same DOM, same events, same hash
  routes, same visual output. The test suite must pass unchanged.

## Summary

A pure structural refactor of `web/map/app.js` into five modules matching the
SL-073 design's intended module layout, plus cleanup of the 8 most contained
RV-049 findings. No behaviour change. No new dependencies. No build step. The
existing `test.html` suite remains the regression gate.

## Follow-Ups

- **IMP-086** (RV-049 F-10): Pin vendor dependencies with version metadata + SRI hashes.
- **IMP-087** (RV-049 F-11): Manual theme toggle (light/dark/auto).
- **IMP-088** (RV-049 F-12): Test framework adoption with DOM rendering coverage.
- **IMP-089** (RV-049 F-13): Semantic HTML landmarks, sections, ARIA.
