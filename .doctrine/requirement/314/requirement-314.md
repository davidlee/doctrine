# REQ-314: Two-stage projection — trunk isolated to stage-2

## Statement

The projection runs in two stages; the substrate guarantees their **separation**, not
their sequencing (audit-gating *between* them is orchestrator process, SPEC-021, not
enforced here). As discrete checks:

1. **Stage-1** (`dispatch sync --prepare-review`) materialises `review/<N>` +
   `phase/<N>-NN` and commits the CAS journal, writing **nothing to trunk**.
2. Stage-1 evidence-ref creation is zero-oid CAS / report-not-clobber (REQ-312): a
   re-run while those refs already exist **refuses with a stale-ref report** and does not
   refresh them in place. Stage-1 is therefore not freely re-runnable — a re-run after a
   `refresh-base` advance requires the prior evidence refs to be absent first.
   (Idempotent *replay* is stage-2's property, REQ-315, not stage-1's.)
3. **Stage-2** (`dispatch sync --integrate --trunk`) is opt-in and is the **only** step
   that moves trunk.
4. A stage-1 never followed by stage-2 leaves trunk untouched and `dispatch/<N>`,
   `phase/*`, `review/*` intact — the substrate never auto-integrates.

## Rationale

Isolating every trunk write to a separate opt-in step is what lets the orchestrator
interpose audit between preparation and integration (ADR-012 D4/D5). The substrate's job
is only to make that interposition *possible* — a trunk-free stage-1 and an opt-in
stage-2 — not to enforce that audit actually ran; that enforcement is process (SPEC-021).
