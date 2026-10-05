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

## Token budget (PHASE-05 VA-1)

- 2026-10-05 · `review show RV-012` (five findings, default depth), bytes/4
  estimate (no tokenizer in the jail): table ≈294 tokens (1175 B), CLI JSON
  ≈497 (1989 B pretty; 1467 B compact ≈367). RFC-032 D5 budget ~200: over.
- Finding titles are about half the table; not cuttable without losing
  content. Remaining levers are small (default role-label line, `findings: N`
  line, zero-filled `outstanding` maps in JSON) or CLI-wide (pretty-printed
  `--json` whitespace, ≈26%) and out of this slice's scope.
- **Accepted by the user** (2026-10-05: "accept it, unless there's an obvious
  way to reduce the token count without omitting important information") —
  no obvious lossless reduction found.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
