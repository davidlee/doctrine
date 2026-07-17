# SPEC-025: Web explorer

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

The web explorer is the mechanism that turns doctrine's authored corpus into
a browsable, editable local web application: `doctrine map serve` /
`doctrine onboard` start a loopback-only HTTP server (`src/map_server/`,
wired by `src/commands/map.rs`) that serves an embedded single-page frontend
(`web/map/src/`) over three independent read/write surfaces — the
presentation catalog graph, the actionability (priority) graph, and
per-entity concept maps. It is a **container** in the C4 sense: a whole
runtime component (Rust HTTP server + embedded TypeScript SPA) sitting
beside, and consuming, the graph engine container (**SPEC-001**) and the
relation contract (**SPEC-018**) rather than re-implementing either. This
spec fixes the durable architecture — component boundaries, the wire
contracts between server and frontend, and the invariants those boundaries
hold — not any single UI change or feature.

## 2. Scope

In scope: the `map_server` module (`routes`, `state`, `assets`, `markdown`,
`shell`, `error`, `open`) and its CLI wiring (`commands/map.rs`); the three
graph/view builders it serves — `catalog::graph::CatalogGraph` (presentation
projection), `priority::graph::PriorityGraph` via
`priority::surface::survey_view_for_map` (the actionability view), and
`concept_map`'s DSL model (`parse_dsl`/`check`/`get_dsl`/`set_dsl` and the
edit functions) — strictly at the point each is read or mutated through this
server's HTTP surface; the JSON wire contracts those endpoints emit and
accept; and the `web/map/src/` frontend (state, router, model, api, render,
search, priority, concept-map, dot, svg, viewport, zoompan) that consumes
them.

Out of scope: the graph engine's internal mechanics (cordage core, the
policy/adapter split, channel semantics) — owned by SPEC-001 and consumed
here only through `PriorityGraph`/`survey_view_for_map`; the relation
authoring model and vocabulary (`RELATION_RULES`) — owned by SPEC-018 and
consumed here only through the already-hydrated `Catalog`/`CatalogGraph`;
the entity engine and corpus scan upstream of `catalog::hydrate::Catalog`
(SPEC-004 territory); the concept-map entity's own CLI verbs (`new`, `list`,
`show`) beyond the HTTP read/mutate surface; and any single-change UI/UX
decision (colours, layout, copy).

## 3. Principles

- **Three graph builders, one HTTP surface, never merged.** The
  presentation graph (`/api/graph`), the actionability graph
  (`/api/survey`), and a concept map (`/api/concept-map/{id}`) are three
  independently-fetchable, independently-shaped resources built by three
  separate pure functions (`CatalogGraph::from_catalog`,
  `survey_view_for_map`, `concept_map::parse_dsl`). Nothing on the server
  reconciles them into a single graph shape; the frontend fetches each on
  its own schedule.
- **Route handlers stay engine-tier (ADR-001).** `routes.rs`'s module
  comment states the discipline explicitly: thin wrappers over
  `catalog`/`priority`/`concept_map`/`assets`/`markdown`/`shell`, no
  duplicated graph policy or entity semantics in a handler body.
- **One lock guards one atomic generation.** `AppState.stores` is a single
  `Arc<RwLock<DataStores>>` holding `catalog`, `priority_graph`, and `graph`
  together (SL-089 D9); `POST /api/refresh` rebuilds all three and replaces
  them in one write, so a reader never observes a fresh catalog paired with
  a stale priority graph or vice versa.
- **Rendering is delegated, never reimplemented.** The server does not
  generate SVG. It shells out to the local `dot` binary behind an
  injectable `DotRenderer` trait (`RealDotRenderer` in production,
  `FakeDotRenderer` in tests), bounded by a body-size cap and a process
  timeout.
- **DOT is authored client-side; the server is a text-in/bytes-out
  boundary.** `web/map/src/dot.ts` builds DOT source for both the semantic
  graph and concept-map diagrams entirely in TypeScript; the only thing
  that crosses the wire to `/api/dot/svg` is finished DOT text, and the
  only thing that comes back is SVG bytes.
- **Concept-map storage duality is inherited, not resolved here.** The map
  explorer's concept-map endpoints read and write only the DSL text field
  (`concept_map::get_dsl`/`set_dsl`); they are unaware of the parallel
  `[[relation]]` edge store the same entity can also carry (ISS-041, §8).
- **The frontend has one mutable state singleton.** `state.ts` is the sole
  place application state lives; `router.ts`, `model.ts`, `dot.ts`, and
  `viewport.ts` are DOM-free pure functions over wire payloads or that
  state, while `render.ts`, `search.ts`, `priority.ts`, `concept-map.ts`,
  and `svg.ts` are the impure layer touching the DOM. `app.ts` is the
  composition root wiring them together.
- **Concept-map writes are optimistic-concurrency-safe, never
  lock-based.** A mutation carries the SHA-256 of the DSL text it was read
  against; the server refuses (`StaleConceptMap`) if the on-disk hash has
  since changed, rather than serialising writers.
- **Loopback-only is a security property, not a feature.** The listener
  binds `127.0.0.1` unconditionally; `--open` only ever opens a local
  browser tab, it never changes what the server binds to.

## 4. Requirements

`DataStores` is replaced only as a whole — `catalog`, `priority_graph`, and
`graph` are never mutated independently once built; every refresh
reconstructs and swaps all three under one write-lock acquisition.
`/api/dot/svg` enforces a 1 MiB request-body ceiling
(`shell::DOT_BODY_LIMIT`) via `axum::DefaultBodyLimit`, scoped to that route
only; the `dot` child process is bounded by a 10-second wait timeout and is
killed on drop. `CatalogGraph::from_catalog` is pure — no disk reads, no
`cordage` dependency — so the presentation projection is unit-testable
without a filesystem. `entity_markdown` gates every disk read on prior
membership in the in-memory graph snapshot: a canonical id or memory uid
must resolve to a node in `stores.graph.nodes` before the handler touches
the filesystem, so a deleted or unknown entity 404s without probing disk.
Concept-map DSL-editing functions (`add_edge_to_dsl`, `remove_edge_from_dsl`,
`rename_node_in_dsl`, `relabel_edge_in_dsl`, `rename_node_occurrence_in_dsl`,
`relabel_rel_all_in_dsl`) operate purely on DSL text; `mutate_concept_map` is
the sole place that touches the filesystem (`read_concept_map`,
`fsutil::write_atomic`), so the edit algebra itself stays disk-free. Assets
are embedded at build time via `RustEmbed` over `web/map/dist/`
(`assets::Assets`); the server ships with no separate static-file deploy
step and no runtime dependency on the frontend source tree. The listener
always binds `Ipv4Addr::LOCALHOST`; no configuration flag widens the bind
address.

## 5. Success Measures

`doctrine map serve` binds and serves the index page and every embedded
asset (JS/CSS/SVG/JSON/WOFF2, content-typed by `assets::content_type_for`)
with no separate install or build step at run time. `GET /api/health`
reports `dot.ok`/`dot.version` and `graph.ok` independently, so an operator
or the frontend itself can detect a missing Graphviz binary and degrade the
semantic/concept-map views without the whole server failing. An oversized
DOT body is rejected before it reaches the `dot` process, and a wedged `dot`
process is bounded to a 10-second failure rather than hanging the request.
`POST /api/refresh` gives every subsequent `/api/graph` or `/api/survey`
caller a same-generation view of the corpus — no client-observable
in-between state where nodes and priority scores disagree. Two concurrent
editors of the same concept map can never silently overwrite one another:
a stale write is refused, never merged or dropped silently.

## 6. Behaviour

On start-up (`map_server::serve`), the shell hydrates a `Catalog` from disk,
builds a `PriorityGraph` from the same root, projects a `CatalogGraph` from
the catalog, and assembles them into `DataStores` behind one `RwLock`
inside `AppState`; it then binds the loopback listener and starts the axum
`Router`. The router exposes: `GET /` (embedded `index.html`); `GET
/assets/{*path}` and `/vendor/{*path}` (embedded static assets); `GET
/api/health` (dot + graph liveness); `GET /api/graph` (a clone of the
current `CatalogGraph`, serialised as JSON: a `nodes` map keyed by
canonical id or memory uid, a flat `edges` list carrying a
`Validated`/`Raw` label and a `Resolved`/absent target, and a top-level
`units` block); `GET /api/survey` (the actionability view built fresh from
`priority_graph` on every request — kind, status, actionability class,
score, rank, and direct blockers per node, plus a typed edge list); `POST
/api/refresh` (full rescan + atomic store swap); `POST /api/dot/svg` (DOT
text in, SVG bytes out, via the injected `DotRenderer`); `GET
/api/entity/{id}/markdown` (resolves a canonical ref or memory uid against
the live graph snapshot, then reads the entity's `.md` body from disk); and
`GET`/`POST /api/concept-map/{id}` (read returns parsed nodes, edges, and
diagnostics plus a DSL content hash; write applies one typed mutation
against a required base hash, persists via atomic file write, and returns
the freshly re-parsed map).

On the frontend, `app.ts` reads the URL hash through `router.ts`
(`#/focus/<id>[?depth=N][&cmFocus=<key>]` or `#/edge/<id>`), fetches the
graph and/or survey JSON through `api.ts`, normalises it into `state.graph`
via `model.ts` (`Map`-based nodes/edges/incoming/outgoing indices), and
renders. The **semantic** view and concept-map diagrams share one pipeline:
`dot.ts` builds DOT text from the normalised graph or a concept map's
parsed nodes/edges, `api.ts` posts it to `/api/dot/svg`, and the returned
SVG is injected into the DOM, after which `svg.ts` wires click/hover
handlers by reading each Graphviz `<g class="node">`'s `<title>` (the
carrier of the real entity id, distinct from the visible label) and
`viewport.ts`/`zoompan.ts` fit and pan/zoom it. The **actionability** view
is a separate pipeline: `priority.ts` lays out the `ActionabilityView` JSON
directly with `d3-dag` (`graphStratify` + `sugiyama`) and its own
`d3-zoom` instance, with no DOT round-trip and no server-side rendering
dependency. Concept-map edits build a typed mutation
(`add_edge`/`remove_edge`/`rename_node`/`relabel_edge`/
`rename_node_occurrence`/`relabel_rel_all`), attach the last-seen DSL hash,
and POST to `/api/concept-map/{id}`; a `409`-class `StaleConceptMap`
response is the client's signal to refetch before retrying.

## 7. Verification

The Rust route surface is exercised through `map_server::tests::test_app`,
which assembles a full `AppState` (real catalog + priority graph hydrated
from a fixture root, `FakeDotRenderer` injected in place of the real
Graphviz process) and drives the axum `Router` directly, so the handler
suite runs without a `dot` binary on the test host and without a live TCP
bind. The frontend is verified by `vitest` over the modules in
`web/map/src/`, with `state.ts`'s module comment noting that
`model.test.ts` substitutes a scoped mock of the `AppState` shape for
isolated testing; `bun run build` chains `typecheck`, `lint`
(`eslint --max-warnings=0`), `test`, and `vite build`, so a frontend change
cannot land without all four passing. Cross-boundary correctness — that a
DOT string built by `dot.ts` round-trips through the real `dot` binary to
valid SVG, and that `injectHitRects`/`extractNodeId` correctly recover ids
from Graphviz's actual `<title>` output — is exercised by hand and by the
project's `just gate`/`just check` posture rather than by a dedicated
contract test; no test today asserts the `/api/graph` and `/api/survey`
JSON shapes against the frontend's `RawGraph`/`ActionabilityView` types
directly, so the two sides are kept in sync by convention and code review,
not by a generated or shared schema.

## 8. Open Questions

- **ISS-041** (closed, wont-do at the relation-contract layer) documented
  that `RELATION_RULES` declares concept-map `contextualizes` edges as
  writable into `[[relation]]` via `doctrine link`, while
  `catalog::scan::outbound_for` returns an empty edge set for the `CM`
  source kind — so those edges are writable but invisible to every read
  path, including this server's `/api/graph`. The map explorer's own
  concept-map reader never looks at `[[relation]]` either; it reads only
  the DSL string. Two disconnected concept-map edge stores exist today,
  and this spec does not choose between them — it only records that the
  explorer's read/write surface is scoped to the DSL half.
- **IDE-015** (open idea) — "bridge concept map to relation graph": whether
  a concept map's DSL edges should ever be projected into the priority
  graph (so a concept map could feed `next`/`survey`, or the semantic
  graph could render a concept map's edges inline) is unresolved. Today
  the concept-map explorer and the semantic/actionability explorers are
  fully separate data domains sharing only the frontend shell and the SVG
  render/interaction plumbing.
- Whether `/api/refresh` should ever become incremental rather than a full
  rescan is inherited from SPEC-001's v1 full-recompute posture (H1) —
  not a decision local to this spec, and not revisited here.
- No shared schema (generated types, OpenAPI, or a contract test) currently
  keeps the Rust wire types (`CatalogGraph`, `ActionabilityView`, the
  concept-map JSON shape) and the frontend's hand-written `types.ts`
  mirrors in lockstep; whether that gap is worth closing is open.
