# DEC-101: Obligation runbooks: ordered steps as asset data, verifiers as executables

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The full design is `.doctrine/slice/233/sketches/runbook-runner.md`. This record
carries the decision and the two rulings that shape it; the sketch carries the
shape, the worked asset, and the phase boundary.

## The authoring rule

Recorded here because without it a runbook accretes invariant-restatements —
which is exactly how the four shipped process fragments ended up stating what
the machine refuses instead of what to do next.

> **Could a project legitimately do this differently?**
> **Yes** → runbook step. Overridable, verifier substitutable.
> **No** → engine invariant. Enforced by `apply` / `advance`. Never a step.

True invariants stay out of runbooks altogether. A user cannot override "every
mutation compare-and-swaps against the run's revision" — the code does that
regardless, so an override would not be *different*, it would be *false*. This is
the same line DEC-102 draws for customization, which is fair evidence it is the
real one.

## Identity: reference is not equivalence

A durable step id lets you say *which* step you mean. It does not say whether the
step you mean today is the same obligation an earlier discharge was made against.
Those are different problems, and conflating them was the sharpest defect the
adversarial review found in this design's first revision — which had gone as far
as calling verifier substitution "safe by construction" on the strength of the id
alone. It is not: the id carries the contract's *name*, never the contract.

So **a discharge binds the digest of the step's definition** — `id`, `text`,
`required`, `verify` — and goes stale by construction when any of them moves.
Editing a step's prose, flipping it to required, or substituting its verifier all
invalidate the discharge and force it to be re-made under the contract that
actually holds. Deleting a step and later reusing its id cannot resurrect the old
record, because the digest differs; re-adding it byte-identically legitimately
does, because the obligation genuinely did not change.

This is not new machinery. `Attestation` binds a subject to the exact fingerprint
reviewed, and DEC-066 liveness expires it the moment that content moves — content
binding with stale-by-construction semantics, already built.

*Corrected at revision 3:* earlier revisions of this record wrote that "section
attestations already bind payload fingerprint, disposition, node and revision",
citing `reviewing.md`. They do not — `Attestation` is `{id, subject, fingerprint,
reviewer}`. The four-part binding is `AcceptanceAttestation`'s derived digest. The
shipped `reviewing.md` fragment states the same error, which is how it propagated;
it is filed as backlog rather than fixed here, since PHASE-08 owns fragment bodies.

Granularity is **per step**, not per asset. A whole-runbook digest would reset
every discharge because someone fixed a typo in the last step.

## Progression is attestation

There is no verb meaning "move on" without a name attached. You advance by
attesting a **named** step. `sequence` and `set` are two admission rules over that
one act, not one rule behind a flag: a `sequence` holds a cursor and refuses any
discharge that does not name the step at it; a `set` holds no cursor and derives
no refusal from ordering. Both gate identically on every required step being
discharged.

**PHASE-16 ships `sequence` only.** `set` moves to PHASE-08, which converts the
two genuinely coverage-set checklists and will have real instances to design it
against. Shipping it earlier would land a mode with no user and — since the
override seam is deferred — no end-to-end test able to reach it. The distinction
above is unchanged and is the argument for adding it then; only the delivery was
premature.

> **Amended 2026-08-01** (owner ruling; RV-325 round 5 F-11). **The assignment
> of `set` to PHASE-08 is withdrawn.** This is an amendment to an accepted
> decision rather than a reading of it, because PHASE-08's `EN-2` gate cannot
> narrow a commitment this record made.
>
> **What changed under the assignment.** SL-233's D14 reclassified both named
> checklists — present-design's fifteen content items and the adversarial attack
> list — as stage framing (fragment prose), so the two conversions this paragraph
> promised PHASE-08 no longer happen. RV-325 F-5 then settled a runbook on edge 2
> at two steps, and edge 4 carries three: **five order-independent steps shipping
> under `sequence`**. So the precondition named here — *real instances to design
> it against* — is **met**, by different instances than the ones named.
>
> **`set` is nonetheless deferred out of PHASE-08.** Not for want of a user: that
> warrant is withdrawn as false wherever it appears. `set`'s *admission* is cheap
> (`run.rs:1369-1377`, plus a `Mode` variant referenced nowhere outside
> `runbook.rs`). Its *render* is the open question — what a **cursorless** runbook
> renders under `EX-14`'s token bound — and it is unsketched and outside `EN-2`'s
> scope. Landing it in PHASE-08 would hand that design choice to implementation,
> which F-5 ruled is the one thing the gate exists to prevent.
>
> **The distinction in the paragraph above is untouched.** `sequence` and `set`
> remain two admission rules over one act. What moved is only *when* the second
> arrives, and *who owns it*: **IMP-373**, carrying the render question and an
> evidence-triggered repayment condition — observed `DischargeNotAtCursor`
> refusals (`refusal.rs:198`) against the edge-2/edge-4 runbooks, collected in
> PHASE-09's exercise.
>
> **The imposed order is conceded, not defended.** *"Imposing an order on a
> coverage set is fake determinism"* applies to those five steps at reduced
> scale, and is accepted as cheap. It is not argued to be meaningful.
>
> PHASE-16 `EX-7` and `EX-14` carry the same forward assignment and are annotated
> to match; neither discharged obligation changes.

The consequence is that the mechanism's honest limit is structural rather than
documentary:

- a step **with** a verifier — the attestation is corroborated by an exit code,
  and the verifier's output is recorded alongside it;
- a step **without** one — the attestation stands alone: recorded,
  revision-bound, auditable, and *believed*;
- a step that cannot be done — **discharge-with-reason**, reason required,
  rendered in the envelope and kept in the record.

The third case is load-bearing, not a nicety. Once the gate blocks on required
steps, a project whose runbook carries a step its agent cannot satisfy would
otherwise wedge the run. A disclosed deviation beats both a wedged machine and a
silent skip.

## The trust boundary — decided, not slipped

An executable in a projected asset means `doctrine design apply` runs
project-authored code.

Accepted. The runbook is committed alongside the repository's own `justfile` and
CI configuration, at exactly that trust level — an agent already runs those. The
owner's ruling: *"if an attacker can write hymns, all agents are compromised and
we're fucked anyway."*

At v1 the surface is narrower than that sentence implies: [[DEC-102]] defers the
override seam, so the only runbook that runs is the **shipped embedded** one. The
ruling is unchanged and holds when the seam lands.

Gating custom executables behind a `doctrine.toml` opt-in was considered and
rejected as theatre: an attacker who can write your runbook can write your
justfile.

## What this deliberately does not claim

`/canon` and `/retrieve-memory` have no possible verifier — no exit code proves
an agent read something. The runner guarantees **sequenced, gated and
auditable**; only some steps are additionally **verified**.

Stated at this volume because this slice has been bitten more than once by a
surface claiming detection it did not have — [[DEC-100]]'s watermark contract,
and RV-321 F-3's over-claim followed by the same shape one level up in the
record of F-3's own remedy.

## Where the mechanism is not allowed to go

Three seams the design names explicitly, because each was got wrong at least once.

**The clearance clause belongs on `gate::advance`, not `gate::can_advance` — and
it is a third derived input, not a new `Condition`.** `can_advance` decides which
stage *edges* are legal and nothing else; clearance is `advance`. But the closed
`Condition` set cannot hold this clause: it is payload-free with nowhere for a
step identity, its satisfaction test is existential where the gate needs *every*
required step, and its refusal carries only conditions. Every guard in the machine
today is a **static** conjunction fixed at compile time; a runbook guard is a
**dynamic** one whose members come from an asset and carry identity. So `advance`
gains a runbook standing beside `ReviewStanding` — its precedent — and one refusal
variant that names the outstanding steps.

**Truth does not flow from user-customisable material into the closed vocabulary.**
The runbook does not derive, satisfy, or feed the incumbent conditions. Once the
override seam lands a project could delete a step and a built-in condition would
derive true with no work done — the name still promising what it no longer
delivers. There is also nothing to reconcile: `governing-context-recorded` and
`initial-concerns-recorded` are stubs, specified in nine words, named in no
shipped guidance, and bound to an arbitrary draft section's fingerprint.

**No subprocess runs inside `apply`'s admit→persist span.** `apply` reads one
snapshot, and the only pre-write recheck covers the authored watermark before the
snapshot write. That window is negligible today purely because nothing slow is in
it. Verifiers therefore run shell-side and their results enter through
`DerivedInput` — *not* through caller-authored payload, which has no slot for them
and whose obvious workaround is refused outright, because a verifier result is
precisely a fact Doctrine must derive rather than accept on a caller's word.
Gate-time re-verification obeys the same rule.

**And the runbook guard is evaluated per-edge, not cumulatively.** A discharge
that goes stale after its stage cleared **warns**; it does not block and does not
un-advance. It is the first non-cumulative condition in the machine, which is why
it is written down rather than left to whoever implements it first.

## Related

- [[DEC-077]] — the v1 thin-skill-and-prompt-pack decision this narrows.
- [[DEC-102]] — the customization line, decided under the same authoring rule.
- [[DEC-100]] — the watermark contract; the over-claim precedent §*does not
  claim* answers to.
