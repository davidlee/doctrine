# Notes SL-220: Ledgered value claims

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Audit harvest (RV-277, 2026-07-17)

Durables lifted from the runtime phase sheets before they are discarded.

### Adjudications (mid-phase, orchestrator/operator)

- **PHASE-02 mechanical fallout**: optionalising `Judgement.b/response` broke
  31 consumer sites across 8 files; adjudicated — PHASE-02 absorbs *mechanical*
  Option-handling only (anchor rows filtered at row-iteration boundaries,
  commented transitional), the typed `PairRow` seam stayed a PHASE-03
  deliverable. Zero golden churn held.
- **PHASE-03 additivity pin**: compile's anchor input stayed the facet builder
  until PHASE-05 (D12 — no resolution outcome changes before the flip);
  `anchor_map()` existed unconsumed for one phase.
- **PHASE-04 D13 WriteClass**: reused `WriteClass::Orchestrator` with labels
  "value pin" / "value pin --retire" — no new variant; TTY gate threaded as
  `is_interactive: bool` pure input.
- **PHASE-05 accepted deviation**: `ReasonKind::ValueUnmigratedFacet` (design
  §6 named `ValueFacetUnmigrated`; the substring `ValueFacet` trips the NF-001
  facet-symbol tripwire). Semantics + D11 JSON token unchanged.
- **PHASE-05 VA-2 churn review**: flip churn confined to e2e_compare_elicit
  (2 tests) + e2e_compare_inference (3 fixtures); e2e_priority_golden and
  e2e_compare_estimate green unchanged (their facet fixtures have no
  comparison rows — rung 5 serves identical numbers).
- **PHASE-06 tangle ratchet**: Command-tier baseline 86→99 bumped by the
  orchestrator with rationale (7661b3fd); worker delta replayed verbatim onto
  the bumped base — the halt was the ratchet working.
- **PHASE-06 renders**: hoisted-anchor (ValueProvenance::Authored) stays
  rendered as projected (rung 2 fed scoring; IDE-040 captures a
  tier-attributed shape); demotion disclosure satisfied as-is (superset
  condition already names the case).

### Handoff contracts worth keeping

- `ClaimTier::is_anchored()` is the single D4/D14 predicate;
  `claim_token()` in store.rs is the sole display-token source.
- `value_source_token()` in view.rs is the single D11 JSON token source.
- Grep-gate exceptions (the only sanctioned `EntityFacets.value` display-path
  reads): surface.rs rung-5 ladder read; commands/compare.rs elicit
  `facet_valued` consumption fill.

### Operational

- **v3-binary interregnum**: corpus is v3-only from the PHASE-07 session emit;
  ALL corpus verbs via `.dispatch/doctrine-v3-sl220` until /close integrates
  to trunk (edge-installed binary is pre-v3).
- Migration evidence chain: phase0-live/neutral → phase7-postflip-premigration
  (flip effect isolated) → phase7-postmigration (migration all-zero neutral).
  Census 185/185/0/0; strips 185/185 verified; idempotent re-run no-op.
- CLI drift (PHASE-01): design §5 named `doctrine reports survey`; shipped
  verb is `doctrine survey --json`. Scripts call the real verb.
- verify-vt for a dispatched slice: run from the admitted candidate surface
  (14/14 PASS attributed at f8e7ca38); primary-tree FAIL pre-integration is a
  mechanical artifact.
- Audit repair channel: RV-277 F-4 doc fix landed on the candidate branch
  (f8e7ca38), ref advanced by CAS update-ref, re-admitted with
  `--review RV-277`.
