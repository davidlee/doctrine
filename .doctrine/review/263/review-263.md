# Review RV-263 — design of SL-211

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

- Candidate-active branch condition: does `record-integration` preserve SPEC-022 / `plan_candidate_trunk_row`'s `close_target`-admission contract, or does it reopen a raw-`review/<N>` fallback that the shipped integrate path intentionally forbids?
- Earned-surface truthfulness: does the proposed `review/<N>` / admitted-`close_target` selector match the actual landed lineage shapes, and where does it fail closed versus silently wave through dropped repairs?
- Journal row semantics: confirm `expected_old = planned = applied = reviewed_oid`, `status = Verified` is replay-safe under `advance_row` and does not mutate refs on stray later `--integrate`.
- Close-gate compatibility: verify unchanged `trunk_integration` accepts the proposed ancestor row and fail-closes on ambiguous / empty-planned / mismatched-target shapes.
