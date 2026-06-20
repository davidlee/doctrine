# IMP-128: Author a tech spec for the git interaction model (dispatch/candidate/review/integrate vs git's object+ref model)

## Motivation
The dispatch/candidate/review/integrate machinery was designed incrementally — many
disjoint local decisions guided by generalised technical intuition, not a deep,
unified model of git's object + ref model or how the whole hangs together. The
result works but has no single authoritative description of the **intended** model,
so questions like "where must an audit-time repair live to actually integrate?" have
no canonical answer (witnessed: RV-116 on SL-127 — the audit skill says "repair on
the candidate interaction branch" while close sources `close_target` from raw
`review/<N>`; the two only reconcile under specific sourcing assumptions).

## Scope
A `.doctrine/spec/tech/` spec that defines, coherently and against git's actual
model (objects, refs, merge-base, CAS, worktrees):
- the ref taxonomy and lifecycle: `dispatch/<N>`, `review/<N>`, `phase/<N>-NN`,
  `candidate/<N>/<label>` (review_surface vs close_target), and which are mutable
  vs immutable evidence (R2);
- the pinned-fork-point invariant (RV-030 F-1) and `refresh-base`'s explicit-advance
  exception;
- the **repair → integrate** path: where a fix-now repair belongs and how it
  provably reaches trunk (the gap this item was born from);
- prepare-review projection (boundary cuts, the run-ledger source — see ISS-039),
  and the stage-2 integrate CAS contract.

## Value
Turns accumulated intuition into a checkable model; future slices touching dispatch
stop rediscovering the same ambiguities. Precursor/​companion to a possible
consolidation of `dispatch.rs`/`git.rs` seams.

## Links
Born from RV-116 (SL-127 audit). Related: ISS-039 (boundaries.toml not committed),
RV-030 (pinned fork-point), ADR-006/011/012, SPEC-021/012.
