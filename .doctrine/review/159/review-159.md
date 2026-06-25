# Review RV-159 — reconciliation of SL-153

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Lines of attack:**

1. **PHASE-01 — `apply_scalar` pure core**: does the new leaf seam match design
   §5.2 (set/clear, create-vs-refuse contract distinction, CHR-019 worst-case,
   no-op guard, module doc)? Are the 3 unit tests covering the specified
   scenarios?

2. **PHASE-02 — `spec edit` (descends_from / parent)**: does `run_edit` follow
   the phased validation-then-single-write contract? Are all gates present
   (tech-only descends_from, subtype-aware parent, relation::lookup reuse E4,
   target existence, parent acyclicity E1)? Are the plan's VT-1–VT-5
   verification criteria covered by tests?

3. **PHASE-03 — `spec interactions add/remove`**: does add ride the AoT-push
   pattern, enforce tech-only + target-as-PK, canonicalize existing on-disk
   targets (E3)? Does remove use the NEW `remove_interaction_edges` helper
   (not `dep_seq::remove_after`, E2)? Are VT-1–VT-8 covered?

4. **PHASE-04 — shipped memory refresh**: does the relating-entities memory
   drop the stale hand-editing guidance and surface the three new spec verbs?

**Invariants held:**
- All writes edit-preserving (toml_edit mutate → write_atomic).
- No parallel implementation; existing suites green unchanged.
- Pure/imperative split maintained (apply_scalar is pure, shells do I/O).
- Clap ArgGroup intra-field mutual exclusion.
- Canonical-idiom consistency: store canonical, compare canonical (E3).

**Review surface:** candidate/153/review-001 (admitted at 9c045f5c, cand-153-review-001, RV-159).

## Synthesis

Three findings, all terminal. Two fix-now (applied on the candidate), one tolerated.

The implementation is solid. The three load-bearing contracts — `apply_scalar`
create-vs-refuse distinction, the `run_edit` phased pre-validation-then-single-write
shell, and the `remove_interaction_edges` AoT-index-remove (not `remove_after`)
— all match design. CHR-019 grounding is pinned in the `apply_scalar` VT-1 test.
The parent acyclicity gate (E1) correctly builds a prospective parent map and walks
from the proposed target, catching self-parent and 2-node cycles before any write.
The canonical-to-canonical comparison for interactions (E3) correctly canonicalizes
existing on-disk row targets before matching, and the add no-op is properly
informational. Kind validation rides `relation::lookup` + `check_target_kind` for
declared rows (E4) with a narrow product-parent branch for the undeclared case (R2).

**Fix-now items applied (candidate branch, 9c045f5c):**
- F-1: 6 edit test functions (VT-1 through VT-5) plus 1 canonical-storage test
  added to `spec.rs`, exercising descends_from tech/product gating, parent
  subtype/existence/acyclicity gates, clear-present/clear-absent, multi-field
  single-write-once, and all-no-op mtime hold.
- F-2: `run_interaction_add` now stores `canonical_target` instead of raw `target`;
  the new `interaction_add_stores_canonical_target` test asserts the stored form.

**Tolerated:**
- F-3: Edit confirmation messages omit the source spec ref ("Set parent = X" vs
  "Set SPEC-005 parent = X"). Cosmetic; the user typed the source in the CLI
  invocation. Interactions messages already demonstrate the richer pattern.

**Standing risks:**
- R2 (RELATION_RULES honesty drift — product parent undeclared) remains open per
  slice scope; routed to the follow-up UX review backlog item.
- The candidate's `map_server/assets.rs` build failure (RustEmbed web/map/dist
  absent) is a pre-existing worktree infrastructure issue, not a slice regression.

## Reconciliation Brief

### Per-slice (direct edit)

No per-slice design/governance changes needed. All fix-now code items were applied
on the candidate interaction branch and admitted. The slice can close directly.

### Governance/spec (REV)

None. No spec, ADR, or policy changes surfaced by this audit.

## Reconciliation Outcome

All three findings are terminal (verified / tolerated). Both fix-now items (F-1, F-2)
were applied on the candidate interaction branch (cand-153-review-001, 9c045f5c)
and admitted. No per-slice design/governance changes or REV authorship needed.

Reconcile pass complete — handoff to /close.
