# IMP-292: Audit-time conformance signal degradation

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at the SL-219 audit (RV-276 F-3, tolerated): `slice conformance`
could not produce a clean mechanical verdict from either surface an auditor
actually has, and the algebra had to be reconstructed by hand across two runs.

Two independent defects:

1. **Authored-metadata paths pollute the undeclared cell.** Funnel boundary
   ranges legitimately include orchestrator commits that edit the selector
   registry itself (e.g. `plan(SL-219): declare … design-target`), so
   `.doctrine/slice/NNN/slice-NNN.toml` surfaces as an undeclared *source*
   delta. Conformance compares source deltas to design-targets; `.doctrine/**`
   authored metadata should be classified out of the algebra (or into its own
   cell), not reported as scope creep.

2. **Candidate worktrees cannot answer conformance at all.** Phase-completion
   state is runtime tier and is not provisioned into candidate worktrees, so
   the verdict there degrades to `incomplete — recorded row for PHASE-NN,
   which is not a completed phase` for every phase — precisely in the tree
   audits are told to review (ADR-012 candidate surface). Either provision the
   phase-status snapshot into candidates or teach conformance to read it from
   the parent tree.

Acceptance sketch: a post-drive audit reads `conformant / undelivered /
undeclared` with zero manual reconstruction from at least one of (parent
tree, candidate worktree), with registry-noise paths excluded or reported in
their own cell.
