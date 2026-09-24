# Implementation Plan SL-261: Design adopt verb

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Six phases take the authored-watermark crossing from the `adopt_authored`
payload key to the `doctrine design adopt` verb (`DEC-279`), retire the key
through a new retired wire-key roster (`DEC-278`), and bring the spec, stage
guidance and memories in line. Each phase ends green.

## Sequencing & Rationale

The order builds the new crossing beside the old one and deletes the old one
only once nothing needs it.

- **PHASE-01, roster first.** The roster is independent of the verb and small.
  Landing it first, empty, proves the mechanism (identity match, refusal,
  rendering, pins) with a test-local roster before any key depends on it. It
  also carries the one piece of contract-walk surgery (`PAYLOAD` to `static`,
  walk by reference) that `RV-374` `F-5` required.
- **PHASE-02, a pure refactor.** Splitting `run_apply` is the riskiest change:
  it is the most ordering-sensitive code in the subsystem. Isolating it as a
  behaviour-preserving phase makes the existing suites the proof, with no test
  edits allowed (VA-1). The wire `adopt_authored` keeps working, mapped onto
  `Crossing::Adopt`, so nothing downstream changes yet. The one-read rule
  (`RV-374` `F-1`) lands here because it is a property of the pipeline's
  adoption input, not of the verb.
- **PHASE-03, the verb.** Built on the split. Two crossings coexist for the
  length of this phase and the next; that is scaffolding inside the slice, not
  a shipped parallel implementation.
- **PHASE-04, the diff.** Separated so the new dependency lands on its own and
  the verb's core behaviour is settled first.
- **PHASE-05, retirement.** Deletion, the roster row, refusal wording, and the
  test migration. After this there is one crossing. It comes after the verb and
  diff so the migrated parser-readout probes can use `adopt --dry-run`.
- **PHASE-06, governance and guidance.** Last, so the spec revision and the
  memories describe landed behaviour.

## Notes

- `RV-374` carries no instrument-routed findings (`F-1` owner-fix, `F-2`
  review, the rest minor), so no criterion transcription from the ledger is
  owed.
- `ISS-477` (ordinary mutations on a locked run) and `IDE-056` (post-audit
  regression guard) are out of scope.
