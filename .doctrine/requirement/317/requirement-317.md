# REQ-317: Repair-to-integrate propagation contract

## Statement

A repair committed on a `review_surface` candidate does **not** affect trunk: trunk
integration consumes the admitted `close_target` OID (REQ-316) and never auto-propagates
`review_surface` changes. For such a repair to reach trunk, a `close_target` must be
admitted whose source carries it — sourced from the repaired candidate
(`--source refs/heads/candidate/<N>/<label>`), or given the fix by cherry-pick onto the
close-target followed by re-admission.

## Rationale

The decoupling is an intentional consequence of admit-by-OID (REQ-316), not a defect:
the substrate never auto-propagates a mutable-branch change into the trunk payload. The
operator-facing obligation this creates — choosing the right `--source` at `/close` and
not silently dropping a candidate repair (the `/close` default `--source review/<N>` is a
no-repair straight-through) — is owned by the close/audit **process** (SPEC-021), not the
substrate. This requirement fixes only the substrate fact that process relies on. (Born
from RV-116; the substrate and the code already agreed — the gap was that it was never
written down.)
