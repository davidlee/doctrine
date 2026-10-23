# Notes SL-286: Codex MCP registers at user level with an absolute path

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-24 · PHASE-01 · d3ed42648

### Produced

- Plan: plan.toml / plan.md, five phases (924373d8f, 8a768671a). Plan-time
  constraint: codex_mcp imports no command module; callers pass exec, so the
  command tangle baseline holds (see plan.md § Sequencing).
- PHASE-01 done — rootless `serve --mcp` (d81d63eaf). Adds `root::find_opt`,
  `root::DOCTRINE_DIR`, `root::is_doctrine_project`, and the `ProjectRoot`
  resolution in mcp_server. Behaviour change: an explicit `--path` without
  `.doctrine/` is now rootless. Gate green; negative control observed red.
- Minted: ISS-533 — MCP server replies to JSON-RPC notifications (pre-existing).
- Design-time decisions DEC-403..DEC-411; EVD-033; RV-427 (concluded).

### Learned

- mem.pattern.delegation.pi-writable-in-worktree — calibration appended
  (subtree write roots, judgement no-answer turns, transcript budget).
- EVD-033 — `codex mcp add` replaces the whole table; `mcp get` shows the
  effective entry (ASM-015 invalidated).

### Open

- ASM-014 — where codex shows the rootless instructions (VH in PHASE-05).
- RSK-234 · IDE-069 · ISS-528 (partially resolved by DEC-406) · ISS-533.
- Reconcile: REV for REQ-484 (SPEC-011) + rootless-serve requirement;
  supersede DEC-323, DEC-332 → DEC-406 and DEC-325 → DEC-407; README rewrite.

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
