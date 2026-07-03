# Review RV-240 — reconciliation of SL-194

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit of SL-194 (Actionability interestingness
findings — the RFC-007 workstream-2 text-first probe). Both phases landed via the
/dispatch funnel (claude arm): PHASE-01 core catalogue (S=3d338655) and PHASE-02
β-family (S=d30ed33a). Reviewed surface: the candidate impl bundle
`candidate/194/review-001` (tip 6f332f03, impl_bundle rebased onto main), NOT the
raw evidence refs (`review/194`, `phase/194-NN` are immutable, R2).

Lines of attack:
- **Conformance** — does the git-touched set match the design-target selectors
  (scope creep / dropped work)?
- **Behaviour vs catalogue** — do all nine detectors (7 core + OrderInstability +
  ArmResequencing) fire/stay-silent per the design catalogue and VT criteria?
- **Purity boundary** — shell (surface.rs) owns all disk (one scan, β endpoints
  pre-built); findings.rs stays pure/graph-only.
- **β semantics (D4, SL-172)** — endpoints {0,1}, β≡cfg.estimate.skew, silent when
  no non-terminal interval estimate exists.
- **Worker deviations** — the two ratified interpretations (arm-order basis;
  non-payload `moved` magnitude) — faithful resolutions or design mutations?
- **Governance** — ADR-001 layering (finding module engine-layer, pure), ADR-015,
  ADR-017, STD-001 named constants, render-source-of-truth.
