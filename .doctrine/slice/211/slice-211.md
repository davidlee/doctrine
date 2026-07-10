# Split-lineage close recovery

Batches **IMP-236 + IMP-169** — one shippable change: a dispatched slice whose
code reached trunk by a sanctioned route that leaves no journal **trunk row**
(operator direct-land, manual merge, external integration) cannot reach `done`,
because the close-gate demands a trunk row no verb writes for it. This slice
makes the variation **recordable** and the gate **recognise** it.

Scope narrowed from the original three-item batch: **IMP-127** (ingest a
hand-resolved 3-way merge) is split to **SL-212** — its clean non-FF auto-merge
face reverses ADR-012 D2/D4 FF-only and is gated on RFC-006 external review. This
slice keeps only the two moves that record a row / recognise an already-landed
tip, which do **not** touch the FF-only invariant.

## Context

The sanctioned dispatch close chain is a ladder: `slice status done` refuses
without a journal trunk row → only `sync --integrate` writes it → integrate
refuses without an admitted `close_target` → … So when the operator lands the
reviewed code by any route *other than* the dispatch-native integrate (a manual
merge onto trunk when pre-dispatch `edge→main` promotion was skipped; a
direct-land after a candidate conflict), the code is on trunk, green, reviewed —
but no trunk row exists and `done` is unreachable.

The close-gate (`ledger.rs::trunk_integration`) distinguishes: journal
absent/zero-rows → `NotDispatched` (waves through); journal has rows but none
target trunk → `Blocked("no trunk row")`. A funnel-driven dispatched slice
journals `review/<N>` + `phase/<N>-NN` rows, so a non-native land hits the second
arm and refuses — even though trunk genuinely holds the reviewed code. Neither
integrate path writes that row for this shape: `plan_candidate_trunk_row` needs
an admitted `close_target`; `plan_trunk_row` needs a ff the split lineage forbids.

**RFC-016 §C + §D is the governing frame.** §C: "IMP-169 (recognise
manual/external integration) and IMP-236 (direct-land records a trunk row) are
the same move" — an operator-carried contract becomes a refusal-with-prescription
in the gate. §D: every legal variation gets a first-class recorded row at the
moment it happens ("manual land → trunk row"); the contrapositive is the belt — a
variation that cannot get a row is refused at the point of variation, not
discovered at close. Contract text stays in SPEC-022 as rationale; enforcement
lives in the verb.

**Observed cost, live lifecycle debt:** SL-147 (manual merge over an advanced
trunk, stranded at `reconcile`), SL-190 (hand-wrote a verified trunk row into
`journal.toml` by hand to force `done`). Both are shipped-but-lifecycle-incomplete
today. Recurs on any base drift.

Memory (acceptance oracle): `mem.pattern.dispatch.split-lineage-close-conflict-direct-land`
(the SL-190 corrected finding — the hand-written trunk row is exactly what this
slice mechanises), `mem.pattern.dispatch.close-deadlock-refresh-base-recovery`
(high-trust), `mem.pattern.dispatch.close-preff-trunk-absorbs-repair`.

## Scope & Objectives

1. **Record a trunk row for a sanctioned non-native land (IMP-236).** A verb that
   writes the verified journal trunk row (`target_ref = trunk`, `planned_new_oid`
   = the landed tip, an ancestor of trunk) once the reviewed code is on trunk by
   direct-land / manual merge — replacing the hand-edited `journal.toml` SL-190
   resorted to. The row must be *earned*: the recorded tip must be an ancestor of
   the live trunk and carry the reviewed surface (no unreviewed code waved
   through).
2. **Close-gate recognises manual/external integration (IMP-169).** The
   `trunk_integration` gate accepts the row written by (1) — and, where no
   candidate/journal path can run at all, refuses with a **prescription** naming
   the record verb, not a dead end. RFC-016 §C refusal-with-prescription.

Objective: a split-lineage dispatched slice whose code is genuinely on trunk,
green, and reviewed reaches `done` through a sanctioned, provenance-checked path —
no hand-edited `journal.toml`, no forfeited integrity — **without touching
ADR-012 FF-only**.

## Non-Goals

- **IMP-127** — ingest a hand-resolved 3-way merge / clean non-FF auto-merge.
  Split to SL-212; reverses ADR-012 D2/D4, gated on RFC-006. This slice records a
  row over a tip the operator *already* landed; it does not merge onto trunk.
- **Prevention** of split lineage (IMP-201 code-tier / IMP-174 authored-tier) —
  separate efforts; IMP-174 coordinates with RFC-015.
- The broader RFC-016 machine (`dispatch next` state machine, candidate
  auto-sourcing, bundle export/ingest) — this slice is the §C/§D beachhead, not
  the whole direction. Design must not preclude it.
- A blanket `--force` or any bypass of the row-earned check.

## Affected surface (coarse — /design refines)

- `src/ledger.rs` — `trunk_integration` close-gate; trunk-row validation.
- `src/dispatch.rs` — trunk-row write path; the record verb's plumbing.
- CLI surface for the record verb (`dispatch sync` / journal family).
- Close skill + dispatch-mechanics memory — the SL-190 hand-write pattern retires
  into a verb; the two recovery memories become the acceptance oracle.

## Risks / Assumptions / Open Questions

- **R1** The recorded trunk row must be *earned* — validate the tip is an
  ancestor of live trunk and carries the reviewed surface, or the gate degrades
  to a rubber stamp (the integrity the gate exists to give).
- **R2** IMP-236 and IMP-169 must share **one** recorder/gate seam (two sources —
  candidate-less land vs external merge — one row schema), not parallel
  implementations. RFC-016 §D: rows consumed mechanically downstream.
- **OQ-1** IMP-169 carries a **stale dispatch reservation**
  (`refs/doctrine/reservation/IMP/169`, 2026-06-24, empty tree). Confirm no
  active drive and reap before execute.
- **OQ-2** Is IMP-236's row-write a distinct verb from IMP-169's external-
  integration recognition, or one verb with two sources? (R2.) Resolve in
  `/design`.
- **OQ-3** Does the existing `close-preff-trunk-absorbs-repair` pattern already
  cover part of IMP-236 (pre-FF trunk so the standard integrate writes the row)?
  If so this slice may only need IMP-169's recognition + a prescription. Resolve
  in `/design`.
- **A1** The pure/imperative split holds — ref/merge ops in the thin shell;
  row planning/validation pure (pass OIDs in).

## Verification / Closure intent

- Replay the SL-147 / SL-190 shapes through the new record verb to `done` with no
  hand-edited journal and no forfeited integrity.
- Negative: an *un-earned* row (tip not an ancestor of trunk, or missing the
  reviewed surface) is refused.
- Behaviour-preservation: existing dispatch/ledger suites stay green unchanged;
  the native (clean-base) integrate + close path is byte-unchanged.
- The two recovery memories re-verified against the shipped verbs (stale →
  current).

## Follow-Ups

- SL-212: IMP-127 ingest (RFC-006 / ADR-012 Revision gated).
- Batch B: IMP-201 (code-tier prevention); IMP-174 (authored-tier, coordinates
  with RFC-015).
- The full RFC-016 zero-rescue direction (`dispatch next`, auto-sourcing).
