# Implementation Plan SL-213: Comparison constraint layer

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases, tracking the design's tier structure one-to-one: wire (PHASE-01) →
resolution (PHASE-02) → compilation (PHASE-03) → projection (PHASE-04) →
priority wiring (PHASE-05) → surfaces + determinism (PHASE-06). The design
(`design.md`, review-hardened under RV-265 + a parallel external pass) is
canon; every phase objective cites its D/R/C/P/S rule ids rather than
restating semantics.

## Sequencing & Rationale

The order is forced by data dependency: each tier consumes exactly the
previous tier's output artifact (rows → active set → `ConstraintSet` →
projection → `value_dim`). Two deliberate couplings and one deliberate split:

- **PHASE-01 bundles capture with the wire model.** The schema redefinition
  (design D1 — v1 retired in place) breaks `commands/compare.rs` at compile
  time; landing them together keeps every phase boundary green. The module
  split (`comparison.rs` → `comparison/`) also happens here, while the diff
  is smallest.
- **PHASE-05 bundles the store move with the wiring.** `load_sessions` moves
  to `comparison/store.rs` in the same phase that gives it its second caller
  (the priority build shell) — moving it earlier would be a churn-only phase.
  This is also where the behaviour-preservation gate bites: EN-2 records the
  green baseline, EX-1 holds the empty-ledger identity.
- **Projection (PHASE-04) is split from compilation (PHASE-03)** even though
  both are pure: the prototype battery is large, and the C-rules and P-rules
  fail independently — separate phases keep red/green cycles small and the
  obligation-1 golden (C3) from tangling with placement goldens.

TDD note: PHASE-02–04 are pure-function phases — tests are inline `mod tests`
in the new files, red first, per rule id (the VT keywords mirror this).
PHASE-06 carries the single new e2e file (`tests/e2e_compare_inference.rs`)
so the golden surface lives in one place.

## Notes

- **Premise check (plan-time re-grep, 2026-07-11):** `effective_raw_value`
  (`src/priority/graph.rs:109`), `DEFAULT_VALUE` (`:103`),
  `build_from_with_cfg` (`:298`), `PriorityConfig::load_from_table`
  (`src/priority/config.rs`), `load_sessions` (`src/commands/compare.rs:273`),
  `kinds::VALUE_BEARING`, cordage `CycleDiagnostic` + provenance accessors —
  all live as the design states. One nit: `listing.rs` exports `Format` but no
  `RenderOpts`; `compare list` extends compare.rs's own `render_row`/ordering
  helpers instead (design §4 S2's "existing machinery" resolves to that).
- **Cordage reuse decision point** sits inside PHASE-03: attempt
  `CycleDiagnostic`-shaped reuse first; the sanctioned fallback is a local
  Tarjan in `compile.rs` with a deliberate-duplication comment (design §1).
  Neither outcome changes the phase's criteria.
- **GAUGE_STEP** enters as a pure parameter in PHASE-04 and gets its config
  home (`priority/config.rs`, default 0.25) in PHASE-05 — the sensitivity
  sweep runs in PHASE-04 where the parameter is free.
- **VH-1 (PHASE-06)** doubles as the Phase C entry-criterion dry run: capture
  a small real ledger over this repo's backlog and eyeball the explain/list
  story. Its outcome feeds the RFC-019 Phase C gate, not this slice's close.
- Risk watch (from the slice): degradation quality — if the quarantine
  findings read as a wall of pairs on the VH-1 dry run, that is a finding
  against S4's rendering, not against the C-rules; route it to `/feedback`
  rather than re-opening compilation semantics.
