<!-- Shipped reference. Published, not projected: there is no copy on disk in an
     installed project — read it with
     `doctrine library show reference/design-run-obligations.md`. It carries the
     rationale for the design-run boundary obligations; the runbook that owns the
     inquiring boundary points here for it. -->

# Design-run obligations

A design run moves through stages. Most of what carries it is **craft**: how to
frame a question, how to weigh a tradeoff, how to read an accepted decision. That
craft has no finish line and is delivered continuously.

At some stage boundaries, though, the run will not advance honestly until a small
number of **discrete acts** are done. This document explains what distinguishes
such an act — an *obligation* — from the surrounding craft, and why the two acts
at the inquiring boundary qualify.

## Obligation or lens

- A **lens** is guidance with no truthful completion point. It improves every turn
  and can never be declared finished — the craft of asking one question at a time,
  offering options with their tradeoffs, preferring a multiple-choice question
  where one fits. A lens shapes the whole stage; it is not a gate.
- An **obligation** is a discrete act whose completion *completes* it. It states
  its own completion condition, in its text, at authoring time. Satisfying that
  condition discharges the act rather than starting it.

**The discriminator is the condition.** If a truthful completion condition can be
stated for the act, it is an obligation. If no such condition can be stated — only
a direction of travel — it is a lens. This is decidable over the whole set before
any run, from the text alone; no observation of a particular run is needed to
settle it.

**What is *not* the test is whether a verifier ships.** A verifier strengthens a
discharge from an agent's attestation to a mechanically checked fact; it is a
separate axis. Its absence is a fact about the *evidence available* for an act, not
about the act's kind. An obligation with no reachable verifier is still an
obligation: it is discharged by an agent asserting the act is done, and that
assertion is the honest evidence. Demoting such an act to a lens would not make it
easier to check; it would only remove the refusal that makes it matter.

**Why blocking is the point.** A lens carries no refusal — a run that ignores it
simply reads worse. An obligation does: the run will not advance past the boundary
until each required act is discharged. That refusal is the entire difference
between the two, and it is the reason a small set of acts is worth imposing on
every run.

## The two inquiring-boundary obligations

### Record what this inquiry settled that outlives the session

*Complete when every answer this inquiry accepted that outlives the session
carries a settled knowledge record shaping this slice.*

The warrant is cost and irreversibility. The answers are in context **now**, and
the stage ends: after it, nothing in the run record holds them. What survives is
what was recorded — and the record, not the transcript, is what a resumed or
reviewing agent reads. An omission here is not paid at the boundary; it is paid by
whoever later has to reconstruct the reasoning from nothing.

### Confirm the slice scope against the decisions this inquiry accepted

*Complete when the scope asserts nothing those decisions contradict and omits
nothing they add to it.*

The warrant is where the failure surfaces. A scope that silently diverges from an
accepted decision is not noticed at the boundary — it is noticed at audit, after
the work built on it, at the point where the divergence is most expensive to
unpick. The act is a few minutes at the boundary and a slice's worth of rework
afterwards.

Both conditions are state-visible at the boundary where they fire: the accepted
decisions are recorded, and the scope document is on disk. That is a property of
the *evidence*, not a reason the acts are obligations — they would be obligations
on the strength of their conditions alone.

## Why these two, and not the craft around them

The craft of asking questions belongs to the stage's continuous guidance, not to a
boundary. It has no truthful completion condition, and inventing one would be
dishonest: neither "asked well" nor "captured everything important" can be
satisfied, only approached.

Two failure modes to avoid when deciding what belongs here:

- **Counting files is not a criterion.** An act is not demoted to a lens because
  the list it would join is long, nor promoted because the list is short. The
  condition decides.
- **A direction of travel in a step's costume.** *"Capture everything important"*
  reads like an obligation and behaves like a lens: it can be asserted without
  ever being true. That is precisely the shape the condition test rejects.

## How they are delivered

Each obligation ships as a step in the runbook the run walks at that boundary. The
step's text states its completion condition; the step is required; and the run's
advance is refused until the step is discharged. The refusal — not the receipt —
is what makes the obligation bind.
