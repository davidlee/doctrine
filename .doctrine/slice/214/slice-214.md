# Knowledge authoring skill

## Context

The knowledge-record command surface ships (SPEC-019: `doctrine knowledge
new|show|list|status`; **seven** kinds since SL-159 — ASM/DEC/QUE/CON/EVD/HYP/CPT;
IMP-182's "four kinds" is stale), but no agent skill routes to it. Live census
2026-06-26: zero knowledge records, zero `shapes` edges. *(Corrected at audit,
RV-280 F-6: five records — ASM-001, DEC-001, QUE-001, QUE-171, CON-001 — landed
after that census in the SL-158/ADR-017 era. The premise holds as "no records
authored through a routed skill"; the slice's dogfood ids are therefore -002.)*
This is RFC-007 workstream 3 ("populate — make gating bite") and the
root cause of the empty population blocking RFC-008 (trinary-actionability
gating, SL-158) — the gate has nothing to gate on until records exist.

Originates from IMP-182.

## Scope & Objectives

- An authoring skill (`/knowledge`, or folded into an existing skill — design
  decides) that routes:
  - "capture an assumption / decision / question / constraint" → `knowledge new`
  - "settle it" → `knowledge status`
  with the truth-vs-work boundary (PRD-010: records *shape* work, never *are*
  work), the relation seam (`shapes` / `spawns`, ADR-004 outbound-from-record),
  and the gating model (ADR-017: association ≠ gating — the *dependent* authors
  `needs → record`; records never author dep/seq) made legible.
- Capture touchpoints in existing skills so records are authored *during*
  design/route, not as an afterthought (light pointers, not re-implementations).
- Routing-table row in `install/routing-process.md` — **after** the skill is
  installable (ADR-009 F14: never route to a shipped-not-reachable skill).
- Authored under `plugins/` (the install source); POL-002 binds — no repo-local
  couplings, must resolve in a fresh client.

## Non-Goals

- Gating mechanics themselves (RFC-008 / SL-158 — the consumer, not this slice).
- Record↔record associative class (IMP-053).
- DEC-vs-ADR guidance doc (IDE-007) and knowledge lint (IDE-009).
- Engine/CLI changes to the knowledge surface — if a gap blocks the skill,
  `/consult`, don't widen scope silently.

## Risks & Open Questions

- Standalone `/knowledge` vs folded into an existing memory/governance skill —
  design fork; standalone adds a routing row, folded risks burying the trigger.
- Skill invisible until installed + re-embedded (`touch src/skills.rs` →
  `cargo build` → `doctrine install`) — sequencing matters for the routing row.
- Chicken-egg: real validation is population; a skill can install cleanly and
  still never fire. Capture-touchpoint placement is the lever.

## Verification / Closure Intent

- VA: skill installs into a fresh client and resolves with no repo-local
  couplings (POL-002 reflex check).
- VA: dry-run authoring of each kind (ASM/DEC/QUE/CON) via the skill produces
  well-formed records with `shapes`/`spawns` edges.
- VH: routing row lands only after install; boot snapshot points at a
  reachable skill.

## Summary

## Follow-Ups
