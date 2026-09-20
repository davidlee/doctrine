# Implementation Plan SL-260: Design-review finding routing convention and trial

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Design §9.1 states seven things that must be true **at close**. They are not
phase criteria, and this plan's whole job is to distribute them: which phase
owes which, and what evidence discharges it.

| §9.1 | what must be true at close | owed by |
|---|---|---|
| 1 | the convention text is in all three files, provisional, citing `RFC-026` | `PHASE-01` `EX-1`/`EX-2`, `PHASE-02` `EX-1`/`EX-3` |
| 2 | a rebuilt-and-installed tree delivers it, verified through rendered output | `PHASE-01` `VA-1`, `PHASE-02` `VA-1` |
| 3 | every normative clause is stated in exactly one place | `PHASE-02` `EX-4`/`VA-2` |
| 4 | `CON-006` exists and is cited from all three surfaces; nothing implies enforcement it lacks | `PHASE-01` `EX-4`, `PHASE-02` `EX-5`/`VA-3` |
| 5 | the trial-report chore exists, gated `after`, citing `DEC-276` | `PHASE-03` `EX-1`–`EX-4` |
| 6 | `RFC-026` `P10`'s *Open* clause records where its items were settled | `PHASE-03` `EX-5` |
| 7 | no file under `src/` is touched | every phase's waived `VT` |

Item 3 has no exception, so it gets a real sweep with a positive control rather
than an assertion — and it can only run once all three surfaces exist, which is
what puts it at the end of `PHASE-02` rather than inside either text phase.

## Sequencing & Rationale

**Three phases, ordered by what each one makes possible for the next.**

`PHASE-01` lands the **normative owner** first. Both remaining shipped edits are
pointers at the fragment, so writing them before the thing they point at would
invert the dependency and make `PHASE-02`'s single-owner check meaningless —
there would be nothing to be the single owner *of*. The phase is deliberately
one file and two edits: the appended routing section, and the amendment to the
stand-alone rule that the routing section would otherwise silently contradict.
Shipping the first without the second would leave the file self-contradicting
for the length of a phase, which is why they are one unit of work and not two.

`PHASE-02` hangs the obligation at its other two firing moments and then proves
the shape. The two pointer edits are small; the phase's weight is in `VA-2`, the
sweep over every normative clause class §9.1 item 3 enumerates. That sweep is
the structural mitigation for `R9` (three surfaces drifting apart): the design's
answer to drift is that only one surface carries content, so the check that the
answer actually holds is the one thing this plan cannot delegate to a later
stage.

`PHASE-03` stands up the apparatus for a trial this slice will not run. It is
last because the chore is `after` `SL-260` and because it owns two things that
must not exist earlier: the capture rule, whose first firing moment is the first
eligible design pass; and `Q2` and `Q4`, which design §6 deliberately left
unminted because the chore is their intended owner and minting them sooner would
create exactly the second owner `RFC-026` `P2` exists to prevent.

**Why no separate verification phase.** Each phase verifies its own delivery
through rendered output, because the failure §9.2 item 2 guards against — a
stale embed shipping nothing (`R7`) — is silent, and a phase that ends without
having rendered its own edit cannot honestly claim to be done. Only the
cross-surface check waits, and it waits exactly as long as it must.

## Notes

**The tripwire, and why the plan states a different command than the design.**
Design §9.1 item 7 is a hard constraint on every phase: a `src/` change newly
binds `STD-001`, `STD-003` and `POL-002` and falsifies the no-tooling claim the
whole slice rests on. §9.2 verifies it with `git diff --stat <base>..HEAD --
src/`. That form is wrong on this branch, not in principle but in practice:
`edge` carries other slices' commits interleaved with this one's, so a
range-diff over `src/` reports their changes as this slice's. `plan.toml` uses
the commit-scoped form instead — no commit scoped `SL-260` touches `src/` — with
the pathspec-free run as its positive control. Same assertion, correct
instrument. The base is the parent of this slice's first commit, pinned in
`PHASE-01` `VT-2`.

**Why the `src/` rows are waived `VT`s.** The structured `VT` mandate is a
keyword-in-file gate, and a criterion asserting that *no* file changed has no
file for keywords to live in. `waived = true` with a stated reason is the
sanctioned shape for exactly this, and it keeps the design's declared mode
rather than quietly demoting the tripwire to an agent's judgement. The check is
a command, and it runs at every phase end and again at close.

**`CON-006` binds criterion authoring.** No criterion in this plan may assume a
check `CON-006` records as absent. In particular: nothing here treats the slice
close gate as establishing that a criterion was transcribed — it forces the
verify *act* and reads nothing. Where this plan wants a check, it names an agent
performing one (`VA`) or a command producing evidence (`VT`), never a shipped
mechanism that does not exist.

**Rendering a reviewing turn.** `PHASE-01` `VA-1` needs a design run at stage
`reviewing` to render against, because the fragment body rides `doctrine design
resume` and a locked run emits nothing. `SL-253`'s run is at that stage as this
plan is written; any run in that stage serves, and `resume` is a pure read, so
this borrows another slice's run without touching it. `EN-3` makes the
availability of such a run an entrance condition rather than a surprise at
verification time — and routes to `/consult` if none exists, because the
fallback a hurried agent would reach for (reading `install/` directly) is
precisely the check §9.2 item 2 forbids.

**The single-owner sweep spans two corpora, not one.** §9.1 item 3 enumerates
twelve normative clause classes, and ten of them are shipped text — those are
swept over `install/` and `plugins/` for exactly one hit each. The other two,
the eligibility rules and the collection procedure, are not shipped at all:
their owners are the slice scope and `DEC-276`/design §9.4, and the only way
this slice could give either a second owner is by letting the trial chore
restate it. A sweep of the shipped tree would return zero for them and read as
a pass. So they are checked where the risk actually is, against the chore, in
`PHASE-03`.

**`doctrine install` is read before it is run.** `PHASE-02` needs the installed
plan skill to verify delivery, but a bare `install` rewrites every skill in the
primary tree. The criterion runs `--dry-run` first and then narrows to the one
skill under test. The ledger doc needs no install at all — `library show` reads
the embedded asset, so `cargo build` is the whole precondition there.

**`PHASE-02` edits the skill that governs this stage.** The transcription
pointer lands in `plugins/doctrine/skills/plan/SKILL.md`, which is the `/plan`
skill itself. The edit changes what a *future* planner is told; it does not
retroactively bind this plan, and `SL-260` is excluded from its own trial by
construction (scope §5). Worth knowing before the installed skill changes under
a later session mid-stage.

**One loose end this plan adopts rather than leaves.** The thirteen `DEC`
records this slice minted still read `proposed` while the design that they shape
is locked; the corpus's settled decisions read otherwise. No later stage owns
the transition — `/reconcile` and `/close` do not touch knowledge-record status
— so `PHASE-03` `EX-7` discharges it instead of letting close find it. It costs
one command and is not new scope.

**The `VT` keyword floors were checked non-vacuous.** Every keyword named in a
`VT` mandate returns zero occurrences in its target file today, so each floor
asserts something the phase must actually add rather than passing on text that
was already there.

**What this plan does not schedule.** The trial itself, and any judgement about
its outcome — design §9.3. Closure depends on the convention being in effect and
mechanically checkable, on nothing that postdates this slice.
