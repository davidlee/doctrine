# Checkout-independent integrate

> Title is a handle. Precisely: *make the never-checked-out (trunk) advance fully
> ref-based by stripping the pure-ref leg's speculative post-CAS resync.* The
> always-checked-out `edge` target keeps its safe atomic leg (see Context).

## Context

`dispatch sync --integrate` advances trunk/edge refs at the end of a dispatch run.
`advance_row` (`src/dispatch.rs:1812`) branches on whether the target is checked
out:

- **`advance_pure_ref`** (`dispatch.rs:1822`) — the not-checked-out leg: pure CAS
  (`update_ref_cas`), **then a speculative post-CAS re-probe + resync**
  (`1842-1848`): if the ref raced into a checkout during the CAS window, it
  `reset --hard`s a clean racer or warns `RacedDesync` on a dirty one.
- **`advance_checked_out`** (`dispatch.rs:1859`) — the checked-out leg: `git merge
  --ff-only` in the live worktree (`ff_advance_in_worktree`), syncing ref + index
  + tree atomically; refuses non-FF (`integrate-nonff-checkout`, 1885).

**Where the hazard actually lives (RFC-005 H2's own localization).** R1 (None-leg
`RacedDesync`, low×high), R3, R4 (IMP-122 resync hardenings) are **all** in the
pure-ref leg's post-CAS resync (`1842-1848`). RFC-005 names the checked-out leg
the **safe** one — *"FF-merge syncs ref+index+tree atomically, no phantom"* —
proven by `integrate_trunk_checked_out_ff_leaves_clean_tree` (e2e). The earlier
ISS-038 phantom was the *pre-SL-121* pure-CAS-on-checked-out path, already retired
by SL-121's leg-aware advance + M4 gate.

**The real invariants (confirmed 2026-06-26):**
- **`main` (trunk) is never checked out.** It is a contention-buffer ref
  (`git fetch . edge:main` promotes it); no worktree holds it. `worktree_for_ref`
  always returns `None` → it takes the pure-ref leg. There is **no `main` worktree
  to drop** (OQ-A) — already realized.
- **`edge` IS checked out** — AGENTS.md mandates *"the primary worktree stays on
  `edge`."* `--edge refs/heads/edge` therefore hits a **live** ref → the
  checked-out leg. Its atomic ff is the **safe** advance; force-CASing it would
  desync the dev's own tree (the ISS-038 phantom). The checked-out leg is
  **load-bearing**, not legacy.

So the speculative None-leg resync defends a None→Some transition that **cannot
happen** under these invariants: nobody checks out `main`; `edge` is *always*
already checked out (Some leg, never the None leg). It guards an impossible window
— and the guard *is* the R1/R3/R4 hazard. **Delete the condition, don't harden the
window** (RFC-005 OQ-5 steer).

## Scope & Objectives

1. **Strip the pure-ref leg's speculative post-CAS resync.** `advance_pure_ref`
   becomes CAS-and-done: on `RefCas::Updated` the disposition is always
   `AdvancedPureRef` — no re-probe, no resync, no `RacedDesync`. R1/R3/R4 dissolve
   at the mechanism. (`dispatch.rs:1842-1848` removed.)

2. **Retire `resync_worktree_hard`.** Its only production caller is the deleted
   resync; remove the fn (`git.rs:1373`) and its unit test (OQ-D — grep-confirmed
   sole caller). The `RacedDesync` disposition + its `report_integrate`
   warning-line branch go with it.

3. **Keep the checked-out leg unchanged.** `advance_checked_out` /
   `ff_advance_in_worktree` stay — they are the safe atomic path for the
   always-checked-out `edge`. `ff_advance_in_worktree` keeps its sole caller
   (OQ-D), so it and its unit tests stay.

4. **Keep the M4 dirty pre-gate.** It only ever fires for a checked-out target
   (`worktree_for_ref` is `None` for `main`), i.e. it is edge-dirty protection —
   still wanted. Unchanged (`dispatch.rs:1753`).

5. **ADR-012 mechanism Revision.** Restate the integrate mechanism: the
   not-checked-out advance is **pure ref CAS with no worktree resync** (the
   speculative None→Some resync is removed as defending an impossible transition).
   The **FF-only trunk posture (D2/D4) and the CAS-replay safety contract (D4) are
   preserved unchanged** — every advance stays a 3-arg CAS; no force-push, no
   auto-resolve; non-FF still refused. Route per ADR-013.

6. **Behaviour-preservation.** Integrate safety semantics stay green *unchanged*:
   idempotent replay (no-op if `target==planned`), moved-target refusal, FF land,
   non-FF refusal, clobbered-prepared-ref refusal, **and the checked-out ff
   regression** (`integrate_trunk_checked_out_ff_leaves_clean_tree`,
   `integrate_trunk_not_checked_out_advances_ref_leaves_live_checkout_clean`).
   Tests in `tests/e2e_dispatch_sync.rs` (PHASE-05 set, 727–1010) are the proof;
   only the `resync_worktree_hard` unit test (`git.rs:4023+`) is removed with its
   fn.

## Non-Goals

- **B — non-FF trunk auto-merge + conflict surgery (RFC-006).** Merging a
  concurrently-advanced trunk + the ephemeral surgery worktree + IMP-127 ingest
  **reverse ADR-012 D2/D4 FF-only**. Routed to **RFC-006** for external review.
  B touches `plan_trunk_row` (a merge-at-plan oid producer); it does not touch the
  advance leg this slice edits, so the two are independent. The H2
  shared-trunk-race self-unblock is B's payoff, not this slice's.
- **Pure one-leg integrate (alternative (ii), rejected).** Forcing every target
  ref to be not-checked-out (so the checked-out leg could retire) fights AGENTS.md's
  primary-on-edge mandate and is operationally hostile. Not pursued.
- **R2 — `/close` recovery procedure** (ISS-030 detector has no remedy). Cheap
  independent skill fix; carry separately.
- **Candidate-flow rewrite**; **RV baton / coord-worktree placement (D1, D5)**;
  **IMP-174 split-brain**; **non-dispatch / solo integrate paths**.

## Summary

The integrate hazard RFC-005 H2 still carries (R1/R3/R4) lives entirely in the
not-checked-out leg's *speculative post-CAS resync* — a guard against a ref racing
into a checkout mid-advance. Under the real invariants that race is impossible
(`main` is never checked out; `edge` is always already checked out and rides the
safe atomic ff leg), so the guard only adds hazard. Strip it: the pure-ref advance
becomes CAS-and-done, `resync_worktree_hard` + the `RacedDesync` disposition
retire, R1/R3/R4 dissolve. The safe checked-out leg and the M4 gate stay (edge
needs them). FF-only and D4 CAS-replay are preserved; the ADR-012 Revision is
mechanism-only. The non-FF auto-merge that *reverses* FF-only is split to RFC-006.

## Follow-Ups

- **RFC-006** — non-FF trunk auto-merge + conflict surgery (IMP-127), the ADR-012
  D2/D4 reversal; external review gates any Revision.
- R2: `/close` ISS-030 recovery procedure (independent skill fix).
