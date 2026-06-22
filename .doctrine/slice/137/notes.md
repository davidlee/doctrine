# SL-137 implementation notes

## Design
- Pure consumption surface over existing Catalog — no new modelling, no write path.
- D1: `relation { list, census }` under top-level `Relation`; link/unlink stay top-level.
- D2: `--include-memory` bool (default off); data source axis separate from resolution axis.
- D6: `--target` canonical-normalised via `integrity::parse_canonical_ref`.
- Engine/pure split: `relation_query.rs` has no I/O; `commands/relation.rs` owns diagnostics + RenderOpts.

## Implementation
- 2-phase dispatch via pi arm (worker subagents): PHASE-01 pure engine, PHASE-02 CLI shell.
- PHASE-01: `src/relation_query.rs` — 869 lines, 12 unit tests.
- PHASE-02: `src/commands/{cli,relation,guard}.rs` + `tests/relation_cli.rs` — 4 files, 630 lines, 9 e2e tests.
- Behaviour preservation: no edits to `catalog`/`relation`/`listing` internals.

## Audit (RV-145)
- 3 findings (all minor/nit, fix-now): test coverage gaps for VT-3/VT-4, stale dead_code lint.
- All fixed and verified. No spec/governance changes needed.
- Reconciliation brief: empty.

## Key gotchas
- `web/map/dist/` (CHR-020) missing from coordination tree — RustEmbed compile failure.
- Diagnostics policy (F1): edge-dropping Warnings counted separately from classification Warnings.
- Memory targets match by UID (F3), not by authored key alias.
