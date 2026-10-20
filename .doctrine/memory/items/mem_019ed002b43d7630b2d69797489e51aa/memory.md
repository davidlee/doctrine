# No standalone plan validation — malformed plan.toml surfaces late

There is no standalone plan.toml validation. A malformed plan.toml only surfaces when slice phases parses it at execution time. The author gets no feedback at plan-authoring time.


## A concrete instance, and it is worse than "late" — it can be silent

SL-284 (2026-10-20). A `[[phase]]` header for PHASE-01 had been commented out
while the block's keys were edited. TOML has no error for this: the orphaned keys
(`id`, `name`, `objective`, `entrance_criteria`, …) were simply absorbed into the
preceding table — `[requirements]` — and the file parsed **cleanly**.

Nothing warned. `doctrine slice phases` then reported four phases instead of five,
and progress read `0/4` while PHASE-01 was invisible: no runtime sheet, so no
criteria, so no verification, and the slice looked complete at 3/4 when it was
1/5 done. It was caught only by reading the plan against the design's phase list,
not by any verb.

So the sharper statement is: a structurally wrong plan can parse into a
*different, valid* plan. "Malformed surfaces late" understates it — the failure
mode is a plan that is valid TOML, valid to the engine, and wrong.

## What to do about it

- After authoring or editing a plan, cross-check the phase count against the
  design: `doctrine slice plan <ID>` and `doctrine slice phases <ID>` should
  enumerate exactly the phases the design names.
- Be suspicious of a phase-count *decrease* after an edit; an absorbed header
  removes a phase without removing its prose.
- Do not "tidy" a plan by commenting out a header. Comment the whole block.
