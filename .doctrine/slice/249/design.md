<!-- doctrine:section sec-1 -->
# 1. Design Problem

Between a ruling being known and a record holding it, Doctrine loses content —
twice, silently, and at the moment the content is most available.

The **structured tier has no writer.** `doctrine knowledge` offers `new | list |
show | inspect | status | paths`. Filling a `[facet]` means hand-editing TOML,
and the corpus records the result: decision facets 35/142 populated, question
4/38, and `answer`/`answered_by`/`answered_on` on `QUE` at **0 of 38**.
Population is binary and lock-step — a record carries every textual field or
none — which is the signature of a population that either engaged the tier
deliberately or never touched it at all.

The **design-run wire has no slot.** `CreateRecord` accepts `kind`, `title`,
`slug` and `acceptance`. An agent disposing a checkpoint holds the ruling in
hand as it writes, and has nowhere to put it. On SL-248's run six dispositions
reached for the nearest-looking key and sent their prose as `body`;
`Declaration::body` is *section* prose, read only behind a guard on
`derived.section_digests`, so on a `cp-` subject it was accepted and discarded.
`DEC-155`…`DEC-160` minted hollow and ~15k tokens were re-authored. The operator
caught it, not the tool.

The two are one story — a missing slot and a silent sink — and fixing the sink
alone converts a silent loss into a dead end.

What must be true at the end:

- Every facet field of every facet-bearing kind is writable through a verb, and
  the prose tier with it.
- A `form = "create"` disposition mints a record already holding its facet and
  its prose, in one act.
- A wire key that is inert at its subject's kind is refused at submission,
  across the whole field set rather than at the instances someone noticed.
- The kinds the corpus actually has are the kinds governance describes, and that
  correspondence is checked by something other than a promise.

<!-- doctrine:section sec-2 -->
# 2. Current State

## The read side is complete, and stays put

`src/knowledge.rs` carries the whole structured read model: `RecordFacet` as an
enum-of-structs with one variant per kind, four closed value-enums (`Confidence`,
`Basis`, `ConstraintSource`, `Provenance`), a kind-blind `RawFacet` superset for
the tolerant parse, `validate_facet` dispatching on `record_kind` through the
`"" -> None` seam, and a `#[cfg(test)]` `render_facet` pinning the byte-stable
round-trip. `A1` assumes this is sound and unchanged; nothing below revises it.

The facet field inventory it defines is the fact that shapes the CLI surface:
**31 slots, 30 distinct names, exactly one shared** (`confidence`, on assumption
and evidence) — assumption 8, decision 7, constraint 6, question 5, evidence 3,
hypothesis 2, concept 0.

## The write side is a single module with a single consumer

`src/facet_write.rs` holds `set_facet_mixed` / `apply_set_mixed`, a mixed-type
`[facet]` writer over `toml_edit`, serving `doctrine risk set` at
`src/commands/facet.rs:711`. Its `deletes at SL-222 deletion phase` marker covers
only three float-valued symbols and its reason string's premise is false; that is
`CHR-056`, not this slice. The module is **anchored by no spec** — no `sources`
list names it and the string appears nowhere in the authored corpus. Its only
governance is `SPEC-004`'s edit-preserving clause. (`inq-9`, open.)

Two write postures already exist and are distinguished deliberately in
`src/dep_seq.rs`: `apply_status` refuses a missing key (F-1: scaffold-seeded,
edit in place, never create), `apply_scalar` creates one. `DEC-170` rules facet
fields into the first class.

## The status verb is deliberately uncoupled

`set_record_status` documents itself: *"No resolution coupling: `status` and
`updated`, nothing else."* So a question reaches `answered` with its answer
fields empty, which is exactly the 0-of-38 above. `accepted_status` is the
counter-example worth noticing — it derives a per-kind answer from the kind's own
vocabulary rather than enumerating one, and says so.

## The design-run wire

`CreateRecord` (`src/design_run/submission.rs`) carries `kind`, `title`, `slug`,
`acceptance` — no facet, no prose. `Declaration` carries
`#[serde(deny_unknown_fields)]` (`A2`, confirmed), so an unknown key is already
refused; what is *not* refused is a **known** key inert at the subject's kind,
which is `ISS-318`. Its two observed instances are `body` on a `cp-` subject
(accepted and discarded) and `dispose` on an `inq-` subject (skipped by
`plan_checkpoints`, `src/commands/design.rs:882`).

Record creation runs through DEC-086's step sequence: reserve → materialise →
apply effects. `create_record` is the single fresh-creation path for all seven
kinds and its own doc forbids a second reservation+scaffold path;
`apply_record_effects` is step 5, already idempotent, already applying the
acceptance→status move and the `shapes` edge.

## Governance as it stands

`SPEC-019` is emphatically four-kind, repeating "four" at nine sites, and carries
a now-false self-description (*"forward-intent: no code is shipped yet"*).
`PRD-010` § 4 carries the same enumeration plus an extension rule. The corpus has
seven kinds. `SL-159` (`EVD`+`HYP`) and `SL-197` (`CPT`) each ruled their
contracts in a closed slice's design and neither landed the governance axis it
scoped — the debt this slice pays (`inq-7`, open on whether it discharges them
by name).

<!-- doctrine:section sec-3 -->
# 3. Forces & Constraints

## Binding

- **`SPEC-004`** — mutating verbs write entity TOML **edit-preservingly**. Ride
  `toml_edit`; never reserialise. This is the one clause governing
  `src/facet_write.rs` today.
- **`DEC-170`** — F-1 posture on facet fields: refuse an absent key, never
  create it; spell a cleared value as `""`, never by omission. Facet fields are
  scaffold-seeded, so an absent one is damage. `set_facet_mixed` currently
  *creates* missing keys, so riding it needs either a call-site guard or a
  posture parameter.
- **`DEC-088`** — a decision reaches `accepted` only through a content-bound user
  attestation applied at step 5. No verb this slice adds may offer a second
  route.
- **`DEC-168`** — the facet-and-prose write happens at step 5
  (`apply_record_effects`), not by pre-filling the scaffold. Forced by
  crash-resume: `materialise_record_at` re-scaffolds from a journal carrying only
  id, title and slug.
- **`ADR-013`** — the governance amendment routes through a REV landing at
  reconcile, over **two** entities (`SPEC-019`, `PRD-010` — `DEC-175`).
- **`ADR-004` / `SPEC-018`** — `link`/`unlink` own relations; `edit` does not
  touch them.
- **`ADR-001`** — leaf ← engine ← command, no cycles. The pure write seam is a
  leaf; the CLI verbs and the design-run wire are both consumers of it.
- **`STD-001`** — no magic strings. Every field name, status token and kind
  prefix in this design must have exactly one spelling with one owner.
- **`POL-002`** — platform independence: no host-project convention (cargo
  layout, `just` recipes) leaks into engine rules. `DEC-176`'s canary is a
  project-local test, never a `validate` rule.

## Not binding, and worth stating

- **`SPEC-013`** owns the uniform `<kind> <verb>` grammar but states no
  convention for field-mutation verbs. `memory edit` / `backlog edit` are
  precedent, not governance, so the subverb shape and `settle` are unconstrained
  by it.
- **`PRD-019` / `SPEC-029`** govern the design run's identity, idempotency and
  revision-CAS and say nothing about what a created record is *populated* with.
  Objective 2 is ungoverned space, not prohibited space.

## Mechanical

- **Rust has no reflection over struct fields.** Any correspondence between a
  type's fields and something else is authored, and the design question is only
  how it is kept total (`DEC-169`).
- **`Declaration` carries `deny_unknown_fields`; `ApplyRequest` cannot** (it
  carries a `#[serde(flatten)]` envelope). The facet payload therefore lands
  inside `Declaration`, where the guard already holds.
- **Serde's `skip_serializing_if` is total over `Declaration`**, so a
  fully-populated value serialises to exactly the wire key set — the pin
  `DEC-169` uses instead of a proc macro.
- **A `toml_edit` root insert-if-missing is safe; a subtable-nested one is not**
  (`mem_019ee9fd51d87aa38a2dfb31ad6c4eec`, which scopes its own proof and says
  so). `[facet]` fields are subtable-nested, which is why F-1 stands.

## Pressures in tension

The slice exists to close a live data-loss hole, and the user's standing
tie-breaker is to prefer whichever defensible answer lands that fix sooner. Set
against it: this design touches shared machinery (the entity engine, the design
run wire) where the behaviour-preservation gate applies, and it writes governance
that outlives the fix. `DEC-165`'s phase boundary is how both are honoured — the
wire fix ships first and alone, the facet-bearing surfaces follow.

<!-- doctrine:section sec-4 -->
# 4. Guiding Principles

Five, each earned by a ruling this run took rather than asserted as taste.

**Derive the correspondence; do not declare it.** Three separate tables sit in
this design — wire keys against subject kinds, facet keys against record kinds,
status tokens against their evidence fields. Every one of them is authored,
because Rust gives no reflection; none of them may be *checked* by hand. `serde`
key sets pin the first (`DEC-169`), name correspondence derives the third
(`DEC-178`), and the second is pinned against the typed model. A table that can
silently miss a new field is `ISS-318` recurring inside its own fix.

**Refuse where the operator is; report where the reader is.** The same damage
warrants opposite treatment on the two paths. A write-time refusal fires at the
record it names, with someone holding the thing that caused it (`DEC-170`). A
read-time refusal fires at every later reader of an unrelated record and breaks
the tool you would use to diagnose it (`DEC-177`). Symmetry is not the goal;
locality of the report is.

**One table, many consumers — never one table per consumer.** The per-kind facet
field sets are needed by the six subverbs, by `settle`'s coverage, and by the
doctor tripwire. Three hand-maintained copies is the failure this slice is
already paying for elsewhere.

**Make the wrong call unrepresentable where it costs nothing; refuse it clearly
where it does.** Per-kind subverbs make a wrong-field error unspellable, and
relocate rather than remove the kind-mismatch check — accepted, because the check
is one comparison against a value the id already resolves. Where a refusal is
what is left, it teaches: *"`ASM-003` is an assumption; use `knowledge edit
assumption`."*

**Ship the fix ahead of the governance that describes it.** `DEC-165`: the
rulings gated, objective 4 never did. The phase boundary is load-bearing and the
plan must hold it — the wire fix may not quietly absorb facet work to save a
phase.

<!-- doctrine:section sec-5 -->
# 5.1 System Model

## The shape of the change

One authored table, one pure write seam, four consumers. Nothing here is a new
subsystem; every box below already exists except the table.

```
                    ┌──────────────────────────────────────────┐
                    │ src/knowledge.rs                          │
                    │                                           │
  read (unchanged)  │  RawFacet ──validate_facet──▶ RecordFacet │
                    │      ▲                            ▲       │
                    │      │ serde key-set pin (P1)     │ P2    │
                    │  ┌───┴───────────────────────┐    │       │
                    │  │ facet_fields(kind)        │────┘       │
                    │  │ the authored partition    │            │
                    │  └───┬────────┬─────────┬────┘            │
                    └──────┼────────┼─────────┼─────────────────┘
                           │        │         │
              ┌────────────┘        │         └──────────────┐
              ▼                     ▼                        ▼
   six `edit <kind>` subverbs   `settle <ID> <state>`   doctor tripwire
   + kind-blind `edit <ID>`     (coverage derived)      (inert-key warning)
              │                     │
              └──────────┬──────────┘
                         ▼
              src/facet_write.rs  ── toml_edit, F-1 guarded ──▶  record-NNN.toml
                         ▲
                         │ (same seam, second caller)
              apply_record_effects (DEC-086 step 5)  ◀── CreateRecord{facet, body}
```

## The authored table

The one artefact this design adds:

```rust
/// One facet field: its key, and the shape the writer must emit.
pub(crate) struct FacetField {
    pub(crate) name: &'static str,
    pub(crate) shape: FieldShape,
}

pub(crate) enum FieldShape {
    Text,
    List,
    Closed(&'static [&'static str]),   // the variant tokens, from the enum's own serde form
}

/// Every field one record kind owns, in template order — the single authored
/// derivation of the per-kind field sets (STD-001).
pub(crate) fn facet_fields(kind: RecordKind) -> &'static [FacetField];
```

It lives in `src/knowledge.rs`, adjacent to `validate_facet`, because the table
*is* the data form of what that function's arms already say in code. Splitting
them across modules is how the two drift. (Cohesion pressure noted: that module
also carries the knowledge CLI, since there is no `src/commands/knowledge.rs`.
This design does not relayer it; see § 8.)

Authoring is forced — Rust has no reflection over struct fields, and `DEC-169`
already refused a proc macro written for one table. So the design question is
only how it is kept honest, and there are two independent pins:

- **P1 — union totality.** Derive `RawFacet`'s serde key set (it gains
  `Serialize`; every field is already `#[serde(default)]`) and assert it equals
  the union of `facet_fields` over `RecordKind::ALL`. A facet field added to the
  model and forgotten in the table is a test failure, not a silent absence. This
  is `DEC-169`'s read-through-serde idiom, second application.
- **P2 — per-kind placement.** For each kind, write every field the table gives
  it, read the record back through the untouched `validate_facet`, and assert the
  typed facet holds it. A field filed under the wrong kind is discarded on read —
  so P2 catches exactly what P1 cannot, using the read model as the oracle rather
  than a second list.

P1 and P2 together make the table total *and* correctly partitioned without any
name being typed twice.

## What each consumer takes from it

| consumer | reads | derives |
|---|---|---|
| `knowledge edit <kind>` | `facet_fields(kind)` | its flag set; a test asserts clap's arg names equal the table's |
| `knowledge settle` | `facet_fields(kind)` | settleable states, by `<state>_by`/`<state>_on` name lookup (`DEC-178`) |
| `doctor` tripwire | all kinds | key → honouring kind, for the warning message (`DEC-177`) |
| step-5 write | `facet_fields(kind)` | shape dispatch for the `toml_edit` emit |

This is what discharges the rider `DEC-177` and `DEC-178` both carry: the table
exists as data, so neither falls back.

## The write mechanism

`src/facet_write.rs::set_facet_mixed` is the mechanism, already live under
`doctrine risk set`, already `toml_edit` and therefore already edit-preserving
per `SPEC-004`. It creates missing keys, which `DEC-170` forbids for facet
fields. The seam gains a **posture parameter** rather than a guard at each call
site: one enum threaded to the insert decision, `RiskFacet`'s caller passing the
creating posture it has today and knowledge passing F-1. A call-site guard would
have to be repeated by every future caller and would be the parallel
implementation `AGENTS.md` forbids.

## The phase boundary

`DEC-165` splits the slice and the split is load-bearing, not cosmetic:

- **Phase A — the wire fix, no facet anywhere.** Objective 3's inert-key refusal
  (`Declaration` keys × design-run subject kinds, `DEC-169`'s serde-pinned table)
  plus the prose half of objective 2 (`CreateRecord.body` written at step 5 via
  the existing `entity::write_body`). Neither touches a record kind, a facet, or
  knowledge governance. Together they are exactly what would have prevented the
  SL-248 loss.
- **Phase B onward — the facet surfaces.** The table, the six subverbs, the
  kind-blind `edit`, `settle`, `CreateRecord.facet` at the same step-5 site, and
  the doctor tripwire.

The plan must hold the boundary: Phase A may not absorb facet work to save a
phase.

<!-- doctrine:section sec-6 -->
# 5.2 Interfaces & Contracts

## The CLI surface

Three shapes, splitting by *tier* as `OQ-1` settled — invariant fields
kind-blind, `[facet]` kind-dispatched — plus the coupled transition.

```
doctrine knowledge edit <ID> [--title T] [--tags a,b] [--body B|-] [--body-mode replace|append]
doctrine knowledge edit <kind> <ID> [--<field> V]…          # six kinds; flags = that kind's fields
doctrine knowledge settle <ID> <state> --by WHO [--<text-field> V]
```

Flag names are the field's own name, kebab-cased — `--validation-plan`,
`--waiver-reason`, `--decided-by`. The args stay clap-derive-declared, as the
rest of this CLI is, and `facet_fields(kind)` is their **oracle**: a test asserts
each subverb's arg names equal its kind's table row, so the two cannot drift. See
§ 5.5 for why the table does not build the args directly. This refines `DEC-178`'s illustrative
`--reason` to `--waiver-reason`; the ruling's substance is untouched.

`--body` / `--body-mode` (`replace` | `append`, `-` reads stdin) are lifted from
`memory edit` unchanged, and ride `entity::write_body`, which that verb already
uses. Nothing new on the prose tier.

## What `settle` derives, and the one thing it does not

`DEC-178` derives *coverage*: a state is settleable when the kind's facet carries
`<state>_by` and `<state>_on`. That is mechanical over `facet_fields` and yields
exactly four transitions.

It does not derive *which field holds the outcome*. `QUE`'s is `answer`, `CON`'s
is `waiver_reason`, and no naming rule connects either to its state token without
contrivance. So one small annotation is authored beside the table:

```rust
/// A resolving transition: its state, and the field whose content the
/// transition exists to capture. The actor/date pair is NOT here — it is
/// derived from the state token (DEC-178).
pub(crate) struct Settlement {
    pub(crate) state: &'static str,
    pub(crate) captures: Option<&'static str>,
}
pub(crate) fn settlements(kind: RecordKind) -> &'static [Settlement];
```

Four rows: `QUE` answered/`answer`, `ASM` validated/none, `ASM`
invalidated/none, `CON` waived/`waiver_reason`. Pinned two ways — every
`captures` name must appear in `facet_fields(kind)`, and the set of `state`
tokens must equal the set the by/on derivation yields. The second pin is the one
that matters: it means the annotation can add detail to the derived set but
cannot quietly extend it.

Stating this plainly rather than claiming full derivation: `DEC-178`'s reach
ruling stands, and the honest scope of "derived" is the coverage, not the whole
row.

## The pure seam

```rust
/// A caller's raw field assignment, before the table has seen it.
pub(crate) struct RawEdit<'a> { pub(crate) field: &'a str, pub(crate) value: RawValue }
pub(crate) enum RawValue { Text(String), List(Vec<String>) }

/// One validated mutation, ready to write.
pub(crate) struct FacetEdit { pub(crate) field: &'static FacetField, pub(crate) value: RawValue }

/// Validate raw assignments against the kind's table: every field is owned by
/// this kind, every `Closed` value parses to a variant, every `List` value is a
/// list. Pure — no clock, no disk, no io.
pub(crate) fn plan_facet_edits(kind: RecordKind, given: Vec<RawEdit>)
    -> Result<Vec<FacetEdit>, FacetEditRefusal>;

/// Apply planned edits to a record's TOML, edit-preservingly (SPEC-004).
pub(crate) fn apply_facet_edits(path: &Path, edits: &[FacetEdit]) -> anyhow::Result<()>;
```

The split is the project's pure/imperative rule: `plan_facet_edits` holds every
decision and is exhaustively testable without a filesystem; `apply_facet_edits`
is the thin shell over `facet_write::set_facet_mixed`. `settle` is
`plan_facet_edits` plus `set_record_status` in one act — it composes the two
seams rather than introducing a third.

A cleared value is `RawValue::Text(String::new())`, written as `""`. `DEC-170`
forbids clearing by omission, and `plan_facet_edits` has no way to express it.

## The write posture

`set_facet_mixed` gains one parameter:

```rust
pub(crate) enum KeyPosture {
    /// Create the key if absent — the posture `doctrine risk set` has today.
    Create,
    /// F-1: refuse if absent; the key is scaffold-seeded, so its absence is
    /// damage (DEC-170).
    RequirePresent,
}
```

`risk set` passes `Create` and behaves exactly as it does now — the
behaviour-preservation gate holds by construction. Knowledge passes
`RequirePresent`.

## The wire

```rust
pub(crate) struct CreateRecord {
    pub(crate) kind: String,
    pub(crate) title: String,
    pub(crate) slug: Option<String>,
    pub(crate) acceptance: Option<AcceptanceDeclaration>,
    /// The record's `.md` prose (phase A).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<String>,
    /// The record's `[facet]` fields, by field name (phase B).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) facet: BTreeMap<String, RawValue>,
}
```

**On calling it `body`.** The tempting alternative is a distinct name — `prose` —
on the reasoning that `body` is the key that ate SL-248's content. Rejected: the
loss was a **level** error, not a naming collision. `Declaration::body` is the
body of a *section*; `CreateRecord::body` is the body of a *record*; in both
cases it is the `.md` content of the thing the subject names, which is what
`entity::write_body` and `memory edit --body` already call it. A fourth spelling
for one concept costs more than it buys, and objective 3 is what makes the level
error loud — see the refusal below.

`facet` is a `BTreeMap<String, RawValue>` rather than a typed per-kind struct
because `CreateRecord.kind` is a runtime token; the map is validated by
`plan_facet_edits` against `facet_fields(kind)` at admission, which is the same
check the CLI runs, not a second one.

## Refusals

Every refusal below is typed and names its remedy. The first four are new; the
fifth is `ISS-318`'s.

| condition | message shape |
|---|---|
| subverb names a kind the id contradicts | `ASM-003` is an assumption; use `knowledge edit assumption` |
| `knowledge edit concept …` | concept records carry no facet fields; use `knowledge edit CPT-001` (`DEC-173`) |
| state has no settlement | `accepted` is not a settle transition for a decision; use `knowledge status` |
| facet key absent on write | malformed record 042: `[facet]` is missing `choice` — restore the key and retry; the file is left untouched (`DEC-170`) |
| `Declaration` key inert at subject's kind | `body` is inert at `cp-4`; it is honoured for `sec-` subjects. To carry a record's prose, use `dispose.create.body` |

That last message is the whole of the SL-248 recovery: the six dispositions that
sent prose as `body` would each have been refused, at submission, with the key
they were reaching for named in the refusal.

<!-- doctrine:section sec-7 -->
# 5.3 Data, State & Ownership

## The authored tables, and who owns each

Three correspondences exist in this design. Each has exactly one owner, one
spelling, and a pin that fails loudly rather than a convention that erodes.

| table | owner | pinned by |
|---|---|---|
| facet key → owning kind, with shape | `src/knowledge.rs`, beside `validate_facet` | P1 union vs `RawFacet`'s serde key set; P2 per-kind round-trip through `validate_facet` |
| resolving state → captured field | `src/knowledge.rs`, beside the above (four rows) | every `captures` name ∈ `facet_fields(kind)`; state set ≡ the by/on derivation |
| `Declaration` wire key → honouring subject kind | `src/design_run/submission.rs` | key set of a fully-populated `Declaration`'s serde form (`DEC-169`) |

The third is not the first two. It is `Declaration`'s ~16 wire keys against the
design-run *subject* kinds (`inq-`, `sec-`, `att-`, `fnd-`, `cp-`), touches no
record kind, and needs no knowledge governance — which is the fact that lets
Phase A ship without objective 4 (`DEC-165`, `DEC-169`).

## Who may write `[facet]`

Exactly two callers, both through `apply_facet_edits`:

1. the six `knowledge edit <kind>` subverbs and `settle`, at the CLI;
2. `apply_record_effects` at DEC-086 step 5, for a `form = "create"` disposition.

`doctrine risk set` remains a third caller of `facet_write` itself, on the
`[facet]` of a *backlog* entity with the `Create` posture — a different table in
a different kind's file. No other code path writes a knowledge `[facet]`, and
`plan_facet_edits` is the only way to construct a `FacetEdit`, so that is
enforced by the type rather than by convention.

## Who owns status

`set_record_status` owns the token, unchanged and uncoupled. `settle` **calls**
it; it does not write `status` itself, so there remains one writer and one
vocabulary check.

The exception is the one `DEC-088` reserves: a decision reaches `accepted` only
through `apply_record_effects`, bound to a content-derived digest. `settle` has
no route there — not by a guard, but because `accepted` is not in the derived
settleable set at all (`DEC-178`). The reservation is upheld by the shape of the
derivation rather than by a check someone could later relax.

## Where the payload lives during a mint

`CreateRecord`'s `body` and `facet` are **not** journalled, and do not need to
be. Recovery here is submission-keyed retry: the caller re-submits the same
`submission_id`, `plan_checkpoints` rebuilds the `MintPlan` from that request,
and `execute_mint` resumes at the first incomplete step against the id the
journal already names. The payload is present on the retry because the retry
carries it.

This is what makes `DEC-168`'s siting sound rather than merely convenient. The
rejected alternative — pre-filling the scaffold inside `create_record` — would
have needed the payload at step 4, which `materialise_record_at` reaches from the
journal alone (id, title, slug), so it would have forced the payload into the
journal. Step 5 needs no such widening.

The window `DEC-168` names stays open and is not new: between
`IntentState::Materialised` and `IntentState::Applied` the record exists hollow
on disk. Status and the `shapes` edge already land in that window. A facet and
prose write joins them on the same terms, and is idempotent on the same terms —
the same payload written twice produces the same bytes.

## Storage tiers

Nothing here changes the tiering, and it is worth stating which tier each new
thing lands in:

- **Code** — the three tables. Not data files: they are the typed model's own
  partition and travel with it (`POL-002` — no host-project state).
- **Authored** — `record-NNN.toml`'s `[facet]` and `record-NNN.md`. Both are
  committed and diffable; both are written edit-preservingly, so a hand-authored
  record and a verb-written one are byte-indistinguishable.
- **Runtime** — the design run's own state, including the checkpoint
  dispositions. Disposable; the records it mints are not.
- **Derived** — nothing added. The doctor tripwire computes its findings per
  scan and stores none.

## Scaffold seeding is the precondition for all of it

`install/templates/knowledge-*.toml` seed `[facet]` with every field of that kind
present and empty. That fact is what makes F-1 correct (`DEC-170`), what makes an
absent key mean damage rather than absence, and what makes the empty-string
clear spelling a round-trip rather than a mutation. The templates are therefore
load-bearing for the write posture, and the objective 4 REV should say so — a
future template edit that drops a seeded field would silently convert every
record of that kind into one the writer refuses.

<!-- doctrine:section sec-8 -->
# 5.4 Lifecycle, Operations & Dynamics

Five paths. Four are new; the fifth is the one that lost the content.

## A — filling a facet at the CLI

```
doctrine knowledge edit decision DEC-042 --choice "…" --rationale "…"
```

```mermaid
sequenceDiagram
    participant U as caller
    participant C as knowledge CLI
    participant P as plan_facet_edits (pure)
    participant W as apply_facet_edits → facet_write
    U->>C: edit decision DEC-042 --choice …
    C->>C: resolve_ref → (Decision, 42)
    C->>C: subverb kind vs id prefix
    Note over C: mismatch → refuse, naming the right subverb
    C->>P: (kind, raw assignments)
    P->>P: field owned by kind? value shape? closed-enum token?
    Note over P: any failure → typed refusal, nothing written
    P-->>C: Vec<FacetEdit>
    C->>W: (path, edits)
    W->>W: toml_edit; F-1 refuse if a key is absent
    W-->>U: DEC-042: choice, rationale
```

Every decision is in `P`, which has no filesystem. The shell does two things the
pure layer cannot: resolve the id and write the bytes. A failure anywhere before
`W` leaves the file untouched; a failure inside `W` leaves it untouched too,
because `facet_write` builds the whole document before it writes.

## B — settling

```
doctrine knowledge settle QUE-198 answered --by david --answer "…"
```

One act, three writes, in this order:

1. `plan_facet_edits` validates `answer` (the state's `captures` field, required
   — omitting it is a refusal, not an empty write), `answered_by` from `--by`,
   and `answered_on` from `clock::today()`.
2. `apply_facet_edits` writes all three.
3. `set_record_status` moves `status` and `updated`.

**Ordering is deliberate: evidence first, token second.** A crash between them
leaves a record whose facet says who answered it and when, still sitting at
`open` — visibly incomplete, and re-running the same command completes it. The
reverse order would leave `answered` with an empty answer, which is the exact
state the corpus is already full of and which nothing would flag.

`settle` is not atomic across the two files, and does not pretend to be. It is
*ordered* so that the surviving state is the honest one.

## C — minting a filled record

```mermaid
sequenceDiagram
    participant A as agent
    participant R as design apply
    participant M as execute_mint
    participant K as knowledge
    A->>R: declare cp-N dispose{form:create, kind, title, body, facet}
    R->>R: admission — Declaration keys inert at cp-? facet keys owned by kind?
    Note over R: either failure → typed refusal, run revision unchanged
    R->>M: MintPlan
    M->>K: steps 2–4 reserve → materialise (scaffold, hollow)
    M->>M: journal Materialised
    M->>K: step 5 — status, shapes edge, body, facet
    M->>M: journal Applied
    M-->>A: DEC-179
```

The facet payload is validated at **admission**, before any id is reserved. A
disposition naming a field the kind does not own is refused with the run
unchanged — no hollow record, no reserved id burned. That is the same
`plan_facet_edits` the CLI runs, not a second check.

**On a crash between `Materialised` and `Applied`:** the record exists hollow.
Re-submitting the same `submission_id` resumes at step 5 with the payload the
retry carries, and every effect there is idempotent — `set_authored_status`
writes only on a change, `append_edge` returns `Noop`, and a facet-and-prose
write of the same payload produces the same bytes. Nothing is removed or
rewritten to repair a runtime failure (`DEC-083`).

## D — the refusal that would have caught SL-248

```
declare: [{ subject: "cp-4", body: "## The two candidate sites\n…", dispose: {…} }]
```

Admission consults the `Declaration`-key table: `body` is honoured for `sec-`
subjects and inert for `cp-`. Refused, at submission, with the run's revision
unmoved and the message naming `dispose.create.body` as the key the caller
wanted.

All six of SL-248's dispositions take that path. The loss becomes a refusal the
agent can act on in the same turn, which is the whole of objective 3's value —
the content is still in the agent's context at the moment of the refusal, which
it was not by the time the operator noticed.

## E — the standing scan

`doctrine doctor` walks the corpus and warns for each `[facet]` key populated but
inert at its record's kind, naming the record, the key, and the kind that would
honour it. It reads; it never refuses and never repairs. `knowledge list` keeps
working on a damaged corpus, which is the whole point (`DEC-177`).

## Phase dynamics

`DEC-165`'s boundary in operational terms:

- **Phase A** ships paths **D** and the prose half of **C**. After it, the
  SL-248 class of loss cannot recur — a mis-keyed payload is refused and a
  correctly-keyed one lands.
- **Phase B onward** ships **A**, **B**, **E** and the facet half of **C**.

The ordering is not merely permitted by `DEC-165`; it is the ordering that gets
the observed defect closed first. Nothing in Phase A reads `facet_fields`, so the
table is not on its critical path.

## Failure posture, stated once

No path repairs. Every refusal above leaves the corpus exactly as it found it and
names what to do. The one operation that can leave partial state is **B**, and
it is ordered so the partial state is the one a human would rather find.

<!-- doctrine:section sec-9 -->
# 5.5 Invariants, Assumptions & Edge Cases

## Invariants

Each is a property a test asserts, not a habit.

- **I1 — byte-stable round-trip.** A record written by any verb parses to the
  same typed `RecordFacet` and re-renders to the same bytes. The existing
  `render_facet` round-trip suite is the oracle and stays green unchanged.
- **I2 — the table is total.** `⋃ facet_fields(k) over RecordKind::ALL` equals
  `RawFacet`'s serde key set. A model field with no table row fails the build's
  tests.
- **I3 — the table is correctly partitioned.** Every field the table gives a kind
  survives `validate_facet` for that kind. A field on the wrong row is discarded
  on read, so I3 catches what I2 cannot.
- **I4 — one writer per concern.** `status` only through `set_record_status`;
  `[facet]` only through `apply_facet_edits`; the `.md` only through
  `entity::write_body`. `plan_facet_edits` is the sole constructor of a
  `FacetEdit`, so I4's facet half is a type property.
- **I5 — `accepted` is unreachable from `settle`.** Not by a guard: the token is
  absent from the derived settleable set (`DEC-088`, `DEC-178`).
- **I6 — no facet key is ever created.** F-1 posture; an absent key is a refusal
  naming the record (`DEC-170`).
- **I7 — a refusal leaves the corpus byte-identical.** Every validation precedes
  every write, on both the CLI and the admission path.
- **I8 — `doctrine risk set` is unchanged.** It passes `KeyPosture::Create` and
  exercises the same code it does today; its suite stays green unchanged (the
  behaviour-preservation gate).
- **I9 — the wire-key table is total.** A fully-populated `Declaration`'s serde
  key set equals the table's (`DEC-169`).

## Assumptions

- **A1 (carried)** — the read model is sound and stays put. If the write path
  forces a change to `RecordFacet` or `validate_facet`, that is a `/consult`
  trigger, not a quiet edit.
- **A2 (confirmed)** — `Declaration` carries `deny_unknown_fields` and
  `CreateRecord` sits inside it, so the payload extension needs no serde fight.
- **A3 (to verify in phase)** — *every* kind's scaffold template seeds *every*
  field of that kind present and empty. `knowledge-decision.toml` is confirmed
  (`DEC-170`); the other six are assumed on the strength of the shared authoring
  convention and are cheap to check. **If any template omits a field, F-1 turns
  every existing record of that kind into one the writer refuses** — so this is
  the first thing Phase B verifies, not something to discover at the first write.
- **A4** — `RawFacet` can gain `Serialize` as a derive-only change. It is a
  private struct with no manual `Deserialize`, so nothing observable moves.

## Why the table does not build the CLI args

`facet_fields` is a runtime value; clap's derive is compile-time. Generating the
six subverbs' args from the table would mean the builder API for these six
commands alone, in a CLI that is derive-declared throughout — a second idiom, for
the benefit of not typing thirty flag names once.

So the args are derive-declared and the table is their **oracle**: one test per
kind asserting `Command::get_arguments()`'s names equal `facet_fields(kind)`
kebab-cased. Drift fails the test; the surface stays in one idiom. This is the
`accepted cost` shape the `OQ-1` settlement already used — relocate the check
rather than remove it, when relocation is one comparison.

## Edge cases

- **Concept.** No facet subverb (`DEC-173`); `knowledge edit concept CPT-001` is
  refused, naming the kind-blind verb. `knowledge edit CPT-001` reaches
  everything a concept has, since its content is its prose (`DEC-172`).
- **The one shared field name.** `confidence` belongs to both assumption and
  evidence, with the same closed enum. Per-kind subverbs make that unambiguous
  at the call site with no special case; the table simply lists it twice, and I2
  compares sets, so the duplicate is not a discrepancy.
- **Clearing.** `--choice ""` writes `""`; a list flag given no values writes
  `[]`. Clearing by omitting the key is unspellable (`DEC-170`).
- **Re-settling.** `settle` refuses when the record already holds the target
  state: a transition from a state to itself is not a transition. Amending an
  answer already given is `knowledge edit question --answer`, which is the verb
  for changing a field. This keeps `answered_on` meaning *when it was answered*
  rather than *when the command last ran*.
- **Withdrawn records.** `settle` refuses on a withdrawn status, reusing the
  existing predicate rather than a second list.
- **A hand-deleted facet key** surfaces at the next write as an F-1 refusal
  naming the record. It is *not* visible to `doctor`: the tripwire this slice
  adds warns on keys that are present and inert, not on keys that are missing.
  The mirror check is cheap and tempting, and is deliberately out of scope —
  adding a facet field to an existing kind would make every prior record of that
  kind trip it, so the missing-key direction needs a migration story this slice
  does not owe. Recorded as a follow-up.
- **A record predating a new facet field** is the same case seen from the other
  end, and is the reason the mirror check is not free. Whoever adds a facet field
  to a kind must also seed it into existing records, or every write to them
  refuses. The objective 4 REV should carry that as a stated consequence of the
  F-1 posture.
- **Large prose.** `--body -` reads stdin, as `memory edit` does. No size rule is
  invented here.

<!-- doctrine:section sec-10 -->
# 6. Open Questions & Unknowns

Three inquiry nodes stayed open through the blocking set, deliberately. One is
now answered; two resolve later, and *later* is the right place for them.

## Answered in drafting

- **`inq-12` — extract the shared edit transaction, or add a fourth bespoke
  verb?** Neither: `DEC-179`. The transaction is already extracted — `memory`,
  `backlog` and `spec` all ride `dep_seq` and `entity::write_body` — and what
  remains bespoke is each verb's flag set, which has no field in common with the
  others. SL-249 adds a caller and one parameter, not a duplicate. No refactor
  phase enters the plan.

## Open, and resolving at REV authorship

Both belong to reconcile, when the REV is actually written. Recording the
recommendation now so the authorship is not re-derived:

- **`inq-7` — does this REV explicitly discharge `SL-159`'s undelivered
  governance axis, and where is the lineage recorded?** Recommendation: yes, by
  name. `DEC-174` already elevates `SL-159`'s `EVD`/`HYP` rulings with citation
  and `DEC-172` does the same for `SL-197`'s `CPT`, so the REV is already paying
  the debt in substance; saying so explicitly is what lets `ISS-316` narrow
  honestly rather than absorb a second slice's obligation in silence. The
  countervailing consideration is that a REV claiming to discharge another
  slice's axis is asserting something about work it did not do — which is why
  this wants the REV's author to look at it, not a design-time ruling.
- **`inq-9` — should the REV give `src/facet_write.rs` a spec source anchor, and
  to which spec?** Recommendation: `SPEC-004`, as shared substrate. The module is
  the entity engine's edit-preserving `[facet]` write mechanism, kind-agnostic,
  now serving backlog risk facets and knowledge facets both; `SPEC-019` is a
  consumer, and anchoring a shared writer to one consumer is how the next
  consumer ends up outside governance. This slice makes the anchor more
  necessary, not less — it adds `KeyPosture`, a behavioural axis with no
  governing sentence anywhere.

## Unknowns

- **`A3` — do all seven scaffold templates seed every field of their kind?**
  Confirmed for `knowledge-decision.toml` only. Not a judgement call, just an
  unchecked fact, and the first thing Phase B checks: an omission makes F-1
  refuse every write to every existing record of that kind.
- **Can `ADR-013`'s apply path auto-apply a prose-heavy amendment?** Carried
  unverified from the scope card. It affects how the REV lands at reconcile, not
  what it says. Worth probing before reconcile rather than at it.

## Deliberately not asked here

- **The missing-key mirror of the doctor tripwire.** Cheap to build and out of
  scope: adding a facet field to an existing kind would make every prior record
  trip it, so it needs a migration story this slice does not owe (§ 5.5).
- **`src/knowledge.rs` carries both the typed model and the CLI**, since there is
  no `src/commands/knowledge.rs`. This design adds to both halves and makes the
  module larger. Splitting it is a real improvement and a different slice; doing
  it here would put a layering refactor in front of the data-loss fix.

<!-- doctrine:section sec-11 -->
# 7. Decisions, Rationale & Alternatives

Twelve rulings were taken on the design run `dr-019fd6b6` and each carries its
own context, alternatives and rationale as a durable record. They are cited here,
not restated — the reasoning lives in the record, and a summary that drifts from
it is worse than a pointer.

| record | ruling | where it binds |
|---|---|---|
| `DEC-165` | the governance amendment does not gate the wire fix | § 5.1 phase boundary, § 5.4 |
| `DEC-168` | filled records are written at DEC-086 step 5 | § 5.3, § 5.4 path C |
| `DEC-169` | wire-key tables are pinned by a serde key-set test | § 5.3, I9 |
| `DEC-170` | facet writes refuse absent keys; empty is `""` | § 5.2 `KeyPosture`, I6 |
| `DEC-172` | concept records carry no facet by design | § 5.5 edge cases |
| `DEC-173` | concept gets no facet subverb | § 5.2 refusals |
| `DEC-174` | `EVD`/`HYP` contracts are elevated from `SL-159` unchanged | objective 4 |
| `DEC-175` | `PRD-010`'s kind-set clause is a stale enumeration | objective 4, § 2 |
| `DEC-176` | kind coverage in governance is pinned by a canary | § 9 |
| `DEC-177` | read stays tolerant; inert keys are caught by `doctor` | § 5.4 path E |
| `DEC-178` | one settle verb, reach derived from facet names | § 5.2, I5 |
| `DEC-179` | the edit verbs already share their machinery | § 6, and the plan's phase set |

## Decided in drafting

These are design-local: they follow from the rulings above rather than standing
beside them, and they are recorded here because implementation depends on them.

- **D1 — one authored table, two pins.** `facet_fields(kind)` is authored because
  Rust has no reflection and `DEC-169` already refused a proc macro for one
  table. P1 (union ≡ `RawFacet`'s serde key set) and P2 (per-kind round-trip
  through `validate_facet`) are independent and between them total: P1 catches a
  field missing from the table, P2 catches one on the wrong row. *Alternative:*
  per-consumer tables. Rejected — three copies of the fact this slice exists to
  make writable.
- **D2 — `KeyPosture` on the writer, not a guard at each call site.** A call-site
  guard has to be repeated by every future caller and is the parallel
  implementation `AGENTS.md` forbids. The parameter also keeps `doctrine risk
  set` on its existing posture by construction.
- **D3 — the table is the CLI args' oracle, not their source.** `facet_fields`
  is runtime, clap's derive is compile-time; generating six commands through the
  builder API would introduce a second idiom for the sake of not typing thirty
  names once. A per-kind test comparing `Command::get_arguments()` to the table
  closes the drift instead (§ 5.5).
- **D4 — the wire's prose key is `body`.** *Alternative:* `prose`, on the ground
  that `body` is the key that ate SL-248's content. Rejected: that was a level
  error, not a naming collision, and `entity::write_body` / `memory edit --body`
  already own the spelling. Objective 3 is what makes the level error loud.
- **D5 — the create payload is validated at admission**, by the same
  `plan_facet_edits` the CLI runs, before any id is reserved. A bad payload
  therefore costs no hollow record and no burned id.
- **D6 — `settle` writes evidence before the status token.** Not atomic across
  two files and not pretending to be; ordered so the surviving state after a
  crash is a record that says who settled it and when while still sitting at its
  open status, rather than a settled status with an empty answer.
- **D7 — `settle` refuses a state-to-itself transition.** Keeps `answered_on`
  meaning *when it was answered*. Amending an answer is `knowledge edit
  question --answer`, which is the verb for changing a field.

<!-- doctrine:section sec-12 -->
# 8. Risks & Mitigations

## Carried from the scope card

- **`R1` — the amendment is authorship, not annotation**, now across two entities
  (`SPEC-019`, `PRD-010` — `DEC-175`) plus a third amendment row for SPEC-019's
  false self-description. *Mitigation:* `DEC-176`'s canary makes coverage an
  observable rather than a claim, and `DEC-172`/`DEC-174` reduce the authorship
  to elevation-with-citation rather than fresh judgement. What is left is prose,
  and prose is what `R4` warns about.
- **`R2a` — `SL-246` ordering.** `SL-249`'s REV lands first; `SL-246` then
  derives its per-kind field lists from governance. *Mitigation:* nothing here
  changes it, but note that `SL-246` can now derive from `facet_fields` in code
  ahead of the REV if it needs to — the table is the same fact, earlier.
- **`R3` residual — surface coherence.** Six subverbs, a kind-blind `edit`, and
  `settle`. *Mitigation:* `D3`'s oracle test keeps the flag names honest, and the
  refusal catalogue (§ 5.2) is the surface's teaching layer.
- **`R4` — objective 4's completion is easy to assert.** *Mitigation:* `DEC-176`.
  This is the risk the slice has already recurred on once (`SL-159`), so the
  canary is not belt-and-braces; it is the control.

## New, from drafting

- **`R5` — a template omits a seeded facet field (`A3`).** If any of the six
  unverified `knowledge-*.toml` templates omits a field, F-1 makes every existing
  record of that kind refuse every write. Impact is total for that kind and
  invisible until the first write. *Mitigation:* Phase B's first act is a test
  asserting every template seeds exactly `facet_fields(kind)` — which is a third
  application of the table as oracle, and cheap because the table exists.
- **`R6` — the phase boundary erodes under convenience.** Phase A is small and
  the facet work is adjacent; the temptation to "just add the table while we're
  here" is exactly how `DEC-165`'s ordering is lost, and with it the property
  that the data-loss fix ships first. *Mitigation:* the plan states the boundary
  as an exit criterion, not a preference — Phase A's exit asserts that no symbol
  from the facet table is referenced by anything it ships.
- **`R7` — adding a facet field later is a corpus migration, not an edit.**
  F-1's posture means a new field must be seeded into every existing record of
  that kind or every write to them refuses. *Mitigation:* state it as a
  consequence in the objective 4 REV, where whoever adds a kind or a field will
  be reading. This risk is created by `DEC-170` and is worth the trade; it is
  not worth leaving undocumented.
- **`R8` — `src/knowledge.rs` grows on both axes.** The module holds the typed
  model, the read seam, the CLI, and now the tables and three more verbs.
  *Mitigation:* none taken here, deliberately (§ 6). Splitting it in front of the
  data-loss fix inverts the slice's priority. Recorded so the next reader knows
  it was seen rather than missed.

## Assumptions restated as risk

`A1` — if the write path forces a change to `RecordFacet` or `validate_facet`,
the behaviour-preservation gate is at stake and the correct move is `/consult`,
not a quiet edit. Nothing in this design foresees one: the write path reads the
model's shape through a table and never mutates the model.

<!-- doctrine:section sec-13 -->
# 9. Quality Engineering & Validation

## The gate that constrains everything else

This slice touches shared machinery — the entity engine's write leaves and the
design-run wire — so the behaviour-preservation gate applies: the existing suites
are the proof and must stay green **unchanged**. Two of them matter most and
neither may be edited to accommodate this work:

- the knowledge round-trip suite (`render_facet` byte-stability, `I1`);
- `doctrine risk set`'s suite, which is what `KeyPosture::Create` exists to keep
  true (`I8`).

An edit to either is a signal that the design is wrong, not that the test is.

## Test surface by invariant

| invariant | test shape |
|---|---|
| `I1` round-trip | existing suite, unchanged |
| `I2` table totality | `RawFacet` serde key set vs `⋃ facet_fields` |
| `I3` partition | per kind: write every table field, read through `validate_facet`, assert present |
| `I4` one writer | type-level for facets (`plan_facet_edits` is the sole constructor); test for status and body |
| `I5` `accepted` unreachable | assert the derived settleable set excludes every `DEC` state |
| `I6` F-1 | write to a record with a hand-deleted key → refusal naming the record, file byte-identical |
| `I7` refusal is inert | every refusal case asserts the file's bytes before and after |
| `I8` `risk set` | existing suite, unchanged |
| `I9` wire-key totality | fully-populated `Declaration` serde key set vs the table |

Plus the three oracle tests the tables earn: clap args vs `facet_fields` (`D3`),
templates vs `facet_fields` (`R5`), and `settlements`' state set vs the by/on
derivation (§ 5.2).

## The criteria that close the slice

Restated from the scope card with what drafting changed:

- Every facet field of the **six** facet-bearing kinds round-trips through its
  kind's subverb; `knowledge edit concept` is refused, naming the kind-blind
  verb. *Test-verified.*
- A subverb naming a kind the id contradicts is refused with the correct subverb
  named. *Test-verified.*
- A `Declaration` key inert at its subject's kind is refused across the whole
  field set — and specifically, a `body` on a `cp-` subject is refused naming
  `dispose.create.body`. *Test-verified, and this is the SL-248 replay.*
- A `form = "create"` disposition mints a record whose facet **and** prose are
  populated in one act with no follow-up write. *Test-verified; the criterion
  that closes the data loss.*
- `settle` populates the captured field, the actor and the date and moves the
  status in one act; omitting the captured field is a refusal; the settleable set
  is derived, not listed. *Test-verified.*
- A populated `[facet]` key inert at its record's kind is reported by `doctor`,
  and `knowledge list` still succeeds on that corpus. *Test-verified.*
- Every kind in `kinds::RECORD` is named in `SPEC-019` and `PRD-010`.
  *Test-verified by `DEC-176`'s canary — a project-local test, never a `validate`
  rule (`POL-002`).*
- `IMP-403` leads 1 and 2 are demonstrably closed; leads 3–5 carry their own
  follow-up items.

## What evidence changes

The `answer`/`answered_by`/`answered_on` population on `QUE` is the slice's
headline measurement at 0 of 38. It is **not** a closure criterion: this slice
closes the hole, it does not backfill the corpus, and a criterion that moved with
authoring behaviour would measure the wrong thing. The honest post-slice
measurement is that a record minted through a create disposition after Phase B
carries its facet — which the mint test asserts directly.

<!-- doctrine:section sec-14 -->
# 10. Review Notes

Where a reviewer should press, in the order I would press.

1. **`D1`'s two pins — are they actually total together?** P1 compares sets, so a
   field listed under two kinds is invisible to it (and `confidence` legitimately
   is). P2 is per-kind and would not notice a field that appears on an extra row
   *and* validates there. The claim is that no facet field validates for a kind
   that does not own it, which holds because `validate_facet`'s arms read
   disjoint field sets — but that is an argument about the current code, not a
   property the pins enforce. If a reviewer wants a third pin, this is where it
   goes.
2. **`D6`'s ordering argument.** It assumes a crash between two writes is the
   failure mode worth optimising for. If the more likely failure is a refusal
   *inside* `set_record_status` after the facet write has landed — a foreign-kind
   state, say — then the record ends with settlement evidence it never earned.
   The mitigation would be validating the state token before writing anything;
   worth deciding explicitly rather than inheriting.
3. **`DEC-177`'s scope, revisited against `D5`.** The doctor tripwire exists
   because the read path stays tolerant. But `D5` now validates facet payloads at
   admission, and the CLI validates at `plan_facet_edits` — so the only remaining
   producer of an inert key is a hand-edit. A reviewer might reasonably ask
   whether the tripwire still earns its place. I think it does, precisely because
   hand-edits are the population that gets no other feedback, but the argument is
   weaker than it was when the ruling was taken.
4. **`inq-7` and `inq-9` left open into reconcile.** Both are recorded with a
   recommendation (§ 6). If a reviewer thinks either should have been ruled here,
   the counter-argument is that both are about what the REV *says*, and the REV
   does not exist yet.
5. **`R8`.** `src/knowledge.rs` gains tables and three verbs and is already
   carrying the CLI. The design says explicitly that splitting it is out of
   scope. That is a judgement about sequencing, not about whether the module is
   too big — a reviewer who disagrees is disagreeing about priority, which is the
   user's call and is already recorded.
6. **Anything phase A touches that reads `facet_fields`.** `R6` says the boundary
   erodes under convenience. The cheapest review is a grep.

