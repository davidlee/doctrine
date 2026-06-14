# Review RV-028 — reconciliation of SL-065

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-065 (product_level axis + symmetric same-subtype
parent + subtype-blind acyclicity) against design.md, plan.toml and ADR-004.
Lines of attack across the three phases:

- PHASE-01 — FR-005/FR-006 authored on PRD-002; REQ-082/083 (tech) untouched.
- PHASE-02 — `ProductLevel` mirrors `C4Level`; `Spec.product_level` optional;
  `spec show` branches by subtype, tech render byte-identical (behaviour gate).
- PHASE-03 — `parent_findings` one symmetric same-subtype rule; `self_parent` +
  `parent_cycle` subtype-blind by deletion; `descends_from` untouched (tech-only).

Invariants held: ADR-004 outbound-only (no reciprocal children stored/rendered);
storage rule (ephemeral child→parent map never persisted); behaviour-preservation
(tech suites green unchanged); defer-before-close (design §8 follow-ups land in
backlog).

## Synthesis

**Closure story.** SL-065 reconciles cleanly. The three phases implement the
locked design without drift: the symmetric parent rule (`registry.rs::
parent_findings`) collapses the three tech-only special-cases into one
same-subtype rule with `on_product` as the subtype selector (EX-1); acyclicity is
subtype-blind by net deletion (EX-2); `descends_from` stays tech-only (EX-3,
`sweep_descent_on_product_subject`). Render branches by subtype with tech output
byte-identical and product gaining `product level:`/`parent:` lines (VT-2). The
PHASE-03 invariants are pinned at the **write seam** (`spec validate` CLI:
`sweep_parent_product_*`), per the invariant-test-drives-the-write-seam rule, with
unit-level siblings on the pure helpers. Live `spec validate` is corpus-clean and
`just check` is green.

**Findings.** Four, all terminal, none a blocker — the close-gate does not bite.

- F-1/F-2/F-3 (minor/minor/nit) — **fix-now**, reconciled in this unit:
  `clean()` now declares `product_specs={PRD-001}` (kills the false-positive
  footgun for future parent-edge baselines); the `parent_cycle` comment now
  matches design §4 precision (a cross-family ring *does* report — it cannot forge
  a *spurious additional* cycle); a new test `parent_cycle_mixed_family_ring_is_
  still_reported` pins the subtype-blind ring invariant against future dedup
  tightening.
- F-4 (minor) — **follow-up**: design §8 OQ-1/OQ-2 deferrals captured as IMP-069
  (level-adjacency validate tightening) and IMP-070 (descends_from→capability-level
  PRD constraint), so they survive slice close.

**Standing risks / accepted tradeoffs.** Level fields remain advisory tags with no
invalid-kind validation (D5) and no level-adjacency constraint (OQ-1, now IMP-069)
— a conscious v1 deferral. `descends_from` stays unconstrained on product level
(OQ-2, now IMP-070). Mixed-family parent rings are a non-realistic case (chains are
within-family) but the reporting invariant is now pinned. No design defect found;
nothing tolerated without rationale.
