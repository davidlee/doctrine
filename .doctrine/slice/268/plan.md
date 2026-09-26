# Implementation Plan SL-268: Review ledger v2

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Ten phases. The first two change no behaviour: a golden (a fixed record of
today's exact CLI output) pins the review surface, then `src/review.rs` is split
into its engine-tier and command-tier halves. The next six each land one of the
design's behaviour changes, and each updates the golden in the same commit, so
the golden's diff is the evidence of what changed. The last two bring the
shipped guidance and the governance record in line with what landed.

The locked design (`design.md`) is the binding reference for every phase. Design
section ids (`sec-N`) and the numbered test list in its Verification section
(`sec-8 VT n`) are cited from the criteria.

## Sequencing & Rationale

1. **PHASE-01 golden, PHASE-02 split.** The split is the one change with no
   observable effect, so it goes first, while the golden and the unchanged suites
   can prove that. Every later phase then edits smaller files in their final
   homes. The 97 in-file unit tests move to `src/review/tests.rs` verbatim. The
   command tier can import both halves, so no test needs rewriting to follow the
   split. New engine tests go in `#[cfg(test)]` modules of the engine files they
   cover.
2. **PHASE-03 fail-safe reads.** This changes readers only. Landing it before the
   writer changes means later phases already read their own output fail-safe.
   It also settles `Vocab<T>` and `vocabulary_defects` before PHASE-05 adds two
   more closed vocabularies.
3. **PHASE-04 journal, then PHASE-05 new acts.** PHASE-04 folds `Verb` and
   `TurnAct` into one `Act` and gives every existing act its turn row. PHASE-05
   then adds `amend` and `reopen` onto that single table, plus the closed
   disposition and route vocabularies. Split this way, neither phase is too
   large to review.
4. **PHASE-06 uniform `done`.** This needs the journal (conclude writes a
   review-level turn) and `reopen` (which clears `concluded`). Most of the
   pre-existing test flips land here, and EX-6 names the allowed ones up front
   (RV-396 `F-1`).
5. **PHASE-07 write ergonomics.** `-` / `@path` resolution goes last among the
   verb phases, so it covers every prose flag (`--note`, `--basis`, `--response`)
   in its final form, once.
6. **PHASE-08 prime.** Prime is independent of PHASE-03 to PHASE-07 and confined
   to `src/review/prime.rs`, so it may run in parallel with them after PHASE-02.
   It is placed here for serial execution.
7. **PHASE-09 guidance and the bounded review.** The guidance describes the
   finished binary, and the bounded review checks the guidance against it, so
   this waits for every code phase.
8. **PHASE-10 governance.** The tech spec describes what landed, with live
   anchors (DEC-321), so it comes after the code. The REV is drafted here and
   applied at reconcile.

## Notes

- **Execution mode.** The golden's write-path tests skip under the worker marker
  (the ISS-260 pattern), so a confined dispatch worker cannot prove PHASE-01 or
  any golden update. The coordinator-tree run is authoritative: whoever
  executes a phase, the coordinator runs `doctrine check gate` on the landed
  tree before concluding it. PHASE-01 and PHASE-02 are best executed in the main
  tree (solo or capsule), not via dispatch.
- **PHASE-02 size.** The split moves about 6 000 lines in one phase by design (R1:
  no behaviour change and no schema edit in the same diff). Splitting the moved
  test module further is out of scope. Record it as a follow-up if it hurts.
- **Test flips.** Any pre-existing assertion change not named in the phase's
  exit criteria is a finding, not a fix (design sec-8).
- **Closure items** (ISS-314, ISS-366, IMP-029, the fulfilled items) and the two
  memory updates (design sec-6) happen at `/close`, not in a phase.
- **VT test names are a contract.** Each VT mandate's keywords are the test
  function names the phase must write. `/phase-plan` may add tests, but not
  rename these.
