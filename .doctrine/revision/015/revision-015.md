# REV REV-015 — reconcile SL-172

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile narrative for SL-172 (drove by RV-189 finding F-3). The slice shipped the
ADR-015 cost-model rewrite (`est_cost` skew + bare anchor) and the corpus
`max_upper` aggregation; canon still describes the superseded midpoint model and the
v1 aggregation deferral. Both amendments were authorized at the SL-172 design session
(slice-172.md OQ-1: amend, not supersede). This REV lands them.

### [RV-189 F-3] ADR-015 §1+§2+§4 — `value_dim` cost term rewrite (modify)

`est_cost` replaces the inline estimate **midpoint** divisor with a skew +
data-driven bare-anchor model.

Before (ADR-015.md §1, ~L38-65):

> `value_dim` is the authored value adjusted by the global value coefficient, kind
> and tag coefficients, divided by the estimate **midpoint**:
>
> `value_dim = (coefficients.value × value × kind_weight × tag_multiplier) / estimate_midpoint`
>
> - absent estimate ⇒ `estimate_midpoint = 1.0`

After:

> `value_dim` is the authored value adjusted by the global value coefficient, kind
> and tag coefficients, divided by `est_cost`:
>
> `value_dim = (coefficients.value × value × kind_weight × tag_multiplier) / est_cost`
>
> - has estimate ⇒ `est_cost = lower + β(upper − lower)` (skew β, default `0.65`;
>   β = 0.5 recovers the legacy midpoint — INV-1)
> - bare (absent estimate) ⇒ `est_cost = max_upper(corpus) + margin` (default
>   margin `1.0`), a data-driven anchor that dominates non-terminal estimated items
>   (INV-2); empty corpus ⇒ `1.0` fallback
> - `est_cost ≥ EPSILON` always (no div-by-zero — INV-3)
> - β / margin are operator knobs under `[priority.estimate] {skew, margin}`
>   (skew ∈ [0,1], margin ≥ 0)

Rationale: the bare `est_cost = 1.0` cheapest-cost rule made absent-estimate items
outrank everything (ISS-057 inversion); the `max_upper + margin` anchor makes the
inversion structurally impossible.

### [RV-189 F-3] SPEC-020 REQ-310 / FR-011 — lift v1 aggregation deferral (modify)

REQ-310 (confidence band for estimate bounds) stays `active`; what lifts is the
"drives **no** aggregation in v1 / aggregation deferred" prose, now that the corpus
`max_upper` aggregation ships in the priority engine.

Before (SPEC-020 §99-102 + REQ-310 statement):

> the band has no gating, aggregation, or normalization effect … it drives **no**
> predicate, aggregation, normalization, or validation in v1 (REQ-310 / FR-011).

After: retain the no-gating / no-normalization framing of the band itself, but drop
the blanket "no aggregation in v1" deferral — record that corpus-level `max_upper`
aggregation of upper bounds now ships caller-side in the priority cost anchor
(SL-172), consistent with §418's "interpretation/ROI arithmetic stay a caller
concern". The band remains display-framing; the aggregation is the consuming
engine's, not a new facet-schema semantic.
