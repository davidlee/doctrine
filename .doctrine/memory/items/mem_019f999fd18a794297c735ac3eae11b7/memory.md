# Spec draft→active is a hand-edit — no `spec status` verb exists, though `spec req status` does for requirements

A spec's status is advanced by editing `status =` in its TOML — `spec edit` covers
descent/parent scalars only and there is no `spec status`. Requirements are the
opposite: `spec req status` transitions them any→any. Generalising from either axis
to the other guesses wrong.

## Why it matters

Two standing guardrails — "use the CLI, don't guess command shapes" and "read
entities via `show`, not raw files" — make the spec half undiscoverable. An agent
obeying them cannot find the transition at all, and an agent that finds it can't
tell whether hand-editing is sanctioned or a violation. It is sanctioned:
`SpecStatus` is a closed enum (`Draft | Active | Deprecated | Superseded`),
hand-edited, and git is the trail (no date stamps).

The asymmetry is the trap, and it bites in both directions. At CHR-046 I
generalised spec→requirement and wrongly reported that neither axis had a verb; a
research thread generalised requirement→spec and asserted `spec req status` worked
for specs too. Same asymmetry, two wrong answers.

## What the statuses mean

- Spec `draft` → `active`: **no documented gate exists anywhere in the corpus.**
  PRD-002 §6 says a spec "opens in draft" and never defines what promotion
  requires; PRD-012 §6 adds no tech-specific dialect. It is a human judgement
  call, conventionally: open questions resolved, requirements substantive.
- Requirement `pending` = "declared, not started"; `active` = "in force,
  verified". So activating a requirement asserts **delivery**, not authoring
  completeness — check the code exists before flipping.

## How to apply

- Spec: hand-edit `status = "draft"` → `"active"` in `spec-NNN.toml`. Precedent
  abd843922 (`req(ISS-023): activate 20 draft specs + 161 requirements`).
- Requirement: `doctrine spec req status` (free any→any, edit-preserving).
- Before activating, spot-check that the acceptance criteria's named symbols
  actually resolve — the criteria cite real functions/types, so a grep per
  requirement is a cheap delivery proof. Don't infer delivery from the prose being
  complete: authoring completeness and delivery are independent, which is exactly
  what `pending` on a fully-authored requirement means.
- Do **not** read requirement substance from `spec req list` — its `prose` column
  reports only the `.md` tier and shows `—` for requirements whose structured
  acceptance criteria are complete. Read `spec show <SPEC>`'s synthesized
  Requirements section instead. See [[mem.concept.doctrine.reading-entities]].

Related: [[mem.fact.revision.spec-prose-modify-target]] (a REV, not a hand-edit,
is the route for spec-prose *decision* amendments — status is not one).
