# IMP-033: Cross-kind dep/seq capture — extend needs/after sequencing beyond backlog to specs/slices

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`needs` (hard dependency) and `after` (soft sequence, with `rank`) are authored
**only on backlog items** today (`src/backlog.rs:404-417`; verbs `backlog needs` /
`backlog after`). But the dep/seq *intent* is cross-kind: e.g. a content-schema
spec genuinely **needs** a storage-layer spec; coarse delivery sequencing across
specs or slices wants an **after** preference. There is no way to author that —
only backlog carries the fields.

## Why it matters

Surfaced during SL-046 design. SL-046 (the reference/lineage reader) deliberately
excludes dep/seq — those are a different semantic layer (actionability/blocking,
SL-047's overlays), and their cross-kind *capture* is a schema change. This item
captures the capture-side gap so the idea isn't lost.

Distinct from IMP-016 (reference/lineage links — specs↔ADR, product↔product).
This is **sequencing/dependency** edges, not reference edges.

## Scope of the fix (NOT SL-046)

- Extend the `needs`/`after` authored schema (and verbs) to specs and slices (and
  any work-like kind) — per SPEC-001 D4 the dep/seq authored schema is owned by
  **PRD-009**'s capture surface; the cross-kind extension likely rides the
  relation-governance ADR that **SL-048** needs.
- Once captured, SL-046's adapter projects them as dep/seq overlays and SL-047
  consumes them for cross-kind blockers/order — no reader change beyond admitting
  the new kinds' edges.
- Open question for the ADR: which kinds may sequence which (a spec depending on a
  slice would invert the abstraction gradient).

Related: [[SL-048]] · [[SL-047]] · PRD-009 (dep/seq capture surface) · SPEC-001 D4.
