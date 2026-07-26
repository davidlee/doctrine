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

## Defect 3 — base-commit content is structurally invisible (SL-228 audit, RV-312 F-2)

Third independent defect under the same acceptance sketch, on the **undelivered**
side rather than the undeclared one.

A phase's recorded delta is the range `[start, end]`, and a range excludes its own
`start` commit's patch. For any phase driven by the **base beat** (D-P3-1), `start`
IS the orchestrator's base commit — the one that lands the ADR-001 `[tiers]` row,
the empty module stub, the `mod` declaration, and any `.doctrine/` artefact the
worker cannot write. So everything the base commit delivers falls outside every
phase's recorded delta and can never register as conformant.

Evidence (SL-228): `slice conformance 228` reported 8 undelivered, of which three
were demonstrably delivered — `.doctrine/adr/001/layering.toml`,
`.doctrine/spec/tech/021/funnel-machine.md`, `src/main.rs`. `git log
29e966f9d..dispatch/228 -- <each>` returns exactly one commit for all three,
`b8e64e109 base(SL-228): PHASE-03 base`, and `b8e64e10` is precisely PHASE-03's
recorded boundary `--start`.

This is structural, not a recording mistake: the base beat exists because ADR-001's
layering gate is symmetric (a `[tiers]` row with no module is `StaleEntry`; a module
with no row is `Unclassified`), so a new module's row+stub+decl MUST land as an
orchestrator commit before a worker can implement into it. Every base-beat phase
therefore inherits a permanently dirty undelivered cell — which trains readers to
discount the signal, the exact failure conformance exists to prevent.

Also confirmed at the same audit: **defect 1 reproduces exactly.** All 6 undeclared
cells were `.doctrine/**` authored metadata (`dispatch/228/funnel.toml`,
`slice/228/{benchmark.md,evidence/…,notes.md,plan.toml,slice-228.toml}`) — zero
genuine source scope-creep.
