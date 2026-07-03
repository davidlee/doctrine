# Implementation Plan SL-196: Per-edge relation descriptor

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four sequential phases carry one optional free-text `descriptor` cell along the
data's own flow: **write → read → hydrate/render → search**. Each phase is a
single layer of the pipe, ends green, and is independently verifiable. The whole
slice rides the SL-176 `Degree` seam verbatim (design D1) — every site descriptor
touches, `degree` already touches; the work is re-skinning a closed enum as free
text and extending it to the read/hydrate/search legs degree never needed.

## Sequencing & Rationale

**Why this order — the payload dependency.** Each phase consumes the prior's
output, so the sequence is forced, not stylistic:

- **PHASE-01 (write)** puts the cell on disk. Nothing downstream can be tested
  until a descriptor can be authored. It also owns the gate and conflict rules
  (the write path is the ONLY enforcement point — OQ-6/INV-5), so admissibility
  is settled first.
- **PHASE-02 (read)** makes the cell round-trip. This is the leg the first
  design inventory missed (external F-C): `RelationRow` + `read_block` +
  `RelationEdge` constructors. Without it PHASE-01 writes bytes no reader
  recovers. Split from PHASE-01 as its own verification unit precisely because
  the write/read asymmetry (write-gated, read-permissive) is a distinct
  behaviour to pin.
- **PHASE-03 (hydrate + render)** lifts the read value onto `CatalogEdge` and
  shows it. Gated on PHASE-02 (needs `RelationEdge.descriptor`). Carries the
  serde-omission obligation (external F-D) — the one place a naive `Option` field
  would silently change the `/api/graph` contract.
- **PHASE-04 (search)** is the only consumer that needs the hydrated edge
  projected and joined back to its source entity. Last because it depends on
  PHASE-03's `CatalogEdge.descriptor` and on nothing depending on it.

**Not parallelisable.** All four phases mutate `relation.rs` and the catalog/
search modules — file-overlapping. Serial execution (solo or one dispatch worker
per phase), no fan-out.

**Largest phase.** PHASE-01 is the widest (table column + gate + full write
threading + CLI + conflict generalisation). It is cohesive — one authoring path —
so it is not split, but it carries the most sites and the most compile churn
(R1); expand its runtime sheet carefully at `/phase-plan`.

**De-risked at design.** The two softest spots are already resolved, not carried
into execution: D4/R3 (the hydrate hedge) — the raw `RelationEdge` is provably the
element the hydrate loop iterates (`scan.rs:122`), so no source-keyed re-scan
fallback survives into PHASE-03; and the `contextualizes` scope question — closed
(read-dropped, excluded; the latent bug is ISS-211, out of this slice).

## Notes

- **Constructor churn (R1).** PHASE-02's `descriptor: None` default lands at every
  `RelationEdge` construction site: `relation.rs:768/780`, the `with_role` sites in
  `relation_query.rs`, `spec.rs:1133`, `rec.rs:456`, `review.rs:1428`. Compile-
  driven — the build names them; don't hand-hunt.
- **Behaviour-preservation gate.** Shared entity-engine machinery: every existing
  relation / catalog / search suite must stay green UNCHANGED (absent descriptor ⟹
  prior behaviour). New VTs are strictly additive.
- **STD-001 (F6, low priority).** `"degree"`/`"role"`/`"label"` cell names are bare
  literals today, not single-sourced constants — a bare `"descriptor"` literal is
  consistent. Do not mint a lone constant for it; if the codebase later single-
  sources cell names, descriptor joins then.
- **Cross-slice (OQ-5).** The driver (CPT saying annotated things) needs `CPT`
  added to the `references:concerns` `sources` array — SL-197's job, not a phase
  here. Descriptor ships source-agnostic; CPT authoring is dark until SL-197 wires
  it.
