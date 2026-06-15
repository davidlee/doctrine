# Implementation Plan SL-071: Entity corpus scanner / hydrator substrate

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six ordered phases that deliver a reusable entity corpus scanner + hydrator
substrate, staged from mechanical extraction through richer types, graph
projection, consumer migration, and optional debug CLI. The plan follows the
5‑patch migration in design §1 exactly — the phases are thin expansions of
those patches, not a reorganisation.

Every phase gates on the prior; no phase depends on a later one. The behaviour‑
preservation gate is threaded through PHASE‑01 (move with re‑exports, tests
must pass unchanged), PHASE‑02 (equivalence tests that pin the existing
output), and PHASE‑05 (migrate consumers without test drift).

## Sequencing & Rationale

### PHASE-01 — Mechanical re-home + compatibility re-exports

Before any new types or behaviour, the six existing items (`EntityKey`,
`ScannedEntity`, `scan_entities`, `outbound_for`, `status_and_title_for`,
`title_for`) are moved from `src/relation_graph.rs` into the new
`src/catalog/scan.rs`. The move is mechanical: remove the definitions, add
them to `catalog::scan`, add `pub(crate) use` re-exports in the old module.

This is the highest-risk patch — any import break or visibility mismatch
causes cascading compile failures. The re-export alias pattern (design D7)
ensures one body, one source of truth. `dep_seq_for` and `require_minted`
stay in `relation_graph.rs` per design D1/D2.

**Why first?** The move changes the module boundary without changing behaviour.
If this gate fails (tests break, clippy warns, dead code), the entire plan
is blocked. Better to pay that cost before touching any new types.

### PHASE-02 — Equivalence tests

Five fixture tests are added BEFORE richer catalog types so that any
subsequent changes are pinned against known-good output:

- `scan_order_is_stable` — the scan order is load-bearing for byte-identical
  `inspect` output; KINDS-table / id-ascending must not regress.
- `catalog_scan_matches_legacy_shape` — entity identity, status, title, and
  outbound edges are pinned as a tuple. Uses a fixture with ≥3 entities
  spanning ≥2 KINDS entries with id gaps.
- `inspect_output_is_byte_identical` — the end-to-end `inspect --json`
  surface. Golden comparison, not inlined.
- `priority_graph_shape_unchanged` — node/edge/overlay counts are pinned.
- `validate_relation_findings_unchanged` — the validate walk's finding
  strings on a known-dangling-edge fixture.

**Why second?** These tests are the behaviour-preservation gate made explicit.
They catch any drift from the re-home (PHASE-01) or from future richer
types (PHASE-03/04). Without them, we'd have only the legacy test suites
which may be too coarse to catch subtle scan-order or hydration regressions.

### PHASE-03 — Richer catalog types

`Catalog`, `CatalogEntity`, `CatalogEdge`, `EdgeTarget`, `EdgeOrigin`,
`CatalogDiagnostic`, and `Severity` are added in `src/catalog/hydrate.rs`
and `src/catalog/diagnostic.rs`. `scan_catalog` wraps `scan_entities` with a
pure `Catalog::from_scanned` projection.

Target classification uses `integrity::parse_canonical_ref` — the existing
oracle — with four outcomes mapped to three `EdgeTarget` variants (design D5).
No new disk read; no new parsing path.

Diagnostics are limited to edge classification at this stage — the fail-fast
`scan_entities` still bails on malformed entities. The `CatalogDiagnostic`
type is plumbed for a follow-up slice that adds an error-tolerant walk (design
diagnostic limitation note).

**Why third?** The new types depend on `ScannedEntity` being in `catalog::scan`
(PHASE-01) and on the equivalence tests (PHASE-02) to catch regressions. The
existing `scan_entities` is not restructured — `Catalog` is a pure projection
on top.

### PHASE-04 — Presentation-neutral graph

`CatalogGraph` with `NodeKey`, `CatalogNode`, `outgoing`, and `incoming`
methods is added in `src/catalog/graph.rs`. It is a pure projection of
`Catalog` — no cordage, no disk. Edges with unresolved/unvalidated targets
appear in the edge list but have no target node in the BTreeMap.

`neighbours(depth)` is deferred per design D10 — it involves traversal that
is not needed for debug output and adds complexity without a consumer.

**Why fourth?** The graph is a projection of `Catalog`; it has no purpose
until `Catalog` exists. It's placed before consumer migration so that
PHASE-05 can consider whether any consumer benefits from `CatalogGraph`
queries — though the current design says none do yet.

### PHASE-05 — Consumer migration

Existing consumers are migrated to use `catalog` types where beneficial.
`relation_graph` already consumes `catalog::scan::scan_entities` via the
PHASE-01 re-exports — zero changes needed. `validate_relations` may optionally
consume `Catalog.edges` for dangler detection (not forced). `priority` stays
on `ScannedEntity`.

The gate: every existing test must pass with zero test-code changes.
`rg 'for kref in integrity::KINDS' src/` outside `catalog/` hits exactly the
IllegalRows walk in `validate_relations` and no other entity-scanning loop.

**Why fifth?** Migration is the riskiest behavioural change — it must happen
only after the new types are proven (PHASE-03/04) and the equivalence tests
can detect every regression (PHASE-02). The re-exports from PHASE-01 mean
`relation_graph` never breaks during earlier phases.

### PHASE-06 — Debug CLI

`doctrine catalog scan --json` and `doctrine catalog graph --json`
subcommands are added for developer inspection. Thin JSON dump — no colour,
no pagination, no table format. Accepts `--root <path>` for non-default
corpus scanning.

Optional per design D12 — not gating for acceptance.

**Why last?** The CLI depends on `Catalog` and `CatalogGraph` (PHASE-03/04)
but is not required for any consumer. It can be skipped without blocking
slice closure.

## Notes

- `dep_seq_for` and `require_minted` remain in `relation_graph.rs` per design D1/D2. Reconsideration is a follow-up, not part of this slice.
- SourceSpan is `(file, field)` only — line/col deferred. No TOML span parser exists in the codebase.
- `neighbours(depth)` is deferred — add when the concept mapper or inspect-like view needs it.
- Patch 6 (CLI) is optional per D12. If time-constrained, skip it and close the slice after PHASE-05.
- The `catalog` module is engine-tier per ADR-001 — depends only on leaf-tier modules (`entity`, `integrity`, `relation`, `projection`, `meta`, `fsutil`, `listing`) and kind modules, never on command modules.
- Tests in `relation_graph.rs`'s `mod tests` that exercise `catalog::scan` functions through re-exports remain in place — relocation is a tracked follow-up (cite design D8). The behaviour-preservation gate is proven by these tests passing unchanged.
