# ISS-316: EVD/HYP/CPT lifecycle vocabularies and supersession rules are ungoverned

> **Narrowed at `SL-249`'s close, 2026-08-09 — read this first.** `REV-050`
> landed the four→seven amendment across `SPEC-019` and `PRD-010`, in **both**
> authored tiers, and `tests/governance_kind_coverage.rs` now fails if any kind
> in `kinds::RECORD` goes unnamed in either entity. So the *facet contracts* and
> the *verb set* are governed. What survives is the residue § *Narrowed scope*
> names at the foot of this item. Everything from here to that section is the
> **original 2026-08-05 statement, retained for provenance and now largely
> discharged** — do not act on it without reading the foot first.

## What

`SPEC-019` (*Knowledge-record entity surface*) is emphatically a **four-kind**
spec. Its responsibilities open with *"Bind **four** `record_kind`s (assumption,
decision, question, constraint) onto four engine `Kind`s"* (`SPEC-019:9`), and
it repeats "four" at `:12`, `:30`, `:35`, `:55`, `:139`, `:150` — including
*"Evidence is a single shared minimal support structure across all four kinds"*
and *"one verb set dispatches across all four prefixes"*.

The corpus has **seven** record kinds. `EVD`, `HYP`, and `CPT` appear in
`SPEC-019` **nowhere** — zero occurrences of the ids or the words
evidence-as-a-kind / hypothesis / concept.

Verified 2026-08-05:

```
$ doctrine spec show SPEC-019 | grep -c "EVD\|HYP\|CPT\|hypothesis\|concept"
0
```

They were added later — `src/relation.rs:552` carries the note *"SL-159
PHASE-02: EVD/HYP added so evidence/hypothesis records may be governed"* — and
the entity-surface spec never followed.

## What is actually ungoverned

The code declares typed facets for all seven (`src/knowledge.rs:495-580`), so
the *implementation* is complete; it is the spec that lags. Specifically
unspecified:

| kind | facet in code | spec'd |
|---|---|---|
| evidence (`EVD`) | `datum`, `provenance`, `confidence` | no |
| hypothesis (`HYP`) | `proposition`, `predicts` | no |
| concept (`CPT`) | none — `ConceptFacet {}`, "every concept rides its attributed prose body" (`knowledge.rs:570-573`) | no |

Also unspecified for these three: the per-kind lifecycle vocabulary, the
supersession rules, and whether `CPT`'s deliberately empty facet is a designed
property or an omission. `SPEC-019`'s *"per-kind lifecycle vocabulary"*
responsibility enumerates four.

## Why it matters

Not merely tidiness. `SL-246` hit this directly: it must decide which facet
fields constitute a bounded "deciding fields" render per kind, and for three of
seven kinds there is no governance to defer to — so any field list for them is
**invention presented as derivation**. The same hole will reappear for any
future work that needs a per-kind contract (lint legs on `IDE-009`, the
knowledge filters on `IMP-398`).

Compounding it, those three kinds are also the thinnest in evidence: `EVD` n=12,
`CPT` n=1, and **`HYP` n=0 — never used in this corpus at all**. So neither
governance nor precedent constrains them.

## Shape of the fix

A `REV` against `SPEC-019` — the spec is the `revises` target and this is
governance truth, so it routes through a Revision rather than a direct edit
(`ADR-013`). Scope: widen the four-kind framing to seven, specify the three
missing facet shapes and lifecycles, and rule explicitly on whether `CPT`'s
empty facet is intended.

Worth checking at the same time whether `PRD-010` carries the same four-kind
framing, since a spec and its parent product spec drifting together is the
usual shape (see `mem.fact.revision.spec-prose-modify-target`).

## Provenance

Found during `SL-246`'s pre-design research round (2026-08-05), verifying a
research thread's claim that there were no revision candidates. There were.

## Related

- `SL-246`: surfaced it; blocked on it only for the `EVD`/`HYP`/`CPT` field
  lists, which it will mark as invented rather than derived
- `SPEC-019`: the revision target
- `PRD-010`: likely carries the same four-kind framing — check
- `IDE-009`, `IMP-398`: future consumers of a per-kind contract

## Narrowed scope — what is still open after `SL-249`

`SL-249` absorbed the facet-contract half as its objective 4, and its Non-Goals
declared the rest out of scope by name. Three things survive.

1. **The per-kind lifecycle vocabularies for `EVD`, `HYP` and `CPT`.**
   `SPEC-019`'s *"per-kind lifecycle vocabulary"* responsibility now names seven
   kinds, but the three added kinds' status tokens and legal transitions are
   still unwritten. `knowledge settle` and `knowledge status` both dispatch on
   that vocabulary, so this is not documentation debt alone.
2. **Supersession rules for the same three kinds.**
3. **`CPT`'s deliberately empty facet — designed property or omission?**
   `ConceptFacet {}` carries the in-code note *"every concept rides its
   attributed prose body"* (`SL-197` `D2`), which reads as intentional, but no
   governance says so, so nothing stops a later change from "fixing" it.

Plus the legibility note carried over from `ISS-332`: three `SPEC-019` sections
are **knowingly** four-kind — they discuss the original four for a reason that
survives at seven — but read as stale rather than as scoped. They need a word
saying which they are. The kind-coverage canary cannot tell the difference; its
one exemption (`spec-019.md`'s *"four coupled sites"*, which counts integration
points, not kinds) is held to a count identity rather than mere presence, so a
duplicated exempt phrase cannot inherit the exemption silently.

**Provenance thinness still binds.** `HYP` has n=0 in this corpus and `CPT`
n=1, so a lifecycle vocabulary written now is invention, not derivation — the
same objection `SL-246` raised against inventing field lists. That argues for
waiting for use rather than specifying ahead of it.

Related, added at this close: `SL-249` (the slice that narrowed this),
`REV-050` (the amendment), `ISS-332` (resolved `fixed` — `SPEC-019`'s
closed-enum lists omitted `provenance`; its legibility note is the paragraph
above).
