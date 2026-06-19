# Web map UX polish: actionability/concept-map view interactions

## Context

The Doctrine Map Explorer (`web/map/`, TypeScript + Vite, embedded via
`rust-embed`) exposes two graph views — **Semantic** (the DOT/Graphviz entity
graph, incl. concept maps) and **Actionability** (the dep/seq priority graph,
SPEC-001 projection). View mode is held in `state.viewMode`
(`'semantic' | 'actionability'`) and toggled by the header view buttons; the
left sidebar lists/filters entities and drives `state.focusId`.

User walkthrough of the live view surfaced five concrete UX defects/gaps. None
is enormous; they are coherent polish on the same view-interaction surface, so
they ride one slice rather than five backlog items.

## Scope & Objectives

Five UX items (design phase will root-cause each before fixing):

1. **View-toggle button state desync** — the Actionability / Semantic header
   button highlight (`view-btn--active`) does not reliably follow
   `state.viewMode`. The active-button highlight must track the *selected* mode.
   Surface: `app.ts` (~L536–539 highlight pass; the ISS-020 `viewModeChanged`
   logic nearby).

2. **Hover tooltip on actionability nodes** — actionability-graph nodes have no
   hover affordance. Add a tooltip matching the existing "hover for details"
   pane (parity with the semantic-graph hover). Surface: `render.ts` /
   `svg.ts` (actionability render path), reusing the details-pane content.

3. **Filter sidebar `[ ] all` checkbox alignment** — the "all" kind-filter
   checkbox is mis-aligned relative to the per-kind checkboxes below it.
   CSS-only fix. Surface: `sidebar.css` / the filter markup in `search.ts`.

4. **Concept-map edit UX** — current edit flow is awkward. Desired:
   - clicking a table cell focuses *that* table (inline edit of the cell),
   - a pencil button provides explicit "edit all" (the current global toggle).
   Surface: `concept-map.ts` (`renderEdgeTable`, `renderEditToggle`,
   `cm-editable-node` click handlers).

5. **Left-pane selection wiring** — selecting an item in the left sidebar
   should change *view mode*, not just focus:
   - choosing a **concept map** should activate **Semantic** view (today it
     does nothing visible if already in actionability mode),
   - choosing an **actionable** entity (SL/backlog with dep/seq edges) should
     **focus that element in the Actionability graph**.
   Surface: `app.ts` `onFocus` handler / `search.ts` `onFocus` wiring /
   `state.viewMode` transition; `isConceptMap` and actionability-eligibility
   checks (cf. the terminal/no-edge placeholders at `app.ts` ~L446–450).

## Non-Goals

- Predominantly the exposure surface — **one scoped backend exception**: a new
  `relabel_edge` concept-map mutation (item 4 needs it; relations are
  display-only today). No other backend / CLI / actionability-graph *data*
  changes.
- No broad frontend refactor (modular decomposition is IMP-085; theme toggle
  IMP-087; semantic-HTML/ARIA IMP-089; vendor pinning IMP-086). Touch only
  what each fix needs; flag larger cleanups as follow-ups rather than absorbing
  them.
- No new view modes or graph kinds.

## Affected Surface

- `web/map/src/app.ts` — view-toggle highlight, onFocus → mode transition (1, 5)
- `web/map/src/render.ts`, `svg.ts` — actionability node tooltip (2)
- `web/map/src/concept-map.ts` — cell-focus edit + pencil edit-all (4)
- `web/map/src/search.ts` — filter markup, left-pane onFocus (3, 5)
- `web/map/src/sidebar.css` (+ maybe `concept-map.css`, `graph.css`) — alignment, tooltip, pencil styling
- `src/concept_map.rs`, `src/map_server/routes.rs` — new `relabel_edge` mutation (item 4)
- Tests: vitest specs alongside (`app`/`model`/`router` test pattern); Rust tests in `routes.rs`/`concept_map.rs` for `relabel_edge`.

## Risks / Assumptions

- Item 5 couples left-pane selection to mode — must not regress the ISS-020
  view-mode re-render path or the terminal/no-edge placeholders.
- "Actionable" eligibility for an entity must be decided consistently with the
  graph's own admission rule (status-terminal items, no-dep/seq items) so a
  click doesn't land on an empty actionability view.
- Tooltip (item 2) should reuse the details-pane content builder, not fork a
  second renderer (DRY).

## Open Questions (resolved in design.md)

- **OQ-1** Governance linkage → **resolved**: slice `specs` PRD-011.
- **OQ-2** Item 4 interaction → **resolved**: cell click = select (node cell
  focuses the node); explicit **Edit this** (selected field) / **Edit all**
  buttons. Relation "Edit this" adds a scoped backend `relabel_edge` mutation.
- **OQ-3** Item 5 "actionable" / mode switch → **resolved**: `requiredMode`
  predicate, switch-if-undisplayable, applied in the `goto` selection handler
  (not a render-time derive — see design D1).

## Verification / Closure Intent

- Each of the 5 items has an observable acceptance check (button highlight
  tracks mode; tooltip appears on hover with details parity; checkbox aligned;
  cell-click + pencil behave as specified; left-pane selection drives the right
  view + focus).
- vitest green for any extracted pure logic (mode-transition decision,
  actionable-eligibility predicate).
- Manual walkthrough in the live dev server confirming each item.
- No regression in existing web/map vitest suites (behaviour-preservation).

## Summary

## Follow-Ups
