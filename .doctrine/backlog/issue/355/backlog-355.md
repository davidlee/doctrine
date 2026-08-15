# ISS-355: Successful agent_declaration prints no change row

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

A successful `agent_declaration` apply (e.g. declaring the blocking set) returns
exit 0 and prints only `revision N stage <stage>` — **zero event rows, zero
change-log entries**. Confirming the declaration actually landed requires
grepping raw runtime state, which the guardrails tell agents not to read.

## Why it matters

The success output is **byte-identical to the silent-drop failure mode** that
`ISS-346` (*design apply silently absorbs unknown payload keys*) describes. An
agent that mistypes an act name and an agent that submits a correct one see the
same thing. So the one observable an agent could use to distinguish "applied" from
"absorbed" is absent exactly where it is most needed, and the two defects
compound: `ISS-346` makes the wrong payload succeed, this makes the right payload
look like it failed.

Observed three times in the sweep, twice on consecutive days, each time costing a
raw-state grep to resolve.

## Evidence

Friction observations (`.doctrine/observations/records/`):

- `019ff4a1-088f` — `agent_declaration` lands with no event row / change-log entry
- `019ff675-5c42` — apply emits no event row (logged explicitly as the silent-drop twin)
- `019ffb44-aa1f` — successful `agent_declaration` prints no change row (reads as discard)

## Scope note

Distinct from `IMP-390` (*Envelope reports state, not what to do next*). `IMP-390`
is about the envelope not naming the **next** act; this is about the response to a
**completed** act not confirming it happened. Fixing `IMP-390` would not emit this
row.

## References

- `ISS-346`, `ISS-333` — the silent-absorb half of the pair
- `IMP-390` — envelope contract discovery
- `QUE-219` — submission strictness against snapshot forward-compatibility
