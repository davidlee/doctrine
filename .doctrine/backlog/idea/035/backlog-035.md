# IDE-035: Multi-dimensional value with per-dimension coefficients

Expand the single `value` facet into named dimensions — e.g. user value,
maintainability, operational cost — each with its own coefficient, generalising
the treatment ADR-015 already gives risk (`risk_dim = exposure × risk_coeff`
as a separate score dimension).

Surfaced during RFC-019 (comparison-based value elicitation) OQ-4
adjudication. RFC-019's ledger deliberately reserves the seam: the optional
`lens` column on comparison rows tags which value dimension a judgement spoke
to, so lens-tagged evidence accumulates before any engine change. If per-lens
evidence demonstrably diverges (the same pair ordered differently under
different lenses), that is the signal this idea is worth building — per-lens
constraint sets become per-dimension constraint sets almost for free.

Scope sketch, if built:

- `[value.<dimension>]` facet storage or a keyed magnitude table per entity.
- `[priority.coefficients]` grows per-dimension entries; `value_dim` becomes a
  coefficient-weighted sum of dimensions (absent dimensions collapse to the
  identity, per ADR-015's absent-facet posture).
- `explain` decomposes the value dimension by named dimension.
- ADR-015 amendment required (formula change, not just input provenance).

Probably not hugely material near-term (per the adjudication); recorded so the
itch and its trigger condition are durable.
