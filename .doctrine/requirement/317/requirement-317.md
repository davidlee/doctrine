# REQ-317: Repair-to-integrate propagation contract

## Statement

A repair committed on a review-surface candidate does **not** auto-flow to the
close-target. To land such a repair on trunk, the operator must source the close-target
candidate from the repaired candidate (`--source refs/heads/candidate/<N>/<label>`), or
cherry-pick the fix onto the close-target and re-admit. The `/close` default
`--source review/<N>` is the legacy straight-through path, correct only when no repair
happened on the candidate. The chosen source must be explicit at close.

## Rationale

The decoupling is an intentional consequence of admit-by-OID (REQ-316), not a defect:
the model never auto-propagates a mutable-branch change into the trunk payload. The
failure it guards against is silent — a fix made on the candidate that never reaches
trunk because the close-target was built from raw evidence. Stating the contract makes
the propagation step a conscious operator choice (born from RV-116).
