# CHR-060: Rename facet_write::FacetField to FacetValue

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

The deferred half of `ISS-329`'s ruling. `SL-249` `PHASE-03` introduced a second
`FacetField` into the crate — a declaration struct, the row type of
`knowledge::facet_fields` — colliding with the pre-existing
`facet_write::FacetField` enum at `src/facet_write.rs:56`.

`ISS-329` offered three ways out and judged option 2 — renaming the *older*
type — the more honest one, because that type is a facet **value to write**
(`Str { key, value }` / `Arr { key, values }`), not a field. The user ruled
option 1 plus this item: `SL-249` renamed its own new type to `FacetFieldRow`,
settling `PHASE-04`'s import seam inside the slice's scope, and the honest
rename is costed separately here rather than smuggled into a slice scoped to
the facet write seam.

## What

Rename `facet_write::FacetField` → `FacetValue` (or `FacetEdit`; pick at the
time, `FacetValue` reads truest to the two variants).

## Why it is not free

The call sites are the facet-setting command surfaces — `risk set`,
`value set`, `estimate set` — all outside `SL-249`'s scope. That is the whole
reason it was deferred, so scope it honestly rather than as a one-line rename.

## Read this before acting on the rename (`SL-249` close, 2026-08-09)

**The premise is contested, and `SL-249` `design.md` § 5.2 is its only carrier.**
`ISS-329` argued the older type is *a value to write, not a field*, which is the
entire justification for renaming it to `FacetValue`. `SL-249` found the
opposite: `facet_write::FacetField` carries **key + value** (`Str { key, value }`
/ `Arr { key, values }`), so it really is a *field*, and the better claim to the
name `FacetValue` belongs to an **unkeyed** value type. `PHASE-06` then shipped
exactly such a type — `WireFacetValue`, unkeyed, forced separate by layering
(`RawValue` lives in `knowledge`, command tier; `design_run` is `leaf, out=0`).

`RV-351` `F-7` raised this precisely because nobody reopens a chore until
pickup: if the rename's argument is dropped and only the type name survives,
whoever picks this up executes a rename whose premise may be wrong. Reconcile
recorded the dispute in `design.md` § 5.2 rather than settling it.

Read `doctrine slice show 249` § 5.2 before renaming. Settle the premise first;
the rename may be the wrong move, or may want a different target name.

## Note

`FacetFieldRow` was chosen for the new type over `FacetFieldSpec` and
`FacetFieldDecl`: `Row` is the tree's dominant suffix for a table row type
(~20 precedents), `Spec` is a loaded word in doctrine because it names an
entity kind, and `Decl` is an abbreviation with essentially no precedent. Once
this chore lands, `FacetFieldRow` could revert to `FacetField` — but it need
not, and `Row` remains the more descriptive name of the two.
