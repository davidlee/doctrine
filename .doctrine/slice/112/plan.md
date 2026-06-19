# Implementation Plan SL-112: Machine-check ADR-001 layering via a `syn` fitness gate

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases turn the locked design into the gate: **spike → gate → governance**.
The mechanism is fixed (design.md): a `syn` dependency-fitness `cargo test`
hard-gating cross-tier direction over literal `crate::` edges, forcing
sub-classification of mixed umbrellas, and ratcheting intra-tier cycles by a
non-increasing per-tier cyclic-edge count. The plan's shape is driven by one fact:
**the gate's value is unknown until the boundary is measured, and the classification
is judgement-laden + gameable** (design §4, D7). So measurement and the go/no-go
come *first*, as their own phase, before a line of the gate is committed.

## Sequencing & Rationale

**Why spike-first (PHASE-01).** The two adversarial passes (design §10) showed the
hard questions are empirical, not logical: how large is the genuine upward baseline
(is the gate worth building, design R6)? which umbrellas are altitude-mixed and need
sub-classifying (catalog, priority — C2/round-2)? what are the real per-tier
cyclic-edge counts? PHASE-01 builds only the impure half — the `syn`
extractor — and uses it to author `layering.toml` and measure. It ends at a
**human go/no-go gate** (VH-1): a small upward baseline over a meaningful engine core
is *go*; a mostly-baseline result means the gate would be a fig leaf and routes to
`/consult` rather than auto-advancing. Building the extractor here is not throwaway —
it is exactly the shell PHASE-02 wraps.

**Why the gate is its own phase (PHASE-02).** With the map authored and baselines
frozen, PHASE-02 adds the **pure** `check()` verdict (the four assertions) and the
bite-proof tests. Keeping the verdict pure and unit-tested (VT-2) — including the
negative self-tests that prove each violation class fires — is what answers ADR-001's
original "brittle homegrown test" objection. The real-graph assertion (VT-1) is green
by construction once PHASE-01's baselines are honest; the load-bearing evidence is
EX-3/VA-1: a hand-introduced upward edge must actually fail `just gate`. A gate that
cannot be shown to bite is not enforcement.

**Why governance is last and separate (PHASE-03).** ADR-001 *currently rejects* this
test (design D5/F-5), so shipping the gate without amending the ADR is a live
contradiction — but the amendment is only coherent once the gate exists and the layer
map is real. It is a **close co-requisite**, not optional polish, and per ADR-013 a
governance change is authored as a REV at `/reconcile`, not a raw hand-edit of accepted
governance. PHASE-03 fixes the contradiction at its only correct moment: after the gate
is green, through the governance path.

**Ordering is strict.** PHASE-02 cannot start until PHASE-01 returns *go* (EN-1);
PHASE-03 cannot start until the gate is green and committed (EN-1). No phase is
file-disjoint from its predecessor's output, so the sequence is serial by data
dependence, not just convention.

## Notes

- **Scope boundary held.** The plan enforces, it does not untangle. The command-tier
  SCC and `state→install` are *baselined and ratcheted*, never resolved here
  (Non-Goals); the engine crate split, rule-3 purity, and baseline burn-down are
  Follow-Ups.
- **F-5 carried forward.** PHASE-01's extractor must exclude doc-comment `crate::`
  links and `#[cfg(test)]` scope (VT-1); the regex probe used in design §2 had exactly
  this blind spot (a false `conduct→dtoml` edge was a doc-comment), so the `syn` walk
  is load-bearing, not a nicety.
- **Concurrent-agent hygiene.** Commit each phase's output the instant it is coherent —
  a concurrent agent on `main` already swept this slice's uncommitted design edits into
  an unrelated commit once.
