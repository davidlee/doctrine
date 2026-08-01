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

### Updated 2026-08-01 — the instances changed, and so did the warrant

**The two checklists DEC-101 named no longer exist as checklists.** SL-233's D14
reclassified the fifteen content items and the attack list as *stage framing*
(fragment prose), not runbook steps. For a while the slice concluded from this
that `set` had **no user at all** and was deferred *"on merit"*. **That claim is
withdrawn as false.** RV-325 F-5 settled edge 2 at two steps, and edge 4 carries
three — **five order-independent steps shipping under `mode = "sequence"`**, whose
cursor refuses any discharge that does not name the step at it
(`src/design_run/run.rs:1369-1377`). Nothing makes knowledge capture precede scope
reconciliation, or repeat-pass judgement precede selector recording.

So DEC-101's own precondition — *real instances to design the mode against* — is
now **met**, and this item is deferred in spite of that, not because of it. The
surviving warrant is the rendering question above: `set`'s *admission* is cheap
(six lines, plus a `Mode` variant referenced nowhere outside `runbook.rs`), but its
*render* is a design question that sits outside SL-233 PHASE-08's `EN-2` gate.
Adding it there would hand that choice to implementation, which is precisely what
F-5 ruled the gate exists to prevent.

## Why deferral is clean — and what it now costs

Nothing depends on it. There is no stub, no `todo!()`, no dead arm, and no
half-shipped surface:

- `Mode` has exactly one variant, so no code branches on a missing one;
- `mode = "set"` is a hard parse refusal, not a silent acceptance;
- a test asserts that refusal, so the boundary cannot erode unnoticed;
- the refusal message is honest — *"this runbook is a sequence"*
  (`src/design_run/refusal.rs:353`).

**What it costs is no longer nothing, and is stated rather than denied.** Five
steps carry an imposed order that means nothing. DEC-101's *"imposing an order on
a coverage set is fake determinism"* applies to them, at reduced scale (five steps
across two edges, not nineteen on one). That is **conceded and judged cheap**, not
argued away — the steps are attestation-only and the render names the step at the
cursor, so the cost is the order in which an agent *attests*, not the order in
which it *works*. That reasoning is untested, which is why it carries a trigger
rather than a promise.

### Repayment condition (added 2026-08-01, owner ruling)

This item repays when agents are observed hitting
`Refusal::DischargeNotAtCursor` (`src/design_run/refusal.rs:198`) against the
edge-2 / edge-4 runbooks on an order that carries no meaning.

Collected during **SL-233 PHASE-09's exercise (CHR-049)**, *not* from the run
record: a refusal aborts the write, so nothing is journalled — the change log
records applied changes only. The refusal string is machine-emitted and quotable,
which makes this an observation rather than a judgement.

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
