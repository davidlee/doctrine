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
- **Cost projection.** Tier-3 projection for the estimate domain: derived
  cost + diagnostics, degrading deterministically, feeding `est_cost` where
  authored estimates are absent. Resolution ladder (design D2, REV against
  ADR-015): `authored (operator pin) > projected (non-Gauge) > bare anchor` —
  source precedence only; gauge tier renders, never divides.
- **Elicit frames for comparative sizing.** Team-facing frame (`more-work`)
  in the closed frame vocabulary, wired through `compare record`
  admissibility and the elicit render; `sizing-probe` candidate kind in the
  existing queue (existence-admitted, no yield claim — design §4). Records
  admissible per A2 (capture + anchor mass; not probe subjects — records
  are not frontier members).
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

- **R1** *(resolved at design, D1)* — dissolved: the est-domain latent is the
  operative scalar cost, not the range; point-anchor machinery applies.
  Estimate *uncertainty* is deliberately out (Phase E feasible-region model).
- **R2** *(bounded at design, §6.6)* — behaviour preservation is a hard VT:
  zero est-domain rows ⇒ bitwise-identical scoring; the feed only ever adds
  cost sources for evidenced items.
- **A1** — one ledger, one schema (v2); domain is carried per row by frame,
  no parallel store. *(Confirmed at design.)*
- **OQ-1** *(resolved at design, §2)* — two independent per-domain systems
  (`DomainSystem`), shared resolution pass.
- **OQ-2** *(resolved at design, Q3=C, §4)* — `sizing-probe` candidate kind
  in the existing queue, existence-admitted; engine yield-ranking of estimate
  questions stays Phase-E-gated (SL-217 D17).

## Verification / closure intent

- Unit: est-domain compile/project semantics (ordering, equality, cost
  anchors, contradiction surfacing, deterministic degradation) — design §6.
- Scoring-feed ladder goldens incl. regime-flip and INV-2 restatement pins;
  fed-costs > 0 property (design §6.5).
- Behaviour preservation: existing suites green unchanged; corpora without
  est-domain rows score bitwise-identically (design §6.6).
- E2E: capture → compile → project → feed → visible score shift; probe
  round-trip (design §6.10).
- VA: RFC-019 posture holds (capture-everything, infer-only-what-is-sound).
- Governance: REV against ADR-015 approved before the scoring-feed phase.

## Summary

## Follow-Ups
