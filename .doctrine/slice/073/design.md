# SL-073 Design: Doctrine Map Frontend

## Hard Contracts

These are binding constraints for implementation. Every module and phase must
satisfy them; the acceptance checklist is gated by them.

- **Project-authored doctrine content is untrusted display input.** DOT strings
  are quoted/escaped before Graphviz; SVG is sanitized before injection; Markdown
  is rendered with raw HTML disabled then sanitized.
- **DOT node statement keys MUST be canonical entity refs.** Display label MAY
  differ, but SVG handler identity extraction depends on `<title>` preserving the
  DOT node key. The `<g class="node"><title>` element is the sole identity
  extraction point — never parse `<text>` content.
- **Edge `(source, label, target)` tuples are canonical-unique.** The catalog
  may not contain duplicate semantic edges — coalesce during normalization if
  the server ever produces them (defensive: the SL-072 server does not today).
  Edge ID `e_source_label_target` is thus durable across refreshes. If multiple
  origins later need distinct provenance, the ID formula will include
  `origin_file`; the current design does not need this complexity.
- **Explicit search navigation MUST return null on miss** — no fallback to first
  node. Bootstrap and hash repair MAY fall back (resolveFocus). Search Enter uses
  a separate `findFocus` path.
- **Depth is valid in the closed range 0..=3 across CLI, URL, state, and
  model.** Depth 0 means "show only the focused node" (focus node + no
  neighbourhood edges).
- **Async graph render and markdown render MUST use stale-response guards.**
  A monotonically increasing `graphRenderSeq` token gates DOT/SVG injection;
  a focus-id comparison gates Markdown injection.
- **Kind filtering is a list/table filter, not a graph filter.** The sidebar
  label reads "List/table filter"; the SVG always shows the full neighbourhood.
  Search Enter-to-focus bypasses the filter by design.
- **Frontend modules ship as separate files** loaded via `<script>` tags in
  `index.html` (dependency order: `api.js`, `model.js`, `dot.js`, `router.js`,
  `render.js`, `app.js`). No ES module loader, no build step.
- **`web/map/` must never contain secrets, local config, test fixtures with
  private paths, source maps with local absolute paths, or generated dev
  artifacts.** Only committed, intentional static files. `.eslintrc.json` is
  a dev-only tooling file that `RustEmbed` will embed (the `#[folder = "web/map/"]`
  glob has no exclusion mechanism); it is harmless — served at
  `/assets/.eslintrc.json` on loopback, no secrets — and not worth a build-script
  workaround.

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
  graphRenderSeq: 0,       // monotonically increasing; guards stale SVG injection
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
// All api functions return Promises. Errors reject with ApiError instances:
//   class ApiError extends Error {
//     constructor(message, { status, body, endpoint }) { ... }
//   }
// Render states branch on error.status, not substring matching.

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

model.findFocus(query, graph)
  // Like resolveFocus but returns null on miss — no fallback.
  // Used by search Enter (explicit navigation intent).

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
  // label is the canonical entity ref (id), not the display title.
  //   The display title appears in the hover pane; the node label is ID-only.
  //   The DOT node key is the canonical ref, so Graphviz `<title>` preserves it.
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
  // Neighbourhood edges (from current focus neighbourhood), filtered by
  // source kind when kindFilter is active.
  // Columns: src_id | src_title | label | tgt_id | tgt_title
  // Src/tgt IDs are clickable (setFocus).

render.renderEdgeDetail(container, edge)
  // Reached via clicking an edge ID in the relationship table.
  // Columns: field | value (metadata table)
  // Fields: edge id, source (clickable → focus), label, target (clickable → focus),
  //   origin_file
  // Trivial (~30 LOC) and useful for provenance inspection; included in this slice
  // rather than deferred because the hash model already supports it.

render.renderError(container, error)
  // Error state: message + optional DOT fallback.
```

## 4. SVG Click & Hover Handling

SVG text from `/api/dot/svg` is sanitized through DOMPurify with the SVG profile
(`USE_PROFILES: {svg: true}`), then injected as inline DOM. DOMPurify will strip
nothing from clean Graphviz output but defends against future SVG injection vectors
(e.g. a user-editable DOT feature). Project-authored catalog text (entity IDs,
titles, statuses, relation labels) is treated as untrusted display input.

### Stale-render guard

```js
function renderGraph(neighbourhood, focusId, depth) {
  state.graphRenderSeq += 1;
  var seq = state.graphRenderSeq;
  var dotText = dot.graphToDot(neighbourhood, focusId, depth);
  api.renderDot(dotText).then(function(svg) {
    if (seq !== state.graphRenderSeq) return;  // stale — discard
    injectAndWire(svg, neighbourhood);
  });
}
```

Refresh also increments `graphRenderSeq`, invalidating any in-flight DOT render.

### Post-injection wiring:

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

The sidebar filter is labeled **"List/table filter"** in the UI — it is not a
graph filter. The SVG always shows the full neighbourhood regardless of filter
state. Filtering affects only the sidebar entity list and the relationship table.

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
It only affects the sidebar list and the relationship table.

Relationship table filtering: the table shows neighbourhood edges (from the
current focus neighbourhood). When kindFilter is active, only edges whose
*source* node passes the filter are shown — outgoing edges from visible
entities regardless of target kind (e.g. SL→ADR edges appear when SL is
checked even if ADR is not).

The graph SVG always shows the full neighbourhood — filtering the SVG nodes
would require DOT regeneration and is a separate concern (deferred).

Acceptance case: uncheck Governance, focus SL-072 (governed_by ADR-001).
Verify ADR-001 remains in the SVG but not in the sidebar.

## 7. Search

Single input at top of sidebar. On keystroke:

1. Live-filter the entity list in the sidebar (substring match on id + title,
   case-insensitive). Text filter applies *after* the kind filter — search
   matches the full corpus but only displays results that also pass the
   current kind filter.
2. On Enter: run `model.findFocus(query, state.graph)` — returns null on
   miss. If non-null and different from current focus, navigate to it.
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

### Link policy

- **External links** (URLs starting with `http://` or `https://`): open in a new
  tab via `target="_blank"` with `rel="noopener noreferrer"`. Applied during
  DOM post-processing after sanitization — add the attributes to `<a>` elements
  whose `href` starts with `http`.
- **Relative links**: stripped (href removed, text preserved as `<span>`).
  Relative links in project Markdown point to local filesystem paths that don't
  resolve in the browser; displaying them as dead links is more confusing than
  removing them.
- **Anchor links** (`#fragment-only`): preserved — intra-document navigation.

### Cache and error states

Markdown cache:
- Populated on fetch; cleared on refresh (`POST /api/refresh`).
- REQ entities return 501 → display "Markdown not implemented for requirements"
  styled as an info message, not an error.
- Stale-request guard: compare resolved id against `state.focusId`. If the
  focus changed while the fetch was in flight, discard the response.

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
| **Stale DOT render** | Focus/depth change while DOT is in-flight | graphRenderSeq mismatch → discard SVG, no DOM mutation |
| **Refresh** | User clicks refresh | Increment graphRenderSeq (kills in-flight renders), clear markdown cache, re-fetch graph, re-resolve focus, re-render |

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
  with `background: var(--kind-{PREFIX})`. Text colour assigned per fill at
  implementation time: white for dark fills (purple, red, blue, rust, violet),
  dark for light fills (orange, amber, green, teal, grey). Sidebar pills and
  graph node labels are tested separately — they use different text-on-fill
  combinations.
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
      #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(0..=3))]
      depth: u8,
  }

  pub(crate) async fn run_serve(path: Option<PathBuf>, args: MapServeArgs) -> anyhow::Result<()> {
-     let root = crate::root::find(path, &crate::root::default_markers())?;
+     let root = crate::root::find(args.path.or(path), &crate::root::default_markers())?;
      // …
  }
```

## 13. Test Strategy

### Pure JS unit tests

The normalization, BFS, search resolution, DOT generation, routing, and
filtering logic is testable without a browser. Add a minimal `web/map/test.html`
that loads the JS files and runs assertions in the console. No external test
runner, no Node dependency — just script tags and `console.assert`.

Cover these pure functions:
- `dot.dotQuote` — DOT escaping edge cases (quotes, backslashes, newlines)
- `model.normalizeGraph` — duplicate edge coalescing, unresolved edge filtering,
  edge ID generation, node Map population
- `model.neighbourhood(depth=0/1/2/3)` — BFS correctness at each depth,
  disconnected nodes, cyclic graphs
- `model.resolveFocus` — null, exact, loose canonical, title match, substring,
  fallback
- `model.findFocus` — same resolution but returns null on miss
- `router.parseHash` / `router.buildHash` — round-trip for all view/id/depth combos
- `model.searchFilter` — substring match, case-insensitive, sorted output
- `model.kinds` — count and sort by kind prefix

### Rust test

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
- An entity with raw HTML in `.md` (`<script>alert(1)</script>`)
- An entity with DOT-unsafe characters in title (quotes, backslashes)
- Multiple kinds for filter testing

### Acceptance checklist

1. `doctrine map serve --open --focus SL-072 --depth 2` opens `#/focus/SL-072?depth=2`
2. `--depth 0` shows only the focused node (no neighbourhood edges)
3. Graph renders with kind-coloured nodes, SL-072 highlighted as focus
4. SVG contains `<g class="node"><title>SL-072</title>` for SL-072; clicking it navigates
5. Hovering a node shows detail pane; node highlights in SVG
6. Clicking a node changes focus, updates graph, markdown, relationship table
7. Depth selector (0-3) updates neighbourhood and hash
8. Search bar live-filters entity list on keystroke
9. Enter on search with valid ref navigates; with nonsense query shows "No match"
   inline and preserves current focus (no fallback to first node)
10. Kind filter (labeled "List/table filter") show/hide entities in sidebar list
    and relationship table; SVG unchanged
11. Uncheck Governance, focus SL-072 → ADR-001 remains in SVG, absent from sidebar
12. Markdown renders below graph with safe HTML output
13. Raw HTML in markdown source does not execute
14. REQ entity shows "Markdown not implemented" info, not a broken state
15. Refresh clears markdown cache, reloads graph, preserves focus
16. Rapid focus changes during DOT render → only the last SVG appears (no stale flash)
17. `--path` flag serves from specified directory
18. Light/dark theme follows system preference
19. Edge detail page (`#/edge/e_…`) opens from relationship table edge ID click,
    shows metadata table with source/target links

## 14. Design Decisions

| Decision | Rationale |
|---|---|
| Inline SVG (not `<object>` blob) | Loopback-only. Project-authored catalog text is treated as untrusted display input — DOT strings are quoted/escaped, SVG is sanitized, Markdown is sanitized. Inline enables click/hover handlers without postMessage bridging |
| ID-only DOT labels | Clean graph; title/status in hover pane. Multi-line labels deferred as togglable option |
| Per-kind colour palette | Visual discrimination of entity kinds. Palette chosen for contrast; implementation assigns text colour per fill (dark text on light fills, white on dark). Sidebar pills and graph node labels are tested separately |
| Kind filter in sidebar only ("List/table filter") | Full neighbourhood always in SVG; filtering only sidebar + relationship table avoids DOT regeneration on filter toggle |
| Live search filter + Enter to focus | Exploration (filter) and navigation (findFocus, no-fallback) in one input |
| Hover pane below graph | More immediate and legible than browser tooltips; fixed position avoids layout shift |
| Semantic CSS, no tailwind | User preference; future htmx migration; hand-editable |
| No new Rust deps | Frontend-only slice; `--path` flag reuses existing CLI machinery |
| Edge detail page included | Trivial (~30 LOC); already in hash model; reached via edge ID click in relationship table; useful for provenance inspection |
| JSDoc types + strict eslint, no TypeScript | Low-ceremony correctness: `@type` on normalization/focus/neighbourhood functions, eslint for globals/hoisting/equality bugs. SPA is a stepping stone to htmx — full TS migration can happen later in an hour if the SPA survives the cutover |
| Frontend modules as separate files | `api.js`, `model.js`, `dot.js`, `router.js`, `render.js`, `app.js` loaded via `<script>` tags in dependency order. No ES module loader, no build step. Clean separation for future htmx migration |
| Edge (source,label,target) unique-tuple contract | Coalesce duplicates during normalization; edge ID `e_source_label_target` is durable. If provenance splitting is needed later, add `origin_file` to the formula |

## 15. Design Revision Notes

Integrated adversarial review feedback (2026-06-16):
- Hard Contracts section added — binding constraints for implementation.
- Security rationale revised: "no attacker input path" → "project-authored
  content is untrusted display input".
- DOT node identity invariant made explicit: DOT node key MUST be canonical ref.
- Edge ID durability strengthened by declaring `(source,label,target)` tuple
  uniqueness canonical — coalesce duplicates during normalization.
- Search miss semantics corrected: `findFocus` (null on miss) for Enter,
  `resolveFocus` (with fallback) for bootstrap/hash repair.
- Stale-response guards added for DOT/SVG render (graphRenderSeq token).
- Kind filter labeled "List/table filter" in UI; relationship table scope
  clarified to neighbourhood edges with source-kind filtering.
- Depth resolved to 0..=3 everywhere (CLI, state, model, acceptance).
- Error model: `ApiError` class with status codes, not substring matching.
- Markdown link policy added: external in new tab, relative stripped.
- Colour accessibility claim tightened: palette chosen for contrast;
  text colour assigned per fill at implementation time.
- Frontend module strategy: separate files loaded via `<script>` tags.
- Edge detail page reachability defined (edge ID click in relationship table).
- Test strategy expanded: pure JS unit tests in `web/map/test.html`.
- `web/map/` hygiene rule added.

## 16. Open Questions

- **Multi-line DOT labels**: Deferred as a togglable option. May not work well on crowded graphs.
- **Graphviz unavailability UX**: DOT source fallback shown in `<pre>`. Could be prettier but low priority — Graphviz is the expected runtime dep.
- **Edge hover in SVG**: Deferred. Mapping SVG edge elements to relationship table rows adds complexity for low payoff — the table already shows all visible edges.
- **Future htmx migration**: JS modules keep rendering logic in `render.*` functions that return DOM elements, not innerHTML strings — easier to migrate to server-side templates later.
- **TypeScript migration**: Deferred. If the SPA survives the htmx cutover, converting to `.ts` is an hour's work given the existing JSDoc type annotations. Not worth the upfront ceremony for a disposable bridge.
