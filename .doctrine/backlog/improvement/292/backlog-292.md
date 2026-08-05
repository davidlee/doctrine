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

## Defect 4 — a governance design target discharged at reconcile is undeliverable by construction (SL-244 reconcile, RV-345 F-2)

Fourth defect under the same acceptance sketch, again on the **undelivered** side,
and it is a whole class rather than a beat-specific edge.

A slice may declare a spec / ADR / policy path as a design-target selector, which
is exactly what `SL-244` did with `.doctrine/spec/tech/029/**`. The `/reconcile`
skill and `ADR-013` then route governance truth through a `REV` — so the write
that discharges that target lands during **reconcile**, after the last phase is
`completed`. Every recorded delta is a phase boundary (`slice record-delta` binds
`[start, end]` to a `PHASE-NN`, and there is no reconcile-stage row to bind), so a
reconcile-stage write is outside every boundary and the cell reads `undelivered`
forever.

Evidence (`SL-244`): `REV-048` landed the owed `SPEC-029` amendment in both tiers,
`spec validate SPEC-029` clean, and `slice conformance 244` still reports
`undelivered (1): .doctrine/spec/tech/029/**` at close.

The two available workarounds are both wrong, which is what makes it structural.
Extending the last phase's boundary to swallow the reconcile commit falsifies the
phase record. Removing the selector (`slice selector rm`) is the sanctioned repair
for a *spurious* undelivered row, but this one is not spurious — the target was
genuinely declared and genuinely delivered, and dropping it erases the promise
audit checks against. What is missing is a stage the registry can attribute a
reconcile write to, or a conformance rule that treats a target discharged by a
`done` `REV` as delivered.

Consequence, same as defects 1–3: `/close` reads a red cell that is correct in
mechanism and wrong in meaning, and the reader is trained to discount the signal.
