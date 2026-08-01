# IMP-373: Runbook set mode: coverage-set admission and its rendering bound

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Add `Mode::Set` to the obligation-runbook runner: a coverage set whose steps
carry no meaningful order, so admission derives no refusal from ordering. Today
`Mode` is a **one-variant enum** (`src/design_run/runbook.rs:201-206`) and
`mode = "set"` is refused at parse with `unknown variant`, locked by a case in
`validation_refuses_the_whole_domain_before_anything_executes`
(`src/design_run/runbook.rs:986-993`).

## Why it was deferred out of SL-233 PHASE-08

DEC-101 (obligation runbooks — ordered steps as asset data, verifiers as
executables) shipped `sequence` only in PHASE-16 and assigned `set` to PHASE-08,
on the reasoning that PHASE-08 converts the two genuinely coverage-set
checklists — present-design's fifteen content items and the adversarial pass's
attack list — and would have real instances to design the mode against
(`sketches/runbook-runner.md:536-543`, `:688-692`).

That reasoning still holds as an argument for adding it *eventually*. It was
deferred again because of a live conflict the record already names
(`.doctrine/slice/233/notes.md:425-427`):

> Only the cursor step carries its `text` — every step's prose is both the
> deferred `set` half **and** the token regression `EX-14` forbids.

`EX-14` bounds runbook rendering: only the step **at the cursor** carries its
full `text`; the rest render as one line each. A coverage set has no cursor by
definition, so `set` mode has no rendering rule and cannot simply inherit one.
Resolving that is a design question about token budget, not a variant addition,
and PHASE-08's job is to *shrink* guidance prose — reopening the token bound
inside it is the wrong place.

## Why deferral is clean

Nothing depends on it. There is no stub, no `todo!()`, no dead arm, and no
half-shipped surface:

- `Mode` has exactly one variant, so no code branches on a missing one;
- `mode = "set"` is a hard parse refusal, not a silent acceptance;
- a test asserts that refusal, so the boundary cannot erode unnoticed;
- the refusal message is honest — *"this runbook is a sequence"*
  (`src/design_run/refusal.rs:353`).

## What the work is

1. Decide the rendering rule for a cursorless runbook under `EX-14`'s token
   bound. This is the actual open question. Options include: render the first
   N undischarged steps in full; render all as one line each and require a
   fetch act for detail; or bound by byte budget rather than by count.
2. Add `Mode::Set` and the admission rule — a set derives no refusal from
   ordering; the gate condition (every `required` step discharged) is
   unchanged.
3. `RunbookStanding.cursor` becomes `Option` in a second sense: absent because
   the mode has none, versus absent because the runbook cleared. Those two must
   not be conflated — `cleared()` already distinguishes.
4. Convert whichever checklists the target-state work assigns to set-shaped
   edges.

## Related

- DEC-101 — the runbook decision; its consequences name the `sequence`/`set`
  split as two admission rules over one act, not one rule behind a flag.
- `src/design_run/gate.rs:199` `boundary_runbook` — the edge-keyed selector,
  one row populated, with an in-code comment assigning the next two to PHASE-08.
- `install/design-prompts/exploring.toml` — the shipped `sequence` exemplar and
  its authoring rule.
- IMP-372 — the deferred override seam; a sibling deferral out of the same
  slice.
