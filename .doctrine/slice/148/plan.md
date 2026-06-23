# Implementation Plan SL-148: Git-ref reservation backend

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases, sequenced by one load-bearing constraint: the **behaviour-preservation
gate**. The numbered callers' existing suites are the proof that `LocalFs` and the
observable CLI are unchanged (design I3, ASM A2). So the plan front-loads the *risky
but inert* refactor (the seam + the ~10 call-site swap) and proves it green before a
single line of remote git code exists, then adds the remote capability strictly behind
the new seam, and isolates the one behaviour-gate-sensitive act — the default-reach
flip — into a reversible final phase.

The phase cut mirrors the design's own phasing drivers (design §5.2 F-3, §9, notes
"What the planner needs to know"):

- **PHASE-01** is the *blast radius*, not the *new feature*. F-3 found the real change
  is mechanical breadth — every Fresh materialise site swapping `&LocalFs` for one
  `reserve::backend(root)?` helper — not the seam signature. Landing this alone, green,
  retires the largest risk (a wide refactor) under the cheapest proof (the existing
  suite, unchanged).
- **PHASE-02** builds the remote plumbing in `git.rs` and, critically, the
  **classification** that R2 made load-bearing — proven in isolation with units + the
  bare-remote substrate before any allocator depends on it. A transport/auth/policy
  failure misread as a CAS rejection would corrupt the retry loop; this phase exists to
  make that misread impossible (`--porcelain`, F-9/F-10) before it can do harm.
- **PHASE-03** is the feature: the `GitRef` backend, the `[reservation]` config, and
  `resolve_backend` as the single selection/degradation point. It ships behind a
  `local` default so back-compat (POL-002) holds the instant it lands.
- **PHASE-04** is the survey read path (REQ-022) — independent of the write path,
  orderable in parallel, placed after the backend so it has refs to list.
- **PHASE-05** flips the default to `auto`. The *mechanism* lands in PHASE-03; this is
  only the default value plus a stdout-suite sweep, kept last and trivially reversible.

## Sequencing & Rationale

**Why the gate is PHASE-01's exit, not a later checkpoint.** The seam enrichment (D1)
touches a dozen modules. If GitRef code rode in alongside it, a suite failure could not
be localised to "refactor" vs "new behaviour". Forcing the suite green with `LocalFs`
still the only backend makes PHASE-01 a pure, reviewable identity transform — the
agent-verified parity check (VA-1) plus the byte-identical suite (VT-1) are the whole
contract.

**Why remote plumbing precedes the backend (PHASE-02 before PHASE-03).** The R2 risk
(first remote surface; a transport error must never read as `AlreadyHeld`) is settled at
the lowest possible layer — `push_ref_cas`'s porcelain classification — and unit-proven
there. PHASE-03 then consumes a primitive whose failure taxonomy is already trustworthy,
so its own tests can focus on *allocation* correctness (contention, reach, content-free)
rather than re-litigating error parsing. The bare-remote substrate is built in PHASE-02
because it is the substrate for both phases' cross-clone VTs (R5: jail-safe, no network).

**Why the auto contract lands in PHASE-03 but the default flip waits for PHASE-05.**
D8 (the user-decided fail-closed `auto` with operator opt-in fallback) is *behaviour* and
belongs with the backend that implements it — it is tested in PHASE-03 under an explicit
`reach=auto`. What is risky is making `auto` the **default**, because that is what could
move stdout under tooling that asserts on it (R4). Separating "auto works" (PHASE-03)
from "auto is default" (PHASE-05) means the flip is a one-line, isolated, reversible
change whose only job is to prove the gate still holds.

**Dependency shape.** Strictly serial 01 → 02 → 03 → (04 ∥ 05-after-04). PHASE-04 could
in principle run alongside PHASE-03's tail, but it reads refs the backend creates, so the
plan keeps it after. PHASE-05 must be last: it depends on both the backend (03) and a
green survey (04) so the final suite sweep covers the whole surface.

## Notes

**Deferred-to-execution detail** (design §10, OQ-3): the exact `--force-with-lease`
create-flag form (`:<zero>` vs `:`) and the empty-tree oid source (well-known constant vs
`mktree`) are confirmed against the bare-remote substrate in PHASE-02 (VT-2), not assumed
now. lazyspec prior art (`scratch/lazyspec.git.research.md`, MIT) ships the zero-oid form
and is the crib for the git mechanics (push-by-oid of a dangling commit, glob-fetch,
`cat-file` committer-timestamp parse, FIFO mock).

**Spec reconcile is not a phase** — it is a `/reconcile` follow-up (design R7): SPEC-008
prose for the remote reservation ref class + the new `git.rs` remote ops, a SPEC-022
cross-reference, and a PRD-005 §6 note recording the D8 tightening. No code; tracked, not
scheduled here.

**Out of scope, unchanged:** `LocalFs`, the `max(local ∪ trunk) + 1` algorithm, and
`validate`/`reseat` (slice Non-Goals). The mixed-reach collision (E5/A3) stays a
documented limit backstopped by `validate`/`reseat`, not defended in code this slice.

**Deliberate two-pass call-site touch.** PHASE-01 keeps `reserve::backend(root)` minimal
(returns `Box<dyn Claim>`, LocalFs) so the gate proof is a pure seam swap. PHASE-03 then
widens it to `(Box<dyn Claim>, ReservedIds)` to seed the scan, re-touching the same Fresh
call-sites. This is accepted, not an oversight: introducing an always-empty `ReservedIds`
in PHASE-01 would be speculative generality, and the second pass is mechanical and
re-proven by the gate. The alternative (final signature in PHASE-01) was rejected to keep
the behaviour-gate phase a minimal identity transform.

**Risk the plan still carries into execution:** PHASE-03 is the heaviest phase (backend +
config + resolve_backend + loop wiring + six VTs). If `phase-plan` finds its task
breakdown oversized, the natural fault line is config/`resolve_backend` (selection +
degradation, LocalFs-only-testable) ahead of the `GitRef` backend + contention VTs —
split there rather than mixing the cut.
