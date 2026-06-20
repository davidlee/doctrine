# Notes SL-121: dispatch sync --integrate: clean exit state and legible outcome

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-02 — worktree-aware integrate advance + report (commit 84107448)

Done; green (2005 bin units + 21 e2e incl. 5 new VTs; clippy clean). Built inline
on `dispatch/121` (claude dispatch arm abandoned — ISS-034).

**Design defect for reconcile (RECONCILE ACTION).** design §2.2 None-leg resync
names `git reset --keep planned`; it CANNOT fix the desync (the ref already
advanced under the live checkout → HEAD==planned → `reset --keep` sees no diff →
worktree stays stale; proven empirically). Corrected to `git reset --hard
<planned>` gated on the clean re-check (content-safe). User-approved via /consult.
**Fold §2.2 wording `reset --keep`→`reset --hard` (+ the clean-gate rationale)
into design.md at reconcile.** Memory:
mem.pattern.dispatch.reset-keep-cant-resync-already-advanced-ref (verified).

**Refinements (note for reconcile / audit):**
- Tracked-clean predicate lifted to `git::tree_clean` (leaf, ADR-001) so the dirty
  pre-gate, the §2.5 ff re-check, and `worktree::gather_tree_clean` share one
  `--untracked-files=no` definition. Design §5 said "reuse gather_tree_clean (no
  move)"; implemented as predicate-at-leaf (git.rs is a leaf, can't call
  worktree.rs). Behaviour-preserved.
- New git.rs shells: `ff_advance_in_worktree` (`enum FfAdvance{Advanced,Raced}`),
  `resync_worktree_hard`, `tree_clean`. dispatch.rs: `advance_row` +
  `advance_pure_ref` + `advance_checked_out` + `report_integrate`; `Disposition`
  retired `Applied` for `AdvancedResynced`/`AdvancedPureRef`/`RacedDesync` +
  `label()`.
- Exact report tokens (tests assert literally): success `advanced+resynced` /
  `advanced+pure-ref` / `no-op`; refusal tokens `integrate-dirty-worktree` /
  `integrate-nonff-checkout` / `raced-checkout-desync`.
- VT-5 (exact-CAS) has no dedicated new e2e: covered by `replay_ref` units +
  idempotent re-run + the plan-time ff-gate (existing e2e). Noted, not a gap.
- §7 concurrency boundary (VA-1) confirmed content-safe; see phase-02 sheet.

**For PHASE-03:** the integrate report now emits the trunk row's advanced
disposition (`advanced+resynced`/`advanced+pure-ref`); the committed
`dispatch/<slice>` journal trunk row carries `planned_new_oid` (=`applied_new_oid`
on success) — the stable read surface OQ-5 needs. `short_oid` abbreviates to 12.
