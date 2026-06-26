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
operate on the git object DB / refs by default, materialise a worktree only for
genuine conflict surgery. This *dissolves* R1/R3/R4 at the mechanism rather than
guarding each window.

**The building blocks already exist** — in the *candidate* flow, not integrate:
`merge_tree` (`git.rs:844`, `merge-tree --write-tree`), `commit_tree_merge`
(`git.rs:875`), `commit_tree` (`git.rs:818`), and the ephemeral `--worktree`
conflict-surgery pattern in `candidate_create` (`dispatch.rs:946`).
`update_ref_cas` (`git.rs:913`) is already the universal CAS primitive. git is
**2.54.0** here — `merge-tree --write-tree` is available. So this is largely
*lifting an existing object-DB merge seam into integrate and retiring the
worktree leg* — DRY on a proven seam (no parallel implementation), not greenfield.

## Scope & Objectives

Target model (the one landing seam):

```
land(ref, planned, expected_old):
  FF?          → CAS                                  (advance_pure_ref today)
  clean non-FF → merge-tree --write-tree → commit → CAS   (lift from candidate)
  conflict     → ephemeral private worktree → human resolves → same CAS  (IMP-127)
```

1. **Promote the bare leg to the default.** Make `advance_row` land via the
   object-DB path (FF→CAS; clean non-FF→`merge-tree`→`commit-tree`→CAS) regardless
   of checkout state. Retire `advance_checked_out` / `ff_advance_in_worktree` from
   the integrate path.

2. **One landing seam.** Both tree-producers (FF planned-oid; merge-tree result)
   feed a single `land(ref, tree/commit)` that does commit-tree (when needed) +
   `update_ref_cas`. Don't fork the lander — that's how divergence returns.

3. **Conflict = ephemeral surgery, same lander (absorbs IMP-127).** On a true
   conflict, materialise a *throwaway, private* worktree (never the shared
   buffer), let it be hand-resolved, and ingest the resolution through the **same**
   CAS lander. This is the integrate-side twin of the candidate `--worktree` path.

4. **Drop the live-checkout dependence.** Remove / re-scope the integrate-path
   checkout assumptions the bare model makes moot: the M4 dirty pre-gate
   (`dispatch.rs:1753`), the post-CAS resync (`advance_pure_ref:1842-1848`,
   `resync_worktree_hard`), the None-leg RacedDesync disposition. `main` need not
   be checked out at all for the buffer role — confirm and, if so, stop requiring
   it.

5. **ADR-012 Revision.** The rewrite changes the integrate *mechanism* ADR-012
   fixes (D1 isolated coordination worktree as write target; D2 trunk FF-only
   opt-in; the live-checkout integrate posture). D4's CAS-replay semantics are
   **preserved** (every advance stays a 3-arg CAS; no force-push, no auto-resolve)
   — the Revision restates *how* the tree is produced, not the safety contract.
   Route the mechanism change through a Revision per ADR-013.

6. **Behaviour-preservation.** The integrate safety semantics stay green:
   idempotent replay (no-op if `target==planned`), moved-target refusal, FF/clean
   land, clobbered-prepared-ref refusal. Tests in `tests/e2e_dispatch_sync.rs`
   (PHASE-05 integrate set, lines 727–927) are the proof; the `git.rs` primitive
   unit tests adapt as legs are retired.

## Non-Goals

- **R2 — `/close` recovery procedure.** The missing ISS-030 recovery is a cheap,
  independent skill fix; worth landing regardless but **not** this slice's
  mechanism (carry separately).
- **Candidate-flow rewrite.** The candidate object-DB merge already works; this
  slice *reuses* it, it does not re-author it.
- **RV baton / coordination-worktree placement (ADR-012 D1, D5).** The
  coordination worktree as the run's SSoT + the RV-refuses-on-fork posture are
  untouched except where integrate stops needing a checked-out *trunk*.
- **The split-brain authored-state hazard (IMP-174).** Adjacent, separate axis.
- **Non-dispatch / solo integrate paths**, if any exist outside this seam.

## Summary

Retire integrate's live-checkout leg. Land the trunk by pure object-DB merge
(FF→CAS; clean 3-way→`merge-tree`→`commit-tree`→CAS) through one CAS lander, with
an ephemeral private worktree only for genuine conflict surgery (absorbing
IMP-127). `main` is a contention-buffer ref, never a working folder, so the live
checkout bought only hazard — RFC-005 H2's R1/R3/R4 dissolve at the mechanism.
Reuses the candidate flow's proven object-DB merge seam (DRY). Mechanism change
routes through an ADR-012 Revision; D4 CAS-replay safety is preserved.

## Follow-Ups

- R2: `/close` ISS-030 recovery procedure (independent skill fix).
- Confirm and, if clear, drop the `main` worktree entirely (buffer is the ref +
  CAS, not a folder).
