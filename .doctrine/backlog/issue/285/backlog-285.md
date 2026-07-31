# ISS-285: Design-run exploring gate conditions are unimplemented stubs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found during SL-233 PHASE-16's second adversarial design review (2026-07-31),
while establishing what the runbook runner's gate clause would sit beside. Not
PHASE-16's to fix — recorded so it is not rediscovered.

## What

`Condition::GoverningContextRecorded` and `Condition::InitialConcernsRecorded`
guard the `Exploring → Inquiring` edge (`src/design_run/gate.rs:163-166`). Both
are caller-claimed (`is_derived() == false`, `:92-97`). Three facts together make
them unusable as specified:

1. **Their entire specification is nine words**, at
   `.doctrine/slice/233/design.md:268` — *"exploring → inquiring: governing
   context and initial concerns recorded"*. There is no other definition of
   either condition anywhere in the repository.

2. **No shipped guidance names them.** A search of `install/`, `plugins/` and
   `.doctrine/spec/` returns nothing for either condition or its kebab token.
   Not in `install/design-prompts/inquiry.md`, not in a hymn, not in
   `plugins/doctrine/skills/design/SKILL.md`. Every occurrence outside `gate.rs`
   is in a **test** — `src/design_run/tests.rs:162`, plus `e2e_design_delegation`,
   `e2e_design_review`, `e2e_design_projection`, `e2e_design_state`.

3. **Their subject must be a draft section.** Evidence binds through
   `current_fingerprint`, which resolves only from `next.sections`
   (`src/design_run/run.rs:1283-1290`); anything else is `Refusal::UnknownNode`.
   Every test binds them to `sec-01` / `sec-1` / `sec-{index}`.

## Why it matters

An agent cannot satisfy a condition it has never been told exists. Today the only
route to clearing this edge is reading the source or the test suite, which makes
the guard decorative for its intended audience.

Point 3 compounds it: a claim that governing context is in view gets bound to the
fingerprint of an arbitrary draft section, and DEC-066 liveness then expires the
claim when that section's prose changes — an event with no relationship to the
fact being claimed. The binding does no semantic work.

## Not closed by the runbook runner

SL-233 PHASE-16 lands an obligation runbook guarding this same edge, and
deliberately **does not** derive, satisfy, or feed these two conditions
(PHASE-16 `EX-19`). Two reasons, both recorded on DEC-101: truth may not flow
from user-customisable material into a Doctrine-owned closed vocabulary; and
these conditions are a placeholder for what the runbook records rather than a
competing account of it. So the runbook lands *beside* this issue and makes the
emptiness more visible rather than less.

## Options, not yet chosen

- Specify and document them, giving each a subject rule that means something.
- Derive them from run state now that there is run state worth deriving from —
  but see `EX-19`: only safe for framework-owned inputs.
- Retire them and let the runbook guard be the edge's whole guard.

The third is cleanest and the most disruptive. It wants deciding *after* PHASE-16
has shipped, when there is operational evidence about whether the runbook guard
suffices on its own.

See also [[ISS-286]] — the section-only subject rule, the same defect's other half.
