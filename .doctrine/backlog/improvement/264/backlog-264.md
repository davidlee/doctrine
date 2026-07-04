# IMP-264: Clickable wikilinks in web map memory view

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Memory bodies rendered in the web map's markdown pane contain `[[mem.key]]`
wikilinks. They render as literal text — a reader can't follow a link to the
referenced memory. Navigation between related memories requires manually finding
the node in the graph.

## Proposed approach (server-side resolution)

Resolve wikilinks in the backend before serving the memory markdown, reusing the
resolver that already exists for the `memory resolve-links` CLI verb.

- **Seam:** `read_memory_markdown()` (`src/map_server/markdown.rs:37`) currently
  reads the `.md` raw. Insert a rewrite pass: `[[mem.key]]` → markdown anchor
  `[label](#/focus/mem_<uid>)`.
- **Reuse:** `resolve_wikilink()` / `extract_wikilinks()` in `src/links.rs`
  already map `[[mem.key]]` → `mem_<uid>` via the key→uid map from
  `memory::collect_all`. CLI-only today; lift into the map route.
- **Unresolved:** `resolve_wikilink` returns `Err(target)` when the key doesn't
  resolve — leave those as literal text (don't emit a dead link).

## Why not the frontend

`/api/graph` node payloads (`CatalogNode`, `src/catalog/graph.rs:33`) carry the
uid but **no name slug**, so the client can't map `[[mem.signpost.x]]` →
`mem_<uid>` without a contract change or a new lookup endpoint. Server-side
sidesteps this — resolver + key→uid map already sit together in `links.rs`.

## Frontend: ~zero change

The resolved anchor works for free:
- Router already parses `#/focus/mem_<32hex>` (`web/map/src/router.ts`).
- `applyLinkPolicy()` (`web/map/src/render.ts:538`) **preserves** `#`-prefixed
  hrefs (strips only non-http/non-`#`). So `[label](#/focus/mem_<uid>)` survives
  DOMPurify + the link policy and navigates on click.

## Open design choices

1. **Link label** — memory title (nicer, needs a title lookup; `collect_all`
   already yields it) vs. raw `mem.key` slug (zero extra lookup).
2. **Non-memory wikilinks** — `links.rs` regex is mem-only today. Scope to
   memories, or extend to other entity kinds (`SL-`, `ADR-`, …)? Start
   mem-only.

## Provenance

Surfaced during the SL-201 map-focus / onboarding fixes — once memory nodes and
onboard deep-links worked, following inline `[[...]]` references was the obvious
next gap. Feasibility scouted: **medium, leaning easy** (one backend seam, reused
resolver, no new API, no frontend work).
