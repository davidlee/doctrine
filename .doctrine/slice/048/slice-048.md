# Structural cross-corpus relation edges: governance seam + spec-ADR + product-product

## Context

Slice **3 of 3** in the graph-relations work, and the realisation of **IMP-016**:
cross-corpus relations are **prose-only** today. The capture surface is uneven —
slices have `specs`/`requirements`/`supersedes`; specs have `descends_from` +
members; backlog has `specs`/`slices`/`needs`/`after`/`drift`. But:
- governance kinds (POL/STD/ADR) carry a `[relationships]` block
  (`supersedes`/`superseded_by`/`related`/`tags`) that is **parsed but inert —
  never queried** (`src/governance.rs`);
- there is **no structural spec↔ADR edge** (a spec citing a governing ADR is
  prose);
- there is **no product↔product edge** (PRD-to-PRD links are prose).

SL-046 makes the graph read **all existing authored relations** (including the
inert governance block, read-only). This slice **mints the missing authored
edges** so the connective tissue is structured, queryable, and feeds the graph —
once a live reader (SL-046) already exists, so new edges are not born inert.

**Sequenced last on purpose.** Authoring capture before a reader recreates the
exact inert-seam bug this work exposed. SL-046 (read) + SL-047 (rank) must exist
first.

## Scope & Objectives

1. **Activate the governance `[relationships]` seam as authored, validated
   relations** — a `link`-style verb (or `--related`/`--supersedes` flags on the
   existing governance verbs) so POL/STD/ADR relations are *authorable*, not just
   hand-edited inert TOML; surfaced by `show` and the SL-046 query.
2. **Spec↔ADR structural edge** — a spec can cite the ADR(s) that govern it as a
   typed relation, not prose.
3. **Product↔product structural edge** — PRD-to-PRD links (e.g. PRD-011 reads
   PRD-009's seam) become a typed relation.
4. **Forward-edge validation** where the target is a numbered kind in
   `integrity::KINDS`; free-text refs (e.g. `DEC`) carry unvalidated
   (mem.pattern.entity.free-text-ref-not-forward-validated). ADR-004 holds:
   **outbound only**, reciprocity derived (no stored inbound).

## Non-Goals

- **No reverse/inbound storage** — inbound stays derived (ADR-004; SL-046).
- **No graph/CLI build** — SL-046 reads these edges; SL-047 ranks. This slice is
  capture-surface only.
- **No `cordage` change.**
- **No re-modelling existing relation fields** — additive; existing
  slice/spec/backlog relations untouched.

## Affected Surface

- `src/governance.rs` — make the `[relationships]` block authorable + validated
  (the spine of POL/STD/ADR relations).
- `src/spec.rs` — spec↔ADR + product↔product relation fields + author verb.
- `src/main.rs` — relation-authoring verbs/flags.
- `src/integrity.rs` — forward-edge validation against `KINDS` (extend the
  existing dangling-citation logic, do not duplicate).
- Kind specs (SPEC-005/006/016) — updated to describe the now-live relations.

## Risks, Assumptions, Open Questions

Blocking prerequisite — **relation governance must be decided first**:
- The cross-corpus relation model is a **project-global decision** → a **new ADR**
  is almost certainly required (what kinds may link to what, edge semantics,
  validation policy, how it composes with ADR-004 outbound-only). This is *not*
  slice-local. Route: `/spec-tech` / `doctrine adr new` before this slice's
  design.
- Kind-spec updates (SPEC-005 ADR surface, SPEC-006 spec composition, SPEC-016
  governance kinds) — all currently **draft**; they describe the relationship
  seams and must be settled.

Open questions (for the governance decision, not here):
- **Verb shape** — a uniform `link`/`relate` verb across kinds vs per-kind flags.
  The "uniform destructive + lifecycle verbs across kinds" theme (IMP-006) is
  adjacent — a uniform relation verb may belong with it.
- **Which relation kinds are reciprocal-meaningful** vs purely lineage.
- **Validation strictness** — hard-fail on dangling numbered-kind target vs warn
  (the reseat/validate precedent: report danglers, never rewrite).

Assumptions:
- SL-046 + SL-047 land first; this slice's edges immediately light up in the
  graph + query (the anti-inert ordering).
- Canonical id is the stable cross-kind target key.

## Verification / Closure Intent

- A governance relation authored via the new verb is **validated**, **persisted**
  in `[relationships]`, surfaced by `show` and the SL-046 query (no longer inert).
- A spec↔ADR and a product↔product edge are authorable, validated, and appear in
  both the authoring entity's outbound view and the target's **derived inbound**
  view (ADR-004 — derived, not stored).
- Forward-edge validation rejects/flags a dangling numbered-kind target; free-text
  refs pass unvalidated.
- Existing relation fields and all suites stay green (additive change).

## Follow-Ups

- Uniform cross-kind relation/link verb (compose with IMP-006).
- Any further kinds' relationship seams not covered here.
- Revisit whether reciprocal relation *display* (derived inbound badges) warrants
  a shared render helper across `show` surfaces.
