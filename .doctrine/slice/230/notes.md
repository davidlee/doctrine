# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design (RV-307 round 6 contested) · 59bfb4c9b

### Produced

- RV-307 — design-facet inquisition ledger. 30 findings over 6 rounds.
  Round 6 verified F-18/F-21/F-23, contested F-6/F-19/F-20/F-22/F-24, and
  raised F-25–F-30 (2 blockers, 3 majors, 1 minor). Uncommitted for the
  architect; no code changed and `doctrine check gate` was not rerun.
- REV-034 — SPEC-007 + REQ-147 amendment. Proposed, applied at close.
- IMP-317 — route `retrieve` staleness through the shared claim-surface
  constructor. Gated on QUE-175.
- QUE-175 — should claim-surface drift feed retrieve-side ranking (design OQ-2,
  promoted; now gates IMP-317 too).
- EVD-001, QUE-173 — from the pre-external design rounds.

### Learned

Recorded in `design.md` § 10 (the pointer table + the three closing lessons);
not restated here per that section's own rule. In brief, by id:

- the recurring defect class — adjacent paths inheriting dead assumptions:
  F-13, F-15, F-17, F-18, F-19, F-20, F-23.
- a remedy can be right in its conclusion and wrong in every reason: F-16.
- a probe that cannot distinguish the two outcomes proves nothing: F-17, F-23.
- history sections must point, never restate: F-14, F-16, F-22.

Candidate for `/record-memory` if they recur outside this slice — the probe
falsifier and the history-points-not-restates rules are not memory-specific.

### Open

- **QUE-175** — retrieve-side adoption. Blocks IMP-317; R7 stands until answered.
- **QUE-173** / design OQ-3 — digest-based invalidation. No longer load-bearing
  on attestation truth; would buy master coverage + uncommitted-edit detection.
- **REV-034** — unapplied by design until close.
- **R5** — masters uncovered by every invalidation path (D6 scope boundary).
- **R7** — `retrieve::git_facts` keeps the raw scope seam; weaker notion of drift
  drives ranking.
- **RV-307** — `await=responder`; F-6/F-19/F-20/F-22/F-24 contested and
  F-25–F-30 open. Four outstanding blockers: F-6, F-20, F-25, F-27.
  The close gate is not satisfied.
- design OQ-4 (other kinds adopting `write_body`), OQ-5 (narrowing the *source*
  leg to declared scopes — raised and deliberately not taken).

### Trend, recorded deliberately

Six rounds, and the defect rate has not decayed — round 6 produced two blockers,
three majors and a minor. The same generative pattern remains live: D10 makes
F-6's uncovered evidence audible but still attests it (F-25), while D11 shares a
current-worktree surface with a historical query and erases link-retarget drift
(F-27). The design is not ready to lock.
