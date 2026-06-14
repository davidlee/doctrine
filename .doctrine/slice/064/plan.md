# Implementation Plan SL-064: Coordination-branch isolation: dedicated worktree + integration-sync seam for dispatch

Prose companion to `plan.toml`. Narrative only - no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md section reference forms. -->

## Overview

SL-064 ships the accepted ADR-012 topology and the remaining ADR-006 placement
amendments in three layers:

1. Lock the governance boundary and OQ-D fence.
2. Build the coordination-tree and projection mechanisms in code.
3. Rewire the dispatch/worktree skills and prove the full lifecycle end to end.

The phase boundary deliberately keeps the new projection machinery below the CLI
until its git plumbing and committed run ledger are testable on their own. The
sync verb is then split by ADR-012's two stages: prepare-review first, integrate
and replay second. Skill prose lands after those command surfaces exist, so the
skills cite real behavior rather than aspirational shell recipes.

## Sequencing & Rationale

PHASE-01 is first because SL-064 has two governance owners. ADR-006 owns the
placement and identity refinement; ADR-012 owns topology, routing, projection,
and the D1 tightening. The plan-gate for OQ-D also belongs up front: the slice
must not accidentally turn marker absence into a claimed identity proof while it
adds a markerless coordination tree.

PHASE-02 isolates the coordination branch before projection exists. The
coordination worktree is the inner-loop foundation: once the funnel writes
`dispatch/<slice>` instead of the session tree, the shared-main contention
surfaces are structurally out of the per-batch path.

PHASE-03 builds the reusable primitives: filtered tree composition, commit-tree,
CAS ref update, and the committed run ledger. This keeps the most delicate git
work out of command-flow code and gives prepare-review and integrate the same
tested substrate.

PHASE-04 implements stage-1 prepare-review. It materializes the exact objects
audit will inspect, without writing trunk. This is the first externally visible
projection point and the right place to wire `dispatch sync` into the
Orchestrator-verb guard.

PHASE-05 implements stage-2 integration and replay. It is separate from
prepare-review because it has different failure semantics: target refs may have
moved, audit has already happened, and the command must be idempotent after
partial application.

PHASE-06 updates the source skills after the commands are real. This rewrites
the human/agent operating loop from "the coordination branch is the deliverable"
to "coordination is the SSoT, review/phase refs are the deliverables, audit gates
those refs, and integration is explicit."

PHASE-07 is the system proof. The earlier phases cover pieces; this phase proves
the user-visible claim of SL-064: dispatch runs in a dedicated coordination
worktree, leaves reviewable refs, keeps unreviewed code off trunk by default,
removes only the worktree directory at conclude, and preserves behavior of the
existing funnel suites.

## Notes

- `IMP-065` is not implemented here. SL-064 ships the transitional markerless
  coordination path with the OQ-D fence and impersonation tests; the positive
  coordination marker remains the dedicated follow-up.
- `IMP-041` is resolved by the new lifecycle: worker/coordination worktree
  directories are cleanup targets, while `dispatch/*`, `review/*`, and `phase/*`
  refs survive until their integration lifecycle says otherwise.
- `IMP-043` is no longer a per-batch import concern. Moved-target handling lives
  at sync time through CAS refusal and replay.
