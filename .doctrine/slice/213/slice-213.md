# Comparison constraint layer

## Context

RFC-019 Phase B, as revised under external reviews 2 and 3 (2026-07-11;
verdict: accept as written, four design obligations bound to this slice's
design gate). Phase A (SL-210, done) shipped the capture side: typed,
append-only pairwise judgements in merge-clean session files under
`.doctrine/comparisons/`, schema v1, with a `compare` verb group and
admissibility at capture. Nothing yet *consumes* the ledger. REV-022
(ADR-015 value-provenance amendment) is applied — the resolution policy
(authored wins; projection fills absence; `DEFAULT_VALUE = 1.0` fills the
rest; anchor conflicts surface as contradictions) is settled governance.

This slice builds the inference side per the RFC's **three-tier model**:
tier 1 evidence (the ledger — lossless, possibly inconsistent), tier 2
constraint set (derived interpretation with **deterministic degradation**),
tier 3 projection (derived scalar + diagnostics feeding `value_dim`). It also
carries the **schema v2** additive columns the revised RFC assigns to Phase B.

## Scope & Objectives

Strictly additive, pure over `(ledger, authored facets, entity statuses,
config)`; disk stays at the scan seam.

- **Schema v2** (RFC-019 T2, additive; version-gated):
  - `response` generalising `preferred`: `prefer-a | prefer-b | equal |
    incomparable`. Deterministic v1 mapping (`preferred = a` →
    `response = prefer-a`, etc.); `preferred` and `response` never coexist
    in v2 files. `equal` compiles to a tier-2 equality/band; `incomparable`
    compiles to no constraint, recorded as asked (selector fodder).
  - Ratio **magnitude** column beside `form = ratio` (still not elicited —
    OQ-6 stays open; the column makes the form usable when it lands).
  - Explicit `supersedes = <row-uid>`.
  - Capture verb accepts the new responses and `--supersedes`.
- **Row-validity resolution** (RFC-019 T6 revised) — raw parsed rows →
  active-row set, deterministic across merged session files:
  - **Explicit supersession** by `supersedes` uid; **implicit** only within a
    single session file — same identity key `(pair, domain, frame, form,
    lens, rater)`, higher `seq` wins. Cross-session same-key rows are
    concurrent evidence, both active; conflicts flow to tier-2 degradation,
    never a lexicographic winner. `(date, session_uid, row_seq)` survives as
    deterministic iteration/display order only; cherry-pick duplicates
    collapse by uid.
  - Tombstones evict their target by uid.
  - Lifecycle event-effect table: terminal item → rows inert for elicitation,
    active for inference; superseded item → rows evicted with reprobe hint,
    no silent transfer; decomposed item → parent rows stay parent-scoped,
    children inherit nothing (A3).
- **Tier-2 constraint compilation** (RFC-019 T1 revised) — active rows +
  authored anchors → constraint set → per-item bounds:
  - `equal-effort` → strict inequality; `equal` → equality/band; authored
    `value` → point anchor (the only unconditionally hard constraints).
  - **Lens partition** (T5): pooled compilation consumes unlensed rows only;
    lens-tagged rows inert for `value_dim`, reported per-lens.
  - **Deterministic degradation, never infeasibility**: preference cycles →
    SCC tied-group collapse + finding + reprobe marks; anchor conflicts →
    anchors win, a deterministic feasibility-restoring residual set excluded
    + finding. Every exclusion visible in `explain`.
  - Bounds are the *display* projection of the joint feasible set (`explain`
    renders them); the constraint graph is retained for downstream joint-set
    reasoning (Phase C determinacy) — decisions are never computed from the
    interval box.
- **Tier-3 projection** — one deterministic scalar within bounds per item:
  sign-aware (isotonic spacing generalises across signs; BT-style fit only
  inside provably-positive components); anchor-free components gauge on
  `DEFAULT_VALUE = 1.0`; resolution per applied ADR-015 (authored >
  projected > default).
- **Wiring** — a pre-pass beside the existing base pre-pass
  (`src/priority/graph.rs` step 2c): shell loads the ledger once (impure
  seam), inference pure, projected magnitudes flow through the
  `effective_raw_value` seam into `value_dim` (and burndown, consistently).
- **`explain` surface** — value-source line with provenance, bounds, judgement
  counts by rater kind (T7 disclosure), and residual/contradiction
  diagnostics; findings rendered like needs-cycle diagnoses.

### Design obligations (external review 3 — settle at design, before plan)

1. **Degraded-SCC propagation semantics** — collapse must not manufacture
   relations beyond existing rows (A>B>C>A + A>D must not create B–D, C–D).
   Candidates: existential / universal / member-level bounds retained /
   anchor-only propagation through conflicted components.
2. **Residual-selection policy** — deterministic, feasibility-restoring,
   anchors preserved, retained evidence maximised, documented. Complexity
   class chosen deliberately (not minimum-cardinality by accident).
3. **`prefer-first` treatment** — must never compile to `v_A > v_B`. Own
   compiler over costs (`v_A·c_B > v_B·c_A`) vs inert initially vs
   reclassified domain.
4. **v1→v2 compatibility** — implement the RFC's deterministic mapping; v1
   files parse under the v2 reader forever (or via explicit upgrade — design
   decides the mechanism, the semantics are fixed).

Plus the RFC's **formal contract** items: ε vs mathematical strictness; open/
unbounded endpoint representation; projection stability under small evidence
deltas; scale behaviour under anchor changes.

## Non-Goals

- Determinacy predicate, elicitation queue, pair selection — Phase C (gated
  on empirical evaluation of this slice's output against a real ledger).
- `incomparable` sub-vocabulary (why-incomparable) — Phase C-adjacent schema
  refinement, noted in RFC.
- Tension narrative in `next`, session surfaces — Phase D.
- Estimate/risk domain constraint semantics — sibling extension after C.
- Agent-row demotion knob (T7) — policy seam named; mandatory before
  stakeholder surfaces (Phase D+), not built here.
- Ratio-row elicitation — OQ-6 open; v2 carries the magnitude column only.
- Any governance edit — REV-022 already applied.

## Affected surface

- `src/comparison.rs` — schema v2 wire model + row-validity resolution +
  active-row set (pure).
- New engine module for constraint compilation + propagation + projection
  (exact home at design; ADR-001 layering — pure core, no disk/clock).
- `src/priority/` — pre-pass wiring (`graph.rs`), findings (`findings.rs`),
  explain/render (`render.rs`, `view.rs`, `surface.rs`).
- `src/commands/compare.rs` — capture accepts `equal`/`incomparable` +
  `--supersedes`; `list` resolution annotations (design decides depth).
- Tests: determinism suite mirroring the no-NaN/total-order suite;
  propagation, degradation, supersession, lens-partition, compat suites.

## Risks, assumptions, open questions

- **OQ-B1 (design)**: projection algorithm choice and its stability
  guarantee under evidence growth (formal-contract item).
- **OQ-B2 (design)**: `compare list` — how much resolution/degradation
  annotation lands here vs Phase C.
- **OQ-B3 (design)**: `equal` band semantics — exact tie vs ε-band (v2
  band-tolerance column ships only if design picks ε-bands).
- **OQ-B4 (design)**: constraint-graph retention shape for Phase C joint-set
  determinacy — what the pure layer exposes beyond per-item bounds.
- **Assumption**: v2 stays additive — v1 sessions remain parseable; the
  version gate is the migration mechanism.
- **Risk (medium)**: degradation quality — SCC + residual machinery must stay
  terminating and give actionable findings, not a wall of pairs. Mitigation:
  mirror the needs-cycle diagnosis machinery; review-3 obligations 1–2 force
  the semantics to be chosen deliberately.
- **Risk (low)**: score perturbation for no-ledger corpora — behaviour
  preservation gate: priority suites unchanged when `.doctrine/comparisons/`
  is absent or empty.

## Verification / closure intent

- **Behaviour preservation**: with no ledger, every existing priority suite
  passes unchanged.
- **Compat**: v1 golden session files parse under v2; deterministic
  `preferred` → `response` mapping; v2 golden wire shape.
- **Resolution**: explicit supersession; within-session implicit revision
  (identity key includes lens + form); cross-session concurrency (both rows
  active); tombstone eviction; lifecycle effects.
- **Degradation**: cycles collapse per the chosen semantics (never
  infeasible, no manufactured cross-SCC relations per obligation 1); anchor
  conflicts exclude a deterministic residual set; lens-tagged rows never
  enter the pooled component; `prefer-first` never compiles to raw-value
  order.
- **Propagation**: chains (`A > B > C` with one anchor bounds all three),
  anchor propagation both directions, `equal` bands.
- **Determinism**: same merged file set ⇒ same active rows, bounds,
  residuals, projections on any replica; no clock/rng in the pure layer;
  no-NaN / total-order invariants extended to projected values.
- **Resolution policy**: authored > projected > `DEFAULT_VALUE` exact;
  authored-vs-bounds conflict is a finding, not a silent win; anchor-free
  gauge = `DEFAULT_VALUE`.
- **`explain`**: value-source line renders provenance + bounds + rater-kind
  counts + residual diagnostics for all three sources.

## Summary

Makes Phase A's evidence load-bearing under the review-hardened contract:
schema v2 vocabulary, explicit supersession, deterministic active-row
resolution, constraint compilation that degrades deterministically instead of
going infeasible, and a projected scalar feeding `value_dim` under the
applied ADR-015 provenance policy — with the four review-3 obligations
settled at design before any plan.

## Follow-Ups

- Empirical evaluation of this slice's inference on a real ledger over this
  repo's backlog — Phase C's entry criterion.
- Phase C slice: determinacy over the joint set + elicitation queue.
- Phase D: tension narrative; agent-row demotion knob before stakeholder
  surfaces.
- Estimate-domain sibling batch after C; Phase E after that.
