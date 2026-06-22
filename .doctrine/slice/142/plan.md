# Implementation Plan SL-142: Wire tag coefficients into priority scoring

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Three ordered phases. The change touches <30 lines of production code across
four files; the phases carve the logical seam at natural boundaries (data
first, formula second, goldens last) so each phase can be verified independently
before proceeding.

## Sequencing & Rationale

### PHASE-01: Data path

Data flows before formula. Tags must arrive at `base_score` via `EntityFacets`
before the multiplier can be computed. The read path changes (read_facets →
ScannedEntity → build_from → EntityFacets) are purely additive — no existing
behaviour changes. VT-2 through VT-4 cover the TOML parsing edge cases (present,
absent, malformed); VT-5 pins the no-normalize invariant (RV-143 F-4) with a
mixed-case tag that would change if `normalize_tag` ever crept into the read path.

### PHASE-02: Formula

The formula change is the core of the slice. The delta-form multiplier
(`max(0.0, 1.0 + Σ(coeff − 1.0))`) is locked by ADR-015 §1 and the inquisition
(RV-143). Six test cases cover the identity, promotion, multi-tag, single demotion,
and multi-demote floor scenarios. The `dead_code` expect removal is a trivial
accompaniment. VT-3 (just gate) ensures the full workspace stays green.

### PHASE-03: Goldens

A low-risk phase that simply re-runs the existing golden test suite and updates
expected output. The identity semantics (default tags → ×1.0) mean the current
corpus (no tag-bearing entities) should produce no shift, but this is verified
explicitly rather than assumed. If goldens changed, the diff must be reviewed
for correctness.

## Notes

- The `tag::normalize_tag` question was settled in RV-143 F-4: NOT called in
  `read_facets`. SL-136 normalises at rest; the read path passes raw strings.
- IMP-109 (TOML double-parse) is adjacent and tracked separately.
- RFC-002 item B (seed `[priority.tag_coefficients]` in doctrine.toml) remains
  for after this slice lands.
