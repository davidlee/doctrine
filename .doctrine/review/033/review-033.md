# Review RV-033 — reconciliation of SL-070

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-070 "CLI list UI coloring & improvements".
Probes:
- **Type widening conformance** — every `Fixed(AnsiColors::Cyan)` became
  `Fixed(DynColors::Ansi(Cyan))`; no site missed; existing goldens green.
- **Gruvbox palette** — 12 Rgb entries, segment_hue widened, paint_tag updated.
- **Alternating title** — TITLE_EVEN/TITLE_ODD wired on 8 surfaces; Alternate
  early return in paint_cell.
- **Hue maps** — backlog_kind_hue (5 kinds), memory_type_hue (6 types), trust_hue
  (3 levels) wired via ByValue.
- **Behaviour-preservation gate** — 1323 tests pass unchanged, just check clean.

Invariants:
- DynColors::Ansi(Cyan) emits byte-identical ANSI to bare AnsiColors::Cyan.
- color=false path is byte-clean (no ANSI) — VT-2/3/4 hold.
- --json output is untouched (colours are table-only).
