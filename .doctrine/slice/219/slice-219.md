# Estimate comparison domain

## Context

RFC-019 estimate-domain batch — the sibling extension riding the SL-210
ledger and the SL-213/SL-217 machinery, sequenced after Phase C proved the
value path (RFC-019 § Implementation path). Originates from IMP-286.

The value domain is fully wired: capture (SL-210, schema v2), constraint
compilation + projection (SL-213), and the elicitation queue (SL-217). The
estimate domain today is capture-only at best: "which is more work?" rows
(`c_A > c_B`) are admissible frames in the ledger vocabulary but the
inference tiers leave them inert — captured losslessly, never compiled
(RFC-019 § frames). Nothing constrains, projects, or elicits comparative
sizing evidence.

Domain boundary is settled governance (RFC-019 OQ-4, A2): the estimate
domain **includes records** — settle-cost is intrinsic even where authored
value is not. Admissibility is domain commensurability, not a kind gate.

## Scope & Objectives

Strictly additive; same purity posture as SL-213/217 — pure over
`(ledger, authored facets, statuses, config)`, disk stays at the scan seam.

- **Est-domain constraint semantics.** Compile `est_cost`-domain rows
  (`c_A > c_B` ordering; equality merge; anchors where a row carries a
  magnitude) into a constraint set over costs. The estimate facet is a
  *range* `[lower, upper]` with skew β (ADR-015 §1), not a scalar — the
  semantics of an ordering row over ranges, and how authored ranges act as
  bounds/anchors, is the core design work.
- **Range projection.** Tier-3 projection for the estimate domain: derived
  cost + diagnostics, degrading deterministically, feeding `est_cost` where
  authored estimates are absent (mirror of the value-domain resolution
  policy — authored wins; projection fills absence; engine default fills the
  rest).
- **Elicit frames for comparative sizing.** Team-facing frames ("which is
  more work?") in the closed frame vocabulary, wired through `compare
  record` admissibility and the elicit render; records included in the
  candidate pool per A2.
- **Coupling honesty.** `prefer-first` compiles to the weighted inequality
  `v_A·c_B > v_B·c_A` over *current costs* (Phase B obligation) — est-domain
  inference that moves projected costs must not silently invalidate
  value-domain compilations; the interaction is a design decision, not an
  accident.

## Non-Goals

- **Cross-domain yield ranking** — Phase E (IMP-287). Estimate questions
  stay curator-surfaced (SL-217 D17) until the feasible-region entry
  criterion is met.
- **Estimate feasible-region model (RV-260 F-5)** — what an answer
  constrains over `(lower, upper)`, hypothetical answer space for a range
  refinement. Entry criterion for Phase E, not this slice; design here
  should avoid foreclosing it.
- **Ratio/band row vocabulary** — voids the D8 marginal-exactness lemma;
  the phase admitting them revisits `determined` (LP or richer propagation).
- **Scoping decision context / budget cut** — Phase E.

## Affected surface

- `src/comparison/**` — compile/project/query/wire: domain-aware
  compilation, est-domain projection.
- `src/priority/elicit.rs`, `src/priority/**` — frames, render, config.
- `src/commands/compare.rs` — capture admissibility for est-domain frames.
- `src/estimate.rs`, `src/estimate/**` — range/skew model touchpoints.
- `tests/e2e_compare_elicit.rs` — end-to-end coverage.

## Risks, assumptions, open questions

- **R1** — range-valued facet breaks the scalar assumptions of the SL-213
  interval machinery; naive reuse may be unsound (D-bounds warning: joint
  set, not marginal boxes).
- **R2** — projection feeding `est_cost` perturbs `value_dim` for *every*
  scored item; behaviour-preservation for corpora with no est-domain rows
  is a hard gate.
- **A1** — one ledger, one schema (v2); domain is carried per row by frame,
  no parallel store.
- **OQ-1** — separate `ConstraintSet` per domain vs a domain-tagged unified
  set (design decision).
- **OQ-2** — do est-domain candidates enter the SL-217 queue (value-only per
  Q4) or remain a distinct curator surface until Phase E?

## Verification / closure intent

- Unit: est-domain compile/project semantics (ordering, equality,
  anchors/ranges, contradiction surfacing, deterministic degradation).
- Behaviour preservation: existing suites green unchanged; corpora without
  est-domain rows score identically.
- E2E: capture → compile → project → elicit round-trip for a sizing frame.
- VA: RFC-019 posture holds (capture-everything, infer-only-what-is-sound).

## Summary

## Follow-Ups
