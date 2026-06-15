# SL-071 Implementation Notes

## PHASE-01 (2026-06-15) — Mechanical re-home + compatibility re-exports

- **Created** `src/catalog/mod.rs` + `src/catalog/scan.rs`.
- **Moved** 6 items from `src/relation_graph.rs` to `catalog::scan`:
  `outbound_for`, `EntityKey`, `ScannedEntity`, `scan_entities`,
  `status_and_title_for` (private), `title_for` (private).
- **Re-exports** added in `relation_graph.rs` as `pub(crate) use` aliases
  for the 4 `pub(crate)` items; private helpers unreachable from outside
  `catalog::scan` — no re-export needed.
- **Imports** in `catalog::scan.rs` are minimal: `std::path::Path`,
  `crate::entity`, `crate::integrity`, `crate::listing`,
  `crate::relation::RelationEdge`. Full-path references to kind modules
  (`crate::slice::relation_edges(...)` etc.) work unchanged.
- **No import changes** needed in `relation_graph.rs` — the moved items shared
  the same imports as kept code.
- **Registered** `mod catalog;` in `main.rs` (alphabetical: after `boot`,
  before `clock`).
- **Gate**: 1324 tests pass unchanged; `cargo clippy` zero warnings; `just gate`
  passes workspace-wide.

## PHASE-02 (2026-06-15) — Equivalence tests

Six equivalence tests added to pin scan behaviour before richer catalog types:

- **4 unit tests** in `src/catalog/scan.rs` `#[cfg(test)] mod tests`:
  - `scan_order_follows_kinds_table_then_id_ascending` — proves KINDS-table
    order, ids ascending per kind, not readdir order
  - `scan_entity_shape_matches_expected` — pins (canonical key, prefix, status,
    title, outbound tuples) for 4 entities across 2 KINDS
  - `priority_graph_node_set_matches_scanned` — node set = scanned set, all keys
    resolve
  - `validate_reports_dangling_edge_and_ignores_free_text` — dangler detection +
    free-text exclusion
- **2 integration tests** in `tests/e2e_sl071_equivalence.rs`: black-box CLI
  goldens comparing `doctrine inspect --json` to checked-in
  `tests/fixtures/sl071_inspect_sl*_golden.json` via `include_str!` (VA-1
  compliant)

Fixture: 4 entities (SL-001, SL-003, ADR-002, REQ-005) spanning 2 KINDS with id
  gaps.

**Surprise**: Cordage `Graph` struct exposes no public `edge_count()` /
  `node_count()` — asserted node cardinality via `pg.attrs.len()` and key
  resolution instead. Recorded as
  `mem.fact.cordage.graph-no-public-edge-count`.

**Gate**: 1330 tests (1328 unit + 2 integration), 0 failures; `cargo clippy` zero
  warnings; `just gate` passes.

**Commits**: `707f7fd` (tests + fixture + memory), `257bb29` (verify memory).

## PHASE-03 (2026-06-15) — Richer catalog types

- **Created** `src/catalog/diagnostic.rs` with `CatalogDiagnostic` + `Severity`.
- **Created** `src/catalog/hydrate.rs` with `Catalog`, `CatalogEntity`,
  `CatalogEdge`, `EdgeTarget`, `EdgeOrigin`, `SourceSpan` types.
- **Implemented** `Catalog::from_scanned(root, &[ScannedEntity])` — pure
  projection. Classifies edge targets via `integrity::parse_canonical_ref` into
  `Resolved` / `UnresolvedRef` / `UnvalidatedText`. Entity paths derived from
  `EntityKey` + `Kind.dir`. `BTreeSet<EntityKey>` for O(log n) lookups.
- **`scan_catalog(root)`** — thin wrapper: `scan_entities` then `from_scanned`.
  No second KINDS walk.
- **Diagnostics**: one `Warning` per `UnresolvedRef`, one `Info` per
  `UnvalidatedText`. Fail-fast on bad entities preserved (that's `scan_entities`).
- **9 new tests**: entity hydration, resolved/unresolved/unvalidated edge
  classification, diagnostic generation, path derivation, scan integration,
  `classify_target` edge cases (unknown prefix, no dash, parses-absent,
  parses-present).
- **Design note**: `SourceSpan.file` and `EdgeOrigin.file` use entity directory
  path — the entity stem is not carried on `ScannedEntity`, and each entity dir
  has exactly one TOML, so the dir path is unambiguous.
- **Dead code**: `cfg_attr(not(test), expect(dead_code, ...))` on structs and
  functions. Fields read by tests but not yet by prod consumers (PHASE-04/05/06).
- **Gate**: 1337 tests (1330 + 9 new), 0 failures; `cargo clippy` zero warnings;
  `just gate` passes workspace-wide. PHASE-02 equivalence tests green unchanged.
