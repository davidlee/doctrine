# Conformance verdict kernel

## Context

`crates/doctrine-control/src/conformance.rs` is 14,252 lines in one file. It is
the capsule property suite `SL-248` landed, and it works: `RV-352` records two
independent gate runs at `297 passed; 0 failed`, with `backend verify` exiting 0
and all fourteen properties and five axes `Proven`.

The problem is not correctness, it is **cost of opinion**. `RSK-231` asks
whether the conformance surface can be reduced so that correctness is checkable
without loading the whole design; today forming a defensible opinion about one
row requires loading the production backend, the fixture, the placement
validator, and several phases of `SL-248`'s authored plan.

Four accepted decisions converge on this slice, and none of them can land alone:

- **`DEC-190`** splits the file into a small backend-neutral **verdict kernel**
  and a per-mechanism **payload layer**. Owner's direction: *a few hundred lines
  of policy and a bonfire for the remaining 13,500*.
- **`DEC-191`** stops the final AND-reduction. Assurance is a vector over
  escape fronts, not a rank and not a word; the verdict publishes the vector.
  `AdmissionVerdict` already carries `rows: Vec<(RowId, RowVerdict)>` — the
  vector is already in the type, and only the collapse to `Admitted` is wrong.
  Authority stays an invariant floor (`ADR-020`'s territory, unamended).
- **`DEC-194`** renames the mechanism axis to **qualification**, because
  *admission*, *verify* and *conform* each already name a different event on
  `ADR-020`'s work transaction. Its consequences require the rename to land
  *with* this split, since these types move anyway — otherwise it is a second
  migration.
- **`DEC-189`** refuses to port row *membership* across mechanisms. Four of the
  fourteen rows lose their host-facing delta under a hypervisor boundary; a
  microVM earns rows bubblewrap never could. Porting membership would rebuild
  `ISS-341`'s defect family on a new mechanism.

**`DEC-190`'s sequencing gate is clear.** It says *"do not extract before
`QUE-211` settles — a tier-blind kernel is a kernel rewritten."* `QUE-211` is
`answered`: `DEC-191` refused both of its candidates and put a per-front vector
in place of a rank. The central type's shape is therefore known before the
extraction starts, which is the precondition that was missing.

## Scope & Objectives

1. **Extract the verdict kernel.** Backend-neutral types and discipline:
   `Property` / `Axis` (the taxonomy, not its membership), `RowVerdict` and the
   distinctions between its variants, the verdict types, the four evidential
   tiers (rows, axes, `Claim`/`AuxOutcome`, `Unrowed`/`Reading`), `DEC-156`'s
   one-property-removed control discipline, and `BackendId`'s deliberately open
   construction.
2. **Publish the vector instead of collapsing it.** Replace the all-or-nothing
   AND over a fixed row set with a per-front outcome a reader sees before any
   summary. Fronts are **escape** fronts and the rendering must say so
   (`CPT-002`), so a strong profile is not read as a strong safety claim.
   Authority remains a floor: a composition that weakens it is not a weaker
   posture, it is not a capsule.
3. **Apply `DEC-194`'s rename** across the extracted surface —
   `QualificationVerdict`, `Qualification::{Qualified, Disqualified}`, the verb
   `backend qualify`, exits `EXIT_QUALIFIED` / `EXIT_DISQUALIFIED`. This closes
   the existing `EXIT_REFUSED` / `NotAdmitted` mismatch in passing.
4. **Isolate the bubblewrap payload** behind the kernel's seam, unmigrated and
   unported, so what is namespace-shaped is visibly namespace-shaped.
5. **Carry the `REV` against `REQ-459`** as a phase of this slice. `REQ-459`
   enumerates one undifferentiated property list with canonical-authority inside
   it, which is exactly the conflation the kernel un-conflates; shipping the code
   without the revision leaves the spec contradicting the binary. `IMP-405`'s
   platform-versus-mechanism rename folds in, and `CPT-002`'s threat priority
   lands in `SPEC-030` § Overview or § Concerns.

### Affected surface

- `crates/doctrine-control/src/conformance.rs` — the split's subject.
- `crates/doctrine-control/src/main.rs` — `run_backend`, `run_backend_verify`,
  `admit`, `render_verdict`, `render_outcome`, and the exit constants.
- `.doctrine/spec/tech/030/` — via the `REV`, not by direct edit.
- `justfile` — `capsule-check`, which is how this crate is reached at all.

### Risks and assumptions

- **The behaviour-preservation gate and `DEC-191` pull against each other.**
  `AGENTS.md` requires the existing suites to stay green *unchanged* when shared
  machinery moves, and `DEC-190` names `RV-352`'s baseline as the bar. But
  `DEC-191` deliberately changes what the verdict renders. The reconciliation is
  that **row verdicts** must be identical and the **reduction and rendering**
  are what changes — that distinction has to hold explicitly at design time or
  the gate will be either falsely red or quietly weakened.
- **"Not ranked" must not become "nothing can fail."** `DEC-191` leaves the
  admission floor unset, and `RFC-025` flags this directly: that would be
  `ISS-341`'s defect family a fourth time. The floor is a design decision this
  slice must take, not inherit.
- `crates/doctrine-control` is outside every default gate selection — Linux-only,
  live-`bwrap` rows — and is reached by `just capsule-check`, not `just gate`.
  Verification design must account for that or this slice's proof does not run.
- Assumes `ADR-020` is not reopened. `DEC-191` was constructed to land without
  doing so, and step 2's *"same authority properties"* sits entirely on the floor.

### Open questions

- **`OQ-1` — does this slice implement the Firecracker row set, or derive it?**
  `DEC-189` says which four bubblewrap rows lose their delta under a hypervisor
  and names four a microVM earns. But there is no Firecracker backend in
  `doctrine-control`; the microVM work lives in `/workspace/microvm-spike` and
  has not graduated. The narrow reading — derive the membership as a design
  artefact the kernel is shaped to admit, and implement nothing against a
  backend that does not exist — keeps this slice inside the measured comfortable
  band. The wide reading pulls a backend implementation in and roughly triples
  it. Scoped narrow here, pending the owner.
- **`OQ-2` — where does the admission floor sit** once the AND-reduction is
  gone? Named as a risk above; it is a decision, and it may want its own record.

### Verification and closure intent

Done is: the kernel is separable and reviewable without loading a confinement
mechanism; `RV-352`'s row-level baseline reproduces unchanged while the verdict
publishes a per-front vector; `backend qualify` replaces `backend verify` with
its exits renamed; the bubblewrap payload is behind the seam and unported; and
`SPEC-030` no longer contradicts the binary.

## Non-Goals

- **A Firecracker or microVM backend.** Not built here; see `OQ-1`.
- **Migrating the bubblewrap payload to a second mechanism.** `DEC-190` refuses
  it and `DEC-189` explains why porting membership would be actively harmful.
- **A ranking or scoring apparatus.** `DEC-191` refused a rank as firmly as it
  refused equivalence. The fix is to stop collapsing, not to build an ordering.
- **Reopening `ADR-020`.** Authority is the floor and stays where it is.
- **The anti-heresy work `CPT-002` argues is under-weighted.** Named in the
  spec via the `REV` so a reader is not misled; building it is not this slice.
- **`DEC-191`'s front list as a closed enumeration.** It is a sketch. Closing it
  is separate work.

## Summary

Split the 14k-line capsule conformance suite into a small backend-neutral
verdict kernel and an unported bubblewrap payload; stop collapsing the per-front
vector the verdict already carries; rename the mechanism axis to qualification
while those types are moving; and revise `SPEC-030` `REQ-459` so the spec stops
conflating the authority floor with confinement strength.

## Follow-Ups

- The Firecracker row set's implementation, once a backend exists (`OQ-1`).
- `DEC-191`'s front list closed from a sketch into an enumeration.
- `IDE-050` — definition records for bounded-context terms, the general remedy
  for the collision `DEC-194` fixed one instance of.
