# REQ-317: Repair-to-integrate propagation contract

## Statement

A repair committed on a review-surface candidate does **not** auto-flow to the
close-target. To land such a repair on trunk, the operator must source the close-target
candidate from the repaired candidate (`--source refs/heads/candidate/<N>/<label>`), or
cherry-pick the fix onto the close-target and re-admit. The `/close` default
`--source review/<N>` is the legacy straight-through path, correct only when no repair
happened on the candidate. The chosen source must be explicit at close.

**Normative guard.** Before creating or admitting a `close_target`, an operator/agent
MUST determine whether the reviewed `review_surface` candidate has drifted from its
recorded `merge_oid` or carries repair commits. If it has, `/close` MUST NOT use
`--source refs/heads/review/<N>` unless that omission is deliberate and documented — it
would silently drop the repair from the trunk payload. `candidate status` (the drift
report) is the mechanical signal this check reads.

## Rationale

The decoupling is an intentional consequence of admit-by-OID (REQ-316), not a defect:
the model never auto-propagates a mutable-branch change into the trunk payload. The
failure it guards against is silent — a fix made on the candidate that never reaches
trunk because the close-target was built from raw evidence. Stating the contract makes
the propagation step a conscious operator choice (born from RV-116).
