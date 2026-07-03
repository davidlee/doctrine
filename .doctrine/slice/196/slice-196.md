# Per-edge relation descriptor

## Context

Descends from **IMP-244 part 2** (part 1 — the `concept`/CPT knowledge kind —
is handled off-backlog on a separate track; CPT chosen over recycling the
SL-160-retired `CON` prefix, so the two are independent).

Doctrine's relation vocabulary (`RelationLabel`, ADR-010) is a **closed** set of
structural labels, refined by a **closed** role dimension on `references`
(ADR-016). Concept-map / ontology-like usage wants lightweight human-readable
nuance on an edge — "contrasts with story-point estimation", "frames estimation
as attention burden" — without minting a new formal label or role for each
shade. Minting labels for nuance would bloat the closed vocabulary and defeat
ADR-016's structure/intent split (a new *label* is only for a change in
structure/graph-effect/inbound-semantics; a new *role* only for intent). A
descriptor is neither: it is **free text with no graph meaning**.

This rides the existing taxonomy cleanly: the relation model already separates
closed semantic classes (label + role) from runtime rules, and treats the
taxonomy as *descriptive*, not the source of runtime truth. A descriptor adds a
third, **non-semantic** attribute alongside label and role — inert to validation,
inference, graph effects, and edge identity.

## Driver

The motivating use case: a **concept record (`CPT`, IMP-244 part 1 / SL-197)
saying annotated things about entities** —
`link CPT-001 references SL-128 --role concerns --descriptor "frames estimation as
attention burden"`. The `descriptor` is the statement; `references:concerns`
("aboutness/relevance, source → any numbered") is the carrier.

## Scope & Objectives

Add an **optional, free-text `descriptor: String`** facet to the
**`references:concerns`** relation edge, riding the existing `Degree` facet seam
(SL-176). Source-agnostic — legal for every source the rule already admits (SL,
backlog, records, and `CPT` once SL-197 wires it).

1. **Data model** — a `Tier::One` `[[relation]]` row may carry an optional
   `descriptor` cell. Free text only: stored, round-tripped, searchable,
   renderable. **Excluded from edge identity** — identity stays the
   `(label, role, target)` triple, exactly as `degree` is excluded. Never an input
   to validation, inference, graph traversal/effects, or reciprocity.

2. **Admissibility (per-`(source,label,role)` row column)** — a
   `descriptor_bearing: bool` column on `RelationRule` (mirroring `degree_bearing`),
   `true` on exactly the `references:concerns` row. Reject (not ignore) on any
   non-bearing row — the established house rule (`validate_link` bails, symmetric
   to `degree`/`role`, `relation.rs:1447`).

3. **Write seam** — `doctrine link` gains `--descriptor <text>`; threaded through
   `validate_link` (gate) → `append_edge(…, descriptor)` → the row builder,
   mirroring `degree` end-to-end. Append-conflict: reject a differing descriptor on
   the same edge (degree precedent); `unlink`+relink is the change path.

4. **Surfacing** — descriptor renders on the **outbound** edge in `inspect`/`show`;
   descriptor text is included in the search index (via `CatalogEdge.descriptor` +
   a source-join in the search doc-build).

## Out of scope (adversarial-pass F0)

IMP-244 also listed `contextualizes`, `related`, `interactions`. All excluded:
- **`contextualizes`** — `CM`-source, authored via concept-map **DSL** lines, a
  separate write path `link`/`append_edge` never touches. Descriptor there needs a
  DSL grammar change → **follow-up**.
- **`related`, `interactions`** — **symmetric** edges (`inbound_name == label`); a
  directed descriptor is a category error. `interactions` keeps its free-text
  `notes` (untouched); `related` gets nothing (YAGNI).

## Non-Goals

- **No new label or role.** Closed `RelationLabel` / role vocabularies unchanged
  (ADR-016 preserved).
- **No `notes`→`descriptor` migration / no `--notes` rename.** `interactions`
  keeps `notes` untouched.
- **No descriptor on symmetric, structural, or DSL-authored edges.**
- **No CPT source-set wiring** — SL-197 must add `CPT` to the `references:concerns`
  `sources` array (it assumes RECORD auto-inheritance, but the set is
  hand-enumerated). Flagged to SL-197.
- **No graph semantics.** Descriptor never affects validation, inference,
  traversal, priority, reciprocity (ADR-004), or dedup/identity.
- **Part 1 (concept/CPT kind)** — SL-197.

## Summary

One optional free-text cell on the `Tier::One` `references:concerns` `[[relation]]`
row, riding the proven `Degree` seam, inert to all graph semantics, source-agnostic.
Renders outbound; searchable via the hydrated `CatalogEdge`. Purely additive — no
migration, no `--notes` rename, no template change. Behaviour-preservation gate:
existing relation validation, storage-tier behaviour, and graph effects stay green
unchanged (IMP-244 acceptance).

## Follow-Ups

- `contextualizes` descriptor via concept-map DSL grammar.
- `interactions.notes` search indexing.
- `related` symmetric `notes` — if demand arises.
- SL-197: add `CPT` to `references:concerns` sources (driver dependency).
