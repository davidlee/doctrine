# Notes SL-075: Map Explorer UX overhaul

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
