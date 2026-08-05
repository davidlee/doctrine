# DEC-150: The facets level carries what rules, not the argument; tiers annotate the existing field match

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The level

| kind | `facets` | excluded |
|---|---|---|
| `DEC` | context, choice, rationale | alternatives, consequences, decided_by, decided_on |
| `QUE` | question, why_matters, **answer** | answered_by, answered_on |
| `CON` | statement, source, applies_to, **waiver_reason** | waived_by, waived_on |
| `ASM` | claim, confidence, basis, **invalidated_by** | validation_plan, validated_by, validated_on, invalidated_on |
| `EVD` | datum, provenance, confidence — *all* | — |
| `HYP` | proposition, predicts — *all* | — |
| `CPT` | none — [[DEC-149]]'s by-design marker | — |

## Two exclusion arguments, one of which fails

The pre-design research round proposed nearly this list. Its exclusions rested
on two different claims:

**Bulk.** `alternatives` and `consequences` are where the size is. `DEC-080` is
the measurement — a 274-byte prose body under a facet that renders at roughly
4.5 KB, almost all of it those two fields. Dropping them is the whole reason the
level costs less than `full`. This argument holds.

**Never populated.** `answer` / `answered_by` / `answered_on` are 0 of 38 across
the corpus, so the round excluded them. Under [[DEC-149]] that reasoning is not
available: the renderer is built for a corpus whose facets are filled, and a
question that *has* been answered rendered without its answer is absurd. Same for
`waiver_reason` on a waived constraint and `invalidated_by` on a dead
assumption — each is short, each changes whether the record still binds, and
each is unpopulated today only because the corpus is thin.

So the criterion is not size. It is: **what rules, and what changes whether the
ruling still stands.** Size follows.

## `ISS-316` does not bite here

The round flagged that `SPEC-019` governs four record kinds while the corpus has
seven, so any field list for `EVD` / `HYP` / `CPT` would be invention needing to
be labelled as such. It turns out there is nothing to invent: `EVD` has three
fields, `HYP` has two, `CPT` has none, and every one of those fields is a
deciding field. The honest subset is *all of them*.

The governance gap is real and stays on [[ISS-316]]. It just does not block this
question.

## Why not a field-list constant

`STD-001` points at a single-source named constant, and the obvious shape is a
per-kind list of field names. That table would sit beside `format_facet`'s
existing per-kind match, which already encodes the field order — two structures
describing the same seven field sets, free to drift. A field added to the match
and not the list would render in `knowledge show` and silently vanish from the
composed read.

Annotating each field line in the match that already exists gives one table, one
order, one place. It also makes classification unavoidable: a new field cannot
be added without saying which tier it is in.

Shapes [[SL-246]]. Resolves that slice's `inq-6` (scope `OQ-2`). Downstream of
[[DEC-149]]; the encoding is [[DEC-147]]'s one-code-path principle again.
