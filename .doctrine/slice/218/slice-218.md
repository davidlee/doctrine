# Tension narrative

## Context

RFC-019 Phase D. Phases A (SL-210, capture), B (SL-213, inference), and C
(SL-217, elicitation queue) are shipped. ADR-015 composes engineering's
seq/dep structure with stakeholder value order — structure wins for
`after`/`needs`, value fills the silence — but where surviving structure
overrides the value order, the override is silent today. Phase D adds the
*surfacing*: `next`/`explain` render the value-rank vs delivery-rank
disagreement ("SL-x outranks SL-y on value; SL-y is cheaper and unblocking —
surfaces first") rather than resolving it invisibly.

Fulfils IMP-285 (partially — see Non-Goals).

**Pre-D obligation rides in this slice.** SL-217's Deferred list names D7
(rank-aware quarantine / agent-testimony demotion knob) a pre-Phase-D
obligation: `confirm_boost` (D13) is selection-order bias only, with no
determinacy semantics, so today fourteen cheap agent comparisons close the
same questions as fourteen stakeholder decisions (RFC-019 T7;
product-critique tension #3). The knob — agent rows inert-until-confirmed, or
excluded from *determinacy* so agent evidence proposes orderings but never
retires a question — is mandatory before any stakeholder-facing surface
treats the ordering as elicited truth.

## Scope & Objectives

1. **D7 demotion knob.** A config knob demoting `rater = agent` ledger rows
   at the determinacy level (shape per design: inert-until-confirmed vs
   excluded-from-determinacy vs quarantine-on-rank). Disclosed in renders;
   distinct from D13's `confirm_boost` selection bias, which keeps its
   no-determinacy-semantics contract.
2. **Tension detection.** A pure function over (value order, delivery order)
   in the priority layer that identifies pairs where surviving seq/dep
   structure overrides the `value_dim` order. Sketch: detection lives in
   `src/priority/`, consuming the same projections `next`/`explain` already
   assemble; no new scan pass.
3. **Tension narrative render.** `next`/`explain` callouts naming the
   overridden pair and the structural reason. Wording must respect SL-217
   D5 (claims are `value_dim`-order claims, never full-score-order) and D15
   (internal-order stability phrasing, never prefix-membership), and frame
   magnitudes honestly (product-critique tension #6: cardinal spacing is
   manufactured from ordinal evidence).

## Non-Goals

- **Session mode + read-write web elicitation surface** — RFC-019 OQ-1
  defers these until Phase C's interaction is proven in use (RFC-002
  demotion discipline). They remain the open tail of IMP-285; this slice
  does not close the backlog item alone.
- Silent re-resolution of tensions — the point is rendering, not a new
  ordering policy. ADR-015 composition semantics are unchanged.
- Challenger-fringe / prefix-membership stability (SL-217 Deferred, blocked
  on the D5 full-score determinacy cut).
- True human-confirmation candidate kind (dependency tracing of determinacy
  on agent-only edges) — post-D7, RFC territory.

## Open questions (design to settle)

- OQ-1: D7 knob shape — demote at determinacy level (which variant:
  inert-until-confirmed / excluded-from-determinacy / quarantine-on-rank)
  vs render-level disclosure only. Does the knob gate stakeholder surfaces
  or ship as plain config?
- OQ-2: D7 placement — PHASE-01 entrance for the narrative phases (current
  assumption), or split into its own preceding slice if design shows the
  determinacy-semantics change sprawls.
- OQ-3: Tension detection seam — exact boundary between the pure detection
  fn (priority layer) and the render seam in `next`/`explain`.

## Risks & Assumptions

- D7 touches determinacy semantics (`src/comparison/`) — the D8
  marginal-exactness lemma and SL-213/217 determinacy goldens must stay
  green unchanged for the knob-off path (behaviour-preservation gate).
- Assumes tension detection needs no new evidence source: value order and
  delivery order are both already computed on the `next`/`explain` paths.

## Verification / closure intent

- VT: pure detection fn unit-tested over constructed (value order, delivery
  order) fixtures, incl. no-tension and multi-tension cases.
- VT: e2e goldens for `next`/`explain` tension callouts; wording goldens
  pinning D5/D15-compliant phrasing.
- VT: D7 knob determinacy tests — agent-only evidence with knob on does not
  retire questions; knob off preserves existing goldens byte-identical.
- VA: render wording reviewed against product-critique tensions #3/#6.

## Summary

## Follow-Ups
