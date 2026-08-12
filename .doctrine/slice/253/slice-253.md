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
6. **The same `REV` revises `REV-051`'s criterion-3 disposition** (owner's
   direction, 2026-08-12). `REV-051` is `done` and applied; it records
   `REQ-459` criterion 3 as *"discharged structurally — one suite parameterised
   by backend; a second backend passing it edits nothing."* `DEC-189` contradicts
   that: if row membership is a function of the mechanism's available deltas,
   there is no single parameterised suite for a second backend to pass, and the
   structural discharge does not hold. Shipping the kernel makes an applied
   revision's recorded reading false, so the correction rides this slice rather
   than being left for a reader to notice.

### Affected surface

- `crates/doctrine-control/src/conformance.rs` — the split's subject. The
  AND-reduction is `admission` at `:5166`, with one production call site
  (`:5326`); the verdict types are `:2682-2838`.
- `crates/doctrine-control/src/main.rs` — `run_backend`, `run_backend_verify`,
  `admit` (`:153`), `render_verdict`, `render_outcome` (`:217`), and the exit
  constants (`:60,63`). `admit` and `render_outcome` are the only two production
  consumers of the collapsed scalar, and are exactly what `DEC-191` changes.
- `crates/doctrine-control/src/backend.rs` — `BackendId` at `:811`. It is a
  kernel type sitting in `leaf` tier while `conformance` is `engine`, so the
  kernel's dependency on it is a **new `ADR-001` edge to check**, not a given.
- `.doctrine/spec/tech/030/` and `REQ-459` — via the `REV`, not by direct edit.
- `.doctrine/adr/001/layering.toml` — `:257` carries the literal `backend verify`
  and is the *only* accepted-governance file in `DEC-194`'s rename radius; the
  tier classifications at `:261`/`:264` need revisiting after the split.
- `justfile` — `capsule-check` (`:111-113`) and `capsule-verify` (`:130-146`).
  Neither is wired into `check` or `gate`, so this slice's proof does not run
  under the default gate and verification design must say how it is run.

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

- **`OQ-1` — Firecracker row set: implement or derive? — RESOLVED narrow**
  (owner, 2026-08-12). Derive the membership as a design artefact the kernel is
  shaped to admit; implement nothing against a backend that does not exist.
  There is no Firecracker backend in `doctrine-control` — the microVM work is in
  `/workspace/microvm-spike` and has not graduated. The wide reading would have
  pulled a backend implementation in and roughly tripled the slice. Retained
  here rather than deleted because the reasoning binds later phases: the kernel
  must be *shaped* by `DEC-189`'s membership analysis without *implementing* it.
- **`OQ-2` — where does the admission floor sit** once the AND-reduction is
  gone? **Promoted from open question to blocking design decision**, and it is
  the first thing design should settle rather than the last. It is not
  hypothetical: `EVD-021` records that the vacuous path already exists, is
  asserted by a test, and is held shut only by a guard `DEC-189` dissolves.
  Likely wants its own decision record.

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
