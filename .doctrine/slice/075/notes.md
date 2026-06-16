# Notes SL-075: Map Explorer UX overhaul

## 2026-06-16 — PHASE-02 complete (f772ea4)

JS logic improvements. Gate green. All 5 design decisions applied:

- **D9 (depth bugfix):** removed premature `state.depth = d` from click handler.
  `render()` now detects `depthChanged` and re-renders graph.
- **D6 (kind-priority sort):** added `model.kindOrder` map + `compareNodes` /
  `compareEdgesBySource` comparators. Updated all 5 sort sites: `renderEntityList`,
  `renderRelationshipTable`, `model.searchFilter` (2 sites), `wireSearch`.
- **D5 (tooltips):** `dot.nodeAttrs` tooltip now `id: title · kind · status` with
  `kindLabel || kindPrefix` fallback.
- **D7 (legend dimming):** `dimLegend(neighbourhood)` reads `data-labels` attrs,
  diffs against current edge labels, toggles `.legend-dimmed`. Called in
  `renderGraphPane` `.then()` after `wireSvgHandlers`.
- **D8 (filter/search DRY):** `renderEntityList(query)` accepts optional query;
  `renderFilteredEntities` delegates; `wireSearch` input handler delegates.

All VA verification deferred to browser session.

## 2026-06-16 — PHASE-01 complete (5febc5d)

HTML + CSS structural refactors. Gate green. All 4 design decisions applied:

- **D1 (SVG bg transparent):** removed `.graph-area svg { background: … }`
  rules from CSS. SVG inherits page `--bg`. DOT `bgcolor="transparent"` already set.
- **D2 (filter 1-col):** filter-grid now `grid-template-columns: 1fr`;
  checkboxes reordered per D2 importance (PRD/SPEC first, QUE/CON last).
- **D3 (hover pane):** `min-height: 3.4rem; overflow: hidden` — no wobble.
- **D4 (depth selector):** moved from sidebar to main pane below `.graph-area`.
  Added `data-depth` attrs; updated `wireDepthButtons` and `render()` depth-sync
  to read `data-depth` instead of `textContent`.
- **D7 (legend HTML/CSS):** `data-labels` attrs on all 7 legend items;
  `.legend-dimmed { opacity: 0.3 }` in CSS.

**VA-1 / VA-2 deferred:** visual verification requires browser. VA-2 reload
half depends on D9 (PHASE-02 bugfix).

## 2026-06-16 — code-review batch 1 (bd8823d)

Gate: green (1471 passed). JS syntax: clean.

### 🔴 Blockers fixed

- **render() focus-change bug**: `render()` was skipping `renderGraphPane` on
  same-depth focus switches, only applying `applyFocusHighlight`. BFS is
  centre-centric — changing focus at *any* depth changes the neighbourhood.
  Fixed: always re-render on focus change; apply highlight first for instant
  visual feedback before async render completes.

- **Keyboard nav reset**: `state.listNavIndex` was reset on ANY non-nav key in
  the `keydown` handler, resetting during search typing. Removed the catch-all
  `else` branch. Reset now only happens when the entity list DOM rebuilds
  (via `renderEntityList`/`renderFilteredEntities`).

### 🟠 Structural improvements

- `buildEntityItem(node)` factory extracted — 3 call sites now DRY
  (`renderEntityList`, `wireSearch` input handler, `renderFilteredEntities`).

- Fullscreen button: rewired from inline `onclick` with escaped quotes to
  `wireMarkdownPane(container)` — `addEventListener` on `.fullscreen-toggle`.

- `collectKindFilter`: reads `data-kinds` attribute from checkboxes instead of
  parsing label `textContent.split('/')`. HTML checkboxes now carry
  `data-kinds="SL"` / `data-kinds="ADR,POL"` etc.

- Edge colour map: replaced `indexOf` substring chain with `dot._EDGE_COLORS`
  exact-label lookup (13 entries).

### 🟡 Minor

- Hit-area rect: added `bbox.width > 0 && bbox.height > 0` guard.
- `render()` syncs depth button `.active` class on every render.
- Escape key calls `input.blur()` after clearing.

### Open design questions (for /design)

- Static edge legend — always shows 7 types regardless of graph content.
  Should it be dynamic (only present types)?
- filter/search DRY: `renderFilteredEntities` delegates to `renderEntityList`
  or runs search — but search DOM building is its own code path. The factory
  helps but the branching is still two-path.
- SVG background + `style="filled"` interaction — CSS sets `background: #fcfcfc`
  on SVGs; `style="filled"` on DOT nodes applies node fills. Coherent in testing
  but the dark-theme SVG background (`#f5f5f5`) hasn't been visually verified.

### Remaining scope (not yet addressed)

Of the 16 items in scope, addressed so far: #1 (click feedback), #2 (diagram
legibility + style=filled), #5 (hover pane flex), #6 (toggle-all), #7
(fullscreen), #8 (link colour), #9 (hit-area rect), #11 (edge colours), #12
(filter labels+desc), #13-14 (sidebar reorder), #15 (search-preserving filter),
#16 (keyboard nav).

Not yet addressed: #3 (sidebar entity click feedback — clicks work now but
no additional transition), #4 (depth button click feedback — depth changes
 trigger full redraw, active sync added), #10 (diagram colours — root cause
 same as #2).

Items #3, #4, #10 may be satisfied by the render fix + depth sync. Check
before scoping new work.
