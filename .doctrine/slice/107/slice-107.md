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

Wire **both** facets into `main` against the reconciled SPEC-020 contract:

1. **Value module** — add `mod value;` + port `src/value.rs` (`ValueFacet`,
   single finite `f64` magnitude, present-facet validation, unit `magic_beans`,
   pure display) onto current `main`. Re-create SL-101 design §7.2 tests V1–V7.
2. **Estimate wiring** — connect the already-present, dead `src/estimate.rs`
   (parse seam, `[estimation].unit` resolution, validation matrix, display). This
   is wiring, not a rewrite — the module exists.
3. **`dtoml.rs`** — add `estimation` + `value` sub-config fields to `DoctrineToml`
   (`#[serde(default)]`, tolerant-absent), mirroring the `conduct`/`verification`
   shape. Re-apply the `dispatch/101` dtoml delta onto main's evolved struct.
4. **`SliceDoc` / `slice.rs`** — two optional facet fields, detailed-display
   rendering (`Estimate: …` / `Value: …`, present / `none recorded`; verbose
   spread per SPEC-020).
5. **Graph exposure** — expose both facets' metadata through the catalog/hydrate
   seam per SPEC-020 (PRD-014 REQ-265 / REQ-285) as a policy-free contract. Confirm
   exact scope against SPEC-020 sources during `/design`.
6. **`install/doctrine.toml.example`** — commented `[estimation]` / `[value]`
   sections documenting the unit defaults.

Affected surface: `src/main.rs`, `src/value.rs` (new), `src/estimate.rs`,
`src/dtoml.rs`, `src/slice.rs`, the catalog/hydrate seam, `install/doctrine.toml.example`.

## Non-Goals

- **Re-deriving the contract.** PRD-014 / SPEC-020 are reconciled and authoritative;
  this slice implements them, it does not revise them.
- **Merging `dispatch/101*`.** Stale and conflicting; port by hand.
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

- Both facets reachable from `main`: `mod value;` present, `estimate.rs` no longer
  dead (external refs exist), `[estimation]`/`[value]` parsed by `dtoml.rs`.
- Facet display renders in slice detailed view; graph hydration exposes both.
- Value tests V1–V7 + estimate tests green; **behaviour-preservation gate** — the
  existing suites stay green unchanged.
- `just gate` clean (workspace), `cargo clippy` zero warnings, `spec validate` clean.
