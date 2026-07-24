# Implementation Plan SL-226: CLI graph emitter and ego-view

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, dependency-ordered inside-out along the ADR-001 layering: the
owned-graph filter pipeline first (PHASE-01), the BFS bounding that consumes
the filtered universe second (PHASE-02), the pure DOT emitter that renders
either (PHASE-03), and the thin command that composes them last (PHASE-04).
Each phase lands green and gate-clean on its own; nothing user-visible ships
until PHASE-04, which is acceptable — the verb is the only public surface and
the inner layers are exercised by their own suites from day one.

## Sequencing & Rationale

- **PHASE-01 before PHASE-02**: D4 fixed the pipeline as filter-then-bound, so
  the BFS's input contract (a filtered `CatalogGraph`) must exist and be
  tested first. The serde exact-equality VT lands here too, pinning the D11
  by-construction parity claim before anything consumes projections.
- **PHASE-02 before PHASE-03**: the emitter's fixtures want bounded ego
  subgraphs (focus penwidth, boundary cases) — building it after the BFS lets
  VT-C fixtures reuse real projections instead of hand-rolled graphs.
- **PHASE-03 before PHASE-04**: the command is pure composition; landing it
  last keeps it thin (ADR-001) and lets its tests assert end-to-end behaviour
  (D6 error split, formats) against already-proven layers. The `dot_escape`
  lift (D8) rides PHASE-03 because that is when `catalog/dot.rs` is born; the
  concept-map suite's unchanged green is that phase's preservation gate.
- Phases 01/02 both edit `src/catalog/graph.rs` — serial by design (same
  file, no dispatch parallelism); 03 touches `catalog/dot.rs` +
  `concept_map.rs` + `catalog/mod.rs` + `layering.toml`; 04 touches the
  command tree. File-disjointness across 03/04 exists but the EN chain keeps
  them serial anyway — this slice is small enough that parallel dispatch buys
  nothing.

## Notes

- Design is canon: D1–D15 in `design.md` §7; RV-298 (done, 14/14 verified)
  is the adversarial record. Where a phase sheet and the design disagree, the
  design wins.
- The D10 shape/style split deliberately diverges from `dot.ts` (upstream
  defect, ISS-237) — do not "fix" the Rust tables back to parity.
- VA-1 (PHASE-04) needs a host with graphviz `dot`; in the jail it may be
  absent — the criterion stays VA (agent judges, capturing output) rather
  than silently converting to a test that would skip.
- PRD-016 §2's interchange-demotion sentence is revisited at reconcile, not
  during phases (slice scope, Context).
