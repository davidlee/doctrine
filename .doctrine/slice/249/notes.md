# Notes SL-249: Knowledge facet write seam

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-06 · stage `design` (pre-run) · f31b6187

### Produced

- `SL-249` — this slice; scope card carries four objectives, four settled
  questions, R1/R2/R2a/R4 and A1/A2.
- `CHR-056` — SL-222's unretired dead `facet_write` float writers + stale marker.
- Research artefact — `.doctrine/slice/249/research/` (runtime tier), three
  threads, baseline stamped at f31b6187.
- `ISS-318` — widened from one instance to the inert-key defect class; absorbed
  the `body`-on-checkpoint instance from SL-248's run.
- `IMP-403` — lead 2 corroborated with the SL-248 evidence; related to `ISS-318`.
- `ISS-316` — absorbed as objective 4, narrowed to its lifecycle-vocabulary half.

### Learned

- The facet write seam already exists and ships: `facet_write::set_facet_mixed` /
  `apply_set_mixed`, consumer at `src/commands/facet.rs:711` serving
  `doctrine risk set`. Objective 1 is wiring.
- SL-222's `deletes at SL-222 deletion phase` marker covers only three
  float-valued symbols; its reason string's premise is false (the migration
  scripts are Python). → `CHR-056`.
- Two write postures already exist in `dep_seq` and the choice is forced:
  `apply_status` refuses a missing key (scaffold-seeded, F-1), `apply_scalar`
  creates one. Facet fields are scaffold-seeded blank, so F-1 applies.
- The facet field inventory is 31 slots / 30 distinct names / 1 shared — the fact
  that decided OQ-1.
- `src/commands/knowledge.rs` does not exist; the knowledge CLI is in
  `src/knowledge.rs`.

### Open

- `OQ-4` (slice card) — is `ConceptFacet`'s emptiness designed or an omission?
  No governance answers it; code corroborates "designed". The REV must rule.
- `OQ-6` (slice card) — does the inert-key refusal extend to `validate_facet`'s
  read path, or is read-tolerance deliberate?
- `R1` — the SPEC-019 amendment is owed a REV (ADR-013), not an in-place edit.
- `R2` — `EVD`/`HYP`/`CPT` facet contracts need rulings, not transcription of
  current code.
- `R2a` — ordering dependency: SL-249's REV lands before `SL-246` derives its
  per-kind field lists.
- Unverified limit: whether `PRD-010` also carries the four-kind framing (if so
  the REV grows to two entities); whether ADR-013's apply path can auto-apply a
  prose-heavy amendment.
- `CHR-056` — open, not a blocker.

## Design surface triage
<!-- exploring stage, runbook step `explore.triage`, design run dr-019fd6b6 rev 5 -->
as-of 2026-08-06 · stage `design` (run open, `exploring`)

### Constraining governance

Read this pass, in force, and binding on the design:

- **`PRD-010`** (Epistemic and Governance Records) — **newly verified, and it
  changes objective 4's scope.** Research left "does PRD-010 also carry the
  four-kind framing?" as an open Limit. It does, and more strongly than
  `SPEC-019` does: §4 carries it as a hard *constraint* — *"The kind set is
  exactly the four initial kinds — assumption (`ASM`), decision (`DEC`),
  question (`QUE`), constraint (`CON`) … and may not be extended without a
  reserved id."* The shipped corpus has seven. So the `EVD`/`HYP`/`CPT`
  divergence is not merely an unwritten enumeration in the tech spec; it is a
  **live contradiction of an active PRD constraint**. The REV grows to two
  entities, and the PRD half amends a constraint rather than swapping a numeral.
- **`SPEC-019`** (Knowledge-record entity surface) — the four-kind enumeration
  (responsibility 1, §"Four kinds, one engine") and the verb-set responsibility
  (`new`/`show`/`list`/`status` + uniform `link`/`unlink`/`supersede`; no
  `edit`, no settle verb). Also carries a now-false self-description: *"It is
  **forward-intent**: no code is shipped yet"* — a third amendment row.
- **`SPEC-004`** (Entity engine) — *"mutating verbs write entity TOML
  edit-preservingly"*. This is the binding mechanism constraint on objective 1:
  ride `toml_edit`, never reserialise.
- **`SPEC-013`** (CLI surface) — owns the uniform `<kind> <verb>` grammar and
  the listing spine, but states no convention for field-mutation verbs. `memory
  edit` / `backlog edit` are precedent, not governance. The subverb shape
  settled in OQ-1 is therefore unconstrained by it.
- **`ADR-013`** — the governance amendments route through a REV, not an
  in-place edit.
- **`ADR-004` / `SPEC-018`** — `link`/`unlink` own the relation seam; `edit`
  does not touch relations. Already a stated non-goal.
- **`PRD-019` / `SPEC-029`** (Managed design workflow / Design run engine) —
  govern the wire's identity, idempotency and revision-CAS, and say nothing
  about what a created record is *populated* with. Objective 2 is ungoverned
  space, not prohibited space.
- **`ADR-001`**, **`STD-001`**, **`STD-002`**, **`POL-002`** — ordinary; they
  shape the implementation, they do not gate it.

**Checked and found absent — `src/facet_write.rs` is anchored by no spec.**
No spec's `sources` list names it and the string appears nowhere in the
authored corpus (only in disposable runtime phase sheets). Positive control run
on both greps. `SPEC-020` governs the `[estimate]`/`[value]` *parse* side and
sources `estimate.rs` / `entity.rs` / `catalog/hydrate.rs`; the write module is
outside it. So objective 1 rides a module whose only governance is `SPEC-004`'s
one edit-preservingly clause.

### Shaping decisions (settled before the run opened)

Carried from the scope card, not re-litigated here: prose rides the wire
(was OQ-3); a dedicated settle verb rather than an optional `edit --answer`
flag (was OQ-2); facets only, lifecycle vocabularies stay on `ISS-316` (was
OQ-5); per-kind facet subverbs under a kind-blind `knowledge edit` (was OQ-1,
decided on the 31-slot / 30-distinct-name field inventory).

### Open questions carried into the run

- **`SL-249` `OQ-4`** — is `ConceptFacet`'s emptiness designed or an omission?
  The REV must rule; `edit`'s behaviour for a `CPT` falls out of the answer.
- **`SL-249` `OQ-6`** — does the inert-key refusal extend to `validate_facet`'s
  read path, or is read-tolerance deliberate?

### Open questions this pass added

- **N1 — the PRD half of the REV amends a constraint, not an enumeration.**
  `PRD-010`'s wording anticipates extension (*"without a reserved id"*), so the
  amendment has a shape available to it beyond "four → seven". What that
  reserved-id clause requires, and whether the three shipped kinds satisfy it
  retroactively, is unanswered.
- **N2 — should the REV anchor `facet_write.rs` to a spec?** `SL-249` makes an
  unanchored write module load-bearing for a second entity family. Adding a
  source anchor is cheap; deciding *which* spec owns it (`SPEC-004` as shared
  substrate, or `SPEC-019` as the consumer) is a design call.
- **N3 — `SL-159` owes the same debt this slice is paying.** `SL-159` (EVD+HYP)
  scoped a *"Governance axis — routes through a Revision (ADR-013): cut after
  design, settle in reconciliation"*. No revision in the corpus amends
  `SPEC-019` or `PRD-010` on the kind set (`REV-013` touches `SPEC-019` on an
  unrelated `needs`/`after` row). `SL-197` added `CPT` with no governance axis
  at all. This is the **third instance** of the pattern the research round
  already flagged twice — `SL-222`'s promised-but-uncriterioned deletion, and
  `ISS-318`'s silent sink. Whether `SL-249`'s REV explicitly discharges
  `SL-159`'s debt (and whether `ISS-316` should record that lineage) wants a
  ruling, not a silent absorption.
- **N4 — where does the key→honouring-kind table come from?** Spelled as
  literals it becomes an eighteenth hardcoded record-kind prefix site
  (`mem_019f05f6550d7fc3b4fe0dbd4dacf7a7`, which records that the existing ~17
  have no drift canary and are findable only by grep). Derived from the typed
  `RecordFacet` model it cannot drift. The scope card says the table is
  authored once with one consumer; it does not say from what.
- **N5 — the settle verb's name, and its reach.** Governance does not
  constrain it. Whether the other kinds' resolving transitions
  (`ASM`→validated, `DEC`→accepted, `CON`→waived) take the same treatment is
  explicitly design's call per the scope card.
- **N6 — three bespoke `edit` verbs already share no machinery.** `memory edit`
  (13 flags, full transaction), `backlog edit` (status+resolution), `spec edit`
  (descent scalars). A fourth compounds it; extracting the shared transaction
  shape is the opportunity. Research says in scope *only if it stays cheap* —
  design decides.

### Risks and assumptions

Carried unchanged from the scope card: `R1` (the amendment is authorship, now
across two entities), `R2` (the three ungoverned kinds need rulings, not
transcription of code), `R2a` (`SL-246` ordering dependency — `SL-249`'s REV
lands first), `R3`-residual (seven subverbs + kind-blind `edit` + settle verb
is itself a surface to keep coherent), `R4`-lesson (**objective 4's criteria
must name observables, not intent** — the `SL-222` failure mode, and N3 shows
it has already recurred on this exact spec), `A1` (the read model is sound and
stays put), `A2` (**confirmed** — `Declaration` carries `deny_unknown_fields`,
`CreateRecord` sits inside it; corroborated independently by
`mem_019fd03e13397240b4eb05af218f5cf5`).

One scoping correction from memory: `mem_019ee9fd51d87aa38a2dfb31ad6c4eec`
establishes that a `toml_edit` **root** insert-if-missing is safe (it cannot
tail-land inside a trailing subtable), which reads at first glance like a
licence to drop the F-1 refuse. It is not — the memory scopes its own proof to
root keys and says so. `[facet]` fields are subtable-nested, so the F-1
in-place-edit posture the research round settled on stands.
