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
fn satisfied(condition: Condition, run: &DesignSnapshot, observed: &ObservedFacts) -> bool
```

`ObservedFacts` is not decoration and is derived in the next subsection but one;
the short version is that some conditions bind to canonical state the snapshot
does not contain, and a signature that cannot see it would recreate the
existential scan under a new name.

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

The governing reading, stated here because four later choices follow from it and
were previously asserted without it: **a condition guards a transition, not a
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

```rust
pub(crate) struct Binding {
    /// What the attestation's own recorded content covers.
    pub(crate) coverage: Coverage,
    /// Canonical facts outside the run that must ALSO still match. Conjunctive
    /// with the coverage, and with each other.
    pub(crate) observed: &'static [ObservedFact],
}

pub(crate) enum Coverage {
    /// The attested artefact's own recorded content.
    Artefact,
    /// Every section's current digest — the coverage rule.
    EverySection,
}
```

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
/// canonical state the run does not own.
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

### Reach and force

Two independent axes, and the earlier draft's single `Scope` enum collapsed them.
**Reach** is how far a guard is re-derived; **force** is whether failing it bars
the move.

```rust
pub(crate) enum Reach {
    /// Re-derived at this edge and every edge above it (DEC-067).
    Cumulative,
    /// Evaluated only on the edge that names it.
    EdgeLocal,
}

pub(crate) enum Force {
    /// Failure refuses the move and appears in `GateNotCleared::missing`.
    Blocking,
    /// Failure is evaluated, rendered, and never bars the move.
    Advisory,
}
```

The split is paid for by a real row rather than by symmetry. `DEC-126` makes
integrated-pass currency **derived state that warns rather than blocks** — it
calls this the second non-cumulative element in the machine, on `DEC-101`'s
stale-discharge precedent — and the reason is termination, not taste: dispose a
finding, integrate it, and the sections move, so whole-map currency invalidates
the pass that found it. Blocking on that enforces review-to-fixpoint, which
`RFC-026` E3 refutes and `DEC-126` explicitly rejected. But the fact is still
**cumulative** in reach: it is whole-map currency, re-derived like any other. A
single enum cannot spell cumulative-and-advisory; it forces an advisory row to
surrender its reach.

Three of the four cells are occupied, which is the test the split had to pass:

|                | blocking                | advisory                 |
|----------------|-------------------------|--------------------------|
| **cumulative** | seven rows              | `integrated-pass-current` |
| **edge-local** | `drafting-readiness-attested` | — |

`cumulative_conditions` accumulates the `Cumulative` rows from every edge below
the target and the `EdgeLocal` rows from the final window only — the edge being
crossed. `Advisory` rows are evaluated and reported and never appear in
`GateNotCleared::missing`.

**The discriminator for force is repair cost, not binding.**
`section-attestations-current` and `integrated-pass-current` share the
`EverySection` coverage and take opposite force, which looks contradictory until
the artefacts are compared. Per-section attestations invalidate individually, so
an edit costs re-attesting the sections it touched: bounded, so the row can
block. The integrated pass is one artefact covering everything, so any edit kills
all of it: unbounded, so the row must warn. `Coverage::EverySection` is therefore
doing one job, and the force column carries what looked like its second.

**The severity summary is not a row.** `DEC-126` pairs it with integrated-pass
currency as derived state that warns, and the two have different shapes: currency
is a predicate and can be a guard, whereas an outstanding-findings count by
severity has no satisfied/unsatisfied reading at all. It is envelope and receipt
content — rendered at the boundary to inform the user's decision to run another
round — and it is not in this vocabulary.

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

`DEC-126`, restated only as far as this section's vocabulary needs. Ten in, ten
out: **two Derived, eight Attested, zero Claimed.**

| condition | derivation | coverage | observed | reach | force | activation |
|---|---|---|---|---|---|---|
| `blocking-inquiries-dispositioned` | `Engine(Dispositions)` | — | — | cumulative | blocking | active |
| `materialisation-current` | `Engine(Materialisation)` | — | — | cumulative | blocking | active |
| `governing-context-recorded` | `Attested([{GovernanceConfirmed, User}])` | artefact | `[GovernanceEdges]` | cumulative | blocking | pending `IMP-391` |
| `initial-concerns-recorded` | `Attested([{GraphReviewed, User}, {BlockingSetDeclared, Agent}])` | artefact | — | cumulative | blocking | pending `IMP-391` |
| `user-accepts-sufficiency` | `Attested([{SufficiencyAccepted, User}])` | artefact | — | cumulative | blocking | active |
| `drafting-readiness-attested` | `Attested([{DraftingReady, Agent}])` | artefact | — | edge-local | blocking | active |
| `section-attestations-current` | `Attested([{SectionReviewed, …}])` | every section | — | cumulative | blocking | active |
| `review-disposition-attested` | `Attested([{ReviewDisposed, User}])` | artefact | — | cumulative | blocking | active |
| `integrated-pass-current` | `Attested([{IntegratedPassConducted, Adversarial}])` | every section | — | cumulative | **advisory** | active |
| `user-acceptance-attested` | `Attested([{DesignAccepted, User}])` | every section | — | cumulative | blocking | active |

**Retired:** `required-sections-exist` — no implementation to extend, and a
mandatory section list is craft under `DEC-102`.

**Added:** `drafting-readiness-attested`, which `DEC-126` names explicitly as the
replacement — *"the same shape as `user-accepts-sufficiency` one stage later.
Retiring outright would leave `drafting → reviewing` guarded by
`materialisation-current` alone, which is trivially true of an empty document."*
It is a vocabulary addition, not bookkeeping, and is recorded as one.

**Folded, and split:** `integrated-review-present` and `blocking-findings-disposed`
become `review-disposition-attested`, per `DEC-126`. The *currency* half of
`integrated-review-present` does not fold — it becomes `integrated-pass-current`,
advisory, deriving over the same attestation as `review-disposition-attested`'s
`Conducted` arm with different coverage and different force. Two conditions over
one act is not duplication: they ask different questions, and the model exists to
let them differ. Its derivation is already in the tree as
`ReviewStanding::integrated_current` — *"an integrated adversarial pass covers
current content"* — so the row costs no new machinery, only a reclassification
from blocking to warning.

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

Reach and force are per-row commitments, so each owes an argument rather than a
default.

- `blocking-inquiries-dispositioned`, `materialisation-current` — **cumulative,
  blocking.** Both derive over state that keeps moving after their edge: a
  regression can re-open nodes, and sections change at every later stage. Locking
  a design whose bytes nobody materialised locks prose that was never written.
- `governing-context-recorded` — **cumulative, blocking.** The governance edge
  set is canonical state outside the run and can change at any point; `DEC-121`
  exists precisely so the design is not built on unconfirmed governance.
- `initial-concerns-recorded` — **cumulative, blocking.** The inquiry graph keeps
  moving, and the artefact binds to its own content, so re-derivation is cheap
  and catches a graph re-seeded after the acceptance.
- `user-accepts-sufficiency` — **cumulative, blocking.** The acceptance is over
  the inquiry map; if the map moves, the acceptance no longer covers it.
- `drafting-readiness-attested` — **edge-local, blocking.** It is a judgement
  that drafting may *begin*. Once drafting has happened, re-asserting it at a
  later edge asks a question with no meaning; the content drift it might have
  caught is `materialisation-current`'s job, and that row is cumulative. Blocking
  because `DEC-126` retired the alternative guard on this edge.
- `section-attestations-current` — **cumulative, blocking.** `DEC-066` on section
  digests, and repair is bounded: re-attest what moved.
- `review-disposition-attested` — **cumulative, blocking.** A regression re-opens
  the design, so disposition must be re-earned; it cannot deadlock, because
  `Waived` is always available.
- `integrated-pass-current` — **cumulative, advisory.** Whole-map currency, and
  unbounded repair. Argued above.
- `user-acceptance-attested` — **cumulative, blocking.** `EverySection` coverage
  is the whole point of the row.

### The contract table

```rust
pub(crate) struct Contract {
    pub(crate) derivation: DerivationRule,
    pub(crate) reach: Reach,
    pub(crate) force: Force,
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

So the vocabulary and the const rows come from **one generating source** rather
than being written twice and tested for agreement. A declarative macro takes the
condition list once — variant, kebab token, contract — and emits the `Condition`
enum, `Condition::ALL`, and `CONTRACTS` from it. Two of the three sets are then
equal by construction and cannot be made to disagree, and the third, the prose
asset corpus, is the only one a test has to check, because it lives outside Rust:

- **generated** — `Condition`, `ALL` and `CONTRACTS` from one list;
- **tested** — every generated key has a prose asset, and the corpus has no key
  the vocabulary does not.

This is the same move `STD-001` asks for and the reason `CONTRACTS` is an
enumerable array rather than only a `const fn` match: the array is what the
generator emits and what the asset test iterates.

### Invariants

1. **Totality is two-way generated and one-way tested.** Vocabulary, const rows —
   one source, equal by construction. Prose assets — set-equal by test. A missing
   match arm fails the build.
2. **No Claimed tier.** `ConditionKind` has two variants and is a projection of
   `DerivationRule`, so the defect tier is unrepresentable rather than merely
   unused.
3. **One discharge source.** The discharging act is stated once, in the
   derivation rule. Refusal remedy and rendered prose are both injected from it.
4. **Derivation is uniform.** `satisfied` has no branch on kind.
5. **Reach and force are independent.** Neither is derivable from the other, and
   every row states both. `Advisory` never appears in `GateNotCleared::missing`.
6. **A guard is evaluated on its own edge and, if `Cumulative`, on every edge
   above it.** `EdgeLocal` rows are admitted from the crossing edge only.
7. **Binding is conjunctive.** An attestation covers its own content; every named
   observed fact must also still match.
8. **Observed facts are never persisted.** They are transient input; the run
   stores only the fingerprint an attestation was made over.
9. **`DEC-101` holds.** The closed `Condition` set remains the key; no
   satisfaction is sourced from runbook steps or any other open vocabulary.

### Verification impact

- `satisfied`'s signature change touches every call site. Existing `gate.rs` unit
  tests are the behaviour-preservation proof and stay green where behaviour is
  unchanged.
- The e2e suites encode the claimed path, including tests binding conditions to
  `sec-1`. Those change legitimately; each change is an argued edit, not a
  green-chase.
- **Prose coverage** — every generated condition key has a prose asset and the
  corpus carries no orphan. The vocabulary/`CONTRACTS` equality needs no test: it
  is generated from one source.
- **Wrong actor does not satisfy** — an attestation by the wrong `ActorClass`
  leaves the condition unmet. The property the existential scan could not express.
- **Missing conjunct does not satisfy** — `initial-concerns-recorded` with the
  user's graph review but no agent blocking-set declaration is unmet, *and the
  refusal names which act is missing*.
- **Stale observed fact invalidates** — the governance edge set moving after the
  attestation leaves the condition unmet; an unobservable fact reads as changed.
- **An empty observed set still binds its artefact** — a governance sweep that
  found nothing, whose search evidence is then edited, is unmet. The strict-path
  case, and the reason `Binding` is conjunctive.
- **Advisory never blocks** — `integrated-pass-current` failing is reported and
  absent from `GateNotCleared::missing`, and the move succeeds.
- **Edge-local is not accumulated** — `drafting-readiness-attested` is required
  crossing `drafting → reviewing` and absent from the enforced set crossing
  `reviewing → locked`.
- **`Pending` rows are not enforced** — and are still returned by
  `boundary_conditions` for the renderer.

### Carried forward

- `ActKind`'s full membership is fixed by the sections specifying each act. This
  section fixes that it is closed and that `ActRequirement` pairs it with an actor.
- `ISS-310` is decided in the attested-acts section, with the type cost of each
  candidate answer now stated. It also warns that if this slice ships without
  repairing `review_standing`, the defect survives the slice that named it —
  a scope question for that section.
- `review-disposition-attested`'s `Conducted { review }` arm awaits `IMP-392`
  (`DEC-125`). It is an unbuilt variant, not a pending row.
- Whether `ObservedFact` grows beyond `GovernanceEdges` is left open. One member
  is enough to justify the seam — the alternative is a special case in `satisfied`
  for exactly one condition — but a second member would test whether the
  refresh/compare/absence semantics generalise.
- Research currency is the third instance of the reach question and is **not** in
  this vocabulary. If it ever became a guard it would be advisory on the same
  termination argument, but it lives outside the design run today and settling it
  is outside `SL-244`.

