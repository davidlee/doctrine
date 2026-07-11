# Per-component gauge scope

## Context

SL-213 shipped the tier-3 projection gauge (design §3 P8) with **global**
corpus-H: the anchor-free gauge spread normalises by corpus-wide max height,
and fires only when the corpus has no anchors anywhere. RV-266 F-3 adjudicated
that as prototype-validated; IMP-279 records the reconsidered refactor this
slice executes: key the gauge **per weakly-connected component**.

Compatibility analysis (IMP-279, from RV-266 follow-on discussion 2026-07-11):
of the 22 validated scenarios only S2 has ≥2 disjoint components — the
divergence surface is nearly unconstrained by design evidence. Per-component
scope preserves P10/P11/P9, satisfies P12 locality **strictly** (the property
test can be unscoped and strengthened), and honours P8's original "centred"
wording — global-H compresses shallow islands into the low band (S2 pendant
mean 0.6, not centred on DEFAULT).

## Scope & Objectives

- Re-key the P8 gauge branch in `src/comparison/project.rs` per component
  (height `h` and normaliser `H` computed within each weakly-connected
  component, not corpus-wide).
- Adjudicate at design time the one real behavioural change: in a mixed corpus
  an anchor-free island currently takes P7 flat DEFAULT + P5 ladders (losing a
  judgement never drops below the unjudged baseline); per-component P8 gives a
  centred spread where the loser sits BELOW default. Consistent with the
  validated pure-gauge regime (0.4 < 1.0), but flips "lost one comparison"
  from neutral to negative relative to unjudged entities.
- Re-pin the `s2_partial_order_gauge` golden (pendant 0.8/0.4 → 1.333/0.667).
- Unscope and strengthen the P12 locality property test.
- Add a mixed-corpus-island golden.
- Reword SL-213 design.md §3 P1/P8/P12 back to component scope (mechanism per
  design conversation — direct amendment vs revision; the design doc belongs
  to a closed slice).

## Non-Goals

- No change to tiers 1–2 (ledger, row-validity resolution, constraint
  compilation) or to the anchored regime (P3–P6, P13, P14).
- No change to `GaugeConfig` constants or their homing.
- No elicitation/ratio-magnitude work (OQ-6 stays open).
- No live-data migration machinery — single-component corpora are unaffected;
  risk is low.

## Risks, Assumptions, Open Questions

- **R1** Mixed-regime semantics change is user-visible in `explain` output;
  needs explicit design adjudication (the reason this is sliced, not a quick
  fix).
- **A1** Weakly-connected-component identification is already available or
  cheap in the projection's merged-class graph.
- **OQ-1** How to amend a closed slice's design doc (SL-213 design.md §3) —
  direct edit vs revision routing; settle in `/design`.

## Verification / Closure Intent

- Re-pinned S2 golden green; new mixed-corpus-island golden green.
- Strengthened (unscoped) P12 locality property test green.
- Existing anchored-regime and single-component goldens unchanged — the
  behaviour-preservation gate for untouched regimes.
- `just gate` clean.

## Summary

## Follow-Ups
