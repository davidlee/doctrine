# EVD-004: Five-field change claims did not outperform existing owners in the Stage 0 trial

## Observation

RFC-027 Stage 0 reconstructed the proposed phase-local record
(`id · path · purpose · satisfies · confidence`) over two evidence-rich completed
slices: SL-224 and SL-233. Thirteen representative rows required 65 authored
cells and did not materially improve the three named consumers over the honest
baseline of selectors, intent notes, plan objectives and criteria, conformance,
and Git.

- `path` repeated selector or observed-Git facts.
- `purpose` repeated plan objectives, design prose, or an implementation-time
  inference.
- `satisfies` repeated criteria or requirements while introducing an ambiguous
  many-to-many maintenance choice.
- `confidence` was retrospective inference and changed no decision.

The strongest counterexample was SL-233 PHASE-16. Commits `80a1eba6d` and
`f8a1a0487` added `src/commands/verify.rs` and
`tests/runbook_fixture/mod.rs` outside the hand-enumerated design-target
selectors. The existing conformance read identified both paths, their selector
class mismatch, their phase provenance, and the exact repair. A reconstructed
change row made them more conspicuous only after discovery; it neither found
their need earlier nor explained the drift more precisely.

## Supported disposition

This evidence falsifies RFC-027 H3 in its strong form: the proposed authored
five-field row did not demonstrate a distinct normative fact or enough consumer
benefit to justify a new owner and its maintenance cost. It does not support
schema work or RFC-027 Stage 2.

The useful residual is a derived read over existing owners. The concrete
plan-time case is already captured by ISS-251, while IMP-310 owns the missing
selector-conformance technical specification. Neither requires a semantic
change-claim record.

This evidence does **not** dispute RFC-027's broader semantic-continuity thesis,
criterion evolution, or worker discovery. Those hypotheses have different
facts and consumers.

## Limits

- The trial was retrospective and sampled only SL-224 and SL-233.
- It cannot measure contemporaneous authoring time or a future reader's elapsed
  time.
- Neither sample supplied a clean governing-claim withdrawal, so the proposed
  `satisfies` field was not tested against a genuine downstream withdrawal
  sweep.
- SL-233 was still reconciling when sampled; its conformance report was evidence
  of a declaration gap, not closure evidence.

A future attempt to reopen H3 would therefore need a distinct fact not already
owned by selectors, plans, criteria, entity relations, coverage, or Git, plus a
blind plan-time trial against a named consumer. Reconstructing the same five
fields post hoc is not such evidence.

## Sources

- RFC-027 — Stage 0 shape, consumers, and falsifier.
- SL-224 — completed selector/conformance control.
- SL-233 — PHASE-16 declaration gap and criterion-evolution material.
- Git: `e414a7ef2`, `ceba47cd8`, `80a1eba6d`, `f8a1a0487`.
- ISS-251 — plan-named paths omitted from design-target selectors.
- IMP-310 — selector-conformance specification coverage.
