<!-- doctrine:section sec-1 -->
## Governing context

The records this design is bound by, and the judgement about *how* each binds.
The records are canonical and hold their own content; this section cites and
judges, and deliberately restates nothing. Applicability — including the
not-applicable checks and the reason each was set aside — is recorded in
`.doctrine/slice/244/research/research.md` § Thread 1.

- `DEC-101` — narrowing an open vocabulary to a closed one is a type error.
  Binds the *sourcing* of satisfaction, not the condition set itself.
- `DEC-102` — seal content when an override would make it false. Its override
  seam does not exist yet; see the assumption in the concerns section.
- `DEC-066`, `DEC-067` — evidence liveness against current content, and
  cumulative re-derivation of clearance on every forward move. Together they are
  why clearance is never stored, and why this design cannot answer with a flag.
- `STD-001` — single-source named constants. Reaches further here than usual:
  the kebab condition token is load-bearing in four places.
- `SPEC-029` — owns the gate table and describes evidence as payload-claimed.
  The certain revision candidate of this slice.
- `ADR-001` — module layering. `design_run` is tier `leaf`, which is why the
  gate is modelled on `src/review.rs::can` rather than reusing it.

The one structural fact the design turns on is in-tree and argued by the
incumbent, not asserted by this slice: `Condition` is payload-free, and that has
already cost the codebase a refusal variant — `RunbookNotDischarged` exists
because `GateNotCleared`'s `Vec<Condition>` has *"nowhere for a step identity to
ride"* (`src/design_run/refusal.rs:166-170`).

**This section's own provenance is a finding against the mechanism it describes.**
It exists because the `exploring → inquiring` boundary can bind clearance only to
a `design.md` section (`src/design_run/run.rs:1471-1478`), so the run could not
record that governing context had been captured until that context was restated
as prose. Holding this section to citation-plus-judgement narrows the duplication
but does not dissolve it. Recorded as `EVD-012`, which assesses the forced
restatement as a violation of the single-source-of-truth rule and as poorly
conceived independent of that. `DEC-126` is where the design answers it.

<!-- doctrine:section sec-2 -->
## Concerns carried into the design

Stated at the outset so later sections argue against a fixed list rather than a
remembered one. Each is a live risk to this design, not a general caution.

- **Snapshot versioning.** A shape change to the design-run snapshot costs the
  one live run there is — `SL-243`'s. It sits in the gitignored runtime tier, so
  the cost is bounded, but it is not zero and it is not recoverable.
- **Envelope byte budget.** The refusal path has no byte budget; the envelope has
  a hard one, and `clearances` already rides it uncapped. Anything this design
  adds to the envelope competes with that.
- **A fourth prose loader.** Three prose systems already have three loaders. A
  fourth needs an argument, not a precedent.
- **`is_derived()` asymmetry.** Six conditions claimed, four derived. This is
  `IMP-361`'s known deferred gap, already stated in `gate.rs`'s own doc comment —
  a carried debt, not a discovery of this slice.
- **`CHR-049` is one moderated run.** Adequate to inform `ISS-285`'s deferred
  choice; not adequate to settle it alone.

**Assumption the design carries.** `DEC-102`'s override seam does not exist. The
available move is embedded-and-`fixed`-with-a-citation — the pattern runbooks
already use. If that seam is built later, this design's choice becomes a
constraint it need not have accepted.

**Interim states this slice knowingly ships**, stated rather than accidental:
`exploring → inquiring` passes on the runbook alone until `IMP-391` builds the
attested checkpoints; the `Conducted { review }` arm and the severity summary are
unbuildable until `IMP-392` migrates findings onto `RV`.

<!-- doctrine:section sec-3 -->
## The condition model

The foundational section: every later section is written against the vocabulary
fixed here.

### Current behaviour

A `Condition` is a fieldless enum variant. Its entire data content is which of
ten names it is (`gate.rs`, the `Condition` enum). Three associated items are the
whole API: `ALL`, `is_derived()`, `as_str()`.

Satisfaction forks on that boolean:

```rust
fn satisfied(condition: Condition, facts: &DerivedDesignFacts, standing: ReviewStanding) -> bool {
    if condition.is_derived() {
        standing.holds(condition) == Some(true)
    } else {
        facts.satisfies(condition)
    }
}
```

The two arms are not two ways of answering one question. They are two different
questions:

- **the derived arm** recomputes the answer from the run's own state
  (`ReviewStanding`, itself derived and never stored);
- **the claimed arm** runs an existential scan over evidence rows —
  `live_evidence().any(|e| e.condition() == condition)` in
  `DerivedDesignFacts::satisfies` — which asks only whether *someone asserted
  this against some subject whose bytes have not moved*.

The `DEC-066` fingerprint binding makes a claim **expire**. It never makes one
**true**.

### Target behaviour

`DEC-120`, as sharpened on 2026-08-03: **every condition is derived. What varies
is who can author the state it derives over.** Attested names *input provenance*,
not a second decision procedure.

So the fork above **dissolves**. There is one derivation:

```rust
fn satisfied(condition: Condition, run: &DesignSnapshot, derived: &DerivedInput) -> bool
```

Two inputs, and the second is deliberately the *whole* derived-input record
rather than a parameter per fact class. A condition derives over run-owned state,
over what the shell read out of the authored document, or over canonical state
outside the run — and which of those it needs is a property of the rule, not of
the signature. An earlier draft passed a narrow `&ObservedFacts` instead, and
`Engine(Materialisation)` immediately became underivable: the watermark
comparison needs `authored_fingerprint`, which lives on `DerivedInput` and was no
longer in scope. One derived-input parameter is also what lets a new observed
fact arrive without touching every call site.

`ConditionKind` remains `DEC-120`'s vocabulary, but it is **not a field and not a
dispatch key**. It is a projection of the derivation rule:

```rust
pub(crate) enum ConditionKind { Derived, Attested }

impl DerivationRule {
    pub(crate) const fn kind(&self) -> ConditionKind { /* discriminant */ }
}
```

Two variants, not three. `Claimed` is named in `DEC-120` as the defect class and
is **not representable** — a tier with no legitimate members is not a tier, and
leaving it constructible invites a new member.

### Conditions are edge guards

The governing reading, stated here because several later choices follow from it
and were previously asserted without it: **a condition guards a transition, not a
progression.** It is a guard on one edge of the stage machine, and what it means
is fixed by the edge it sits on.

Two consequences the rest of this section depends on.

**There is no global default for how far a guard reaches.** Reach is a per-row
commitment, argued per row, because the argument is contextual: research
invalidated by a later change should plausibly force a restamp, while document
review invalidated by every revision would regress the run backwards after every
dispositioned finding. Same mechanism, opposite right answer. A model that reads
*cumulative unless excepted* or *edge-local unless argued* gets one of those two
cases wrong by construction.

**`DEC-067` is a mechanism, not the default.** Its cumulative re-derivation
exists so that a regression cannot leave stored clearance behind and a return
forward cannot inherit clearance it no longer earns. That is why `advance`
re-derives from current content rather than reading a flag. It is the right
mechanism for most rows and it is not automatically right for all of them, and
this section now says which rows take it and why.

**A guard bars a move.** A derived fact that must *not* bar a move is therefore
not a condition at all, whatever else it resembles. Where those facts go, and why
they are excluded rather than softened, is the last subsection before the
classification.

### The derivation rule

One coupled value, not independent `kind` and `subject` fields. Independent
fields made `Derived` + "derived over an attestation" representable and
meaningless; the coupling makes the contradiction unspellable rather than merely
discouraged.

```rust
pub(crate) enum DerivationRule {
    /// Recomputed from run-owned state; the variant names which state.
    Engine(EngineSource),
    /// Derived over recorded attestations of one or more acts.
    Attested(AttestationRule),
}

pub(crate) enum EngineSource {
    /// The inquiry map's dispositions.
    Dispositions,
    /// The authored watermark against current section digests.
    Materialisation,
}
```

**The discharging act is stated exactly once**, inside the rule. There is no
`remedy: &'static str` beside it: a refusal's remedy text is *rendered from* the
rule, so the two cannot disagree. This is `DEC-123`'s injection requirement —
*"the renderer injects the kind and the discharging act from the const into the
rendered contract, so the prose never restates them and cannot contradict them"*
— applied to the refusal channel as well as the prose one.

### Attestation: acts, actors, and what invalidates them

```rust
pub(crate) struct AttestationRule {
    /// Every act that must be present. A CONJUNCTION.
    pub(crate) acts: &'static [ActRequirement],
    /// What the attestations bind to, and therefore what invalidates them.
    pub(crate) binding: Binding,
}

pub(crate) struct ActRequirement {
    pub(crate) act: ActKind,
    /// Who must have authored it. NOT optional — though one requirement names
    /// the actor indirectly; see `RequiredActor` below.
    pub(crate) actor: RequiredActor,
}

pub(crate) enum ActorClass { User, Agent, Adversarial }
```

Two things this shape exists to fix, both of which the single-act sketch could
not express.

**A condition may require more than one act.** `DEC-121` is explicit: *"Initial
concerns. Two acts, not one."* The user reviews the seeded inquiry graph and
steers before interrogation goes deep; and the agent declares which questions it
considers settled or settleable without asking, for confirmation. `DEC-121` calls
the second the more consequential and notes it currently has no expression at all
— *"an agent marks its whole graph blocking by fiat, and the difference between
eight round-trips and four is invisible."* A rule carrying one act cannot say
this, and a refusal over it cannot say which half is missing — which was
`DEC-121`'s own stated reason for rejecting the fold into
`user-accepts-sufficiency`.

**Actor authority is required, not optional.** `DEC-126`'s discriminator *is*
actor identity — the line is not *is it a judgement* but *does the actor's
identity matter*. A rule that omits the actor cannot express the very conditions
it classifies.

**On `ActorClass` and the two incumbents.** `Reviewer { Human, Adversarial }`
(`attestation.rs`) records who reviewed a *section*;
`traversal::Authority { AgentProposed, UserPinned }` records who directed a
*traversal*. Neither is this axis, and this design deliberately does **not**
unify all three — they answer different questions, and collapsing them is the
hierarchical-state-machine error `DEC-065` rejects. What it does require is that
`Reviewer` become expressible as an `ActorClass` for
`section-attestations-current`, because that condition's rule must name a
required actor and today's derivation names none. See `ISS-310`.

**One row's required actor is run data, and the actor slot must say so.**
`section-attestations-current` is the only row whose required actor is not a
constant of this table: *which* reviewer lanes each section needs is declared
per run under `DEC-073`'s review policy rather than fixed here. It is also the
only row carrying `PerSection` coverage, but those are two facts and not one —
coverage fixes the subject set a requirement ranges over, the actor slot fixes
who must have authored the act, and the same discipline this section applies to
reach and coverage applies here.

`ISS-310` is answered in the attested-acts section. What belongs here is the type
each candidate answer costs, because `acts` is a conjunction — two
`ActRequirement` entries mean *both*, never *either*:

- **human**, a fixed actor — fits `ActRequirement` exactly;
- **either** — needs the actor slot to admit a disjunction, which the conjunctive
  `acts` slice deliberately does not;
- **per section, where the section opted in** — is not a typing gap at all. The
  requirement would vary per *section* from run data, so no shape of the actor
  slot puts it in a `&'static` table;
- **per run, from `DEC-073`'s policy** — the answer taken, and expressible
  statically, because the table names *where the actor comes from* rather than
  naming the actor:

```rust
pub(crate) enum RequiredActor {
    /// Fixed by the rule. Seven of the eight requirements.
    Fixed(ActorClass),
    /// Read from the run's review policy (DEC-073). Exactly one requirement.
    RunPolicy,
}
```

The table stays `&'static` and stays total; resolution moves into `satisfied`,
which already receives the whole snapshot rather than a parameter per fact class,
so the policy arrives with no signature change. What makes this work where the
per-section candidate fails is *what varies*: a per-run policy is one value an
evaluation reads once, where a per-section rule would need the table to hold a
different actor per subject.

**Conjuncts must be current together, and currency is not yet ordering.** A
conjunction of independent acts is satisfiable by acts that never met: an old
user `GraphReviewed` and a later agent `BlockingSetDeclared` would both be
*present*, and `DEC-121`'s interaction — the agent declares, the user confirms —
would not have happened. Coverage closes most of this by construction: where the
rule's binding covers a run-owned subject set, every conjunct must be current
against the *same present* state, so a review made over a superseded inquiry map
is not live and the pair cannot be assembled from different eras. What coverage
does **not** give is order — that the user's confirmation came after, and over,
the agent's declaration. That requires the confirming act's artefact to cover the
declaring act's digest, which is a statement about artefact shape rather than
about this rule, and it is specified in the attested-acts section. Recorded here
so it is a deferral and not an omission.

```rust
pub(crate) struct Binding {
    /// What the attestation's own recorded content covers.
    pub(crate) coverage: Coverage,
    /// Canonical facts outside the run that must ALSO still match. Conjunctive
    /// with the coverage, and with each other.
    pub(crate) observed: &'static [ObservedFact],
}

/// The run-owned subject set an attestation is current against. `Artefact` is
/// the degenerate case: it covers nothing but itself.
pub(crate) enum Coverage {
    /// The attested artefact's own recorded content, and nothing else.
    Artefact,
    /// Each required act carries a covered map that must equal every section's
    /// current digest.
    EverySection,
    /// Each required act carries a covered map that must equal every inquiry
    /// node's current canonical material.
    InquiryMap,
    /// Quantified over subjects instead of carried by the act: every current
    /// section must have its OWN live act of the required kind and actor,
    /// against that section's current digest.
    PerSection,
}
```

**`PerSection` is not `EverySection` with the acts spread out**, and the
incumbent is where the difference is visible: `review_standing`
(`snapshot.rs:419-433`) derives both, ten lines apart, in two different shapes.
`acceptance_current` is `acceptance.is_current(&current)` — whole-map equality
against a map *stored on the act*. `sections_attested` is a quantification with
no stored map at all, assembling its covered set from the attestation set at
evaluation time:

```rust
!current.is_empty()
    && current.iter().all(|(id, fingerprint)| {
        attestations.iter().any(|held| {
            held.subject() == id && held.fingerprint() == fingerprint
        })
    })
```

Two properties follow that whole-map equality does not have, and both would be
lost by spelling this row `EverySection`.

**A departing section is not a failure.** The quantification ranges over the
*current* sections, so removing one drops it from the requirement and the row
still holds. `ContentCoverage::is_current` compares maps in both directions, so
a departure invalidates. For a stored acceptance that is right — it was given
over a document that no longer exists. For per-section review it is wrong: the
reviews that remain are still good.

**The empty case must be refused, and equality cannot refuse it.** The incumbent
carries an explicit guard with its reason stated in-line — *"a run holding no
sections cannot have current attestations for them, and reporting `true` would
let an empty draft lock."* An empty stored `ContentCoverage` against an empty
current map compares **equal**, so an `EverySection` spelling would report this
row satisfied on a document with no sections. The guard is part of what
`PerSection` means, not a check beside it.

`EverySection` is otherwise the incumbent pattern generalised, not a new idea:
`ContentCoverage` already stores the subject→fingerprint map an acceptance was
made over and compares it with the current one (`attestation.rs:200-214`). One
row needs it — `user-acceptance-attested`.

`InquiryMap` is the same *comparison* over the other run-owned subject set, but
**not** over digests, and the difference is forced rather than chosen. A section
carries its own fingerprint because the shell digests a declared body. An inquiry
node does not, and cannot cheaply be given one: nodes are mutated by pure code —
disposed, re-parented, `needs` edited — with no shell round-trip to re-digest,
and `DerivedInput` is built *before* `apply` runs the batch, so a shell-supplied
node digest map would be a pre-batch answer to a post-batch question.

`ContentCoverage`'s own doc already makes this argument in its own case —
*"Deliberately not a composite digest. The pure layer never hashes … Comparing
the map is pure, needs no new injection, and cannot bind to a stale value"*
(`attestation.rs:192-195`). `InquiryMap` takes that route one step further and
compares the **material** rather than a digest of it. So the incumbent type
generalises over what it covers:

```rust
pub(crate) struct ContentCoverage<T> {
    covered: BTreeMap<DesignId, T>,
}

impl<T: Eq> ContentCoverage<T> {
    /// Whether every covered subject still carries what it was covered at —
    /// and nothing has joined or left.
    pub(crate) fn is_current(&self, current: &BTreeMap<DesignId, T>) -> bool {
        &self.covered == current
    }
}
```

`ContentCoverage<Fingerprint>` is today's type with today's behaviour: whole-map
equality, so a subject joining or leaving still invalidates.
`ContentCoverage<NodeMaterial>` is the inquiry map. One comparison, two
instantiations, and a refusal can name *which node* moved rather than reporting
that the map did.

**What `NodeMaterial` holds, and what it deliberately omits.** `InquiryNode`
(`inquiry.rs:204-222`) carries `id`, `question`, `provenance`, `lifecycle`,
`disposition`, `parent`, `needs` and `seq`. The material is all of them except
`lifecycle` and `disposition`.

What the user reviewed under `DEC-121` is *the set of questions and how they
relate* — the graph they steered. A question later being answered is progress
through that graph, not a change to it. Admit disposition and
`user-accepts-sufficiency` expires on the next disposal, permanently, because
disposal is precisely what the following stage does. The exclusion is not a
convenience: disposition is already guarded by
`blocking-inquiries-dispositioned`, so admitting it here would double-guard one
fact while destabilising two conditions.

Re-wording a node, re-parenting one, adding one or removing one all change the
material and expire the review — which is the property `DEC-121` asks for, since
steering the graph's shape is what the user is doing.

**Binding is conjunctive, and the empty case is why.** An earlier draft made
these alternatives — an attestation bound to its artefact *or* to an observed
fact. `DEC-121` makes that unsafe on the row that needs it most: the confirmation
artefact for `governing-context-recorded` carries dismissals, their reasons, and
the search evidence, and *"the empty case becomes the strict path in both
checkpoints: a governance sweep finding nothing … shown with what was searched
and never skippable."* Bind only to the observed edge set and an empty sweep
fingerprints a stable nothing: the condition holds forever while the entire
substance of the strict path — what was searched — is bound to nothing at all.
An attestation is always over its own recorded content; observed facts are an
additional conjunct, never a substitute.

### Observed facts: what the snapshot cannot see

`DEC-121` makes the artefact for `governing-context-recorded` the **confirmed
governance edge set**, and states that *"enumerating governance ids in design
prose is not the artefact and does not substitute for it — that would be queried
data in prose."*

That state is authored **outside** the design run. `DesignSnapshot` carries
`receipts`, `map`, `gate`, `sections`, `review` and `delegation` groups and no
relationship graph; `DerivedInput` carries section digests, authored sections,
the authored fingerprint, runbook facts and verifier results, and no canonical
edge observation. So a derivation with only the snapshot in hand can check that
an attestation *exists* and nothing more — which is the existential scan
returning under a better name.

The repair rides an existing seam rather than inventing one:

```rust
/// Canonical state the shell observed this invocation, for conditions whose
/// binding names one. Transient — never persisted, and never a mirror of
/// canonical state the run does not own. Arrives as a field on `DerivedInput`.
pub(crate) struct ObservedFacts {
    pub(crate) facts: BTreeMap<ObservedFact, Fingerprint>,
}

pub(crate) enum ObservedFact {
    /// The slice's canonical governance relationship edge set.
    GovernanceEdges,
}
```

It arrives on `DerivedInput`, where every other shell-established fact already
arrives, and for the reason stated there: *a fact about what an asset says is one
Doctrine derives, never one it takes on a caller's word.* `DEC-123` independently
forces the same placement — *"no subprocess inside apply's admit→persist span, so
a condition verifier can never run inside `advance` — shell-side, entering as
derived input."*

Semantics, stated so they are not assumed:

- **Refresh** — observed each evaluation, never cached in the snapshot.
- **Comparison** — the attestation stores the fingerprint it was made over; the
  condition holds while the observed fingerprint still matches. This is exactly
  `DEC-066` applied to a subject the run does not own.
- **Absence** — an unobservable fact reads as **changed**, not unchanged. The
  gate stays shut on a missing answer, the rule `ReviewStanding::holds` already
  follows.

**Every observed fact owes a typed projection and a deterministic encoding.** A
fingerprint over an unspecified projection is not a comparison — attestation-time
and evaluation-time code can hash semantically identical state differently, or
omit a row one of them considered irrelevant, and the condition then expires or
survives for no reason anybody can name. So each member of `ObservedFact` must
define which rows it includes and how they are canonically ordered and encoded
before hashing, and that definition is part of the member, not of the call site.
`GovernanceEdges`' projection is specified in the attested-acts section alongside
the artefact it is compared against; this section fixes that the obligation
exists and applies to every future member.

### Reach: how far a guard is re-derived

```rust
pub(crate) enum Reach {
    /// Re-derived at this edge and every edge above it (DEC-067).
    Cumulative,
    /// Evaluated only on the edge that names it.
    EdgeLocal,
}
```

`cumulative_conditions` accumulates the `Cumulative` rows from every edge below
the target and the `EdgeLocal` rows from the final window only — the edge being
crossed.

**Reach and coverage are independent, and conflating them is a live error.**
Reach says *when* a guard is re-derived; coverage says *what can make it fail*.
A row with `Cumulative` reach and `Artefact` coverage is re-derived on every
forward move and can still only fail if its own artefact changes — which is
coherent, and much weaker than "cumulative" sounds. An earlier draft of the
per-row arguments below claimed three rows were invalidated by state their
coverage did not see, and every one of those claims was false. Each argument now
names only what its coverage can observe.

**Reach is observable only below the top edge.** `cumulative_conditions(to)`
walks the edges beneath the target, so for a row sitting on the *last* window —
`reviewing → locked` — cumulative and edge-local coincide: there is no higher
edge at which the distinction could show. Those rows are marked `Cumulative`
because that is what they mean, not because a test at that edge could tell.
The distinction is observable exactly where it was argued: `drafting → reviewing`
rows, evaluated while crossing `reviewing → locked`.

### What warns instead of guarding

`DEC-126` makes integrated-pass currency and an outstanding-findings summary by
severity **derived state that warns rather than blocks**. Neither is a condition,
and the count stays at nine: a guard bars a move, so a fact that must never bar
one does not belong in the vocabulary.

**Why currency must not block, precisely.** Not repair cost — a whole-document
re-review is finite, and expense alone has never made a guard optional. The
reason is convergence. Dispose a finding, integrate it, and the sections move, so
whole-map currency invalidates the very pass that found it. A guard on that
demands a fixpoint the process cannot reach, which is `RFC-026` E3's termination
result and `DEC-126`'s stated reason for rejecting it: *"that enforces
review-to-fixpoint, a termination rule `RFC-026` E3 provably refutes."* Review
terminates when the user declines another round, so staleness must inform that
decision rather than bar it. The general rule, so the next such fact is not
re-argued from scratch: **a derived fact whose own repair invalidates it cannot
be a guard.**

**The mechanism is a lamp on the turn envelope.** `ReviewStanding::integrated_current`
already computes *"an integrated adversarial pass covers current content"*, so
the fact needs no new derivation — only a channel. It renders as a scalar flag on
`TurnEnvelope`, following `cursor_stale`, which is the same shape already in the
tree: a derived staleness marker rendered inline only when true
(`render/envelope.rs:983`). Three properties follow, and they are the reason this
is a lamp rather than a section of its own:

- **no passive cost** — nothing renders while the pass is current, unlike
  `frontier` and `blockers`, whose headings render unconditionally;
- **not evictable** — the byte-budget ladder in `evict_one` holds lists, not
  scalars, so a warning cannot be dropped in favour of the material it is warning
  about;
- **no rung to choose** — and therefore no repeat of the partial-overflow bug
  that doc comment records against excluding the active path.

The severity summary rides the same channel for a different reason: it is a
count by severity, with no satisfied/unsatisfied reading at all, so it could not
have been a guard whatever its force.

### Activation: the model this slice can turn on

`Activation` exists for one job: stop a row that **cannot be satisfied** from
barring the edge forever.

```rust
pub(crate) enum Activation {
    Active,
    /// Specified now, enforced when the named work lands.
    Pending { blocked_on: &'static str },
}
```

Two rows qualify, and both were stated as interim rather than discovered:
`governing-context-recorded` and `initial-concerns-recorded` are `Pending` on
`IMP-391`, because until the checkpoints exist there is no act to attest and the
rows would bar `exploring → inquiring` permanently.

`review-disposition-attested` does **not** qualify, and an earlier draft's
"partly pending" cell was wrong. `DEC-125` gives it
`Conducted { review } | Waived { reason }` and only `Conducted` is unbuildable
against the runtime `Finding` model; `Waived` works, so the condition is
satisfiable, so it never bars the edge and needs no activation exception. It is
`active`, and the missing arm is what it actually is — an unbuilt enum variant
tracked by `IMP-392`. The workflow consequence is stated rather than hidden:
until `IMP-392` lands, `reviewing → locked` is crossable by *waiving* review with
an admission-checked reason, and the refusal text says so.

Activation is a column on the one classification table rather than a second table
to drift against the first, and the enforced set is that table filtered:

- `boundary_conditions` returns the **unfiltered** target row set for the edge —
  still `const`, still total. Its parameter changes from `(Stage, Stage)` to the
  closed `Advance` type introduced below; what it returns does not. It is what
  the `DEC-124` stage-entry renderer reads, which is how `Pending` rows are shown
  to the agent marked not-yet-enforced — so that renderer resolves
  `Advance::between` first, as `advance` itself will.
- `cumulative_conditions(to)` — already non-`const` and already `Vec`-returning —
  applies both filters, reach and activation, as it accumulates.

Neither filter costs a `const`: a `const fn` matching on `Advance` is still one.
`boundary_conditions` has exactly one caller today (`gate.rs:223`, inside
`cumulative_conditions`), and it is the enforcement path where the filter
belongs, so the two sets never had to share a name.

### The classification

`DEC-126`, restated only as far as this section's vocabulary needs. Ten in, nine
out: **two Derived, seven Attested, zero Claimed.**

| condition | derivation | coverage | observed | reach | activation |
|---|---|---|---|---|---|
| `blocking-inquiries-dispositioned` | `Engine(Dispositions)` | — | — | cumulative | active |
| `materialisation-current` | `Engine(Materialisation)` | — | — | cumulative | active |
| `governing-context-recorded` | `Attested([{GovernanceConfirmed, User}])` | artefact | `[GovernanceEdges]` | cumulative | pending `IMP-391` |
| `initial-concerns-recorded` | `Attested([{GraphReviewed, User}, {BlockingSetDeclared, Agent}])` | inquiry map | — | cumulative | pending `IMP-391` |
| `user-accepts-sufficiency` | `Attested([{SufficiencyAccepted, User}])` | inquiry map | — | cumulative | active |
| `drafting-readiness-attested` | `Attested([{DraftingReady, Agent}])` | artefact | — | edge-local | active |
| `section-attestations-current` | `Attested([{SectionReviewed, …}])` | per section | — | cumulative | active |
| `review-disposition-attested` | `Attested([{ReviewDisposed, User}])` | artefact | — | cumulative | active |
| `user-acceptance-attested` | `Attested([{DesignAccepted, User}])` | every section | — | cumulative | active |

**Retired:** `required-sections-exist` — no implementation to extend, and a
mandatory section list is craft under `DEC-102`.

**Added:** `drafting-readiness-attested`, which `DEC-126` names explicitly as the
replacement — *"the same shape as `user-accepts-sufficiency` one stage later.
Retiring outright would leave `drafting → reviewing` guarded by
`materialisation-current` alone, which is trivially true of an empty document."*
It is a vocabulary addition, not bookkeeping, and is recorded as one.

**Folded:** `integrated-review-present` and `blocking-findings-disposed` become
`review-disposition-attested`, per `DEC-126`. The pass's *currency* is not folded
and is not a row — it warns, per the subsection above.

**Corrected:** `user-acceptance-attested`'s coverage is `EverySection`, not
`Artefact`. `LockAcceptance` already carries `ContentCoverage`, whose
`is_current` compares the covered map against every current section
(`attestation.rs:212`), and `DEC-120` names acceptance-covers-current-content as
the incumbent template. Self-binding would have let a design edit leave the
acceptance apparently current, deleting a guarantee that already ships.

`section-attestations-current`'s required actor is `RequiredActor::RunPolicy`
rather than a constant. `ISS-310` is decided in the attested-acts section: the
required lanes are the run's, declared under `DEC-073`. Naming a fixed actor here
would be the buried assumption this section is supposed to refuse — and would
delete a capability `DEC-073` already grants.

### Why each row reaches as far as it does

Reach is a per-row commitment, so each owes an argument — and the argument may
name only what the row's coverage can observe.

- `blocking-inquiries-dispositioned`, `materialisation-current` —
  **cumulative.** Engine rows: recomputed from the map and from the watermark
  against current digests on every evaluation, so they are live by construction.
- `governing-context-recorded` — **cumulative.** The observed edge set is
  refreshed each evaluation, so governance changing after the crossing unmakes
  the confirmation. This is the row's `observed` conjunct doing the work; its
  artefact coverage holds the dismissals and search evidence.
- `initial-concerns-recorded`, `user-accepts-sufficiency` — **cumulative**, and
  this is why both carry `InquiryMap` coverage. A re-seeded or materially changed
  graph moves the map, so a review or an acceptance made over the old one is no
  longer current. With `Artefact` coverage the cumulative label would have been
  inert, which is precisely the error the previous draft made.
- `drafting-readiness-attested` — **edge-local.** It is a judgement that drafting
  may *begin*; once drafting has happened, re-asserting it at a later edge asks a
  question with no meaning, and the content drift it might be imagined to catch
  is `materialisation-current`'s job, which is cumulative. Its coverage is
  `Artefact`, so cumulative reach would add nothing observable in any case —
  edge-local is the honest label, not merely the chosen one.
- `user-acceptance-attested` — **cumulative**, on `EverySection` coverage:
  `DEC-066` over section digests against the map the acceptance was given over,
  which is the whole point of the row.
- `section-attestations-current` — **cumulative**, on `PerSection` coverage. The
  same `DEC-066` reading one quantifier out: a section whose digest moves loses
  its own act, and the row is unmet until that section is reviewed again. Editing
  one section does not unmake the others' reviews, which is the difference from
  the row above.
- `review-disposition-attested` — **cumulative**, and today the label outruns
  what its coverage can enforce. `Artefact` coverage means only the disposition
  record's own content invalidates it; binding it to the finding set — so that
  new findings unmake a prior disposition — needs the `RV`-backed model
  `DEC-125` specifies and `IMP-392` builds. Recorded as a gap rather than
  asserted as a guarantee. Note also that this row sits on the last window, so
  the reach label is not observable at its own edge either way.

### The contract table

```rust
pub(crate) struct Contract {
    pub(crate) derivation: DerivationRule,
    pub(crate) reach: Reach,
    pub(crate) activation: Activation,
    /// Key of the narrative prose asset — the condition's existing kebab token.
    pub(crate) prose: &'static str,
}

pub(crate) const CONTRACTS: [(Condition, Contract); N];
```

`DEC-123` requires set equality over **three** enumerations: the `Condition`
vocabulary, the const rows, and the prose asset keys. An earlier draft proposed
proving that equality with three assertions. It does not work, and the failure is
specific: a walk over `Condition::ALL` plus a count assertion proves facts about
`ALL`, not about the enum. Add a variant, give it an `as_str` arm, leave it out of
`ALL`, `CONTRACTS` and the corpus, and every assertion still passes while the
variant has no boundary, no contract and no prose.

So these enumerations are not written twice and tested for agreement. **One
declarative macro takes the condition list once — grouped by the edge each row
guards, carrying variant, kebab token and contract — and emits the `Condition`
enum, `Condition::ALL`, `CONTRACTS`, and `boundary_conditions`' match arms.**

Grouping the source by edge is what closes the last hole rather than moving it.
Generating the vocabulary and the contracts alone would still leave
`boundary_conditions` hand-written, so a generated condition could hold a
contract and prose and sit on no boundary at all — never evaluated, never
rendered, and invisible to a set-equality test over the three sets that *were*
generated. With the edge as the source's outer key, a condition that guards
nothing cannot be written down.

**Keying by an edge is not the same as keying by a *lawful* edge.** The source's
outer key is a `(Stage, Stage)` pair — a type with twenty-five values of which
four are lawful. A row keyed `(Exploring, Locked)` therefore compiles, joins every
generated set, and is never evaluated, because `advance` refuses that pair before
any condition is read. A condition guarding a transition that does not exist is
the same defect as one guarding no transition, and is invisible to exactly the
same tests.

The fix is to stop using a type that can say it. The four lawful forward
transitions get a closed type:

```rust
/// The design run's four guarded forward transitions.
pub(crate) enum Advance {
    ExploringInquiring, InquiringDrafting, DraftingReviewing, ReviewingLocked,
}

impl Advance {
    /// The forward graph, written once.
    pub(crate) const fn between(from: Stage, to: Stage) -> Option<Self> { /* … */ }
}
```

`boundary_conditions` takes an `Advance`, and the macro emits it as a match over
that enum. Both directions then close by construction, with no assertion and no
macro arithmetic:

- **a condition cannot name an unlawful transition** — there is no such value to
  name;
- **a lawful transition cannot go unguarded** — the generated match is
  non-exhaustive if the source omits one, which is a build failure.

`can_advance(from, to)` becomes `Advance::between(from, to).is_some()`, so the
graph has exactly one home, and `advance` resolves the transition once rather
than testing a pair and re-matching it downstream.

Two alternatives were weighed. **Generating `can_advance` from the condition
source** closes the same hole, but makes that source authoritative for the state
machine: appending a row would silently add a transition, putting the graph in a
table that reads as a condition list — the weakest available home for it. **A
compile-time legality assertion** keeps two enumerations and binds them by test,
and needs two assertions rather than the obvious one, since without *every lawful
transition has a row* you can add a transition and forget to guard it. The closed
type needs neither, and is the move this design already makes for `Claimed` and
for `AgentActKind`: render the illegal value unrepresentable rather than rejected.

**`Advance` is the forward relation only, and the asymmetry is deliberate.** The
run has two transition relations and they differ in kind. The forward one is a
closed set of four, each carrying a guard contract. The backward one covers *"the
backward and non-adjacent pairs"* (`gate.rs:199`) — every later-to-earlier pair,
not merely adjacent ones — and is barred not by a condition but by a missing
reason (`DEC-067`, `gate.rs:367`). Nothing is closed to enumerate on that side, so
naming a counterpart type would assert a symmetry that does not exist.

Three consequences follow, stated because a reader will otherwise assume their
opposites.

**The condition vocabulary governs the forward relation only.** A backward move's
requirement is a *reason*, authored in the payload rather than derived from state.
Keeping it out of the vocabulary is what keeps `satisfied` free of payload
inspection — the same derived-versus-claimed line
`Refusal::DerivedConditionClaimed` already polices.

**A backward move invalidates nothing, and needs no mechanism to do so.** Every
clearance is derived and never stored (`DEC-066`, `DEC-067`), and every act binds
to *content* — section fingerprints, node material, artefact digests, observed
facts — never to a stage. There is no cleanup on retreat because nothing stored
would need cleaning.

**Re-crossing re-earns the transition, with no special case.** A run that goes
`reviewing → drafting → reviewing` re-derives every cumulative condition on the
way back up. Sections edited during the excursion have invalidated their own
attestations by fingerprint; untouched ones are still current. The excursion is
handled nowhere, which is the point.

That leaves exactly one enumeration outside Rust, and one test:

- **generated** — `Condition`, `ALL`, `CONTRACTS` and `boundary_conditions`, from
  one list keyed by `Advance`. Equal by construction; no test can fail because no
  disagreement is expressible.
- **tested** — the prose asset corpus: every generated key has an asset, and the
  corpus has no key the vocabulary does not.

This is the same move `STD-001` asks for, and the reason `CONTRACTS` is an
enumerable array rather than only a `const fn` match: the array is what the
generator emits and what the asset test iterates.

### Invariants

1. **Totality is generated, not tested, except at the Rust boundary.**
   Vocabulary, const rows and boundary membership come from one source and cannot
   disagree. The prose corpus is set-equal by test. A missing match arm fails the
   build.
2. **Every condition guards a lawful *forward* transition, and every one of them
   is guarded.** Unrepresentable otherwise: the source is keyed by `Advance`,
   whose four values are the forward graph, so no row can name a fifth — and a
   value the source omits makes the generated match non-exhaustive.
3. **No Claimed tier.** `ConditionKind` has two variants and is a projection of
   `DerivationRule`, so the defect tier is unrepresentable rather than merely
   unused.
4. **One discharge source.** The discharging act is stated once, in the
   derivation rule. Refusal remedy and rendered prose are both injected from it.
5. **Derivation is uniform.** `satisfied` has no branch on kind, and takes the
   whole derived input rather than a parameter per fact class.
6. **A guard bars a move.** Every condition is blocking; a derived fact that must
   not bar a move is envelope content, not a row.
7. **Reach and coverage are independent.** Reach fixes when a guard is
   re-derived, coverage fixes what can make it fail, and a row's stated argument
   may name only what its coverage observes.
8. **A guard is evaluated on its own edge and, if `Cumulative`, on every edge
   above it.** `EdgeLocal` rows are admitted from the crossing edge only.
9. **Binding is conjunctive.** An attestation covers its own content; every named
   observed fact must also still match; and every conjunct act must be current
   against the same covered state.
10. **Observed facts are never persisted as facts**, and each defines its own
    projection and encoding. They are transient input; what the run stores is the
    fingerprint an attestation was made over, keyed by the fact it is of. The key
    names which projection to recompute; it is not the fact.
11. **`DEC-101` holds.** The closed `Condition` set remains the key; no
    satisfaction is sourced from runbook steps or any other open vocabulary.

### Verification impact

- `satisfied`'s signature change touches every call site. Existing `gate.rs` unit
  tests are the behaviour-preservation proof and stay green where behaviour is
  unchanged.
- The e2e suites encode the claimed path, including tests binding conditions to
  `sec-1`. Those change legitimately; each change is an argued edit, not a
  green-chase.
- **Prose coverage** — every generated condition key has a prose asset and the
  corpus carries no orphan. The vocabulary / `CONTRACTS` / boundary equalities
  need no test: they are generated from one source.
- **Wrong actor does not satisfy** — an attestation by the wrong `ActorClass`
  leaves the condition unmet. The property the existential scan could not express.
- **Missing conjunct does not satisfy** — `initial-concerns-recorded` with the
  user's graph review but no agent blocking-set declaration is unmet, *and the
  refusal names which act is missing*.
- **A stale conjunct does not satisfy** — a graph review made over a superseded
  inquiry map, paired with a current blocking-set declaration, is unmet. The
  acts-from-different-eras case.
- **Coverage invalidates** — a node edit unmakes `user-accepts-sufficiency`; a
  section edit unmakes `user-acceptance-attested`. Both against the incumbent
  `ContentCoverage` behaviour, at its two instantiations.
- **Per-section coverage invalidates only its own subject** — editing one
  section unmakes `section-attestations-current` until that section is reviewed
  again, and leaves every other section's attestation live. The quantified
  reading, distinguished from the whole-map one on the same edit.
- **A departing section is not a failure** — removing a section leaves
  `section-attestations-current` satisfied on the remaining reviews, while the
  same removal unmakes `user-acceptance-attested`. The one case where the two
  coverages give opposite answers, which is why they are two variants.
- **An empty document does not satisfy per-section review** — a run holding no
  sections leaves `section-attestations-current` unmet. The incumbent's
  degenerate guard; whole-map equality would report it satisfied, so this test
  is what stops the split from being cosmetic.
- **Progress does not invalidate** — disposing an inquiry leaves
  `user-accepts-sufficiency` satisfied, where re-wording or re-parenting the same
  node unmakes it. The material/progress line, tested from both sides.
- **An unlawful transition is not writable, and a lawful one cannot be
  unguarded** — `Advance` has four values, so no source row can name a fifth, and
  omitting one makes the generated match non-exhaustive. Both are build failures,
  so there is no runtime test to write.
- **A backward move clears nothing and breaks nothing** — retreating from
  `reviewing` to `drafting` leaves every recorded act current, and re-advancing
  without edits succeeds on the same evidence.
- **A backward excursion re-earns only what moved** — editing one section during
  the excursion leaves that section's attestation unmet on the way back up, and
  every untouched section's still current.
- **Stale observed fact invalidates** — the governance edge set moving after the
  attestation leaves the condition unmet; an unobservable fact reads as changed.
- **An empty observed set still binds its artefact** — a governance sweep that
  found nothing, whose search evidence is then edited, is unmet. The strict-path
  case, and the reason `Binding` is conjunctive.
- **Edge-local is not accumulated** — `drafting-readiness-attested` is required
  crossing `drafting → reviewing` and absent from the enforced set crossing
  `reviewing → locked`.
- **`Pending` rows are not enforced** — and are still returned by
  `boundary_conditions` for the renderer.
- **The currency lamp is rendered and never refuses** — a stale integrated pass
  sets the envelope flag and `reviewing → locked` still succeeds. Tested at the
  render surface, not the gate, because it is not a condition.

### Carried forward

- `ActKind`'s full membership is fixed by the sections specifying each act. This
  section fixes that it is closed and that `ActRequirement` pairs it with an actor.
- **Conjunct ordering** — that a confirming act's artefact covers the declaring
  act's digest is an artefact-shape requirement, specified in the attested-acts
  section. Coverage gives simultaneity; only the artefact can give order.
- **`GovernanceEdges`' projection and encoding** — which relation rows, in what
  canonical order, under what encoding. Specified with the artefact it is
  compared against, in the attested-acts section.
- `ISS-310` is decided in the attested-acts section: the required lanes come from
  `DEC-073`'s per-run review policy, which this slice builds. The repair reaches
  `review_standing` and the envelope's `review_outstanding`, and deliberately not
  `live_reviews` — that section says why.
- **`review-disposition-attested`'s cumulative reach is not yet enforceable.**
  Binding it to the finding set requires `DEC-125`'s `RV`-backed model
  (`IMP-392`); until then its `Artefact` coverage means only its own record
  invalidates it.
- `review-disposition-attested`'s `Conducted { review }` arm awaits `IMP-392`.
  It is an unbuilt variant, not a pending row.
- Whether `ObservedFact` grows beyond `GovernanceEdges` is left open. One member
  is enough to justify the seam — the alternative is a special case in `satisfied`
  for exactly one condition — but a second member would test whether the
  refresh/compare/absence semantics generalise. **And the one member has no
  enforced consumer in this slice**: `GovernanceEdges` is named only by
  `governing-context-recorded`, which is `Pending` on `IMP-391`, so it is
  filtered out of the enforced set and nothing compares it until that work
  lands. The seam is specified alongside the act it serves, which is the right
  place to specify it; but the argument above is weaker than it reads, because
  the alternative this slice actually faces is not one special case — it is
  none.
- Research currency is the third instance of the reach question and is **not** in
  this vocabulary. It is a warning-shaped fact by the same convergence argument,
  it lives outside the design run today, and settling it is outside `SL-244`.

<!-- doctrine:section sec-4 -->
## The attested acts

`sec-3` fixed that `ActKind` is closed and that every requirement pairs an act
with a required actor. This section fixes the membership, each act's artefact,
and the three things `sec-3` deferred here: the actor for
`section-attestations-current`, conjunct ordering, and `GovernanceEdges`'
projection.

The result is **less ratification than it first looks**. Two of the eight acts
have a home a gate can read, four have one it cannot, and two have none at all.
One comparison covers three of the four coverages, but only after it is
generalised over what it covers; the fourth keeps the incumbent quantification
rather than joining it — and the eight acts settle into three record shapes.

### One contract, as few shapes as the acts need

The gate needs one question answered — *is there a recorded act of this kind, by
this actor, still current against this binding?* — and that is a question about a
**view**, not about a struct. `ActKind` and `ActorClass` unify the contract.

An earlier draft added *"where each act is stored stays as it is"*, on `ADR-010`'s
shape: unify the contract and the write seam, keep storage bespoke. That reading
does not survive the membership below. Six of the eight acts have no home a gate
can read, so this section adds storage — and once it does, *keep what ships*
stops being conservatism and becomes an argument for carrying more shapes than
the acts need. The rule has to be narrower, and it cuts both ways:

> Add a shape only where an act has no readable home; keep an incumbent shape
> only where it says something a general one cannot.

`Attestation` (`attestation.rs:36-41`) passes that test. It carries no
acceptance, it holds a `reviewer` lane, and there is one per *section* rather
than one per act. It is a different kind of record, not a special case of a
general one.

`LockAcceptance` (`attestation.rs:260`) fails it. It is `AcceptanceAttestation`
plus `ContentCoverage` — which is exactly `CheckpointAct` (below) with
`act: DesignAccepted`, `covered: Sections(…)`, `observed: {}` and
`confirms: None`. It is *subsumed*, not merely similar, so keeping it beside
`CheckpointAct` would be duplication this design introduced rather than
duplication it inherited. `DesignAccepted` becomes a `CheckpointAct` and
`LockAcceptance` retires into it.

That leaves **three** record shapes for eight acts, where an earlier draft
carried four: `CheckpointAct` for the five user acts given at a checkpoint,
`AgentDeclaration` for the two agent acts, and `Attestation` for the one act that
is per-section. `ADR-010`'s shape still holds; what changes is that *bespoke* now
means justified rather than incumbent.

### The acts, and where each lives

| act | actor | stored as | binding |
|---|---|---|---|
| `GovernanceConfirmed` | User | **new** — checkpoint act | artefact + `[GovernanceEdges]` |
| `GraphReviewed` | User | **new** — checkpoint act | inquiry-map coverage |
| `BlockingSetDeclared` | Agent | **new** — agent declaration | inquiry-map coverage |
| `SufficiencyAccepted` | User | **new** — checkpoint act | inquiry-map coverage |
| `DraftingReady` | Agent | **new** — agent declaration | artefact |
| `SectionReviewed` | `RequiredActor::RunPolicy` | `att-` attestation | per-section coverage |
| `ReviewDisposed` | User | **new** — checkpoint act | artefact |
| `DesignAccepted` | User | **new** — checkpoint act (run-level) | every-section coverage |

One incumbent home survives: **`Attestation`** (`attestation.rs:36-41`) — `id`,
`subject`, `fingerprint`, `reviewer`, declared with an `att-` subject and
content-bound to one section, which is exactly `SectionReviewed`. The subsection
above says why `LockAcceptance` does not survive beside `CheckpointAct`.

### Why the checkpoint acceptance is not a home

An earlier draft assigned `GovernanceConfirmed`, `GraphReviewed`,
`SufficiencyAccepted` and `ReviewDisposed` to the generic `cp-` acceptance,
reading its digest doc as covering everything a checkpoint judgement needs. It
does not work, for three reasons that compound.

**The type carries no act identity.** `AcceptanceAttestation`
(`attestation.rs:116-122`) is `authority`, `basis`, `turn`, `digest`. No
`ActKind`, no coverage, no observed fingerprint. Four acceptances of four
different acts are four values of one shape, and nothing in them says which act
each discharges — so a rule naming `GraphReviewed` cannot find its act, let alone
check its binding.

**The digest is opaque and its preimage is discarded.** Its doc says the digest
*"covers the checkpoint payload fingerprint, the inquiry disposition, and the run
revision current when the acceptance was given"*. But the payload it was derived
from is not kept, so at evaluation time there is no material to recompute the
comparison against. A digest whose preimage is gone can detect nothing; it can
only be carried.

**It never reaches the snapshot.** A checkpoint's acceptance rides a
`CheckpointPlan` and is journalled. `ReviewGroup` (`snapshot.rs:322-334`) holds
exactly `attestations`, `findings`, `integrated` and `acceptance: LockAcceptance`.
A gate whose signature is `DesignSnapshot` plus `DerivedInput` cannot see a
checkpoint acceptance at all.

So they get a persisted, snapshot-admitted record — the four above, and
`DesignAccepted` joining them per the subsection on record shapes:

```rust
/// A user act, in the form the gate reads it (DEC-121). Named for the four
/// checkpoint acts; `DesignAccepted` is the run-level fifth.
pub(crate) struct CheckpointAct {
    id: DesignId,
    act: ActKind,
    /// The acceptance itself. Embedded, not widened — its constructor still
    /// sets `AcceptanceAuthority::User`, so DEC-088's guarantee is carried.
    acceptance: AcceptanceAttestation,
    /// What it was given over. `None` is `Coverage::Artefact`.
    covered: Option<CoveredSet>,
    /// Each observed fact as it stood when the act was given.
    observed: BTreeMap<ObservedFact, Fingerprint>,
    /// The agent declaration this act confirms, where its rule names one.
    confirms: Option<Fingerprint>,
}

/// A covered map in the shape the act's `Coverage` selector names.
pub(crate) enum CoveredSet {
    Sections(ContentCoverage<Fingerprint>),
    Nodes(ContentCoverage<NodeMaterial>),
}
```

`CoveredSet` is a sum rather than one map type because `sec-3` generalised
`ContentCoverage` over what it covers, and a *persisted* coverage must name which
instantiation it is. The alternative — one map keyed by a stringly-typed value —
would put the selector back in the data where the type can no longer check it.

**Two variants against `Coverage`'s four, and the gap is not an omission.**
`Artefact` is `covered: None`, as the field's own doc says. `PerSection` has no
`CoveredSet` at all and cannot: it is the one coverage carried by no act, being
a quantification the derivation performs over the section set, and its only row
stores its acts as per-section `Attestation`s rather than as `CheckpointAct`s.
So the two variants cover every case a `CheckpointAct` can lawfully be in, and
a `CheckpointAct` whose rule named `PerSection` is a value admission refuses —
one more instance of the pairing check below.

**The rule's slots and the record's slots are not two spellings of one fact**,
and the apparent duplication resolves rather than needing removal. The rule is a
*requirement* — what an act of this kind must have been given over. The record is
*evidence* — what this act was actually given over. Two things that must agree,
not one thing written twice.

The pairing is not one slot but a **correspondence between two structures**, and
stating it per slot would leave the next slot to be found the way this one was:

| rule slot | record slot | what agreement means |
|---|---|---|
| `Binding::coverage` | `CheckpointAct::covered` | the `CoveredSet` variant is the one the `Coverage` names, and `Artefact` pairs with `None` |
| `Binding::observed` | `CheckpointAct::observed` | the map's key set is exactly the rule's `ObservedFact` list — no missing fact, no extra one |

What was missing is who checks the correspondence. **Admission does**, over the
record as a whole rather than slot by slot: a `CheckpointAct` that does not
correspond to its rule is refused on write. Every stored act therefore satisfies
the invariant by construction, and the gate never re-checks it — the same move
`sec-3` makes for the generated tables, one layer out.

Two consequences worth naming. An act whose `observed` map is simply **absent**
where its rule names a fact is refused rather than treated as an empty
observation, so the strict-path argument `sec-3` makes for conjunctive binding
cannot be evaded by omitting the field. And because the rule is `&'static` and
the check is total, a *new* `ObservedFact` on an existing rule invalidates every
stored act of that kind at admission-time semantics rather than silently
admitting acts that never observed it — which is the correct reading, since such
an act was given over less than the rule now requires.

**Wire, replacement, and snapshot home.** Left unstated, these are where the
implementations diverge, so:

- **Declared** as a run-level field on `ApplyRequest`, joining `WRITER_ACTS`
  (`submission.rs:664`) — the array whose own doc says a new *field* that can
  change state and did not join the list **is** the gap. This slice adds three
  such fields: this one, the agent declaration, and the review policy. So the
  array goes from `[WriterAct; 7]` to `[WriterAct; 10]`, each new row pairing a
  `WRITER_ACT_*` key constant (`STD-001`) with the predicate that detects it —
  the pairing being what that doc calls authoritative rather than merely
  parallel. It carries an `AcceptanceDeclaration`,
  so `AcceptanceAttestation::bind` stays the only route to user authority and
  `DEC-088` is untouched.
- **Replaced by act**, not by id: at most one live `CheckpointAct` per `ActKind`,
  a new one superseding the prior, the way `run.rs:1062` retains-then-pushes an
  attestation. Two live acts of one kind would make *which one is current*
  ambiguous with no rule to break the tie.
- **Stored** in its own snapshot group, for the reason `delegation` and `runbook`
  have theirs: a recorded act is its own state model, and `ReviewGroup`'s members
  are all about review.

`DesignAccepted` rides the same three, which is what retiring `LockAcceptance`
means concretely: its wire type is already `AcceptanceDeclaration`
(`submission.rs:627`), so the run-level acceptance field keeps its shape and
changes where it lands.

### Coverage is a selector three times and a mechanism once

Three of `sec-3`'s four `Coverage` variants need no new comparison.
`ContentCoverage<T>` holds a `BTreeMap<DesignId, T>` and answers
`is_current(current)` by whole-map equality (`attestation.rs:200-214`), and
run-local ids are one space — `sec-`, `inq-`, `cp-`, `att-`, `fnd-` are all
`DesignId`. So for those three the variant selects *which current map is handed
to the same comparison*:

- `EverySection` — the section digest map, `ContentCoverage<Fingerprint>`. The
  incumbent use, unchanged.
- `InquiryMap` — the node material map, `ContentCoverage<NodeMaterial>`. Same
  comparison, different covered type, for the reason `sec-3` gives: nodes carry
  no fingerprint and cannot be given a trustworthy one, because they are mutated
  by pure code after any shell digest would have been taken.
- `Artefact` — no map. The record's own content is its binding, so there is
  nothing to compare against; represented as an absent coverage rather than an
  empty one, because an empty map is *not* the same claim as no claim.

`PerSection` is the exception and is the reason this heading is not the one an
earlier draft carried. It hands nothing to `is_current`, because the act it
ranges over stores no covered map: `Attestation` is `id`, `subject`,
`fingerprint`, `reviewer`. Its comparison is the quantification `sec-3` quotes
from `review_standing` (`snapshot.rs:419-433`) — for every current section,
some live attestation matches that section's id and digest, over a non-empty
section set. That is an incumbent mechanism being *kept*, not a new one being
added, so the count of comparisons in the tree does not change either way.

`ContentCoverage`'s own doc argues for the reuse across the other three: *"One
type, two users — the integrated review and the lock acceptance — so whole-run
currency has a single owner rather than a spelling in each."* A third user rides
the seam as its author intended. Generifying is what lets it; `is_current`'s
body does not change.

**On size.** `ContentCoverage<NodeMaterial>` stores material rather than digests,
and two acts cover the inquiry map, so a run holds roughly two copies of its
question set. At fifteen nodes that is negligible, and it is gitignored runtime
state rather than authored content. The trade is deliberate: material is what
makes the comparison pure and the refusal specific, and a digest would buy
compactness at the cost of both.

### The other new shape: agent declarations

`BlockingSetDeclared` and `DraftingReady` have no home, and `AcceptanceAttestation`
must not become one. Its constructor *sets* `AcceptanceAuthority::User` rather
than accepting it — *"the constructor is the only route to the type, so a value of
it always carries user authority"* — and its single-member authority enum is the
claim being made. Widening that enum to admit an agent would delete the
guarantee, which is `DEC-088`'s and not this slice's to spend.

So the agent acts get their own record, with their own closed vocabulary.

```rust
/// The acts an agent may declare. Closed, and deliberately NOT `ActKind`:
/// an agent-authored `DesignAccepted` is the value that must not exist.
pub(crate) enum AgentActKind {
    BlockingSetDeclared,
    DraftingReady,
}

/// An agent's declaration about the state of its own work (DEC-121).
///
/// Deliberately NOT an `AcceptanceAttestation`: nothing here is accepted truth,
/// and the authority enum's single membership is the point of that type.
pub(crate) struct AgentDeclaration {
    id: DesignId,
    act: AgentActKind,
    /// The stated basis, as an acceptance carries one.
    basis: String,
    /// The harness turn it was declared in, when the caller knew it.
    turn: Option<String>,
    /// What it was declared over. `None` is `Coverage::Artefact` — the
    /// declaration's own content is its binding.
    covered: Option<CoveredSet>,
    /// The digest of this declaration's CLAIM — `act` and `basis`, and nothing
    /// else. Shell-computed at declare time; a confirming `CheckpointAct` names
    /// it. See below for why `covered` is excluded.
    fingerprint: Fingerprint,
}

impl From<AgentActKind> for ActKind { /* widen, never narrow */ }
```

**A closed sub-enum, not a validating constructor.** `act: ActKind` would admit
an agent-authored `DesignAccepted` and rely on a check to refuse it. That is the
shape this section already rejected for `AcceptanceAuthority` and `sec-3`
rejected for `Claimed`: the illegal value should be unrepresentable rather than
rejected. `impl From<AgentActKind> for ActKind` is the one direction needed — a
requirement names an `ActKind`, and the view answers it by widening the
declaration's kind, never by narrowing a requirement's.

It is also **not** "the same triple minus the authority claim": it drops nothing
and adds `id`, `act`, `covered` and `fingerprint`. `turn` is kept for the reason
an acceptance keeps it, and `basis` because a declaration owes its reader one.

**Replacement is by act, not by id.** At most one live declaration per
`AgentActKind`; a new one replaces the prior, the way `run.rs:1062` retains-then-
pushes an attestation. Two live `BlockingSetDeclared`s would make *which one did
the user confirm* ambiguous, and removing that ambiguity is what the confirmation
link below exists for. Its wire shape is a run-level field on `ApplyRequest`,
joining `WRITER_ACTS` as the second of this slice's three new writers.

**What the fingerprint is taken over.** `act` and `basis`, each on its own line,
LF-terminated, UTF-8, sha256 — the same hash the rest of the run uses. `id` is
excluded because the engine allocates it and it is not content; `turn` because it
is a harness detail rather than part of the claim; and `covered` because its
currency is already the coverage mechanism's job. Folding the covered map into
the digest would mean specifying a canonical encoding for `NodeMaterial` *and*
would conflate two questions the conjunct wants answered separately: **is this
the claim the user was shown** (the fingerprint) and **has the material moved
since** (`is_current`). Two checks, each on the mechanism suited to it. Stating
this is not pedantry — leaving it to the implementer is the defect this section
already fixed once, for `InquiryMap`.

`DEC-121` is the reason this exists at all: *"an agent marks its whole graph
blocking by fiat, and the difference between eight round-trips and four is
invisible."* An agent declaration that is recorded, bound, and expirable is what
makes that difference visible.

### `section-attestations-current` takes the run's review policy

`ISS-310`, decided: the required reviewer lanes are **the run's**, read from the
review policy. `sec-3`'s `RequiredActor::RunPolicy` is the const table's way of
saying so.

**This is not a new decision — it is an unbuilt one.** `DEC-073` already settled
it:

> Each run has a small review policy declaring the required reviewer lanes and,
> when both lanes are required, their intended order. This directly supports
> human-only review, adversarial-only review acting as a human proxy, and both
> human and adversarial review in either order.

That is the same shape `DEC-121` found on the exploring edge: an interaction
`SL-233` specified and never built, whose residue is a condition that looks
enforced and is not. `ISS-310` is what this particular absence looks like from
the gate's side — `Reviewer` is recorded on every attestation and read by
nothing, which the tree states about itself at `attestation.rs:77-80`: *"read
surface with no reader — the gate derives review standing from coverage, not from
who reviewed."*

**The shape.** `Reviewer` is closed at two, so the lawful policies are exactly
four values. Enumerate them:

```rust
/// The reviewer lanes a run requires, and the order it intends them in
/// (DEC-073). Membership is what the gate checks; order is DECLARED, not
/// enforced.
#[derive(Default)]
pub(crate) enum ReviewPolicy {
    #[default]
    HumanOnly,
    AdversarialOnly,
    HumanThenAdversarial,
    AdversarialThenHuman,
}
```

An earlier draft used `Vec<Reviewer>`, described as *ordered, non-empty,
duplicate-free* — three properties that type does not have. `Vec` admits `[]` and
`[Human, Human]`, so the prose claimed a guarantee nothing enforced: `ISS-310`'s
own defect shape, reappearing inside its fix. The enum makes all three
structural.

**Two variants differ only in data the gate ignores**, which is intended rather
than overlooked. `HumanThenAdversarial` and `AdversarialThenHuman` present the
same membership to `satisfied`; the difference is read by the renderer and the
runbook, which is where `DEC-073`'s *intended order* is meant to act. Said here
because a reader who finds the gate discarding a distinction the type draws will
otherwise read it as a bug.

`HumanOnly` is the default — `DEC-074`'s posture — and the field is
`#[serde(default)]`, so an existing snapshot parses and an existing run behaves
exactly as it does today.

**The policy is mutable, and that is a deliberate hole with a deliberate fence.**
A run whose policy loosens from `HumanOnly` to `AdversarialOnly` clears
`section-attestations-current` on adversarial attestations alone — `ISS-310`'s
hole, re-entered through the front door. The design accepts it for three reasons.

A user may legitimately change their mind, and a policy nobody can revise is one
they will route around by hand-editing runtime state, which is strictly worse
than revising it in the open. The `SL-243` repair depends on it: that repair is a
lane *swap*, so a tighten-only rule would forbid the very case that argued for
the policy. And a stricter gate would be theatre — the agent drives the CLI, so
any rule expressible in the payload is one an agent can satisfy.

So the fence is authority and visibility, not prohibition:

- **Changing the policy is a user act.** It is a run-level field on
  `ApplyRequest` and the third of this slice's `WRITER_ACTS` additions, and it
  rides an `AcceptanceDeclaration`, so `AcceptanceAttestation::bind` is the only
  route and the change carries `AcceptanceAuthority::User` exactly as an
  acceptance does. An agent cannot
  relax the rules as housekeeping; it must present the change as the user's, in
  the same shape as every other user judgement in the run.
- **Every change lands in the change log**, so the sequence *conditions unmet →
  policy loosened → conditions met* is legible after the fact rather than
  inferable from a final state that looks clean.
- **The current policy renders in the envelope**, so no reader has to guess which
  lanes a run requires.

Stated plainly, because the design should not imply more than it delivers: **the
review policy is a declaration of intent, not a security boundary.** What it buys
is that hurrying a run along must be done in the user's name and leaves a trace,
rather than being a silent shortcut.

**Order is declared and rendered, never enforced.** Three reasons.
`Attestation` (`attestation.rs:36-41`) is `id`, `subject`, `fingerprint`,
`reviewer` — no turn, no sequence, no timestamp — so lane order is not derivable
from what is stored, and enforcing it would mean new state `DEC-073` never asked
for. `DEC-073` says *intended* order and `DEC-074` says *"in either order"*; both
read as the user choosing, not the engine policing. And prompting a sequence is
the runbook's job under the split `DEC-121` drew — the step prompts the
interaction, the condition holds the artefact.

**Where the policy is read, and where it is not.** Three call sites read the
attestation set. Two take the policy; one must not.

| site | the question it answers | takes the policy |
|---|---|---|
| `snapshot.rs:428` `review_standing` | is the condition met? | yes — this is `ISS-310` |
| `envelope.rs:799` `review_outstanding` | does the envelope call this section settled? | yes |
| `run.rs:1495` `live_reviews` | which attestations stopped being live this apply? | **no** |

Repairing only the first would leave a gate that refuses a section the envelope
simultaneously renders as reviewed — the same defect one surface further out. The
third is excluded on a stated ground rather than by oversight: it feeds
`invalidation_rows`, which reports the *death of a recorded act*. An adversarial
attestation going stale is a fact whatever the policy requires, and filtering it
would silently stop reporting it. **A sweep for "readers of `attestations`" gets
this wrong**, which is why the exclusion is written down here rather than left to
the implementer.

**What the integrated pass covers, and what it does not.** `DEC-073`'s last
paragraph: *"The integrated adversarial review required by `DEC-066` remains a
closure-grade authored RV review; section attestations do not silently satisfy or
replace it."* Adversarial coverage of the whole document is separately
guaranteed, by `DEC-066` — which is what makes an adversarial-only *section*
policy a lawful choice rather than a hole. An earlier draft cited `DEC-074` for
this. That was wrong twice over: `DEC-074` is about section posture, and far from
mandating the integrated pass it explicitly grants the adversarial-only run this
policy implements.

`Reviewer` maps into `ActorClass` — `Human → User`, `Adversarial → Adversarial` —
as an `impl From`, not a merge. `sec-3` refused to unify the three actor-ish
vocabularies and that refusal stands; this is the one direction it required.

**Cost, stated.** `SL-243` is the one live run and attested its sections
adversarially. Under the policy it is repaired by declaring its policy
`AdversarialOnly` — a command against runtime state rather than a hand-edit of
it.
The escape hatch the one live run needs turns out to be the one the product
already wanted, which is some evidence the shape is right.

### `ActorClass::Adversarial` is reachable, but no static row names it

With the required lanes coming from run data, `Adversarial` is reachable as a
*resolved* requirement while no `&'static` row names it: the table holds
`RequiredActor::RunPolicy` for the one row whose lane varies, and resolution can
yield `Adversarial`.

Worth stating, because `sec-3` refused an empty `Claimed` tier and someone will
ask whether this is the same case. It is not, and less so than under any other
`ISS-310` answer. `ActorClass` classifies *recorded acts*; adversarial section
reviews are recorded, rendered, read — and now requirable. `Claimed` differed in
kind: it had no legitimate members among the acts themselves, which is why it was
made unrepresentable rather than merely unused.

### Ordering: the confirmation names the declaration

`sec-3` deferred this, having shown that coverage gives simultaneity but not
order: an old `GraphReviewed` and a later `BlockingSetDeclared` could both be
current against the same map while the interaction `DEC-121` specifies — the
agent declares, the user confirms — never happened. It also named the fix, that
*the confirming act's artefact covers the declaring act's digest*. That is an
artefact-shape requirement, and it needs a field.

An earlier draft tried to obtain it for free by **co-location** — recording
`BlockingSetDeclared` into the checkpoint payload that `GraphReviewed` is given
over, so the acceptance digest would cover the declaration by construction. It
does not work, and it fails for the reason that sank the checkpoint acceptance as
a home: the payload is discarded and the digest is opaque, so at evaluation time
there is nothing to recompute the comparison against. Hashing at submission is
insufficient when the comparable material does not survive submission.

So the link is explicit and persisted. `AgentDeclaration` carries its own
`fingerprint`; `CheckpointAct::confirms` holds the fingerprint of the declaration
the user was shown. **The lookup is by act, not by fingerprint** — the conjunct
in the rule already names which agent act it requires, so the gate fetches that
declaration and the fingerprint answers only *is it still the one confirmed*.
`confirms` therefore needs no act tag of its own; it would be naming a fact the
rule supplies.

The ordering property follows. A declaration made or edited after the
confirmation carries a different fingerprint, so `confirms` no longer matches and
the conjunction is unmet. Unlike the co-location scheme this compares two stored
values, so a refusal can name the act, say that a confirmation exists, and say
that the declaration it was given over has since changed — three facts, where the
discarded-payload scheme could only report that a digest did not match.

### The governance edge projection

`ObservedFact::GovernanceEdges` is a fingerprint, and `sec-3` fixed that every
observed fact owes a typed projection and a deterministic encoding or the
comparison means nothing. This is that definition.

**What is included.** The slice's outbound `governed_by` edges **and** its
`references` edges carrying `--role concerns`. That is `DEC-121`'s own
definition — *"The artefact is the resulting **edge set** (`governed_by`,
`references --role concerns`)"* — stated in both its prose and its `choice`
facet. `ADR-004` stores relations outbound-only and derives reciprocity, so both
classes are readable from the slice's own record with no traversal and no second
source that could disagree.

**What an earlier draft excluded, and why the exclusion was wrong.** It admitted
only `governed_by`, arguing that `references` edges are topical and that
admitting them would let an unrelated link added months later expire a governance
confirmation the user gave correctly. The argument is sound against the set it
imagined and irrelevant to the one `DEC-121` names: `--role concerns` is not an
incidental topical link but a deliberate assertion that the target is a concern
of this slice. **The role filter is what does the work the exclusion was reaching
for**, and it does it without contradicting a settled decision. An unrelated
`references` edge in any other role stays out, so the failure mode the draft
feared is closed by the filter rather than by the narrower set.

**Encoding.** Each edge as `kind`, `role`, `target_id` — single-space separated,
`-` where a relation carries no role — one per line, LF-terminated, UTF-8, the
**whole lines** sorted ascending, hashed with the same sha256 the rest of the run
uses for content digests. Sorting whole lines is what makes the fingerprint a
fact about the *set* rather than about insertion order: two runs that added the
same edges in different orders must agree, or the fact expires for no reason.

**Kind and role are load-bearing, and were not before.** The earlier draft
omitted them because a single-class projection made the label a constant in every
line. With two classes admitted that reasoning is false, and omitting them is a
defect rather than a simplification: `governed_by ADR-004` and
`references(concerns) ADR-004` would encode identically, so swapping one for the
other would not move the fingerprint — the projection would be blind to exactly
the kind of change it exists to observe.

**Absence.** A slice record that cannot be read is an unobservable fact, which
`sec-3` fixes reads as **changed**. The gate stays shut rather than opening on a
missing answer.

### Verification impact

- **The policy decides the actor** — a section carrying only an adversarial
  attestation leaves `section-attestations-current` unmet under `HumanOnly` and
  satisfies it under `AdversarialOnly`. The refusal names the missing lane, not
  a fixed one.
- **Gate and envelope agree** — the same section under the same policy is
  reported unsettled by `review_outstanding` exactly when the condition is unmet.
  Asserted against both surfaces, because their disagreeing was the finding.
- **Invalidation is not policy-filtered** — an adversarial attestation going
  stale under `HumanOnly` still emits its invalidation row.
- **Order is not enforced** — a run whose policy is `HumanThenAdversarial`
  clears with the adversarial attestation recorded first. The order is rendered;
  the gate does not read it.
- **Adversarial review is still recorded** — under `HumanOnly` the section
  renders its adversarial attestation, so the act is visible without being
  sufficient.
- **A late declaration does not satisfy** — editing `BlockingSetDeclared` after
  the `GraphReviewed` act leaves `initial-concerns-recorded` unmet, because
  `confirms` names a fingerprint the declaration no longer carries, *and the
  refusal names that declaration*.
- **Node material invalidates; progress does not** — re-wording or re-parenting a
  node unmakes `GraphReviewed` and `user-accepts-sufficiency`; disposing one
  leaves both satisfied.
- **An agent cannot declare a user act** — `AgentActKind` has two members, so an
  agent-authored `DesignAccepted` does not compile. A build-time property, so
  there is no runtime test to write.
- **A mismatched coverage is refused on write** — a `CheckpointAct` whose
  `CoveredSet` is `Sections` where its rule's `Coverage` names `InquiryMap` is
  refused at admission, so no stored act can violate the correspondence.
- **A mismatched observed set is refused on write too** — a `GovernanceConfirmed`
  act carrying no `observed` entry, or one keyed by a fact its rule does not
  name, is refused at the same point. Asserted separately from the coverage case
  because the correspondence is over the whole record, and testing only the slot
  that was found first is what left this one unchecked.
- **A policy change is a user act and is logged** — a payload that changes the
  policy without an `AcceptanceDeclaration` is refused, and an accepted change
  emits a change row naming both the old and new policy.
- **Loosening the policy does clear the gate** — the deliberate hole, asserted
  rather than left implicit: `HumanOnly` unmet, changed to `AdversarialOnly`,
  then met. The test exists so that removing the hole later is a visible break
  rather than a silent tightening.
- **A declaration's coverage does not move its fingerprint** — editing an inquiry
  node leaves `BlockingSetDeclared`'s fingerprint unchanged, so `confirms` still
  matches while `is_current` fails. The two checks answer different questions and
  are asserted separately.
- **The projection is order-independent** — two edge sets with the same members
  added in different orders fingerprint equal.
- **Kind and role are observed** — replacing `governed_by ADR-004` with
  `references(concerns) ADR-004` unmakes `governing-context-recorded`. The case
  a target-only encoding could not see.
- **A concerns edge expires governance; another role does not** — adding
  `references --role concerns` unmakes `governing-context-recorded`; adding a
  `references` edge in any other role leaves it satisfied.
- **`Artefact` is absent coverage, not empty coverage** — a `DraftingReady`
  declaration stays current across section edits, where an empty
  `ContentCoverage` would read as stale the moment any section existed.

### Carried forward

- **`ActKind` is closed at eight.** A ninth act is a decision, not a
  declaration — it would mean a condition acquired a discharging act nobody
  specified.
- **`AgentActKind` is closed at two**, and is the narrower vocabulary. A third
  agent act would mean an agent acquired a discharging role nobody specified.
- **`DEC-073`'s policy is built here, not decided here.** No superseding record
  is owed. What this slice adds is the binding from the policy to
  `section-attestations-current`, and the two derivations that read it.
- **`SL-243` is repaired by policy**, not by hand: declaring `AdversarialOnly`
  clears it.
- **The snapshot changes in four ways, not one** — `ReviewPolicy` on the run
  header, a checkpoint-act group, an agent-declaration group, and
  `LockAcceptance` retiring into `CheckpointAct`. The last is a migration rather
  than an addition, and is the only one that touches data an existing run already
  holds. `sec-2` records one shape change as this slice's cost to the one live
  run; that passage must be revisited before planning.
- **The policy is not a security boundary**, and the design says so. If a later
  slice wants it to be one, the missing piece is not a stricter gate but a way to
  distinguish a user's payload from an agent's — which is `DEC-088`'s standing
  limitation, not this slice's to close.

