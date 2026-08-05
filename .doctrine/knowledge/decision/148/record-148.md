# DEC-148: Knowledge selection filters on the source's kind, not on the relation label

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question was framed as label curation

`SL-246`'s `OQ-3` asks which inbound labels qualify, and its risk R2 counts the
noise: 11 of `SL-244`'s 28 inbound edges are `references(originates_from)` from
backlog items. Running the command shows why that framing misleads:

```
$ doctrine inspect SL-244
inbound:
  originated from: IDE-047, IMP-391, … ISS-315      ← 11 backlog items
  concerned by:    DEC-140, DEC-141, DEC-142, DEC-144, EVD-012
  shaped_by:       DEC-120, … DEC-139, EVD-012, QUE-206
  reviews:         RV-344                            ← a review
```

Neither noisy group has a knowledge record as its source. A label allow-list
would exclude them, but only because those labels happen not to be record
labels — the right result reached by the wrong rule.

## Filtering by source kind

`shaped_by`, `concerned by`, `spawned_by`, `supported_by`, `disputed_by` and
`superseded by` are, by construction, the inbound names of the labels a record
can point outward with. So "source prefix ∈ `kinds::RECORD`" selects exactly
that set — the closed vocabulary comes along for free rather than being
restated.

It also stays right where a two-label list quietly wouldn't:

| subject | inbound label | what it carries |
|---|---|---|
| backlog item | `spawned_by` | the `QUE` that produced it |
| knowledge record | `supported_by` / `disputed_by` | its evidence |
| knowledge record | `superseded by` | what replaced it |

All three are knowledge shaping that entity. None is `shapes` or `concerns`.

## And it leaves nothing to drift

`kinds::RECORD` is the canonical constant, but roughly seventeen sites already
spell `"ASM" | "DEC" | "QUE" | "CON"` as literals instead of reading it
([[mem.pattern.doctrine.record-kind-touch-sites]], high severity — adding a kind
is a grep task, not a checklist task). An allow-list written as literals would
be the eighteenth site. A membership test against `kinds::RECORD` is none.

## One argument, named and rejected

`concerned by` contributed exactly the three decisions on the specimen that
render nothing at the `facets` level — `DEC-140`, `DEC-141`, `DEC-142`. Dropping
that label would make the output look better. `DEC-144` is also `concerned by`
and is substantive, so the label is not the problem: curating it out to dodge a
rendering defect fixes the wrong thing. The empty-facet question is `inq-5`, and
it gets fixed there.

Shapes [[SL-246]]. Resolves that slice's `inq-4` (scope `OQ-3`, risk R2).
