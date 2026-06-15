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
- Content follows design §2.1 principles: signpost/concept framing, fresh synthesis, point-don't-restate

## PHASE-02 complete — 2026-06-15

- Signpost-only filter: `produce()` Memory arm passes `Some(MemoryType::Signpost)`
- Governance nudge: `gov_nudge()` for empty Policies/Standards, `is_marker` extended
- 3 new tests: signpost-only exclusion, governance check warnings, populated governance no-warn
- 55 boot tests pass, zero clippy warnings
- `just check` green (1322 tests)
- Commit: 9a0d1dd

### Key details

- `is_marker` now detects both `marker()` and `gov_nudge()` — governance sections properly reported
- No false positives: populated governance does not trigger warnings
- Memory section renders only signpost-type rows (~16 vs ~137 prior)
