# IMP-257: Dispatch mid-drive authored-commit guard

## Problem

The dispatch funnel binds `worktree-base == --base == coord-HEAD` rigidly.
Any authored correction committed to the coordination branch between `arm-spawn`
and `import` advances HEAD off the worker's fork base, causing
`verify-worker: wrong-base` → forced re-fork with a full re-dispatch.

Witnessed 3 times, ~276k tokens in avoidable re-dispatches:
- SL-193-P01: F5 design-note commit on `dispatch/193` advanced coord HEAD →
  re-dispatch PHASE-01 (~85k tok)
- SL-193-P02: `slice selector add` mid-drive advanced HEAD again →
  re-dispatch PHASE-02 (~106k tok)
- SL-191-P02: selector-add mid-drive considered triggering a re-fork (caught by
  memory check, detour saved ~40k)

Root cause: the dispatch skills don't warn that an authored commit mid-batch
strands the in-flight worker. The funnel cadence says "commit after import" but
doesn't prohibit interleaved authored commits — and corrections (design
amendments, selector fixes) are a natural mid-drive need.

## Fix direction

- **Skill-level guard**: `/dispatch` skill adds a red-flag: "all authored
  corrections (design amendments, selector fixes) MUST land before arm-spawn.
  An authored commit between arm-spawn and import will strand the in-flight
  worker and force a full re-dispatch."
- **Funnel-level check**: `dispatch arm-spawn` records current HEAD as
  `arm_base`; a pre-import check (`dispatch verify-worker` already checks
  `merge-base --is-ancestor B HEAD`) could be augmented with a warning when
  HEAD has advanced beyond the arm base BEFORE the worker returns.
- **Stretch — re-anchor tolerance**: `import --allow-reanchor` (IMP-043)
  would 3-way merge onto a moved HEAD when the delta is path-disjoint from the
  intermediate commits. This is the structural fix, but IMP-043 is currently
  empty-bodied and unplanned.

## Related

- RFC-011 case-notes: SL-193, SL-191-P02
- IMP-043 (import re-anchor — the structural fix)
- IMP-256 (selector completeness — prevents the most common driver of mid-drive
  selector-adds)
- IMP-233 (arm-spawn correct-root targeting — adjacent base-correctness fix)
