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
