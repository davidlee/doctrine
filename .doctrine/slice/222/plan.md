# Implementation Plan SL-222: Ledgered estimate claims

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Nine phases in three bands, mirroring SL-220's proven shape:

- **Evidence first** (PHASE-01): the pre-flip ranking snapshot is the base
  every later delta is judged against — cheap, and worthless if captured
  late.
- **Strictly additive machinery** (PHASE-02..04): wire columns, the generic
  claims pass, pipeline wiring. Each lands with the engine gate (existing
  suites green; corpora without estimate anchor rows score
  bitwise-identically) because nothing consumes the new rows yet. The
  in-pipeline bare-anchor derivation (PHASE-04) is deliberately placed in
  the additive band: with zero anchor rows it reproduces the facet-derived
  value exactly, so the F-4 re-siting is proven behaviour-preserving
  *before* the flip depends on it.
- **Gated behaviour change, then teardown** (PHASE-05..09): governance gate
  (REV + SPEC-020 disposition map, operator-approved) → the flip (ladder +
  anchor swap + verbs + probes + explain, one release so canon, verbs, and
  resolution move together) → remaining render surfaces → migration →
  deletion.

## Sequencing & Rationale

**Why the REV sits at PHASE-05, not earlier or later.** The REV describes
machinery (claim resolution, the ladder, row-sourced bare anchor) that must
exist to be reviewable — but must be approved before any resolution outcome
changes (design E11, the D12 mirror). Between PHASE-04 and PHASE-06 the
machinery is complete and inert: the exact window the gate needs.

**Why the flip is one phase.** Splitting the ladder from the verb re-plumb
would ship a window where `estimate set` writes facets the ladder no longer
reads (silent scoring drift), and splitting explain/JSON from the ladder
would ship renders that lie about provenance. The flip's blast radius is
wide but internally coupled; its EX list is correspondingly the largest, and
the class-b golden churn is enumerated there, not smeared across phases.

**Why migration precedes deletion, and both are late.** Migration needs the
flip (its rows must resolve at rung 4, and the interregnum rung 5 shadows
un-stripped residue). The deletion trigger is *both domains migrated on this
corpus* (SL-220 D6) — so PHASE-09 is strictly after PHASE-08, and its
grep-gate + tripwire close the slice. The bare-anchor re-source (PHASE-04)
lands well before any strip: `max_upper` never sees an empty-input window
(design §5 sequencing pin).

**Dependency spine**: 01 → 02 → 03 → 04 → 05 → 06 → {07, 08} → 09.
PHASE-07 and PHASE-08 are file-disjoint enough to parallelise under
dispatch (render surfaces vs script + corpus execution); both need
PHASE-06; PHASE-09 needs PHASE-08 only, but waits for 07 in practice (full
suite green is its EX-3).

**Evidence cadence**: snapshots at PHASE-01 (pre-flip), PHASE-06 EX-4
(post-flip), PHASE-08 EX-3 (post-migration), PHASE-09 EX-3 (final). Four
points because the flip and the migration move rankings for different
reasons (source re-ranking vs facet→claim tier motion), and the audit needs
them separable (scope R1).

## Notes

- The value-pass refactor gate (PHASE-03 EX-3) is the plan's riskiest
  correctness claim: the `.value → .operative` re-path touches the claims
  battery, store, graph, surface, and elicit. The gate is behavioural
  (assertion semantics + goldens, RV-282 F-3), and the churn class is
  enumerated in phase notes at execution — an assertion whose *expected
  values* change fails the phase.
- PHASE-06 VT-4 keys on `estimate_claims` appearing in `elicit.rs`; if the
  probe recomposition ends up consuming a narrower view type instead, update
  the mandate keywords at phase-plan time (mandates are floors, not designs).
- The migration script's non-mutating `--check` (RV-282 F-7) is a deliberate
  divergence from the SL-220 template — do not copy
  `migrate_value_facets.py`'s check path verbatim.
- NF-001 symbol-substring tripwire: check new names (`CostClaim`,
  `CostUnmigratedFacet`, tripwire helpers) before committing (design §6
  naming hazard; RV-277 F-5 precedent).
