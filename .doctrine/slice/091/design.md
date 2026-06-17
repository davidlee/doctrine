# SL-091 Design: Frontend dev server with TypeScript, HMR, and hot reload

## Architecture

**Dev topology:**

```
Browser :5173  →  Vite dev server (HMR, TS→JS via esbuild)
                  │  /src/*, /vendor/*  →  serves from web/map/
                  │  /api/*             →  proxy → Rust :8080
                  │  /                  →  index.html (Vite-injected HMR client)
```

Zero Rust changes for dev. Vite is a dev server + TS transpiler + bundler — no framework (no JSX, React, Svelte, Vue). Plain DOM manipulation with types and `import`/`export`.

**Release topology:**

```
Browser :8080  →  Rust map server
                  │  /assets/*, /vendor/*  →  rust-embed from web/map/dist/
                  │  /api/*                 →  handlers (unchanged)
                  │  /                      →  web/map/dist/index.html
```

`bun run build` → `tsc --noEmit` + `vite build` → production output in `web/map/dist/`.

## File layout

```
web/map/
  package.json           ← bun, devDeps: vite, typescript; deps: d3, d3-dag, markdown-it, dompurify
  tsconfig.json          ← strict, moduleResolution "bundler", target es2020
  vite.config.ts         ← proxy /api → localhost:8080, root ., build → dist/
  index.html             ← updated: single <script type="module" src="/src/app.ts">
  test.html              ← updated: imports src/*.ts via ES modules
  dist/                  ← gitignored, Vite output
  node_modules/          ← gitignored
  public/               ← Vite copies as-is to dist/
    vendor/
      github-markdown.css
  src/
    types.ts             ← shared interfaces
    model.ts             ← state singleton + pure functions (leaf)
    router.ts            ← hash routing (leaf)
    api.ts               ← HTTP layer (leaf)
    dot.ts               ← DOT generation (leaf)
    svg.ts               ← shared SVG DOM manipulation (leaf)
    render.ts            ← entity-graph DOM construction
    search.ts            ← search, filters, depth, refresh wiring
    concept-map.ts       ← CM rendering + editing
    priority.ts          ← D3 force layout for actionability view
    app.ts               ← entry point, imports everything
    style.css            ← moved from web/map/
```

## Rust diff

- `src/map_server/assets.rs`: `#[cfg_attr(debug_assertions, folder = "web/map/")]` +
  `#[cfg_attr(not(debug_assertions), folder = "web/map/dist/")]`.
  Debug builds embed raw source (always present — `cargo test`/`cargo build` work
  without `dist/`). Release builds embed optimized Vite output (requires
  `bun run build` before `cargo build --release`).
- `.gitignore`: add `web/map/dist/`, `web/map/node_modules/`
- No route handler changes — `index()` and `asset()` still serve from `Assets`

## TypeScript migration — module-by-module (leaf → root)

Each module converts from `/* global X */` + `window.X = {}` + IIFE to ES module
`import`/`export`. The `state` object (currently `window.state` in `model.js`)
becomes a named export from `model.ts`. Every module that mutates state imports
it directly.

### types.ts

Shared interfaces extracted from the implicit contracts across the codebase:

```ts
// API shapes
export interface RawGraph {
  nodes: Record<string, RawCatalogNode>;
  edges: RawEdge[];
}

export interface RawCatalogNode {
  title: string;
  status: string;
  kind_label: string;
}

export interface RawEdge {
  source: string;
  label: { Validated?: string; Raw?: string };
  target: { Resolved?: string };
}

export interface ActionabilityView {
  kind: string;
  nodes: ActionabilityNode[];
  edges: ActionabilityEdge[];
}

export interface ActionabilityNode {
  id: string;
  title: string;
  kind: string;
  status: string;
  actionability: string;
  rank: number;
  blockers: string[];
}

export interface ActionabilityEdge {
  source: string;
  target: string;
  kind: string;
}

// Normalized internal types
export interface Graph {
  nodes: Map<string, CatalogNode>;
  edges: Edge[];
  incoming: Map<string, Edge[]>;
  outgoing: Map<string, Edge[]>;
  edgeById: Map<string, Edge>;
}

export interface CatalogNode {
  id: string;
  title: string;
  status: string;
  kindPrefix: string;
  kindLabel: string;
  raw: RawCatalogNode;
}

export interface Edge {
  id: string;
  source: string;
  label: string;
  target: string;
  raw: RawEdge;
}

export interface Route {
  view: 'focus' | 'edge';
  id: string | null;
  depth: number;
  cmFocus: string | null;
}

export interface ConceptMap {
  id: string;
  title: string;
  status: string;
  description: string;
  dslHash: string;
  nodes: CmNode[];
  edges: CmEdge[];
  diagnostics: unknown[];
}

export interface CmNode {
  key: string;
  label: string;
}

export interface CmEdge {
  from_key: string;
  from_label: string;
  rel: string;
  to_key: string;
  to_label: string;
  line?: number;
}

export interface Neighbourhood {
  nodes: Set<string>;
  edges: Edge[];
}

export interface CmNeighbourhood {
  nodes: CmNode[];
  edges: CmEdge[];
}

export interface EditingNode {
  key: string;
  label: string;
}

// Mutable application state
export interface AppState {
  // Graph data
  graphRaw: RawGraph | null;
  graph: Graph;

  // Navigation
  focusId: string | null;
  depth: number;

  // Caches
  markdownCache: Map<string, string>;
  conceptMapCache: Map<string, ConceptMap>;

  // Concept map editing
  editingConceptMap: boolean;
  editingNode: EditingNode | null;
  cmFocusNode: CmNode | null;
  renderedCmFocus: string | null;
  cmCacheMutationSeq: number;
  renderedCmCacheSeq: number;

  // Rendering flags
  dotAvailable: boolean;
  hoveredId: string | null;
  viewMode: 'semantic' | 'actionability';
  actionabilityView: ActionabilityView | null;
  priorityZoomId: string | null;
  kindFilter: Set<string> | null;
  graphRenderSeq: number;
}

// Error type
export class ApiError extends Error {
  status: number;
  body: string;
  endpoint: string;
  constructor(message: string, status: number, body: string, endpoint: string);
}
```

### router.ts

Pure string parsing — no DOM, no state dependency.

```ts
import type { Route } from './types';

export function parseHash(depth: number): Route { ... }
export function buildHash(view: string, id: string, depth: number, cmFocusNode: CmNode | null): string { ... }
export function setFocus(id: string, depth: number): void { ... }
export function setEdge(edgeId: string, depth: number): void { ... }
```

Auxiliary `parseQueryString`, `clampDepth` stay module-private.

### api.ts

HTTP layer — depends only on `types.ts` for `ApiError`, `RawGraph`, `ActionabilityView`,
`ConceptMap`.

```ts
import { ApiError, type RawGraph, type ActionabilityView, type ConceptMap } from './types';

export function fetchGraph(): Promise<RawGraph>;
export function fetchActionabilityGraph(): Promise<ActionabilityView>;
export function refreshGraph(): Promise<{ ok: boolean }>;
export function fetchHealth(): Promise<{ ok: boolean; dot: { ok: boolean; version?: string }; graph: { ok: boolean } }>;
export function renderDot(dotText: string): Promise<string>;
export function fetchMarkdown(id: string): Promise<string>;
export function fetchConceptMap(id: string): Promise<ConceptMap>;
export function mutateConceptMap(id: string, action: string, params: Record<string, string>, baseHash?: string): Promise<unknown>;
```

### dot.ts

Pure text generation — depends on `State` type for `NODE_STYLES`, `EDGE_STYLES`.

```ts
import type { Graph, ConceptMap, CmNeighbourhood } from './types';

export function dotQuote(s: string): string;
export function graphToDot(graph: Graph, focusId: string, nb: Neighbourhood): string;
export function cmGraphToDot(cm: CmNeighbourhood, focusKey: string | null): string;
export const NODE_STYLES: Record<string, { fill: string; font: string; shape: string }>;
export const EDGE_STYLES: Record<string, { color: string; style?: string }>;
```

### svg.ts

DOM manipulation over `<svg>` elements. No state dependency, consumes raw DOM.
Option types (`SvgHandlerOpts`) defined in this module since they are
module-specific, not shared.

```ts
export function injectHitRects(svgEl: SVGSVGElement): void;
export function wireHandlers(svgEl: SVGSVGElement, opts: SvgHandlerOpts): void;
export function applyFocusHighlight(svgEl: SVGSVGElement, focusId: string, prevFocusId: string | null, getTitle: (g: SVGGElement) => string): void;
export function dimLegend(svgEl: SVGSVGElement, edgeLabels: string[]): void;
```

### model.ts

Pure functions for graph normalization + querying, plus the `state` singleton.

```ts
import type { RawGraph, Graph, CatalogNode, Edge, Neighbourhood, ConceptMap, CmNeighbourhood, ActionabilityView, AppState } from './types';

export const state: AppState;

// Normalization
export function normalizeGraph(raw: RawGraph): void;  // mutates state.graph
export function normalizeConceptMap(raw: unknown): ConceptMap;
export function setActionabilityView(view: ActionabilityView): void;  // mutates state.actionabilityView

// Lookup
export function findFocus(query: string | null, graph: Graph): string | null;
export function resolveFocus(query: string | null, graph: Graph): string | null;

// Neighbourhood
export function neighbourhood(focusId: string, depth: number, graph: Graph): Neighbourhood;
export function cmNeighbourhood(cm: ConceptMap, focusKey: string | null, depth: number): CmNeighbourhood;

// Search
export function searchFilter(query: string, graph: Graph): CatalogNode[];

// Kind ordering
export const kindOrder: Record<string, number>;
export function compareNodes(a: CatalogNode, b: CatalogNode): number;
export function compareEdgesBySource(ea: Edge, eb: Edge): number;

// Utilities
export function encodePart(s: string): string;
export function pascalToSnake(s: string): string;
export function buildNodeLabelList(cm: ConceptMap): string[];
export function buildRelLabelList(cm: ConceptMap): string[];
```

### render.ts

DOM construction for entity-graph views. Imports `model`, `dot`, `api`, `svg`.
Module-specific option types (`GraphPaneOpts`, `RelationshipTableOpts`,
`EdgeDetailOpts`, `RenderedElements`) defined here, not in `types.ts`.

```ts
import type { Graph, CatalogNode, Edge, Neighbourhood } from './types';

export let elements: RenderedElements;

export function cacheElements(doc: Document): void;
export function el(tag: string, attrs?: Record<string, string>, children?: (HTMLElement | string)[]): HTMLElement;
export function escapeHtml(s: string): string;
export function escapeAttr(s: string): string;
export function setViewMode(mode: 'entity-graph' | 'actionability' | 'concept-map' | 'edge'): void;

export function entityList(opts: { container: HTMLElement; graph: Graph; query: string; kindFilter: Set<string> | null; focusId: string | null; onFocus: (id: string) => void }): void;
export function graphPane(opts: GraphPaneOpts): void;
export function focusHeader(opts: { container: HTMLElement; focusId: string | null; graph: Graph }): void;
export function hoverPane(opts: { container: HTMLElement; node: CatalogNode | null }): void;
export function relationshipTable(opts: RelationshipTableOpts): void;
export function markdownPane(opts: { container: HTMLElement; id: string; cache: Map<string, string> }): void;
export function edgeDetail(opts: EdgeDetailOpts): void;
```

### search.ts

DOM event wiring for search, filters, depth, refresh. Imports `model`, `render`.

```ts
export function renderFilteredEntities(opts: { list: HTMLElement; graph: Graph; query: string; kindFilter: Set<string> | null; focusId: string | null; onFocus: (id: string) => void }): void;
export function wireFilters(opts: { container: Document | HTMLElement; onChange: (filterSet: Set<string> | null) => void }): void;
export function wireSearch(opts: { input: HTMLInputElement | null; list: HTMLElement; graph: Graph; getFocusId: () => string | null; getKindFilter: () => Set<string> | null; onFocus: (id: string) => void }): void;
export function wireDepthButtons(opts: { container: Document | HTMLElement; onDepthChange: (d: number) => void }): void;
export function wireRefresh(opts: { button: HTMLButtonElement | null; onRefresh: () => void }): void;
export function collectKindFilter(container: Document | HTMLElement): Set<string> | null;
```

### concept-map.ts

Module-specific option types (`CmDiagramOpts`, `CmEdgeTableOpts`,
`CmAddEdgeFormOpts`) defined here.

```ts
import type { ConceptMap, CmNeighbourhood } from './types';

export function renderDiagram(opts: CmDiagramOpts): void;
export function renderEdgeTable(opts: CmEdgeTableOpts): void;
export function renderAddEdgeForm(opts: CmAddEdgeFormOpts): void;
export function renderDiagnostics(opts: { container: HTMLElement; diagnostics: unknown[] }): void;
export function renderEditToggle(opts: { header: HTMLElement; editing: boolean; onToggle: () => void }): void;
```

### priority.ts

D3 force-directed layout for the actionability view. Imports `d3`, `d3-dag`.
Module-specific types (`LayoutNode`, `LayoutEdge`, `PriorityRenderOpts`) defined here.

```ts
import type { ActionabilityView } from './types';

export function layoutGraph(view: ActionabilityView): { nodes: LayoutNode[]; edges: LayoutEdge[] };
export function renderGraph(opts: PriorityRenderOpts): void;
```

### app.ts

Entry point. Imports all modules. `bootstrap()` wires event listeners on
`DOMContentLoaded`, `renderView()` handles hashchange + initial render.

```ts
import { state, normalizeGraph, resolveFocus, setActionabilityView, model } from './model';
import { parseHash, setFocus, buildHash } from './router';
import * as api from './api';
import { cacheElements, graphPane, focusHeader, hoverPane, render, relationshipTable, markdownPane, edgeDetail, escapeHtml, setViewMode } from './render';
import { renderFilteredEntities, wireFilters, wireSearch, wireDepthButtons, wireRefresh } from './search';
import * as cm from './concept-map';
import * as priority from './priority';
import './style.css';

function bootstrap(): void { ... }
function renderView(): void { ... }

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', bootstrap);
} else {
  bootstrap();
}
```

The `state` object is mutated directly by `model.normalizeGraph()`, `model.setActionabilityView()`,
and various handlers in `app.ts`. This preserves the current imperative style.

## Vendor dependencies

| Package     | Current bundle          | TS source              |
|-------------|-------------------------|------------------------|
| d3          | `vendor/d3.v7.min.js`   | `d3` npm + `@types/d3` |
| d3-dag      | `vendor/d3-dag.min.js`  | `d3-dag` npm (TS-first)|
| markdown-it | `vendor/markdown-it.min.js` | `markdown-it` npm + `@types/markdown-it` |
| DOMPurify   | `vendor/purify.min.js`  | `dompurify` npm (v3+ ships types) |
| github-markdown.css | `vendor/github-markdown.css` | moved to `public/vendor/`, copied by Vite's `publicDir` |

D3 v7 is imported as `import * as d3 from 'd3'` (tree-shaken by Vite in prod build).
d3-dag is imported as `import { dagStratify, sugiyama, ... } from 'd3-dag'`.

## Vite config

```ts
export default defineConfig({
  root: '.',
  server: {
    proxy: { '/api': 'http://localhost:8080' }
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: 'index.html'
    }
  },
  publicDir: 'public',  // github-markdown.css lives here, copied as-is to dist/
});
```

## Verification

- `tsc --noEmit` — zero errors at every phase
- `bun run dev` — Vite starts, asset serving + HMR works
- `bun run build` — produces `dist/` with `index.html`, hashed JS/CSS bundles, `vendor/`
- `cargo test -p doctrine` — all map server integration tests pass (debug embed uses `web/map/`)
- `cargo build --release` — succeeds after `bun run build` (embed uses `web/map/dist/`)
- `web/map/test.html` — converted to `<script type="module">` in Phase 10; served via Vite, all tests pass
- Manual smoke: navigate graph, search, filter, toggle views, edit concept maps — identical behaviour
- `just gate` — zero clippy warnings

## Decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Vite dev server, no Rust feature flag | Vite proxies `/api` → Rust; zero Rust dev-mode code. Release: `vite build` → embed `dist/`. One-line `#[folder]` change. |
| D2 | Functional-core, imperative-shell for TS migration | Exported `state` singleton mutates directly (preserves current style). Pure functions take explicit params instead of reading `state` from closure — enables `tsc` to catch signature mismatches. |
| D3 | Module-by-module conversion, leaf → root | Keeps the feedback loop tight. Each module is a small, reviewable diff. Old `.js` files live until their `.ts` replacement is verified. |
| D4 | `bun` as package manager | Already in flake (1.3.13). Faster than npm. `bun run` + `bun add` replace npm equivalents. |
| D5 | d3-dag from npm (not vendored UMD) | d3-dag is TS-first, ships types. `npm install d3-dag` gives full type safety. |
| D6 | No framework — plain DOM | Vite is strictly dev server + TS transpiler + bundler. No JSX, React, Svelte, Vue. Plain `document.querySelector` / `innerHTML` / `addEventListener`. |

## Open questions

None — resolved during design.

## Risks

| Risk | Mitigation |
|------|-----------|
| `d3-dag` API surface differs between vendored UMD and npm version | The vendored bundle is a minified build of d3-dag. Check version match; if different, adapt the priority.ts calls. |
| `dompurify` global vs ES module import | DOMPurify v3+ ships types and supports ES module import. `import DOMPurify from 'dompurify'`. The `window.DOMPurify` global is dropped. |
| `markdown-it` API break between vendored and npm | The vendored bundle is markdown-it v14.x (minified). npm `markdown-it` v14 should be identical. |
| Vite proxy doesn't forward WebSocket upgrades needed for future features | Not needed now. If we later add WebSocket endpoints, add `ws: true` to the proxy config. |
| `test.html` uses bare `<script src="/assets/...">` tags — incompatible with ES modules | During transition, test against old `.js` files + `tsc --noEmit` for `.ts`. Convert `test.html` to `<script type="module">` with `import` statements in Phase 10, served via Vite. |
