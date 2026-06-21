# Notes SL-132: Wire estimate/value display in show path

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01: Implementation notes (2026-06-21)

### Files changed

- `src/facet.rs` (new) — `EntityFacets` aggregation struct, leaf tier
- `src/estimate.rs` — ungated `mod display`, `DEFAULT_LOWER_CONFIDENCE`, `DEFAULT_UPPER_CONFIDENCE`, `resolve_confidence`
- `src/estimate/display.rs` — added `format_estimate_confidence`; preserved `format_estimate_normal`/`format_estimate_verbose` with function-level expects per D7
- `src/value.rs` — added `format_value_normal`
- `src/slice.rs` — `run_show` now loads full `DoctrineToml` (replaces `load_conduct`), resolves units+confidence, constructs `EntityFacets`, passes to `format_show`. `format_show` now renders estimate/value rows after dates, before relationships.
- `src/main.rs` — added `mod facet;`
- `.doctrine/adr/001/layering.toml` — `facet = "leaf"`
- `tests/e2e_estimate_non_blocking.rs` — added `facet.rs` to NF-001 allowlist

### Design decisions enacted

- D1: `run_show` loads full `DoctrineToml` via `load_doctrine_toml`, single TOML read
- D2: Confidence display formula implemented as `format_estimate_confidence`
- D3: Absent facets → no row (additive only)
- D4: Value is single magnitude, no confidence framing
- D5: `EntityFacets` wraps existing data, no new parse path
- D7: Module-level `#[expect(dead_code)]` removed from estimate display; function-level expects on preserved helpers

### Verification

All VT-1 through VT-12 green:
- VT-5 (golden absent) passed first, proving strictly additive
- VT-1 to VT-4: unit tests for present/absent facets
- VT-9, VT-10: custom confidence bounds and zero-width edge cases
- VT-11, VT-12: shell integration (fixture) and malformed config error propagation
- Full suite: 0 failures attributable to SL-132
- Gate: `just gate` passes (cargo fmt + clippy + layering)

### Notes for audit

- `load_conduct` helper remains in `slice.rs` (used by the status transition code), no duplication
- `format_estimate_normal` and `format_estimate_verbose` preserved per D7 with function-level expects
- Confidence bounds resolved in `run_show` (shell), passed as pure values to `format_show` — follows ADR-001 pure/impure split
- `EntityFacets` struct is ready for SL-133 extension (risk) and SL-136 (tags) per D6
