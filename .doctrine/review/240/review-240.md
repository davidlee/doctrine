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

## Synthesis

**Closure story.** SL-194 lands clean. Mechanical conformance is exact — 8
conformant paths, **0 undeclared, 0 undelivered** (`slice conformance 194`): every
git-touched file (findings/graph/surface/order/render/mod.rs + cli.rs/guard.rs) was
declared by a design-target selector. `check gate` is green (clippy zero-warning +
full workspace suite) on the candidate bundle. The VT-backing evidence is
comprehensive: 14 `priority::findings` tests (every detector incl. both β-family +
`beta_family_silent_without_betas`), 16 `priority::surface` (incl.
`beta_endpoints_some…none` = PHASE-02 VT-1 and the vt7 order.rs-extraction
behaviour-preservation set = VA-1), 28 `priority::render` (incl. β-family json
payload/magnitude + no-beta-section = VT-3). `slice verify-vt 194` passed all 9 VTs
before prepare-review.

**Differentiated value confirmed.** The probe's thesis — findings-over-flat-list —
holds on live corpus data. The sharpest signal composes on one hub: `IMP-085` is
simultaneously a Fork (gates 4 sunk arms) and an ArmResequencing (its arm order is
β-contested, IMP-089↔IMP-087 swapping between optimistic and pessimistic cost). No
per-node flat list expresses "this hub gates four items *and* their relative
priority is estimate-sensitive." VH-1 full verdict: **useful** (recorded in
design.md PHASE-02 verdict section).

**Standing risks / consciously-accepted tradeoffs.**
- **R1 starvation** — joins / gating-fan-out / value-inversion / provenance emit
  nothing on the current corpus (thin ADR-017 gating records + sparse authored
  estimates). Expected and design-documented; the mechanism is proven by the unit
  fixtures, not the live corpus. Activates as data grows (RFC-007 workstream 3).
- **R5 endpoint-contested false-negative** — the {0,1} sweep misses interior order
  flips that share a sign at both endpoints (pairwise score diff non-monotone in β).
  Known, design-narrowed (the finding claims *endpoint* sensitivity only); finer
  flip-β grid deferred (IMP-243 territory), not this slice.
- **R4 quiescent-tree precondition** — a concurrent needs/after commit mid-sweep can
  misattribute topology churn as β instability (one self-correcting spurious line).
  Accepted at probe grade.
- **F-2 volume (tolerated)** — 62 order-instability lines (~⅓ of adjacent frontier
  pairs) reads as high-volume. The detector is correct-by-design; a magnitude /
  score-gap threshold is the refinement, captured as IMP-247. Rendering fold-into-
  survey captured as IMP-248. Both originates_from SL-194.

**Worker deviations (F-1, aligned).** Neither was a design mutation: magnitude =
"positions/arms moved" was mandated in the catalogue table (the `moved:usize` field
carries it internally; payloads unchanged), and the arm-order-among-arms basis is a
faithful resolution of an underspecified derivation (fork arms `need` the hub → are
absent from the actionable frontier, so any "arm order *within* the frontier" would
be perpetually empty). Both ratified by the user at VH-1 and documented in design.md.

**No blockers. No governance/spec divergence.** ADR-001 layering held (findings.rs
is pure engine-layer, disk confined to the surface.rs shell); STD-001 named
constants (BETA_LO/BETA_HI, the ε consts); render-source-of-truth respected.

## Reconciliation Brief

The audit surfaced **no** spec/governance divergence and **no** residual per-slice
prose drift requiring a reconcile write:

### Per-slice (direct edit)
- **None.** design.md already tells the truth — the PHASE-02 verdict section
  (committed 52679b8a) documents both ratified interpretations and the two deferred
  follow-ons; the catalogue table already mandated the `moved` magnitude semantics.
  F-1 disposed *aligned*, F-2 *tolerated* — neither maps to a design edit.

### Governance/spec (REV)
- **None.** No ADR / spec / REQ divergence surfaced; ADR-001/015/017 + STD-001 held.

### Follow-up work (already captured, not reconcile surface)
- IMP-247 — order-instability magnitude/score-gap threshold (F-2 refinement).
- IMP-248 — findings rendering follow-on (arc-strip / fold-into-survey).
- Both tagged area:priority/cli/ux and linked `references SL-194 --role originates_from`.

**Reconcile is a no-op write pass** for SL-194: the brief is intentionally empty of
write items. Reconcile confirms the clean audit and advances the lifecycle.

## Reconciliation Outcome

No-op write pass. Both RV-240 findings are terminal with no write surface:

- **F-1 (verified · aligned)** — the two ratified worker interpretations were
  faithful resolutions, not design mutations. design.md already tells the truth
  (PHASE-02 verdict section, 52679b8a). No edit.
- **F-2 (verified · tolerated)** — order-instability volume; correct-by-design,
  refinement deferred to IMP-247 / IMP-248 (both `originates_from SL-194`). No edit.

### Per-slice (direct edit)
- None. design.md + slice-194.md already truthful.

### Governance/spec (REV)
- None. No ADR / spec / REQ divergence; ADR-001/015/017 + STD-001 held.

Reconcile pass complete — handoff to /close.
