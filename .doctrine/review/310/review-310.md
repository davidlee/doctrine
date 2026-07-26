# Review RV-310 — design of SL-231

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

External adversarial review supplied by an Opus reviewer after SL-231's first
design and PRD-018/SPEC-028 activation. The pass attacks identity and replay
under concurrency, correction fail-open semantics, worker confinement,
filesystem guarantees, trustworthy measurement provenance, CLI contract
placement, enrichment privacy, storage disposition, pagination and search
semantics, hostile content, wire vocabulary, and ADR-001 classification.

The integration must explicitly disposition every supplied finding, preserve
the accepted collection-first/non-aggregation boundary, and reconcile both the
slice design and the active evergreen technical contract before Plan.
