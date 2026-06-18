# Implementation Plan SL-102: SPEC-020: Estimate display rendering

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Single-phase plan: the display functions are pure, self-contained, and have no
internal dependencies. All three functions (`format_bound`, `format_estimate_normal`,
`format_estimate_verbose`) are implemented together in one phase because they share
the formatting primitive (`format_bound`), and the tests naturally cover all three
in concert.

## Sequencing & Rationale

**PHASE-01** is the entirety of the slice. No subsequent phases are needed:

- `format_bound` is the primitive — implemented first, tested, then consumed by
the other two.
- `format_estimate_normal` and `format_estimate_verbose` are independent of each
other but share the bound formatter.
- The sub-module wire (`pub mod display;`) is a one-line mechanical edit.
- Integration into `slice show` is out of scope per SL-102 non-goals; that wiring
happens in a future slice.

## Notes

- Tests live in `src/estimate/display.rs` — a `#[cfg(test)] mod tests` block at
the bottom of the file, alongside the display functions. Keeps the leaf function
and its tests co-located; no cross-module test imports needed.
- The `unit` parameter is contractually non-empty (the caller resolves it via
`resolve_unit`). A `debug_assert!(!unit.is_empty())` in the display functions
documents this at zero runtime cost.
