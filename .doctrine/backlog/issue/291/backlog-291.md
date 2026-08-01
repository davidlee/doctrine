# ISS-291: SKILL.compact.md still teaches the pre-SL-233 eight-stage design machine

Surfaced by SL-233 PHASE-08's `VA-4`/`VA-7` disposition table, which is total
over `sketches/thin-adapter.md`'s `(a.3)` census — and `SKILL.compact.md` is not
in that census.

## What

`plugins/doctrine/skills/design/SKILL.compact.md` — 81 lines / 4,208 bytes,
**unchanged** by PHASE-08. It carries its own `## Workflow` section teaching an
ordered eight-stage sequence ("Complete in order; each stage depends on the one
before it"), which is exactly the machine PHASE-08 removed from the active
`SKILL.md` and relocated into the design run.

## Why it was correctly left alone by PHASE-08

- It is **not** a `(a.3)` destination and **not** part of the 214-line /
  10,178-byte incumbent baseline `VA-4`/`VA-7` measure against.
- It self-declares its status: *"Experimental compressed port of the design
  skill. Not the active skill — `SKILL.md` is. Kept for comparison / later
  A/B."*
- It contains no `slice design` invocation, so `VA-3`'s sweep is unaffected.
- Repairing it sits outside every PHASE-08 exit criterion.

## Why it still matters

Two live consumers make a stale copy more than untidiness:

1. **It ships.** Both distribution channels install from the github remote at
   `main`, so a shipped file describing a workflow the binary no longer runs is
   an agent-visible contradiction the moment anyone reads it instead of
   `SKILL.md`.
2. **PHASE-09's evaluation kit names a *"pre-SL-233 skill baseline"* as a
   compared treatment** (SL-233 `PHASE-09 EX-8(a)`, `VA-4`). This file is the
   nearest thing to one still in the tree, which cuts both ways — useful as a
   fixture, misleading as guidance.

## Options, not yet chosen

- **Delete it.** The A/B it was kept for is now against a machine, not a prose
  variant, so the comparison it was preserving may no longer be the interesting
  one.
- **Port it** to the thin-adapter shape, keeping it as a genuine compact variant.
- **Freeze it as a named PHASE-09 fixture** — move it under the evaluation kit
  where "pre-SL-233 baseline" is its job rather than a side effect, and stop
  shipping it as a skill.

Owner's call. The third option is the only one that serves `PHASE-09 EX-8(a)`
directly.
