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
back in as a `BTreeMap`. The payload contract follows both.

Two tables already describe fragments of this payload and neither is displaced:

- `ApplyRequest::WRITER_ACTS` (`submission.rs:989`) — nine acts, each with the
  predicate detecting its presence. Answers *does this payload write*, for
  `EX-2`. `delegation` is absent and must stay absent; `A2` records why.
- `Declaration::WIRE_KEYS` (`submission.rs:563`) — fifteen declaration keys, each
  with the subject kind that honours it. Answers *is this key inert here*, for
  `ISS-318`.

Neither answers *what may this payload contain*. The new table does, and sits
beside them.

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
    pub(crate) tagging: Tagging,
    pub(crate) unknown_keys: UnknownKeys,
    pub(crate) form: TypeForm,
}

pub(crate) enum TypeForm {
    Struct(Cow<'static, [KeyContract]>),
    Enum(Cow<'static, [VariantContract]>),
}

/// `const fn` constructors, so the table's authoring site never spells `Cow`.
pub(crate) const fn strukt(rows: &'static [KeyContract]) -> TypeForm { … }
pub(crate) const fn enumeration(of: &'static [VariantContract]) -> TypeForm { … }
```

**Why `Cow` and not `&'static`.** The const table is entirely borrowed and would
never need it. `sec-3`'s `Extern` region is the reason: the command tier assembles
that sub-contract from `facet_fields` while the process runs, so its row slice
cannot be `'static`. Every *string* in an injected contract already is — kind
tokens, facet names, the `KNOWN` sets — so the borrow-or-own split lands on these
two slices and nowhere else, and the constructors keep it out of twelve hand-written
table rows. `sec-7` records the alternatives.

`WireType` is the recursion:

```rust
pub(crate) enum WireType {
    Text,
    Integer,
    Boolean,
    Id,                              // DesignId — a run-scoped `inq-N` / `sec-N` / `cp-N`
    Named(&'static TypeContract),    // the closure edge
    Token(&'static [&'static str]),  // a string field with a closed vocabulary
    Seq(&'static WireType),
    Map(&'static WireType),
}
```

`Token` is not a second way to spell an enum. It is what a facet field's
`FieldShape::Closed` actually is on the wire — a plain string drawn from a closed
set, with no Rust type name a caller could ever see. `Named` pointing at an
invented enum would say something false about the payload.

## The variant row, and where a payload actually sits

`TypeForm::Enum` holds these:

```rust
/// One enum variant: its token, and the keys its payload carries — empty for a
/// unit variant.
pub(crate) struct VariantContract {
    pub(crate) token: &'static str,
    pub(crate) payload: &'static [KeyContract],
}
```

**Where those keys sit on the wire is a function of `Tagging`, not a field of its
own.** Under `Internal(tag)` serde inlines them *beside* the tag: a `create`
disposition is `{"form":"create","kind":…,"title":…}`, with `CreateRecord`'s keys
at the same level as `form` rather than nested under it — `Dispose`
(`submission.rs:213`) is internally tagged and its `Create(CreateRecord)` is a
newtype variant. Under `External` the payload nests under the token,
`{"blocking-set-declared":{"blocking":[…]}}`. Under `Bare` there is no payload.

Deriving placement from the tagging rather than storing it is what keeps the two
from disagreeing. The renderer still has to *say* it: a rendering that listed
`Dispose`'s four tokens and stopped would leave a caller believing `create`'s
payload nests, which is precisely the class of error this contract exists to end.

## The three fields that carry the semantics

`DEC-219` decided the contract carries wire shape *plus engine semantics*, and
these are the three places that decision is spent. Each earns its keep against a
failure that actually happened.

**`Tagging`** — the failure that cost a round trip on this slice's own run.

```rust
pub(crate) enum Tagging {
    NotTagged,                    // a plain struct
    Bare,                         // a unit-variant enum: "governance-confirmed"
    Internal(&'static str),       // { "act": "export", … }   — DelegationAct
    External,                     // { "blocking-set-declared": { … } } — AgentAct
    Untagged,                     // discriminated by shape alone — WireFacetValue
}
```

Three sibling act types spell their discriminant three different ways —
`DelegationAct` internally on `tag = "act"` (`submission.rs:877`), `AgentAct`
externally (`attestation.rs:671`), and `CheckpointActDeclaration.act` as a bare
`ActKind`. A contract stating field types alone would have said nothing useful
about any of them.

`Untagged` is the fifth and least guessable: `WireFacetValue` (`submission.rs:254`)
is `#[serde(untagged)]` over `List(Vec<String>)` and `Text(String)`, so a facet
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
    Refused,           // deny_unknown_fields — a misspelt key is a refusal
    SilentlyDropped,   // serde(flatten) forbids deny_unknown_fields — ISS-333
}
```

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
has no members, the enum collapses to a single variant, and the field can be
deleted outright. A contract that had quietly implied uniform
refusal would instead have been wrong for the whole intervening period, and
wrong in the direction that costs a caller a revision to discover.

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
`Declaration.reviewer` — each one hop below a field nobody had walked. That is
the argument for **deriving the closure mechanically at implementation time and
diffing it against this list**, rather than trusting either.

**The closure spans five files**, all inside `design_run`: `submission.rs` (every
wire struct), `attestation.rs` (`Reviewer`, `ActKind`, `ReviewPolicy`,
`ReviewDisposition`, `AgentAct`), `inquiry.rs` (`Provenance`,
`InquiryLifecycle`), `traversal.rs` (`Authority`, `Posture`) and `mod.rs`
(`Stage`). `design_run` is one leaf-tier module (`layering.toml:31`), so none of
those crossings creates a module-graph edge — but an implementer walking the
closure visits all five.

## What bounds it, and why the bound is a fact rather than an opinion

Three exclusions, each by construction.

**`#[serde(skip)]` fields are outside the wire.** `Declaration::resolved_record`
is the live case. It is excluded because serde does not accept it, not because
someone judged it internal — which is the same test `DEC-227` used to reject a
boundary drawn by judgement.

**A type that serialises as a scalar is not a struct row.** `DesignId`
(`ids.rs:132`) carries `#[serde(try_from = "String", into = "String")]`, and
`ReviewRef` (`attestation.rs:475`, reached through `ReviewDisposition::Conducted`)
is a newtype over `String`. Both sit in the *type* closure and neither has wire
keys, so both render as scalars — `Id` and `Text` — not as types with rows. The
test is mechanical again: what serde emits, not what the Rust declaration looks
like.

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
variant per kind, deliberately *not* an untyped bag; the three closed facet
value-enums (`Confidence`, `Basis`, a constraint's `source`) each carry a `KNOWN`
set, and `FieldShape::Closed` takes its tokens from that set rather than a
retyped literal, on `STD-001`.

The correspondence with `sec-2`'s model is close enough to be suspicious, and it
is not a coincidence — both are describing the same wire. `FieldShape::Text` and
`FieldShape::List` are exactly `WireFacetValue`'s two untagged variants, which is
*why* `WireFacetValue` is untagged: a facet value is a string or a list of
strings because a facet field is `Text` or `List`.

## The resolution: inject a contract, not a token list

Three ways to handle the extern region, and the third is the one this design
takes.

1. **Render it as free text.** Honest about the type, useless to the caller, and
   it reintroduces the guess-and-submit loop the slice exists to remove — for
   `kind`, and worse for `facet`.
2. **Duplicate the vocabulary in `design_run`.** Forbidden by `ADR-001`, and it
   would be a second list free to drift from `RecordKind` and `facet_fields` —
   the failure `DEC-221` spent the whole inquiry avoiding.
3. **Have the shell inject a sub-contract.** The leaf declares that a region of
   the closure is externally supplied; the command tier — which may import both
   `design_run` and `knowledge` — builds a `TypeContract` from `RecordKind::ALL`
   and `facet_fields`, and hands it in at render time.

Option 3 is not a new mechanism. It is precisely how `design_run::prompt` already
works: that module names an asset *key* and never reads it, and the shell resolves
the bytes and hands back a `BTreeMap` for `contract_block` to place
(`prompt.rs:185`). The payload renderer takes the same split, with a richer
payload.

Concretely, `WireType` gains one variant:

```rust
/// A region of the closure owned by a tier this one cannot import.
/// The table names the source; the renderer receives the contract.
Extern(&'static str),            // e.g. "knowledge::RecordKind"
```

and the render entry point takes `&BTreeMap<&'static str, TypeContract>`
alongside the table. Mapping `FacetFieldRow` into `KeyContract` is mechanical:
`Text` and `List` become `WireType`s directly, and `Closed(tokens)` becomes an
enum `TypeContract` whose variants are those tokens.

What a caller then gets, for a payload key that today is an undocumented open
map: the seven kinds, and per kind the facet names it owns, in template order,
with closed vocabularies enumerated where they exist.

## One thing the contract must say carefully

`facet_fields` is enforced **post hoc, not at the write seam**. An unrecognised
facet key is accepted by `create` and surfaces later as a `doctor` inert-facet-key
finding (`doctor_checks.rs:161-175`). So the contract describes *what a kind
owns*, not *what will be refused* — and it must say so, or it makes a promise
about refusal that the write path does not keep.

That distinction is exactly what `sec-2`'s `UnknownKeys` field exists to carry,
one level up. The two are the same disclosure discipline applied at two depths.

## The cost of this choice, stated

`Extern` is a hole in the single-source guarantee: nothing at compile time forces
the shell to supply a contract for every `Extern` the table names, nor forces the
supplied contract to match `facet_fields`.

Both close by test rather than by type, and `sec-8` commits to the pair —
set-equality between the `Extern` sources the table names and the sources the
shell resolves, and equality between each injected sub-contract and the
`facet_fields`/`RecordKind::ALL` it claims to mirror. This is weaker than the
compile-error pins `DEC-221` buys elsewhere, and the weakness is confined to the
injected region. Recording it here rather than in a footnote, because a reader
implementing `sec-5` will otherwise assume the generator is total on its own.

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
exhaustive match over the real type:

```rust
payload_variants! {
    Dispose {
        Create      = "create",
        Adopt       = "adopt",
        Unresolved  = "unresolved",
        NonDurable  = "non-durable",
    }
}
```

expanding to a `VARIANTS: &[&str]` used by the contract table, plus:

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

What it does *not* pin is that the token strings equal serde's actual renames. A
`#[serde(rename_all = "kebab-case")]` type could have its variant renamed while
the contract's literal stays put. That is a per-variant round-trip test —
serialise a value of each variant, assert the token matches — and `sec-8` carries
it. The exhaustiveness barrier guarantees the test has a case for every variant,
which is what makes a round-trip test sufficient here and insufficient on its own
elsewhere.

## The struct pin: `I9`, extended down the closure

Each of the twelve struct types in `sec-3`'s closure gets `I9`'s treatment: an
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

Two regions, both named rather than discovered later.

**The injected sub-contract.** `sec-3`'s `Extern` region is supplied by the
command tier at render time, so no compile-time pin in `design_run` can see it.
It closes by test: set-equality between the `Extern` sources the table names and
the sources the shell resolves, and equality between each injected sub-contract
and the `facet_fields` / `RecordKind::ALL` it mirrors.

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
| Token ↔ serde rename | per-variant round trip | test |
| Type and presence correctness | round trip per key | test |
| Injected `Extern` region | set-equality against `facet_fields` | test |
| Published document | golden against disk-source | test |
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
(`artifact.rs:26`), and `design_run::prompt` already names asset *keys* it never
reads while the shell resolves bytes and hands back a `BTreeMap` for
`contract_block` to place (`prompt.rs:13-16, 185`). The payload contract takes the
same split, with `sec-3`'s injected sub-contracts riding the `bodies` slot.

```rust
// design_run::payload_contract — leaf, pure.
pub(crate) const PAYLOAD: TypeContract;

pub(crate) fn render_json(extern_contracts: &ExternContracts) -> String;
pub(crate) fn render_prompt(extern_contracts: &ExternContracts) -> Vec<String>;
pub(crate) fn render_document(extern_contracts: &ExternContracts) -> String;

pub(crate) type ExternContracts = BTreeMap<&'static str, TypeContract>;
```

Three entry points rather than one with a format parameter, because the three
outputs have genuinely different shapes — a JSON document, a line sequence, and a
Markdown page — and a single function returning one of three unrelated things
would be a `match` pretending to be an abstraction. They share the walk over
`PAYLOAD`, not the emission.

## `--format json`: bespoke, on the turn envelope's precedent

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

The JSON is the `sec-2` types serialised, which makes the contract and its
rendering the same artefact viewed twice rather than two artefacts to reconcile.

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
payload ApplyRequest  unknown-keys: silently-dropped   (ISS-333 — a misspelt top-level key is discarded)
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

A variant's payload is rendered where it actually lands, which the tagging
already determines (`sec-2`): inlined beside the tag under `Internal`, nested
under the token under `External`, absent under `Bare`. A token list alone would
be the contract making this slice's own mistake.

Field order is the commitment, as it is for `contract_line`. The parenthetical on
`sparse` is the one place the rendering adds words the table does not carry, and
it is a fixed string per `Presence` variant rather than per key — so it cannot
drift row by row.

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

1. resolving `ExternContracts` from `RecordKind::ALL` and `facet_fields`
   (`sec-3`), which requires importing both `design_run` and `knowledge` and is
   therefore only possible here;
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

The pointer rides the `Apply` variant's doc comment (`commands/design.rs:124-125`).
Clap's derive splits a doc comment at the first paragraph — line one becomes
`about`, the whole becomes `long_about` — so a second line costs nothing in
`doctrine design --help`'s table and appears in `doctrine design apply --help`,
which is where a caller reading about `--input` is standing.

**This is the one place the address cannot be single-sourced, and the answer is a
test.** Every verb in `commands/design.rs` takes its help text from a doc comment,
and nothing in the crate passes an expression into a `#[command(…)]` attribute —
79 of them, and every one is `subcommand` (46), `flatten` (30), or a single
literal `name`/`about`, `group`, `alias`, `visible_alias`. So the address exists
as a literal here and as a const at the other two points.

`STD-001`'s force does not evaporate because a macro cannot take an expression; it
relocates to a pin, the same move `sec-4` makes for the injected `Extern` region.
`render_subcommand_help(&["design", "apply"], …)` must contain
`PAYLOAD_CONTRACT_POINTER` — the seam already exists and already has four
`help_snapshot_*` users (`main.rs:889-930`). `sec-8` carries it.

Builder-side injection would single-source it properly. `DEC-224` already declined
to leave derive for the *enumerating* form, and a pointer is a far weaker reason
to leave it.

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
| `src/design_run/payload_contract.rs` | **new** — the model types (`sec-2`), `payload_variants!` and its per-enum invocations (`sec-4`), the `PAYLOAD` table, the three renderers (`sec-5`), `PAYLOAD_CONTRACT_POINTER` and `PAYLOAD_CONTRACT_PATH` | ~500 table, ~250 renderer |
| `src/design_run/mod.rs` | one `pub(crate) mod payload_contract;` | 1 |
| `src/design_run/render/envelope.rs` | `contract_pointer` field, its projection, one pushed line (`sec-6`) | ~10 |
| `src/commands/design.rs` | `Contract` variant, `ContractArgs`, `ContractFormat`, `run_contract`, `extern_contracts()`, the `Apply` doc-comment pointer, the `map_err` at 1504 | ~70 |
| `src/commands/guard.rs` | `Contract(_) => Read` | 1 |
| `install/design-payload-contract.md` | **new** — the generated document | rendered |
| `publication/manifest.toml` | one `[[entry]]` | 7 |
| `src/design_run/tests.rs` + in-module tests | the pin ladder (`sec-8`) | ~400 |
| `src/main.rs` tests | the help-pointer pin and the guard classification row (`sec-8`) | ~15 |

**`submission.rs` and `attestation.rs` are read, not written.** Every wire type
in `sec-3`'s closure keeps its current definition; the contract describes them
from outside. This is what makes the behaviour-preservation gate on shared
machinery vacuous here rather than argued — there is no behaviour to preserve
because no behaviour is touched. The only additions near those types are the
exhaustive `I9`-style literals, which live in tests.

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

Out-degree stays inside `design_run`: `submission` and `attestation` for the
`payload_variants!` match arms, and nothing else. `layering.toml:31` keeps
`design_run = "leaf"  # out=0` unchanged, because an intra-module `use` creates no
crate-level edge (`ADR-001`).

## The model change `sec-3`'s injection forces

`sec-3` has the command tier build a `TypeContract` at render time from
`RecordKind::ALL` and `facet_fields`. That contract's row slices are assembled
while the process runs, so they cannot be `&'static [KeyContract]` as first
drafted. Two positions change, and `sec-2` carries the amended block:

```rust
pub(crate) enum TypeForm {
    Struct(Cow<'static, [KeyContract]>),
    Enum(Cow<'static, [VariantContract]>),
}

/// `const fn` constructors, so the const table's authoring site never spells `Cow`.
pub(crate) const fn strukt(rows: &'static [KeyContract]) -> TypeForm { … }
pub(crate) const fn enumeration(variants: &'static [VariantContract]) -> TypeForm { … }
```

Every *string* in an injected contract is already `&'static` — `RecordKind`'s
tokens, `FacetFieldRow::name`, and the `KNOWN` sets behind `FieldShape::Closed` —
so `Cow` is needed at the two slice positions and nowhere else. Nothing else in
the model moves, and `WireType::Named(&'static TypeContract)` still works for the
const table's closure edges: an injected contract is only ever the *target* of an
`Extern` lookup, never the target of a `Named`, so it needs no `&'static` address
of its own.

One `WireType` variant is added with it:

```rust
Token(&'static [&'static str]),   // a string field with a closed vocabulary
```

A facet field's `FieldShape::Closed(tokens)` is precisely that — a plain string on
the wire whose admissible values are a closed set with no Rust type name visible
to a caller. Modelling it as `Named` pointing at an invented enum would state
something false about the wire.

**Two alternatives, both rejected.** `Box::leak` in the command tier keeps the
model `&'static`-uniform and spends one leaked allocation plus every future
reviewer's attention on it. Const-fn assembly of the injected rows is fully static
and leak-free, but the const-fn array gymnastics cost more to write and read than
`Cow` plus two constructors. `Cow` is the ordinary answer to *same type, sometimes
borrowed, sometimes owned*, and the constructors keep it out of the table.

## What the command tier assembles

```rust
/// The only place `design_run` and `knowledge` are both in scope (ADR-001).
fn extern_contracts() -> ExternContracts;
```

One entry, keyed `"knowledge::RecordKind"`: an enum-form `TypeContract` with one
`VariantContract` per member of `RecordKind::ALL` (`knowledge.rs:161`), each
carrying that kind's `facet_fields` rows in template order, mapped
`Text → WireType::Text`, `List → Seq(Text)`, `Closed(tokens) → Token(tokens)`.
Around 25 lines, no branching beyond that three-arm map.

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
  three bind here: leaf purity (`sec-2`, `sec-5`), one generator for three
  renderings plus a tested pin where a single source is inexpressible (`sec-6`),
  and nothing host-project-specific in the shipped document.

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

## Three things need no test

The compile barriers from `sec-4` and `sec-6`, restated as the evidence they are:
`payload_variants!`'s dead exhaustive match over each enum; the exhaustive no-`..`
literals behind the key-set pin; and the `Contract` arms in `commands/design.rs`'s
dispatch and `guard.rs:430`, both non-exhaustive-match errors until written. None
is a test, and each fails before one can run.

## The oracle discipline

Every pin below takes its oracle from **the types themselves** — a serialised
value, or a table the compiler proved complete. Never a second hand-written list:
that is tier 1, which the research excluded because it is blind to exactly the
failure it claims to prevent.

Where an assertion is a set-equality that would pass vacuously if its extraction
broke, it carries a positive control first, on `the_artefact_cites_no_repo_private_id`'s
pattern (`artifact.rs:360-370`) — assert the detector fires on a known-bad input,
then assert the real input is clean.

## 1 — Key sets: twelve types, one assertion body

```rust
fn assert_keys_described<T: Serialize>(value: &T, contract: &TypeContract);
```

One body, twelve call sites. Each type gains a `#[cfg(test)] fn
fully_populated()` beside its definition, on `Declaration::fully_populated`'s
precedent (`submission.rs:653`) — an exhaustive literal with no `..`, so a new
field is a compile error at the fixture before it can be a missing contract row.

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

Catches the commonest table defect — a row whose `ty` was copied from the row
above it. It does **not** separate `Text` from `Id` or `Token`, which are all
strings on the wire: `Id` is pinned by the fixture's own `DesignId::parse`, and
`Token` by pin 4.

## 3 — Presence: the all-null fixture

`Sparse` serialises `Omitted` and `Null` identically — both `serialize_none`
(`submission.rs:74-82`) — but `skip_serializing_if = "Sparse::is_omitted"` drops
only `Omitted`, while `Option` fields drop on `is_none`. So a fixture with every
`Sparse` field set to `Sparse::Null` and every `Option` field `Some` emits JSON
`null` for **exactly** the sparse keys.

- `{keys serialising to null}` == `{keys declared `Sparse`}`
- `{keys present in a minimal value}` ⊇ `{keys declared `Required`}`

Two fixtures, not twelve: only `Declaration` (`submission.rs:126-133`) and
`TraversalDeclaration` (`731-736`) carry `Sparse<T>`.

## 4 — Tokens against serde's renames, and why the coverage is total

Per enum, one sample value per variant. The token is extracted **through the
table's own `Tagging` claim**: `Bare` reads the JSON string, `External` the sole
object key, `Internal(tag)` the value at `tag`, `Untagged` contributes no token
and is skipped.

Then `{extracted tokens}` == `VARIANTS`.

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
covered by pin 2's JSON-kind walk.

## 5 — The injected region: two set-equalities

`sec-3` named this the soft spot and `sec-4` confirmed it closes by test alone.

- **Nobody is missing.** `{Extern sources named in PAYLOAD}` == `{keys
  extern_contracts() returns}`. Catches a table naming a region no one supplies,
  and a supplier for a region no one names.
- **The mapping is faithful.** Per kind: the injected variant's facet key set
  equals `facet_fields(kind)`'s names in order, and each `Token` vocabulary equals
  the `KNOWN` set behind that field's `FieldShape::Closed`. The comparison is
  against `knowledge`'s tables directly, not against the builder that read them —
  what it pins is that the mapping dropped, reordered or retyped nothing.

This is the weakest rung — a test where the rest of the ladder has a compile
error — and it is confined to the injected region, as `sec-3` records.

## 6 — The document

`render_document()` against the committed file read **from disk** via
`CARGO_MANIFEST_DIR`, never through the embed (`artifact.rs:373-375`), plus the
`#[ignore]`d regeneration test the golden's failure message names.

Publication needs no new test:
`assert_unprojected_install_assets_are_published` (`asset_source.rs:132`) already
fails on an `install/` asset that is neither published nor flagged.

## 7 — The three pointer surfaces

- **help** — `render_subcommand_help(&["design", "apply"], …)` contains
  `PAYLOAD_CONTRACT_POINTER`. This is the pin `sec-6` owes for the one address
  that cannot be single-sourced; the seam already has four `help_snapshot_*` users
  (`main.rs:888-930`).
- **refusal** — a payload carrying an unknown key on `Declaration` is refused with
  a message containing both serde's own text and the pointer.
- **envelope** — `design show --format prompt` emits the `contract` line, and the
  JSON rendering carries `contract_pointer`.
- **classification** — `cls(&["doctrine", "design", "contract"])` is `None`, i.e.
  `Read`, on `observation_write_class_split`'s shape (`main.rs:338`, `866`).

## 8 — `DEC-228`'s worked example, and a defect in the pin as recorded

`DEC-228` pins the example by asserting it deserialises as a valid `ApplyRequest`.
**As written it cannot**: `DECLARATION_EXAMPLE` carries `"known_revision":<n>`
(`envelope.rs:75-80`), and `<n>` is not a JSON value. The example is a *template*,
and a template does not parse.

The pin therefore substitutes before parsing — `<uid>` → a well-formed run uid,
`<n>` → `0`, `<unique>` → a submission id — and then asserts the result
deserialises. The substitution table is itself self-pinning: a fourth placeholder
introduced into the example makes the parse fail rather than pass quietly, which
is the right direction to fail in.

Two further assertions on the same value, both from `DEC-228`'s purpose: the
parsed request carries a `traversal.cursor` — the omission that cost fifteen
source reads — and the JSON half is separable from the trailing parenthetical,
which means splitting today's single `concat!` into a JSON const and a prose
const so the test parses the first alone.

## Alignment with the slice's closure intent

| Closure intent | Discharged by |
|---|---|
| The contract is reachable from the binary without reading `src/`, `cursor` among it | pin 7 (three surfaces) + pin 1 (the closure includes `TraversalDeclaration`) + pin 8 |
| A test fails if a payload field is added, removed or renamed without the contract following | pins 1 and 2, over the compile barrier |
| The surface is total over the payload, not over `WRITER_ACTS` | pin 1 across all twelve types; nothing in the ladder reads `WRITER_ACTS` |
| `ISS-333` option 3 discharged and recorded as such | not a test — a close-time statement, whose shipped half is `sec-2`'s `UnknownKeys::SilentlyDropped` |

## Not pinned, deliberately

- **serde's behaviour.** The pins compare the contract against what serde does;
  they do not re-test serde.
- **The document's English.** The golden pins bytes to the renderer, which is a
  drift pin, not a review of the prose.
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
is in the same struct — `SL-244` added `outstanding` and `pass_stale` at
`TURN_ENVELOPE_VERSION = 1`. If a reviewer reads `A1` as *no new key at all*, the
assumption is contradicted and the pointer has to move into
`DECLARATION_EXAMPLE`'s trailing prose; `sec-6` records why that is worse but not
why it is impossible.

**`A2` — `delegation`'s absence from `WRITER_ACTS` is correct for that table.**
Holds and is now load-bearing in the opposite direction: the contract keys off
`sec-3`'s closure and no pin reads `WRITER_ACTS`, so the two tables can answer
different questions without either being wrong.

**`A3` (new) — clap derive takes a literal.** Nothing in the crate passes an
expression into a `#[command(…)]` attribute, so the `--help` pointer is a literal
pinned by a test (`sec-6`, `sec-8` pin 7). If an implementer establishes that the
builder attribute accepts the const, the literal and its pin collapse into one
source, which is strictly better. Nothing above depends on which way this goes.

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
`Declaration::fully_populated`'s doc sentence saying why the `..` is forbidden.

**`R5` (new) — the contract can be correct and still too expensive to read.**
Thirteen struct types, roughly sixty keys and a dozen enums render as perhaps a
hundred lines in `--format prompt`. That is one fetch against `RFC-026 E8.7`'s
measured fifteen source reads, so it is a good trade at session scale — but it is
not free, and nothing in this design bounds it. If measurement shows the full
render is what stops agents fetching, the escalation is a filter, not a smaller
contract.

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

The `Cow` change in `sec-2` is a representation call rather than a decision: if an
implementer finds a const-fn assembly of the injected rows that stays `&'static`
without const-fn gymnastics, taking it changes nothing above `sec-7`.

## What this design does not answer

- `ISS-333`'s serde mechanism. The contract discloses the hole
  (`UnknownKeys::SilentlyDropped`); repairing it is a Non-Goal and stays with
  `ISS-333` on its serde axis.
- Per-act narrative prose assets on `DEC-122`'s pattern. Deferred by `DEC-219`,
  not rejected.
- Client-project authored or overridden contracts. A Non-Goal, and `DEC-226`
  seals the published document against it.
- `IMP-390`'s other three candidates, which stay with `IMP-390`.

