# Entity reads carry their knowledge records

## Context

The managed design workflow (`PRD-019`) moved the substance of slice design into
knowledge records. A design no longer argues and concludes in one document — it
argues in `design.md` and *rules* in `DEC` records. Nothing renders the two
together, so the ruling and the argument for it can only be read apart.

`SL-244` is the specimen. Its `design.md` is **3,456 lines** and cites **20
distinct `DEC` records by id**, quoting fragments of them inline (`DEC-121`
appears 8+ times as quoted phrases). Fifteen knowledge records point at the slice
— twelve via `shapes` (ten `DEC`, plus `EVD-012` and `QUE-206`) and three `DEC`
via `references(concerns)`. `doctrine slice show SL-244` renders **none** of them:
it emits outbound relations only, as does every other kind's `show`.

The derivation already exists. `doctrine relation list --target SL-244` returns
the inbound edges correctly. It lives in a third command family, reachable from
neither the entity nor the knowledge record, so nothing leads an agent to it from
where the question is actually asked. A substantial part of this change is
routing to an existing capability rather than building a new one.

Originates from `IMP-398` (knowledge record discoverability), whose remaining
candidates stay on the card.

## Scope & Objectives

**The composed read.** An entity rendered together with the *content* of the
knowledge records that point at it — not a list of ids, and not a graph. The
subject is any entity kind that carries knowledge relationships (`SL`, `SPEC`,
`PRD`, `ADR`, `RFC`), with the slice/design case as the motivating and acceptance
specimen.

**Relation-keyed, one hop.** The record set is derived from inbound relation
edges (`ADR-004`: storage is outbound-only, reciprocity is derived). Prose
citations are explicitly *not* consulted — see Non-Goals.

**A verbosity dial.** Three levels, defaulting to today's behaviour:

| level | renders |
|---|---|
| skip | no knowledge (the current, default read) |
| facets | the deciding fields only — a `DEC`'s `choice` + `rationale`, a `QUE`'s `question` + `why_matters` |
| full | complete record bodies |

The middle level is the one that earns the feature. Measured on the specimen it
costs **~30% of `full`** (31.8 KB vs 107 KB across the fifteen records) — a real
saving, but not the order of magnitude first assumed; see `research/research.md`
§ *The specimen, re-measured*.

**Objectives**

1. An entity read can carry its inbound knowledge records' content.
2. The verbosity dial is explicit and defaults to no change in existing output.
3. The inbound derivation is factored so a later transitive closure (`IMP-398`
   S5 — the recursive knowledge view) extends it rather than replaces it.
4. Empty facets are legible as gaps, not as absence of content.

## Non-Goals

- **No prose-citation parsing on the read path.** `SL-244`'s `design.md` cites
  ~10 `DEC` records it holds no edge to (`DEC-063` `shapes` `SL-233`; likewise
  `DEC-065/066/067`, `-073/074`, `-086/088`, `-101/102`) — inherited governing
  context, cited but unlinked. Closing that gap is a *validate-and-warn* concern
  on the authoring path, deferred to its own backlog item. This slice reads
  edges; it does not scan prose.
- **Not the recursive knowledge closure.** Transitive walking through
  knowledge→knowledge edges, halting at non-knowledge nodes, is `IMP-398` S5.
  This slice keeps the seam open for it (objective 3) and stops at one hop.
- **Not the remaining `IMP-398` candidates** — `knowledge list --kind`,
  `--related-to`, the `KN` kind alias, repeatable `graph --label`, graph focus
  anchoring. All stay on the card.
- **Not a corpus facet-hygiene pass.** Empty facets are rendered honestly, not
  backfilled.

## Affected surface

Seeded as coarse `scope-relevant` selectors; the exact touch-set is `/design`'s.

- `src/knowledge.rs` — record read model, facet access, `list`/`show` seam
- `src/relation.rs` — the relation vocabulary and inbound derivation
- `src/commands/relation.rs` — where `relation list --target` already derives it
- `src/catalog/**` — hydration and the key-indexed read model
- `src/slice.rs`, `src/spec.rs`, `src/adr.rs` — the per-kind `show` renders
- `src/commands/design.rs` — the design-run read surface

## Governing context

- `ADR-004` — relations stored outbound-only; reciprocity is derived. Fixes the
  derivation as a reverse scan, not a stored back-edge.
- `SPEC-018` — cross-corpus relation contract; the label vocabulary and roles.
- `SPEC-019` — knowledge-record entity surface; record kinds and typed facets.
- `SPEC-013` — CLI surface; flag grammar and listing conformance.
- `PRD-019` — managed design workflow; why design content lives in records.
- `PRD-010` — epistemic and governance records.

## Risks, assumptions, open questions

**R1 — the facet tier is routinely empty.** The `facets` level depends on
structured fields that authors habitually leave blank while writing the whole
record as prose (`mem.pattern.doctrine.amend-knowledge-both-tiers`; observed on
`DEC-099`, `ASM-007`, `QUE-201`). A middle level that renders blanks on real
records is worthless. Objective 4 is the mitigation, and the feature may need to
name the gap loudly enough to drive hygiene.

**R2 — inbound noise swamps the signal.** `SL-244` carries 28 inbound edges, of
which **11 are `references(originates_from)` from backlog items** — harvest
exhaust, not design content. An uncurated inbound render makes the common read
worse. Which labels qualify is `OQ-3`.

**R3 — token cost.** `full` on `SL-244` is 3,456 lines plus sixteen records.
Bounded by construction (`skip` is the default), but the dial's levels must be
worth their price on every axis the project weighs.

**A1** — the inbound edge set is a sufficient proxy for "the knowledge that
shapes this entity", accepting the ~10 cited-but-unlinked records as a known,
separately-tracked miss.

**OQ-1 — surface.** A flag on the existing `<kind> show` / design read, or a
distinct kind-agnostic verb (`knowledge digest <ref>`)? Determines whether this
is one seam or one per kind.

**OQ-2 — facet selection per record kind.** Which fields constitute the
`facets` level for each of the seven record kinds.

**OQ-3 — label curation.** Which inbound labels qualify: `shapes` and
`references(concerns)` clearly; `references(originates_from)` clearly not.
Allow-list, grouping, or caller-controlled.

**OQ-4 — empty-facet rendering.** Visible gap marker versus silent blank
versus fallback to prose.

**OQ-5 — closure seam shape.** How far to factor the inbound derivation now so
S5's typed traversal extends it (objective 3) without speculative generality.

**OQ-6 — the composition seam** (added post-research). `CatalogEntity`
(`src/catalog/hydrate.rs:111-113`) carries identity and body but **no facet**, so
the inbound derivation `relation list --target` already performs cannot render
facets as-is. Either `CatalogEntity` grows a `RecordFacet`, or each composing
`show` calls `knowledge::read_record` per inbound id. The load-bearing
architectural choice.

**Post-research reordering.** `OQ-4` is primary and `OQ-2` is largely settled by
evidence — governance declares no field tiering and the corpus populates facets
as an all-or-nothing unit (35/35/35 on decisions), so field selection is
low-stakes and gap handling is the whole design. See
`research/research.md` § *Design-input deltas*.

## Verification / closure intent

`SL-244` is the acceptance specimen: reading its design with the dial at
`facets` must surface the twelve `shapes` decisions' rulings without the 11
`originates_from` backlog rows, and must do so at a cost a working agent would
actually pay. Closure additionally requires the existing kind-`show` suites green
unchanged (the behaviour-preservation gate on shared machinery), and the default
read byte-identical to today's.

## Summary

## Follow-Ups
