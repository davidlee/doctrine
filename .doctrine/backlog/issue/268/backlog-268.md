# ISS-268: Boundary row spanning a mid-drive refresh-base attributes trunk to the phase

Surfaced at the SL-231 conclude cadence.

## Observed

`doctrine slice conformance 231` reports **45 undeclared** cells, of which 42 are
trunk content that SL-231 never touched — `.doctrine/slice/232|233|234/**`,
`.doctrine/backlog/**`, `.doctrine/review/**`, `Cargo.lock`, `Cargo.toml`, the
plugin manifests, `src/review.rs`, `tests/e2e_*`. The honest touched set is 28
files with three undeclared (`.doctrine/dispatch/231/funnel.toml`,
`.doctrine/slice/231/slice-231.toml`, `src/worktree/allowlist.rs`).

## Cause

PHASE-01's boundary row is `[095fca404, 02da8ebf47]`, recorded at its conclude
(`addc6d178`) — long-standing, not a mis-derive at `prepare-review`. A
`refresh-base` merge (`0d2cb5671`, 20 trunk commits) landed *inside* that range:
the orchestrator captured `B` pre-refresh, ran `refresh-base`, then spawned and
imported. `conformance_outcome` (`src/slice.rs:2876`) folds each row's
`start..end` `--name-status`, so every file the merge brought in reads as that
phase's delta.

The true post-refresh PHASE-01 delta is 7 files
(`git diff --name-only 0d2cb5671..02da8ebf47`), all of them declared bar the
funnel file.

## Relation to IMP-231 / SL-189

IMP-231 fixed this class on the pi/subprocess arm and asserts the claude arm "cuts
a `phase/<N>` ref at the code commit, so it does not have this problem". That is
true of the **projected refs** but not of the **recorded row**: `slice
conformance` reads the row, not the ref. So the residual is arm-independent —
any `refresh-base` between `B` capture and conclude invalidates `B` as a delta
start, whichever arm recorded it.

## Candidate fixes

- Re-base the open phases' recorded `code_start_oid` at `refresh-base` time (the
  merge commit becomes the new start), so a mid-drive refresh cannot straddle a
  row.
- Or have `conformance_outcome` walk `start..end` with `--first-parent` / skip
  merge commits, so a merged range contributes only the phase's own commits.
- Or compute the row's delta from the imported code commit's own diff (mirror
  what IMP-231 proposed for the pi arm), which is immune to whatever else lands
  in the span.

The first is cheapest and fixes the data; the second fixes every already-recorded
row retroactively.

## Impact

Attribution noise only — the delivered artifact is verified via the projected
candidate. But it costs the auditor a multi-step investigation (read
`conformance_outcome`, five per-range diffs, a history walk over
`boundaries.toml`) before the report can be trusted, and until disproved it reads
as a serious scope violation. Related: IMP-282, IMP-292, ISS-224.
