<!-- doctrine:section sec-1 -->
# 1. Current behaviour and target behaviour

A design run keeps a **change log**: an ordered list of rows, each naming one
material thing a submission did. It is the only sanctioned way to find out what
an `apply` changed — the run's own snapshot is runtime state, and the guardrails
tell agents not to read it. The shell renders every row the run hands back, so
whatever the log reports is what a caller sees.

This slice is about a gap between those two sentences. Some of what a submission
does never reaches the log, so the log's silence means two different things and
the caller cannot tell which.

## Current

Three code paths record an **act** — a durable statement by a person or an agent
that the run then holds as current. They disagree about whether recording is
worth reporting, and nothing states why.

| path | what it records | row emitted today |
|---|---|---|
| `record_declaration` (`run.rs:539`) | an agent declaration — e.g. `blocking-set-declared` | **none** |
| `record_act` (`run.rs:577`), no disposition | a user act at a checkpoint — e.g. `governance-confirmed` | **none** |
| `record_act`, carrying a review disposition | the same, plus how it disposed the current review pass | `ReviewDisposed` |
| run-level `acceptance` (`run.rs:407`) | a `DesignAccepted` checkpoint act, built through the same constructor | `AcceptanceAttested`, run-wide, no terms |

`record_declaration` validates the basis, resolves the fingerprint, admits the
record against its gate rule, hands it to the declaration group and returns
`Ok(())`. Nothing is pushed onto `pending`, so nothing reaches `Applied::rows`,
so the shell's render — the loop that extends its output with every row in
`Applied::rows` at `commands/design.rs:1771` — has nothing to render.
`record_act` is the same story with one exception carved out of it: it returns `Option<Pending>`, and the `Some` arm is
reached only when the act carries a disposition.

So the same operation — *the run now holds this act* — is reported, unreported,
or reported under a name that describes something else, depending on which of
three constructors reached it. The split is not documented anywhere, and the
acceptance arm's row is the odd one out twice over: it is the only one that
fires, and it is run-wide and term-free while every sibling event is
subject-bearing.

There is a fourth path, and it is the one that shows the shape of the defect
most clearly. `invalidation_rows` (`run.rs:1842`) emits `ActInvalidated` when an
act **leaves the live set** — when the `(ActKind, DesignId)` pair it contributed
to `live_acts` (`run.rs:1777`) is absent after the apply. It is **derived** —
computed from a before/after difference rather than pushed by whoever did the
recording — and its doc gives the reason: a new way of moving a fingerprint
cannot forget to report what it killed.

Read that boundary precisely, because the design later depends on it. The pair
leaves the set when the act's covered material moves, which is what the doc means
by ceasing to bind. It does **not** leave the set when an act is displaced by
another of the same kind: `act_id` derives the id from the act (`run.rs:671`), so
a same-kind replacement contributes an identical pair and the difference is empty
in both directions. The doc comment on `live_acts` describes replacement as "a
death worth a row"; that is not what the code does, and the discrepancy is
`ISS-367`, raised from this design run and deliberately left to it (`sec-3`).

The consequence for this slice is narrower than *the log reports deaths but not
births*, and it is worth stating at the true width: the change log reports **some**
acts ceasing to bind, and **no** acts being made. A first act of any kind leaves
no trace at all.

### Why this is a correctness problem and not untidiness

`ISS-355` is the report: a successful `agent_declaration` apply exits 0 and
prints `revision N stage <stage>` and nothing else. `ISS-346` and `ISS-333`
describe a different defect — an unknown payload key is discarded, the revision
still bumps, a receipt is still written, and the command still exits 0.

Those two produce **byte-identical output**. The one observable that could
separate *your submission landed* from *your submission was silently dropped* is
absent exactly where it is needed, and the sanctioned route to check by hand
does not exist. This is not hypothetical: drafting this slice required seven
acts, and confirming each one landed cost a second command every time. The
sharpest instance came at the drafting-to-reviewing crossing, where one
submission carried both a `drafting-ready` declaration and a stage move: the
stage move rendered a row, the act recorded beside it in the same submission
rendered nothing.

## Target

Recording an act becomes a member of the material-change vocabulary. Every path
in the table above reports through one event, `ActRecorded`, whose subject is
the record's own id and whose payload is the single term `act=<kind>` — the
exact mirror of `ActInvalidated`, which already reports the death of a recorded
act by subject with the act kind as its one term.

| path | row after this change |
|---|---|
| `record_declaration` | `ActRecorded` |
| `record_act`, no disposition | `ActRecorded` |
| `record_act`, with a disposition | `ActRecorded`, **and** `ReviewDisposed` for the disposition (`DEC-241`) |
| run-level `acceptance` | `ActRecorded`; `AcceptanceAttested` retires to readable-only |

Every recording is now reported. Emission is explicit at the point of record
rather than derived, because a recording is an *occurrence* and no before/after
difference can see one — re-recording the same value leaves the difference empty
in both directions.

What this does **not** deliver is a symmetric account of an act's life. Births
become complete; deaths do not, because `ActInvalidated` still reports only the
deaths visible in the live-set difference and `ISS-367` is the gap. After this
slice the log says *every act that was made* and *the acts that died of coverage
movement*. Claiming more than that would promise the reader a symmetry the
change does not build.

## The one existing observable this changes

Every row above except one is a row that does not exist today. The exception is
the review-disposing arm, which goes from one row to two: it keeps
`ReviewDisposed` and gains `ActRecorded` beside it.

That is deliberate, and `DEC-241` carries the argument. In short: the two rows
are different claims — *the run now holds this act*, which is true of every act
however it arrived, and *this is how it disposed the current review pass*, which
only the disposing arm has to say. Suppressing the first on that one arm would
put an exception back into the seam `DEC-238` unified, keyed on a field of the
record being stored. Folding the act kind into `ReviewDisposed` instead was
refused on `R5` grounds: that event already carries up to two terms, and a third
would sit level with `WIDEST_PAYLOAD_EVENT`'s exemplar, which neither
compile-time assert would catch.

The cost is a test update wherever an exact row set is asserted for that path.

## What does not change

- **`ActInvalidated` stays derived.** Its set-difference construction is a
  feature, not an oversight: it is what stops a future fingerprint-mover
  forgetting to report what it killed. Nothing here converts it to an explicit
  push, and nothing here widens what it can see (`ISS-367`).
- **The shell.** The render described above already extends its output with every
  row in `Applied::rows`. It needs no change, and it is also one of `SL-251`'s
  design-targets, so it is fenced out of this slice's selectors.
- **The absorption mechanism.** A row saying an act was recorded is not a claim
  that every key in the submission was understood. Unknown-key refusal is
  `ISS-333` / `ISS-346` / `ISS-327` / `ISS-328`, gated on `QUE-219`, and
  `REQ-478` states the bound explicitly so the new row cannot be mistaken for
  coverage it does not provide.
- **Refused writes.** An `apply` abandoned in the pre-write window records
  nothing, and the assertion that pins this —
  `assert_eq!(after.change_log.rows.len(), 0)` at `commands/design.rs:2878` —
  stays true unchanged. This was checked because `R4` flagged the risk of a
  capsule test pinning current behaviour; that assertion is about an abandoned
  write, not about a successful one, so the risk does not land there.

<!-- doctrine:section sec-2 -->
# 2. The change-event vocabulary

The change log's vocabulary is a closed Rust enum, `ChangeEvent`
(`src/design_run/change_log.rs:56`). Closed is load-bearing: the containment
check that proves a rendered row fits its budget enumerates the vocabulary, so
a member nobody sized cannot slip past a hand-picked example, and a compile-time
assert derives the event-name bound from the roster rather than trusting a
constant.

This section adds one member and splits one roster. Both are settled decisions —
`DEC-237` (*`ActRecorded` mirrors `ActInvalidated`*) and `DEC-239` (*readable and
emittable vocabularies are different sets*) — and what follows is the shape they
take in the type, not a re-argument.

## The new member

```rust
/// The run now holds this act.
///
/// The mirror of [`ChangeEvent::ActInvalidated`]: same subject convention, same
/// single `act` term. Emitted explicitly at the point of record rather than
/// derived, because a recording is an occurrence and no before/after difference
/// over `live_acts` can see one (`DEC-238`).
ActRecorded,
```

- **Token**: `act_recorded`, 12 bytes against `DESIGN_EVENT_NAME_BYTES` = 32.
  The current widest is `section_fingerprint_changed` at 27, so the compile-time
  assert is untouched.
- **Subject**: the record's own `DesignId` — `cpa-<act>` or `agd-<act>`,
  whichever group holds it. Not run-wide: the record has a run-local id, which is
  precisely `ChangeRow`'s stated test for carrying a subject.
- **Payload**: exactly one term, `act=<kind>`, built with
  `PayloadTerm::token(PayloadKey::Act, …)`. `PayloadKey::Act` already exists
  (`change_log.rs:293`) and already means *which recorded act this row is about* —
  no key is minted.

```rust
ChangeEvent::ActRecorded => &[(PayloadKey::Act, ValueKind::Token)],
```

One term, not more, and `DEC-237` gives both halves of the reason: the subject id
and the act kind together answer `ISS-355`'s question completely (*did what I
sent land?*), and a fourth term anywhere in this vocabulary would silently
falsify `WIDEST_PAYLOAD_EVENT` (`render/mod.rs:198`), which hardcodes
`StageMoved` as the three-term exemplar and which neither compile-time assert
would catch moving.

### Why the row is redundant-looking but is not

The subject already encodes the kind — `act_id` derives the id from the act
(`run.rs:671`), so `agd-graph_reviewed` and `act=graph_reviewed` say the same
thing twice. That is deliberate and it is inherited, not invented here:
`ActInvalidated` carries the same pair for the same reason. A row is
**self-contained** by contract (`change_log.rs`, module doc) — it renders without
consulting history — and a reader who has to know the id-minting rule in order to
read the kind off a subject is consulting history that happens to live in their
head. The two events are read side by side; making one legible by a rule the
other states as a term would be the worse asymmetry.

## The two rosters

`ChangeEvent::ALL` is replaced by two constants. The distinction is between what
a persisted snapshot may **contain** and what this binary may newly **write**.

```rust
/// Every event a persisted snapshot may contain and the renderer must handle.
pub(crate) const READABLE: [ChangeEvent; 23] = [ … ];

/// Every event this binary may newly write. A strict subset of `READABLE`.
pub(crate) const EMITTABLE: [ChangeEvent; 22] = [ … ];
```

The premise is already written down in this module — `change_log.rs:69-70`, in
the doc for the existing `evidence_invalidated` alias, says *"the vocabulary a
run writes is not the vocabulary it must read."* One `ALL` forced the two to be
equal, which is why `ISS-315` became a live breakage rather than a routine
retirement, and why *retain the variant but stop writing it* was not an available
move.

Each consumer takes the roster that matches the question it asks:

```mermaid
flowchart LR
  R["READABLE (23)"] --> A["compile-time assert<br/>widest name ≤ 32 bytes<br/>change_log.rs:46"]
  R --> B["rendered_payload_fits_its_cap_…<br/>e2e_design_state.rs:1219"]
  R --> D["round-trip + alias guard<br/>(new)"]
  E["EMITTABLE (22)"] --> C["every_material_event_kind_persists_a_change_row<br/>e2e_design_state.rs:1081"]
  E --> F["no fixture row carries a<br/>non-EMITTABLE member (new)"]
  E --> S["compile-time assert<br/>EMITTABLE ⊆ READABLE<br/>change_log.rs (new)"]
  S --> R
  L["LegacyAcceptanceAttested"] -.->|the difference| R
```

*The subset relation is **proved, not asserted at runtime** — a compile-time
assert in the same idiom as the widest-name one it sits beside, which
`change_log.rs:42-46` already documents as "proved rather than asserted". It is
also the only thing in `src/` that reads `EMITTABLE`, since both roster tests
live in a separate compilation unit, so dropping it stops the bin's own test
build. `sec-4` carries the proof and both reasons.*

- **`READABLE`** governs rendering and bounds. A historical row must stay
  renderable within the same budget, so the widest-name assert and the payload-cap
  table both quantify over it.
- **`EMITTABLE`** governs writer coverage — in one direction only, and the
  asymmetry is worth naming because a reader will otherwise assume both.
  `every_material_event_kind_persists_a_change_row` proves every member **is**
  driven. Nothing in the type proves the converse, that nothing outside the
  roster is written: `Pending::about` and `Pending::run_wide` (`run.rs:257`,
  `:266`) take any `ChangeEvent`, so a writer emitting a retired member would
  compile and would leave every roster test green.

  That converse is bought with evidence rather than with types (`RV-360` `F-2`).
  A second check asserts that **no row the `every_event_fixture` ladder produces
  carries a member absent from `EMITTABLE`** — exhaustive over what the writer
  actually does on the path that drives every emittable member, rather than over
  a hand-maintained array. It is weaker than a type-level guarantee and stronger
  than the roster alone — and `sec-4` records it as **defence in depth**, not as
  the discharge of any `REQ-478` criterion. A writer path the ladder never
  traverses stays invisible to it.

## Retiring `AcceptanceAttested`

`AcceptanceAttested` becomes readable-only:

```rust
/// Read-only history. Superseded by [`ChangeEvent::ActRecorded`] once acceptance
/// flows through the shared record seam; retained because the change log is
/// append-only history and `ChangeEvent` deserialises strictly.
LegacyAcceptanceAttested,   // payload_terms() => &[]

// …and in `as_str`, which after this section is the token's only source:
ChangeEvent::LegacyAcceptanceAttested => "acceptance_attested",
```

Three things are unchanged on purpose: the wire token, the rendered token — the
same string, now spelled once — and the stored payload shape (run-wide,
term-free). Only the **Rust** name moves, and
that is the point — the type warns every future construction site that the
variant is history. `DEC-239` refused both alternatives on the record: a bare
serde alias renames the variant but not the row, so old run-wide term-free rows
would render as `act_recorded` with an empty payload and contradict the
self-contained contract; whole-row normalisation at deserialise would let the
reader manufacture a subject and an `act` term the writer never stored, and the
next snapshot write would persist the manufactured history as though it had
always had that shape.

### The retirement forces the wire token to get one source

Every member's wire token has **two** sources today, and they agree by
convention rather than by construction. `as_str` hand-writes one;
`#[serde(rename_all = "snake_case")]` derives a second from the Rust identifier.
Nothing holds them together — rename a variant and serde's token moves while
`as_str`'s stays, and both halves still compile. The retirement does not create
that defect. It is the first change to make an instance of it *visible*, because
`LegacyAcceptanceAttested` derives `legacy_acceptance_attested` against an
`as_str` still spelling `acceptance_attested`.

Pinning the wire name with `#[serde(rename = "acceptance_attested")]` would put
that literal beside `as_str`'s and settle for the same convention one degree
more explicitly. `STD-001` is **required** and asks that a recurring meaningful
token be named once and referenced everywhere, and a design section is not a
surface that can grant an exception to a required standard. A drift guard
detects divergence; it does not single-source anything (`RV-360` `F-4`).

The module already owns the answer, three times over: `DesignId` is
`#[serde(try_from = "String", into = "String")]` (`ids.rs:131`), `IntentSubject`
the same (`attestation.rs:935`), and `PayloadTerm` carries
`#[serde(try_from = "PayloadTermWire")]` in this very file (`change_log.rs:387`).
`Refusal`'s own `Display` doc names serde's `try_from` as the boundary it exists
to cross (`refusal.rs:457-460`), so this is the seam the module was built for.

```rust
#[serde(try_from = "String", into = "String")]
pub(crate) enum ChangeEvent { … }

impl From<ChangeEvent> for String {
    fn from(event: ChangeEvent) -> String { event.as_str().to_owned() }
}

impl TryFrom<String> for ChangeEvent {
    type Error = Refusal;
    fn try_from(raw: String) -> Result<ChangeEvent, Refusal> {
        ChangeEvent::READABLE
            .into_iter()
            .find(|event| event.as_str() == raw)
            .or_else(|| (raw == LEGACY_ACT_INVALIDATED).then_some(ChangeEvent::ActInvalidated))
            .ok_or_else(|| Refusal::UnknownChangeEvent { raw })
    }
}
```

`as_str` becomes the single spelling of every token, and `rename_all`, `alias`
and `rename` all leave the enum. Three consequences worth stating.

- **Strictness is preserved, not relaxed.** An unmatched token is a `Refusal`,
  exactly as the derive refused it. `ISS-315`'s defect class depends on that,
  and `sec-4`'s legacy-fragment test pins it.
- **One legacy literal remains**, `LEGACY_ACT_INVALIDATED` — the
  `evidence_invalidated` alias. It spells a *retired* token, so it has no live
  `as_str` counterpart to drift from, and it is named once, which is what
  `STD-001` asks.
- **`Refusal` gains a variant**, so `src/design_run/refusal.rs` joins this
  slice's selectors. That is a fence widening, declared here rather than
  discovered at execution — `sec-4` records it in the code-impact table.

`sec-4`'s round-trip check survives with a different job. It no longer guards
drift between two spellings, because there is only one; it guards the alias arm
and `TryFrom`'s refusal of an unknown token, which are what can still regress.

**Migration surface, measured** — `event = "acceptance_attested"` across
`.doctrine/state/slice/*/design.toml`: 7 rows over 6 live snapshots (243 carries
two; 244, 248, 249, 251 and 254 one each). The predicate is given so the
measurement can be re-run rather than trusted; live runs accrue rows, so it is a
floor rather than a constant. The `evidence_invalidated`
legacy that `ISS-315` broke on is 8 rows — the same order, and it is the worked
precedent for how this repo carries a retired token.

**Not adopted:** full type enforcement — an `EmittableEvent` wrapper or a second
enum making legacy emission unrepresentable. `Pending` still accepts any
`ChangeEvent`. `DEC-239` weighed this and judged a near-duplicate enum to cost
more synchronisation machinery than this vocabulary warrants; that judgement
stands, and the residual it leaves is the one named above and covered by
evidence rather than dissolved.

## What is not unified

`ActRecorded` does **not** absorb `ReviewAttested`. These are two lifecycle
families, and the split is by replacement key rather than by taste:

| family | replacement key | recorded | invalidated |
|---|---|---|---|
| act records (`CheckpointAct`, `AgentDeclaration`) | **act kind** — one live answer per kind | `ActRecorded` | `ActInvalidated` |
| section attestations | **attestation id** — reviewer lanes coexist | `ReviewAttested` | `ReviewInvalidated` |

`CheckpointActGroup::record` (`snapshot.rs:361`) retains-then-pushes keyed on
`ActKind`; `AgentDeclarationGroup::record` (`:387`) on the act discriminant;
attestations key on id, and that module's own doc calls the difference "the whole
design". The inverse events already encode the split — `ActInvalidated` carries
`act`, `ReviewInvalidated` carries `section` + `attestation` — so unifying the
forward events would leave the vocabulary disagreeing with itself about how many
families there are.

<!-- doctrine:section sec-3 -->
# 3. The emit seam

Three constructors record an act and they give three answers about whether that
is worth reporting. The repair is not to teach each of them to emit a row — that
is three places a fourth constructor could fail to join. It is to make storing an
act and reporting it one operation with one production route, so the split cannot
re-form by a caller choosing differently. This is `DEC-238`.

## Why the row cannot be derived

`ActInvalidated` is not emitted by whoever changed the content. It is computed by
`invalidation_rows` (`run.rs:1842`) from a before/after difference over
`live_acts` (`run.rs:1777`), and the doc gives the reason: a new way of moving a
fingerprint cannot forget to report what it killed. That set already chains
**both** record kinds, so a reverse difference looks like a zero-signature-change
fix.

It cannot work. For a slot already holding `v`, *no operation* and *record the
same value* both read `before = v, after = v`, so no function of the pair
separates them. `DEC-238` carries the full argument, including why adding the
fingerprint to the tuple only detects unequal replacement and why an occurrence
identity in the tuple would turn the operation into state. The general statement
is the one to carry forward: **a derived row can only report changes in the key
of the set it differences.**

There is a live consequence of that in the current code, and it is captured
rather than repaired here. Because `act_id` derives the id from the act kind
(`run.rs:671`), a same-kind replacement leaves `(ActKind, DesignId)` identical,
so `ActInvalidated` never fires for it — even though `CheckpointActGroup::record`
retains-then-pushes and the act it displaced is genuinely dead. That is
**`ISS-367`**, authored from this run, `concerns DEC-238`, sequenced `after
SL-256`. This slice does not widen to it: `ActRecorded` reports the recording,
which is `ISS-355`'s complaint, and under-reported invalidation is a separate
defect on the other end of the lifecycle. `sec-1` states the boundary in the same
terms, so the design does not promise a symmetry `ISS-367` withholds.

## Where the row is built

**Not on `RecordedAct`** (`attestation.rs:796`). That sum's third arm, `Section`,
carries no record at all — its doc says so, and "the absence is the finding" — and
every accessor on it resolves from the shape without reading a field. Giving it
`id() -> Option<&DesignId>` would put a question one arm cannot answer into an
abstraction whose whole value is that its accessors are total.

So: a second, narrower sum over the two kinds that have an address.

```rust
/// One act record that can be stored and named in a change row.
enum ActRecord {
    Checkpoint(CheckpointAct),
    Agent(AgentDeclaration),
}

impl ActRecord {
    /// The borrowed view admission and the gate interrogate.
    fn admission_view(&self) -> RecordedAct<'_>;
    fn id(&self) -> &DesignId;
    fn kind(&self) -> ActKind;
    /// Into whichever group holds this shape.
    fn insert(self, next: &mut DesignSnapshot);
}
```

This is not parallel modelling, and the test for that is whether the two sums
answer the same question. They do not. `RecordedAct` asks *can admission and the
gate interrogate this act?* — borrowed, three arms, total accessors. `ActRecord`
asks *can this concrete record be stored and named in a row?* — owned, two arms,
addressable. The `Section` arm is exactly the case that separates them.

**It is sited in `run.rs`, and `attestation.rs` was never available.** Cohesion
points here on its own: `ActRecord` exists for the writer, and the writer's other
pieces — `Pending`, `act_id`, `record_declaration`, `record_act` — are all in
`run.rs`, while `attestation.rs` holds the record types and the reader's view of
them. But the boundary is harder than a preference. `src/design_run/attestation.rs`
is named in this slice's Non-Goals as one of `SL-251`'s design-targets,
deliberately out of bounds; it is not merely absent from the selector list. Had
cohesion pointed the other way, this would have been a scope question rather than
a siting question. It does not, so the two agree.

## The seam

```rust
fn admit_and_record(
    next: &mut DesignSnapshot,
    record: ActRecord,
    rule: Option<ActRule>,
    derived: &DerivedInput,
) -> Result<Pending, Refusal>
```

It admits through `admission_view()`, inserts into the right group, and returns
the row — mandatory, not `Option<Pending>`. What that buys, at the strength the
code supports:

- The three production recording paths collapse to one. The asymmetry this slice
  exists to delete cannot re-form by a caller choosing differently, because there
  is one place left that chooses.
- A record kind added **through the seam** cannot omit its row. The return type
  is `Pending`, so there is no arm in which emission is optional.

What it does **not** buy is unbypassability, and an earlier draft of this section
claimed otherwise (`RV-360` `F-1`). `CheckpointActGroup::record` (`snapshot.rs:361`)
and `AgentDeclarationGroup::record` (`:387`) are `pub(crate)`, so the storage
sinks stay reachable from anywhere in the `design_run` tree. Wrapping them in
`ActRecord::insert` stops *using* the direct route; it does not close it. A future
production caller could still store an act without producing a row.

Closing it properly would mean restricting those methods' visibility, which
breaks 13 existing direct call sites: three in `fixture.rs` (`:211`, `:212`,
`:242`) and ten in `tests.rs` (`:1094`, `:1099`, `:1104`, `:1127`, `:1132`,
`:1141`–`:1143`, `:2443`, `:2842`). The first file is outside this slice's
selectors; the second is one of `SL-251`'s design-targets.
So the invariant is weakened rather than enforced, `DEC-238` carries an appended
correction saying so, and what remains is a convention held by there being one
obvious route rather than a guarantee held by the type system.

`admit_against` (`run.rs:652`) is **deleted**, not left beside it. It has exactly
two callers (`:565`, `:619`) and both become `admit_and_record`, so keeping it
would leave two ways to admit a record with only one of them emitting — which is
the defect this section is closing, one layer down.

## The two record paths

```rust
fn record_declaration(
    next: &mut DesignSnapshot,
    declared: &AgentActDeclaration,
    derived: &DerivedInput,
) -> Result<Pending, Refusal>

fn record_act(
    next: &mut DesignSnapshot,
    declared: &CheckpointActDeclaration,
    derived: &DerivedInput,
    payload_digest: &str,
) -> Result<Vec<Pending>, Refusal>
```

Two changes worth naming.

**`record_declaration` takes the declaration, not the whole `ApplyRequest`, and
returns `Pending` rather than `()`.** Its caller hoists
`request.agent_declaration.as_ref()`, which it is the only reader of. The current
early `return Ok(())` for an absent declaration is `ISS-355` in miniature —
*nothing was supplied* and *something happened* sharing one return value — and
removing it is what makes the mandatory row expressible in the type.

**`record_act` returns `Vec<Pending>`, one or two rows.** One on every path;
two on the arm that carries a review disposition, where `ActRecorded` and
`ReviewDisposed` sit side by side (`DEC-241`). The order within the vector is
`ActRecorded` first, then `ReviewDisposed` (`DEC-238`): the recording is what
makes the disposition addressable.

The order is contractual rather than incidental, and the mechanism is worth
naming because it is not visible from the signature. `apply` assigns
`ChangeRow.index` from `pending.into_iter().enumerate()` (`run.rs:494-504`), so
the vector's order **is** the stored order — and
`within_revision_index_is_candidate_order_not_submission_order`
(`e2e_design_state.rs:786-815`) already pins that mapping with an exact ordered
`(index, subject)` vector.

**Construction order runs opposite to vector order, deliberately.** The optional
disposition row is built **before** `admit_and_record` is called, so a
row-construction failure precedes mutation — while the mandatory `ActRecorded`
row, which does not exist until `admit_and_record` returns, has to be placed
**first**. An implementor who lets the vector follow construction order emits the
pair backwards; `sec-4`'s check 3 asserts the ordered pair and is what catches it
(`RV-360` `F-10`).

## The call sites

Three, all in `apply` (`run.rs:380-410`).

```rust
// before
record_declaration(&mut next, request, derived)?;
if let Some(declared) = request.checkpoint_act.as_ref() {
    pending.extend(record_act(&mut next, declared, derived, payload_digest)?);
}
if let Some(declared) = request.acceptance.as_ref() {
    record_act(&mut next, &CheckpointActDeclaration { … }, derived, payload_digest)?;
    pending.push(Pending::run_wide(ChangeEvent::AcceptanceAttested, Vec::new()));
}

// after
if let Some(declared) = request.agent_declaration.as_ref() {
    pending.push(record_declaration(&mut next, declared, derived)?);
}
if let Some(declared) = request.checkpoint_act.as_ref() {
    pending.extend(record_act(&mut next, declared, derived, payload_digest)?);
}
if let Some(declared) = request.acceptance.as_ref() {
    pending.extend(record_act(&mut next, &CheckpointActDeclaration { … }, derived, payload_digest)?);
}
```

The acceptance arm is where three things land at once. Its discarded return value
becomes an `extend` — the comment justifying the discard ("carries no
disposition, so it owes no `review_disposed` row") described a return type that
no longer means that. The explicit `AcceptanceAttested` push is deleted, and the
`DesignAccepted` act now reports through the same `ActRecorded` row as every
other act, subject `cpa-design_accepted`. That is what makes `AcceptanceAttested`
redundant and therefore retirable (`sec-2`, `DEC-239`), and it is also the arm
whose row goes from run-wide and term-free to subject-bearing — a change in
rendered output that `sec-1` counts and the verification section pins.

**The build order does not move.** The declaration still runs before the act, so
an act confirming a declaration is given the fingerprint of the record the engine
just wrote, and `DEC-121`'s *the agent declares and the user confirms* still
arrives in one submission.

```mermaid
sequenceDiagram
    participant A as apply
    participant D as record_declaration
    participant K as record_act
    participant S as admit_and_record
    participant N as next: DesignSnapshot
    participant P as pending: Vec<Pending>

    A->>D: agent_declaration
    D->>S: ActRecord::Agent(record), rule
    S->>N: admit, then insert into declarations
    S-->>D: Pending(ActRecorded, agd-<kind>, act=<kind>)
    D-->>P: 1 row

    A->>K: checkpoint_act
    K->>S: ActRecord::Checkpoint(record), rule
    S->>N: admit, then insert into acts
    S-->>K: Pending(ActRecorded, cpa-<kind>, act=<kind>)
    K-->>P: 1 row, + ReviewDisposed if the act disposed the pass

    A->>K: acceptance → CheckpointActDeclaration{DesignAccepted}
    K->>S: same seam
    S-->>K: Pending(ActRecorded, cpa-design_accepted, act=design_accepted)
    K-->>P: 1 row

    A->>A: invalidation_rows(before, after) — unchanged, still derived
    A-->>P: ActInvalidated / ReviewInvalidated for what died
```

*The diagram shows the production funnel: every arrow that reaches `next` on a
recording path passes through `admit_and_record`, and the only row-producing path
that does not is the derived one at the bottom. It is a picture of what the code
does, not of what the module prevents — the sinks `S` writes to remain callable
directly, as above.*

## Invariants this must not disturb

- **Admission still precedes storage.** `admit_and_record` admits before it
  inserts; a refused record leaves the snapshot untouched and returns `Refusal`,
  exactly as the two `admit_against` call sites do today.
- **A refused or abandoned write emits nothing.** Unchanged, and pinned by an
  assertion that stays true unchanged — see `sec-1`.
- **`ActInvalidated` stays derived**, computed after all three record paths have
  run, so an act displaced during this apply is still differenced against the
  snapshot the apply started from.
- **`Pending` keeps its private fields and its two constructors.** Nothing here
  needs a third; `ActRecorded` is a subject-bearing row and takes
  `Pending::about`.

<!-- doctrine:section sec-4 -->
# 4. Verification and code impact

## Code impact

| path | intended change |
|---|---|
| `src/design_run/change_log.rs` | `ActRecorded` and `LegacyAcceptanceAttested` variants; `ALL` → `READABLE` + `EMITTABLE`; `as_str` and `payload_terms` arms for both; the compile-time widest-name assert re-quantified over `READABLE`; a `const fn` `is_subset` and the compile-time assert that `EMITTABLE ⊆ READABLE`; `#[serde(try_from = "String", into = "String")]` with its `From`/`TryFrom` impls, replacing `rename_all` / `alias` / `rename` (`sec-2`); and one doc correction — `:69-70` attributes the eight legacy `evidence_invalidated` rows to the `SL-243`/`SL-244` runs, and all eight are `SL-244`'s |
| `src/design_run/run.rs` | `ActRecord` sum; `admit_and_record`; `admit_against` deleted; `record_declaration` and `record_act` signatures; the three call sites in `apply`; the `AcceptanceAttested` push deleted |
| `src/design_run/bounds.rs` | one doc reference to `ChangeEvent::ALL` (`:49`) follows the rename to `READABLE` |
| `src/design_run/refusal.rs` | one new variant, `UnknownChangeEvent { raw }`, for `ChangeEvent`'s `TryFrom`. **A new selector** — the fence widens by one file, declared here rather than discovered at execution (`sec-2`) |
| `src/design_run/snapshot.rs` | test module only — a legacy-fragment pin for the retired token |
| `src/design_run/render/mod.rs`, `render/change_row.rs` | expected untouched. `change_row.rs` renders from `row.terms` and names no `ChangeEvent` at all; `mod.rs` names exactly one, `WIDEST_PAYLOAD_EVENT` (`:198`), whose exemplar this slice does not move. In the fence so a change to either reads as a conformance signal rather than a silent edit |
| `tests/e2e_design_state.rs` | the two roster enumerations re-pointed; one fixture comment corrected; seven new checks |

**The `render` selector narrows.** This slice currently fences
`src/design_run/render/**`, which covers `render/envelope.rs` — a file the
Non-Goals separately declare out of bounds as one of `SL-251`'s design-targets,
and one that slice has now changed. The glob and the Non-Goal contradict each
other, and the glob is what `doctrine slice conformance` reads. It is replaced by
the two files named above, both untouched by `SL-251`'s landed capsule.

**Two files inside the blast radius but outside the fence, both checked.**
`src/design_run/tests.rs` and `src/design_run/fixture.rs` call the storage sinks
directly (`sec-3`), so a reader will ask whether they move. They do not: neither
references `ChangeEvent::ALL`, `admit_against`, or any event this slice renames.
`tests.rs` is one of `SL-251`'s design-targets, so if that answer ever changes it
is a scope question, not an edit.

## What is verified, and how

Everything below is `VT`. The behaviour is a rendered row on a command's output
and a persisted row in the change log — both machine-observable, neither needing
an agent's judgement.

### New — the defect, pinned

Written in the idiom of `a_waived_disposition_row_names_its_arm_and_carries_its_reason`
(`tests/e2e_design_state.rs:1110`): find the row by event, then assert its subject
and its ordered terms. Asserting all three matters, because each alone is a fact
the snapshot already held.

1. **`an_agent_declaration_alone_renders_a_change_row`** — `ISS-355`'s exact
   report. An apply carrying only `agent_declaration` yields a row with event
   `ActRecorded`, subject `agd-<kind>`, terms `[<kind>]`. This is the red.
2. **`a_checkpoint_act_without_a_disposition_renders_a_change_row`** — the
   sibling `record_act` path, subject `cpa-<kind>`. Named separately because
   `ISS-355` reported one of two and the second was found in scoping.
3. **`a_disposing_act_renders_both_its_recording_and_its_disposition`** — two
   rows for one act (`DEC-241`), `ActRecorded` before `ReviewDisposed`
   (`DEC-238`), asserted as an ordered pair rather than as set membership. The
   order is the contract, and `sec-3` names why an implementor can invert it:
   construction order runs opposite to vector order. This check is what catches
   that.
4. **`an_acceptance_reports_through_the_shared_act_row`** — the run-level
   `acceptance` field yields `ActRecorded` with subject `cpa-design_accepted` and
   term `act=design_accepted`, and **no** `acceptance_attested` row. This is the
   one test that would catch the retirement being half-done.

### New — the roster invariants

5. **`the_retired_event_is_readable_but_not_emittable`** —
   `LegacyAcceptanceAttested` is absent from `EMITTABLE` and present in
   `READABLE`. The subset half of the invariant is not a test — it is proved at
   compile time, below.
6. **`no_row_the_ladder_produces_carries_a_non_emittable_event`** — every row in
   `every_event_fixture`'s change log has an event in `EMITTABLE`. This is the
   converse of check 5 and a **partial** answer to `RV-360` `F-2`: the roster
   array cannot prove that nothing outside it is written, because `Pending`'s
   constructors take any `ChangeEvent`, so what evidence there is has to come
   from what the writer actually emitted on a path that drives every emittable
   member. It fails if a retired member is still being pushed anywhere the ladder
   reaches, and says nothing about a writer path the ladder does not reach.
7. **`every_readable_event_token_round_trips_through_serde`** — for each
   `READABLE` member, `TryFrom(Into(event))` returns that member; and the
   retired `evidence_invalidated` alias resolves to `ActInvalidated`. After
   `sec-2`'s change the token has one source, so the round trip is true by
   construction and this is **not** a drift guard. What it guards is what
   construction does not give for free: the alias arm, and `TryFrom` refusing a
   token no member spells (`RV-360` `F-4`). No round-trip guard exists for this
   enum today.

### New — the retired token still parses

**`a_snapshot_written_before_the_acceptance_attested_retirement_still_parses`**,
in `snapshot.rs`'s test module, mirroring
`a_snapshot_written_before_the_act_invalidated_rename_still_parses` (`:745`) —
the same defect class, the same whole-file tier, and the precedent this repo
already set for carrying a retired token. A snapshot fragment carrying a run-wide,
term-free `acceptance_attested` row parses, and the row keeps that shape rather
than acquiring a manufactured subject.

Pinned at the whole-file tier for the reason that doc already gives:
`ChangeEvent` deserialises strictly, so one unrecognised `event` fails the entire
snapshot rather than one row. That is `ISS-315`'s defect class, and 7 rows across
6 live runs are the population at stake.

### Amended — the two roster enumerations and one comment

- **`every_material_event_kind_persists_a_change_row`** (`:1081`) iterates
  `EMITTABLE`. It is the check that makes an unwired member fail loudly rather
  than be quietly absent, so it must not enumerate a member no writer may
  produce.
- **`rendered_payload_fits_its_cap_for_every_event_kind`** (`:1219`) iterates
  `READABLE`, so the retired member's saturated payload stays inside the budget.
  This is the real guard on `R5` (`WIDEST_PAYLOAD_EVENT` is a hardcoded exemplar
  that neither compile-time assert would catch moving), and `ActRecorded`'s single
  term keeps well clear of it.
- **The `every_event_fixture` ladder needs no new input, but its narration is
  wrong after this change.** The fixture's payloads already record acts, so
  `ActRecorded` is driven for free. The comment at `:937-943` names the review
  vocabulary as `finding_raised, finding_disposed, acceptance_attested`, and
  retiring the third makes that false — and `ActRecorded` arrives from this
  block's own run-level `acceptance` (`:950-955`) as well as from act submissions
  earlier and later in the ladder. The behavioural fixture is sufficient; the
  load-bearing narrative is not, and correcting it is part of this slice's
  test-file change (`RV-360` `F-6`).

### Not a test

Two facts are proved at compile time rather than asserted at runtime:

```rust
const _: () = assert!(widest(&ChangeEvent::READABLE) <= DESIGN_EVENT_NAME_BYTES);
const _: () = assert!(is_subset(&ChangeEvent::EMITTABLE, &ChangeEvent::READABLE));
```

The first is the event-name bound. `act_recorded` is 12 bytes against a 32-byte
bound whose current widest is 27, so it cannot fail — but it is the discharge of
`REQ-437` (SPEC-029 `NF-002`) and the rename must carry it across rather than
drop it.

The second is `sec-2`'s subset relation, which the split would otherwise leave to
a runtime assertion. `is_subset` is a `const fn` in the module's existing
slice-recursion idiom (`widest`, `:31-40`), comparing members by `as_str()` —
already the event's identity everywhere else (STD-001). Two shorter spellings are
closed off by the workspace lint gate and are named so nobody re-derives them:
comparing discriminants trips `clippy::as_conversions`, and deriving `READABLE`
from `EMITTABLE` by a const-block copy trips `clippy::indexing_slicing`.

**The second assert is also load-bearing against the lint gate**, and that is the
reason it must not be dropped later as redundant. After the split it is the only
thing in `src/` that reads `EMITTABLE`: both roster tests live in
`tests/e2e_design_state.rs`, a separate compilation unit, because the crate is
binary-only and that file `#[path]`-includes the module. The module's dead-code
exemption is `not(test)`-scoped (`mod.rs:68`) and the crate denies `unused`
(`Cargo.toml:224`), so with no `src/` reader `cargo check` passes and
`cargo test --bin doctrine` fails to compile. No attribute substitutes for the
assert: `cfg(test)` holds in **both** compilation units, so
`cfg_attr(test, expect(dead_code, …))` is unfulfilled in the e2e unit and stops
*that* build instead, and `cfg_attr(test, allow(dead_code, …))` escapes
`clippy::allow_attributes` (`Cargo.toml:252`) only because `just gate` runs
clippy without `--all-targets` (`justfile:67`).

Neither assert says anything about what a writer may emit, so `DEC-239`'s ruling
against type enforcement at the construction seam, and `RV-360` `F-2`'s residual
as `sec-2` and the criterion table state it, both stand exactly as written.

## Why nothing else in the suite moves

`DEC-241` says any test asserting an exact row set for the disposing path moves.
The suite was checked rather than assumed, and it takes three arguments rather
than one generalisation. Two earlier drafts each offered a single absolute and
each was false (`RV-360` `F-7`, `F-8`), so nothing below is stated as a rule
where a per-fixture fact is what carries it.

### Stored-row assertions

**Most select by event before asserting**, so a row of a different event is
invisible to them — including the disposing arm's own assertions
(`tests/e2e_design_review.rs:1415`, and the `disposition_rows` helper at
`:1425`), which filter on `ChangeEvent::ReviewDisposed`.

**Three do not, and they are named rather than covered by the generalisation.**
`change_log_floor_is_recorded_not_inferred` (`:708-725`) and
`retention_evicts_oldest_revisions_and_advances_the_floor` (`:1150-1165`) scan
every row's revision; `within_revision_index_is_candidate_order_not_submission_order`
(`:786-815`) maps every row without filtering. None of their fixtures records an
act, so no new row enters their scan. An act reaches a fixture only through
`agent_declaration`, `checkpoint_act`, or the run-level `acceptance` — the three
writer-act keys (`submission.rs:765-784`) — and none of those three payloads
carries one. That is a fixture-level fact, and it is what an implementor must
re-check if any of those fixtures is later given an act to record.

### Rendered-delta assertions

**An added row is substitutive, not additive**, so `contains`-versus-equality is
not the property that protects anything here. `ENVELOPE_CHANGE_ROWS` is 10
(`render/mod.rs:78`) and `changes` cuts by `(revision, index)` descending
(`render/envelope.rs:1073-1080`), keeping the **newest** rows — so a new
`ActRecorded` row can push the row an assertion wants out of the window. Each
assertion over rendered delta content is therefore named with its own reason:

| assertion | why it is stable |
|---|---|
| `change_log_floor_is_recorded_not_inferred` (`:730-735`) | wants `inq-late`, the newest row; fixture records no act |
| `distinct_ids_sharing_a_long_prefix_render_distinguishably` (`:1291-1313`) | two rows in the whole log, no acts; its second half reads a one-revision delta |
| `elided_reason_carries_an_explicit_marker` (`:1253-1269`) | runs on `every_event_fixture`, which **does** record acts — stable only because the stage-move row it wants is the newest |
| `normal_envelope_stays_within_named_limits_on_a_large_run` (`e2e_design_projection.rs:301`, `:312`) | asserts cap saturation and a byte ceiling, never row identity. `large_run` records an act (`:237`); both hold under more rows because the eviction ladder, not the headroom, is what makes the ceiling true (`render/mod.rs:126-134`) |
| `locked_run_emits_no_contract_block` (`e2e_design_review.rs:734`, `:747`) | reads contract-block lines, not delta rows |

### Two further checks, both clean

- **No test indexes rows positionally.** There is no `rows[0]` or `since(0)[n]`
  anywhere in the design e2e suite.
- **The one exact row-count assertion is about an abandoned write** and stays
  true unchanged (`sec-1`).

So the fence holds and no test file outside it needs an edit. The residual is
stated rather than assumed away: if the suite disagrees at execution, the remedy
is a selector widening recorded at plan time and a conformance re-run, not a
quiet edit outside the fence.

## Conformance and coordination

- `doctrine slice conformance` reports no edit outside the fenced surface — in
  particular none inside `SL-251`'s design-targets, of which
  `src/commands/design.rs` is one. This design needs no change there: the shell
  already extends its output with every row in `Applied::rows`, which is why the
  repair is entirely engine-side.
- **`SL-251` has landed, and it is on `edge`.** `eca2c9a11` is an ancestor of the
  branch this design is read against, so there is no second tree to reconcile and
  every source anchor here is an `edge` anchor. An earlier revision carried
  `commands/design.rs` line numbers for both trees; that device is retired rather
  than updated (`RV-360` `F-9`). Its diff was checked against this slice's
  selectors rather than argued from the file list `R4` anticipated: it touches ten
  files under `src/`, **none** of which is one of this slice's selectors, and no
  file under `tests/` at all. The only intersection was the `render/**` glob,
  narrowed above.
- **The `SL-251` coordination note has a third site, in shipped source.** The
  scope tracks that slice's `design.md` ¶ at 422–428 and its ledger row at 2289.
  The capsule landed a third at `payload_contract.rs:501`, on
  `UnknownKeys::SilentlyDropped`: *"the key is accepted and discarded in silence,
  and a correct submission prints no change row either, so the caller has no
  observable that separates landed from discarded."* All three take the same
  one-clause touch at `SL-251`'s reconcile, and the nuance is the same in each:
  the conclusion survives, because a misspelt key rides *inside* an otherwise-valid
  struct and an `ActRecorded` row says nothing about a key dropped within it. Only
  the unqualified premise goes false, and only for act-recording submissions.
- **`SL-238` is disjoint** and was checked, not assumed: its selectors are
  backlog, priority and command-tier paths, and the intersection with this
  slice's is empty. Neither list's size is restated here — both are read from
  `doctrine slice selector list`, and this slice's moved twice inside one
  revision (the `render` narrowing, then `refusal.rs`), so a count in prose would
  have gone stale twice over. `ISS-355` appears in its design once, as prose illustrating a
  footer line format, and its fixtures pin classes rather than corpus counts — so
  closing `ISS-355` costs it nothing. `STD-003` was minted from that slice, which
  strengthens `DEC-240` rather than threatening it: the standard's own subject is
  a degraded *read*, and this defect is emission on a successful write.
- `ISS-355` closed. `ISS-367` (`live_acts` blind to same-kind replacement) stays
  open and sequenced `after SL-256` — it is the other end of the act lifecycle and
  is deliberately not repaired here.

### `REQ-478`

Authored `pending` under SPEC-029 by `DEC-240` (*Report every recorded mutation on
the change log*). Its three acceptance criteria are discharged by checks above —
the mapping is not one-to-one, so it is stated:

| criterion | discharged by |
|---|---|
| an apply that records an act emits a row naming its subject and kind | checks 1–4 |
| every emittable member is driven by the ladder | `every_material_event_kind_persists_a_change_row` over `EMITTABLE` |
| retired vocabulary is readable-only and carries no emission obligation | check 5 (roster membership) and the legacy-fragment round-trip (it still parses) |

The third criterion is a **relieving** clause, and reading it as a positive proof
obligation is the mistake an earlier revision made. `REQ-478`'s own statement says
completeness is stated over the emittable roster "because `DEC-239` splits the two
— the readable half exists to keep historical snapshots parsing, and holding it to
an emission obligation would force a live path for vocabulary that is deliberately
dead." So the criterion asks that the retired member be **exempt** from criterion
2's exhaustiveness demand, not that emission be proven impossible. Check 5 puts it
in `READABLE` and out of `EMITTABLE`; criterion 2's roster test quantifies over
`EMITTABLE` and therefore never demands it; the legacy fragment evidences the
readable half. That is the whole criterion.

**Check 6 is defence in depth, not discharge** (`RV-360` `F-2`). It fails if a
retired member is still being pushed anywhere the `every_event_fixture` ladder
reaches, which is worth having and is strictly more than the roster array proves.
It is not a proof that nothing outside `EMITTABLE` is written: `Pending::about`
and `Pending::run_wide` take any `ChangeEvent`, and a future writer path the
ladder never traverses stays invisible to it. `DEC-239` weighed type enforcement
and declined it; that residual is named here at its true size rather than
absorbed into a criterion that never asked for it.

**The coverage cell must bind a runnable check, not just a mode.** An earlier
draft gave the recipe as `--mode VT` alone. That does not record a `VT` check at
all: `CoverageRecordArgs::has_check` (`src/coverage_store.rs:289-300`) returns
false when no alias, command, extra-arg or matcher is supplied, so `record` takes
the attestation branch (`:132-139`), stores the default `Verified` status with
today's date, and the verifier then treats a check-less entry as backfill and
leaves it alone — the branch is the `let … else` at `src/coverage_verify.rs:156-162`,
whose guarantee `run`'s contract doc states in prose at `:125-135`. The result reads as verified
`VT` with no test bound to it — the same empty claim this slice exists to close
(`RV-360` `F-3`). The recipe is therefore:

```
doctrine coverage record --slice 256 --requirement REQ-478 --change 256 --mode VT \
  --command cargo --command test --command --test --command e2e_design_state \
  --matcher-source stdout \
  --matcher-pattern 'an_agent_declaration_alone_renders_a_change_row \.\.\. ok' \
  --regex
```

The matcher is a **positive control**, and that is why it names one test rather
than matching `test result: ok`: a pass-count pattern also matches a run in which
the new checks were never compiled, whereas this one fails unless `ISS-355`'s own
case ran and passed. The remaining checks ride the same target, whose non-zero
exit fails the command outright.

Recorded once those checks exist, not now — a coverage cell pointing at a test
that has not been written is the same empty claim again. `REQ-478` moves
`pending` → `active` at close, on that evidence.



