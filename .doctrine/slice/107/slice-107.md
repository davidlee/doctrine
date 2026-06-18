# Estimate & Value facet integration

## Context

SL-101 ("Estimate & Value facets") closed `done` (terminal, ADR-009) but its
delivery to `main` is incomplete. The spec side is reconciled (REV-002, commit
`446806cb`): PRD-014 is now "Estimation & Value" with unit defaults
`espresso_shots` (estimate) and `magic_beans` (value); SPEC-020 carries both the
`EstimateFacet` and `ValueFacet` contracts. The code side is not:

- `src/estimate.rs` exists on `main` but is **dead/unwired** — `#![allow(dead_code)]`,
  zero external references (`grep estimate::|EstimateFacet|EstimationConfig`
  outside the file = nothing).
- `src/value.rs` is **absent**; `src/main.rs` declares `mod estimate;` only.
- `src/dtoml.rs` `DoctrineToml` wires only `[conduct]` + `[verification]` — no
  `[estimation]`, no `[value]`.
- `src/slice.rs` carries no facet fields and no detailed-display rendering.
- `install/doctrine.toml.example` has no `[estimation]` / `[value]` sections.

The Value module + all wiring live only on stale `dispatch/101*` branches (~34
commits behind `main`, conflicting on `estimate.rs`) — **reference material, not
a merge source** (see memory `mem.fact.doctrine.sl-101-facets-unintegrated`). This
slice ports that code by hand onto current `main` and re-verifies.

This slice is the origin-continuation of SL-101 (`related`) and implements the
reconciled contract in SPEC-020 / PRD-014 (`specs`).

## Scope & Objectives

**Narrow boundary** (decided with the User): SL-107 ships exactly the
*wiring/integration foundation* SL-101 designed but never landed on `main`.
Estimate **display** rendering is SL-102's charter (`ready`, REQ-273); estimate
**graph** exposure is SL-103's (`proposed`, REQ-274). SL-107 does **not** touch
either — it makes the facet *data* present, parsed, and configured so SL-102/103
build on it.

Port source: `candidate/101/review-001` — the orphaned-but-audit-fixed (F-1/F-2)
tip that already embodies this exact narrow scope. `main` has diverged ~34 commits
past the candidate base (`ec2de06`); port by hand, do not merge the branch.

1. **Value module** — drop `src/value.rs` (`ValueFacet`, single finite `f64`,
   present-facet validation, unit `magic_beans`) onto `main` + `mod value;` in
   `main.rs` (no main drift on either; candidate version applies clean). Tests
   V1–V7 (SL-101 design §7.2).
2. **Estimate dead-code hygiene** — replace `estimate.rs`'s blanket
   `#![allow(dead_code)]` with item-level `#[cfg_attr(not(test), expect(dead_code,
   …))]` on the still-pending fns (`resolve_unit`, `resolve_confidence`,
   `parse_optional`, the two confidence consts), citing SL-102/SL-103. The facet
   *types* go live via wiring; the display/graph helpers stay `expect`-dead until
   their owning slices consume them. (Logic/tests untouched — main vs candidate
   `estimate.rs` differ *only* in this attribute treatment; both already carry
   `espresso_shots`.)
3. **`dtoml.rs`** — add `estimation: estimate::EstimationConfig` +
   `value: value::ValueConfig` to `DoctrineToml` (`#[serde(default)]`, tolerant-
   absent), mirroring `conduct`/`verification`. No main drift; candidate delta clean.
4. **`SliceDoc` (`slice.rs`)** — two optional facet fields
   (`estimate: Option<…>`, `value: Option<…>`), parsed via serde. **Parsed, not
   rendered** — display is SL-102. Update the existing test fixtures
   (`estimate: None, value: None`).
5. **`install/doctrine.toml.example`** — commented `[estimation]` / `[value]`
   sections documenting the unit defaults.

Affected surface: `src/value.rs` (new), `src/estimate.rs`, `src/main.rs`,
`src/dtoml.rs`, `src/slice.rs`, `install/doctrine.toml.example`.

## Non-Goals

- **Display rendering** (`Estimate: …` / `Value: …` lines, verbose spread) — SL-102
  (REQ-273). SL-107 leaves the SliceDoc fields parsed-but-unrendered.
- **Graph / catalog-hydrate exposure** — SL-103 (REQ-274).
- **Re-deriving the contract.** PRD-014 / SPEC-020 are reconciled and authoritative;
  this slice implements them, it does not revise them.
- **Merging `candidate/101/review-001` or `dispatch/101*`.** Orphaned/stale and
  conflicting; port by hand.
- **Reopening SL-101.** Terminal (`done`); this is a fresh slice.
- Everything SPEC-020 / PRD-014 already exclude: aggregation, cost-vs-value
  arithmetic, simulation/prediction, classification/gating, richer v1 facet fields,
  per-entity units, any estimate/value gate on dispatch/execute/audit/close.

## Summary

(Filled at close — rollup of what shipped vs the design.)

## Follow-Ups

- Confirm graph-exposure scope (objective 5) against SPEC-020 sources at `/design`;
  split out if it sprawls past a single shippable change.

## Verification / Closure Intent

- `mod value;` present; `src/value.rs` lands with V1–V7 green.
- `estimate.rs` blanket `#![allow(dead_code)]` gone — replaced by item-level
  `expect(dead_code)`; the facet *types* (`EstimateFacet`, `EstimationConfig`,
  `ValueFacet`, `ValueConfig`) are now referenced from live code (SliceDoc fields +
  dtoml config). The `expect`s prove the display/graph helpers are the *only*
  residual unused surface, owned by SL-102/SL-103.
- `[estimation]`/`[value]` parsed by `dtoml.rs`; `SliceDoc` carries + round-trips
  both optional fields (parsed, not rendered).
- **Behaviour-preservation gate** — existing suites stay green unchanged; estimate
  E-tests + value V-tests green.
- `just gate` clean (workspace), plain `cargo clippy` zero warnings (the `expect`s
  must not fire), `spec validate` clean.
