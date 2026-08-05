# DEC-149: An unfilled facet is marked, never papered over; the renderer assumes a healthy corpus

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The shape

```
shaped_by
  DEC-120 — Condition satisfaction is derived, attested, or claimed
    context:   …
    choice:    …
    rationale: …
  QUE-206 — <title>
    (no facet recorded · 6.7 KB prose · doctrine knowledge show QUE-206)
  CPT-001 — <title>
    (concepts carry no facet · 1.2 KB prose · doctrine knowledge show CPT-001)
```

Two markers, because they state two different facts. `QUE-206` is a record whose
ruling was never entered. `CPT-001` is a kind that has no facet to enter —
`ConceptFacet {}` is empty by construction and `format_facet` returns an empty
string for it, since *"every concept rides its attributed prose body"*
(`src/knowledge.rs:570-573`). One wording for both would make the corpus look
worse than it is, and would teach readers to skip the marker.

## Why not a prose fallback

It is the arm that sounds most helpful and behaves worst.

- **The cost stops being predictable.** `QUE-206`'s body alone is 6.7 KB —
  most of the `facets` level's entire measured budget across all fifteen
  specimen records. A level whose price depends on which records happen to be
  unfilled is not a level.
- **It launders the defect.** The reader can no longer tell a record carrying a
  structured ruling from one that does not. That is precisely the concealment
  `format_facet` performs today, wearing a different hat.

## Why not silent

`SL-246` objective 4 exists to prevent it: four of fifteen records on the
acceptance specimen would render as an id and nothing else, conveying *less*
than opening the record would.

## Building for the corpus we want

The governing instruction was to render as if the facet content problem were
already solved. A renderer shaped around today's 24% fill rate would encode the
defect and have to be undone when [[IMP-403]] lands. The marker is the honest
minimum: it states the gap, compensates for nothing, and simply stops firing
once facets are filled.

## One renderer, two policies

A second facet renderer beside `format_facet` would duplicate the per-kind
field-order tables — seven of them — which is the thing that drifts. Instead
`format_facet` takes two policy inputs:

| caller | fields | empty |
|---|---|---|
| `knowledge show` / `inspect` | all populated | silent (unchanged bytes) |
| the composed read | the `facets` subset | marked |

`SPEC-013` pins rendered output byte-exact per verb, so `knowledge show` keeping
its current policy is what keeps its goldens green. This is [[DEC-147]]'s
caps-on-one-code-path principle applied a second time — the levels cannot
disagree about field order because there is one field-order table.

It also makes [[IMP-403]]'s eventual fix cheap: when `knowledge show` should
stop concealing gaps, that is a policy flip at one call site.

## Boundary

`SL-246` does not change `knowledge show`'s output. The concealing behaviour
there is real and recorded, and it belongs to [[IMP-403]].

Shapes [[SL-246]]. Resolves that slice's `inq-5` (scope `OQ-4`, objective 4,
risk R1).
