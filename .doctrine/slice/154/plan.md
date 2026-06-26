# Implementation Plan SL-154: Reliable conformance-registry capture

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases close the two conformance-registry population leaks (ISS-051 solo,
ISS-052 funnel) and the absorbed ledger-commit fix (ISS-039), built on the
provenance discriminator the design locked at Rev 6. The shape is **data model →
isolated probe → the two consumers (solo path, funnel path) → command/doc
closeout**:

- **PHASE-01** lays the `Provenance` field on `BoundaryRow` and the *incoming-keyed*
  merge inside `record_source_delta`. Everything downstream writes or reads
  provenance, so this is the keystone and goes first.
- **PHASE-02** adds the live coord-worktree probe — file-disjoint from PHASE-01
  (git.rs only), so it can run in parallel; both the solo guard and the
  ledger-commit locator depend on it.
- **PHASE-03** fixes the solo path (ISS-051): the live-worktree guard, the `Solo`
  stamp, and reopen eviction.
- **PHASE-04** commits the boundaries ledger at prepare-review (ISS-039) — the
  SPEC-022-legal source the funnel half reads.
- **PHASE-05** is the keystone of the funnel half (ISS-052): the projection-source
  guard (D11), the authoritative derive, and the primary-rooted gate, plus the
  `Funnel` stamp. It consumes PHASE-04's committed ledger.
- **PHASE-06** closes the surface: `record-delta` sets `Manual` (the PHASE-01 merge
  makes it safe), and the dispatch skills document the now-enforced beat.

No new authored tier; the registry stays runtime/disposable. The conformance
consumer and its algebra are untouched — this slice only fixes its input substrate.

## Sequencing & Rationale

**Why data-model-first.** The provenance field and its merge seam (PHASE-01) are the
single dependency every writer shares. Authoring them once, in isolation, with a
pure merge-table unit test, means PHASE-03/05/06 each just pick the right *incoming*
provenance and trust the seam — no provenance logic is duplicated across writers
(the merge is race-free precisely because it lives in one RMW, not N callers; this
was the pass-7 fix). `forget_source_delta` ships here too: it is a `record_source_delta`
sibling (the registry writer layer), even though its only caller is PHASE-03's reopen.

**Why the git probe is its own phase.** `live_worktree_for_ref` (PHASE-02) is a
self-contained git.rs change with a behaviour-preservation constraint
(`worktree_for_ref`'s signature must not move — existing callers). Isolating it keeps
that constraint auditable and lets it land in parallel with PHASE-01; PHASE-03 and
PHASE-04 both consume it.

**Why the funnel half splits PHASE-04 → PHASE-05.** PHASE-04 makes the committed
ledger *exist* (the ISS-039 commit) — restoring `plan_phases` projection and giving
the derive a source. PHASE-05 *consumes* it (guard + derive + gate). Splitting on
that producer/consumer seam keeps each phase's verification focused: PHASE-04 proves
the splice (idempotent, validated, the no-pre-commit fixture); PHASE-05 proves the
population invariants (D11 total/partial/false-halt, derive authority, gate rooting).
The ordering inside PHASE-05's `prepare_review` insert — **guard → derive → gate,
all before ref projection** — is load-bearing: the guard reads the registry before
the derive backfills it, and a halt at any step creates no refs so the operator's
record-delta → re-run is clean (design F1).

**Why record-delta + docs land last (PHASE-06).** `record-delta` setting `Manual` is a
one-line change whose *correctness* depends on PHASE-01's merge; its *integration
proof* (a bare record-delta cannot clear a funnel/legacy D11 halt) depends on
PHASE-05's guard existing. The skill docs can only describe the enforced beat once
PHASE-05 enforces it. Grouping the trivial code change with the docs keeps the
keystone phases (01, 05) clean.

**Mixed-mode coherence (objective 3)** is not a separate phase — it is an emergent
property verified across PHASE-03 (solo rows on the dispatch branch) and PHASE-05
(the union gate + D11's provenance discrimination). The SL-153 shape (P01/P02 solo,
P03/P04 funnel) is the canonical fixture for PHASE-05 VT-3.

## Notes

- **Behaviour-preservation** is a standing gate, asserted per phase: PHASE-02 keeps
  `worktree_for_ref` callers green; PHASE-03 keeps the solo stamp-present path
  byte-identical (provenance aside); PHASE-04/05 keep `e2e_dispatch_lifecycle`
  (phase/064-01) + `e2e_dispatch_sync` (incl. the double-write pin) green.
- **OQ-6 (DRY)** — whether to factor a shared `splice_ledger_file` for `commit_journal`
  + `commit_boundaries` — is a PHASE-04 implementation judgement, decided at execution
  if it reads cleanly (not a plan-level commitment).
- **No migration machinery.** Legacy `Unknown` rows are inert on closed slices and
  halt loudly on active ones (cleared by a one-time hand-reclassification) — no phase
  builds backfill (design D12, User decision).
- **Follow-ups, not phases:** IMP-172 (derived per-phase nav view) and IMP-173 (F4
  run-state ownership signal) are backlogged `after SL-154`; neither is in scope here.

## Critical plan pass (2026-06-26) — dispositions

Hostile pass over `plan.toml`/`plan.md` against `design.md` Rev 6, with the named
seams spot-checked in source. Coverage verified: every decision D1–D12 and the §5.2
worklist lands in some phase's EX/VT; no phase carries scope absent from the design.

Seeded concerns:
- **PHASE-05 size → DISMISSED (keep together).** EX-2/EX-3/EX-4 (guard → derive → gate)
  are one contiguous insert in `prepare_review`, *between* the committed-ledger read
  (dispatch.rs:1523) and the ref projection (`with_journaled_projection`, :1542),
  sharing locals (`primary`, the committed-phase set, `boundaries`). Splitting D11 from
  derive+gate would fork one code block across two phases with interleaved locals —
  *worse* coupling, not better. The real seam is producer/consumer: PHASE-04 makes the
  committed ledger *exist*, PHASE-05 *consumes* it. The 7 VTs are small, cohesive, and
  all pin the one population beat. Cut confirmed.
- **PHASE-01 merge feasibility → DISMISSED (clean).** `record_source_delta` (state.rs:613)
  already does a read-modify-write upsert via `iter_mut().find()`; the "blind whole-row
  replace" is the single line `*existing = row` (state.rs:642). Reading
  `existing.provenance` before that line is trivial — the existing row is in hand. EX-2
  wording holds. The F-6 guard (`is_ancestor` + non-merge, :618) runs unchanged on every
  call *including* the PHASE-05 derive's primary-rooted upserts; the committed-ledger oids
  resolve in the primary object-db (shared `.git`), so the guard passes (empty-code rows:
  `is_ancestor(x,x)` is true).
- **Irreducible-manual VT gap → CONFIRMED, fixed.** Added **PHASE-03 VT-5**: it pins the
  accepted floor (pure-solo lost stamp → no row → conformance `Incomplete`, named) AND
  closes EX-2's otherwise-unverified absent-stamp branch. The behaviour rides untouched
  conformance code; the VT is a don't-break assertion, the slice's natural floor.

Phase-plan watch-items (execution detail, not plan-criteria defects):
- **PHASE-02 parser restructure.** `parse_worktree_for_ref` (git.rs:1163) *early-returns*
  on the `branch` match line; git porcelain emits `prunable` *after* `branch` in a block,
  so the extended parser must switch to **block-accumulate** (read the whole block, then
  decide) to see the marker. EX-1's "surface the full block" implies it; VT-3 pins the
  `worktree_for_ref` path-return behaviour through the restructure.
- **PHASE-03 reopen-eviction ordering.** The evict-branch (clear `code_start_oid` +
  `forget_source_delta`) must run **before** the `capture_phase_boundary` call
  (state.rs:412) so a reopen → `in_progress` re-stamps a fresh start; the existing
  stamp-once guard (:505) would otherwise keep the stale stamp. EX-3 is correct on intent;
  this is the placement.
- **PHASE-04 new fixture needs a *live coord worktree*.** `commit_boundaries` only fires
  under `live_worktree_for_ref` (EX-3); the existing e2e fixtures pre-commit the ledger
  and run without one (so they no-op the splice and stay green — EX-4). VT-1's
  no-pre-commit fixture must register a real linked worktree for `dispatch/NNN`, else the
  splice never runs and the test proves nothing.

DAG note (informational): PHASE-04's EN on PHASE-01 is *soft* (a pre-PHASE-01 ledger
round-trips without the field; the upgrade adds one harmless content-idempotent ref
advance). PHASE-03 (solo half) is independent of the PHASE-04→05 funnel half — it shares
only PHASE-01/02 — so it may run parallel to PHASE-04/05; serial is the default and fine.
The scope doc names two dispatch skills; the plan's PHASE-06 EX-2 names three
(`dispatch{,-agent,-subprocess}`) matching the design-target — the plan is authoritative
(reconcile the scope doc's count at close if it matters).

**Verdict: plan is implementation-ready.** One VT added; no boundary, sequencing, or
coverage defect found. Ready to flip `ready`.

