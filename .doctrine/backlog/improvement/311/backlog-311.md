# IMP-311: Zero-rescue dispatch Cluster 2 — `dispatch next` state machine

**Source:** RFC-016 pass (2026-07-24). **Home:** RFC-016 (Cluster 2, the open front).
Cluster 1 (moves B/C — false-red elimination) landed via SL-224 / SL-225; this is
the RFC's undelivered *core*.

## Problem

Dispatch cadence still lives as **prose the orchestrator must hold** (SPEC-021 D1,
eight-step ordered contract) plus a **memory corpus of recovery idioms** recalled
mid-run — ~40–49k/phase ceremony context and rescue-archaeology burden (RFC-011
evidence). The zero-rescue target — LLM never diagnoses/sequences/recalls; it asks
the tool what's next, does it, halts on refusal — is not built.

## Fix direction (RFC-016 moves A + E + D-tail)

Per RFC-016's routing, this is **Revision first (SPEC-021/022 prose), then a slice**:

1. **Revision on SPEC-021/022** introducing:
   - **Move A** — the funnel as an explicit state machine in the binary; every
     funnel verb checks legality and refuses out-of-order naming the expected next
     verb; a new **`dispatch next`** verb emits the single prescribed action.
     Generalises SL-199's confined-arm machine to one machine, three doors
     (main-thread / subprocess / confined) — OQ-2.
   - **Move E** — no-shell-git-in-funnel: ship read verbs covering *every* funnel
     read (OQ-7 enumerate first), then prohibit shell git inside the funnel.
     **Absorbs ISS-234** (coord-worktree reverse-diff) — the durable fix is
     read-verbs making tree state irrelevant, **not** an interim auto-sync
     (so **IDE-028** is explicitly the wrong path). First OQ-6 memory-retirement:
     checkout-import recovery idioms retire when shell-git leaves the funnel.
2. **Slice(s)** implementing `dispatch next` + the read verbs + the **memory-blind
   benchmark** (OQ-5): a fresh orchestrator with zero dispatch memories completes a
   standard run plus the top-5 quirk scenarios by following verb outputs alone.
   OQ-5's *prioritisation* is already done (RFC-011 case-notes mined 2026-07-23);
   what remains is the harness + measurement, against the Cluster-1-cleaned baseline.

## Progress

- **Step 1 (Revision) — DONE (2026-07-24).** Landed as **REV-032** (approved +
  applied, `done`), reshaped introduce-only per adversarial review **RV-300**. It
  introduced 6 `pending` forward-intent requirements: SPEC-021 FR-008/009/010/011
  (REQ-384/385/386/387), SPEC-022 FR-010/011 (REQ-388/389). The 4
  active-requirement modifies (REQ-287/293/294/318) were deferred to a **ship-time
  sibling REV** at slice close (RV-300 F-1/F-3, retrospective-charter constraint).
- **Step 2 (Slice) — NEXT.** The descending slice builds the funnel state machine +
  `dispatch next` + git read verbs and the OQ-5 memory-blind benchmark, reconciling
  the 6 FRs `pending → active`. Item stays open until that slice lands.

## Open questions carried from RFC-016

OQ-1 (`next` prescribes + verbs refuse — lean both), OQ-2 (run-state home; one
machine vs per-arm; SL-206 nomination collapses main-thread + spawned orchestrator),
OQ-3 (`dispatch bundle export/ingest` metadata), OQ-4 (candidate auto-sourcing;
default close_target ← repaired review_surface), OQ-5 (benchmark harness), OQ-6
(memory disposition), OQ-7 (read-verb coverage enumeration — blocks the no-shell-git
rule).

## Residual move-D tail (frame here, may split)

IMP-174 (split-brain close), IMP-201 (split-lineage bundle), IMP-304 (superseding
candidate replaces a Failed/Pending trunk row — rescue→verb). RFC-015 (governance
ref namespace) is the structural complement; C/D moves stay valid under either
RFC-015 outcome.

## Drafting sources

RFC-016 (moves A–E, alternatives, OQs); RFC-011 case-notes-analysis; SL-199 design
(state-machine-behind-tools precedent); SL-206 unjail-direction + PHASE-08 de-risk
findings; SPEC-021 / SPEC-022.

Related: RFC-016, RFC-011, RFC-015, SPEC-021, SPEC-022, SPEC-012, SL-199, SL-206,
ISS-234, IDE-028, IMP-174, IMP-201, IMP-304.
