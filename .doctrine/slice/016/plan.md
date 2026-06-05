# Implementation Plan SL-016: Break slice↔state cycle: extract plan types

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

One phase. The change is a pure, behaviour-preserving relocation of the authored
phase-plan model out of the slice command shell into a neutral engine-leaf
module, severing the lone `slice ↔ state` import cycle (design.md §1, §5.1).

## Sequencing & Rationale

**Why one phase, not several.** The move is atomic at the compiler. You cannot
relocate `Plan`/`PlanPhase` while leaving `state.rs` and `slice.rs` pointing at
the old path — the crate stops compiling until *every* import repoints in the
same step. Splitting this across phases would strand the tree non-compiling
between them, which violates the "end each phase green" rule. So the new module,
the consumer repoints, and the test relocation all land together.

**Why no red/green cycle in the usual sense.** This is a refactor, not a
feature: there is no new behaviour to drive out with a failing test. The
discipline here is *behaviour preservation* — the existing suite is the proof
(design.md §9). The "green" we end on is the unchanged suite plus two structural
gates the move is specifically for: the crate compiles acyclically (VT-1), and
`state.rs` no longer imports `crate::slice` (VT-2). Those two are the closure
evidence that the cycle is actually gone, not merely hidden.

**Why the read/parse split shapes the work.** `Plan::parse` is pure and moves to
`plan`; `read_plan` is disk IO and stays in `slice` (design.md §7 D1, ADR-001).
This is what keeps the new module a clean engine leaf with zero in-crate deps —
the lowest-coupling outcome — rather than dragging filesystem access into the
engine. It also dictates the test split (EX-4): pure-parser tests follow `parse`
into `plan`; the renderer-and-parser contract tests stay with the renderer in
`slice`.

## Notes

Adversarial pass A3 (design.md §10) is the reason EX-4 is selective rather than
"move all parse tests": the scaffold-acceptance test depends on slice-private
`render_plan_toml`, so it is a slice-side contract test, not a pure-parser one.
Mechanical care, not new design.
