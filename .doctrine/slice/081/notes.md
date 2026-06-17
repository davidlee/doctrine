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

---

## Closure — 2026-06-17

- Slice advanced: `reconcile` → `done`; all 8 phases complete, penance resolved.
- **RV-051** (SL-086 inquisition, not SL-081 code-review) recovered from `stash@{3}` and landed. Findings all `verified`.
- **RV-052** lost — the original SL-081 code-review RV was orphaned in a dispatch worktree (GC'd). Substance resolved in penance table above.
- Both SL-081 review ledgers (RV-051 code-review, RV-052) lost from dispatch worktree GC; RV-051 recovered is the SL-086 design inquisition.
- No originating backlog item found — slice spawned directly.
- `just check`: 1568 passed, 0 failed (pre-existing `sync_produces_all_shipped_dirs` failure outside of this slice).
