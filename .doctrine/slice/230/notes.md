# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design (RV-307 round 6 disposed) · 3b91f780

### Produced

- RV-307 — design-facet inquisition ledger. 30 findings over 6 rounds, 7
  blockers. Round 6 verified F-18/F-21/F-23, contested F-6/F-19/F-20/F-22/F-24
  and raised F-25–F-30; all eleven disposed `fix-now`, `await=raiser`.
- REV-034 — SPEC-007 + REQ-147 amendment. Proposed, applied at close.
- IMP-317 — route `validate` **and** `retrieve` staleness through a
  history-stable claim surface. Scope widened and cost corrected by F-27/F-28.
  Gated on QUE-175.
- IMP-318 — persist attested coverage on the verification stamp. Closes D10's
  weak-reading residual (R8). Same schema-change class as QUE-173/OQ-3.
- QUE-175 — should claim-surface drift feed retrieve-side ranking (design OQ-2,
  promoted; now gates IMP-317 too).
- EVD-001, QUE-173 — from the pre-external design rounds.

### Learned

Recorded in `design.md` § 10 (the pointer table + the three closing lessons);
not restated here per that section's own rule. In brief, by id:

- the recurring defect class — adjacent paths inheriting dead assumptions:
  F-13, F-15, F-17, F-18, F-19, F-20, F-23.
- **its inverse** — an invariant generalised past the domain it holds in:
  F-26, F-27. The fix for over-specificity was over-generality.
- a remedy can be right in its conclusion and wrong in every reason: F-16.
- a probe that cannot distinguish the two outcomes proves nothing: F-17, F-23.
- history sections must point, never restate: F-14, F-16, F-22, and F-22 again
  at round 6 (the F-21 paragraph still asserting a superseded cut).
- **classify by probe outcome, not by the shape of the string**: F-25, F-26.
  Existence-on-disk is checkout-dependent; git history is not.

Candidate for `/record-memory` if they recur outside this slice — the probe
falsifier, the history-points-not-restates rule, and the
domain-of-an-invariant check are not memory-specific.

### Open

- **QUE-175** — retrieve-side adoption. Blocks IMP-317; R7 stands until answered.
- **QUE-173** / design OQ-3 — digest-based invalidation. No longer load-bearing
  on attestation truth; would buy master coverage + uncommitted-edit detection.
- **REV-034** — unapplied by design until close.
- **R5** — masters uncovered by every invalidation path (D6 scope boundary).
- **R7** — **both** `validate` and `retrieve::git_facts` keep the raw scope seam
  (widened by F-27); the weaker notion of drift drives staleness and ranking.
- **R8** — an attestation does not record what it covered; a consumer cannot
  distinguish a full stamp from a partial one. 32 active items affected.
  Routed as IMP-318.
- **RV-307** — `await=raiser`; all 11 round-6 findings disposed `fix-now`,
  none verified. Four blockers (F-6, F-20, F-25, F-27) are answered-and-unverified,
  so the close gate is **not** satisfied and `/plan` stays blocked pending round 7.
- **4 currently-stamped active memories** will refuse under D10's stale class
  once implemented — each a moved-source-path correction, not a design defect.
  Named in the F-25 disposition; a migration beat for `/plan`.
- design OQ-4 (other kinds adopting `write_body`), OQ-5 (narrowing the *source*
  leg to declared scopes — raised and deliberately not taken).

### Trend, recorded deliberately

Six rounds; the rate has not decayed, but the *kind* of defect changed. Rounds
1–5 found remedies authored against a finding rather than against the invariant
it instantiates. Round 6 found the opposite in the fix for that — I9 generalised
past `verify` into `validate`, where canonicalising a historical query erases
committed retarget drift (F-27), and a class boundary drawn on filesystem
existence rather than git history (F-25). Two decisions were refuted, not two
edges corrected.

That is a smaller and more checkable remaining question — *does each rule hold
across the domain it is stated over* — but it is not yet evidence of exhaustion,
and the user has been shown both readings. Lock on the ledger, not on confidence.
