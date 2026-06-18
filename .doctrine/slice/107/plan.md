# Implementation Plan SL-107: Estimate & Value facet integration

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-107 ports the *integration delta* SL-101 designed but never landed on `main`,
hand-copied from `candidate/101/review-001` (D3: port, don't merge — `main`
diverged ~34 commits past the candidate base, conflicting on `estimate.rs`). The
delta is ~6 mechanical file touches; design internals (types, parse, validate,
config) are already locked in SL-101 §3–§7 and are not re-derived here. PHASE-01
makes the facet *data* present, parsed, and configured; **display is SL-102
(REQ-273), graph is SL-103 (REQ-274)** — out of scope (D2 narrow boundary).

## Sequencing & Rationale

**One phase (OQ-1 resolved → single).** The design leaned single-phase on size;
the deciding constraint is dead-code ordering. The facet *types* only become
non-test-live once wired — `EstimateFacet`/`ValueFacet` go live through the
`SliceDoc` fields, `EstimationConfig`/`ValueConfig` through `DoctrineToml`. Land
`value.rs` as its own phase and plain `cargo clippy` (the project gate, bins/lib)
warns `dead_code` on the as-yet-unreferenced `ValueFacet`/`ValueConfig`, so a
value-module-only phase **cannot pass its exit gate in isolation**. Splitting
would strand a dead-code window. One phase keeps a single clean gate at exit.

Work order *within* PHASE-01 (TDD red/green/refactor, value.rs before wiring per
the design lean):

1. **value.rs leaf + `mod value;`** — port verbatim; V1–V7 + custom-deserialize
   tests green via `cargo test` (test build references every item, so it is
   green even before the types are wired into live code).
2. **Wire the types live** — `DoctrineToml` gains the two `#[serde(default)]`
   config fields; `SliceDoc` gains the two `Option<…>` facet fields + fixture
   `None, None` updates + round-trip tests. This is the step that makes all four
   facet types referenced from non-test code.
3. **estimate.rs attr swap** — only now remove the blanket `#![allow(dead_code)]`
   and add the 5 item-level `expect`s (D1). Doing it after step 2 means the
   types are already live, so the residual `expect` surface is exactly the
   display/graph helpers SL-102/103 will consume — the tripwire is honest.
4. **install example + full gate** — commented `[estimation]`/`[value]` sections,
   then `just gate` (workspace), plain `cargo clippy` zero warnings, `spec
   validate`. Commit.

The `estimate.rs` change is attr-only (D1): `main` and the candidate differ
*solely* in dead-code treatment — both already carry `espresso_shots`. No logic
merge, no test change. This is why the port is low-risk; the behaviour-
preservation gate (VT-4) is the proof, not a promise.

## Notes

- **No code without this approved plan.** Hand-port only — no `merge`/`cherry-pick`
  from candidate or `dispatch/101*` (D3; the branches are stale/orphaned reference).
- **Pure tier preserved (ADR-001):** `estimate.rs`/`value.rs` stay leaves — config
  passed in, file read in the shell. No clock/disk/rng/git added.
- **expect, not allow (D1 invariant):** if any `expect(dead_code)` fires
  "unfulfilled", a type/fn went live unexpectedly — stop and reassess scope, don't
  silence it.
