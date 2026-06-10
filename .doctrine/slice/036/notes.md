# Notes — SL-036 cordage graph core

## Design-stage assessment after adversarial round 2 (2026-06-10, commit c726044)

Verdict given to user: foundation sound, fit for purpose; residual risk is not
correctness but the unused API. Three open concerns flagged (observations, not
integrated findings — candidates for round 3 or the adapter slice):

- **Explanation path enumeration blowup.** `Explanation.paths` enumerates every
  chain to root (`Vec<Vec<NodeId>>`); a diamond lattice has exponentially many
  paths in depth — bites even at ~50 nodes. Neither external reviewer landed it
  (GPT m36 adjacent). Fix direction: return the predecessor sub-DAG (or direct
  + one canonical chain); policy enumerates on demand.
- **Degraded taint is maximally conservative.** One dep cycle near the root
  degrades everything downstream in `U`. Correct under REQ-076 but blunt; the
  gentler still-not-false alternative is condensation ordering (SCC members tie
  at equal level, NodeId-broken). Core-internal change, no interface impact —
  defer until a consumer complains, but expect it.
- **Pre-consumer API churn (risk R2 realised).** Validating consumer is a
  fixture suite. Opaque-handle-heavy API (OverlayId/OrderSpec/ChannelSpec/seed
  maps) means semantically-wrong-but-valid wiring compiles. Expect a
  usage-driven interface rev when adapter/policy slices land; cheap while
  workspace-internal. Recommendation given: lock after one diminishing round —
  remaining unknowns are findable by the first consumer, not by more review.

Performance explicitly assessed a non-concern at H1/H2 scale (worst cases
microseconds at hundreds of nodes); do not spend review budget there.

## Round-2 process facts

- 55 external findings (GPT-5.5: 41, Opus: 14) deduped to F10–F29 (20
  integrated, 3 rejected with reasons) — full source map in design.md §10.
- F11 (per-layer lexicographic order_key unsound: level equality ≠
  incomparability) was self-found during integration — neither reviewer caught
  it. Lesson: integrating a fix is itself a review pass on adjacent machinery.
- F19 confirmed by checking SPEC-001 D4/D5 directly: design pass 1 kept the
  (rank,age)-MIN parent while spec eviction removes the min (weakest) — i.e.
  round 1 kept the weakest. Cross-checking the parent spec caught what both
  externals only smelled.
- Upstream wording note parked in design.md §6: SPEC-001 D9/D10 "seq *rank*
  within a dep-eligible set" should read seq-*topology*; rank is eviction
  strength only. Post-lock SPEC revision, same channel as T1.
