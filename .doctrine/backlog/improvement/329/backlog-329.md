# IMP-329: Single-source the observation facet field enumeration

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised as RV-317 F-5 (code-review of SL-231 PHASE-01..03) and dispositioned
`follow-up` to here.

## The hazard

The 33 optional facet string fields are hand-enumerated in **three** independent
places. Adding or renaming a facet field requires editing all three, and missing
one fails **silently, differently, each time**:

| Site | Lines | Silent failure if a field is missed |
|---|---|---|
| `wire.rs:305` `Facets::string_values` | ~117 | lexical search stops matching that field |
| `wire.rs:540` `validate_facet_strings` | ~159 | NUL / `FACET_STRING_LIMIT` bypassed for that field |
| `commands/observation.rs:547` `merge_explicit_facets` | 173 | design §3.1 "explicit caller values win" silently inverted — enrichment overwrites an explicit value |

The three walks are also the three longest functions in the SL-231 delta, and
their length is *entirely* this repetition — there is no branching complexity
worth preserving. Fixing the enumeration fixes the length.

## Current state and why this is not urgent

Verified at review time (`0fe9572b`): the enumerations are **consistent** — a
field-by-field diff of `string_values` against `validate_facet_strings` yields an
empty symmetric difference. So this is a latent drift hazard, not a live defect,
and nothing in SL-231 PHASE-04 or PHASE-05 depends on the refactor.

It still warrants owned work: this is the same shape as SL-231 PHASE-03 defect 2,
where a hand-rolled comparator was byte-identical to the service's and was
correctly recorded as "latent drift, not a live bug" — which is exactly what it
was, right up until it wasn't. The facet set is expected to grow: design §2.3
versions each facet independently, and QUE-176 will add usage-facet sources.

## Shape

Derive all three walks from a single field enumeration — a macro listing each
facet group's string fields once, or a visitor the three call sites share. The
invariant to protect: **adding a facet field touches exactly one place.**

## Not deferred with this item

The cheap interim guard belongs in SL-231, not here: a test asserting
`string_values()` and the validated-field set have equal cardinality, so a
one-sided addition fails loudly while this item waits. RV-317 F-5's disposition
folds that single test into the cleanup turn addressing F-1.
