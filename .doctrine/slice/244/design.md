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

```rust
pub(crate) enum Binding {
    /// The attested artefact's own recorded content.
    Artefact,
    /// Every section's current digest — the coverage rule.
    EverySection,
    /// A canonical fact observed OUTSIDE the run, refreshed each evaluation.
    Observed(ObservedFact),
}
```

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
/// binding is `Binding::Observed`. Transient — never persisted, and never a
/// mirror of canonical state the run does not own.
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

### Clearance scope

`DEC-126` makes integrated-pass currency and the outstanding-findings severity
summary **derived state that warns rather than blocks** — it calls this the
second non-cumulative element in the machine, on `DEC-101`'s stale-discharge
precedent. A model with only "cumulative condition" in it cannot express that, so
scope is its own dimension:

```rust
pub(crate) enum Scope {
    /// Re-derived at this edge and every edge above it (DEC-067).
    Cumulative,
    /// Evaluated only on the edge that names it.
    EdgeLocal,
    /// Never blocks a move; rendered as a warning.
    Advisory,
}
```

`cumulative_conditions` stops accumulating every boundary row blindly and
accumulates the `Cumulative` ones. `Advisory` rows are evaluated and reported and
never appear in `GateNotCleared::missing`.

Concretely: `review-disposition-attested` is **blocking** and invalidated by its
own artefact moving. Integrated-pass currency and the severity summary are
**advisory** — they inform the user's decision to run another round rather than
barring the move, which is `RFC-026` E3's termination rule and the reason
`DEC-126` refused review-to-fixpoint.

### Activation: the model this slice can turn on

The classification below is the **target**. Two of its rows cannot be enforced
when this slice lands, and both were stated as interim rather than discovered:

- `DEC-121` — until `IMP-391` builds the checkpoints, `exploring → inquiring`
  passes on the runbook alone;
- `DEC-125`/`DEC-126` — `review-disposition-attested`'s `Conducted { review }`
  arm is unbuildable until `IMP-392` migrates findings onto `RV`.

Rather than a second boundary table to drift against the first, activation is a
column on the one table, and the boundary set is derived by filtering it:

```rust
pub(crate) enum Activation {
    Active,
    /// Specified now, enforced when the named work lands.
    Pending { blocked_on: &'static str },
}

pub(crate) fn boundary_conditions(from: Stage, to: Stage) -> impl Iterator<Item = Condition>
```

One source, no migration step, and the interim gate is a *projection* of the
target rather than a fork of it. When `IMP-391` lands, the change is one column
value and the row it belongs to — and the test that a `Pending` row is absent
from the enforced set turns into its opposite by the same edit.

**Cost, stated rather than absorbed.** The incumbent
`const fn boundary_conditions(from, to) -> &'static [Condition]` is
const-evaluable, and `gate.rs`'s module doc makes a point of it — *"one total,
pure, const-evaluable predicate that owns legality"*. A filter over an activation
column is not const in stable Rust, so this signature gives that up. Three ways
out, and this section does not pick one:

1. **Accept the loss.** The predicate stays pure and total; only compile-time
   evaluability goes. Nothing currently consumes `boundary_conditions` in a const
   context, so the loss is theoretical today.
2. **Two const slices per edge** — target and enforced — generated from one
   source by a macro. Keeps `const`, costs a macro and the thing the single table
   was bought to avoid: two artefacts that can disagree, now merely generated
   rather than hand-written.
3. **Keep `boundary_conditions` const and target-shaped**, and filter activation
   one level up in `advance`. Preserves the incumbent signature exactly, and puts
   the interim rule where the interim behaviour is, at the cost of the boundary
   table no longer being the single answer to *what does this edge require*.

(3) is the current preference on the grounds that it changes least, but it is a
preference and not a decision.

`Pending` rows are still rendered to the agent, marked as not yet enforced. An
agent that reads the contract sees where the design is going; the gate does not
pretend to check it.

### The classification

`DEC-126`, restated only as far as this section's vocabulary needs. Ten in, nine
out: **two Derived, seven Attested, zero Claimed.**

| condition | derivation | binding | scope | activation |
|---|---|---|---|---|
| `blocking-inquiries-dispositioned` | `Engine(Dispositions)` | — | cumulative | active |
| `materialisation-current` | `Engine(Materialisation)` | — | cumulative | active |
| `governing-context-recorded` | `Attested([{GovernanceConfirmed, User}])` | `Observed(GovernanceEdges)` | cumulative | pending `IMP-391` |
| `initial-concerns-recorded` | `Attested([{GraphReviewed, User}, {BlockingSetDeclared, Agent}])` | `Artefact` | cumulative | pending `IMP-391` |
| `user-accepts-sufficiency` | `Attested([{SufficiencyAccepted, User}])` | `Artefact` | cumulative | active |
| `drafting-readiness-attested` | `Attested([{DraftingReady, Agent}])` | `Artefact` | edge-local | active |
| `section-attestations-current` | `Attested([{SectionReviewed, …}])` | `EverySection` | cumulative | active |
| `review-disposition-attested` | `Attested([{ReviewDisposed, User}])` | `Artefact` | cumulative | partly pending `IMP-392` |
| `user-acceptance-attested` | `Attested([{DesignAccepted, User}])` | `Artefact` | cumulative | active |

Retired: `required-sections-exist` — no implementation to extend, and a mandatory
section list is craft under `DEC-102`. Folded: `integrated-review-present` and
`blocking-findings-disposed` become `review-disposition-attested`.

`section-attestations-current`'s required actor is deliberately left as `…`: it
is `ISS-310`'s open question — human, either, or human-plus-adversarial where the
section opted in — and it is a decision rather than a transcription. Naming it
here without deciding it would be exactly the buried assumption this section is
supposed to refuse.

### The contract table

```rust
pub(crate) struct Contract {
    pub(crate) derivation: DerivationRule,
    pub(crate) scope: Scope,
    pub(crate) activation: Activation,
    /// Key of the narrative prose asset — the condition's existing kebab token.
    pub(crate) prose: &'static str,
}

pub(crate) const CONTRACTS: [(Condition, Contract); N];
```

An enumerable **array**, not only a `const fn` match. That distinction is load
bearing: a `const fn` match plus an iteration over `Condition::ALL` proves
nothing about `ALL`'s completeness — a variant could join the enum, be handled by
the exhaustive match, and be absent from `ALL` and from the asset corpus with
every test still green.

`DEC-123` requires set equality over **three** enumerations. So:

1. every `Condition` variant appears in `Condition::ALL` — enforced by the
   existing `widest_condition`-style compile-time walk plus a count assertion;
2. `CONTRACTS`'s key set equals `Condition::ALL`, with no duplicates;
3. the prose asset key set equals `Condition::ALL`.

Three assertions, in the `WRITER_ACTS` template `DEC-101` already established.

### Invariants

1. **Totality is three-way.** Vocabulary, const rows, prose assets — set-equal.
   Missing any one fails a test; a missing match arm fails the build.
2. **No Claimed tier.** `ConditionKind` has two variants and is a projection of
   `DerivationRule`, so the defect tier is unrepresentable rather than merely
   unused.
3. **One discharge source.** The discharging act is stated once, in the
   derivation rule. Refusal remedy and rendered prose are both injected from it.
4. **Derivation is uniform.** `satisfied` has no branch on kind.
5. **Liveness is preserved but no longer uniform.** `DEC-066`/`DEC-067` govern
   `Cumulative` rows unchanged. `EdgeLocal` and `Advisory` are the stated
   exceptions, and `Advisory` never blocks.
6. **Observed facts are never persisted.** They are transient input; the run
   stores only the fingerprint an attestation was made over.
7. **`DEC-101` holds.** The closed `Condition` set remains the key; no
   satisfaction is sourced from runbook steps or any other open vocabulary.

### Verification impact

- `satisfied`'s signature change touches every call site. Existing `gate.rs` unit
  tests are the behaviour-preservation proof and stay green where behaviour is
  unchanged.
- The e2e suites encode the claimed path, including tests binding conditions to
  `sec-1`. Those change legitimately; each change is an argued edit, not a
  green-chase.
- **Three-way set equality** over vocabulary, `CONTRACTS`, and asset keys.
- **Wrong actor does not satisfy** — an attestation by the wrong `ActorClass`
  leaves the condition unmet. The property the existential scan could not express.
- **Missing conjunct does not satisfy** — `initial-concerns-recorded` with the
  user's graph review but no agent blocking-set declaration is unmet, *and the
  refusal names which act is missing*.
- **Stale observed fact invalidates** — the governance edge set moving after the
  attestation leaves the condition unmet; an unobservable fact reads as changed.
- **Advisory never blocks** — an advisory row that does not hold is reported and
  absent from `GateNotCleared::missing`.
- **`Pending` rows are not enforced** — and are still rendered.

### Carried forward

- `ActKind`'s full membership is fixed by the sections specifying each act. This
  section fixes that it is closed and that `ActRequirement` pairs it with an actor.
- `section-attestations-current`'s required actor is `ISS-310`'s open decision.
- `review-disposition-attested`'s `Conducted { review }` arm awaits `IMP-392`
  (`DEC-125`); its activation column carries that.
- Whether `ObservedFact` grows beyond `GovernanceEdges` is left open. One member
  is enough to justify the seam — the alternative is a special case in `satisfied`
  for exactly one condition — but a second member would test whether the
  refresh/compare/absence semantics generalise.

