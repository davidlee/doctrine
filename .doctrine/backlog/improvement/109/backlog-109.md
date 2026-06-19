# IMP-109: catalog scan: read each entity TOML once, derive status/title/facets from one table (eliminate read_facets second parse + divergent-read window)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced by the SL-103 audit (RV-100 F-4, disposition `aligned`).

`src/catalog/scan.rs` parses each entity TOML twice per scan: `status_and_title_for`
parses it for `(status, title)`, then `read_facets` re-reads and re-parses the same
file for `[estimate]`/`[value]`. Two consequences:

- A redundant per-entity parse (≈corpus size). Negligible on the tooling/map
  surface — design D3 consciously accepted it.
- A divergent-read window: if the file vanishes/garbles between the two reads,
  `read_facets` returns absent rather than re-diagnose (the status read is the
  authority and re-diagnoses next scan — RV-094 F-6, benign).

The cleaner fix the design itself records (§5.1 NOTE): read each entity TOML **once**
and derive status/title/facets from the one parsed table, eliminating both the second
parse and the divergent-read window. Out of scope for SL-103; not a defect — the
shipped code matches the ratified design. Touches the `read_meta`/`Meta` seam SL-101
deliberately kept facet-free, so scope it carefully.
