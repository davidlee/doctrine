# SL-076 Design: Concept Maps in the Map Explorer + Web Authoring

## Hard Contracts

- **Concept maps are entities.** A `CM-NNN` entity appears in the catalog graph, sidebar list, search, and kind filter identically to any other entity kind. The only difference is the detail view (a CM diagram instead of an entity-relation graph).
- **DSL mutations reuse existing `concept_map` functions.** No parallel write path. `run_add`, `run_remove`, `run_rename_node` already handle line-level DSL editing with comment/blank-line preservation. The web routes call the same pure functions.
- **TOML preservation.** All mutations go through `get_dsl` → pure mutation → `set_dsl` (which uses `toml_edit::DocumentMut`). All other TOML fields and comments outside the `dsl` value survive byte-for-byte.
- **Structured data from Rust, dumb JS.** JS never parses the DSL. `parse_dsl`, `derive_node_key`, `levenshtein`, and check heuristics live only in Rust. JS receives `{nodes, edges, diagnostics}`.
- **Stale-render guard applies to CM diagrams.** The existing `graphRenderSeq` token guards both entity graph and CM diagram DOT renders.
- **Authoring is additive and reversible.** Add edge, remove edge, rename node. No raw DSL textarea. No web-based creation (CLI-only for now).
- **Autocomplete prevents term duplication.** Source/target inputs suggest existing node labels; relation input suggests existing relation labels. Duplicate edge submission is caught client-side (match against cache) and server-side (409).
- **Routes do not own DSL semantics.** All DSL parsing, key derivation, mutation, and validation logic lives in `src/concept_map.rs`. Routes are thin I/O wrappers that call pure engine functions and map typed errors to HTTP responses. If `concept_map.rs` later swells, it splits along natural seams (dsl / mutate / check / io) — routes never gain direct DSL knowledge.
- **Input normalization is canonical.** `source`, `rel`, `target`, `old`, and `new` are trimmed before mutation. Empty-after-trim is 400. Internal whitespace is preserved. Case is preserved. Duplicate/collision matching uses derived keys for node identity and exact normalized relation text.

## 1. Architecture & Module Layout

### Rust tiering (ADR-001)

```
Command tier (unchanged):
  src/commands/map.rs       → unchanged

Engine tier:
  src/integrity.rs           → add CONCEPT_MAP_KIND to KINDS (1 KindRef row)
  src/catalog/scan.rs        → add "CM" arm to outbound_for (empty, like REQ/KNOWLEDGE)
  src/concept_map.rs         → visibility promotions + 3 new pure functions + typed error enum (see §2.1)
  src/map_server/routes.rs   → GET /api/concept-map/:id, POST /api/concept-map/:id
  src/map_server/error.rs    → add CM-specific error variants
  src/map_server/state.rs    → unchanged

Leaf tier:
  (none touched)
```

Minimal changes to `concept_map.rs`: ~6 visibility promotions (`read_concept_map`,
`get_dsl`, `parse_dsl`, `check`, `set_dsl`, `ConceptMapDoc`, `CONCEPT_MAP_DIR` →
`pub(crate)`) plus 3 new pure mutation functions extracted from the CLI shell verbs,
plus one `ConceptMapMutationError` enum (see §2.1). The CLI verbs are left as-is — they
remain the thin I/O wrappers they already are. No new dependencies.

### JS changes (SL-073 module layout preserved)

```
web/map/
  api.js      → + fetchConceptMap(id), mutateConceptMap(id, action, params)
  model.js    → + normalizeConceptMap(raw), conceptMapCache, editingConceptMap state
  app.js      → + CM diagram rendering, authoring UI (add edge form, remove buttons,
                 inline rename, autocomplete), diagnostics panel, toggle between entity/CM views
  dot.js      → + cmGraphToDot(conceptMapData) — thin wrapper, shares DOT-escape helpers
  style.css   → + authoring form styles, CM-specific layout (.cm-edge-row, .add-edge-form, …)
  index.html  → + add-edge form container, edit-toggle button container, diagnostics panel container
  router.js   → unchanged (#/focus/CM-001 resolves via existing hash model)
```

`dot.js` needs one new function, not structural changes. `cmGraphToDot` is a
lightweight wrapper — concept map nodes and edges are isomorphic to entity graph
nodes and edges for DOT generation. The only difference: CM nodes have uniform
styling (no kind/status attributes).

## 2. API Routes

### Input normalization (all POST actions)

All string fields (`source`, `rel`, `target`, `old`, `new`) are trimmed before
any mutation logic. Empty-after-trim returns 400 `empty_field`. Internal
whitespace is preserved. Case is preserved.

Duplicate edge detection matches on the normalized (trimmed) triple:
`(source, rel, target)`.

Node collision detection operates on **derived keys**, not raw labels. The
key derivation (`derive_node_key`) is the single source of identity.

### `GET /api/concept-map/:id`

Reads the concept map TOML, parses the DSL, returns structured data.

```
200:
{
  "id": "CM-001",
  "title": "System Architecture",
  "status": "draft",
  "description": "High-level architecture concept map",
  "dsl_hash": "sha256:abc123def456",
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
- `dsl_hash` is the hex-encoded SHA-256 of the raw DSL text (before TOML wrapping). Used for optional stale-write detection (see `base_hash` in POST).

### `POST /api/concept-map/:id`

Body: `{ "action": "...", ...params, "base_hash": "optional" }`. Three actions.

All three actions accept an optional `base_hash` field. If present and the current
DSL's SHA-256 does not match, the server returns 409 `stale_concept_map` and does
not mutate. If absent, last-write-wins semantics apply (no revision guard).

#### `add_edge`

```json
// Request
{ "action": "add_edge", "source": "User Story", "rel": "expresses", "target": "User Need" }

// 200 — success; returns updated nodes/edges so frontend can re-render without second fetch
{
  "ok": true,
  "nodes": [ ... ],
  "edges": [ ... ],
  "dsl_hash": "sha256:newhash"
}

// 409 — exact (source, rel, target) triple already exists
{ "error": "duplicate_edge", "message": "edge already exists at line 5", "line": 5 }

// 400 — empty field after trim
{ "error": "empty_field", "message": "source must be non-empty" }

// 409 — stale write guard (if base_hash provided)
{ "error": "stale_concept_map", "message": "concept map was modified since last read; refresh and retry" }
```

#### `remove_edge`

```json
{ "action": "remove_edge", "source": "User Story", "rel": "expresses", "target": "User Need" }

// 200 — returns updated nodes/edges
{ "ok": true, "nodes": [ ... ], "edges": [ ... ], "dsl_hash": "sha256:newhash" }

// 404 — edge not found
{ "error": "edge_not_found", "message": "edge not found: User Story > expresses > User Need" }

// 400 — empty field after trim
{ "error": "empty_field", "message": "source must be non-empty" }

// 409 — stale write guard
{ "error": "stale_concept_map", "message": "concept map was modified since last read; refresh and retry" }
```

If the DSL contains multiple identical edge lines (possible in legacy hand-edited
files; `add_edge` prevents new duplicates), `remove_edge` removes **one**
matching line (first match in line order). Removing an edge that exists multiple
times requires multiple POST calls — the frontend should refresh after each.

#### `rename_node`

```json
{ "action": "rename_node", "old": "User Story", "new": "User Narrative" }

// 200 — returns updated nodes/edges + occurrence count
{ "ok": true, "occurrences": 3, "nodes": [ ... ], "edges": [ ... ], "dsl_hash": "sha256:newhash" }

// 409 — rename would produce a key collision with an existing node
{ "error": "node_collision", "message": "rename would collide with existing node 'User Narrative' at line 3", "existing_label": "User Narrative", "line": 3 }

// 400 — empty field after trim
{ "error": "empty_field", "message": "new label must be non-empty" }

// 409 — stale write guard
{ "error": "stale_concept_map", "message": "concept map was modified since last read; refresh and retry" }
```

Collision check (409) uses **derived keys**, not raw labels:

```text
old_key = derive_node_key(old)
new_key = derive_node_key(new)

If new_key != old_key AND any existing node has key == new_key:
    reject 409 (silently merging two distinct nodes is not allowed).

If new_key == old_key:
    allow. This is a case-only or canonicalization rename.
    The DSL text is still edited — source/target labels on every edge line are
    replaced. occurrences reflects the number of edge lines actually changed.
    If old == new after trim (text-identical), occurrences is 0 — frontend
    may treat as no-op but the server still returns 200.
```

This collision check is **new** — the existing CLI `run_rename_node` does not
perform it. The CLI can gain it in a follow-up; the web route requires it for
safe inline editing.

### entity_markdown path verification

`GET /api/entity/CM-001/markdown` — the existing `markdown::read_entity_markdown`
constructs the .md path from the `EntityKey` and the kind's directory. For CM
entities, this must resolve to `.doctrine/concept-map/001/concept-map-001.md`.
The function uses `KindRef.stem` (not prefix) for file naming (the stem is
`"concept-map"`, producing `concept-map-001.md`).

An explicit Rust route/unit test verifies this: seed a CM fixture, call
`read_entity_markdown`, and assert the returned path contains
`concept-map-001.md` (not `cm-001.md`). This is a latent bug surface in existing
code — the test prevents regression.

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

    let dsl = concept_map::get_dsl(&toml_text)?;
    let dsl_hash = hex::encode(sha2::Sha256::digest(dsl.as_bytes()));
    let parsed = concept_map::parse_dsl(&dsl);
    let diagnostics = concept_map::check(&parsed);

    // Assemble response with dsl_hash ...
}

async fn mutate_concept_map(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
    Json(body): Json<ConceptMapMutation>,
) -> Result<impl IntoResponse, MapServerError> {
    let id = concept_map::parse_ref(&id_str)?;
    let cm_root = state.root.join(CONCEPT_MAP_DIR);
    let (_doc, toml_text, _body) = concept_map::read_concept_map(&cm_root, id)?;

    // Stale-write guard (optional)
    if let Some(base_hash) = &body.base_hash {
        let current_hash = hex::encode(sha2::Sha256::digest(
            concept_map::get_dsl(&toml_text)?.as_bytes()
        ));
        if current_hash != *base_hash {
            return Err(MapServerError::StaleConceptMap);
        }
    }

    let new_toml = match body.action {
        MutationAction::AddEdge { source, rel, target } => {
            concept_map::add_edge_to_dsl(&toml_text, &source, &rel, &target)?
        }
        MutationAction::RemoveEdge { source, rel, target } => {
            concept_map::remove_edge_from_dsl(&toml_text, &source, &rel, &target)?
        }
        MutationAction::RenameNode { old, new } => {
            concept_map::rename_node_in_dsl(&toml_text, &old, &new)?
        }
    };

    // Write back. Synchronous file I/O is explicitly blessed for the map server
    // (single-user loopback tool; TOML files are small). If the server migrates
    // to an async file I/O layer, use tokio::task::spawn_blocking here.
    let name = format!("{id:03}");
    let stem = format!("concept-map-{name}");
    let toml_path = cm_root.join(&name).join(format!("{stem}.toml"));
    std::fs::write(&toml_path, &new_toml)?;

    // Re-parse for response
    let new_dsl = concept_map::get_dsl(&new_toml)?;
    let new_hash = hex::encode(sha2::Sha256::digest(new_dsl.as_bytes()));
    let parsed = concept_map::parse_dsl(&new_dsl);
    // ... assemble response with updated nodes/edges + dsl_hash
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

#### Typed mutation error

A small typed error enum replaces `anyhow::Result` on the three pure mutation
functions. It maps cleanly to `MapServerError` via `From`, avoiding string-matching
and ensuring every error path produces the correct HTTP status.

```rust
/// Errors from pure concept-map mutation functions.
/// Each variant maps to a specific HTTP response via `From<ConceptMapMutationError> for MapServerError`.
pub(crate) enum ConceptMapMutationError {
    /// A required field (source, rel, target, old, new) is empty after trimming.
    EmptyField { field: &'static str },
    /// An add_edge would create a duplicate of an existing edge.
    DuplicateEdge { line: usize },
    /// A remove_edge target does not match any edge line.
    EdgeNotFound { source: String, rel: String, target: String },
    /// A rename_node would produce a key collision with an existing node.
    NodeCollision { existing_label: String, line: usize },
    /// The TOML document has no `dsl` key.
    MissingDsl,
    /// The TOML document is structurally invalid (can't parse with toml_edit).
    InvalidToml(String),
}
```

#### New pure functions

Three pure functions extracted from the CLI shell verbs. Each takes the full
TOML text, mutates the DSL block, and returns the updated TOML text. No I/O,
no stdout. The existing CLI verbs (`run_add`, `run_remove`, `run_rename_node`)
remain as-is — they are thin I/O wrappers and the duplication is trivial.

All three apply input normalization (trim) before mutation and return
`Result<String, ConceptMapMutationError>`. The collision logic in
`rename_node_in_dsl` operates on derived keys (see §2 API Routes for the
precise algorithm).

```rust
/// Append an edge line. Returns updated TOML.
/// Input fields are trimmed. Empty-after-trim → EmptyField.
/// Returns DuplicateEdge if an identical (source, rel, target) triple already exists.
pub(crate) fn add_edge_to_dsl(
    toml_text: &str,
    source: &str,
    rel: &str,
    target: &str,
) -> Result<String, ConceptMapMutationError> { ... }

/// Remove one matching edge line (first match if duplicates exist).
/// Returns updated TOML.
/// Input fields are trimmed. Empty-after-trim → EmptyField.
/// Returns EdgeNotFound if no matching edge is found.
pub(crate) fn remove_edge_from_dsl(
    toml_text: &str,
    source: &str,
    rel: &str,
    target: &str,
) -> Result<String, ConceptMapMutationError> { ... }

/// Rename a node label across all DSL edges. Returns updated TOML.
/// Input fields are trimmed. Empty-after-trim → EmptyField.
/// Returns NodeCollision if `derive_node_key(new) != derive_node_key(old)`
/// and any existing node already holds `derive_node_key(new)`.
///
/// Collision check is new — not present in the CLI `run_rename_node`.
/// Case-only renames (same key, different label text) are allowed and
/// edit DSL text; occurrences reflects the count of edge lines changed.
/// If old == new after trim, occurrences is 0 (text-identical no-op, 200).
pub(crate) fn rename_node_in_dsl(
    toml_text: &str,
    old: &str,
    new_label: &str,
) -> Result<String, ConceptMapMutationError> { ... }
```

Each reuses the existing `get_dsl` / `set_dsl` pair and the line-level logic
already in `run_add`/`run_remove`/`run_rename_node`.

#### Module seam guard

`concept_map.rs` now serves CLI commands, web routes, validation, DSL editing,
and diagnostics. That is acceptable for this slice. If the module swells, split
along natural seams:

```
src/concept_map.rs              public façade / command support
src/concept_map/dsl.rs          parse, derive_node_key, line model
src/concept_map/mutate.rs       add/remove/rename pure mutations
src/concept_map/check.rs        diagnostics
src/concept_map/io.rs           read_concept_map, get_dsl, set_dsl
```

The hard contract above ("routes do not own DSL semantics") is the enforcement
mechanism — routes must not inline DSL parsing, key derivation, or validation.
They call through `concept_map`'s public API.

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
    EmptyField { field: &'static str },
    StaleConceptMap,
}
```

Each maps to an HTTP status code via the existing `IntoResponse` impl:
- `BadConceptMapId` → 400
- `ConceptMapNotFound` → 404
- `DuplicateEdge` → 409
- `EdgeNotFound` → 404
- `NodeCollision` → 409
- `EmptyField` → 400
- `StaleConceptMap` → 409
- `ConceptMapParseError` → 500

`From<ConceptMapMutationError> for MapServerError` provides the safe downcast:
```rust
impl From<ConceptMapMutationError> for MapServerError {
    fn from(e: ConceptMapMutationError) -> Self {
        match e {
            ConceptMapMutationError::EmptyField { field } => MapServerError::EmptyField { field },
            ConceptMapMutationError::DuplicateEdge { line } => MapServerError::DuplicateEdge { line },
            ConceptMapMutationError::EdgeNotFound { source, rel, target } => MapServerError::EdgeNotFound { source, rel, target },
            ConceptMapMutationError::NodeCollision { existing_label, line } => MapServerError::NodeCollision { existing_label, line },
            ConceptMapMutationError::MissingDsl | ConceptMapMutationError::InvalidToml(_) => MapServerError::ConceptMapParseError(e.to_string()),
        }
    }
}
```

## 3. Frontend State Model

### New state fields (model.js)

```js
conceptMapCache: new Map(),   // id → { nodes, edges, diagnostics, dslHash }
editingConceptMap: false,     // authoring mode toggle
editingNode: null,            // { key, label } — node currently being renamed inline
```

`conceptMapCache` is cleared on refresh alongside `markdownCache`. The refresh
handler (`wireRefresh` in app.js) must clear both caches. After a CM mutation,
the cache is updated in-place from the POST response (including `dslHash`);
a manual refresh re-fetches from the server. The catalog graph (which holds the
CM entity node's `status`) is NOT updated by CM mutations — status changes require
a refresh, consistent with all other entity kinds.

When the focused entity changes or a graph refresh changes entity status, any
stale `conceptMapCache` entries for unfocused CMs are eligible for eviction —
the frontend re-fetches on next focus.

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
5. If diagnostics are non-empty, render diagnostics panel (see §6).
6. Render CM edge table.
7. If `editingConceptMap`, show add-edge form and wire remove/rename controls.
8. Render markdown pane (unchanged — `GET /api/entity/CM-001/markdown` already works if the entity is in the catalog graph).

## 4. DOT Generation & SVG Wiring

### DOT escape helper contract

The existing `dotQuote` helper (or a new sibling like `dot.escapeStringContent`)
must: **escape content but NOT add surrounding quotes**. The caller adds the
enclosing double quotes. If the existing `dotQuote` already returns quoted DOT
strings, extract an `escapeStringContent` that returns bare escaped content.

```js
// Contract: dot.escapeStringContent escapes \ " newline and DOT-special characters
// (>, ], }), but does NOT add surrounding quotes.
// Caller adds quotes: '"' + dot.escapeStringContent(value) + '"'
```

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
    lines.push('  "' + dot.escapeStringContent(node.key) + '" [label="' + dot.escapeStringContent(node.label) + '"];');
  });

  lines.push('');

  conceptMapData.edges.forEach(function(edge) {
    lines.push('  "' + dot.escapeStringContent(edge.from_key) + '" -> "' + dot.escapeStringContent(edge.to_key) +
      '" [label="' + dot.escapeStringContent(edge.rel) + '"];');
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
- On submit: `POST /api/concept-map/:id` with `add_edge` and `base_hash` from the cache's `dslHash`.
  - 200: update `conceptMapCache` with returned nodes/edges/hash, re-render diagram + edge table, clear form.
  - 409 `duplicate_edge`: show inline warning "This edge already exists at line {n}" below the form.
  - 409 `stale_concept_map`: show "Concept map was modified elsewhere — refreshing" and auto-refetch.

### Edge table (edit mode)

```
┌─ CM Edges ────────────────────────────────────────────┐
│ Source          │ Rel         │ Target        │        │
│ User Story      │ expresses   │ User Need     │  [✕]   │
│ Capability      │ aggregates  │ Requirements  │  [✕]   │
└───────────────────────────────────────────────────────┘
```

- Each row has a `[✕]` remove button. On click: confirm (optional — could be instant), `POST` with `remove_edge` and `base_hash`.
- 200: update cache, re-render.
- 404: show inline "Edge no longer exists — it may have been removed elsewhere"; refresh edge list from cache.
- 409 `stale_concept_map`: auto-refetch.

### Node rename

Clicking a node label in the edge table or the SVG opens an inline `<input>`:

```
┌─ CM Edges ────────────────────────────────────────────┐
│ [User Narrative___] │ expresses   │ User Need     │    │
└───────────────────────────────────────────────────────┘
```

- Pre-filled with current label. On Enter: `POST` with `rename_node` and `base_hash`. On Escape: cancel.
- 200: update cache, re-render.
- 409 `node_collision`: show inline warning "Rename would collide with existing node '{label}'"; input stays open with current value.
- 409 `stale_concept_map`: auto-refetch, input cancelled.

### Autocomplete data flow

- After every successful mutation, the response includes updated `nodes`/`edges`/`dslHash`.
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
│                      │ ┌ Diagnostics ────────────────────────┐ │
│                      │ │ ⚠ line 7: "SL-001" looks like an    │ │
│                      │ │   entity reference                  │ │
│                      │ │ ⚠ line 9: similar node labels:      │ │
│                      │ │   "User Story" / "User Stories"     │ │
│                      │ └─────────────────────────────────────┘ │
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

### Diagnostics panel

Appears in view mode when `diagnostics` is non-empty. Each diagnostic line shows:
- `⚠` icon
- `line N:` if the diagnostic has a line number
- The diagnostic message (variant-specific text)

Diagnostic variants and their messages:
| Variant | Message |
|---|---|
| `CanonicalNodeCollision` | Node label "{label}" collides with key "{key}" (first label "{first}" takes precedence) |
| `SelfEdge` | Self-referencing edge: "{label}" → "{label}" |
| `SimilarNodeLabel` | Similar node labels: "{a}" / "{b}" |
| `RelationDrift` | Relation "{rel}" appears only once — possible typo |
| `EntityRefLike` | "{label}" looks like an entity reference |

Diagnostics with no `line` field (e.g., `CanonicalNodeCollision` for the deduplication pass) omit the `line N:` prefix.

### Layout when CM is focused (edit mode)

Same as above, plus:
- [Done] button replaces [Edit]
- Add edge form appears above the CM Edges table
- Each edge row has a [✕] button
- Clicking node labels triggers inline rename
- Diagnostics panel hidden (authoring mode is for editing, not diagnostics)

### State table

| Element | Entity mode (existing) | CM view mode | CM edit mode |
|---|---|---|---|
| Graph pane | Entity graph SVG | CM diagram SVG | CM diagram SVG (same) |
| Header | Entity title + kind + status | CM title + "concept map · draft" | Same + [Done] button |
| Hover pane | Entity details (id, kind, status) | Node label + "(concept map node)" | Same |
| Depth selector | Visible (0–3) | Hidden | Hidden |
| Diagnostics panel | Hidden | Visible (if non-empty) | Hidden |
| Edge table | Entity relationship table | CM edges (read-only) | CM edges + [✕] per row |
| Add edge form | Hidden | Hidden | Visible |
| Markdown pane | Entity .md body | CM .md body | CM .md body |

### Error states

| State | Trigger | Behaviour |
|---|---|---|
| CM not found | GET 404 | "Concept map {id} not found" in graph pane; sidebar still shows CM node |
| CM parse failure | Rust TOML parse error | "Failed to load concept map: {message}" error |
| Add edge 409 duplicate | Duplicate edge | Inline warning "This edge already exists at line {n}" |
| Add edge 400 | Empty field | Inline validation on offending field |
| Remove edge 404 | Already removed | Inline message; refresh edge list from cache |
| Rename node 409 collision | Key collision | Inline warning "Rename would collide with existing node '{label}'" |
| Any POST 409 stale | base_hash mismatch | Auto-refetch, discard mutation, notify user |
| Bad action discriminator | POST with unknown action | "Unknown action: {action}" error (400) |
| Stale DOT render | graphRenderSeq mismatch | Silently discard (same guard as entity graph) |
| File I/O error | Disk read/write failure | 500 "Internal server error" |

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

### Rust tests — registration & catalog

1. **KINDS registration**: `integrity::KINDS` contains a `"CM"` prefix row. `scan_entities` picks up seeded CM entities. No panic in `outbound_for` with `"CM"` prefix.
2. **Markdown path assembly**: seed a CM fixture, call `read_entity_markdown`, assert the returned path contains `concept-map-001.md` (not `cm-001.md`). This is an explicit Rust route/unit test.

### Rust tests — GET

3. **GET /api/concept-map/:id**: seeded CM fixture → 200 with correct nodes/edges/diagnostics/dsl_hash. Non-existent id → 404. Bad id format → 400.
4. **GET diagnostics — CanonicalNodeCollision**: CM with multiple labels sharing a key → diagnostics include collision warning, nodes deduplicated first-wins.
5. **GET diagnostics — malformed DSL lines**: CM with bad DSL lines (e.g., missing `>`) → diagnostics include `MalformedLine` or `EmptyLabel` (parse-time findings).
6. **GET I/O error mapping**: missing TOML file → 404; invalid TOML syntax → 500; TOML without `dsl` key → graceful (empty nodes/edges or parse error depending on check semantics).

### Rust tests — POST mutations (happy path)

7. **POST add_edge**: valid edge → 200 with updated nodes/edges/hash. Edge line count increases by 1.
8. **POST remove_edge**: existing edge → 200 with updated nodes/edges/hash. Edge line count decreases by 1.
9. **POST rename_node**: valid rename → 200 with `occurrences` matching affected edge count. All edge source/target labels updated.

### Rust tests — POST mutations (error paths)

10. **POST add_edge duplicate**: existing edge → 409 with `line` field.
11. **POST remove_edge not found**: non-existent edge → 404.
12. **POST rename_node collision**: rename to existing derived key with different spelling/punctuation → 409 with `existing_label` and `line`.
13. **POST case-only rename**: "User Story" → "USER STORY" → 200. `occurrences` is the count of edge lines actually changed. DSL text is edited (case changed on source/target labels). Keys are unchanged.
14. **POST text-identical rename**: "User Story" → "User Story" → 200, `occurrences: 0`.
15. **POST empty field trim**: `"  "` source/rel/target/old/new → 400 `empty_field` for each action.
16. **POST bad action discriminator**: `{ "action": "invalid_action" }` → 400 (not 500).

### Rust tests — stale-write guard

17. **POST with matching base_hash**: mutation succeeds (200).
18. **POST with mismatched base_hash**: returns 409 `stale_concept_map`, file unchanged.
19. **POST without base_hash**: mutation succeeds (last-write-wins, no guard).

### Rust tests — TOML/DSL preservation

20. **TOML field preservation**: add edge, verify all other fields (`slug`, `title`, `status`, `description`, `created`) survive unchanged.
21. **DSL comment preservation**: add edge, verify comments before/inside/after multiline DSL survive. Remove edge, verify same.
22. **Whitespace/case preservation**: mutation preserves internal whitespace and case for relation text. Duplicate detection uses exact normalized relation text.

### Rust tests — adversarial labels & DOT

23. **Labels with special characters**: labels containing `"`, `\`, `]`, newline, `>`, DSL delimiter characters (`→`, `>`). Verify DOT generation does not break, `cmGraphToDot` output is valid DOT.
24. **Duplicate edge detection under normalization rules**: edges with varying whitespace around relation text, exact same `(source, rel, target)` after trim.

### Rust tests — collision edge cases

25. **Rename within same key, different spelling**: "User Story" → "user-story" (same derived key) → allowed, updates DSL text.
26. **Rename to existing key with different spelling**: "User Need" → "User-Story" when "User Story" exists and both derive to same key → 409.
27. **Rename sole key-holder**: node with one label renamed to different-cased version → allowed.

### JS tests

28. **CM detection**: `isConceptMap("CM-001")` returns true when a CM entity exists in the graph. Returns false for non-CM entities like `"SL-001"`.
29. **normalizeConceptMap**: parses the GET response shape into the internal cache format, including `dslHash`.
30. **cmGraphToDot**: produces valid DOT with correct node/edge counts, ellipse shape, CM colour. Test with hostile labels (quotes, backslashes, newlines).
31. **Autocomplete**: datalist is rebuilt from cache after add/remove/rename mutations.
32. **Diagnostics panel**: render non-empty diagnostics → panel visible with correct messages. Empty diagnostics → panel hidden.
33. **Stale-write handling**: 409 `stale_concept_map` response → cache auto-refetched, user notified.
34. **Cache invalidation**: cache entry for CM-001 is stale after refresh. Focus change between CMs and non-CMs correctly switches render path.

### Integration smoke test

35. `doctrine concept-map new "Test Map"` → `doctrine concept-map add CM-001 "A" "relates to" "B"` → `doctrine map serve --open --focus CM-001`. Verify: CM appears in sidebar, diagram renders with 2 nodes + 1 edge, hover pane shows "A" on mouseenter.
36. Edit mode: toggle Edit, add edge "B > depends on > C", verify diagram updates, `doctrine concept-map show CM-001` shows 2 edges.
37. Remove edge: click ✕ on "A > relates to > B", verify diagram updates, CLI `show` confirms removal.
38. Rename node: click "A" in edge table, rename to "Alpha", verify diagram updates, all edges reference "Alpha".

### Gate

- `just check` — root package tests pass
- `just gate` — workspace clean (clippy zero warnings)

## 9. Stale-Write Posture

An optional `base_hash` mechanism guards against accidental overwrites from
multiple browser tabs:

- GET returns the current DSL's SHA-256 as `dsl_hash`.
- POST accepts an optional `base_hash`. If present and mismatched, the server
  returns 409 `stale_concept_map` and does not mutate.
- If `base_hash` is absent, last-write-wins semantics apply. This preserves
  simplicity for single-tab usage and non-web clients.

The frontend always sends `base_hash` from its cached `dslHash`. On 409, it
auto-refetches the latest data and notifies the user.

For a single-user loopback tool this is a low-cost correctness improvement.
Two browser tabs will see the 409 on collision rather than silently overwriting
each other's work.

## 10. Open Questions & Deferred

### Acceptance tradeoffs (explicit)

- **Last-write-wins without `base_hash`.** Clients that omit `base_hash` get no
  revision guard. Simple scripts and non-web callers can skip it. This is an
  acceptable tradeoff for a single-user tool — the hash is optional, not required.
- **Synchronous file I/O in async handlers.** The POST handler uses `std::fs::write`
  inside an async route. For a single-user loopback server serving small TOML files,
  this is not a blocking concern. If the server migrates to an async file I/O layer,
  wrap in `tokio::task::spawn_blocking`.

### Deferred to follow-up slices

- **Entity-ref support**: CM node labels that match canonical entity refs (`SL-001`) get special styling and click-to-navigate. Requires extending `parse_dsl`/`check` to classify nodes as entity-refs, and DOT generation to add URL attributes.
- **Web-based CM creation**: "New Concept Map" button in the web UI. Requires a `POST /api/concept-maps` route and the full scaffold path (which currently lives in the CLI `run_new`).
- **Visual graph editing**: drag nodes, draw edges. Requires a canvas/SVG editing library and a fundamentally different interaction model.
- **Cross-kind structural relations**: `doctrine link CM-001 governed_by ADR-001`. Requires the `outbound_for` arm to read `[[relation]]` rows and the concept map TOML to support a `[relationships]` table.
- **CM description editing**: inline edit of the `description` field in the web UI.
- **CLI rename collision check**: port the key-based collision logic from `rename_node_in_dsl` to `run_rename_node` so CLI and web share the same safety check.

### Open questions resolved in design

- **Q: How does the CM diagram relate to the entity graph?** A: Replace the graph pane when a CM is focused (option A). A "back" mechanism is implicit — clicking any non-CM entity in the sidebar switches back.
- **Q: What shape is the authoring interface?** A: Form-based line operations (option B) — add edge form, remove buttons, inline rename. No raw DSL textarea.
- **Q: How do CMs appear in the sidebar?** A: Uniform list (option A) — CM entities appear in the same entity list with a `CM` kind pill, same hash route (`#/focus/CM-001`).
- **Q: Where does parsing live?** A: Rust. JS receives structured `{nodes, edges, diagnostics}`. No JS port of `parse_dsl`, `levenshtein`, or `derive_node_key`.
- **Q: Mutation API style?** A: Single `POST` with `action` discriminator — three actions (`add_edge`, `remove_edge`, `rename_node`), each maps 1:1 to an existing Rust function.
