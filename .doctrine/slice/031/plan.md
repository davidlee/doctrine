# Implementation Plan SL-031: Dispatch orchestrator funnel: worker-mode workers and import-verify-commit-record

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The design carries two coupled deliverables in one slice, deliberately phased
A→B (design §1): **A** is concrete, testable, unblocked Rust on the command tier;
**B** is the dispatch funnel, mostly orchestrator skill-prose (VA) with one tested
mechanical seam. The plan keeps that ordering and splits B's mechanical building
blocks from its orchestration prose, giving three phases:

- **PHASE-01 (A)** — production trunk-aware minting at the 5 `run_new` sites plus
  the command-tier `KindRef` registry refactor.
- **PHASE-02 (B.1)** — the `branch-point-check` verb (the funnel's one VT seam)
  and the `/worktree mode=worker` contract (the worker side SL-029 stubbed).
- **PHASE-03 (B.2)** — the `/dispatch` orchestrator funnel (VA), the R-5 import
  belt, and the IMP-002 / IMP-003 reconciliation that closes the slice.

The design is locked through three adversarial passes (design §10); the plan
refines it into phases and does not reopen any settled decision.

## Sequencing & Rationale

**Why A first (PHASE-01).** A is the only fully VT-testable deliverable and it is
unblocked — the SL-032 reframe (design §2) opened the gate. Shipping it first
banks concrete value and de-risks the larger reframe before any skill-prose lands.
It is also self-contained on the command tier: the engine `Kind` is untouched
(X-4), so the behaviour-preservation gate (R-3) is the proof that the registry
refactor and the minting wiring change nothing observable in the existing suites.
The registry shape is fixed *here*, by the consumer that needs it — doing it in
SL-032 would have guessed the shape blind, then reshaped (design §5.2).

**Why the verb + worker contract next (PHASE-02).** PHASE-03's funnel *composes*
these two: the orchestrator calls `branch-point-check` at the batch-commit
boundary, and it spawns workers that honour the `mode=worker` contract. Building
and testing them before the orchestration prose means PHASE-03 wires together
pieces that already hold, not pieces still in flux. The verb is the funnel's sole
mechanical seam (OQ-2 — orchestration stays skill-prose, mirroring SL-029); the
worker contract is the half SL-029 declared but shipped only as solo.

**Why the funnel and reconcile last (PHASE-03).** The funnel is orchestration
discipline — file-disjoint batching, the strict import→verify→commit→record
cadence, the recovery prose. Its correctness rests on skill-prose conformance
(VA), not Rust enforcement, so it is honestly the last and least mechanical layer.
The IMP-002 / IMP-003 reconciliation rides here because the slice's value is only
complete once the funnel ships; the reconcile is the closure tail (`/close`
confirms it, VH).

**The cross-phase coupling that must not be lost.** A (PHASE-01) and B (PHASE-03)
are ordered for delivery but are **not failure-independent** (design §2, C-II).
A's trunk-minting guarantee protects the solo/team divergent-worktree world; it
does *not* protect the funnel workers, because under D2 the workers never mint at
all — and D2a's activation fails OPEN (no `Agent` env seam, C-I). The belt that
keeps an unarmed worker from reintroducing the exact D3 collision A removes is the
orchestrator's **R-5 import-time `.doctrine/`-path reject**, which lives in
PHASE-03. It is sound where the env is not, because the trusted sole writer
(worker-mode OFF) runs it. PHASE-03 must keep R-5 — it is the belt that closes A's
guarantee inside the funnel, not an optional hardening.

## Notes

- **Storage discipline.** Phase status is runtime state under `.doctrine/state/`,
  materialised by `doctrine slice phases`; never record progress in this file or
  in `plan.toml`.
- **Verification classes are honest (design §9).** Only the mechanical seams carry
  VTs — minting (PHASE-01 VT-1..VT-4), the `branch-point-check` verb (PHASE-02
  VT-1/VT-2). The worker contract and the whole funnel are VA (skill conformance);
  the IMP reconciliation is VH (confirmed at `/close`). This mirrors the SL-029
  precedent: a tested verb only where a guarantee must be physically mechanical.
- **No design reopening.** The §10 ledger (D2a fails-open → R-5 belt; registry
  command-tier, engine untouched; file-disjoint + serial fallback; import = net
  diff `B..S` via `git apply`, clean-tree precond, verify-isolate) is settled.
  Surface any new design problem through `/design`, not by editing the plan.
- **Per-phase expansion.** Run `/phase-plan PHASE-01` to expand the runtime sheet
  with a concrete task breakdown just before `/execute`.
