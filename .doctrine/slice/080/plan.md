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
— the routing row must not point at a deferred skill. `doctrine claude install`
must succeed before the routing rows are edited. The intra-phase order is:
(1) install and verify, (2) edit the routing rows (update audit→close chain
and add /reconcile row), (3) regenerate boot.md and verify coherence.

### Entrance criteria: soft preferences vs hard gates

PHASE-02 and PHASE-03 entrance criteria are **soft coherence preferences**, not
hard structural gates — they recommend reading /reconcile first so the audit and
close handoff prose is consistent, but PHASE-02 and PHASE-03 may proceed
concurrently if the design shapes (D3, D4) are understood. PHASE-04 is a **hard
structural gate**: ADR-009 F14 forbids routing at a deferred skill
(shipped-not-reachable), so all three skills must be installed and embedded
(`doctrine claude install` succeeded) before the routing rows are edited.

PHASE-02 and PHASE-03 are file-disjoint and could run in parallel. Serial
execution with short phases keeps the narrative chain coherent across files
(audit → reconcile → close) and avoids cross-skill drift.

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
- D9 (inspect-only, no re-audit) is captured as PHASE-01 EX-7 — reconcile
  inspects targets for applicability and edit-point location but does not
  perform new issue discovery.
- IMP-008 stale-prose items (1–4) are restated inline in PHASE-03 EN-2 — the
  implementer does not need to cross-reference the IMP-008 file.
