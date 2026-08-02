# The rubric — five evidence classes, scored separately

This is the artefact `VH-1` asks the owner to accept **before** CHR-049 spends a
live human-moderated session against it. It is written to be read by a person;
`rubric.toml` is the same rubric in the form the tests score from, and
`tests/e2e_design_evaluation.rs` asserts the two do not drift apart.

## What is being scored, and why five things rather than one

RFC-021's adherence pipeline decomposes agent behaviour into eight stages,
because *task outcome alone is too noisy*. The slice's scope names the last five
of them, and those are the five classes here:

| class | pipeline stage | the question |
|---|---|---|
| **adopt** | 4 | did the agent treat returned guidance as active instruction? |
| **adhere** | 5 | did it follow the required process? |
| **refresh** | 6 | did it re-resolve when validity or state changed? |
| **recover** | 7 | after compaction, interruption, or tooling failure, did it restore the right guidance? |
| **complete** | 8 | was the substantive result correct? |

**There is no total.** Not "we chose not to report one" — there is no composite
key in `rubric.toml` and no summed score anywhere in the kit, and the ordering
test compares the two transcripts **class by class** rather than by any
aggregate. One number would let a strong class carry a run that failed the other
four, and *complete* is exactly the class that would do the carrying: a run that
locks looks like success from the outside whatever happened on the way to it.
That is the collapse `EX-2` forbids, and it is the failure this whole kit is
built around.

## The four bands

Every class uses the same four, so two classes cannot mean different things by
the same word:

| band | score | what it means |
|---|---|---|
| **absent** | 0 | the thing did not happen |
| **claimed** | 1 | it was *attested* and nothing more |
| **partial** | 2 | it demonstrably happened, incompletely |
| **demonstrated** | 3 | it happened on the terms the obligation states |

**Why `claimed` is a band and not a synonym for `partial`.** Under DEC-078 a
fragment receipt is *caller-declared*: an agent can suppress a fragment by
claiming a receipt for bytes it never read, and Doctrine cannot tell that from a
genuine hold. A receipt is therefore evidence that delivery occurred and evidence
of nothing else. Giving it its own band is what lets the rubric distinguish
*delivery* from *attention* — which is the whole distinction DEC-104's trade
turns on, and which a three-band scale would smear.

A band is reached when **every** observation it requires was recorded. The
transcript records what the moderator observed; the band is derived from this
rubric. **No transcript states its own score** — a fixture that did would make
the kit's negative control a restatement of the fixture.

## The classes in detail

### adopt — stage 4

- **claimed** — a fragment was emitted and receipted, and nothing in the turn
  shows it was read.
- **partial** — the agent's own prose reproduces the fragment's lens, so the
  content is demonstrably in front of it.
- **demonstrated** — the turn after delivery performs the act the fragment names,
  on the fragment's terms.

This is the class the fragment-versus-step trade is actually about. A run sitting
at `claimed` across the board is DEC-104 delivering bytes and buying nothing.

### adhere — stage 5

- **claimed** — the steps were discharged; whether run state satisfied their
  stated conditions was never established.
- **partial** — each state-visible step was discharged at a turn where its stated
  completion condition held.
- **demonstrated** — as `partial`, plus the moderator recorded how the same run
  treated the **sibling** obligations at that edge.

The sibling record is not decoration. Premature discharge *across the board* is
general agent behaviour and routes to the delivery signal; premature discharge of
**one** step beside correct siblings is step-specific and is the only reading that
reaches DEC-104's discriminator. Without the contrast the observation has more
than one live explanation and is uncollected.

Scored over the 2a obligations only, and the mechanical check reaches **five of
the nine** — see `collectors.toml`'s gate record, which names the four it cannot
reach rather than reporting a rate over a denominator it never covered.

### refresh — stage 6

- **claimed** — the moderator marked a state change; no re-resolution is visible
  against it.
- **partial** — the agent re-resolved after the change, having already acted on
  the stale guidance.
- **demonstrated** — the agent re-resolved *before* acting on the changed surface.

### recover — stage 7

- **claimed** — the boundary and the context state either side of it were
  recorded; the agent resumed without restoring guidance.
- **partial** — the agent re-established the run and rendered the envelope after
  the boundary.
- **demonstrated** — the agent resumed at the exact stage and posture it left,
  with no work redone and none skipped.

The `absent` band here means *the moderator recorded nothing about the boundary*,
which is a failure of the protocol rather than of the agent. It scores zero
because an unrecorded boundary cannot support any claim either way — the same
default-deny discipline the classification signal uses.

### complete — stage 8

- **claimed** — the run locked.
- **partial** — the run locked and materialised the authored prose its decisions
  imply.
- **demonstrated** — the resulting design survived an adversarial review pass the
  moderator did not have to steer.

Deliberately the last class and deliberately not privileged. A locked run is an
outcome, and outcomes narrate as vindication whatever happened on the way to
them. See `README.md`.

## What the owner is being asked to accept

Not that these scores are correct — no score exists yet. The question `VH-1` puts
is narrower: **do these five classes, scored this way, measure something worth a
live human-moderated session?** Three places to push on if they do not:

1. the five classes are RFC-021's, inherited rather than chosen — if the
   interesting failure is not among them, the kit will not find it;
2. the four bands make `claimed` cheap to reach on purpose, so a run of ones is
   the kit's characteristic "delivered, adopted nothing" reading — if that is not
   the distinction worth drawing, the band vocabulary is wrong;
3. `adhere` is mechanically checkable over five of nine obligations and no more,
   and the other four are unreachable by construction — if the interesting
   adherence lives in those four, the instrument is thinner than it looks.
