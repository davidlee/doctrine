# REQ-315: CAS journal recovery contract with worktree-aware advance

## Statement

Every projection is mediated by a journal committed to `dispatch/<N>` **before any
external ref mutation**. Each row carries
`{target_ref, expected_old_oid, planned_new_oid, applied_new_oid, status}`. Replay is an
idempotent 3-way classification against the live ref: `current == planned_new_oid` →
no-op; `current == expected_old_oid` → advance; otherwise → **refuse and report a moved
target**. The advance leg is worktree-aware: a not-checked-out target advances by pure
`update_ref_cas` (post-CAS re-probe + resync if newly checked out); a checked-out target
advances by `merge --ff-only`; a non-ff advance on a checked-out ref is **refused**, and
a dirty checked-out target fails the whole integrate with zero refs moved.

## Rationale

Intent-before-action plus idempotent replay makes any interrupted sync recoverable by
re-running it — no partial, unrecorded state. Worktree-awareness prevents the phantom
reverse-diff (ISS-022/030) where a pure `update-ref` moves a ref out from under a live
index/worktree (SL-121).
