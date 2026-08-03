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
    /// Who must have authored it. NOT optional.
    pub(crate) actor: ActorClass,
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

**One row's requirement is quantified, and `ActRequirement` cannot hold it.**
`section-attestations-current` is the only row whose coverage is `EverySection`,
and the only row whose required actor is undecided. That is one fact, not two:
its requirement is quantified over sections rather than fixed by its rule, so the
`…` in its classification row below stands for a *rule*, not for an unfilled
`ActorClass`. This matters because `acts` is a conjunction — two `ActRequirement`
entries mean *both*, never *either* — so of `ISS-310`'s three candidate answers
only the first is expressible as drawn:

- **human** — fits `ActRequirement` exactly, as one required actor;
- **either** — needs the actor slot to admit a disjunction, which the conjunctive
  `acts` slice deliberately does not;
- **human plus adversarial where the section opted in** — is not a typing gap at
  all. The requirement varies per section, from run data, so it cannot live in a
  `&'static` table however the actor slot is shaped.

`ActRequirement` stays as drawn, because it is correct for the eight rows that
name one subject and one required actor. What is recorded here is that the open
question carries a type cost that differs by answer, so `ISS-310` is decided in
the attested-acts section with that cost visible rather than discovered.

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
    /// Every section's current digest.
    EverySection,
    /// Every inquiry node's current digest.
    InquiryMap,
}
```

`EverySection` is the incumbent pattern generalised, not a new idea:
`ContentCoverage` already stores the subject→fingerprint map an acceptance was
made over and compares it with the current one (`attestation.rs:200-214`).
`InquiryMap` is the same shape over the other run-owned subject set, and it
exists because two rows need it — see the reach arguments below.

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

- `boundary_conditions(from, to) -> &'static [Condition]` is **unchanged** —
  still `const`, still the full target row set for the edge. It is what the
  `DEC-124` stage-entry renderer reads, which is how `Pending` rows are shown to
  the agent marked not-yet-enforced.
- `cumulative_conditions(to)` — already non-`const` and already `Vec`-returning —
  applies both filters, reach and activation, as it accumulates.

There is no signature change and no `const` to trade away. `boundary_conditions`
has exactly one caller today (`gate.rs`, inside `cumulative_conditions`), and it
is the enforcement path where the filter belongs, so the two sets never had to
share a name.

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
| `section-attestations-current` | `Attested([{SectionReviewed, …}])` | every section | — | cumulative | active |
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
`Artefact`. `AcceptedDesign` already carries `ContentCoverage`, whose
`is_current` compares the covered map against every current section
(`attestation.rs:212`), and `DEC-120` names acceptance-covers-current-content as
the incumbent template. Self-binding would have let a design edit leave the
acceptance apparently current, deleting a guarantee that already ships.

`section-attestations-current`'s required actor is deliberately left as `…`: it
is `ISS-310`'s open decision, and — per the attestation subsection above — it is
a quantified rule rather than a value, so naming it here would be exactly the
buried assumption this section is supposed to refuse.

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
- `section-attestations-current`, `user-acceptance-attested` — **cumulative**,
  on `EverySection` coverage: `DEC-066` over section digests, which is the whole
  point of both rows.
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

That leaves exactly one enumeration outside Rust, and one test:

- **generated** — `Condition`, `ALL`, `CONTRACTS` and `boundary_conditions`, from
  one list. Equal by construction; no test can fail because no disagreement is
  expressible.
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
2. **Every condition guards an edge.** Unrepresentable otherwise: the generator's
   source is keyed by edge.
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
10. **Observed facts are never persisted**, and each defines its own projection
    and encoding. They are transient input; the run stores only the fingerprint an
    attestation was made over.
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
- **Coverage invalidates** — an inquiry-map edit unmakes `user-accepts-sufficiency`;
  a section edit unmakes `user-acceptance-attested`. Both against the incumbent
  `ContentCoverage` behaviour.
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
- `ISS-310` is decided in the attested-acts section, with the type cost of each
  candidate answer now stated. It also warns that if this slice ships without
  repairing `review_standing`, the defect survives the slice that named it —
  a scope question for that section.
- **`review-disposition-attested`'s cumulative reach is not yet enforceable.**
  Binding it to the finding set requires `DEC-125`'s `RV`-backed model
  (`IMP-392`); until then its `Artefact` coverage means only its own record
  invalidates it.
- `review-disposition-attested`'s `Conducted { review }` arm awaits `IMP-392`.
  It is an unbuilt variant, not a pending row.
- Whether `ObservedFact` grows beyond `GovernanceEdges` is left open. One member
  is enough to justify the seam — the alternative is a special case in `satisfied`
  for exactly one condition — but a second member would test whether the
  refresh/compare/absence semantics generalise.
- Research currency is the third instance of the reach question and is **not** in
  this vocabulary. It is a warning-shaped fact by the same convergence argument,
  it lives outside the design run today, and settling it is outside `SL-244`.

