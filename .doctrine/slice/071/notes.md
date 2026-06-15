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
