# Notes SL-278: Review read surface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design review passes

- 2026-10-04 · RV-416 (adversarial, GPT) concluded: F-1..F-6 verified, all
  integrated. No further pass recommended before lock. A second pass would
  probe: (a) the census axis split — that `FindingFilter` and the text-only
  `listing::retain` together reproduce the list hide-set exactly; (b)
  `ReviewOutput::Showed { #[serde(flatten)] view }` producing byte-identical
  inner JSON to the CLI's `{"review": view}`. Both are cheaper to settle by the
  sec-8 tests (`census_and_show_agree_on_finding_axes`,
  `cli_json_and_mcp_show_carry_identical_view`) during execution than by
  another reading.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
