# Doctrine audit phase

The audit phase sits between implementation and closure. It is evidence
gathering, conformance checking, and reconciliation — not code review.

## Where it fits

```
route → slice → design → plan → phase-plan → execute → audit → close
```

The audit phase has its own skill (`/audit`) and its own artifact: the
**review ledger** (RV kind, ADR-007).

## The review ledger (RV kind)

`doctrine review new --facet reconciliation --target SL-NNN` creates a
structured audit substrate. The ledger is turn-based with a baton: one party
raises findings, the other disposes, verifies, contests, or withdraws them.
Every finding carries a severity and status.

Key verbs:
- `doctrine review raise` — record a finding.
- `doctrine review dispose` — resolve a finding.
- `doctrine review verify` — confirm a resolution.
- `doctrine review contest` — challenge a resolution.

The close gate (D-C9b) refuses `→reconcile`/`→done` while an RV targeting the
slice carries an unresolved blocker.

## Audit vs code review

Audit is conformance against design and plan. Code review is code quality and
correctness. They are separate skills (`/audit` vs `/code-review`) and produce
separate evidence. The audit phase may surface findings that warrant code
review, and vice versa.

For the full review ledger verb surface and coordination protocol, see
[[signpost.doctrine.review]].

See [[signpost.doctrine.lifecycle-start]] for the full lifecycle,
[[signpost.doctrine.requirements]] for coverage reconciliation,
and [[pattern.doctrine.core-loop]] for the workflow loop.
