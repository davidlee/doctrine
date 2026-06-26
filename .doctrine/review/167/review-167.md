# Review RV-167 — reconciliation of SL-158

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject:** SL-158 — Trinary actionability (3 phases, dispatched).

**Lines of attack:**

1. **D1 — Trinary partition.** Does `StatusClass::Gating` correctly split `eligible`
   (= Workable) from `blocks` (!= Terminal)? Are the four knowledge rows'
   unsettled/settled boundaries correct? Do the two design-flipped tests read
   right? Does the three-way cover canary hold?

2. **D2 — Target-admissibility gate widening.** Does `is_admissible_dep_target`
   admit records (ASM/DEC/QUE/CON) but exclude governance (SPEC/ADR/POL/STD)?
   Is the source gate unchanged (records still cannot author dep/seq)? Does
   the refusal message reflect the new boundary?

3. **D6/D3 — Records author references + estimate confirm.** Does RECORD appear
   in the `concerns` source-set? Does `link QUE references SL --role concerns`
   resolve? Does `estimate set` round-trip on a record without rejection?

4. **Conformance.** Design-target selectors covered `partition.rs`, `dep_seq.rs`,
   `relation.rs`. Two files touched undeclared: `knowledge.rs` (D3 VT-7 test)
   and `relation_graph.rs` (fixture updates for D6). Two declared files
   undelivered: SPEC-001/SPEC-019 (D4 canon-moves-first — delegated to
   reconcile per design).

5. **Behaviour-preservation.** Do existing priority/dep/relation suites stay
   green (except the two design-flipped partition tests)? Does the three-way
   cover reduce to binary where `gating == ∅`?

## Synthesis

All four design decisions (D1, D2, D3, D6) are faithfully implemented across
three phases. The trinary partition is the cleanest possible diff: one new enum
variant, one new field, four rows reclassified — `channels.rs`/`graph.rs`/
`render.rs` untouched. The predicate semantics (`eligible == Workable`, `blocks !=
Terminal`) absorb `Gating` with zero code change, exactly as designed.

The D2 gate widening restores ADR-017's intent: records are now admissible
`needs`/`after` targets while governance stays excluded and records cannot
author dep/seq. The refusal message is clear and specific.

D6 grants records the `references(concerns)` edge they were illegally barred
from. D3 is confirmatory — estimate/value already round-tripped; a VT now pins
it. The optionality wiring (record base → referenced target) activates once a
record is sized.

**2607 tests pass, 0 fail.** Behaviour-preservation holds: the three-way cover
reduces to the old binary everywhere `gating == ∅`. The two consumer-revision
tests correctly reflect the new semantics.

**No blockers.** Four minor findings, all `aligned` — conformance noise
(undeclared test files, undelivered specs per D4), ADR-017 prose reconciliation
tracked for close, and behaviour-preservation confirmed.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §3 Code impact:** add `src/knowledge.rs` and `src/relation_graph.rs`
  to the design-target touch-set (test infrastructure supporting D3/D6
  verification — F-1).

### Governance/spec (REV)

- **SPEC-001 / PRD-011:** author D-decision + requirement for the trinary
  status class (`Gating`), the `eligible`-vs-`blocks` split, and records as
  admissible `needs` targets (D4, undelivered per design — F-2).
- **SPEC-019:** revise D7 / NF-003 / OQ-2 — records become `Gating` (unsettled)
  / `Terminal` (settled), no longer all-inert (D4 — F-2).
- **ADR-017:** reconcile prose — the 'source-only gate' premise is false;
  the target gate also required widening (D2). The code is correct; the ADR
  prose needs updating (F-3).

## Reconciliation Outcome

### Direct edits applied
- `design.md` §3: added `src/knowledge.rs` and `src/relation_graph.rs` to
  design-target touch-set (RV-167 F-1).

### REVs completed
- **REV-013** (`reconcile-sl-158`): done — SPEC-001 gained D13 + REQ-239;
  SPEC-019 D7 + OQ-2 revised for SL-158 landing; ADR-017 §3 prose corrected
  (covers RV-167 F-2, F-3). Rationale in revision-013.md.

### Aligned / verified
- RV-167 F-1: aligned — undeclared test files acknowledged.
- RV-167 F-2: aligned — undelivered specs per D4, now delivered via REV-013.
- RV-167 F-3: aligned — ADR-017 prose corrected via REV-013.
- RV-167 F-4: aligned — behaviour-preservation confirmed (2607 tests green).
