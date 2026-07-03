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

## Scope & Objectives

Add an **optional, free-text `descriptor: String`** attribute to relation edges,
admissible only on a closed **allow-set of broad, non-structural labels**.

1. **Data model** — a relation edge may carry an optional `descriptor`. Free text
   only: stored, round-tripped, searchable, renderable. **Never** an input to
   validation, inference, graph traversal/effects, or edge identity (two edges
   differing only in descriptor are the *same* edge).

2. **Admissibility (closed allow-set)** — descriptor is legal only on broad
   associative/contextual/epistemic labels:
   - **Allow:** `contextualizes`, `related`, `interactions`, `references` with
     `role = concerns`. (Candidate future record→work "informs/bears-on" edge
     inherits the rule when it lands — out of scope here.)
   - **Deny:** all structural / lifecycle edges — `parent`, `descends_from`,
     `members`, `supersedes`, `fulfils`, `owning_slice` (and, by default, every
     label not in the allow-set).
   - The allow/deny decision is a property of `(label, role)`, colocated with
     `RELATION_RULES` — **design decides** reject-vs-ignore on illegal placement
     (IMP-244 acceptance: "rejected or ignored **consistently by policy**").

3. **Write seam** — `doctrine link` accepts an optional `--descriptor <text>`;
   the unified write path (ADR-010 seam: `validate_link` / `append_edge`) gates
   it against the allow-set and threads it to bespoke storage.

4. **Storage round-trip** — descriptor persists in the bespoke per-kind relation
   store (ADR-010: storage stays bespoke) for the eligible labels and reads back
   intact via `show` / accessors.

5. **Surfacing** — rendered relation views (`show`, relation census/inspect) show
   descriptor text adjacent to its edge; descriptor text is included in the
   search index.

## Non-Goals

- **No new label or role.** The closed `RelationLabel` / role vocabularies are
  unchanged (ADR-016 preserved).
- **No graph semantics.** Descriptor never affects validation, inference,
  traversal, priority, reciprocity (ADR-004), or dedup/identity.
- **No reverse/inbound descriptor.** Outbound-only (ADR-004); inbound is derived
  and carries no authored descriptor.
- **No structural-edge descriptors** — the deny-set is firm.
- **The future record→work "informs/bears-on" label** — named as a forward
  inheritor of the rule, not built here.
- **Part 1 (concept/CPT kind)** — separate track.

## Summary

One optional free-text attribute on an edge, admissible on a closed allow-set of
broad labels, inert to all graph semantics. Behaviour-preservation gate: existing
relation validation, storage-tier behaviour, and graph effects stay green
unchanged (IMP-244 acceptance). Open design questions: (a) reject vs silently-drop
on illegal placement; (b) whether the attribute lives on a uniform edge
representation or is threaded per bespoke store; (c) how `references`-role
gating composes with the label allow-set.

## Follow-Ups

- Future record→work "informs/bears-on" relation inherits the descriptor allow-rule.
- Part 1: `concept`/CPT knowledge kind (off-backlog).
