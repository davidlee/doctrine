# REQ-312: Evidence refs are immutable: zero-oid CAS, report-not-clobber

## Statement

`review/<N>` and `phase/<N>-NN` are created exactly once under **zero-oid CAS** — the
create succeeds only if the ref does not already exist. A stale evidence ref from a
prior run causes the command to **fail with a report; it is never overwritten**.
Evidence refs are never advanced after creation.

## Rationale

Reconstructable, immutable evidence is what makes audit trustworthy: an auditor can
re-derive a published evidence ref and rely on it not having moved. Clobber-on-stale
would silently destroy the prior run's evidence and break that guarantee; report-not-
clobber surfaces the collision for an explicit operator decision.
