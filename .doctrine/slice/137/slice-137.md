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

# Filter by target ("who references ADR-001")
doctrine relation list --label governed_by --target ADR-001

# Filter by source kind
doctrine relation list --source-kind SL

# Multiple filters combined
doctrine relation list --label specs --source-kind IMP

# Label provenance: validated vocab (default) | memory raw | both
doctrine relation list --labels all

# Validation lens: only edges whose target does not resolve
doctrine relation list --unresolved
```

Table columns: `source │ label │ target │ state`, sorted by label then
source. `state ∈ {resolved, unresolved, free_text}` (always present in JSON).

### `doctrine relation census`

```bash
doctrine relation census
doctrine relation census --labels all
```

Per-label distribution WITH a target-resolution health breakdown:
`label │ count │ resolved │ unresolved │ free_text`, sorted count desc
then label asc (`count == resolved + unresolved + free_text`). Default
`--labels validated` (closed `RelationLabel` vocabulary); `raw` = memory
free-labels; `all` = both.

### Flags (both verbs)

- `--labels validated|raw|all` — label provenance axis (default `validated`).
- `--unresolved` (list only) — restrict to non-resolving targets.
- `--format table|json` / `--json`.

### Implementation

Read-only over `Catalog` (already hydrated by `scan_catalog`). No new disk
I/O. Pure projection/filter in a new engine module `relation_query.rs`; the
command shell (`commands/relation.rs`) does root-find → scan → render. Rides
the `listing` spine (`Column` + `render_columns` + `json_envelope`), the
`coverage_view` precedent. Only `Error`-severity scan diagnostics go to
stderr (Warning/Info are surfaced by `--unresolved` / the census breakdown).

## Non-Goals

- No graph export (DOT/GraphML) — that's a different surface (proposal 0007)
- No transitive query — that's SL-138
- No write path — read-only
- No 3-way `--target-state` flag — deferred; `--unresolved` bool covers it now
- No grouping of link/unlink under `relation` — they stay top-level (design D1)

## Terrain

| File | Change |
|------|--------|
| `src/relation_query.rs` (new, engine) | pure projection/filter/render: `project_list`, `project_census`, `TargetState`, `LabelScope`, `Column` defs |
| `src/commands/relation.rs` (exists; hosts link/unlink) | add `RelationCommand{List,Census}` + `run_relation_list`/`run_relation_census` |
| `src/commands/cli.rs` | register top-level `Relation` subcommand |
| `src/catalog/hydrate.rs` | Reuse `Catalog` — no changes |
| `src/listing.rs` | Reuse `Format`, `Column`, `render_columns`, `json_envelope` — no changes |

## Dependencies

- `Catalog` from `scan_catalog` — already built, no changes needed
- SL-135 (CM scan gap) — soft `after` (query surface should see fixed data)
