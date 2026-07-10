# Split-lineage close recovery

Batches **IMP-127 + IMP-236 + IMP-169** — one wound, one shippable change: a
dispatched slice whose code reached trunk by any sanctioned non-native route
(hand-resolved 3-way merge, manual merge, direct-land) cannot reach `done`,
because the close-gate demands a journal **trunk row** that no verb will write
for a split-lineage slice.

## Context

The sanctioned dispatch close chain is a ladder — every rung gates the next:
`slice status done` refuses without a journal trunk row → only `sync --integrate`
writes it → integrate refuses without an admitted `close_target` → `admit` refuses
without a Doctrine-computed clean `merge_oid` → `candidate create` runs its own
all-or-nothing 3-way merge and records **no** `merge_oid` on any conflict. So a
single conflicting path dead-ends the whole close, and hand-resolving the git
merge does not help — `admit` validates the recorded OID, which stays empty.

The close-gate (`ledger.rs::trunk_integration`) distinguishes: journal
absent/zero-rows → `NotDispatched` (waves through); journal has rows but none
target trunk → `Blocked("no trunk row")`. A funnel-driven dispatched slice
journals `review/<N>` + `phase/<N>-NN` rows, so a direct-land hits the second arm
and `done` refuses even though trunk genuinely holds the reviewed, green code.

Neither integrate path writes that row for a split-lineage slice:
`plan_candidate_trunk_row` sources the admitted `close_target` OID and refuses
(the conflicted candidate blocks admission — IMP-127); `plan_trunk_row` (legacy)
sources the `phase/<N>-NN` chain tip and requires it to fast-forward trunk — the
exact ff split lineage forbids.

**Observed cost, live lifecycle debt:** SL-104 (direct-land escape, forfeited
CAS provenance), SL-147 (manual merge, stranded at `reconcile`), SL-190
(hand-wrote a verified trunk row by hand to force `done`). SL-147 and SL-190 are
shipped-but-lifecycle-incomplete today. Recurs on any base drift.

Memory: `mem.pattern.dispatch.split-lineage-close-conflict-direct-land`,
`mem.pattern.dispatch.close-deadlock-refresh-base-recovery` (high-trust, the
refresh-base recovery), `mem.pattern.dispatch.close-preff-trunk-absorbs-repair`
(an existing pre-FF alternative — assess for reuse before writing new merge code).

## Scope & Objectives

Three seams, one coherent recovery path:

1. **Ingest a hand-resolved merge (IMP-127).** A verb — the "it's complicated"
   path, *not* a `--force` — that adopts an operator-performed (base, source)
   3-way merge as the candidate's Doctrine merge, so `admit` has a real
   `merge_oid` to validate. The merge still happens and is still validated; the
   operator just performs it by hand when the internal auto-merge conflicts.
2. **Record the trunk row after a sanctioned direct integration (IMP-236 +
   IMP-169).** Once the resolved candidate (or an externally-merged tip) is on
   trunk, a sanctioned path writes the verified journal trunk row
   (`target_ref = trunk`, `planned_new_oid` = the landed tip, ancestor of trunk)
   so the close-gate passes — covering both the ingested-candidate route (236)
   and manual/external integration where no candidate exists (169).

Objective: any split-lineage dispatched slice whose code is genuinely on trunk,
green, and reviewed can reach `done` through a sanctioned, provenance-preserving
path — no hand-edited `journal.toml`, no forfeited CAS integrity, no `--force`.

## Non-Goals

- **Prevention** of split lineage (routing phase commits into a staging ref;
  landing authored-tier deltas cleanly) — that is batch B (IMP-201 / IMP-174).
  This slice is *recovery* only: the split has already happened.
- A blanket `--force` that bypasses the 3-way merge or the OID CAS.
- Gate-binary-not-on-edge candidate rebuild for self-modifying close (IMP-203).
- Concurrency/shared-trunk race handling beyond what the existing integrate CAS
  already provides.

## Affected surface (coarse — /design refines)

- `src/dispatch.rs` — candidate create/admit seam; `plan_trunk_row` /
  `plan_candidate_trunk_row`; integrate legs.
- `src/ledger.rs` — `trunk_integration` close-gate; trunk-row planning/validation.
- CLI surface for the new ingest + record verbs (`dispatch candidate` /
  `dispatch sync` families).
- Close skill + dispatch-mechanics memory (recovery path documentation) — the
  two stale memories above become the acceptance oracle.

## Risks / Assumptions / Open Questions

- **R1** Adopting a hand-made merge OID must not weaken the CAS provenance the
  candidate seam exists to give — the ingest must still validate the merge is a
  true 3-way of the recorded (base, source), not an arbitrary tree.
- **R2** Two write paths for the trunk row (ingested-candidate vs
  manual/external) risk divergence — prefer one recorder with two sources over
  parallel implementations.
- **OQ-1** Does the pre-FF-trunk alternative
  (`close-preff-trunk-absorbs-repair`) already cover the IMP-236 case such that
  only IMP-127 + IMP-169 need new code? Resolve in `/design`.
- **OQ-2** Should IMP-169's manual/external path be a distinct verb or the same
  record-trunk-row verb IMP-236 needs, sourced differently? (R2.)
- **A1** The pure/imperative split holds — merge/ref ops stay in the thin shell;
  planning/validation stays pure (pass OIDs in).

## Verification / Closure intent

- Replay each historical dead-end (SL-104 / SL-147 / SL-190 shapes) through the
  new path to `done` with no hand-edited journal and no forfeited CAS.
- Behaviour-preservation: existing dispatch/ledger suites stay green unchanged;
  the non-split (clean base) close path is byte-unchanged.
- The two recovery memories are re-verified against the shipped verbs (stale →
  current).

## Follow-Ups

- Batch B: IMP-201 (code-tier prevention) + IMP-174 (authored-tier split-brain).
- IMP-203 (gate-binary-not-on-edge candidate rebuild).
