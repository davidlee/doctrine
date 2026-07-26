# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design (RV-307 round 5 disposed) · 0b12ed0c0

### Produced

- RV-307 — design-facet inquisition ledger. 24 findings, 5 rounds, 5 blockers.
  All disposed `fix-now`; none deferred, none tolerated.
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
- **RV-307** — `await=raiser`, 8 findings answered in round 5 and not yet
  verified. **No blocker is terminal-verified**; the close gate is not satisfied.
- design OQ-4 (other kinds adopting `write_body`), OQ-5 (narrowing the *source*
  leg to declared scopes — raised and deliberately not taken).

### Trend, recorded deliberately

Five rounds, and the defect rate has not decayed — round 5 produced a blocker
(F-20) and four majors. Two of the last three blockers arose because a remedy was
authored against the finding rather than against the invariant it instantiates
(F-15 → F-20; F-18 → F-23). Treat that as a live generative pattern when
assessing whether the design is ready to lock, not as history.
