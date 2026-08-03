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
conceived independent of that. `DEC-121` is where the design answers it: the
artefact becomes the confirmed governance edge set, so the subject stops being a
prose section at all — which is why that record notes it settles `ISS-286`'s
subject-rule complaint as a side effect. `DEC-126` is where the condition is
reclassified to match.

<!-- doctrine:section sec-2 -->
## Concerns carried into the design

Stated at the outset so later sections argue against a fixed list rather than a
remembered one. Each is a live risk to this design, not a general caution.

- **Snapshot versioning.** A shape change to the design-run snapshot costs the
  live runs there are. The snapshot is gitignored runtime state, so the cost is
  bounded; today it is also *known*, having been read off the one such run rather
  than inferred. `SL-243` holds no section attestations, so the review policy
  costs it nothing and the two new act groups start empty. It holds one value in
  the acceptance slot that retires into `CheckpointAct`, carrying no act to
  migrate as. And it clears `user-accepts-sufficiency` through an evidence row
  this design replaces with an act that cannot be reconstructed from what is
  stored — an active, cumulative row, so it bars the next forward move. It holds
  three declared sections and has never been materialised, so nothing else it
  holds is at stake, and the repair is two acts rather than a migration: the user
  re-gives the acceptance as `SufficiencyAccepted`, and the agent declares
  `DraftingReady` — a condition this design *adds* to the very edge that run is
  standing at, which is why the second act is a cost and not just workflow. The
  concern is retained because the next live run need not be this cheap, not
  because this one is expensive.
- **Envelope byte budget.** The refusal path has no byte budget; the envelope has
  a hard one, and `clearances` already rides it uncapped. Anything this design
  adds to the envelope competes with that.
- **A fourth prose loader.** Three prose systems already have three loaders. A
  fourth needs an argument, not a precedent.
- **`is_derived()` asymmetry.** Six conditions claimed, four derived. This is
  `IMP-361`'s known deferred gap, already stated in `gate.rs`'s own doc comment —
  a carried debt, not a discovery of this slice.
- **`CHR-049` is one moderated run, and an incomplete one.** Four sessions
  reached roughly the halfway point of a single design run. Its yield is on the
  record rather than recalled — the ten backlog items tagged
  `cluster:design-run`, all minted 2026-08-01/02, inside the exercise window of a
  chore opened 2026-07-27 and still open. So the exercise is well evidenced and
  the sample is still one run, stopped halfway. That is exactly enough to inform
  `ISS-285`'s deferred choice and not enough to settle it alone, and the reason
  is the sample rather than the rigour.

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
    /// The agent declaration this act must confirm, where the interaction has
    /// an order. `Some` on exactly one requirement today; `None` means this
    /// act confirms nothing and its record must carry no confirmation.
    pub(crate) confirms: Option<AgentActKind>,
    /// Whether this act disposes of a review, and therefore must carry
    /// `DEC-125`'s two-armed value. True on `ReviewDisposed` alone.
    pub(crate) disposes_review: bool,
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
- **either** — needs the actor slot to admit a *disjunction*, satisfiable by one
  lane or by the other, which the conjunctive `acts` slice deliberately does not.
  This is not what `RunPolicy` does below, and the two should not be read as the
  same move: a run-declared lane set is a conjunction whose arity the run fixes,
  never an alternation the rule leaves open;
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

impl RequiredActor {
    /// The lanes an act must satisfy — ALL of them, never any of them.
    /// `Fixed` is the singleton case; `RunPolicy` yields the policy's
    /// membership, which is one lane or two.
    fn resolve(self, policy: ReviewPolicy) -> &'static [ActorClass] { /* … */ }
}
```

The table stays `&'static` and stays total; resolution moves into `satisfied`,
which already receives the whole snapshot rather than a parameter per fact class,
so the policy arrives with no signature change. What makes this work where the
per-section candidate fails is *what varies*: a per-run policy is one value an
evaluation reads once, where a per-section rule would need the table to hold a
different actor per subject.

**Resolution fixes the conjunction's arity, so the `acts` slice is a template for
it rather than the conjunction itself.** `Fixed` resolves to one lane, so seven
requirements mean exactly what they say. `RunPolicy` resolves to the policy's
membership — one lane under `HumanOnly` or `AdversarialOnly`, two under either
ordered variant — so the single `SectionReviewed` requirement stands for one
required act or for two, and which is not known until the run is read. Said
plainly because a reader counting `&'static` entries will otherwise count the
requirements wrong, and because a refusal owes the missing **lane**, not merely
the missing act.

For `section-attestations-current` that composes with `PerSection` into a nested
quantification: for every current section, for every lane the policy requires,
some live attestation carries that section's id, its current digest, and that
lane's reviewer. The incumbent (`snapshot.rs:419-433`) has the section dimension
and no lane dimension, which is precisely what `ISS-310` is. Keeping the lane
dimension in the actor slot rather than folding it into `PerSection` is the same
independence this section insists on between reach and coverage: coverage fixes
the subject set a requirement ranges over, the actor slot fixes who must have
authored each act, and one row needing both is not a reason to merge two axes.

**Conjuncts must be current together, and currency is not yet ordering.** A
conjunction of independent acts is satisfiable by acts that never met: an old
user `GraphReviewed` and a later agent `BlockingSetDeclared` would both be
*present*, and `DEC-121`'s interaction — the agent declares, the user confirms —
would not have happened. Coverage closes most of this by construction: where the
rule's binding covers a run-owned subject set, every conjunct must be current
against the *same present* state, so a review made over a superseded inquiry map
is not live and the pair cannot be assembled from different eras. What coverage
does **not** give is order — that the user's confirmation came after, and over,
the agent's declaration. So order is split across the two: the **rule** names
which declaration a requirement must confirm, which is what `confirms` is; the
**artefact** carries the digest that answers whether it is still the one
confirmed, which is a statement about record shape and is specified in the
attested-acts section. Putting the requirement in the rule is what makes a
missing confirmation a refusal rather than a silently weaker act — a record may
omit only what its rule leaves `None`.

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
    /// section must have its OWN live act of the required kind, one per
    /// resolved lane, against that section's current digest.
    PerSection,
}
```

**`PerSection` is not `EverySection` with the acts spread out**, and the
incumbent is where the difference is visible: `review_standing`
(`snapshot.rs:419-433`) derives both, nineteen lines apart, in two different
shapes.
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

```rust
/// What an inquiry-map coverage compares, per node. Persisted inside
/// `ContentCoverage<NodeMaterial>`, so it is stored and compared by `Eq` — and
/// carries no digest of its own, which is the whole point of the variant.
pub(crate) struct NodeMaterial {
    question: String,
    provenance: Provenance,
    parent: Option<DesignId>,
    needs: BTreeSet<DesignId>,
    seq: u64,
}
```

`id` is absent because it is the covered map's key rather than part of what a
node is compared at. Nothing here is new state: every field is `InquiryNode`'s
own, and each already derives what a stored comparison needs.

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

**Both warnings derive over the run's current review pass**, which is a record
this design has to add rather than a derivation it can borrow. `DEC-125` mints an
`RV` on entry to `reviewing` — *"the review is a first-class artefact from the
start"* — so a pass exists from that moment, before any disposition and whether
or not one ever arrives. Nothing in the run holds it:

```rust
/// The review pass the run is currently on. Minted on entry to `reviewing`
/// (DEC-125), and REPLACED — never reopened — on a later entry.
///
/// Deliberately independent of `review-disposition-attested`: it exists before
/// that row is answered and under BOTH arms of the answer, which is what lets
/// the warnings below derive on a waived run and what gives a disposition
/// something to bind to.
pub(crate) struct ReviewPass {
    /// The `RV` minted for this pass. `sec-4`'s `ReviewRef`.
    pub(crate) review: ReviewRef,
    /// The section digests the pass was opened over.
    pub(crate) covered: ContentCoverage<Fingerprint>,
}
```

`IntegratedReview` (`attestation.rs:224`) is the shape this **replaces**, not one
it rides. That record is `id` plus a `ContentCoverage` — `ReviewPass` with a
`DesignId` where a `ReviewRef` belongs, which is exactly the defect `DEC-125`
names when it says the run *"cannot hold or resolve an `RV`"*. An earlier draft
had the lamp riding `ReviewStanding::integrated_current` unchanged and migrating
later; that deferred the one field that makes the record usable. `sec-4` rules on
the retirement.

The two warnings are then derived, never stored:

- **currency** — `pass.covered.is_current(current_section_digests)`. The same
  comparison `integrated_current` performs today (`snapshot.rs:419-433`), over a
  record that can name an `RV`.
- **outstanding findings by severity** — a count over the pass's findings,
  filtered to the states that are still live:

```rust
/// Outstanding findings on the current pass. A fixed record rather than a map:
/// the ledger's severity vocabulary is closed at four (`review.rs:548-553`), so
/// an absent key and a zero count would be one fact with two spellings.
pub(crate) struct OutstandingBySeverity {
    pub(crate) blocker: u32,
    pub(crate) major: u32,
    pub(crate) minor: u32,
    pub(crate) nit: u32,
}
```

**Outstanding means not terminal — `status ∉ {verified, withdrawn}`** — which is
D-C9b's `doc_unresolved_blockers` filter (`review.rs:1490`) with the severity
restriction lifted. That is deliberately **wider** than `DEC-138`'s gating
predicate, which blocks on `open` and `contested` only. A warning should report
everything still live, including findings sitting answered in the raiser's court;
a gate should bar only on what the responder has not answered. The two filters
differ by the `answered` state, they differ on purpose, and the difference is the
whole of `DEC-138` — so spelling them with one shared filter would silently pick
a side.

Both arrive shell-read on `DerivedInput`, beside `ObservedReview` below and for
the reason that field states: the ledger is an asset outside the run, and a fact
about what an asset says is one Doctrine derives rather than takes on a caller's
word. Currency needs no shell input beyond the pass itself, the section digests
already being there.

It renders as a scalar flag on `TurnEnvelope`, following `cursor_stale`, which is
the same shape already in the tree: a derived staleness marker rendered inline
only when true (`render/envelope.rs:983`). Three properties follow, and they are
the reason this is a lamp rather than a section of its own:

- **no passive cost** — nothing renders while the pass is current, unlike
  `frontier` and `blockers`, whose headings render unconditionally;
- **not evictable** — the byte-budget ladder in `evict_one` holds lists, not
  scalars, so a warning cannot be dropped in favour of the material it is warning
  about;
- **no rung to choose** — and therefore no repeat of the partial-overflow bug
  that doc comment records against excluding the active path.

The severity summary rides the same channel for a different reason: it is a
count by severity, with no satisfied/unsatisfied reading at all, so it could not
have been a guard whatever its force. It renders as one line of four counts,
omitted entirely when every count is zero — the same render-only-when-it-says-
something rule the currency flag follows, so neither warning has a passive cost.

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
a stated reason, and the refusal text says so.

**`Waived` is a permanent arm, not an interim crutch**, and the sentence above
would read as the opposite left on its own. `DEC-125` gives the row two arms
because both are wanted: a review conducted, or a review declined for a stated
reason. The gate's job is to make the choice deliberate and legible, not to
extract a toll — a user who decides this design does not need another
adversarial pass says so, and the run advances carrying the reason. What
`IMP-392` unblocks is the `Conducted` arm's predicate; nothing about the waiver
is provisional. Nor is `Conducted` the *stricter* arm in any general sense — it
is the only one that can be unmet, but it is also satisfiable over an empty
ledger. `DEC-138` below fixes what each arm actually requires.

**A disposition disposes of one pass, and the act has to say which.** An earlier
draft claimed this property from two mechanisms that do not deliver it. Acts
replace by act, so a later `Conducted` displaces a `Waived` — true, and it says
nothing about a pass that arrives *after* the waiver. `DEC-125` mints a second
`RV` on re-entry to `reviewing` — also true, and with `Artefact` coverage and no
recorded pass identity the old waiver is still live, still the run's one
`ReviewDisposed`, and still clearing an edge over a pass nobody has looked at.
The claim was right and unsupported, which is worse than wrong.

So the binding is explicit, on both arms:

```rust
pub(crate) enum ReviewDisposition {
    Conducted { review: ReviewRef },
    Waived { reason: String },
}
```

— and the act carries, beside it, the `ReviewRef` of the pass it disposed. The
row is satisfied only while that reference equals the run's current
`ReviewPass::review`. Three properties fall out, none needing a mechanism of its
own:

- **A foreign `RV` cannot answer the row.** `Conducted` naming a review that is
  not this run's current pass is refused at admission, not merely unmet — the
  same *refuse the false claim on write* move `DEC-138` makes for an unconcluded
  pass. A syntactically valid `ReviewRef` pointing at another slice's ledger was
  otherwise a satisfying answer.
- **Re-entry stales the prior disposition.** Minting a new `ReviewPass` moves
  `current`, so the stored act's reference no longer matches and the row is unmet
  until the user disposes *this* pass. That is what makes waiving non-terminal in
  fact rather than in prose.
- **`Waived` binds too, and that is not symmetry for its own sake.** A waiver
  that recorded no pass identity would carry forward across every future entry to
  `reviewing`, which is precisely the hole. Declining *this* pass is the claim
  being made; declining every pass the run will ever have is not one the user was
  asked for.

The reference is on the act rather than inside `Conducted` because both arms need
it and only one names an `RV` for its own purposes. `sec-4` gives it a slot.

**What the user is bound to is responding, not agreeing** — `DEC-138`, and it
fixes both arms at once.

`Conducted { review }` is satisfied while the minted `RV` carries no finding that
is both `blocker`-severity **and** in `open` or `contested` state. Findings
sitting in the responder's court hold the edge; the raiser's assent is not
required, and that is the sharp half.

**The property is not that contest cannot re-block — it can — but that the
responder always holds an act that clears.** `contest` moves a finding
`answered → contested` (`review.rs:2354`), which is blocking again, and the
responder's re-disposal clears it again. Compare terminal resolution, which
blocks on `{open, answered, contested}`: there, disposing moves `open → answered`
and the edge stays shut, so **no act available to the responder opens it** — only
the raiser's `verify` or `withdraw` does. That is the whole difference between an
obligation to respond and a wait on someone else's assent, and it is one state
wide.

**The predicate is per-finding, and deliberately not the review-level `await`.**
`derived_status` (`review.rs:1009-1022`) reads `status` and never `severity`, so
`await == Responder` fires on an open `nit` — binding to it would hold this edge
for a typo. Its own doc says as much: *"a priority summary … never an exclusive
gate — the turn gate is per-finding `can`."* The filter this row needs is written
by this slice: D-C9b's `doc_unresolved_blockers` (`review.rs:1490`) is
`severity == Blocker && status ∉ {verified, withdrawn}`, which is the same test
minus the state restriction — it counts `answered`, and that one difference is
what this row turns on.

`Waived { reason }` is satisfied **unconditionally with respect to the review** —
before a pass has run, and over live findings. An earlier draft made it
inadmissible while findings were outstanding, on the reasoning that an
unconditioned waiver is a synonym for dismissing them. It is not, and the ledger
is what keeps them different: a waiver leaves every finding standing on the `RV`,
undisposed and legible, beside a stated reason in the change log. *I have read
these and I am proceeding* is a different claim from *these findings do not
exist*.

**The reason itself is checked, and "unconditionally" does not reach it.**
`reason: String` prevents an omitted field and not an empty one, and a blank
waiver is the exact claim this arm is supposed to make impossible: the whole
defensibility of clearing an edge over live blockers is that the record says
*why*. So a `Waived` whose reason is empty or whitespace is refused at admission.
That is not a new rule — the run already holds it one field over, and in the same
words: `Refusal::AcceptanceBasisMissing` refuses an acceptance with no basis
because *"an auditable claim with nothing stated is not one"* (`run.rs:331-333`).
A waiver is an auditable claim with an audience of one, and it earns the same
guard. The distinction to keep is that the check is on the **reason**, never on
the review: no count of findings, no pass state, nothing the arm exists to be
free of.

**Neither half works alone, which is why they are one decision.** Block without
exit and a contesting raiser owns the user's progression, with no termination
proof. Exit without block and waiving and disposing collapse into the same act,
spending the ledger `DEC-125` bought.

**Withdrawal is not the escape, and treating it as one was the error.** `withdraw`
is a real `RV` verb and it stays available, but it means *raised in error*.
Forcing a user who disagrees with a live finding to spell their decision as a
retraction of the whole review is worse than letting them say what they mean —
and it is the shape this design refuses everywhere else. The fence is the one
`sec-4` argues for the review policy: **authority and visibility, not
prohibition.** Both arms are user acts carrying an `AcceptanceDeclaration`, so
`AcceptanceAttestation::bind` is the only route; both land in the change log.

**And no more than that.** It is presentation and trace, not prevention: `--as` is
cooperative role assertion and explicitly not a security boundary (`ADR-007`,
`review.rs:2813`), and an `AcceptanceDeclaration` cannot distinguish a user's
payload from an agent's — `DEC-088`'s standing limitation, which this slice does
not close. So the achievable property, and the one an implementation must keep, is
that **either arm must be taken in the user's name and must leave a change row.**
An agent that wants past this gate can take it; what it cannot do is take it
quietly, or as its own.

**Why this does not reintroduce the fixpoint**, stated because it is the row that
motivated making currency a lamp — and stated carefully, because the obvious
claim here is too strong. Disposal is always available to the responder, so they
are never stuck. But finding state is **not** monotone: `contest` cycles
`answered` back to `contested`, so the block alone bounds nothing. **Termination
comes from the waiver**, which is the second reason that arm cannot carry a
precondition. What the alternatives lack is not a weaker version of this — binding
to the raiser's assent has no exit at all, and binding to **content currency**
diverges outright, because integrating a finding moves the sections the pass
covered (`RFC-026` E3, and `DEC-126`'s stated reason for refusing it).

**`Conducted` is admissible only over a concluded pass**, and that closes the one
hole this row had. `DEC-125` mints the `RV` on *entry* to `reviewing`, and an
empty finding list reads as `(Done, None)` — so without a rule the arm would be
satisfied the moment the stage was entered, naming a review that had not
happened. Nothing *defaults* to `Conducted`: until the user performs
`ReviewDisposed` the act does not exist and the edge is barred. But a user or an
agent reaching for a disposition sees two arms, and `Conducted` reads as the
normal path, so the claim wants refusing rather than merely discouraging.

**The evidence is a marker on the `RV`, not its findings and not its prose.**
Findings cannot carry it: a clean pass that found nothing is structurally
identical to one never run — `findings: []`, `status: Done`, `await: None`
either way — so testing for findings would refuse exactly the review that went
best. The `RV`'s `## Synthesis` section *does* distinguish them, and is refused
on a different ground: the gate would be parsing authored prose to answer a
condition, which is the fourth prose loader `sec-2` names as a live risk, in its
thinnest and most brittle form. So the signal is structured state on the `RV`,
recording that a pass concluded, and it lands with `IMP-392` — which is already
opening that record to add findings a section reference. One migration, not two.

`Waived { reason }` is where an unreviewed run goes, and it costs nothing it was
not already going to pay: declining a pass and never getting to one are both true
things to say, both clear the edge, and both leave the reason on the record. What
the pair now buys is that the *front door* cannot be claimed falsely — which is
the same concern as `DEC-138`'s fence, one step in. There the risk was an agent
taking the exit as its own; here it is an agent taking the normal path without
having walked it.

**Where the `RV` enters, and why it is not an observed fact.** Both halves of
this row read state the run does not own, and this section has so far named
exactly one route for that. It is the wrong one here. `ObservedFact` is a
*currency* mechanism — an act stores the fingerprint it was made over, and the
condition holds while the observed fingerprint still matches — and neither half
of this row compares against a stored value. *Has a pass concluded* and *does
the ledger carry an undisposed blocker* are evaluated fresh every time they are
asked. So this row's `observed` column stays empty, and its input is a second
kind rather than a missing entry in the first.

The route is what `DerivedInput` already is. `apply` takes it as a parameter and
threads it into the handlers that admit declarations (`run.rs:224-230`), so a
check *inside* the admit→persist span reads shell-observed state without the
pure layer touching disk — which is what `DEC-123`'s no-subprocess constraint
requires, and what `runbook` and the verifier results already do on that same
field:

```rust
/// The `RV` this invocation must resolve, because an act names one. Shell-read
/// and never persisted — a field beside `runbook`, for the reason that one
/// states: a fact about what an asset says is one Doctrine derives, never one
/// it takes on a caller's word.
pub(crate) struct ObservedReview {
    /// `sec-4`'s type, declared with the disposition that names one.
    pub(crate) reference: ReviewRef,
    /// Whether the ledger carries `sec-4`'s concluded-pass marker.
    pub(crate) concluded: bool,
    /// Findings that are `blocker`-severity AND in `open` or `contested` —
    /// DEC-138's predicate, deliberately not D-C9b's `doc_unresolved_blockers`.
    /// Carried as the ledger's own `F-n` ids: they identify rows on the `RV`,
    /// not subjects in the run, so they are not `DesignId`s.
    pub(crate) undisposed_blockers: Vec<String>,
}
```

**Which check reads which is the whole of the admission/gate split.** Admission
reads `concluded`, because a `Conducted` arm naming a review that never ran is a
false claim and `DEC-138` refuses it on write rather than leaving it merely
unmet. The gate reads `undisposed_blockers`, because that is a live property to
be re-derived at every crossing rather than a fact about the moment the act was
written. `ActRequirement::disposes_review` is what tells both which acts need
the resolution — the slot is already there for the correspondence check, and
this is the second thing it buys.

**Which `RV` the shell resolves** is worth fixing here, because an implementer
would otherwise have to invent it. There is at most one: acts replace by act, so
a run holds one live `ReviewDisposed`. The shell resolves the review the
*payload* names where the batch carries a disposition, and the one the stored act
names otherwise — which covers the case `run.rs` already allows, a disposition
recorded and a stage taken in one submission, where the incoming act is the one
the crossing must be judged against.

**Absence is refusal, not satisfaction.** An `ObservedReview` the shell could
not produce — an unreadable `RV`, or an act naming none — leaves admission
refusing and the gate unmet. That is this section's absence rule again, and
`RunbookNotDischarged`'s fail-closed empty case one place further out.

**Two gates read this `RV` under different rules, deliberately.** This row gates
the design run's `reviewing → locked` on *undisposed blockers*; the `RV`
close-gate (D-C9b) gates the **slice's** `audit → reconcile` and
`reconcile → done` on every `blocker` being terminal — verified or withdrawn.
Different subjects at different altitudes: advancing a design run is not closing
the work it designs, and closure may reasonably want the stricter thing. Stated
because one rule implemented twice and inconsistently is what this looks like
from a distance.

Severity is the ledger's own `blocker` rather than a notion this design invents;
`DEC-125` brings it, and it is already the only severity that gates anything.
**Only `Conducted` consumes any of this.** It is the arm that needs the
`RV`-backed finding set and the concluded marker, and it is the arm that waits on
`IMP-392`; `Waived` reads no `ObservedReview` at all, needing nothing but a
non-blank reason and the current pass's identity. `DEC-138` says so in as many
words — *"The `Waived` arm needs nothing from it"* — and an earlier draft of this
paragraph claimed the opposite, which would have disabled the deliberately
available exit for exactly as long as the interim lasts. What is the same on both
arms is that neither is a degraded path: `Waived` is permanent, not a stand-in
for `Conducted` until the migration lands. That is the footing `DEC-125` sets:
*"SL-244 specifies its conditions against the RV-backed model; the migration is
its own item."*

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
| `initial-concerns-recorded` | `Attested([{GraphReviewed, User, confirms BlockingSetDeclared}, {BlockingSetDeclared, Agent}])` | inquiry map | — | cumulative | pending `IMP-391` |
| `user-accepts-sufficiency` | `Attested([{SufficiencyAccepted, User}])` | inquiry map | — | cumulative | active |
| `drafting-readiness-attested` | `Attested([{DraftingReady, Agent}])` | artefact | — | edge-local | active |
| `section-attestations-current` | `Attested([{SectionReviewed, RunPolicy}])` | per section | — | cumulative | active |
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
- `review-disposition-attested` — **cumulative**, and the label bites through
  the derivation rather than through the coverage. `Artefact` coverage means
  only the disposition record's own content invalidates it; what keeps the row
  live is that the `Conducted` arm re-reads
  `ObservedReview::undisposed_blockers` on every evaluation, so a blocker raised
  after the disposition unmakes it, with no stored value that could have gone
  stale instead. That is a property of the input route rather than of the
  binding, which is why the two are specified apart. Until `IMP-392` supplies
  the finding set the route has nothing to read, so the guarantee is dated
  rather than absent. Note also that this row sits on the last window, so the
  reach label is not observable at its own edge either way.

### The contract table

```rust
pub(crate) struct Contract {
    pub(crate) derivation: DerivationRule,
    pub(crate) reach: Reach,
    pub(crate) activation: Activation,
    /// Key of the narrative prose asset — the condition's existing kebab token.
    pub(crate) prose: &'static str,
}

pub(crate) const CONTRACTS: [(Condition, Contract); 9];
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
   observed fact must also still match; every conjunct act must be current
   against the same covered state; and where a requirement names a confirmed
   declaration, the act must still name that declaration's current digest.
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
- **An undisposed blocker holds the edge** — a `Conducted { review }` disposition
  naming an `RV` that awaits the responder on a `blocker` leaves
  `review-disposition-attested` unmet, and the refusal names the finding. Needs
  `IMP-392`'s finding set, so it is specified here and lands with that item.
- **A contested finding holds it again, and one disposal clears it again** — the
  responder disposes and the edge clears; the raiser contests and it is unmet;
  the responder re-disposes and it clears. Asserted as the three-step cycle
  rather than as a single edit, because the property is not *contest cannot
  re-block* — it can — but that **the responder always holds an act that
  clears**. Under terminal resolution there is no such act.
- **A waiver clears over live findings, and dismisses none of them** — a
  `Waived { reason }` disposition with blockers undisposed clears the edge, and
  the findings are still on the `RV`, still undisposed, with the reason in the
  change log. Both halves asserted, because the arm is only defensible if the
  second is true.
- **Neither arm is takeable except in the user's name, and neither is silent** —
  a disposition payload of either arm without an `AcceptanceDeclaration` is
  refused, an accepted one carries `AcceptanceAuthority::User`, and it emits a
  change row. Asserted as those three facts and **not** as *an agent cannot
  author it*: `--as` is cooperative role assertion and explicitly not a security
  boundary (`ADR-007`, `review.rs:2813`), and an `AcceptanceDeclaration` cannot
  distinguish a user's payload from an agent's (`DEC-088`). The stronger test
  would assert a property this system does not deliver, which is the defect this
  row exists to describe rather than to commit.
- **A waiver is not terminal** — a run waived to `locked`, regressed to
  `drafting` and re-advanced mints a second `RV`, and a later `Conducted`
  disposition displaces the waiver by act replacement. Both halves asserted: the
  fresh artefact, and the displacement.

### Carried forward

- `ActKind`'s full membership is fixed by the sections specifying each act. This
  section fixes that it is closed, and that `ActRequirement` pairs it with an
  actor and — where the interaction has an order — with the declaration it
  confirms.
- **Conjunct ordering** — the rule half is here, in `ActRequirement::confirms`.
  What the attested-acts section owes is the record half: the digest a
  declaration carries and a confirmation names. Coverage gives simultaneity;
  only that pair can give order.
- **`GovernanceEdges`' projection and encoding** — which relation rows, in what
  canonical order, under what encoding. Specified with the artefact it is
  compared against, in the attested-acts section.
- `ISS-310` is decided in the attested-acts section: the required lanes come from
  `DEC-073`'s per-run review policy, which this slice builds. The repair reaches
  `review_standing` and the envelope's `review_outstanding`, and deliberately not
  `live_reviews` — that section says why.
- **`review-disposition-attested` is owed two things by `IMP-392`**, both
  needing the same missing input — `DEC-125`'s `RV`-backed ledger — so they are
  one gap with two consequences. *`Conducted`'s blocker test:* which findings
  are `blocker` and in `open`/`contested` is unreadable, so
  `ObservedReview::undisposed_blockers` has nothing to fill it from and the
  row's cumulative reach is dated rather than absent. *`Conducted`'s
  admissibility:* the concluded-pass marker does not exist, so until it does the
  arm can name a review that never ran. That marker is the one thing this slice
  asks `IMP-392` to add rather than merely to expose; `sec-4` gives it a shape
  and says why findings and prose cannot supply it. Both are `Conducted`'s.
  `DEC-138`'s waiver arm needs nothing from `IMP-392` — it reads no
  `ObservedReview` at all — so the row stays satisfiable throughout the interim.
  What is the same on both arms is that neither is degraded: `Waived` is a
  permanent arm, not the interim's stand-in for `Conducted`.
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
and the four things `sec-3` deferred here: the actor for
`section-attestations-current`, conjunct ordering, `GovernanceEdges`'
projection, and the shape of the concluded-pass marker.

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

The subsumption is of the **type**, not of whatever the slot currently holds.
Because any run-level acceptance writes it, a stored value carries no act to
migrate as, and a migration that assumed `DesignAccepted` would mislabel one.
With a single live run that has never been materialised, that is a hand repair
rather than a mechanism — recorded so nobody writes the assuming migration.

`IntegratedReview` (`attestation.rs:224`) fails it, and an earlier draft had it
passing. `ReviewGroup` holds it beside the other two (`snapshot.rs:330`); it is
`id` plus a `ContentCoverage`; and it records no act by any actor in `ActKind` —
it records that a *pass* happened. That was read as saying something no general
shape could. It is not: `sec-3`'s `ReviewPass` says exactly that and one thing
more, a `ReviewRef` where this record has a `DesignId`, which is the difference
`DEC-125` names when it says the run *"cannot hold or resolve an `RV`"*. A shape
that is another shape minus the field that makes it usable is subsumed, not
distinct — `LockAcceptance`'s case again.

**It retires with this slice, not with `IMP-392`.** The earlier draft deferred it
so the currency lamp could keep riding `ReviewStanding::integrated_current`
unchanged, which read as conservatism and was really a warning left derived over
a record that cannot name the artefact it is warning about. `ReviewPass` is
minted on entry to `reviewing` whatever `IMP-392` has landed, so the lamp's
input exists now; what waits on that item is the *finding set* the severity
summary counts, not the pass. Its condition retires with it, since
`integrated-review-present` folds into `review-disposition-attested`.

`Evidence` (`facts.rs:23-27`) fails the test, and is the incumbent it was never
applied to. It is a condition, a subject and a fingerprint — a `CheckpointAct`
with `covered: Some(Sections(…))` and no act identity, which is
`LockAcceptance`'s subsumption one field weaker. It also has nothing left to say
it to: its sole consumer is `DerivedDesignFacts::satisfies` (`facts.rs:95-99`),
the existential scan `sec-3` dissolves, and none of the eight acts is stored as
one. So `Evidence`, `EvidenceDeclaration` (`submission.rs:486`) and the
`evidence` wire field retire. It is named here rather than left implied because
two surfaces read it and would otherwise be found by whoever deleted the type;
`sec-5` carries what the retirement costs the envelope and the change log.

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

That table **is** `ActKind`'s membership, which is why the type is written here
and not earlier: `sec-3` uses it as a field type and defers the members to
whichever section specifies each act, and this is that section.

```rust
/// Every act a condition may require, in the order the table above lists them.
/// Closed at eight. `BlockingSetDeclared` and `DraftingReady` are the two an
/// agent may author, narrowed by `AgentActKind` below.
pub(crate) enum ActKind {
    GovernanceConfirmed,
    GraphReviewed,
    BlockingSetDeclared,
    SufficiencyAccepted,
    DraftingReady,
    SectionReviewed,
    ReviewDisposed,
    DesignAccepted,
}
```

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

**What reaches the snapshot reaches a slot with no act identity.** A checkpoint's
own `cp-`-scoped acceptance rides a `CheckpointPlan` and is journalled, so a gate
reading `DesignSnapshot` plus `DerivedInput` never sees it. The *run-level*
`AcceptanceDeclaration` does reach the snapshot — into `ReviewGroup`'s single
`acceptance` slot (`snapshot.rs:322-334`), which `run.rs:334-347` overwrites on
every accepted declaration at any stage, last-write-wins. `SL-243` is the
demonstration: at `drafting`, never locked, holding one acceptance whose basis is
its own `inquiring → drafting` settle-and-advance. So the incumbent offers one
unnamed acceptance where this design needs five named ones — the first reason
again at run scope, and the reason `CheckpointAct` is replaced *by act*.

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
    /// Each observed fact as it stood when the act was given. Deliberately the
    /// bare map rather than `sec-3`'s `ObservedFacts`: that type is transient by
    /// construction and says so, and this one is persisted (invariant 10).
    observed: BTreeMap<ObservedFact, Fingerprint>,
    /// The agent declaration this act confirms, where its rule names one.
    confirms: Option<Fingerprint>,
    /// How the act disposes of a review, where its rule names a disposition,
    /// and WHICH pass it disposed. `Some` on `ReviewDisposed` alone.
    disposition: Option<DisposedPass>,
}

/// A disposition and the pass it was given over (`sec-3`).
///
/// The pass reference sits BESIDE the arm rather than inside `Conducted`,
/// because both arms bind to it and only one names an `RV` for its own reasons.
/// The row is satisfied only while `pass` equals the run's current
/// `ReviewPass::review`, which is what makes a waiver dispose of one pass.
pub(crate) struct DisposedPass {
    pub(crate) pass: ReviewRef,
    pub(crate) disposition: ReviewDisposition,
}

/// A covered map in the shape the act's `Coverage` selector names.
pub(crate) enum CoveredSet {
    Sections(ContentCoverage<Fingerprint>),
    Nodes(ContentCoverage<NodeMaterial>),
}

/// `DEC-125`'s two arms, given a home. Admissibility is `DEC-138`'s.
pub(crate) enum ReviewDisposition {
    /// A pass was run and is being disposed of. Admission refuses a `review`
    /// that is not the current pass, and one whose `RV` carries no concluded
    /// marker; the gate then reads its undisposed blockers. Both through
    /// `sec-3`'s `ObservedReview`. The field is retained beside `DisposedPass`
    /// rather than folded into it because the two can only differ by being
    /// refused, and a value that says the same thing twice is checkable.
    Conducted { review: ReviewRef },
    /// The user declines a pass, on the record. Admissible over any review
    /// state; the reason must be non-blank (`sec-3`), which is the only thing
    /// admission checks here.
    Waived { reason: String },
}

/// A canonical `RV` id, as the run records it.
///
/// Deliberately **not** a `DesignId`. Run-local ids are one space and an `RV`
/// sits outside it — which is the defect `DEC-125` names when it says the run
/// "cannot hold or resolve an RV (`IntegratedReview.id` is a DesignId)".
/// Validated at the wire boundary and opaque to the pure layer, so `design_run`
/// can name a review without depending on the review module (`ADR-001`).
pub(crate) struct ReviewRef(String);
```

**The arm is a slot, not prose.** `DEC-125` fixed the two arms and this design
carried them in eight passages without ever saying where the value lives — so
`Conducted`'s `RV` reference and `Waived`'s reason had no home in any type. The
slot is the fourth of exactly this shape on `CheckpointAct`, beside `covered`,
`observed` and `confirms`: per-act optional, `None` for the seven acts whose rule
names no disposition, and paired with its rule by the same admission check.

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
| `ActRequirement::confirms` | `CheckpointAct::confirms` | a confirmation is present exactly when the rule names a declaration, and absent exactly when it does not |
| `ActRequirement::disposes_review` | `CheckpointAct::disposition` | a disposition is present exactly when the rule names one — `ReviewDisposed` alone — and absent exactly when it does not |

The third row is why `confirms` is on the requirement rather than inferred from
the conjunction. Reading it off the rule's agent conjunct would work only while
no rule has two of them — an invariant nothing states — and, worse, would leave
`confirms: None` corresponding to its rule perfectly, so an act could drop the
ordering guarantee without failing anything. A slot the rule names is a slot
admission can require.

What was missing is who checks the correspondence. **Admission does**, over the
record as a whole rather than slot by slot: a recorded act that does not
correspond to its rule is refused on write, whichever shape holds it. Every
stored act therefore satisfies the invariant by construction, and the gate never
re-checks it — the same move `sec-3` makes for the generated tables, one layer
out.

**The correspondence ranges over three record shapes, and three of its four rows
reach only one.** Writing it against `CheckpointAct` alone was the same
slot-by-slot mistake one level up: two of the eight acts are `AgentDeclaration`s
and one is an `Attestation`, and a rule naming a slot those shapes lack would be
an unenforceable requirement rather than a refused act.

- **Coverage applies to all three.** `AgentDeclaration::covered` pairs with its
  rule's `Coverage` exactly as `CheckpointAct::covered` does — `Artefact` with
  `None`, `InquiryMap` with `Nodes(…)` — so `BlockingSetDeclared` and
  `DraftingReady` are admitted under the same check. `SectionReviewed` is the
  degenerate case already argued: `PerSection` is carried by no act, so an act of
  any shape whose rule names it is refused.
- **The other three rows reach `CheckpointAct` alone, by rule rather than by
  record shape.** Only a requirement whose act is one of the five given at a
  checkpoint may name a confirmation, a disposition or an observed fact, and the
  generated table is where that holds. So neither `AgentDeclaration` nor
  `Attestation` carries those slots — not because either was given fewer, but
  because no rule can ask them for one. Keying the restriction on
  `ActorClass::Agent` instead would have covered the two agent acts and left
  `SectionReviewed` — whose actor is a reviewer lane, not an agent — free to name
  a confirmation with nowhere to put it.

Each of the three earns the restriction separately, and only the third is a
decision rather than a consequence. `confirms` runs one way — `DEC-121`'s
interaction is *the agent declares, the user confirms* — so the confirming act is
always a user act at a checkpoint; an agent act confirming another, or a section
review confirming anything, is an interaction nobody has specified. `disposition`
is `DEC-138`'s, and both of its arms are user acts carrying an
`AcceptanceDeclaration`, so an agent-authored disposition is exactly the value
`AgentActKind` exists to make unrepresentable and a per-section one has no
subject to dispose. `observed` is not intrinsic: a declaration or a section
review *could* be bound to canonical state outside the run. It is refused because
that state is governance, confirming governance is `GovernanceConfirmed`'s job,
and that is a user act under `DEC-121`. If a later act wants the pairing it is a
decision, and the slot arrives with it — which is this section's rule for shapes,
applied to a slot.

**How the restriction is actually held, because the type does not hold it.** An
earlier draft closed this by saying *a rule that asked would not compile against
a record that cannot hold it*. That is false, and the reason is worth keeping:
`ActRequirement`'s four fields are independent, so
`{ act: BlockingSetDeclared, actor: Fixed(Agent), confirms: Some(…) }` is a
perfectly well-typed row. It compiles, it joins every generated set, and every
`AgentDeclaration` written against it is then refused at admission for lacking a
slot no agent record has — an **active condition that nothing can satisfy**,
which is precisely the state `Activation` exists to keep out of the table.
Claiming unrepresentability where only rejection is delivered is the same
overclaim this section corrects for `AgentActKind` two subsections down.

So the check is generated, not typed: the macro that emits `CONTRACTS` also emits
a compile-time assertion per row that a requirement naming `confirms`, a
disposition or an observed fact has an act in the checkpoint five. It is a
`const` predicate over data the macro already has, so the failure is a build
error at the offending row rather than a test somewhere else — which is the same
standard the `Advance` keying meets, and the honest version of what this
paragraph used to claim. Coupling `ActRequirement` into per-family variants would
also work and costs a type split across four rows to prevent a mistake one
assertion catches; the assertion is the cheaper answer, and it is the one that
keeps the table flat enough to read.

Two consequences worth naming. An act whose `observed` map is simply **absent**
where its rule names a fact is refused rather than treated as an empty
observation, so the strict-path argument `sec-3` makes for conjunctive binding
cannot be evaded by omitting the field. And a *new* `ObservedFact` added to an
existing rule invalidates every already-stored act of that kind — which is the
correct reading, since such an act was given over less than the rule now
requires.

**That second one is the gate's doing, not admission's**, and saying otherwise
would claim a mechanism that does not exist: admission runs on write and never
re-runs, so it cannot reach an act stored before the rule changed. The
invalidation falls out of the conjunct comparison instead — the gate looks up
the fingerprint for each fact the rule names, finds none stored for the new one,
and an absent stored fingerprint reads as **changed**, exactly as `sec-3` makes
an unobservable fact read. So the one qualification on *the gate never
re-checks the correspondence*: it does not re-derive whether the record matches
its rule, but it does read the record **through** the rule, and a rule that has
grown since simply finds less than it asks for. The correspondence stays an
admission-time invariant for every act admitted under the rule in force; a rule
change is the one thing that can retire an act, and it does so by making it
unmet rather than by making it invalid.

**Wire, replacement, and snapshot home.** Left unstated, these are where the
implementations diverge, so:

- **Declared** as a run-level field on `ApplyRequest`, joining `WRITER_ACTS`
  (`submission.rs:664`) — the array whose own doc says a new *field* that can
  change state and did not join the list **is** the gap. This slice adds three
  such fields — this one, the agent declaration, and the review policy — and
  retires one, `evidence`, with the record it declares against. So the
  array goes from `[WriterAct; 7]` to `[WriterAct; 9]`, each new row pairing a
  `WRITER_ACT_*` key constant (`STD-001`) with the predicate that detects it —
  the pairing being what that doc calls authoritative rather than merely
  parallel. It carries an `AcceptanceDeclaration`,
  so `AcceptanceAttestation::bind` stays the only route to user authority and
  `DEC-088` is untouched.
- **Replaced by act**, not by id: at most one live `CheckpointAct` per `ActKind`,
  a new one superseding the prior. The mechanism is the incumbent's —
  `run.rs:1062` retains-then-pushes — but the **key is not**, and the difference
  matters. An `Attestation` is replaced by its own id, deliberately, because two
  lanes reviewing one section must coexist under a both-lanes policy; that is
  what a per-subject record needs and what a per-act one must not do. Reading the
  incumbent's key as this design's is how a two-lane run would silently lose a
  lane. Two live acts of one kind would make *which one is current* ambiguous
  with no rule to break the tie.
- **Stored** in its own snapshot group, for the reason `delegation` and `runbook`
  have theirs: a recorded act is its own state model, and `ReviewGroup`'s members
  are all about review.

`DesignAccepted` rides the same three, which is what retiring `LockAcceptance`
means concretely: its wire type is already `AcceptanceDeclaration`
(`submission.rs:627`), so the run-level acceptance field keeps its shape and
changes where it lands.

### The wire types, and the order the records are built in

Naming the fields is not enough, because the persisted records above hold
engine-authored material — an allocated `id`, a covered map, observed
fingerprints, a declaration digest — and a caller cannot be trusted with any of
it. So the wire types are strictly the *claim*, and the engine builds the record:

```rust
/// A user act at a checkpoint. `covered`, `observed` and `confirms` are
/// ABSENT — the engine supplies each from state the caller cannot assert.
pub(crate) struct CheckpointActDeclaration {
    pub(crate) act: ActKind,
    pub(crate) acceptance: AcceptanceDeclaration,
    /// Present exactly on `ReviewDisposed`, per the correspondence.
    pub(crate) disposition: Option<ReviewDisposition>,
}

/// An agent declaration. Carries its payload and its basis, and nothing else.
pub(crate) struct AgentActDeclaration {
    pub(crate) act: AgentActKind,
    pub(crate) basis: String,
    pub(crate) turn: Option<String>,
}

/// A policy change, which is a user act like any other.
pub(crate) struct ReviewPolicyDeclaration {
    pub(crate) policy: ReviewPolicy,
    pub(crate) acceptance: AcceptanceDeclaration,
}
```

The three join `ApplyRequest` as `checkpoint_act`, `agent_declaration` and
`review_policy` — `Option` each, since `Batch` refuses to carry an order
(`DEC-063`) and two acts of one kind in one submission would need one.

**The order is load-bearing and the incumbent already fixes most of it.**
`DerivedInput` is built *before* `apply` runs the batch (`design.rs:1270`), and
the run re-observes section fingerprints *after* the declaration loop and before
the stage move (`run.rs:318`). A coverage captured from pre-batch state would
record what the act was *not* given over. So:

1. **Admit.** Every refusable thing, including each act's correspondence against
   its rule, while the authored tier is untouched.
2. **Mutate.** Declarations, dispositions, traversal — the batch's own effects.
3. **Re-observe.** Section digests and node material, so what follows covers what
   this batch left behind. The incumbent's step, one subject wider.
4. **Construct.** The engine allocates each record's `id`, fills `covered` from
   step 3 in the shape the rule's `Coverage` names, fills `observed` from
   `DerivedInput`, and fills `confirms` from the declaration the rule names.
5. **Evaluate.** The stage move, against records that already exist.

Step 4 is what lets a declaration and its confirmation arrive in **one**
submission: the `AgentDeclaration` is constructed and fingerprinted before the
`CheckpointAct` that confirms it, in one fixed order rather than by the caller
naming a digest it would have had to compute. That case is not hypothetical —
`run.rs` already allows a disposition recorded and a stage taken together, and
`DEC-121`'s interaction is two acts about one graph.

**The record ids stay**, and they are `DesignId`s on the run's existing prefixes
— `cpa-` for checkpoint acts, `agd-` for agent declarations. They are not
addressable by a caller and no rule reads them, so the case for dropping them is
real; they are kept because `invalidation_rows` reports the death of a recorded
act *by subject*, and an act with no id has nothing to name itself with in a
change row.

### The concluded-pass marker

`sec-3` fixes that `Conducted` is admissible only over a pass that concluded,
and that the signal must be structured state on the `RV` rather than its
findings or its prose. This is what that state is, because a description is not
something `IMP-392` can build.

**Shape.** One key in the `RV`'s `[review]` table — `concluded`, a boolean,
absent by default — never set by an edit to a finding. `IMP-392` is already
opening this record to give findings a section reference, which is why the marker
rides that item rather than one of its own.

**The writer is a new verb, `doctrine review conclude <RV>`, and naming it is
half the finding.** An earlier draft said the marker was *"set by the verb that
ends a pass"* without checking whether one exists. None does: the review surface
is `new`, `raise`, `dispose`, `verify`, `contest`, `withdraw`, `list`, `show`,
`status`, `prime`, `unlock`, `paths` — every finding verb moves one finding, and
`status` is derived and writes nothing. Since the marker is deliberately *not* a
function of the finding set, no existing verb can set it as a side effect without
acquiring a second meaning, and `dispose` is the worst candidate of them: it is
per-finding and the responder's, where concluding is per-pass and the raiser's.

Its shape, so `IMP-392` has something to build:

- **Authority — the raiser's**, on the `review.rs` axis that already separates
  `raise`/`verify`/`withdraw` from `dispose`. Concluding is the reviewer saying
  *I have finished reading*, which is exactly the claim the design run's
  `Conducted` arm is repeating one layer out.
- **Idempotent.** Concluding a concluded pass is a no-op, not a refusal. The
  marker is a latch, and the verb's job is to set it.
- **Open findings are allowed, and this is the normal case.** A pass that found
  things concludes with them outstanding; disposing them is the responder's work
  afterwards, and `DEC-138` is precisely the rule for advancing while some are
  still live. Requiring a clean ledger would make the marker a second, stricter
  spelling of the gate it feeds.
- **No unset.** A concluded pass cannot be un-concluded — it happened. A run that
  wants another pass gets a new `RV`, which is what `sec-3`'s `ReviewPass`
  replacement already does on re-entry to `reviewing`.
- **It takes the same per-review lock** every mutating review verb takes, so a
  concurrent `raise` cannot interleave with it; `review unlock` remains the
  escape hatch it already is.

**Why an authored field is lawful here, where a status field is not.** The `RV`
carries an explicit refusal of exactly this shape, in a comment at the top of
its own TOML: *"no status — a review's status is DERIVED from its findings
(ADR-007 D-C8), so it is never stored (the storage rule forbids derived data in
authored files)."* The marker is not that, and the difference is the one
`sec-3` already established: a pass concluding is an **event**, and it is not a
function of the finding set, because a clean pass and one never run present the
same findings, the same derived status and the same `await`. Storing what
cannot be derived is the storage rule holding rather than bending — and a
reader who meets that comment before this subsection will otherwise read the
marker as violating it.

**Absence.** No marker is not-concluded. There is no third state and no
migration: every `RV` minted before this lands reads as unconcluded, which is
the right answer for a ledger nobody closed and the conservative one for a
ledger somebody did.

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
/// The acts an agent may declare, each carrying what it declares. Closed, and
/// deliberately NOT `ActKind`: an agent-authored `DesignAccepted` is the value
/// that must not exist.
///
/// Tagged with its payload rather than paired with an optional field beside it,
/// so `DraftingReady` cannot carry a blocking set and `BlockingSetDeclared`
/// cannot omit one.
pub(crate) enum AgentActKind {
    /// The inquiries the agent considers blocking — `DEC-121`'s artefact, and
    /// the thing the user's `GraphReviewed` confirms. Every id must be a node
    /// of the covered map; an id outside it is refused at admission.
    BlockingSetDeclared { blocking: BTreeSet<DesignId> },
    /// The agent's judgement that drafting may begin. Its basis is the claim.
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
    /// else. Shell-computed and arriving on `DerivedInput`; a confirming
    /// `CheckpointAct` names it. See below for why `covered` is excluded.
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

**It carries no `observed`, `confirms` or `disposition`**, and that is the rule
stated with the correspondence above rather than a shape decision taken here: no
requirement whose actor is an agent may name any of the three, so there is
nothing for those slots to hold.

**Replacement is by act, not by id.** At most one live declaration per
`AgentActKind`; a new one replaces the prior, on the same retain-then-push
mechanism and with the same key difference from `Attestation` noted above. Two
live `BlockingSetDeclared`s would make *which one did the user confirm*
ambiguous, and removing that ambiguity is what the confirmation link below exists
for. Its wire shape is a run-level field on `ApplyRequest`,
joining `WRITER_ACTS` as the second of this slice's three new writers.

**What the fingerprint is taken over.** `act` and `basis` — where `act` is its
kebab name followed, for `BlockingSetDeclared`, by its node ids ascending, one
term per line. Every line LF-terminated, UTF-8, sha256, the same hash the rest of
the run uses. `id` is excluded because the engine allocates it and it is not
content; `turn` because it is a harness detail rather than part of the claim; and
`covered` because its currency is already the coverage mechanism's job.

**The blocking set is inside the fingerprint, and that is the point of the
confirmation.** `DEC-121`'s interaction is *the agent declares which questions it
considers blocking, the user confirms* — so what the user confirmed must be the
set, not merely that a declaration existed. Ids ascending, because a set has no
order and two declarations of the same questions must agree; this is the same
whole-line-sort discipline the governance edge projection uses below, and for the
same reason. An earlier draft hashed `act` and `basis` alone while the set lived
nowhere at all, so a re-declaration with different questions carried the same
fingerprint and `confirms` still matched.

**And it is computed where every other digest is.** The pure layer never hashes
— *"it is handed the digest as a derived fact"* (`ids.rs:177-178`) — so this
arrives on `DerivedInput` beside `section_digests`, on the same channel and for
the same reason: a digest the caller supplied would be a claim about content
rather than a fact about it. Left unsaid, this is the one slot in the new shapes
with no route to a value. Folding the covered map into
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
  acceptance does. An agent cannot relax the rules as *housekeeping*; it must
  present the change as the user's, in the same shape as every other user
  judgement in the run. It can still present it — that is what the subsection
  below means by declaration of intent rather than security boundary.
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
| `render/envelope.rs:799` `review_outstanding` | does the envelope call this section settled? | yes |
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

**Both lanes means reading twice, and neither reading is a gate fee.** Under
either ordered variant an adversarial reviewer reads each section and then reads
the document — a defensible template, since the per-section pass catches what is
local and the integrated pass catches what is not. It must not become a toll.
Both readings are escapable by a user act, and the design says so in one place
rather than leaving it inferable from two subsections that never mention each
other: the integrated pass is declinable through `review-disposition-attested`'s
`Waived { reason }` arm, which `sec-3` records as a permanent arm rather than an
interim one; and the per-section lanes are escapable by revising the run's
policy, which this subsection has already argued must stay mutable. A user who
says skip it is obeyed. What the gate keeps is not the reading but the record of
the choice — a reason on the waiver, a change row on the policy — which is the
whole of what it is for.

`Reviewer` maps into `ActorClass` — `Human → User`, `Adversarial → Adversarial` —
as an `impl From`, not a merge. `sec-3` refused to unify the three actor-ish
vocabularies and that refusal stands; this is the one direction it required.

**Cost, stated — and it is not this policy's.** `SL-243` is the one live run and
it holds no section attestations at all (`[review] attestation = []`), so
`HumanOnly` costs it nothing and `AdversarialOnly` would buy it nothing. An
earlier draft claimed it had attested adversarially and offered the policy as its
repair; that was reasoned from this design instead of read off the run. What the
run does lose is `sec-2`'s to record, and none of it is a policy question.

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

So the link is explicit and persisted, and it is spelled across both halves.
`ActRequirement::confirms` is the rule half — `Some(BlockingSetDeclared)` on the
`GraphReviewed` requirement, `None` everywhere else. `AgentDeclaration` carries
its own `fingerprint`, and `CheckpointAct::confirms` holds the fingerprint of the
declaration the user was shown: the record half.

**The lookup is by act, not by fingerprint.** The rule names which declaration
must be confirmed, so the gate fetches that declaration and the stored
fingerprint answers only *is it still the one confirmed*. `confirms` on the
record therefore needs no act tag of its own — it would be naming a fact the rule
supplies — but the rule must name it, because that is what makes the
confirmation **required** rather than merely possible. Admission enforces the
pairing, per the correspondence above, so a `GraphReviewed` act that names no
declaration is refused on write and never becomes a silently order-free act the
gate would have to catch later.

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
- **Both lanes are required, per section** — under `HumanThenAdversarial` a
  section carrying only a human attestation leaves `section-attestations-current`
  unmet, and the refusal names the *adversarial lane* specifically. The nested
  quantification, tested where the single-lane policies cannot reach it.
- **Order is not enforced** — a run whose policy is `HumanThenAdversarial`
  clears with the adversarial attestation recorded first. The order is rendered;
  the gate does not read it.
- **Waiving is not a degraded path** — `reviewing → locked` clears on a
  `Waived { reason }` disposition with no integrated pass recorded and no
  findings raised, and the reason is rendered. Asserted so that a later slice
  tightening this is a visible break rather than a silent one, the same reason
  the loosening test exists. The findings-outstanding case is `sec-3`'s, with
  `DEC-138` — and it clears there too, which is the point of that record.
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
- **The agent-declaration channel cannot encode a user act** — `AgentActKind` has
  two members, so `AgentDeclaration { act: DesignAccepted }` does not compile.
  A build-time property of *that shape*, so there is no runtime test to write —
  and deliberately not the claim that an agent cannot author a user act, which
  `--as` and `DEC-088` make false. What narrows here is one channel, not the
  caller.
- **A mismatched coverage is refused on write** — a `CheckpointAct` whose
  `CoveredSet` is `Sections` where its rule's `Coverage` names `InquiryMap` is
  refused at admission, so no stored act can violate the correspondence.
- **And on an agent declaration too** — a `BlockingSetDeclared` whose
  `CoveredSet` is `Sections` where its rule names `InquiryMap` is refused at the
  same point. Asserted separately because the correspondence was first written
  against one record shape and holds over three; testing only the shape it was
  written against is what left this unchecked.
- **A mismatched observed set is refused on write too** — a `GovernanceConfirmed`
  act carrying no `observed` entry, or one keyed by a fact its rule does not
  name, is refused at the same point. Asserted separately from the coverage case
  because the correspondence is over the whole record, and testing only the slot
  that was found first is what left this one unchecked.
- **A disposition is refused where the rule names none, and required where it
  does** — a `ReviewDisposed` act carrying no `ReviewDisposition` is refused, and
  a `DesignAccepted` act carrying one is refused. The fourth correspondence row,
  asserted like the other three.
- **`Conducted` over an unconcluded `RV` is refused on write** — naming the
  auto-minted `RV` before any pass has concluded is refused at admission,
  reading `ObservedReview::concluded`, so the claim cannot enter the record. An
  `RV` the shell could not read is refused on the same branch, since absence is
  not-concluded. Complementary to the waiver test: the same run clears
  immediately under `Waived { reason }`. Needs `IMP-392`'s marker, so it is
  specified here and lands with that item.
- **A blocker raised after the disposition unmakes it** — a `Conducted` act that
  cleared `reviewing → locked` leaves the row unmet once a new `blocker` is
  raised `open` on the named `RV`, with no edit to the act itself. The gate
  re-reading `ObservedReview::undisposed_blockers` is what gives this row its
  cumulative reach, and asserting it is what stops that label from being
  decorative. Needs `IMP-392`'s finding set, so it lands with that item.
- **A missing confirmation is refused on write, and so is a gratuitous one** — a
  `GraphReviewed` act carrying no `confirms` is refused because its rule names a
  declaration, and a `SufficiencyAccepted` act carrying one is refused because
  its rule does not. Both directions, because the correspondence is
  presence-exactly-when and not presence-at-most.
- **A rule that grows an observed fact unmakes acts stored before it** — adding
  a second `ObservedFact` to an existing rule leaves an already-stored act of
  that kind unmet, with no re-admission and no migration. Asserted at the gate,
  because that is where the mechanism is: admission cannot reach a record it has
  already written.
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
- **`SL-243` needs no policy repair.** It holds no section attestations, so the
  lane question never arises for it. Its actual cost is `sec-2`'s to record.
- **The snapshot changes in five ways, not one** — `ReviewPolicy` on the run
  header, a checkpoint-act group, an agent-declaration group, `LockAcceptance`
  retiring into `CheckpointAct`, and the gate group's evidence rows retiring with
  the record. The last two are migrations rather than additions, and are the only
  ones that touch data an existing run already holds. `sec-2` carries both costs,
  read off `SL-243` rather than inferred — it is the same run's
  `user-accepts-sufficiency` evidence row that section already prices.
- **The policy is not a security boundary**, and the design says so. If a later
  slice wants it to be one, the missing piece is not a stricter gate but a way to
  distinguish a user's payload from an agent's — which is `DEC-088`'s standing
  limitation, not this slice's to close.
<!-- doctrine:section sec-5 -->
## The contract channels

`sec-3` fixed the model and `sec-4` fixed the acts. Both wrote promises this
section has to keep: nine separate places say *the refusal names* something — the
missing act, the missing lane, the node that moved, the declaration that changed,
the finding that blocks — and no section has yet said where any of that rides.
Both cite `DEC-122`, `DEC-123` and `DEC-124` without naming a channel. This is
that section.

`DEC-124` gives two channels and denies a third: the refusal carries the remedy
for each unmet condition; a stage-entry receipt carries the contracts for that
stage's outbound edge; the turn envelope gets nothing new. Taken in that order,
plus what the envelope and the change log lose when the claimed arm dissolves.

### What a contract is, and why it stays a pure function of the binary

A contract has two halves in two homes. `DEC-122` puts the narrative in a prose
asset keyed by the condition's kebab token; `DEC-123` keeps the structure in the
`CONTRACTS` const and forbids the prose from restating it — *"the renderer
injects the kind and the discharging act from the const into the rendered
contract, so the prose never restates them and cannot contradict them."* The
rendered contract is those two composed, and the composition happens at emission
rather than being authored anywhere.

What is injected is every field `sec-3`'s `Contract` holds except the prose key
itself: the derivation's kind, the acts it names with their required actors, the
binding, the reach, and the activation. Reach and activation are injected for the
same reason as the kind — they are exactly the properties a prose author would
restate and get wrong, and `Pending` in particular must render *marked
not-yet-enforced* rather than being described as enforced by prose written before
`IMP-391` lands.

**`DEC-124`'s no-digest rationale survives `sec-4`, and one shape is why.** That
record rejects a digested receipt because *"a contract is a pure function of the
binary … All ten were walked; none is parameterised by the run."* One row now
looks like a counterexample: `section-attestations-current` takes
`RequiredActor::RunPolicy`, and the lanes it resolves to are run data. The
contract is unaffected, because the table names *where the actor comes from*
rather than naming the actor — so the injected line reads `actor: the lanes this
run's review policy requires`, which is a constant string in a `&'static` table.
The resolved lanes are state, and state belongs to the complaint, not to the
contract. This is the same property `sec-3` relied on to keep the table
`&'static` and total; it pays twice, and had the per-section candidate won
instead, both payments would have been lost together.

**One arithmetic correction to `DEC-124`, which changes nothing it decided.** Its
addressing consequence reads *"at reviewing→locked the set is all ten"*. The
citation resolves — `cumulative_conditions` is `gate.rs:212-226` and does
accumulate every edge below the target — but the vocabulary is nine now, and the
enforced set at that edge is **six**: `governing-context-recorded` and
`initial-concerns-recorded` are filtered by activation, `drafting-readiness-attested`
by reach. Three of the six are inherited from edges below. The consequence the
number was supporting — an agent can fail at the last edge on an earlier edge's
condition, so addressing must be per-condition — is untouched, and is what the
rest of this section builds against.

### The refusal carries a const remedy and a run-shaped complaint

These are two different things and a refusal that conflates them is the one this
slice is replacing.

**The remedy is a total function of the condition.** `Contract::remedy()` renders
it from the `DerivationRule` — `sec-3`'s invariant 4, one discharge source, so
the refusal text and the injected contract line cannot disagree because they are
the same value formatted twice. It reads nothing outside the const table, which
is what keeps the whole of this leg inside tier `leaf`: no asset, no shell, no
`DerivedInput`.

That is worth stating rather than assuming, because `sec-1`'s structural fact is
the opposite case. `RunbookNotDischarged` exists *because* `GateNotCleared`'s
`Vec<Condition>` had *"nowhere for a step identity to ride"*
(`refusal.rs:166-170`) — a step's obligation text is not recoverable from the
step's identity, so it had to be carried. A condition's remedy **is** recoverable
from the condition's identity, once the contract table exists. So the remedy
needs no new refusal field at all, and `Condition` keeps the fieldless
`Copy`/`Ord`/serde shape `DEC-122` promised it would.

**The complaint is not a function of the condition**, and that is what has to
ride. Nine promises across the two prior sections need it:

| promise | source |
|---|---|
| which conjunct act is missing | `sec-3`, `DEC-121`'s stated reason for two acts |
| which *lane* is missing, not merely which act | `sec-3`, resolution fixes the arity |
| which node moved, rather than that the map did | `sec-3`, the reason coverage stores material |
| which section lost its own review | `sec-3`, `PerSection` |
| that the document holds no sections at all | `sec-3`, the degenerate guard |
| which observed fact moved or is unobservable | `sec-3` |
| which declaration a confirmation no longer names | `sec-4`, three facts not one |
| which findings hold the edge | `sec-3`, `DEC-138`'s predicate |
| that the waiver arm is available and why | `sec-3`, *"the refusal text says so"* |

So the refusal's payload changes shape:

```rust
GateNotCleared {
    from: Stage,
    to: Stage,
    /// Every condition the edge required and did not get, each with every way
    /// it failed. Replaces `missing: Vec<Condition>`.
    unmet: Vec<Unmet>,
},
```

```rust
pub(crate) struct Unmet {
    pub(crate) condition: Condition,
    /// EVERY way this condition is unmet, not the first. A conjunction that
    /// fails on both halves says both — which is `DEC-121`'s own reason for
    /// refusing the fold into `user-accepts-sufficiency`.
    pub(crate) causes: Vec<Cause>,
}

/// One way a condition failed. A variant per failure mode rather than a message,
/// for the reason `ReviewStanding`'s four booleans are four: each is repaired by
/// a different act, and collapsing them reports one outstanding thing where
/// several are.
pub(crate) enum Cause {
    /// No live act of the required kind. `lanes` is the lanes with no act —
    /// one entry under `Fixed`, up to two once `RunPolicy` resolves.
    ActMissing { act: ActKind, lanes: Vec<ActorClass> },
    /// `PerSection`: these sections have no live act for the named lane.
    SectionsUnreviewed { subjects: Vec<(DesignId, ActorClass)> },
    /// A per-section requirement over a run holding no sections. Distinct from
    /// an empty `SectionsUnreviewed`, which cannot occur.
    NoSections,
    /// The act is live and what it was given over has moved.
    CoverageStale { act: ActKind, moved: Vec<DesignId> },
    /// A fact the rule names has moved, or could not be observed at all.
    ObservedStale { act: ActKind, fact: ObservedFact },
    /// The act names a declaration that has since changed.
    ConfirmationStale { act: ActKind, declaration: AgentActKind },
    /// The named `RV` carries blockers in `open` or `contested`, by `F-n` id.
    BlockersUndisposed { findings: Vec<String> },
    /// The disposition was given over a pass that is no longer the run's
    /// current one — `sec-3`'s re-entry rule. Names both, because the repair is
    /// to dispose the new pass and the reader has to know there is one.
    PassSuperseded { disposed: ReviewRef, current: ReviewRef },
    /// The act is live and names an `RV` the shell could not read at all.
    /// Distinct from `BlockersUndisposed`, which reports a ledger that WAS
    /// read; reporting an unreadable one as carrying blockers would name
    /// findings nobody has seen.
    ReviewUnavailable { review: ReviewRef },
    /// Engine rows, which fail for their own reasons and name their own state.
    InquiriesOpen { nodes: Vec<DesignId> },
    MaterialisationStale,
}
```

`ReviewUnavailable` is what makes the evaluator total over `sec-3`'s absence
rule. A stored `Conducted` act can name an `RV` that later becomes unreadable —
the act is present, its correspondence was checked when it was written, and
admission cannot reach back. `sec-3` says the gate must then be unmet, and
without this variant there was no truthful value to be unmet *with*: the act is
not missing, its coverage has not moved, `ObservedReview` is deliberately not an
`ObservedFact` so no observed-fact variant applies, and `BlockersUndisposed`
would assert a finding set that was never read. A `Result` with no true error
value for a branch the design specifies is not a total function.

`causes` is non-empty in fact and not in type. The evaluator is its sole
producer and returns satisfied where it pushed nothing, so an empty `Unmet`
cannot be constructed by any path that exists — but nothing in the type says so,
and this design does not introduce a non-empty-vector type to make it say so.
Stated rather than claimed as an invariant.

**`satisfied` returns the diagnosis, and this changes `sec-3`'s sketch.** That
section fixed the *inputs* — one derivation, no branch on kind, the whole
`DerivedInput` rather than a parameter per fact class — and wrote the return as
`bool`. A `bool` cannot carry any row of the table above. So:

```rust
fn satisfied(condition: Condition, run: &DesignSnapshot, derived: &DerivedInput)
    -> Result<(), Vec<Cause>>
```

One function, not a predicate plus an explainer. Two functions would be two
derivations that can disagree — a gate that refuses while the explanation says
nothing is wrong is worse than either alone, and it is the drift class this
design has closed everywhere else by construction. `advance`'s filter
(`gate.rs:325-333`) collects `Unmet` instead of `Condition` and is otherwise
unchanged.

**`ContentCoverage` owes a diff, not just a verdict.** `CoverageStale::moved`
needs the subjects, and `is_current` returns `bool` (`attestation.rs:212`). So
the generic type `sec-3` introduced gains `diff(&current) -> Vec<DesignId>` —
keys present on one side only, plus keys whose value differs — and `is_current`
becomes `diff(current).is_empty()`. Still one comparison with one home. This is
the payoff `sec-3` named when it chose material over a digest: *"a refusal can
name which node moved rather than reporting that the map did."* With a composite
digest there would be nothing to diff.

**Rendering, and the discipline it bends.** `Refusal`'s `Display` doc calls
itself *"a terse, single-line rendering"* and says *"the data is the contract —
tests assert on the variant, never on this text"* (`refusal.rs:267-272`). The
second half stands untouched and is what makes the first half negotiable: the
shell's only crossing is `anyhow!("{refused}")` (`design.rs:1419-1422`), so a
refusal that does not say a thing in `Display` does not say it to any caller.
`GateNotCleared` therefore renders one line per unmet condition — token, causes,
remedy — and the module doc's single-line claim is amended for that variant and
for `VerifierFailed`, which already breaks it — that variant embeds a newline
before the verifier's own output (`refusal.rs:338`). An earlier draft named
`RunbookNotDischarged` as the precedent and was wrong: both of its branches stay
on one line, the `regressed` clause appending to the first rather than following
it. `DEC-124` gives this path no
byte budget, which is the whole reason the remedy rides here rather than in the
envelope.

A structured error channel was the alternative: `refusal()` returning something
richer than an `anyhow::Error` so the shell could lay the causes out. Rejected
as a second output contract built for one variant. If the CLI ever grows a
machine-readable error envelope this leg joins it with no change to the data,
which is precisely because the data is already the contract.

**The refusal cites the contract by token**, which is the per-condition
addressing `DEC-124` forces. That address is a real one — it is the asset key's
stem, below — so an agent that fails at `reviewing → locked` on
`user-accepts-sufficiency` is told the token, the cause and the remedy, and can
name the contract even though no receipt for that stage delivered it. What it
cannot do today is *fetch* the narrative: the pull verb is `DEC-124`'s deferred
candidate, and this section does not build it. The residual is that the narrative
half of an inherited condition's contract is unreachable in-session. It is
carried forward, and it is small precisely because the remedy is const-derived
and always present.

**`IMP-390`'s gate-condition face is discharged here.** That item is *envelope
reports state, not what to do next*; `DEC-124` names the refusal leg as its
answer for gate conditions specifically. What closes it is the remedy, not the
complaint: a caller told six conditions are outstanding has state, and a caller
told which act discharges each has a next step. The item itself is wider than
this slice and stays open — the envelope's other surfaces are not `SL-244`'s —
but the condition face is answered rather than deferred, and nothing else in this
design was going to answer it.

### The other refusal: admission

`sec-4` promises, in six places, that a malformed act is *"refused at
admission"* — a `CoveredSet` whose variant its rule does not name, an `observed`
map keyed by a fact its rule does not name, a missing or gratuitous confirmation,
a disposition present where no rule names one, a `Conducted` over an unconcluded
or foreign pass, a blank waiver reason, a blocking set naming a node outside its
coverage. Every one is a real rule and none had a refusal to be raised as. The
`Refusal` enum is closed (`refusal.rs:19`) and holds nothing that fits: these are
not gate failures, so `GateNotCleared` is wrong twice over — wrong stage in the
lifecycle, and wrong data, since there is no condition to name.

One variant covers all of them, because they are one fact:

```rust
/// A recorded act does not correspond to the rule it is written against
/// (`sec-4`). Raised on WRITE, before anything is persisted — so every stored
/// act satisfies the correspondence by construction and the gate never
/// re-checks it.
ActAdmissionInvalid {
    act: ActKind,
    /// Non-empty. Every way this act failed its rule, not the first.
    causes: Vec<ActFault>,
},
```

```rust
pub(crate) enum ActFault {
    /// The rule names this coverage; the act carries that one (or none).
    CoverageMismatch { required: Coverage, carried: Option<Coverage> },
    /// The observed map's key set is not the rule's `ObservedFact` list.
    ObservedKeys { missing: Vec<ObservedFact>, extra: Vec<ObservedFact> },
    /// A confirmation is present where the rule names none, or absent where it
    /// names one. Presence-exactly-when, so one variant with a direction.
    Confirmation { expected: Option<AgentActKind>, carried: bool },
    /// Likewise for a disposition.
    Disposition { expected: bool, carried: bool },
    /// `Conducted` naming a review that is not the run's current pass.
    ForeignPass { named: ReviewRef, current: ReviewRef },
    /// `Conducted` over a pass whose ledger carries no concluded marker.
    PassNotConcluded { review: ReviewRef },
    /// A `Waived` whose reason is empty or whitespace.
    WaiverReasonMissing,
    /// A blocking set naming nodes outside the map it was declared over.
    BlockingSetUnknownNodes { nodes: Vec<DesignId> },
}
```

**One variant, not eight refusals**, on `RunbookNotDischarged`'s own precedent
and its stated reason: *"the fact refused is one fact … `EX-8` budgets exactly
one gate refusal. What differs is the repair"*. Here the fact refused is *this
act does not correspond to its rule*, and what differs is which slot — so the
slot rides a field, exactly as `outstanding` and `regressed` do there.

The parallel with `Unmet`/`Cause` is deliberate and the two must not be merged.
They answer different questions at different times: `ActFault` is *this record is
malformed against its rule*, raised once on write; `Cause` is *this condition is
not met by the records that exist*, re-derived at every crossing. Folding them
would give the gate variants it can never raise and admission variants it can
never check, which is the drift `sec-4` closes by making admission the sole owner
of the correspondence.

### The stage-entry receipt

**The register.** `fragment_section` (`design.rs:1832-1851`) is the live one: the
caller declares what it holds with `--known-fragment name@digest`, the run emits
the identity line unconditionally and the body only when the declaration does not
match. Its persisted sibling is **not** a register at all —
`DesignSnapshot::fragments` has exactly one reader (`design.rs:1863-1864`) and no
writer anywhere in the tree, so `fragment_lines` reports *NOT held by this run*
for every name a caller declares. Reading that group as the receipt store is the
mistake available here, and it is not this slice's to repair; the contract
receipt rides the per-invocation declaration, which works.

**Emission.** Beside the fragment and the runbook step, in `design resume`, and
selected the same way. `forward_runbook` (`design.rs:1811`) currently finds the
stage's single outbound forward edge by walking `Stage::ALL` and asking
`can_advance` — a walk whose own doc explains it is re-deriving the table rather
than re-stating it. `sec-3`'s closed `Advance` retires that: the edge is
`Advance::from_stage(stage)`, and both the runbook selector and the contract
block take it. A locked run emits neither, which is the same real answer
`Fragment::for_stage` and `boundary_runbook` already give at that stage — no
outbound edge, nothing to guard, nothing to deliver.

**Which contracts the receipt carries.** Exactly two things, unioned:

> the **enforced set** at the edge — `cumulative_conditions` after both filters —
> **plus** the `Pending` rows on that edge's own boundary, marked not-yet-enforced.

The first half is a reading of `DEC-124`'s *"the contracts for that stage's
outbound edge"*, and it is the wider of the two available: what the edge judges
by, not the edge's own rows alone. The narrow reading would hand an agent
standing at `reviewing` a receipt covering three conditions while the edge it is
about to cross judges six, and the three it omitted are exactly the ones an agent
is least likely to have seen — they were inherited from stages it may never have
stood at.

The second half is what stops that from over-delivering. `Pending` rows are
`Cumulative` too, so a rule of *reach-filtered, activation-annotated* would carry
`governing-context-recorded` and `initial-concerns-recorded` into every later
edge's receipt — eight contracts at `reviewing → locked`, two of them describing
work that will not be asked for there and, once `IMP-391` lands, will already
have been done two stages earlier. Restricting the annotated rows to the current
boundary shows a `Pending` row exactly where an agent can act on it, which is the
edge it guards.

So the counts are: **four** at `drafting → reviewing`, **six** at
`reviewing → locked`, and **two** at `exploring → inquiring` — both of them
`Pending`, which is the honest picture of an edge that today passes on the
runbook alone. Small, bounded by the vocabulary, delivered once.

`DEC-124`'s per-condition **addressing** consequence was argued against the narrow
reading, and it survives over-satisfaction: the refusal cites one condition at a
time whatever the receipt carried, and a future pull verb needs the address
whether or not the receipt happened to include it.

Neither half needs a third accumulator. The enforced set *is*
`cumulative_conditions`, unchanged in signature and behaviour; the pending rows
are `boundary_conditions` for the one edge, filtered to `Pending` — which is what
`sec-3` already committed that function to supporting by returning its rows
unfiltered. The renderer composes two functions it does not reimplement, so
there is no second copy of the reach rule to drift from the gate's.

**Where the prose lives.** `design-prompts/conditions/<token>.md` — the store
`DEC-122` names, in a subdirectory of it.

```rust
impl Condition {
    /// The embedded asset key the shell resolves. One condition, one file;
    /// leaf tier names the key and never reads it, exactly as `Fragment` does.
    pub(crate) fn contract_asset_key(self) -> String {
        format!("{STORE}/conditions/{}.md", self.as_str())
    }
}
```

The subdirectory earns itself. `STORE` already holds two families keyed by two
vocabularies — four `Fragment` stems as `.md`, four `RunbookKey` stems as
`.toml` — and a third family of nine `.md` files keyed by a third vocabulary
would share an extension with the first and avoid collision only by luck. More
usefully, the corpus half of `sec-3`'s set-equality test needs to *enumerate* the
contract assets; under a prefix that is a filter, and flat it is a filter minus a
hand-maintained exclusion list for the four fragment stems, which is the shape
`STD-001` exists to refuse.

**Publication.** Nine `[[entry]]` rows in `publication/manifest.toml`, `kind =
"guidance"`, `customization = "fixed"` — `DEC-122`'s ruling, and the same reason
in the same words the manifest header already gives for the obligation runbooks:
*"not that a project may not disagree with it, but that the override seam does
not exist yet (DEC-102 ruling (b); IMP-372 carries the resolution)."* This is a
precedent being followed, not a policy being set.

It also means the asset corpus is bounded twice by two independent tests, one of
which already exists. `assert_unprojected_install_assets_are_published`
(`asset_source.rs:126-148`) reads both manifests from disk and fails if an
`install/` asset is neither a projection base backing nor published — so a
contract asset that shipped without its entry fails today's suite. `sec-3`'s
set-equality test is the other side: every generated key has an asset, and the
corpus holds no key the vocabulary does not.

**The receipt token is bare, and that is `DEC-124`'s no-digest ruling reaching
the wire.** `--known-contracts <edge>`, repeatable, where `<edge>` is
`Advance`'s kebab name. Plural because one declaration names a whole edge's
block: that *is* what edge-grained delivery means, and a singular flag would
imply a per-condition receipt the design deliberately does not have.

The shape is worth defending, because `Fragment` refuses exactly it. A bare name
is not a fragment receipt — *"a receipt that binds no bytes is not a receipt"*
(`prompt.rs:188-195`) — since fragment bytes move independently of the name. A
contract has no such freedom: `DEC-124` walked every condition and found none
parameterised by the run, and the subsection above shows the one row that came
closest still is not. So there is nothing for a digest to detect within a run's
life.

The residual is a binary upgrade mid-session, which moves both halves of every
contract while a declared receipt still names the edge. It is named rather than
closed: the run has no way to know, the window is a caller that keeps its context
across an upgrade, and the repair if it ever bites is the one `DEC-124` already
left on the shelf — *"if a real contract-change case emerges, `Fragment`'s digest
mechanism is there to borrow."*

**And it is not folded into the fragment receipt**, which was the tempting
economy: one stage, one emission, one token. It fails on the change models. A
fragment binds bytes by digest and a contract deliberately does not, so folding
forces one of them to change — either the fragment loses a guarantee it has, or
the contract acquires machinery `DEC-124` refused. Two tokens on one emission is
the cheaper answer.

**The rendered block.** The field set is the commitment; the punctuation is the
renderer's.

```
contracts drafting-reviewing
contract blocking-inquiries-dispositioned derived engine(dispositions) cumulative active
  discharge: dispose every blocking inquiry on the map
  <narrative asset body>
contract user-accepts-sufficiency attested inquiry-map cumulative active
  discharge: the user accepts sufficiency (SufficiencyAccepted)
  <narrative asset body>
contract materialisation-current derived engine(materialisation) cumulative active
  discharge: materialise the run's declared sections
  <narrative asset body>
contract drafting-readiness-attested attested edge-local active
  discharge: the agent declares DraftingReady
  <narrative asset body>
```

Four, not the edge's own two — the first two are inherited from
`inquiring → drafting`, which is the whole reason the receipt is the enforced set.
No `Pending` row appears here, because both of them sit on `exploring → inquiring`
and this is not that boundary.

Declared held, the header lines still ride and the bodies do not — the same rule
`fragment_section` follows, and for the reason its doc gives: *"a caller that
declared a stale receipt, or lost the bytes it claimed, must still be able to
tell what it is missing."*

### The envelope gets nothing new, and loses one thing

`DEC-124` denies the envelope any contract content, against a hard ceiling with
an uncapped neighbour. That holds — `sec-3`'s currency lamp and severity summary
are derived state that warns, not contract prose, and the lamp is a scalar chosen
against exactly this budget.

Both losses below follow from one retirement that is `sec-4`'s, not this
section's: `Evidence` fails that section's shape rule, so the record, the
`evidence` wire field and its `WRITER_ACTS` member all go. What lands here is
what that costs the two surfaces which read it.

What the envelope loses is that uncapped neighbour. `SectionRow::clearances`
(`envelope.rs:194`) is built by `clearances_for` (`envelope.rs:809-822`) from
`gate.live_evidence()` filtered to the row's subject — which is the claimed arm's
evidence store, and every condition becoming derived leaves it with no source.
The field is **not** rebuilt from the new acts. Of the nine conditions exactly one
is per-section, and `review_outstanding` on the same row already reports it —
repaired by `sec-4` to read the run's policy, which is the repair that makes it
sufficient. A second per-section list would restate one bit beside itself.

So `sec-2`'s envelope-budget concern is partly retired rather than merely
survived: the uncapped list it names as the competitor for bytes goes away, and
what this design adds to the envelope is two scalars. Recorded here because the
concern was stated as live and a reader should not have to infer that it moved.

**The change log's evidence feed re-sources.** `invalidation_rows`
(`run.rs:1520-1536`) diffs the live evidence set across an apply and emits an
`EvidenceInvalidated` row naming the condition and the subject fingerprint. The
row's *meaning* — this clearance stopped being live, here is what it was bound to
— is exactly what a reader of the new model needs, so the feed survives with its
input changed from the evidence set to the act set. Its vocabulary member is
renamed to match, since a name outliving its source is how a reader is misled for
free; the change log is bounded and evicted and the snapshot is gitignored
runtime state, so the rename costs one live run nothing. `live_reviews`' sibling
feed is untouched, on `sec-4`'s stated ground.

### Verification impact

- **A refusal names the missing act and the missing lane** — a run under
  `HumanThenAdversarial` with only a human attestation refuses with
  `SectionsUnreviewed` naming the section and the adversarial lane. The nested
  quantification, asserted at the refusal rather than only at the gate.
- **A conjunction that fails twice says both** — `initial-concerns-recorded` with
  neither act recorded yields two `ActMissing` causes in one `Unmet`, not one.
- **A stale conjunct is a different cause from a missing one** —
  `CoverageStale` where the act exists and the map moved, `ActMissing` where it
  does not, and the two are asserted on the same condition.
- **The diff names the node** — re-parenting one inquiry node yields
  `CoverageStale { moved }` holding that node's id and no other. The property the
  material-over-digest choice was made for.
- **An empty document is `NoSections`, not an empty `SectionsUnreviewed`** — the
  degenerate guard reaching the refusal, so the two are distinguishable by a
  caller and not only by the gate.
- **The remedy is the contract's** — the refusal's remedy text for a condition
  equals the `discharge` line the receipt injects for the same condition.
  Asserted as equality between two renderings of one value, which is what makes
  invariant 4 testable rather than merely stated.
- **A refusal reads no asset** — the gate leg is exercised with no embedded
  corpus available and still renders every remedy. The tier boundary, asserted
  rather than trusted.
- **An unreadable ledger is `ReviewUnavailable`, not `BlockersUndisposed`** — a
  stored `Conducted` act whose `RV` the shell cannot read leaves the row unmet
  naming the review, and names no findings. Both halves, because reporting
  findings nobody read is the failure mode this variant exists to prevent.
- **A superseded pass is named as one** — a run waived at `reviewing`, regressed
  to `drafting` and re-advanced yields `PassSuperseded` carrying both the
  disposed reference and the current one, rather than reading as satisfied.
- **Every `sec-4` admission promise raises `ActAdmissionInvalid`** — one test per
  `ActFault` variant, each asserting the variant and its payload, since the
  section states eight distinct malformations and a single "is refused" assertion
  would pass on the wrong one.
- **An act failing twice reports twice** — a `GovernanceConfirmed` with both a
  mismatched coverage and a missing observed key yields two `ActFault`s in one
  refusal, the same not-just-the-first rule `causes` follows.
- **The receipt covers what the edge judges by** — at `reviewing` the block
  carries six contracts, three of them from edges below, and at `drafting` four,
  two of them from below. Asserted as set equality against
  `cumulative_conditions` for that edge, not as a count, so the test does not
  pass on the right number of wrong rows.
- **A `Pending` row renders marked, on its own boundary and nowhere else** —
  `governing-context-recorded` appears in the `exploring` receipt annotated
  not-yet-enforced, does not appear in the refusal for that edge, and does not
  appear in the `reviewing` receipt at all. The third clause is what distinguishes
  this rule from reach-filtered-and-annotated, which would carry it everywhere.
- **A declared receipt elides bodies and keeps identity** — `--known-contracts
  drafting-reviewing` suppresses every narrative body and no header line.
- **An unknown or mismatched edge token elides nothing** — the bodies ride, on
  the fail-open-to-delivery rule the fragment register already follows.
- **A locked run emits no contract block** — the same real answer as its empty
  fragment and absent runbook, asserted so that `None` is not read as a gap.
- **Every condition has an asset and the corpus has no orphan** — set equality
  over the generated vocabulary and the `design-prompts/conditions/` prefix.
- **Every contract asset is published** — the existing disk-source reachability
  gate, which needs no new test and is named here because it is what makes the
  manifest entries non-optional.
- **The envelope carries no contract prose** — the rendered envelope for a run at
  any stage contains no contract body and no remedy, so the byte-budget ruling is
  asserted rather than assumed.

### Carried forward

- **The narrative half of an inherited condition is unreachable in-session.** An
  agent refused at `reviewing → locked` on a lower edge's condition gets the
  token, the cause and the remedy, and — if it entered at `reviewing` — the
  contract too, because the receipt carries the enforced set. It is a caller that
  entered at an earlier stage and kept its context that can hold an address it
  cannot resolve. The pull verb `DEC-124` names as a phase-plan candidate is the
  close; this slice does not build it.
- **The digest residual is a binary upgrade mid-session**, and the borrowable
  mechanism is `Fragment`'s. Named by `DEC-124`, unchanged by this section.
- **`DesignSnapshot::fragments` is dead state** — one reader, no writer, so
  `fragment_lines` reports every declared fragment as not held. Out of scope, and
  written down because it is the wrong register to build on.
- **Whether the contract corpus ever becomes project-authorable** is `DEC-122`'s
  deferral and `IMP-372`'s seam, and the `fixed` declaration is what keeps this
  design from promising it early. `IDE-047`'s structured extraction reads the
  same const the injection does, so it costs nothing here either.
