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
- `src/priority/config.rs` — add `β` (skew) and `margin` knobs to
  `Coefficients`/`PriorityConfig` with `f64_or` clamping + defaults (0.65, 1.0).
- **ADR-015 amendment** — the `value_dim` cost term changes at the governance
  tier. Route a Revision against ADR-015 (ADR-013); this slice carries the impl.
- Update goldens: `base_score_absent_estimate_uses_midpoint_one` and the
  `e2e_priority_*` / `e2e_backlog_list_order_golden` fixtures.

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

- **OQ-1 (governance):** Does the ADR-015 cost-term change land as an amendment
  to ADR-015 or a superseding ADR? Resolve in `/design` via the Revision route.
- **OQ-2:** `max_upper` corpus coupling makes `base_score` depend on a
  board-wide aggregate → scores become *relative* (adding a big-estimate item
  lowers every bare item). Confirm acceptable; document the property.
- **OQ-3:** additive `margin` is scale-sensitive (decisive on 1–6, weak on
  0–100). Default `1` accepted; revisit if a multiplicative margin is wanted.
