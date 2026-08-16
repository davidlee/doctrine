# IMP-437: Emit seam is bypassable: act storage sinks are pub(crate)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`SL-256` unifies act recording onto one `admit_and_record` seam so that storing
an act and reporting it on the change log are one operation. `DEC-238` originally
claimed that made the emission non-forgettable — *"a future third record kind
cannot be added without going through it."*

That is false, and `RV-360` `F-1` (external adversarial pass) established it.
`CheckpointActGroup::record` (`src/design_run/snapshot.rs:361`) and
`AgentDeclarationGroup::record` (`:387`) are `pub(crate)`, so the storage sinks
stay reachable from anywhere in the `design_run` module tree. Wrapping them in
`ActRecord::insert` stops *using* the direct route; it does not close it. A
future production caller can still store an act and emit no row — which is the
exact regression the seam was built to prevent.

`DEC-238` now carries an appended correction stating the weaker true property:
three production paths collapsed to one, and no arm in which emission is
optional. It is a convention held by there being one obvious route, not a
guarantee held by the type system.

## Why it was not fixed in `SL-256`

Restricting those methods' visibility breaks the eight existing direct callers
in `src/design_run/fixture.rs` and `src/design_run/tests.rs`. The first is
outside `SL-256`'s selectors; the second is one of `SL-251`'s design-targets. So
the fix needs its own scope.

## Shape of the repair

Make the storage capability reachable only through an API that also returns the
mandatory row. Candidates worth weighing rather than a settled direction:

- a private module owning the groups, with `admit_and_record` as its only
  public surface;
- a witness type that only the seam can construct, required by `record`;
- leaving it as convention and closing this item, if the risk stays theoretical.

Whichever is chosen, `fixture.rs` and `tests.rs` need a sanctioned construction
route so the test suites do not force the sink back open.
