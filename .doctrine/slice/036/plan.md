# Implementation Plan SL-036: cordage graph core crate

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases building `crates/cordage` inside-out along the data-flow of
`build()` itself: model & validation → resolution passes 1–2 → order
composition (passes 3–4) → traversal & channels → explanation plus the
slice-wide verification net. The design (design.md, locked 2026-06-10 after
four adversarial rounds, F1–F48) is canon; the plan adds **sequencing only**,
no design decisions. Every fixture row of design.md §9 is assigned to exactly
one phase's verification — two rows split across phases (the arity×reject
pipeline row and the REQ-077 determinism row) with the split noted on both
halves, so coverage stays auditable.

## Sequencing & Rationale

**Why this cut.** Each phase boundary is a point where the crate compiles,
the suite is green, and a coherent public surface is pinned — never a
half-implemented pass. The order follows dependency, not convenience:

- **PHASE-01 before everything**: the §5.2 model vocabulary and the
  build-error contract (F14/F22/F38) are referenced by every later test;
  getting BTree storage and the explicit adjacency orderings (F21) in first
  means determinism is structural from day one, not retrofitted. `build()`
  here validates and stores but resolves nothing — the passes arrive as
  behaviour changes on an already-stable API.
- **PHASE-02 (passes 1–2) before PHASE-03 (passes 3–4)**: pass 3's intra-SCC
  exclusion and pass 4's taint seeding both key to the **post-arity** SCC
  marks pass 2 produces (F46), and pass 2's Reject detection reads the
  **authored pre-arity** set (F30) — the pipeline interaction that produced
  two review blockers is exactly where the phase boundary sits, so each side
  is pinned independently before they meet.
- **PHASE-04 (channels) after PHASE-03**: channels do not depend on ordering
  (I7 — values invariant under `OrderSpec`), but *verifying* that invariance
  (the §9 eviction-scope row) requires pass-3 eviction to exist. Sequencing
  channels after ordering makes I7 testable the day `evaluate` lands.
- **PHASE-05 last**: `explain` assembles every other surface (order_key,
  paths over resolved views, evictions), and the golden net — permutation
  determinism, naive-oracle property tests (F24/R1), the Appendix-B denylist
  scan (REQ-079/R4) — is only meaningful over the finished crate. Ending here
  leaves the slice audit-ready: the net is the standing boundary proof.

**TDD shape.** Red/green/refactor per §9 row: each phase's VT list is its
red-test inventory. Fixture vocabulary stays overlay-neutral (`Reject`/`Evict`
overlays, layers `x`/`y` — F42); `age` is always test-supplied (A1, clock-free).

**Risk placement.** R1 (hand-rolled SCC/topo bugs) is mitigated where the
algorithms land (PHASE-02/03 fixtures) and again by the PHASE-05 oracle
property tests. R3 (diamond double-count) sits in PHASE-04's rollup fixture.
R4 (boundary erosion) is closed structurally in PHASE-01 (zero deps) and
permanently in PHASE-05 (denylist in the suite).

## Notes

- Known-opens from review (notes.md "Round-4 outcome") are **deliberate
  deferrals**, not plan gaps: explanation path-enumeration combinatorics
  (predecessor sub-DAG is the flagged fix direction), full-downstream taint
  extent, pre-consumer API churn. First-consumer territory — do not resolve
  them inside these phases.
- The doctrine root crate gains **no** path dependency on cordage in this
  slice (slice-036.md § Affected surface) — the crate stands alone with its
  own suite; the adapter slice wires the consumer.
- Gate per repo convention: plain `cargo clippy` (not `--all-targets`) +
  `just check`. Jail: the built binary lands at
  `~/.cargo/doctrine-target-jail/debug/`, not `./target/debug/`.
