# SPEC-020: Estimate graph exposure

## Context

SL-101 delivers the `EstimateFacet` model. This slice wires it into the catalog
hydration pipeline so graph tooling (Cordage) can consume estimate metadata through
a stable, policy-free contract.

**Depends on SL-101** — the facet model must be parseable and validatable before
it can be exposed to the graph.

## Scope & Objectives

- **FR-006 (REQ-274)** — Catalog/graph hydration exposes per estimated node:
  entity id, kind, `lower`, `upper`, project unit, relations/edges, lifecycle state.
- **Value facet exposed alongside estimate (design D1)** — the generic scan-side
  reader carries both; the contract is symmetric (`value` magnitude + value unit).
  *No requirement covers value graph exposure yet — see Open Questions.*
- Units are a top-level `units` block on the catalog/graph (design D2), not
  per-node — they are project-wide constants, not node properties.
- Contract is policy-free — no aggregation, traversal, or interpretation.
- Graph-facing surface uses graph-neutral vocabulary (passes the whole-word denylist:
  no project/task/schedule/capacity). Contract field names (`units`/`estimate`/
  `value`/`lower`/`upper`) are clear of the banned set.

## Non-Goals

- Aggregation, simulation, thresholds → PRD-014 non-goals.
- Display rendering → SL-102.
- Confidence bounds exposure (SL-102 display concern) — stays dead-code-expected.
- Surfacing facets in the map_server HTTP view — UI concern, out of scope.

## Open Questions

- **OQ-1 — value graph exposure traceability.** D1 widens this slice past FR-006
  (estimate only). Before close: add a value-exposure REQ under SPEC-020, or widen
  REQ-274's scope to both facets. Carried into reconciliation; must not ship
  un-traced scope.

## Summary

The facet models exist (SL-101) but are wired only onto the show path
(`SliceDoc`); the scan/catalog path carries no facet data. This slice adds a
kind-agnostic `read_facets` read in the scan shell (`src/catalog/scan.rs`),
carries `estimate`/`value` through `ScannedEntity → CatalogEntity` (hydrate.rs)
→ `CatalogNode` (graph.rs), and resolves project-wide units once from
`doctrine.toml` into a top-level `units` block. Hydration stays pure (units
injected); the read and unit resolution live in the shell. See `design.md`.
