# Design: Web map UX polish — actionability/concept-map view interactions

Scope: `slice-110.md`. Five UX defects/gaps on the same view-interaction
surface (`web/map/src/`). This design roots each in its current code path,
states the target, and names the exact seam. Two small **pure predicates** are
the load-bearing additions; the rest is wiring and CSS.

## Cross-cutting additions (the two seams)

Everything below leans on two new pure functions, extracted so the behaviour is
unit-testable and not duplicated (`no parallel implementation`, pure/imperative
split — both are clock/DOM-free):

- **`requiredMode(node)` → `'semantic' | null`** (in `model.ts`). The mode a
  focus entity can *only* be shown in. Concept maps render solely in the
  semantic graph → `'CM'` returns `'semantic'`. Every other kind shows in the
  entity graph (and in actionability when eligible) → returns `null` (no forced
  mode). This is the whole "switch-if-undisplayable" rule (item 5): only a CM is
  display-constrained; nothing is actionability-only. Applied at the **selection
  event** (`goto`), not as a render invariant — see D1.

- **`hoverDetailHtml(node)` → `string`** (in `render.ts`, exported). The inner
  markup of the hover-details pane, factored out of `hoverPane` so the new
  actionability tooltip (item 2) renders identical content from the same source.

## Item 5 — left-pane selection drives the view (foundational)

### Current behavior
`goto(id) → setFocus(id)` writes only the focus hash (`router.ts:107`).
`state.viewMode` is never touched by selection — it changes only when a
view-toggle button is clicked (`app.ts:162`). So choosing a concept map from the
left pane while in actionability mode re-focuses but renders nothing visible
(CMs don't appear in the priority graph). Choosing an actionable item from the
left pane while *in* actionability mode focuses it in state but does **not** zoom
the priority graph to it — `priorityZoomId` is set only on in-graph node click
(`app.ts:399`).

### Target behavior
Both the mode switch and the zoom happen in the **`goto(id)` selection handler**
— fired on the pick (left pane *or* in-graph click), not on every render:

```ts
function goto(id: string): void {
  const node = state.graph.nodes.get(id)
  if (requiredMode(node) === 'semantic') {
    state.viewMode = 'semantic'                 // CM picked → show it
  } else if (state.viewMode === 'actionability') {
    state.priorityZoomId = id                   // actionable picked in actionability → zoom to it
    state.priorityZoomPending = true
  }
  setFocus(id, state.depth)
}
```

Why the handler and not a render-time derive (this reverses the first-pass
design — see D1): the actionability branch renders the **whole** priority graph
regardless of focus (`focusId` only drives the zoom-highlight; the empty-view
placeholder fires only when the *entire* view has zero nodes). So a CM focused
in actionability is **not** a blank dead-end — it just shows the full graph with
nothing zoomed. A render-time `requiredMode` revert would therefore fire on
*every* render and make the **Actionability** toggle button dead whenever a CM
is focused (click → instant revert to semantic). Scoping the switch to the
selection event avoids that and still satisfies the user intent ("picking a CM
does nothing" → now switches to semantic). In-graph actionability node click
already sets `priorityZoomId` before calling `goto`, so the zoom line is a
no-op there; it only adds the missing left-pane path. Background-click reset is
untouched (it doesn't go through `goto`).

### Code impact
- `model.ts` — add `requiredMode`.
- `app.ts` — `goto` (L40) gains the switch + zoom; no `renderView` derive.

## Item 1 — view-toggle button state follows mode

### Current behavior
The `view-btn--active` highlight pass sits at the **end** of `renderView`
(`app.ts:536–540`), off `state.viewMode`. The edge-detail branch returns early
at `app.ts:322`, so clicking a toggle while an edge is focused never reaches the
highlight → stale button.

### Target behavior
Extract `highlightViewButtons(mode)` and call it **once, early** in `renderView`
(immediately after the item-5 derive sets the final `state.viewMode`), before
any early return. Remove the late duplicate. Now the active button tracks the
selected/derived mode in every page mode.

### Code impact
- `app.ts` — new module-private `highlightViewButtons`; hoisted call; delete
  L536–540 block.

## Item 2 — hover tooltip on actionability nodes

### Current behavior
`priority.ts` node groups wire `mouseenter`/`mouseleave` to optional callbacks
(`priority.ts:287–294`); `app.ts` routes those to the side `hoverPane`
(`app.ts:414–422`). There is no on-graph tooltip.

### Target behavior
A styled HTML tooltip overlay inside the graph container, showing the same
content as the details pane (`hoverDetailHtml`). On `mouseenter` populate +
show; on `mousemove` position near the cursor; on `mouseleave` hide. Native
`<title>` is rejected — it can't match the pane styling the user asked for.

### Code impact
- `render.ts` — extract + export `hoverDetailHtml`; `hoverPane` reuses it.
- `priority.ts` `renderGraph` — append one absolutely-positioned tooltip element
  to `container`; drive it from the existing per-node hover listeners using the
  loop's `node` (`{id, title, kind, status}`).
- `graph.css` / `priority.css` — `.priority-tooltip` styling (reuse
  `hover-detail-*` rules where possible).

## Item 4 — concept-map cell-select + Edit this / Edit all

### Current behavior
One global **Edit/Done** toggle (`cm-edit-toggle`, `concept-map.ts:326`) flips
the entire edge table into edit mode: every node label becomes a `cm-editable-node`
(click → rename input), every row gets a ✕ remove button, and the add-edge form
appears (`concept-map.ts:186–272`). Outside edit mode the table is inert. Each
row is three fields: `[source node] · [relation] · [target node]`.

### Target behavior
Selection and editing split into a select gesture + two scoped buttons:

- **Click a cell → select that field.** A node cell also focuses that node in
  the CM graph (the existing `cmFocus` highlight + neighbourhood filter); the
  relation cell selects that edge. Selection is plain navigation — no inline
  input appears on click.
- **Edit this** → inline-edit *only the currently-selected field*:
  - node cell → `rename_node` (exists today; keyed by `from_key`/`to_key`, so it
    renames that node across every row),
  - relation cell → `relabel_edge` (**new** backend op, below).

  Enter commits, Esc cancels. Disabled when nothing is selected.
- **Edit all** → today's global edit mode (all fields editable + ✕ + add-edge
  form), unchanged.

New transient state to carry the selected field:
`cmSelectedField: { kind: 'node' | 'rel'; key: string } | null` (node `key`, or
edge identity `source>rel>target` for `rel`). `editingConceptMap` (edit-all)
stays; add an `editingField` flag distinguishing single-field edit from edit-all
so `renderEdgeTable` renders one input vs the full edit surface.

### Backend: `relabel_edge` mutation (new)
Relations are display-only today — the three mutations are `add_edge`,
`remove_edge`, `rename_node` (`routes.rs:35–55`), each delegating to a pure
line-based DSL transform in `concept_map.rs` (the DSL is `source > rel > target`
per line). Add a fourth, mirroring `remove_edge_from_dsl` exactly but **rewriting
the `rel` segment of the matched line instead of dropping it**:

```rust
// routes.rs
#[serde(rename = "relabel_edge")]
RelabelEdge { source: String, old_rel: String, new_rel: String, target: String },
// apply arm → concept_map::relabel_edge_in_dsl(&old_dsl, source, old_rel, new_rel, target)

// concept_map.rs — near-copy of remove_edge_from_dsl; reuses
//   ConceptMapMutationError::{EmptyField, EdgeNotFound}; no new error variants.
pub(crate) fn relabel_edge_in_dsl(
    old_dsl: &str, source: &str, old_rel: &str, new_rel: &str, target: &str,
) -> Result<String, ConceptMapMutationError>
```

This is the only backend change in the slice (a deliberate, scoped exception to
the frontend-only posture — see slice §Non-Goals).

### Code impact
- `src/concept_map.rs` — `relabel_edge_in_dsl` (mirrors `remove_edge_from_dsl`).
- `src/map_server/routes.rs` — `MutationAction::RelabelEdge` variant + apply arm.
- `state.ts` / `types.ts` — `cmSelectedField`, `editingField`.
- `api.ts` — no change (`mutateConceptMap` already takes a generic action).
- `concept-map.ts` — `renderEditToggle` becomes two buttons (Edit this / Edit
  all); `renderEdgeTable` gains a select-on-click path and a single-field-edit
  render arm (node label *or* relation).
- `app.ts` — wire the two buttons; route cell selection to `cmFocus` (nodes) /
  `cmSelectedField` (rel); `handleRelabelEdge` calling `mutateConceptMap`.
- `concept-map.css` — selected-cell affordance, two-button layout, pencil icon.

## Item 3 — filter sidebar `[ ] all` checkbox alignment

### Current behavior
The "all" kind-filter checkbox sits above the per-kind checkboxes
(`.kind-checkbox` rows) but is mis-aligned with them.

### Target behavior / code impact
CSS-only: align the "all" row to the same grid/inset as the per-kind rows in
`sidebar.css`. No JS change. (Confirm the markup is consistent; adjust the
`search.ts` filter markup only if the misalignment is structural, not stylistic.)

## Verification alignment

- **`requiredMode`** — `model.test.ts`: `CM` → `'semantic'`; `SL`/`ISS`/`IMP`/
  `undefined` → `null`.
- **`hoverDetailHtml`** — `render` unit test: given a node, returns markup
  containing id, title, kind, status; `hoverPane` output unchanged
  (behaviour-preservation).
- **`relabel_edge_in_dsl`** — Rust unit tests mirroring `remove_edge`:
  200/relabel-persists, `EdgeNotFound` (404), `EmptyField` (400); plus a
  `routes.rs` handler test for the new action.
- **Item 5 switch** — `requiredMode` unit test (above) covers the predicate; the
  `goto` switch + zoom verified manually in the dev server.
- **Item 1** — `highlightViewButtons` toggles the right `view-btn--active`
  given a mode (DOM unit test).
- **Item 4** — select → Edit this commits a single field; Edit all unchanged
  (manual + any extractable pure bits, e.g. selected-field → render decision).
- **Item 3** — visual.
- **No regression** in existing `web/map` vitest suites (`dot`, `model`,
  `router`, `viewport`, `priority`).

## Design decisions

- **D1 — switch mode in the `goto` selection handler, not `renderView`, not the
  hash.** First-pass design put a `requiredMode` derive in `renderView`; the
  adversarial pass killed it: actionability renders the whole graph regardless
  of focus, so CM-in-actionability is not a dead-end, and a render-time revert
  would make the Actionability toggle dead while a CM is focused. The switch is
  selection-scoped instead. Hash-encoded mode (deep-linkable, back-button-exact)
  is more correct but needs `Route`/`parseHash`/`buildHash` + every `setFocus`
  caller + the toggle buttons rewired — out of polish scope. → Follow-up.
- **D2 — `requiredMode` returns `null` for non-CM.** "Switch-if-undisplayable"
  means we only force a switch when the current view genuinely cannot show the
  focus. The entity (semantic) graph shows everything, so only a CM-in-
  actionability is a dead-end. Terminal/no-edge items keep their existing
  actionability placeholder — not auto-switched (informative, user chose the
  mode).
- **D3 — tooltip reuses `hoverDetailHtml`.** One content source for pane +
  tooltip; rejects a forked renderer and native `<title>`.
- **D4 — item 4 click = select, not edit.** Per user: a cell click navigates/
  selects; editing is an explicit button. Keeps accidental edits out and gives
  a clear scoped (Edit this) vs bulk (Edit all) split.

## Open questions — resolved

- OQ-1 governance → slice `specs` PRD-011 (done; the actionability/priority
  product intent this view exposes).
- OQ-2 item-4 interaction → resolved to select + Edit this / Edit all (D4).
- OQ-3 "actionable" → resolved via `requiredMode`/eligibility-in-view; no
  kind-list duplication (D2).

## Follow-ups

- Deep-linkable view mode in the URL hash (`?view=…`) — the larger, separate
  want behind D1.
- Larger frontend cleanups remain their own backlog items: IMP-085
  (decomposition), IMP-086 (vendor pinning), IMP-087 (theme), IMP-089 (ARIA).

## Doctrinal alignment

- **ADR-001 layering** — both new predicates are pure (`model.ts` /
  `render.ts`); impurity (DOM, hash) stays in the `app.ts` shell. No new cycles.
- **Pure/imperative split** — `requiredMode`/`hoverDetailHtml` take inputs,
  touch no clock/rng/disk/DOM.
- **Behaviour-preservation** — existing `web/map` vitest suites *and* the
  `routes.rs` mutation tests must stay green unchanged; `hoverPane` output is
  byte-identical after the extraction. The `relabel_edge` add is purely additive
  (new enum variant + apply arm + DSL fn) — existing mutation arms untouched.
- **No parallel implementation** — tooltip reuses the pane builder; mode rule is
  a single predicate, not scattered kind-checks.
