# Memory body wikilinks as catalog edges

## Context

Memories cross-reference two ways, and the two surfaces are disjoint:

- **TOML relations** (`[[relation]]` rows in `memory.toml`) — read by the catalog
  (`scan_memory_entities` → `read_catalog_record` → `record.relations`) and
  rendered as web-map edges.
- **Body wikilinks** (`[[mem.<key>]]` in the `.md` body) — read by the CLI
  (`links::extract_wikilinks`, used by `resolve-links`/`backlinks`/`show`) but
  **never by the catalog**.

`MemoryCatalogRecord` (`src/memory.rs:1278`) has no body field; the memory node
is hydrated with `body: None` (`src/catalog/hydrate.rs:305`) and its edges come
solely from `record.relations`. So a memory whose associations live only in body
wikilinks — as the shipped onboarding corpus does (`See [[mem.signpost.doctrine.
lifecycle-start]]`) — draws **zero edges** in the web map.

Enabling dependency **ISS-213** has landed (`93880c77`): `mem_key_map` now spans
both corpora keyed on the full `mem.…` `memory_key`, and `classify_target`
already resolves both `mem.<key>` and `mem_<uid>` target forms. Body wikilinks
extract as `mem.`-prefixed strings, so they resolve through the existing map with
no new resolver. **SL-200** relinks the shipped corpus body wikilinks to the
`mem.`-prefixed form the extractor requires.

## Scope & Objectives

Make memory body wikilinks a **first-class catalog edge source**, alongside TOML
relations.

- Load the memory `.md` body during scan (impure layer) so `Catalog::from_scanned`
  stays pure over the record data. Carry it on `MemoryCatalogRecord`.
- In the memory hydration loop, `extract_wikilinks(body)` → `classify_target`
  (reusing the existing `mem_key_map` resolution) → emit a `CatalogEdgeLabel::Raw`
  edge per resolved link, matching how untyped TOML memory relations render.
- **Dedup** on the resolved `EdgeTarget`: a body wikilink and a TOML relation to
  the same target draw one edge, not two.
- **Diagnostics:** an unresolved body wikilink emits a dangling-reference
  `Warning`, parity with TOML relations. The extractor's `mem.word.word` shape
  excludes prose false-positives, so warning does not misfire.
- **Edge label** is a single-source named constant (STD-001) — no inline
  `"related"` literal.

Affected surface:

- `src/catalog/scan.rs` — `read_catalog_record` / `scan_memory_entities` (body load)
- `src/catalog/hydrate.rs` — memory edge loop in `Catalog::from_scanned`
- `src/memory.rs` — `MemoryCatalogRecord` body field

## Non-Goals

- **Clickable focus links in the map markdown renderer** (body `[[mem.…]]` →
  `#/focus/mem_<uid>`, currently inert text) — deferred to a separate item.
- Populating the memory `CatalogEntity.body` for rendering — bodies are read
  transiently for edge extraction only; the node stays `body: None` unless a
  consumer needs the prose.
- Any change to CLI body-wikilink resolution (`resolve-links`/`backlinks`) — only
  the catalog/map is affected.
- Corpus curation (prefix-less key relinking) — owned by SL-200.

## Summary

<!-- filled at close -->

## Follow-Ups

- Map markdown renderer: turn body `[[mem.…]]` into clickable `#/focus/mem_<uid>`
  focus links (deferred nice-to-have from ISS-214).
