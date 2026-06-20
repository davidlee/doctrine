# REQ-315: CAS journal recovery contract with worktree-aware advance

## Statement

Every projection is mediated by a journal whose rows carry
`{target_ref, expected_old_oid, planned_new_oid, applied_new_oid, status}`. The contract
is five discrete checks:

1. **Journal-before-mutation:** the journal is committed to `dispatch/<N>` before any
   external ref mutation.
2. **3-way replay classification** against the live ref: `current == planned_new_oid` →
   no-op; `current == expected_old_oid` → advance; otherwise → **refuse and report a
   moved target**.
3. **Not-checked-out advance:** pure `update_ref_cas`, with a post-CAS re-probe that
   resyncs (or warns on a dirty tree) if the ref is now checked out.
4. **Checked-out advance:** `merge --ff-only` in the target worktree so ref, index, and
   tree move together; a **non-ff** advance on a checked-out ref is **refused**
   (`integrate-nonff-checkout`), never reset.
5. **Dirty pre-gate:** a dirty checked-out target fails the whole integrate before the
   first journal commit (`integrate-dirty-worktree`), zero refs moved.

## Rationale

Intent-before-action plus idempotent replay makes any interrupted sync recoverable by
re-running it — no partial, unrecorded state. Worktree-awareness prevents the phantom
reverse-diff (ISS-022/030) where a pure `update-ref` moves a ref out from under a live
index/worktree (SL-121).
