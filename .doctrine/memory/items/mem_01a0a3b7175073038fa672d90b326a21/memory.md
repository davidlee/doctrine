When code says *"pinned by `X` below, so a future `Y` fails here"*, that sentence
is a **claim about `X`**, not documentation. Open `X` and check it asserts the
claim. Very often it asserts something weaker that happened to be sufficient the
day it was written.

## The instance (`RV-367` `F-4`, `SL-259` `PHASE-03`)

`contract_check::walk_enum`'s untagged arm walks a value against every untagged
variant's payload. Its comment:

> every untagged variant in the closure is a `Shape` **carrying no keys**
> (pinned by `every_untagged_variant_is_a_shape` below, so a future untagged
> variant with a key surface fails here rather than slipping past unwalked)

The pin asserts `matches!(variant.payload, VariantPayload::Shape(_))`. But
`Shape` holds a `WireType`, and `WireType::Named(&TypeContract)` is a legal
inhabitant — a shape that *does* carry a key surface. So the "carrying no keys"
half is unpinned, and the arm is unsound without it. The comment also names the
wrong failure mode: the consequence is a **false refusal** of a sibling
variant's legitimate value, not an unwalked pass-through.

## Why this is worth a memory rather than a one-off

`RV-366` (reconciliation) raised nine findings on `SL-259` and **all nine** were
the design's prose lagging the tree. `RV-367` (code-review) then read the doc
comments *written in that slice* against the code beside them and found four of
six to be the identical defect one layer down — a cited pin that does not pin
(`F-4`), a helper widened with a stated rationale the same commit violates
(`F-5`), a disjointness argument true per-revision and false across revisions
(`F-1`), a tracked issue whose premise the code had already invalidated (`F-3`).

Two reviews, two layers, one failure mode. In a codebase that argues for itself
in prose this densely, **the prose is where the defects are**, and it is
cheap to check: the citation names the artefact, so the check is one jump.

## The move

For each `SL-NNN`-era comment that cites a guard:

1. Jump to the cited artefact.
2. State the comment's claim as a predicate.
3. Ask whether the artefact's assertion **implies** that predicate, or merely
   coincides with it on today's inhabitants. Coincidence is the common answer,
   and it is a finding.

Sibling: [[mem.pattern.doctrine.conformance-measure-must-match-the-claim]] is
the authoring-side twin — a canary's *measure* must be as strict as its claim.
This is the reviewing-side one: a *citation* must be as strong as its sentence.
Also [[mem_019fa18161f47651af7687d8dccbbc67]] — a negative grep result is
untrustworthy without a positive control, the same "green for the wrong reason"
family.
