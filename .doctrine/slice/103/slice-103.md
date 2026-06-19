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
  *Deliberate traced-pending scope: no existing REQ covers value graph exposure
  (REQ-274 is estimate-only; REQ-278/279/280 govern the value model/validation/unit,
  not its graph projection). Requirement authored + spec-homed at reconcile — see
  Open Questions and design §7 [RV-094 F-1].*
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
- Rendering facets in the **web map UI** (`/api/map`'s `{key,label}` DTO) — UI
  concern, out of scope. *Note [RV-094 F-3]: `/api/graph` serves `CatalogGraph`
  raw, so it DOES surface the facet contract — that exposure is in scope; only the
  rendered web map view is excluded.*

## Open Questions

- **OQ-1 — value graph exposure traceability [RV-094 F-1; CHR-011].** D1 widens
  this slice past REQ-274 (estimate only). Reconciled position: this is deliberate
  scope, not a silent gap. At reconcile — author a value-graph-exposure requirement
  (sibling in intent to REQ-274), decide its spec home (SPEC-020 is titled "Estimate
  graph exposure", so value may want a rename/extension or its own spec), bind
  SL-103 to it, and also trace SL-103 → REQ-280 (value unit, realised in §5.4). The
  CLI cannot mint an un-homed requirement, so it is captured as intent now and
  formally minted at reconcile. Tracked: CHR-011.

## Summary

The facet models exist (SL-101) but are wired only onto the show path
(`SliceDoc`); the scan/catalog path carries no facet data. This slice adds a
kind-agnostic `read_facets` read in the scan shell (`src/catalog/scan.rs`),
carries `estimate`/`value` through `ScannedEntity → CatalogEntity` (hydrate.rs)
→ `CatalogNode` (graph.rs), and resolves project-wide units once from
`doctrine.toml` into a top-level `units` block. Hydration stays pure (units
injected); the read and unit resolution live in the shell. See `design.md`.
