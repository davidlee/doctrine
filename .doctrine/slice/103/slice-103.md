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
- Contract is policy-free — no aggregation, traversal, or interpretation.
- Graph-facing surface uses graph-neutral vocabulary (passes the whole-word denylist:
  no project/task/schedule/capacity).
- Vocabulary: `node`, `member`, `value`, `width`, `overlay`.

## Non-Goals

- Aggregation, simulation, thresholds → PRD-014 non-goals.
- Display rendering → SL-102.

## Summary

Extend `src/catalog/hydrate.rs` (CatalogEntity) with estimate fields, and/or
extend the graph projection in `src/catalog/graph.rs`. Pure — reads from
already-hydrated catalog data.
