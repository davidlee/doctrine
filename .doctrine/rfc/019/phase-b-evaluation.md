# RFC-019 Phase B inference — empirical evaluation

**Chore:** CHR-042 · **Gates:** IMP-280 (Phase C) via `needs` · **Date:** 2026-07-11
**Subject:** the Phase B inference layer shipped in SL-213 (three-tier
evidence → constraint → projection), run against a genuine pairwise-value
ledger over this repo's own live backlog.
**Verdict:** **entry criterion met with caveats** — Phase C may open; the
caveats below are Phase C design inputs, not Phase B defects. No new issues filed.

---

## 1. Method

A ledger of **32 genuine pairwise judgements** was recorded with
`doctrine compare record` over **19 live backlog items** spanning four themes I
can honestly judge (dispatch/correctness, gate/architecture, elicitation/priority,
read-ergonomics). All rows are `rater=agent` (disclosed per RFC-019 T7,
`by=opus48-chr042`), frame `equal-effort` (value domain), judging intrinsic
value-for-the-project with effort held equal and consequence excluded (A1).

The full v2 response vocabulary was used where honest: `prefer-a/prefer-b`
(order), `equal` (one pair genuinely indistinguishable), `incomparable` (two
pairs where the value axes genuinely don't commensurate — ecosystem reach vs
internal hygiene).

**Honesty discipline (per the chore's constraint):** no cycles or conflicts were
manufactured. Anchor conflicts that arose are *real* disagreements between honest
comparative judgement and pre-existing **fiat** `value` scalars — precisely the
"abstract judgement made too early" RFC-019 argues against. Genuine transitive
judgement produced **no intransitivity**, so the SCC/cycle path is un-exercised
here (see §5). The ledger design deliberately seeded two regions: a
consistent projection region (to observe bounds/projection/gauge) and a region
where honest judgement contradicts stale anchors (to observe D4 degradation).

Ledger lives at `.doctrine/comparisons/` (authored tier, committed). Observed via
`doctrine explain <ID>` (value-source line), `doctrine compare list`, and
`doctrine findings`, using the tree-local `./target/debug/doctrine`.

### Resolution-status distribution (32 rows)

| status | count | share | mechanism |
|---|---:|---:|---|
| `active` | 21 | 66% | compiled into constraints |
| `quarantined(anchors)` | 9 | 28% | D4 violation-closure (anchor conflict) |
| `no-constraint` | 2 | 6% | `incomparable` — captured, zero constraint |
| SCC-quarantined | 0 | 0% | no genuine intransitivity arose |

(One further `tombstoned` row exists — a locality-probe judgement withdrawn after
the stability test in §4; the withdraw fully reverted state.)

---

## 2. Useful? — **yes**, projection discriminates and is legible

The value-source line carries exactly what the RFC promised — provenance,
bounds, rater-kind counts, and a calibration hint:

| item | value-source line | reading |
|---|---|---|
| IMP-280 | `projected · bounds (2.5 ‥ 2.8) · from 5 judgements (0h/5a)` → **2.6** | tight two-sided bracket |
| IMP-270 | `projected · bounds (2.4 ‥ 2.5) · from 2 judgements` → **2.5** | very tight |
| IMP-246 | `projected · bounds (1.5 ‥ 2.5) · from 5 judgements` → **2.0** | mid, well-placed |
| IMP-247 | `projected · bounds (unbounded ‥ 2.5) · from 2` → **1.3** | one-sided ceiling |
| IMP-273 | `projected · bounds (unbounded ‥ 1.3) · from 2` → **0.7** | one-sided ceiling |
| IMP-245 | `gauge · no anchor in component · set a value to calibrate` → **1.0** | correct default degradation |

Bounds discriminate item-to-item (they do **not** collapse to near-uniform);
the projected point sits sensibly inside each bracket; anchor-free disconnected
components fall back to `DEFAULT_VALUE = 1.0` under the gauge convention (D10),
labelled distinctly from `projected`. Rater-kind counts render honestly
(`0 human, N agent`) — the T7 elicitation lever (prioritise *human confirmation*
of load-bearing agent-seeded orderings) has real data to act on.

**Caveat C1 (end-to-end, not Phase B).** Projection moves the *value* tier, but
the *score* tier can mask it: `value_dim = value / est_cost`, and most of the
non-anchored projection targets are **bare (un-estimated)** items whose
`est_cost` anchors to `max_upper + margin`. IMP-280 projects to value 2.6 yet
its `value_dim` is 0.1 (est_cost ≈ 26). This is the ADR-015 bare-item
denominator, orthogonal to Phase B — but it means the projection's payoff in
`next`/`survey` is only visible on items that **also** carry estimates. Phase B
did its job; the pipeline's estimate side gates the observable win.

---

## 3. Degradation — **sane and actionable**, with one sharp, load-bearing edge

Anchor conflicts fire correctly and **anchors win** (REV-022 / D4). `findings`
decomposes them per anchor-pair, names the exact quarantined row uids, and states
the explicit exits:

```
anchors IMP-198=3.6 vs IMP-274=5.0 conflict — quarantines {…6 uids…};
  exit: supersede a conflicting row, tombstone one, or edit an anchor
```

`explain` mirrors it on the item: `value 5.0 — authored (see anchor-conflict
finding: IMP-198, IMP-236, IMP-255)`. The gauge-disconnect finding names IMP-245
with its own exit. This is actionable direction, not a wall of pairs.

**The sharp edge — D4 closure is broad, and that interacts with a densely
pre-anchored corpus (the key finding).** Violation-closure (D4) quarantines
*every edge participating in any anchor-contradicting structure*, not just the
directly-contradicting rows. One stale anchor — **IMP-274 = 5.0**, which honest
judgement placed mid-pack — quarantined **6 of the 9** inert rows, including
judgements that are themselves anchor-*consistent* (e.g. IMP-236 ≻ IMP-255,
anchors 2.8 > 2.5) but sit inside the forced-floor/ceiling closure. Net: **28%
of a genuine ledger went inert off effectively three stale-anchor tensions**
(IMP-274 grossly high; IMP-236 ≺ IMP-257 by a hair; IMP-260 ≻ IMP-263 vs an
honest `equal`).

This is D4 **by design** — "loudness is a feature; the likeliest defect is a
stale anchor" — and the design intent is vindicated: the corpus *is* full of
indefensible fiat scalars (nearly every backlog item carries one), which is
RFC-019's founding thesis. But the empirical *rate* is a Phase C input: over a
dense-anchor corpus, honest evidence that grazes even one stale anchor
sterilises a disproportionate slice of the ledger. **Phase C's question-selector
must treat stale-anchor resolution as first-class**, or elicitation will spend
much of its evidence budget in closure quarantine rather than firming bounds.

---

## 4. Stable? — **yes**, determinism and locality both hold

- **Determinism.** Two consecutive runs over the same merged file set produced
  **byte-identical** active rows, bounds, projections, and residual findings.
- **Locality (P10–P15).** Adding one edge (IMP-247 ≻ IMP-245, pulling the
  gauge-isolated IMP-245 into the ordered graph) perturbed **only its
  weakly-connected component**: IMP-245 gauge 1.0 → projected 1.6; IMP-247
  1.3 → 1.9; IMP-246 2.0 → 2.2 (budgeted interpolation re-spacing the component,
  D9). **Bounds — the hard invariant — held**; projected *points* moved ≤ 0.6
  strictly within unchanged bounds; the other 15 items were byte-identical.
  Withdrawing the probe fully reverted every value. No wild reorder: the P10–P15
  stability contract holds empirically.

---

## 5. Coverage limits (honest)

- **SCC / preference-cycle degradation (D3) is un-exercised.** Genuine transitive
  judgement produced no intransitivity, and the chore forbids manufacturing one.
  Assurance for that path rests on the SL-213 design + `projection-prototype.py`
  scenario battery, not this ledger.
- **All rows are agent-rater.** Human-vs-agent trust weighting (T7 / D7's
  rank-aware quarantine) is present in the surfaces (`0 human, N agent`) but not
  differentially exercised — the demotion knob is a later seam (D7).
- **Ratio frame / `prefer-first` priority domain** not exercised — value domain
  `equal-effort` only, matching Phase B's shipped constraint compiler.

---

## 6. Verdict and Phase C inputs

**Entry criterion: met with caveats.** Phase B inference is *useful* (bounds
discriminate; provenance is legible), *stable* (deterministic + local), and
*degrades sanely and actionably* (anchors win; findings name culprit + exits).
IMP-280 (Phase C) may open. Caveats — all Phase C design inputs, none a Phase B
defect:

1. **C1 — estimate-gated payoff.** Projection's win is visible in score/`next`
   only on items that also carry estimates; bare items sink on `value_dim`
   regardless of a good projected value. (ADR-015 interaction, not Phase B.)
2. **C2 — quarantine rate over a dense-anchor corpus.** D4 closure is broad by
   design; over this corpus of fiat anchors, honest evidence brushing one stale
   anchor took 28% of the ledger inert. Phase C's **selector must prioritise
   stale-anchor resolution** (surface the anchor as the likeliest defect, route
   to edit/supersede) so the evidence budget firms bounds instead of pooling in
   closure. This is also the strongest empirical argument *for* Phase C.
3. **C3 — cycle path unverified empirically.** Carry the SL-213 prototype battery
   as the standing assurance; consider a Phase C dogfood that deliberately elicits
   a contested triad once human raters are in the loop.
