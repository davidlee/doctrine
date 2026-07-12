# Implementation Plan SL-219: Estimate comparison domain

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Six phases, strictly additive until PHASE-04 flips the scoring feed on behind
an approved REV. The cut follows the design's dependency spine: vocabulary →
constraint system → projection/feed → consumption → elicitation → surfaces.
Every phase ends green with all existing suites unchanged (the
behaviour-preservation gate is per-phase, not slice-end).

## Sequencing & Rationale

- **PHASE-01 wire** first because everything downstream consumes est-domain
  rows; it is also the only phase touching capture, so admissibility defects
  surface before any inference exists to confuse.
- **PHASE-02 pipeline split** is the structural risk (touching shipped value
  machinery); it lands while nothing consumes est output, so a mistake cannot
  reach scoring. The `authored_est_cost` extraction happens here — before
  PHASE-03/04 need it from two call sites.
- **PHASE-03 projection & feed** builds the derived cost artifact but wires no
  consumer. The D8 byte-identity invariant (value goldens unchanged under
  parameterization) gates here, isolated from the ladder change that would
  otherwise confound a regression.
- **PHASE-04 REV & scoring feed** is the only behaviour-changing phase and
  sits behind two gates: the REV approval (VH-1 — design §3 requires it before
  the ladder lands) and the zero-est-rows bitwise-identity VT. Sequenced
  before probes so PHASE-05's postcondition ("fed on next refresh") is
  testable end-to-end.
- **PHASE-05 probes** extends the elicit queue only after costs actually move
  on evidence — a probe whose answer changed nothing would be untestable
  theatre.
- **PHASE-06 surfaces & e2e** last: every render has real machinery behind it,
  and the e2e round-trip exercises the whole spine. The §6 checklist sweep
  (VA-1) closes the verification ledger before /audit.

## Notes

- REV timing (design §3): drafted during PHASE-04's first task, approved
  before its ladder commit. Phases 01–03 are REV-independent by construction.
- `compile.rs` must show an empty diff across the slice (design D9); PHASE-02
  VA-1 checks it at the phase boundary, /audit re-checks via `slice
  conformance`.
- Deferred seams (design § Deferred) are out of every phase: est anchor-review,
  component-calibration probes, priority-domain compiler, Phase E machinery.
