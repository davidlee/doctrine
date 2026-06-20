# REQ-314: Two-stage audit-gated projection

## Statement

Conclude is followed by two stages with audit between them; conclude itself performs
neither. The contract, as discrete checks:

1. **Stage-1** (`dispatch sync --prepare-review`) materialises `review/<N>` +
   `phase/<N>-NN` and commits the CAS journal, writing **nothing to trunk**.
2. Stage-1 evidence-ref creation is zero-oid CAS / report-not-clobber (REQ-312): a
   re-run while those refs already exist **refuses with a stale-ref report** and does not
   refresh them in place. Stage-1 is therefore not freely re-runnable — a re-run after a
   `refresh-base` advance requires the prior evidence refs to be absent first.
   (Idempotent *replay* is stage-2's property, REQ-315 / REQ-320, not stage-1's.)
3. **Audit** runs from the parent/root against the prepared refs.
4. **Stage-2** (`dispatch sync --integrate --trunk`) is opt-in and post-audit only; it is
   the only step that moves trunk.
5. A failed audit blocks trunk integration while leaving `dispatch/<N>`, `phase/*`, and
   `review/*` intact.

## Rationale

Unreviewed code must never reach trunk. Separating projection from integration, with
audit as the gate, makes "reviewed" a structural precondition of "integrated" rather
than a discipline an operator might skip (ADR-012 D4/D5).
