# CHR-060: `facet_write::FacetField` — the `FacetValue` rename target is wrong; settle the name

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

> **Rewritten at `SL-249`'s close, 2026-08-09.** This chore previously read
> *"Rename `facet_write::FacetField` → `FacetValue`"*. **Do not do that.** Its
> premise was checked against the code at close and is false; executing it as
> written would take the name `FacetValue` from the type that earns it and give
> it to one that does not. The original instruction is preserved in § *What this
> chore used to say* below, for anyone who finds it cited elsewhere.

## The premise, and why it is false

`ISS-329` was raised when `SL-249` `PHASE-03` introduced a second `FacetField`
into the crate, colliding with the pre-existing `facet_write::FacetField`. It
judged that renaming the *older* type was the more honest fix, on the grounds
that the older type is *"a facet value to write, not a field"*.

It is not. `facet_write::FacetField` carries a **key alongside its value** and
exposes a `key()` accessor:

```rust
pub(crate) enum FacetField {
    Str { key: &'static str, value: String },
    Arr { key: &'static str, values: Vec<String> },
}
```

Key + value is a field. `FacetValue` would be a *less* accurate name for it, not
a more accurate one.

## Where the name actually belongs

Three types are in play, and they are genuinely distinct:

| type | shape | what it is |
|---|---|---|
| `facet_write::FacetField` | `Str { key, value }` / `Arr { key, values }` | a field **with its value** — a pending write |
| `knowledge::FacetFieldRow` | `{ name, shape }` | a field **declaration** — no value |
| `design_run::WireFacetValue` | `Text(String)` / `List(Vec<String>)` | an **unkeyed value** |

The third is the one `FacetValue` describes exactly, and its own doc comment
(`src/design_run/submission.rs`) says it declined the name **solely because this
chore has a claim staked on it**:

> *"`CHR-060` is an open chore aimed at that name for `facet_write::FacetField`;
> taking it here would plant the very two-modules-one-name defect `ISS-329` was
> raised for, and would foreclose an open backlog item by squatting on its
> target."*

So as written, this chore is not merely wrong — it is **holding a better name
hostage** for a rename that runs backwards.

## What is actually left to settle

`ISS-329`'s underlying complaint is real: two field-ish types in one crate are
confusable, and `FacetFieldRow` was a compromise, not a settlement. Three
questions, in order:

1. **Does `facet_write::FacetField` want a different name after all?** Not
   `FacetValue`. But `FacetFieldRow` is the field *declaration* and this one is
   the field *assignment*, so `FacetEdit` or `FacetAssignment` names the
   distinction the current pair leaves implicit. `CHR-060` floated `FacetEdit`
   in passing even while arguing for `FacetValue`; that half was the sounder
   one. Call sites are the facet-setting surfaces — `risk set`, `value set`,
   `estimate set` — which is why the rename was deferred out of `SL-249` in the
   first place.
2. **Does `WireFacetValue` drop its prefix once unblocked?** Not automatically.
   The prefix has a second, independent justification the doc comment also
   gives: `WireKey` is the same-file precedent, and `WireFacetValue` is the wire
   form of `knowledge::RawValue`, which the crate already names elsewhere. That
   argument survives this chore's collapse. Decide it on its own merits, not as
   a consequence.
3. **The pointer in `submission.rs` is now stale either way.** It cites this
   chore by its old aim. Whatever is decided above, that doc comment must be
   corrected in the same change — otherwise the next reader inherits exactly the
   trap this rewrite removes.

**A defensible outcome is that nothing is renamed at all** and only (3) is
executed. That is a smaller change than the chore originally described, and it
is the one to reach for if (1) does not produce a clearly better name.

## Provenance

`RV-351` `F-7` (`SL-249`'s implementation audit) raised the contested premise
and observed that nobody reopens a chore until pickup, so a warning left only in
the slice's `design.md` § 5.2 would reach no one. Reconcile recorded the dispute
there; this rewrite settles it against the code, which was one file read away
the whole time.

Related: `mem.pattern.verification.re-derive-every-inventory-at-use` — the same
failure mode one axis over. A claim carried forward from a prior reading
(*"that type is a value"*) was wrong when re-derived against the tree, and
nothing failed in the meantime to announce it.

## What this chore used to say

> Rename `facet_write::FacetField` → `FacetValue` (or `FacetEdit`; pick at the
> time, `FacetValue` reads truest to the two variants). The deferred half of
> `ISS-329`'s ruling: the user ruled option 1 plus this item — `SL-249` renamed
> its own new type to `FacetFieldRow`, settling `PHASE-04`'s import seam inside
> the slice's scope, and the honest rename was costed separately here rather
> than smuggled into a slice scoped to the facet write seam.

`FacetFieldRow` was chosen for the new type over `FacetFieldSpec` and
`FacetFieldDecl`: `Row` is the tree's dominant suffix for a table row type (~20
precedents), `Spec` is a loaded word in doctrine because it names an entity
kind, and `Decl` is an abbreviation with essentially no precedent.
