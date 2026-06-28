# Priority cost skew and bare-estimate anchor

Resolves ISS-057. Amends the ADR-015 `value_dim` cost term.

## Context

`value_dim = (coeff.value × value × kind_weight × tag_term) / est_cost`
(ADR-015 §1). Today `est_cost` is the plain estimate **midpoint**
(`graph.rs:84`), and a *bare* (no-estimate) item is hard-coded to
`est_mid = 1.0` (`graph.rs:87`) — the cheapest-possible cost. Two defects:

1. **Bare-item inversion (ISS-057).** A value-bearing item with no estimate is
   priced as the cheapest item on the board, so it outranks items carrying
   honest estimates. The estimator is penalised. Observed on IMP-120.

2. **Midpoint mis-prices every estimated item.** Software estimate→actual is
   right-skewed (long tail): expected cost > midpoint. `(lower+upper)/2`
   systematically under-costs *all* estimated items, not just bare ones. The
   bare inversion is the visible symptom of the same modelling error.

Decided cost model:

```
est_cost(item) =
    has_estimate:  lower + β·(upper − lower)        # β skew; β=0.5 ≡ legacy midpoint
    bare:          max_upper(corpus) + margin        # data-driven anchor
value_dim = (coeff.value × value × kind_weight × tag_term) / est_cost
```

- **β (skew)** ∈ [0,1], default **0.65** — skew on by default (the long-tail
  thesis applies to known items too; β=0.5 recovers today's behaviour exactly as
  the migration anchor).
- **margin**, default **1** (additive). Anchor is `max_upper` across estimated
  items in the scored set, `+ margin` → a bare item always costs more than the
  worst *estimated* item. Self-scaling; needs no fixed 1–6 scale assumption.
- **Empty-corpus fallback**: no item carries an estimate → `max_upper` undefined
  → fall back to `1.0` (legacy behaviour, no inversion possible).

## Scope & Objectives

- `src/priority/graph.rs` — replace midpoint with `est_cost`; compute
  `max_upper` corpus aggregate in the scan pre-pass and thread it into
  `base_score` as an input (preserve the pure-layer / date-uid pattern — no
  corpus read inside the pure fn).
- `src/priority/config.rs` — new `[priority.estimate]` `{ skew, margin }` on
  `PriorityConfig`, `f64_or` clamping + defaults (0.65 / 1.0), skew∈[0,1],
  margin≥0.
- `src/commands/config.rs` — extend `doctrine config show/set/get/unset` to the
  two new keys (operator surface parity; codex F4).
- **Governance amendment (REV)** — the change rewrites **ADR-015 §1+§2+§4** *and*
  lifts **SPEC-020 REQ-310 / FR-011**'s v1 aggregation deferral. Authorized by
  consult in the design session; REV authored at lock, landed at `/reconcile`.
- Anchor folds **non-terminal** entities only (`status_class != Terminal`) so
  closed work cannot poison live ranking (codex F3).
- Update tests: retire `base_score_absent_estimate_uses_midpoint_one`, migrate the
  midpoint-coupled `base_score_*` assertions + explain `va1`, recompute
  `e2e_priority_golden` / `e2e_priority_cross_kind`. (`e2e_backlog_list_order` is
  NOT score-sorted — out of scope.)

## Non-Goals

- **Visibility / `⚠ no estimate` column** — owned by SL-171 (`next` columns).
  This slice is scoring only; no list/next/survey render change.
- Risk-dim, leverage, optionality, consequence — untouched.
- Changing the estimate facet schema or its bounds semantics.
- β > 1 tail-extrapolation past `upper` — out of scope; β capped at 1.0.

## Summary

(filled at close)

## Follow-Ups

## Open Questions

- **OQ-1 (resolved):** Amend, not supersede — REV spans ADR-015 §1+§2+§4 **and**
  SPEC-020 REQ-310/FR-011 (v1 aggregation deferral, intentionally lifted via
  consult). Authored at design-lock, landed at `/reconcile`.
- **OQ-2 (resolved):** corpus coupling makes scores *relative* — accepted; the
  non-terminal anchor population bounds it (closed items can't poison live order).
- **OQ-3 (resolved):** additive `margin`, default `1`; multiplicative margin is a
  future knob, not this slice.
