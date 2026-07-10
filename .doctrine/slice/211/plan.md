# Implementation Plan SL-211: Split-lineage close recovery

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The design ships one capability: a `dispatch sync --record-integration --trunk
<ref>` stage that records a Verified journal trunk row over an already-landed
payload, so a split-lineage dispatched slice reaches `done` without a hand-edited
journal. The gate (`ledger.rs::trunk_integration`) is unchanged — it already
accepts a row whose planned oid is an ancestor of live trunk. The work is
therefore concentrated in `src/dispatch.rs` (source seam, planner, verb, handler),
a one-block copy edit in `src/slice.rs` (the IMP-169 §C prescription), and a
doc-tier reconciliation of the memories the verb supersedes.

Four phases, sized to the seams they touch and ordered so each rests on a green
predecessor.

## Sequencing & Rationale

- **PHASE-01 — extract the shared payload seam first.** The recorder's earned
  surface is, per SPEC-022 (RV-263), *identical* to integrate's trunk payload:
  phase-chain tip (legacy) / admitted `close_target` (candidate, refuse if none).
  That decision lives inline in `integrate()` today and is duplicated across the
  two advancing planners. Extracting `resolve_trunk_payload` before building the
  recorder is what keeps R2 honest — one seam, three consumers, no parallel
  implementation. It lands first as a behaviour-preserving refactor so the existing
  suites (the behaviour-preservation proof, VA-1) pin it before new behaviour piles
  on. Doing it after the recorder would tempt a second copy of the resolution.

- **PHASE-02 — the pure planner + earned check (the R1 crux) on top of the seam.**
  `plan_recorded_trunk_row` is pure over OIDs (`is_ancestor` is the only git call),
  so it is unit-testable in isolation before any CLI or worktree fixture exists.
  Front-loading the earned check here — the row is refused unless the payload is
  genuinely an ancestor of trunk — means the integrity gate is proven at the
  cheapest layer, red/green, before the verb can be driven at all. The row shape
  (`expected_old = planned`, D2) is fixed here because its replay-safety is a
  property of the row, not the handler.

- **PHASE-03 — wire the verb, handler guards, and prescription; prove it
  end-to-end.** Only now does the CLI stage, the `deliver_to`-match guard (F-4), and
  the existing-row replace/no-op logic (F-2) go in, plus the `slice.rs` prescription
  (IMP-169 §C). The e2e (VT-1) is the load-bearing test: land a payload out-of-band,
  record, reach `done`. This phase is where the two-file touch-set (`dispatch.rs` +
  `slice.rs`) meets; run serially under `/execute`.

- **PHASE-04 — reconcile the doc tier last.** The recovery memories currently
  instruct `merge --no-ff review/<N>` and, in the SL-190 case, hand-writing the
  journal row — both invalidated by this slice (the sanctioned land is now the
  *payload*, recorded by the verb). Leaving them stale is an active footgun, so the
  correction is slice work, not a loose closure note; but it can only be written
  truthfully once the verb is shipped and green, so it comes last. Verified by agent
  inspection + human acceptance (no test judges memory prose).

## Notes

- **`ledger.rs` is deliberately untouched.** No phase edits the gate; its existing
  suites staying green is the behaviour-preservation evidence that the recorded
  ancestor row is accepted without weakening the fail-closed branches.
- **FF-only (ADR-012 D2/D4) is never approached.** The recorder advances no ref; it
  commits a single journal row. There is no `--force` or earned-check bypass in any
  phase.
- **Housekeeping (not a phase, do before execute):** reap the stale IMP-169
  reservation `refs/doctrine/reservation/IMP/169` (design OQ-1).
- The RV-263 external review (both findings verified closed) is the design-lock
  evidence this plan descends from.
