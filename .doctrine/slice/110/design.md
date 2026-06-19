# Design: Web map UX polish — actionability/concept-map view interactions

Scope: `slice-110.md`. Five UX defects/gaps on the same view-interaction
surface (`web/map/src/`). This design roots each in its current code path,
states the target, and names the exact seam. Two small **pure functions** are
the load-bearing additions; the rest is wiring and CSS.

> **Revision (post external review).** Codex (GPT-5.5) adversarial pass found 1
> blocker + 7 majors against the locked draft, all verified against code. Item 5
> was rebuilt on the universal focus funnel (was a `goto`-only patch that missed
> table-rows / deep-links / back-button); `relabel_edge` gained a duplicate-edge
> guard; edge identity is label-based not key-based; the hover extraction escapes
> all fields. See `## Review integration` for the finding-by-finding ledger.

> **Revision 2 (post-VH-walkthrough, RV-098).** The dispatched implementation
> shipped and was audited on the live dev server. Items 1, 2, 3 passed (item 3
> needed a follow-up markup fix — landed). **Items 4 and 5 failed acceptance**
> (RV-098 F-4 blocker, F-5 major) and are **re-designed below**:
> - **Item 4** — the "Edit this / Edit all" top-button model is rejected. New
>   model: per-cell **hover-revealed pencils** (inline-in-place edit), an `[ ] edit
>   all` **checkbox at the top of the edge table** that toggles edit *scope*
>   (single instance vs all matching), **always-visible** per-row `✕` delete, and
>   the **add-edge form relocated directly below the hover area**. Two new backend
>   ops back the single-instance edits (per-occurrence node rename; bulk
>   relabel-by-rel). See the rewritten **Item 4** + **D5/D6**.
> - **Item 5** — D2's "non-member stays in actionability" is **reversed**: picking
>   a non-actionability entity while on the actionability graph now **switches to
>   Semantic**. `focusTransition` simplifies (drops `requiredMode`/`node`). See the
>   rewritten **Item 5** + **D2**.

## Cross-cutting additions (the two seams)

Everything below leans on two new pure functions, extracted so the behaviour is
unit-testable and not duplicated (`no parallel implementation`, pure/imperative
split — both clock/DOM-free):

- **`focusTransition(current, focusId, isActionabilityMember, currentPriorityZoomId)`
  → `{ viewMode, priorityZoomId }`** (in `model.ts`). The whole item-5 rule as one
  pure table (see Item 5). **Revision 2:** the signature drops `node` (and the
  inner `requiredMode`) — the only remaining use of `node` was its id, which equals
  the focused id, so the shell passes `focusId` directly; the CM-forces-semantic
  case is now subsumed by the non-member→semantic rule (a CM is never an
  actionability member). Takes the current zoom id so the semantic "leave it alone"
  row can echo a concrete `priorityZoomId` (it cannot express "unchanged" without
  it). Applied at **focus-change** in `renderView`, gated on `focusChanged` — not
  on every render (see D1), not in `goto`.

- **`hoverDetailHtml(node)` → `string`** (in `render.ts`, exported). The inner
  markup of the hover-details pane, factored out of `hoverPane` so the new
  actionability tooltip (item 2) renders identical content from the same source.
  **Escapes every interpolated field** (id, title, kindLabel, status) — this
  fixes a latent gap where `hoverPane` escaped only `title` (render.ts:507). For
  catalog/actionability nodes only; the concept-map hover (`renderCmHoverPane`)
  stays a separate, intentionally-different renderer (see D3).

## Item 5 — focus-change drives the view (foundational)

### Current behavior
`renderView` is the `hashchange` handler (`app.ts:225`) and **already diffs
focus**: it saves `prevFocusId` (`app.ts:273`), sets `state.focusId = route.id`
from the parsed hash (`app.ts:276`), and computes `focusChanged`
(`app.ts:349`). But it never acts on a focus change to switch mode or zoom:
- Picking a concept map from the left pane while in actionability mode re-focuses
  but renders nothing visible (CMs don't appear in the priority graph).
- Picking an actionable item while *in* actionability mode focuses it in state but
  does **not** zoom the priority graph — `priorityZoomId` is set only on in-graph
  node click (`app.ts:399`), so left-pane / table-row / deep-link picks don't zoom.

Every selection path — left-pane `li` (`onFocus → goto → setFocus`), in-graph
click, actionability table row (sets the hash directly, `render.ts:356`),
deep-link, back-button — funnels through `renderView` via the hash. `goto` is
**not** that funnel; the funnel is the `route`/`focusChanged` derive already in
`renderView`.

### Target behavior
Add a **switch+zoom derive inside `renderView`, gated on `focusChanged`**, placed
right after `state.focusId` is updated (≈`app.ts:276`) and **before**
`highlightViewButtons` (item 1), so the active button reflects the post-switch
mode:

```ts
// renderView, after route parse sets state.focusId; focusChanged = id !== prevFocusId
if (focusChanged) {
  const member = actionabilityNodeIds.has(state.focusId ?? '')   // from state.actionabilityView
  const t = focusTransition(state.viewMode, state.focusId, member, state.priorityZoomId)
  state.viewMode = t.viewMode
  if (t.priorityZoomId !== state.priorityZoomId) {
    state.priorityZoomId = t.priorityZoomId
    state.priorityZoomPending = t.priorityZoomId !== null
  }
}
highlightViewButtons(state.viewMode)
```

Pure transition (DOM/clock-free, in `model.ts`) — **Revision 2 table** (D2
reversed):

| case | → `viewMode` | → `priorityZoomId` |
|---|---|---|
| actionability + member | `'actionability'` | `focusId` |
| actionability + **non-member** | `'semantic'` | `null` |
| current is semantic | `'semantic'` | `currentPriorityZoomId` *(echo)* |

```ts
function focusTransition(
  current: ViewMode,
  focusId: string | null,
  isActionabilityMember: boolean,
  currentPriorityZoomId: string | null,
): { viewMode: ViewMode; priorityZoomId: string | null }
```

The rule collapses to: **on the actionability graph, focusing a member zooms to
it; focusing anything else switches to Semantic (where it is visible) and clears
the now-stale zoom. In Semantic, focus never auto-switches mode** (the user chose
Semantic; the asymmetry is deliberate — see D2). Because a concept-map node is
never an actionability member, the old `requiredMode` CM-forces-semantic case is
subsumed by the non-member→semantic row; `requiredMode` and the `node` argument
are removed.

Why the `focusChanged` gate and render-time (this reverses **both** earlier passes
— see D1): firing on every render made the **Actionability** toggle dead
(first pass); scoping to `goto` missed table-rows / deep-links / back-button
(second pass). The existing `focusChanged` machinery fires the derive **once per
real focus change** — a toggle click or depth change re-renders without changing
`focusId`, so the derive is skipped and the toggle stays live.

`goto` reverts to trivial `setFocus(id, state.depth)`. The manual
`priorityZoomId` set at the in-graph click site is removed — the funnel owns zoom
now (the in-graph click changes focus → `focusChanged` → derive sets it).

### Code impact
- `model.ts` — simplify `focusTransition` (drop `requiredMode` + `node` arg); the
  non-member branch now returns `semantic`.
- `app.ts` — `focusChanged`-gated derive passes `state.focusId` (not the node);
  build `actionabilityNodeIds` from `state.actionabilityView`; `goto` → `setFocus`;
  drop the manual in-graph `priorityZoomId` set.
- `model.test.ts` — update the `focusTransition` table tests: actionability +
  non-member now → `{semantic, null}`; CM focus (any mode) → `semantic`; semantic
  focus echoes the zoom; member → `{actionability, focusId}`.

## Item 1 — view-toggle button state follows mode

### Current behavior
Two bugs stack. (a) **Class mismatch:** the CSS styles `.view-btn.active`
(`priority.css:23`) and the initial HTML seeds `class="view-btn active"`
(`index.html:84`), but `renderView` toggles `view-btn--active` (`app.ts:539`) —
a *different* (BEM) class with no CSS rule. So the active highlight never tracks
the toggled class; the seed `active` is never removed either. (b) **Early
return:** the highlight pass sits at the **end** of `renderView`
(`app.ts:536–540`); the edge-detail branch returns early at `app.ts:322`, so
toggling while an edge is focused never reaches it.

### Target behavior
- **Unify the class** on the BEM convention used by `depth-btn--active`
  (`app.ts:344`): rename `priority.css:23` `.view-btn.active` → `.view-btn--active`;
  change the HTML seed `class="view-btn active"` → `class="view-btn view-btn--active"`
  for the default (semantic) button. Now the class the code toggles is the class
  the CSS styles.
- **Hoist** the highlight: extract `highlightViewButtons(mode)` and call it
  **once, early** in `renderView` (right after the item-5 derive sets the final
  `state.viewMode`, before any early return); delete the late `L536–540` block.

### Code impact
- `priority.css` — rename `.view-btn.active` → `.view-btn--active`.
- `web/map/index.html` — default button seed class.
- `app.ts` — module-private `highlightViewButtons`; hoisted call; delete
  `L536–540`.

## Item 2 — hover tooltip on actionability nodes

### Current behavior
`priority.ts` node groups wire `mouseenter`/`mouseleave` to optional callbacks
(`priority.ts:287–294`); `app.ts` routes those to the side `hoverPane`
(`app.ts:414–422`). There is no on-graph tooltip. `hoverPane` itself escapes
only `node.title`; `id`, `kindLabel`, `status` are injected raw (`render.ts:507`).

### Target behavior
A styled HTML tooltip overlay inside the graph container, showing the same
content as the details pane via the new `hoverDetailHtml`. On `mouseenter`
populate + show; on `mousemove` position near the cursor; on `mouseleave` hide.
Native `<title>` is rejected — can't match the pane styling the user asked for.

`hoverPane` is refactored to call `hoverDetailHtml`, which escapes **every**
field (closes the latent raw-injection gap). The hover content is no longer
byte-identical to today's markup — it is the *escaped* form; the behaviour test
asserts escaping, not legacy bytes.

### Code impact
- `render.ts` — extract + export `hoverDetailHtml` (escape-all); `hoverPane`
  reuses it.
- `priority.ts` `renderGraph` — append one absolutely-positioned tooltip element
  to `container`; drive it from the existing per-node hover listeners using the
  loop's `node` (`{id, title, kind, status}`).
- `graph.css` / `priority.css` — `.priority-tooltip` styling (reuse
  `hover-detail-*` rules where possible).

## Item 4 — concept-map per-cell pencil edit + edit-all scope

### Current behavior (shipped in PHASE-04, rejected at VH — RV-098 F-4)
The dispatched implementation shipped the "select + Edit this / Edit all" model:
`cmSelectedField` + `editingField` state, a `renderEditToggle` that appends two
buttons (**Edit this** / **Edit all**) to the **focus header**, and a
click-to-select gesture. The live walkthrough rejected it:
- the two buttons render on **every** entity, not just concept maps —
  `renderEditToggle` is called outside the `if (isCm)` guard (`app.ts:602` vs the
  `if (isCm)` block at `:614`);
- a single top **Edit this** button whose target ("this") is invisible is
  confusing — the user wants a per-field affordance;
- clicking a cell drops into batch edit instead of plainly selecting/highlighting;
- "Edit all" is a confusing name for a mode, and clicking a field while in it
  bulk-edits.

The DSL is `source_label > rel > target_label` per line; all mutations match **by
label** and dedupe by derived key (`parse_dsl`). PHASE-01 shipped `relabel_edge`
(single edge by triple, key-based dup guard) — **retained**; it backs one cell of
the new matrix.

### Target behavior (Revision 2)
No top buttons. The edge table is the whole edit surface:

- **`[ ] edit all` checkbox at the top of the edge table** — a pure **scope**
  toggle (state `cmEditAll: boolean`). It does *not* gate which cells are editable;
  it switches every edit between *this instance* and *all matching instances*
  (matching = all table rows sharing the clicked label).
- **Per-cell hover-revealed pencil.** Every cell (source node, relation, target
  node) shows a pencil on hover. Click the pencil → that cell becomes an inline
  `<input>` in place (Enter commits, Esc cancels). The op is chosen by
  (cell-kind × `cmEditAll`):

  | cell | edit all OFF (single) | edit all ON (all matching) |
  |---|---|---|
  | node | `rename_node_occurrence` (**new**) — rename just this row's endpoint | `rename_node` (existing) — rename that label everywhere |
  | relation | `relabel_edge` (existing) — relabel just this edge | `relabel_rel_all` (**new**) — relabel every edge with this rel |

- **Plain click on a node cell** (not the pencil) → `cmFocus` highlight that node
  (existing highlight + neighbourhood filter); relation-cell plain click is inert.
- **Always-visible per-row `✕` delete** (`remove_edge`, existing) — no longer
  gated behind a mode.
- **Add-edge form always visible, relocated directly below the hover area**
  (`.hover-detail`) — moved out of the bottom-of-panel slot.
- All of the above renders **only for concept maps** — the buttons-everywhere bug
  is fixed by deleting `renderEditToggle` and rendering the checkbox *inside* the
  edge table, which is already `isCm`-gated (`renderCmEdgeTable`).

The op-selection is a pure function (unit-testable):

```ts
type CmCell = 'from' | 'rel' | 'to'
type CmEditOp = 'rename_node_occurrence' | 'rename_node' | 'relabel_edge' | 'relabel_rel_all'
function cmEditOp(cell: CmCell, editAll: boolean): CmEditOp
```

New transient state (replaces `editingConceptMap` / `cmSelectedField` /
`editingField` / `editingNode`):
- `cmEditAll: boolean` — the scope toggle.
- `cmEditingCell: { from_label: string; rel: string; to_label: string; cell: CmCell } | null`
  — which cell's pencil is active (the inline input). Carries the full edge labels
  (to locate the row) + which segment; the current label is `from_label` / `rel` /
  `to_label` by `cell`.
- `cmFocusNode` — retained for the node highlight (navigation).

### Backend ops (`concept_map.rs` + `routes.rs`)
Four ops back the matrix; **two are new**. All match by label, guard duplicates
key-based (`derive_node_key`), and short-circuit no-ops — same shape as
`relabel_edge_in_dsl` (PHASE-01).

- **`rename_node` (existing)** — label-global rename; backs node + edit-all ON.
- **`relabel_edge` (existing, PHASE-01)** — single edge by triple; backs relation
  + edit-all OFF. (Spec retained in PHASE-01 history; unchanged.)
- **`rename_node_occurrence` (new)** — rewrite **one** edge's endpoint label.
  Inputs `{ source, rel, target, cell: 'source'|'target', new_label }`: find the
  first line matching the triple (`EdgeNotFound` else), rewrite only the named
  endpoint segment to `new_label` (other rows using the old label untouched),
  empty-reject, no-op when unchanged, **key-based dup guard** (the rewritten triple
  must not collide with another line).
- **`relabel_rel_all` (new)** — rewrite the `rel` segment of **every** line whose
  `rel == old_rel` to `new_rel`. Inputs `{ old_rel, new_rel }`. Empty-reject, no-op
  when equal, **atomic** key-based dup guard: if any rewrite would collide with an
  existing or other-rewritten triple, reject the whole op (`DuplicateEdge { line }`)
  — no partial write.

Reuse `EmptyField` / `EdgeNotFound` / `DuplicateEdge`; **no new error variants**.

> Pre-existing gap (unchanged): `add_edge_to_dsl`'s dup check is label-based, not
> key-based — distinct label spellings deriving the same key slip it. Out of
> scope; tracked as a backlog item (see Follow-ups).

### Code impact
- `src/concept_map.rs` — `rename_node_occurrence_in_dsl`, `relabel_rel_all_in_dsl`
  (both ride the `relabel_edge_in_dsl` shape: label-match + key-based dup guard +
  no-op short-circuit). `relabel_edge_in_dsl` / `rename_node_in_dsl` unchanged
  (behaviour-preservation).
- `src/map_server/routes.rs` — two new `MutationAction` variants + apply arms.
- `web/map/index.html` — move `.cm-add-edge-form` to directly below `.hover-detail`.
- `types.ts` / `state.ts` — add `cmEditAll`, `cmEditingCell`; **remove**
  `editingConceptMap`, `cmSelectedField`, `editingField`, `editingNode`.
- `api.ts` — no change (`mutateConceptMap` takes a generic action).
- `concept-map.ts` — **delete** `renderEditToggle`; `renderEdgeTable` renders the
  `[ ] edit all` checkbox header, per-cell hover pencils, always-on `✕`, the
  inline-input arm gated on `cmEditingCell`, and node-cell plain-click → cmFocus;
  add the pure `cmEditOp`; keep `cmSelectedFieldFromCell`'s label-capture logic
  (re-homed onto `cmEditingCell`).
- `app.ts` — **delete** the `renderEditToggle` call (kills buttons-everywhere);
  wire the checkbox to `cmEditAll`, pencil clicks to `cmEditingCell`, commit to the
  matrix op (`cmEditOp`) via `mutateConceptMap`; render the add-edge form
  unconditionally for CMs in its new slot.
- `concept-map.css` — hover-pencil affordance, inline-input, checkbox header,
  always-on delete, add-form-below-hover layout.

## Item 3 — filter sidebar `[ ] all` checkbox alignment

### Current behavior
The "all" kind-filter checkbox sits above the per-kind checkboxes
(`.kind-checkbox` rows) but is mis-aligned with them.

### Target behavior / code impact
CSS-only: align the "all" row to the same grid/inset as the per-kind rows in
`sidebar.css`. No JS change. (Confirm the markup is consistent; adjust the
`search.ts` filter markup only if the misalignment is structural, not stylistic.)

## Verification alignment

- **`focusTransition`** (Revision 2) — `model.test.ts` table: actionability +
  member → `{actionability, focusId}`; actionability + non-member →
  `{semantic, null}` (the D2 reversal); semantic focus → `{semantic,
  currentPriorityZoomId}` (echoes the passed zoom id); `null` focusId → safe.
  This is the whole item-5 behaviour; only the 1-line membership wiring in
  `renderView` is manual.
- **`hoverDetailHtml`** — `render` unit test: returns markup containing id,
  title, kindLabel, status; asserts **unsafe chars are escaped** in every field
  (regression test for the old raw injection); `hoverPane` delegates to it.
- **`relabel_edge_in_dsl`** — Rust unit tests: relabel-persists (200);
  `EdgeNotFound` (404); `EmptyField` (400); **`DuplicateEdge` by key-collision**
  (target `(from_key,new_rel,to_key)` exists under a *different label spelling* —
  the case the label check misses); **`old_rel == new_rel` → DSL unchanged**
  (no-op, no false collision); plus a `routes.rs` handler test for the new action.
- **Item 1** — `highlightViewButtons` toggles `view-btn--active` for the matching
  data-view (DOM unit test); a render-pass test confirms the class survives the
  edge-detail early return.
- **Item 4** (Revision 2) — pure `cmEditOp(cell, editAll)` unit-tested over the
  4-cell matrix. Rust unit tests for the two new ops: `rename_node_occurrence`
  rewrites one endpoint and leaves other rows' use of the old label intact, with
  key-collision `DuplicateEdge`, `EdgeNotFound`, `EmptyField`, and no-op cases;
  `relabel_rel_all` rewrites every line sharing the rel, atomic-rejects on any
  key-collision (no partial write), no-op when equal, plus `routes.rs` handler
  tests for both new actions. Click/commit/pencil/checkbox wiring is manual
  (VH-1). `relabel_edge` / `rename_node` tests stay green unchanged.
- **Item 3** — visual.
- **No regression** in existing `web/map` vitest suites (`dot`, `model`,
  `router`, `viewport`, `priority`) and the `routes.rs` mutation tests.

## Design decisions

- **D1 — switch+zoom is a render-time derive gated on `focusChanged`, in
  `renderView`.** First pass put an *ungated* `requiredMode` derive in
  `renderView` → fired every render → dead Actionability toggle. Second pass moved
  it to `goto` → missed every non-`goto` selection path (table rows set the hash
  directly, deep-links, back-button). Resolution: reuse the `prevFocusId` /
  `focusChanged` machinery already in `renderView` (the *universal* funnel every
  path reaches via `hashchange`) and gate the derive on `focusChanged`. Fires once
  per real focus change; plain re-renders (toggle, depth) don't change `focusId`
  so the toggle stays live. Hash-encoded mode (deep-linkable, back-button-exact)
  is more correct but needs `Route`/`parseHash`/`buildHash` + every `setFocus`
  caller + the toggle buttons rewired — out of polish scope. → Follow-up.
- **D2 (Revision 2 — reversed) — on the Actionability graph, focusing a
  non-member switches to Semantic.** The original D2 left a non-member focus on
  the actionability graph (showing an informative placeholder) and only cleared
  the stale zoom. VH rejected this: picking a non-actionable entity should take
  you to the graph where it *is* visible (Semantic). New rule: actionability +
  member → zoom; actionability + non-member → **switch to Semantic, clear zoom**;
  Semantic focus never auto-switches (the user chose Semantic — the asymmetry is
  deliberate: Semantic shows everything, so it is never a dead-end). Because a CM
  is never an actionability member, this subsumes the old CM-forces-semantic case
  — `requiredMode` and the `node` argument are dropped (the function needs only
  the focused id for the zoom target).
- **D3 — tooltip reuses `hoverDetailHtml`, escape-all.** One content source for
  the catalog/actionability pane + its tooltip; rejects a forked renderer and
  native `<title>`. Scope is catalog/actionability only — the concept-map hover
  (`renderCmHoverPane`, `concept-map.ts:56`) is deliberately different content and
  stays separate. The extraction also escapes all fields (previously only
  `title`), so it is not byte-identical to the old pane markup.
- **D4 (Revision 2 — superseded by D5).** The original D4 (cell click = select;
  explicit **Edit this** / **Edit all** buttons) was rejected at VH (RV-098 F-4):
  the top buttons are distant and the "this" target is invisible. Replaced by the
  per-cell pencil model in D5. Retained here as the rejected baseline.
- **D5 — per-cell hover pencils; `edit all` is a scope toggle, not a mode.** Each
  editable cell carries a hover-revealed pencil that inline-edits *that* field in
  place — the affordance sits on the thing it edits (no ambiguous "this"). The
  `[ ] edit all` checkbox (top of the edge table, not the focus header) flips the
  *scope* of every edit between the single clicked instance and all rows sharing
  that label — it never changes which cells are editable, and a plain field-click
  never edits (node cells highlight; relations are inert). Delete affordance (`✕`)
  and the add-edge form are **always** present (the latter relocated directly below
  the hover area), so there is no "edit mode" to enter. Identities stay
  **label-based** (a label can recur and distinct labels collide on key), so
  `cmEditingCell` carries the full edge labels.
- **D6 — two new label-based DSL ops back the single-instance edits.** The matrix
  needs `rename_node_occurrence` (rename one endpoint occurrence) and
  `relabel_rel_all` (relabel every edge sharing a rel); `rename_node` and
  `relabel_edge` cover the other two cells. Both new ops ride the
  `relabel_edge_in_dsl` shape (label-match, key-based dup guard, no-op
  short-circuit, reused error variants); `relabel_rel_all` rejects atomically on
  any collision (no partial write). The user accepted the expanded backend
  exception (now three CM mutations beyond the original frontend-only posture);
  the pre-existing `add_edge` key-collision blind-spot remains a separate backlog
  item, not fixed here.

## Review integration

External adversarial pass (Codex / GPT-5.5) against the locked draft; all
findings verified against code before disposition.

| # | sev | finding | disposition |
|---|---|---|---|
| F1 | major | CSS `.view-btn.active` vs code `view-btn--active` — hoist alone wouldn't style | **accepted** — Item 1 unifies class on BEM + hoist |
| F2 | major | non-member focus → stale `priorityZoomId`, silent no-zoom | **accepted** — `focusTransition` clears on non-member (D2) |
| F3 | major | `goto` not the funnel; table-rows/deep-links/back-button bypass switch+zoom | **accepted** — Item 5 rebuilt on `renderView` `focusChanged` funnel (D1) |
| F4 | major | DSL matches by label; key-based rel identity is ambiguous | **accepted** — `cmSelectedField` rel identity = labels (D4) |
| F5 | blocker | relabel can synth a duplicate triple; mirroring remove misses it | **accepted** — `relabel_edge_in_dsl` guards `DuplicateEdge` like add |
| F6 | major | `hoverPane` escapes only `title`; byte-identical extraction keeps raw injection | **accepted** — `hoverDetailHtml` escapes all fields; byte-identical claim dropped (D3) |
| F7 | minor | single-field edit vs global `editing` flag underspecified (codex symbol names were wrong) | **accepted, narrowed** — `editingField` arm specified independent of `editing` |
| F8 | minor | `renderCmHoverPane` is a separate hover renderer | **accepted** — `hoverDetailHtml` scoped catalog/actionability only (D3) |
| F9 | major | pure seams thin; item 5 punted to manual | **accepted** — `focusTransition` is a pure table test covering all item-5 cases |

Second pass (Codex, against the integrated revision):

| # | sev | finding | disposition |
|---|---|---|---|
| G1 | blocker | relabel "use add_edge's dup check" insufficient — add_edge checks labels, parse dedups by key; collision corrupts DSL | **accepted** — guard is key-based (`derive_node_key`), excludes matched line; pre-existing add_edge gap flagged for backlog |
| G2 | major | relabel-to-itself underspecified | **accepted** — `old_rel == new_rel` → DSL unchanged, before the dup scan |
| G3 | major | `focusTransition` can't express "unchanged zoom" — no current zoom arg | **accepted** — signature takes `currentPriorityZoomId`, echoes it |
| G4 | major | node selection key-only repeats the collision class; `rename_node` is label-based | **accepted** — `cmSelectedField` node carries `label`; submits `old_label` |
| G5 | minor | slice-110.md OQ-3 still says "applied in `goto`… not render-time" | **accepted** — OQ-3 reworded to the `renderView` `focusChanged` funnel |

Third pass — **VH walkthrough on the shipped implementation** (RV-098, live dev
server). Items 1, 2, 3 accepted; items 4 and 5 failed and drove **Revision 2**:

| # | sev | finding | disposition |
|---|---|---|---|
| RV-098 F-4 | blocker | item-4 Edit this/Edit all rejected: buttons on every entity (outside `isCm`), ambiguous "this", click drops to batch edit, "Edit all" confusing | **re-designed** — per-cell pencils + `edit all` scope checkbox + always-on delete + add-form below hover (D5/D6); buttons-everywhere fixed by deleting `renderEditToggle` |
| RV-098 F-5 | major | item-5 D2: picking a non-actionable entity on the actionability graph should switch to Semantic, not sit on a placeholder | **re-designed** — D2 reversed: non-member → Semantic; `focusTransition` simplified |
| RV-098 F-1 | major | item-3 `all` checkbox not left-aligned with per-kind rows (CSS-only got intra-row spacing only) | **fixed + verified** — markup moved out of the right-floated header (candidate `0a9d1e1b`) |

## Open questions — resolved

- OQ-1 governance → slice `specs` PRD-011 (done; the actionability/priority
  product intent this view exposes).
- OQ-2 item-4 interaction → **Revision 2**: per-cell hover pencils + `edit all`
  scope checkbox (D5), superseding the rejected Edit this / Edit all split (D4).
- OQ-3 "actionable" → resolved via membership-in-view (`isActionabilityMember`);
  **Revision 2**: a non-member focus on the actionability graph switches to
  Semantic (D2), so no `requiredMode` kind-check is needed.

## Follow-ups

- Deep-linkable view mode in the URL hash (`?view=…`) — the larger, separate want
  behind D1.
- `add_edge_to_dsl` key-collision blind-spot (label-based dup check; distinct
  label spellings deriving the same key slip it) — pre-existing, tracked as a
  backlog item (D6), not fixed in SL-110.
- Larger frontend cleanups remain their own backlog items: IMP-085
  (decomposition), IMP-086 (vendor pinning), IMP-087 (theme), IMP-089 (ARIA).

## Doctrinal alignment

- **ADR-001 layering** — both new functions are pure (`focusTransition` in
  `model.ts`, `hoverDetailHtml` in `render.ts`); impurity (DOM, hash, membership
  lookup) stays in the `app.ts` shell. No new cycles.
- **Pure/imperative split** — `focusTransition` / `hoverDetailHtml` / `cmEditOp`
  take inputs, touch no clock/rng/disk/DOM; the new DSL ops
  (`rename_node_occurrence_in_dsl`, `relabel_rel_all_in_dsl`) are pure string
  transforms. Membership and DOM stay in the `app.ts` shell.
- **Behaviour-preservation** — existing `web/map` vitest suites *and* the
  `routes.rs` mutation tests stay green unchanged. The two new CM ops are additive
  (new enum variants + apply arms + DSL fns) — `relabel_edge` / `rename_node` /
  `remove_edge` / `add_edge` arms untouched. `focusTransition`'s contract changes
  by design (Revision 2); its own model tests update with it. Frontend state
  removals (`editingConceptMap`/`cmSelectedField`/`editingField`) are item-4-local.
- **No parallel implementation** — tooltip reuses the pane builder; the item-5
  rule is one pure transition; the four CM edit cells route through one pure
  `cmEditOp` selector, not scattered per-cell branching.
