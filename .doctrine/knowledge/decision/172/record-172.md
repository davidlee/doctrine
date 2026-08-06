# DEC-172: Concept records carry no facet by design

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The answer already existed

`src/knowledge.rs` documents `ConceptFacet` as *"currently empty (D2, unit
struct)"*. That `D2` is a citation — it names a doc-local decision in SL-197's
design, the slice that added the concept kind. SL-197 is closed, so the decision
is accepted, not provisional.

D2 reads:

> **empty `ConceptFacet`** (unit-like struct, `#[derive(Default)]`, **no
> fields**) — intentional, a first in the `RecordFacet` enum, not an oversight.
> IMP-244's sections (Definition, Notes, Distinguish from, Examples, Related)
> are **prose in the md body**. A structured `definition` field would duplicate
> the md Definition section (two sources of truth, drift).

So ISS-316's question — designed or omitted? — has an answer with a reason. What
it lacked was a *home*: the ruling sat in one slice's design, where a reader of
SPEC-019 would never find it.

## Corroboration

- `validate_facet`'s concept arm is `RecordKind::Concept => { let _ = raw;
  RecordFacet::Concept(ConceptFacet::default()) }` — raw facet input for a `CPT`
  is discarded deliberately, not by omission.
- `install/templates/knowledge-concept.toml` seeds a bare `[facet]` header with
  no fields, present only for the scaffold-order invariant that pins
  meta→`[facet]`→`[evidence]`→`[relationships]`.
- DEC-149 already draws the distinction this ruling settles, on the render side:
  two markers, one for *unfilled*, one for *no facet by design*.

## The wording that caused the confusion

Both the code doc and the template say *"currently empty"*. D2 says
*intentional*. Those disagree, and the hedged one is what a reader meets first —
which is why ISS-316 raised the question and why SL-249 carried it as `OQ-4`
into design.

The REV should retire the hedge with the ambiguity. Elevating the ruling while
leaving "currently" in place would fix the governance and keep the trap.

## What this is not

This does not settle concept's lifecycle vocabulary or its supersession rules.
Those stay ungoverned and stay on ISS-316, per SL-249's settled OQ-5.
