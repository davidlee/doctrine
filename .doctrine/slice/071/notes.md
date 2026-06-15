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

## PHASE-04 (2026-06-15) — Presentation-neutral graph (CatalogGraph)

- **Created** `src/catalog/graph.rs` with `CatalogGraph`, `NodeKey`, `CatalogNode`.
- `from_catalog(&Catalog)` — pure projection, no cordage dependency.
- `outgoing(node)`: returns all edges whose source matches (incl.
  UnresolvedRef/UnvalidatedText).
- `incoming(node)`: returns only edges with `Resolved(target)` matching the node.
- `neighbours(depth)` deferred per design D10.
- 4 fixture tests covering node/edge counts, outgoing with unresolved targets,
  incoming exclusion of unresolved, and incoming correctness.
- **Gate**: 1341 tests, 0 failures; `cargo clippy` zero warnings.

## PHASE-05 (2026-06-15) — Consumer migration

- **`relation_graph`**: already consumes via re-exports — zero changes (EX-1 ✓).
- **`validate_relations`**: dangler detection migrated from `scan_entities` +
  manual iteration to `scan_catalog` + iteration of `catalog.edges`.
  - Builds `BTreeMap<EntityKey, &'static entity::Kind>` from `catalog.entities`
    for label validation via `relation::lookup`.
  - Produces identical finding strings — the existing test
    `validate_relations_reports_danglers_and_illegal_rows` passes unchanged.
  - IllegalRows re-read preserved as a separate KINDS walk (Catalog has no
    raw-TOML handle) — EX-5 ✓.
  - Imported `EdgeTarget` from `crate::catalog::hydrate`.
- **`priority`**: stays on `ScannedEntity` via re-exports — zero changes (EX-3 ✓).
- **`coverage_scan`**: not migrated (EX-4 ✓).
- **`#[expect(dead_code)]` retirement**: Removed unfulfilled expects from
  `scan_catalog` and `Catalog::from_scanned` — both now consumed externally.
  Remaining struct-level expects on `Catalog`, `CatalogEntity`, `CatalogEdge`,
  `EdgeOrigin`, `SourceSpan` are NOT unfulfilled (they have `pub(crate)` fields
  accessed externally — Rust treats them as reachable).
- **VA-1**: `rg 'for kref in integrity::KINDS' src/` outside `catalog/` hits
  exactly the IllegalRows walk (relation_graph.rs:348) — no other entity-scanning
  KINDS loop remains outside catalog.
- **VA-2**: Re-exports in `relation_graph.rs` are pure aliases — no logic wrappers.
- **Files changed**: `src/catalog/hydrate.rs` (-14 lines, 2 expect attrs),
  `src/relation_graph.rs` (+32/-28, dangler migration + import).
- **Gate**: 1341 tests, 0 failures, zero test changes; `cargo clippy` zero
  warnings; `just gate` passes workspace-wide.

## PHASE-06 (2026-06-15) — Debug CLI

- **Added** `CatalogCommand` sub-enum with `Scan` and `Graph` variants, each
  carrying `--json` + `--root <path>` flags.
- **Added** `Command::Catalog` variant (between Boot and Claude in the enum).
- **Wired** classification: `Catalog { .. } => Read` — merged into existing
  read-only group (Validate | Inspect | Survey | Next | Blockers | Explain)
  to satisfy `clippy::match_same_arms`.
- **Implemented** `run_catalog_scan` / `run_catalog_graph` — resolve root
  via `root::find`, validate `.doctrine/` dir exists, scan + serialize,
  write stdout. Error path: non-zero exit + stderr via `anyhow::bail!`.
- **Serde plumbing**:
  - Added `Serialize` derives to `Catalog`, `CatalogEntity`, `CatalogEdge`,
    `EdgeTarget`, `EdgeOrigin`, `SourceSpan`, `entity::Kind` (skipping `scaffold`
    fn pointer), `RelationLabel`, `CatalogDiagnostic`, `Severity`,
    `CatalogGraph`, `CatalogNode`, `EntityKey`.
  - Custom `Serialize` for `NodeKey` — required because `BTreeMap<K, V>` in
    serde_json requires string keys; serializes `Entity(key)` as `key.canonical()`.
- **Retired** `#[expect(dead_code)]` from `Catalog`, `CatalogGraph`,
  `CatalogNode`, `from_catalog` constructor, plus `CatalogEntity`,
  `CatalogEdge`, `EdgeOrigin`, `SourceSpan`. Retained on `outgoing` /
  `incoming` (not consumed by this CLI).
- **Root validation**: `root::find` passes explicit paths through without
  checking existence; added `.doctrine/` directory check in both run
  functions so `--root /nonexistent` fails with non-zero exit + stderr
  (EX-4).
- **3 integration tests** in `tests/e2e_catalog_cli.rs`: valid JSON
  (scan + graph), non-existent root non-zero exit.
- **Files changed**:
  - `src/main.rs` (+64 lines: `CatalogCommand` enum, `Command::Catalog`
    variant, classify + dispatch, `run_catalog_scan` / `run_catalog_graph`)
  - `src/catalog/hydrate.rs` (serialize derives, dead_code removal)
  - `src/catalog/graph.rs` (serialize derives, dead_code removal, custom
    `NodeKey::Serialize`)
  - `src/catalog/scan.rs` (serialize derive on `EntityKey`)
  - `src/catalog/diagnostic.rs` (serialize derives)
  - `src/entity.rs` (serialize derive on `Kind`, skip `scaffold`)
  - `src/relation.rs` (serialize derive on `RelationLabel`)
  - `tests/e2e_catalog_cli.rs` (new, 3 tests)
- **Gate**: 1344 tests (1341 + 3 new), 0 failures; `cargo clippy` zero
  warnings; `just gate` passes workspace-wide.
