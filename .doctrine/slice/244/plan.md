# Implementation Plan SL-244: Gate conditions carry their own contract

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

The design replaces the design-run stage gate's core model: a payload-free
`Condition` whose satisfaction is an existential scan over evidence rows becomes
a generated contract table, eight named acts, and one derivation that returns a
diagnosis. Around that centre sit four things that can each be built and proved
on their own — a closed forward relation, a generalised coverage comparison, the
run's review policy, and a first-class review pass — and three that can only be
built after it: the readable contract, the two warning lamps, and the published
diagram.

That shape is what the eight phases are. The plan's whole sequencing argument is
about **which side of the centre each piece falls on**, and the answer is not
obvious in either direction: two pieces that look downstream of the model are
actually upstream of it, and one piece that looks separable is not separable at
all.

## Sequencing & Rationale

### Four phases before the model, and why each is genuinely before it

`PHASE-01` closes the forward relation. The design keys its condition source by
`Advance` — a closed four-member type — precisely so that a row cannot name an
unlawful transition and a lawful transition cannot go unguarded. That key has to
exist before the source it keys. It is also the cheapest possible first phase:
two hand-written stage-pair matches collapse into one, `forward_runbook` stops
re-deriving a table it could read, and nothing observable moves. A changed test
in that diff is a defect in the refactor, which is why its behaviour-preservation
check is written as an agent check over the diff rather than as a test.

`PHASE-02` generalises `ContentCoverage` and adds the inquiry-map material. Both
are prerequisites of the act records rather than consequences of them: an act's
`covered` slot is typed by the generalised comparison, and `CoverageStale`'s
ability to name the node that moved is exactly what `diff` buys. The design chose
material over a digest for this map — nodes are mutated by pure code after any
shell digest would have been taken — and the diff is the payoff that choice was
made for. Doing it here means the incumbent's two users prove the generification
is behaviour-preserving before anything depends on the new instantiation.

`PHASE-03` builds the run's review policy. This is the phase most likely to be
questioned, because `ISS-310` reads as part of the condition model — and it is
not. `DEC-073` already decided the policy; what is missing is that nobody built
it, so `Reviewer` is recorded on every attestation and read by nothing. Repairing
that is a self-contained change to an incumbent derivation, testable on its own
terms, and it delivers `RequiredActor::RunPolicy` a resolvable value to resolve
*to* before any rule names it. Sequenced after the model instead, it would have
to be threaded back through a table that was already written assuming a fixed
actor.

`PHASE-04` mints the review pass. This is the other piece that looks downstream
and is not. `review-disposition-attested` binds to the run's *current* pass, so
without a pass there is nothing for a disposition to be given over and
`reviewing → locked` becomes uncrossable the moment the vocabulary flips. The
mint rides `DEC-086`'s journalled intent, which is live in the command tier
today and wired for three record kinds; adding the `RV` half is the smallest of
the three things that seam decomposes into. Landing it here also lets the phase
retire `IntegratedReview` in place while the gate's behaviour is still the
incumbent's — a clean before/after, rather than one migration hidden inside a
larger one.

### One phase that cannot be split, and the honest reason

`PHASE-05` is the slice's centre and is deliberately its largest phase. The
temptation is to cut it in three — tables, then records, then evaluation — and
that cut does not exist. The rule names an act *kind*; the record is constructed
in the shape its rule's coverage names; admission checks the correspondence
between the two; and the evaluator reads the records through the rules. Any
ordering of those four leaves one of them stubbed. Worse, the vocabulary flip is
atomic: the moment `drafting-readiness-attested` arrives and
`required-sections-exist` retires, a run that cannot record the new acts cannot
cross the edges those acts guard, and the e2e suites go red with no legitimate
repair available until the acts land. Splitting the phase would therefore buy
smaller diffs at the cost of either a parallel implementation or a period of
green-chasing — and this project's standards forbid the first outright.

So the phase is large and the control is placed elsewhere. Its entrance criteria
require the runtime phase sheet to carry an ordered task breakdown cut so that
each task ends with the crate compiling, and its verification carries two agent
checks rather than only tests: one that every changed e2e test is argued by
fixture and by condition, and one that applies `RV-344` pass 2's
parallel-implementation lens to the phase's own diff with the suspects named in
advance. Two of those suspects are pairs the design deliberately keeps separate —
whole-map currency beside the per-section quantification, and the two blocker
filters — and a reviewer who does not know they are intentional will try to merge
them.

### Three phases after the model

`PHASE-06` makes the contract readable. It is after the model because it renders
the model: the receipt injects structure from the const table and the narrative
asset carries only what the const cannot say, which is the whole of `DEC-123`'s
injection rule. It is also where the slice's stated acceptance test first becomes
true — an agent can learn what a transition requires without reading source — so
that test is written as a human check on this phase rather than deferred to
close, where nobody would take it.

`PHASE-07` ships the two warning lamps. They are last among the runtime surfaces
because they are the design's own boundary case: derived facts that must inform a
decision without barring it. Building them beside the conditions invites treating
them as conditions; building them after, against a vocabulary that is already
fixed at nine, makes the exclusion structural. The phase is small, and it is
sequenced after `PHASE-05` has already retired the envelope's uncapped
`clearances` neighbour, so its additions are measured against the budget that
retirement leaves rather than against the one the design was written under.

`PHASE-08` publishes the stage-machine diagram. It is last because it is pinned
to the finished tables — the golden test is only meaningful once the vocabulary
has stopped moving — and because it is the one deliverable with no dependent.
It is not optional and not a write-up: `SL-233`'s failure was attending to
everything except the interaction design that was most of its point, and shipping
the machine's own description is the corrective.

### What this plan does not carry

Three things belong to the slice and to no phase, and are named here so they are
not discovered at close.

The `SPEC-029` revision is `/reconcile`'s under `ADR-013`. Five of that spec's
stated responsibilities move, and `sec-1` of the design already holds the
work-list; a phase authoring it would be a phase writing governance, which is not
what phases do.

`SL-243` owes five acts by hand at its next forward move. That falls due when
this code lands, not before, and it is not this slice's phases to perform. The
user priced and accepted that bill on 2026-08-04.

`IMP-391` and `IMP-392` arrive partly done and their bodies say so. Nothing in
these phases re-plans their delivered halves: the wire acts and the artefact
storage are `PHASE-05`'s, the `RV` mint is `PHASE-04`'s, and what remains to
those items is the interaction and the finding-set migration respectively.

## Notes

### Two premises this plan checked rather than inherited

The design was locked at author-time and the tree has moved since, so every
concrete `file:line` it load-bears on was resolved against the current tree
before these phases were cut. All of them hold, including the two cited by bare
module name, which are `src/design_run/ids.rs` and `src/design_run/prompt.rs`.
No phase is scaffolded against a stale path. This is the second of `RV-344`
pass 2's four lines of attack, discharged at planning time because that is where
it was cheapest; the other three land inside the phases that touch each surface.

The design's macro-generated tables have **no in-tree precedent** — there is no
`macro_rules!` anywhere in `src/`. That is not an objection to the design, whose
argument for generation over assertion is sound and whose alternatives it weighed
explicitly. It is a note that `PHASE-05` introduces a mechanism this codebase has
never used, so the first task in its breakdown should be the macro against a
throwaway row, not the macro against nine real ones.

### One dependency the plan flags rather than re-decides

The design records `review-disposition-attested`'s `Conducted` arm as needing two
things from `IMP-392`: the concluded-pass marker, and the finding set its blocker
predicate reads. The first is unambiguous — the marker does not exist, no verb
sets it, and `sec-4` specifies its shape for that item to build.

The second is worth checking rather than assuming, and `PHASE-04` is where it
gets checked. `doc_unresolved_blockers` already reads an `RV`'s findings by
severity and status today, and the design itself describes the predicate this
slice needs as "the same test minus the state restriction". If that read is
right, `ObservedReview::undisposed_blockers` is buildable now and only its
*admissibility* half waits on the marker — which would make the interim narrower
than the design states, in the slice's favour. `PHASE-04`'s exit criteria are
written to build the predicate either way; what the phase must not do is inherit
the wider claim without testing it. This is `RV-344` pass 2's
unbuilt-versus-misread line, landed on the phase that touches the surface.

**Settled, 2026-08-04 (`PHASE-04` `EX-7`).** The read is right: the finding set is
readable today. `FindingStatus` already carries `Open` and `Contested`
(`src/review.rs`), `parse_finding_status` and `Severity::parse` are both live, and
the predicate needs nothing else. `IMP-392` is therefore narrower than the design
states — the **concluded-pass marker alone**. `PHASE-05`'s `VA-3` clause fires:
its second and third deferred assertions land live in `PHASE-05` rather than
waiting on `IMP-392`; only the first (`Conducted` over an unconcluded `RV` refused
at admission) stays deferred.

**One criterion amended at the same time.** `PHASE-04`'s `VT-4` originally also
required *the gate reads that as unmet*, which presumes a condition reading
`ObservedReview`. That condition is `review-disposition-attested` and it lands in
`PHASE-05`, together with the act that names a review at all — so no reader exists
in `PHASE-04` and the clause could only have been satisfied hollowly. `PHASE-05`'s
`VT-9` already carries it verbatim (*a stored act naming an unreadable RV is
`ReviewUnavailable` and names no findings*), so the clause is dropped rather than
duplicated, and `VT-4`'s `test_file` moves to `src/review.rs` — the tier where an
unreadable ref can actually exist. The row now tracks where its evidence lives.

### Three risks the phases carry, named rather than absorbed

**`PHASE-04` extends a journal vocabulary, not just a call site.** Minting an
`RV` through `DEC-086`'s seam means the recovery intent has to be able to say
*this submission is claiming an `RV`*, and today it says `DEC`, `QUE` or `ASM`.
That is a change to the intent's own vocabulary and to the journal format a
half-finished submission is resumed from. The design's argument that nothing in
`DEC-086`'s decomposition resists it is sound, and it is an argument about
whether the seam *admits* a fourth kind, not about whether adding one is free.
The phase's agent check is written to exercise the recovery path under
interruption rather than reason about it, because that is where a journal-format
change actually bites.

**`PHASE-03` and `PHASE-04` both edit a type `PHASE-05` retires.**
`ReviewStanding`'s four booleans become derivation rules, so the struct does not
survive the model. This is churn and it is worth being explicit that it is the
carrier being replaced and not the work: the nested lane quantification
`PHASE-03` writes is the body of `PerSection`'s derivation, and the currency
comparison `PHASE-04` re-sources onto `ReviewPass` is the body of the lamp. The
tests written against both survive unchanged. Ordering the policy after the model
instead would mean writing `RequiredActor::RunPolicy` against a policy that does
not exist yet, which trades real churn for a real fiction.

**The research advisory reports drift, and it is benign.** The baseline moved
because this slice's own `design.md` landed and its scope was amended by
`DEC-139` — not because the tree moved under the research. It is left standing
rather than restamped, because restamping without refreshing the threads would
attest a currency nobody established. `/phase-plan` should read it as what it
is: a record that the artefact predates the design it fed, not a signal to
re-run research.

### The interim states these phases knowingly pass through

Between `PHASE-04` and `PHASE-05` the run holds a review pass nothing derives
over, and between `PHASE-05` and `PHASE-07` it derives warnings nothing renders.
Both are states where the tree is coherent and incomplete rather than
dual-pathed, which is the distinction that matters: at no point do two
implementations of one question coexist. Where a phase leaves new state with no
reader, it is marked with the tree's existing slice-tagged `expect(dead_code)`
idiom naming the phase that will read it, so an unused warning is never silenced
without an address.

The snapshot is the one place where "coherent and incomplete" has a cost outside
the tree, and it is confined to a single phase on purpose. `PHASE-02` through
`PHASE-04` add serde-defaulted fields and remove one key an existing snapshot can
carry unread, so a live run parses throughout; `PHASE-05` is where the shape
moves and `SL-243`'s five-act hand repair falls due. Breaking it once,
deliberately, is cheaper than three partial migrations of a gitignored tier.

### Per-phase completion

Every phase ends on the project's own bar rather than on its criteria alone:
`doctrine check gate` green with zero clippy warnings, `cargo fmt` clean, the
phase's notes current, and the phase flipped in the state tree. That is not
restated per phase in `plan.toml` — it is the standing convention, and a phase
that meets its exit criteria on a red gate has not finished.
