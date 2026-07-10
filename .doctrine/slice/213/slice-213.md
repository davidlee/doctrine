# Comparison constraint layer

## Context

RFC-019 Phase B (externally reviewed, RV-260 all-verified). Phase A (SL-210,
done) shipped the capture side: typed, lossless, append-only pairwise
judgements in merge-clean session files under `.doctrine/comparisons/`, with a
`compare` verb group and admissibility at capture. Nothing yet *consumes* the
ledger. REV-022 (ADR-015 value-provenance amendment) is already applied —
the governing policy (authored wins; projection fills absence;
`DEFAULT_VALUE = 1.0` fills the rest; anchor conflicts surface as
contradictions) is settled governance, not a question for this slice.

This slice builds the inference side: row-validity resolution, pure constraint
propagation to per-item value bounds, contradiction surfacing, and a
deterministic point projection feeding `value_dim` — making the Phase A
evidence actually move priority.

## Scope & Objectives

Strictly additive, pure over `(ledger, authored facets, entity statuses,
config)`; disk stays at the scan seam.

- **Row-validity resolution** (RFC-019 T6) — from raw parsed rows to the
  active-row set, deterministic across merged session files:
  - Supersession: later row wins per (pair, frame, rater); recency by the
    total ordering key `(date, session_uid, row_seq)` (RV-260 F-4).
    Cherry-pick duplicates collapse by row uid.
  - Tombstones: withdrawal rows (shipped in Phase A) evict their target by
    uid.
  - Lifecycle event-effect table: terminal item → rows inert for elicitation,
    active for inference; superseded item → rows evicted with reprobe hint,
    no silent transfer; decomposed item → parent rows stay parent-scoped,
    children inherit nothing (A3).
- **Constraint propagation** (RFC-019 T1) — active rows + authored anchors →
  per-item feasible bounds `(lower, upper)`:
  - Order frames as strict inequalities; equal/band answers as
    equalities/bands; authored `value` magnitudes as point anchors that
    propagate (`A > B`, `B = 3.2` ⇒ `v_A ∈ (3.2, ∞)`).
  - Lossless: nothing inferrable discarded; unrecognised frames captured but
    inert.
  - Contradictions **detected and surfaced**, never smoothed: strict
    preference cycles and anchor conflicts diagnosed like needs cycles
    (`src/priority/findings.rs` posture); residual contradictions mark the
    pair for reprobe. Inference terminates regardless.
  - Anchor lifecycle (RV-260 F-8): bounds are a pure function of *current*
    anchors + active rows — no snapshotting; `value clear` legitimately
    relaxes derived bounds.
  - Sign discipline (RV-260 F-1): order inequalities sign-agnostic; ratio
    rows admissible only where both sides' regions are provably positive,
    otherwise inert-with-reason (capture-side refusal remains Phase C's
    elicitation concern; inference just refuses to consume unsound rows).
- **Point projection** (RFC-019 T1) — one deterministic scalar within each
  item's bounds for `value_dim`:
  - Candidate algorithms implementation-owned (isotonic spacing generalises
    across signs; BT-style fit only within provably-positive components).
  - Must reproduce `DEFAULT_VALUE = 1.0` for no-evidence items (REV-022).
  - Resolution per amended ADR-015: authored > projected > default.
- **Wiring** — a pre-pass beside the existing base pre-pass
  (`src/priority/graph.rs` step 2c): scan reads the ledger once (impure
  seam), inference pure, projected magnitudes flow into `value_dim`.
- **`explain` surface** — value-source line with provenance and bounds:
  `value: 6.2 (projected, bounds (3.2, 9.1), 9 judgements)` vs
  `value: 8.0 (authored)`; contradiction findings rendered like needs-cycle
  diagnoses.

## Non-Goals

- Elicitation queue, pair selection, determinacy-driven question yield,
  binary insertion — Phase C. (The determinacy *predicate* over bounds ×
  `est_cost` is Phase C's opening move; ships there unless propagation tests
  need it earlier — design decides.)
- Tension narrative in `next`, session surfaces — Phase D.
- Estimate/risk domain constraint semantics — sibling extension after C.
- Budget water-line / cut report — Phase E.
- Any capture-side change (schema, verbs, admissibility) — Phase A is done;
  schema version bump only if design finds a defect.
- REV-022 — already applied; no governance edit rides this slice.
- `value set` warn-on-non-value-bearing (REV-022 Q1) — rides an RFC-019
  phase, but not necessarily this one; design may pull it in if trivially
  adjacent.

## Affected surface

- `src/comparison.rs` — extend: row-validity resolution, active-row set
  (pure).
- New engine module for constraint propagation + projection (exact home at
  design; ADR-001 layering — pure core, no disk/clock).
- `src/priority/` — pre-pass wiring (`graph.rs`), findings
  (`findings.rs`), explain/render surface (`render.rs`, `view.rs`,
  `surface.rs`).
- `src/commands/compare.rs` — possibly: evidence listing gains
  active/superseded/withdrawn annotations (design decides).
- Tests: determinism suite mirroring the existing no-NaN/total-order suite;
  propagation tests (chains, anchor conflicts, preference cycles, recency
  supersession, tombstone eviction, lifecycle effects).

## Risks, assumptions, open questions

- **OQ-B1 (design)**: projection algorithm choice (isotonic vs constrained
  BT within positive components) and its determinism/stability guarantees
  under evidence growth — small evidence delta should not wildly reorder
  projections.
- **OQ-B2 (design)**: where the lifecycle event-effect table gets its entity
  statuses — the priority scan already loads statuses; confirm the pure
  seam's input shape.
- **OQ-B3 (design)**: does `compare list` grow resolution annotations here,
  or stay raw until Phase C's queue needs them?
- **OQ-B4 (design)**: band width semantics for "roughly equal" answers —
  exact equality vs ε-band; RFC-019 leaves this implementation-owned.
- **Assumption**: Phase A schema is sufficient — full typed row landed day
  one; no migration expected (schema version field exists if wrong).
- **Risk (medium)**: contradiction diagnosis quality — cycle detection over
  inequality graphs must stay terminating and give actionable findings, not
  a wall of pairs. Mitigation: mirror the needs-cycle diagnosis machinery.
- **Risk (low)**: score perturbation for corpora with existing ledgers —
  none exist outside this repo yet; behaviour-preservation gate applies to
  no-ledger corpora (priority suites unchanged when `.doctrine/comparisons/`
  is absent or empty).

## Verification / closure intent

- **Behaviour preservation**: with no ledger (absent/empty directory), every
  existing priority suite passes unchanged.
- **Propagation**: chains (`A > B > C` with one anchor bounds all three),
  anchor conflicts surfaced, preference cycles diagnosed + terminating,
  recency supersession per `(date, session_uid, row_seq)`, tombstone
  eviction, lifecycle effects (terminal/superseded/decomposed).
- **Determinism**: same merged file set ⇒ same active rows, bounds,
  projections on any replica; no clock/rng in the pure layer; no-NaN /
  total-order invariants extended to projected values.
- **Resolution policy**: authored wins; projected fills; `DEFAULT_VALUE`
  fallback exact; authored-vs-bounds conflict is a finding, not a silent
  win.
- **`explain`**: value-source line renders provenance + bounds for all three
  sources.

## Summary

Makes Phase A's evidence load-bearing: deterministic active-row resolution,
lossless bounds propagation with surfaced contradictions, and a projected
scalar feeding `value_dim` under the ADR-015 provenance policy — the
inference substrate Phase C's elicitation queue reads.

## Follow-Ups

- Phase C slice: elicitation queue + capture loop (determinacy predicate,
  guaranteed-yield selection).
- Phase D: tension narrative in `next`/`explain`.
- Estimate-domain sibling batch after C; Phase E after that.
