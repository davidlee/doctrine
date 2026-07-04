# ISS-214: Memory body wikilinks are invisible to the web map — catalog scan never reads memory .md bodies

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Defect

Memories express cross-references two ways:

- **TOML relations** (`[[relation]]` rows in `memory.toml`) — read by the
  catalog (`read_catalog_record` → `record.relations`) and rendered as edges in
  the web map.
- **Body wikilinks** (`[[mem.<key>]]` in the `.md` body) — read by the CLI
  (`memory resolve-links` / `backlinks` / `show` via `links::extract_wikilinks`)
  but **never by the catalog**.

`scan_memory_entities` → `read_catalog_record` (`src/memory.rs`) reads **only**
the TOML; `MemoryCatalogRecord` has no body field and the catalog never calls
`extract_wikilinks`. So a memory whose associations live in body wikilinks
produces **zero edges in the web map**, regardless of key correctness.

The two link surfaces are disjoint: the CLI sees body wikilinks, the map sees
TOML relations, and neither sees the other. Authors who cross-link via body
prose (the shipped onboarding memories do exactly this) get no map graph.

## Evidence

Confirmed during SL-200 `/design`. The shipped onboarding corpus links entirely
via body wikilinks (`See [[signpost.doctrine.lifecycle-start]]`); the live map
shows those memories with zero/near-zero edges. The catalog code path
(`scan_memory_entities`) reads no `.md` body.

## Fix direction

Add memory **body wikilinks as a catalog edge source**: during memory scan,
read the `.md` body, `extract_wikilinks`, resolve, and emit catalog edges
alongside the TOML `[[relation]]` edges. Design decisions:

- **Edge label/semantics** for a body wikilink (unlabeled) — likely `Raw`
  (`related`), matching how untyped memory relations already render.
- **Dedup** against a TOML relation to the same target (don't double-draw).
- **Resolution** must cover shipped keys — see **ISS-213**
  (`build_memory_key_map` currently indexes only `items/`; the CLI path already
  resolves shipped keys from the `.key` field, so unify the map onto the same
  basis).
- Consider whether the map markdown renderer should also turn body `[[mem.…]]`
  into clickable focus links (`#/focus/mem_<uid>`) — currently rendered as inert
  text; adjacent nice-to-have.

## Relations / boundaries

- Depends on / pairs with **ISS-213** (shipped-key resolution) — a body-wikilink
  edge to a shipped memory needs shipped keys to resolve.
- Alternative to authoring TOML relations (SL-200): SL-200 takes the cheap path
  (TOML relations, already ingested) rather than this structural change. This
  issue is the "make body wikilinks first-class in the map" option, deferred.
- CLI body-wikilink resolution is **not** broken by this — only the map is
  affected. The separate corpus bug (prefix-less keys not matching the extractor
  regex) is orthogonal and lives with the corpus curation, not here.

## Verification

- A memory whose only cross-reference is a body `[[mem.<key>]]` wikilink draws a
  resolved edge in the web map graph.
- No duplicate edge when both a body wikilink and a TOML relation point at the
  same target.
- Existing TOML-relation edges and CLI backlinks stay green
  (behaviour-preservation).
