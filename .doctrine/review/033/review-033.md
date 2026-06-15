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

## Synthesis

SL-070 is conformance-clean with one deviation found and fixed inline.

**Evidence:** 1324 tests pass, zero clippy warnings, `just check` green. All 10
column-definition sites correctly widen `Fixed(AnsiColors::Cyan)` →
`Fixed(DynColors::Ansi(Cyan))`. No bare `AnsiColors::Cyan` survives (VT-8). All 8
title-column surfaces wire `Alternate([TITLE_EVEN, TITLE_ODD])` correctly;
REQ_COLUMNS and COVERAGE_COLUMNS correctly omit it (no title column). All 3
`ByValue` hue maps (backlog kind, memory type, memory trust) are wired and tested.
`CoverageRow::status_hue` return type widened to `Option<DynColors>` (VT-9).

**F-1 (major, verified):** TITLE_EVEN/TITLE_ODD RGB tuples in `src/listing.rs`
did not match design.md §3. The comments on the constants claimed
`#ebdbb2`/`#d5c4a1` (matching the design), but the Rgb tuples were
`(235,235,235)`=#ebebeb and `(215,184,57)`=#d7b839 — incorrect values. Fixed
in-place during audit; corrected to `Rgb(235,219,178)`/#ebdbb2 and
`Rgb(213,196,161)`/#d5c4a1. Tests remain green.

**F-2 (minor, tolerated):** EX-3 visual verification cannot be executed in this
environment (no display). All automated VT criteria pass; a human must verify
rendering on a real terminal before final close.

**Standing risks:**
- The gruvbox tag palette uses 24-bit truecolour — terminals older than ~2015
  will show fallback colours. Accepted per design risk assessment.
- TITLE_EVEN/TITLE_ODD contrast may be too subtle on very bright terminals —
  noted in implementation notes; feedback loop open.

**Closure readiness:** The review ledger is resolved (2 findings, both
terminal). Automated gates are green. Hand-off to `/close` is unblocked once
visual EX-3 is confirmed by a human.
