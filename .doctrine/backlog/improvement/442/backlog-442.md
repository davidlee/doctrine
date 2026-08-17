# IMP-442: Evidence records cannot state what they deliberately do not prove

`SPEC-019` gives the `EVD` record a typed facet of `datum` / `provenance` /
`confidence`. There is nowhere to record **which direction was asserted**, or
**what the run deliberately left open** — so an evidence corpus cannot say what it
does not cover, and its gaps are invisible by construction.

Two disciplines, both carried up from oubliette's probe harness into its `ADR-003`
clause 4, where they are stated as applying to any evidence record in either
corpus:

- **Assert both directions.** A record that establishes only the denial, or only
  the success, says which half it has and does not claim the other. *A
  denial-only test passes for the wrong reason* — the probe harness takes an
  expectation (`check <name> <ok|deny> <cmd…>`) rather than a bare command for
  exactly this. A refusal observed without the matching admission is one
  observation, not two.
- **Name what is not established.** The neighbouring question the run refuses, and
  why refusing it is right. `oubliette:EVD-008` was written this way before the
  rule and is the shape.

## Why it is separate from ADR-022

`ADR-022` (*Evidence ownership between peer corpora*) decides which **corpus**
mints a record. These two disciplines are about a record's **quality**, apply
equally to a corpus with no peer at all, and would be a `SPEC-019` facet change
routed through a `REV`. `ADR-022`'s *Neutral* section states the exclusion and
names this item's subject; bundling them would have made that ADR about two
things. Oubliette bundled them because both fell out of one reading of its probe
documentation, not because they share a subject.

## Open

Whether *not established* is a **required** field is the live question, and
`oubliette:ADR-003` argues against: a required field gets filled with "nothing"
the first time it is inconvenient, which is worse than an absent one. It left the
clause unenforced by choice. The same argument probably holds here, which would
make this a facet plus a convention in `/knowledge`, not a validator.

Not backfilled either way — the line is added to an older record when it is next
touched for another reason.
