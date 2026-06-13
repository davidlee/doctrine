# REQ-192: Enforce worker-sole-writer via a disk-marker-primary, fail-closed guard: refuse authored/Orchestrator/Hook-mint writes under `worker_mode = (is_linked_worktree && marker_present) OR env DOCTRINE_WORKER`; a marker-absent linked worktree is fail-closed; reads and the fork-side read verbs stay open; `write_class` is exhaustive (a new verb is a compile error).

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
