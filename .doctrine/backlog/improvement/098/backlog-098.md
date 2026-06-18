# IMP-098: Zero-finding review derives active instead of done — needs a token round to go terminal

## Context

A review ledger (RV kind) with zero findings should derive `done` — every finding
is vacuously terminal per review-ledger.md §6. Instead the engine derives `active`
with `await=raiser` when `findings = 0, rounds = 0`. The baton never registers a
raiser round completing, so the review stays active.

## Repro

1. `doctrine review new --facet reconciliation --target SL-NNN`
2. Write the brief and synthesis — no findings to raise
3. `doctrine review status RV-NNN` → `active · await=raiser · findings 0 · rounds 0`
4. Expected: `done · await=none`

## Workaround

Raise a token nit finding and immediately withdraw it:

    doctrine review raise RV-NNN --severity nit --title "..." --detail "..."
    doctrine review withdraw RV-NNN --finding F-1 --as raiser

This gives the engine a terminal raiser round and flips `await=raiser` → `done`.

## Where

Review status derivation — likely the baton rebuild logic in `review_status` or
the derived-status computation. The zero-findings case needs to short-circuit to
`done` rather than requiring a raiser round.

## Evidence

RV-055, RV-056, RV-061, RV-066 — all clean reconciliation audits with zero
findings. All required the token-finding workaround to go terminal.
