# Implementation Plan SL-172: Priority cost skew and bare-estimate anchor

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases. Config knobs land first (PHASE-01), the scoring rewrite consumes
them (PHASE-02), and the operator CLI surfaces them (PHASE-03). The governance
amendment (the REV spanning ADR-015 + SPEC-020) is **not** a plan phase — it is
authored once the design locked and *lands at* `/reconcile` as the durable record
(design §6 OQ-1). The plan is the code change only.

## Sequencing & Rationale

**PHASE-01 — config before consumer.** `EstimateCost { skew, margin }` is pure
plumbing on `PriorityConfig` with fail-safe clamps. Landing it first keeps the
scoring phase focused on the formula, and the knobs are *not* dead code — the
PHASE-01 default/clamp/round-trip tests read both fields. The clamp for `skew` is
a genuinely new path (`[0,1]` cap, not just the existing NaN/inf/negative
handling), so it carries its own verification.

**PHASE-02 — the value delivery.** The cost rewrite is the heart of the slice and
the one phase that moves observable behaviour. It is sequenced after PHASE-01 so
`base_score` can read the knobs. Three things land together here because they are
one causal unit — separating them would leave an intermediate red state:
1. the pure `est_cost` / `floor_eps` / `CostCtx` and the `base_score` signature
   change;
2. the `max_upper` anchor fold (non-terminal only — the F3 fix) in `build_from`,
   before the base pre-pass so mint stays deterministic;
3. the test migration + golden recompute. The midpoint-coupled unit assertions
   and the explain `va1` breakdown are *guaranteed* movers (they hard-code
   midpoint arithmetic), so they migrate in the same phase that changes the
   arithmetic. `channels::value_dim` only reads `base_score.value_dim` — one
   source of truth, nothing parallel to update.

The golden recompute (VH-1) is reviewed for *direction*, not blind-accepted:
bare value-items must sink, and an estimated set's relative order must stay stable
under a β-only change (β=0.5 reproduces the legacy order exactly — the regression
anchor).

**PHASE-03 — operator ergonomics.** Extending `doctrine config` to the two keys
depends only on PHASE-01 (the keys must exist), and is file-disjoint from PHASE-02
(`commands/config.rs` vs `priority/graph.rs` + `priority/surface.rs`). It is
sequenced last as the lowest-risk, lowest-value increment; it could be dispatched
in parallel with PHASE-02 once PHASE-01 is in, if isolation is wanted.

## Notes

- **Behaviour-preservation anchor.** INV-1 (β=0.5 ≡ midpoint) is the proof that
  the rewrite is a faithful generalisation, not a silent re-pricing of the
  mechanism — the *default* re-prices (β=0.65, deliberate D1), but the mechanism
  at β=0.5 is byte-identical to today.
- **Anchor population ≠ mint population** (design §5.4): terminal items are still
  scored (so `inspect`/`explain` report a number) but excluded from the anchor.
  VT-4 pins this so a future refactor can't quietly fold them back in.
- **Governance gate.** No code phase authors the REV; reconcile does. The design
  consult already authorized the direction (lifting SPEC-020's v1 aggregation
  deferral), so implementation is unblocked.
