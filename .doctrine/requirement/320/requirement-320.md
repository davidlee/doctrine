# REQ-320: Crash-safe idempotent recovery: no force-push, no auto-resolve

## Statement

No projection operation force-pushes a ref or auto-resolves a moved/conflicted target.
A moved target is reported and the operation halts; a conflicted `refresh-base` leaves
markers and `MERGE_HEAD` for manual resolution. Any interrupted sync is recovered by
re-running it: journal-before-mutation (REQ-315) plus idempotent 3-way replay
guarantees a re-run converges without duplicating or losing an advance.

## Rationale

Concurrent and foreign writers make a moved target a normal event, not an error to
paper over. Refusing to force or auto-merge keeps every ref advance an explicit,
recoverable step and makes data loss structurally unreachable in the projection path.
