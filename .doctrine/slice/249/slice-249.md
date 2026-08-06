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
   two observed instances. The correspondence table this needs is the same
   key→honouring-kind knowledge (2) needs, so it is authored once.

4. **Extend SPEC-019 to the kinds that actually exist** — ISS-316. The spec is
   emphatically four-kind (`assumption`, `decision`, `question`, `constraint`),
   repeating "four" at nine sites; the corpus has seven. `EVD`, `HYP` and `CPT`
   appear nowhere in it, so their facets, per-kind lifecycle vocabularies and
   supersession rules are ungoverned, and whether `ConceptFacet`'s emptiness is
   designed or omitted is unrecorded. This is a precondition, not a tidy-up: a
   write seam covering seven kinds cannot derive its per-kind field sets from a
   four-kind spec, and inventing them would be, in ISS-316's phrase, *invention
   presented as derivation*. The same pass registers the verbs this slice adds
   (R1), so SPEC-019 ends coherent on both axes.

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

- **Facets only, not lifecycle vocabularies** (was OQ-5). Objective 4's spec pass
  writes the `EVD` / `HYP` / `CPT` facet contracts, because the write seam
  load-bears on them. Their per-kind lifecycle vocabularies and supersession
  rules stay ungoverned and stay on ISS-316, which therefore does **not** fully
  close on this slice — it narrows to the lifecycle half.

**Still open for `/design`:** the CLI shape for ~5 facet fields × 7 kinds (OQ-1).
Candidates are one flat flag set that *refuses* flags inert at the record's kind
(ISS-318's rule applied to the second call site — one table, two consumers), a
per-kind subverb, or a repeatable `--facet key=value` validated against the kind.

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

## Risks, assumptions, open questions

- **R1 — the SPEC-019 amendment is authorship, not annotation.** Both spec
  divergences are now in scope (objective 4), which means this slice writes
  governance as well as code. The verb-set responsibility enumerates `new` /
  `show` / `list` / `status` plus uniform `link`/`unlink`/`supersede`, and must
  grow `edit` and the settle verb; the four-kind enumeration must become seven.
  A REV at reconcile is the mechanism (governance changes route through a
  Revision, ADR-013), not a quiet in-place extension.
- **R2 — the three ungoverned kinds need rulings, not transcription.** `EVD` and
  `HYP` have facets in code (`datum`/`provenance`/`confidence`;
  `proposition`/`predicts`) that the spec can adopt. `CPT` does not: its facet is
  an empty struct on the stated ground that *every concept rides its attributed
  prose body*, and whether that is a designed property or an omission is an open
  question ISS-316 raises and nobody has answered. Per-kind lifecycle
  vocabularies and supersession rules for all three are likewise unwritten.
  Transcribing the code would launder the current implementation into
  governance — the anti-pattern the spec exists to prevent.
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
- **R3 — surface creep on the flat-flag shape.** ~5 fields × 7 kinds is a large
  flag set on one verb; `memory edit` already carries 15 and reads as a wall.
  Mitigated by the refusal rule making the wrong flag an error rather than a
  silent no-op, but the ergonomics are the point of the slice and a bad shape
  fails it.
- **A1 — the read model is sound and stays put.** `RecordFacet`, `validate_facet`
  and `render_facet` are assumed correct and unchanged; this is additive on the
  write side. If the write path forces a change to the read model, that is a
  signal to re-enter `/consult`.
- **A2 — the design-run wire can carry a facet without a `deny_unknown_fields`
  fight.** `Declaration` has it; `ApplyRequest` cannot (it carries a
  `#[serde(flatten)]` envelope). `CreateRecord` sits inside `Declaration`, so the
  extension is assumed safe — worth confirming in design, not at execution.
- **OQ-1** — flat flags with refusal, per-kind subverb, or `--facet key=value`?
- **OQ-4** — is `ConceptFacet`'s emptiness a designed property or an omission?
  Raised by ISS-316; no governance answers it. Research found corroborating code
  evidence (`validate_facet`'s concept arm explicitly discards raw input) and a
  DEC-149 render-side aside, so the REV can *rule* with support rather than
  invent — but it must rule, because `edit`'s behaviour for a `CPT` falls out of
  the answer.
- **OQ-6** — does the inert-key refusal extend to the *read* path? Research found
  the same defect class one tier deeper: `validate_facet` silently discards a
  field belonging to another kind. There is a real argument for read-tolerance (a
  hand-edited corpus should not become unreadable), so this wants a deliberate
  ruling rather than consistency by reflex.
*(OQ-2, OQ-3 and OQ-5 settled by the user before design — see § Settled before
design.)*

## Verification / closure intent

- The `knowledge edit` verb round-trips every facet field of all seven kinds:
  write → `knowledge show` reflects it → re-parse is stable. Test-verified.
- A flag or key inert at the record's / subject's kind is **refused**, not
  ignored — proved at both call sites (`knowledge edit`, `design apply`) against
  one table. Test-verified.
- A `form = "create"` checkpoint disposition mints a record whose facet **and
  prose** are populated from the payload, in one act, with no follow-up write.
  Test-verified, and this is the criterion that closes the SL-248 data loss.
- Settling a question through the dedicated verb populates
  `answer`/`answered_by`/`answered_on` and moves the lifecycle in one act; the
  answer cannot be omitted. Test-verified.
- SPEC-019 enumerates seven record kinds with no residual "four", carries a facet
  contract for `EVD` / `HYP` / `CPT`, and lists the verbs this slice ships.
  Verified by agent against the spec text and `knowledge new --help`. ISS-316
  narrows to its lifecycle-vocabulary half rather than closing.
- The existing knowledge suites stay green unchanged (the behaviour-preservation
  gate for shared machinery).
- IMP-403 leads 1 and 2 are demonstrably closed; leads 3–5 are recorded as
  follow-ups with their own items rather than silently dropped.
