# SL-075 Design

## 1. Target behaviour

All 16 scope items from the UX review are addressed. This design covers the
remaining design-quality concerns that surfaced after batch 1 landed at
`bd8823d` — SVG background coherence, filter layout, hover pane stability,
tooltips, sort order, edge legend presence signalling, filter/search DRY, and
a depth-button bugfix.

## 2. Design decisions

### D1: SVG background → transparent

**Decision:** Drop the hardcoded light SVG background (`.graph-area svg {
background: #fcfcfc; }` and dark-theme `#f5f5f5`). DOT has `bgcolor="transparent"`
set on the graph — nodes render on the page `--bg` (white in light, `#1a1a1a` in
dark). SVG inherits page background.

**Rationale:** The forced light background was a belt-and-suspenders workaround
for when nodes were transparent outlines. With `style="filled"` active (item #2 /
#10), node fills carry their own contrast. Letting the SVG go transparent
harmonises it with the theme switch.

**Fallback gate (binary):** if any node fill or edge line is illegible on the
dark `--bg` (`#1a1a1a`), restore the light-theme-only faint fill —
`@media (prefers-color-scheme: light) { .graph-area svg { background: #fcfcfc; } }`
— keeping dark theme transparent. The gate is one visual check; if it fails
for *any* element, the fallback is applied unconditionally (no per-element
graduation).

**Edge colours on dark bg:** `#888888` (depends/requires) on `#1a1a1a` is
~1.6:1 contrast — below the WCAG AA 3:1 minimum for UI components. This alone
likely triggers the fallback. If the fallback fires, consider bumping the
depends/requires edge colour to `#aaaaaa` as a lighter touch.

**Verification:** `doctrine map serve --open --focus SL-072 --depth 2` in both
themes. All node fills and edge lines legible against `--bg`. Gate: fail-fast
if any element is not legible; apply fallback unconditionally rather than
iterating.

### D2: Filter checkboxes → 1-column, kind-importance order

**Decision:** Reorder the 12 filter checkboxes into a single column, ordered by
project importance (matching the sidebar entity-list ordering, D5):

1. PRD / SPEC — Products / Specs
2. ADR / POL — Decisions / Policies
3. STD — Standards
4. SL — Slices
5. ISS / IMP / CHR / RSK — Issues / Risks / Chores
6. REV — Revisions
7. RV — Reviews
8. REQ — Requirements
9. IDE — Ideas
10. REC — Records
11. ASM / DEC — Assumptions / Decisions
12. QUE / CON — Questions / Concerns

CSS: `grid-template-columns: 1fr` instead of `1fr 1fr`. No JS change.

### D3: Hover pane → fixed height

**Decision:** Set `.hover-detail` to `min-height: 3.4rem` (accommodates
title + meta on two lines) and `overflow: hidden` to prevent layout wobble
when transitioning between placeholder and populated state.

### D4: Depth selector → below graph

**Decision:** Move the depth button group from the sidebar (`.depth-selector`) to
the main pane, immediately below `.graph-area`. Relabel buttons from bare digits
to "Depth: 0 / 1 / 2 / 3" to make the control self-describing when displaced from
the sidebar context. Depth change triggers full graph reload (router via hash).

### D5: Tooltips → title, kind, status

**Decision:** Replace bare-ID DOT tooltips with a descriptive string:
`"SL-075: Map Explorer UX overhaul · Slice · started"`.
Format: `id: title · kind · status` where `kind` is `kindLabel` when non-empty,
falling back to `kindPrefix`.

Implemented in `dot.nodeAttrs()` — the node object carries `title`, `kindLabel`,
`kindPrefix`, and `status`. No CSS or interaction change.

### D6: Sort order → kind-priority then numeric-ID

**Decision:** All entity lists (sidebar, search results) sort by kind
importance first (same order as D2 filter list), then by numeric ID within
each kind group. The relationship table sorts edges by the **source node's**
kind priority then numeric ID — edges themselves don't carry kindPrefix; the
resolved `srcNode` does.

Implementation: a `kindOrder` map in `model.js`:

```javascript
model.kindOrder = {
  PRD: 1, SPEC: 1, ADR: 2, POL: 2, STD: 3, SL: 4,
  ISS: 5, IMP: 5, CHR: 5, RSK: 5, REV: 6, RV: 7,
  REQ: 8, IDE: 9, REC: 10, ASM: 11, DEC: 11, QUE: 12, CON: 12
};
```

Sort comparator for nodes:

```javascript
function compareNodes(a, b) {
  var ordA = model.kindOrder[a.kindPrefix] || 99;
  var ordB = model.kindOrder[b.kindPrefix] || 99;
  if (ordA !== ordB) return ordA - ordB;
  var numA = parseInt(a.id.split('-').pop(), 10) || 0;
  var numB = parseInt(b.id.split('-').pop(), 10) || 0;
  if (numA !== numB) return numA - numB;
  return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
}
```

For the relationship table, edges carry no kindPrefix — sort by resolved
source node:

```javascript
function compareEdgesBySource(ea, eb) {
  var sa = state.graph.nodes.get(ea.source);
  var sb = state.graph.nodes.get(eb.source);
  if (!sa || !sb) return (ea.id < eb.id ? -1 : 1);
  return compareNodes(sa, sb);
}
```

Affected call sites: `renderEntityList()`, `renderRelationshipTable()` (edge
sort via source node), `renderFilteredEntities()` / `wireSearch`
search-results path, `model.searchFilter()` (already sorts — replace
comparator).

### D7: Edge legend → static, dim absent types

**Decision:** Keep the 7 static legend entries. Diff the edge labels in the
current neighbourhood against each legend item and add `.legend-dimmed` to
absent types (`opacity: 0.3` in CSS).

Implementation: add `data-labels` to each HTML legend item matching
`dot._EDGE_COLORS` keys. The dimming pass runs inside `renderGraphPane`'s
`.then()` callback — after `wireSvgHandlers(svgEl, …)` — so it fires when
the SVG DOM is present and the neighbourhood edges are known. A standalone
`dimLegend(neighbourhood)` function keeps it testable.

### D8: Filter/search DRY → `renderEntityList(query)`

**Decision:** Fold `renderFilteredEntities` into `renderEntityList` by passing
an optional query parameter. `renderEntityList` applies `model.searchFilter` when
a non-empty query is present, falls back to the full node set otherwise. The
kind-filter check is shared. `renderFilteredEntities` becomes a thin wrapper that
reads the search input and delegates.

### D9: Depth button bugfix

**Decision:** Remove the premature `state.depth = d` assignment from the depth
button click handler. `render()` already derives `state.depth` from the route;
the shadow write caused `prevDepth === state.depth` in render, suppressing graph
re-render on depth change.

Before:
```javascript
state.depth = d;
router.setFocus(state.focusId, d);
```

After:
```javascript
router.setFocus(state.focusId, d);
```

The button `.active` sync in `render()` (lines 708–712) already handles visual
state.

## 3. Code impact

| File | Change |
|---|---|
| `web/map/style.css` | Drop SVG bg rules (§D1); hover pane fixed height (§D3); 1-col filter grid (§D2); legend dimming class (§D7) |
| `web/map/index.html` | Reorder filter checkboxes (§D2); move depth selector to main (§D4); legend `data-labels` attrs (§D7); depth button labels (§D4) |
| `web/map/dot.js` | Tooltip content (§D5); no colour-table changes |
| `web/map/model.js` | `model.kindOrder` map + `compareNodes`/`compareEdgesBySource` comparators (§D6); searchFilter sort (§D6) |
| `web/map/app.js` | `renderEntityList(query)` refactor (§D8); depth button handler bugfix (§D9); `dimLegend()` in `renderGraphPane` `.then()` (§D7); kind-priority sort (§D6) |

No changes to `router.js`.

## 4. Verification

- `node --check web/map/*.js` — clean
- `just check` / `just gate` — green (no Rust changes)
- Visual: `doctrine map serve --open --focus SL-072 --depth 2`
  - D1: All node fills + edge lines legible in both themes; fallback applied
    unconditionally if any element fails (§D1)
  - D2: Filter checkboxes single-column, ordered, no truncation
  - D3: Hover pane height stable on node hover
  - D4/D9: Depth selector below graph, active sync, graph reloads on click
  - D5: SVG tooltips show title + kind + status (kind falls back to prefix)
  - D6: Sidebar entities sorted kind-first then numeric-ID; relationship
    table sorted by source-node kind
  - D7: Legend entries dim when edge type absent from current graph
  - D8: Search + kind filter composable, no duplicate list-building paths
