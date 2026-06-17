# SL-083 Implementation Plan: Rationale & Sequencing

## Why these phases, in this order

The six phases follow a strict dependency order — each phase only touches
concerns whose dependencies are already stable. This minimizes the blast radius
of any single extraction and ensures `test.html` can gate every phase
incrementally.

### PHASE-01: Pre-extraction cleanup (dot.js, model.js, api.js)

These three cleanup items (F-6, F-8, F-15) are in already-extracted modules
with no dependencies on the new module boundaries. They are small, isolated,
and independently testable. Completing them first removes noise from the
extraction phases — the remaining app.js code is the target, not the side-show.

F-6 (dot.js NODE_STYLES) and F-8 (model.js bfsCore) carry the highest
correctness risk among the cleanups because they touch logic with test
coverage. PHASE-01 catches any regression before a single module boundary is
drawn.

### PHASE-02: Extract svg.js

`svg.js` is the leaf module — it depends on nothing except the DOM. It has no
dependency on `render.js`, `search.js`, or `concept-map.js`, so extracting it
first carries zero risk of forward-reference errors.

F-14 (handler factory consolidation) lands here because `svg.wireHandlers` is
the natural home. The `extractId` callback parameter (design review fix) is
introduced here and flows through to the `applyFocusHighlight` signature in
the same phase.

### PHASE-03: Extract render.js

`render.js` depends on `model.js`, `dot.js`, `api.js` (all stable since
PHASE-01) and `svg.js` (stable since PHASE-02). It is the largest extraction
(~350 lines) and carries the most DOM-construction surface.

Three cleanup items land here because they are render-concerned: F-5
(encodeAttr removal + escapeHtml/escapeAttr namespacing), F-7 (data-kind
attributes), and F-9 (DOM element cache). Bundling them avoids touching the
same call sites twice.

The `setViewMode` CM-container clearing (design review fix) lands here because
`render.setViewMode` is the single central visibility gate.

### PHASE-04: Extract search.js

`search.js` depends on `model.js` (stable) and `render.js` (stable since
PHASE-03). Its `renderFilteredEntities` function composes `render.entityList`,
so the render module must exist first.

Keyboard nav state (`listNavIndex`) moves to closure-local — this is the only
state-ownership question in search.js, and it has no cross-module implications.

### PHASE-05: Extract concept-map.js

`concept-map.js` depends on `model.js`, `dot.js`, `api.js` (all stable since
PHASE-01) and `svg.js` (stable since PHASE-02). It does NOT depend on
`render.js` or `search.js`, so it could theoretically precede PHASE-04.
However, placing it after `search.js` means the orchestrator (`app.js`) has
already shrunk by ~500 lines before tackling the second-largest extraction
(~350 lines), reducing cognitive load.

D2 (pure renderer) is enforced here: mutation handlers stay in `app.js`.
Node-click callbacks fire unconditionally; editing-state branching is the
orchestrator's concern. The `renderCmDiagnostics` test in `test.html` is
updated to call `cm.renderDiagnostics` — the only test that directly exercises
a CM render function.

### PHASE-06: Shrink app.js and wire together

All modules exist. `app.js` is now pure orchestration: bootstrap, render
dispatch, CM mutation pipeline, error display, markdown rendering. F-16
(safeStorage) lands here because it lives in `app.js` and is called during
bootstrap.

The `index.html` script load order is finalized in this phase — prior phases
add their `<script>` tags sequentially, but the final order must match the
design exactly.

The full manual checklist is run and results recorded in slice notes. This is
an explicit acceptance gate (design review fix).

## Verification strategy summary

| Gate | Scope | When |
|------|-------|------|
| `test.html` | Full suite | After every phase |
| Manual checklist 1-7 | Entity-graph UI | PHASE-03 |
| Manual checklist 3-7 | Search + depth + refresh | PHASE-04 |
| Manual checklist 8-11, 16 | CM UI + stale-panel guard | PHASE-05 |
| Full manual checklist 1-16 | End-to-end | PHASE-06 |

`test.html` regression is the continuous gate. Manual checklist items are phase-gated
to catch DOM regressions before they compound.

## Risk surface

- **Highest risk**: PHASE-03 (render.js). Largest extraction, most call sites,
  most DOM surface. Mitigated by the manual checklist run immediately after.
- **Medium risk**: PHASE-05 (concept-map.js). CM editing state machine is
  complex (view/edit toggle, rename inline input, edge add/remove). Mitigated
  by D2 keeping mutation handlers in `app.js` — only DOM construction moves.
- **Low risk**: PHASE-01 (cleanup), PHASE-02 (svg.js), PHASE-04 (search.js),
  PHASE-06 (app.js). Well-bounded, small surface, good test coverage or
  trivial changes.

## Phase dependency graph

```
PHASE-01 (cleanup: dot, model, api)
    ↓
PHASE-02 (svg.js)
    ↓
PHASE-03 (render.js) ← needs svg.js
    ↓
PHASE-04 (search.js) ← needs render.js
    ↓
PHASE-05 (concept-map.js) ← needs svg.js, model, dot, api
    ↓
PHASE-06 (app.js + integration) ← needs everything
```

All phases are serial. No phase is file-disjoint with any other (they all
touch `app.js`), so parallel execution is not possible.
