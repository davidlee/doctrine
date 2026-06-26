# IMP-182: /knowledge authoring skill: route agents to create + transition ASM/DEC/QUE/CON records

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

The knowledge-record **command surface** ships (SPEC-019: `doctrine knowledge
new|show|list|status`, the four kinds ASM/DEC/QUE/CON, SL-059). But there is **no
agent skill** routing to it — no `/knowledge` analogous to `/slice`, `/spec-tech`,
`/backlog`. So in practice agents never author records: a live check (2026-06-26)
found **zero knowledge records and zero `shapes` edges** in the corpus.

This is the RFC-007 **workstream 3** ("populate — make gating bite") gap, and it is
the root cause of the empty population that reshaped **RFC-008** (trinary-actionability
gating): the gate mechanism (ws1) has nothing to gate on, and cannot be census-validated,
until records exist. The chicken-egg is real — this item is the ws3 half.

## What

An agent skill (`/knowledge`, or a section folded into an existing memory/governance
skill) that routes the intent "capture an assumption / decision / question /
constraint" to `knowledge new`, and "settle it" to `knowledge status`, with the
truth-vs-work boundary (PRD-010: records *shape* work, never *are* work) and the
relation seam (`shapes` / `spawns`, ADR-004 outbound-from-record) made legible.

Pairs with: a recall/authoring prompt so records get created *during* design/route
rather than as an afterthought.

## Coordination

- **RFC-007 ws3** — the umbrella; this is its first concrete entry.
- **RFC-008 / SL-158** — the consumer waiting on a populated graph (ws1).
- **SPEC-019** — the surface this skill drives; **IMP-053** (record↔record
  associative class), **IDE-007** (DEC vs ADR guidance), **IDE-009** (knowledge lint)
  are adjacent.
