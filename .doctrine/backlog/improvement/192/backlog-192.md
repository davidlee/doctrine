# IMP-192: Solo slice reaches close with stale runtime phase-rollup + noisy binding warning

A slice driven **without dispatch** (solo or lean-serial) never moves its phases
`planned → in_progress → completed` in runtime state — that flip is the
`/execute`/dispatch path's job. So such a slice arrives at `/close`
substantively done (work implemented, RV verified, gate green) but with a stale
rollup (e.g. `0/2`, both sheets `planned`). The close pre-check "confirm X/X
complete" then mis-reads a finished slice as incomplete.

Compounding: when the rollup is then flipped manually
(`slice phase NNN PHASE-0N --status completed`), each flip emits
`phase-binding capture skipped … no code_start_oid stamped` — noise, because the
phase legitimately never entered `in_progress` under a binding on this path.

Fork-driven slices have a related variant: runtime phase state lives in the
fork's gitignored `.doctrine/state/` and never propagates to primary, so the
primary rollup is stale by design (close reads `2/5` for a done slice).

## Cost (RFC-011 instrumentation)

Recurred at SL-166 close, SL-163 close, SL-166 orientation. ~4 investigative
tool calls each time to reconstruct "where does phase state live / why N/M / how
to reconcile" before the picture is clear.

## Proposal

- `/close` (and/or `/audit`) recipe for "solo/fork slice, audited-done, runtime
  rollup stale → reconcile the rollup before transition", so the pre-check does
  not read an expected-stale rollup as dropped work.
- Suppress the `phase-binding capture skipped` warning on the legitimate
  no-binding flip path (or downgrade to info).

Surfaced by: RFC-011 case-notes. Platform issue — the lifecycle/runtime-state
split and the close skill ship with doctrine; affects every user driving a slice
outside dispatch.
