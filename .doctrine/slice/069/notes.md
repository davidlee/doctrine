# Notes SL-069: Shipped memory corpus as a cohesive client onboarding anchor

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 complete — 2026-06-15

- 13 new shipped memories authored via `doctrine memory record --global`
- All 27 embedded masters pass `lint_master`
- `just check` green (1314 tests, zero clippy warnings)
- Commit: 75333d2

### Key details

- UIDs minted by `doctrine memory record --global` (ULID format)
- Symlink aliases auto-created by the CLI
- TOML scaffolds carry correct INV signatures: `repo=""`, `anchor_kind=none`
- All memories cross-reference at least one existing shipped sibling

## PHASE-02 complete — 2026-06-15

- Signpost-only filter: `produce()` Memory arm passes `Some(MemoryType::Signpost)`
- Governance nudge: `gov_nudge()` for empty Policies/Standards, `is_marker` extended
- 3 new tests: signpost-only exclusion, governance check warnings, populated governance no-warn
- 55 boot tests pass, zero clippy warnings
- Commit: 9a0d1dd

## PHASE-03 complete — 2026-06-15

- 5 worst-stale memories refreshed: cli-command-map, file-map, skill-map, lifecycle-start, core-loop
- Cross-references added from existing (7) to new shipped memories
- All 27 pass lint_master; SL-005/007/008 suites green
- Commit: 9150ee6

## PHASE-04 complete — 2026-06-15

- VT-1: sync integration test — exactly 27 INV-signatured shipped dirs
- VT-2: per-memory retrieval surface + ADR-002 signature assertion
- All 7 e2e sync tests pass; full gate green (zero clippy warnings)
- Commit: 9b4e2ee

### Key finding

- `memory show` resolves only from `items/`, not `shipped/` — integration tests
  verify INV signature by reading shipped TOML directly and querying via `find`
