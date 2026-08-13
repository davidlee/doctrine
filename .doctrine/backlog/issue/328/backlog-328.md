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

## Second sighting, 2026-08-13 — the first one that cost something

`SL-254`'s design run, disposing two inquiry nodes with `form = "create"`. The
facet map was sent under `facets`; the field is `facet`
(`submission.rs:302`). Serde dropped it, the apply **succeeded**, and the change
log reported `checkpoint_disposed cp-11 node=inq-11 record=DEC-215
disposition=create` — the success shape, naming a real canonical id. Both
records landed with **every facet empty**. Nothing distinguished it from a
correct apply; it was found only by reading the TOML back.

This upgrades the item's evidence in two ways.

**The failure is silent at the *content* level, not just the key level.** The
2026-08-08 sighting was a near-miss found while adding a field. Here a complete,
well-formed decision — context, choice, alternatives, rationale, consequences —
was discarded wholesale while the run recorded that a durable record had been
created for it. A design run whose whole purpose is that a resolved node cannot
launder "we discussed it" into a durable claim produced exactly that: twelve
resolved nodes, twelve records, two of them hollow.

**Recovery is not re-running the apply.** No refusal covers re-disposing a
checkpoint, so a retry plausibly mints duplicate records rather than correcting
the empty ones. The facets had to be back-filled through
`knowledge edit decision`, whose list flags split on commas with no escape
(`mem.pattern.doctrine.comma-separated-facet-flags-split-prose`), so all eight
`alternatives`/`consequences` elements were rewritten comma-free to survive the
detour. One wrong character, two applies, a second verb, and eight prose
elements re-authored to route around an unrelated footgun.

Observation: `019ff90f-770a-7da0-b8fb-ca5a0a382c5c`.

Does not change the fix's shape or the constraint above — it raises the cost
side of the ledger. Worth noting for whoever costs it: a **read-back check**
(does the record that was just created carry the facets the payload sent?) would
have caught this without touching the serde read path at all, and sidesteps the
snapshot-compatibility problem entirely.

## Related

- `ISS-318` — the class, and the observation this item corrects.
- `ISS-327` — the same class on the subject-state axis.
- `ISS-346` — the discoverability half: `declaration_example` advertises no
  `dispose` form, so a caller assembles the nested payload from the source or
  from guesswork. That is how the wrong key got written in the first place.
- `DEC-183` — the ruling that scoped both out of `SL-249` `PHASE-02`.
