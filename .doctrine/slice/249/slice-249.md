# Knowledge facet write seam

## Context

Knowledge records are two-tier — structured, queried data in `record-NNN.toml`,
prose in `record-NNN.md`. The **read** side of the structured tier is fully
modelled: `src/knowledge.rs` carries `RecordFacet` as an enum-of-structs with one
variant per kind, closed value-enums, a kind-blind `RawFacet` superset for the
tolerant parse, `validate_facet` dispatching on `record_kind`, and `render_facet`
for round-trip.

There is no **write** side. `doctrine knowledge` offers `new | list | show |
inspect | status | paths` and no `edit`. Filling a facet means hand-editing TOML.

The corpus shows the consequence. Measured during SL-246's research round
(2026-08-05, recorded on IMP-403): decision 35/142 facets populated, question
4/38, and `answer`/`answered_by`/`answered_on` on QUE is **0 of 38**. Population
is binary and lock-step — every record carrying one textual field carries all of
them. This is not authors tiring halfway; it is a population that either engaged
the structured tier or never touched it.

Two capture surfaces produce that split, and both are in scope here:

1. **No writer for the structured tier** (IMP-403 lead 1). The kinds that have an
   `edit` verb fill their fields; knowledge, which has none, does not.
2. **Creation seeds every field empty at exactly the moment the content is
   known** (IMP-403 lead 2). `knowledge new` scaffolds `[facet]` with all fields
   present and blank. So does the managed design run: `CreateRecord`
   (`src/design_run/submission.rs`) accepts `kind`, `title`, `slug` and
   `acceptance` — and no facet — although the agent proposing the disposition has
   the ruling in hand as it writes the checkpoint.

There is direct evidence that agents *try* to close gap 2 and are silently
defeated. On SL-248's design run (2026-08-06) six checkpoint dispositions reached
for the nearest-looking key and sent their prose as `body`. `Declaration::body`
is *section* prose, consumed only at `src/design_run/run.rs:1323` behind a guard
on `derived.section_digests`, so on a `cp-` subject it is accepted and discarded.
DEC-155…DEC-160 minted hollow; the loss was caught by the operator, not the tool,
and ~15k tokens of prose were re-authored. That silence is ISS-318, whose sibling
instance is the mirror (`dispose` sent to an `inq-` subject, skipped by
`plan_checkpoints` at `src/commands/design.rs:882`).

The two items are halves of one story: a missing slot and a silent sink. Fixing
the sink alone converts a silent loss into a dead end, which is why they are
sliced together.

## Scope & Objectives

Close the data-loss hole on the path from *ruling known* to *record filled*.

1. **A `knowledge edit` verb** — one read→mutate→write transaction over both
   tiers of a record: the typed `[facet]` fields in the TOML and the `.md` body.
   `memory edit` is the working precedent, including its `--body` / `--body-mode`
   (`replace` | `append`, `-` reads stdin) treatment of the prose tier. The write
   path rides the existing `RecordFacet` model; no new domain modelling.

2. **A facet payload on the design-run disposition wire** — extend `CreateRecord`
   so a `form = "create"` checkpoint mints a *filled* record in one act, rather
   than a hollow one plus a follow-up chore. Shares the pure write seam from (1)
   rather than reimplementing it.

3. **ISS-318's refusal** — a typed refusal when a `Declaration` carries a key
   inert at its subject's kind, naming the subject, its kind, the key, and the
   kind that would honour it. Total over the field set, not two patches for the
   two observed instances. *(The "one table, two consumers" rationale originally
   written here is superseded by the OQ-1 settlement — see § Settled before
   design. The table is still authored once; its consumer is the design wire.)*

4. **Extend SPEC-019 — and PRD-010 — to the kinds that actually exist** —
   ISS-316. The spec is emphatically four-kind (`assumption`, `decision`,
   `question`, `constraint`), repeating "four" at nine sites; the corpus has
   seven. `EVD`, `HYP` and `CPT` appear nowhere in it, so their facets, per-kind
   lifecycle vocabularies and supersession rules are ungoverned, and whether
   `ConceptFacet`'s emptiness is designed or omitted is unrecorded. The same pass
   registers the verbs this slice adds (R1), so SPEC-019 ends coherent on both
   axes.

   *(Amended in design.* **PRD-010 is the second amendment target** — the design
   triage found it carries the four-kind framing too, in § 4. DEC-175 rules how:
   the clause is a stale *enumeration* plus an *extension rule*, and only the
   enumeration is stale, because "may not be extended without a reserved id"
   names a precondition SL-159 and SL-197 met. So the PRD half is a refresh, not
   a reversal.

   **The "precondition, not a tidy-up" claim is withdrawn.** DEC-165 rules that
   objective 4 does not gate objectives 1–3. The precondition argument reaches
   objective 1 and the facet half of objective 2, which do need per-kind field
   sets — but not objective 3 or the prose half of objective 2, which live wholly
   inside the design-run wire and are together exactly what would have prevented
   the SL-248 loss. And it dissolves even there: the rulings objective 4 needs are
   taken in this design run either way, and ADR-013 lands the REV at reconcile
   regardless of build order. The *rulings* gated; objective 4 never did.

   **Completion is an observable, not an assertion** — DEC-176 gives it a
   ~10-line in-crate canary asserting every kind in `kinds::RECORD` is named in
   SPEC-019 and PRD-010. That is R4's demand discharged.*)

### Settled before design

- **Prose rides the wire too** (was OQ-3). The `CreateRecord` extension carries
  the record's `.md` prose as well as the structured facet. The SL-248 loss was
  *prose*; a facet-only extension would not have prevented it, and shipping one
  would leave the hole this slice exists to close.
- **Settling a question gets its own verb** (was OQ-2), rather than an optional
  `edit --answer` flag. DEC-062's argument inside the design run applies
  unchanged — a disposition is part of *resolving*, not a field one may forget —
  and `answer`/`answered_by`/`answered_on` at 0 of 38 is the corpus evidence
  that an optional flag is not enough. Design owns the verb's name and whether
  the other kinds' resolving transitions take the same treatment.

  *(Answered in design — DEC-178. The verb is a kind-blind `knowledge settle
  <ID> <state> --by <who> [--answer|--reason <text>]`, and its reach is derived
  rather than declared: a state is settleable exactly when its kind's facet
  carries `<state>_by` and `<state>_on`. That yields `QUE` answered, `ASM`
  validated, `ASM` invalidated and `CON` waived, and excludes `DEC` `accepted` —
  which DEC-088 reserves to the design run's attested path anyway. `knowledge
  status` stays as the uncoupled token move.)*

- **Facets only, not lifecycle vocabularies** (was OQ-5). Objective 4's spec pass
  writes the `EVD` / `HYP` / `CPT` facet contracts, because the write seam
  load-bears on them. Their per-kind lifecycle vocabularies and supersession
  rules stay ungoverned and stay on ISS-316, which therefore does **not** fully
  close on this slice — it narrows to the lifecycle half.

- **Per-kind facet subverbs over a kind-blind invariant verb** (was OQ-1). The
  CLI splits by *tier*, not by kind:
  - A kind-blind `knowledge edit <ID>` owns the fields every record has —
    `title`, `tags`, and the `.md` body via `--body` / `--body-mode`. The id's
    prefix resolves the kind; nothing is stated twice.
  - Kind-dispatched subverbs own the `[facet]` — one per record kind, carrying
    only that kind's fields and only its closed enums.

  This matches how the repo already separates `estimate` / `value` / `risk` from
  the entity verbs. ~~and it is consistent with the settled OQ-2: `knowledge
  answer QUE-NNN` is already a per-kind verb, so a kind-blind facet surface
  would sit beside it inconsistently.~~ *(That second argument is withdrawn:
  DEC-178 makes the settle verb kind-blind, so it no longer supports this
  conclusion. The conclusion stands on the field inventory below, which this
  section already calls the decisive fact.)*

  **The decisive fact is the field inventory**, which is larger and far more
  disjoint than "~5 × 7" suggested: 31 slots, **30 distinct field names, exactly
  one shared** (`confidence`, on assumption and evidence). Assumption 8, decision
  7, constraint 6, question 5, evidence 3, hypothesis 2, concept 0. A flat flag
  set earns its keep when the flags are mostly common across subjects —
  `memory edit`'s 13 flags all apply to every memory, which is why that precedent
  does not transfer. Here 30 flags would print on every `--help` while at most 8
  are legal for the record in hand.

  Also weighed: subverbs make the wrong-field error **unrepresentable** rather
  than refused, which is the codebase's stated preference (ISS-318's own option 2
  cites the `AgentAct`/`ActKind` split for exactly this reason); four distinct
  closed enums (`Confidence`, `Basis`, `ConstraintSource`, `Provenance`) stop
  having to coexist in one struct; and `CPT`'s empty facet becomes expressible as
  a subverb with no flags rather than 30 refusals. *(That last clause is
  superseded by DEC-173: concept gets **no** facet subverb at all, because a verb
  accepting nothing can only ever be invoked wrongly, and the kind-blind
  `knowledge edit CPT-001` already reaches everything a concept has. Six facet
  subverbs, not seven.)*

  **The accepted cost.** The kind is already in the id prefix, so a subverb
  states it twice and can disagree — `knowledge edit decision DEC-005` is
  redundant, and `… decision ASM-003` is wrong. That needs a mismatch refusal, so
  subverbs *relocate* the check rather than remove it. Accepted because it is one
  comparison against a value the id already resolves, versus a 30-row
  correspondence table consulted per flag. The refusal should carry the teaching
  error the flat option would have given: *"`ASM-003` is an assumption; use
  `knowledge edit assumption`."*

  **Consequence for objective 3.** The key→honouring-kind table is still built
  and still authored once, but its consumer is the design wire, not the CLI. The
  "one table, two consumers" framing in objective 3 above is superseded by this:
  one table, one consumer, and a CLI that does not need it. Design should not
  invent a second consumer to preserve the phrasing.

**Nothing substantive remains open for `/design`** beyond OQ-4 and OQ-6 below.
*(Both now answered — DEC-172 and DEC-177. Design run `dr-019fd6b6` took eleven
rulings in all; each amendment they force on this card is marked in place.)*

## Non-Goals

Bounded to IMP-403 leads 1 and 2. The remaining three leads are real and cheap
but turn a tight slice into a facet-quality programme; they are follow-ups, not
phases here:

- **Lead 3 — validation.** A `decision` may hold `status = "accepted"` with an
  empty `choice` and `doctrine validate` reports clean. An advisory warning is
  out of scope. (See also IDE-009, a knowledge lint verb.)
- **Lead 4 — the render conceals it.** `format_facet` emits the `[facet]` header
  only when an axis is populated while `facet_json` emits every field as `null`;
  the human path is the concealing one. DEC-149 already rules on marking an
  unfilled facet, and SL-246 owns the composed read.
- **Lead 5 — skill guidance.** Whether `/knowledge`, `/design` and
  `/record-memory` instruct an agent to fill the facet at all.
- **The `EVD` / `HYP` / `CPT` lifecycle vocabularies and supersession rules.**
  Ungoverned, and staying that way here — ISS-316 keeps them.

Also out of scope:

- **Backfilling the corpus.** This slice closes the hole; it does not fill the
  ~107 unpopulated decisions behind it.
- **Relations.** `link`/`unlink` own the relation seam (ADR-004, SPEC-018) and
  `edit` does not touch it — the `memory edit` and `backlog edit` precedent.
- **Status transitions.** `knowledge status` owns the per-kind lifecycle
  vocabulary; whether `edit` delegates to it (as `memory edit --status` does) or
  declines it is a design call, but the vocabulary is not re-litigated here.
- **The stale-binary sibling** noted on ISS-318 (`ApplyRequest` carries no
  `deny_unknown_fields`, so an older binary narrows a payload and reports
  success). Same class, different mechanism, separate change.

## Summary

*(Filled at close.)*

## Follow-Ups

*(Filled at close.)*

## Affected surface

- `src/knowledge.rs` — the typed facet model, its render/validate seam, **and the
  knowledge CLI** (verb enum + dispatch live here; there is no
  `src/commands/knowledge.rs`).
- `src/facet_write.rs` — an existing mixed-type `[facet]` writer with a live
  consumer, annotated for deletion. Ride it or resolve the marker; do not build a
  third one.
- `src/design_run/submission.rs` — `CreateRecord`, `Declaration`, and the
  subject-kind correspondence table.
- `src/design_run/admission.rs` — where the correspondence refusal belongs; the
  rule/record check already lives here.
- `src/design_run/refusal.rs` — the typed fault.
- `src/commands/design.rs` — `plan_checkpoints`, ISS-318's first instance.
- `.doctrine/spec/tech/019/` — the seven-kind extension and the verb-set
  responsibility (objective 4), landed through a REV.
- `.doctrine/spec/product/010/` — § 4's kind-set enumeration, the REV's second
  entity (DEC-175).

## Risks, assumptions, open questions

- **R1 — the amendment is authorship, not annotation.** Both spec divergences are
  now in scope (objective 4), which means this slice writes governance as well as
  code. The verb-set responsibility enumerates `new` / `show` / `list` / `status`
  plus uniform `link`/`unlink`/`supersede`, and must grow `edit` and `settle`;
  the four-kind enumeration must become seven. A REV at reconcile is the
  mechanism (governance changes route through a Revision, ADR-013), not a quiet
  in-place extension. *(Amended: the REV spans **two** entities, SPEC-019 and
  PRD-010 — DEC-175. It also carries SPEC-019's now-false self-description, "it
  is forward-intent: no code is shipped yet", as a third amendment row.)*
- **R2 — RETIRED by DEC-172 and DEC-174.** The risk was that the three
  ungoverned kinds would be transcribed from the current structs, laundering
  implementation into governance. It dissolves the same way for all three: the
  contracts are not code awaiting transcription but rulings in closed slices'
  designs that the code implements — `CPT` by SL-197's D2, `EVD` and `HYP` by
  SL-159's §5.3/D5. The REV therefore **elevates with citation** rather than
  inventing. Retained for its finding: the rulings were never ungoverned so much
  as *under*-governed, living where every later reader had to rediscover them.
  Per-kind lifecycle vocabularies and supersession rules remain unwritten and
  stay on ISS-316.
- **R2a — SL-246 is an ordering dependency, not a collision.** Downgraded by the
  research round: SL-246's notes record A3 — *the SPEC-019 four-of-seven gap
  stays outside this slice; the design labels any EVD/HYP/CPT field list as
  invention rather than deriving it*. It has explicitly handed the amendment off.
  SL-249's REV lands first; SL-246 then derives its per-kind field lists from
  governance.
- **R4 — RESOLVED, retained for the lesson.** The facet write mechanism is
  `src/facet_write.rs`'s `set_facet_mixed` / `apply_set_mixed`: live, unmarked,
  and already serving the shipped `doctrine risk set`. The
  `deletes at SL-222 deletion phase` marker covers only three float-valued
  symbols (the `[estimate]`/`[value]` shape SL-222 retired), and SL-222 scoped
  *"risk/tags survive"* from the start. Objective 1 rides the seam.
  **The lesson stands, though:** SL-222's PHASE-09 objective promised
  *"facet_write [value]/[estimate] machinery deletes"* while its exit criteria
  checked only a grep-gate, a tripwire suite and a green build. The deletion
  never happened, the audit did not catch it, and nothing owns it since.
  Objective 4 is the same shape — a prose-heavy amendment whose completion is
  easy to assert. **Its criteria must name observables, not intent.**
- **R3 — RESOLVED by the OQ-1 settlement.** The surface-creep risk on a wide flat
  flag verb is dissolved by per-kind subverbs; the residual is that seven
  subverbs plus a kind-blind `edit` plus a settle verb is itself a surface to
  keep coherent. Design owns the naming. *(Residual narrowed: **six** subverbs,
  not seven — DEC-173 — and the settle verb is named and bounded by DEC-178. The
  surface is `knowledge edit <ID>`, six `knowledge edit <kind> <ID>` subverbs,
  and `knowledge settle <ID> <state>`.)*
- **A1 — the read model is sound and stays put.** `RecordFacet`, `validate_facet`
  and `render_facet` are assumed correct and unchanged; this is additive on the
  write side. If the write path forces a change to the read model, that is a
  signal to re-enter `/consult`.
- **A2 — the design-run wire can carry a facet without a `deny_unknown_fields`
  fight.** `Declaration` has it; `ApplyRequest` cannot (it carries a
  `#[serde(flatten)]` envelope). `CreateRecord` sits inside `Declaration`, so the
  extension is assumed safe — worth confirming in design, not at execution.
  *(**Confirmed** in design: `Declaration` carries `deny_unknown_fields` and
  `CreateRecord` sits inside it.)*
- **OQ-4 — ANSWERED by DEC-172.** Designed, not omitted, and the answer already
  existed as SL-197's design-local D2: a structured `definition` field would
  duplicate the `.md` Definition section and give two sources of truth. The REV
  elevates that ruling rather than authoring a fresh judgement, and retires the
  "currently empty" hedge in both the code doc and the concept template.
- **OQ-6 — ANSWERED by DEC-177.** No, the refusal does not extend to
  `validate_facet`. `read_kind`/`read_all` propagate with `?`, so refusing on
  read would fail `knowledge list` corpus-wide on one hand-mangled record and
  could lock a design run out of `adoptable`. The detection duty moves to
  `doctrine doctor` as a key-presence tripwire in the shape
  `catalog::scan::check_facet_residue` already ships.
*(OQ-2, OQ-3 and OQ-5 settled by the user before design — see § Settled before
design.)*

## Verification / closure intent

- Every facet field of the six facet-bearing kinds round-trips through its kind's
  subverb: write → `knowledge show` reflects it → re-parse is stable.
  Test-verified. *(Six, not seven — DEC-173. `knowledge edit concept` is refused
  with a message naming the kind-blind verb.)*
- A subverb naming a kind the id's prefix contradicts is **refused**, with a
  message naming the correct subverb. Test-verified.
- A `Declaration` key inert at its subject's kind is **refused**, not ignored,
  across the whole field set rather than the two observed instances.
  Test-verified.
- A `form = "create"` checkpoint disposition mints a record whose facet **and
  prose** are populated from the payload, in one act, with no follow-up write.
  Test-verified, and this is the criterion that closes the SL-248 data loss.
- Settling a question through `knowledge settle` populates
  `answer`/`answered_by`/`answered_on` and moves the lifecycle in one act; the
  answer cannot be omitted. Test-verified. *(And the settleable set is derived,
  not listed — a test asserts it equals exactly those (kind, state) pairs whose
  facet carries `<state>_by`/`<state>_on`. DEC-178.)*
- SPEC-019 and PRD-010 enumerate seven record kinds with no residual "four",
  SPEC-019 carries a facet contract for `EVD` / `HYP` / `CPT`, and lists the
  verbs this slice ships. *(Pinned by an in-crate canary asserting every kind in
  `kinds::RECORD` is named in both — DEC-176, the observable R4 demanded. Agent
  verification of the prose sits on top of that, not in place of it.)* ISS-316
  narrows to its lifecycle-vocabulary half rather than closing.
- A `[facet]` key inert at its record's kind is reported by `doctrine doctor` —
  not refused on read. Test-verified. *(DEC-177.)*
- The existing knowledge suites stay green unchanged (the behaviour-preservation
  gate for shared machinery).
- IMP-403 leads 1 and 2 are demonstrably closed; leads 3–5 are recorded as
  follow-ups with their own items rather than silently dropped.
