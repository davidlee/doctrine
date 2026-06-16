# SL-076 Design: Concept Maps in the Map Explorer + Web Authoring

## Hard Contracts

- **Concept maps are entities.** A `CM-NNN` entity appears in the catalog graph, sidebar list, search, and kind filter identically to any other entity kind. The only difference is the detail view (a CM diagram instead of an entity-relation graph).
- **DSL mutations reuse existing `concept_map` functions.** No parallel write path. `run_add`, `run_remove`, `run_rename_node` already handle line-level DSL editing with comment/blank-line preservation. The web routes call the same pure functions.
- **TOML preservation.** All mutations go through `get_dsl` → pure mutation → `set_dsl` (which uses `toml_edit::DocumentMut`). All other TOML fields and comments outside the `dsl` value survive byte-for-byte.
- **Structured data from Rust, dumb JS.** JS never parses the DSL. `parse_dsl`, `derive_node_key`, `levenshtein`, and check heuristics live only in Rust. JS receives `{nodes, edges, diagnostics}`.
- **Stale-render guard applies to CM diagrams.** The existing `graphRenderSeq` token guards both entity graph and CM diagram DOT renders.
- **Authoring is additive and reversible.** Add edge, remove edge, rename node. No raw DSL textarea. No web-based creation (CLI-only for now).
- **Autocomplete prevents term duplication.** Source/target inputs suggest existing node labels; relation input suggests existing relation labels. Duplicate edge submission is caught client-side (match against cache) and server-side (409).

## 1. Architecture & Module Layout

### Rust tiering (ADR-001)

```
Command tier (unchanged):
  src/commands/map.rs       → unchanged

Engine tier:
  src/integrity.rs           → add CONCEPT_MAP_KIND to KINDS (1 KindRef row)
  src/catalog/scan.rs        → add "CM" arm to outbound_for (empty, like REQ/KNOWLEDGE)
  src/concept_map.rs         → visibility promotions + 3 new pure functions (see §2.1)
  src/map_server/routes.rs   → GET /api/concept-map/:id, POST /api/concept-map/:id
  src/map_server/error.rs    → add CM-specific error variants
  src/map_server/state.rs    → unchanged

Leaf tier:
  (none touched)
```

Minimal changes to `concept_map.rs`: ~6 visibility promotions (`read_concept_map`,
`get_dsl`, `parse_dsl`, `check`, `set_dsl`, `ConceptMapDoc`, `CONCEPT_MAP_DIR` →
`pub(crate)`) plus 3 new pure mutation functions extracted from the CLI shell verbs
(see §2.1). The CLI verbs are left as-is — they remain the thin I/O wrappers they
already are. No new dependencies.

### JS changes (SL-073 module layout preserved)

```
web/map/
  api.js      → + fetchConceptMap(id), mutateConceptMap(id, action, params)
  model.js    → + normalizeConceptMap(raw), conceptMapCache, editingConceptMap state
  app.js      → + CM diagram rendering, authoring UI (add edge form, remove buttons,
                 inline rename, autocomplete), toggle between entity/CM views
  dot.js      → + cmGraphToDot(conceptMapData) — thin wrapper, shares DOT-escape helpers
  style.css   → + authoring form styles, CM-specific layout (.cm-edge-row, .add-edge-form, …)
  index.html  → + add-edge form container, edit-toggle button container
  router.js   → unchanged (#/focus/CM-001 resolves via existing hash model)
```

`dot.js` needs one new function, not structural changes. `cmGraphToDot` is a
lightweight wrapper — concept map nodes and edges are isomorphic to entity graph
nodes and edges for DOT generation. The only difference: CM nodes have uniform
styling (no kind/status attributes).

## 2. API Routes

### `GET /api/concept-map/:id`

Reads the concept map TOML, parses the DSL, returns structured data.

```
200:
{
  "id": "CM-001",
  "title": "System Architecture",
  "status": "draft",
  "description": "High-level architecture concept map",
  "nodes": [
    { "key": "user-story", "label": "User Story" },
    { "key": "user-need", "label": "User Need" }
  ],
  "edges": [
    {
      "from_key": "user-story",
      "from_label": "User Story",
      "rel": "expresses",
      "to_key": "user-need",
      "to_label": "User Need",
      "line": 5
    }
  ],
  "diagnostics": [
    { "variant": "EntityRefLike", "label": "SL-001", "line": 7 }
  ]
}

404: { "error": "not_found", "message": "concept map CM-999 not found" }
400: { "error": "bad_id", "message": "not a concept-map reference: `foo`" }
```

- `key` = `derive_node_key(label)` — stable, URL-safe, used as DOT node id and identity for edge matching.
- `nodes` deduplicated by key (first-wins), in parse order.
- `edges` carry `line` for diagnostics display, in parse order.
- `diagnostics` carry check-pass findings: `CanonicalNodeCollision`, `SelfEdge`, `SimilarNodeLabel`, `RelationDrift`, `EntityRefLike`. Parse-time `MalformedLine` and `EmptyLabel` are excluded (they prevent edge creation — the check step covers them).
- `description` from the TOML `description` field — not currently editable via web, but exposed so the UI can display it.

### `POST /api/concept-map/:id`

Body: `{ "action": "...", ...params }`. Three actions.

#### `add_edge`

```json
// Request
{ "action": "add_edge", "source": "User Story", "rel": "expresses", "target": "User Need" }

// 200 — success; returns updated nodes/edges so frontend can re-render without second fetch
{
  "ok": true,
  "nodes": [ ... ],
  "edges": [ ... ]
}

// 409 — exact (source, rel, target) triple already exists
{ "error": "duplicate_edge", "message": "edge already exists at line 5", "line": 5 }

// 400 — empty field
{ "error": "empty_field", "message": "source must be non-empty" }
```

#### `remove_edge`

```json
{ "action": "remove_edge", "source": "User Story", "rel": "expresses", "target": "User Need" }

// 200 — returns updated nodes/edges
// occurrences is the count of edge lines that were renamed.
// If 0, the rename was a no-op (case-only change with no effect on keys).
{ "ok": true, "occurrences": 4, "nodes": [ ... ], "edges": [ ... ] }

// 404 — edge not found
{ "error": "edge_not_found", "message": "edge not found: User Story > expresses > User Need" }
```

#### `rename_node`

```json
{ "action": "rename_node", "old": "User Story", "new": "User Narrative" }

// 200 — returns updated nodes/edges + occurrence count
{ "ok": true, "occurrences": 4, "nodes": [ ... ], "edges": [ ... ] }

// 409 — rename would produce a key collision with an existing node
// Case-only renames where old is the sole key-holder pass (occurrences may be 0).
{ "error": "node_collision", "message": "rename would collide with existing node 'User Narrative' at line 3", "existing_label": "User Narrative", "line": 3 }
```

The collision check (409) fires when `derive_node_key(new) == derive_node_key(existing_label)`
and `existing_label != old`. This prevents silently merging two distinct nodes. This check is
**new** — the existing CLI `run_rename_node` does not perform it. The CLI can gain it in a
follow-up; the web route requires it for safe inline editing.

Case-only renames ("User Story" → "USER STORY") where `old` is the sole key-holder
do NOT collide, but produce `occurrences: 0` — the frontend should treat this as a no-op
and not re-render.

### entity_markdown path verification

`GET /api/entity/CM-001/markdown` — the existing `markdown::read_entity_markdown`
constructs the .md path from the `EntityKey` and the kind's directory. For CM
entities, this must resolve to `.doctrine/concept-map/001/concept-map-001.md`.
The function uses `KindRef.stem` (not prefix) for file naming (the stem is
`"concept-map"`, producing `concept-map-001.md`). **Verify during implementation**
that the path assembly works correctly for the CM kind — if the function derives
the filename from prefix (`"CM"` → `cm-001.md`), fix it. This is a latent bug
in the existing code, not a CM-specific change.

### Route handler structure (thin wrappers)

```rust
// src/map_server/routes.rs — new handlers

async fn get_concept_map(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> Result<impl IntoResponse, MapServerError> {
    let id = concept_map::parse_ref(&id_str).map_err(|_| MapServerError::BadConceptMapId(id_str))?;
    let cm_root = state.root.join(CONCEPT_MAP_DIR);
    let (_doc, toml_text, _body) = concept_map::read_concept_map(&cm_root, id)
        .map_err(|_| MapServerError::ConceptMapNotFound(id))?;

    let dsl = concept_map::get_dsl(&toml_text)?;  // needs pub(crate) visibility
    let parsed = concept_map::parse_dsl(&dsl);
    let diagnostics = concept_map::check(&parsed);

    // Assemble response ...
}

async fn mutate_concept_map(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
    Json(body): Json<ConceptMapMutation>,
) -> Result<impl IntoResponse, MapServerError> {
    let id = concept_map::parse_ref(&id_str)?;
    let cm_root = state.root.join(CONCEPT_MAP_DIR);
    let (_doc, toml_text, _body) = concept_map::read_concept_map(&cm_root, id)?;

    let new_toml = match body.action {
        MutationAction::AddEdge { source, rel, target } => {
            // Reuse run_add's core logic (extracted to a pure function)
            concept_map::add_edge_to_dsl(&toml_text, &source, &rel, &target)?
        }
        MutationAction::RemoveEdge { source, rel, target } => {
            concept_map::remove_edge_from_dsl(&toml_text, &source, &rel, &target)?
        }
        MutationAction::RenameNode { old, new } => {
            concept_map::rename_node_in_dsl(&toml_text, &old, &new)?
        }
    };

    // Write back
    let name = format!("{id:03}");
    let stem = format!("concept-map-{name}");
    let toml_path = cm_root.join(&name).join(format!("{stem}.toml"));
    std::fs::write(&toml_path, &new_toml)?;

    // Re-parse for response
    let new_dsl = concept_map::get_dsl(&new_toml)?;
    let parsed = concept_map::parse_dsl(&new_dsl);
    // ... assemble response with updated nodes/edges
}
```

### §2.1: concept_map.rs changes

#### Visibility promotions (existing functions → `pub(crate)`)

| Symbol | Current visibility | Needed by |
|---|---|---|
| `read_concept_map` | private | GET/POST routes (read TOML + body) |
| `get_dsl` | private | GET/POST routes (extract DSL from TOML) |
| `parse_dsl` | private | GET route (structured response) |
| `check` | private | GET route (diagnostics) |
| `set_dsl` | private | POST route (write-back mutation) |
| `CONCEPT_MAP_DIR` | private | route handlers (join with root) |
| `ConceptMapDoc` | private | GET route (extract title/status/description) |

#### New pure functions

Three pure functions extracted from the CLI shell verbs. Each takes the full
TOML text, mutates the DSL block, and returns the updated TOML text. No I/O,
no stdout. The existing CLI verbs (`run_add`, `run_remove`, `run_rename_node`)
remain as-is — they are thin I/O wrappers and the duplication is trivial.

```rust
/// Append an edge line. Returns updated TOML.
/// Returns `Err` for duplicate edge (with the existing line number).
pub(crate) fn add_edge_to_dsl(
    toml_text: &str,
    source: &str,
    rel: &str,
    target: &str,
) -> anyhow::Result<String> { ... }

/// Remove a matching edge line. Returns updated TOML.
/// Returns `Err` if the edge is not found.
pub(crate) fn remove_edge_from_dsl(
    toml_text: &str,
    source: &str,
    rel: &str,
    target: &str,
) -> anyhow::Result<String> { ... }

/// Rename a node label across all DSL edges. Returns updated TOML.
/// Returns `Err` on key collision (see below).
///
/// Collision check (NEW — not present in the CLI `run_rename_node`):
/// `derive_node_key(new) == derive_node_key(existing_label)` AND
/// `existing_label != old`. This prevents silently merging two distinct
/// nodes through a rename. Case-only renames ("User Story" → "USER STORY")
/// where `old` is the only node with that key pass the check but produce
/// zero occurrences — the caller should treat this as a no-op.
pub(crate) fn rename_node_in_dsl(
    toml_text: &str,
    old: &str,
    new_label: &str,
) -> anyhow::Result<String> { ... }
```

Each reuses the existing `get_dsl` / `set_dsl` pair and the line-level logic
already in `run_add`/`run_remove`/`run_rename_node`.

### MapServerError additions

```rust
pub(crate) enum MapServerError {
    // ... existing variants ...
    BadConceptMapId(String),
    ConceptMapNotFound(u32),
    ConceptMapParseError(String),
    DuplicateEdge { line: usize },
    EdgeNotFound { source: String, rel: String, target: String },
    NodeCollision { existing_label: String, line: usize },
}
```

Each maps to an HTTP status code via the existing `IntoResponse` impl:
- `BadConceptMapId` → 400
- `ConceptMapNotFound` → 404
- `DuplicateEdge` → 409
- `EdgeNotFound` → 404
- `NodeCollision` → 409
- `ConceptMapParseError` → 500

## 3. Frontend State Model

### New state fields (model.js)

```js
conceptMapCache: new Map(),   // id → { nodes, edges, diagnostics }
editingConceptMap: false,     // authoring mode toggle
editingNode: null,            // { key, label } — node currently being renamed inline
```

`conceptMapCache` is cleared on refresh alongside `markdownCache`. The refresh
handler (`wireRefresh` in app.js) must clear both caches. After a CM mutation,
the cache is updated in-place from the POST response; a manual refresh re-fetches
from the server. The catalog graph (which holds the CM entity node's `status`) is
NOT updated by CM mutations — status changes require a refresh, consistent with
all other entity kinds.
`editingConceptMap` is toggled by the Edit/Done button. `editingNode` is set when
an inline rename input is active, cleared on completion.

### CM detection

```js
// A focused entity is a concept map if its kindPrefix is "CM"
function isConceptMap(focusId) {
  var node = state.graph.nodes.get(focusId);
  return node && node.kindPrefix === 'CM';
}
```

Since CM entities appear in the normalized catalog graph after KINDS registration,
this check works with zero new identity plumbing.

### Render dispatch

```js
function render() {
  if (isConceptMap(state.focusId)) {
    renderConceptMap();
  } else {
    renderEntityGraph();  // existing path
  }
}
```

`renderConceptMap()`:
1. If `conceptMapCache` doesn't have the current focus, fetch via `api.fetchConceptMap(id)`.
2. Generate DOT via `dot.cmGraphToDot(cachedData)`.
3. Send to `/api/dot/svg` (existing endpoint, no changes).
4. Wire SVG with `wireCmSvgHandlers`.
5. Render CM edge table.
6. If `editingConceptMap`, show add-edge form and wire remove/rename controls.
7. Render markdown pane (unchanged — `GET /api/entity/CM-001/markdown` already works if the entity is in the catalog graph).

## 4. DOT Generation & SVG Wiring

### cmGraphToDot (dot.js)

```js
dot.cmGraphToDot = function(conceptMapData) {
  var lines = [];
  lines.push('digraph concept_map {');
  lines.push('  rankdir=LR;');
  lines.push('  bgcolor="transparent";');
  lines.push('  nodesep=0.45;');
  lines.push('  ranksep=0.8;');
  lines.push('  node [shape=ellipse, style=filled, fillcolor="' + CM_FILL + '", fontcolor="' + CM_FONT + '"];');
  lines.push('');

  conceptMapData.nodes.forEach(function(node) {
    lines.push('  "' + dot.dotQuote(node.key) + '" [label="' + dot.dotQuote(node.label) + '"];');
  });

  lines.push('');

  conceptMapData.edges.forEach(function(edge) {
    lines.push('  "' + dot.dotQuote(edge.from_key) + '" -> "' + dot.dotQuote(edge.to_key) +
      '" [label="' + dot.dotQuote(edge.rel) + '"];');
  });

  lines.push('}');
  return lines.join('\n');
};
```

- Node key = `derive_node_key(label)` — the Rust-returned key, used as DOT node id and SVG `<title>`.
- Node label = original user-authored label — displayed in the diagram.
- CM colour: `fillcolor="#16A085"` (green-teal), `fontcolor="#ffffff"`. Consistent with the CM kind pill.
- Shape: `ellipse` for all nodes (no kind/status differentiation within a concept map).
- No `URL` or `tooltip` attributes (follow-up: entity-ref nodes would get `URL="#/focus/SL-001"`).

### SVG wiring

Reuses the entity graph handler pattern. Key differences:
- **Click**: in view mode, no-op (CM nodes don't navigate). In edit mode, triggers `startRenameNode(key)`.
- **Hover**: shows the node label in the hover pane (not entity metadata).
- **Hit-area rect**: identical pattern (bbox-based transparent `<rect>`).
- **Stale-render guard**: existing `graphRenderSeq` token covers both entity and CM renders.

### Hover pane for CM nodes

```
┌────────────────────────────────────────────┐
│ User Story                                 │
│ (concept map node)                         │
└────────────────────────────────────────────┘
```

Simple label display. The `(concept map node)` subtitle distinguishes it from entity hover panes.

## 5. Authoring UI

### Add edge form

```
┌─ Add edge ────────────────────────────────────────────┐
│ [Source▾]  [relation▾]  [Target▾]        [Add edge]    │
└───────────────────────────────────────────────────────┘
```

- `▾` indicates a `<datalist>`-backed text input. The datalist is rebuilt from `conceptMapCache` after every mutation.
- Source/Target datalist: existing node labels (deduplicated).
- Relation datalist: existing relation labels (deduplicated).
- Client-side validation: non-empty after trim. Inline error message on blur if empty.
- On submit: `POST /api/concept-map/:id` with `add_edge`.
  - 200: update `conceptMapCache` with returned nodes/edges, re-render diagram + edge table, clear form.
  - 409: show inline warning "This edge already exists at line {n}" below the form.

### Edge table (edit mode)

```
┌─ CM Edges ────────────────────────────────────────────┐
│ Source          │ Rel         │ Target        │        │
│ User Story      │ expresses   │ User Need     │  [✕]   │
│ Capability      │ aggregates  │ Requirements  │  [✕]   │
└───────────────────────────────────────────────────────┘
```

- Each row has a `[✕]` remove button. On click: confirm (optional — could be instant), `POST` with `remove_edge`.
- 200: update cache, re-render.
- 404: show inline "Edge no longer exists — it may have been removed elsewhere"; refresh edge list from cache.

### Node rename

Clicking a node label in the edge table or the SVG opens an inline `<input>`:

```
┌─ CM Edges ────────────────────────────────────────────┐
│ [User Narrative___] │ expresses   │ User Need     │    │
└───────────────────────────────────────────────────────┘
```

- Pre-filled with current label. On Enter: `POST` with `rename_node`. On Escape: cancel.
- 200: update cache, re-render.
- 409: show inline warning "Rename would collide with existing node '{label}'"; input stays open with current value.

### Autocomplete data flow

- After every successful mutation, the response includes updated `nodes`/`edges`.
- The cache is updated in-place, then `<datalist>` options are rebuilt from the new cache.
- Zero extra API calls for autocomplete — the cache is the source of truth.

## 6. UI Layout & States

### Layout when CM is focused (view mode)

```text
┌──────────────────────┬──────────────────────────────────────────┐
│ Sidebar              │ Main                                     │
│                      │                                          │
│ [Search input      ] │ CM-001: System Architecture              │
│                      │ concept map · draft                      │
│ ☑ Concept Maps (CM) │                                          │
│ ☑ Slices  ☐ Gov    │ ┌──────────────────────────────────────┐ │
│ ☑ Specs   ☐ Reqs   │ │                                      │ │
│                      │ │       Concept Map Diagram           │ │
│ Entity list          │ │       (Graphviz SVG)                │ │
│ (CM-001 highlighted) │ │                                      │ │
│ CM-001 · System Arch │ └──────────────────────────────────────┘ │
│ SL-071 · Scanner     │                                          │
│ SL-072 · Map Server  │ ┌ Hover detail pane ──────────────────┐ │
│ …                    │ │ User Story                          │ │
│                      │ │ (concept map node)                  │ │
│ [Refresh]            │ └──────────────────────────────────────┘ │
│                      │                                          │
│                      │ [Edit]                                   │
│                      │                                          │
│                      │ ┌ CM Edges ───────────────────────────┐ │
│                      │ │ Source       │ Rel        │ Target   │ │
│                      │ │ User Story   │ expresses  │ User Need│ │
│                      │ │ Capability   │ aggregates │ Reqs     │ │
│                      │ └──────────────────────────────────────┘ │
│                      │                                          │
│                      │ ┌ Markdown ───────────────────────────┐ │
│                      │ │ # Concept Map: System Architecture   │ │
│                      │ └──────────────────────────────────────┘ │
└──────────────────────┴──────────────────────────────────────────┘
```

### Layout when CM is focused (edit mode)

Same as above, plus:
- [Done] button replaces [Edit]
- Add edge form appears above the CM Edges table
- Each edge row has a [✕] button
- Clicking node labels triggers inline rename

### State table

| Element | Entity mode (existing) | CM view mode | CM edit mode |
|---|---|---|---|
| Graph pane | Entity graph SVG | CM diagram SVG | CM diagram SVG (same) |
| Header | Entity title + kind + status | CM title + "concept map · draft" | Same + [Done] button |
| Hover pane | Entity details (id, kind, status) | Node label + "(concept map node)" | Same |
| Depth selector | Visible (0–3) | Hidden | Hidden |
| Edge table | Entity relationship table | CM edges (read-only) | CM edges + [✕] per row |
| Add edge form | Hidden | Hidden | Visible |
| Markdown pane | Entity .md body | CM .md body | CM .md body |

### Error states

| State | Trigger | Behaviour |
|---|---|---|
| CM not found | GET 404 | "Concept map {id} not found" in graph pane; sidebar still shows CM node |
| CM parse failure | Rust TOML parse error | "Failed to load concept map: {message}" error |
| Add edge 409 | Duplicate edge | Inline warning "This edge already exists at line {n}" |
| Add edge 400 | Empty field | Inline validation on offending field |
| Remove edge 404 | Already removed | Inline message; refresh edge list from cache |
| Rename node 409 | Key collision | Inline warning "Rename would collide with existing node '{label}'" |
| Stale DOT render | graphRenderSeq mismatch | Silently discard (same guard as entity graph) |

## 7. Kind Registration & Catalog Integration

### integrity::KINDS addition

```rust
KindRef {
    kind: &concept_map::CONCEPT_MAP_KIND,
    stem: "concept-map",
    state_dir: None,  // no runtime state tree for concept maps
},
```

Inserted after the knowledge-record kinds (CON) and before the REV row, or at the end. Order determines entity-list sort order in `scan_entities` — placing it near the end gives CM a lower priority, matching the `kindOrder.CM = 20` JS constant.

### outbound_for arm

```rust
"CM" => Ok(Vec::new()),  // concept maps currently author no tier-1 [[relation]] edges
```

Empty, like REQ and knowledge records. Follow-up: when concept maps gain cross-kind structural relations
(link to ADRs, specs, slices), this arm populates.

### status_and_title_for

The default `_` arm uses `meta::read_meta(tree_root, kref.stem, id)` with stem `"concept-map"`.
The CM TOML has `id`, `slug`, `title`, `status` — exactly the `Meta` fields. The extra `description`
and `dsl` fields are ignored by serde (unknown keys). No special case needed.

### Kind colour

CM pill colour: `#16A085` (green-teal), `fontcolor="#ffffff"` for dark background, `#ffffff` text on pills. Added to the CSS custom property palette and the JS `kindOrder` map.

### Kind filter checkbox

```
☑ Concept Maps (CM)
```

Added to the filter grid in `index.html`. Toggle behaviour matches all other kinds.

## 8. Verification & Test Cases

### Rust tests

1. **KINDS registration**: `integrity::KINDS` contains a `"CM"` prefix row. `scan_entities` picks up seeded CM entities. No panic in `outbound_for` with `"CM"` prefix.
2. **GET /api/concept-map/:id**: seeded CM fixture → 200 with correct nodes/edges/diagnostics. Non-existent id → 404. Bad id format → 400.
3. **POST add_edge**: valid edge → 200 with updated nodes/edges. Duplicate → 409 with line number. Empty source → 400.
4. **POST remove_edge**: existing edge → 200. Non-existent edge → 404.
5. **POST rename_node**: valid rename → 200 with occurrences + updated lists. Collision → 409.
6. **TOML preservation**: add edge, then verify all other fields (`slug`, `title`, `status`, `description`, `created`) survive unchanged.
7. **DSL comment/blank-line preservation**: add edge, verify comments and blanks survive. Remove edge, verify same.

### JS tests

8. **CM detection**: `isConceptMap("CM-001")` returns true when a CM entity exists in the graph. Returns false for non-CM entities like `"SL-001"`.
9. **normalizeConceptMap**: parses the GET response shape into the internal cache format.
10. **cmGraphToDot**: produces valid DOT with correct node/edge counts, ellipse shape, CM colour.
11. **Autocomplete**: datalist is rebuilt from cache after add/remove/rename mutations.

### Integration smoke test

12. `doctrine concept-map new "Test Map"` → `doctrine concept-map add CM-001 "A" "relates to" "B"` → `doctrine map serve --open --focus CM-001`. Verify: CM appears in sidebar, diagram renders with 2 nodes + 1 edge, hover pane shows "A" on mouseenter.
13. Edit mode: toggle Edit, add edge "B > depends on > C", verify diagram updates, `doctrine concept-map show CM-001` shows 2 edges.
14. Remove edge: click ✕ on "A > relates to > B", verify diagram updates, CLI `show` confirms removal.
15. Rename node: click "A" in edge table, rename to "Alpha", verify diagram updates, all edges reference "Alpha".

### Gate

- `just check` — root package tests pass
- `just gate` — workspace clean (clippy zero warnings)

### TOCTOU posture

No lock on the write path. The POST handler does: read TOML → mutate in memory →
write file. Two concurrent browser tabs mutating the same CM could interleave
reads before either writes (last-write-wins). For a single-user loopback tool
this is acceptable. The 409 on duplicate edge and 404 on edge-not-found provide
enough feedback that the second writer can retry after a cache refresh.

## 9. Open Questions & Deferred

### Deferred to follow-up slices

- **Entity-ref support**: CM node labels that match canonical entity refs (`SL-001`) get special styling and click-to-navigate. Requires extending `parse_dsl`/`check` to classify nodes as entity-refs, and DOT generation to add URL attributes.
- **Web-based CM creation**: "New Concept Map" button in the web UI. Requires a `POST /api/concept-maps` route and the full scaffold path (which currently lives in the CLI `run_new`).
- **Visual graph editing**: drag nodes, draw edges. Requires a canvas/SVG editing library and a fundamentally different interaction model.
- **Cross-kind structural relations**: `doctrine link CM-001 governed_by ADR-001`. Requires the `outbound_for` arm to read `[[relation]]` rows and the concept map TOML to support a `[relationships]` table.
- **CM description editing**: inline edit of the `description` field in the web UI.

### Open questions resolved in design

- **Q: How does the CM diagram relate to the entity graph?** A: Replace the graph pane when a CM is focused (option A). A "back" mechanism is implicit — clicking any non-CM entity in the sidebar switches back.
- **Q: What shape is the authoring interface?** A: Form-based line operations (option B) — add edge form, remove buttons, inline rename. No raw DSL textarea.
- **Q: How do CMs appear in the sidebar?** A: Uniform list (option A) — CM entities appear in the same entity list with a `CM` kind pill, same hash route (`#/focus/CM-001`).
- **Q: Where does parsing live?** A: Rust. JS receives structured `{nodes, edges, diagnostics}`. No JS port of `parse_dsl`, `levenshtein`, or `derive_node_key`.
- **Q: Mutation API style?** A: Single `POST` with `action` discriminator — three actions (`add_edge`, `remove_edge`, `rename_node`), each maps 1:1 to an existing Rust function.
