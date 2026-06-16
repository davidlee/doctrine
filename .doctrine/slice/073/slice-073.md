# Doctrine Map Frontend: interactive browser explorer

## Context

SL-072 delivered the map server — loopback HTTP surface (`doctrine map serve`) with
`/api/graph`, `/api/refresh`, `/api/dot/svg`, `/api/entity/{id}/markdown`, and
`/api/health`. It embedded a **placeholder** browser app (`web/map/`) just enough
to exercise the routes manually. The design explicitly left "full interactive map
UX" out of scope for SL-072.

A Python prototype (`doctrine_graph_server.py`) demonstrates the desired product
shape: focus resolution, N-hop neighbourhood projection, Graphviz SVG rendering,
hash-routed navigation, relationship tables, and entity detail pages — all as
server-side HTML. This slice ports that interactive experience into the browser
as a static SPA, consuming the Rust API surface SL-072 built.

The user's design brief (conversation 2026-06-15) provides the authoritative
architecture and behaviour specification. This scope document records the
decisions from that brief.

## Scope & Objectives

Replace the `web/map/` placeholder with a real embedded static frontend that:

1. Loads the `CatalogGraph` JSON from `GET /api/graph`.
2. Normalizes nodes/edges into internal maps with incoming/outgoing indexes.
3. Resolves a focus entity by canonical ref, loose ref, or title substring
   (matching the Python prototype's `Graph.resolve` behaviour).
4. Computes N-hop neighbourhood projection (incoming + outgoing edges).
5. Builds DOT text client-side and renders SVG via `POST /api/dot/svg`.
6. Injects SVG as inline DOM with click/hover handlers. Inline is safe here:
   loopback-only server, self-generated content, no attacker input path.
7. Supports clickable nodes for navigation, hover detail pane showing
   title/status/kind, and hover highlight on the SVG node.
8. Provides per-kind colour palette for visual entity-type discrimination.
9. Renders focused entity Markdown body via `GET /api/entity/{id}/markdown`,
   sanitized through markdown-it (html:false) + DOMPurify.
10. Provides depth selector (0-3), search bar with live sidebar filtering
    plus Enter-to-focus, kind-type filter checkboxes, node list sidebar,
    relationship table (src_id | src_title | label | tgt_id | tgt_title).
11. Includes edge detail page (`#/edge/e17`): metadata table for a single
    relationship.
12. Handles error states: Graphviz unavailable (DOT fallback), REQ markdown 501,
    empty graph, malformed focus, stale markdown request discard.
13. Preserves focus across refresh (`POST /api/refresh`).
14. Light/dark theme via `prefers-color-scheme` with CSS custom properties.

### Affected surface

- `web/map/index.html` — replace placeholder with real app shell
- `web/map/app.js` — replace placeholder with full interactive app
- `web/map/style.css` — replace placeholder with layout styling + light/dark theme
- `web/map/.eslintrc.json` — strict lint config (browser env, es5, eqeqeq,
  no-unused-vars, no-implicit-globals)
- `web/map/vendor/` — markdown-it, DOMPurify, github-markdown.css (unchanged)
- `src/commands/map.rs` — add `--path` flag (IMP-079)

No new Rust dependencies. No new server routes. No asset-embedding changes.

The `RustEmbed` folder in `src/map_server/assets.rs` already embeds `web/map/`;
no asset-embedding changes needed.

## Non-Goals

- Server-side HTML pages (Python prototype's model). The Rust server remains
  API-only + static assets. The browser owns routing and rendering.
- JS build chain (npm, TypeScript, bundler). Vanilla JS, no build step.
- Non-loopback hosting or collaboration semantics.
- Graph semantics in the frontend. The frontend owns only visible projection;
  canonical graph semantics stay in `CatalogGraph`.
- Stats/diagnostics page (optional nicety, defer).


## Architecture

```
web/map/
  index.html          — app shell, loads vendor + app.js
  app.js              — SPA: state, api, model, dot, router, render
  style.css           — layout: sidebar + main (graph + markdown)
  vendor/             — markdown-it.min.js, purify.min.js, github-markdown.css
```

### Modules in app.js

```
state       — graphRaw, graph (normalized), focusId, depth, hoveredId,
              kindFilter, markdownCache
api         — fetchGraph, refreshGraph, renderDot, fetchMarkdown, fetchHealth
model       — normalizeGraph, resolveFocus, neighbourhood, searchFilter, kinds
dot         — dotQuote, nodeAttrs, edgeAttrs, graphToDot
router      — parseHash, buildHash, setFocus, setEdge
render      — renderShell, renderSidebar, renderGraphPane, renderHoverPane,
              renderMarkdownPane, renderRelationshipTable,
              renderEdgeDetail, renderError
```

### Data contract

Consume raw `CatalogGraph` JSON (`{ nodes: {...}, edges: [...] }`).
Normalize client-side:

```js
function normalizeGraph(raw) {
  // → { nodes: Map<string, Node>, edges: Edge[], incoming: Map<string, Edge[]>,
  //     outgoing: Map<string, Edge[]>, edgeById: Map<string, Edge> }
}
```

Normalized node: `{ id, title, status, kindLabel, rawKey, raw }`
Normalized edge: `{ id, source, target, label, resolved, raw }`

Only resolved edges rendered in SVG. Unresolved edges skipped (diagnostics later).

### Navigation model

Hash routes only:

```
#/                       default focus
#/focus/SL-072           focus with default depth
#/focus/SL-072?depth=2   focus with explicit depth
#/edge/e17               relationship detail
```

### SVG handling

SVG injected as inline DOM. Security: loopback-only server, self-generated DOT
from local catalog data, no attacker-controlled input path. Post-injection,
click/hover handlers are attached to `<g class="node">` elements. Node id
extracted from the `<title>` child (standard Graphviz SVG output).

Graphviz DOT uses `bgcolor="transparent"` so the graph background inherits the
page theme in both light and dark modes.

### Markdown security

`markdown-it` with `html: false`, output sanitized through `DOMPurify.sanitize()`
before `innerHTML`.

### UI layout

```
┌────────────────────┬──────────────────────────────────────┐
│ Sidebar            │ Main                                 │
│                    │                                      │
│ Search/focus       │ Focus title/status                   │
│ Depth selector     │ Graph SVG (inline + handlers)        │
│ Refresh/health     │ Visible relationships table          │
│ Node list          │ Markdown panel                       │
└────────────────────┴──────────────────────────────────────┘
```

### States handled

1. Loading — fetching /api/graph
2. Server unavailable — /api/graph failed at bootstrap; show error + retry button
3. Empty graph — no nodes; show refresh button
4. Focused graph — SVG + hover pane + relationship table + markdown
5. Entity markdown — rendered below graph, stale-request guarded
6. Graphviz unavailable — error message + DOT source fallback
7. Markdown unavailable — structured message (404 / 501 REQ / 500)
8. Search miss — inline message in sidebar, preserve current focus
9. Hover — detail pane below graph shows title/status/kind of hovered node

## Risks & Assumptions

- **Inline SVG security**: Loopback-only, self-generated DOT from local catalog.
  No attacker-controlled input. The SVG DOM surface is equivalent to the
  JSON graph the browser already holds in memory.
- **markdown-it size**: already vendored in SL-072 (124 KiB). Acceptable.
- **No JS test harness**: manual acceptance verification. SL-072 established this
  as the project's posture for the browser surface.
- **Graphviz behaviour stability**: DOT generation targets standard Graphviz.
  Node `<title>` extraction relies on stable `<g class="node">` structure
  in Graphviz SVG output — well-established, low risk.

## Verification / Closure Intent

Manual acceptance against the acceptance criteria in the design brief:

1. `doctrine map serve --open --focus PRD-013 --depth 2` opens correct hash route
2. Browser fetches graph, normalizes, renders focused SVG with kind-coloured nodes
3. SVG node click changes focus without page reload; edge detail URLs are durable across refresh
4. Hovering a node shows detail pane below graph; node highlights in SVG
5. Bootstrap failure (server unreachable) shows error message with retry
6. Search live-filters entity list on keystroke; Enter resolves and navigates
7. Search accepts canonical ID, loose forms (`SL71`, `SL 71`), title substring
8. Depth selector (0-3) updates neighbourhood and hash route
9. Kind filter checkboxes show/hide entities in sidebar and relationship table
10. Focused entity Markdown renders sanitized below graph
11. Raw HTML in markdown source does not execute
12. SVG sanitized through DOMPurify SVG profile, then injected as inline DOM
13. Refresh clears markdown cache, reloads graph, preserves focus; edge detail URLs remain stable
14. Graphviz unavailable → readable error + DOT source fallback
15. REQ markdown 501 → "Markdown not implemented for requirements" info message
16. Light/dark theme follows system preference
17. `--path` flag serves from specified directory
18. No new Rust dependencies (`cargo tree` diff empty vs SL-072)

## Follow-Ups

- Stats/diagnostics page (deferred)
- Multi-line DOT labels as togglable option (deferred)
- Edge hover in SVG highlighting corresponding relationship table row (deferred)
- Browser test harness (deferred until JS testing need is proven)
- Future htmx + minijinja migration once UX settles
