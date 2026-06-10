# SL-037 — implementation notes

Durable, committed. Harvested from disposable phase sheets at each phase end.

## PHASE-01 (commit `4e56756`, branch `sl-037-phase-01`)

- **R1 canary clean.** Governance migrated with zero per-kind config on
  `Column<R>`; extractors stayed trivial non-capturing `fn(&GovRow)->String`.
  Weak evidence only (gentlest kind) — slice markers (P2) and spec subtype ids
  (P3) are the real over-config tests.
- **A4 held.** `GovRow` feeds both table and JSON from one materialisation
  (`gov_rows`); no display-row split needed for governance.
- **Golden repins (R2, intended).** `e2e_adr_cli_golden` + `e2e_standard_cli_golden`
  table goldens (default + `--all`) repinned slug-free per D4. Their JSON goldens
  passed UNCHANGED — the live D2 proof. Expect the same shape per kind in P2/P3.
- **Memory guard pinned.** `tests/e2e_list_conformance.rs` black-box test asserts
  `memory list --columns` fails with "--columns is not supported for `memory list`"
  and that the rejection is the guard, not a clap parse error (D9/R4, VT-3).
  IMP-017 removes the guard when memory adopts the model.
- **Column<R> derive posture.** No `derive(Copy)` (design) and no `derive(Debug)`
  either — both add spurious `R:` bounds; tests assert errors via
  `.err().map(|e| e.to_string())` instead of `unwrap_err`.
- **dead_code arc.** Leaf-before-consumer trips `-D unused` mid-TDD-chain; when
  the consumer lands in the SAME phase, finish the chain before running the gate —
  no `cfg_attr(not(test), expect(dead_code))` needed (that pattern is for
  cross-phase gaps; see mem `dead-code-expect-vs-cfg-test`).
