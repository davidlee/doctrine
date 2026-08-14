## The shape

A runtime artefact whose **reader** pins to a different tree root than its
**writer's neighbour** is invisible exactly when the work arrives from elsewhere.
Two roots, one directory, one caller.

The instance that taught this — `boundaries_path()` resolving
`git::primary_worktree(cwd)` while its sibling `phases_dir()` joined the local
`project_root`, so an adopted worktree reported **`10/10` phases and zero source
deltas at once** — **is fixed** (`ISS-350`, 2026-08-14). Keep reading for the half
that is not.

## Fixed: the source-delta registry

`resolve_registry_root()` in `src/state.rs`: the registry resolves **local when
the local tree already holds one, else the primary worktree**, applied identically
to read, write and evict. The registry *file* is the witness, not the slice's
state directory — that directory also holds a design snapshot, a design journal
and phase sheets, so "does the primary know this slice" is *false* for a slice
designed in the primary and built in a capsule. That was the reported case, and
it is why the obvious directory-existence predicate is wrong.

Two stderr advisories now disclose the root choice, because the *silent* choice is
what produced three findings about a registry that was never empty (`RV-356`
`F-7` → `F-14` → `F-15`):

- crossing trees on read or write names the tree it resolved to;
- writing while another live worktree also holds a registry for that slice names
  that tree — the half the resolver cannot fix, since in the primary the local
  root *is* the primary.

**Do not hand-copy `boundaries.toml` between trees any more.** The old workaround
here told you to; it now creates a second registry that wins in its own tree, and
nothing reconciles the two.

## Still true: phase sheets are local-rooted, deliberately

`phases_dir()` was left local on purpose (changing it would move dispatch's
completion semantics). So a linked worktree with **no** state of its own still
reads rows from the primary against an *empty* local completed-set, and
conformance reports `incomplete — partial coverage` rather than a clean verdict:

    doctrine slice conformance 231 -p .worktrees/SL-231-p01
    → note: source-delta registry resolved to /workspace/doctrine — …
    → SL-231: conformance incomplete — partial coverage

That reading is an artefact of the split, not a finding about the slice. The note
is what tells you so.

## Still true: runtime state does not travel with a branch

A tree reconciling a slice built elsewhere needs the slice's
`.doctrine/state/slice/<NNN>/` to arrive with the checkout (the capsule sideband
does this) — a branch alone will not carry it.

## The generalisation — the durable part

Whenever a runtime artefact is written by one tree and read by another, **the root
each side resolves is part of the contract**. `primary_worktree(cwd)` and
`project_root` are not interchangeable, and a function that mixes them is correct
only in the primary tree. The capsule workflow makes "work arrives from elsewhere"
routine, so this class gets more common, not less.

Which tree *should* own a slice's runtime state is still open: `QUE-216`, shaping
`RFC-025`. Two candidate answers (adoption into the primary as a ritual; promoting
the registry to the authored tier so it travels) cost asymmetrically, and the tree
taxonomy is mid-change.
