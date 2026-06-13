---
name: slice
description: Use when code-changing intent has emerged and there is no governing slice yet — to scope the change into a slice (scope document + metadata) before any design or implementation. Routed to from /route.
---

# Slice

You are converting intent into a concrete, scoped unit of change. A slice is one
coherent change — not a project-global decision (that is an ADR) and not
evergreen spec material (that lives under `doc/*`).

## Process

1. **Confirm the frame.** Code-changing intent, and no governing slice already
   covers it. Pull the constraints first: `/canon` for ADRs, `doc/*`, and
   conventions; `/retrieve-memory` for subsystem gotchas on the surface you will
   touch.

2. **Create the slice:**

   ```
   doctrine slice new "<title>" [--slug <slug>]
   ```

   Allocates the next id and scaffolds `slice-nnn.toml` (metadata, relations,
   lifecycle status — starts at `proposed`) + `slice-nnn.md` (scope document).

3. **Make scope explicit** in `slice-nnn.md`:
   - what changes, and why
   - in scope vs out of scope (name the boundary)
   - affected surface — concrete paths / modules
   - risks, assumptions, open questions
   - verification / closure intent — how "done" will be judged

4. **Record structure** in `slice-nnn.toml`: metadata, lifecycle status. Honour
   the storage rule — structured data in TOML, prose in MD, never queried/derived
   data in prose. **Relations are written with `doctrine link`, not hand-authored
   rows** — it validates the pair against `RELATION_RULES` (the legal vocabulary;
   `using-doctrine.md` § Relating entities). e.g. `governed_by` an ADR, `specs` a
   spec, `supersedes` a prior slice.

5. **Check the altitude.** If the work is really a project-global decision →
   `doctrine adr new`. If it is evergreen specification → author under `doc/*`.
   Keep the slice to one shippable change; split if it sprawls.

## Next

You MUST shape the design before planning: record the lifecycle move
(`doctrine slice status <id> design` — bare number) and hand off to `/design`.
Do not jump to `/plan` or `/execute` from a bare slice — `/route`'s gate forbids
it. If genuine tradeoffs or unknowns surface while scoping, `/consult`.
