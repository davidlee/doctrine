# Reliable conformance-registry capture

## Context

RFC-004 v0.1 (SL-147) shipped the audit conformance consumer: `slice
conformance <SL>` diffs declared `design-target` selectors against the slice's
**actual** git delta. The actual-side input is the **arm-neutral conformance
registry** — `.doctrine/state/slice/NNN/boundaries.toml` (runtime tier) — which
must carry **one `[[boundary]]` row per landed phase** (`phase`,
`code_start_oid`, `code_end_oid`). The consumer fail-closes when the registry is
absent or incomplete, so an unpopulated registry makes conformance unavailable at
audit — exactly when it is wanted.

Two landing paths feed that registry, and both are leaking in practice:

- **Solo (in-tree) path** — `state.rs::capture_phase_boundary`, bound to
  `set_phase_status`: stamps `code_start_oid` on the `in_progress` flip, records
  `(start, end=HEAD)` on the `completed` flip. **ISS-051**: SL-147's final phase
  (PHASE-06) got no row; more broadly the final phase of a slice is the fragile
  case. Whether the true defect is the `completed`-flip timing (HEAD read before
  the phase's last commit lands) or a missed transition is for `/design` to
  root-cause — the *observable* is a missing final-phase row.

- **Dispatch funnel path** — the funnel is meant to record each landed phase's
  boundary into the conformance registry. **ISS-052**: SL-153 (dispatched
  P03/P04) produced **no** conformance-registry rows at all; the SHAs existed
  only in the *dispatch ledger* (`.doctrine/dispatch/153/boundaries.toml`, a
  different file). The funnel's conformance-registry write never fired.

SL-153 sharpens both: it is **mixed-mode** — P01/P02 landed solo (before the
dispatch drive started at `ab2c642f`), P03/P04 dispatched — yet the registry was
entirely empty, so *both* paths failed on the same slice. The post-SL-152
topology matters here: SL-152 converged worker *creation* onto one seam, so the
old "claude arm vs subprocess arm" axis is gone. The right axis for **recording**
is the landing path — **funnel-landed vs solo-landed** — not the harness.

North star: **every landed phase, by either landing path, deposits exactly one
conformance-registry row — no missed final phase, no missed funnel write —** so
`slice conformance` runs at audit without manual `record-delta` bootstrap.

## Scope & Objectives

1. **Solo final-phase capture (ISS-051).** Make the solo binding deposit the
   final phase's row reliably. Root-cause the missing-final-phase observable
   (completion-flip HEAD timing vs missed transition) and fix at the
   `state.rs::capture_phase_boundary` / `set_phase_status` seam so the last
   phase is no weaker than the interior ones. No double-record with the funnel
   path (the existing branch arm-guard stays correct).

2. **Funnel conformance-registry write (ISS-052).** Make the dispatch funnel
   populate the conformance registry — one row per landed phase — as an
   **enforced beat** at `prepare-review` (the pre-audit conclude beat), not an
   orchestrator-issued `slice record-delta` step that is documented yet
   skippable. A derive reads the dispatch boundaries ledger and upserts each row
   into `state/slice/NNN/boundaries.toml`; a primary-rooted completeness gate
   `bail!`s on any gap so a slice cannot reach audit incomplete.

3. **Mixed-mode coherence.** A slice whose phases land by different paths (some
   solo, some funnel — SL-153) must still end with a complete registry. The two
   writers must compose without gap or duplication across the mode boundary.

4. **`record-delta` stays the escape hatch.** The manual
   `slice record-delta <SL> PHASE-NN --start --end` verb remains for correction /
   bootstrap; this slice removes the *need* to use it on a normal slice, it does
   not remove the verb.

5. **Commit the dispatch boundaries ledger (absorbs ISS-039).** ISS-052's
   spec-legal fix is blocked on ISS-039: SPEC-022 §"Run-ledger object-db
   sourcing" mandates the run ledger — *including* `boundaries.toml` — be
   tree-read from the `dispatch/NNN` tip, never the working filesystem,
   identically stage-1/stage-2. Today the claude arm never commits
   `boundaries.toml` to the branch, so `read_ledger` reads empty (this is why
   `plan_phases` projects 0 phase-cuts). This slice commits the ledger onto
   `dispatch/NNN` alongside `journal.toml` (a `prepare-review` splice mirroring
   `commit_journal`), bringing the impl into SPEC-022 conformance. Then the
   derive *and* `plan_phases` read the same committed source — no working-file
   read, no F4 divergence, and claude per-phase review cuts are restored.
   Bounded to the **claude arm**; the codex/pi phase-ref coupling stays IMP-171.

Affected surface (coarse — `/design` refines):
- `src/state.rs` — `capture_phase_boundary`, `record_source_delta`, the
  `set_phase_status` binding (solo path; ISS-051).
- `src/dispatch.rs` — `prepare_review`: splice-commit the boundaries ledger onto
  `dispatch/NNN` (absorbs ISS-039), then derive the registry from the committed
  ledger + primary-rooted completeness gate (ISS-052).
- `src/ledger.rs` — dispatch-ledger reader/writer; the source of the SHAs the
  derive mirrors, and the working-file reader for the splice-commit.
- `src/boundary.rs` — `BoundaryRow` gains a `provenance` field (`Solo|Funnel|
  Manual|Unknown`), the per-phase landing-path discriminator the projection-source
  guard (D11) keys on (design Rev 4 / D12).
- Dispatch skills (`dispatch`, `dispatch-subprocess`) — drop the skippable
  orchestrator `record-delta` instruction once the funnel beat is enforced.

## Non-Goals

- **codex/pi symmetric ledger + derive (IMP-171).** The dispatch boundaries
  ledger is claude-arm-only; a symmetric codex/pi ledger couples to `phase/<N>`
  projection turning on unconditionally — deferred. The ISS-039 commit absorbed
  here is bounded to the claude arm.
- **Worker creation / base integrity** (RFC-005 H1, SL-152) — converged already;
  not touched here.
- **Selector authoring adoption** — that design-time `design-target` selectors
  get seeded is a `/slice` + `/design` skill-flow concern (SL-153 had none);
  noted as a follow-up, not this slice's mechanism.
- No change to the conformance consumer (`slice conformance`) or its algebra —
  this slice only fixes its *input substrate*.
- No new authored tier; the registry stays runtime/disposable.

## Summary

Close the two conformance-registry population leaks RFC-004 v0.1 left: the solo
binding's missing final-phase row (ISS-051) and the dispatch funnel's never-fired
conformance-registry write (ISS-052), so every landed phase — by either landing
path — deposits exactly one boundary row and `slice conformance` runs at audit
without a manual bootstrap. Stands alone as an RFC-004 follow-up.

## Follow-Ups

- Selector-authoring adoption: wire `/slice` + `/design` to seed `design-target`
  selectors so the declared side is populated too (SL-153 had none).
- IMP-171: codex/pi symmetric ledger + derive (couples to phase-ref projection).
