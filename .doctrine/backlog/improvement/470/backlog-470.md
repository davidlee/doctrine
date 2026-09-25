# IMP-470: Inquiring runbook names no growth obligation for the inquiry map

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Nothing obliges a run to grow its map. The engine permits it on any turn and the
sparse contract supports it (`DEC-063`), and a per-turn craft lens does encourage
it — `install/design-prompts/inquiry.md` says "Add inquiry nodes for the questions
that actually shape the design" and "Add a `needs` edge only where one question
genuinely cannot be answered before another".

But that is a generic lens, not an obligation. The `inquiring` runbook's exit
conditions are exactly two acts — `inquire.knowledge` and `inquire.scope` — and
neither mentions the map. The only explicit re-inquiry nudge in the whole pack
fires at `drafting-ready`, i.e. late
(`install/design-prompts/conditions/drafting-readiness-attested.md`).

The failure mode is therefore not refusal but reading: an agent can take "add
inquiry nodes" as a set-up instruction, declare the map once, and then treat it as
finished. There is nothing in the pack that says otherwise. In the corpus, that is
the common shape — the map is largest and most connected on the runs that kept
adding to it during inquiry.

## Why this is the cheap lever

Prose only; no engine change, no state-machine change. `RFC-031`'s stated principle
is to prefer observed use over building from the backlog, and this is the cheapest
possible intervention on the observe side: upgrade the instruction and re-measure
(`CHR-065`).

## Shape

Name growth where an agent cannot miss it — a step in the `inquiring` runbook
("a new answer may reveal a node; declare it"), or a line in the every-turn
fragment stating that the map is live rather than a preamble. It is worth stating
the *cost* in the same breath, since it is currently unstated and cuts the other
way (`IMP-469`): a shape change re-faces the cumulative gates.

## Implementation note

`install/` is a RustEmbed root, so a prompt-asset edit needs a rebuild to ship, and
`doctrine install` to refresh an installed copy.

## References

- `install/design-prompts/inquiry.md:37-41,45`; `install/design-prompts/inquiring.toml`
- `ISS-299` — the map never reaches the user (the sibling visibility gap)
- `IMP-469`, `RFC-031`
