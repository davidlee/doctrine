# DEC-221: Drift pin splits: enums generated, struct fields literal-pinned

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Two things to pin, and they are not alike

`DEC-219` made enum token vocabularies and serde tagging part of the contract, so
the pin has to keep two different kinds of thing honest.

A **struct's field set** is reachable from one serialised value. Construct the
type exhaustively, serialise it, take the key set, compare. That is `SL-249`'s
`I9` (`submission.rs:649`, test at `tests.rs:2900`), and it works because the
exhaustive no-`..` literal makes a newly added field a **compile error at the
pin** rather than a silent omission from the table.

An **enum's variant set** is not reachable that way. No single value carries it,
so there is no serialised artefact to compare against. The repo's honest answer
today is that there is no pin at all: `ActKind::ALL` (`attestation.rs:92`, eight
members) and `Advance::ALL` (`gate.rs:917`, four) are hand-written arrays. Their
`as_str` matches are exhaustive, so a new variant forces a token — but nothing
forces the variant into `ALL`. A tenth act would render a contract silently
omitting itself, which is precisely the failure a contract exists to prevent.

## Why not tier 2 everywhere

Because on the enum half, tier 2 has no oracle. The mechanism would degrade to a
hand-written list checked against another hand-written list — the tier-1 pattern
already excluded from this slice's scope for being blind to the failure it claims
to prevent. `SL-244`'s `condition_vocabulary!` (`gate.rs:438`) exists precisely
because its author reached this conclusion on the neighbouring axis, and its doc
comment argues explicitly against hand-written `ALL` arrays.

## Why not tier 3 everywhere

Not on principle — on blast radius. Generation has a measured cost: **clippy does
not lint tokens a macro body wrote** (`gate.rs:459`). Confined to a handful of
small closed enum definitions that is a fair price, because tier 2 buys nothing
there. Extended over the payload structs it would put most of `submission.rs`
beyond the linter, for a slice scoped as a documentation surface, and it would
displace a pin that already yields a compile error. Generation would buy lint
blindness and nothing else.

## The split follows the oracle

Each mechanism goes where its cost is lowest and where an oracle actually exists.
Neither existing precedent is displaced: `I9` keeps doing what it does well, and
`condition_vocabulary!`'s pattern extends to the enums the contract now has to
enumerate.

The consequence is that `inq-6`'s totality argument runs twice — once over the
act inventory, once over each enum's variant set. `DEC-227` settles the first;
the second is discharged by construction wherever generation reaches.

## Held loosely, on the record

The user accepted this with an explicit qualification, and it belongs in the
record rather than in a session transcript: this is type-design detail an
implementing agent is better placed to navigate, and later findings may overturn
it. Superseding this record is expected, not exceptional.

Two conditions were named as grounds to revisit without ceremony:

1. the enum set proves small and stable enough that per-enum generation costs
   more than it saves; or
2. the count of nested payload types makes per-type `fully_populated` literals
   the dominant cost.

`DEC-227` has since put the closure at roughly twelve wire-participating structs,
which is squarely the second condition. That does not overturn this record on its
own — twelve mechanical literals is a known, bounded cost — but an implementer
who finds it worse than it looks has standing here to change the mechanism rather
than to work around it.
