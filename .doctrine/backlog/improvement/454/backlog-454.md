# IMP-454: Graph node labels carry titles under a bolded handle

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine graph` labelled every node with its bare canonical id. Readable only
if you have the corpus memorised — a fan-out of `DEC-255`/`DEC-256`/`DEC-257`
says nothing. SL-245 made the graph renderable inline (`-X`), which made the
gap visible: the title was already carried on the node, but only in the
`tooltip`, which a terminal raster cannot show.

Each node now reads as a bolded handle over its wrapped title.

## Decisions

- **D1 — titles on by default, no flag.** The id-only label is not worth
  preserving as the default. Clamp/disable flags can follow if the raster
  proves too large in practice.
- **D2 — handle is the citable form: canonical id, or a memory's readable
  key.** `CatalogKey::Memory::canonical()` is an opaque uid (`mem_019ebe…`),
  useless as a label, so `memory_key` is plumbed from `MemoryCatalogRecord`
  through `CatalogEntity` to `CatalogNode`. `graph --format json` gains it as
  an optional field on memory nodes (additive).
- **D3 — wrap at `LABEL_WRAP_COLS` (22), two tokenizers over one packer.**
  Titles break on whitespace; memory keys have none, so they break *after* a
  `.` or `-` with the delimiter riding the line it terminates. Unwrapped keys
  made boxes three times the width of their own content.
- **D4 — `monospace` for node and edge labels.** Keeps the wrap column count
  honest: a line's character count is its real width. Costs ~33% raster width
  against the Graphviz default, which matters because `-X` fits to the window.
- **D5 — HTML-like labels (`label=<…>`, `<BR/>`, XML escaping).** The only way
  to weight part of a label; `record` shapes carry ports, not formatting. It
  also separates handle from title without spending a blank line, and its
  tighter line spacing made nodes shorter than the `\n` form. Ghost nodes keep
  quoted labels — they have no handle to weight.

## Not in scope

- Flags to clamp or disable (width/height bounds, titles off).
- Web-map parity: `web/map/src/dot.ts` still emits id-only labels, so the CLI
  and the browser view now disagree.
- A `CatalogNode` fixture helper — adding one optional field touched 29
  hand-rolled constructions across `dot.rs`, `graph.rs`, `map_server/routes.rs`.
