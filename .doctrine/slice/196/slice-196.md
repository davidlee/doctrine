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

## Directionality principle

Free-text edge annotation is named by the edge's **directedness**, read off the
model's own `inbound_name` (the derived reciprocal spelling):
`inbound_name == label.name()` ⟺ **symmetric** edge; otherwise **directed**.

- **`descriptor`** *refines a directed edge* (A→B): `contextualizes`
  (→`contextualized_by`), `references:concerns` (→`concerned by`).
- **`notes`** *annotate a symmetric pairing* ({A,B}): `interactions`
  (→`interactions`), `related` (→`related`).

Putting a `descriptor` on a symmetric edge is a category error (describes which
direction?). This slice builds `descriptor` for the **directed** broad edges and
leaves the existing `interactions.notes` (symmetric) untouched.

## Scope & Objectives

Add an **optional, free-text `descriptor: String`** facet to **directed** broad
relation edges, riding the existing `Degree` facet seam (SL-176).

1. **Data model** — a Tier::One `[[relation]]` row may carry an optional
   `descriptor` cell. Free text only: stored, round-tripped, searchable,
   renderable. **Excluded from edge identity** — identity stays the
   `(label, role, target)` triple (two edges differing only in descriptor are the
   *same* edge), exactly as `degree` is excluded (`relation.rs` §A.5). Never an
   input to validation, inference, graph traversal/effects, or reciprocity.

2. **Admissibility (closed allow-set, per-rule column)** — a
   `descriptor_bearing: bool` column on `RelationRule` (mirroring `degree_bearing`),
   `true` on exactly the directed broad labels:
   - **Allow:** `contextualizes`, `references` with `role = concerns`.
   - **Deny (reject, not ignore):** every non-bearing label — structural/lifecycle
     (`parent`, `descends_from`, `members`, `supersedes`, `fulfils`,
     `owning_slice`) **and** symmetric labels (`related`, `interactions`). Policy
     is the established house rule: `validate_link` bails on a facet set against a
     non-bearing label, symmetric to `degree`/`role` (`relation.rs:1447`).

3. **Write seam** — `doctrine link` gains `--descriptor <text>`; threaded through
   `validate_link` (gate) → `append_edge(…, descriptor)` → the row builder
   (`row.insert("descriptor", …)` only when present, load-bearing for diff
   stability), mirroring `degree` end-to-end. Append-conflict semantics: **design
   §2 decides** (reject differing descriptor on same edge, like `degree`, vs
   overwrite/no-op).

4. **Surfacing (unify at the view layer, not storage)** — one render/search
   helper treats *both* `descriptor` (directed) and the existing
   `interactions.notes` (symmetric) as "edge annotation text": rendered adjacent
   to the edge in `show`/`inspect`, and included in the search index. Interactions
   `notes` gains search indexing as a consequence (it has none today).

## Non-Goals

- **No new label or role.** Closed `RelationLabel` / role vocabularies unchanged
  (ADR-016 preserved).
- **No `notes`→`descriptor` migration / no `--notes` rename.** `interactions`
  keeps `notes` (symmetric) untouched; no breaking change to `spec interaction add`.
- **No descriptor on symmetric or structural edges** — deny-set is firm.
- **No `notes` on `related`** — symmetric, would take `notes` not `descriptor`; no
  present demand (YAGNI). Deferred.
- **No graph semantics.** Descriptor never affects validation, inference,
  traversal, priority, reciprocity (ADR-004), or dedup/identity.
- **The future record→work "informs/bears-on" label** — named as a forward
  inheritor of the descriptor rule (it is directed), not built here.
- **Part 1 (concept/CPT kind)** — separate track.

## Summary

One optional free-text cell on a Tier::One `[[relation]]` row, admissible on the
two directed broad labels (`contextualizes`, `references:concerns`), riding the
proven `Degree` seam and inert to all graph semantics. The view layer unifies
descriptor + the retained symmetric `interactions.notes` for render and search.
Behaviour-preservation gate: existing relation validation, storage-tier
behaviour, and graph effects stay green unchanged (IMP-244 acceptance). Remaining
design question: append-conflict semantics on re-link with a differing descriptor.

## Follow-Ups

- Future record→work "informs/bears-on" relation inherits the descriptor rule.
- `related` symmetric `notes` — if demand arises.
- Part 1: `concept`/CPT knowledge kind (off-backlog).
