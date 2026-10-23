# Notes SL-286: Codex MCP registers at user level with an absolute path

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-24 · plan (design locked rev 41) · 1f0f8c67e

### Produced

- Decisions: DEC-403, DEC-404, DEC-405, DEC-406, DEC-407, DEC-408, DEC-409,
  DEC-410, DEC-411
- Evidence: EVD-033 · Assumptions: ASM-014 (open), ASM-015 (invalidated)
- Backlog: IDE-069, RSK-234 · Review: RV-427 (concluded, 20/20 verified)
- design.md (locked)

### Learned

- EVD-033: `codex mcp add` replaces the whole server table; `codex mcp get`
  shows the effective (merged) entry, not the user layer (ASM-015).
- Friction observations recorded: ISS-356 and ISS-512 recurrences; route
  requirement on severe findings surfaces only at lock.

### Open

- ASM-014 (VH at verification) · IDE-069 · RSK-234 (hooks still bake
  current_exe) · ISS-528 (partially resolved by DEC-406)
- Reconcile: REV for REQ-484 (SPEC-011) + rootless-serve requirement;
  `doctrine supersede` DEC-323, DEC-332 → DEC-406 and DEC-325 → DEC-407;
  README rewrite (see design § Revision and reconcile).

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
