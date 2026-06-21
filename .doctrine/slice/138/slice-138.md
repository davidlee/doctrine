# Relation-transitive walk for inspect

## Context

`blockers --transitive` walks dep edges transitively via cordage's
`reachable`. There is no equivalent for relation edges. You can't ask
"show everything shaped by ASM-001, transitively" or "show the full
descends_from tree rooted at PRD-001."

The cordage graph already builds with relation overlays — the walk
machinery exists. This is a flag on `inspect`, reusing the same
`reachable` infrastructure.

RFC-001's thesis explicitly names transitive impact query (proposal 0003)
as a consumption surface gap.

## Scope & Objectives

```bash
# Transitive walk on one relation label
doctrine inspect SL-047 --transitive --label governed_by
# → SL-047 and everything it (transitively) governs

# Transitive walk on all labels
doctrine inspect PRD-001 --transitive
# → Full transitive closure of all relation edges from PRD-001

# Depth limit to prevent explosion
doctrine inspect ADR-001 --transitive --max-depth 5
```

### Implementation

- Reuse `cordage::reachable` (same as `blockers --transitive`)
- Build a relation overlay filtered to one label (or all labels if omitted)
- Walk outward from the source entity
- Render as indented tree or table (matching `blockers --transitive` format)
- Default depth limit: 10 (safety valve)

## Non-Goals

- No inbound transitive walk (only outward from source)
- No cycle detection beyond what cordage already provides
- No graph export

## Terrain

| File | Change |
|------|--------|
| `src/commands/inspect.rs` | Add `--transitive`, `--label`, `--max-depth` flags |
| `src/relation_graph.rs` | Build single-label overlay for cordage walk |
| `src/priority/` (cordage) | Reuse `reachable` — no changes |

## Dependencies

- SL-137 (corpus relation query) — soft `after` (builds on same catalog query surface)
- Cordage `reachable` — already used by `blockers --transitive`
