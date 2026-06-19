# Enforce ADR-001 layering: engine crate extraction + dependency fitness gate

## Context

ADR-001 (`leaf ← engine ← command, no cycles`) is review-only. It has no
automated enforcement, and the 2026-06-19 architecture audit confirmed the drift
the ADR itself predicted: cycles between the relation engine and the command tier
(addressed by SL-111). ADR-001 explicitly names two escalations once cycles
recur — promoting the engine to its own crate (a compiler-enforced boundary) and
adding a fitness function. We are well past the trigger.

The leverage here is durability: a boundary the compiler or CI checks stops the
drift from re-growing the moment a future change reintroduces an upward edge.
Without it, every structural fix (SL-111, etc.) decays back under review-only
pressure.

## Scope & Objectives

Mechanism decided in `/design`: a **`syn` dependency-fitness test now**; the engine
**crate split is deferred** to a follow-on slice (the layer map this slice authors
de-risks that later cut). The gate is a `cargo test` under `just gate`.

- **Classify-first (PHASE-01, go/no-go).** Author an authoritative module→tier map
  (`LAYER_MAP`) *with a per-module rationale* and measure the upward-edge baseline +
  per-tier cycle tangle, **before** building the gate. A small upward baseline + a
  meaningful engine core is the go condition; a mostly-baseline result re-routes to
  `/consult` rather than shipping a fig leaf. (The design probe found ADR-001's tier
  table *wrong*, not just stale — e.g. `input` is engine, not leaf.)
- **Hard-gate cross-tier direction.** Any upward edge fails `just gate` with the
  offending edge named, except a small *enumerated, frozen* `ACCEPTED_VIOLATIONS`
  baseline (each entry annotated + follow-up). The accepted set may shrink, never grow.
- **Ratchet intra-tier cycles by count.** The whole graph is *not* acyclic (the
  command tier is a large intra-tier SCC; the engine core is clean bar
  `conduct↔dtoml`). Where a hard gate is unachievable in scope, freeze a per-tier
  **cyclic-edge count** (same-tier edges inside a non-trivial SCC) and fail on any
  increase — monotonic-down. (A bare `Σ(SCC−1)` was rejected: it misses a new bad
  edge added *inside* an existing blob — external review C1.)
- **Encode the map as canon** — an authored `.doctrine/adr/001/layering.toml`
  (companion to the governing ADR, structured per the storage rule, not ADR prose,
  not a code `const`); ADR-001 carries the rule + tier definitions and points at it.
  Classification is **variable-granularity most-knowing-wins**: top-level module by
  default (tier = highest altitude of any non-test file); a mixed umbrella
  (`catalog`, `priority`) is sub-classified, and the gate **forces** that (a
  `MixedUmbrella` violation) so coarse granularity cannot launder a pure sub-file's
  upward edge.
- **Overturn + amend ADR-001 — required for closure** (via REV, ADR-013). ADR-001
  *currently rejects* this test, so the slice cannot close until the reversal lands:
  rule 1 hard-gated (literal `crate::` path edges); rule 2 enforced as a
  non-increasing cyclic-edge ratchet (same-tier edges inside a non-trivial SCC; the
  engine baseline is 0 after CHR-015) with the command tangle openly recorded as
  unmet-and-tracked; rule 3 deferred; `input` reclassified.

Closure intent: a *new* deliberate upward edge (or a new intra-tier cycle, or a
grown baseline) fails the gate locally; `just gate` runs the check; the layer map +
baselines are recorded as canon, not folklore.

## Non-Goals

- **Resolving** the pre-existing tangle — SL-111 broke the *engine→command* cycles;
  the *intra-command* SCC and the `conduct↔dtoml` core wart (CHR-015) are baselined
  and ratcheted, **not** untangled here.
- Resolving the `install`-as-utility wart ADR-001 flagged (the `state→install`
  upward edge), beyond classifying it and baselining it.
- Enforcing rule 3 (engine purity) — deferred to a follow-up (the impure-leaf
  refinement).
- A full crate-per-tier split / workspace reorganisation; scope is the gate, not
  the crate extraction.

## Summary

Make ADR-001 a machine-checked boundary via a `syn` dependency-fitness `cargo test`:
**hard-gate** the cross-tier direction rule (small frozen `ACCEPTED_VIOLATIONS`
baseline) and **ratchet** intra-tier cycles by a per-tier count that may not grow,
so the boundary stops eroding instead of relying on review. Classify-first PHASE-01
is the go/no-go. Engine crate split deferred. ADR-001 amended via REV at reconcile.

## Follow-Ups

- **Engine crate split** — the compiler-enforced boundary, a later slice seeded by
  this slice's `LAYER_MAP`.
- **CHR-015** — *done*: `conduct↔dtoml` engine-tier cycle broken; engine tangle
  baseline is 0.
- **Rule 3 (engine purity)** — the impure-leaf refinement (tag `git`/`clock` impure,
  forbid engine→impure-leaf in the same gate framework).
- **Burn down the baselines** — shrink `ACCEPTED_VIOLATIONS` (e.g. `state→install`
  helper extraction) and the command tangle count as cohesion work lands.
