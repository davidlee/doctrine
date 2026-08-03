# IDE-047: Structured records extracted from condition-contract prose

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`DEC-122` puts a `Condition`'s contract in prose keyed off the closed vocabulary.
Prose is the right first home: it is what an agent reads, it needs no seam, and
the contracts `DEC-121` specifies are narrative by nature.

The forward direction, named when that choice was made and **deliberately not
built**: parts of a contract may later want to be **structured records the CLI
interacts with directly**, rather than paragraphs an agent relays and can garble
in transmission — possibly with prompt fragments guiding that interaction, the way
runbook steps guide a discharge.

The move is extraction, not replacement: prose stays the human-facing register,
and structure is lifted out of it where a consumer needs to query rather than
read. Which parts want lifting is the open question and should be answered by an
actual consumer wanting them, not in advance.

`DEC-122`'s only concession to this is shape, not machinery: contracts stay
addressable **per condition** rather than living in one monolithic document,
because that granularity is what extraction needs and it costs nothing now.

Related: `DEC-122`, `DEC-120`, `IMP-375` (user hook-in points — the sibling
direction), `IMP-372` (the override seam beneath both), `SL-244`.
