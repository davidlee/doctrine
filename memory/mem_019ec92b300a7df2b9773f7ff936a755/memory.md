# Doctrine backlog work intake

The backlog is the work-intake surface — ideas, issues, improvements, chores,
and risks that may become slices but haven't been scoped yet.

## Kinds

- **issue** (`ISS-NNN`) — a bug or problem to fix.
- **improvement** (`IMP-NNN`) — an enhancement or refactor.
- **chore** (`CHR-NNN`) — a routine task, cleanup, or follow-up.
- **risk** (`RSK-NNN`) — a risk to track and mitigate.
- **idea** (`IDE-NNN`) — a suggestion to evaluate later.

## Membership test

A record belongs in the backlog, not a knowledge record or an ADR, when it:
- Has a lifecycle: `open → resolved → closed`.
- Is something someone will *do* (or decide not to do).
- Is not a standing rule (policy/standard), not a decision (ADR), and not a
  durable observation (memory/knowledge record).

## Promotion to slice

Backlog items can be promoted to slices when they are scoped and actioned.
A slice that captures a backlog item references it in its relationships.
Items that are deferred-but-needed-later must be captured in backlog or a slice
before closing the current work item out.

See [[signpost.doctrine.lifecycle-start]] for the full lifecycle,
[[concept.backlog.work-intake-membership]] for the membership test,
and [[concept.doctrine.routing-gate]] for the route-before-you-act gate.
