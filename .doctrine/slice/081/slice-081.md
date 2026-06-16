# Surface memory entities and their relations in the catalog graph + web explorer

## Context

SL-072 (map server) + SL-073 (map frontend) shipped an interactive web graph
explorer over the Doctrine entity-relation corpus. It consumes `CatalogGraph`
built from the SL-071 catalog scan. The catalog scan walks `integrity::KINDS` —
the numbered-entity kinds (slice, ADR, spec, requirement, backlog, review, REC,
knowledge records, concept map, revision). Memory is explicitly excluded: it is a
*named* kind (uid-based, not numeric), and the `integrity.rs` header states
"Memory is a named kind (`mem_<uid>` dirs, key aliases) with no numeric id, so it
is out of scope here (D-A)."

Result: 567 nodes in the production catalog graph, 0 of them memory entities. The
user has 30+ local memory entities (under `.n/items/`) and ~30 shipped global
masters (under `.n/shipped/`), none visible in the explorer.

Memory TOML carries `[[relation]]` rows in the format spec (`label` + `target`),
but the deserialization struct `RawRelation` is fieldless — serde discards the
keys so no outbound edges are ever read. No memory entity currently has an
authored relation, but the TOML surface is defined and ready.

The catalog scan, hydrate, and graph layers all use `EntityKey { prefix: &str, id:
u32 }` as identity. Memory's uid (`mem_019ecf851db57cf0b32611c2d20ea2ed`) doesn't
fit this shape. The graph projection (`CatalogGraph`) encodes nodes as
`NodeKey::Entity(EntityKey)`, serialized as the canonical ref string — a span that
must accommodate memory.

## Scope & Objectives

1. **Scan memory entities into the catalog.** A separate scan pass over
   `.n/items/` and `.n/shipped/` that collects memory entity metadata (uid,
   title, status, memory_type) — analogous to the `scan_entities` KINDS walk but
   for the named-entity directory shape. Runs alongside the existing KINDS scan
   in `scan_catalog`.

2. **Extend identity to accommodate memory.** `EntityKey` gains a memory variant,
   or a new identity type bridges both numbered and named entities through a
   common serialization surface. The serialized node key for memory entities must
   be the uid string (e.g. `"mem_019ecf851db57cf0b32611c2d20ea2ed"`), matching
   the existing `NodeKey` JSON serialization pattern.

3. **Parse memory relations.** Replace the fieldless `RawRelation` with one that
   reads `label` and `target`. Wire outbound relations from memory entities into
   the catalog edge set, with the same `classify_target` resolution as numbered
   entities (resolved / unresolved ref / unvalidated text).

4. **CatalogGraph projection.** Memory entities appear as `CatalogNode`s with
   `kind_label = "MEM"`. Catalog edges from memory entities use the memory uid as
   the source identity. No new edge target variants needed — memory targets other
   entities via canonical refs (`SL-001`, `ADR-010`), not memory uids.

5. **Frontend compatibility.** The SL-073 frontend already handles any
   `kindLabel` — it maps labels to a colour palette with a fallback. A `"MEM"`
   label renders without frontend changes. Entity detail pages fetch markdown via
   `/api/entity/{id}/markdown` — must accept memory uid as the id parameter.

6. **Markdown route.** The existing `/api/entity/{id}/markdown` handler resolves
   entities by canonical ref via `integrity::parse_canonical_ref`. Memory uids
   don't parse as canonical refs — the handler must accept memory uid as a
   fallback path, reading `memory.md` under `.n/items/{uid}/` or
   `.n/shipped/{uid}/`.

## Non-Goals

- `doctrine link` for memory entities (linking a memory uid as source or target).
  The `link` verb requires a canonical ref; extending it is a separate slice.
- Memory-to-memory relations (memory entities referencing other memory uids in
  `[[relation]]` rows). Only memory → numbered-entity relations are in scope.
- Memory entity aliases / key resolution. Memory identity is uid-only in the
  graph. The `mem.<key>` symlink is not exposed in the catalog.
- Changing the serialization shape of `CatalogGraph` for existing numbered
  entities. Backward compatibility with the frontend is a hard constraint.
- Graphviz DOT generation for memory nodes. The frontend DOT generator already
  handles any node key string; no server-side changes needed.

## Affected Surface

- **`src/memory.rs`** — replace fieldless `RawRelation` with a struct that reads
  `label` + `target`
- **`src/catalog/scan.rs`** — new scan function for memory entities (named
  entity shape, uid identity) + extend `EntityKey`
- **`src/catalog/hydrate.rs`** — `Catalog::from_scanned` accepts memory entities
  + their outbound edges; `classify_target` may need adjustment for memory
  source identity
- **`src/catalog/graph.rs`** — `NodeKey` gets a `Memory(String)` variant;
  `CatalogNode.kind_label` becomes `"MEM"` for memory nodes
- **`src/map_server/routes.rs`** (or `markdown.rs`) — `/api/entity/{id}/markdown`
  accepts memory uid as a fallback after canonical ref parse fails
- **`src/map_server/state.rs`** — `AppState` may need a memory root for markdown
  resolution
- **No changes to `web/map/`** — the frontend handles memory through the
  existing generic node/edge model

## Risks

- **Identity collision.** Memory uids (`mem_019ec...`) won't collide with
  canonical refs (`SL-001`) — disjoint prefixes, no ambiguity. The `NodeKey`
  JSON serialization already produces strings for numbered entities, so
  `"mem_019ec..."` is indistinguishable from a numbered prefix only if a
  numbered kind with prefix `mem_` exists — it doesn't.
- **Markdown route performance.** Reading memory markdown is one extra disk
  probe per entity detail request. 60 memory entities at ~500 bytes each is
  negligible. Cache if needed, but start with direct read.
- **Frontend colour.** The SL-073 colour palette has ~20 entries for known
  kind labels. `"MEM"` hits the fallback path (grey). Acceptable for v1;
  a pleasant colour can be added later.
- **EntityKey type growth.** Adding a variant to `EntityKey` touches every
  match site. Mitigation: keep the change minimal — either a variant or a
  wrapper enum that keeps `EntityKey` unchanged for numbered entities.

## Verification / Closure Intent

- `doctrine catalog graph` (via `inspect` / `survey`) includes memory nodes
  with `kind_label = "MEM"` and correct uid/title/status
- Memory outbound relations appear in the edge list with resolved targets
- `doctrine map serve --open` shows memory nodes in the browser graph with
  hover detail pane, click-to-focus, and relationship table
- Focused memory entity renders its `memory.md` body below the graph
- `/api/graph` JSON includes memory nodes and edges; frontend parses
  successfully
- No errors for memory nodes in the browser console (JS normalization,
  DOT generation, SVG handling)
- `cargo test` passes — existing catalog/graph/map-server tests green;
  new tests cover memory scan, identity, and edge classification
- `cargo clippy` zero warnings
- `just gate` passes

## Summary

Add memory entities (named, uid-based, living under `.n/items/` and
`.n/shipped/`) to the catalog scan, graph projection, and web explorer
pipeline. Extend `EntityKey`/`NodeKey` to carry memory uids alongside
numbered entity keys. Parse memory `[[relation]]` rows into catalog edges.
Accept memory uids in the `/api/entity/{id}/markdown` route. The SL-073
frontend handles memory generically through the existing node/edge model —
no frontend changes required.

## Follow-Ups

- `doctrine link` support for memory uids as source/target
- Memory-to-memory relations (cross-memory edges)
- Memory key aliases as secondary node identity in the graph
- Pleasant colour for `"MEM"` in the frontend palette
- Memory entity search in the SL-073 search bar (currently resolves by
  canonical ref or title substring — memory uids won't match the ref parser)
