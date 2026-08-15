# SL-251 — worked rendering of the payload contract

Evidence beside the design, not part of it. This is not a design section, carries
no attestation, and locks nothing. It exists because `sec-5` fixes a line *shape*
and `sec-2` fixes a *model*, and neither had ever been run over the real closure.

**Method.** The closure below was derived by walking `ApplyRequest`'s serde
surface in source — `src/design_run/{submission,attestation,inquiry,traversal,
mod,ids}.rs` and `src/knowledge.rs` — and then diffed against `sec-3`'s stated
list. `sec-3` was **not** transcribed. `sec-3` itself commits to this derivation
happening at implementation time ("the argument for deriving the closure
mechanically at implementation time and diffing it against this list"); this is
that commitment paid early, against source rev `ef47e756b`.

**What it found.** The closure matches `sec-3` exactly — twelve structs, fourteen
enums, the same members. The disagreements are all one tier down, in the *model*
and the *renderings*: ten findings, raised on the run as `fnd-10`..`fnd-19`. Four
of them are shapes the model as written cannot express at all.

---

## 1. The derived closure

### Struct types — twelve

Reached from `ApplyRequest` by serde-visible fields only. `deny` = carries
`#[serde(deny_unknown_fields)]`.

| # | type | file:line | wire keys | unknown keys |
|---|---|---|---|---|
| 1 | `ApplyRequest` | `submission.rs:923` | 13 | silently-dropped (`flatten`, `ISS-333`) |
| 2 | `SubmissionEnvelope` | `submission.rs:687` | 3 (flattened into 1) | silently-dropped |
| 3 | `AdoptAuthored` | `submission.rs:696` | 2 | silently-dropped |
| 4 | `TraversalDeclaration` | `submission.rs:729` | 4 | silently-dropped |
| 5 | `StageDeclaration` | `submission.rs:709` | 2 | silently-dropped |
| 6 | `AcceptanceDeclaration` | `submission.rs:315` | 2 | silently-dropped |
| 7 | `Declaration` | `submission.rs:124` | 15 | **refused** (`:123`) |
| 8 | `CreateRecord` | `submission.rs:271` | 6 | silently-dropped |
| 9 | `DischargeDeclaration` | `submission.rs:344` | 3 | silently-dropped |
| 10 | `ReviewPolicyDeclaration` | `submission.rs:806` | 2 | silently-dropped |
| 11 | `CheckpointActDeclaration` | `submission.rs:825` | 3 | **refused** (`:824`) |
| 12 | `AgentActDeclaration` | `submission.rs:855` | 3 | **refused** (`:854`) |

Three refuse, nine discard. `sec-2`'s count confirmed.

`Declaration`'s fifteen keys match `Declaration::WIRE_KEYS: [WireKey; 15]`
(`submission.rs:563`) exactly — `resolved_record` is `#[serde(skip)]` and outside
both. `ApplyRequest`'s thirteen are three flattened plus ten act fields against
`WRITER_ACTS`'s nine rows; `delegation` is the tenth. Both confirmed.

### Enum types — fourteen

| # | enum | file:line | tagging | variants |
|---|---|---|---|---|
| 1 | `Dispose` | `submission.rs:215` | internal `"form"` | 4 |
| 2 | `WireFacetValue` | `submission.rs:254` | **untagged** | 2 |
| 3 | `DischargeClaim` | `submission.rs:335` | bare | 2 |
| 4 | `DelegationAct` | `submission.rs:877` | internal `"act"` | 4 |
| 5 | `AgentAct` | `attestation.rs:673` | **external** | 2 (one bare, one nesting) |
| 6 | `ActKind` | `attestation.rs:45` | bare | 8 |
| 7 | `ReviewPolicy` | `attestation.rs:188` | bare | 4 |
| 8 | `ReviewDisposition` | `attestation.rs:571` | **external** | 2 |
| 9 | `Reviewer` | `attestation.rs:25` | bare | 2 |
| 10 | `Stage` | `mod.rs:126` | bare | 5 |
| 11 | `Posture` | `traversal.rs:38` | bare | 2 |
| 12 | `Authority` | `traversal.rs:26` | bare | 3 |
| 13 | `Provenance` | `inquiry.rs:25` | **internal `"provenance"`** | 4 (two with payload) |
| 14 | `InquiryLifecycle` | `inquiry.rs:69` | bare | 4 |

Membership matches `sec-3`. Two facts `sec-3` does not carry:
`ReviewDisposition` is a **second** externally-tagged enum (`sec-2` names only
`AgentAct`), and `Provenance` is a **second** internally-tagged one (`sec-2`
names only `DelegationAct`) whose tag key is `"provenance"` and whose
`ImportedProse` variant carries four keys.

### Scalars — three, not two

`sec-3` names `DesignId` (`ids.rs:132`, `try_from = "String"`) and `ReviewRef`
(`attestation.rs:475`, newtype over `String`). There is a third:
**`Fingerprint`** (`ids.rs:213`, newtype over `String`), reached through
`Provenance::ImportedProse.fingerprint`. It serialises as a bare string on the
same test — what serde emits, not what the declaration looks like — so it renders
as `text` and is not a struct row. → `fnd-15`.

### Files crossed — six, not five

`submission.rs`, `attestation.rs`, `inquiry.rs`, `traversal.rs`, `mod.rs` — and
**`ids.rs`**, which holds `DesignId` and `Fingerprint`. `sec-3` cites `ids.rs:132`
two paragraphs above the sentence that says five, and the sentence's own purpose
is navigational ("an implementer walking the closure visits all five"). → `fnd-15`.

### Acyclicity — confirmed

Deepest chain, six types:
`ApplyRequest → delegation → DelegationAct::Propose → declare → Declaration →
dispose → Dispose::Create → CreateRecord → facet → WireFacetValue`.
`DelegationAct::Propose.declare: Vec<Declaration>` (`submission.rs:890-891`) is
the one re-entering edge, and `Declaration` reaches no delegation act. No type
appears twice on any path. `&'static TypeContract` holds.

---

## 2. `--format prompt` over the whole closure

Following `sec-5`'s `contract_block` line shape. Three places where the shape had
no form for what the closure contains are marked `‹fnd-N›` and carried below.

```text
payload ApplyRequest  unknown-keys: silently-dropped   (ISS-333 — a misspelt top-level key is discarded)
  run_uid            text                      required
  known_revision     integer                   required
  submission_id      text                      required
  adopt_authored     AdoptAuthored             optional
  traversal          TraversalDeclaration      optional
  stage              StageDeclaration          optional
  acceptance         AcceptanceDeclaration     optional
  declare            [Declaration]             optional
  delegation         DelegationAct             optional
  discharge          DischargeDeclaration      optional
  review_policy      ReviewPolicyDeclaration   optional
  checkpoint_act     CheckpointActDeclaration  optional
  agent_declaration  AgentActDeclaration       optional

type AdoptAuthored  unknown-keys: silently-dropped
  fingerprint        text                      required
  sections           {id: text}                optional            ‹fnd-13›

type TraversalDeclaration  unknown-keys: silently-dropped
  pin                id                        sparse   (omit persists · null clears)
  cursor             id                        sparse   (omit persists · null clears)
  posture            Posture                   optional
  authority          Authority                 optional

type StageDeclaration  unknown-keys: silently-dropped
  to                 Stage                     required
  reason             text                      optional

type AcceptanceDeclaration  unknown-keys: silently-dropped
  basis              text                      required
  turn               text                      optional

type Declaration  unknown-keys: refused
  subject            id                        required
  question           text                      sparse   (omit persists · null clears)
  needs              [id]                      sparse   (omit persists · null clears)
  parent             id                        sparse   (omit persists · null clears)
  provenance         Provenance                optional
  lifecycle          InquiryLifecycle          optional
  body               text                      optional
  attests            id                        optional
  reviewer           Reviewer                  optional
  concerns           id                        optional
  summary            text                      optional
  blocking           boolean                   optional
  resolution         text                      optional
  disposes           id                        optional
  dispose            Dispose                   optional

type CreateRecord  unknown-keys: silently-dropped
  kind               knowledge::RecordKind     required            ‹extern — §4›
  title              text                      required
  slug               text                      optional
  body               text                      optional
  facet              {facet-name: WireFacetValue}  optional        ‹extern — §4, fnd-13›
  acceptance         AcceptanceDeclaration     optional

type DischargeDeclaration  unknown-keys: silently-dropped
  step               text                      required
  outcome            DischargeClaim            required
  reason             text                      optional

type ReviewPolicyDeclaration  unknown-keys: silently-dropped
  policy             ReviewPolicy              required
  acceptance         AcceptanceDeclaration     required

type CheckpointActDeclaration  unknown-keys: refused
  act                ActKind                   required
  acceptance         AcceptanceDeclaration     required
  disposition        ReviewDisposition         optional

type AgentActDeclaration  unknown-keys: refused
  act                AgentAct                  required
  basis              text                      required
  turn               text                      optional

enum Dispose  tagging: internal("form")   variant keys sit BESIDE "form"
  create           → CreateRecord's keys, inlined                  ‹fnd-14›
  adopt              record       text      required
  unresolved         note         text      required
  non-durable        note         text      required

enum WireFacetValue  tagging: untagged   discriminated by JSON shape alone
  ‹no token›         [text]                                        ‹fnd-11›
  ‹no token›         text                                          ‹fnd-11›

enum DischargeClaim  tagging: bare
  attested
  skipped

enum DelegationAct  tagging: internal("act")   variant keys sit BESIDE "act"
  export             id           id        required               ‹fnd-17›
                     obligation   id        required
  propose            id           id        required
                     by           text      required
                     summary      text      required
                     declare      [Declaration]  optional
  accept             id           id        required
  refuse             id           id        required
                     reason       text      required

enum AgentAct  tagging: external   payload nests UNDER the token
  blocking-set-declared    { blocking   [id]   required }
  drafting-ready           — a BARE STRING, not an object          ‹fnd-12›

enum ActKind  tagging: bare
  governance-confirmed
  graph-reviewed
  blocking-set-declared
  sufficiency-accepted
  drafting-ready
  section-reviewed
  review-disposed
  design-accepted

enum ReviewPolicy  tagging: bare
  human-only
  adversarial-only
  human-then-adversarial
  adversarial-then-human

enum ReviewDisposition  tagging: external   payload nests UNDER the token
  conducted          { review     text      required }
  waived             { reason     text      required }

enum Reviewer  tagging: bare
  human
  adversarial

enum Stage  tagging: bare
  exploring
  inquiring
  drafting
  reviewing
  locked

enum Posture  tagging: bare
  breadth
  depth

enum Authority  tagging: bare
  agent-proposed
  user-pinned
  user-locked

enum Provenance  tagging: internal("provenance")   variant keys sit BESIDE "provenance"
  user-directed
  agent-proposed
  shaping-question   record       text      required
  imported-prose     section      id        required               ‹fnd-17›
                     line         integer   required
                     label        text      required
                     fingerprint  text      required

enum InquiryLifecycle  tagging: bare
  open
  resolved
  deferred
  pruned

extern knowledge::RecordKind — supplied by the command tier (sec-3)
  CreateRecord.kind is one of these seven tokens. CreateRecord.facet's keys are
  the facet fields of THAT kind — a dependency on the sibling key, see fnd-13.
  Facet keys are enforced POST HOC by `doctrine doctor`, not refused at the
  write seam (sec-3, doctor_checks.rs:161).

  assumption         claim             text
                     confidence        one of: low | medium | high
                     basis             one of: observation | prior-art | design-inference
                                               | external-source | operator-judgement
                     validation_plan   text
                     validated_by      text
                     validated_on      text
                     invalidated_by    text
                     invalidated_on    text
  decision           context           text
                     choice            text
                     alternatives      [text]
                     rationale         text
                     consequences      [text]
                     decided_by        text
                     decided_on        text
  question           question          text
                     why_matters       text
                     answer            text
                     answered_by       text
                     answered_on       text
  constraint         statement         text
                     source            one of: canon | adr | external | technical
                                               | legal | compatibility | operator
                     applies_to        [text]
                     waiver_reason     text
                     waived_by         text
                     waived_on         text
  evidence           datum             text
                     provenance        one of: inspection | experiment | reproduction
                                               | citation
                     confidence        one of: low | medium | high
  hypothesis         proposition       text
                     predicts          text
  concept            ‹no facet fields — a concept's content is its prose›
```

### R5 — the size evidence

| measure | value |
|---|---|
| rendered lines | **202** |
| non-blank lines | 176 |
| bytes | 8 826 (≈8.6 KB) |
| widest line | 104 cols (the `ISS-333` header) |

This is the only real evidence `R5` has. Reading it: the render is **~8.6× the
1024-byte cap the turn envelope already runs** (`A1`), which confirms `A1` from
the other direction — it cannot ride the envelope even if `DEC-064` were
resolved, and `sec-6`'s decision to push an *address* rather than the contract is
forced rather than merely preferred.

Against the alternative it replaces: `RFC-026` `E8.7` measured **15 payload-shape
source reads** on one run. Fifteen reads of `submission.rs` (1067 lines) plus its
five neighbours is the cost these 202 lines remove, and the removal is per-agent
per-run rather than once.

Composition, for anyone tempted to trim: structs 66 lines (33%, root block
included), enums 70 (35%), the extern region 40 (20%), blanks 26 (13%). The
extern region is the cheapest fifth to cut and the one a caller cannot recover
any other way — it is the part that is *undocumented today*, `sec-3`'s open-map
guess. Cutting the bare enums instead (`ActKind`, `Stage`, `ReviewPolicy`,
`Reviewer`, `Posture`, `Authority`, `InquiryLifecycle`, `DischargeClaim` = 38
lines) would save 19% and reintroduce exactly the vocabulary guessing the verb
exists to end.

Verdict: **no trimming warranted.** 202 lines is within two screens and an order
of magnitude under any context budget that matters. The 104-column header is the
one presentational thing worth revisiting — it exceeds the 100-col width the rest
of the corpus holds to.

---

## 3. `--format json`

`sec-5` says: *"The JSON is the `sec-2` types serialised, which makes the contract
and its rendering the same artefact viewed twice rather than two artefacts to
reconcile."*

**That does not hold, and the sample is how it shows.** `WireType::Named` holds
`&'static TypeContract` — a *reference to the whole type*, not a name. A derived
`Serialize` walking it inlines the target recursively at every edge:

```json
{ "key": "declare",
  "ty": { "Named": { "name": "Declaration", "tagging": "NotTagged",
                     "unknown_keys": "Refused",
                     "form": { "Struct": [ … all fifteen keys …
                        { "key": "dispose", "ty": { "Named": {
                            "name": "Dispose", "form": { "Enum": [
                              { "token": "create", "payload": [ … all six
                                 CreateRecord keys, including "acceptance",
                                 which inlines AcceptanceDeclaration again … ]}
```

Counting inlinings over the real closure: `Declaration` twice (root `declare`,
and `DelegationAct::Propose.declare`), `CreateRecord` twice under those,
`AcceptanceDeclaration` **five** times (root, `ReviewPolicyDeclaration`,
`CheckpointActDeclaration`, and once under each `CreateRecord`). A consumer
cannot tell the two `Declaration` inlinings are one type, which is the single
thing a machine-readable contract exists to tell it. → `fnd-10`.

The JSON has to be a **flat, name-keyed type table** with `Named` rendered as a
name — a bespoke walk, not `serde_json::to_string(&PAYLOAD)`. That is a different
claim from `sec-5`'s and needs saying. It costs nothing: the walk already exists
for `--format prompt`.

The corrected shape, on `TurnEnvelope`'s `schema`/`version` precedent
(`render/envelope.rs:501-502`), which `sec-5` gets right:

```json
{
  "schema": "design-payload-contract",
  "version": 1,
  "root": "ApplyRequest",
  "types": {
    "ApplyRequest": {
      "tagging": "not-tagged",
      "unknown-keys": "silently-dropped",
      "struct": [
        { "key": "run_uid",        "type": "text",                 "presence": "required" },
        { "key": "known_revision", "type": "integer",              "presence": "required" },
        { "key": "submission_id",  "type": "text",                 "presence": "required" },
        { "key": "declare",        "type": { "seq": "Declaration" }, "presence": "optional" },
        { "key": "delegation",     "type": "DelegationAct",        "presence": "optional" }
      ]
    },
    "Declaration": {
      "tagging": "not-tagged",
      "unknown-keys": "refused",
      "struct": [
        { "key": "subject",  "type": "id",     "presence": "required" },
        { "key": "parent",   "type": "id",     "presence": "sparse"   },
        { "key": "needs",    "type": { "seq": "id" }, "presence": "sparse" },
        { "key": "dispose",  "type": "Dispose", "presence": "optional" }
      ]
    },
    "Dispose": {
      "tagging": { "internal": "form" },
      "enum": [
        { "token": "create",      "inlines": "CreateRecord" },
        { "token": "adopt",       "payload": [ { "key": "record", "type": "text", "presence": "required" } ] },
        { "token": "unresolved",  "payload": [ { "key": "note",   "type": "text", "presence": "required" } ] },
        { "token": "non-durable", "payload": [ { "key": "note",   "type": "text", "presence": "required" } ] }
      ]
    },
    "WireFacetValue": {
      "tagging": "untagged",
      "enum": [
        { "shape": { "seq": "text" } },
        { "shape": "text" }
      ]
    },
    "AgentAct": {
      "tagging": "external",
      "enum": [
        { "token": "blocking-set-declared",
          "payload": [ { "key": "blocking", "type": { "seq": "id" }, "presence": "required" } ] },
        { "token": "drafting-ready", "bare-string": true }
      ]
    }
  }
}
```

Three keys in that fragment do not exist in `sec-2`'s model and are the JSON's
half of the four expressibility findings: `inlines` (`fnd-14`), the token-less
variant with a bare `shape` (`fnd-11`), and `bare-string` (`fnd-12`).

Also unspecified by `sec-2`: the serde spelling of the model's own enums.
Derived defaults would emit `"NotTagged"`, `{"Internal":"form"}`,
`"SilentlyDropped"` — PascalCase, externally tagged. The fragment above assumes
`rename_all = "kebab-case"` throughout, matching every other closed vocabulary in
the crate. Not a finding — an implementation choice the design leaves open, noted
so it is not re-derived.

---

## 4. Findings raised — `fnd-10`..`fnd-19`

Raised on run `dr-019ffb40` and dispositioned there; the run holds the text. Ids
and one-line synopses only, per the notes convention.

Four are **expressibility** — the model as written cannot say what the wire does:

| id | concerns | what the model cannot say |
|---|---|---|
| `fnd-11` | `sec-2` | `VariantContract{token, payload:[KeyContract]}` fits neither of `WireFacetValue`'s untagged variants — no token, no keys, the variant *is* a shape |
| `fnd-12` | `sec-2` | `Tagging` is per-type, but `External` is not uniform per-variant: `AgentAct::DraftingReady` is the bare string `"drafting-ready"`, not `{"drafting-ready":{}}` |
| `fnd-13` | `sec-2`, `sec-3` | `WireType::Map(&WireType)` has no key slot. `AdoptAuthored.sections` is keyed by `DesignId`; `CreateRecord.facet`'s keys depend on the **sibling `kind` value**, so `sec-3`'s injected contract has nothing to attach to |
| `fnd-14` | `sec-2`, `sec-5` | `Dispose::Create`'s rendering `→ CreateRecord's keys, inlined` is not derivable — `VariantContract` has no slot naming the inlined type |

Three are **rendering** — the shape does not cover the content:

| id | concerns | what breaks |
|---|---|---|
| `fnd-10` | `sec-5` | `render_json` cannot be the derived `Serialize` of `PAYLOAD`; `Named` inlines recursively (5× `AcceptanceDeclaration`, 2× `Declaration`). Needs a flat name-keyed table |
| `fnd-17` | `sec-5` | the variant line carries one payload key; `DelegationAct::Propose` and `Provenance::ImportedProse` carry four, `Export`/`Refuse` two |
| `fnd-16` | `sec-2`, `sec-3`, `sec-8` | eleven struct blocks render, twelve are in the closure — `SubmissionEnvelope`'s keys are inlined at the root. `sec-8` pin 1 ("twelve types, one assertion body") counts the closure; say which |

Three are **facts traced against source**:

| id | concerns | correction |
|---|---|---|
| `fnd-15` | `sec-3` | three wire scalars, not two — `Fingerprint` (`ids.rs:213`) via `Provenance::ImportedProse`; and six files, not five — `ids.rs` |
| `fnd-18` | `sec-2`, `sec-3` | `Provenance` is a second internally-tagged enum (`tag = "provenance"`, two payload variants); `ReviewDisposition` is a second externally-tagged one. `sec-2`'s tagging examples name one of each |
| `fnd-19` | `sec-2`, `sec-3` | tension: `sec-2` says `Named` at an invented enum "would say something false about the payload", yet `sec-3` resolves the extern region by injecting a `TypeContract` for `knowledge::RecordKind` — a bare string field. Either `kind` takes `Token`, populated from the injection, or the rationale needs qualifying |

**What the closure derivation did *not* find.** `sec-3`'s twelve structs,
fourteen enums, their membership, the three-refuse/nine-discard split, the
thirteen-vs-nine asymmetry, the acyclicity claim and the deepest chain are all
confirmed against source. The post-`fnd-1..9` corrections held. What did not hold
is one tier down, and none of it was reachable without writing the rendering out.

## 5. What this artefact does not do

- It does not amend the design. Every disagreement is a run finding.
- It is not a golden. The eventual golden is `sec-5`'s
  `render_document()`-vs-disk comparison; this predates the generator and will
  differ from it in whitespace and ordering.
- It does not exercise `sec-8`'s pins. Whether `assert_keys_described` stays one
  generic body across twelve `Serialize` shapes remains the second of the three
  probes the self-review pass left open, and is unmoved by this.
