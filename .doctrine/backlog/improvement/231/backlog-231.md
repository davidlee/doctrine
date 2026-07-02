# IMP-231: pi-arm record-delta over-attributes source-delta: sweeps orchestrator-trailed knowledge + refresh-base merge commits

Surfaced by the SL-186 audit (RV-213 F-3, tolerated).

## Observed
`doctrine slice conformance` on a pi/subprocess-arm-driven slice lists `undeclared`
paths that were never part of the delivered code — authored entities
(`slice-NNN.toml`, `adr/**/layering.toml`), orchestrator-trailed knowledge
(memories, backlog items), and files pulled in by a `refresh-base` trunk merge.
For SL-186 the registry showed 17 undeclared while the projected candidate
(`git diff main..candidate/186/review-001`) was a clean 19-file SL-186 delta.

## Cause
On the pi/codex arm the arm's registry write is `slice record-delta <SL> PHASE-NN
--start <B> --end <B+1>`. The `B→B+1` range is a commit span, so it captures
*every* commit the orchestrator lands between the two boundaries — including the
knowledge it trails after the code commit and any interleaved `refresh-base` merge
— not just the imported source diff. The claude arm's `record-boundary` cuts a
`phase/<N>` ref at the code commit, so it does not have this problem.

## Impact
Low — attribution noise only. The delivered artifact is verified via the projected
candidate; conformance stays a "where to look" signal. But it forces the auditor to
hand-diff the candidate against trunk to separate real scope creep from the noise.

## Candidate fixes
- Record the per-phase delta against the *imported code commit's* tree only (the
  single non-merge commit the funnel produces), not the full `B→B+1` span.
- Or exclude authored/knowledge/merge commits from the recorded range on the
  subprocess arm (mirror the claude arm's code-commit-scoped `phase/<N>` cut).

Related: `.doctrine/rfc/011/case-notes.md` (SL-186-P04-conclude note).
