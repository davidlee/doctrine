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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
