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
  tangle count (`Σ(SCC_size−1)`) and fail on any increase — monotonic-down.
- **Encode the map as canon** — a reviewed Rust `const` the gate reads (structured,
  not ADR prose); ADR-001 carries the rule + tier definitions and points at it.
- **Amend ADR-001** (via a REV at reconcile, ADR-013): rule 1 machine-enforced;
  rule 2 enforced as a non-increasing ratchet with the command tangle openly
  recorded as unmet-and-tracked; rule 3 deferred; `input` reclassified.

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
- **CHR-015** — break the `conduct↔dtoml` engine-tier cycle (drive the engine tangle
  baseline to 0).
- **Rule 3 (engine purity)** — the impure-leaf refinement (tag `git`/`clock` impure,
  forbid engine→impure-leaf in the same gate framework).
- **Burn down the baselines** — shrink `ACCEPTED_VIOLATIONS` (e.g. `state→install`
  helper extraction) and the command tangle count as cohesion work lands.
