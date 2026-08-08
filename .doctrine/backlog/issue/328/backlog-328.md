# ISS-328: An unknown key nested inside CreateRecord is dropped, not refused

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`Declaration` carries `#[serde(deny_unknown_fields)]`; `CreateRecord`
(`src/design_run/submission.rs:250`) does not. So a key misspelled or invented
inside a `form = "create"` disposition is swallowed by serde before any of the
run's own checks see it — the caller is told nothing and gets a record built from
what serde kept.

This corrects a claim `ISS-318` makes. Its observation 3 states *"Every **inner**
submission type does carry `deny_unknown_fields`; the outermost one, which is the
only one a caller hand-authors from scratch, structurally cannot."* The first
clause is false: `CreateRecord`, `AcceptanceDeclaration`, `DischargeDeclaration`,
`ReviewPolicyDeclaration` and the `Dispose` variants carry no such attribute.
`CheckpointActDeclaration` and `AgentActDeclaration` do.

Found 2026-08-08 during `SL-249` `PHASE-01`, when `CreateRecord` gained its `body`
field — before that phase, a `dispose.create.body` key was silently swallowed
rather than refused, which is stronger evidence for `SL-249`'s premise than the
slice assumed. `A2` in `SL-249`'s design is unaffected: the payload extension
needed no serde change either way.

## Why it was not fixed in passing

Adding the attribute is one line. Deciding it is not: stored proposal
declarations ride the run snapshot and so outlive the binary that wrote them
(`mem.fact.design-run.snapshot-outlives-the-binary`). A snapshot holding a
`Declaration` written by an older binary is re-read by a newer one, so tightening
a nested type converts previously-readable stored state into a parse failure at
exactly the moment someone is trying to resume. That wants a deliberate answer —
migrate, tolerate on the read path, or accept the break — not a drive-by derive.

`SL-249` `PHASE-02` reasoned about the same tradeoff for the kind axis and left
this one out under `DEC-183`.

## Related

- `ISS-318` — the class, and the observation this item corrects.
- `ISS-327` — the same class on the subject-state axis.
- `DEC-183` — the ruling that scoped both out of `SL-249` `PHASE-02`.
