# IMP-525: Returning to design after lock re-faces every gate

## The pain point

Going back to design once a run has locked — or late in a run, after the human
gates are cleared — costs nearly as much as the first pass. The user reports it
as the main reason variances discovered at planning or execution do not get
written back into the design (2026-11-08, during `SL-287`). The details are
mostly captured in existing items; this item names the user-facing pain and
links them, so it can be scoped as one change.

## What makes it expensive (linked items)

- **No return path.** `Stage::Locked` has no outbound edge (the `/design`
  skill's Residue table), so "return to design" is undefined; `ISS-477` is the
  inverse defect — a locked run accepts `apply` mutations anyway.
- **Gate invalidation on shape change.** `RFC-031` friction 6: adding a node
  after the map is accepted voids `graph-reviewed` / `sufficiency-accepted` and
  re-faces the human. `IMP-469` (resolved) fixed growth for some attestations;
  `IMP-471` (intent unwritten) is the rest. Observed live in `SL-287` rev 19:
  one added inquiry voided `graph-reviewed`.
- **Lock is many commands.** `IMP-464` — locking takes ~eleven commands; every
  re-lock pays it again.
- **Prose edits are whole-section.** `RFC-031` friction 3, `IMP-430`.
- **Reopening loses state.** `ISS-303` (reopening an inquiry discards its
  disposition); `IMP-386` (no cascade staleness from a changed answer, gated on
  `QUE-218`).
- **No exit back to work.** `IMP-412` — a locked run has no handover exit.

## Shape of a fix (not decided)

A deliberate *revise* entry from `locked` that keeps prior attestations live
except over what changed, and re-locks in one step. Consumer: `SL-287`'s
commitment-ledger `intent`-class variances (`DEC-420`), which route back to
design.
