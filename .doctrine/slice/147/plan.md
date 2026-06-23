# Implementation Plan SL-147: Audit path-conformance delta

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases deliver RFC-004 v0.1: a slice carries an accreting `[[selector]]` list,
each phase's source-delta SHA is recorded into an arm-neutral runtime registry, and
`slice conformance` computes the declared-vs-actual path algebra — the killer
consumer. The reviewer-role migration (burning `domain_map`) rides alongside but is
deliberately kept off the critical path. RV-148 findings F-2…F-8 fold into the
criteria below; F-1 (the RFC self-contradiction) was discharged upstream in
RFC-004's reconciliation, not in any SL-147 phase.

## Sequencing & Rationale

The order follows the dependency spine, not the design's narrative order:

- **PHASE-01 (selector schema + CLI)** is the foundational authored surface. It has
  no consumers yet, so it is safe to land first and lets every later phase assume
  selectors exist. Batch-first CLI (variadic `add`, one shared `--intent`) is the
  ergonomic requirement that shaped D3.
- **PHASE-02 (registry + write guard)** is the foundational *data* layer, parallel
  in concept to P1 (no dependency between them). It carries the two structural
  risks: the `BoundaryRow` leaf extraction (ADR-001 cycle avoidance — a pure move,
  gated by the unchanged ledger/dispatch suites) and the cross-worktree resolver
  (reuse `primary_worktree`, never reinvent — F-5). The write-time ancestor +
  non-merge guard (F-6) makes "trunk contributes nothing" *enforced*, not assumed,
  so it belongs with the writer, not the reader.
- **PHASE-03 (algebra + conformance)** is the north star. It depends on P1
  (selectors) and P2 (registry reader). It can be built and tested against
  *synthetic* fixture registries, so it does not wait on P4's real writers. The
  completeness check (F-2, a blocker fix) is *delivered* here because it is a
  *read-time* guard — but it is **authored at `state.rs` altitude** (design D7),
  reading the phase sheets where state lives, not in the conformance/command shell;
  P3 only invokes it. `net()` (F-3) and matched-selector transparency (F-7) are
  pure and unit-tested here.
- **PHASE-04 (solo binding + record-delta as dispatch recorder + manual)** supplies
  real data on every arm. It is separated from P2 (the engine writer) because it is
  the *wiring* layer — command handlers + dispatch integration — over P2's
  writer+guard. The solo capture is **deterministic**: it rides the `slice phase`
  transitions `/execute` already issues (`in_progress`→code_start,
  `completed`→code_end+guard+upsert), no off-critical-path act — the explicit-call
  contract is superseded now that a real CLI hook is confirmed (D5). OQ-conf-1
  dissolves. **Dispatch-compat is the load-bearing integration** (EX-2/EX-4): the
  binding skips in a dispatch coordination context (orchestrator HEAD is the
  coordination base, not a phase delta), and the dispatch funnel records instead by
  calling `slice record-delta --start B --end B+1` at its record beat — the same
  verb, the same writer. **All three arms are covered**: codex/pi-dispatch records
  identically because both dispatch arms run the *shared* funnel and produce the
  same per-phase B→B+1 (the historical claude-only `record-boundary` skip was about
  its committed `phase/<N>` ref-cut consumer, not oid availability — that artifact
  is left untouched). Solo binding and dispatch recorder are mutually exclusive per
  phase.
- **PHASE-05 (burn domain_map + re-point staleness)** is independent of P2 and P4,
  and of P3's *algebra* — but its glob→fileset re-point consumes the shared
  `glob_matches` leaf that P3 lifts (D6), so it lands **after that lift** (P5 EN-2);
  it must not re-implement matching. It is the riskiest surgery on a live subsystem,
  so it is sequenced late where it cannot block the killer consumer. The
  behaviour-preservation gate is explicit:
  the staleness *computation* is the invariant; only its input fixtures migrate
  (F-4 scopes the re-point to slice-backed RVs so non-slice RV targets fail clean,
  not silently).
- **PHASE-06 (skills + dogfood)** wires the lifecycle and proves value. With the
  deterministic binding (P4), SL-147's own **post-binding** phases (P4 onward)
  auto-record as they complete — a genuine on-the-deterministic-path proof.
  Pre-binding phases (P1..P3) carry no rows; either bootstrap them via `record-delta`
  for a full self-diff, or let conformance report `incomplete` for them — itself a
  live demonstration of the F-2 backstop. A separate forward slice (P6 EX-3) is the
  fully-clean, zero-bootstrap proof.

## Notes

- **Going-forward capture is deterministic — no ritual.** Once the P4 binding
  lands, every phase on every slice auto-records its boundary at the `slice phase`
  transitions (solo) or the dispatch beat (dispatch). There is no "remember to do
  X" on anyone's critical path. The *only* manual residue is a **one-time
  bootstrap**: SL-147's own pre-binding phases (P1..P3) ran before the binding
  existed, so they hold no rows. If a full SL-147 self-diff is wanted, `record-delta`
  them — and capture each one's start/end HEAD oid *live* during execution (not from
  git log; reconstruction = the (SL-NNN) archaeology POL-002 forbids), keeping each
  phase's commits contiguous on edge so `start..end` is the real source-delta. This
  is bounded to three phases of one slice, then gone — not an ongoing posture. If
  skipped, conformance simply reports `incomplete` for P1..P3 (F-2 working), and the
  forward-slice proof (P6 EX-3) covers the clean case.
- **Dispatch-compat (the binding's one real assumption, now guarded — all three
  arms covered).** The solo binding captures `HEAD`, correct only where `HEAD` ==
  the phase's code-end — true solo (inline-on-edge or a `/worktree` fork), false in
  dispatch (the orchestrator flips status from the coordination tree at base `B`).
  Verified facts: dispatch records at the funnel beat and does **not** call `slice
  phase` — so the binding is solo-only by construction, and P4 EX-2 *enforces* it
  (skip on a doctrine-owned `dispatch/<N>` coordination signal — POL-002-clean). The
  dispatch arm records by calling `slice record-delta --start B --end B+1` at the
  funnel record beat (P4 EX-4) — **on both arms**: claude AND codex/pi run the shared
  funnel and produce the same per-phase B→B+1, so codex/pi is covered with no
  claude-only gate. The historical claude-only `record-boundary` skip was about its
  committed `phase/<N>` ref-cut consumer (ledger.rs), not oid availability — that
  artifact is left untouched. One known edge: (i) the gate must key on
  *dispatch-coordination*, not *any linked worktree*, or it would wrongly skip a solo
  fork. The codex/pi recorder is verified by its own dispatch fixture (P4 VA-1).
- **OQ-conf-2 (record-delta namespace)** — resolved: `slice record-delta` for v0.1
  (not a neutral cross-arm verb); revisit if a non-slice writer appears.
- **OQ-conf-1 (solo ref ergonomics)** — **resolved by construction** (D5): the solo
  arm passes no `--start`/`--end` on the happy path. `code_start = HEAD` at the
  `slice phase … in_progress` transition, `code_end = HEAD` at `… completed`. The
  ref-choice question dissolves; `record-delta`'s explicit refs survive for the
  manual fallback only.
- **`primary_worktree` bare-repo edge** — out of scope for doctrine's working-tree
  operation; PHASE-02 EX-3 requires the call sites surface a clean named error
  rather than panic.
- **Parallelisation** — P1/P3/P4 all touch `src/commands/*`, so they are *not*
  file-disjoint; P5 (`src/review.rs`) is disjoint but late. Plan for serial
  execution; do not dispatch-parallelise without re-checking disjointness.
- **Deferred (not in any phase)** — glob-breadth *lint* (IMP-162), MCP reader,
  per-PHASE attribution, verb sub-tags, target sum type, durable post-close
  registry (design Non-goals).
