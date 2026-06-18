# Review RV-081 — reconciliation of SL-098

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-098 (Requirements discovery and home-finding) — a
skill-only slice editing 5 skills (design, plan, audit, reconcile, close) to
add implied-requirement discovery and orphan placement to the reconcile loop.

**Lines of attack:** Does each skill edit match its design specification
(design.md §3-§7)? Does the REQ-D → REQ-NNN lifecycle story hold end-to-end
across all 5 skills? Are the specific guardrails (E1 XML sync, B1 multi-spec
siblings, B2 distinct collect/ask states, C1 4f placement, C2 no-op gate, D1
prose list, E2 stuck/withdrawn, E3 read-path, F2 advisory naming) visible?

**Review surface:** candidate/098/review-001 (admitted at 5e267524, linked via
`dispatch candidate admit --review RV-081`).

## Synthesis

SL-098 shipped cleanly — all 5 phases delivered exactly the skill edits the
design specified. The slice was driven through the dispatch funnel (4 worker
batches, file-disjoint where possible: PHASE-02+PHASE-03 parallel, batch 1
PHASE-01 solo, batch 3 PHASE-04 solo, batch 4 PHASE-05 solo).

**Standing risks:** The eslint `@eslint/js` missing-package error in worktree
environments is a persistent infrastructure issue that doesn't affect
skill-only slices but is a general worktree-provisioning gap.

**Tradeoffs consciously accepted:**
- D1: plan.md verification mapping is prose (not plan.toml) — the registry
graduation trade is named honestly in the plan skill.
- F2: close orphan check is advisory, not binary-enforced — the follow-up IMP
for RV-ledger enforcement is referenced.
- B1: multi-spec placement uses sibling REQ-NNNs, not shared REQ-NNN — an
honest constraint until the CLI supports re-membering.
- IMP-097: altitude assessment framework is deferred — `/consult` guardrail
present in reconcile skill.

## Reconciliation Brief

### Per-slice (direct edit)

(No direct-edit items — all findings aligned with implementation.)

### Governance/spec (REV)

(No governance/spec items — all findings aligned with implementation.)

#### Orphaned requirements (REV introduce)

(No orphaned requirements — SL-098 is a skill-guidance slice with no implied
requirements of its own; design §11a-§11e walkthroughs run against
hypothetical artefact state.)

## Reconciliation Outcome

All findings were aligned. No writes needed. Reconcile pass complete —
handoff to /close.
