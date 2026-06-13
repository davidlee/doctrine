# IMP-049: Agent-facing support (skill + memory + docs) for how/when to relate entities structurally

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The structural cross-corpus relation surface exists (SL-048), but nothing teaches
an agent **when** to author a relation, **which** label is legal for a given
source→target, or **how** the tier-1 `[[relation]]` rows differ from prose
relations. Evidence: while scoping SL-057 the agent defaulted to prose relations
("no structural surface in v1") because the scaffold and skills still imply that.

Wanted: the legal `(source, label)` vocabulary surfaced where agents look (a
`/canon`-reachable reference and/or a skill step in `/slice`/`/design`), a memory
recording the relate-vs-prose boundary and the label set, and guidance that
relations are authored structurally now. Pairs with IMP-048 (the write verb) and
ISS-009 (the stale scaffold). Companion to the storage-rule guidance. Surfaced
while scoping SL-057.
