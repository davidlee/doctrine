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

## Round-3 outcome (2026-06-10)

NOT a diminishing round: 15 combined external findings (web + GPT-5.5 + Opus,
user-deduped) → F30–F44, all accepted (2 partial, 1 alternative fix), 0
rejected, none rehash; plus self-found F45 (the F11 pattern again — found
re-deriving F34's machinery). Four blockers in two interaction families:

- **F30** — pass-1 arity eviction broke authored Reject cycles before pass-2
  detection saw them (diagnostic silently lost). Fix: Reject detection on the
  authored pre-arity set; authored SCC = the one cycle concept.
- **F31/F32/F33** — Degraded/taint mis-scoped three ways: seeded from non-spec
  overlays, exclusion-from-U ambiguous (taint-defeating under the natural
  reading), suffix-by-NodeId violated surviving U edges (I2 literally false).
  Fix: spec-scoped seeds, intra-SCC-only exclusion, `Degraded(u32)` carrying
  U-level so the suffix respects surviving edges.

Lesson: both families are *interaction* bugs between individually-sound parts
(pass pipeline; degradation × ordering) — per-section review missed them three
times. Known-open list updated: **taint conservatism partially addressed**
(suffix now edge-respecting, non-spec overlays excluded); residual
conservatism = F30's authored-SCC degradation when arity already broke the
cycle, and full-downstream taint extent — both deliberate, revisit on consumer
complaint. Path-enumeration blowup and API churn remain open, untouched by
round 3 (no reviewer sharpened them).

Round 3 was billed FINAL, but it found 4 blockers — recommendation to user:
one more cheap external pass over the round-3 rewrites (pass 2/3/4 + the
propagation contract) before lock; the blocker trend has not yet hit zero.

## Round-4 outcome (2026-06-10, GPT-5.5 via codex MCP, run in-session)

User authorised running round 4 directly. 3 findings → F46–F48, all accepted:

- **F46** — round 3's F30 "one cycle concept" call reversed with evidence:
  authored-SCC keying of pass-3 exclusion/pass-4 taint destroyed surviving
  valid resolved edges when arity had already broken the cycle. Now: authored
  SCC → diagnostic only; post-arity SCC → order degradation. (This was the
  alternative I weighed and rejected for simplicity at F30 integration — the
  residual-conservatism note from round 3 is RESOLVED, not residual.)
- **F47** — explain()'s "chains to root" was impossible on cyclic Reject views
  (no root). Chains now end at roots or degraded-SCC entry; SCC members are
  endpoints only; in-SCC nodes get [[n]].
- **F48** — I1 wording over-claimed traversal-view acyclicity; tightened.

Trend: blockers 2 (round 2+self) → 4 (round 3) → 1 (round 4), and round 4's
blocker was a choice-cost, not a machinery bug. Remaining known-open: path
enumeration blowup (F47 bounds termination, NOT combinatorics — predecessor
sub-DAG still the fix direction), full-downstream taint extent, pre-consumer
API churn. Assessment: diminishing reached; findable-by-review surface looks
exhausted — remaining unknowns belong to the first consumer. Recommend lock.
