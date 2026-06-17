# SL-083 Design: Decompose `web/map/app.js` into modular frontend components

## Hard Contracts

These are binding constraints inherited from SL-073 (map frontend) and the SL-083
scope document. Every module and phase must satisfy them; acceptance is gated.

- **Pure refactor — no behaviour change.** Same user-visible DOM structure and
  behaviour, same event handling, same hash routes, same visual output in light
  and dark modes. Additive non-behavioural DOM attributes allowed only where
  explicitly listed in this contract: `data-kind` on kind-pill elements (F-7).
  The existing `test.html` suite must pass unchanged before and after every commit.
- **`web/map/` file set remains globals-namespace script loading.** No ES modules,
  no bundler, no build step (SL-073 design §1 Hard Contracts: "No ES module
  loader, no build step").
- **Data-in, data-out DI pattern.** Modules receive state as function parameters;
  only `app.js` reads/writes `state.*` directly. No module reaches into global
  `state` except `model.js` (the state owner) and `app.js` (the orchestrator).
- **SL-073 CSS custom-property palette is untouched.** Kind colours, theme vars,
  and dark-mode media query remain as-is. F-7 (data-kind selectors) only changes
  the CSS selector mechanism, never the visual output.
- **Project-authored doctrine content is untrusted display input** — the same
  XSS pipeline (DOT quoting → server render → DOMPurify SVG profile → inline DOM)
  is preserved unchanged.
- **Concept-map mutation pipeline flow is preserved.** The `api.mutateConceptMap`
  → `updateCache` → `render()` cycle, stale-write recovery (409 → auto-refetch),
  and error formatting are unchanged — they just move from inline in `app.js` to
  named functions in the same file.
- **`test.html` must remain a single self-contained file** with no new external
  dependencies. Tests may update `/* global */` comments to reflect the new module
  namespace objects but must not change assertion logic.

## 1. Module boundaries & load order

### File layout (target)

```
web/map/
  index.html          → load order updated (add svg.js, search.js, concept-map.js)
  style.css           → F-7: [style*="--kind-X"] → [data-kind="X"]
  api.js              → F-15: declarative body construction
  model.js            → F-8: shared BFS core extracted
  router.js           → unchanged
  dot.js              → F-6: switch → NODE_STYLES lookup table
  svg.js              → NEW: injectHitRects, wireHandlers, applyFocusHighlight, dimLegend
  render.js           → EXTRACTED: all entity-graph DOM construction
  search.js           → EXTRACTED: search, keyboard nav, filters, depth buttons
  concept-map.js      → EXTRACTED: CM diagram, edge table, diagnostics, add-edge form, edit toggle
  app.js              → SHRUNK: bootstrap, render dispatch, CM mutation pipeline, error display
  test.html           → updated /* global */ comments
  vendor/             → unchanged
```

### Load order in `index.html`

```
api.js → model.js → router.js → dot.js → svg.js → render.js → search.js → concept-map.js → app.js
```

Dependencies: `svg.js` depends on nothing (pure DOM). `render.js` depends on `model`, `dot`, `api` (markdown fetch). `search.js` depends on `model`, `render` (entity list rendering). `concept-map.js` depends on `model`, `dot`, `api`, `svg`. `app.js` depends on everything.

### Responsibility matrix

| Module | Lines | Owns | Receives from app.js | Exposes on `window` |
|--------|-------|------|---------------------|---------------------|
| `svg.js` | ~60 | SVG DOM manipulation | `svgEl`, callbacks, `extractId` | `svg.injectHitRects`, `svg.wireHandlers`, `svg.applyFocusHighlight`, `svg.dimLegend` |
| `render.js` | ~350 | All entity-graph DOM | `graph`, `focusId`, `depth`, `kindFilter`, callbacks | `render.elements`, `render.cacheElements`, `render.entityList`, `render.focusHeader`, `render.graphPane`, `render.hoverPane`, `render.markdownPane`, `render.relationshipTable`, `render.edgeDetail`, `render.setViewMode`, `render.escapeHtml`, `render.escapeAttr` |
| `search.js` | ~150 | Search/filter/depth UI | container refs, `graph`, DOM callbacks | `search.wireSearch`, `search.wireFilters`, `search.wireDepthButtons`, `search.wireRefresh`, `search.collectKindFilter`, `search.renderFilteredEntities` |
| `concept-map.js` | ~400 | CM DOM (view + edit) | `cm` data, editing state, callbacks | `cm.renderDiagram`, `cm.renderEdgeTable`, `cm.renderDiagnostics`, `cm.renderAddEdgeForm`, `cm.renderEditToggle` |
| `app.js` | ~100 | State, bootstrap, orchestration | (root — owns all state) | bootstrap + render (IIFE-internal) |

## 2. Function signatures

### `svg.js`

```js
var svg = {};

// Inject transparent hit-rect as first child of every <g class="node">.
// Idempotent — skips nodes that already have a hit-rect child.
svg.injectHitRects = function(svgEl) { ... }

// Wire click + mouseenter/mouseleave on every <g class="node">.
// extractId: function(g) → string — reads <title> (CM) or <text> (entity graph).
// handlers: { onClick(id), onHoverEnter(id), onHoverLeave() }
svg.wireHandlers = function(svgEl, extractId, handlers) { ... }

// Apply/remove .doctrine-node--focus on the SVG <g> where extractId(g) === newId.
// extractId: function(g) → string — same contract as wireHandlers.
// oldId: previous focus. newId: current focus.
svg.applyFocusHighlight = function(svgEl, newId, oldId, extractId) { ... }

// Dim legend items whose edge labels are absent from the given neighbourhood.
svg.dimLegend = function(neighbourhood) { ... }
```

### `render.js`

```js
var render = {};

// DOM references populated once at bootstrap
render.elements = {};

// Capture all repeated querySelector targets
render.cacheElements = function(root) { ... }
// Populates: .entityList, .focusHeader, .graphArea, .hoverDetail,
//            .relationshipTable, .markdownPane, .tableToggle, .depthSelector

// Options-object convention for functions with >4 params.

render.entityList = function({ container, nodes, focusId, onFocus }) { ... }

render.focusHeader = function({ container, focusId, graph }) { ... }

// Returns a Promise. seq and getCurrentSeq guard stale renders.
render.graphPane = function({ container, graph, focusId, depth, dotAvailable, seq, getCurrentSeq, onNodeClick, onNodeHoverEnter, onNodeHoverLeave }) { ... }

render.hoverPane = function({ container, node }) { ... }

// currentFocusId for stale-request guard. cache is the markdownCache Map.
render.markdownPane = function({ container, id, cache, currentFocusId }) { ... }

render.relationshipTable = function({ container, edges, graph, focusId, depth }) { ... }

// Replaces graph-area content with edge metadata table.
render.edgeDetail = function({ container, edge, graph, depth, focusId }) { ... }

// Toggle entity-graph vs concept-map UI visibility.
// mode: 'entity-graph' | 'concept-map' | 'edge'
render.setViewMode = function(mode) { ... }

// HTML escaping (moved from app.js; F-5: encodeAttr removed)
render.escapeHtml = function(str) { ... };
render.escapeAttr = function(str) { ... };
```

### `search.js`

```js
var search = {};

// onFocus(id): callback when user selects an entity (click or Enter).
// Keyboard nav state (listNavIndex) is closure-local, not on global state.
search.wireSearch = function({ input, list, graph, onFocus }) { ... }

// onChange(filterSet): callback with Set<string> | null.
search.wireFilters = function({ container, onChange }) { ... }

search.wireDepthButtons = function({ container, onDepthChange }) { ... }

search.wireRefresh = function({ button, onRefresh }) { ... }

search.collectKindFilter = function(container) { ... }

// Composition: model.searchFilter → render.entityList
search.renderFilteredEntities = function({ list, graph, query, kindFilter, focusId, onFocus }) { ... }
```

### `concept-map.js`

```js
var cm = {};

// cm: normalized concept map data. focusKey: string | null.
// seq and getCurrentSeq guard stale renders.
cm.renderDiagram = function({ container, cm, focusKey, depth, dotAvailable, seq, getCurrentSeq, onClick, onHoverEnter, onHoverLeave }) { ... }

// editingNode: { key, label } | null.
cm.renderEdgeTable = function({ container, cm, focusKey, depth, editing, editingNode, onRemoveEdge, onRenameNode, onSubmitRename }) { ... }

cm.renderDiagnostics = function({ container, diagnostics }) { ... }

// onSubmit(source, rel, target): called when user submits the form.
cm.renderAddEdgeForm = function({ container, cm, editing, onSubmit }) { ... }

cm.renderEditToggle = function({ header, editing, onToggle }) { ... }
```

### `app.js` (shrunk)

```js
(function () {
  'use strict';

  var md = null;  // lazy markdown-it instance

  // --- Bootstrap ---
  function bootstrap() { ... }

  // --- Main render dispatch ---
  function render() { ... }

  // --- Concept-map mutation pipeline ---
  function handleAddEdge(source, rel, target) { ... }
  function handleRemoveEdge(source, rel, target) { ... }
  function handleRenameNode(newLabel) { ... }
  function handleStaleWrite() { ... }
  function handleMutationError(err) { ... }
  function showCmFormError(msg) { ... }

  // --- Markdown rendering ---
  function renderMarkdown(text) { ... }

  // --- Error display ---
  function showError(container, msg) { ... }

  // --- safeStorage helper (F-16) ---
  var safeStorage = {
    get: function(key, fallback) { ... },
    set: function(key, value) { ... }
  };

  // Kick off
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', bootstrap);
  } else {
    bootstrap();
  }
})();
```

## 3. Data flow

### Bootstrap sequence

```
bootstrap()
  ├─ render.cacheElements(document)           // one-time DOM ref capture (F-9)
  ├─ search.wireSearch(...)                   // persistent event listeners
  ├─ search.wireFilters(...)
  ├─ search.wireDepthButtons(...)
  ├─ search.wireRefresh(...)
  ├─ wireTableToggle()                        // localStorage-backed (app.js, uses safeStorage)
  ├─ window.addEventListener('hashchange', render)
  ├─ Promise.all([api.fetchHealth(), api.fetchGraph()])
  │     ├─ health → state.dotAvailable
  │     └─ raw → model.normalizeGraph(raw)
  └─ resolve initial focus → router.setFocus → hashchange → render()
```

### Render dispatch

```
hashchange → parseHash() → render()
  │
  ├─ route.view === 'edge'
  │     ├─ state.focusId unchanged (resolved if null)
  │     ├─ render.edgeDetail(...)
  │     ├─ render.entityList(...)
  │     ├─ render.focusHeader(...)
  │     └─ return (skip graph + markdown)
  │
  └─ route.view === 'focus'
        ├─ state.focusId = route.id
        ├─ state.depth = clamp(route.depth)
        │
        ├─ [SYNC] search.renderFilteredEntities(...)
        ├─ [SYNC] render.focusHeader(...)
        ├─ [SYNC] render.relationshipTable(...)
        ├─ [SYNC] render.hoverPane({ container, node: null })
        ├─ [SYNC] sync depth button active states
        │
        ├─ [focus or depth changed?]
        │     ├─ instant highlight: svg.applyFocusHighlight(svgEl, newId, oldId, extractId)
        │     └─ isConceptMap(focusId)?
        │           ├─ YES → cm.renderDiagram(...)
        │           └─ NO  → render.graphPane(...)
        │
        ├─ cm.renderEditToggle(...)
        ├─ cm.renderEdgeTable(...)             // CM only
        ├─ cm.renderAddEdgeForm(...)           // CM + editing
        ├─ cm.renderDiagnostics(...)           // CM + not editing
        ├─ render.setViewMode(isCm ? 'concept-map' : 'entity-graph')
        │
        └─ render.markdownPane(...)
```

### SVG handler data flow

```
render.graphPane()                         cm.renderDiagram()
  │ API returns SVG text                     │
  ├─ DOMPurify sanitize (SVG profile)        ├─ same pipeline
  ├─ inject DOM                              ├─ svg.injectHitRects(svgEl)
  ├─ svg.injectHitRects(svgEl)               └─ svg.wireHandlers(svgEl,
  ├─ svg.wireHandlers(svgEl,                 │     extractId = g → g.querySelector('title').textContent,
  │     extractId = g → g.querySelector('text').textContent,  │     {
  │     {                                    │       onClick: (key) → cmFocusNode toggle → render(),
  │       onClick: (id) → router.setFocus,   │       onHoverEnter: (key) → render.hoverPane(cm node),
  │       onHoverEnter: (id) → state.hoveredId = id → render.hoverPane,  │       onHoverLeave: () → render.hoverPane(null)
  │       onHoverLeave: () → state.hoveredId = null → render.hoverPane  │     })
  │     })                                   │
  └─ svg.dimLegend(neighbourhood)            │
```

### Concept-map mutation flow

> **Callback branching**: `onClick(key)` is called for *all* CM node clicks. It is
> `app.js`'s responsibility to check `state.editingConceptMap` and either start a
> rename (edit mode) or toggle `cmFocusNode` (view mode). `cm.js` fires the callback
> unconditionally — it does not branch on editing state.

```
User clicks "Add edge"
  → cm.js fires onSubmit(source, rel, target)
  → app.js handleAddEdge(source, rel, target)
      ├─ client-side validation (trim, non-empty)
      ├─ api.mutateConceptMap(id, 'add_edge', params, baseHash)
      │     ├─ 200 → updateCache(data) → render()
      │     ├─ 409 stale → handleStaleWrite() → auto-refetch → render()
      │     ├─ 409 duplicate → showCmFormError("line N")
      │     ├─ 400 empty_field → showCmFormError(message)
      │     ├─ 404 edge_not_found → showCmFormError(...)
      │     └─ other → handleMutationError(err)
```

### State access matrix

| Field | Owner | Readers (via params) |
|-------|-------|---------------------|
| `graph.*` | `model.js` (normalizeGraph) | `render.js`, `search.js` |
| `focusId` | `app.js` (render sets it) | `render.js`, `search.js`, `cm.js` |
| `depth` | `app.js` (render sets it) | `render.js`, `cm.js` |
| `markdownCache` | `app.js` (fetch + clear) | `render.markdownPane` |
| `conceptMapCache` | `app.js` (fetch + mutate) | `cm.js` |
| `editingConceptMap` | `app.js` (toggle) | `cm.js` |
| `editingNode` | `app.js` (rename) | `cm.js` |
| `cmFocusNode` | `app.js` (click toggle) | `cm.js`, `router.buildHash` |
| `dotAvailable` | `app.js` (health check) | `render.graphPane` |
| `hoveredId` | `app.js` (mouseenter/leave) | (internal coordination) |
| `kindFilter` | `app.js` (filter callback) | `search.js`, `render.js` |
| `graphRenderSeq` | `app.js` (increment per render) | `render.graphPane`, `cm.renderDiagram` |

## 4. Cleanup items

### F-5: Dead `encodeAttr` removed

`escapeHtml` and `escapeAttr` move from `app.js` to `render.js` (the sole consumer).
`encodeAttr` is deleted — zero callers. `app.js` declares `/* global render */` to
access the escape functions.

### F-6: `dot.nodeAttrs` switch → `NODE_STYLES` lookup

```js
dot.NODE_STYLES = {
  SL:  { fill: '#4A90D9', font: '#ffffff', shape: 'box,rounded' },
  ADR: { fill: '#7B4FBF', font: '#ffffff', shape: 'box' },
  POL: { fill: '#7B4FBF', font: '#ffffff', shape: 'box' },
  STD: { fill: '#9B59B6', font: '#ffffff', shape: 'box' },
  PRD: { fill: '#E67E22', font: '#222222', shape: 'box,rounded' },
  SPEC:{ fill: '#E67E22', font: '#222222', shape: 'box,rounded' },
  REQ: { fill: '#F39C12', font: '#222222', shape: 'box' },
  ISS: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  IMP: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  CHR: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  RSK: { fill: '#C0392B', font: '#ffffff', shape: 'box' },
  IDE: { fill: '#27AE60', font: '#222222', shape: 'box' },
  RV:  { fill: '#1ABC9C', font: '#222222', shape: 'box' },
  REC: { fill: '#95A5A6', font: '#222222', shape: 'box' },
  ASM: { fill: '#3498DB', font: '#ffffff', shape: 'box' },
  DEC: { fill: '#3498DB', font: '#ffffff', shape: 'box' },
  QUE: { fill: '#8E44AD', font: '#ffffff', shape: 'box' },
  CON: { fill: '#8E44AD', font: '#ffffff', shape: 'box' },
  REV: { fill: '#A04000', font: '#ffffff', shape: 'box' },
  CM:  { fill: '#16A085', font: '#ffffff', shape: 'box' }
};
dot.DEFAULT_NODE_STYLE = { fill: '#95A5A6', font: '#222222', shape: 'box' };

dot.nodeAttrs = function(node, focusId) {
  var s = dot.NODE_STYLES[node.kindPrefix] || dot.DEFAULT_NODE_STYLE;
  return {
    label: node.id,
    fillcolor: s.fill,
    fontcolor: s.font,
    shape: s.shape,
    penwidth: (node.id === focusId) ? 3.0 : 1.0,
    tooltip: node.id + ': ' + node.title + ' \u00b7 ' + (node.kindLabel || node.kindPrefix) + ' \u00b7 ' + node.status
  };
};
```

The `depth` parameter is dropped — it was accepted but never used.

### F-7: CSS `[style*="--kind-PRD"]` → `[data-kind="PRD"]`

> **Note**: The `.cm-diagnostics-panel` dark-mode overrides in `style.css` use
> raw color values inside `@media (prefers-color-scheme: dark)` rather than
> custom properties. This is a separate issue (tied to F-11 / IMP-087, the theme
> toggle) and is **out of scope** for SL-083.

Kind pill elements in `render.entityList` and `render.focusHeader` gain a `data-kind`
attribute. CSS text-color overrides switch from fragile inline-style substring
matching to stable data-attribute selectors:

```css
.kind-pill[data-kind="PRD"],
.kind-pill[data-kind="SPEC"],
.kind-pill[data-kind="REQ"],
.kind-pill[data-kind="IDE"],
.kind-pill[data-kind="RV"],
.kind-pill[data-kind="REC"] {
  color: #222;
}
```

The `background` still comes from inline `style="background: var(--kind-X)"` —
that reference is space-insensitive and the data-kind attribute is a stable
selector target.

### F-8: Shared BFS core

Internal function `bfsCore(startId, maxDepth, expandNeighbours, edgeKey)` in
`model.js`. `expandNeighbours` returns `[{ nodeId, edge }]`. `edgeKey` is an
optional function `(edge) → string` for dedup; defaults to `edge.id`.

Entity graph: edge objects carry `.id` — the default `edgeKey` works.
Concept map: edge objects carry `{ from_key, to_key, rel }` — caller passes
`edgeKey: function(e) { return e.from_key + '\x00' + e.rel + '\x00' + e.to_key; }`.

Used by both `model.neighbourhood` (entity graph, directed) and
`model.cmNeighbourhood` (concept map, undirected). Each public function builds
the appropriate expansion closure and delegates to `bfsCore`.

### F-14: Handler factory in `svg.wireHandlers`

The `svg.wireHandlers(svgEl, extractId, handlers)` function consolidates the
closure-over-id pattern that was duplicated in `wireSvgHandlers` and
`wireCmSvgHandlers`. Entity-list click handlers (on `<li>` elements, not SVG)
remain separate — different DOM, different semantics.

### F-15: Declarative `api.mutateConceptMap` body

```js
api.mutateConceptMap = function(id, action, params, baseHash) {
  var body = Object.assign({ action: action }, params);
  if (baseHash !== undefined) body.base_hash = baseHash;
  return fetch('/api/concept-map/' + encodeURIComponent(id), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  }).then(function(r) { ... });
};
```

Callers construct `params` with only the relevant keys. `base_hash` is separate
because it's a concurrency token, not a mutation parameter.

### F-16: `safeStorage` helper

```js
var safeStorage = {
  get: function(key, fallback) {
    try { var v = localStorage.getItem(key); return v !== null ? v : fallback; }
    catch (_) { return fallback; }
  },
  set: function(key, value) {
    try { localStorage.setItem(key, value); }
    catch (_) { /* silently degrade */ }
  }
};
```

Lives in `app.js` (8 lines — not worth its own file). `wireTableToggle` uses it;
future features (theme toggle in IMP-087, last-viewed entity) will reuse it.

## 5. Verification strategy

### Regression gate

`test.html` must pass identically before and after every commit. The test suite
exercises:

- `model.encodePart`, `model.normalizeGraph`, `model.neighbourhood`,
  `model.resolveFocus`, `model.findFocus`, `model.searchFilter`, `model.kinds`,
  `model.normalizeConceptMap`, `model.buildNodeLabelList`, `model.buildRelLabelList`,
  `model.cmNeighbourhood`
- `dot.dotQuote`, `dot.graphToDot`, `dot.cmGraphToDot`
- `router.parseHash`, `router.buildHash`, `router.setFocus`, `router.setEdge`
- `ApiError` construction
- `renderCmDiagnostics` (DOM-rendered output)
- CM neighbourhood edge cases (undirected, disconnected, null cm, focusKey not in nodes)

F-8 (shared BFS) is the only change that touches tested logic — both
`model.neighbourhood` and `model.cmNeighbourhood` must return identical results.

F-6 (`dot.nodeAttrs`) changes implementation but `dot.graphToDot` output must be
byte-identical.

### Manual verification checklist

**Acceptance note**: Manual checklist results must be recorded in the slice notes
(`.doctrine/slice/083/notes/`) before the slice can be accepted. Each item must
be explicitly confirmed pass/fail with any observations.

1. Load `doctrine map serve` — sidebar populates, first entity focused, graph renders.
2. Click a node — focus changes, hash updates, graph re-renders with highlight.
3. Search "SL" + Enter — navigates to first SL match.
4. Arrow-down keyboard nav — highlights items, Enter selects.
5. Toggle kind filter — entity list and relationship table update; graph unchanged.
6. Depth buttons — neighbourhood expands/contracts.
7. Refresh button — graph reloads, markdown cache clears.
8. Click a concept map entity — CM diagram renders, edge table shows, diagnostics visible.
9. Toggle CM edit mode — edge table shows remove buttons, add-edge form appears.
10. Add/remove CM edge — form submits, cache updates, view refreshes.
11. Rename CM node — inline input, Enter commits, Escape cancels.
12. Dark mode — toggle OS preference, verify both themes render correctly.
13. Fullscreen markdown — toggle works, content renders, link policy applied.
14. Edge detail view — clicking edge label in relationship table shows edge metadata.
15. Hide relations table — checkbox toggles, state persists across refresh (localStorage).
16. Navigate entity graph → concept map → entity graph — verify no stale CM panels remain visible.

## 6. Design decisions

### D1: DI pattern (options objects) over global state access

**Decision**: Modules receive state as function parameters via options objects.
Only `app.js` and `model.js` touch the global `state` object.

**Rationale**: The SL-073 design §3 specified parameterized render functions.
The current implementation ignores this. Following the design intent gives us
testable functions, clear ownership, and a documented contract at every call site.
The options-object convention keeps arg lists readable when parameter count
exceeds 4, avoiding the positional-arg fragility that the current inline-HTML
construction already suffers from.

### D2: `concept-map.js` as pure renderer, mutations stay in `app.js`

**Decision**: `cm.js` receives snapshot data and callbacks. Mutation handlers
(`handleAddEdge`, `handleRemoveEdge`, `handleRenameNode`) stay in `app.js`.

**Rationale**: Consistent with the DI decision for `render.js`. Keeps `app.js`
as the single owner of all state transitions and the mutation pipeline
(API call → cache update → re-render). Avoids splitting CM state between two
modules. If CM editing grows complex enough to warrant its own state module,
that's a future slice — SL-083's remit is extraction, not redesign.

### D3: `svg.js` as shared SVG DOM module

**Decision**: Hit-rect injection, click/hover handler wiring, focus highlighting,
and legend dimming live in a single `svg.js` module. Callers provide an
`extractId` function to read node identity from `<title>` (CM) or `<text>`
(entity graph).

**Rationale**: The handler-wiring pattern is literally identical between graph
types — the only difference is identity extraction. Consolidating eliminates the
duplication RV-049 F-14 flagged and gives SVG DOM manipulation a single home.
If future features add more SVG interactivity (zoom, pan, edge hover), they have
an obvious module to land in.

### D4: Options-object convention for functions with >4 parameters

**Decision**: Functions taking 5+ parameters use a single options object instead
of positional arguments.

**Rationale**: The current app.js functions have 6–8 positional parameters.
In the DI pattern, parameter count naturally grows (renderers receive everything
they need). An options object makes call sites self-documenting and eliminates
arg-order bugs. Functions with ≤4 parameters stay positional — the object
overhead isn't justified.

### D5: `render.setViewMode` element scope

**Decision**: `setViewMode(mode)` controls exactly three elements:
`.depth-selector`, `.relationship-table`, `.table-toggle`. When `mode` is not
`'concept-map'`, it also hides/clears all CM containers (`.cm-edge-table`,
`.cm-add-edge-form`, `.cm-diagnostics-panel`) to prevent stale DOM from
persisting across view-mode switches. CM render functions are not called in
non-CM modes.

**Rationale**: Depth and relationship-table are entity-graph concepts. CM
editing UI is a separate visibility axis (editing vs viewing) orthogonal to
graph-type switching. Clearing CM containers in `setViewMode` gives a single,
central crash-clearing gate that avoids race between render dispatch and
per-element visibility toggles.

### D6: `search.js` owns filter composition, delegates DOM to `render.js`

**Decision**: `search.renderFilteredEntities` composes `model.searchFilter` +
kind sorting + `render.entityList`. It does not duplicate DOM construction.

**Rationale**: Search owns the filtering UX pipeline (input → filter → sort →
display) but DOM construction for entity list items is `render.js`'s concern.
This keeps `render.entityList` as the single place that builds the pill + title
+ click handler for an entity item.

## 7. Open questions

None remaining. All design decisions are resolved.

## 8. Risks

- **Low**: Module extraction introduces a regression in edge-case behaviour
  (e.g. keyboard nav state lost across re-renders, stale-render guard broken by
  callback timing). Mitigation: `test.html` suite + manual verification checklist.
- **Low**: `svg.wireHandlers` `extractId` function must handle both `<title>`
  (CM, preserved by DOMPurify SVG profile) and `<text>` (entity graph, where
  `<title>` may be stripped). Mitigation: the current code handles both paths
  correctly; extraction just parameterizes the difference.
- **Negligible**: `dot.graphToDot` output changes due to lookup-table switch
  (F-6). Mitigation: byte-identical constraint in verification strategy.
- **Documented**: The entity-graph handler reads node identity from `<text>`
  content, not `<title>`. The SL-073 design Hard Contract says "`<title>` is the
  sole identity extraction point — never parse `<text>` content" — but DOMPurify
  may strip `<title>`, creating a contradiction the current code resolves in
  favour of reliability. SL-083 inherits this pre-existing drift; resolving the
  contract contradiction is out of scope.
- **Documented**: `model.js` must remain a non-IIFE global scope (not wrapped in
  `(function(){...})()`) because `test.html` calls the bare `padId` helper
  directly. If `model.js` is later modularized, `padId` must be exposed on the
  `model` namespace object or the test updated.
