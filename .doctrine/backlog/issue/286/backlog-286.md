# ISS-286: Gate evidence subjects are section-only, forcing drafting before exploring completes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found during SL-233 PHASE-16's second adversarial design review (2026-07-31).
Not PHASE-16's to fix.

## What

A claimed gate condition is recorded as `Evidence { condition, subject,
fingerprint }`, and the fingerprint is resolved by `current_fingerprint`
(`src/design_run/run.rs:1283-1290`), which looks the subject up in
`next.sections` and returns `Refusal::UnknownNode` otherwise.

So **every** piece of gate evidence must name an existing **draft section**, at
every stage — including the two conditions guarding `Exploring → Inquiring`,
which are about governance reading and initial concerns rather than about any
draft prose.

`declare` has no stage guard; it dispatches on id kind alone (`run.rs:768-770`).
So a section *can* be declared while the run is still `Exploring`.

## Why it matters

The combination is an ordering inversion. To leave `Exploring` an agent must
first **create a draft section** — drafting work, two stages early — and then bind
a claim about its governance reading to that section's bytes. The machine
requires a step from stage 3 as a precondition for finishing stage 1.

It is also a liveness mismatch. DEC-066 expires evidence when the subject's
fingerprint moves, so editing the prose of whichever section was used as an
anchor silently invalidates an unrelated claim about governance being in view.

## Scope

The narrow fix is to widen the subject vocabulary so a condition can bind to
something meaningful for it — the run itself, an inquiry node, or a corpus
digest — rather than forcing a section. That is a change to a shared seam
(`Evidence`, `current_fingerprint`, DEC-066 liveness) and carries the
behaviour-preservation gate: the existing design-run suites are the proof.

Deliberately **not** attempted in SL-233 PHASE-16, whose runbook discharge
records are their own run state rather than `Evidence` rows, precisely so that
the runner does not need this seam widened to land.

Same defect's other half: [[ISS-285]] — the two conditions this most affects are
unimplemented stubs to begin with.
