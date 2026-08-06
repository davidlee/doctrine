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

**Design question deliberately left open for `/design`** (recorded here so it is
not settled by default): the CLI shape for ~5 facet fields × 7 kinds. Candidates
are one flat flag set that *refuses* flags inert at the record's kind (which is
ISS-318's rule applied to the second call site — one table, two consumers), a
per-kind subverb, or a repeatable `--facet key=value` validated against the kind.

Second open question: whether settling a question goes through `edit --answer` or
a dedicated verb that *requires* the answer. DEC-062's precedent inside the
design run — a disposition is part of resolving rather than a field one may
forget — argues for the latter, and `answer` is the worst-populated axis in the
corpus.

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

- `src/knowledge.rs` — the typed facet model and its render/validate seam; the
  new write path.
- `src/commands/knowledge.rs` — the CLI seam for the new verb.
- `src/design_run/submission.rs` — `CreateRecord`, `Declaration`, and the
  subject-kind correspondence table.
- `src/design_run/admission.rs` — where the correspondence refusal belongs; the
  rule/record check already lives here.
- `src/design_run/refusal.rs` — the typed fault.
- `src/commands/design.rs` — `plan_checkpoints`, ISS-318's first instance.

## Risks, assumptions, open questions

- **R1 — spec divergence.** SPEC-019 §"Front the family through one `doctrine
  knowledge` command namespace" enumerates the verb set as `new` / `show` /
  `list` / `status` plus uniform `link`/`unlink`/`supersede`. Adding `edit`
  diverges from the spec as written, so this slice owes SPEC-019 a REV at
  reconcile rather than a quiet extension.
- **R2 — ISS-316 is directly in the path.** SPEC-019 governs four record kinds;
  the corpus has seven (hypothesis, evidence, concept arrived later). A facet
  write seam must cover all seven, so this slice either resolves ISS-316's gap or
  ships a verb wider than its governing spec. `/design` decides which; it is not
  a discovery to be made mid-execution.
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
- **OQ-2** — does settling a question get its own verb (DEC-062's argument) or an
  `edit --answer` flag?
- **OQ-3** — does the facet payload on `CreateRecord` accept the `.md` prose too,
  or only the structured facet? The SL-248 loss was *prose*, so a facet-only
  extension would not have prevented it.

## Verification / closure intent

- The `knowledge edit` verb round-trips every facet field of all seven kinds:
  write → `knowledge show` reflects it → re-parse is stable. Test-verified.
- A flag or key inert at the record's / subject's kind is **refused**, not
  ignored — proved at both call sites (`knowledge edit`, `design apply`) against
  one table. Test-verified.
- A `form = "create"` checkpoint disposition mints a record whose facet is
  populated from the payload, in one act, with no follow-up write. Test-verified,
  and this is the criterion that closes the SL-248 data loss.
- The existing knowledge suites stay green unchanged (the behaviour-preservation
  gate for shared machinery).
- IMP-403 leads 1 and 2 are demonstrably closed; leads 3–5 are recorded as
  follow-ups with their own items rather than silently dropped.
