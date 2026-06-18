# Implementation Plan SL-101: Estimate & Value facets — model, parse, validate, unit

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four sequential phases build two kind-agnostic entity facets from the ground up:
Estimate (bounded attention burden) and Value (single magnitude). Each facet is a
pure-leaf module (ADR-001) with its own model, parse/validate pipeline, config,
and test suite. Config wiring and entity integration follow.

## Sequencing & Rationale

**PHASE-01** builds `estimate.rs` first because it is the more complex of the two —
two-field validation matrix, confidence bounds, the fuller test surface. It
establishes the pattern (Raw struct → custom Deserialize → normalise → validate →
config → resolve) that PHASE-02 follows mechanically.

**PHASE-02** implements `value.rs` as a simpler sibling — one field, no range
validation, no confidence bounds. By implementing estimate first, value becomes a
straightforward adaptation. The two modules are file-disjoint and could be
parallelised, but the learning transfer from estimate to value is high enough that
sequential ordering is more practical for a solo developer.

**PHASE-03** wires both configs into `dtoml.rs` and updates the template. Must
follow PHASE-01 and PHASE-02 (the types must exist). The existing conduct and
verification config tests serve as the behaviour-preservation gate — they must
stay green unchanged.

**PHASE-04** integrates both facets into `SliceDoc` and adds `mod estimate` /
`mod value` to `main.rs`. This is the only phase that touches an existing module
(`slice.rs`), so behaviour-preservation is paramount: all existing slice tests
must pass unmodified. NF-001 (non-blocking guarantee) is verified structurally
here.

## Notes

- All four phases must end green: tests pass, clippy zero, `just check` clean.
- The `install/templates/slice.toml` template is intentionally unchanged — facets
  are opt-in and authored explicitly.
- Display (SL-102) and graph exposure (SL-103) are follow-up slices that consume
  the models built here.
