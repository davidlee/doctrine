# SL-101: Estimate & Value facets — model, parse, validate, unit

## Context

PRD-014 defines the *what* and *why* of estimation: an optional, bounded claim about
human attention burden. SPEC-020 is the *how* — the technical specification for the
estimation facet as a component of the entity engine (SPEC-004).

This slice implements two kind-agnostic entity facets:
- **Estimate** (`[estimate]`) — two bounds (`lower`/`upper`), unit `espresso_shots`
- **Value** (`[value]`) — one magnitude, unit `magic_beans`

Both are pure-leaf modules, parsed from entity TOML, with project-wide units
resolved from `doctrine.toml`. This is the foundation that SL-102 (display) and
SL-103 (graph exposure) build on.

No facet code exists in the codebase today — greenfield within the existing
entity-engine architecture.

## Scope & Objectives

Implement the `EstimateFacet` as a pure, reusable model in a new `src/estimate.rs`
leaf module (ADR-001: leaf ← engine ← command layer).

### In scope

- **FR-001 (REQ-269)** — `EstimateFacet` struct carrying `lower: f64` and
  `upper: f64`. Parse from optional `[estimate]` TOML table on any entity.
  Normalize TOML integers and floats to finite `f64`; reject NaN, ±Infinity at the
  parse boundary.

- **FR-002 (REQ-270)** — Validation matrix: a *present* `[estimate]` must have both
  `lower` and `upper` present, finite, `lower >= 0`, and `upper >= lower`. Any
  violation is a hard parse/validate error — no silent repair. An *absent*
  `[estimate]` parses clean (absence is not malformation).

- **FR-003 (REQ-271)** — Project-wide estimation unit read from
  `doctrine.toml [estimation].unit`, defaulting to `espresso_shots`.
  Also ships default confidence bounds (`lower_confidence`/`upper_confidence`,
  fractions in [0.0, 1.0], defaults `0.1`/`0.9`) — purely informational in
  this slice, structured for downstream Monte Carlo use.
  No entity-local config in v1.

- **FR-004 (REQ-272)** — Round-trip durability: a valid estimate survives
  parse → hydrate → catalog projection unchanged. Verified by tests; not a separate
  code path. Original TOML numeric formatting not retained; normalized form is truth.

- **NF-001 (REQ-275)** — Structural non-blocking guarantee: no dispatch, execute,
  audit, or close predicate reads estimate presence. Proven by the absence of such
  reads in this slice's code (no workflow gate references `[estimate]`).

- **NF-002 (REQ-276)** — Kind-agnostic, reusable, pure-layer: the `EstimateFacet`
  model is defined once; attaching to a new entity kind requires only wiring the
  parse path. Normalization, validation stay in the pure layer; only unit resolution
  touches the shell (reads `doctrine.toml`).

- **NF-003 (REQ-277)** — Forward-compatible schema: the parser tolerates additive
  optional fields beyond `lower`/`upper` so v1 estimates remain valid under future
  extensions. v1 persists no inferred `mode` or `distribution`.

### Affected surface

- **New files:** `src/estimate.rs` (~120 lines) and `src/value.rs` (~90 lines) —
  pure-leaf modules per ADR-001. Import only external crates (`toml`, `serde`,
  `anyhow`).
- **Configuration:** `doctrine.toml [estimation]` and `[value]` tables at the
  project root — shipped as commented-out sections in
  `install/doctrine.toml.example`.
- **Entity TOML:** wire both facets into `SliceDoc` (the slice full-detail reader)
  — one optional field each. `Meta` (list scan) unchanged.

### Architecture decisions

- **D1 (SPEC-020 D1)** — The facets are components on the entity engine, not new
  containers. They add facet models and parse/hydrate wiring; no new `Kind` descriptor.
- **D2 (SPEC-020 D2)** — Bounds normalize at the parse boundary to finite `f64`.
  Non-finite values rejected where they enter.
- **D3 — Two sibling modules, not a shared abstraction.** Estimate and Value are
  separate `src/estimate.rs` / `src/value.rs` — their validation is different enough
  (two-field vs one-field) that a shared abstraction would be heavier than the
  duplication.
- **Pure/impure split** — `parse_optional` and `validate` are pure; `resolve_unit`
  is pure over config (the file read is the shell's job).

## Non-Goals

- **Display rendering** (FR-005 / REQ-273) — pure display functions for
  present/absent/verbose output → SL-102.
- **Graph exposure** (FR-006 / REQ-274) — catalog hydration and the policy-free
  graph contract → SL-103.
- **Named policies, aggregation, simulation** — PRD-014 non-goals, out of scope
  permanently.
- **Estimate write/edit CLI** — authoring estimates is a follow-up concern; this
  slice builds the read side. Estimates are hand-authored in TOML for now.
- **Attaching to every entity kind** — wire one kind (slice) in this slice as proof
  of the mechanical-attachment claim; additional kinds are follow-up.

## Summary

Two new leaf modules:
- `src/estimate.rs` — `EstimateFacet { lower: f64, upper: f64 }`, parse, validate,
  unit resolution (`espresso_shots`), confidence defaults (`0.1`/`0.9`)
- `src/value.rs` — `ValueFacet { value: f64 }`, parse, unit resolution
  (`magic_beans`)

Both use custom `Deserialize` so `SliceDoc` carries `Option<EstimateFacet>` and
`Option<ValueFacet>` directly. `dtoml.rs` gains `[estimation]` and `[value]` config
sections. `Meta` (list scan) is unchanged — facets are show-path detail.

## Follow-Ups

- **SL-102** — Display rendering (FR-005)
- **SL-103** — Graph exposure (FR-006)
- **SL-104** — Hardening: dogfood on real entities, edit-preserving writes, any
  residual NFR concerns
- **Future** — Estimate authoring CLI, additional entity kinds, aggregation policies
