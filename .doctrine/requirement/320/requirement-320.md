# REQ-320: Crash-safe idempotent recovery: no force-push, no auto-resolve

## Statement

The projection path's quality envelope: **no operation force-pushes a ref or
auto-resolves a moved/conflicted target.** A moved target is reported and the operation
halts; a conflicted `refresh-base` leaves markers and `MERGE_HEAD` for manual resolution.
(The journal-before-mutation + idempotent-replay mechanics that make an interrupted sync
safe to re-run are REQ-315; this requirement owns only the never-force / never-auto-resolve
envelope those mechanics operate within.)

## Rationale

Concurrent and foreign writers make a moved target a normal event, not an error to
paper over. Refusing to force or auto-merge keeps every ref advance an explicit,
recoverable step and makes data loss structurally unreachable in the projection path.
