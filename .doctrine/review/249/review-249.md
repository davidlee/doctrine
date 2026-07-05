# Review RV-249 — reconciliation of SL-197

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

SL-197 adds a seventh knowledge record kind (Concept/CPT) in two phases:
PHASE-01 (DRY the record-kind scatter, behaviour-preserving) and PHASE-02
(add Concept). Lines of attack:

1. **Conformance**: one undeclared touch (`tests/e2e_list_conformance.rs`) —
   the concept kind's "draft" status joins knowledge's union vocab, which broke
   the `status_flag_is_recognised_grammar_on_every_kind` assertion that assumed
   knowledge had no draft-bearing kind. Fixed during the funnel.

2. **VT compliance**: all 10 VT criteria pass (PHASE-01: VT-1–4; PHASE-02:
   VT-1–5). VT-6 (supersede CPT rejection) is UNATTRIBUTABLE — supersede.rs was
   not modified by design (D4: CPT is non-supersedable with zero edits), and
   the existing policy gate works correctly.

3. **Design fidelity**: every D1–D4 decision implemented as specified:
   - D1: CONCEPT_STATUSES [draft, active, retired], seed draft, hidden/terminal {retired} ✓
   - D2: empty ConceptFacet with empty [facet] seed header ✓
   - D3: CPT rides RECORD for Shapes/Spawns; concerns added explicitly ✓
   - D4: zero supersede.rs edits, CPT rejected by _ => None ✓
   - D0 (scope): PHASE-01 DRY (P2/P3/P4) + PHASE-02 CPT append, two phases ✓

4. **regression**: S1 baseline captured at each base; diff clean both phases.
   `doctrine check gate` green.

5. **Golden re-pins**: e2e_validate_byte_exact_golden and e2e_knowledge_cli_golden
   re-pinned to 7-kind strings; scaffold-order loop green unedited.

## Synthesis

SL-197 delivers a clean, well-scoped addition of the Concept record kind.
The two-phase approach (DRY first, then add) paid off: PHASE-01's canaries and
vocab-derived messages made PHASE-02 a mechanical append with zero surprise.

The single conformance gap (`tests/e2e_list_conformance.rs`) was a minor
scope leak — the design didn't anticipate that knowledge's union vocab gaining
"draft" would break the flag-recognition assertion. Fixed during the PHASE-02
funnel; the selector has been added post-audit.

Design fidelity is high: all four design decisions (D1–D4) are implemented as
specified, including the zero-edit D4 (supersede gate via existing _ => None).
VT compliance is complete (10/10 pass, 1 UNATTRIBUTABLE per design). S1
regression is clean at both phase boundaries. `doctrine check gate` green.

No unresolved risks. The residual record-kind scatter (stale doc comments in
supersede.rs, integrity.rs, etc.) is deferred to the backlog DRY-the-scatter
item — out of scope per the design.

## Reconciliation Brief

No governance or spec changes required. One per-slice direct edit:

### Per-slice (direct edit)
- slice-197.toml: selector for `tests/e2e_list_conformance.rs` added (F-1 —
  undeclared conformance touch, now closed).
