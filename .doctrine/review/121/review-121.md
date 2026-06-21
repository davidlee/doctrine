# Review RV-121 — design of SL-132

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This design inquisition interrogates SL-132's proposed estimate/value display for `doctrine slice show`.

Lines of attack:

- whether `EntityFacets` is a clean shared projection or a second authority beside the existing catalog scan/hydrate facet projection;
- whether confidence-bound display is specified for zero-width estimates, extreme valid confidence bounds, and invalid config;
- whether unit resolution keeps disk/config reads in `run_show` without coupling `slice.rs` to full-catalog hydration;
- whether absent facets are truly additive and testable without brittle golden machinery;
- whether the deferred risk/tag path avoids repeated projection and call-site churn before SL-133.

Governing doctrine: ADR-001 module layering and the pure/impure split; the storage rule; RFC-002's consumption-surface program; the no-parallel-implementation convention; POL-001 for prose restraint.
