# Corpus-level relation query verb

## Context

The only relation read surface is `inspect <ID>` — per-entity, 1-hop.
There is no verb for "show all `governed_by` edges across the corpus" or
"which entities reference ADR-001?" The data exists in the catalog graph
(1004 edges, fully typed), but the only way to query it is `catalog graph`
(raw JSON, developer-only).

This is a pure consumption surface — zero new modelling. RFC-001's thesis
names this exact gap: the graph is rich on the inside, thin on the outside.

## Scope & Objectives

### `doctrine relation list`

```bash
# All edges of one label
doctrine relation list --label governed_by

# Filter by target
doctrine relation list --label governed_by --target ADR-001

# Filter by source kind
doctrine relation list --source-kind SL

# Multiple filters combined
doctrine relation list --label specs --source-kind IMP
```

Table output: source, label, target — sortable by label then source.

### `doctrine relation census`

```bash
doctrine relation census
```

Edge type distribution: label → count. Active labels only (exclude
Raw/unvalidated). Single table.

### Implementation

Read-only over `Catalog` (already hydrated by `scan_catalog`). No new
disk I/O. Filter edges by label, source kind prefix, target ref. Format
as table or JSON (matching `survey`/`inspect` pattern).

## Non-Goals

- No graph export (DOT/GraphML) — that's a different surface (proposal 0007)
- No transitive query — that's SL-138
- No write path — read-only

## Terrain

| File | Change |
|------|--------|
| `src/commands/relation.rs` (new) | `run_relation_list`, `run_relation_census` |
| `src/main.rs` | Register `Relation` subcommand with `list`/`census` |
| `src/catalog/hydrate.rs` | Reuse `Catalog` — no changes |
| `src/listing.rs` | Reuse `Format` enum for table/json |

## Dependencies

- `Catalog` from `scan_catalog` — already built, no changes needed
- SL-135 (CM scan gap) — soft `after` (query surface should see fixed data)
