# Implementation Plan SL-065: Product-level axis and PRD decomposition parent

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, ordered intent → mechanism → integrity. The slice mirrors the
tech-spec C4 surface onto product specs: a `product_level` altitude tag and a
first-class intra-family `parent` decomposition. The design (`design.md`) locks
five decisions (D1–D5) and defers two tightenings (OQ-1/OQ-2). The plan realises
those decisions in the smallest coherent commits.

## Sequencing & Rationale

**PHASE-01 first — product intent precedes the mechanism.** Shipping
`product_level` and product decomposition without a governing requirement is
drift (design §6). FR-005/FR-006 land on PRD-002 before any source change, so the
later phases descend from authored intent rather than backfilling it. This phase
is pure entity authoring — no source, no tests — so it carries `VA` verification
(agent confirms via `spec req list` / `requirement show`), not `VT`. It is first
because it has no code dependency and sets the contract the next two phases meet.

**PHASE-02 next — the data model and render, in one file.** `ProductLevel` and
the `Spec.product_level` field are inert until something reads them; `spec show`
is that first reader, so enum + field + render belong in one phase (the render is
what un-deads the field, avoiding a dead_code bridge spanning phases). It is
deliberately validate-free: render only needs the field to exist, not the parent
graph to be legal. The behaviour-preservation gate bites here — the subtype
branch must leave tech `show` byte-identical (VT-2).

**PHASE-03 last — the validate model, by deletion.** The symmetric same-subtype
rule and subtype-blind acyclicity are independent of render and live entirely in
`registry.rs`, so they come last and file-disjoint from PHASE-02. The core move
is removing three `on_product` special-cases, not adding a parallel product pass —
so the diff is mostly deletion plus the one inverted-subtype branch in
`parent_findings`. Tests drive the `spec validate` CLI seam, not the registry
helpers in isolation (the invariant-test-drives-the-write-seam rule).

Why not fewer phases: PHASE-02/03 touch different files (`spec.rs` vs
`registry.rs`) and different test surfaces (render goldens vs validate findings);
splitting keeps commits small and the behaviour-preservation gate legible. Why
not parallelise: the slice is small and PHASE-03's entrance leans on PHASE-02's
field existing for end-to-end product-spec fixtures — serial is simpler with no
real wall-clock cost.

## Notes

- REQ-082/083 (PRD-012, tech-only) are untouched throughout — the `parent_findings`
  doc-comment gains the new product REQ citation alongside REQ-083 (PHASE-03 EX-3).
- Deferred (Follow-Ups, not phases): level-adjacency enforcement (OQ-2) and the
  `descends_from`→capability-level constraint (OQ-1). Both are advisory-only in v1.
- The `on_product` field is retained (descriptive: "subject is a product spec");
  only its doc-comments are reworded — it changes role from reject-flag to
  subtype-selector.
