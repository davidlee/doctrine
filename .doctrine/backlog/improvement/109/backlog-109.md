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

## Companion debt: scan↔show facet parse-path duplication (SL-133 OQ-3)

SL-133's design (§5.1, §7 D2/D3, OQ-3) surfaced the *other half* of the same
read_facets smell: the **scan** path (`read_facets`) and the **show** path
(`SliceDoc` serde) parse the same `[estimate]`/`[value]` facets twice through two
separate code paths. The single-carrier fix unifies both — collapse `ScannedEntity`
onto one `EntityFacets` carrier so scan and show derive facets from one projection.
SL-133 deferred it (D2/D3): unifying now reworks done code with a bigger blast
radius. Resolving IMP-109's single-table read and this scan↔show carrier together is
the cohesive cleanup; do them as one piece of work.

## Companion debt: dep/seq re-parse in PriorityGraph (merged from IMP-243)

Surfaced during SL-194's external design review (F-ext-4). `PriorityGraph`
construction reads each entity's TOML **an additional time** beyond the scan:

1. `relation_graph::scan_entities` opens+parses every entity's file for outbound
   ref/lineage edges, facets, and status (the shared scan).
2. `build_from` then calls `relation_graph::dep_seq_for(root, kind, id)` per
   entity for `needs`/`after` — explicitly "NOT part of `scan_entities`"
   (`src/priority/graph.rs:359`), re-reading the same TOMLs.

**Confirm first (5 min):** verify `dep_seq_for` actually re-opens/re-parses the
file vs reading a cheap cached handle. If cached, the win shrinks.

Folding `needs`/`after` into the unified `ScannedEntity` payload (from the
single-read fix above) would eliminate a whole per-entity parse pass on **every**
priority build — `survey`, `next`, `explain`, `blockers`, `inspect`, `findings`.
Payoff beyond perf: a single shared dep/seq map means β-family sweeps are
structurally frozen by construction, dissolving SL-194's quiescent-tree
precondition (R4).

**Blast radius:** touches `scan_entities` / `relation_graph` — shared machinery.
Behaviour-preservation gate requires every priority suite stay byte-identical.

## Provenance

- Originates: SL-103 audit (RV-100 F-4), SL-133 OQ-3 (scan↔show), SL-194 F-ext-4
  (dep/seq)
- Merged with IMP-243 (duplicate — same single-read refactor, dep/seq is the
  third parse to eliminate)
