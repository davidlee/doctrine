# Implementation Plan SL-282: Install explains itself

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Reference forms: `lib:reference/glossary.md` § reference forms. -->

## Overview

Six phases, bottom-up: the pure currency model first, then its readers, then
the install executor that writes the stamps, then the hymn changes that depend
on the stamp, then the user-facing install screen that renders everything,
then dogfood.

```
PHASE-01 model + stores ──► PHASE-02 sites + guard
        │
        └──► PHASE-03 executor ──► PHASE-04 hymns ──► PHASE-05 overview/hint ──► PHASE-06 dogfood
```

## Sequencing & Rationale

- **PHASE-01 before everything.** `install_state` is a pure leaf with no
  callers; it can be tested exhaustively in isolation, and every later phase
  reads it. The stores and the marker reader land with it so PHASE-02 and
  PHASE-03 share one reading path.
- **PHASE-02 is independent of the executor.** Its e2e fabricates marker and
  stamp files, so it does not wait for install to write them. It can run in
  parallel with PHASE-03 (file-disjoint apart from `src/boot.rs`: PHASE-02
  touches the guard and `boot`/`--check` output, PHASE-03 the `wire` split —
  run serially unless the phase sheets confirm disjoint hunks).
- **PHASE-03 before PHASE-04.** The legacy-starter gate and the customise
  refusal both key on the tracked stamp, and the RV-421 F-16 regression
  (install → customise → install) needs install to write it.
- **PHASE-03 changes execution, not prompts.** Restructuring legs into
  `LegOutcome` while the old prompts still exist keeps each phase's diff
  reviewable and the existing install suites meaningful as a
  behaviour-preservation check. PHASE-05 then swaps the screen on a stable
  executor.
- **PHASE-03 overran; PHASE-07 carries its downgrade gate.** PHASE-03's
  EX-5 (downgrade gate, RV-421 F-17) is discharged by PHASE-07, which runs
  after PHASE-03 and before PHASE-05 (the overview renders its warning).
  Array order is execution order; ids are not renumbered.
- **PHASE-04 before PHASE-05.** The overview renders legacy removal as a
  bullet and the hint lists removed starters, so the leg must exist first.
- **PHASE-06 last.** Dogfooding needs the finished binary; it is also where
  this repo's seven starters actually go.

## Notes

- RV-421 `control` findings are transcribed into exit criteria on the
  earliest satisfying phase: F-3 → PHASE-01, F-1 → PHASE-02, F-14 and F-17 →
  PHASE-03 (F-17 moved to PHASE-07), F-16 and F-18 → PHASE-04, F-10 → PHASE-05.
- The REV across SPEC-009 / SPEC-011 / SPEC-023 (DEC-385) is drafted at
  reconcile, not in a phase.
- Shared files: `src/install.rs` is touched by PHASE-01 (marker reader),
  03, 04, 05; serial order avoids conflicts.
- Revision 1 (plan self-review): PHASE-02 gains EX-6 (golden suites that run
  `prompt resolve` on marker-less fixtures will gain an advisory line; adjust
  them deliberately) and EX-7 (status calls the reader from `run_status`, the
  command-tier entry, so no new layering violation). PHASE-04 gains EX-7
  (tests seeded through `[hymns].expose` move to `prompt customise`).
- Watch: PHASE-03 is the largest phase (resolver, `wire` split, leg outcomes,
  stamps, downgrade gate). If its phase sheet exceeds one sitting, split the
  downgrade gate into an appended phase rather than renumbering.
- Interactive e2e drives the binary through piped stdin; `prompt_confirm`
  reads a plain line, so no TTY is required.
