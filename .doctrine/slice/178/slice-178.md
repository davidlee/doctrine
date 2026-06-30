# Close drift-discharge legibility: richer error + skill recipe + shipped memory

## Context

`doctrine slice status <id> done` refuses with a one-line error for
undischarged residual drift. The 3-clause `rec_discharges` predicate and the
accept-REC recipe must be reverse-engineered from `src/slice.rs`. This costs
~4 round-trips per governed close.

Source: IMP-202 (SL-165 PIR S1,S2,S6 — close complexity cascade, HIGH).
The existing recipe memory `mem_019f075f` is local (`.doctrine/memory/items/`),
unshipped — a CLI error pointing there would violate POL-002.

## Scope & Objectives

1. **Richer `slice status done` error** — `src/slice.rs:832-838`: expand the
   one-liner to name each flagged REQ with its current authored status, the
   3-clause accept-REC pattern, and a pointer to shipped knowledge.
2. **`/close` skill documents the recipe** — `.pi/skills/close/SKILL.md`: add a
   "Drift discharge recipe" subsection with the 3-clause predicate, the concrete
   CLI pattern, and the worked example (SL-165 → REC-093/REC-094).
3. **Promote recipe memory to shipped** — move `mem_019f075f` content into a new
   shipped memory under `memory/` (embedded via RustEmbed), so the CLI error and
   the skill can safely reference it per POL-002.

## Non-Goals

- Changing the `rec_discharges` predicate itself (correct, just illegible)
- Auto-authoring `[[status_delta]]` / `[[evidence_ref]]` tables in the REC CLI
- A `--verbose` flag to list missing evidence keys per REQ (separate improvement)
- Surfacing the integrate-step companion memory (separate concern)

## Verification

- **VT**: `slice status <id> done` on a drifted slice produces an error that
  names each undischarged REQ, its current authored status, the 3-clause
  accept-REC pattern, and a pointer to shipped knowledge.
- **VA**: `/close` skill carries a drift-discharge subsection.
- **VT**: a shipped memory exists under `memory/` with the recipe; discoverable
  via `doctrine memory find`.
- **VA**: no shipped artefact references a local project memory.

## Affected Surface

- `src/slice.rs` — error message at the closure-gate drift predicate
- `.pi/skills/close/SKILL.md` — skill documentation
- `memory/` — new shipped memory (RustEmbed corpus)

## Summary

Three small, high-leverage legibility fixes. The predicate is correct but
inscrutable; the skill is silent on the recipe; the recipe memory exists locally
but is unshipped. Each fix is independently shippable; together they collapse
the ~4 round-trip close-discovery cost to near-zero.

## Follow-Ups

- IMP-192 (L0 close/orientation friction cluster — related but broader)
- IMP-216 (related — broad migration of ~46 project-local operational memories
  to shipped reference knowledge; Fix 3 here is the concrete first instance)
