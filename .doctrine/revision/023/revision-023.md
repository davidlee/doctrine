# REV REV-023 — ADR-015 estimate provenance amendment

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-219 (estimate comparison domain, RFC-019 § domains/frames) gives `est_cost`
a second provenance: items without authored bounds but with sizing evidence in
an anchored component take a deterministic point projection from the estimate
constraint layer. That changes the *source* of the `est_cost` divisor — the
score formula itself does not change. The amendment mirrors REV-022 (value
provenance) on the cost side, and pins four policy points the SL-219 design
locked with the user (Q1=A, Q2=A, 2026-07-13; design D1/D2/D6/D11):

1. **Estimate-source resolution** — a three-tier ladder, *source precedence
   only, no numeric-dominance claim*: authored `[estimate]` bounds are an
   **operator pin** (a policy override over accumulated sizing evidence) that
   wins outright and acts, β-resolved, as a point anchor in the estimate
   constraint system; projection fills evidenced absence; the bare anchor
   fills the rest (unchanged).
2. **INV-2 restated.** The current wording — the bare anchor "dominates every
   non-terminal estimated item" — is false once projected costs feed scoring:
   a projection may legitimately exceed the bare anchor (evidence that an
   un-estimated item is *more* work than everything authored). The invariant's
   anti-inversion intent survives precisely: a bare item with NO sizing
   evidence keeps the dominating divisor.
3. **Gauge never divides.** Anchor-free components carry only a conventional
   (render-only) gauge ordering; conventional magnitudes may fill a numerator,
   never a denominator — a near-zero conventional divisor would explode
   `value_dim` on no evidence.
4. **Positivity axiom.** The estimate feasible region is order constraints +
   point anchors + `c > 0`. Settle-cost is a nonnegative physical quantity;
   every anchor and every fed cost is strictly positive (EPSILON floors at the
   formula and consumption seams are the axiom made concrete, subsuming
   INV-3's div-by-zero concern for this domain).
5. **Operator knobs.** `gauge_step` joins β and `margin` under
   `[priority.estimate]`; an edit to any of the three re-runs est projection →
   resolved costs → value determinacy (one reprobe dynamic spans both
   domains).

Scope of the staged delta: one `modify` row (ADR-015). Applied as a
surfaced-for-manual hand-edit after approval; `doctrine boot` after landing.
Approval is the SL-219 PHASE-04 human gate (EN-2/VH-1): the `est_cost` ladder
commit must not land before this REV is approved.

### Change row 1 — modify ADR-015

**Edit 1.1 — §1 Base dimensions › `est_cost` formula block (add the projected
tier).**

Before:

> ```text
> est_cost = lower + β(upper − lower)         # has estimate (skew β, default 0.65)
>          = max_upper(corpus) + margin       # bare (absent estimate; margin default 1.0)
>          = 1.0                              # empty corpus fallback
> ```

After:

> ```text
> est_cost = lower + β(upper − lower)         # authored estimate (skew β, default 0.65)
>          = point projection                 # no authored estimate; sizing evidence in an
>                                             #   anchored component (see Estimate-source resolution)
>          = max_upper(corpus) + margin       # bare (no authored estimate, no fed projection;
>                                             #   margin default 1.0)
>          = 1.0                              # empty corpus fallback
> ```

**Edit 1.2 — §1 Base dimensions › the `est_cost` paragraph (INV-2
restatement).**

Before:

> `est_cost` replaces the former plain estimate **midpoint** divisor. With an
> estimate, cost skews between the bounds by β (β = 0.5 recovers the legacy
> midpoint — INV-1). Bare items anchor on the corpus `max_upper` plus a
> `margin`, a data-driven divisor that dominates every non-terminal estimated
> item (INV-2) — so a bare item can no longer be priced cheapest and outrank
> everything (the ISS-057 inversion). `est_cost` is floored at `EPSILON` (no
> div-by-zero — INV-3). β and margin are operator knobs (§4,
> `[priority.estimate]`).

After:

> `est_cost` replaces the former plain estimate **midpoint** divisor. With an
> authored estimate, cost skews between the bounds by β (β = 0.5 recovers the
> legacy midpoint — INV-1). Bare items anchor on the corpus `max_upper` plus a
> `margin`, a data-driven divisor that dominates every **authored** estimate
> (INV-2, restated by REV-023): projected costs may exceed or undercut the
> bare anchor — evidence, not the default, then sets the divisor. The original
> anti-inversion intent is preserved exactly: a bare item with *no sizing
> evidence* keeps the dominating divisor, so it can never be priced cheapest
> and outrank everything (the ISS-057 inversion). `est_cost` is floored at
> `EPSILON` (no div-by-zero — INV-3; see also the positivity axiom below).
> β and margin are operator knobs (§4, `[priority.estimate]`).

**Edit 1.3 — §1 Base dimensions › absent-data list › estimate bullet.**

Before:

> - absent estimate ⇒ `est_cost = max_upper(corpus) + margin` (the bare anchor;
>   `1.0` when the corpus is empty);

After:

> - absent authored estimate ⇒ resolves through the estimate constraint
>   layer's projection first (see *Estimate-source resolution*); with no fed
>   projection, `est_cost = max_upper(corpus) + margin` (the bare anchor;
>   `1.0` when the corpus is empty);

**Edit 1.4 — §1 Base dimensions › new subsection after *Value-source
resolution*: estimate-source resolution.**

Insert:

> ### Estimate-source resolution (REV-023, SL-219)
>
> The `est_cost` input to `value_dim` resolves by provenance — **source
> precedence only, no numeric-dominance claim**:
>
> 1. **Authored** `[estimate]` bounds — an **operator pin**: a policy override
>    over accumulated sizing evidence. Wins outright, and additionally acts
>    (β-resolved) as a point anchor in the estimate constraint system,
>    propagating cost bounds to items compared against it.
> 2. **Projected** — items without authored bounds but with sizing evidence
>    in an anchored component take the deterministic point projection within
>    their derived bounds (RFC-019 constraint layer, estimate domain).
> 3. **Bare anchor** — items with neither take `max_upper(corpus) + margin`
>    (`1.0` empty-corpus fallback).
>
> **Gauge never divides.** Anchor-free components (sizing evidence but no
> authored estimate anywhere in the component) receive only a conventional,
> render-only gauge ordering (spacing `gauge_step`, centred on the bare
> anchor). Conventional magnitudes may fill a numerator, never a denominator:
> gauge-tier placements are never fed to `est_cost` — those items score at
> the bare anchor, and surfaces must never imply otherwise.
>
> **Positivity axiom.** The estimate feasible region is order constraints +
> point anchors + `c > 0`. Settle-cost is a nonnegative physical quantity:
> every anchor is strictly positive (the β-resolved authored cost floors at
> `EPSILON`) and every fed projection is strictly positive (`EPSILON` floor
> at the consumption seam). Sizing evidence packing items below a small
> anchor therefore compresses into `(0, anchor)` — a forced consequence of
> evidence + positivity; the spacing within the interval is convention,
> disclosed by provenance labels and bounds display.
>
> An authored estimate that conflicts with sizing-derived bounds is a
> surfaced contradiction (the likeliest defect is a stale estimate —
> loudness is a feature), not a silent win. Projected costs and bounds are
> derived at read time from the comparison ledger; they are never authored
> to disk (the same posture as scores and projected values).

**Edit 1.5 — §4 Config contract › `[priority.estimate]` block comment.**

Before:

> ```toml
> [priority.estimate]
> # skew (β, [0,1], default 0.65), margin (≥ 0, default 1.0)
> ```

After:

> ```toml
> [priority.estimate]
> # skew (β, [0,1], default 0.65), margin (≥ 0, default 1.0),
> # gauge_step (render-only gauge spacing, default 0.25)
> ```

**Edit 1.6 — §4 Config contract › append to the durable-domains list.**

> - estimate-source resolution order (authored → projected → bare anchor) is
>   fixed policy, not a knob; projection is deterministic given (ledger,
>   config, anchors); gauge-tier magnitudes cannot be configured into the
>   divisor;
> - β, `margin`, and `gauge_step` are operator knobs; an edit to any re-runs
>   estimate projection → resolved costs → value determinacy (one reprobe
>   dynamic spans both domains).

**Edit 1.7 — References › append.**

> - SL-219 — estimate comparison domain; implements the estimate constraint
>   system, cost feed, and the `est_cost` ladder this amendment admits.
> - REV-023 — this amendment (estimate provenance; INV-2 restatement;
>   gauge-never-divides; positivity axiom).
