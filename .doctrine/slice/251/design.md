<!-- doctrine:section sec-1 -->
# 1. Current behaviour and target behaviour

## Current

`doctrine design apply` accepts one JSON object on `--input` and documents none
of it. `ApplyRequest` (`submission.rs:923`) carries thirteen top-level wire keys —
three flattened in from `SubmissionEnvelope`, ten act fields — and the only
description of any of them that reaches a caller is a single worked example,
`DECLARATION_EXAMPLE` (`render/envelope.rs:75`), which shows two acts of ten and
omits `cursor`.

An agent that needs a shape it has not memorised has exactly two routes.

**Read the source.** `RFC-026 E8.7` measured this: 33 source reads against 52
`doctrine design` calls over `CHR-049`'s run, of which 15 were payload-shape
lookups — the largest single category.

**Submit and read the refusal.** This works, at one revision per attempt, and it
is worse than it sounds. The parse site is
`serde_json::from_str(payload).context("parse the apply payload as JSON")?`
(`design.rs:1504`) — a raw serde message under one context line, naming no
accepted set and pointing nowhere. And where `serde(flatten)` forbids
`deny_unknown_fields`, the outermost type cannot refuse an unknown key at all:
it is discarded, the revision bumps, a receipt is written, no change row prints,
and the command exits 0 (`ISS-333`).

Both failure modes recurred during this slice's own design run, which is the
sharpest available evidence that the cost is real and unmemorable:

- a key spelled at the wrong subject on `Declaration`, refused by
  `deny_unknown_fields` (`submission.rs:123`) without naming what was accepted;
- a payload written for `AgentAct` by analogy with its sibling `DelegationAct`,
  refused with `unknown variant 'act'` — because `AgentAct`
  (`attestation.rs:671`) is externally tagged, `DelegationAct`
  (`submission.rs:877`) is internally tagged on `tag = "act"`, and
  `CheckpointActDeclaration.act` (`submission.rs:825`) is a bare enum. Three
  sibling act types, three taggings, discoverable only from source.

## Target

A caller can ask the binary what an `apply` payload may contain and get a total,
machine-readable answer; and a caller who does not know to ask is told where to
look at the two moments it matters.

**Fetch.** `doctrine design contract [--format json|prompt]` renders the whole
payload contract: for every wire key in the recursive closure rooted at
`ApplyRequest`, its type, whether it is required, the act that owns it, and — for
every enum a key admits — the enum's token vocabulary and its serde tagging
style (`DEC-219`, `DEC-227`).

**Stumble onto it.** The same content is published into the library as a
generated reference doc, sealed `customization = "fixed"`, reachable by
`doctrine library show` and visible in the index an agent browses without knowing
the verb exists (`DEC-224`, `DEC-226`).

**Be told, at the point of failure.** The parse refusal at `design.rs:1504` names
the contract's address, meeting the standard `refusal.rs:483` already sets for
gate refusals: a remedy travels with the refusal so a caller can act *without
fetching anything* (`DEC-225`).

**Be told, before failing.** A one-line pointer rides the turn envelope's no-drop
region, and `design apply --help` carries the same pointer where a caller already
looks (`DEC-225`, `DEC-224`).

The worked example survives all of this rather than being superseded by it: it is
copyable at zero cost, which the contract is not. It is corrected to carry
`cursor` and pinned by a test asserting it deserialises as a valid `ApplyRequest`
(`DEC-228`).

## What does not change

Three things this slice is scoped to leave alone, restated here because a reader
implementing from this section will be tempted by each.

- **`ISS-333`'s mechanism.** The unknown-key silent discard survives. No
  hand-written `Deserialize`, no deserialize-to-`Value`-and-subtract. This slice
  removes the probing loop that defect caused, and discharges `ISS-333`'s third
  candidate option only; the serde axis stays open.
- **Read paths over stored declarations.** Stored proposal declarations ride the
  run snapshot and outlive the binary that wrote them
  (`mem.fact.design-run.snapshot-outlives-the-binary`). A description surface
  does not touch a read path and must not acquire one.
- **The v1 turn envelope wire.** Only the address rides the turn, never the
  contract body — which is what keeps `DEC-064`'s byte budget intact and
  assumption `A1` true.

<!-- doctrine:section sec-2 -->
# 2. The contract model

## Where it sits

`DEC-123` settled the shape of the answer on the neighbouring axis: contract
*structure* rides a const table beside the existing tables, and no third loader
is introduced. The same applies here, with one constraint tighter than first
triaged.

`ADR-001` classifies `design_run` as **leaf, out-degree 0, std + serde derive
only** (`layering.toml:31`). So the contract table and everything that renders it
are pure: no filesystem, no embed lookup, no `crate::install`. This is not a new
posture to invent — `design_run::artifact` already renders the published stage
reference as a pure `render_artifact() -> String`, and `design_run::prompt`
already names asset *keys* while the shell resolves the bytes and passes them
back in. The payload contract follows both.

Two tables already describe fragments of this payload and neither is displaced:

- `ApplyRequest::WRITER_ACTS` (`submission.rs:989`) — nine acts, each with the
  predicate detecting its presence. Answers *does this payload write*, for
  `EX-2`. `delegation` is absent and must stay absent; `A2` records why.
- `Declaration::WIRE_KEYS` (`submission.rs:563`) — fifteen declaration keys, each
  with the subject kind that honours it. Answers *is this key inert here*, for
  `ISS-318`.

Neither answers *what may this payload contain*. The new table does, and sits
beside them.

## What the model has to be able to say

The first draft of this section was written against a remembered payload and then
run over the real closure (`.doctrine/slice/251/render-sample.md`). Four shapes in
that closure could not be expressed at all, and every one of them is a shape a
caller gets wrong unaided. They are listed here because they are the model's
requirements, not its trivia:

1. **An untagged variant.** `WireFacetValue` has no token and no keys — the
   variant *is* a shape, a string or a list of strings (`fnd-11`).
2. **A variant that inlines another named type.** `Dispose::Create(CreateRecord)`
   puts `CreateRecord`'s keys beside `form`, and a rendering has to be able to
   *name* `CreateRecord` rather than silently re-listing its keys (`fnd-14`).
3. **An externally tagged enum whose variants disagree.** `AgentAct` nests one
   payload under its token and emits the other as a bare string (`fnd-12`).
4. **A map whose keys are not free.** `AdoptAuthored.sections` is keyed by section
   id; `CreateRecord.facet`'s legal keys depend on the value of a **sibling**
   field (`fnd-13`).

The types below are shaped by those four. `DEC-221` holds this level of detail
loosely — an implementer may spell it differently on evidence — but a spelling
that cannot state all four is not a different spelling, it is a worse one.

## The row

A contract is a tree of type descriptions rooted at `ApplyRequest`, not a flat
list, because `DEC-227` made totality the full recursive wire closure. Three
element types carry it.

```rust
/// One wire key, wherever it appears in the closure.
pub(crate) struct KeyContract {
    pub(crate) key: &'static str,
    pub(crate) ty: WireType,
    pub(crate) presence: Presence,
}

/// A struct or enum on the wire, named so a refusal and the contract agree.
pub(crate) struct TypeContract {
    pub(crate) name: &'static str,
    pub(crate) form: TypeForm,
}

pub(crate) enum TypeForm {
    Struct {
        unknown_keys: UnknownKeys,
        keys: &'static [KeyContract],
    },
    Enum {
        tagging: Tagging,
        variants: &'static [VariantContract],
    },
}
```

**`tagging` sits inside `Enum` rather than on `TypeContract`.** The first draft
put it on the type and gave `Tagging` a `NotTagged` variant for structs. A struct
has no tagging; `TypeForm::Struct` already says so, and a field whose only job is
to be meaningless on half its inhabitants is a slot for a wrong answer rather than
a fact. Moving it removes the variant and the question together.

**`unknown_keys` sits inside `Struct`, for the same reason and found the same
way.** An earlier draft put it on `TypeContract` beside `form`, which obliged all
fourteen enums in the closure to carry a value for it. `Stage` is a fieldless
externally tagged enum with no key surface at all, so there is no answer it could
give that would be true. That is the `NotTagged` defect one paragraph up,
committed in the other direction and one field over — and the repair is the same:
the field belongs to the inhabitants the question is meaningful for. Unknown keys
are a **struct** property here, and mechanically so. `deny_unknown_fields` is a
serde *container* attribute, and exactly three types in the closure carry it —
`Declaration` (`submission.rs:123`), `CheckpointActDeclaration` (824) and
`AgentActDeclaration` (854) — all three structs. No enum in the closure denies.
If one ever does, the field goes on `TypeForm::Enum` and not on
`VariantContract`, because serde makes it a property of the container rather than
of the arm.

**Every row slice stays `&'static`, and that is a consequence rather than a
wish.** An intermediate draft made all three of them `Cow<'static, [_]>`, on the
grounds that `sec-3`'s injected region is assembled from `facet_fields` while the
process runs and so cannot be `'static`. That reasoning stopped applying the
moment the injected region stopped being a `TypeContract`: `sec-3`'s
`SelectorTable` owns its own rows, nothing the command tier builds is ever a
member of `TypeForm`, and so the const table is entirely borrowed with nothing to
borrow-or-own about. The `Cow`, its two `const fn` constructors, and the
`Box::leak` and const-fn-assembly alternatives weighed against it all go with it.
Worth naming because it is the second time in this design that the honest
modelling of the extern region has removed machinery rather than added it.

`WireType` is the recursion:

```rust
pub(crate) enum WireType {
    Text,
    Integer,
    Boolean,
    Id(&'static [IdKind]),           // a run id, and which kinds this key admits
    Named(&'static TypeContract),    // the closure edge
    Token(TokenSource),              // a string field with a closed vocabulary
    Seq(&'static WireType),
    Map { key: MapKey, value: &'static WireType },
}
```

`Token` is not a second way to spell an enum. It is what a facet field's
`FieldShape::Closed` actually is on the wire — a plain string drawn from a closed
set, with no Rust type name a caller could ever see. `Named` pointing at an
invented enum would say something false about the payload.

**`Named` stays `&'static` because nothing injected is ever reached through it.**
The two externally-sourced regions arrive through `TokenSource` and `MapKey`
instead — see `sec-3` — so the closure edge remains a const reference and the
acyclicity argument there keeps its mechanical basis.

**`Id` carries its admissible kinds, because a bare `id` is the same omission
this slice exists to remove.** `IdKind` (`ids.rs:30-52`) is a closed leaf-side
vocabulary of eight prefixes — `inq-`, `sec-`, `cp-`, `att-`, `fnd-`, `dlg-`,
`cpa-`, `agd-` — each with a `const fn prefix()`, so naming the admitted set
costs `ADR-001` nothing and introduces no second list. Which kinds a key admits
is not uniform, and the engine already decides it per key: `declare` routes on
the subject's kind and admits **five** of the eight — inquiry, section,
attestation, finding, checkpoint — refusing `dlg-`, `cpa-` and `agd-` outright,
because those three are engine-allocated and a caller may never declare one
(`run.rs:1139-1150`). `AdoptAuthored.sections` is narrower still: `sec-` alone.
Rendering all of them as a bare `id` would leave a caller guessing in exactly
the way the missing `cursor` did, and `Declaration::WIRE_KEYS`'s
`KeyHome::At(IdKind::…)` is standing evidence that the per-key answer is already
held rather than something this contract has to invent.

**The map key is a description, not an afterthought.** `Map` carried only its
value type in the first draft, which was enough for neither of the closure's two
maps:

```rust
pub(crate) enum MapKey {
    /// Keys are values of this wire type — `AdoptAuthored.sections` is keyed by
    /// section id, and "map of text" would have lost that.
    Of(&'static WireType),
    /// Keys are supplied by a region this tier cannot import, and *which* keys
    /// are legal is chosen by the value of a sibling field named here.
    /// `CreateRecord.facet`'s keys are the facet fields of the kind in `kind`.
    Extern { region: ExternRegion, selector: &'static str },
}
```

`selector` is a plain field name — a `&'static str` naming a sibling key, not a
type from another tier — so it costs `ADR-001` nothing. It is the only place in
the model where one key's contract depends on another key's *value*, and that
dependency is real: it is the whole of what makes `facet` discoverable rather than
an open bag.

## The variant row, and where a payload actually sits

`TypeForm::Enum` holds these:

```rust
/// One enum variant.
pub(crate) struct VariantContract {
    /// The token a caller writes. `None` only under `Untagged`, where serde emits
    /// no token at all and printing the Rust variant name would say something
    /// false.
    pub(crate) token: Option<&'static str>,
    pub(crate) payload: VariantPayload,
}

pub(crate) enum VariantPayload {
    /// A unit variant.
    Absent,
    /// A struct variant's own keys.
    Keys(&'static [KeyContract]),
    /// A newtype variant over a named type, whose keys arrive in this variant's
    /// place — `Dispose::Create(CreateRecord)`.
    Inlines(&'static TypeContract),
    /// An untagged variant, which is a shape rather than a set of keys —
    /// `WireFacetValue`'s `[text]` and `text`.
    Shape(&'static WireType),
}
```

**Where those keys sit on the wire is a function of `Tagging` and the payload
together, and is derived rather than stored.** That is what keeps the two from
disagreeing, and it is the same discipline `ReviewDisposition` states one tier
over — a value that says the same thing twice is checkable, so the second saying
should be a test rather than a field.

| tagging | payload | on the wire |
|---|---|---|
| `Internal(tag)` | `Keys` / `Inlines` | keys sit **beside** the tag: `{"form":"create","kind":…}` |
| `Internal(tag)` | `Absent` | the tag alone: `{"provenance":"user-directed"}` |
| `External` | `Keys` | nested under the token: `{"blocking-set-declared":{"blocking":[…]}}` |
| `External` | `Absent` | **a bare string**: `"drafting-ready"` |
| `Untagged` | `Shape` | the shape alone, discriminated by JSON form |

**The fourth row is the one that bit.** `AgentAct` is a *mixed* externally tagged
enum: `BlockingSetDeclared` nests and `DraftingReady` is the bare string
`"drafting-ready"`, not `{"drafting-ready":{}}`. A first draft that put `Tagging`
on the type and read "External means the payload nests" would have been wrong for
half of `AgentAct` — and wrong in the expensive direction, because
`AgentActDeclaration`'s `deny_unknown_fields` does not reach inside `act`, so the
object a caller would have sent is discarded in silence.

That row also removes a variant. The first draft carried `Tagging::Bare` for
"a unit-variant enum". *Bare* is what `External` + `Absent` already produces, for
every variant of `Stage`, `ActKind`, `Reviewer` and the rest; storing it beside
the payload gives a `Bare` claim something to contradict. It survives as a word
the **renderer** prints when an external enum's every payload is `Absent`, which
is a reading convenience derived from the model rather than a fact held in it.

The renderer still has to *say* all of this. A rendering that listed `Dispose`'s
four tokens and stopped would leave a caller believing `create`'s payload nests,
which is precisely the class of error this contract exists to end — and the same
rendering over `AgentAct` would have a caller wrapping a bare string in an object.

## The three fields that carry the semantics

`DEC-219` decided the contract carries wire shape *plus engine semantics*, and
these are the three places that decision is spent. Each earns its keep against a
failure that actually happened.

**`Tagging`** — the failure that cost a round trip on this slice's own run.

```rust
pub(crate) enum Tagging {
    Internal(&'static str),       // { "act": "export", … }   — DelegationAct, Provenance
    External,                     // { "blocking-set-declared": { … } } — AgentAct, ReviewDisposition
    Untagged,                     // discriminated by shape alone — WireFacetValue
}
```

Three sibling act types spell their discriminant three different ways —
`DelegationAct` internally on `tag = "act"` (`submission.rs:877`), `AgentAct`
externally (`attestation.rs:673`), and `CheckpointActDeclaration.act` as a bare
`ActKind`. A contract stating field types alone would have said nothing useful
about any of them.

Each mode has a second live member, and the second is the instructive one.
`Provenance` (`inquiry.rs:25`) is internally tagged on `tag = "provenance"` and
carries two payload-bearing variants, `ImportedProse` with four keys — the only
place `u32` and `Fingerprint` enter the closure. `ReviewDisposition`
(`attestation.rs:571`) is externally tagged and *uniform*, both arms carrying
struct payloads, which is what makes `AgentAct`'s asymmetry legible as an
asymmetry rather than as the rule.

`Untagged` is the least guessable: `WireFacetValue` (`submission.rs:254`) is
`#[serde(untagged)]` over `List(Vec<String>)` and `Text(String)`, so a facet
value is discriminated by its JSON shape with no tag anywhere. Nothing in a
field's name or type hints at this, and there is no refusal that teaches it —
a wrongly-shaped value simply fails to match any variant.

**`Presence`** — the wire has three states, not two.

```rust
pub(crate) enum Presence {
    Required,
    Optional,                     // absent means absent
    Sparse,                       // absent means "persist"; null means "clear"
}
```

`Sparse<T>` (`submission.rs:31`) is the run's editing idiom and it is invisible
from a type signature: omitting `parent` persists the existing value, sending
`null` clears it. The envelope's worked example already tries to teach this in a
trailing parenthetical because there is nowhere better to put it. A contract has
somewhere better.

**`UnknownKeys`** — the honest disclosure of `ISS-333`.

```rust
pub(crate) enum UnknownKeys {
    Refused,            // deny_unknown_fields — a misspelt key is a refusal
    SilentlyDropped,    // serde(flatten) forbids deny_unknown_fields — ISS-333
}
```

**Two states, and an earlier draft's third was false.** That draft added
`StoredThenFlagged` — *accepted, written to the record, reported afterwards by
`doctor`* — for `sec-3`'s facet keys, and made the selector table its only
inhabitant. It describes behaviour this codebase does not have. An unrecognised
facet key on a `create` disposition is **refused**: `plan_facet_edits` returns
`FacetEditRefusal::UnknownField` when no `facet_fields(kind)` row matches
(`knowledge.rs:1233-1247`), and the apply path calls it inside `plan_checkpoints`,
which runs before `execute_mint` — so the refusal lands before an id is reserved
(`commands/design.rs:985`). `doctor`'s inert-facet-key check reports the
**authored corpus**, not this seam, and says where the line falls in its own doc
comment: it is a warning there "because the design-run seam's equivalent is a
refusal" (`doctor_checks.rs:141-148`). A contract that had shipped
`StoredThenFlagged` would have sent a caller hunting for a write that never
happened — the same class of harm `SilentlyDropped` exists to prevent, inverted.
`sec-3` carries the correction where it lands.

**The name is the contract here, and a bool would have been the wrong type.**
`denies_unknown: false` is readable two ways — *accepted and stored*, or
*accepted and thrown away* — and only one of them is true. A caller who guesses
wrong on that distinction spends a revision hunting a change that was never
made. `SilentlyDropped` cannot be misread, and that is the whole reason it is
spelled out rather than negated.

**The hole is not one level deep — it is most of the payload.** Exactly three
wire structs carry `deny_unknown_fields`: `Declaration` (`submission.rs:123`),
`CheckpointActDeclaration` (824) and `AgentActDeclaration` (854). The other nine
— including `TraversalDeclaration`, which carries `cursor` — accept an unknown
key and discard it. A misspelt `cursor` therefore behaves exactly like a misspelt
top-level act: the revision bumps, a receipt is written, no change row prints,
and the command exits 0.

**And the missing change row is not a signal, which is why the disclosure has to
be in the contract rather than in the response.** A *correct* submission prints
no change row either (`ISS-355`), so the caller has no observable that separates
*landed* from *discarded*. That is the whole reason this slice buys
discoverability before submission instead of a better report afterwards: after
the fact there is nothing to read. Repairing the response is `ISS-355`'s, and
this design does not touch it.

Two causes sit under that one behaviour, and the difference decides what happens
next rather than what a caller sees. `ApplyRequest` **cannot** deny —
`serde(flatten)` forbids it, which is `ISS-333`'s actual mechanism. The other
eight simply carry no attribute, and nothing structural stops them acquiring one.
So `SilentlyDropped` is the majority row in this contract, not a single top-level
disclosure, and the eventual repair is mostly cheap. Both facts belong in the
design rather than left for a reader to infer from `ISS-333`'s framing.

Annotating those eight is still out of scope, on the Non-Goal that actually
applies: stored proposal declarations ride the run snapshot and outlive the
binary that wrote them, so tightening a read path converts previously-readable
stored state into a parse failure at exactly the moment someone is resuming.

This slice does not repair the mechanism — an explicit Non-Goal — it stops the
mechanism from being a secret. The ordering is deliberate: be truthful about the
protocol now, and change the protocol later. When every wire struct refuses —
the eight attribute-only cases as well as `ISS-333`'s flatten — `SilentlyDropped`
has no members left, `TypeForm::Struct`'s `unknown_keys` collapses to a single
value, and the field can be deleted outright. A contract that had quietly implied
uniform refusal would instead have been wrong for the whole intervening period,
and wrong in the direction that costs a caller a revision to discover.

## Naming, and why the type names ship

`TypeContract::name` carries the Rust type name — `Declaration`, `AgentAct`,
`TraversalDeclaration`. Two reasons, both practical.

A refusal already names these types: serde's `unknown variant 'act'` message
arrives attached to a type, and `DEC-225`'s wrapped parse refusal will carry it.
If the contract used different names, the refusal and the contract would be
describing the same thing in two vocabularies, and the caller would have to
bridge them.

An agent with source in its tree — the confounded majority in this repo, per the
scope — can jump straight from a contract row to the definition. That costs
nothing and is exactly the read the contract is trying to make unnecessary for
everyone else.

It is also what `VariantPayload::Inlines` carries. `Dispose::Create`'s rendering
— *`create` → `CreateRecord`'s keys, inlined* — needs the name, and taking it
from the pointed-at `TypeContract` means the variant row and the type row cannot
disagree about what the inlined type is called.

**And the name is tied to the type rather than merely spelled beside it.** A
`&'static str` holding `"AgentAct"` is a second spelling of an identity the
compiler already owns. Rename the Rust type and that literal, every `Named` edge
reaching it and every JSON key derived from it stay perfectly consistent with
each other and go wrong together — the refusal then cites one vocabulary and the
contract another, which is the single failure this subsection exists to prevent.
Internal consistency is not the property wanted here; agreement with the source
is. Two seams supply it, each where the type identity is already in hand. The
enums get theirs by construction: `payload_variants!` (`sec-4`) is invoked with
the type, so it emits a `stringify!` of that identifier rather than a
typed-again literal. The structs get theirs by assertion: `assert_keys_described`
(`sec-8` pin 1) already receives both the value and its contract, so it also
compares `name` against `T`'s own name. Neither adds a mechanism; both stop an
identity the site already holds from being retyped.

## The root, and what it does not include

```rust
pub(crate) const PAYLOAD: TypeContract = /* ApplyRequest */;
```

One root, from which the closure is reachable. `ApplyRequest`'s thirteen
top-level wire keys are three flattened in from `SubmissionEnvelope` (`run_uid`,
`known_revision`, `submission_id`) and **ten** act fields (`submission.rs:923-967`).
Ten against `WRITER_ACTS`'s nine rows is the asymmetry Objective 3 names —
`delegation` is the tenth — and stating nine here would have reproduced the
defect the slice exists to fix. The flatten is a
presentation detail of the Rust types, not of the wire — a caller sends thirteen
keys at one level — so the contract renders them at one level too, and
`SubmissionEnvelope` appears in the closure only as the reason the root's
`unknown_keys` is `SilentlyDropped`.

**So the closure holds twelve struct types and the rendering emits eleven
blocks**, and no claim in this design may use one number where it means the
other. `SubmissionEnvelope` is a real member of the closure with a real
three-key serde surface — a key-set assertion over it is well defined, and
`sec-8` pin 1 counts twelve for that reason — but it has no block of its own
because its keys are already rendered at the root (`fnd-16`).

Not in the table: `resolved_record`, which carries `#[serde(skip)]` and is
therefore not on the wire at all. The closure is over what serde accepts, and a
skipped field is outside it by construction rather than by editorial choice —
which is the same test `DEC-227` used to reject a boundary drawn by judgement.

<!-- doctrine:section sec-3 -->
# 3. The closure, and what bounds it

## What is in it

`DEC-227` set totality as the full recursive wire closure rooted at
`ApplyRequest`. Traced against the source rather than recalled, that is **twelve
struct types** —

`ApplyRequest`, `SubmissionEnvelope`, `AdoptAuthored`, `TraversalDeclaration`,
`StageDeclaration`, `AcceptanceDeclaration`, `Declaration`, `CreateRecord`,
`DischargeDeclaration`, `ReviewPolicyDeclaration`, `CheckpointActDeclaration`,
`AgentActDeclaration`

— plus **fourteen enums** they admit: `Dispose` (four forms), `WireFacetValue`,
`DischargeClaim`, `DelegationAct` (four acts), `AgentAct`, `ActKind` (eight),
`ReviewPolicy`, `ReviewDisposition`, `Reviewer`, `Stage`, `Posture`, `Authority`,
`Provenance`, `InquiryLifecycle`. `Sparse`'s three states ride the `Presence`
field rather than a type row.

Two of those enums were missing from the first enumeration, and the class matters
more than the errata: `ReviewPolicy` is reachable only through
`ReviewPolicyDeclaration.policy` and `Reviewer` only through
`Declaration.reviewer` — each one hop below a field nobody had walked. That was
the argument for **deriving the closure mechanically and diffing it against this
list**, rather than trusting either.

**That derivation has now been done, ahead of implementation, and this list
survived it.** `.doctrine/slice/251/render-sample.md` walks `ApplyRequest`'s
serde surface in source and diffs the result against the list above: the twelve,
the fourteen, their membership, the refuse/discard split below, the
thirteen-against-nine asymmetry and the acyclicity argument all hold. The
commitment stands for implementation anyway — a derivation performed once by hand
is evidence, not a pin — but the list is no longer the weakest thing in this
section. What the derivation *did* break was one tier down, in `sec-2`'s model and
`sec-5`'s renderings, and is carried there.

Twelve is the count of **closure members**, not of rendered blocks; eleven blocks
render, because `SubmissionEnvelope` is flattened and its keys appear at the root.
`sec-2` states which number belongs to which claim.

## What bounds it, and why the bound is a fact rather than an opinion

Three exclusions, each by construction.

**`#[serde(skip)]` fields are outside the wire.** `Declaration::resolved_record`
is the live case. It is excluded because serde does not accept it, not because
someone judged it internal — which is the same test `DEC-227` used to reject a
boundary drawn by judgement.

**A type that serialises as a scalar is not a struct row.** There are **three**,
not the two first listed. `DesignId` (`ids.rs:132`) carries
`#[serde(try_from = "String", into = "String")]`; `ReviewRef`
(`attestation.rs:475`, reached through `ReviewDisposition::Conducted`) and
`Fingerprint` (`ids.rs:213`, reached through `Provenance::ImportedProse`) are
newtypes over `String`, which serde emits transparently. All three sit in the
*type* closure and none has wire keys, so all three render as scalars — `Id`,
`Text`, `Text` — not as types with rows. The test is mechanical again: what serde
emits, not what the Rust declaration looks like.

**`WRITER_ACTS` is not the axis.** It enumerates nine acts and correctly omits
`delegation`, because it answers *does this payload write* for `EX-2` and the
proposal channel must not count as a write (`A2`). `ApplyRequest` carries thirteen
top-level wire keys — ten act fields against that table's nine rows. A contract keyed off that table would ship a payload field
no caller could discover, which is the asymmetry the scope named.

**The closure is acyclic, and this was checked rather than assumed.** It matters
mechanically: `WireType::Named(&'static TypeContract)` is a const reference, and
a cyclic const graph cannot be expressed that way. The deepest chain runs through the proposal channel:
`ApplyRequest → delegation → DelegationAct::Propose → declare → Declaration →
dispose → Dispose::Create → CreateRecord → facet → WireFacetValue`.
`DelegationAct::Propose` carries `declare: Vec<Declaration>`
(`submission.rs:882-891`) — the one edge that re-enters the declaration subtree
from outside it, and `Declaration` reaches no delegation act, so it closes
nothing. `WireFacetValue` bottoms out in `Vec<String>` and `String` rather than
recursing (`submission.rs:254`). Nothing else re-enters a type already on its own
path.

This is worth stating in the design rather than discovering at the compiler: if
a future payload type does introduce a cycle, `&'static TypeContract` stops
working and the table needs a name-keyed indirection instead. Better to know
which property the representation depends on.

**The closure spans six files**, all inside `design_run`: `submission.rs` (every
wire struct), `attestation.rs` (`Reviewer`, `ActKind`, `ReviewPolicy`,
`ReviewDisposition`, `AgentAct`, `ReviewRef`), `inquiry.rs` (`Provenance`,
`InquiryLifecycle`), `traversal.rs` (`Authority`, `Posture`), `mod.rs` (`Stage`)
and `ids.rs` (`DesignId`, `Fingerprint`). `design_run` is one leaf-tier module
(`layering.toml:31`), so none of those crossings creates a module-graph edge — but
an implementer walking the closure visits all six, and the two scalars in `ids.rs`
are exactly the members most easily mistaken for struct rows.

## The one vocabulary the leaf cannot see

`CreateRecord::kind` is a `String` on the wire, and it stays one deliberately.
The doc comment on the type says why: `design_run` is a leaf with crate
out-degree zero, so the knowledge-kind vocabulary lives where it already lives —
`crate::knowledge::RecordKind` — and the shell resolves the token against it. A
second enum inside `design_run` would be exactly the parallel wire type `ADR-001`
leaves no room for.

So the contract has a region whose admissible tokens are real, closed, and **not
knowable from the tier that renders the contract**.

## And the region is larger and better-structured than `kind` alone

The obvious framing is that `kind` is one string the leaf cannot enumerate. That
undersells both the problem and what is available to solve it.

`CreateRecord::facet` is a `BTreeMap<String, WireFacetValue>` — an *open map* on
the wire. A caller assembling a `create` disposition today has to guess facet
names entirely, and this design run guessed them by copying a previous payload.
That is a strictly worse instance of the same defect the slice exists to remove,
and it sits one level below `kind`.

But the knowledge tier is not an untyped bag. It already publishes a
contract-shaped description of itself:

```rust
pub(crate) const fn facet_fields(kind: RecordKind) -> &'static [FacetFieldRow];

pub(crate) struct FacetFieldRow {
    pub(crate) name: &'static str,
    pub(crate) shape: FieldShape,
}

pub(crate) enum FieldShape {
    Text,
    List,
    Closed(&'static [&'static str]),
}
```

(`knowledge.rs:860-875`, `1028`.) `RecordFacet` is a typed enum-of-structs, one
variant per kind, deliberately *not* an untyped bag; **four** closed facet
value-enums each carry a `KNOWN` set — `Confidence::KNOWN` (on both the
assumption and evidence rows), `Basis::KNOWN` (assumption),
`ConstraintSource::KNOWN` (constraint) and `Provenance::KNOWN` (evidence,
`knowledge.rs:993-1007`) — and `FieldShape::Closed` takes its tokens from that
set rather than a retyped literal, on `STD-001`. An earlier draft of this
sentence counted three and lost `Provenance`; the mapping is generic over
`FieldShape` and never enumerated them, so nothing downstream was built on the
miscount, but a design that claims to have traced the source has to have traced
it.

The correspondence with `sec-2`'s model is close enough to be suspicious, and it
is not a coincidence — both are describing the same wire. `FieldShape::Text` and
`FieldShape::List` are exactly `WireFacetValue`'s two untagged variants, which is
*why* `WireFacetValue` is untagged: a facet value is a string or a list of
strings because a facet field is `Text` or `List`.

**The two halves are one region, not two.** `kind`'s admissible tokens and
`facet`'s admissible keys are both functions of `RecordKind`, and the second is
indexed by the first. Treating them as separate injected vocabularies would create
two lists free to disagree about which seven kinds exist — the defect this section
exists to avoid, reintroduced inside the fix.

## The resolution: inject one region, through a closed seam

Three ways to handle the extern region, and the third is the one this design
takes.

1. **Render it as free text.** Honest about the type, useless to the caller, and
   it reintroduces the guess-and-submit loop the slice exists to remove — for
   `kind`, and worse for `facet`.
2. **Duplicate the vocabulary in `design_run`.** Forbidden by `ADR-001`, and it
   would be a second list free to drift from `RecordKind` and `facet_fields` —
   the failure `DEC-221` spent the whole inquiry avoiding.
3. **Have the shell inject it.** The leaf declares that a region of the closure is
   externally supplied; the command tier — which may import both `design_run` and
   `knowledge` — builds the description from `RecordKind::ALL` and `facet_fields`,
   and hands it in at render time.

Option 3 is not a new mechanism. It is precisely how `design_run::prompt` already
works: that module names what the shell must supply and never reads it, and the
shell resolves the bytes and hands them back for `contract_block` to place
(`prompt.rs:185`). The payload renderer takes the same split, with a richer
payload.

**And it takes the precedent's strength, which the first draft dropped.** That
matters more than the split does. `prompt` does not key the shell's contribution
by a string: `contract_block(edge, bodies: &BTreeMap<Condition, String>)` is keyed
by `Condition`, a **closed leaf-side enum**, and `Fragment` (`prompt.rs:48-63`)
*derives* its asset key from the variant rather than looking the variant up by
string. A first draft of this section reached for `Extern(&'static str)` and a
string-keyed supply map, which is the cited precedent minus the thing that makes
it safe. So:

```rust
/// A region of the closure owned by a tier this one cannot import. Closed, so
/// naming a region and supplying one are the same act.
pub(crate) enum ExternRegion {
    /// `knowledge::RecordKind` and, per kind, `facet_fields`.
    KnowledgeRecord,
}

impl ExternRegion {
    pub(crate) const ALL: [ExternRegion; 1] = [ExternRegion::KnowledgeRecord];
    /// The source this region names, rendered. One spelling (STD-001).
    pub(crate) const fn label(self) -> &'static str { … }   // "knowledge::RecordKind"
}
```

One member today. The cardinality is not the point — the *closure* is, exactly as
it is for `Fragment`'s four and `Stage`'s five.

The supplied description is **one selector table, and deliberately not a
`TypeContract`**:

```rust
/// Key sets chosen by the value of a sibling field. Not a type on the wire and
/// not a tagged union: `kind` and `facet` are two ordinary keys of the same
/// `CreateRecord` object.
pub(crate) struct SelectorTable {
    /// Where a caller can go and read the region. One spelling (STD-001).
    pub(crate) source: &'static str,          // "knowledge::RecordKind"
    /// Each admissible value of the selecting field, with the keys it opens.
    pub(crate) rows: Vec<SelectedKeys>,
    /// `Refused` — an unknown facet key is rejected before the mint, not stored
    /// (`knowledge.rs:1233`).
    pub(crate) unknown_keys: UnknownKeys,
}

pub(crate) struct SelectedKeys {
    pub(crate) token: &'static str,           // a `RecordKind` token
    pub(crate) keys: Vec<KeyContract>,
}
```

A first draft of this section made it an enum-form `TypeContract` with seven
variants, and that was wrong twice over in the same shape. There is no tagged
union here, so no `Tagging` value is true of it — not `Internal`, since there is
no tag key; not `External`, since facet keys do not nest under the kind token;
not `Untagged`, since the rows do carry tokens. And `TypeContract::name` is
defined one section up as the Rust type name a refusal will cite, while nothing
here is a type a refusal ever names. `sec-2` deleted `Tagging::NotTagged` on the
argument that a field meaningless on some of its inhabitants is a slot for a
wrong answer, and moved `unknown_keys` into `TypeForm::Struct` on the same
argument; pressing this region into `TypeForm` would have re-created that
inhabitant one section later, which is why it gets a type of its own instead.

Two consumers read the one table and neither can disagree with the other,
because there is still only one list:

- `CreateRecord.kind` is `Token(TokenSource::Extern(KnowledgeRecord))` — the
  table's row tokens, and still a `Token` rather than a `Named`, so the contract
  goes on saying the true thing about a string field.
- `CreateRecord.facet` is
  `Map { key: MapKey::Extern { region: KnowledgeRecord, selector: "kind" }, value: Named(WIRE_FACET_VALUE) }`
  — the keys of whichever row the sibling `kind` names.

Mapping `FacetFieldRow` into `KeyContract` is mechanical: `Text` and `List` become
`WireType`s directly, and `Closed(tokens)` becomes `Token(TokenSource::Fixed(tokens))`.

What a caller then gets, for a payload key that today is an undocumented open
map: the seven kinds, and per kind the facet names it owns, in template order,
with closed vocabularies enumerated where they exist.

## The seam is drift-proof by construction, except in one narrow place

The first draft of this section conceded the opposite — that nothing at compile
time forces the shell to supply a contract for every region named, nor forces the
supplied contract to match `facet_fields`, and that both close by test. Neither
concession was necessary. Three risks, and the shape of the seam decides which of
them a compiler can hold.

**A region nobody supplies.** Made unspellable rather than tested. The supply is a
struct with one field per region and a `const fn` resolving `ExternRegion` to it
by exhaustive match — so adding a region to `ExternRegion` without supplying it is
a non-exhaustive-match error, and there is no string key left to typo. This seam
must in fact be *stricter* than the precedent it borrows from: `contract_block`
deliberately tolerates an absent body — "an absent body is not placed, and the
header still rides" — because a caller with a stale receipt must still see what it
is missing. An absent extern contract has no such reading. It renders the region
as nothing, silently, which is the discoverability failure itself rather than a
degraded report of it.

**A vocabulary that has moved.** Partly a compile barrier — and the part that is
not was found by review rather than by drafting, so it is stated here at its real
strength. Built from `RecordKind::ALL` and `facet_fields(kind)` directly, a new
facet *field* flows through with no second list to update. `facet_fields` is a
`const fn` whose match over `RecordKind` is already exhaustive, so a new *kind*
is a compile error *there*, at the source, forcing someone to give it a facet
row. And `FieldShape → WireType` written as a match with no wildcard arm makes a
new `FieldShape` variant a compile error at the mapping.

**What that barrier does not cover: `RecordKind::ALL` is not enum-derived.** The
enum is declared at `knowledge.rs:60-68`; `ALL` is a hand-written
`[RecordKind; 7]` two hundred lines away at `159-169`. Adding an eighth variant
compels a `facet_fields` arm and compels nothing about `ALL`, so the builder —
which *iterates* `ALL` — would emit a contract silently missing the new kind. The
incumbent test cannot see it either: `record_kind_from_prefix_round_trips_each_kind`
iterates `ALL` and asserts its seven prefixes are distinct
(`knowledge.rs:3334-3341`), which is green over a stale `ALL` by construction.
This is a pre-existing gap in the knowledge tier, not one this slice introduces,
and repairing `ALL` is not in scope here. What is in scope is refusing to claim a
barrier that is not there: `sec-8` pin 5's row-token equality therefore takes its
oracle from an **exhaustive match over `RecordKind`** — `sec-4`'s own instrument,
pointed at a knowledge-tier enum from a test — and never from `ALL`, which would
compare the table against the same list that dropped the kind.

**A mapping that dropped or reordered something.** This is the residue, and it is
a test — `sec-8` pin 5. It is a narrower claim than the one the first draft made:
not *whether the vocabulary is the same vocabulary* for the parts the barriers
above do settle, but whether a compile-checked mapping was also a faithful one.
That is the kind of thing a set-equality is good at.

So the injected region is no longer the ladder's only test-only rung, but it is
not fully compile-held either, and the honest tally is one-and-a-half barriers
rather than two. *A region nobody supplies* is closed outright. *A vocabulary
that has moved* is closed for facet fields and for the `FieldShape` mapping, and
open at `RecordKind::ALL`, where the iteration authority is hand-maintained. Pin
5 therefore carries two claims rather than one — the mapping is faithful, and the
kind set is the enum's — and it is sized in `sec-8` against both. That is below
what `DEC-221` buys elsewhere, and saying so is the point: the gap is in the
knowledge tier's own list, this slice does not repair it, and a reader who takes
the region as fully compile-held would be relying on a barrier that is not there.

## One thing the contract may say plainly

An earlier draft of this subsection had `facet_fields` enforced **post hoc, not
at the write seam** — an unrecognised facet key accepted by `create` and
surfacing later as a `doctor` inert-facet-key finding — and concluded that the
contract could describe only *what a kind owns*, never *what will be refused*.
That is not this codebase's behaviour, and the correction runs the other way.

**The write path refuses.** `plan_facet_edits` resolves every supplied edit
against `facet_fields(kind)` and returns `FacetEditRefusal::UnknownField` on the
first row it cannot find (`knowledge.rs:1233-1247`). The apply path calls it
inside `plan_checkpoints`, which runs before `execute_mint` — the comment at the
call site says so in terms: siting the validation in that walk is what makes
*before any id is reserved* a property of existing structure rather than of a new
guard (`commands/design.rs:985`). A bad facet key therefore refuses before
anything is written and burns no id.

**`doctor` is a different surface answering a different question.** Its
inert-facet-key check reports the **authored corpus** — it scans each kind's
record tree on disk — and its doc comment draws exactly this line: the check is a
warning there "because the design-run seam's equivalent is a refusal"
(`doctor_checks.rs:141-148`). Reading a corpus-hygiene warning as the design-run
seam's behaviour is what produced the draft's inversion.

So `SelectorTable::unknown_keys` is `Refused`, alongside the three wire structs
that deny, and `sec-2`'s enum keeps two states rather than growing a third to
hold this one. **The contract is stronger than the draft claimed**: `facet` is
not an open bag with a late warning, it is a closed vocabulary checked before
anything is written, and a caller can be told so without the contract promising
more than the write path keeps.

One thing does survive from the discarded reasoning, and it is the part worth
keeping: making the *description* drift-proof says nothing about the *check*
being over the same vocabulary. `sec-8` pin 5 is what holds those together.

## And one thing it does not claim at all: stage admissibility

The closure bounds what the contract is total *over* — the payload. It says
nothing about *when* a payload is admissible, and that bound is worth stating
because the two claims read as one until they come apart. An agent that fetches
this contract learns `dispose` and every field it takes, submits at
`stage = exploring`, and gets exit 0 with a revision bump and no change rows
(`ISS-362`). The contract will have been complete on its own terms and will
still have permitted the class of mistake the state machine exists to prevent.

A stage column would be the wrong repair, and not because it is expensive. There
is no payload-act-to-stage map in the tree to render. `SL-244`'s generated
`CONTRACTS` table is keyed by `ActKind` — the eight *attestation* acts — which is
a disjoint vocabulary from `ApplyRequest`'s ten act fields;
`gate::requirement_for` is `fn(ActKind) -> Option<ActRule>` (`gate.rs:758`) and
never sees `dispose`. Authoring the mapping here would mint a second source of
truth for a rule the engine does not enforce, unpinnable because there is no
behaviour to pin it against, and it would promise refusal at a seam that fails
open — precisely the trap the subsection above avoids for `facet_fields`.

So the contract states the bound rather than the column: total over the payload,
silent on stage, with the state machine named as where admissibility lives.
`ISS-362` is an enforcement defect — a typed refusal naming both the current
stage and the act's required one, and no revision bump, since nothing happened —
and it is repaired at the guard, in its own slice. Once that guard exists as
data, a stage column here becomes derivable and pinnable. That is the order the
two changes have to happen in, and doing them the other way round is what would
ship the promise before the rule.

<!-- doctrine:section sec-4 -->
# 4. Pinning the contract against the types

## What has to be pinned, and what an oracle looks like

`R1` names the risk in one line: a second description of the payload, free to go
stale. `DEC-123` and `SL-249` both answer it the same way — a test, not
discipline. The question is what test, and the answer differs by what is being
pinned.

A **struct's field set** is reachable from one serialised value. `SL-249`'s `I9`
constructs a `Declaration` with an exhaustive no-`..` literal, serialises it,
and compares the key set against `WIRE_KEYS` (`submission.rs:649`, test at
`tests.rs:2900`). A new field is a compile error *at the literal*, before the
test even runs.

An **enum's variant set** is not reachable that way — no single value carries it.
This is why `SL-244` generates rather than asserts, and its macro doc says so
directly: *"Add a variant, give it an `as_str` arm, leave it out of `ALL` and
`CONTRACTS`, and every assertion still passes while the variant has no boundary,
no contract and no prose"* (`gate.rs:436-458`). The repo demonstrates the failure
live — `ActKind::ALL` (eight members) and `Advance::ALL` (four) are hand-written
with no completeness pin.

## The enum pin: an exhaustiveness barrier, not a regenerated type

`DEC-221` settled that enum vocabularies are generated on
`condition_vocabulary!`'s pattern. Drafting found a strictly better instrument in
the same family; `DEC-229` records the change and partially supersedes `DEC-221`
— enums only, the struct half untouched. `DEC-221` anticipated being superseded
on exactly this axis, and the direction is the one it named.

`condition_vocabulary!` **owns** the `Condition` type: it emits the enum
definition, `ALL`, `as_str`, the serde renames and `CONTRACTS` from one list.
That is available because `Condition` was introduced by `SL-244`. It is not
available here. `Dispose`, `DelegationAct`, `AgentAct`, `ActKind`, `Stage`,
`Posture`, `Authority` and the rest already exist, carry data on their variants,
and sit under a behaviour-preservation gate as shared machinery. Regenerating
their definitions through a macro would be the largest and riskiest change in a
slice scoped as a documentation surface — and `R3` records the price of the
generated region: clippy does not lint tokens a macro body wrote
(`gate.rs:459-462`).

The insight that removes the need: **`condition_vocabulary!`'s real guarantee is
not that it writes the type — it is that an omission makes a generated match
non-exhaustive, which is a build failure.** That guarantee can be had without
owning the definition.

For each enum in the closure, the macro emits its token list *and* a dead
exhaustive match over the real type. **Where the type already owns its token
vocabulary, the macro consumes that authority rather than retyping it** — six of
the fourteen closure enums do, and each of those `as_str` bodies cites `STD-001`
as its own reason for existing:

| enum | authority |
|---|---|
| `Stage` | `Stage::as_str` (`mod.rs:147`) |
| `ActKind` | `ActKind::as_str` (`attestation.rs:107`) |
| `ReviewPolicy` | `ReviewPolicy::as_str` (`attestation.rs:221`) |
| `ReviewDisposition` | `ReviewDisposition::arm` (`attestation.rs:597`) |
| `Provenance` | `Provenance::label` (`inquiry.rs:55`) |
| `InquiryLifecycle` | `InquiryLifecycle::as_str` (`inquiry.rs:78`) |

All six are `const fn`, so the token array stays a const:

```rust
payload_variants! {
    // Six: the type names its own tokens; take them.
    Stage via Stage::as_str { Exploring, Inquiring, Drafting, Reviewing, Locked }

    // Six more: no authority exists, so this invocation is where the vocabulary
    // is named for the first time — which STD-001 permits and in fact wants.
    AgentAct {
        BlockingSetDeclared = "blocking-set-declared",
        DraftingReady       = "drafting-ready",
    }
}
```

The fourteen divide six / six / two. Six consume an authority, per the table
above. Six have none and take the naming arm — `DischargeClaim`, `DelegationAct`,
`AgentAct`, `Reviewer`, `Posture`, `Authority`. Two sit outside both arms and are
called out so neither is read as covering them:

- **`WireFacetValue`** is `Untagged` and contributes no tokens at all, so it has
  no vocabulary to take from anywhere. Its two shapes are pin 2's, not pin 4's.
- **`Dispose`** is the one genuine judgement. Its four tokens are
  `DispositionForm::as_str`'s (`inquiry.rs:163`) exactly, but that is a
  **different type** whose vocabulary merely coincides — consuming it would
  assert an equivalence nothing enforces, and a later divergence between the two
  types would then be invisible rather than merely undetected. `Dispose` names
  its own four, and `sec-8` pin 4's round trip keeps them equal to serde's.

An earlier draft of this subsection retyped every vocabulary, including all six
above, and justified it by the round-trip test — treating a drift *detector* as a
substitute for a single source. `STD-001` says the opposite in terms: when a
constant for a value already exists, use it; do not paste the raw literal beside
it. A test that catches the divergence still leaves two places to change.

The macro expands to a `VARIANTS: &[&str]` used by the contract table, plus:

```rust
const fn _pin(value: &Dispose) {
    match value {
        Dispose::Create     { .. } => {}
        Dispose::Adopt      { .. } => {}
        Dispose::Unresolved { .. } => {}
        Dispose::NonDurable { .. } => {}
    }
}
```

The braced pattern is uniform across unit, tuple and struct variants — verified
against the compiler, not assumed — so one macro shape covers every enum in the
closure regardless of what its variants carry.

This pins **both directions at compile time**:

- a variant added to the enum and not to the list makes the match
  non-exhaustive — build failure;
- a variant in the list that does not exist on the enum is an unresolved path —
  build failure.

And it costs far less than generation. The generated region is a token array and
a match with no bodies, so `R3`'s clippy blindness covers nothing that could hold
a bug. Not one existing type definition moves, so the behaviour-preservation gate
on shared machinery is satisfied by construction rather than by argument.

**It also supplies the type's name, which is why `sec-2` can stop hand-spelling
it.** The macro is invoked *with* the type, so `stringify!` on that identifier
emits `TypeContract::name` for every enum in the closure. A rename is then a
build failure at the match rather than a stale literal that every other pin
agrees with.

What it does *not* pin is that the token strings equal serde's actual renames. A
`#[serde(rename_all = "kebab-case")]` type could have its variant renamed while
the contract's tokens stay put. That is a per-variant round-trip test —
serialise a value of each variant, assert the token matches — and `sec-8` pin 4
carries it. The exhaustiveness barrier guarantees the test has a case for every
variant, which is what makes a round-trip test sufficient here and insufficient
on its own elsewhere. For the six enums that consume an existing `as_str`, this
duplicates a guarantee the incumbent types already hold — each of those doc
comments records that it "agrees with the serde rename by construction of the
test that compares them" — and pin 4 covers all fourteen uniformly rather than
carving out an exception it would have to keep current.

## The struct pin: `I9`, extended down the closure

Each of the twelve struct types in `sec-3`'s closure gets `I9`'s treatment — the
eleven with a `TypeContract` directly, `SubmissionEnvelope` through the root's
composition, which `sec-8` pin 1 sets out: an
exhaustive no-`..` literal, serialised, its key set compared against the type's
`KeyContract` rows.

`SL-249` already proved the shape works and already recorded its limit — `RV-349`
`F-4` notes that `I9` alone proves *inventory*, not *mapping*, which is why
`Declaration` carries `I10`'s behavioural matrix beside it. The same limit
applies here: the literal pin proves every key is described, not that any key is
described *correctly*. Type and presence correctness is `sec-8`'s round-trip
work.

Twelve literals is the dominant mechanical cost of this slice, and `DEC-227`
accepted it explicitly when it chose the full closure over top-level-only.

## Where the pins do not reach

Three regions, all named rather than discovered later.

**Nominal identity of a `Named` edge.** `sec-8` pin 2 compares an edge against
its target's key set, which is structural. Two closure types sharing an identical
key set would stay exchangeable; none do today, so the pin is total over the
current closure by a property of the closure rather than by construction. The
escalation — a `payload_struct!` mirroring the enum instrument, binding each
field to its declared `WireType` at the definition — is recorded in `sec-8` and
not taken.

**The injected sub-contract, and only part of it.** `sec-3`'s region is supplied
by the command tier at render time, so no pin *inside* `design_run` can see its
content. That is a smaller gap than it first looks, because `sec-3` closed most
of the seam rather than the content: `ExternRegion` is a closed leaf-side enum
resolved to its supply by exhaustive match, so *whether a named region is
supplied* is a compile error above the leaf, and a builder that derives from
`facet_fields` rather than copying it keeps the facet vocabulary current by
construction. Two things no compiler holds. Whether that mapping was also order-
and content-preserving — and whether the **kind** set is the enum's, since the
builder iterates the hand-maintained `RecordKind::ALL` and a new variant is not
forced into it (`sec-3`). Both are `sec-8` pin 5, which is two equalities for
that reason and not one.

**The published document.** Its pin is `artifact.rs`'s golden test, which
compares `render_artifact()` against the committed file read **from disk at
runtime** rather than from the embed — because `install/` is embedded with no
`rerun-if-changed`, so an incremental build over an `install/` edit serves stale
compiled bytes and would false-green the comparison (`artifact.rs:373-375`). Any
new generated asset inherits that constraint, and `sec-5` and `sec-8` both hold
to it.

## Summary of the pin ladder

| What | Mechanism | Fails at |
|---|---|---|
| Enum variant set | generated exhaustive match | compile |
| Struct field set | exhaustive no-`..` literal (`I9`) | compile |
| An enum's `TypeContract::name` | `stringify!` at the macro invocation | compile |
| A struct's `TypeContract::name` | compared against `T` in the key-set pin | test |
| Token ↔ serde rename | per-variant round trip | test |
| Wire type correctness | JSON kind, plus each `Named` edge against the target's key set (structural, not nominal — `sec-8` pin 2) | test |
| `Sparse` vs the rest | null-set equality on an all-`Null` fixture | test |
| Requiredness | per-key removal probe on the **read** path, never serialization | test |
| A variant payload's field set | exhaustive variant literal (no functional-update form), plus set-equality against the variant's rows | compile + test |
| A variant payload's wire types and requiredness | pins 2 and 3 re-run over pin 4's per-variant samples (`sec-8`, "Pins 1–3 have a second arm") | test |
| A named extern region is supplied | exhaustive match on `ExternRegion` | compile |
| The extern facet vocabulary is current | derived from `facet_fields`, never copied | compile |
| The extern **kind** set is the enum's | exhaustive match over `RecordKind` (`ALL` is hand-maintained) | test |
| The extern mapping is faithful | set-equality against `facet_fields` | test |
| Published document | golden against disk-source | test |
| No repo-private id in shipped output | citation detector, positive control first | test |
| The envelope's worked example | parses as a valid `ApplyRequest` (`DEC-228`) | test |

<!-- doctrine:section sec-5 -->
# 5. One generator, three consumers

## The shape of the split

`STD-001` forbids three renderings that restate each other, and `ADR-001` fixes
where the shared source may live: `design_run` is leaf, out-degree 0, std + serde
derive only (`layering.toml:31`). So the generator is a **pure function from the
table to strings**, and everything that embeds, publishes, or attaches to the CLI
sits above it in command tier.

This is not a posture to invent. `design_run::artifact` already renders the
published stage reference as a pure `render_artifact() -> String`
(`artifact.rs:26`), and `design_run::prompt` already names what the shell must
supply while the shell resolves it and hands it back for `contract_block` to place
(`prompt.rs:13-16, 185`). The payload contract takes the same split, with `sec-3`'s
injected region riding the same slot.

```rust
// design_run::payload_contract — leaf, pure.
pub(crate) const PAYLOAD: TypeContract;

pub(crate) fn render_json(extern_contracts: &ExternContracts) -> String;
pub(crate) fn render_prompt(extern_contracts: &ExternContracts) -> Vec<String>;
pub(crate) fn render_document(extern_contracts: &ExternContracts) -> String;

/// One field per `ExternRegion`, so omitting a region is a missing-field
/// compile error rather than a runtime `None` (`sec-3`). Not a string-keyed map:
/// that was the first draft's weakening of `prompt`'s own typed-key precedent.
pub(crate) struct ExternContracts {
    pub(crate) knowledge_record: SelectorTable,
}
```

Three entry points rather than one with a format parameter, because the three
outputs have genuinely different shapes — a JSON document, a line sequence, and a
Markdown page — and a single function returning one of three unrelated things
would be a `match` pretending to be an abstraction. They share the walk over
`PAYLOAD`, not the emission.

**The walk is the shared thing, and it is not a naive recursion.** `WireType::Named`
holds a reference to a whole `TypeContract`, and the closure is a graph rather than
a tree: `Declaration` is reached from both `ApplyRequest.declare` and
`DelegationAct::Propose.declare`, and `AcceptanceDeclaration` from five places. So
the walk collects **types by name into a flat table** and emits each once, with
every `Named` edge rendered as a name. All three renderings need that; only the
JSON was ever tempted to skip it.

## `--format json`: bespoke, and a flat table

The residue flagged at the sufficiency gate was that nobody had asked what the
renderings concretely look like. This is that decision, and the repo settles most
of it.

`design show --format json` serialises `TurnEnvelope`, whose first two fields are
`schema` (a name) and `version` (an integer, currently 1) —
`render/envelope.rs:501-502`. The contract's JSON carries the same pair, for the
same reason: a consumer that has to branch on shape should read a version rather
than sniff for keys.

**Bespoke, not JSON Schema.** JSON Schema was considered and is the wrong target
here on three counts. It would introduce a second vocabulary the contract must be
translated into, and translation is where drift lives. It conforms to a
specification this project would then owe compliance to, for a document with one
producer and one class of consumer. And it cannot say the three things `DEC-219`
spent the contract's budget on: `UnknownKeys::SilentlyDropped` has no JSON Schema
expression at all, `Presence::Sparse`'s omit-persists / null-clears semantics is
not `required`/`optional`, and serde tagging style is expressible only by
encoding it structurally, which loses the fact that it *is* the tagging style.

**Nor is it the derived `Serialize` of `PAYLOAD`.** A first draft said the JSON
*is* the `sec-2` types serialised — "the same artefact viewed twice rather than two
artefacts to reconcile" — and that does not hold for the reason above.
`Named(&'static TypeContract)` is the type, not its name, so a derived `Serialize`
inlines the target at every edge: over the real closure that is `Declaration`
twice, `CreateRecord` twice beneath those, and `AcceptanceDeclaration` five times.
The document is fat, but the disqualifying part is that a consumer cannot tell the
two `Declaration` inlinings are one type — which is the single thing a
machine-readable contract exists to tell it.

So the JSON is `{schema, version, root, types}` with `types` keyed by name, each
`Named` edge a name string, and the walk shared with `--format prompt`. The
worked fragment is in `.doctrine/slice/251/render-sample.md` §3. It carries three
keys the first draft's model had no way to produce, one per expressibility
finding: a variant that names the type it `inlines`, a token-less variant carrying
a bare `shape`, and the marker that an externally tagged unit variant is a bare
string.

One thing this design deliberately leaves open: the serde spelling of the model's
own enums. Derived defaults would emit `"Internal": "form"` and
`"SilentlyDropped"`; kebab-case throughout would match every other closed
vocabulary in the crate. Either is defensible and neither is load-bearing, so the
implementer picks — recorded here only so it is not re-derived as though it were
unsettled by oversight.

## `--format prompt`: `contract_block`'s line shape

`contract_block` already establishes the house form for contract text an agent
reads: a header line, then one line per row with field order as the commitment,
then indented detail (`prompt.rs:185-226`). A gate condition renders as

```
contract governing-context-recorded attested artefact observes(governance-edges) cumulative
  discharge: the user performs `governance-confirmed`
```

The payload contract follows it — one line per key, indentation carrying the
closure depth:

```
payload ApplyRequest  unknown-keys: silently-dropped   (a misspelt top-level key is discarded, and the command still exits 0)
  run_uid          text                 required
  known_revision   integer              required
  declare          [Declaration]        optional
  traversal        TraversalDeclaration optional

type Declaration  unknown-keys: refused
  subject          id                   required
  parent           id                   sparse   (omit persists · null clears)
  dispose          Dispose              optional

enum Dispose  tagging: internal("form")   variant keys sit BESIDE "form"
  create        → CreateRecord's keys, inlined
  adopt         record   text   required
  unresolved    note     text   required
  non-durable   note     text   required
```

Five rules fix the rest of the shape. Each exists because the closure contains a
case the four lines above do not cover, and every one of them was found by
rendering the real thing rather than by reasoning about it.

- **A variant's payload is rendered where it actually lands**, which the tagging
  and the payload together already determine (`sec-2`'s table): inlined beside the
  tag under `Internal`, nested under the token under `External`, absent under a
  unit payload. A token list alone would be the contract making this slice's own
  mistake.
- **A multi-key payload indents under its variant line**, one key per line, rather
  than wrapping. Four variants need it — `DelegationAct::Propose` and
  `Provenance::ImportedProse` carry four keys each, `Export` and `Refuse` two —
  and field order is the commitment here as it is for `contract_line`, so this is
  a rendering decision rather than a formatting preference.
- **An externally tagged unit variant is marked as a bare string**, because it is
  one. `AgentAct::DraftingReady` renders as `drafting-ready — a BARE STRING, not
  an object`; a caller who wraps it in an object has it discarded in silence.
- **An untagged variant has no token and does not pretend to one.** `WireFacetValue`
  renders its two shapes with the token column struck out, not with its Rust
  variant names, which are not on the wire.
- **`Bare` is a word this renderer prints, not a fact the table holds.** An
  external enum whose every payload is absent renders `tagging: bare`, derived per
  `sec-2` rather than stored beside the variants it could contradict.

The parenthetical on `sparse` is the one place the rendering adds words the table
does not carry, and it is a fixed string per `Presence` variant rather than per
key — so it cannot drift row by row. The `unknown-keys` parenthetical works the
same way: one fixed string per `UnknownKeys` variant.

**And no parenthetical cites a repo-private id.** An earlier draft rendered the
root's line as `(ISS-333 — a misspelt top-level key is discarded)`, which would
have shipped a doctrine-development issue id into every installed client — where
`ISS-333` is either absent or, worse, that client's own unrelated issue. This is
not a new rule to invent: `artifact.rs` already forbids it for the sibling
generated asset, listing `ISS` among fourteen repo-private prefixes and giving
the reason as ids that "resolve to an unrelated record, silently"
(`artifact.rs:325-365`). `POL-002` is the standing form of the same constraint —
shipped behaviour rests on doctrine-owned portable contracts, not on a host
project's records. So the disclosure is rendered as behaviour a caller can act
on, and the `ISS-333` citation stays where it belongs: in this design, in the
source comment on `UnknownKeys::SilentlyDropped`, and in the issue itself. `sec-8`
carries the detector as a pin, since the whole document and both renderings
inherit the constraint rather than just this one line.

## What the whole thing costs, measured

`R5` asked whether the render is too large to be useful. It is not, and this is no
longer an estimate: the full closure was rendered ahead of implementation
(`.doctrine/slice/251/render-sample.md` §2) and comes to **202 lines / 8 826
bytes** — 66 lines of structs, 70 of enums, 40 of the injected region, 26 blank.

Two things follow, and the first was stated wrongly in an earlier draft. It read
"~8.6× the 1024-byte cap the turn envelope already runs", and concluded that the
contract *could not* ride the envelope, making `sec-6`'s address-not-body choice
forced rather than preferred. The denominator was the wrong constant.
`ENVELOPE_DECLARATION_EXAMPLE_BYTES = 1024` caps the **worked example**, one
no-drop line (`render/mod.rs:117-124`); the envelope itself runs
`ENVELOPE_NORMAL_BUDGET_BYTES = 24576` (`render/mod.rs:126-134`) — which this
design already cites correctly two sections over, when pricing the 46-byte
pointer (`sec-6`).

Against the real ceiling, 8 826 B is **36% of the envelope budget**, and the
sketch's saturated table already totals 15 576 B. So the body would *fit*, with
about 174 B to spare at saturation, and the choice is **preferred, not forced**.
It is still the right choice, on grounds that do not need a false number: the
address costs 46 B on every turn where the body would cost 8 826 B on every turn,
for content a caller needs once per session and can fetch on demand; and
`DEC-064`'s budget exists to keep headroom available for design alternatives
rather than to be consumed by the largest thing that fits. Saying *forced* when
the honest answer is *preferred* would have made the decision look inevitable and
left nobody able to re-open it on evidence — which is exactly what `OQ-B` is for.

Second, there is no case for trimming — the cheapest fifth to cut is the injected
region, which is the one part a caller cannot recover any other way and the only
part that is wholly undocumented today.

The one presentational thing the measurement did surface: the root's header line
runs to 104 columns, over the 100 the rest of the corpus holds to. Shorten the
parenthetical, not the disclosure.

## The published document: `artifact.rs`, followed exactly

`DEC-224` chose the library doc and `DEC-226` sealed it. The mechanics are
prescribed by the existing exemplar rather than designed here.

- **A path constant** — `PAYLOAD_CONTRACT_PATH: &str = "install/design-payload-contract.md"`,
  on `ARTIFACT_PATH`'s pattern (`artifact.rs:23`).
- **Three distinct addresses for one artefact**, which is load-bearing and not
  redundancy: the disk/embed file (`install/design-payload-contract.md`), the
  manifest `backing` (the embed key, `design-payload-contract.md`), and the
  manifest `address` (the stable logical name,
  `reference/design-payload-contract.md`). `backing` may move when the source
  tree is reorganised; `address` may not.
- **A manifest row** with all seven required fields — `address`, `backing`,
  `kind`, `title`, `licence`, `provenance`, `customization` — every one fail-closed
  at admission, nothing silently defaulted (`publication.rs:371-429`).
  `customization = "fixed"` per `DEC-226`; `kind = "reference"`.
- **A golden test** comparing `render_document()` against the committed file read
  **from disk at runtime** via `CARGO_MANIFEST_DIR`, never from the embed. This
  is not fussiness: `install/` is embedded with no `rerun-if-changed`, so an
  incremental build over an `install/` edit serves stale compiled bytes and a
  comparison against the embed would false-green (`artifact.rs:373-375`).
- **An `#[ignore]`d regeneration test** that writes the file, with the golden's
  failure message naming it. Re-rendering is a dev act in this repository and
  meaningless in a client project, where the file is a read-only artefact of an
  installed binary — which is the same fact `DEC-226` sealed the doc for.

**Publication is compelled, not elective.** `assert_unprojected_install_assets_are_published`
(`asset_source.rs:132`) already forces every new `install/` asset to be published
or flagged, so the manifest row is a build-gate obligation rather than a step this
design is asking someone to remember.

`.doctrine/slice/251/render-sample.md` is **not** this document and must not be
mistaken for its golden. It predates the generator, was written by hand from
source, and will differ from `render_document()`'s output in whitespace and
ordering. It is evidence for `R5` and for the findings it raised; the golden is
`render_document()` against the committed `install/` file, and nothing else.

## The `--help` pointer is not a fourth rendering

`DEC-224` reduced `design apply --help` to a one-line pointer, and that line is a
plain `&'static str` in the clap attribute — not generated, not walked from
`PAYLOAD`.

This is not a `STD-001` exception. `STD-001` forbids two sources restating the
same content; a pointer restates no content. It names an address, and an address
that goes wrong is a build failure at the verb name rather than silent drift in a
description. The moment the help text tried to enumerate anything, it would need
the generator — which is exactly why `DEC-224` declined the enumerating form.

## What the command tier owns

Four things, all above the leaf:

1. building `ExternContracts` from `RecordKind::ALL` and `facet_fields` (`sec-3`),
   which requires importing both `design_run` and `knowledge` and is therefore
   only possible here — and building it by derivation rather than by hand, which
   is what makes the supply and the facet vocabulary compile barriers, leaving the
   kind list to `sec-8` pin 5 because `ALL` is hand-maintained;
2. the `DesignCommand::Contract` variant, its clap `Args`, its `dispatch` arm and
   its `guard.rs:430` classification — the last two being compile errors until
   written;
3. embedding and publishing the document;
4. wrapping the parse refusal at `design.rs:1504` with the contract's address
   (`sec-6`).

<!-- doctrine:section sec-6 -->
# 6. Reaching the contract

`sec-5` built the pull surface. This section is the push: the two points where
the contract's *address* travels to a caller who did not ask, plus where the verb
attaches on the CLI (`inq-3`).

## The two push points cover disjoint failures

`DEC-225` argues push over pull from `refusal.rs:483`'s own words — a remedy
travels with a refusal so a caller can act *without fetching anything*. The
sharper reason for taking **both** insertion points is that neither reaches the
other's failure.

**A nested misspelling is refused.** `Declaration` (`submission.rs:123`),
`CheckpointActDeclaration` (824) and `AgentActDeclaration` (854) all carry
`deny_unknown_fields`, so a key at the wrong subject, or an act payload written
by analogy with the wrong sibling's tagging, produces a serde error. There *is* a
refusal to attach a remedy to, and the parse-site wrap is what attaches it.

**Everywhere else, a misspelling is not refused at all.** Those three are the
only wire structs that deny unknown fields; the other nine accept and discard —
`ApplyRequest` itself, where `serde(flatten)` forbids the attribute (`ISS-333`),
and eight more that simply lack it, `TraversalDeclaration` and its `cursor`
among them (`sec-2`). The revision bumps, a receipt is written, no change row
prints, and the command exits 0. **No refusal is ever emitted**, so the parse-site
seam is structurally unable to reach three quarters of the payload. The only
surface that reaches a caller who is not being refused is one that rides every
turn.

So the envelope pointer is not redundancy against the refusal. It is the sole
push coverage of the silent-discard mode — nine of twelve wire structs — for as
long as that mode stands, which is the whole of this slice's horizon, since
repairing it is an explicit Non-Goal. `sec-2`'s `UnknownKeys::SilentlyDropped` is the same disclosure on the
pull side; the two are one commitment rendered at two surfaces.

## Point 1 — the parse refusal

`design.rs:1504` today:

```rust
let request: ApplyRequest =
    serde_json::from_str(payload).context("parse the apply payload as JSON")?;
```

becomes one `map_err`:

```rust
let request: ApplyRequest = serde_json::from_str(payload).map_err(|error| {
    anyhow::anyhow!("parse the apply payload as JSON: {error}\n  {PAYLOAD_CONTRACT_POINTER}")
})?;
```

**The site itself is clean, and this was checked rather than assumed.** The parse
runs before `design_run::run::admit` and before anything writes, and `admit`
(`run.rs:207`) is pure — it reads `prior.receipts` and returns. A top-level parse
failure therefore mints no receipt and bumps no revision, and wrapping its
refusal carries no state concern with it. Worth stating because `ISS-361` reads
as though an unparseable submission had been applied and receipt-locked here,
which cannot happen at this site. The behaviour that fits is a parse that
*succeeded* through `ISS-333`'s hole, followed by a corrected resubmission
reusing its `submission_id` and meeting `Refusal::SubmissionReplayed`
(`run.rs:222-226`) — the idempotency guard working as designed. That refusal
naming no remedy is a real defect and stays with `IMP-390`'s fourth candidate,
which this slice explicitly does not take.

Three properties, each deliberate.

**The serde message survives verbatim.** It already carries the most specific
information available — `unknown field 'cursror'`, `unknown variant 'act'`, and
the type it happened at. The wrap adds an address; it does not paraphrase, filter
or classify, because a classifier over serde's message text would be a parser of
an unstable format for no gain.

**The shape matches a gate refusal.** `Refusal::GateNotCleared` writes a first
line then indented detail (`refusal.rs:487-497`). The pointer takes the same
indented-continuation form, so the payload refusal joins the gate refusals in
*form* as well as in having a remedy — one reading habit, not two.

**It names an invocation, not a topic.** `doctrine design contract --format
prompt`, complete enough to paste. "See the payload contract" would leave the
caller a search, which is the state this slice exists to end.

## Point 2 — the turn envelope

Beside `DECLARATION_EXAMPLE`, as `DEC-225` specifies: a `&'static str` field on
`TurnEnvelope` (`envelope.rs:358`), fed by the same const, and one `lines.push`
after the `declare` line (`envelope.rs:1224`).

```
truncated false
declare {"run_uid":"<uid>", … }  (omit a key to persist it, …)
contract doctrine design contract --format prompt
```

**A field, not an append to the example.** Folding the pointer into
`DECLARATION_EXAMPLE`'s trailing parenthetical would cost nothing structurally
and is still wrong twice over: it puts an address inside a constant whose 1024-byte
assertion exists to keep the *example* honest, and it makes the JSON rendering's
`declaration_example` two things at once for any consumer reading it as an
example.

**An additive field is not the wire change `A1` forbids.** The precedent is in
this same struct: `SL-244` added `outstanding` and `pass_stale` at
`TURN_ENVELOPE_VERSION = 1` (`envelope.rs:59`, `338-343`). The version
discriminates *meaning*, and a new key changes no existing key's meaning. What
`A1` forbids is the contract **body** riding the turn, and `sec-1` already fixes
that boundary: the address rides, the body does not.

**Un-elidable by construction.** The omission ladder operates on capped lists via
`Omitted`; a line the renderer pushes unconditionally is outside that machinery
entirely, exactly as `declare` is. It needs no no-drop registration, only the same
placement. Cost is ~46 bytes against `ENVELOPE_NORMAL_BUDGET_BYTES = 24576`
(`render/mod.rs:134`).

## Point 3 — `design apply --help`

`DEC-224` reduced this to a pointer, and `sec-5` already argued why a pointer is
not a `STD-001` exception. What remains is where the bytes live.

The pointer belongs in `Apply`'s long help (`commands/design.rs:124-125`), which
is where a caller reading about `--input` is already standing. Clap separates the
two altitudes — `about` for `doctrine design --help`'s table, `long_about` for
`doctrine design apply --help` — so the pointer appears at the verb without
widening the table above it.

**The address is single-sourced, and an earlier draft's reason for not doing so
did not survive checking.** That draft kept a second hand-typed copy, on cost. It
held that `long_about` *replaces* a doc comment rather than extending it, and
that `concat!` cannot splice a `const &str` — both true — and concluded that
single-sourcing therefore meant moving `Apply`'s entire long help behind a
function returning `&'static str`: more machinery than the pin it removes, for
one line on one verb.

The conclusion does not follow from those premises. `long_about` takes any
expression. `clap_derive` forwards `key = expr` as a method call (`compare.rs:75`
is the in-crate proof, passing a `clap::ArgGroup`), `Command::long_about` accepts
`impl IntoResettable<StyledStr>`, and `String` satisfies that through the blanket
`impl<I: Into<StyledStr>>` (`resettable.rs:190`, `styled_str.rs:161`). A
`format!` is the whole mechanism, and no function has to exist:

```rust
const APPLY_ABOUT: &str = "Validate and apply one sparse idempotent mutation.";

#[command(
    about = APPLY_ABOUT,
    long_about = format!("{APPLY_ABOUT}\n\n{PAYLOAD_CONTRACT_POINTER}"),
)]
Apply(ApplyArgs),
```

The real cost is that one verb in `commands/design.rs` takes its help from two
attributes rather than a doc comment. That is the whole price, and it buys the
address exactly one home.

**`STD-001` was never available to trade against that price.** The standard is
`required`, and it says *when a constant for a value already exists, use it; do
not paste the raw literal beside it*. Its single exclusion is a one-shot literal
with no sibling constant, which this is not. The earlier draft's closing claim —
that `STD-001`'s force "relocates to a pin" — has no basis in the standard's
text: a drift detector reports a divergence, it does not remove the second place
to change.

**This is `A3`'s error class, second instance.** `A3` was retired for asserting,
unchecked, that clap's derive cannot take an expression. The capability claim got
corrected; the *cost estimate* resting on the same unverified reading of the
macro did not, and it rode into the finished draft one subsection later. `sec-9`
records the pattern, because the lesson is about the habit rather than about clap.

`sec-8` pin 7 keeps a help rung, but a smaller one. With the address
single-sourced there is no drift left to detect, so what remains is the ordinary
assertion that the pointer reaches the rendered help:
`render_subcommand_help(&["design", "apply"], …)` contains
`PAYLOAD_CONTRACT_POINTER`. The seam already exists, carries **five**
`help_snapshot_*` users (`main.rs:889-970`) across sixteen
`render_subcommand_help` call sites, and one of those already takes the
two-element path form this needs (`main.rs:1120`, `["memory", "sync"]`).

## Where the pointer lives

One const, in the leaf beside the renderer that carries it:

```rust
// design_run::payload_contract
pub(crate) const PAYLOAD_CONTRACT_POINTER: &str = "doctrine design contract --format prompt";
```

`design_run` is leaf and `commands::design` is command tier, so the refusal wrap
reads it in the direction the layering already allows (`ADR-001`); no edge is
created in either direction.

`--format prompt` is in the pointer rather than left off, because the reader being
pointed is an agent that wants text, and a caller who wants JSON knows to ask for
it.

## Where the verb attaches

`DesignCommand::Contract(ContractArgs)`, a fifth variant beside the existing four
(`commands/design.rs:118-130`), with its arm in the dispatch match (`242-246`) and
its classification in `guard.rs:430`.

**`Read`-classed.** It touches neither the runtime nor the authored tier, which is
the split the existing comment there already states. Two consequences worth
naming: the worker-mode guard refuses `Write`-classed verbs by process, so
`Read` is what lets a confined worker fetch the contract at all; and both the
dispatch arm and the guard arm are non-exhaustive-match compile errors until
written, so neither classification is something an implementer can forget.

**No slice, no root.** Nothing in `PAYLOAD` or in the injected `Extern` contracts
depends on a snapshot — the contract is a property of the binary, not of a run. So
`ContractArgs` carries `--format` alone: no positional slice, and no `-p/--path`,
which every other verb in the file needs only to resolve a project root this one
never reads. It therefore answers in a project with no design run, and outside a
doctrine project entirely.

**A two-member format enum.** `ContractFormat { Prompt, Json }`, defaulting to
`Prompt`, rather than reusing `ShowFormat` (`commands/design.rs:138-147`) — that
enum's third member, `status`, is "the same envelope, for a human at a terminal",
and there is no per-run status to render for a static document. Admitting a token
the renderer must then refuse is the failure this slice is about.

## What is deliberately not a push point

**`resume`'s `contract_section` channel.** `DEC-225` weighed injecting the body
there and declined it: it spends the budget `DEC-064` protects to save one
invocation. Declined, not foreclosed — it is the recorded escalation if cold
re-entry proves to fail before fetching, and `contract_section`'s suppressible
`--known-*` block is the seam it would ride.

**The library document's address.** It is the stumble-onto surface (`DEC-224`),
reached from the index by someone who does not know the verb exists. Pushing a
second address for identical content would hand every refused caller a choice
between two routes to one answer, which is a cost with no matching benefit.

<!-- doctrine:section sec-7 -->
# 7. Code impact

## The whole touch-set

| Path | Change | Rough size |
|---|---|---|
| `src/design_run/payload_contract.rs` | **new** — the model types (`sec-2`), `payload_variants!` and its per-enum invocations (`sec-4`), the `PAYLOAD` table, the three renderers (`sec-5`), `PAYLOAD_CONTRACT_POINTER` and `PAYLOAD_CONTRACT_PATH`, plus the `#[cfg(test)]` per-variant sample values pins 1–4 consume (`sec-8`) | ~500 table, ~250 renderer, ~120 test-only |
| `src/design_run/mod.rs` | one `pub(crate) mod payload_contract;` | 1 |
| `src/design_run/render/envelope.rs` | `contract_pointer` field, its projection, one pushed line (`sec-6`) | ~10 |
| `src/commands/design.rs` | `Contract` variant, `ContractArgs`, `ContractFormat`, `run_contract`, `extern_contracts()`, `APPLY_ABOUT` and `Apply`'s `about` / `long_about` attributes carrying the pointer (`sec-6`), the `map_err` at 1504 | ~70 |
| `src/commands/guard.rs` | `Contract(_) => Read` | 1 |
| `install/design-payload-contract.md` | **new** — the generated document | rendered |
| `publication/manifest.toml` | one `[[entry]]` | 7 |
| `src/design_run/submission.rs` | `#[cfg(test)] fully_populated()` beside each wire struct it defines | ~250 test-only |
| `src/design_run/attestation.rs` | the same, for the closure structs defined here | ~40 test-only |
| `src/design_run/tests.rs` + in-module tests | the pin ladder (`sec-8`) | ~400 |
| `src/main.rs` tests | the help-pointer pin and the guard classification row (`sec-8`) | ~15 |

**No wire type's definition or behaviour changes — but two of their files are
still written.** An earlier draft of this table listed `submission.rs` and
`attestation.rs` as *read, not written* and omitted them entirely, which
contradicted `sec-8` pin 1 in the same design: the fixtures it calls for sit
"beside each type's definition", on `Declaration::fully_populated`'s precedent —
and that precedent is a `#[cfg(test)] pub(super) fn` inside `submission.rs`
(`submission.rs:653`), not something in `tests.rs`. A plan sequenced off the
omission would have allocated no work to the files most of the closure lives in.

Every addition to those two files is `#[cfg(test)]`. Not one wire type's
definition moves and no production path changes, so the behaviour-preservation
gate on shared machinery is vacuous here rather than argued — there is no
behaviour to preserve because none is touched. The distinction the table draws is
*file written* versus *behaviour changed*; collapsing them is what lost the rows.

**Every `payload_variants!` invocation lives in `payload_contract.rs`, and only
there.** An intermediate draft scattered them across the five modules that define
the closure's enums, which contradicted the new module's own row and split macro
ownership across six files. There is no reason to scatter them: the invocation
needs the type in scope, which an intra-module `use` supplies, and the six
`as_str` authorities it consumes are `pub(crate)`. Keeping them beside the
`PAYLOAD` table that reads their output is both simpler and the only arrangement
that lets a reader find the whole vocabulary in one place.

## Where the new module sits

`design_run::payload_contract` is a **sibling of `artifact.rs`, not more of
`submission.rs`**. `artifact.rs`'s own module doc draws the line this follows:
that module is about *describing* where `gate` is about *gating*, and rendering a
string is pure, so the layering is untouched. `submission.rs` holds the wire
types; a description of them is a second responsibility, and putting it in the
same file would give `submission.rs` two.

Not under `render/` either: that tree projects a *run* into a turn envelope, and
everything in it is keyed to a snapshot. The payload contract has no run — which
is exactly the property `sec-6` spends on the verb needing no slice and no root.

Out-degree stays inside `design_run`. `payload_contract.rs` `use`s `submission`,
`attestation`, `inquiry`, `traversal` and `mod` — for the `payload_variants!`
match arms and the six `as_str` authorities they consume (`sec-4`) — and nothing
else. `layering.toml:31` keeps
`design_run = "leaf"  # out=0` unchanged, because an intra-module `use` creates no
crate-level edge (`ADR-001`).

## The model change `sec-3`'s injection forces: none

An intermediate draft of this subsection carried a real one. `sec-3` used to
build the injected region as a `TypeContract`, whose row slices are assembled
while the process runs and therefore could not be `&'static`; the model grew
`Cow` at three positions, two `const fn` constructors to keep `Cow` out of the
table's authoring site, and a rejected-alternatives paragraph weighing `Box::leak`
against const-fn array assembly.

All of it is gone, because `sec-3` no longer builds a `TypeContract`. Its
`SelectorTable` is a type of its own that owns its rows, so nothing the command
tier assembles is ever a member of `TypeForm` or reachable through a `Named`
edge. The const table stays entirely borrowed, `TypeForm` keeps
`&'static [KeyContract]` and `&'static [VariantContract]`, and the acyclicity
argument in `sec-3` keeps its mechanical basis unchanged.

The lesson is worth keeping even though the machinery is not: the `Cow` was the
cost of describing the extern region as something it is not. Modelling it
honestly did not trade one complexity for another — it removed the complexity
along with the misdescription.

Two `WireType` positions still carry the region:

```rust
Token(TokenSource),                       // a string field with a closed vocabulary
Map { key: MapKey, value: &'static WireType },
```

A facet field's `FieldShape::Closed(tokens)` is a plain string on the wire whose
admissible values are a closed set with no Rust type name visible to a caller, so
it is a `Token` with a `Fixed` source. `CreateRecord::kind` is the same shape with
an `Extern` source, which is what lets the seven kinds reach a caller without
`design_run` naming an enum it may not import — modelling either as `Named`
pointing at an invented type would state something false about the wire.

## What the command tier assembles

```rust
/// The only place `design_run` and `knowledge` are both in scope (ADR-001).
fn extern_contracts() -> ExternContracts;
```

One field, `ExternRegion::KnowledgeRecord`: a `SelectorTable` with one
`SelectedKeys` row per member of `RecordKind::ALL` (`knowledge.rs:161`), each
carrying that kind's `facet_fields` rows in template order, mapped
`Text → WireType::Text`, `List → Seq(Text)`, `Closed(tokens) → Token(Fixed(tokens))`,
and `unknown_keys: Refused` on the whole table — the facet check rejects an
unknown key before the mint (`sec-3`). Around 25 lines, no branching beyond that
three-arm map.

Two properties of those 25 lines are load-bearing rather than incidental. It
**iterates** `RecordKind::ALL` and calls `facet_fields` rather than restating
either, so a new *facet field* arrives without an edit here. And the `FieldShape`
map is written **without a wildcard arm**, so a new shape is a compile error at
this function rather than a silent `Text`. A reviewer who "simplifies" either one
has removed a barrier `sec-8` pin 5 is sized against.

**Iterating `ALL` is not itself a barrier, and `sec-3` says why.** `ALL` is
hand-maintained (`knowledge.rs:159-169`), so a new *kind* does not arrive here
automatically: it is forced into `facet_fields` by that function's exhaustive
match and is not forced into `ALL`. This function would then emit a table missing
it, silently. That residue is carried by `sec-8` pin 5's kind-set equality, whose
oracle is an exhaustive match over `RecordKind` and never `ALL`. Deriving `ALL`
from the enum would close it at the source and is the knowledge tier's to make,
not this slice's.

`run_contract` is then: assemble, dispatch on `ContractFormat` to `render_json` or
`render_prompt`, `emit`. No root, no snapshot, no I/O (`sec-6`).

## Coupling, for whoever sequences this

Three chains, stated as dependency rather than as a plan — phase boundaries are
`/plan`'s call.

- The **table** is upstream of everything: the three renderers, the document and
  every pin read it.
- The **document** is downstream of `render_document` and of the manifest entry
  together; `asset_source.rs:132` refuses an `install/` asset that is neither
  published nor flagged, so the asset and its manifest row land as one thing.
- The **pointer** (`sec-6`) shares only the const with the rest. It is independent
  of the table, the renderers and the document in both directions.

## Governance touchpoints

- **`SPEC-029`** names the verb set as *start, show, apply, resume*. A fifth verb
  makes that line stale — a prose update at reconcile, not an amendment
  (`DEC-224`).
- **`publication/manifest.toml`** gains one entry with all seven required fields
  and `customization = "fixed"` (`DEC-226`). Compelled, not elective:
  `asset_source.rs:132` is a build gate.
- **`ADR-019`** is engaged on the publication leg only. `install/` is already a
  RustEmbed root, so no `flake.nix` graft — but note the standing hazard from
  `AGENTS.md`: `craneLib.cleanCargoSource` strips non-Rust assets, and the graft
  is what keeps an embed root intact under `nix build`. Adding a file to an
  existing root inherits the existing graft; adding a *root* would not.
- **`layering.toml`** unchanged, per above.
- **`ADR-001` / `STD-001` / `POL-002`** are the slice's `governed_by` set and all
  three bind here. `ADR-001`: leaf purity, and the extern region injected as data
  from the one tier that may import both (`sec-2`, `sec-3`, `sec-5`). `STD-001`:
  one generator for three renderings, the six existing `as_str` authorities
  consumed rather than retyped (`sec-4`), and the contract pointer named once and
  read by all three surfaces (`sec-6`) — no rung of this slice spells a value
  twice and pins the copies. `POL-002`: nothing host-project-specific in the
  shipped document or either rendering, which is why no repo-private id reaches
  them (`sec-5`, `sec-8` pin 10).

## Corrections this slice owes elsewhere

Neither is a code change, and both are recorded here so they are not rediscovered
at close.

- `mem.fact.design-run.apply-payload-vocabulary` states that neither
  `ApplyRequest` nor `Declaration` denies unknown fields. The second half is
  wrong — `Declaration` carries `deny_unknown_fields` (`submission.rs:123`), which
  is `sec-6`'s whole disjoint-failures argument. Correct at harvest.
- `ISS-346` and `ISS-333` record the same silent-discard defect (minted
  2026-08-12 and 2026-08-09). Merge before closure so the option-3 discharge lands
  on one id.

<!-- doctrine:section sec-8 -->
# 8. Verification

`sec-4` fixed which property each rung of the ladder holds and where it fails.
This section names the tests, their oracles, and the two places drafting found
that the ladder as stated does not reach.

## Four things need no test

The compile barriers from `sec-4`, `sec-5` and `sec-6`, restated as the evidence
they are: `payload_variants!`'s dead exhaustive match over each enum; the
exhaustive no-`..` literals behind the key-set pin; the `Contract` arms in
`commands/design.rs`'s dispatch and `guard.rs:430`, both non-exhaustive-match
errors until written; and — since `sec-3` closed the extern seam — the
`ExternRegion` → `ExternContracts` resolution, where naming a region the shell
does not supply is a missing-field or non-exhaustive-match error rather than a
runtime `None`. None is a test, and each fails before one can run.

## The oracle discipline

Every pin below takes its oracle from **the types themselves** — a serialised
value, or a table the compiler proved complete. Never a second hand-written list:
that is tier 1, which the research excluded because it is blind to exactly the
failure it claims to prevent.

Where an assertion is a set-equality that would pass vacuously if its extraction
broke, it carries a positive control first, on `the_artefact_cites_no_repo_private_id`'s
pattern (`artifact.rs:360-370`) — assert the detector fires on a known-bad input,
then assert the real input is clean.

**And a fixture that leaves a container empty stops testing without failing.**
Pin 2 reads element types *through the value*, so a `Seq(inner)` or `Map` row is
exercised only where the fixture actually put something in it — and the precedent
this design builds on does not: `Declaration::fully_populated` sets
`needs: Sparse::Value(Vec::new())` (`submission.rs:657`), and its doc says the
values are arbitrary because `I9` observed each key's *presence* alone. True of
`I9`, false of this ladder. So *fully populated* carries a second obligation
beside the no-`..` literal: **every `Seq` and `Map` row holds at least one
element, in every struct fixture and every variant sample**, and the walk
**asserts** non-emptiness on those rows rather than trusting it — an emptied
container is then a failure rather than a check that quietly stopped running.
`Declaration::fully_populated`'s value and its doc sentence both change with it,
in the one place `I9` and this ladder share the literal (`tests.rs:2908`).

**A *row*, not a key** — the obligation is stated over where a container is
*declared*, because a container is not always behind a key. `WireFacetValue`'s
`List` arm is `VariantPayload::Shape(Seq(Text))` (`sec-2`): untagged, so it is a
shape rather than a set of keys, and pin 4 hands both its shapes to pin 2. A
key-scoped rule would never reach it, and `WireFacetValue::List(vec![])` would
satisfy `Shape(Seq(Integer))` exactly as well as `Shape(Seq(Text))`. The rule
therefore binds every `Seq` and `Map` **wherever it is declared** — a
`KeyContract`'s `ty`, an untagged variant's `Shape`, or a container nested inside
either — which closes the case by construction rather than by an inventory of
today's rows.

## 1 — Key sets: eleven types one way, the twelfth another

```rust
fn assert_keys_described<T: Serialize>(value: &T, contract: &TypeContract);
```

One body, **eleven** call sites. Each type gains a `#[cfg(test)] fn
fully_populated()` beside its definition, on `Declaration::fully_populated`'s
precedent (`submission.rs:653`) — an exhaustive literal with no `..`, so a new
field is a compile error at the fixture before it can be a missing contract row.
Those fixtures live in `submission.rs` and `attestation.rs` beside the types they
populate, which is why `sec-7`'s touch-set lists both files as written.

**The same call site also pins the type's name**, since it is the one place that
holds both the contract and `T`. `contract.name` is compared against `T`'s own
name, so a Rust rename fails here rather than leaving a stale literal that every
`Named` edge and every JSON key agrees with (`sec-2`). The enums need no
equivalent — `payload_variants!` emits their names by `stringify!` at the
invocation, which is a compile barrier rather than a test.

**`SubmissionEnvelope` is the twelfth member and cannot take that call site**, and
an earlier draft claimed it could. The assertion is a set-equality between a
serialised value's keys and a `TypeContract`'s rows, and `SubmissionEnvelope` has
no `TypeContract`: `sec-2` places its three keys at the root and renders eleven
blocks for twelve members precisely because of the flatten. There is no contract
argument to pass — `PAYLOAD` would be three keys against thirteen rows. Its serde
surface is still a real three-key object, so what is well defined over it is the
root's *composition*, and that is the assertion it gets:

```
{SubmissionEnvelope's keys} ⊎ {the ten act keys} == {PAYLOAD's thirteen rows}
```

A disjoint union, so the pin fails if the flatten ever starts colliding with an
act field as well as if either side loses a key. It is the same property one rung
of the ladder up — that the root's row list is total — reached the only way the
flatten leaves open. Any pin or claim in this design that counts *blocks* rather
than *members* has to say which; this one needs both, and says so.

**`Declaration`'s fixture is reused, not copied.** `I9`
(`tests.rs:2908`) and the contract's pin consume the same literal, so the two
cannot disagree about what a full declaration is. A second literal beside the
first would be the drift this slice exists to prevent, reintroduced in the tests.

Catches: a field added, removed or renamed with no row; a row naming a key no
field emits. `resolved_record` stays out of both sides by `#[serde(skip)]`,
by construction rather than exception.

## 2 — Wire types: one walk, no new fixtures

Serialise the same `fully_populated` value and compare each key's JSON kind
against its declared `WireType`: string against `Text` / `Id` / `Token`, number
against `Integer`, bool against `Boolean`, array against `Seq`, object against
`Named` / `Map`.

**The walk is recursive over the value, not a pass over its top-level keys.** A
`Seq(inner)` row checks *every element* against `inner`, and a `Map` row checks
every value against the declared value type — otherwise `declare`'s
`Seq(Named(Declaration))` says only "array", and any named target would satisfy
it. It recurses the same way into an untagged variant's `Shape`, which is the
other place a container is declared. Non-emptiness is what keeps all of these
from being vacuous, which is why the oracle discipline above makes it a fixture
obligation and asserts it.

A `Map` row is now two claims rather than one, because `sec-2` gave the key a
description: the object's **values** match the declared value type, and its
**keys** match `MapKey`. Under `MapKey::Of(Id)` — `AdoptAuthored.sections` — every
key parses as a `DesignId`. Under `MapKey::Extern` the keys are checked by pin 5,
not here, because the fixture cannot know which kind was selected.

Catches the commonest table defect — a row whose `ty` was copied from the row
above it.

**A JSON kind alone is too coarse for two of the eight `WireType`s, and an
earlier draft stopped there.** It conceded that `Text`, `Id` and `Token` are all
strings and pointed at the fixture's `DesignId::parse` and at pin 4 — but parsing
the fixture's *value* as a `DesignId` says the value is an id, not that the row
chose `Id` over `Text`, nor that the `IdKind` slice it advertises is the set the
engine actually admits. And every `Named` target serialises as an object, so
swapping `ApplyRequest.traversal` and `stage` between `TraversalDeclaration` and
`StageDeclaration` (`submission.rs:926-932`) leaves the JSON kinds identical, pin
1's key sets identical, and pin 9's names resolving and reachable. A materially
false machine-readable contract stayed green under all of them.

Two additions close it, both riding oracles already in the tree.

- **A `Named` edge is checked against the target's key set**, not merely against
  the name table: the walk recurses into the object and compares its keys with
  the target `TypeContract`'s rows. The `traversal`/`stage` swap then fails,
  because `TraversalDeclaration`'s four keys are not `StageDeclaration`'s two.
  This is pin 1's assertion applied at the edge rather than at the type, so it
  costs a call and no new fixture.

  **It is a structural check, not a nominal one, and the difference is the
  residue.** Two closure types with *identical* key sets would remain
  exchangeable under it. None are today — the eleven contract-bearing structs
  have eleven distinct key-name sets — so the pin is total over the current
  closure, but it is total by a property of the closure rather than by
  construction, and a future type that happens to mirror another's keys would
  reopen it. Closing it properly means a `payload_struct!` mirroring `sec-4`'s
  enum instrument, binding each field to its declared `WireType` at the
  definition so the row is compile-pinned. That is real machinery for a hazard
  that is currently theoretical, so it is recorded here as the escalation and not
  taken.
- **`Id` rows are checked against `IdKind::declarable`** (`ids.rs:70-90`) where
  the engine's admissible set is already computed, rather than against a slice
  the table asserts about itself. `declare` admits five of the eight and
  `AdoptAuthored.sections` admits `sec-` alone (`sec-2`), and those are the two
  claims worth failing on.

`Token` stays pin 4's, which reads the tagging and payload together.

## 3 — Presence: the all-null fixture

`Sparse` serialises `Omitted` and `Null` identically — both `serialize_none`
(`submission.rs:74-82`) — but `skip_serializing_if = "Sparse::is_omitted"` drops
only `Omitted`, while `Option` fields drop on `is_none`. So a fixture with every
`Sparse` field set to `Sparse::Null` and every `Option` field `Some` emits JSON
`null` for **exactly** the sparse keys.

- `{keys serialising to null}` == `{keys declared `Sparse`}`
- `{keys whose removal makes the payload fail to deserialise}` ==
  `{keys declared `Required`}`

**The second assertion reads deserialization, and two earlier drafts read
serialization instead.** The first wrote `{keys present in a minimal value}` ⊇
`{Required}`, which catches an optional field falsely declared `Required` and
misses the converse: `SubmissionEnvelope`'s `run_uid`, `known_revision` and
`submission_id` are plain non-`Option` fields (`submission.rs:686-690`), so a
minimal `ApplyRequest` emits all three whatever the contract claims — re-declare
one as `Optional` and the superset still holds while the contract tells a caller
they may omit the compare-and-swap key. Tightening that to an equality fixed the
direction and broke the oracle: `ApplyRequest.declare` carries `#[serde(default)]`
with **no** `skip_serializing_if` (`submission.rs:939-940`), so it serialises as
`[]` in a minimal value while being genuinely optional on input, and the equality
would fail on a correct table.

Serialization was never the right question. *Required* is a property of the
**read** path — a key is required exactly when omitting it makes `from_value`
fail — so the pin asks that directly: take the `fully_populated` value's JSON,
remove one key, attempt the deserialise, and collect the keys whose removal
refuses. `declare` and every other defaulted field fall out on the correct side
by construction, because a default is precisely what makes removal succeed. It
also needs no new fixture and no minimal value.

Two `Sparse` fixtures, not twelve: only `Declaration` (`submission.rs:126-133`)
and `TraversalDeclaration` (`731-736`) carry `Sparse<T>`, and the removal probe
reads the same `fully_populated` values every other pin uses.

## Pins 1–3 have a second arm: the variant payloads

`KeyContract` rows are not a struct's alone. `VariantPayload::Keys` carries the
same row type (`sec-2`), so `DelegationAct::Propose` declares a `key`, a `ty` and
a `presence` for each of `id`, `by`, `summary` and `declare` exactly as
`Declaration` does for its fields — and the three assertions above reach none of
them, because all three range over the twelve `fully_populated` **struct** values.
A contract row with no oracle over it is the defect this ladder exists to
prevent, so the arm is stated here rather than left to implementation to notice.

It needs no new fixture. Pin 4 already constructs one sample value per variant
for all fourteen enums, and each sample is a value of its own type, so the same
three questions are well defined over it:

- **key set** — the sample's serialised keys == the variant's rows (pin 1's
  assertion, at the variant);
- **wire type** — each key's JSON kind against its declared `WireType`, and each
  `Named` edge against its target's key set (pin 2's walk, unchanged);
- **requiredness** — remove one key from the sample's JSON, attempt
  `from_value::<TheEnum>`, and collect the keys whose removal refuses; that set ==
  the variant's `Required` rows (pin 3's probe, on the same read path).

Two mechanics the arm has to name. **The tag is not a row**: under
`Tagging::Internal(tag)` the tag key belongs to pin 4 and the walk skips it, and
under `External` the three checks apply to the wrapped object rather than to the
single-key wrapper. And **the sample is its own compile barrier** — an enum
variant literal has no functional-update form, so every field must be named at
construction, and a new field is a compile error at the sample before it can
become a missing row. That is `I9`'s guarantee, obtained for variants without a
second literal.

**What the arm currently guards.** Exactly one variant key in the closure is not
required today: `DelegationAct::Propose.declare` is `#[serde(default)]`
(`submission.rs:889-891`). Every other key on every struct variant of
`Provenance`, `ReviewDisposition`, `AgentAct`, `Dispose` and `DelegationAct` is a
plain field. So the live hazard is a required variant key declared `Optional` — a
caller told they may omit `DelegationAct::Refuse.reason` — and what the arm keeps
true is that the answer stays *derived* as variants gain fields, rather than
re-established by the next reader from the definitions.

**The samples live in `payload_contract.rs`**, in its test module beside the
`payload_variants!` invocations, not beside each enum. `sec-7` settled that the
invocations do not scatter across the five modules defining the closure's enums;
their samples answer to the same reason, and keeping them together is what keeps
`inquiry.rs`, `traversal.rs` and `mod.rs` out of the touch-set. The struct
fixtures sit beside their types for the opposite reason — they are `I9`'s
precedent, and `Declaration`'s literal is reused by an existing test
(`tests.rs:2908`) rather than copied.

**Why this arm was missed twice.** Pin 3's earlier drafts each corrected the
assertion's *direction* — superset, then equality, then the read-path probe —
while leaving its **domain** unexamined. Two oracles were wrong in a row and the
fixture set was never the thing under inspection. A pin's domain is a claim like
any other in this ladder, and this is the place the design says so.

## 4 — Tokens against serde's renames, and why the coverage is total

Per enum, one sample value per variant. The token is extracted **through the
table's own claim about that variant** — which since `sec-2` is a claim about the
tagging *and* the payload together, not about the type alone:

| tagging | payload | extraction |
|---|---|---|
| `Internal(tag)` | any | the value at `tag` |
| `External` | `Keys` / `Inlines` | the sole object key |
| `External` | `Absent` | **the JSON string itself** |
| `Untagged` | `Shape` | no token; skipped |

Then `{extracted tokens}` == `VARIANTS`.

**The third row is not a refinement, it is a defect this pin had.** Driving
extraction off a per-type `External` claim would look for a sole object key in
`AgentAct::DraftingReady`'s bare string `"drafting-ready"` and find none — the pin
would fail on a correct table, or worse, be written to tolerate the miss and then
be blind to a real one. The per-variant reading is what makes the pin total over a
mixed enum.

Two properties for the price of one assertion, and both matter:

- **`Tagging` acquires its only pin.** Nothing else in the ladder checks that the
  table's tagging claim matches serde's behaviour. Driving the extraction from the
  claim means a wrong claim finds no token and fails here. This is the failure that
  cost this slice's own design run a round trip, so it is the one place a silent
  table defect would be most expensive.
- **The test's own coverage is pinned rather than trusted.** `VARIANTS` is complete
  by the compile-time match, so a forgotten sample value is a set mismatch instead
  of a quiet gap.

`WireFacetValue` is `Untagged` and contributes no tokens; its two shapes are
covered by pin 2's JSON-kind walk. The rendered `bare` label needs no pin of its
own — it is derived from `External` + every payload `Absent` (`sec-2`), so this
pin already covers the facts it is derived from.

## 5 — The injected region: one narrow fidelity test

`sec-3` used to name this the soft spot and concede it closed by test alone. It no
longer does, and the pin shrinks — though not as far as an intermediate draft
claimed. A region nobody supplies cannot be spelled, and a moved *facet
vocabulary* flows through a builder that never copied it: those two are compile
barriers. The **kind** list is not one, because the builder iterates the
hand-maintained `RecordKind::ALL` (`sec-3`). So two things are left for a test,
and both are below.

- **The mapping is faithful.** Per kind: the `SelectedKeys` row's key set equals
  `facet_fields(kind)`'s names in order, and each `Token` vocabulary equals
  the `KNOWN` set behind that field's `FieldShape::Closed`. The comparison is
  against `knowledge`'s tables directly, not against the builder that read them —
  what it pins is that a total, compile-checked mapping was also an order- and
  content-preserving one.
- **The row tokens are the kind tokens — and the oracle is the enum, not
  `ALL`.** `{the selector table's row tokens}` == `{every declared `RecordKind`
  variant's token}`, where the right-hand side comes from an **exhaustive match
  over `RecordKind`** written in the test: `sec-4`'s own instrument, pointed at a
  knowledge-tier enum. An earlier draft compared against `RecordKind::ALL`, which
  is the hand-maintained array the builder already iterates (`knowledge.rs:159-169`),
  so the pin would have agreed with the builder about a kind both had dropped —
  and `sec-3` shows an eighth variant can be added with `ALL` left behind. The
  exhaustive match makes that a build failure in this test instead. It also makes
  `CreateRecord.kind`'s `TokenSource::Extern` and `CreateRecord.facet`'s
  `MapKey::Extern` provably the same kinds rather than two readings of one list.

This is still the ladder's weakest rung — two tests where the rest has compile
errors — but it is now weak about narrow claims rather than about whether the
region is the region, and the second of the two covers a gap in the knowledge
tier that this slice does not otherwise repair.

## 6 — The document

`render_document()` against the committed file read **from disk** via
`CARGO_MANIFEST_DIR`, never through the embed (`artifact.rs:373-375`), plus the
`#[ignore]`d regeneration test the golden's failure message names.

Publication needs no new test:
`assert_unprojected_install_assets_are_published` (`asset_source.rs:132`) already
fails on an `install/` asset that is neither published nor flagged.

## 7 — The three pointer surfaces

- **help** — `render_subcommand_help(&["design", "apply"], …)` contains
  `PAYLOAD_CONTRACT_POINTER`. This is no longer a drift pin: `sec-6` single-sources
  the address through `long_about = format!(…)`, so there is no second spelling to
  diverge, and what remains is the ordinary reachability check that the pointer
  actually renders. The seam already has five `help_snapshot_*` users
  (`main.rs:889-970`) and sixteen `render_subcommand_help` call sites, one of them
  already on the two-element path form (`main.rs:1120`).
- **refusal** — a payload carrying an unknown key on `Declaration` is refused with
  a message containing both serde's own text and the pointer.
- **envelope** — `design show --format prompt` emits the `contract` line, and the
  JSON rendering carries `contract_pointer`.
- **classification** — `cls(&["doctrine", "design", "contract"])` is `None`, i.e.
  `Read`, on `observation_write_class_split`'s shape (`main.rs:338`, `866`).

## 8 — `DEC-228`'s worked example

`DEC-228` first pinned the example by asserting the constant deserialises as a
valid `ApplyRequest`. It cannot, for two independent reasons, and the record was
corrected on both: `DECLARATION_EXAMPLE` carries `"known_revision":<n>`
(`envelope.rs:75-80`) and `<n>` is not a JSON value — the example is a *template*
— and the constant's final `concat!` arm is prose, not JSON.

So the pin parses the **JSON arm alone, after substitution**: `<uid>` → a
well-formed run uid, `<n>` → `0`, `<unique>` → a submission id. The substitution
table is self-pinning: a fourth placeholder introduced into the example makes the
parse fail rather than pass quietly, which is the right direction to fail in.

That needs the JSON arm named as its own const, with
`DECLARATION_EXAMPLE = concat!(JSON_ARM, PROSE)` retained so the existing
compile-time length assertion keeps covering exactly what is rendered. The parsed
request must then carry `traversal.cursor` — the omission that cost fifteen source
reads and the reason `DEC-228` exists.

## 9 — The JSON is a flat table, and every edge lands in it

Appended after the render sample found that the JSON rendering had been specified
as a derived `Serialize` it cannot be (`sec-5`). Two assertions over
`render_json`'s output, both cheap and both structural:

- **Every type appears exactly once.** `types` is keyed by name, so the assertion
  is that the walk emitted no duplicate and did not inline. Without it the failure
  mode is silent and self-similar — a document that looks right and is
  quadratically redundant.
- **Every `Named` edge resolves.** Every type name appearing in a `ty` position is
  a key of `types`, and every key of `types` is reachable from `root`. The second
  half is what catches a type dropped from the walk rather than from the table.

The extern region is exercised in the same pass: `CreateRecord.facet`'s `MapKey`
names `kind` as its selector, and `kind` must be a sibling key of the same type —
a mis-typed selector is otherwise a promise no consumer can act on and no other
pin reads.

## 10 — No repo-private id reaches a shipped surface

Every output this slice ships — `render_prompt`, `render_json` and the published
document — must be free of per-repo sequential ids. In an installed client
`ISS-333` is either absent or that client's own unrelated issue, and a citation
that silently resolves to the wrong record is worse than a dangling one
(`POL-002`).

The detector already exists and is not reimplemented: `cites_a_repo_private_id`
and its fourteen `REPO_PRIVATE_PREFIXES` are `artifact.rs:325-347`'s, written for
the sibling generated asset. The assertion is lifted to cover all three
renderings, and it carries `the_artefact_cites_no_repo_private_id`'s positive
control first (`artifact.rs:360-370`) — fire the detector on a known citation,
*then* assert the real output is clean — so a broken extractor cannot pass
vacuously. `sec-5` fixes the line shape this pin protects; the pin is what stops
the next parenthetical reintroducing one.

## Alignment with the slice's closure intent

| Closure intent | Discharged by |
|---|---|
| The contract is reachable from the binary without reading `src/`, `cursor` among it | pin 7 (three surfaces) + pin 1 (the closure includes `TraversalDeclaration`) + pin 8 |
| A test fails if a payload field is added, removed or renamed without the contract following | pins 1 and 2, over the compile barrier |
| The surface is total over the payload, not over `WRITER_ACTS` | pin 1 across all twelve closure members — eleven by contract, `SubmissionEnvelope` by the root's composition; pin 9 that every one of them reaches the rendering; nothing in the ladder reads `WRITER_ACTS` |
| `facet` is discoverable rather than an open bag | pin 5's two equalities — mapping fidelity over the compile barrier `sec-3` put under it, and the kind set against the enum because `RecordKind::ALL` is not one |
| `ISS-333` option 3 discharged and recorded as such | not a test — a close-time statement, whose shipped half is `sec-2`'s `UnknownKeys::SilentlyDropped` |
| Nothing doctrine-private ships to a client project | pin 10, over `artifact.rs`'s existing detector and its positive control |
| `ISS-355` — a correct submission prints no change row either | not a test and not repaired here; recorded in `sec-2` as the reason the disclosure has to ride the contract rather than the response |

## Not pinned, deliberately

- **serde's behaviour.** The pins compare the contract against what serde does;
  they do not re-test serde.
- **The document's English.** The golden pins bytes to the renderer, which is a
  drift pin, not a review of the prose.
- **The render sample.** `.doctrine/slice/251/render-sample.md` is hand-written
  evidence that predates the generator, not a golden. Pinning the two together
  would pin the generator to a transcription.
- **`ISS-333`'s mechanism.** Untouched by design; the contract discloses it.
- **Whether the contract actually saves anyone reads.** That is `RFC-026`'s
  measurement axis and an observation at close, not an assertion in a test.

<!-- doctrine:section sec-9 -->
# 9. What this design carries forward

The assumptions it stands on, the risks it does not remove, and the questions it
deliberately leaves open. Named here rather than distributed through the prose,
because each is something an implementer or a reviewer has to be able to find.

## Assumptions

**`A1` — no v1 envelope wire change.** Holds, in the precise form `sec-1` and
`sec-6` give it: the contract's *address* rides the turn and its body never does.
The one wire movement is an additive `contract_pointer` field, and the precedent
is in the same struct — `SL-244` added `sections_outstanding_review`
(`envelope.rs:157`), `review_outstanding` (`249`) and `pass_stale` (`339`) at
`TURN_ENVELOPE_VERSION = 1`. If a reviewer reads `A1` as *no new key at all*, the
assumption is contradicted and the pointer has to move into
`DECLARATION_EXAMPLE`'s trailing prose; `sec-6` records why that is worse but not
why it is impossible.

**`A2` — `delegation`'s absence from `WRITER_ACTS` is correct for that table.**
Holds and is now load-bearing in the opposite direction: the contract keys off
`sec-3`'s closure and no pin reads `WRITER_ACTS`, so the two tables can answer
different questions without either being wrong.

**`A3` — retired, and its premise was false.** It read *clap derive takes a
literal — nothing in the crate passes an expression into a `#[command(…)]`
attribute*, and bought the `--help` pointer's extra pin on that basis. One grep
refutes it: `compare.rs:75` passes `clap::ArgGroup::new(…)` into `#[command(group
= …)]`, and clap's derive forwards `key = expr` as a method call throughout. What
actually blocks single-sourcing is narrower and is not an assumption at all — a
`long_about` expression *replaces* a doc comment rather than appending to it, and
`concat!` cannot splice a `const &str`. `sec-6` briefly carried that as a cost
judgement about one line of help on one verb, and **that judgement was the same
error a second time**: it assumed, still without checking, that single-sourcing
required moving the whole long help behind a function. It does not —
`long_about` accepts any expression that is `Into<StyledStr>`, so a `format!`
suffices, and `sec-6` now single-sources the pointer. Nothing above ever depended
on `A3`; what it cost was a ladder rung justified by the wrong reason, and then a
`STD-001` deviation justified by a price that was never charged.

**The class, stated once so it is findable:** an unverified claim about what a
*tool* cannot do, load-bearing on a design decision. It is cheap to check and
expensive to leave — the first instance bought an unnecessary pin, the second
bought an unnecessary standards deviation, and both read as settled reasoning on
the page. Where this design asserts a limit of clap, serde or cargo, it now cites
the source that establishes it.

**`A4` (new) — the closure is acyclic.** Checked in `sec-3`, and
`WireType::Named(&'static TypeContract)` depends on it. A future payload type that
re-enters its own path forces a name-keyed indirection instead.

**Facts the pins depend on**, stated so a failing pin is diagnosable rather than
mysterious: `Sparse::Null` and `Sparse::Omitted` serialise identically while
`skip_serializing_if` drops only `Omitted` (`submission.rs:74-82`, `126`) — pin 3
rests on that asymmetry; and `#[serde(skip)]` keeps `resolved_record` off both
sides of pin 1 without an exception list.

## Risks

**`R1` — a second description of the payload, free to go stale.** Answered by the
ladder at compile time for every rung except the injected region, where two tests
carry it (`sec-8` pin 5). That residue is real and bounded; it is the price
`sec-3` paid for describing a vocabulary the leaf tier may not import.

**`R3` — clippy does not lint tokens a macro body wrote** (`gate.rs:459-462`).
Reduced to near-nothing by `DEC-229`: the generated region is a token array and a
match with empty arms, so there is no code in it that could hold a bug.

**`R4` (new) — the twelve exhaustive literals are one `..` away from useless.**
They are the dominant mechanical cost of the slice and the compile barrier under
pin 1, and an implementer who reaches for `..` in any of them silently deletes that
type's barrier while every test still passes. There is no pin for this — a test
cannot see the difference. It is a review point, and each fixture should carry
`Declaration::fully_populated`'s doc sentence saying why the `..` is forbidden —
now joined by the second obligation `sec-8`'s oracle discipline adds, that no
`Seq` or `Map` key is left empty. That one *is* pinned, and the sentence is there
so an implementer meets it deliberately rather than by accident.

**`R5` (new, and now measured) — the contract can be correct and still too
expensive to read.** No longer an estimate: the full closure was rendered ahead of
implementation and comes to **202 lines / 8 826 bytes**
(`.doctrine/slice/251/render-sample.md` §2, and `sec-5` carries the composition).
That is one fetch against `RFC-026 E8.7`'s measured fifteen source reads, which is
a good trade at session scale, and it is small enough that no bound is worth
designing in. The measurement also settled what the cheapest fifth to cut would
be — the injected region, which is the one part a caller cannot recover any other
way, so there is nothing worth trimming. So the risk is **retired as measured
rather than carried**: if later evidence shows the full render is what stops
agents fetching, the escalation is a filter (`OQ-A`), not a smaller contract.

What the measurement does **not** settle is whether the body could ride the turn.
An earlier draft said it could not, reading 8 826 B as ~8.6× "the turn envelope's
1024-byte cap" — but that constant is `ENVELOPE_DECLARATION_EXAMPLE_BYTES`, the
cap on one worked-example line, not the envelope. The envelope runs
`ENVELOPE_NORMAL_BUDGET_BYTES = 24576` (`render/mod.rs:126-134`), so the body
would fit, and `sec-6`'s address-not-body choice is **preferred rather than
forced**. `sec-5` carries the corrected arithmetic and the grounds the choice
actually rests on. The distinction is load-bearing for `OQ-B`: an escalation
nobody can re-open is not an open question.

## Open questions

**`OQ-A` — per-act filtering.** `doctrine design contract --act declare` is
additive over the same table and is not designed here. The whole-contract render
is the commitment; a filter is the first escalation if `R5` bites.

**`OQ-B` — injecting the body into `resume`.** `DEC-225` weighed and declined it.
It stays the recorded escalation if cold re-entry agents prove to fail before
fetching, and `contract_section`'s suppressible `--known-*` block is the seam.

**`OQ-C` — generalising to other JSON-payload commands.** An explicit Non-Goal.
If the shape proves out here, it is a follow-up slice, not a widening of this one.

## What implementation may still overturn

`DEC-221` is held loosely at the user's direction — the scope says so in terms:
type-design detail the implementing agent may overturn on evidence. `DEC-229` has
already used that latitude once, narrowing the enum half from *generate the type*
to *bar a non-exhaustive match*. Anything further deserves the same treatment: a
record that says what changed and why, not a quiet substitution at the keyboard.

**A reading hazard while that stands.** `DEC-221`'s title still reads as though
enum vocabularies are generated. `DEC-229` made that half-false and `DEC-221`'s
body carries a scope note at its head. Read the note, not the title.

The `Cow` an intermediate draft put on `TypeForm`'s row slices is **gone**, not
held loosely: once `sec-3`'s extern region became a `SelectorTable` owning its own
rows rather than a `TypeContract`, nothing the command tier builds was a member of
the model, and the const table went back to `&'static` throughout. `sec-7` records
the removal. An implementer who reaches for `Cow` again should first check whether
they have re-created the misdescription that made it necessary.

## What this design does not answer

- `ISS-333`'s serde mechanism. The contract discloses the hole
  (`UnknownKeys::SilentlyDropped`); repairing it is a Non-Goal and stays with
  `ISS-333` on its serde axis.
- Per-act narrative prose assets on `DEC-122`'s pattern. Deferred by `DEC-219`,
  not rejected.
- Client-project authored or overridden contracts. A Non-Goal, and `DEC-226`
  seals the published document against it.
- `IMP-390`'s other three candidates, which stay with `IMP-390`.

