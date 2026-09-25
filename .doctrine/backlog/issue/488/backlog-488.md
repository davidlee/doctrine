# ISS-488: A question re-word emits no change row

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

Declaring `{subject: inq-2, question: "<new text>"}` on an existing node records
the new question but emits **no** material-change row naming the re-word. The
only row the apply printed was the consequence, `act_invalidated
cpa-sufficiency-accepted`. `ChangeEvent` (`src/design_run/change_log.rs`) has no
question-changed member; its closed vocabulary comes from `SL-233`'s
projection-bounds sketch §(d).

Found by `RV-390` `F-15` (the `SL-264` audit), replaying on a scratch copy of
`SL-264`'s own locked run.

## Why it matters now

`SL-259` (*Truthful apply*) and `REQ-478` want every mutation the run records to
be reported. `SL-264` made a re-word **load-bearing**: under `DEC-301` re-wording
a covered node re-faces the user's acts. A reader of the change log sees the act
voided without the edit that voided it.

## Neighbours

- `IDE-057` — whether a traversal-only apply owes a change row (same question,
  different field).
- `ISS-450` (resolved) — the needs-at-creation row gap; the same class.
