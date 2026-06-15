# Doctrine Map Server: local web graph explorer

## Context

Doctrine has a rich entity-relation corpus (slices, ADRs, specs, requirements,
memories, backlog items — linked via outbound TOML relations). Navigating this
graph today requires CLI verbs (`inspect`, `survey`, `next`, `explain`,
`blockers`, `catalog`) — each exposes one projection. There is no visual,
interactive explorer.

The map server adds a small loopback HTTP surface that serves an embedded
browser app over existing Doctrine capabilities. It exposes canonical graph
JSON, entity Markdown retrieval, and a trusted Graphviz rendering bridge for
browser-generated DOT. The browser app owns exploratory presentation and
interaction.

The graph model remains Doctrine-owned. Generic graph mechanics remain
`cordage`-owned. The server must not duplicate or fork graph semantics inside
HTTP route handlers.

Dependencies: SL-071 (catalog) provides the `CatalogGraph` projection with
nodes, edges, and diagnostics — the map server consumes it directly, no
independent corpus walk.

## Scope & Objectives

1. **CLI entry.** `doctrine map serve` with flags: `--port` (default 0 =
   OS-assigned), `--open` (launch browser), `--focus <id>` (initial focused
   entity, validated canonical ID), `--depth <n>` (range 1..=3, default 1).
   Loopback-only binding (127.0.0.1) — no `--host` flag.

2. **Embedded browser app (placeholder).** `web/map/` — `index.html`, `app.js`,
   `style.css`, vendor libraries (markdown-it, DOMPurify, github-markdown.css).
   Embedded via `rust-embed` and served as static assets. Hash-routing so no SPA
   fallback needed. The browser app is a **placeholder shell** in SL-072 — the
   Rust server is the deliverable. Full interactive UX is follow-up.

3. **HTTP routes.** Thin axum handlers over existing Doctrine/cordage seams:
   - `GET /` — serve `index.html`
   - `GET /assets/*` / `GET /vendor/*` — embedded static files
   - `GET /api/health` — liveness + dot availability + graph availability
   - `GET /api/graph` — canonical graph JSON from `CatalogGraph`
   - `POST /api/dot/svg` — browser DOT → `dot -Tsvg` (size-capped, timeout,
     no-shell)
   - `GET /api/entity/{id}/markdown` — Markdown body via existing entity render
     seam

4. **Graphviz bridge.** Fixed-shape `dot -Tsvg` process: pipe stdin, cap body
   size (1 MiB), enforce timeout, kill on timeout, never invoke shell, return
   structured errors.

5. **Testability.** `DotRenderer` trait with production and fake implementations
   (the sole abstraction — graph and markdown endpoints use real catalog + temp
   fixtures). Route tests with `tower::ServiceExt::oneshot`. URL construction
   tested as a pure function.

6. **Layer discipline.** Map server owns only HTTP transport, embedded assets,
   fixed API over Doctrine capabilities, Graphviz process bridge, and Markdown
   byte retrieval. It must not duplicate graph policy, infer relation meaning,
   or become a parallel graph domain model.

## Non-Goals

- Duplicating `cordage` graph semantics
- Duplicating Doctrine relation policy
- Inferring durable relationship meaning from labels in route handlers
- Maintaining a durable graph store
- Implementing a parallel priority/actionability engine
- Owning interactive graph projection policy (browser-owned)
- Rendering Markdown to HTML in Rust for the live browser UI
- Exposing arbitrary shell command execution
- Exposing arbitrary filesystem reads
- A second independent entity kind registry

## Affected Surface

- **New:** `src/commands/map.rs` — CLI entry
- **New:** `src/map_server/` — `mod.rs`, `state.rs`, `routes.rs`, `assets.rs`,
  `shell.rs`, `error.rs`, `open.rs`, `markdown.rs`
- **New:** `web/map/` — `index.html`, `app.js`, `style.css`,
  `vendor/markdown-it.min.js`, `vendor/purify.min.js`,
  `vendor/github-markdown.css`
- **Existing consumers:** `src/catalog/` (graph, hydrate) — read-only
- **Cargo.toml:** uncomment `tokio`, `axum` workspace deps; add `webbrowser`
- **CLI:** `src/main.rs` — new `map` subcommand

## Risks

- **axum dependency weight.** First web framework in the repo. Mitigation:
  axum is already a workspace dep, and this is a loopback-only server —
  limited surface.
- **Graphviz availability.** `dot` may not be installed. Mitigation: health
  check reports unavailability; `/api/dot/svg` returns structured error.
  Nothing breaks.
- **CatalogGraph serialization.** The browser receives raw `CatalogGraph` JSON —
  an internal format that may change with catalog evolution. Mitigation: minimum
  contract test verifies required top-level keys; browser normalizes as needed.
- **REQ markdown deferred.** Requirement entities need parent-spec lookup for
  `.md` path resolution — not implemented in SL-072. Mitigation: returns 501
  explicitly; follow-up slice or catalog-owned helper.
- **Concurrent refresh races.** Two concurrent refreshes may both scan; the later
  write wins. Accepted as eventual consistency — the graph is always a valid
  snapshot, just possibly not the latest.

## Verification / Closure Intent

- `doctrine map serve --open` starts a loopback web app
- Browser fetches `/api/graph` and receives valid JSON
- Browser generates DOT from visible projection, posts to `/api/dot/svg`,
  receives SVG
- Clicking a graph node updates the focused view
- Selecting a node fetches and displays its Markdown via
  `/api/entity/{id}/markdown`
- Rust route handlers contain no duplicated graph policy
- All command execution is fixed-shape, loopback-only, size-capped (middleware
  + handler), timeout-bound, and kill-on-drop guaranteed
- `cargo test` passes with new route tests (including graph contract, URL
  construction, error mapping, ID validation, concurrency edge cases)
- `cargo clippy` zero warnings
- `just gate` passes

## Summary

Add `doctrine map serve` — a loopback HTTP server that embeds a browser app
for interactive exploration of Doctrine's entity-relation graph. The server
serves static assets, exposes a canonical graph JSON endpoint via the
SL-071 catalog, retrieves entity Markdown via the existing render seam, and
safely bridges browser-generated DOT to Graphviz SVG. The browser app owns
exploratory presentation and interaction entirely.

## Follow-Ups

- Neighbourhood queries with configurable depth could be added to `CatalogGraph`
  (deferred per SL-071 D10)
- Entity/edge detail JSON endpoints if graph payload is too large (deferred
  until needed)
- The `catalog` CLI could serve as the health-check graph provider
