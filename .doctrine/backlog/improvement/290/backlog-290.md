# IMP-290: Provenance tier for value/estimate facets

**Source:** SL-219 `/elicit` session, 2026-07-13 — engine-design discussion
following a backlog value-refinement pass.

## Problem

~90% of authored `value` and `estimate` facets in the corpus were written by
agents, with **no provenance recorded**. ADR-015's value-source resolution
treats all authored magnitudes as one tier:

> 1. **Authored** `[value]` — *wins outright*, and acts as a point anchor
>    propagating bounds.
> 2. Projected (comparison evidence).
> 3. Default (1.0).

There is no distinction *inside* "authored" between a human anchor and an agent
guess. So an agent-guessed value **wins outright over the projection layer**
(RFC-019 comparison evidence) — the one mechanism that exists to calibrate such
guesses. Agent guesses out-rank the calibration meant to correct them. This is
the root cause of the quarantine knots cleared in this session (agent
comparisons contradicting agent-authored anchors, no provenance to adjudicate).

**Estimates are worse.** `est_cost` is the *divisor* of `value_dim`, so a
guessed estimate scales the entire ranking — and there is **no** projection /
comparison layer for estimates at all (the Phase-E gate; `elicit` refuses to
rank estimate questions). ~90% of the score denominator is un-provenanced agent
guessing with no calibration path.

## Why it matters

Priority `score` is only as trustworthy as its inputs. Today the engine launders
low-confidence agent guesses into authoritative point-anchors that suppress the
human-calibration signal. The more agents groom the backlog, the more the value
signal drifts and the less the comparison ledger can correct it.

## Fix direction (unshaped)

- **Add a provenance/confidence tier to `value` and `estimate`:**
  `human-authored > human-calibrated (comparison) > agent-authored > default`.
- **Flip the resolver:** human-authored wins outright; **agent-authored ranks
  *below* comparison projection**, not above it. Agent guesses become *priors*;
  the human comparison ledger drives. RFC-019 machinery already exists — it is
  currently overridden by "authored wins outright."
- **Extend calibration to estimates** — an estimate-comparison / projection path
  (currently gated out entirely), or at minimum flag agent-authored estimates as
  low-confidence in `explain`.
- Consider **coarsening** authored values to tiers — the disputes in this
  session were all 0.1-resolution (2.5 vs 2.8 vs 3.0), i.e. false precision that
  drives needless calibration churn.

## Cheap diagnostic (baseline)

Force every value to `DEFAULT` (or zero the value coefficient) and diff the
top-N ranking against live. Measures how much the mostly-agent values move the
order — evidence for the tier, and a regression baseline. (Also demonstrates
that "wipe values, rank relationally" regresses to pre-ADR-015 reachability
counting, since `consequence` = leverage + optionality is value-denominated and
`after` carries no score weight.)

## Related

- ADR-015 (multi-dimensional scoring; value-source resolution is the seam)
- RFC-019 (comparison/projection layer being overridden)
- ADR-018 (value-burndown — another value-denominated consumer)
- REV-022 (value-source resolution decisions)
- IMP-289 (sibling facet-integrity gap: burndown blind to ad-hoc completion)
