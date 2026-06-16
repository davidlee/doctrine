# Load concept maps into the Map Explorer and ship a web authoring surface

## Context

Doctrine has concept maps (`CM-NNN`) — a DSL-driven relationship-diagram entity
kind stored under `.doctrine/concept-map/` with TOML metadata, a structured
DSL block (`Source > relation > Target` lines), and Markdown prose. The CLI
(`doctrine concept-map new|list|show|check|add|remove|rename-node|export`)
works, but concept maps are invisible to the Map Explorer (`doctrine map
serve`) — they are not registered in `integrity::KINDS`, so the catalog scan
never picks them up, and they never appear in the entity sidebar, the graph,
or any web surface.

The Map Explorer (SL-072 server + SL-073 frontend) currently serves only the
entity-relation catalog graph. Concept maps are a separate universe with no
browser visibility and no authoring path beyond the CLI.

The user wants concept maps browsable alongside entities in the same web UI
("a concept map is just an entity"), with a low-friction web authoring surface,
and long-term the ability for concept map nodes to reference real entities.

## Scope & Objectives

1. **Register CM in the catalog** — add `CM` to `integrity::KINDS`, the
   `outbound_for` dispatch, and the catalog scan so concept maps appear as
   entity nodes in the catalog graph, the entity sidebar, and entity search.

2. **Concept map detail view** — when a user clicks a CM entity in the
   sidebar, render its internal concept map (the DSL parsed into nodes/edges)
   as a second diagram pane, alongside or replacing the entity-relation graph.

3. **Web authoring interface** — add/edit/remove edges and rename nodes
   directly in the browser. Add form in the sidebar or overlay to author a
   new edge (`Source > relation > Target`). Remove button on selected edges.
   Rename inline on node labels. Each mutation calls the backend to rewrite
   the DSL block in the concept map's TOML file.

4. **Uniform treatment** — concept maps appear in the entity sidebar with
   their `CM` kind pill, are searchable by title/ref, and participate in the
   same navigation model (hash-routed focus, depth). The only difference: a
   focused CM renders its DSL diagram instead of the entity-relation graph.

## Affected Surface

- `src/integrity.rs` — add `CM` to `KINDS`
- `src/catalog/scan.rs` — `outbound_for` arm for `CM` (empty, like REQ/KNOWLEDGE)
- `src/map_server/routes.rs` — new API routes: `GET /api/concept-maps` (list),
  `GET /api/concept-map/:id` (nodes + edges from DSL), `POST /api/concept-map/:id/edge`
  (add edge), `DELETE /api/concept-map/:id/edge` (remove edge),
  `POST /api/concept-map/:id/node` (rename node)
- `src/map_server/state.rs` — no structural changes (concept map data is read
  from disk per-request via `concept_map` module)
- `web/map/app.js` — concept map diagram pane, authoring UI (add edge form,
  remove edge button, rename node), toggle between entity graph and CM diagram
- `web/map/model.js` — concept map data normalization, CM edge/node types
- `web/map/style.css` — authoring form styles, CM diagram pane, edge interaction
- `web/map/index.html` — CM pane container, authoring UI elements
- `web/map/dot.js` — no changes (DOT generation for concept maps is isomorphic)

## Non-Goals

- Entity-ref support in CM node labels (node label `SL-001` does not link to
  the real entity — that is a follow-up slice)
- Drag-and-drop or visual graph editing (text-based authoring only)
- Multi-user or concurrent editing (single-user loopback server)
- Fullscreen markdown for concept maps (already covered by SL-075)
- Concept map relations as tier-1 `[[relation]]` edges (the DSL is the
  concept map's internal edge vocabulary; cross-kind structural relations are
  follow-up)
- New concept map creation from the web (CLI `concept-map new` is the
  creation path; follow-up may add a web creation button)

## Risks

- **DSL write-back**: Mutating the DSL block in the TOML file must preserve
  all other TOML fields, comments, and formatting. The current `concept_map`
  module edits the DSL as a raw string block with line operations — this is
  already tested and works for the CLI `add`/`remove`/`rename-node` verbs.
  Reusing those same functions from the web route avoids a parallel write path.
- **TOCTOU on edit**: Two browser tabs mutating the same concept map could
  race. Acceptable for a single-user loopback tool; mutex on the write path
  is proportional (file lock or in-memory lock per CM id).
- **Catalog scan performance**: Adding one more kind to the scan is
  negligible; concept map directories are small and few.

## Verification

- `just check` — root package tests pass
- `just gate` — workspace clean
- Smoke test: `doctrine concept-map new "Test Map"` then `doctrine map serve
  --open --focus CM-001` — concept map appears in sidebar and diagram renders
- Authoring smoke: add edge via web UI, verify DSL updated; remove edge;
  rename node; verify via `doctrine concept-map show CM-001`
- `doctrine concept-map check CM-001` — clean after web edits
- Catalog scan includes CM entities: `doctrine inspect CM-001` works

## Follow-Ups

- Entity-ref support: CM node labels that match canonical entity refs
  (`SL-001`, `ADR-003`) are styled as entity links and resolve to the real
  entity on click
- Web-based concept map creation (new CM from the browser)
- Visual graph editing (drag nodes, draw edges)
- Concept map relations as tier-1 `[[relation]]` edges (link a CM to an ADR,
  spec, or slice via `doctrine link`)
