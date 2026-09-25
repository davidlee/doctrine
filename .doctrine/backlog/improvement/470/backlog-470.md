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

**Decided (2026-09-25): the every-turn fragment, not the runbook.** The `inquiring`
runbook is the wrong home, by the pack's own authoring rule (`DEC-104`, `RV-325`
`F-5`, stated at the head of `inquiring.toml`): a step is residue that *completes at
the boundary*, and "keep the map live" has no truthful completion point. Adding it
as a step would also restate the obligation and make every discharge of it stale,
deliberately — a real cost for no gain. Guidance with no completion point is a
lens, and the lens is delivered every turn of both stages.

So: one bullet added to the map paragraph in the `inquiry.md` fragment, stating
that the map is declared as it is discovered — an answer that reveals a question
adds a node, an exposed dependency adds an edge, a mooted node is disposed of
rather than left open — and saying why it matters (an undeclared question is
invisible to the user whose acceptance the run asks for).

Deliberately **not** stated in the prose: what a shape change *costs*. That is
`IMP-469`'s subject and `SL-264` will change it; writing the current cost into a
shipped lens would bake in a fact about to move.

## Implementation note

`install/` is a RustEmbed root, so a prompt-asset edit needs a rebuild to ship, and
`doctrine install` to refresh an installed copy.

## References

- `install/design-prompts/inquiry.md:37-41,45`; `install/design-prompts/inquiring.toml`
- `ISS-299` — the map never reaches the user (the sibling visibility gap)
- `IMP-469`, `RFC-031`
