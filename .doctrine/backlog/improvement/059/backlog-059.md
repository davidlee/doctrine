# IMP-059: Unify the cross-corpus harvest pointer — /notes, /handover, /next and the review skills each re-implement the work/knowledge/decision boundary harvest

## Problem

Four skills independently re-implement the same harvest logic: at the end of a
unit of work, the agent should surface what was *produced* (the work delta),
what was *learned* (durable knowledge), and what *decisions* remain open. This
boundary — the work/knowledge/decision split — has no shared surface:

- `/notes` captures implementation observations
- `/handover` writes a context packet for the next agent
- `/next` prints a continuation prompt
- Review skills (audit/reconcile) synthesise findings into their own ledgers

Each skill asks the agent to re-survey the same ground and re-formulate the same
conclusions in a different template. The harvest is duplicated; the templates
drift; nothing enforces that a decision recorded in notes reaches the handover or
the next agent's orientation.

## Fix direction

A single **harvest surface** — a verb or skill that, at the end of any coherent
unit of work, collects the delta, surfaces open decisions, and records durable
knowledge. The existing skills (`/notes`, `/handover`, `/next`, review synthesis)
become projections of the same harvest, not independent re-implementations.

Specific shape deferred to design — but the key property is: one entry point,
one canonical output, consumed by downstream skills rather than re-derived.

## Context

RFC-011 L2' lever: compact durable handoff state. The carry-forward half — making
the cold-start surface single-copy, freshness-marked, and accretive — depends on
this unification. The worker-assembly half is now L6 (SL-186/187). IMP-060
(relocate /handover into core, **done**) was the sibling fix; this item is the
unification.

## Related

- RFC-011 L2' (compact durable handoff)
- IMP-060 (relocate /handover into core — done)
- IMP-042 (code-review skill structurally orphaned)
