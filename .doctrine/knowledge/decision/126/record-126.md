# DEC-126: What the design-run gate should check

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The answer, before the table

**The design-run gate is overwhelmingly an attestation ledger, not a checker.**

Design quality is not mechanically decidable. The six caller-claimed conditions
are what happened when the machine pretended otherwise: it needed a check, could
not write one, and accepted an assertion instead. `DEC-120` names that tier as the
defect class; this record empties it.

The engine's real job at these boundaries is fourfold — make sure the judgements
happened, bind each to the content it was made over, present the state accurately
enough that the judgement is informed, and record it so an auditor can see what
was accepted and why.

## The line

Not *is it a judgement* — nearly everything here is. The discriminator that does
work is: **does the actor's identity matter?**

- **Derived** — the engine recomputes the answer from state it holds, and *who*
  authored that state is irrelevant to the question asked.
- **Attested** — the point of the condition is that a **named actor** rendered a
  judgement. `Reviewer::Human` vs `Reviewer::Adversarial` is recorded precisely
  because it matters (`attestation.rs:20-28`).

## The classification

Ten conditions in, **nine out — two Derived, seven Attested, zero Claimed.**

| condition | today | kind | note |
|---|---|---|---|
| `governing-context-recorded` | claimed | **Attested** | `DEC-121` — the confirmed governance edge set |
| `initial-concerns-recorded` | claimed | **Attested** | `DEC-121` — reviewed graph + declared blocking set |
| `blocking-inquiries-dispositioned` | claimed | **Derived** | dispositions must exist; whose is irrelevant |
| `user-accepts-sufficiency` | claimed | **Attested** | `DEC-088`'s mechanism, generalised |
| ~~`required-sections-exist`~~ | claimed | **retired** | replaced by a drafting-readiness attestation |
| `materialisation-current` | claimed | **Derived** | engine-authored on both sides |
| `section-attestations-current` | derived | **Attested** | reviewer identity is recorded and opt-in |
| ~~`integrated-review-present`~~ | derived | **replaced** | → `review-disposition-attested` |
| ~~`blocking-findings-disposed`~~ | derived | **folded** | → `review-disposition-attested` |
| `user-acceptance-attested` | derived | **Attested** | `DEC-088` |
| *(new)* `drafting-readiness-attested` | — | **Attested** | the author says this is ready to review |
| *(new)* `review-disposition-attested` | — | **Attested** | see below |

Today's four "derived" and the proposed two Derived are **substantially
disjoint** — three of the four become Attested. The old boolean was not a weak
version of this line; it was a different line, tracking implementation coverage
rather than the nature of the condition (`DEC-120`).

This corrects `IMP-361` (*"derive the six remaining gate conditions"*): the answer
is three derive, three attest, one retires. Deriving all six would compute two
human acts and one adversarial one — the error `DEC-120` names.

## Why `required-sections-exist` retires

It has **no implementation**. Grep returns the enum variant, the
`boundary_conditions` row, the kebab token, and tests that record a *claim* about
it — there is no required-section list, no spine, no template anywhere in
`src/design_run/`. Sections are agent-declared at will.

Nor can one be written: which sections a design document must have is exactly what
a project would legitimately disagree about, so under `DEC-102` it is craft and
Doctrine owning a mandatory list is the wrong shape. And section *existence*
cannot express the meaning wanted — a section can exist with an empty body.

Retiring it outright would leave `drafting → reviewing` guarded by
`materialisation-current` alone, which is trivially true of an empty document. So
it is replaced, not deleted: **`drafting-readiness-attested`**, the same shape as
`user-accepts-sufficiency` one stage later. The author says this is ready to
review; the engine derives over that. No list anyone would argue about.

## Why the two review conditions collapse into one attested act

**Review terminates when the user declines another round, not at a fixpoint.**
`ContentCoverage::is_current` is whole-map equality (`attestation.rs:210-214`), and
both `LockAcceptance` and `IntegratedReview` bind that way — so a *productive*
integrated pass invalidates its own clearance, and the gate as built can only be
cleared by a pass that changes nothing.

That is not a safety property. It is a termination rule the project's own evidence
refutes. `RFC-026` **E3** — *"SL-232: a five-round review that provably did not
converge"* — records five rounds each refuting the previous one's repairs, and the
ruling *"Do not open round 6. The next move is a design decision, not a review
round."* `DEC-101`'s notes record the same shape one slice over.

Both conditions also collapse a **decision** into a **state**, the pathology
`DEC-121` found on the exploring edge:

- *whether to run a pass, by whom, or to waive, self-review, or stop after N
  rounds* — a decision, currently unrecorded;
- *whether a pass covers current content* — a state;
- *what remains outstanding* — a state, and `blocking` defaults to `false`, so a
  finding is non-blocking by omission. Zero outstanding **blockers** is entirely
  compatible with six major, five minor and eleven nits.

So they fold into one **`review-disposition-attested`** (Attested, blocking),
informed by derived state that **warns rather than blocks**:

- integrated-pass currency — stale means *the last pass predates your latest
  edits*, not *refused*;
- an outstanding-findings summary **by severity**, presenting everything and not
  only what someone flagged. That dissolves the default-`false` problem without
  flipping the default: you see the six major whether or not anyone ticked a box,
  and you attest in full view of them.

Warn-not-block has an in-repo precedent built for exactly this: `DEC-101`'s
*"a stale discharge WARNS, it does not block and does not un-advance. It is the
first non-cumulative condition in the machine."* This is the second.

**The disposition's shape** follows `DischargeClaim` — arms plus an
admission-checked reason: `Conducted { review }` and `Waived { reason }`. Two, not
five. Self-review is `Conducted` citing an `RV` whose reviewer is the user;
section-by-section substitution is `Waived` naming what replaced it; *rounds
sufficient, stopping* needs no arm at all once staleness only warns. A third arm
waits for a real query to force it, per `DEC-101`'s own discipline.

The obvious objection — this lets someone attest "enough" having run nothing —
does not hold in the way that matters. The vocabulary distinguishes conducted from
waived, both carry reasons, and the record is what an auditor reads. The engine
stops pretending to enforce what it cannot define and records what actually
happened.

## Dependency

`Conducted { review }` and the severity summary both require `DEC-125`: today the
run cannot hold or resolve an `RV`, and its own finding record has no severity
field. This classification is specified against the **`RV`-backed** model.

Related: `DEC-120` (the kinds), `DEC-121` (the exploring pair), `DEC-124` (channels
— the refusal carries the remedy), `DEC-125` (the finding model this depends on),
`DEC-088`, `DEC-101`, `DEC-102`, `RFC-026` E3, `ISS-285`, `ISS-286`, `IMP-361`,
`IMP-390`.
