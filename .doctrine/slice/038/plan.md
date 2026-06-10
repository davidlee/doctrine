# Implementation Plan SL-038: cordage scale harness — durable red tests + findings for the confirmed scale cliffs

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-038 lands a committed, reproducible regression gate plus a findings note for
four cordage scale cliffs — three probe-confirmed (RSK-002 explain exponential,
RSK-003 overflow, RSK-003 eviction quadratic) and one analytical-only folded in
under D5 (RSK-004 evaluate per-node-BFS quadratic). Measure-and-red only: this
slice produces the reds the eventual fixes (A/B/C/D) green; it ships no fix and
touches no cordage `src/`. The locked `design.md` is canon; this plan only
sequences it.

The work is three artifacts — a measurement example, a set of `#[ignore]`d
characterization tests, and a findings note — split into three phases on a strict
data dependency: **evidence, then encode, then consolidate.**

## Sequencing & Rationale

**PHASE-01 (harness & characterization) is first because the evidence is an
input to the other two.** The measurement example is the instrument; until it is
run there are no numbers for the findings note and — critically — no validated
sub-overflow `N` for the evaluate red. The inquisition's C3 charge is exactly
this: the ~5–12k figure is a linear extrapolation from a single probe datum, and
`level_of` recursion depth = chain length, so the safe ceiling is *confessed only
by running it*. PHASE-01's EX-3 makes that empirical pinning a gate, not an
assumption: both `n` must build without `SIGABRT` **and** the larger `evaluate`
must run long enough for the recorded ratio to clear scheduler noise.

**PHASE-02 (the reds) depends on PHASE-01's pinned `N` and recorded numbers.**
The evaluate red hardcodes the validated pair; the overflow red is a self-re-exec
subprocess (a stack overflow is uncatchable in-process — design R1/D2), so it
cannot be a `#[should_panic]`. The reds are `#[ignore]`d because they are slow or
deliberately crashing; the default gate stays green, which EX-3/VT-2 assert
directly. The clippy-coverage gap (design §8) means neither `examples/` nor
`tests/` is linted by `just check`, so VT-3 carries the manual
`cargo clippy -p cordage --examples --tests` as an explicit phase gate.

**PHASE-03 (findings) is last because it reports what the first two produced.**
Its one non-mechanical duty is honesty about provenance: RSK-002/003 carry the
deleted probe's numbers re-confirmed here, while RSK-004 is *first measured by
this harness* — the probe never ran it. The note also records H1's honest
position and the OQ-2 allocation gap, and confirms fixes A/B/C/D are filed as
follow-up work rather than smuggled into this slice.

A single generator — `deep_chain` — drives both the overflow and evaluate cliffs
(at target depth and at sub-overflow depth respectively), which keeps the
duplicated-generator count at three across the `examples/`↔`tests/` boundary, at
D4's revisit threshold rather than over it.

## Notes

- Behaviour-preservation: the 75 existing cordage tests are not touched; the reds
  are new, additive, and ignored by default.
- Zero-dep is re-asserted every phase via an empty `Cargo.toml` diff (EX-4 /
  EX-3) — the contract binds the harness too (D1; criterion/bench-member rejected).
- The reds are characterizations: they *pass* by asserting the bad behaviour
  (exact 2^layers, child rc-134, super-linear ratio). The fix slices flip each
  assertion to the good behaviour — that is what "the reds the fixes green" means.
