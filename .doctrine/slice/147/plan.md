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
- **PHASE-04 (record-delta + dispatch write)** supplies real data to the registry
  on both arms. It is separated from P2 (the engine writer) because it is the *CLI
  surface + dispatch integration* — the one touch to a live dispatch path, kept
  thin under sole-writer. The solo arm's explicit-recording contract (no automatic
  beat) and ref ergonomics (OQ-conf-1) settle here.
- **PHASE-05 (burn domain_map + re-point staleness)** is independent of P2 and P4,
  and of P3's *algebra* — but its glob→fileset re-point consumes the shared
  `glob_matches` leaf that P3 lifts (D6), so it lands **after that lift** (P5 EN-2);
  it must not re-implement matching. It is the riskiest surgery on a live subsystem,
  so it is sequenced late where it cannot block the killer consumer. The
  behaviour-preservation gate is explicit:
  the staleness *computation* is the invariant; only its input fixtures migrate
  (F-4 scopes the re-point to slice-backed RVs so non-slice RV targets fail clean,
  not silently).
- **PHASE-06 (skills + dogfood)** wires the lifecycle and proves value. The dogfood
  is deliberately concrete (F-8): SL-147's own boundaries are recorded *explicitly*
  via `record-delta` from **live-captured per-phase HEAD oids** (the capture ritual
  below), not auto-captured and not reconstructed from git log. If the ritual lapsed,
  the fallback is a forward slice run end-to-end through the wired path (P6 EX-3) —
  never reconstruction, which would be the POL-002 archaeology this slice exists to
  kill.

## Notes

- **Phase-land capture ritual (ALL phases, from PHASE-01).** The dogfood (P6 EX-2)
  needs SL-147's own per-phase boundaries, but `record-delta` lands in P4 and the
  `/execute` auto-capture in P6 — so the early phases have no automatic capture. To
  keep the dogfood POL-002-clean (live-observed oids, never git-log reconstruction):
  at **every** phase land, jot two oids into the phase sheet/notes —
  `--start` = `git rev-parse HEAD` before the phase's first commit, `--end` = HEAD
  after its last commit and **before any trunk merge**. Keep each phase's commits
  **contiguous on edge** — the F-6 guard rejects merge/non-ancestor ranges but does
  *not* exclude a foreign commit interleaved on edge, so contiguity is what keeps
  `start..end` equal to the phase's real source-delta. Phase sheets are runtime
  state but survive to P6 (in-loop lifetime, pre-close). Miss it on a phase → use the
  P6 EX-3 forward-slice fallback, do not reconstruct.
- **OQ-conf-2 (record-delta namespace)** — resolved: `slice record-delta` for v0.1
  (not a neutral cross-arm verb); revisit if a non-slice writer appears.
- **OQ-conf-1 (solo ref ergonomics)** — resolved in PHASE-04 EX-3: `--start` =
  pre-phase HEAD (captured at phase start), `--end` = post-phase HEAD (captured at
  phase completion, before any trunk merge). PHASE-06 wires `/execute` to capture
  and pass these.
- **`primary_worktree` bare-repo edge** — out of scope for doctrine's working-tree
  operation; PHASE-02 EX-3 requires the call sites surface a clean named error
  rather than panic.
- **Parallelisation** — P1/P3/P4 all touch `src/commands/*`, so they are *not*
  file-disjoint; P5 (`src/review.rs`) is disjoint but late. Plan for serial
  execution; do not dispatch-parallelise without re-checking disjointness.
- **Deferred (not in any phase)** — glob-breadth *lint* (IMP-162), MCP reader,
  per-PHASE attribution, verb sub-tags, target sum type, durable post-close
  registry (design Non-goals).
