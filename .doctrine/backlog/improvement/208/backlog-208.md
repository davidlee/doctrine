# IMP-208: S1 regression baseline carry-forward no-op (one suite run per batch)

Deferred from SL-170 PHASE-02 (S1 regression gate), decision D3.

## Context

The S1 funnel runs the test suite TWICE per batch in the worst case: `capture
--base B` pre-spawn, then `diff --base B` at verify. Design §5.4 / OQ-1 specified a
carry-forward optimisation: on a green diff, persist the diff's current-set (the
failure-set at `S`, which after commit == the `B'` tree) as `baseline-<B'>`, so the
next batch's `capture --base B'` is a cache hit — steady state = ONE suite run per
batch.

## Why deferred

`B'` (the post-commit HEAD) is unknown at the pre-commit `diff` step, so `diff`
cannot key the persisted set by `B'`. A sidecar mechanism (`diff` writes
`current-<fp>`; a later `capture --base B'` adopts it) is fragile against tree-drift
between the diff and the B' capture — the fingerprint (INV-8) guards env drift but
not tree content. Shipping a fragile optimisation was judged worse than re-running.

The GATE is unaffected: `capture` + `diff` + the INV-1/5/7/8 invariants all land in
PHASE-02. This is purely a cost optimisation (the gate already works, just pays one
extra suite run per first-batch / fingerprint change).

## Sketch

Persist the diff's current-set under a content-addressed key and let `capture`
verify-then-adopt only when the coord tree sha matches what the sidecar recorded
(fail-closed to a real re-capture on any mismatch). VT-8 (carry-forward
equivalence) becomes the acceptance test.

Relates to SL-170 (S1) and IMP-194 (the finding-granularity extension that reuses
the same diff core).
