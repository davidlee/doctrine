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

## Cross-cutting additions (the two seams)

Everything below leans on two new pure functions, extracted so the behaviour is
unit-testable and not duplicated (`no parallel implementation`, pure/imperative
split — both clock/DOM-free):

- **`focusTransition(current, node, isActionabilityMember, currentPriorityZoomId)`
  → `{ viewMode, priorityZoomId }`** (in `model.ts`). The whole item-5 rule as one
  pure table (see Item 5). Takes the current zoom id so it can return a concrete
  `priorityZoomId` for the "leave it alone" rows (it cannot express "unchanged"
  without it). Wraps the inner **`requiredMode(node)` → `'semantic' | null`**
  check off `node.kind` (`types.ts` Node carries `kind: string`; a CM kind renders
  *only* in the semantic graph → `'semantic'`; every other kind → `null`, no
  forced mode). Applied at **focus-change** in `renderView`, gated on
  `focusChanged` — not on every render (see D1), not in `goto`.

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
  const node = state.graph.nodes.get(state.focusId ?? '')
  const member = actionabilityNodeIds.has(state.focusId ?? '')   // from state.actionabilityView
  const t = focusTransition(state.viewMode, node, member, state.priorityZoomId)
  state.viewMode = t.viewMode
  if (t.priorityZoomId !== state.priorityZoomId) {
    state.priorityZoomId = t.priorityZoomId
    state.priorityZoomPending = t.priorityZoomId !== null
  }
}
highlightViewButtons(state.viewMode)
```

Pure transition (DOM/clock-free, in `model.ts`):

| case | → `viewMode` | → `priorityZoomId` |
|---|---|---|
| `requiredMode(node)==='semantic'` (CM) | `'semantic'` | `null` |
| actionability + member | `'actionability'` | `node.id` |
| actionability + **non-member** | `'actionability'` | `null` *(clears stale)* |
| else (semantic, non-CM) | `current` | `currentPriorityZoomId` *(echo)* |

```ts
function focusTransition(
  current: ViewMode,
  node: Node | undefined,
  isActionabilityMember: boolean,
  currentPriorityZoomId: string | null,
): { viewMode: ViewMode; priorityZoomId: string | null }
```

Why the `focusChanged` gate and render-time (this reverses **both** prior passes
— see D1): firing on every render made the **Actionability** toggle dead
(first pass); scoping to `goto` missed table-rows / deep-links / back-button
(second pass). The existing `focusChanged` machinery fires the derive **once per
real focus change** — a toggle click or depth change re-renders without changing
`focusId`, so the derive is skipped and the toggle stays live. The non-member
branch clears a stale `priorityZoomId` (picking a CM, an ADR/REQ/MEM, or any item
absent from the priority layout no longer leaves the graph zoomed to a
now-invisible node).

`goto` reverts to trivial `setFocus(id, state.depth)`. The manual
`priorityZoomId` set at the in-graph click site is removed — the funnel owns zoom
now (the in-graph click changes focus → `focusChanged` → derive sets it).

### Code impact
- `model.ts` — `focusTransition` (+ inner `requiredMode`).
- `app.ts` — `focusChanged`-gated derive block in `renderView`; build
  `actionabilityNodeIds` from `state.actionabilityView`; `goto` → `setFocus`;
  drop the manual in-graph `priorityZoomId` set.

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

## Item 4 — concept-map cell-select + Edit this / Edit all

### Current behavior
One global edit flag, `editing: boolean` + `editingNode: {key,label}|null`
(`concept-map.ts:37–38`), flips the entire edge table into edit mode: the whole
render is gated `if (editing)` (`concept-map.ts:200, 209, 217, 229`) — every node
label becomes a `cm-editable-node`, every row gets a ✕ remove button, and the
add-edge form appears. Outside edit mode the table is inert. Each row is three
fields: `[source node] · [relation] · [target node]`. The DSL behind it is
`source_label > rel > target_label` per line; the three mutations
(`add_edge`/`remove_edge`/`rename_node`, `routes.rs:35–55`) match lines **by
label**, not by derived key (`remove_edge_from_dsl` compares the raw line
segments to the passed `source`/`rel`/`target`).

### Target behavior
Selection and editing split into a select gesture + two scoped buttons:

- **Click a cell → select that field.** A node cell also focuses that node in the
  CM graph (existing `cmFocus` highlight + neighbourhood filter); the relation
  cell selects that edge. Selection is plain navigation — no inline input on click.
- **Edit this** → inline-edit *only the currently-selected field*:
  - node cell → `rename_node` (exists; submits `old_label`/`new_label` —
    **label-based**, `routes.rs:48`; renames that label everywhere it appears),
  - relation cell → `relabel_edge` (**new** backend op, below).

  Enter commits, Esc cancels. Disabled when nothing is selected.
- **Edit all** → today's global edit mode (all fields editable + ✕ + add-edge
  form), unchanged.

New transient state:
- `cmSelectedField: { kind: 'node'; key: string; label: string } | { kind: 'rel';
  from_label: string; rel: string; to_label: string } | null`. **Both identities
  carry labels, not just derived keys** — every CM mutation (`rename_node`,
  `add/remove/relabel_edge`) is label-based, and distinct labels can derive the
  same key, so the clicked cell's label must be captured from the `CmEdge`
  (`from_label`/`to_label`) at select time. The node `key` is retained for the
  `cmFocus` highlight; the `label` is what the rename submits as `old_label`.
- `editingField: 'node' | 'rel' | null` — drives a single-input render arm
  **independent of** `editing`. Today every editable arm is gated `&& editing`;
  the single-field arm renders one input when `editingField !== null` *without*
  entering edit-all (no ✕, no add-edge form). `editing` (edit-all) is untouched.

### Backend: `relabel_edge` mutation (new)
Add a fourth mutation. It matches the target line **by label** like
`remove_edge_from_dsl`, but **rewrites the `rel` segment instead of dropping the
line** — and, unlike remove, it must guard against creating a duplicate:

```rust
// routes.rs
#[serde(rename = "relabel_edge")]
RelabelEdge { source: String, old_rel: String, new_rel: String, target: String },
// apply arm → concept_map::relabel_edge_in_dsl(&old_dsl, source, old_rel, new_rel, target)

// concept_map.rs
pub(crate) fn relabel_edge_in_dsl(
    old_dsl: &str, source: &str, old_rel: &str, new_rel: &str, target: &str,
) -> Result<String, ConceptMapMutationError>
```

Errors: reuse `EmptyField` and `EdgeNotFound` (from remove) **plus
`DuplicateEdge { line }`**. Relabelling `A > old > B` to `new` can produce a
`(from_key, new_rel, to_key)` triple that already exists on another line; without
a guard `parse_dsl` then emits `DuplicateEdge` and skips the duplicate, leaving
the stored DSL invalid.

The guard must be **key-based**, not the label check `add_edge_to_dsl` currently
uses. `parse_dsl` defines duplicate identity by `(from_key, rel, to_key)`
(`concept_map.rs:421`), but `add_edge_to_dsl`'s own dup check compares *labels*
(`e.from_label == source …`, `:1235`) — so two distinct labels deriving the same
key (`User Story` vs `User-Story`) slip a label check and collide only at parse.
`relabel_edge_in_dsl` therefore: (1) trim and reject empty fields; (2) **if
`old_rel == new_rel` after trim, return the DSL unchanged** (no-op, no false
collision); (3) find the label-matched line (`source`/`old_rel`/`target`), else
`EdgeNotFound`; (4) scan the *parsed* edges for an existing
`(derive_node_key(source), new_rel, derive_node_key(target))` triple **excluding
the matched line**; on hit return `DuplicateEdge { line }`; (5) rewrite the `rel`
segment of the matched line.

> Observed pre-existing gap: `add_edge_to_dsl`'s label-based dup check has the
> same key-collision blind spot. Out of scope here (slice touches only the new
> op) — flagged as a backlog candidate, not fixed in SL-110.

This is the only backend change in the slice (a deliberate, scoped exception to
the frontend-only posture — see slice §Non-Goals).

### Code impact
- `src/concept_map.rs` — `relabel_edge_in_dsl` (remove-shaped match + add-shaped
  duplicate guard).
- `src/map_server/routes.rs` — `MutationAction::RelabelEdge` variant + apply arm.
- `state.ts` / `types.ts` — `cmSelectedField` (label-based rel identity),
  `editingField`.
- `api.ts` — no change (`mutateConceptMap` already takes a generic action).
- `concept-map.ts` — `renderEditToggle` → two buttons (Edit this / Edit all);
  `renderEdgeTable` gains a select-on-click path and a single-field-edit arm
  gated on `editingField` (independent of `editing`).
- `app.ts` — wire the two buttons; route cell selection to `cmFocus` (nodes) /
  `cmSelectedField` (carrying the clicked `label` for both kinds);
  `handleRelabelEdge` calling `mutateConceptMap`.
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

- **`focusTransition`** — `model.test.ts` table: CM → `{semantic, null}`;
  actionability + member → `{actionability, id}`; actionability + non-member →
  `{actionability, null}` (clears stale); semantic + non-CM → `{current,
  currentPriorityZoomId}` (echoes the passed zoom id); `undefined` node → safe (no
  throw, no forced mode). This is the whole item-5 behaviour; only the 2-line
  membership wiring in `renderView` is manual.
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
- **Item 4** — select → Edit this commits a single field via `relabel_edge` /
  `rename_node`; Edit all unchanged. Extract the pure bit if any (selected-field
  → render-arm decision); the click/commit wiring is manual.
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
- **D2 — `requiredMode` returns `null` for non-CM; `focusTransition` clears zoom
  on non-members.** "Switch-if-undisplayable" forces a switch only when the
  current view genuinely cannot show the focus — the semantic graph shows
  everything, so only a CM-in-actionability is a dead-end. For actionability,
  picking an id absent from the priority layout (terminal item, ADR/REQ/MEM,
  non-actionable) is not auto-switched (the user chose the mode, the placeholder
  is informative) but **does** clear any stale `priorityZoomId` so the graph isn't
  left zoomed to an invisible node.
- **D3 — tooltip reuses `hoverDetailHtml`, escape-all.** One content source for
  the catalog/actionability pane + its tooltip; rejects a forked renderer and
  native `<title>`. Scope is catalog/actionability only — the concept-map hover
  (`renderCmHoverPane`, `concept-map.ts:56`) is deliberately different content and
  stays separate. The extraction also escapes all fields (previously only
  `title`), so it is not byte-identical to the old pane markup.
- **D4 — item 4 click = select, not edit.** Per user: a cell click
  navigates/selects; editing is an explicit button. Keeps accidental edits out;
  clear scoped (Edit this) vs bulk (Edit all) split. Both write identities (node
  and rel) carry **labels**, matching the label-based DSL mutations — never
  derived keys alone (keys collide across distinct label spellings).

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

## Open questions — resolved

- OQ-1 governance → slice `specs` PRD-011 (done; the actionability/priority
  product intent this view exposes).
- OQ-2 item-4 interaction → resolved to select + Edit this / Edit all (D4).
- OQ-3 "actionable" → resolved via membership-in-view (`isActionabilityMember`)
  and `requiredMode`; no kind-list duplication (D2).

## Follow-ups

- Deep-linkable view mode in the URL hash (`?view=…`) — the larger, separate want
  behind D1.
- Larger frontend cleanups remain their own backlog items: IMP-085
  (decomposition), IMP-086 (vendor pinning), IMP-087 (theme), IMP-089 (ARIA).

## Doctrinal alignment

- **ADR-001 layering** — both new functions are pure (`focusTransition` in
  `model.ts`, `hoverDetailHtml` in `render.ts`); impurity (DOM, hash, membership
  lookup) stays in the `app.ts` shell. No new cycles.
- **Pure/imperative split** — `focusTransition`/`requiredMode`/`hoverDetailHtml`
  take inputs, touch no clock/rng/disk/DOM. Membership is computed in the shell
  and passed in as a bool.
- **Behaviour-preservation** — existing `web/map` vitest suites *and* the
  `routes.rs` mutation tests stay green unchanged. `hoverPane` output changes
  intentionally (now escaped) — its test is updated to assert escaping, not legacy
  bytes; this is a fix, not a silent behaviour change. The `relabel_edge` add is
  additive (new enum variant + apply arm + DSL fn) — existing mutation arms
  untouched.
- **No parallel implementation** — tooltip reuses the pane builder; the item-5
  rule is one pure transition, not scattered kind-checks or per-path switch logic.
