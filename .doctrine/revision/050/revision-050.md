# REV REV-050 — Record kinds: four to seven

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

### 1. The amendment

The knowledge corpus has **seven** record kinds. Its governance describes
**four**. This revision closes that gap in the two entities that carry the
enumeration.

`src/kinds/mod.rs` defines `kinds::RECORD` as seven: assumption (`ASM`),
decision (`DEC`), question (`QUE`), constraint (`CON`), evidence (`EVD`),
hypothesis (`HYP`), concept (`CPT`). `SPEC-019` (Knowledge-record entity
surface) is emphatically four-kind — including a section heading, *"Four kinds,
one engine"* — and `PRD-010` (Epistemic and Governance Records) § 4 carries the
same enumeration. The three later kinds appear **zero** times in either
entity's authored tiers.

This is not a discovery. `SL-159` ruled the `EVD` and `HYP` contracts and
`SL-197` ruled `CPT`; each shipped its implementation and neither landed the
governance axis it scoped. The debt has been outstanding since, and it is the
debt `SL-249` objective 4 exists to pay.

Two decisions of this slice's design govern how it is paid:

- **`DEC-174`** elevates `SL-159`'s `EVD`/`HYP` rulings with citation, and
  **`DEC-172`** does the same for `SL-197`'s `CPT`. The contracts are therefore
  *carried forward with attribution*, not re-derived. This revision exercises
  judgement about where the rulings are recorded, not about what they say.
- **`DEC-175`** rules how `PRD-010` binds. Its kind-set clause is a stale
  **enumeration** plus a live **extension rule** — "may not be extended without
  a reserved id". Only the enumeration is stale. The rule names a precondition
  that `SL-159` and `SL-197` **met**: both reserved ids before extending. So the
  PRD half of this amendment is a *refresh of a fact*, not a reversal of a rule,
  and the extension rule is preserved **verbatim**.

Scope boundary, stated because an earlier draft of this work crossed it: what
is amended is the **enumeration**, the per-kind **facet contracts** for the
three new kinds, and the **verb set**. What is *not* amended is their
**lifecycle vocabularies** or **supersession rules** — `ISS-316` owns those, and
widening here would absorb its scope silently.

The three new kinds' facet contracts, from `src/knowledge.rs`:

| kind | facet fields |
|---|---|
| evidence (`EVD`) | `datum` (text), `provenance` (closed), `confidence` (closed) |
| hypothesis (`HYP`) | `proposition` (text), `predicts` (text) |
| concept (`CPT`) | **none** — `[facet]` empty by design (`DEC-172`, `DEC-173`); a concept's content is its prose |

`CPT`'s emptiness is a contract, not an omission: `knowledge edit concept`
**refuses**, because there is no facet to edit. That refusal is part of what
this revision records.

### 2. `inq-7` — the `SL-159` lineage: **accepted, by name, with a narrowed claim**

Design § 6 recommends that this revision explicitly discharge `SL-159`'s
undelivered governance axis and record the lineage, so that `ISS-316` can narrow
honestly rather than absorb a second slice's obligation in silence. § 6 also
records the countervailing consideration, and left the call to this author
because of it: *a revision claiming to discharge another slice's axis is
asserting something about work it did not do.*

**Dispositioned: accept the recommendation, with the claim stated precisely
enough to be true.**

The countervailing consideration is real, and the way past it is not to weaken
the claim but to say exactly which claim is being made. `SL-159` and `SL-197`
each had two obligations: to build the kind, and to govern it. **They discharged
the first themselves** — `EVD`, `HYP` and `CPT` are implemented, tested and in
the corpus; nothing here re-does or claims their implementation work. What they
did not land is the **governance axis**: the amendment to `SPEC-019` and
`PRD-010` that would make the corpus and its specs agree.

That second obligation is what this revision discharges, and it is discharged in
substance and not merely by assertion — `DEC-174` and `DEC-172` carry those
slices' own rulings forward with citation, which is why this is an act of
recording rather than of re-judgement.

So: **`REV-050` discharges the governance axis scoped by `SL-159` and `SL-197`
and left unlanded by both.** It makes no claim about their implementation.
`ISS-316` may narrow to the lifecycle-vocabulary and supersession questions for
`EVD`/`HYP`/`CPT`, which this revision deliberately does not touch.

### 3. `inq-9` — a spec anchor for `src/facet_write.rs`: **accepted, `SPEC-004`**

Design § 6 recommends anchoring `src/facet_write.rs` to `SPEC-004` (Entity
engine) as shared substrate. **Dispositioned: accept.**

The module is the entity engine's kind-agnostic, edit-preserving `[facet]` write
mechanism. It serves backlog risk facets and knowledge facets **both**, today.
`SPEC-019` is one of its consumers, and anchoring a shared writer to a single
consumer is precisely how the *next* consumer arrives outside governance — the
same failure shape this revision is here to repair on the kind axis.

§ 6's own sharpener applies with force: `SL-249` adds `KeyPosture` to this
module — a behavioural axis (present-key vs absent-key write posture) with **no
governing sentence anywhere in the corpus**. The anchor is therefore *more*
necessary after this slice than before it, not less.

Recorded as the third `[[change]]` row (`modify SPEC-004`). The row and this
disposition agree by construction: were `inq-9` declined, the row would have
been dropped.

### 4. Two consequences of the `F-1` write posture

`SL-249` ships a write posture in which **an absent facet key is a refusal, not
a default**. Two consequences follow that are not obvious from the posture
itself, and both are recorded here because they bind future work that this
revision's readers will do.

**(a) The scaffold templates are load-bearing for the write posture.** A future
template edit that drops a seeded field silently converts every record of that
kind into one the writer *refuses* — and the conversion is invisible until the
first write against a record of that kind. Nothing in the template's own
neighbourhood says so. Design `A3` establishes the posture is well-defined
today, by reading all seven templates during `RV-349` rather than assuming:
assumption 8 fields, decision 7, constraint 6, question 5, evidence 3,
hypothesis 2, and concept an empty `[facet]` annotated *"seeded for
scaffold-order invariant"* — § 2's 31-slot inventory matched exactly, the
degenerate kind included. `R5` keeps its test regardless, because what is at
stake was never one reading of the templates but the invariant across every
future edit of them.

**(b) Adding a facet field to an existing kind is a corpus MIGRATION, not an
edit.** This is `R7`. Because an absent key is a refusal rather than a default,
a new field must be **seeded into every existing record of that kind** at the
moment it is added. A field added to the type and the template alone leaves the
entire back catalogue of that kind unwritable, with no error until someone tries
to write one. Whoever adds a facet field owes the migration in the same change.

### 5. `D8a` — `DEC-168`'s rationale is corrected; its conclusion stands

`DEC-168` ("Filled records are written at mint step 5") records three counts
against pre-filling the scaffold. **Count (2), crash-resume, is false about the
code**, as `RV-349` `F-1` round two established: the resume arm takes only the
reserved id from the journal and takes `title`/`slug` from the plan rebuilt from
the retry, so step 4 reaches the payload **exactly as step 5 does**. The
rationale presents as mechanically forced a choice that was not.

**The conclusion is unchanged, on its real grounds.** Step 5 remains right
because `apply_record_effects` already exists, is already idempotent, and
already carries the acceptance-to-status move and the `shapes` edge — so the
content write joins effects that resume as a unit, instead of splitting one
record's write across two steps with different resume semantics. That is a
sound argument; it simply is not the argument the record made.

The correction is **executed** against `DEC-168` in this phase, using
`doctrine knowledge edit decision --rationale` — the verb `SL-249` shipped in
`PHASE-04`. Design § 7 anticipated that the correction would have to ride this
revision "for want of a verb to amend a knowledge record, which is the hole this
slice exists to close". The hole closed in time to be used on itself: the record
is amended directly, and this revision records that the amendment rode it rather
than substituting for it.

### 6. What this revision does not do

- It does not touch `EVD`/`HYP`/`CPT` **lifecycle vocabularies** or
  **supersession rules** (`ISS-316`).
- It does not re-derive the three kinds' contracts; it carries `SL-159`'s and
  `SL-197`'s rulings forward with citation (`DEC-174`, `DEC-172`).
- It does not relax `PRD-010`'s extension rule, which was complied with.

### 7. Landing point

Created, populated, approved and **applied** in `SL-249` `PHASE-07`, rather than
at reconcile as design § 3 and § 5.3 describe. That is `DEC-182`, settled by the
user on 2026-08-08. The departure from the design's wording is carried to
reconcile as `EX-11`; the design is not edited to match the plan.

The `modify` rows are expected to be **surfaced** by `revision apply` for manual
handling rather than auto-landed — `apply` auto-lands only `status` rows. The
prose amendments in this phase are that manual handling.
