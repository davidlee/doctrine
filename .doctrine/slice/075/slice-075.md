# Map Explorer UX overhaul — click feedback, diagram legibility, sidebar layout, filter polish, fullscreen markdown, keyboard nav

## Context

SL-073 shipped the Doctrine Map Explorer SPA with functional correctness
verified by RV-039 (8/8 high-severity findings resolved). A product-side UX
review surfaced 16 acceptance-blocking issues — zero correctness impact, all
perceptual / interaction design.

This slice captures the changes from that review.

## Scope — 16 items

| # | Area | Change |
|---|---|---|
| 1 | Click feedback | Client-side focus transition — SVG node highlight swap without Graphviz API call on same-depth navigation |
| 2 | Diagram legibility | SVG background fill (white/light grey) + `style=filled` on DOT nodes (was missing — all nodes rendered as transparent outlines) |
| 3 | Click feedback | Sidebar entity-list item click → immediate visible transition (active state, header, table, markdown) |
| 4 | Click feedback | Depth button click → immediate full redraw (depth change triggers API call) |
| 5 | Hover pane | Flex layout with gap, styled title/meta, min-height |
| 6 | Filter | Toggle-all checkbox wired to all kind checkboxes with bi-directional sync |
| 7 | Markdown | Fullscreen toggle button in markdown toolbar |
| 8 | Link colour | CSS `--link` variables — #2563eb on light, #60a5fa on dark |
| 9 | SVG hit area | `cursor: pointer` on nodes + transparent hit-area `<rect>` injected into each `<g>` |
| 10 | Diagram legibility | Root cause: `style="filled"` was missing from DOT node statements — all nodes transparent. Fixed. |
| 11 | Edge styling | Colour + fontcolour by semantic edge label in `dot.js`; static legend in sidebar |
| 12 | Filter labels | 2-column grid with abbreviation + description; header row with "Filter by kind" + "all" toggle |
| 13 | Sidebar layout | Depth selector moved above filter area |
| 14 | Sidebar layout | Refresh button moved under search bar |
| 15 | Filter | `renderFilteredEntities()` checks active search query; filter toggle preserves text filter |
| 16 | Keyboard nav | ArrowUp/ArrowDown highlights entity list items; Enter selects; Escape clears |

## Affected surface

- `web/map/app.js` — render split, client-side focus transition, toggle-all, keyboard nav, fullscreen button, search-preserving filter
- `web/map/style.css` — SVG background, link colour, active states, hover pane layout, filter grid, legend, fullscreen overlay, keyboard highlight
- `web/map/index.html` — sidebar reorder (search → refresh → depth → filter), filter grid with descriptions, edge legend
- `web/map/dot.js` — `style="filled"` on node statements, edge colour/fontcolour by semantic label
- `web/map/router.js` — no changes
- `web/map/model.js` — no changes

## Non-Goals

- Backend changes
- New data or entity types
- Performance optimisation beyond the client-side transition split
- Accessibility audit (keyboard nav is a start, not full a11y)

## Risks

- None. All changes are client-side presentation/interaction. Gate (`just check` / `node --check`) verified green.

## Verification

- `node --check web/map/*.js` — clean
- `just check` — 4 tests, 0 failures (root package)
- `just gate` — workspace clean
- Visual: `doctrine map serve --open --focus SL-072 --depth 2` — all 16 items exercisable

## Follow-Ups

- Full a11y pass (ARIA labels, focus management, screen-reader announcements)
- Edge legend could be dynamic (only show types present in current graph)
- Entity list keyboard nav could work without search active (navigate full list)
