# Review RV-311 — design of SL-231

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External adversarial re-review after RV-310 and REV-036. The pass re-ran the
three original blocker attacks, then probed worker-mode enforcement,
source-to-field enrichment composition, correction/read edge states,
cross-spec authority, atomic-publication verification, and deferred-view scope.

## Synthesis

The original blockers remained closed. Eight findings were integrated directly:
the worker guard is now a named enforcement seam; enrichment is a complete
source-to-field mapping; resolved-chain, retracted-terminus, correction-recovery,
temporary cleanup, constructional publication verification, SPEC-013 exception,
and deferred `by-month` boundaries are explicit.

F-5's request to defer the measurement admission seam was not adopted because
it would reopen DEC-048. The design instead distinguishes the closed
registered-source admission check from the harness extraction adapter, which
remains owned by QUE-176 and the first instrumentation slice. REV-037 carried
the observation contract changes into SPEC-013, SPEC-028, REQ-405, REQ-406,
REQ-407, REQ-408, REQ-409, REQ-410, and REQ-413. REV-038 aligned SPEC-013's
active member requirements with the same exception.
