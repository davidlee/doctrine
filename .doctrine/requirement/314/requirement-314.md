# REQ-314: Two-stage audit-gated projection

## Statement

Conclude projects outward in two stages with audit between them. **Stage-1**
(`prepare-review`) materialises `review/<N>` + `phase/<N>-NN` and commits the CAS
journal, writing nothing to trunk; it is idempotent and re-pins to the current
fork-point on re-run. **Audit** runs from the parent/root against the prepared refs.
**Stage-2** (`integrate`) is opt-in and post-audit only. A failed audit blocks trunk
integration while leaving `dispatch/<N>`, `phase/*`, and `review/*` intact.

## Rationale

Unreviewed code must never reach trunk. Separating projection from integration, with
audit as the gate, makes "reviewed" a structural precondition of "integrated" rather
than a discipline an operator might skip (ADR-012 D4/D5).
