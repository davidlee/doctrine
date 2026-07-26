# Implementation Plan SL-231: Friction observation ledger and capture interface

The plan cuts the collection primitive as five independently green increments.
It deliberately leaves reporting and aggregation to a later slice.

## Sequencing and rationale

PHASE-01 establishes the pure contract first. The wire, correction projection,
and query semantics carry most of the product invariants and can be attacked
without filesystem or adapter noise. The new top-level module is classified in
ADR-001 at the same time so layering never spends a phase in an ambiguous state.

PHASE-02 isolates the shared-machinery risk. The entity engine's existing safe
parent walk is extracted into `fsutil`, the complete-content no-clobber
publication primitive is added, and the entity suite supplies behaviour-
preservation evidence before the observation store depends on the new seam.

PHASE-03 makes the trusted CLI usable end to end and lands the exhaustive
worker-mode classification in the same command-enum increment. All policy stays
in the shared observation service; the command layer adapts arguments and
renders receipts, records, diagnostics, and projections.

PHASE-04 crosses the confinement boundary only after the core and CLI contracts
are proven. MCP root resolution, friction-only authority, worker-role
conformance, doctor fixtures, and the shipped Claude worker grant land together
so the bounded capability cannot exist without its ceiling.

PHASE-05 distributes the narrow temporary-file ignore rule and activates
dogfooding last. Until both capture routes and worker refusal diagnostics are
green, governance continues to point at the historical case-note mechanism.
Activation retains that historical corpus; it does not migrate or delete it.
Because project governance is boot input, the phase also regenerates the
runtime boot snapshot and verifies it is current.

## Scope guards

The production measurement-source registry remains empty pending QUE-176.
IMP-319 owns subprocess-worker capture parity, IMP-320 owns opt-in boot
instruction, and IDE-005 owns future harness detection. Reporting,
aggregation, retention, consumer cursors, chronological symlink views, and
numbered promotion remain outside SL-231.

The plan's review pass strengthened two load-bearing checks: PHASE-02 now binds
the shared parent-walk extraction to the existing entity rollback/materialise
tests and an explicit macOS/Linux portability review; PHASE-05 binds governance
activation to boot regeneration.

Observations are authored and review-visible by default. Projects may ignore
the authoritative record tree, or developers may exclude it locally, but the
documentation must state that this trades away shared durability, correlation,
and audit history unless another transport replaces Git.
