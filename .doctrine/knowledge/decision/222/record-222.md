# DEC-222: Capsule results normalize to one canonical commit per phase

## Context

A capsule worker has broad local authority and may use Git history however its
harness finds useful. That local history is evidence, not canonical project
history. Carrying arbitrary worker history across the authority boundary would
make canonical commit shape, review, and verification depend on worker choices.

## Decision

Each accepted capsule result is normalized by the trusted control plane into
exactly one canonical commit for the completed phase. Capsule-local commits may
be retained as forensic evidence, but they do not enter canonical history.

The published result is pinned and verified before normalization. The canonical
commit records the phase result against its contracted base under the admission
journal and expected-tip compare-and-swap rules.

## Consequences

- Workers may commit freely without acquiring canonical-history authority.
- Review and audit receive one stable result commit per phase.
- Full-history preservation is not a v0 canonical mode; adding it would require
  a later explicit decision.
- The publication contract still needs to settle how an unfinished dirty worktree
  becomes the pinned capsule result; this decision fixes the canonical side only.
