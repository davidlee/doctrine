# Review RV-300 — code-review of REV-032

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External adversarial review of REV-032 against RFC-016, SPEC-021, SPEC-022,
ADR-006/012/013/014, REV-030, ISS-234, and the dispatch/git implementation.

Lines of attack:

- preserve the planned-versus-verified distinction when adding forward intent to
  largely retrospective specs;
- falsify every source claim about phase readiness, receipt/guidance derivation,
  funnel legality gates, and existing/missing git primitives;
- test whether the proposed persisted state model can enforce the active
  import/verify/conclude cadence rather than merely naming states;
- test whether working-tree-free reads actually eliminate ISS-234's latent
  reverse-diff hazard;
- challenge OQ-4 exclusion, cross-arm scope, requirement altitude, change-payload
  reviewability, and structural/FK integrity.

`review prime RV-300` was attempted and refused because selector-cache priming is
slice-only while this ledger validly targets REV-032. Evidence was therefore read
directly through the entity `show` surfaces and source.
