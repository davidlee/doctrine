# Implementation Plan SL-216: Per-component gauge scope

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases: code-then-prose. PHASE-01 lands the per-component placement
(design §2 mechanism, §3 verification) inside `src/comparison/project.rs`
plus one membership unit test in `src/priority/findings.rs`. PHASE-02 amends
the P1–P15 contract narrative (SL-213 design.md §3) and sweeps every stale
global-scope comment — code first so the prose describes shipped semantics,
not intent.

## Sequencing & Rationale

- **Code before contract prose.** D2's amendment must state what the code
  does; amending first would leave a window where the contract lies about the
  binary. PHASE-02's EN-1 pins this.
- **TDD order inside PHASE-01**: the four RED-capable cases (p12 case 4
  first-anchor flip, mixed-corpus island, singleton island, disconnect
  membership) are written against the shipped global trigger before
  `components()` exists; the s2/s8 re-pins flip RED→GREEN with the
  implementation. The retained p12 case 1 (anchored↔anchored) and the
  preservation set must never go red — that is the behaviour-preservation
  gate (design §2: anchored machinery is pure over adjacency, no
  cross-component edges, so per-component invocation is bitwise-identical for
  anchored components).
- **Comment edits split by phase**: comments adjacent to code the
  implementation touches (module note's mechanics, s2/s8 golden comments)
  move in PHASE-01 with their code; contract-narrative amendments (design.md,
  variant/producer docs, e2e ISS-050 story, module header's adjudication
  framing) ride PHASE-02 so the sweep audit sees them together.
- Solo execution (Fable, this session) — single-file semantics-dense change;
  dispatch funnel overhead exceeds the work (user-adjudicated 2026-07-12).

## Notes

- e2e_compare_inference is expected green **unchanged** in PHASE-01: it
  asserts finding presence and explain shape, not island values; its render
  string "no anchor in component" already speaks component language. If it
  goes red, that is signal, not noise — stop and re-derive.
- `HashSet` is clippy-disallowed (determinism) — `components()` uses BTree
  collections and min-member ordering.
- Golden audit sweep (design §3): s2 and s8 are the only multi-component
  cases in the shipped suite; the sweep in PHASE-01 confirms no third mover.
