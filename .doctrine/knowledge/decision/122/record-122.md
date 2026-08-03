# DEC-122: Condition contracts ship as embedded prose declared fixed

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The choice

A `Condition`'s contract — what the edge requires, what act discharges it, what
the remedy is when it does not hold — lives as **prose keyed off the closed
`Condition` vocabulary**, in the design-prompt store, alongside `Fragment` and
`RunbookKey`. Shipped from the embed, `customization` declared `fixed` with a
citation to `IMP-372`.

Nothing is narrowed into the closed set: the vocabulary is the **key** and the
contract is the **value**, a total function out of ten known members. `DEC-101`'s
type error is not in reach of this shape.

## Why not Rust-side data

An associated const or a table beside `boundary_conditions` is the house pattern
for keying static data off this enum, and it buys totality as a *compile* error
rather than a test. It was rejected on the slice's own acceptance test.

That test is: an agent can learn what a transition requires **without reading
`src/**` or the test suite**. Rust-side data satisfies it only after something
renders it, so prose is the shorter path to the actual goal — and the goal is
sharper still in an installed client project, which has no source to read at all.
Two supporting reasons: the contract stops `Condition` being a fieldless
`Copy`/`Ord`/serde enum and taxes every match site; and `DEC-077` already weighed
the recompile-per-sentence cost when it split prose from mechanism.

`DEC-121`'s two interactions settle it. Their contracts are several sentences
each about who does what and what the artefact is. That is prose in any world;
putting it in a const only means prose living somewhere awkward.

## Totality is solved, not novel

`the_writer_act_table_covers_every_key_writer_act_checks`
(`tests/e2e_design_delegation.rs:395-400`) builds a `BTreeSet` from each of two
enumerations and asserts equality — *"a seventh act fails here instead of quietly
widening the class."* `DEC-101` cites this as its own template. Applied to
`Condition`, a contract-less condition is a test failure rather than a silent gap.

## Forward directions this must not foreclose — and must not pre-build

Two were named when the choice was made. Neither is in scope; both are reasons to
keep the shape honest rather than reasons to add joints now.

1. **Structure extracted from the prose, with CLI surface over it.** Parts of a
   contract may later want to be records the CLI can interact with directly rather
   than paragraphs an agent relays — possibly with prompt fragments guiding that
   interaction. Captured as `IDE-047`.
2. **User hook-in points.** A project wanting to add its own qualifications or
   emphases at arbitrary points, rather than replace a whole asset. `IMP-375`
   (project extension interface for design-prompts assets) is the home; `IMP-372`
   owns the general resolution mechanism beneath it.

The governing instruction, in the user's words: *get the core right for Doctrine
with one eye on extensibility, rather than try to build everything out of knees.*
So: do not add extension points, hook registries, or partial-override machinery to
this slice. Do avoid shapes that would have to be demolished to add them — chiefly,
keep the contract addressable per condition rather than as one monolithic
document, since that granularity is what both directions need and it costs nothing
now.

## Left open

Whether the *machine-relevant* part of a contract (which `DEC-120` kind, which act
discharges it, what the refusal cites) is a small const beside the prose rather
than parsed out of it. Two homes split on who reads it may be right, or may be how
a fourth prose system arrives unannounced. `SL-244`'s `inq-5` owns it.

Related: `DEC-120` (the kinds a contract must state), `DEC-121` (the interactions
whose contracts these are), `DEC-101` (key-vs-value; the totality template),
`DEC-077` (prose/mechanism split), `DEC-102` (seal vs craft; the override seam it
deferred), `IMP-372`, `IMP-375`, `IDE-047`.
