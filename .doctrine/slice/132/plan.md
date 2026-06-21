# SL-132 implementation plan — rationale and sequencing

## Why one phase

The change is small (~150 lines across 5 files) and all parts are mutually
dependent:

- `EntityFacets` struct is consumed by `format_show`
- `format_estimate_confidence` and `format_value_normal` are called by
  `format_show`
- `run_show` must construct `EntityFacets` and resolve units before
  passing to `format_show`
- The `dead_code` expects must come off for the module to compile
- The `layering.toml` entry must exist for `just gate` to pass

None of these can ship independently — they form a single coherent
delivery. Splitting into multiple phases would create artificial
red/green cycles where intermediate states don't compile or fail gate.

## Implementation order

Within the single phase, work proceeds in dependency order:

1. **Create `src/facet.rs`** — `EntityFacets` struct (leaf tier, pure data)
2. **Add `layering.toml` entry** — `facet = "leaf"` so gate passes
3. **Ungate `estimate::display`** — remove module-level `#[expect(dead_code)]`,
   add function-level gates on `format_estimate_normal`/`format_estimate_verbose`
4. **Write `format_estimate_confidence`** — confidence-framed display with
   configurable percentile bounds
5. **Write `format_value_normal`** — magnitude + unit display
6. **Wire `run_show`** — replace `load_conduct` with `load_doctrine_toml`,
   construct `EntityFacets`, resolve units, pass to `format_show`
7. **Update `format_show`** — accept `&EntityFacets` + unit strings, render
   estimate/value rows after dates, before relationships
8. **TDD** — golden test first (VT-5: both absent → byte-identical), then
   additive tests (VT-1 through VT-4), then edge cases (VT-9, VT-10),
   then integration (VT-11, VT-12)

## Verification strategy

- **VT-5 first** (golden absent-facet test): proves the change is strictly
  additive. Capture current `format_show` output, assert unchanged.
- **VT-1 through VT-4**: unit tests for `format_show` with facets present/absent.
  These test the pure formatter, not the shell.
- **VT-9, VT-10**: edge cases — custom confidence bounds, zero-width estimate.
- **VT-11**: integration test — `run_show` against a fixture directory with
  slice TOML + `doctrine.toml`.
- **VT-12**: integration test — malformed `doctrine.toml` propagates error.
- **VT-6, VT-7, VT-8**: gate checks — no dead_code warnings, existing tests
  unchanged, `just gate` passes.

## Non-goals for this plan

- No backlog/governance show path changes
- No risk facet display
- No `survey`/`next` output changes
- No verbose estimate display mode (existing helpers preserved, not wired)
