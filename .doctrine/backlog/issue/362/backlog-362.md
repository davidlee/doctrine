# ISS-362: Act admissible at the wrong stage silently no-ops

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

A `dispose` batch submitted at `stage = exploring` is **accepted**: exit 0, the
revision bumps, a receipt is written — and **no change rows** are produced. The act
is admissible in form but inert at the current stage, and the engine reports that
as success rather than as a typed refusal naming the stage the act requires.

## Why it matters

Stage is the design run's core sequencing constraint; an act at the wrong stage is
precisely what the state machine exists to catch. Reporting it as success inverts
the guarantee — the machine's own ordering rule fails open.

The cost compounds with `ISS-355` (*successful apply prints no change row*): with
no change rows on legitimate success either, an agent has **no observable at all**
that separates "landed", "absorbed" (`ISS-346`) and "inert at this stage" (this
issue). Three distinct outcomes, one indistinguishable response.

## Evidence

- `019fd226-ad55` — dispose batch at wrong stage accepted, no change rows

## Shape of a fix

A typed refusal naming both the current stage and the act's required stage, and —
importantly — **no revision bump**, since nothing happened. That last part matters
for `ISS-361`: a revision consumed by a no-op is a revision the caller cannot
account for.

Worth checking whether this is act-specific (`dispose`) or general to every act
whose stage guard is absent — `run.rs` dispatches on id kind alone for `declare`,
which suggests the guard is missing broadly rather than in one arm.

## References

- `ISS-355` — successful apply prints no change row
- `ISS-346` — design apply silently absorbs unknown payload keys
- `ISS-361` — unparseable submission applied and receipt-locked
- `QUE-219` — submission strictness against snapshot forward-compatibility
