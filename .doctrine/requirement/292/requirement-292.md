# REQ-292: Integration is two-stage and audit-gated: stage-1 sync --prepare-review materialises review/<slice> and phase/<slice>-NN plus a CAS journal committed before any ref mutation and writes no trunk; audit runs from the parent/root against the prepared refs; stage-2 sync --integrate is opt-in, fast-forward-only, expected-tip-CAS, and reports (never auto-resolves, never force-pushes) a moved/non-ff target.

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
