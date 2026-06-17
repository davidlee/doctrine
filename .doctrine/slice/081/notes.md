# SL-081 implementation notes

## Review penance — RV-051 + RV-052

**2026-06-17**. Applied all fix-now findings from both review ledgers
against the `review/081` candidate branch.

### Fixes applied

| Finding | Summary | Change |
|---|---|---|
| RV-051 F-1 / RV-052 F-1 | D10 blank-edge emission | Added `continue` after empty-field diagnostics in `from_scanned` memory loop |
| RV-051 F-3 | Missing UnresolvedRef diagnostic for memory edges | Added diagnostic generation mirroring numbered-entity pattern |
| RV-051 F-4 | Edge origin field inconsistency | Resolved by F-1 `continue` — empty-label rows never reach edge push |
| RV-051 F-7 | Missing integration test for memory-edge resolution | Added `memory_edge_pipeline_resolves_and_diagnoses` and `memory_empty_relation_fields_surface_diagnostics_not_edges` tests |
| RV-051 F-2 | EdgeTarget serialization change undocumented | Added risk-table entry in design.md |
| RV-052 F-2 | Slice status stale at `plan` | Advanced to `implement` |
| RV-052 F-3 | `src/catalog/diagnostic.rs` missing from affected-surface table | Added row to design.md table |
| RV-052 F-4 | Missing gate evidence | Recorded below |

### Gate results

- `cargo build` — success
- `cargo clippy --workspace` — zero warnings
- `npx eslint web/map/` — clean
- `cargo test --package doctrine -- catalog::hydrate` — 11 passed, 0 failed
- New tests: `memory_edge_pipeline_resolves_and_diagnoses`, `memory_empty_relation_fields_surface_diagnostics_not_edges`

---

## PHASE-08 — Integration wiring

**2026-06-17**. Executed inline.

- `scan_catalog` wired to `scan_memory_entities`
- `memory_type` propagation gap fixed
- 1567 tests pass (dispatch baseline)
- 142 MEM entities in catalog graph
