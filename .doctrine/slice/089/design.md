# SL-089 Design: Map Explorer — backlog filtering, actionability graph, prioritization views

## Hard Contracts

1. **ADR-001** (module layering): web frontend is leaf tier. The map server is engine
   tier. The priority subsystem is engine tier. No leaf→engine inversion.
2. **ADR-004** (relations outbound-only): edges in the graph are stored outbound;
   inbound is derived. The survey endpoint returns the priority engine's computed
   inbound consequence, not raw edge reversal.
3. **ADR-010** (relation modelling): the semantic graph view is the existing
   `CatalogGraph` projection. The actionability view is a different projection
   (`PriorityGraph` → `ActionabilityView`), not a new edge label.
4. **Priority engine owns truth**: client-side JS must not reimplement eligibility,
   blocking, or ordering. The frontend consumes canonical actionability-graph JSON
   from the server — it is a render-only consumer. JS performs mechanical SVG layout
   (d3-dag computes coordinates from the server-supplied graph structure); it does
   not determine what is blocked, eligible, actionable, or ranked. If the frontend
   diverges from the CLI, the API is wrong.

## 1. Architecture & Module Layout

```
┌─────────────────────────────────────────────────────────┐
│ Browser (leaf tier)                                     │
│                                                         │
│  api.js   fetchActionabilityGraph() → GET /api/survey   │
│  model.js  state.viewMode, state.actionabilityView      │
│  priority.js  D3 sugiyama layout + render (NEW)         │
│  dot.js      DOT string builder (unchanged)             │
│  svg.js      SVG DOM injection (unchanged)              │
│  render.js   entity list, tables, markdown (unchanged)  │
│  search.js   kind filters (split checkboxes)            │
│  app.js      view toggle wiring, render dispatch        │
├─────────────────────────────────────────────────────────┤
│ Map server (engine tier)                                │
│                                                         │
│  routes.rs    GET /api/survey handler (NEW)             │
│  state.rs     AppState { stores: RwLock<DataStores> }   │
│  priority/surface.rs  survey_view_for_map(g) (NEW)      │
│  priority/view.rs     ActionabilityView, Actionability- │
│                       Node, ActionabilityEdge (NEW)     │
│  priority/graph.rs    build(root) (existing)            │
│  priority/channels.rs eligible, blocked_by, … (exist)   │
│  priority/render.rs   survey_json(rows) (unchanged)     │
└─────────────────────────────────────────────────────────┘
```

### View toggle

Two rendering modes, selected by a radio/segmented control in the graph area header:

| Mode | Data source | Renderer | Edges shown |
|---|---|---|---|
| **Semantic** (default) | `GET /api/graph` | DOT → Graphviz SVG | all relation edges |
| **Actionability** | `GET /api/survey` | D3 (`d3-dag` sugiyama) | `needs` (hard block) + `after` (soft seq) |

Toggling switches the renderer, edge filter, and table columns. The toggle is
per-session in `state.viewMode`; not persisted. Default is `'semantic'`.

When actionability mode is active on a non-work entity (ADR, SPEC, REQ, etc. —
no `ActionabilityNode` exists for it):
- **Graph area**: show inline message: "This entity has no dep/seq edges —
  switch to a work entity or use Semantic view."
- **Relationship table**: falls back to showing entity details from
  `state.graph.nodes` (id, kind, status, title) — the same data shown in
  the entity list, not the actionability column layout.
- **Entity list**: unchanged — still shows all entities matching the current
  search + kind filter.

### Kind filter interaction contract

- **Entity list**: kind filters + search apply. Only entities of checked kinds
  matching the search term appear in the entity list.
- **Actionability graph**: NOT filtered by kind checkboxes. The graph always
  shows all work entities with dep/seq edges, regardless of which kind
  checkboxes are checked. The actionability graph is a global view of work
  ordering — filtering it by kind would hide blockers and misrepresent the
  dependency structure.
- **Relationship table in actionability mode**: shows every row from the
  actionability view (unfiltered by kind). In semantic mode, follows existing
  behaviour.

### New / modified files

| Path | Status | Purpose |
|---|---|---|
| `src/priority/view.rs` | **modify** | Add `ActionabilityView`, `ActionabilityNode`, `ActionabilityEdge` types |
| `src/priority/surface.rs` | **modify** | Extract `survey_for_map` (pure) + add `survey_view_for_map(g) → ActionabilityView` |
| `src/map_server/state.rs` | **modify** | Add `DataStores` wrapper under single `RwLock` |
| `src/map_server/routes.rs` | **modify** | Add `GET /api/survey`; atomic refresh via `DataStores` |
| `src/map_server/mod.rs` | **modify** | Build catalog + priority_graph + graph at startup into `DataStores` |
| `web/map/vendor/README.md` | **new** | Version, source URL, checksum, license for each vendored asset |
| `web/map/vendor/d3.v7.min.js` | **new** | D3 v7, vendored locally |
| `web/map/vendor/d3-dag.min.js` | **new** | d3-dag sugiyama layout, vendored locally |
| `web/map/priority.js` | **new** | Consumes actionability-graph JSON, renders D3 layered dep graph |
| `web/map/model.js` | **modify** | Add `viewMode`, `actionabilityView`, setter |
| `web/map/api.js` | **modify** | Add `fetchActionabilityGraph()`, wire refresh to flush cache |
| `web/map/app.js` | **modify** | View toggle wiring, dispatch semantic vs actionability render |
| `web/map/render.js` | **modify** | `--active` class on toggle button |
| `web/map/search.js` | **modify** | Actionability-mode column changes (minor) |
| `web/map/index.html` | **modify** | Split kind checkboxes; view toggle control; D3 script tag |
| `web/map/style.css` | **modify** | CSS variables for node/edge colours; toggle control styles |

## 2. Backend — `/api/survey` endpoint

### AppState (revised — D9)

```rust
/// All three priority data stores — built and replaced atomically.
struct DataStores {
    catalog: Catalog,
    priority_graph: PriorityGraph,
    graph: CatalogGraph,
}

pub(crate) struct AppState {
    pub(crate) root: PathBuf,
    pub(crate) stores: Arc<RwLock<DataStores>>,  // single lock = atomic refresh
    pub(crate) dot_renderer: Arc<dyn DotRenderer>,
}
```

All three stores live under ONE `RwLock<DataStores>`. A refresh builds the full
`DataStores` on the stack, then replaces the contents in a single `write()`
acquisition — no window where a reader sees a fresh `catalog` but a stale
`priority_graph`. The build sequence:

1. `scan_catalog(&root)` → `Catalog`
2. `priority::graph::build(&root)` → `PriorityGraph` (scans disk, same as CLI)
3. `CatalogGraph::from_catalog(&catalog)` → `CatalogGraph`

At startup (`serve()`), build once. On `POST /api/refresh`, rebuild and swap.

`CatalogGraph` is a pure projection of `Catalog` — storing both is deliberate
redundancy (D8): the `/api/graph` handler reads `graph` under a read lock
without touching `Catalog`. Removing `graph` and projecting on-the-fly from
`catalog` would require either a write lock or a clone of the full `Catalog`
on every `/api/graph` request.

### New view types (priority/view.rs)

```rust
/// One node in the actionability graph — the render source of truth for
/// the web UI. Carries the server-computed rank (topological layer over
/// the dep overlay) so the frontend never computes ordering.

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActionabilityNode {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub actionability: String,   // "actionable" | "blocked" | "terminal"
    pub consequence: u32,
    pub rank: u32,               // topological layer: 0 = no non-terminal blockers
    pub blockers: Vec<String>,   // direct non-terminal blockers (canonical refs)
}

/// One edge in the actionability graph.

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActionabilityEdge {
    pub source: String,          // canonical ref of the prerequisite
    pub target: String,          // canonical ref of the dependent
    pub kind: String,            // "needs" (hard block) | "after" (soft sequence)
}

/// The full actionability graph for the web UI.

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActionabilityView {
    pub kind: String,            // "actionability_graph"
    pub policy_version: String,  // "priority.v2"
    pub nodes: Vec<ActionabilityNode>,
    pub edges: Vec<ActionabilityEdge>,
}
```

### New surface function

```rust
// src/priority/surface.rs

/// Pure survey over an already-built PriorityGraph. No disk scan.
///
/// Identical to the existing `survey()` body, minus the `graph::build(root)?`
/// preamble. The original `survey()` is refactored to:
///   pub(crate) fn survey(root: &Path, all: bool) -> anyhow::Result<Vec<SurveyRow>> {
///       let g = graph::build(root)?;
///       Ok(survey_for_map(&g, all))
///   }
///
/// Filtering (when `all == false`):
///   1. channels::eligible(&g, k)  — status-class gate (Workable only)
///   2. !channels::promoted(&g, k) — exclude promoted-backlog items
/// These two filters exactly match the CLI `survey` default.
pub(crate) fn survey_for_map(g: &PriorityGraph, all: bool) -> Vec<SurveyRow> {
    // The existing decorate-sort-undecorate pipeline from surface::survey(),
    // moved into a pure function. Zero behavioural change.
}

/// Build the actionability graph view for the web UI from a PriorityGraph.
///
/// Pure over the graph — no disk, no clock. Returns nodes with server-computed
/// ranks, plus the `needs` and `after` edges among work entities.
///
/// Node set (default, all=false): eligible AND !promoted — exactly the
/// `survey_for_map` filter. Every node carries its rank (topological layer
/// over the dep overlay: 0 = no non-terminal blockers).
///
/// Edges:
///   - `needs` edges: dep overlay, non-terminal source only → oriented
///     prerequisite→dependent (matching the B→A flip stored in the graph).
///   - `after` edges: seq overlay, oriented prerequisite→dependent.
///     Both source and target must be in the node set.
pub(crate) fn survey_view_for_map(g: &PriorityGraph, all: bool) -> ActionabilityView {
    // 1. Build rows via survey_for_map (the canonical eligible set + ordering).
    // 2. Compute rank per node: topological layer over the dep overlay,
    //    counting only non-terminal sources.
    // 3. Extract needs edges (dep overlay, non-terminal src, both ends in node set).
    // 4. Extract after edges (seq overlay, both ends in node set).
    // 5. Assemble ActionabilityView.
}
```

### Handler

```rust
// src/map_server/routes.rs

async fn survey(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, MapServerError> {
    let stores = state.stores.read().await;
    let view = crate::priority::surface::survey_view_for_map(&stores.priority_graph, false);
    let body = serde_json::to_string_pretty(&view)
        .map_err(|e| MapServerError::Other(e.into()))?;
    Ok((
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        body,
    ))
}
```

### Route

```rust
.route("/api/survey", get(survey))
```

### Refresh (D9 consistent)

```rust
async fn refresh(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, MapServerError> {
    let catalog = crate::catalog::hydrate::scan_catalog(&state.root)
        .map_err(MapServerError::Other)?;
    let pg = crate::priority::graph::build(&state.root)
        .map_err(|e| MapServerError::Other(e.into()))?;
    let g = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);

    let stores = DataStores { catalog, priority_graph: pg, graph: g };
    *state.stores.write().await = stores;  // single write = atomic swap

    Ok(Json(json!({"ok": true})))
}
```

### JSON shape

Returned by `GET /api/survey`:

```json
{
  "kind": "actionability_graph",
  "policy_version": "priority.v2",
  "nodes": [
    {
      "id": "ISS-011",
      "title": "SubagentStart hook merge keys ...",
      "kind": "ISS",
      "status": "open",
      "actionability": "actionable",
      "consequence": 3,
      "rank": 0,
      "blockers": []
    },
    {
      "id": "IMP-047",
      "title": "Trinary actionability...",
      "kind": "IMP",
      "status": "open",
      "actionability": "blocked",
      "consequence": 0,
      "rank": 2,
      "blockers": ["IMP-033", "IMP-012"]
    }
  ],
  "edges": [
    {"source": "IMP-033", "target": "IMP-047", "kind": "needs"},
    {"source": "IMP-012", "target": "IMP-047", "kind": "after"},
    {"source": "ISS-011", "target": "IMP-033", "kind": "needs"}
  ]
}
```

The frontend consumes `nodes` + `edges` directly. `policy_version` is carried
for future compatibility. `kind: "actionability_graph"` distinguishes the
envelope from the CLI `survey` JSON.

`rank` is the topological layer computed server-side: layer 0 = no non-terminal
blockers (ready to start); layer N = 1 + max(layer of each non-terminal blocker).
`blockers` lists direct non-terminal prerequisites (the same set `blocked_by`
returns). `edges` carry the full graph structure for the D3 renderer — `needs`
edges are hard blocks (solid red, arrowhead), `after` edges are soft sequence
(dashed amber, no arrowhead).

### Test plan (backend)

| Test | What it proves |
|---|---|
| `survey_returns_200_valid_json` | Endpoint responds with `kind: "actionability_graph"` envelope |
| `survey_actionable_row` | Unblocked open item → `actionability: "actionable"`, `rank: 0`, `blockers: []` |
| `survey_blocked_row` | Item with unsatisfied `needs` → `actionability: "blocked"`, `blockers` non-empty, `rank > 0` |
| `survey_excludes_terminal` | Closed/done item absent from nodes (default) |
| `survey_consequence_count` | Item blocked by 2 others → `consequence: 2` on each blocker |
| `survey_needs_edges_present` | dep overlay edges appear in `edges[]` with `kind: "needs"` |
| `survey_after_edges_present` | seq overlay edges appear in `edges[]` with `kind: "after"` |
| `survey_empty_graph` | No entities → `nodes: []`, `200` |
| `survey_refresh_updates` | Add entity, POST refresh, GET survey → new item appears |
| `survey_rank_topological` | Chain A→B→C: rank(A)=0, rank(B)=1, rank(C)=2 |
| `survey_terminal_blocker_no_edge` | Terminal node as `needs` source → no edge emitted |

### Design decisions

- **D1: Dual-store in AppState.** `Catalog` + `PriorityGraph` live alongside
  `CatalogGraph`. All three refreshed atomically under a single `RwLock`.
  Rationale: the priority engine already speaks `Catalog`-adjacent types;
  storing both avoids coupling the survey endpoint to the lossy graph
  projection.
- **D2: `survey_for_map` is pure extraction, not new logic.** The existing
  `surface::survey()` body is split: disk scan stays in the CLI path; the pure
  filter/decorate/sort/map logic moves to `survey_for_map`. Zero behavioural
  divergence.
- **D3: `survey_view_for_map` returns the full actionability graph shape.**
  Nodes carry `rank` computed server-side via topological layer over the dep
  overlay; edges carry `needs` and `after` relationships. The frontend consumes
  these as a pure graph structure — d3-dag computes coordinates, but the
  frontend never determines what is blocked, eligible, actionable, or ranked.
- **D4: D3 for actionability, Graphviz DOT for semantic.** Two different
  visualization problems. D3 sugiyama handles layered DAGs cleanly; Graphviz
  DOT handles hierarchical relation webs. Separate renderers, same app shell.
- **D5: Frontend is render-only.** No JS reimplementation of eligibility,
  blocking, or ordering. d3-dag receives a graph structure and computes
  coordinates; the server supplies the graph (nodes + ranks + edges). If
  frontend diverges from CLI output, the API is wrong.
- **D6: `all=false` by default on survey endpoint.** Matches CLI default.
  `?all=true` is a future addition.
- **D7: View toggle not persisted.** Keeps scope small. Follow-up can add
  `localStorage`.
- **D8: `<script>` tag for D3 vendor (not ES module).** Matches existing vendor
  pattern (`markdown-it.min.js`, `purify.min.js`). ES5 IIFE convention preserved.
- **D9: `CatalogGraph` is deliberate redundancy.** Storing alongside `Catalog`
  avoids per-request projection cost on the hot `/api/graph` read path.
- **D10: `DataStores` wrapper under single `RwLock`.** Atomic refresh — no window
  where a reader sees a fresh catalog but stale priority_graph. All handlers and
  the refresh path use `state.stores` consistently.
- **D11: `survey_for_map` preserves promoted-backlog exclusion.**
  `!channels::promoted(&g, k)` filter exactly matches CLI `survey` default.

## 3. Frontend — D3 actionability graph

### priority.js module contract

```javascript
// priority.js — D3 layered dep-graph renderer for the actionability view.
// Consumes the GET /api/survey response (ActionabilityView). Depends on d3
// and d3-dag (global, loaded via <script> tags).

var priority = {};

// Compute a D3 sugiyama layout from the server-supplied graph structure.
// The server provides nodes (with rank) and edges — d3-dag only computes
// (x, y) coordinates.  Returns { nodes: [{..., x, y}], edges: [{..., points}] }.
// Does NOT determine eligibility, blocking, or rank.
priority.layoutGraph = function(view) { ... };

// Render the laid-out graph as SVG into a container element.
// opts: { container: Element, layout: layoutResult, focusId: string,
//         depth: number, onNodeClick: fn(id), onNodeHover: fn(id|null) }
priority.renderGraph = function(opts) { ... };
```

### D3 usage

- **Layout**: `d3-dag` sugiyama (layered DAG). Two vendor files: `d3.v7.min.js`
  (for `d3-selection` DOM manipulation) and `d3-dag.min.js` (UMD bundle,
  extends the `d3` global with `d3.sugiyama()`, `d3.graphStratify()`, etc.).
  d3-dag receives the graph structure (nodes + edges) from the server and
  computes only (x, y) coordinates — it does not determine rank, eligibility,
  or blocking.

- **Nodes**: circles or rounded rects, styled by CSS classes:
  - `.priority-actionable` — ready to start
  - `.priority-blocked` — held by blockers
  - `.priority-terminal` — closed/done/resolved

- **Colours via CSS variables** (dark-theme compatible — OQ-3):
  ```css
  :root {
    --priority-actionable-bg: #27AE60;
    --priority-actionable-fg: #ffffff;
    --priority-blocked-bg: #E67E22;
    --priority-blocked-fg: #ffffff;
    --priority-terminal-bg: #95A5A6;
    --priority-terminal-fg: #ffffff;
    --priority-needs-edge: #C0392B;
    --priority-after-edge: #E67E22;
  }
  ```
  JS reads computed styles or applies classes only — no hardcoded colour values.

- **Edges**:
  - `needs` (hard block): solid line, `var(--priority-needs-edge)`, arrowhead
  - `after` (soft seq): dashed line, `var(--priority-after-edge)`, no arrowhead

- **Focus**: highlighted border on the focused node. Depth controls work as
  before (BFS neighbourhood from focus, same 0–3 range).

- **Hover**: tooltip/panel showing title, status, blockers list.

- **Click**: navigates to the entity (sets focus, same as semantic view).

- **Consequence**: shown as a small badge/count on each node (inbound dep count).

### Layout strategy

The DAG is rendered top-to-bottom (`rankdir=TB` equivalent). `rank` is supplied
by the server: rank 0 (actionable, no blockers) at the top; `needs` edges point
downward (blocker → blocked). This gives a natural "what should I do first?"
reading: top items are ready to start.

`after` edges are overlaid but do not drive the rank assignment — they annotate
existing node positions with dashed lines.

### Relationship table in actionability mode

When `viewMode === 'actionability'`, the relationship table columns change:

| Before (semantic) | After (actionability) |
|---|---|
| src_id, src_title, label, tgt_id, tgt_title | id, kind, status, actionability, blockers, consequence, title |

Sources: `ActionabilityNode` fields, joined with `state.graph.nodes` for titles.

### Edge legend update

In actionability mode, the edge legend collapses to two items:
- **needs** (hard prerequisite) — solid red
- **after** (soft sequence) — dashed amber

In semantic mode, the existing full legend is shown.

## 4. Backlog kind filtering

### Current state

The kind filter groups backlog kinds into one row:

```html
<label class="kind-checkbox">
  <input … data-kinds="ISS,IMP,CHR,RSK"> ISS/IMP/CHR/RSK Issues / Risks / Chores
</label>
<label class="kind-checkbox">
  <input … data-kinds="IDE"> IDE Ideas
</label>
```

### Target state

Split into five individual rows:

```html
<label class="kind-checkbox">
  <input … data-kinds="ISS"> ISS Issues
</label>
<label class="kind-checkbox">
  <input … data-kinds="IMP"> IMP Improvements
</label>
<label class="kind-checkbox">
  <input … data-kinds="CHR"> CHR Chores
</label>
<label class="kind-checkbox">
  <input … data-kinds="RSK"> RSK Risks
</label>
<label class="kind-checkbox">
  <input … data-kinds="IDE"> IDE Ideas
</label>
```

No logic changes needed — `search.collectKindFilter` already aggregates by
`data-kinds` and `search.renderFilteredEntities` already applies `kindFilter`.
This is purely an HTML change.

### Actionability graph vs kind filters

The actionability graph is NOT filtered by kind checkboxes. It always shows all
work entities with dep/seq edges — filtering the graph by kind would hide
blockers and misrepresent the dependency structure. The kind checkboxes affect
the entity list only. The relationship table in actionability mode also shows
all rows from the actionability view (unfiltered).

## 5. View toggle UI

### Location

Graph area header, between the focus-header and the graph-area:

```html
<div class="view-toggle">
  <button class="view-btn active" data-view="semantic">Semantic</button>
  <button class="view-btn" data-view="actionability">Actionability</button>
</div>
```

### Behaviour

- Clicking a button sets `state.viewMode`, updates `--active` classes, re-renders
  the graph area and the relationship table.
- The toggle is always visible (even when focused on a non-work entity).
- Non-work entity in actionability mode: show inline message in graph area instead
  of attempting to render.

### CSS

```css
.view-toggle {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}
.view-btn {
  padding: 4px 12px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
}
.view-btn.active {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}
```

## 6. Data flow

```
Browser startup
  │
  ├─ GET /api/graph        ──→ state.graph (all nodes + edges)
  └─ GET /api/survey       ──→ state.actionabilityView (nodes + edges)

Every render cycle:
  │
  ├─ viewMode === 'semantic'
  │   └─ dot.js builds DOT from state.graph neighbourhood
  │      └─ POST /api/dot/svg ──→ inline SVG
  │
  └─ viewMode === 'actionability'
      └─ priority.js layouts + renders from state.actionabilityView
         (no server round-trip)

POST /api/refresh:
  │
  ├─ server: atomically rebuilds DataStores (single write lock)
  └─ client: state.actionabilityView = null; re-fetch /api/graph + /api/survey
```

## 7. Design decisions

| Id | Decision | Rationale |
|---|---|---|
| D1 | Dual-store in AppState | Priority engine speaks `Catalog`-adjacent types; storing both avoids lossy back-projection |
| D2 | `survey_for_map` pure extraction | Zero behavioural divergence from CLI — same logic, different call site |
| D3 | D3 for actionability, Graphviz DOT for semantic | Two different visualization problems. D3 sugiyama handles layered DAGs cleanly; Graphviz DOT handles hierarchical relation webs. Separate renderers, same app shell |
| D4 | Frontend is render-only | No JS reimplementation of eligibility/blocking/ordering. d3-dag computes only (x,y) coordinates from the server-supplied graph. Rank, actionability, and edge semantics are server-side |
| D5 | `all=false` by default on survey endpoint | Matches CLI default. `?all=true` is a future addition |
| D6 | View toggle not persisted | Keeps scope small. Follow-up can add `localStorage` |
| D7 | `<script>` tag for D3 vendor (not ES module) | Matches existing vendor pattern (`markdown-it.min.js`, `purify.min.js`). ES5 IIFE convention preserved |
| D8 | `CatalogGraph` is deliberate redundancy | Storing alongside `Catalog` avoids per-request projection cost on the hot `/api/graph` read path |
| D9 | `DataStores` wrapper under single `RwLock` | Atomic refresh — no window where a reader sees a fresh catalog but stale priority_graph. All handlers and refresh use `state.stores` consistently |
| D10 | `survey_for_map` preserves promoted-backlog exclusion | `!channels::promoted(&g, k)` filter exactly matches CLI `survey` default behaviour |
| D11 | Actionability graph returned as explicit `ActionabilityView` (nodes + edges) | The frontend receives the complete graph structure: nodes with server-computed ranks, `needs` and `after` edges. No reconstruction or rank computation in JS |
| D12 | CSS variables for node/edge colours | Dark-theme compatible (OQ-3 resolved). JS reads computed styles; no hardcoded colour values |

## 8. Vendor assets

Vendored JavaScript libraries under `web/map/vendor/`:

| File | Source | License |
|---|---|---|
| `d3.v7.min.js` | https://d3js.org/d3.v7.min.js | ISC |
| `d3-dag.min.js` | https://unpkg.com/d3-dag@1/build/d3-dag.min.js (UMD) | MIT |

A `web/map/vendor/README.md` records for each file:
- exact version
- source URL
- SHA-256 checksum
- license
- regeneration command (e.g. `curl -Lo d3.v7.min.js https://d3js.org/d3.v7.min.js`)

These files are served via the existing `/vendor/{*path}` route
(`assets::serve_embedded`). They are loaded via `<script>` tags in `index.html`,
matching the existing `markdown-it` + `purify` vendor pattern.

## 9. Verification alignment

| What | How |
|---|---|
| Survey endpoint returns canonical data | Backend integration tests: seeded graph → actionability graph JSON matches expected nodes + edges |
| Rank is computed server-side | Test: chain A→B→C → rank(A)=0, rank(B)=1, rank(C)=2 |
| Needs/after edges present in JSON | Test: dep overlay edges → `kind: "needs"`, seq overlay → `kind: "after"` |
| Actionability view renders correct dep ranks | Manual verification: D3 graph shows nodes in topological layers (rank 0 at top, blocked items below their blockers) |
| View toggle switches renderer | Manual verification: click toggle → graph area contains D3 SVG, click back → contains Graphviz SVG |
| Kind filter split works | Manual verification: uncheck ISS → ISS nodes absent from entity list |
| Non-work entity shows message, table shows entity info | Manual verification: focus ADR-001, toggle to actionability → inline message in graph area, relationship table shows entity details from `graph.nodes`, entity list unchanged |
| Refresh flushes cache | Integration test: POST refresh → actionabilityView null → re-fetched |
| Priority engine not reimplemented in JS | Code review: no eligibility/blocking/dep-rank logic in web/map/ beyond consuming actionability graph JSON; `priority.js` only calls `d3-dag` layout + SVG rendering |
| Colour dark-theme compatibility | Manual verification: switch to dark theme → CSS variables resolve correctly |

## 10. Open questions

- **OQ-1** (RESOLVED): `d3-dag` is a separate npm package (`npm i d3-dag`), not
  bundled in d3 v7. Loaded via `<script>` tag from vendored file, it extends the
  `d3` global. Two vendor files needed: `d3.v7.min.js` + `d3-dag` UMD bundle.
- **OQ-2**: Should `GET /api/survey` accept `?all=true` in this slice or defer?
  Defer — no known use case yet for terminal items in the web UI.
- **OQ-3** (RESOLVED): D3 SVG colours use CSS variables (`--priority-*-bg`,
  `--priority-*-edge`) so dark theme works automatically. No hardcoded colour
  values in JS.
