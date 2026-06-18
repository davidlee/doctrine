# SPEC-020: Estimate display rendering

## Context

SL-101 delivers the `EstimateFacet` model, parse, and validation. This slice adds
pure display-rendering functions that consume the normalized model and produce
human-readable output.

**Depends on SL-101** — the `EstimateFacet` struct and the resolved unit must exist.

## Scope & Objectives

- **FR-005 (REQ-273)** — Pure display functions:
  - Present: `Estimate: <lower>-<upper> <unit>` (normalized compact bounds)
  - Absent: `Estimate: none recorded`
  - Verbose: adds attention spread (ratio) and attention width (absolute)
  - `lower == 0` → spread ratio reported unavailable, width still shown
- Normal display classifies nothing — no "wide/risky/split-worthy" labels.

## Non-Goals

- Graph exposure → SL-103
- Estimate authoring/writes
- Classification thresholds

## Summary

Pure rendering functions in `src/estimate.rs` (or a display sub-module), driven
by unit tests.
