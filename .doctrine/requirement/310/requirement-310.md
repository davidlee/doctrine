# REQ-310: Confidence band for estimate bounds

## Statement

An estimate's `lower`/`upper` bounds are interpreted as a project-wide **confidence
band**: `lower` sits at percentile P-low, `upper` at P-high. The band resolves from
`doctrine.toml [estimation].lower_confidence`/`upper_confidence`, defaulting `0.1`
and `0.9`; each bound must be finite, in `[0,1]`, with `lower < upper`. The band
**frames** the bounds for authoring and display only — it drives **no** predicate,
aggregation, normalization, or validation in v1. Confidence is **estimate-only**:
there is no entity-local confidence field, and the value facet carries no band.

## Rationale

SL-101 shipped `resolve_confidence` + `DEFAULT_*_CONFIDENCE` as dead code with no
governing requirement. The percentile band is the intended reading of estimate
bounds (user ruling, design D1) — `resolve_confidence` is the resolver, not
accidental residue. Homing it here makes the code spec-traceable and discharges the
RV-114 F-3 placeholder. The v1 display-only constraint keeps the facet policy-free
(SPEC-020's standing contract): interpretation, gating, and ROI arithmetic stay a
caller/Cordage concern. Wiring the band into the `show` display path is deferred to
IMP-112.
