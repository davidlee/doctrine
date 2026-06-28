# IMP-201: Split-lineage close (code tier) — route dispatch PHASE-01 into the bundle

**Source:** SL-169 PIR S4 (split-lineage close, MEDIUM). **Home:** RFC-005.

The dispatch worker executed PHASE-01 directly on its worktree's `edge` fork while
phases 02–05 landed in the journaled bundle. At close, edge carried a divergent
partial PHASE-01 against the complete reviewed candidate → a split-lineage
convergence dance (pre-FF main, resolve duplicate helper, converge edge later).
This is the CODE tier — distinct from IMP-174 (authored-state split-brain).

**Fix direction:** route all phase commits into a `review/<slice>` staging ref,
never directly onto the worktree's edge fork; or `/close` detects the split
(edge-lineage commits touching declared paths but not ancestors of the candidate)
and offers a convergence recipe.

Related: RFC-005; IMP-174 (authored-tier sibling); governed_by ADR-012.
