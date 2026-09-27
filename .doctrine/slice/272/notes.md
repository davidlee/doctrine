# Notes SL-272: Apply truth residue

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (2026-09-27)

Evidence: `research/research.md` (runtime tier; ✓ rows load-bearing).

### Open questions (become inquiry nodes)

- `ISS-488` row payload — term-free, prose, or digest pair?
- `ISS-488` emission condition — any declared `question`, or only a changed text?
- `ISS-454` — suppress the displacement row (code) or document the limit (prose)?
- `ISS-482` — new refusal variant, or reuse `BlockingJudgementWithdrawn`?
- `IMP-499` — the remedy wording for `SubmissionExpired`.

### Shaping decisions (proposed)

- `ISS-488` → term-free `node_question_changed`, only on changed text, none at
  creation. Constrained by `DEC-237`, no hashing in the engine, and the
  widest-payload exemplar.
- `ISS-454` → code fix per `RV-367` `F-1`: gate the row on `live_acts_before`.
- `ISS-482` → new finding-home refusal; the inquiry text's remedy is false there.

### Risks

- A new change-log event read by an older binary shows as one unreadable row
  (`DEC-249`), not a refused run. Accepted.
- Threading `live_acts_before` widens two private signatures; no public surface.

### Assumptions

- `REQ-478` supersedes the `SL-233`-era "state, not delta" comment at the
  re-word site (`run.rs:1467`). No governance record says otherwise.

### Constraining governance

`REQ-478`, `DEC-237`, `DEC-239`, `DEC-249`, `DEC-301`, `STD-001`, `ADR-001`.
Not applicable, with reasons: research Thread 1.

## Further review (after `RV-405` repairs, design rev 23)

A second pass would probe only what the repairs introduced: that
`Batch::validate` is the single seam every finding declaration crosses (direct,
proposal rehearsal, accept); that the `reportable` set cannot over-report when
a declare coverage-kills an act recorded earlier in the same apply (it cannot:
declarations run before acts); and that `Presence::OptionalNullRefused` is not
a parallel state to an existing one. All three are checkable by `VT-1`, `VT-3`
and `VT-6` at implementation; no further design pass is recommended.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-27 · design/reviewing rev 21 · a62fcb2db

### Produced

- `DEC-335`, `DEC-336` (accepted) · `RV-405` (design review, F-1..F-4 fix-now)
- `IMP-499` (filed pre-slice, fulfilled here)

### Learned

- `DEC-336`'s predicate is incomplete: one apply can record `DesignAccepted`
  twice (`RV-405` `F-3`) — the fix may amend `DEC-336`.
- `RV-389` `F-18`'s premise (a finding's `null` reads as absent, so proposals
  may drop it) is what `ISS-482` overturns (`RV-405` `F-1`).

### Open

- `RV-405` `F-1`..`F-4` — design repairs owed before section review and lock.
