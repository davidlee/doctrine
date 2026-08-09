# IMP-415: Skills do not instruct an agent to fill a knowledge facet

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`IMP-403` lead 5, minted at `SL-249`'s close. `SL-249` closed leads 1 and 2 and
named 3–5 as follow-ups rather than dropping them.

## What

Unexamined, and cheap to examine: whether `/knowledge`, `/design` and
`/record-memory` actually tell an agent to fill the `[facet]` at all, and
whether the templates' seeded-empty shape (`knowledge new` scaffolds every
field present and blank) reads to an agent as *optional*.

`IMP-403`'s measurement is what makes this worth checking rather than
assuming: population is **binary and lock-step** — every record carrying one
textual field carries all of them, no record is partially filled. That is not
authors tiring halfway; it is a population that either engaged the structured
tier or never knew it was there. A guidance gap fits that distribution better
than a friction gap does.

## Why it is now worth doing

`SL-249` removed the excuse. Before it, an agent that wanted to fill a facet
had to hand-edit TOML; now `knowledge edit --facet` exists and a design-run
`form = "create"` disposition can carry the facet in the minting payload. The
skills still describe the pre-`SL-249` world. Sequence this **after** the
`SL-249` verbs are documented, not before.

## Related

`IMP-403` (parent), `IMP-413` and `IMP-414` (leads 3 and 4), `SL-249` (the
verbs the guidance should now name), `SPEC-019`.
