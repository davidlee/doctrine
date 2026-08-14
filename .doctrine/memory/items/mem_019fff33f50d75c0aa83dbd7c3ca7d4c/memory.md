## The shape

A runtime artefact whose **reader** pins to a different tree root than its
**writer's neighbour** is invisible exactly when the work arrives from elsewhere.

Concretely, in `src/state.rs`:

- `boundaries_path()` (`:943`) resolves `git::primary_worktree(cwd)` on read
  **and** on write — it always points at the primary tree.
- its sibling `phases_dir()` (`:135`) joins the local `project_root`.
- `registry_completeness()` (`:1208`) consumes both.

So in an adopted or capsule-delivered worktree the same function reports
**`10/10` phases and zero source deltas at once**. Two roots, one directory, one
caller.

## Why it bites at audit

`slice conformance` goes quiet rather than wrong. There is no error: the registry
reads as *empty*, which an auditor naturally interprets as "no phase recorded a
source delta" — a finding about the slice. It is not. It is a finding about which
tree the reader was pointed at.

`SL-254`'s audit mis-diagnosed this **twice** before getting it right (`RV-356`
`F-7` -> `F-14` -> `F-15`). The registry was never empty: all ten phases recorded
automatically with `provenance = "solo"`, and the capsule sideband
(`refs/capsule/a/state/implementation`) delivered them into the audit worktree —
where the reader never looked.

## Check before concluding

When conformance reports an empty or partial registry in **any tree that is not
the primary**, verify the file before believing the reading:

    ls .doctrine/state/slice/<N>/boundaries.toml     # in THIS tree
    ls <primary>/.doctrine/state/slice/<N>/boundaries.toml

If the local tree has it and the primary does not, the reading is an artefact.
Copy it across to make `slice conformance` resolve — and note that it is runtime
tier, so **it does not travel with a branch**: anyone re-running conformance on
another tree must copy it again.

## The generalisation

The capsule workflow makes "work arrives from elsewhere" routine, so this class
gets more common, not less. Whenever a runtime artefact is written by one tree
and read by another, the root each side resolves is part of the contract —
`primary_worktree(cwd)` and `project_root` are not interchangeable, and a
function that mixes them is correct only in the primary tree.

## Fix

`ISS-350` owns the resolver decision (three directions recorded, plus the
primary-pinned-**write** hazard, which is the sharper half: a fork writing its
registry mutates the primary tree's file).
