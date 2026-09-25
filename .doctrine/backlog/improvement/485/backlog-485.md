# IMP-485: Conformance registry mode for post-phase review repairs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at the SL-264 audit (RV-390 F-3, tolerated; F-4). The selector registry
is keyed by phase. Review repairs that land after the last phase's row (here,
RV-389's four repair commits) belong to no phase, so `slice conformance` never
sees them:

- a path touched only by a repair reads as **undelivered** (SL-264's
  `install/design-prompts/delegation.md`);
- an undeclared path a repair introduces is invisible, and the audit has to find
  it by hand (`git diff --stat`).

Review-before-audit is now routine, so this recurs on every slice. Inventing a
PHASE row would misattribute the work.

Acceptance sketch: a non-phase registry row (e.g. `review_repair`, keyed by the
RV that drove it) that conformance folds into delivered/undeclared, so a
post-review audit reads a clean verdict with no manual reconstruction.

Related: IMP-292 (other audit-time conformance noise), IMP-282.
