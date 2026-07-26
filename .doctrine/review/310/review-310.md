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

## Synthesis

All thirteen findings are terminal. The three blockers changed the design's
foundations: identity now routes directly to one UUID-derived path, correction
validity is scoped per control, and dogfood activation follows the caller's
actual capture capability. The remaining findings tightened filesystem
publication, measurement trust, enrichment privacy, storage disposition,
read/search semantics, hostile-content limits, wire vocabulary, and layering.

F-6 was correct that the original interaction with SPEC-013 was unresolved.
Its stronger implication—that observations must join the entity
`CommonListArgs` spine—does not follow from SPEC-013's entity-kind scope.
The reconciled contract instead declares the non-entity UUID exception, reuses
shared grammar/rendering/tokenization, and owns observation-specific keyset and
golden coverage.

REV-036 carries the corresponding evergreen change across SPEC-028 and
REQ-405–REQ-413. PRD-018's product intent and the collection-first,
non-aggregation boundary remain unchanged.

Standing follow-ups are explicit: IMP-319 for subprocess-worker capture,
IMP-320 for conditional boot guidance, IDE-005 for named environment-based
harness detection, and QUE-176 for trustworthy measurement adapters, with
`claude -p` the first candidate.

## Harvest

- Produced: RV-310; DEC-044 through DEC-052; REV-036; reconciled SL-231 scope,
  design, and SPEC-028 contract.
- Learned: UUID identity, correction fail-open semantics, and capability-aware
  capture must be fixed before storage/navigation convenience or reporting.
- Open: IMP-319, IMP-320, IDE-005, QUE-176.
