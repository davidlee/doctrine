# SL-073 Design: Doctrine Map Frontend

## 1. Architecture & Module Layout

### Tier placement (ADR-001)

```
Command tier:
  src/commands/map.rs   → add --path flag (IMP-079)

Engine tier:
  src/map_server/*      → unchanged from SL-072
  web/map/              → embedded static frontend (RustEmbed)
    index.html          → app shell
    app.js              → SPA (state, api, model, dot, router, render)
    style.css           → semantic layout, light/dark theme, kind pills
    vendor/             → markdown-it, DOMPurify, github-markdown.css (unchanged)
```

No new Rust modules. No new Rust dependencies. No server route changes. The
frontend is a pure consumer of the SL-072 API surface.

Note: `.eslintrc.json` is a dev-only tooling file that `RustEmbed` will
include in the binary (the `#[folder = "web/map/"]` glob has no exclusion
mechanism). It is harmless — served at `/assets/.eslintrc.json` on loopback,
no secrets — and not worth a build-script workaround.

The user intends a future migration to htmx + minijinja templates once UX
settles. This design keeps markup semantic and separable — JS modules do not
sprawl into HTML generation where a server-side template would later be cleaner.
`renderShell` returns DOM structure; it doesn't interleave data-fetching with
element creation.

### Data flow (unchanged from SL-072)

```
Browser                          axum router              Catalog / FS
  │                                  │                        │
  ├─ GET /api/graph ────────────────►│                        │
  │◄── CatalogGraph JSON ───────────┤                        │
  │                                  │                        │
  ├─ POST /api/refresh ─────────────►│                        │
  │◄── {"ok":true} ─────────────────┤                        │
  │                                  │                        │
  ├─ GET /api/entity/{id}/markdown ─►│                        │
  │◄── text/markdown ───────────────┤                        │
  │                                  │                        │
  ├─ POST /api/dot/svg ─────────────►│                        │
  │◄── image/svg+xml ───────────────┤                        │
```

## 2. Core Types (JS)

### State

```js
var state = {
  graphRaw: null,          // raw CatalogGraph JSON from /api/graph
  graph: {                 // normalized (populated by normalizeGraph)
    nodes: new Map(),      // id → NormalizedNode
    edges: [],             // NormalizedEdge[]
    incoming: new Map(),   // id → NormalizedEdge[]
    outgoing: new Map(),   // id → NormalizedEdge[]
    edgeById: new Map(),   // edgeId → NormalizedEdge
  },
  focusId: null,           // canonical ref of focused entity
  depth: 1,                // neighbourhood depth (0-3, clamped)
  markdownCache: new Map(),// id → markdown text; cleared on refresh
  dotAvailable: false,     // from /api/health dot.ok
  hoveredId: null,         // currently hovered node id (for detail pane)
  kindFilter: null,        // Set of kind prefixes, or null = all
};
```

### NormalizedNode

```js
{
  id: "SL-072",                  // canonical ref
  title: "Doctrine Map Server",  // from CatalogEntity.title
  status: "done",                // from CatalogEntity.status
  kindPrefix: "SL",              // from EntityKey.prefix
  kindLabel: "slice",            // human-readable kind
  raw: { /* original CatalogEntity */ }
}
```

### NormalizedEdge

```js
{
  id: "e_SL-072_governed_by_ADR-001",  // durable: source_label_target
  source: "SL-072",     // source canonical ref
  target: "ADR-001",    // target canonical ref
  label: "governed_by", // relation label
  resolved: true,       // always true (unresolved filtered during normalize)
  raw: { /* original CatalogEdge */ }
}
```

### Kind colour palette

Grouped for visual discrimination. Colours are accessible on both light and dark
backgrounds (verified AA contrast for normal text on all fills; see palette below)
contrast for graph nodes where fill carries the signal).

```
SL          #4A90D9  blue         — slices (the primary change unit)
ADR  POL    #7B4FBF  purple       — governance decisions/policies
STD         #9B59B6  light purple — standards
PRD  SPEC   #E67E22  orange       — product & tech specs
REQ         #F39C12  amber        — requirements
ISS  IMP    #C0392B  dark red     — issues & improvements
CHR  RSK    #C0392B  dark red     — chores & risks
IDE         #27AE60  green        — ideas
RV          #1ABC9C  teal         — reviews
REC         #95A5A6  grey         — knowledge records
ASM  DEC    #3498DB  light blue   — assumptions & decisions
QUE  CON    #8E44AD  violet       — questions & constraints
REV         #A04000  rust         — revisions
```

**Graphviz node style**: fill colour from palette, font colour `#ffffff` for
dark fills (purple, red, blue, rust, violet), `#222222` for light fills
(orange, amber, green, teal, grey). Shape: `box,rounded` for slices/specs,
`box` for everything else.

**Sidebar pills**: small rounded inline spans with `background: var(--kind-{prefix})`,
white text, compact padding. Used in entity list items beside the ID.

## 3. Module Contracts

### api — HTTP layer

```js
// All return Promises. Errors surface as rejected promises with message
// strings suitable for display.

api.fetchGraph()        // GET /api/graph → CatalogGraph JSON
api.refreshGraph()      // POST /api/refresh → {ok:true}
api.renderDot(dotText)  // POST /api/dot/svg → SVG text
api.fetchMarkdown(id)   // GET /api/entity/{id}/markdown → markdown text
api.fetchHealth()       // GET /api/health → {dot:{ok,version}, graph:{ok}}
```

### model — data normalization & query

```js
model.normalizeGraph(raw)
  // raw CatalogGraph → populates state.graph (nodes Map, edges[], incoming/outgoing/edgeById Maps)
  // Edge id: "e_" + source + "_" + label + "_" + target (durable across refreshes)
  // Only resolved edges included; unresolved skipped silently

model.resolveFocus(query, graph)
  // query: string | null → canonical id string
  // Resolution order (Python prototype behaviour):
  //   1. null/empty → first sorted node id
  //   2. Exact canonical match (case-insensitive)
  //   3. Loose canonical: "SL71", "SL 71", "sl-71" → "SL-071"
  //   4. Exact title match (case-insensitive)
  //   5. Substring in id, title, status, or kind (shortest match wins)
  //   6. Fallback → first sorted node id

model.neighbourhood(focusId, depth, graph)
  // BFS from focusId through incoming + outgoing edges
  // depth clamped [0, 3]
  // → { nodes: Set<string>, edges: NormalizedEdge[] }

model.kinds(nodes)
  // → Map<prefix, count> sorted by prefix

model.searchFilter(query, graph)
  // Live filter: substring match on id or title (case-insensitive)
  // → NormalizedNode[] sorted by id
```

### dot — DOT generation

```js
dot.dotQuote(value)     // escape for DOT string literals
dot.nodeAttrs(node, focusId, depth)
  // → { label, fillcolor, fontcolor, shape, penwidth, tooltip, URL }
  // URL attribute: "#/focus/{id}?depth={depth}" — preserved in SVG for
  //   reference but real navigation via click handlers

dot.edgeAttrs(edge, depth)
  // → { label, tooltip }

dot.graphToDot(neighbourhood, focusId, depth)
  // { nodes: Set, edges: Edge[] } → DOT string
  // Sorted output for determinism
  // Graph attrs: rankdir=LR, bgcolor="transparent" (inherits page background for
  //   dark mode compatibility), nodesep=0.45, ranksep=0.8
```

### router — hash routing

```js
router.parseHash()
  // window.location.hash → { view: 'focus'|'edge', id, depth }
  // Default: { view: 'focus', id: null, depth: 1 }

router.buildHash(view, id, depth)
  // → "#/focus/SL-072" or "#/focus/SL-072?depth=2" or "#/edge/e17?depth=2"

router.setFocus(id, depth)    // update hash + trigger render
router.setEdge(edgeId, depth) // update hash + trigger render
```

### render — DOM construction

```js
render.renderShell(container)
  // Builds top-level layout: sidebar + main (graph area + markdown area).
  // Called once on bootstrap, then individual panels update in place.

render.renderSidebar(graph, focusId, kindFilter, onFilterChange, onFocusClick, onRefresh)
  // Search bar, type filter checkboxes, compact entity list.

render.renderGraphPane(container, neighbourhood, focusId, depth, onDepthChange)
  // DOT generation → POST /api/dot/svg → inline SVG with click/hover handlers.

render.renderHoverPane(container, node)
  // Below the graph: title, status pill, kind badge for hovered node.

render.renderMarkdownPane(container, id)
  // Fetch + render sanitized markdown.
  // Stale-request guard: if focus changes while fetch is in-flight, discard
  //   the response (compare id against current state.focusId on resolution).
  // Cache: populated on successful fetch, cleared on refresh. Cache hit skips
  //   fetch entirely for the current session.

render.renderRelationshipTable(container, edges)
  // src_id | src_title | label | tgt_id | tgt_title
  // Src/tgt IDs are clickable (setFocus).

render.renderEdgeDetail(container, edge)
  // edge id | source (clickable) | label | target (clickable) | origin_file
  // Navigation links to focus source / focus target.

render.renderError(container, error)
  // Error state: message + optional DOT fallback.
```

## 4. SVG Click & Hover Handling

SVG text from `/api/dot/svg` is sanitized through DOMPurify with the SVG profile
(`USE_PROFILES: {svg: true}`), then injected as inline DOM. DOMPurify will strip
nothing from clean Graphviz output but provides pipeline symmetry with Markdown
and defends against future SVG injection vectors (e.g. a user-editable DOT feature).
Post-injection:

```js
function wireSvgHandlers(svgEl, neighbourhood) {
  // For each <g class="node"> in the SVG:
  //   - Extract node id from <title> text (Graphviz puts the node id there)
  //   - attach onclick → router.setFocus(id, state.depth)
  //   - attach mouseenter → state.hoveredId = id; renderHoverPane()
  //   - attach mouseleave → state.hoveredId = null; renderHoverPane()
  //   - add .doctrine-node class for CSS targeting
  // Edge hover (highlight corresponding table row) deferred — low payoff
  //   vs complexity of mapping SVG edge elements to relationship rows.
  //
  // For the focused node:
  //   - add .doctrine-node--focus class (extra border/fill styling)
  //
  // For the hovered node:
  //   - add .doctrine-node--hover class (highlight fill/stroke)
}
```

Graphviz `<g class="node">` elements contain a `<title>` with the node id.
This is the reliable extraction point. No textContent parsing of `<text>`
elements.

## 5. Hover Detail Pane

A fixed-height panel between the graph SVG and the relationship table:

```
┌────────────────────────────────────────────┐
│ SL-072: Doctrine Map Server                │
│ slice · done                               │
└────────────────────────────────────────────┘
```

Empty state: "Hover a node for details" in muted text.

## 6. Kind Filter

Sidebar checkboxes, one row per kind group:

```
☑ Slices (SL)      ☐ Governance (ADR/POL/STD)
☑ Specs (PRD/SPEC) ☐ Requirements (REQ)
☐ Backlog           ☐ Reviews (RV)
☐ Memory            ☐ Revisions (REV)
```

Toggle behaviour:
- All checked = show all (including unclassified kinds)
- Some unchecked = filter entity list to only entities whose kind prefix matches
  a checked group. Relationship table filters by *source* kind — shows outgoing
  edges from visible entities regardless of target kind (e.g. SL→ADR edges
  appear when SL is checked even if ADR is not).
- Filter is live — no submit button

Interaction with focus: filtering does not change the focused entity or graph.
It only affects the sidebar list and the relationship table below the graph.

The graph SVG always shows the full neighbourhood — filtering the SVG nodes
would require DOT regeneration and is a separate concern (deferred).

## 7. Search

Single input at top of sidebar. On keystroke:

1. Live-filter the entity list in the sidebar (substring match on id + title,
   case-insensitive). Text filter applies *after* the kind filter — search
   matches the full corpus but only displays results that also pass the
   current kind filter.
2. On Enter: run `model.resolveFocus(query)` and navigate to the best match.
3. On Escape: clear the input, restore full list.
4. If Enter resolves to the current focus, no navigation occurs.
5. If Enter finds no match, show inline "No match for '{query}'" message in
   the sidebar; preserve current focus.
6. Enter-to-focus bypasses the kind filter by design — it is a navigation
   intent, not a filter operation. The user may land on an entity outside
   their current filter group.

## 8. Markdown Rendering

```js
function renderMarkdown(text) {
  var raw = md.render(text);            // markdown-it, html: false
  return DOMPurify.sanitize(raw);       // belt-and-suspenders sanitize
}
```

Markdown cache:
- Populated on fetch; cleared on refresh (`POST /api/refresh`).
- REQ entities return 501 → display "Markdown not implemented for requirements"
  styled as an info message, not an error.

## 9. UI Layout

```text
┌──────────────────────┬──────────────────────────────────────────┐
│ Sidebar              │ Main                                     │
│                      │                                          │
│ [Search input      ] │ SL-072: Doctrine Map Server              │
│                      │ slice · done                             │
│ ☑ Slices  ☐ Gov    │                                          │
│ ☑ Specs   ☐ Reqs   │ ┌──────────────────────────────────────┐ │
│                      │ │                                      │ │
│ Entity list          │ │          Graphviz SVG                │ │
│ (compact, filtered)  │ │          (inline)                    │ │
│                      │ │                                      │ │
│ SL-071 · Scanner     │ └──────────────────────────────────────┘ │
│ SL-072 · Map Server  │                                          │
│ SL-073 · Frontend    │ ┌ Hover detail pane ──────────────────┐ │
│ ADR-001 · Layering   │ │ ADR-001: Module layering             │ │
│ …                    │ │ adr · accepted                       │ │
│                      │ └──────────────────────────────────────┘ │
│ [Refresh]            │                                          │
│                      │ Depth: [1] [2] [3]                       │
│                      │                                          │
│                      │ ┌ Relationship table ─────────────────┐ │
│                      │ │ src_id │ src_title │ label │ tgt_id… │ │
│                      │ │ SL-072 │ Map Srv   │ gov.. │ ADR-001 │ │
│                      │ └──────────────────────────────────────┘ │
│                      │                                          │
│                      │ ┌ Markdown ───────────────────────────┐ │
│                      │ │ # SL-072 Design: Doctrine Map Server │ │
│                      │ │                                      │ │
│                      │ │ ## 1. Architecture & Module Layout…  │ │
│                      │ └──────────────────────────────────────┘ │
└──────────────────────┴──────────────────────────────────────────┘
```

## 10. States

| State | Trigger | Behaviour |
|---|---|---|
| **Loading** | Bootstrap, no graph data yet | Show "Loading graph…" in main area |
| **Server unavailable** | `/api/graph` or `/api/health` fails at bootstrap | Show "Could not reach the Doctrine server. Is `doctrine map serve` running?" with retry button |
| **Empty graph** | `/api/graph` returns `{nodes:{},edges:[]}` | Show "No entities found. Try refreshing." with refresh button |
| **Focused graph** | Normal operation | SVG + hover pane + relationship table + markdown |
| **Graphviz unavailable** | `/api/health` dot.ok = false, or `/api/dot/svg` returns 503 | Show "Graphviz not available" message + DOT source in `<pre>` as fallback |
| **Markdown 404** | Entity exists but no `.md` file | Show "No markdown body for {id}" muted |
| **Markdown 501** | REQ entity | Show "Markdown not implemented for requirements" info message |
| **Markdown 500** | Server error | Show "Failed to load markdown: {message}" error |
| **Search miss** | Enter on query with no resolve match | Show inline message, preserve current focus |
| **Refresh** | User clicks refresh | Clear markdown cache, re-fetch graph, re-resolve focus, re-render |

## 11. CSS Approach

- Semantic HTML, no utility-class framework, no tailwind.
- CSS custom properties for the colour system:
  ```css
  :root {
    --kind-SL: #4A90D9;
    --kind-ADR: #7B4FBF;
    /* … all kind colours … */
    --bg: #ffffff;
    --fg: #1a1a1a;
    --muted: #6b6b6b;
    --border: #e0e0e0;
    --hover-bg: #f5f5f5;
  }
  @media (prefers-color-scheme: dark) {
    :root {
      --bg: #1a1a1a;
      --fg: #e0e0e0;
      --muted: #9b9b9b;
      --border: #333333;
      --hover-bg: #2a2a2a;
    }
  }
  ```
- Layout: CSS Grid (`grid-template-columns: 280px 1fr`), not flexbox hacks.
- Kind pills: `display: inline-block; border-radius: 3px; padding: 0 4px; font-size: 0.8em;`
  with `background: var(--kind-{PREFIX})` and white/dark text per contrast.
- SVG node highlights: CSS classes `.doctrine-node--focus` and `.doctrine-node--hover`
  applied to `<g>` elements, using `filter: brightness(1.2)` or stroke changes.

## 12. CLI Change (IMP-079)

```diff
// src/commands/map.rs

  #[derive(clap::Args)]
  pub(crate) struct MapServeArgs {
      #[arg(long, default_value = "0")]
      port: u16,
+     #[arg(long)]
+     path: Option<PathBuf>,
      #[arg(long)]
      open: bool,
      #[arg(long, value_parser = validate_focus)]
      focus: Option<String>,
      #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=3))]
      depth: u8,
  }

  pub(crate) async fn run_serve(path: Option<PathBuf>, args: MapServeArgs) -> anyhow::Result<()> {
-     let root = crate::root::find(path, &crate::root::default_markers())?;
+     let root = crate::root::find(args.path.or(path), &crate::root::default_markers())?;
      // …
  }
```

## 13. Test Strategy

No automated browser tests. Manual acceptance against the criteria in the scope
document. The Rust test suite already covers:

- All API routes (SL-072 route integration tests)
- Asset serving (embedded index.html, vendor files)
- Error mapping (MapServerError → status codes)
- URL construction (map_url pure tests)

The only new Rust test: `--path` flag routes to `root::find` correctly.
SL-072 has no CLI arg-parse tests for `map serve`; add a test in
`src/commands/map.rs` (or `tests/` if command tests live there):
  - `map_serve_path_flag_passed_to_root_find` — parse `--path /tmp/foo`,
    verify the value reaches `root::find`. Existing `root::find` correctness
    is covered by its own tests; only the CLI wire-up is new.

### Manual acceptance fixture

Build a test corpus with:
- SL-071, SL-072, SL-073 (slices with edges between them)
- ADR-001 (governed_by from slices)
- PRD-013 (requirement reconciliation spec)
- REQ-001 (for 501 markdown)
- An entity with rich `.md` body (SL-072 design.md)
- An entity with empty `.md` body
- Multiple kinds for filter testing

### Acceptance checklist

1. `doctrine map serve --open --focus SL-072 --depth 2` opens `#/focus/SL-072?depth=2`
2. Graph renders with kind-coloured nodes, SL-072 highlighted as focus
3. Hovering a node shows detail pane; node highlights in SVG
4. Clicking a node changes focus, updates graph, markdown, relationship table
5. Depth selector (0-3) updates neighbourhood and hash
6. Search bar live-filters entity list on keystroke
7. Enter on search resolves and navigates to best match
8. Kind filter checkboxes show/hide entities in sidebar list and relationship table
9. Markdown renders below graph with safe HTML output
10. Raw HTML in markdown source does not execute (test: `<script>alert(1)</script>` in an `.md`)
11. REQ entity shows "Markdown not implemented" info, not a broken state
12. Refresh clears markdown cache, reloads graph, preserves focus
13. `--path` flag serves from specified directory
14. Light/dark theme follows system preference

## 14. Design Decisions

| Decision | Rationale |
|---|---|
| Inline SVG (not `<object>` blob) | Loopback-only, self-generated content — no attacker input path. Inline enables click/hover handlers without postMessage bridging |
| ID-only DOT labels | Clean graph; title/status in hover pane. Multi-line labels deferred as togglable option |
| Per-kind colour palette | Visual discrimination of entity kinds; all fills verified AA contrast for normal white/dark text |
| Kind filter in sidebar only | Full neighbourhood always in SVG; filtering only sidebar + relationship table avoids DOT regeneration on filter toggle |
| Live search filter + Enter to focus | Exploration (filter) and navigation (resolve) in one input |
| Hover pane below graph | More immediate and legible than browser tooltips; fixed position avoids layout shift |
| Semantic CSS, no tailwind | User preference; future htmx migration; hand-editable |
| No new Rust deps | Frontend-only slice; `--path` flag reuses existing CLI machinery |
| Edge detail page included | Trivial (~30 LOC); already in hash model; useful for provenance inspection |
| JSDoc types + strict eslint, no TypeScript | Low-ceremony correctness: `@type` on normalization/focus/neighbourhood functions, eslint for globals/hoisting/equality bugs. SPA is a stepping stone to htmx — full TS migration can happen later in an hour if the SPA survives the cutover |

## 15. Open Questions

- **Multi-line DOT labels**: Deferred as a togglable option. May not work well on crowded graphs.
- **Graphviz unavailability UX**: DOT source fallback shown in `<pre>`. Could be prettier but low priority — Graphviz is the expected runtime dep.
- **Edge detail page styling**: Included but minimal (metadata table). May warrant richer UX later.
- **Edge hover in SVG**: Deferred. Mapping SVG edge elements to relationship table rows adds complexity for low payoff — the table already shows all visible edges.
- **Future htmx migration**: JS modules keep rendering logic in `render.*` functions that return DOM elements, not innerHTML strings — easier to migrate to server-side templates later.
- **TypeScript migration**: Deferred. If the SPA survives the htmx cutover, converting to `.ts` is an hour's work given the existing JSDoc type annotations. Not worth the upfront ceremony for a disposable bridge.
