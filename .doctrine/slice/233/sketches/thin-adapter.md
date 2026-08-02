# Sketch: the thin design adapter

PHASE-08 `EN-2`. Companion to `sketches/target-machine.d2` (diagram C), which
carries the same dispositions in visual form and is the primary artefact — this
document is its prose, written to be attacked.

## Why this gate exists here

RV-315 F-17 established that **no specification owns a skill body**. PRD-003
disclaims what a skill *says*; SPEC-010 treats a skill as opaque payload. So
rewriting `plugins/doctrine/skills/design/SKILL.md` from an eight-state workflow
machine into a thin activation/recovery adapter is a real design act with no
governing artefact above it. A sketch plus a non-author RV is the only scrutiny
it gets, which is why `EN-2` is an entrance criterion rather than a child slice
(plan.md, *The three design gates*).

## Posture: this sketch asserts; it did not drive

The owner's ruling of 2026-07-30 rejected the six-question sketch **as a
driver**:

> "(e) in the bin, and the rest are not much guarantee of a useful outcome … a
> setup to spend the rest of the week's tokens without getting this thing
> shipped."

Order was inverted to **design conversation first, paperwork back-solved**. The
sketch and its non-author RV still land, because `EN-2` is authored and criteria
ids are immutable; what changed is that this document records what was
established rather than being the instrument that establishes it.

That conversation happened across two sittings. It produced DEC-101 (obligation
runbooks — ordered steps as asset data, verifiers as executables), DEC-102
(craft is overridable, invariants stay sealed), the PHASE-16 implementation
those two decisions specified, and — on 2026-08-01 — twelve dispositions
recorded in diagram C as **D1–D12**. Every answer below cites one of them. Where
this document and diagram C disagree, **the diagram governs**; it is the one
under version control as the decision record.

**Two further decisions arrived after the dispositions and reshaped several of
them.** **DEC-103** — *instruction is delivered at the point of effect* — was ruled
on 2026-08-01 in response to a self-check pass on this sketch, and it is the
reason D8, D10 and D11 read differently here than they did when first recorded.
**D14** was ruled later the same day, in response to RV-325 F-2: *a set of
heuristics to apply while doing the work is not a checklist to tick off; it is
framing for the activity, and an exit gate delivers it when the damage is done.*
It is recorded as **DEC-104**, a separate record rather than an amendment —
DEC-103 governs *when* an obligation is delivered, DEC-104 governs *what kind of
asset carries it*, and conflating the two is the error DEC-104 corrects.
D14 is the larger of the two in effect — it splits tier 2 into steps and
fragments, empties the edge-3 runbook of all but one obligation, and moves
`:111-113` out of the skill altogether.

Effects are marked **(DEC-103)** or **(D14)** at each site. Both rulings were
provoked *by* this document's own worst reasoning — DEC-103 by two failed
deletions, D14 by a nineteen-step gate this sketch had talked itself into calling
cheap. That is disclosed rather than tidied away.

## The governing claim

> The incumbent skill is not a workflow machine that needs porting. It is a
> **prose imitation of a machine that already exists in Rust**, and the rewrite
> is mostly deletion.

Everything here follows from that. If the reviewer breaks it, the phase's shape
changes rather than its details.

## The frame: four destinations, four edges

Two structural facts do the load-bearing work, and neither is invented here.

**The mediation axis is four destinations, and the rule separating the first two
is already shipped** — `install/design-prompts/exploring.toml:8-13`:

> THE AUTHORING RULE. Could a project legitimately do this differently?
> Yes → a runbook step. Overridable, verifier substitutable.
> No → an engine invariant, enforced by `apply` / `advance`.

DEC-102 states the same line on the asset-policy axis: an asset is sealed when a
project override would make its content **false** rather than merely different.

**Note what that rule does not have: a prose branch.** It is total over its two
answers, and an earlier revision of this sketch nonetheless described tier 3 as
"what survives both tests". Nothing survives both tests. Tier 3 was invented
here, unauthorised by the rule this document opens with — and most of the (b)
answer was spent legitimising what lands in it.

**And the rule is missing a distinction its own store already draws (D14).** The
`Yes` branch names *a runbook step* as though tier 2 were one thing. It is two,
and the code says so in as many words: `runbook.rs` imports `STORE` from
`prompt.rs` because *"a runbook is the structured sibling of the prose it guards,
in the same store, so spelling the directory twice would be two sources for one
address"* (`runbook.rs:20-24`). One directory, two asset kinds:

| | **2a — runbook step** (`*.toml`) | **2b — stage fragment** (`*.md`) |
|---|---|---|
| selected by | `boundary_runbook(from, to)` — an **edge** | `Fragment::for_stage(stage)` — a **stage** (`prompt.rs:107-115`) |
| delivered | at the advance attempt, once | **every turn you are in the stage**, re-emitted unless the agent holds an exact `name@digest` receipt (DEC-078) |
| discharged | yes — cursor, CAS turn, gate-blocking | no — it is framing, nothing to tick |
| strongest outcome | `verified`, but only where a verifier exits zero | n/a |

**2b is the destination this sketch was missing, and its absence is what forced
tier 3 into existence.** An earlier revision had no way to say *"this is framing
for an activity"* and so wrote every such obligation either into a runbook step
(ceremony) or into retained prose (residue). Both were wrong for the same class
of content.

**DEC-103 (2026-08-01) supplies the missing clause and reshapes this sketch:**

> Instruction — and guardrails governing invariants the system does not itself
> enforce — is **delivered at the moment it must take effect.** Prose is not a
> destination; it is a *failure to locate a delivery moment*, and where it
> remains it is labelled as such.

Three corollaries do the work below. **DRY is the wrong model for agent
instruction** — its premise, stated once and referenced perfectly thereafter, is
false for agents, so "something with more authority already says this" only
warrants a deletion when that authority fires *where and when the obligation
bites*. **Multi-hook is a reason to deliver repeatedly, not to stop delivering.**
And **general vigilance is the weakest available form**: "be on the lookout for
X" is worth little where "apply this lens to this artefact, now" is worth a great
deal.

So tier 3 keeps its place on the axis but changes meaning. It is a **residue
register**, not a destination, and every item in it must carry *why no delivery
moment was found*. Shrinking it is still the point of the rewrite; what is new is
that the items which remain are no longer defensible merely because a triage
chose to leave them.

**The 2a/2b discriminator, and why it is decidable rather than a matter of
taste.** Ask one question of each obligation:

> **Can this be truthfully completed at a specific boundary, with a finite
> result whose completion is necessary before advancing?**

- **Yes — a finite, gate-worthy act.** Recording selectors; reconciling the
  slice scope against the decisions this run accepted. It earns **2a**, its own
  discharge.
- **No — a continuous heuristic or quality lens with no truthful completion
  point.** *Consider data-flow boundaries*, *don't hide assumptions in polished
  prose*. There is no moment at which an agent has finished holding these in
  mind; they frame the whole activity. It is **2b**.

**Verification strength is a second, orthogonal axis — not the sorting rule.**
`DischargeOutcome` has three arms in the record but **two on the wire**: a caller
may say `attested` or `skipped` and **may never say `verified`**, because that
word is reachable only through a check that exited zero, so *"a step carrying no
verifier cannot acquire the stronger word by any route"* (`runbook.rs:604-606,
745-752`). A verifier therefore *strengthens* a step's discharge; its
availability does not decide whether the obligation is a step. **Codicil:** a 2a
step shipping no verifier must state why mandatory attestation earns blocking the
edge — every such warrant is given in (a.3) below.

> **AMENDED — RV-325 round 3, F-3 (owner ruling, 2026-08-01).** This rule
> previously sorted by *whether a verifier could ever corroborate* an obligation.
> That is unreproducible: *could ever* admits an imagined future verifier for
> almost any lens, and none of the surviving 2a steps ships one, so the rule
> narrated a judgement rather than making it — the weakness DEC-104's own
> consequences declared open and F-3 pressed. Completability is decidable at the
> boundary, and it gives mechanically unverifiable but genuinely mandatory human
> acts somewhere honest to sit instead of demoting them to lenses. **Every
> assignment the original rule made survives unchanged**; they are now derivable
> rather than asserted. DEC-104's ruling — *heuristics are framing, not a
> checklist* — is untouched.

A set of heuristics is not a checklist to tick off. Rendering fifteen quality
lenses as fifteen dischargeable steps converts one thing an agent might hold in
mind into fifteen things it clicks through — the incumbent's attention failure
relocated into TOML, which is exactly the harm RV-325 F-2 named.

**And a lens delivered at an exit gate arrives after the damage is done.** A
runbook fires when you attempt to *leave* a stage; framing for how to draft has
to be present *while* drafting. `Fragment::for_stage` delivers on precisely that
moment, every turn, for the whole stage. Under DEC-103 that is not a weaker
delivery than a step — it is a **stronger** one, and the ruling's error was never
in the ruling. It was this sketch reading *"delivered at the moment it takes
effect"* as *"therefore a runbook step."*

**There are exactly four places an obligation can be hung (D5).** `can_advance`
is total on non-terminal stages, so each has exactly one outbound forward edge
and edge-keying is isomorphic to origin-state-keying. Five stages, four forward
edges, each already carrying a static table of engine conditions
(`boundary_conditions`, `gate.rs:161-185`) and optionally a runbook
(`boundary_runbook`, `gate.rs:199-204`). A **step** lands on one of four edges;
there is no fifth, and that is what makes the disposition below finite rather
than a matter of taste.

**D5 bounds 2a only.** It says nothing about 2b, which keys on *stages* — five of
them, four carrying a fragment (`Locked` yields `None`). An earlier revision read
D5's finiteness as bounding the whole axis and concluded *"or it stays prose"*.
That inference was unsound: edge-keying is exhaustive over steps, not over
delivery.

**The evidence for D5, corrected.** An earlier draft cited `gate.rs:186-204` for
this. That is `boundary_runbook`'s *doc comment asserting* the isomorphism —
citing the claim as evidence for itself. The actual proof is three facts:

1. `can_advance` (`gate.rs:150-158`) is a nine-line `const fn` over a literal
   four-arm `matches!`. Read it and the claim is settled, not weighed.
2. `stage_gate_table_admits_only_legal_forward_moves`
   (`src/design_run/tests.rs:56-77`) asserts the admitted set **exhaustively
   over all 25 ordered pairs**. A fifth forward edge cannot land silently; it
   turns that test red.
3. Three consumers already depend on the linearity, not one: `boundary_runbook`
   (`gate.rs:199-204`), `forward_runbook` (`src/commands/design.rs:1801-1806`,
   which *walks* `Stage::ALL` to find the single edge rather than restating the
   table), and `cumulative_conditions` (`gate.rs:208-218`, which iterates
   `Stage::ALL.windows(2)` and so assumes a chain).

So D5 is not an assumption this sketch makes; it is a property the engine holds
and a test locks. What remains is a *durability* question — see the reviewer
section — and the blast radius of its failure is wider than the runbook keying:
fact 3 says a branch breaks the cumulative-condition walk too.

---

## (a) Line-by-line disposition of the incumbent

The incumbent is **214 lines / 10,178 bytes**. It has two layers to dispose of:
the eight numbered states, and the obligations scattered through and beneath
them.

### (a.1) The eight states — and the fact that there are not eight

Before disposing of them: **the incumbent contradicts itself about its own
state count.** The numbered list at `SKILL.md:19-28` has eight. The
`<Process State Machine>` XML immediately below it at `:30-57` has **seven** —
"Integrate Review Feedback" is not a state there; it is folded into Adversarial
review's five outbound transitions.

This diagram resolves it **in the engine's favour** — which here happens to
agree with the XML. The distinction matters, because the tie-break is not
"the XML wins": on state 3 the XML *does* model "Propose 2-3 approaches" as a
state with its own outbound transitions (`:37-40`), and the disposition below
rules it not-a-state anyway, because no `Stage` corresponds. The operative rule
throughout is that `Stage` and `can_advance` decide; the XML is corroboration
where it agrees and is overridden where it does not.

On state 7, the engine agrees. Integration is gated at the Reviewing exit
by `Condition::IntegratedReviewPresent` and `Condition::BlockingFindingsDisposed`
(`gate.rs:179-184`), which is what "work done while in a state, checked when you
leave it" looks like when a machine models it. The prose promoted a checkpoint
to a state. `EN-2` says "eight-state machine" because the numbered list is the
face the skill presents; the disposition below covers all eight so nothing is
lost to the discrepancy.

| # | incumbent state | lines | disposition | reason |
|---|---|---|---|---|
| 1 | Explore context | `:61-75` | **MOVED** — `Stage::Exploring` + the shipped `exploring.toml` runbook | already done in PHASE-16; five steps, one of them verified by `doctrine verify research-current` |
| 2 | Ask clarifying questions | `:76-117` | **MOVED** — `Stage::Inquiring` + a new edge-2 runbook (D12) | the stage exists; the craft becomes asset data |
| 3 | Propose 2-3 approaches | `:104-113` | **NOT A STATE** — craft inside Inquiring, folded into the edge-2 runbook | no `Stage` corresponds; diagram A already observed the prose merges 2 and 3 while the XML splits them |
| 4 | Present design | `:118-143` | **MOVED** — `Stage::Drafting` + `design-prompts/drafting.md` (2b) | the fifteen content items are a quality lens, not a checklist: they become fragment framing delivered every drafting turn, **not** steps (D14, RV-325 F-2) |
| 5 | Write design.md | — | **NOT A STATE** — an *effect* (D2) | `materialise` is a CLI verb and `Condition::MaterialisationCurrent` guards drafting → reviewing (`gate.rs:174-177`). Writing the doc is an engine-checked consequence of being in Drafting |
| 6 | Adversarial review | `:162-191` | **SPLIT** — `Stage::Reviewing`; the attack surfaces to `design-prompts/reviewing.md` (2b), the discrete acts to a new edge-4 runbook | *"attack vague sections, hidden assumptions, weak verification"* is a lens set and takes the same ruling as state 4; what remains as steps is what completes at the boundary |
| 7 | Integrate review feedback | — | **NOT A STATE** — work inside Reviewing (D3) | gated at the exit by two conditions; the incumbent's own XML already agrees |
| 8 | Transition to planning | `:26-28` (+ XML `:54-56`) | **RETAINED** as adapter prose (D8) | `Stage::Locked` is terminal and `for_stage` yields `None` there, so there is no outbound forward edge to key a runbook to. See below |

**Two citation corrections in that table, and one of them moves the arithmetic.**

- **State 8 was cited as `:157-162`. It is not there.** `:144-161` is the
  selector section (state 8's neighbour, disposed separately at D10) and `:162`
  is the `### Adversarial review` heading. There is no `### Transition to
  planning` body section at all: state 8 exists only as the numbered-list entry
  at `:26-28` and the XML state at `:54-56`. **This is not cosmetic.** `:26-28`
  falls inside `:15-58`, which (d)'s table marks *deleted outright* — so the
  Locked-exit instructions D8 rules the adapter **retains** were sitting inside
  the block the arithmetic deletes. Double-disposed, in the table `VA-4` was
  amended to require accounting for every part of the body. (d) now excepts them.
- **States 2 and 3 overlap by construction.** `:104-113` is inside `:76-117`.
  That is deliberate — state 3 is craft *within* state 2's section, which is the
  disposition — but a range appearing twice in a table whose whole job is
  exhaustive accounting needs saying rather than leaving to be found. (d)'s
  arithmetic counts `:76-117` once.

**On state 8 (D8, re-warranted under DEC-103).** Three options were considered:
(a) the adapter keeps the exit instructions as prose; (b) a terminal runbook
keyed to Locked; (c) `doctrine design` grows a conclude/handoff output. (b)
breaks D5's edge-keying — Locked has no outbound forward edge, so it needs a new
selector shape for one row. The ruling took (a): **if this programme succeeds,
`/plan` receives the same treatment**, and the stage-to-stage handoff interface
is better designed once against two real cases than guessed at now against one.

**The ruling stands; its warrant narrows.** DEC-103 makes this the weakest prose
in the rewrite, and the sketch should say so rather than list it beside the
genuine residue. The Locked exit has the *most precise trigger moment in the
entire file* — the instant the run reaches terminal — so "no delivery moment
could be located" is emphatically not the reason it stays prose. The reason is
**deferral**: we know the moment, we are declining to build the mechanism until a
second case shows what shape it wants. That is a legitimate warrant and a
different one. Recorded here as **deferred and known-weak**, not as legitimately
kept, and it is the residue most likely to be repaid first.

### (a.2) The residue — obligations with no state of their own

The eight states are the easy half. The incumbent also carries a `Guardrails`
section, an `Outcomes` section, an orphaned process section with no state at
all, and standing "be on the lookout" rules scattered inside states. Diagram A
classed these `vigilance` and `orphan`, and they are harder precisely because a
runbook step needs an **edge** and "be on the lookout" has none.

**The eight guardrails (`:192-204`), triaged — D9/D11 — plus two items imported
from elsewhere in the file.** The denominator needs stating, because an earlier
draft labelled a ten-row table "the eight guardrails" and diagram C reported a
third, larger figure again:

- **`:192-204` itself is eight items** — the standalone canon line `:194` plus
  seven bullets (`:196`, `:197`, `:198`, `:199`, `:200-201`, `:202`, `:203`).
  Their triage is **1 delete / 1 hymn / 5 fragment / 1 step / 0 keep**.
- **One row below comes from outside that range** and is marked as such:
  `:111-113`, inside the clarifying-questions section. With it the table reads
  **1 delete / 1 hymn / 5 fragment / 1 step / 1 boot = 9**.
- **The `step` column collapsed from six to one under D14.** Five of the six were
  lenses, not acts. That is the single largest correction in this revision and the
  reason RV-325 F-2's ceremony objection dissolves rather than being traded away.
- **`A s6.a6` (= `SKILL.md:187-188`) is NOT a row here** — RV-325 F-1. It sits
  inside `:162-191`, which already moves wholesale to the edge-4 runbook, so
  listing it as a separate deletion double-disposed it. It appears below only as
  a **clause split** within that move, not as a triage outcome of its own.
- Diagram C's D11 header reported `4 / 1 / 6 / 3 = 14` at one point and this
  document reported `4 / 1 / 2 / 3` at another. Both are superseded; the
  arithmetic above is the live one, and the drift is disclosed because three
  different counts across two artefacts is what a reviewer finds first and
  learns nothing from.

**The triage below is DEC-103's, not the original one.** The first pass sorted by
*is this stated with more authority elsewhere* and produced four deletions and
three keeps. Two of those deletions did not survive checking and the keeps did
not survive the ruling. The table now sorts by **when must this arrive**, and the
shape of the change is that almost everything has a moment:

| line | guardrail | when it must arrive | disposition |
|---|---|---|---|
| `:196` | design/plan conflict → reconcile via design | — | **DELETE** — the boot snapshot states it as a standing precedence fact, in context at every turn: *"The plan is not higher authority than the design or `/canon`"* |
| (`A s6.a6` = `:187-188`) | *see below — not a row.* Two clauses, disposed separately inside the `:162-191` move | | **RV-325 F-1** |
| `:194` | the design doc is canon for design intent | — | **TO THE SEALED HYMN** — see below |
| `:197` | don't present the whole design as settled before foundational sections are validated | **throughout drafting** | **DRAFTING FRAGMENT (2b)** — a lens on how the draft is being built, not a gate on leaving |
| `:198` | don't hide unresolved assumptions in polished prose | **throughout drafting** | **DRAFTING FRAGMENT (2b)** (DEC-103) — *was* DELETE against the hymn's *"show provenance, unresolved branches and blockers rather than a tidy surface"*. That bullet's subject is **the inquiry map** (`hymns/stage/design.md:19-21`, under *Provisional is not evidence*); `:198`'s is the **design document's prose**. Different object, not different wording |
| `:199` | don't confuse detailed design with implementation planning | **throughout drafting** | **DRAFTING FRAGMENT (2b)** (DEC-103) — *was* KEPT, then briefly an edge-3 step. The failure mode is plan content leaking into design sections *as they are written*; catching it at the exit is catching it late |
| `:200-201` | a polished full-file rewrite is not progress while hard questions are open | **throughout drafting** | **DRAFTING FRAGMENT (2b)** (DEC-103) — *was* KEPT. Same register as `:198`; both are prose in one fragment, so "merging them" is no longer a question that needs answering |
| `:202` | don't move to planning while scope tells an older story | leaving Reviewing | **STEP on edge 4**, merging with `:115-116` — reconciling `slice-nnn.md` is a finite edit with a determinate subject, done at the edge and necessary before advancing, so it is 2a |
| `:203` | governance is not optional **when the design makes architectural or workflow choices** | making a choice; reviewing it | **DRAFTING *and* REVIEWING FRAGMENTS (2b)** (DEC-103) — *was* DELETE against `explore.canon`, which guards **edge 1** and discharges once at the start of the run. The obligation bites throughout both stages. The incumbent's own `Outcomes` line says governance shapes *"both the draft and the critical review"*. **The duplication across two fragments is ruled acceptable** (owner, 2026-08-01) |
| `:111-113` | qualify doc-local acronyms (OQ-17, F-4) by their containing artifact | **every message, every stage, every skill** | **TO THE BOOT SNAPSHOT** (owner, 2026-08-01) — it is doctrine-wide, not design-scoped, and not a project concern. See below |

**`:187-188` — one bullet, two obligations, and only one of them was covered
(RV-325 F-1).** The line reads:

> *"Multiple passes of review & feedback may be required before acceptance. Do
> not presume approval until it is explicitly granted."*

An earlier revision deleted it whole, as `A s6.a6`, calling it *the strongest
deletion in the set — covered twice*. It was covered twice on the **second**
clause and **not at all** on the first:

- *Do not presume approval* → stage hymn `:24` (*"You propose; the user accepts.
  A payload cannot declare itself accepted."*) **and**
  `Condition::UserAcceptanceAttested`, which refuses the edge-4 advance without
  it. Genuinely doubly covered. **DELETE stands** for this clause.
- *Multiple passes may be required before acceptance* → **nothing states it.**
  The engine *permits* iteration — DEC-067's `regress` returns the run to the
  edge's origin to re-face every guard — but a mechanism existing is not the
  instruction arriving. That is DEC-103's whole point, applied to this sketch's
  own reasoning. **STEP on edge 4**, firing at the moment an agent believes one
  pass sufficed: *"a single review pass is not evidence of sufficiency; say what
  a further pass would probe, or why none is needed."*

**And the lines were disposed twice.** `:187-188` is inside `:162-191`, which
three tables move wholesale to the edge-4 runbook, while D11 separately deleted
it. Same defect as state 8's `:26-28`, found by the same means — a range claimed
by two dispositions in a table whose job is to sum to the incumbent. It is now
disposed once, as a clause split inside the `:162-191` move.

**Why `:203` is duplicated across two fragments rather than discharged twice.**
D10's *"a worse lie than prose"* objection was applied to it in an earlier draft,
and it does not transfer: that objection is about **false completeness** — a
discharge asserting an iterative process is finished. Under D14 the question
dissolves entirely, because a fragment discharges nothing. `:203` is a lens on
choice-making, and it appears verbatim in `drafting.md` and `reviewing.md`. That
is two copies of one sentence in two assets, which the owner ruled acceptable on
2026-08-01 — the DRY instinct that would object here is the exact instinct
DEC-103 rejects.

**Why `:111-113` leaves this file entirely.** Every earlier revision treated it as
the sketch's hardest case — *"unenforced by construction; no edge can hold a
per-message obligation"* — and made it the sole legitimate residue. That framing
had the scope wrong. Qualifying doc-local acronyms by their containing artefact is
not a *design* obligation: it applies to `/audit` reading `F-` findings, `/plan`
citing `OQ-`, `/reconcile` citing `RV-` findings, and to every skill that puts a
bare enumerated id in front of a human. Owner ruling, 2026-08-01: it is
**doctrine-wide, framework-level, and belongs in the boot snapshot**.

The placement is not arbitrary. The boot snapshot already carries the *first half*
of exactly this rule, authored at `install/routing-process.md:60-63`:

> **Reference forms.** Entity ids — prefixed, 3-digit zero-padded … Doc-local
> enumerations — bare (`OQ-1`, `D1`, `R1`, `Q1`, `C1`).

That paragraph rules on how a doc-local id is *written* and says nothing about how
it is *introduced*. `:111-113` is the missing second clause of a rule already in
context every turn, in every skill, in every project. It is not residue; the
triage was looking for its home inside `/design` and it was never there.

**The edit lands in PHASE-08, atomically with the deletion.**

> **AMENDED — RV-325 round 3, F-4 (owner ruling, 2026-08-01).** The previous
> revision deleted `:111-113` here and carried the replacement as backlog
> (IMP-376), on the reasoning that a boot-snapshot edit is a governance change
> and does not belong in a phase whose design gate is mid-review. **F-4 is
> right that this creates an interval of zero delivery** — unbounded, since
> IMP-376 is unscheduled — for a rule DEC-103 says must arrive where it bites.
> The rewrite would have been enforcing its own governing principle everywhere
> except on itself.
>
> The scope objection turns out to be nearly vacuous on inspection. IMP-376's
> target is `install/routing-process.md:60-63`, and **PHASE-08 already rewrites
> that file** (`EX-2`), re-embeds it, regenerates the snapshot and refreshes the
> projection (`EX-3`, `EX-5`). The clause lands in a paragraph already inside
> this phase's blast radius, and the whole delivery tail already runs. The cost
> is two lines of authored guidance and no new machinery — against an unbounded
> gap in a doctrine-wide rule. The IMP-374 analogy does not hold: that deferral
> concerns a file PHASE-08 does not otherwise touch.
>
> `PHASE-08 EX-10` is appended to carry it, and IMP-376 closes when the phase
> does rather than outliving it.

Whether the obligation *consistently fires* from that position remains an
evaluation question — CHR-049 carries the live post-close exercise (DEC-079) and
is the natural vehicle. That is a question about efficacy, not about delivery,
and it no longer stands between the deletion and its replacement.

**Why `:194` goes to the hymn rather than being deleted.** It looks like a
duplicate of the hymn's *"The run is the record"* and is not. The run is
authoritative for **procedural state**; the design doc is canon for **design
intent**; `adopt_authored` is the only lawful crossing between them
(`install/hymns/stage/design.md:32-36`). Stating both is what stops that reading
collapsing into "the run is authoritative for everything", which would license
exactly the behaviour the watermark exists to refuse. It is an invariant a
project cannot legitimately override, so by the authoring rule it is hymn
material, not a step.

**Why `:202` merges with `:115-116`.** Both say the slice scope must be
reconciled with current understanding before moving on. The incumbent states the
obligation twice, in two registers, in two places — once as a step inside
Inquiring and once as a guardrail. Collapsing two prose mentions into one step
on one edge is a small result, but it is the kind of result that only appears
once obligations are forced onto a finite set of edges.

**The orphan: design-target selector recording (`:144-161`) — D10, REVERSED
under DEC-103. Now STEPS on edges 3 and 4.** Diagram A classed it an orphan:
prose process with no state at all. Diagram B gap g4 confirms the mechanism half
— `doctrine slice selector add … --intent design-target` exists as a command with
no design-run hook and no prompt. Its own prose says *"as the code-impact section
locks"*, which is edge-shaped and points at edge 3.

**The original ruling kept it as prose**, on the reasoning that the single-edge
reading was too neat: in practice this wants more than one hook across design and
plan — it is iterative, research can trigger it, a decision can trigger it — and
modelling an iterative multi-hook obligation as one step on one edge would be *a
worse lie than prose*, because a discharged step asserts a completeness the
obligation does not have.

**That objection is real and it is not an argument for prose.** Unpicked, it has
two parts, and only one of them survives:

- *Overclaiming is a genuine harm.* A step reading "record the design-target
  selectors" discharged once does assert the touch-set is settled when it is not.
- *Multi-hook is not the harm.* It was doing the work in the original ruling, and
  under DEC-103 it argues the opposite way: an obligation firing at several
  moments is hung at **every** one of them.

So the remedy is step text that does not overclaim, not the absence of delivery.
*"Record the design-target selectors the code-impact section now commits to"* is
true at each firing and asserts nothing about finality — the same shape a
`sequence` runbook already tolerates for anything revisable. It lands on edge 3
(where the code-impact section locks) and edge 4 (where the review may have moved
it), which is exactly where the incumbent's own prose says it fires.

**Under D14 this is the one obligation in the guardrail sweep that is clearly 2a**,
and it is worth saying why, because it is the discriminator's cleanest case.
Recording the selectors is finite: it is done or it is not, it is done *at* the
edge, and the run cannot honestly advance without it. Every other guardrail in
the sweep — *foundation sections first*, *assumptions not hidden in polish* —
has no such completion point.

It is also the only one that could carry a verifier: selectors either exist in
the run's state or they do not, `doctrine slice selector list <id>` reads them,
and `slice conformance` diffs them against git actuals at audit. **That is a
remark about evidence, not about classification** (F-3) — the step is 2a because
it completes at a boundary, and it would remain 2a if no such command existed.
Whether PHASE-08 actually wires that verifier is left to implementation; the
step is warranted either way.

**The residual is real and stays open:** the hooks *across design and plan* are
still unmodelled, because `/plan` has no runbook machinery yet. What DEC-103
changes is that this is now a **known gap in coverage** rather than a
justification for delivering nothing at the two moments we can already reach.

**The `Outcomes` section (`:205-214`) — DELETE, and this was found late.** Ten
lines of seven bullets, and the triage above did not originally cover them; they
were swept only when (d)'s arithmetic refused to close. Nearly every bullet
restates something now mechanised: *short feedback loops* is per-section
acceptance (edge-3 step plus `SectionAttestationsCurrent`); *verification impact
explicit* is a content item; *an internal adversarial pass* is `Stage::Reviewing`
plus the edge-4 runbook; *scope and design stay aligned* is the merged `:202`
step; *ADRs, policies and standards shape the draft and the review* is
`explore.canon` plus the attack list. What remains is a statement of purpose,
which the adapter keeps in two or three lines rather than seven bullets.

This omission is disclosed rather than quietly repaired because it is evidence
about the method: a triage driven by a question list covers what the list names.
`Outcomes` was named by nothing.

### (a.3) What lands where, in total

| tier | destination | source lines |
|---|---|---|
| 1 | already-existing engine conditions and refusals | states 5 and 7 in full |
| 1 | the sealed stage hymn | `:194` |
| 2a | `exploring.toml` — **shipped** | `:61-75` |
| 2a | edge-2 runbook — the discrete acts of inquiry (**D12**) | `:107-109` (knowledge capture), `:115-116` |
| 2a | edge-3 runbook — **one step** | `:144-161` selector recording |
| 2a | edge-4 runbook — the discrete acts of review | `:187-188` first clause (repeat-pass), `:202` + `:115-116`, `:144-161` |
| 2b | `inquiry.md` — question craft | `:76-117` less `:107-109`, `:111-113`, `:115-116` |
| 2b | `drafting.md` — the fifteen content items **as one lens**, plus four standing lenses | `:118-143`, `:197`, `:198`, `:199`, `:200-201`, `:203` |
| 2b | `reviewing.md` — the attack surfaces, plus governance | `:162-191` less `:187-188`, `:203` |
| — | the boot snapshot (`install/routing-process.md:60-63`) — **in this phase, `EX-10`** | `:111-113` |
| 3 | retained adapter prose — **residue register** | `:26-28` (deferred, D8) — *one item* |
| — | deleted outright | `:15-58` less `:26-28`, `:196`, `:187-188` second clause only, most of `:205-214` |

`:203` and `:144-161` each appear twice. That is the point of DEC-103, not a
bookkeeping error: an obligation that fires at two moments is delivered at both.
`(d)`'s line arithmetic counts each source range once.

**Three things about this table are new, and each was a defect before.**

1. **Edge 3's runbook has one step.** It had nineteen. Fifteen content items and
   four guardrails were never dischargeable acts; they were the framing for the
   activity the edge terminates.
2. **The `2b` rows did not exist.** They are where the "craft" in DEC-102's
   *"craft is overridable"* actually lives.
3. **`:111-113` is not in tier 3.** The residue register is down to a single
   deferred item, which is the honest floor this rewrite was reaching for.

**On the third runbook (D12), re-warranted and narrowed under D14.** The runbook
sketch's §9 assigned PHASE-08 *two* checklists. A third was added on 2026-08-01,
on edge 2, because **DEC-102 overclaims**: its consequences say *"the craft has
genuinely moved out of skill prose into data"*, and the record's own central
example — `SKILL.md:98` *"prefer multiple choice questions when possible"*, against
this project's `CLAUDE.md`, which asks for prose forks in design loops — is
**inquiry** craft, while PHASE-16 shipped the **exploring** runbook.

**D14 satisfies that claim by a different route and mostly dissolves D12.** A
question-asking preference is a lens, not an act, so `:98` and its neighbours
belong in `inquiry.md` — which *is* data, and is the honest home. What remains for
an edge-2 runbook is the discrete residue: knowledge capture (`:107-109`) and
scope reconciliation (`:115-116`). The reviewer should press on whether that
residue justifies a third runbook at all, or whether edge 2 should carry no
runbook and the two acts attach elsewhere.

**`inquiry.md` is delivered in TWO stages, not one — Exploring as well as
Inquiring.** `Fragment::for_stage` maps `Stage::Exploring | Stage::Inquiring =>
Fragment::Inquiry` (`prompt.rs:107-115`); there is one `Inquiry` variant serving
both, and the mapping is exhaustive over `Stage` by construction, so nothing in
this rewrite can narrow it without adding a variant and an asset.

> **RV-325 round 3, F-8.** Earlier revisions of this sketch and of diagram C both
> said *"delivered every Inquiring turn"*. That was **wrong about the mechanism**,
> and it mattered: it meant a second delivery moment sat outside the disposition
> entirely. Stated and warranted here rather than left to arrive as a side effect
> of a shared variant.

**The cross-stage delivery is wanted, not merely tolerated.** The code says why
in its own words — *"Exploring and inquiring are both question-shaping work"*
(`prompt.rs:109`) — and every item moving into `inquiry.md` holds in both:
one question per message, two or three options with tradeoffs plus a
recommendation, focus on purpose / constraints / success criteria, and `:98`'s
multiple-choice preference. None of it is Inquiring-specific. **Splitting the
mapping would cost a new `Fragment` variant, a fifth asset, and a duplicate of
substantially the same prose — to separate two stages that want the same craft.**

One consequence to name: during Exploring an agent now receives **both**
`exploring.toml` (shipped, five steps, fired at edge 1) and `inquiry.md` (every
turn). They do not overlap — the runbook's steps are boundary acts (scope,
research, canon, memory, triage), the fragment is question craft.

**And this is not special to Exploring — it is the shape of every stage.** An
earlier revision called Exploring *"the first place in the design where a stage
carries both asset kinds at once"*, which is wrong: under D14 **all four
non-terminal stages carry both, by construction**. Inquiring holds the edge-2
runbook and `inquiry.md`; Drafting the edge-3 runbook and `drafting.md`;
Reviewing the edge-4 runbook and `reviewing.md`. Exploring is notable only
because its runbook already ships, so it is where the pairing can be *observed*
today rather than where it first occurs.

That correction matters beyond the sentence. **The two asset kinds are not
alternatives that a stage picks between — they are a pair a stage always has**:
the framing that rides the whole activity, and the acts that end it. Stating it
as an exception described D14 as narrower than it is.

**SETTLED — RV-325 round 3, F-5 (owner ruling, 2026-08-01): edge 2 carries a
runbook, with exactly these two steps.** The previous revision left this open and
called it a disclosure. F-5 is right that it is not one: PHASE-08 must author the
target assets, so leaving the fork open hands a design choice to implementation,
which is the one thing this gate exists to prevent. It is also now answerable on
the rule rather than on taste — the amended discriminator asks whether each
obligation completes at a boundary, and both of these do. A file count is not a
reason to demote an act to a lens.

| step | source | why it completes at edge 2 |
|---|---|---|
| `inquire.knowledge` | `:107-109` | the inquiry has produced a determinate set of answers; sweeping *them* for what outlives the session is finite and is done once, at the moment the stage ends |
| `inquire.scope` | `:115-116` | reconciling `slice-nnn.md` against the understanding just accepted is a discrete edit with a determinate subject |

**Step text is bounded, and that bound is load-bearing.** *"Capture everything
important"* has no completion point and would be a lens wearing a step's costume
— the exact failure D14 corrects. The steps read instead:

- *"Record, via `/knowledge`, what this inquiry settled that outlives the
  session"* — the subject is **this run's answers**, not the design at large. The
  step does **not** enumerate the record kinds; `:107-109` names three of a closed
  seven, and a skill restating a Doctrine-owned vocabulary is a second source of
  truth that already disagrees.
- *"Update or explicitly confirm the slice scope against the decisions this
  inquiry accepted"* — *or explicitly confirm* is what makes it finite; without
  it the step is unsatisfiable when nothing changed.

**Two further texts are fixed by the round-11 authoring gate**, and are recorded
here so PHASE-08 authors them rather than inventing them. Each names where its
finite result lands, which is what makes the step's completion condition
state-visible (the gate table below, rows 5 and 9):

- **`explore.triage`**, amending the shipped text — *"Record the design surface
  triage in the slice notes: open questions, risks, assumptions, shaping
  decisions, constraining governance."* The shipped text says *"Triage the design
  surface: …"* and states no landing site.
- **repeat-pass judgement**, edge 4 — *"Record, in the slice notes, what a further
  review pass would probe — or why none is needed."*

#### The codicil: why each unverified 2a step earns its gate

The amended discriminator demands this of every 2a step shipping no verifier.
**Eight of the nine qualify.**

> **CORRECTED — RV-325 round 11 (F-17, sweep).** This read *"four distinct
> obligations qualify (`exploring.toml` is shipped and already carries a verifier
> on `explore.research`)"*. The parenthetical is **invalid as reasoned**: one step
> of five carries a verifier, and `explore.scope`, `explore.canon`,
> `explore.memory` and `explore.triage` ship none. A whole asset was excused on
> one of its members. They are 2a assignments this slice made, so the codicil
> reaches them, and they are warranted below rather than left as a disclosure.

| obligation | edges | why mandatory attestation earns blocking the edge |
|---|---|---|
| `explore.scope` (shipped) | 1 | the governing specs and ADRs are what every later stage argues *from*; entering Inquiring without them produces questions the corpus already answers, and the cost surfaces as rework across the whole run rather than at the edge |
| `explore.canon` (shipped) | 1 | governance not in view at the start is governance discovered at audit. `:203`'s lens cannot fire on rules the agent never loaded, so this step is the precondition of a 2b obligation two stages later |
| `explore.memory` (shipped) | 1 | the corpus exists to stop rediscovery; an unretrieved memory is paid for twice — once in tokens, once in the wrong answer — and neither cost is visible at the moment it is incurred |
| `explore.triage` (shipped) | 1 | the triage is the design surface's inventory. Without it Inquiring has no agenda and the stage's own exit has nothing to be measured against |
| knowledge capture (`:107-109`) | 2 | the answers are in context *now* and gone after the stage; the cost of omission is unrecoverable, and the record is what a resumed or reviewing agent reads instead of the transcript |
| scope reconciliation (`:115-116`, `:202`) | 2, 4 | a scope that silently diverges from an accepted design is the defect `slice conformance` exists to catch at audit — far later and far more expensively |
| selector recording (`:144-161`) | 3, 4 | the code-impact section commits to a design target the run cannot honestly leave unrecorded; `slice conformance` diffs those selectors against git actuals at audit, so an unrecorded commitment surfaces far later and against a slice that has already closed. **No verifier ships today** — and *evidence available in principle* is not a warrant, which is the whole of F-3; the warrant is the audit-time cost, not the verifier's hypothetical existence |
| repeat-pass judgement (`:187-188` first) | 4 | the finite result is the **statement** — what a further pass would probe, or why none is needed. Blocking is the point: this fires exactly when an agent believes one pass sufficed, which is when it is least likely to volunteer the reasoning |

**The count, stated against both predicates, because conflating them is what got
it wrong.** *"Two of the four are attestation-only"* stood here and was wrong
under every reading (RV-325 F-9):

| predicate | count |
|---|---|
| **ships a verifier today** | **0 of 4** — this is the one the diagram states, and it is the one that governs what an implementation will observe |
| could carry a verifier at all | **1 of 4** — selector recording; `slice selector list` and `slice conformance` already exist to reach it |
| no reachable verifier even in principle | **3 of 4** — knowledge capture, scope reconciliation, repeat-pass judgement |

> **The counts above are over the four this slice RELOCATES, and they are about
> VERIFIERS.** Both qualifications matter after round 11. Over all nine, *ships a
> verifier today* is **1 of 9** (`explore.research`). And **verifier-reachability
> is not state-visibility**: a verifier is an argv whose exit code decides, so
> *"the scope contradicts nothing the accepted decisions settle"* has no reachable
> verifier and is still **state-visible** at edge 4 — both its terms are readable
> from run state by an agent that cannot be expressed as an exit code. F-17 read
> this row as condemning three obligations under the authoring gate; it does not,
> and the two predicates are now kept apart by name.

So **all four will report `attested`**, exactly as a fragment would. That is a
real cost and it is not disguised: what distinguishes them from a fragment is not
the receipt, it is that the run **cannot advance** until an agent asserts the act
is done. A fragment carries no such refusal.

**But the deeper half of DEC-102's claim fails, and D14 makes that visible.** The
record says craft is *overridable*. The shipped authoring rule says the same —
*"Yes → a runbook step. **Overridable**, verifier substitutable"*. Neither is true
of this store:

- Runbooks resolve to `{STORE}/{name}.toml` and fragments to `{STORE}/{name}.md`
  (`runbook.rs:185-189`, `prompt.rs:68-70`) — **embedded assets, both**.
- `doctrine prompt check` *"loads `.doctrine/hymns` plus the embedded hymns …
  it knows nothing about this store's `*.toml` siblings"* (`runbook.rs:318-322`).
  The hymn cascade's disk overlay, its `replaces` suppression and its seal/expose
  machinery **do not reach `design-prompts/` at all**.
- *"Verifier substitutable"* means something narrower than it sounds: the closed
  thing is the **placeholder vocabulary** a `verify` argv may interpolate
  (`runbook.rs:59-66`), substitutable *within* an authored asset. It is not a
  project override.

So there is presently **no interface by which a project expresses its own design
priorities.** The design even anticipates one — `RUNBOOK_STEP_ID_BYTES` is sized
at 64 rather than 32 because the latter *"would fit the shipped five while leaving
**a project** no room"* (`runbook.rs:55-57`) — headroom reserved for authorship the
load path does not admit. Carried as backlog (see below); named here because
DEC-102's overclaim is larger than the tense problem already disclosed.

Both remaining runbooks ship as `mode = "sequence"` (D7), and **the DEC-101
tension is conceded, not dissolved.**

> **This paragraph previously claimed the tension was *gone rather than
> conceded*** — on the ground that the two coverage sets that provoked it (the
> content items, the attack list) had become fragment prose under D14, and prose
> has no order to fake. **The premise was true and the conclusion did not follow.**
> F-5 then settled edge 2 at two steps, and edge 4 carries three: **five
> order-independent steps under `sequence`**, whose cursor *refuses any discharge
> that does not name the step at it* (`run.rs:1369-1377`). Nothing makes knowledge
> capture precede scope reconciliation, or repeat-pass judgement precede selector
> recording. Withdrawn by the post-round-4 self-audit; D7's amendment governs.

So the honest statement is D7's original one, with a new subject: an imposed order
on a short list of discrete acts is **fake determinism, accepted as cheap** — five
steps across two edges rather than nineteen on one. It is not claimed to be
meaningful. Arguing that it is would be an assertion against a named risk, which
is what **F-2** punished.

**`set` mode stays deferred to IMP-373, on a warrant that survives inspection.**
Not *on merit* — that claim is withdrawn as false, since these five steps are
precisely the "real instances to design it against" DEC-101 named when it assigned
`set` to PHASE-08. The deferral rests instead on **F-5's own rule applied to
itself**. `set`'s *admission* is cheap: six lines at `run.rs:1369-1377`, plus a
`Mode` variant referenced nowhere outside `runbook.rs`. Its *render* is not —
`RunbookStanding::cursor` drives the turn lines (`runbook.rs:435-470`), and what a
**cursorless** runbook renders under `EX-14`'s token bound is unsketched and sits
**outside `EN-2`'s gate**. Shipping `set` here would either widen that gate at
round 4 or hand a design choice to implementation — the one thing F-5 says this
gate exists to prevent. D6's consequence is load-bearing again: an obligation
admissible only as a set stays prose.

**Repayment is evidence-triggered.** IMP-373 repays when agents are observed
hitting `Refusal::DischargeNotAtCursor` (`refusal.rs:198`) against these runbooks
on an order that carries no meaning. Collected in PHASE-09's exercise, **not** from
the run record: a refusal aborts the write, so nothing is journalled — the change
log records applied changes only. The refusal string is machine-emitted and
quotable, which makes this an observation rather than a judgement.

**The upstream records now say this too — RV-325 F-11.** The ruling first landed
only here and in the diagram, while **DEC-101's body still assigned `set` to
PHASE-08** and **PHASE-16's `EX-7` / `EX-14` still moved both the mode and its
cursorless rendering there**. A sketch and a gate cannot narrow an accepted
decision or a discharged criterion by describing them differently. DEC-101 now
carries a dated amendment withdrawing the assignment; `EX-7` and `EX-14` carry
annotations recording that their *forward assignment* moved to IMP-373, with
**neither discharged obligation changed** — PHASE-16 shipped `sequence` only and
rendered the cursor step, correct then and correct now. PHASE-08 holds no
criterion assigning `set`, and `VH-1` already reads it as deferred, so nothing
downstream contradicts this. The same false rationale in `runbook.rs`'s `Mode`
doc comment is corrected with them.

> **Contested and re-fixed at round 6.** That repair amended DEC-101's **prose
> tier only**. `record-101.toml`'s consequences still carried the live structured
> assignment, so `doctrine knowledge show` — which synthesizes both tiers — emitted
> the assignment *beside its own withdrawal*. This project states the two-tier
> reading rule everywhere and it was broken in the record used to state it. The
> structured consequence now carries the amendment too. Swept with it: `PHASE-09
> EX-8` still asserted *"IMP-373 … stays deferred on DEC-104's own merits"*,
> withdrawn at round 5 and never swept into the criterion. The contest **accepted**
> the completed-phase annotations, on the ground that they preserve the discharged
> obligations.

### (a.3.1) The admission bound — RV-325 F-2

**F-2 broke the reasoning above, and the break is structural rather than a matter
of taste.** DEC-103 requires every obligation to be delivered at every moment it
fires and states **no admission limit, merge rule, optionality rule, or cost
threshold**. Applied literally, edge 3 accretes the fifteen content items plus
`:197`, `:198`+`:200-201`, `:199`, `:203` and `:144-161` — **nineteen or more
required obligations on one edge**. And the shipped mechanism makes that serial,
not merely long:

- `Mode::Sequence` is *"a cursor, one step at a time, no skipping"*
  (`src/design_run/runbook.rs:201-205`).
- `discharge` on a submission is an **`Option`, not a `Vec`** — *"`sequence` mode
  admits exactly the step at the cursor, so a batch of two would need an order
  `Batch` explicitly refuses to carry (DEC-063)"* (`src/design_run/submission.rs:636-641`).

So nineteen obligations at edge 3 is **nineteen separate CAS mutation turns**
before the run can advance. D7's *"costs almost nothing"* was asserted without
evidence against exactly the risk this slice already names — **R1, protocol
ceremony exceeds its value** (`design.md:568-572`) — and PHASE-09 `EX-3` exists to
measure it *"on evidence rather than impression"*, but PHASE-09's `EN-1` requires
PHASE-08 complete. The measurement lands after the shape is built. That is not a
bound; it is a post-mortem.

**The first proposed remedy was wrong, and is recorded because the correction is
instructive.** It read: *obligations that share a moment and a subject merge into
one step whose text carries the coverage list* — a **granularity** bound, on the
premise that DEC-103 governs *when* and had been silently read as also governing
*how many*. That diagnosis was half right. The ruling does govern only *when*. But
merging nineteen steps into six leaves the content in the wrong asset kind
entirely, and the arithmetic was doing the work that a missing distinction should
have done.

**The owner's ruling of 2026-08-01 supplies the actual bound, and it is a
destination rule, not a counting rule:**

> A set of heuristics to apply while doing the work is **not** a checklist to tick
> off. It is framing for the activity, and an exit gate delivers it when the
> damage is done.

That is D14. The admission question was never *how many steps may share a moment*
— it was *which obligations are steps at all*. Sorted by completability at the
boundary (§(a.2)), the edge-3 population splits cleanly: **one** obligation that
finishes at the edge and must finish before advancing, and eighteen lenses with
no truthful completion point, which belong in `drafting.md` where they are
delivered on every drafting turn rather than once at the exit.

**Edge 3 is one step, not six and not nineteen.** F-2's ceremony objection is not
bounded, traded or mitigated; the population that generated it is gone. The
`Mode::Sequence` and `Option<DischargeDeclaration>` facts F-2 established remain
exactly as stated — they are simply no longer load-bearing against a one-step
edge.

**What this costs, stated plainly.** A fragment cannot be discharged, so nothing
records that an agent read the drafting lens. Fragment delivery is bound by
`name@digest` and re-emitted unless an exact current receipt is held (DEC-078),
which proves the bytes *arrived*; it proves nothing about attention. The earlier
nineteen-step shape would have produced nineteen attestations — and that is the
trade: **fewer receipts, better odds the content is actually in front of the agent
when it matters.** RV-315 F-17's point applies to this sketch too, and nothing
about it is comfortable: no test can fail here. `VH-1` is the judge.

#### What would show this trade was chosen wrongly

> **RV-325 round 3, F-7.** The paragraph above previously stopped at *"no test
> can fail here"*, which is true of the *gate* and was quietly doing duty as a
> reason not to state a decision rule at all. Those are different things. An
> unfalsifiable claim is not excused by an ungoverned artifact — it is exactly
> the case where the falsifier has to be written down in advance, because after
> the fact any outcome narrates as vindication. Pre-registered here, at the gate
> that chose the shape, rather than left to the exercise that measures it.

The claim is: *delivering craft as every-turn fragment prose puts it in front of
an agent more reliably than delivering it as discharged runbook steps at an exit.*
It is **falsified** by any of the following:

| signal | result that would revise the placement |
|---|---|
| **adherence** | fragment-delivered craft is followed *no more often* than the same content was as a step. The 2b content is doing no work that a lens in the incumbent's prose was not already doing, and moving it bought nothing |
| **receipt churn** | agents re-request or re-receipt fragments at a rate implying they are not read on arrival — delivery without attention, which is the whole thing the trade purchased |
| **cost** | per-turn fragment tokens exceed the interaction cost of the step shape they replaced. The trade was *fewer receipts*; if it is not also cheaper, its only remaining claim is attention, and the adherence signal must carry it alone |

**The comparison is against the step shape, not against nothing.** The baseline
is the pre-SL-233 skill, where the same content is present as skill prose an
agent may scroll past — and where available, a fixture exercising the
nineteen-step edge-3 shape D14 rejected, which is the treatment the trade was
actually made against. Without one of those, an observation says only *"the
fragment arrived"*, which nobody disputes.

**If falsified, the revision is mechanism-side and DEC-104 stands** — deliver the
same lenses better, do not reclassify them. **The lever is narrower than it
first looks, and naming it precisely is the point.** DEC-078's suppression is
**caller-declared**: the digest already rides every emission as a `name@digest`
header (`prompt.rs:27, :89`), and the body is omitted only when a caller
*declares* `--known-fragment <name>@<digest>` back. So an agent can suppress a
fragment by claiming a receipt for bytes it never read, and Doctrine cannot tell
that from a genuine hold. Two candidates follow, in escalating cost:

- **Bound the suppression, or take it off the caller.** Re-emit unconditionally
  every *n* turns regardless of a held receipt, so a claimed-but-unread receipt
  cannot suppress indefinitely. Cheap, and it directly attacks the failure the
  receipt-churn signal detects.
- **Shorten or split the fragment** so the lens is not buried in bytes an agent
  skims. Costlier — it re-opens what belongs in each fragment — but it is the
  only lever that helps when the bytes genuinely arrive and are genuinely not
  absorbed.

Neither touches the discriminator, because none of the three delivery signals is
evidence about it.

> A third candidate stood here — *"deliver a fragment digest in the envelope so
> non-attention is visible"* — and was **withdrawn on inspection**: the digest is
> already emitted, so the suggestion was either vacuous or a restatement of the
> first. Recorded rather than deleted, because a mechanism list nobody checked
> against the mechanism is the same failure as a classification rule nobody
> checked against the code.

> **RV-325 round 4, F-10.** This paragraph previously said the 2b content
> *"returns to 2a as a coverage-set step"* and that *"a negative result reopens
> IMP-373; it does not reopen DEC-104"*. **F-10 is right that this is
> incoherent, and the way it is wrong is worse than the incoherence.** DEC-104
> classifies that content as 2b *precisely because it has no truthful boundary
> completion point*. Moving it to 2a on the strength of a delivery result either
> falsifies that classification or is inadmissible under it — the fallback
> cannot both contradict DEC-104 and exempt it.
>
> Worse: **exempting DEC-104 from its own falsifier is the exact manoeuvre F-7
> raised, re-inserted one level up.** F-7 objected to a claim shielded from
> evidence; the remedy shielded the governing decision instead. Naming that
> plainly matters more than the repair, because it is the second time in this
> review the same instinct has surfaced.
>
> The premise was also a category error, which is the substantive fix. **None of
> the three signals is evidence about completability.** Adherence, receipt churn
> and cost all measure whether *every-turn fragment delivery works*; none of them
> could show that *consider data-flow boundaries* finishes at a moment. So a
> mechanism failure warranted a mechanism remedy all along, and the coverage-set
> fallback was answering a question nobody had asked. **IMP-373 is no longer the
> named fallback.**
>
> Its *deferral* no longer rests on *"the merits DEC-104 already gave"* either —
> that warrant was **withdrawn as false** by the post-round-4 self-audit, which
> found five order-independent steps under `sequence` after F-5's settlement. See
> (a.3) above and D7's amendment: `set` is deferred because its render is
> unsketched and outside `EN-2`'s gate, not because nothing needs it.

**But DEC-104 must be falsifiable too, or this section repeats its own defect.**
A fourth signal is therefore pre-registered, and it is the *only* one that
reaches the discriminator:

| signal | result that would reopen DEC-104 |
|---|---|
| **misclassification** | either direction, and in both directions the firing **reopens the discriminator for that obligation** — it does not reclassify it. (i) A 2a step is discharged at a turn where **its subject did not yet exist in run state** — *"scope reconciled"* attested before the design settles — **beside siblings at the same edge that were discharged correctly**. (ii) An agent **explicitly asks whether a 2b lens is complete, or requests a gate or checkpoint on one**, quoted verbatim, **beside lenses in the same run it did not treat that way** |

**Both halves were re-anchored after round 4, and the reason is the same one F-3
gave.** They previously read *"cannot honestly be complete"* and *"agents treating
a lens as having a completion moment they would gate on"* — inferences about a
mind, with no adjudicator and no rubric anchor named. That is structurally F-3's
criticism of the original discriminator: sound as a rule, un-cashable in practice.
A falsifier nobody can collect leaves DEC-104 unfalsifiable by the same route F-7
raised, one level down.

- **(i) is now a run-state predicate**, checkable from the run record without
  inferring honesty. **Round 10 restated it and round 11 rescoped it**: the
  observation is *discharged at a turn where the step's **stated condition** was
  not satisfied*, collected over the **state-visible** conditions — rows 2, 5, 7
  (edge 4), 8 and 9 of the gate table below.
  > **SUPERSEDED, both halves.** This previously scoped (i) to *"selector
  > recording and knowledge capture"* and disclosed `inquire.scope`'s *"or
  > explicitly confirm"* branch as uncollectible. **Knowledge capture is now
  > out** — its condition's left-hand term (what this inquiry settled) is not in
  > run state, and a record's *presence* was never its completion. **`inquire.scope`
  > is now in, at edge 4** — the gate asks for a **state condition**, not an
  > artefact, and *"the scope contradicts nothing the accepted decisions settle"*
  > holds or fails whether or not an edit occurred. The confirm branch's
  > artefact-lessness bore on the withdrawn subject-presence test; it does not
  > bear on this one, and F-17 inherited that reading from this stale paragraph.

  Disclosed rather than papered over: a conservative signal that never fires
  falsely is worth more than a broad one that needs an inference to fire at all —
  and the covered fraction is reported rather than assumed, five of nine.
- **(ii) is now a speech act**, not a moderator's read of what an agent *would* do.
  Half (ii) cannot be run-state anchored **at all** — a lens is fragment prose with
  no discharge event, so there is no state transition to observe. Dropping it was
  the alternative and was rejected: it would leave the falsifier one-directional,
  able to detect *2a-was-never-a-step* and never *2b-should-have-been-2a*.

Neither can be satisfied by delivery data, which is what `VA-5` actually demands.

**The sibling contrast and the weakened consequence are RV-325 F-12's repair, and
the defect it caught is the one this section exists to prevent.** The re-anchor
above fixed the *evidence* — no more intent-reading — and left the *consequence*
overclaimed: it read the raw event as showing *"the obligation has no real
boundary and was never a step"*. It does not. **A 2a discharge before its subject
exists has three live causes:**

| cause | what it is | where it routes |
|---|---|---|
| **A** | the obligation has no real boundary | reclassify to 2b |
| **B** | a correctly classified act, falsely or prematurely attested | ordinary noncompliance — the *adherence* signal |
| **C** | step text that misled a compliant agent about when it completes | reword — *if that arm earns its exit* (round 7, below); otherwise reclassify |

A signal with three explanations pointed at one consequence is not a falsifier —
it is the exact defect `VA-5` names, reproduced inside the repair meant to satisfy
it. **Third instance this review of a fix reproducing its own finding's defect.**

**The two corrections.** First, the consequence is weakened to what the evidence
carries: a firing **reopens the discriminator's application to that step**, and
the reopening admits **reword** or **reclassify** — and, as first written at round
5, a third outcome **stand**, *which round 6 removed* (see below). *Reopening* was
always the right verb; asserting the event *proved* misclassification was the
overclaim, and it is withdrawn. Second, the observation
must carry a **within-run control**: at the same edge, in the same run, how were
the sibling 2a steps discharged? One step premature beside correct siblings is
step-specific and reaches the discriminator; everything premature is agent-general
and routes to adherence instead. That is the separation `VA-5` demands, finally
doing work rather than being asserted.

**Scale, disclosed.** CHR-049 specifies **one** moderated exercise, so the
within-run control is the collectible discriminator and cross-run corroboration is
opportunistic. The contrast separates **B** from **{A, C}**; it does **not**
separate **A** from **C** — that is settled at the reopening, which is precisely
why the resolution set is named here rather than left to whoever reads the result.
*Round 7 finishes the job: the reopening's choice between those two is itself
pre-registered (F-15, below), because naming a resolution set and leaving the
selection to judgement is the routing manoeuvre with one step renamed.*

#### `stand` is removed — RV-325 F-13, and the fourth instance of the pattern

**The round-5 repair stopped the signal over-routing to reclassification by adding
an exit that routes it nowhere.** `VA-4` requires every falsifying signal to name
a result that *would revise the placement*. An unconditioned `stand` let any
firing be narrated after the fact as an anomalous run, with nothing pre-registered
selecting that exit — `VA-5`'s route-to-the-cheapest-revision manoeuvre arriving
by the opposite door, as **route-to-no-revision**. At N=1 the sibling control
separates step-specific from agent-general behaviour but cannot rule out anomaly,
so `stand` was always reachable.

**Once the signal fires, the discriminator reopens — reword or reclassify, no
third exit.** The two arms are **not** equivalent under `VA-4`, and round 7 stops
offering them as though they were: which one a firing takes is itself
pre-registered, below.

**The anomaly control moves into the firing condition**, where it can be stated
before any run rather than chosen after one. The signal fires only when all hold:

1. the within-run sibling control is satisfied;
2. the run crossed **no context boundary** at that edge — neither the deliberate
   break and resume CHR-049's protocol induces, nor an **incidental** one
   (context exhaustion, compaction, or harness-initiated summarisation);
3. the run was not aborted or restarted mid-edge;
4. the moderator protocol **recorded the context state** across the window
   containing the discharge. Without that record (2) is undecided, and the
   observation is **uncollected, not weighed** — the same disposal the sibling
   control already takes.

**If that signal fires, DEC-104's discriminator reopens** — not IMP-373, and not
the delivery mechanism. The three delivery signals and this one have different
subjects and therefore different consequences, and keeping them apart is what
stops any result from being narrated into whichever revision is cheapest.

`PHASE-09 EX-8` carries all four into the kit and `CHR-049` runs them. Neither
may substitute a narrative for the comparison.

#### The reopening is pre-registered too — RV-325 F-14 and F-15, the seventh instance and its two faces

**Round 6 narrowed the exit set and the exclusion set in one move, and each
narrowing lost something the thing it replaced had covered.** That is why F-14 and
F-15 arrived together, and why answering them in sequence produces a wrong answer.
It is the **seventh instance** of a repair reproducing its own finding's defect,
in the shape `## Learned` now names: *overcorrect along the axis the finding
named, land off it in the other direction.*

**Face one — the exit set (F-15).** This section previously argued: *"Both revise
the placement or its boundary contract, which is what `VA-4` asks: rewording a
step's text **is** a change to its boundary contract, not a null outcome."* **That
is withdrawn as false.** Rewording a 2a step's boundary text leaves it a 2a step
on the same edge; it revises the obligation's *contract*, not its *placement*, and
`VA-4`'s noun is **placement**. The sentence broadened the criterion's noun so the
cheap arm would qualify — the manoeuvre F-15 names and forbids, and structurally
what `stand` was doing: an exit that changes no placement, reached by a route
nothing pre-registers. Removing `stand` closed the zero-revision exit and
installed a no-placement-revision exit in its place.

**`VA-4` is not amended, and does not need to be.** It requires each falsifying
signal to name *a* result that would revise the placement — reclassification is
that result. What defeats the criterion is not the presence of a cheap arm but an
**unreachable expensive one**: a signal naming a placement revision that nothing
can compel names a result it can never produce. So the repair goes to
reachability, not to the noun.

**The reopening is where the manoeuvre had migrated to.** Every other link in this
chain is pre-registered — the observation, the sibling control, the firing
condition, the resolution set — and the *choice between the two arms* was left to
"the reopening", explicitly: the scale note above separates **B** from **{A, C}**
and concedes it does not separate **A** from **C**. Two exits of very different
cost, selected after the run by no stated rule, is
route-to-the-cheapest-revision with the routing step renamed. That breaches
`VA-5` on its own terms, independently of `VA-4`.

**The rule, pre-registered.** A firing reopens the discriminator for that step,
and the reopening resolves like this:

- **Reclassify to 2b is the default** — what a firing means unless the reword arm
  earns its exit.
- **Reword must produce an artefact to be taken.** The reopening records a
  *corrected completion condition* meeting the authoring gate's standard below —
  truthful for the obligation, checkable in run state. No corrected condition, no
  reword.
  > **Round 11 kept `checkable in run state` HERE, having withdrawn it from the
  > classification limb, and the asymmetry is deliberate.** Only a state-visible
  > condition can fire this signal at all, so the reopening always starts from
  > one. Dropping the requirement would let a step **escape the signal by
  > rewording its condition out of visibility** — a firing answered by becoming
  > uncollectible, which is the route-to-no-revision manoeuvre this whole
  > pre-registration exists to close, arriving by a door round 11 would otherwise
  > have opened. The gate classifies on *truthful*; the reword arm additionally
  > demands *visible*, because it is buying back a step that was already being
  > watched.
- **Reword is one-shot per step.** The reworded boundary is a prediction. A second
  firing on the same step after a reword spends cause **C** and reclassifies, with
  no further reopening.

That inverts the cost gradient the round-6 asymmetry rested on: the cheap arm now
costs a checkable artefact, and failing to produce one compels the expensive one.
Nothing here reclassifies on the raw event — F-12's correction is untouched, and
cause **B** still routes to adherence and never reaches the discriminator.

**Rounds 8–10 tried to make this reopening decide the classification from run
evidence, and it cannot.** The chain — *no candidate text ⇒ no boundary exists*,
then *enumerate the turns where the subject is present* — is **withdrawn**
(RV-325 F-16 and its contest). Subject absence proves a step is not complete;
subject presence does not prove a truthful completion moment exists, and the
selector obligation above is the counter-example: it commits to *what the
code-impact section now commits to*, so a recorded selector can coexist with an
unfinished obligation. The reasoning is in the ledger; what the design carries is
the rule below.

#### The classification test is an authoring gate, not a run inference — RV-325 round 10, owner ruling; the limbs split at round 11

**Every 2a step states its own completion condition, in its text, at authoring
time.** The condition must be *truthful* — satisfying it discharges the
obligation rather than starting it. **An obligation whose completion condition
cannot be stated is not a step; it is 2b.** That is the discriminator, it is
decidable over the whole set before any run, and anyone can re-run it.

> **AMENDED — RV-325 round 11, owner ruling (F-17).** Round 10 also required the
> condition to be *checkable in run state*. **That clause is withdrawn from the
> classification limb.** It contradicted a round-3 ruling this design never
> reopened — *"verification strength is an orthogonal axis, not the sorting rule"*
> (`target-machine.d2:449`), which exists precisely to give *"mechanically
> unverifiable but genuinely mandatory human acts — semantic scope reconciliation,
> knowledge capture — somewhere honest to sit rather than being demoted to
> lenses"* (`:464-468`). Round 10 argued against **run inference**, not against
> that orthogonality; the clause rode in as a qualifier rather than a considered
> reversal, and *applying* the gate is what exposed the conflict. Two independent
> reductios: it demotes the two acts round 3 protected by name, and it condemns
> **four of `exploring.toml`'s five shipped steps** — a runbook whose only
> admissible steps carry mechanical verifiers is a verifier list, which is DEC-101
> and DEC-102 gutted. State-visibility survives as a grade on a condition's
> **evidence**, on the collection limb below. That is the round-3 orthogonality
> restated one level down, not a new escape.

**A stated condition is *state-visible* when both its terms can be read from run
state.** That is a property of the condition, not of the obligation. It decides
what the exercise can *collect*; it never decides what the asset *is*.

**What the exercise collects is adherence, not classification.** With a stated
condition, the observation is mechanical: *did the agent discharge at a turn where
its stated condition was not satisfied?* No inference about a mind, no claim about
what does not exist. It routes as before — agent-general across siblings is
adherence; step-specific means the condition was mis-stated, and the repair is the
condition. **The reopening keeps its two arms and its firing condition; what it no
longer does is derive the classification.** It is collected **only over
state-visible conditions**, and the kit **names which steps those are and reports
the fraction they cover** — an obligation the signal cannot reach is named, never
silently counted into a rate.

##### The gate, applied — all nine 2a obligations

Round 10 wrote the gate and worked one obligation. **F-17 is right that a
discriminator which has not reproduced its own assignments has not been applied**
— and the four this slice relocates were never the whole set. `exploring.toml`
already ships five, and they are 2a assignments this slice made (`(a.3)`, tier 2a,
`:61-75`).

| # | 2a obligation | its stated completion condition | state-visible? |
|---|---|---|---|
| 1 | `explore.scope` | the slice's governing specs and ADRs can be named, and prior art is located or its absence established | **no** — nothing records a read |
| 2 | `explore.research` | the slice's pre-design research round is current | **yes** — `doctrine verify research-current`, the one shipped verifier |
| 3 | `explore.canon` | the ADRs, policies and standards governing this surface have been loaded in this run | **no** |
| 4 | `explore.memory` | memory has been retrieved over the files and subsystems the slice expects to touch | **no** |
| 5 | `explore.triage` | the slice notes carry a triage entry naming each of open questions, risks, assumptions, shaping decisions, constraining governance | **yes**, once the text names its landing site — below |
| 6 | knowledge capture (`:107-109`) | every answer this inquiry accepted that outlives the session has a settled knowledge record shaping this slice | **no** — the left-hand term is in context, not in run state |
| 7 | scope reconciliation (`:115-116`, `:202`) | `slice-nnn.md` asserts nothing the accepted decisions contradict and omits nothing they add to scope | **yes at edge 4** (`design.md` carries the decisions); at edge 2 only so far as the sibling step recorded them |
| 8 | selector recording (`:144-161`) | the recorded selector set matches the code-impact section's current commitments | **yes** — `slice selector list` against the section |
| 9 | repeat-pass judgement (`:187-188` first) | the slice notes carry the statement of what a further review pass would probe, or why none is needed, written after the last pass | **yes**, once the text names its landing site — below |

**Every one states a condition, so every one survives as 2a.** That is the gate
reproducing its own assignments — what F-17 asked for, and what round 10 left
undone. Note what the exercise would have shown had it not: the reclassification
would have been the *design's* to make here, before PHASE-08 authors anything,
which is why this is an authoring gate and not a run inference.

**Five of nine are state-visible, and two of those five were bought.** Rows 5 and
9 name a landing site their earlier texts did not. Naming where a finite result
lands is the cheapest way to move a condition onto the collection limb, and it is
worth taking where the artefact was wanted anyway:

- **`explore.triage`** — *"Record the design surface triage in the slice notes:
  open questions, risks, assumptions, shaping decisions, constraining
  governance."* PHASE-08 already owns `exploring.toml`'s before/after under
  `VA-7(2)`, so this costs no new machinery.
- **repeat-pass judgement** — *"Record, in the slice notes, what a further review
  pass would probe — or why none is needed."* The statement was already the finite
  result the codicil warrants; it simply had nowhere stated to land.

**Row 6 was left alone deliberately.** Making knowledge capture's condition
state-visible needs a durable artefact enumerating *what this inquiry settled* — a
new obligation invented to make an old one measurable. That is F-2's ceremony
objection arriving by the back door, and the trade is refused. The step stays 2a
on a truthful condition with weak evidence, which is the position round 3 ruled
honest and this amendment restores.

**The coverage number is a disclosure, not a footnote.** The adherence signal
reaches **five of nine** obligations. At `N=1` that is a thin instrument, and the
kit must state the denominator it actually covered rather than report a rate over
a set it never reached — the same discipline already required of its deny rate.

**Why this replaces the run-side inference rather than supplementing it.**
Classification is a property of an obligation's text, not of a run: no single
exercise can establish that no truthful completion moment exists. `CHR-049` feeds
evidence into the design; the owner arbitrates the design. Building taxonomical
truth from one observation was the category error underneath rounds 5–10.

**Face two — the exclusion set (F-14).** `stand` covered *anomaly*: unbounded, and
post hoc. Its replacement covers three **named** events, and the enumeration omits
the commonest one — **incidental context exhaustion or compaction**. Nothing marks
it: CHR-049's observation list records *"refresh and exact resume across a
deliberate context break"* and no other context event. So a run can satisfy the
written condition after an unmarked context loss, forcing a revision for exactly
the anomaly the control exists to exclude.

**The run record cannot close this, and was never going to.** Doctrine observes
writes, not an agent's context — the same limit already disclosed for IMP-373's
repayment signal, which is collected from the exercise rather than the journal
because a refusal aborts the write. Of F-14's three remedies, then: *bounding the
exercise below the context window* is unverifiable in advance and would still need
an observation to confirm it held; a *mechanically observable proxy* in the run
record does not exist, for the reason just given; **instrumenting every context
boundary** is the one that lands. Its site is the **moderator protocol** — which
is SL-233's to write, not CHR-049's (that chore's Boundary says so in terms), so
the repair belongs here rather than as a request on the chore.

**Default-deny is what makes the condition decidable.** Firing condition (4)
demands a positive record of the context state; its absence leaves (2) undecided
and disposes of the observation as uncollected. A condition satisfied only on
positive evidence is decidable; one satisfied by the *absence of a mark* is not.

**Cost, disclosed.** Default-deny lowers the probability the signal fires at all,
and at N=1 a falsifier that cannot fire is a formality. Two things bound that:
recording context boundaries is a standing moderator obligation with a named
field rather than a lucky observation, so the deny branch should be rare; and if
it is *not* rare, the kit must **report that** — a run whose context state went
unrecorded is evidence the exercise cannot support this signal, and saying so is
the honest outcome rather than absorbing it.

**The round-6 warrant for removing `stand` has lapsed, and is replaced rather than
restated.** It read: *"A resolution set whose cheap arm is safe does not need an
escape hatch."* Once reword must earn its exit and reclassification is the
default, the cheap arm is no longer freely available, and that warrant no longer
holds. **`stand` goes because it was post-hoc and unconditioned** — anomaly
belongs in the pre-registered firing condition, which is where F-14's repair now
puts all of it, including the incidental case. One principle, twice in one turn:
*every exit and every exclusion is pre-registered; nothing is chosen after the
run.*

**Why these two could not be answered in sequence.** F-15's repair makes
reclassification the default, raising the cost of a false firing from a text
improvement to a placement revision — precisely the cost the round-6 asymmetry
said the cheap arm was there to absorb. F-14 shows the firing condition cannot
currently exclude the likeliest cause of a false firing. Tightening the resolution
set alone would convert an unrecorded compaction into a forced reclassification;
tightening the firing condition alone would only reduce the frequency of an exit
that changes nothing anyway. Each repair is what makes the other safe.

### (a.4) One obligation that changes rather than moves

`:107-109` says: *"capture what should outlive the session via `/knowledge`: an
unresolved question → QUE, a locked design choice → DEC, an assumption the
design carries → ASM."*

That names **three of a closed seven** — `assumption, decision, question,
constraint, evidence, hypothesis, concept` (`doctrine knowledge new --help`). It
is the same defect class as everything else in this phase: a skill restating a
Doctrine-owned closed vocabulary, incompletely, becoming a second source of
truth that can disagree. Here it already does.

The edge-2 step therefore says *record what outlives the session via
`/knowledge`* and **stops**. It also states no linking conventions, because
`/knowledge` already pins them (`plugins/doctrine/skills/knowledge/SKILL.md:42-43,
60-62`): `link <REC-ID> shapes <TARGET>`, `spawns` for work it caused,
`EVD-n supports|disputes <REC-ID>`, and *"the dependent work item authors the
edge … records never author `needs`/`after` themselves."* Restating those in
`/design` would recreate the same problem one level up.

Checked, because the two look like they might be the same mechanism and are not:
**`src/design_run/` writes no knowledge records at all.** Every `DEC-` occurrence
in it is a doc comment citing a decision. Run inquiries are runtime state scoped
to one run and gated by `BlockingInquiriesDispositioned`; knowledge records are
durable, repo-level, and outlive the slice. Complementary, not parallel — so the
step is legitimate rather than a duplicate seam.

---

## (b) What the skill still owns on activation

Two things, and deliberately nothing about workflow order:

1. **Establish or resume the run** — `doctrine design start <slice>` when none
   exists, `doctrine design resume` when one does.
2. **Surface the envelope and do what it says.** The envelope carries the stage,
   the next obligation, and the outstanding runbook steps. The adapter's job is
   to render and obey, not to decide.

Plus a residue register of exactly **one** item, carrying why it is there:

| item | why it is not delivered at a moment |
|---|---|
| `:26-28` the Locked-exit instructions (D8) | **Deferred, not ambient.** The moment is precise — the instant the run reaches terminal. `Stage::Locked` has no outbound forward edge to key a runbook to and `Fragment::for_stage` yields `None` there, so neither 2a nor 2b can hold it as the machine stands. The mechanism is declined until `/plan` gives a second case to design the handoff interface against |

**This answer has now shrunk three times, and each contraction came from being
asked a better question.** The first revision listed **three** items and argued
their smallness was the claim. DEC-103 asked *when must this arrive* and two of
the three answered immediately — `:199` and `:200-201` fire while a draft is being
built — leaving two. The owner's D14 ruling then asked *what kind of thing is
this*, and `:111-113` turned out to be neither design-scoped nor unenforceable,
just badly homed: it belongs in the boot snapshot, doctrine-wide.

**One item is the floor, and it is a defensible one.** `:26-28` is not residue
because no moment exists — the moment is the most precisely identifiable in the
whole file. It is residue because building the mechanism for a single case is how
you get an interface shaped by one example. That is a deferral with a named
condition for repayment, which is a different sentence from *"we could not place
this"*, and telling those two apart is precisely what `VH-1` exists to judge.

**Live evidence, and it now cuts the other way.** During the session that produced
these revisions, the agent authoring the triage violated `:111-113` while
presenting that triage to the owner in unexpanded `D`-numbers. That was recorded
as proof the obligation was *unenforceable*. It was proof of something narrower
and more useful: an obligation buried in one skill's prose does not fire when the
work is happening in a *conversation about* that skill. Moving it to the boot
snapshot is a direct response to the failure, not a shrug at it — and whether it
fires from there is a real question, which is why it carries an evaluation
follow-up rather than an assumption of success.

---

## (c) What it owns on recovery

**When no run exists:** start one. There is nothing to recover.

**When a stale one exists:** `doctrine design resume --run <id>
--known-revision <rev> --known-fragment <frag>` — the compact re-entry
projection, already built (diagram B; `resume` is an existing verb, not
something this phase adds).

The adapter needs **no per-state recovery logic**, and D5 is why: the run
reports where it stands, and the outstanding obligations come back with it. A
recovering agent does not reconstruct which of eight states it was in; it reads
one envelope. The stage hymn already forbids the alternative — *"Recover through
`doctrine design resume`, never by replaying the conversation. Plain resume
never infers missing procedural history: if evidence is absent, it is absent,
and saying so is the correct answer"* (`stage/design.md:11-13`).

The incumbent has no recovery section at all, which is worth stating plainly:
this is not a capability being thinned, it is one being written down for the
first time.

---

## (d) Target body size

**Baseline: 214 lines / 10,178 bytes.** The target is **≤ 60 lines / ≤ 3,000
bytes** for `SKILL.md` alone — a little over a quarter of the incumbent.

**This target tightened under DEC-103**, from ≤ 70 / ≤ 3,500. It is derived, not
chosen, so when the ruling moved five obligations out of prose and into steps the
arithmetic moved with them. What leaves, by the section boundaries in the file:

| lines | section | disposition |
|---|---|---|
| 41 | `:15-58` the state machine, **less `:26-28`** | deleted outright |
| 15 | `:61-75` explore | already a runbook |
| 42 | `:76-117` clarifying questions, **all of it** | `inquiry.md` (2b), less `:107-109`/`:115-116` to the edge-2 runbook and `:111-113` to the boot snapshot |
| 26 | `:118-143` present design | `drafting.md` (2b) |
| 18 | `:144-161` selector recording | edge-3 + edge-4 runbooks (**D10 reversed**) — the only 2a survivor of the guardrail sweep |
| 30 | `:162-191` adversarial review | `reviewing.md` (2b), less `:187-188` first clause to the edge-4 runbook |
| 13 | `:192-204` guardrails, **all of it** | 1 deleted, 1 hymned, **5 to fragments, 1 stepped** |
| 10 | `:205-214` outcomes | deleted but for a purpose statement |

That is **195 lines leaving**, against 192 before the D14 ruling, 174 before
DEC-103 and 177 in the first draft. The 19 lines not in that table are exactly
what remains of the incumbent: frontmatter `:1-14`, the `## Process (detail)`
heading `:59-60`, and the Locked exit `:26-28`. **The table sums to the
incumbent**, which is what `VA-4` requires of it.

What stays or arrives: frontmatter and heading (~16), a purpose statement (~3),
activation / recovery / degradation prose (~25), the one-item residue register
(~3), pointers (~5) — **≈ 52 lines**. The 60/3,000 target is that arithmetic with
a small margin, not an aspiration; it is unchanged from the previous revision
because the margin absorbed the difference.

**Asset bytes are counted separately, and D14 changes their shape as well as
their size.** Roughly 120 lines of process detail are converted, but now across
**two asset kinds**: short `*.toml` runbooks carrying discrete acts, and
substantially larger `*.md` fragments carrying the craft — `drafting.md` alone
absorbs the 26-line content-item list plus five guardrails, against a shipped
baseline of 15 lines. The honest direction is worth naming twice: **the gross
corpus grows while the skill shrinks**, because obligations are being moved into
machinery rather than deleted.

> **AMENDED — RV-325 round 3, F-6 (owner ruling, 2026-08-01).** This paragraph
> used to claim that stating the two asset kinds separately was *"precisely the
> manoeuvre `VA-4` was amended to require"*. **It is not, and the gap is the
> criterion's, not the paragraph's.** `VA-4` as amended names four destinations
> — *deleted / engine invariant / runbook step naming the edge / retained as
> prose* — and requires *"the runbook TOML bytes stated **separately**"*. There
> is **no 2b fragment slot in that vocabulary at all**, so the disposition table
> `VA-4` demands cannot even classify the majority destination, let alone count
> its bytes. `VA-4` was amended to close a relocation loophole under DEC-101, and
> D14 reopened it one asset kind over: relocation into `*.md` fragments is
> invisible to the check, which is the *same defect* one file-extension away.
>
> `PHASE-08 VA-7` is appended to amend it, and per-destination-asset byte **and
> line** totals are required, including duplicated content (`:203` lands in two
> fragments and is counted in both). The `VA-12` precedent governs the form:
> append an amending criterion, never renumber.
>
> **`VA-7` closes the vocabulary over this table's rows rather than adding the
> one kind F-6 named** — a self-audit caught the first draft adding only
> *stage fragment*, which still left **the sealed stage hymn** (`:194`) and
> **the boot snapshot** with no slot. The second of those is a destination
> `EX-10` created in the same round, so the F-4 and F-6 repairs would not have
> composed: two of these twelve rows would have been unclassifiable by the
> criterion written to account for all of them. That is F-9's finding — fix the
> class, not the instance — recurring inside the repair for F-6, and it is
> recorded here because catching it required re-reading the table against the
> criterion rather than trusting that the cited instance was the whole of it.
>
> The rule this leaves behind, which is the durable part: **a destination
> appearing in (a.3) with no slot in `VA-7` is a defect in the criterion, not
> in the table.** The table is the census; the criterion must be total over it.

**On that amendment.** `VA-4` originally asked an agent to judge whether the
body *"substantially shrank"*. Target-state work showed the check was defeatable
by construction: under DEC-101 the checklists move into a **different file**, so
shrinkage is satisfiable by relocation alone and no byte count distinguishes
relocation from mechanism. That is the same defect class `VA-5` exists to catch
on `EX-9`, sitting inside the criterion written to prevent it. On 2026-08-01,
pre-start and pre-evidence, `VA-4`'s judgement clause was withdrawn to a new
`VH-1` (the owner reads the prose and judges, bounded by D7/D8/D10 as the
decisions that make residue legitimate) and `VA-4` was strengthened into a
disposition table that must account for every part of the incumbent body with
runbook bytes stated separately. Ids retained, coverage unchanged in total,
following the `VT-2` precedent in the same plan; disclosed in `plan.md`.

**Two notes for whoever judges `VH-1`.**

*The criterion's text uses the old question numbering.* `VH-1` in `plan.toml`
names "the Locked exit keeps its instructions in the adapter **(Q1)**" and
"design-target selector recording stays where the incumbent prose puts it
**(Q4)**". Those were renumbered when diagram C recorded the rulings. The
crosswalk: **Q1 → D8**, **Q2 → D7**, **Q3 → D9/D11**, **Q4 → D10**. The criterion
text was left alone rather than amended a second time in one week.

*One of `VH-1`'s three bounds no longer bounds anything.* The criterion says
residue is legitimate where D7, D8 or D10 accounts for it. **D10 is now
reversed** — the selector recording is delivered on edges 3 and 4 — so nothing
remains under its warrant, and `VH-1` is correspondingly **stricter** than when
it was written. That is disclosed rather than treated as a reason to re-amend:
a verification criterion becoming harder to satisfy mid-phase, because the design
improved, needs no repair.

---

## (e) The degradation path

`EN-2` requires this and the owner binned it as a driver — *"(e) in the bin"* —
so it is answered briefly and on evidence.

**The rule: detect and surface. Do not self-heal, and do not fall back to
executing the old machine.**

An earlier draft of this answer argued that an installed skill cannot outrun its
binary, because skills are projected from the binary's own embed. **That is
wrong for Claude and is corrected here**: skills ship from
`github:davidlee/doctrine@main` on a manual plugin update that the harness may or
may not honour, so a skill and its binary can desync **in both directions** — a
new skill against an old binary, or an old skill against a new one.

The consequence is bounded rather than alarming. `doctrine design` absent or
erroring is a condition the adapter reports and stops on; the typed `Refusal`
vocabulary already serves the "errors" half with messages that name what was
objected to. Engineering a graceful degradation ladder is unwarranted against a
known user population of roughly four, all of whom are reachable. Detection
beats degradation here, and saying so is a decision rather than an omission.

---

## (f) Which skills cross-reference `/design`, and what breaks

Swept over `plugins/` and `install/`, then widened to the memory corpus.

**Eight shipped skills name it:** `slice` (hands off to it, `:42`, `:60`, `:64`),
`plan` (four clauses returning to it, `:3`, `:20`, `:26`, `:33`), `phase-plan`
(`:43`), `preflight` (`:31`), `research` (declares `/design` its consumer,
`:85-86`), `spec-tech` (`:9`, `:66`), `spec-product` (`:18`, `:343`), `pair`
(`:107`).

**Two shipped reference docs:** `install/routing-process.md:16` — the routing
table row, already a PHASE-08 target via `EX-2` — and `install/review-ledger.md:92`.

**Six memory items**, two in the shipped corpus and four local.

**One false positive:** `install/manifest.toml:61-64` matches `stage/design`,
the sealed hymn, not the skill.

**What breaks: nothing, and the shape of the sweep is why.** Every hit is a
**routing** reference — "go to `/design`", "come back from `/design`", "`/design`
consumes this". Not one references the skill's internals, its state names, or
its numbered steps. So the adapter keeps the activation name and the handoff
contract, and every cross-reference remains true.

The owner's ruling on this question: **state resumption accounts for these and
other naturally arising re-entry points, and probing that is the reviewer's
job.** That is recorded as an assertion, not a proof, deliberately — it is
offered as the thing in this document most worth attacking, and (c) is where the
attack should land.

---

## What this sketch does not claim

- **It does not claim the runbooks are authored.** Their step text is specified
  by source line, not written. Authoring is phase work.
- **It does not claim the one-item residue register is the minimum.** It is the
  minimum *this* triage found, and the triage has been wrong in three distinct
  directions already: two deletions that dropped an obligation where it fires
  (`:198`, `:203`); three "ambient" items that had a delivery moment the instant
  anyone asked (`:199`, `:200-201`, `:144-161`); and one item held as
  unenforceable residue through every revision until the owner observed it was
  not design-scoped at all (`:111-113`). What survives is `:26-28`, deferred
  rather than kept. It is labelled, not defended.
- **It does not claim the vigilance denominator was always complete.** It is now:
  a sweep of diagram A for every `vigilance` and `orphan` node returns
  `A s2.rules` (which decomposes into `r1`/`r2` → `inquiry.md` and `r3` =
  `:111-113` → the boot snapshot), `A s3.note` (diagram meta-commentary about A itself, not an
  obligation in the skill), `A sel.x3` (the D10 trigger, now delivered), `A
  s6.a6` (deleted) and `A guard` (the eight guardrails). No uncounted obligation
  remains. That sweep was run only after a reviewer-facing note admitted it had
  not been.
- **It does not claim `EX-7`'s handover adapter is designed.** `EN-2`'s six
  questions are all about the *design* skill; the handover convergence
  (DEC-058, `plugins/doctrine/skills/handover/SKILL.md`, 114 lines / 5,037
  bytes) lands in this phase **unsketched and outside this gate**. Disclosed
  rather than absorbed.
- **It does not claim diagram B is current.** B predates PHASE-16: it says "six
  writer acts" where there are now seven, and its gap g1 is closed. Where this
  document inherits from B, check B's age before treating a difference as a
  conflict.

## For the reviewer

Ranked by what costs most if it is wrong.

**This list was re-ranked after a self-check pass.** D5 previously sat at #1 as
the assumption most likely to be wrong; checking it against source showed it is
the best-evidenced claim in the document, and it has moved to #5 restated as a
durability question. The triage that was #3 has moved up, because the same
self-check found two of its four deletions do not hold — and the owner's ruling
on those two became DEC-103, which is now #1 in its own right. Both movements are
disclosed rather than quietly re-ordered — where the ranking was wrong is itself
evidence about the method.

**One disclosed defect this sketch has not repaired.** DEC-102's `consequences`
assert the `SKILL.md:98` / `CLAUDE.md` contradiction *"is therefore fixed FOR
THIS REPO"* in the present tense, and it is not — `:98` is verbatim unchanged and
stays that way until PHASE-08 lands. D12 makes the claim true on delivery. The
tense is wrong *now*, in an `accepted` record, and the repair (amend to
conditional, independent of which runbook ships) was proposed and not yet ruled
on. Flagged here rather than silently amended: unilaterally editing an accepted
decision record to make a sketch look consistent is the wrong instinct.

1. **DEC-103 is new, it reshaped this document, and it is the thing to attack
   first.** *Instruction is delivered at the moment it must take effect; prose is
   a failure to locate a delivery moment, not a destination.* It was ruled after
   D1–D12, and it reversed D10, re-warranted D8, repaired two failed deletions,
   and collapsed the tier-3 residue from three items to two. Three attacks are
   open on it, and they land on the ruling rather than on the phase:
   **Two of the three attacks previously listed here have been resolved against
   the sketch, by RV-325 F-2 and the owner's D14 ruling respectively.** They are
   recorded rather than deleted, because what replaced them is the live surface:

   - ~~*Is it too strong?*~~ **Landed, as F-2.** The unbounded reading produced a
     nineteen-step edge 3 and nineteen serial CAS turns. The resolution was not a
     limit on the ruling but D14: most of that population were never steps. Edge 3
     now has one.
   - ~~*The residual worth attacking is the trade.*~~ **Landed, as F-7.** A
     fragment cannot be discharged, so the rewrite buys delivery-where-it-bites at
     the cost of every attestation that the drafting lens was read. F-7's point was
     not that the trade is wrong but that *no observation could show it was* — the
     sketch stopped at "no test can fail here", which is true of the gate and was
     doing duty as a reason to state no decision rule. Now pre-registered.
     **Then landed a second time, as F-10** — the first remedy routed a negative
     *delivery* result into a *classification* change (2b content back to a
     coverage-set 2a step) while exempting DEC-104 from being reopened by it,
     which is both incoherent and F-7's own objection re-inserted one level up.
     Four signals now, routed by subject: three delivery signals whose revision
     is mechanism-side and leaves DEC-104 standing, and one misclassification
     signal that reopens the discriminator and nothing else. `PHASE-09 EX-8`,
     with `VA-5` to stop the two being merged into one score.
   - ~~*Is `:111-113` genuinely unenforceable?*~~ **Landed twice.** First on
     scope: the owner's answer was that it is not design-scoped at all and belongs
     in the boot snapshot, doctrine-wide. Then on timing, as **F-4** — the sketch
     deleted it here and backlogged the replacement, creating an unbounded interval
     of zero delivery for a rule DEC-103 says must arrive where it bites. Now
     atomic in PHASE-08 (`EX-10`). **The live question is whether an obligation in
     the boot snapshot actually fires** — the snapshot is large and always present,
     which is the definition of the ambient delivery DEC-103 distrusts. Placement
     is asserted; proof is deferred to CHR-049. That is an efficacy question and no
     longer a delivery gap.
   - ~~*Does it pull against DEC-101?*~~ **Dissolved.** The two coverage sets that
     created the tension are fragment prose now, and prose has no order to fake.
   - ~~*Is D14's discriminator sound?*~~ **Landed, as F-3 — the sharpest finding
     of the review, and it was one this sketch disclosed rather than found.** The
     rule sorted by whether a verifier *could ever* corroborate an obligation,
     while conceding that none of the surviving 2a steps ships one. F-3's
     diagnosis was exact: *could ever* admits an imagined future verifier for
     almost any lens, so the rule could not reproduce its own assignments. The
     replacement makes completability-at-a-boundary the discriminator and demotes
     verifiability to an orthogonal evidence-strength axis. **Every assignment
     survived**, which is the tell that the original was a sound judgement resting
     on the wrong reason.

   The evidence DEC-103 was ruled on is in (a.2): `:203` deleted against
   `explore.canon`, a step on **edge 1**, for an obligation that bites at edges 3
   and 4; `:198` deleted against a hymn bullet whose subject is **the inquiry
   map**, not the design document's prose. Both were defensible under DRY and
   both dropped the obligation where it fires. The two surviving deletions
   (`:196` → boot snapshot; `A s6.a6` → hymn `:24` **and**
   `Condition::UserAcceptanceAttested`) are stated with their sources so you can
   check them the same way.

2. **DEC-102's overclaim is specific, not rhetorical — which shifts the choice
   between the two repairs.** The record's `consequences` do not merely say the
   craft has moved into data; they say the `SKILL.md:98` / `CLAUDE.md`
   contradiction *"is therefore fixed FOR THIS REPO"*. `SKILL.md:98` is verbatim
   unchanged. Worse for the cheap repair: DEC-102's own `alternatives` field
   rejects the keep-everything-fixed option **because it "leaves the SKILL.md:98
   contradiction unfixable in place"** — the decision's rejection reasoning leans
   on that contradiction becoming fixable. So "amend DEC-102 instead" withdraws a
   load-bearing consequence rather than softening a flourish, and D12 is the
   cheaper repair of the two once the record is read in full. This is still worth
   attacking from the other side; it is no longer the balanced fork an earlier
   draft of this sketch described.

   **D14 changed this fork again; F-5 settled it.** Inquiry craft now lands in
   `inquiry.md` rather than an edge-2 runbook, which satisfies DEC-102's *"moved
   out of skill prose into data"* without D12 — so D12's original warrant is met
   by other means, and what remains for it is two discrete acts. The sketch left
   that open and called it a disclosure; **F-5 was right that it is not one**,
   because PHASE-08 must author the assets and an open fork hands the choice to
   implementation. Ruled: **edge 2 carries a runbook of exactly two steps**, and
   the tense problem is untouched by that ruling — `inquiry.md` is what makes
   DEC-102's claim true on delivery, with or without the runbook.

   **What remains genuinely unruled here is the tense itself.** DEC-102 says the
   contradiction *"is therefore fixed"* while `SKILL.md:98` is verbatim unchanged.
   The repair was proposed, is not ruled on, and round 3 declined to raise it.
   More seriously, the *"overridable"* half of DEC-102 is false either way (see
   #3), which no runbook can repair.

3. **Nothing in `design-prompts/` is project-overridable, and two shipped
   artefacts say otherwise.** The authoring rule promises *"Yes → a runbook step.
   **Overridable**, verifier substitutable"*; DEC-102 promises craft that a
   project can bend. Both are false of this store: runbooks and fragments alike
   resolve to embedded assets (`runbook.rs:185-189`, `prompt.rs:68-70`), and
   `prompt check` *"knows nothing about this store's `*.toml` siblings"*
   (`runbook.rs:318-322`) — the `.doctrine/hymns` overlay, `replaces` suppression
   and seal/expose machinery do not reach it. `design.md §7` chose this store
   deliberately and for a sound reason (`KNOWN_STAGE_LABELS` is an *enforced*
   lifecycle vocabulary and `drafting` is not a lifecycle stage), but in doing so
   it **bundled two separable questions and answered only one**: whether
   intra-design obligations should be `stage`-band labels (no, correctly), and
   whether they should be project-overridable (never asked — inherited from the
   mechanism). The attack: does D14 make that bundling worse by moving *more*
   craft into the un-overridable store, and should `§7` be reopened before
   PHASE-08 lands rather than after? This sketch says backlog; argue it the other
   way if you can.

4. **(f) is asserted, not proven.** The claim that state resumption accounts for
   every re-entry point is the owner's ruling, recorded as such. It is the most
   probeable claim in the document and (c) is where to probe it.

5. **D5 is durability-limited, not doubtful.** The four-edge claim is settled by
   `gate.rs:150-158` (a nine-line literal `matches!`) and locked by an
   exhaustive 25-pair test (`src/design_run/tests.rs:56-77`), so a fifth edge
   cannot land silently. Do not spend a turn re-deriving it. The residual worth
   probing is narrower and real: **is a second forward edge ever wanted?** If
   design §5.4 or DEC-067 contemplates a branch — an optional stage, a skip, a
   conditional path — then D5 is true-but-temporary and every disposition in (a)
   is scheduled to move. Note also that the blast radius is wider than the
   runbook keying: `cumulative_conditions` (`gate.rs:208-218`) walks
   `Stage::ALL.windows(2)` and assumes a chain, and `forward_runbook`
   (`src/commands/design.rs:1801-1806`) walks `Stage::ALL` to find the single
   edge. A branch breaks three call sites, not one.

6. **The `Outcomes` omission is disclosed evidence about method.** A triage
   driven by a question list covers what the list names, and `Outcomes` was
   named by nothing until (d)'s arithmetic refused to close. Ask what else the
   six questions do not name. The self-check pass above is a second instance of
   the same lesson: the citation errors it found (state 8's line range, D5's
   circular cite, three different guardrail counts across two artefacts) were all
   in material no question interrogated.

7. **The authoring gate is now applied, and the number it produced is the live
   surface.** *"The gate, applied"* works all nine 2a obligations; all nine state
   a condition, and **five of nine** are state-visible. Two questions, and neither
   is settled by the table:
   - **Is a signal reaching five of nine worth running at `N=1`?** It fires only
     where a condition is state-visible, so the four it cannot reach include the
     one the whole argument has circled since round 3 — knowledge capture. A
     falsifier that structurally excludes the hardest case is thin, and the
     honest reading may be that the misclassification signal is a *post-close*
     instrument rather than a CHR-049 one. Argue that if you can; the disclosure
     is here rather than absorbed into a rate.
   - **Should knowledge capture have bought visibility after all?** The trade was
     refused because the artefact enumerating *"what this inquiry settled"* is a
     new obligation invented to measure an old one — F-2's objection by the back
     door. But `/knowledge` records already exist, and a step that named its own
     sweep's result would cost less than the argument has. The refusal is a
     judgement about ceremony, not a proof.

   **What is settled, and should not be re-opened:** the gate classifies on
   *truthful*, not on *visible* (round 11, restoring round 3), and no run can
   establish an obligation's classification (round 10). Rounds 5–10 cost six
   rounds to the second of those.
