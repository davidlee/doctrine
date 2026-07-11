# Implementation Plan SL-217: Elicitation queue

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, one per architectural layer of the design (D4's module split),
each landing green and behaviour-preservation gated. The design
(`design.md`, 18-decision ledger, triple-reviewed) is the canonical reference;
this plan sequences it and pins the verification mandates minted from design
§5. No design work remains — every phase implements pinned decisions.

- **PHASE-01 — Query predicates** (`comparison/query.rs`): the pure leaf.
  Feasible-region reasoning (D7/D8), the `determined` extremum algorithm
  (D9), signed hypothetical yield (D10/D11 groundwork), and the D8 proof's
  backing property suite with a test-only backtracking extension oracle.
- **PHASE-02 — Queue assembly** (`priority/elicit.rs` + `priority/config.rs`):
  the engine. Three candidate sources (comparison pairs, median-probe D14,
  anchor-review D12), D13 ranking, D15 state machine, D17 bare-estimate mask,
  and the `[priority.elicit]` config seam with named consts.
- **PHASE-03 — CLI surface** (`commands/compare.rs`, render, JSON, e2e): the
  thin shell. `compare elicit` verb (D1), human render with the pinned D15
  wording, JSON schema v1 (D16), read-only guarantee (D18), and the
  end-to-end suites (determinism, capture loop, cost ceiling).

## Sequencing & Rationale

Strictly bottom-up along the dependency chain: predicates are consumed by
assembly, assembly by the verb. Each phase's tests need only that phase's
layer plus what is already shipped — PHASE-01 tests without priority
fixtures (D4's stated payoff), PHASE-02 rides existing priority fixtures,
PHASE-03 exercises the binary surface. No file overlap between phases except
`priority/mod.rs`/`comparison/mod.rs` one-line module registrations, so the
behaviour-preservation gate (existing suites green, unchanged) is checkable
at every phase boundary, not just at audit.

Config consts ride PHASE-02 (first consumer), not a separate phase — a
constants-only phase would end unexercised.

Test placement follows repo idiom: unit batteries inline in their module
(`#[cfg(test)]`, as in `project.rs`/`channels.rs`); cross-layer e2e in
`tests/e2e_compare_elicit.rs`, patterned on `tests/e2e_compare_inference.rs`
(which carries the SL-213 shuffled-load-order determinism suite this slice
extends).

VT keyword mandates are contracts, not guesses: names like
`extension_oracle`, `coupling_boundary_infimum`, `capture_loop_round_trip`,
`cost_ceiling_eval_corpus` prescribe test identifiers the phase must land, so
`verify-vt` has real teeth (IMP-209).

## Notes

- **Implementation traps already pinned at design — do not re-derive:**
  synthetic hypothetical rows need fresh session identity (R3 trap, §2);
  negative anchors are legal (no positivity assumption, D7); scores via
  `total_cmp`, never float keys; BTree collections only (HashSet
  clippy-disallowed).
- **Est-cost inputs** come from `priority::graph` (`CostCtx` bare-item
  anchor); frontier order from `priority::order::frontier_order` via the
  existing `surface.rs` seam. Nothing new touches disk (D18).
- **Behaviour preservation** is a standing entry/exit condition, not a final
  audit step: EN-2 (PHASE-01) records the green baseline; each phase's EX
  requires pre-existing suites unchanged; PHASE-03 VA-1 audits the whole
  slice diff.
- **Out of plan scope** (filed follow-ups, not phases): IMP-281 (REQ kinds
  join VALUE_BEARING), curation skill over the JSON surface, challenger
  fringe, Phase E scoping context, D7 demotion knob.
- RFC-011 case-notes instrumentation applies throughout execution.
