# ISS-329: Two FacetField types in one crate

Surfaced by SL-249 PHASE-03 (T1's red) — rustc's own recovery hint offered
`use crate::facet_write::FacetField` for an unresolved `FacetField`.

## What

Two unrelated types now share the name `FacetField`:

- `src/facet_write.rs:56` — an **enum**, a facet *value to write*
  (`Str { key, value }` / `Arr { key, values }`), consumed by `set_facet_mixed`.
  Pre-existing.
- `src/knowledge.rs` — a **struct**, a facet field's *declaration*
  (`name: &'static str`, `shape: FieldShape`), the row type of `facet_fields`.
  Added by SL-249 PHASE-03, named verbatim from the slice's design § 5.1.

Different modules, so it compiles and no criterion fails. PHASE-03 implemented
the design as written rather than renaming unilaterally.

## Why it matters

SL-249 PHASE-04's `plan_facet_edits` derives the write payload from the
declaration table — it reads `knowledge::facet_fields` and emits
`facet_write::FacetField`. That is one function importing both types under one
name, which forces a rename-on-import or a path qualification at the exact seam
where the distinction matters most. STD-002 (naming conventions) and the "naming
things well is VERY important" standard both bite here.

## Options

1. Rename the new declaration type — e.g. `FacetFieldSpec`, `FacetFieldDecl`.
   Cheapest; diverges from the design's verbatim declaration, so it wants a
   design amendment or a reconciliation note.
2. Rename the older write-payload enum — e.g. `FacetValue` / `FacetEdit`. More
   honest (it is a value, not a field) but touches `risk set` / `value set` /
   `estimate set` call sites, all outside this slice's scope.
3. Leave both and qualify at the seam. No code churn; the ambiguity is permanent
   and PHASE-04 pays it every time.

Recommendation is (2) on the merits and (1) on cost. Ruling owed before PHASE-04
lands its write seam.

## Ruling (user, 2026-08-08): option 1, plus a backlog item for option 2

The new declaration type is renamed and `facet_write::FacetField` is left
untouched. The honest rename of the older enum is costed separately as
`CHR-060`, rather than smuggling three unrelated command surfaces (`risk set`,
`value set`, `estimate set`) into a slice scoped to the facet write seam.

**The new name is `FacetFieldRow`.** `Row` is this tree's dominant suffix for a
table row type (~20 precedents — `ListRow`, `PhaseRow`, `SpecRow`, `SliceRow`),
and the type is literally the row type of `facet_fields`. `Spec` was rejected as
a loaded word — it names an entity kind in doctrine — and `Decl` as an
abbreviation with essentially no precedent in the crate.

Consequence, recorded in `SL-249`'s notes under *Open*: design § 5.1 names the
type `FacetField` verbatim, so **a design amendment is owed at reconcile**. It
rides the `PHASE-07` REV alongside the `D8a` and `I10` wording corrections.
