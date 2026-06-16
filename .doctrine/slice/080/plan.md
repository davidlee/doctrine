# Implementation Plan SL-080: Reconcile skill + audit/reconcile seam disentanglement

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Four phases enact the four scope items from the design. No code; all work is
skill prose (SKILL.md files) plus one routing source edit and install
verification.

## Sequencing & Rationale

**PHASE-01 (reconcile) leads.** The `/reconcile` skill is the centrepiece and
has no dependency on the retunes — it defines the artefact shapes (reconciliation
outcome, REV authoring flow) that the retuned audit and close skills will
reference. Authoring it first also surfaces any design refinements before the
audit/close prose is locked.

**PHASE-02 (audit retune) follows.** Audit must drop in-place writing and add
the reconciliation brief. The brief shape is defined in design D3; having the
reconcile skill already written lets audit's handoff prose cross-reference it
concretely rather than describing a deferred target.

**PHASE-03 (close retune) follows.** Close's spec-coherence gate depends on
understanding the reconcile outcome shape (REV done, RV-native dispositions).
PHASE-01 provides that context. Before writing, verify the IMP-008 stale-prose
list — the file was partially retuned since IMP-008 was filed (it already uses
`doctrine slice status` and references RV-NNN, not audit.md), but the
spec-coherence gate is the new addition.

**PHASE-04 (routing wire) goes last.** ADR-009 F14 mandates shipped-not-reachable
— the routing row must not point at a deferred skill. All three skills must
exist before the row is added. This phase also regenerates boot.md and verifies
`doctrine claude install` succeeds.

### Why not parallelise PHASE-02 and PHASE-03?

They are file-disjoint (audit SKILL.md vs close SKILL.md) and could theoretically
run in parallel. However, the mental model is cumulative — audit's handoff to
reconcile, reconcile's writing surface, and close's verification of reconcile's
work form a narrative chain. Serial execution with short phases keeps the prose
coherent across files and avoids reconciliation drift between the two retunes.

## Dependencies

- **ADR-003** §7 (hard audit/reconcile edge), §11 (deferred machinery)
- **ADR-009** §1 (FSM topology, `reconcile → done` seam, `reconcile → design`
  back-edge), F2/F14 (shipped-not-reachable)
- **IMP-008** stale-prose list (close SKILL.md baseline verification)
- **`install/review-ledger.md`** — RV mechanics referenced by all three skills
  (read on demand during implementation; plan does not repeat the verbs)

## Notes

- The `slice reconcile` CLI verb is explicitly out of scope (D8, ADR-003 §11).
  The reconcile skill drives existing verbs (`doctrine revision *`, direct file
  edits).
- OQ-2 (one REV per slice vs per finding) is resolved in design: one REV per
  slice by default, with split rule. Reconcile skill implements the split rule.
