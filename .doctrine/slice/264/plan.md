# Implementation Plan SL-264: Inquiry-map growth without re-attestation

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

The slice has three objectives (`slice-264.md`): stop a *shape* change to the
inquiry map voiding an attested judgement about it; make the blocking set a
property of the node rather than a free-standing agent act; and wire
`needs: null` so the published sparse contract is not a lie. Objectives 1 and 2
are one change — the design says so outright (`design.md` sec-1) — because the
narrowed binding is only safe alongside a derived set: `blocking_inquiries_open`
used to be protected by the map-moved staleness of the two attested rows, and
design sec-2 replaces that guard with a full-set comparison of blocking marks.

The plan is five phases. Four do the work of the two substantive objectives plus
the defect; the last closes the prose mirrors and measures the property on a real
run. Every phase ends green, and no phase leaves a half-landed guard.

## Sequencing & Rationale

**PHASE-01 — `needs: null`.** Independent of everything else, smallest blast
radius, and closes `ISS-481` on its own. The semantics already exist
(`Sparse::apply_collection` maps `Null` and `[]` to one clearing); the phase is
wiring plus the tests. Doing it first means the largest change does not carry an
unrelated defect into its diff.

**PHASE-02 — the `blocking` node attribute.** `InquiryNode` and `NodeMaterial`
gain the judgement; the wire key becomes two homes; import seeds
`blocking: true`. This phase deliberately does *not* touch the gate: the block
set still derives from the stored declaration, so the change is observable
without moving any verdict. Splitting here is what keeps each phase green — the
alternative, one phase for the whole of sec-3, would be oversized and would land
the wire contract and the gate semantics in one unreviewable diff.

**PHASE-03 — the legacy class.** `ActKind::is_legacy`, the restated
one-row-per-kind invariant, `Refusal::RetiredAct`, `live_acts`'s exclusion and the
frozen `ConfirmationStale` read. This is a prerequisite, not a tidy-up: retiring
the key makes `admit_and_record`'s `None` arm reachable, so the class and the
retirement must land together or the engine acquires a path that stores an
unchecked record (`RV-386` F-16's concrete defect). It precedes PHASE-04 because
it freezes the legacy set, which is exactly the content the per-node fallback
then reads. It also carries a pin hazard: `stale_conjunct_does_not_satisfy`
re-declares the blocking set to isolate `CoverageStale`, so its *setup* must be
re-based here while its *assertion* survives to the next phase.

**PHASE-04 — the narrowed predicate and the derived set, together.** The atomic
pair. `CoveredSet::moved` takes the `Coverage` an act's rule names, so the gate
and the change log cannot disagree (`RV-386` F-13); `ReviewedGraph` compares the
marks over the full set whatever their lifecycle (`F-15`); `blocking_inquiries_open`
returns the open blockers from the same one judgement. `stale_conjunct_does_not_satisfy`
is replaced rather than deleted — the design's VT-3 pin, observed green at the
phase entrance.

**PHASE-05 — mirrors, integration, measurement.** The two generated goldens are
regenerated inside the phases that invalidate them (PHASE-02 and PHASE-03 for the
payload contract, PHASE-04 for the stage table), so this phase carries only the
authored prompt prose, the full gate, and the `RFC-031` fitness re-measure on a
real run. Keeping the goldens with their inducing change is what stops a phase
ending on a knowingly-red golden.

## Notes

- **Premise re-grep (plan-time, step 2).** Every concrete reference in
  `design.md` sec-5 resolves against the current tree: `InquiryNode`,
  `NodeMaterial`, `ChangeEvent`, `Coverage` (`src/design_run/gate.rs:91`),
  `CoveredSet` (`attestation.rs:535`), `KeyContract`, `inert_key`,
  `Sparse::apply_collection`, `coverage_fault`, `requirement_for`,
  `stale_conjunct_does_not_satisfy`, `Presence`, and both goldens'
  render/regen tests (`artifact.rs`, `commands/design.rs`). `is_legacy` and
  `ReviewedGraph` are **new** symbols the design introduces; the line numbers in
  the design have drifted, which is expected and immaterial.
- **Research advisory.** `doctrine slice research 264` reports drift on
  `design.md` and `slice-264.md` only — the slice's own intent documents, which
  the pre-design round necessarily precedes. The code-map threads name
  `src/design_run/**`, unchanged since the research round, so the drift is
  artefact-only and the baseline was restamped. The slice's selectors already
  carry Thread 2's hotspot map.
- **Design-to-phase criterion mapping.** Design `VT-1`→PHASE-04 `VT-1`/`VT-2`;
  `VT-2`→PHASE-01 `VT-1`/`VT-2`; `VT-3`→VA-1 on PHASE-04 and the two golden
  regenerations; `VT-4`→PHASE-04 `VT-3`; `VT-5`→PHASE-02 `VT-1`/`VT-2`/`VT-3`;
  `VT-6`→PHASE-03 `VT-1`/`VT-2`/`VT-3`/`VT-4` and PHASE-04 `VT-4`;
  `VT-7`→PHASE-02 `VT-4`. Design ids are not plan ids.
- **Residual concerns carried from `RV-385`'s verification pass.** The payload
  contract's variant check needs a named legacy exception (PHASE-03 EX-4); no
  `Presence` variant yet expresses "required at create, optional at update, `null`
  refused" (PHASE-02 EX-4, where the compiler forces the choice); the refusal
  layer that fires first is not stated (PHASE-03 EX-2); `DEC-126`'s table row is
  committed to be amended but unamended — a `/reconcile` catch, not a phase;
  and `design.md` sec-6's VT-6 prose has a garbled sentence, noted for
  reconciliation. None gates the lock.
- **Plan-as-revision (step 7).** The critical pass moved two things into the plan
the first pass under-specified: PHASE-03's pin hazard above, and the fixture churn
that "`blocking` required at creation" forces on every node-declaring payload
fixture (PHASE-02 EX-8). Both are sequencing facts that would otherwise surface
as a red phase.
- **Closure obligations, not phases.** The slice closes `IMP-469` and `ISS-481`,
and records the `DEC-062` reconciliation on `RFC-031` T4 (slice closure intent).
Those are `/reconcile` and `/close` work: no phase criterion can own them, and
neither is evidence a phase could produce.
- **Deferred, by name** (design sec-7): `IMP-386`, `ISS-299`, `IMP-389`,
  `IMP-471`, `IDE-057`, and the finding-home sibling `ISS-482`.
