# Notes SL-286: Codex MCP registers at user level with an absolute path

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open

## Design review passes

- **RV-427** (2026-10-23/24): adversarial pass by pi (quotient `review` profile,
  deepseek-flash, max thinking), run in place of codex (budget). Five rounds,
  20 findings, all verified; the final round raised none.
- **Further pass: none needed before lock.** The bake rule (DEC-404) was the
  only area still producing findings late, and the PATH-identity replacement
  closed it with two small fixes that verified clean. What a further pass would
  probe, if wanted at audit: the consent ordering against install's existing
  `Proceed?` flow in the real code (DEC-381); the `codex_mcp` module boundary
  against ADR-001 layering; and the VH items (EVD-032 recurrence check,
  ASM-014 disclosure visibility).
