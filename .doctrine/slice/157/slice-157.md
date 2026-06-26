# Checkout-independent integrate

## Context

`dispatch sync --integrate` advances the trunk ref (`main`) at the end of a
dispatch run. Today it has **two legs**, branching on whether the target ref is
checked out (`advance_row`, `src/dispatch.rs:1812`):

- **`advance_checked_out`** (`dispatch.rs:1859`) — the "dumb" leg: `git merge
  --ff-only` in the live worktree, syncing ref + index + tree together. Refuses
  non-FF (`integrate-nonff-checkout`, line 1885).
- **`advance_pure_ref`** (`dispatch.rs:1822`) — pure CAS (`update_ref_cas`) when
  the target is **not** checked out; re-probes and resyncs any worktree that
  raced onto the ref.

The checked-out leg is the source of **RFC-005's H2** correctness hazard: keeping
a live shared checkout in step with a moving ref is where the phantom-revert
(ISS-038), the None-leg RacedDesync window (R1), and the IMP-122 resync sharp
edges (R3/R4) all live. SL-121 closed the tracked-dirty phantom chain with the M4
dirty pre-gate (`dispatch.rs:1753`); a low-likelihood None-leg window survives.

**Decisive premise (confirmed 2026-06-26):** `main` is **never worked in** — it
exists purely as a contention-buffer ref. Nothing reads a checked-out `main` as a
working folder. So the live-checkout leg buys nothing the ref + CAS doesn't, and
its only legacy is the hazard class.

**Direction (RFC-005 OQ-5 steer):** make integrate **checkout-independent** —
advance the trunk/edge refs purely against the git object DB, never via a live
worktree. This *dissolves* R1/R3/R4 at the mechanism rather than guarding each
window. The **FF-only trunk posture (ADR-012 D2/D4) is preserved unchanged** — a
non-FF trunk advance still refuses at plan time. (Auto-merging a non-FF trunk is
an ADR-reversing capability, split to **RFC-006** for external review — see
Non-Goals.)

**The building block already exists** — `update_ref_cas` (`git.rs:913`) is the
universal CAS primitive; `advance_pure_ref` (`dispatch.rs:1822`) already lands the
trunk through it whenever the ref is not checked out. This slice *promotes that
existing leg to the sole mechanism* and retires the checked-out twin — DRY on a
proven seam (no parallel implementation), not greenfield.

## Scope & Objectives

The one landing seam — **policy-free**, agnostic to how `planned_new_oid` was
derived:

```
land(ref, planned_new_oid, expected_old):
  current == planned_new_oid → no-op            (idempotent replay, D4)
  current != expected_old    → refuse           (moved target, D4)
  else                       → update_ref_cas(ref, planned_new_oid, expected_old)
```

FF-policy stays at **plan time** (`plan_trunk_row:1984` ff-gate, unchanged): a
non-FF trunk still refuses before any ref moves. Keeping the land seam free of
FF/derivation policy is the deliberate forward-compat seam for RFC-006 (which adds
a merge-at-plan oid producer without touching `land`).

1. **Object-DB CAS becomes the sole advance mechanism.** `advance_row` classifies
   (no-op / moved / advance) then lands every real advance through
   `update_ref_cas`, regardless of checkout state. Retire `advance_checked_out` /
   `ff_advance_in_worktree` from the integrate path.

2. **One policy-free landing seam.** Both trunk (FF-gated planned oid) and edge
   (force-moveable aggregate) feed the same CAS lander. Don't fork it — that is
   how divergence returns. The seam is agnostic to oid derivation, so RFC-006's
   non-FF merge producer drops in without rework.

3. **Drop the live-checkout dependence.** Remove the integrate-path checkout
   assumptions the object-DB model makes moot: the M4 dirty pre-gate
   (`dispatch.rs:1753`), the post-CAS resync (`advance_pure_ref:1842-1848`,
   `resync_worktree_hard`), and the None-leg RacedDesync disposition. `main` need
   not be checked out at all for the buffer role (OQ-A — confirm, then stop
   requiring it; drop the `main` worktree if clear).

4. **ADR-012 mechanism Revision.** The change restates *how* integrate advances a
   ref — always object-DB CAS, no checkout leg — leaving the **FF-only trunk
   posture (D2/D4) and the CAS-replay safety contract (D4) intact**. Every advance
   stays a 3-arg CAS; no force-push, no auto-resolve, non-FF still refused. Route
   the mechanism change through a Revision per ADR-013.

5. **Behaviour-preservation.** The integrate safety semantics stay green
   *unchanged*: idempotent replay (no-op if `target==planned`), moved-target
   refusal, FF land, **non-FF refusal** (`integrate_trunk_refuses_non_fast_forward`,
   e2e:803, stays as-is), clobbered-prepared-ref refusal. Tests in
   `tests/e2e_dispatch_sync.rs` (PHASE-05 integrate set, 727–927) are the proof;
   the `git.rs` primitive unit tests for the retired legs
   (`ff_advance_in_worktree_*`, `resync_worktree_hard_*`) adapt as those legs go.

## Non-Goals

- **B — non-FF trunk auto-merge + conflict surgery (RFC-006).** Merging a
  concurrently-advanced trunk (`clean non-FF → merge-tree → commit-tree → CAS`),
  the ephemeral private surgery worktree, and the IMP-127 hand-resolved-conflict
  ingest **reverse ADR-012 D2/D4's FF-only posture**. That is an ADR-reversing
  decision routed to **RFC-006** for external review, *not* this slice. SL-157's
  policy-free land seam is built so RFC-006 extends it without rework. The H2
  shared-trunk-race self-unblock (mem.close-integrate-shared-trunk-race) is the
  payoff there, not here.
- **R2 — `/close` recovery procedure.** The missing ISS-030 recovery is a cheap,
  independent skill fix; worth landing regardless but **not** this slice's
  mechanism (carry separately).
- **Candidate-flow rewrite.** The candidate object-DB merge already works; SL-157
  does not touch it (and, being FF-only, does not yet reuse its merge leg —
  RFC-006 will).
- **RV baton / coordination-worktree placement (ADR-012 D1, D5).** The
  coordination worktree as the run's SSoT + the RV-refuses-on-fork posture are
  untouched except where integrate stops needing a checked-out *trunk*.
- **The split-brain authored-state hazard (IMP-174).** Adjacent, separate axis.
- **Non-dispatch / solo integrate paths**, if any exist outside this seam.

## Summary

Retire integrate's live-checkout leg; advance the trunk/edge refs purely through
the object-DB CAS lander (`update_ref_cas`) regardless of checkout state. `main`
is a contention-buffer ref, never a working folder, so the live checkout bought
only hazard — RFC-005 H2's R1/R3/R4 dissolve at the mechanism. The FF-only trunk
posture and D4 CAS-replay safety are preserved unchanged; non-FF still refuses at
plan time. The ADR-012 Revision is mechanism-only. The non-FF auto-merge
capability that *reverses* FF-only is split to RFC-006 for external review; the
land seam is kept policy-free so RFC-006 extends it without rework.

## Follow-Ups

- **RFC-006** — non-FF trunk auto-merge + conflict surgery (IMP-127), the
  ADR-012 D2/D4 reversal; external review gates any Revision.
- R2: `/close` ISS-030 recovery procedure (independent skill fix).
- Confirm and, if clear, drop the `main` worktree entirely (buffer is the ref +
  CAS, not a folder).
