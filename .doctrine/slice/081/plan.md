# Implementation Plan SL-081: Surface memory entities and their relations in the catalog graph + web explorer

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Eight phases take memory entities from invisible (0 of ~60 in the catalog graph)
to fully visible in both the CLI inspect output and the web explorer's graph +
detail pane. The work threads through five modules — memory.rs, catalog/scan.rs,
catalog/hydrate.rs, catalog/graph.rs, and the map_server — with a narrow
cross-section of type changes in the catalog layer and additive functions
everywhere else.

## Sequencing & Rationale

### PHASE-01 — Memory TOML types (memory.rs)

Foundation phase. `RawRelation` goes from a fieldless serde-discard stub to a
real struct with `label` and `target`. `MemoryCatalogRecord` formalises the
projection one memory entity presents to the catalog. `read_catalog_record`
reuses `RawMemoryToml` (the same struct `Memory::parse` uses — single source of
truth) and validates uid shape only. No stricter validation: `Memory::parse`
remains the authority for vocabulary checks.

This phase is self-contained in `memory.rs`. All existing memory tests must
stay green — `RawRelation`'s new fields are `#[serde(default)]`, so TOML
without `[[relation]]` rows parses identically to before.

### PHASE-02 — Catalog identity types (hydrate.rs)

The largest phase. Introduces two new enum types at the catalog boundary:

- **`CatalogKey`** — `Numbered(EntityKey)` | `Memory(String)`. `EntityKey` stays
  unchanged (`Copy`, numbered-only, KINDS-backed) — the priority and relation
  graph modules never see a `Memory` key. The manual `Serialize` impl flattens
  both variants to a string, matching the pre-existing `NodeKey` serialization
  contract.
- **`CatalogEdgeLabel`** — `Validated(RelationLabel)` | `Raw(String)`. Memory
  edges carry their authored label verbatim; numbered edges use the existing
  validated vocabulary.

The phase then revises `CatalogEntity`, `CatalogEdge`, `EdgeTarget`, and
`CatalogDiagnostic` to use these types. `from_scanned` gains a `memory` parameter
and builds memory nodes with `kind_label = "MEM"`, `kind = None`. `classify_target`
switches from `BTreeSet<EntityKey>` to `BTreeSet<CatalogKey>`.

This is a mechanical type-ripple phase — ~15 type changes, no new algorithms.
Existing tests must pass with the updated types.

### PHASE-03 — Memory scanner (scan.rs)

Additive phase. `scan_memory_entities` walks `MEMORY_ITEMS_DIR` and
`MEMORY_SHIPPED_DIR` via `entity::scan_named` (real dirs only, no symlinks),
reads each `memory.toml` via `read_catalog_record`, validates uid == dirname,
and collects diagnostics into the out-param `Vec`. Items override shipped on
uid collision. Malformed records produce `CatalogDiagnostic::Error` — never
silently skipped (D6).

Depends on PHASE-01 for `read_catalog_record` and PHASE-02 for `CatalogKey`
(in diagnostic `entity_key`). Could execute in parallel with PHASE-02 if
needed, but sequenced after since PHASE-02 is the chokepoint.

### PHASE-04 — Graph projection (graph.rs)

Replaces the `NodeKey` enum with a `CatalogKey` re-export. Adds
`memory_type: Option<String>` to `CatalogNode` (serialized in JSON, skipped
when `None`). Updates `outgoing`/`incoming` match arms: `CatalogKey::Numbered`
behaves as before; `CatalogKey::Memory` returns empty (memory has no incoming
edges in scope, and outgoing edges are already in the flat edge list).

Must come after PHASE-02 (needs `CatalogKey`).

### PHASE-05 — Memory markdown reader (markdown.rs)

Self-contained additive phase. `read_memory_markdown` tries `items/<uid>/memory.md`
then `shipped/<uid>/memory.md` using `safe_join` for path containment and async
`read_to_string` with `NotFound` fallthrough — no sync `exists()` check. Ships
before PHASE-06 (which consumes it).

No phase dependencies — uses only existing `is_uid`, `safe_join`, and dir
constants. Can execute in parallel with PHASE-01..04.

### PHASE-06 — Markdown route fallback (routes.rs)

Extends `entity_markdown` with a memory uid path: canonical ref parse first
(existing behaviour), then `is_uid` fallback with graph membership check via
`CatalogKey::Memory`. Returns 200 with `memory.md` body, 404 if not in graph,
400 for bogus `mem_` strings.

Must come after PHASE-04 (needs `CatalogKey::Memory` for membership check) and
PHASE-05 (needs `read_memory_markdown`).

### PHASE-07 — Relation validation (relation_graph.rs)

Updates `validate_relations` for the new edge type. Memory edges (`CatalogKey::Memory`
source) are skipped — no vocabulary exists for them yet. Numbered edges with
`CatalogEdgeLabel::Raw` are reported as internal corruption. The existing
`entity_kinds.get()` call is tightened from `if let` to an explicit `let Some`
with an error message for the `None` case (which is a bug, not expected).

Must come after PHASE-02 (needs `CatalogKey`/`CatalogEdgeLabel`). Can execute
in parallel with PHASE-03..06.

### PHASE-08 — Integration & gate

Wires `scan_catalog` to call `scan_memory_entities`, runs the full test suite,
verifies `just gate` passes, and confirms memory nodes appear in the production
catalog graph. The final integration point — all new code meets all existing
code.

## Parallelization opportunities

| Phase group | Phases | File-disjoint? |
|---|---|---|
| Foundation | PHASE-01, PHASE-05 | Yes — memory.rs (01) and markdown.rs (05) are separate modules |
| Catalog core | PHASE-02 | Sequential after PHASE-01 |
| Catalog extensions | PHASE-03, PHASE-04 | PHASE-03 after 01+02; PHASE-04 after 02. Can be parallel with each other |
| Validation | PHASE-07 | After PHASE-02; parallel with 03/04/05/06 |
| Integration | PHASE-08 | After all |

Recommended dispatch order: 01+05 parallel → 02 → 03+04+07 parallel → 06 → 08.
Or serial if dispatch infrastructure is not available.

## Notes

- No new dependencies. All new code uses existing crates (toml, serde, anyhow,
  axum, tokio).
- Frontend (`web/map/`) is untouched — the SL-073 frontend handles `kindLabel`
  generically with a fallback color palette.
- `EntityKey` stays `Copy` — priority/graph.rs remains numbered-only.
- `CatalogKey` is `Clone` (not `Copy`) because `Memory(String)` carries a heap
  allocation. The BTreeSet in `classify_target` has ~600 entries — a 24-byte
  String clone per lookup is negligible.
- Memory→memory relations are NOT in scope — no `CatalogKey::Memory` target
  resolution needed.
