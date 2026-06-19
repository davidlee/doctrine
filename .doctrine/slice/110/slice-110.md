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
   button highlight does not reliably follow `state.viewMode`. The
   active-button highlight must track the *selected* mode. (Design found the
   CSS-styled class `.view-btn.active` and the code-toggled class
   `view-btn--active` differ — unify on the BEM form + hoist the highlight past
   the edge-detail early return.) Surface: `priority.css` / `index.html` (class
   unify) + `app.ts` (~L536–539 highlight pass).

2. **Hover tooltip on actionability nodes** — actionability-graph nodes have no
   hover affordance. Add a tooltip matching the existing "hover for details"
   pane (parity with the semantic-graph hover). Surface: `render.ts` /
   `svg.ts` (actionability render path), reusing the details-pane content.

3. **Filter sidebar `[ ] all` checkbox alignment** — the "all" kind-filter
   checkbox is mis-aligned relative to the per-kind checkboxes below it.
   CSS-only fix. Surface: `sidebar.css` / the filter markup in `search.ts`.

4. **Concept-map edit UX** — current edit flow is awkward. Desired (rev 2, after
   the shipped Edit this/Edit all model was rejected at VH — RV-098 F-4):
   - per-cell **hover-revealed pencil** → inline-edit *that* field in place;
   - an `[ ] edit all` **checkbox at the top of the edge table** = a scope toggle
     (single instance vs all rows sharing the label), not a mode;
   - plain node-cell click highlights the node; **always-visible** per-row `✕`
     delete; the **add-edge form relocated directly below the hover area**.
   Two new label-based DSL ops back the single-instance edits
   (`rename_node_occurrence`, `relabel_rel_all`); `rename_node` / `relabel_edge`
   back the others. Surface: `concept-map.ts` (`renderEdgeTable`, delete
   `renderEditToggle`), `app.ts`, `concept_map.rs` / `routes.rs`, `index.html`.
   See design §Item 4, D5/D6.

5. **Selection wiring** — selecting an item should change *view mode*, not just
   focus:
   - choosing a **concept map** should activate **Semantic** view (today it
     does nothing visible if already in actionability mode),
   - choosing an **actionable** entity (SL/backlog with dep/seq edges) should
     **focus that element in the Actionability graph**.
   (Design found the universal seam is the `renderView` `focusChanged` derive —
   the funnel *every* selection path reaches via `hashchange`, not just the
   left-pane `onFocus` — so the rule lands once as a pure `focusTransition` and
   covers table-rows / deep-links / back-button too.) Surface: `app.ts`
   `renderView` focus-change derive + pure `focusTransition` in `model.ts`;
   actionability-membership check (cf. the terminal/no-edge placeholders at
   `app.ts` ~L446–450).

## Non-Goals

- Predominantly the exposure surface — **a scoped set of CM mutation
  exceptions**: `relabel_edge` (PHASE-01, shipped) plus two new ops the rev-2
  item-4 matrix needs (`rename_node_occurrence`, `relabel_rel_all`). No other
  backend / CLI / actionability-graph *data* changes.
- No broad frontend refactor (modular decomposition is IMP-085; theme toggle
  IMP-087; semantic-HTML/ARIA IMP-089; vendor pinning IMP-086). Touch only
  what each fix needs; flag larger cleanups as follow-ups rather than absorbing
  them.
- No new view modes or graph kinds.

## Affected Surface

- `web/map/src/app.ts` — view-toggle highlight, onFocus → mode transition (1, 5)
- `web/map/src/render.ts`, `svg.ts` — actionability node tooltip (2)
- `web/map/src/concept-map.ts` — per-cell pencils, edit-all checkbox, inline edit, delete (4)
- `web/map/index.html` — relocate `.cm-add-edge-form` below `.hover-detail` (4)
- `web/map/src/search.ts` — filter markup, left-pane onFocus (3, 5)
- `web/map/src/sidebar.css` (+ `concept-map.css`, `graph.css`) — alignment, tooltip, pencil/inline-edit styling
- `src/concept_map.rs`, `src/map_server/routes.rs` — `relabel_edge` (done) + new `rename_node_occurrence`, `relabel_rel_all` (item 4)
- Tests: vitest specs alongside (`app`/`model`/`concept-map`/`render` pattern); Rust tests in `routes.rs`/`concept_map.rs` for the new ops.

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
- **OQ-2** Item 4 interaction → **resolved (rev 2)**: per-cell hover pencils →
  inline edit in place; `[ ] edit all` checkbox = scope toggle; plain node-cell
  click highlights; always-on `✕`; add-edge form below the hover area (design D5).
  Supersedes the rejected Edit this / Edit all model (D4). Four ops back the
  cell×scope matrix; two are new (`rename_node_occurrence`, `relabel_rel_all`).
- **OQ-3** Item 5 "actionable" / mode switch → **resolved (rev 2)**: pure
  `focusTransition`, applied as a `focusChanged`-gated derive in the `renderView`
  funnel every selection path reaches via `hashchange` (not `goto`; design D1).
  A non-member focus on the actionability graph switches to Semantic (D2).

## Verification / Closure Intent

- Each of the 5 items has an observable acceptance check (button highlight
  tracks mode; tooltip appears on hover with details parity; checkbox aligned;
  per-cell pencil inline-edits with correct single/all scope, plain click
  highlights, delete + add-form always present; non-member pick on the
  actionability graph switches to Semantic, member pick zooms).
- vitest green for any extracted pure logic (mode-transition decision,
  actionable-eligibility predicate).
- Manual walkthrough in the live dev server confirming each item.
- No regression in existing web/map vitest suites (behaviour-preservation).

## Summary

## Follow-Ups
