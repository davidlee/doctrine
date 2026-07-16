# SL-219 implementation notes

## Dispatch drive, claude arm — 2026-07-16, PHASE-01..03

Funnel: base-clean prove → marker-clear + regression capture at B →
arm-spawn → worker (self-commit via `worker_commit`) → verify-worker →
`dispatch_import` → tree-sync → regression diff → `dispatch_conclude_phase` →
`dispatch_reap`. Regression baseline was 0 failures at every base; diff clean
at every S. Boundary rows landed per phase (registry via conclude).

### PHASE-01 — wire vocabulary & admissibility (import 5e763a49, boundary 5dcb443b)

- `DOMAIN_ESTIMATE`, `FRAME_MORE_WORK`, frame-table row, `admissible_estimate_pair`
  derived from `kinds::VALUE_BEARING ∪ kinds::RECORD` (D3; RSK admitted).
  Capture routes admissibility by frame-derived domain. resolve.rs: tests +
  docs only (R4 confirmed priority-scoped).
- Worker surfaced **CHR-044**: ~31–36 e2e write-golden binaries red under the
  worker marker (`worker fork (signal: marker): refusing authored write`) —
  environmental, skip helper keys on `DOCTRINE_WORKER` env the claude arm never
  sets. Pre-declare in every worker prompt (saves diagnosis cost — proven in
  PHASE-02/03).
- Worker also flagged the pre-split leak: store.rs fed ALL Active rows into one
  compile (an Active est row would mint a value constraint) — fixed by design
  in PHASE-02; PHASE-02 prompt carried it explicitly.

### PHASE-02 — per-domain pipeline split & est compile (import 6c296a45, boundary d862c2d3)

- `Pipeline` → `DomainSystem { constraint_set, anchors, projection }` ×
  value/estimate; shared resolution; compile-input selection by row domain.
  `authored_est_cost` extracted (one formula site); row-gated est anchor
  builder `comparison_est_anchor_map` in graph.rs; `ComparisonDomain` finding
  discriminator set at construction (D9); est `AnchorConflict` D1 wording.
  compile.rs / query.rs untouched (VA-1 verified by orchestrator diff).
- **Accepted deviations** (documented in code): `constraining_by_class` and
  `active_judgements` stay top-level on `Pipeline`, value-domain scoped —
  design pins DomainSystem to exactly three fields; both existing consumers
  are value-system (est leak into elicit/human-only recompile would reproduce
  the PHASE-01 defect).

### PHASE-03 — est projection & cost feed (import cdce07fa, boundary cf740a47)

- `ProjectionCfg` conveys `gauge_step` + `gauge_center` (D8); value site =
  `VALUE_PROJECTION_PARAMS`; existing goldens byte-identical. Est projects
  with `(EST_GAUGE_STEP, gauge_center = ctx.absent)`; `bare_cost_anchor`
  helper extracted in graph.rs (shared by CostCtx fold + est center, D7).
  `cost_feed` = est projection minus Gauge tier (D2 table), positivity
  property (D11). `EST_GAUGE_STEP` 0.25 + `[priority.estimate] gauge_step`
  parse/clamp.
- **Accepted deviations**: (1) `VALUE_PROJECTION_PARAMS` threads the shipped
  `[priority.gauge] step` config knob via functional update — passing the
  const verbatim would have silently killed a tested knob; goldens
  byte-identical either way. (2) `cost_feed` carries
  `#[cfg_attr(not(test), expect(dead_code))]` until PHASE-04 wires the ladder
  (catalog/graph.rs:84 idiom). (3) `pipeline_from_sessions`/`load_pipeline`
  grew `value_cfg`/`est_cfg` params — store cannot compute `ctx.absent`
  (needs ScannedEntity + status_class, above comparison per ADR-001);
  graph.rs threads finished params.

#### PHASE-04 — REV & scoring feed (import fccbc80e, boundary 4f847832)

- EN-2/VH-1 gate CLEARED first: REV-023 (ADR-015 estimate provenance
  amendment) drafted per design §3's five numbered items, USER-APPROVED,
  applied to ADR-015, status done — edge commits 4233c61e (draft) /
  4a12e576 (apply); `doctrine boot --check` clean (EX-3, orchestrator
  authored writes in the primary tree).
- Ladder live at the single seam: `graph::est_cost` authored →
  `cost_feed` (EPSILON-floored) → bare anchor. Feed keyed by
  `EntityKey::canonical()`; ladder took key+feed params rather than
  widening `CostCtx` (Copy; a map cannot ride it). `build_from` derives
  the feed from ONE pipeline load; `load_comparison_projection` removed
  (subsumed by `load_comparison_pipeline`). `cost_feed` dead_code attr
  dropped; `CostFeed` alias added.
- 8 VT-1 ladder goldens (graph.rs) incl. regime-flip and
  projected-above-bare-anchor INV-2 pin; 3 VT-2 tests (elicit.rs). Full
  unit binary 3530 passed / 0 failed; existing suites green unchanged
  (EX-2). Regression diff clean at S.
- **Accepted deviations**: findings.rs test call-sites touched
  (mechanical, forced by the pinned `build_from_with_cfg` signature
  growth); `surface.rs::beta_endpoints` gains a shared cost_feed param so
  next/explain/findings derive the feed once per pipeline.
- VA-1 spec-coherence note for /close: ADR-015 is now the only governance
  naming `est_cost` resolution (estimate-source section present
  post-apply); SPEC-020 describes the estimate facet, not scoring.
- CHR-044 rider (worker relay): `doctrine check commit` cannot complete
  locally under the worker marker — the recipe hard-stops at the first
  e2e write-golden binary, hiding the rest of the gate; the server-side
  `worker_commit` gate runs unconfined and passed. A marker-aware skip
  would restore local gate parity.

### PHASE-05 — Sizing probes (import 61e73a75, boundary 2cbd2df3)

- Two-round worker (Opus). Round 1 halted commit-gate-red on ONE
  out-of-scope e2e golden (`tests/e2e_compare_elicit.rs` stall render) —
  invalidated by the INTENDED behaviour (its corpus was sizing-eligible,
  so the queue now mints a probe). Worker correctly refused the
  out-of-scope edit; orchestrator /consult → USER adjudicated **Option B**
  (preserve each golden's original intent; probe-render e2e stays in
  PHASE-06's file). Selector extended: `tests/e2e_compare_elicit.rs`
  design-target (a44daea5) — import initially Refused{undeclared-scope}
  until declared.
- Option B refinement (worker-verified, accepted): stripping the corpus
  estimates would flip it STABLE (they were load-bearing for the
  zero-yield bridge); instead the corpus gains one est-domain
  `incomparable` row — asked-once retires the probe with zero cost
  movement, so the golden genuinely stalls AND pins residual disclosure.
  Tier-2 "no estimated item to calibrate against" wording pinned in
  `render_stable_is_member_scoped` (estimate-free corpus) instead.
- Delta: `CandidateKind::SizingProbe` — pool predicate keyed on
  `ItemCosting.bare_estimate`; gauge exclusion folded into
  `est_evidenced` (gauge members are est-evidenced by construction);
  median target + authored-only fallback tiers; existence admission
  (yield 0, basis calibration, score 0, no yield_by_answer); D15 gating;
  disclosure rides `state_detail` (schema v1). 8 VT-1/VT-2 tests
  (elicit.rs), 5 VT-3 tests (compare.rs). Unit + e2e green; regression
  diff clean at S.

### PHASE-06 — Surfaces & e2e (import fa69eb6b, boundary 2b8955ca)

- Cost-source block in explain: `ReasonKind::Cost{Authored,Projected,
  BareAnchor,Gauge}` + `Explanation.cost_source` in view.rs (undeclared →
  declared design-target 63f5d7ca; additive, exact SL-213 value-source
  precedent); render fragments; est rater split via
  `Pipeline.est_constraining_by_class` (NoConstraint excluded). New e2e
  `tests/e2e_compare_estimate.rs`: score-shift, probe round-trip,
  domain-tagged AnchorConflict + JSON parity. Unit 3548 passed / 0
  failed; regression diff clean at S.
- findings.rs / compare.rs were in the declared set but needed ZERO edits
  — domain-tagged findings and probe/frame render fully landed in
  PHASE-04/05.
- **Accepted deviations** (audit attention): (1) a 4th cost shape
  `authored (via class anchor)` for facet-less members hoisted by an
  `equal` merge — a real design §4 path the "three shapes" enumeration
  missed; exercised by the probe round-trip e2e. (2) cost-source block
  gated on est engagement (est projection non-empty) — mirrors the
  value-source "bare divisor is a floor, not a citable source" posture;
  keeps every pre-SL-219 explain golden byte-identical; the standalone
  bare-anchor shape renders only in est-active corpora.
- **VA-1 sweep seed (worker va1_map)**: §6.1–§6.6, §6.8–§6.10 all map to
  landed tests. §6.7 reprobe is PARTIAL — beta_endpoints_* + β-sweep
  cover the pieces, but no single e2e pins "authored-range/β edit
  re-runs BOTH est projection AND value determinacy" as one round-trip.
  Verify or waive at audit.
- CHR-044 sightings: e2e_priority_cross_kind (2 fails, marker refusing
  `slice new` spawn) — environmental, pre-declared, chore stands.

## Drive complete — conclude cadence

All phases PHASE-01..06 landed with committed boundary rows. Next:
verify-vt → prepare-review → slice status audit → /audit (fresh context).
