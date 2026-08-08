# Notes SL-249: Knowledge facet write seam

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-08 · stage `started`, run `dr-019fd6b6` rev 89 `locked` · 5e4bba035
· `PHASE-01` `PHASE-02` completed, `PHASE-08` next

### Produced

- `SL-249` — this slice; four objectives, design locked, plan authored.
- `RV-349` — the design pass: 35 findings, all verified, concluded and disposed
  `conducted` (4d0f02428, 192039f5b).
- Design run `dr-019fd6b6` (runtime tier) — 17 nodes, locked at rev 89.
- `DEC-165` `DEC-168` `DEC-169` `DEC-170` `DEC-172` `DEC-173` `DEC-174`
  `DEC-175` `DEC-176` `DEC-177` `DEC-178` `DEC-179` — the twelve rulings.
- `DEC-182` — objective 4's REV lands in `PHASE-07`, not at reconcile; `shapes`
  this slice.
- `plan.toml` / `plan.md` — eight phases (3fbc17784, 396866db2, 192039f5b);
  runtime phase sheets materialised.
- `design-history.md` — the non-normative working history `sec-14` shed.
- `CHR-056` — SL-222's unretired dead `facet_write` float writers.
- `ISS-318` — widened to the inert-key defect class.
- `IMP-403` — lead 2 corroborated with SL-248 evidence.
- `ISS-316` — absorbed as objective 4, narrowed to its lifecycle half.
- `HYP-001` — the corpus's first hypothesis record.
- `mem.pattern.review.done-is-not-concluded` (3e2d8d28e).
- Friction observations `019fd6c8-ecbf-7b71-9001-a4ba464daf48`,
  `019fdfcc-ce76-77e0-8c30-7e3ce8603ae8`, `019fdfe5-72a7-7033-94b6-4debec579fd3`.
- Gate status: no code modified this session — the planning probe of the
  `dead_code` denial was reverted and `git diff` on `src/` is clean.
- `DEC-183` — `I10` quantifies over the subject-kind axis only; the `PHASE-02`
  `EN-2` ruling, accepted by the user (dd8a7f7b6).
- `ISS-327` — the state axis of `ISS-318`'s class. `ISS-328` — the nested
  `CreateRecord` sibling, which also corrects a false claim in `ISS-318`'s
  observation 3 and in
  `mem.pattern.serde.flatten-forbids-deny-unknown-fields` (both amended in place).
- `mem.pattern.testing.mapping-oracle-lives-below-the-check` — how `I10` pins a
  mapping without the table deciding its own verdict.
- `PHASE-02` (85c322373, 58efc3be9, 5e4bba035) — `Declaration::WIRE_KEYS` and
  `inert_key` in `submission.rs`, wired at `Batch::validate`;
  `Refusal::InertKey`; `IdKind::declarable`; `I9`, its predicate-agreement
  sibling, `I10`'s generated matrix, and the `SL-248` replay at the shell edge.
  `VT-1`–`VT-3` PASS under `slice verify-vt`; gate clean.
- `PHASE-01` (e76a95e6e) — `CreateRecord.body`; `knowledge::write_record_body`;
  `RecoveryIntent.payload_digest` + `resumable_under`; the `plan_checkpoints`
  payload digest lifted to unconditional and used twice. `VT-1`–`VT-4` PASS
  under `slice verify-vt`; gate clean.

### Learned

- The facet write seam already ships: `facet_write::set_facet_mixed` /
  `apply_set_mixed`, consumer at `src/commands/facet.rs:711`. Objective 1 is
  wiring.
- The facet field inventory is 31 slots / 30 distinct names / 1 shared —
  re-derived against the current tree at plan time and unchanged.
- `src/commands/knowledge.rs` does not exist; the knowledge CLI is in
  `src/knowledge.rs`.
- `src/facet_write.rs` is anchored by **no spec**; its only governance is
  `SPEC-004`'s edit-preserving clause. → open `inq-9`.
- Objective 3's table is `Declaration` keys × design-run **subject** kinds, not
  facet keys × record kinds — why `DEC-165`'s phase split works.
- `Cargo.toml` `[lints]` sets `warnings = "deny"` + `unused = "deny"`, so
  `-D dead-code` is a **hard error**, not a warning — verified by probe. An item
  cannot land a phase ahead of its first production consumer without
  `cfg_attr(not(test), expect(dead_code, …))` on every link of the chain
  (`mem.pattern.lint.dead-code-staged-ahead-cfg-test`). This shaped the phase
  cut.
- `revision apply` auto-lands only `status` rows and **surfaces** prose rows for
  manual handling — resolves design §6's stated unknown, and is half of
  `DEC-182`'s argument.
- `entity::write_body` creates an absent file under both `BodyMode`s (§10 press
  item 4, now evidence).
- `doctor_checks.rs`'s `*_findings(root) -> Vec<Finding>` is the tripwire's
  precedent — not `catalog::scan`, which the design named and which does not
  exist under that name.
- **`CreateRecord` does NOT carry `#[serde(deny_unknown_fields)]`** — only
  `Declaration` does. So before `PHASE-01` a `dispose.create.body` key was
  **silently swallowed**, not refused. `EN-2`'s conclusion is unaffected (the
  extension needed no serde change), but the design's surrounding language, and
  the `PHASE-01` plan entry, both read as though the nested payload were guarded.
  It is not. Stronger evidence for the slice's premise than was expected, and an
  open sibling — see Open below.
- `PHASE-01/VA-1`, discharged 2026-08-08 against `git diff` of e76a95e6e: no
  facet field table, no `[facet]` write, no `RecordKind`-dispatched behaviour.
  The single match for `facet` in the added lines is doc prose on
  `RecoveryIntent.payload_digest` naming the future blast radius. `EX-6` holds.
- `EX-4` re-verified by grep over `src/commands/design.rs`: one payload digest
  expression (`:967`), feeding both `acceptance_digest` and the intent. No second
  digest domain.
- `-D dead-code` forces test and consumer into ONE step, not two: `MintPlan`'s
  new field would not compile until `execute_mint` read it, so `VT-2` could not
  be observed red by the ordinary red/green rhythm. Discharged with a **positive
  control** instead — the guard was temporarily short-circuited and `VT-2` failed
  at its `unwrap_err`, then restored. Any phase of this slice staging an item
  ahead of a consumer owes the same control.
- `src/design_run/` sites its unit tests in ONE `src/design_run/tests.rs`, not in
  per-file `mod tests`. The plan's `test_file` for `VT-3`/`VT-4` names
  `attestation.rs` — the module under test, which is what `verify-vt` gates on
  (keywords over that file, which pass). The tests themselves ride the existing
  seam rather than opening a parallel one. `slice conformance` surfaces the
  consequence as one **undeclared** file, `src/design_run/tests.rs`. Left
  undeclared deliberately: the selector is a scope statement and settling scope
  divergence is reconcile's, not a phase's. Visible is the correct resting state.
- `VT-2`'s fixture can only reach the `Applied` intent state. The six-step crash
  points are e2e-only by construction (`design.rs`'s `no_fault` doc: a crash is
  observable only across a process boundary), so the pre-write abandon hook is
  the only in-process way to leave an intent journalled — and it leaves it fully
  applied. The guard sits at step 1 and fires at every state, so the assertion is
  sound; the earlier states are covered by the unit predicate (`VT-3`/`VT-4`),
  not by the fixture. Stated so an audit does not read `VT-2` as wider than it is.
- **`EN-2`'s answer, and the thing it turned up.** Each `Declaration` wire key is
  consumed by exactly one arm of `declare` — a clean key → kind mapping. But four
  keys are read on only *one* of their honouring kind's two paths: `provenance`
  only where a node is created, `lifecycle` only where one is updated,
  `concerns` and `blocking` only where a finding is raised. So `I10` as written
  in design § 5.5 was **falsified by current behaviour**, not merely undefined —
  a `provenance` key on a held node is silently accepted, the third state `I10`
  says does not exist, at the *honouring* kind. `DEC-183` scopes `I10` to the
  kind axis; design § 5.5's wording is owed a narrowing correction at reconcile.
- **The oracle has to sit below the check.** `I10`'s effectful side is measured on
  `run::declare`, which never reads the table; the refused side on the full
  admission path, which does. Asserting *exactly one* per cell is what catches a
  swapped table — the disjunction alone ("effectful **or** refused") is satisfied
  by a table that refuses everything, which is the vacuous form. Proven by three
  positive controls, each of which fired with the right message.
- `Batch::validate` was the right home for the check: it is the batch's admission
  gate and runs before any arm has touched the working snapshot, and the shell's
  pass 1 (`commands/design.rs:1479`) runs it before any id is reserved — so
  `EX-1`'s *corpus untouched, revision unmoved* is a property of existing
  structure rather than something `PHASE-02` had to build.
- No existing e2e fixture was sending an inert key: the whole suite went green
  unchanged. Worth recording because `SL-244`'s retirement of `evidence` left
  three fixtures sending a dead key for two tasks
  (`mem.pattern.serde.flatten-forbids-deny-unknown-fields`), so the negative here
  is evidence, not an assumption.
- `pub(super)` on a fn whose return type is private trips `-D private-interfaces`.
  `Pending` was widened with it; its fields stay private, so nothing outside the
  module can read or build one.
- A review reading `done` is **not** concluded — `done` is derived from findings
  (ADR-007 D-C8), while a design run's `conducted` disposition needs
  `review.concluded`, set only by `doctrine review conclude`. →
  `mem.pattern.review.done-is-not-concluded`.

### Open

- `inq-7` `inq-9` — SL-159 lineage; a spec anchor for `facet_write.rs`. Both
  ride the `PHASE-07` REV with recommendations recorded in design §6.
- `inq-1` `inq-2` `inq-3` — framing parents, resolved by their children.
- `D8a` — `DEC-168`'s recorded rationale is known-false; the correction rides
  the `PHASE-07` REV for want of an amend verb.
- `PHASE-07/EX-11` — the `DEC-182` departure is carried to reconcile as a
  design-wording item; design §3 and §5.3 still say reconcile.
- **Design § 5.5's `I10` wording overstates what its test proves** — it asserts
  the disjunction over *every* submission, and the generated matrix quantifies
  over *some* submission at each kind. A prose correction owed at reconcile, per
  `DEC-183`. `ISS-327` and `ISS-328` carry the code half; neither is a blocker.
- `PHASE-02` widened `slice conformance`'s undeclared set from one file to three
  — `src/design_run/tests.rs` (`PHASE-01`), now also `src/design_run/ids.rs` and
  `src/design_run/run.rs`. Same judgement as `PHASE-01`'s and for the same
  reason: the selector is a scope statement, and settling scope divergence is
  reconcile's to do, not a phase's. Both new entries are one-line widenings
  (`IdKind::declarable`; `declare` and `Pending` to `pub(super)`).
- `R1` — the amendment is authorship across two entities.
- `R2a` — ordering: SL-249's REV lands before `SL-246` derives its field lists.
- `IMP-403` leads 3–5 — owed as backlog items at close, not by any phase.
- `CHR-056` — open, not a blocker.
- Standing user steer: where two answers are defensible, prefer the one that
  lands the fix sooner.

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

## Review pass — RV-349, and what a further pass would probe

Written at the close of the design run's `reviewing` stage, after the pass
concluded. One external adversarial pass (codex, `RV-349`) over multiple rounds
against `design.md`, then one in-session raiser round over `sec-14` alone
(`F-21`–`F-23`, raised against revision 77 — not the external lane, which matters
because the run's review policy is adversarial-only). Every finding upheld on
evidence. The ledger is the record; this is the forward-looking half.

**Is another pass warranted? The in-session round revised this answer.** The
first form of this note said yes, but not of this artefact: the external pass had
reached the point where each round's findings were about the *record* of the
review rather than the design — `sec-14`'s accuracy about itself, and the
successive rules written to keep it accurate — and that had stopped paying for
itself. The design's substance did settle early and has been stable since; no
round after the first two changed an architectural choice. But the round that
followed this note found a **major** in the same artefact: `F-21`, the section
asserting that its own fingerprint bound its claims about *other* sections, when
`missing_lanes` matches an attestation on its subject's own bytes. That is not
the record-accuracy class. It is item 4 below — an unverified claim about code,
one grep from settling — and it was sitting inside the rule written to stop
exactly that pattern. The judgement to carry forward: the artefact is exhausted
for *review-record* findings and demonstrably was not for *unchecked-claim* ones,
which is the category the list below already ranks first by expected yield.

**What a further pass should probe** — carried in `design.md` § 10 in full, and
summarised here so the slice notes stand alone:

1. `I10`'s cell semantics — "observably effectful or refused" needs a per-key
   definition before the matrix test is written. The largest thing still
   undefined, and the place a test can pass while the mapping is wrong.
2. Whether `settle` still earns a separate verb now that `F-2` made it one
   write of one document. `DEC-062`'s argument survives; `DEC-178`'s
   coupled-multi-write argument does not.
3. `D8a`'s correction to `DEC-168`'s rationale rides the objective 4 REV for
   want of any other vehicle. Someone should check that is legitimate rather
   than merely available.
4. The design's remaining unverified code claims — `entity::write_body` on an
   absent file, `resolve_ref`'s refusal surface, `catalog::scan` as the
   tripwire's precedent. None was checked in any round; each is one command
   from being evidence or a finding. This is where a further pass has the best
   expected yield, because it is the category that produced `F-1`, `F-2` and
   `F-14`.

**The pass's two durable lessons**, for the plan and for the next design of this
size:

- *A totality asserted rather than enumerated.* The dominant defect class, in
  the design and in its own fixes alike. A claim of this kind inherits the
  credibility of the argument around it and so never attracts the one command
  that would settle it. Where the design now makes one, it states the
  enumeration or the identity beside it.
- *The artefact nobody re-reads.* The scope card drifted from the design more
  than once; a criterion widened past the card's own non-goals; a finding sat
  contested on the ledger while its substance was fixed elsewhere; `sec-14`
  drifted from the ledger repeatedly. In each case the artefact was the one
  updated last, by hand, after the substantive work. `review.scope` fires once
  at the end of a stage, which is the wrong cadence for a multi-round review.
  Mechanising that comparison is the improvement worth making, and it belongs
  to the design-run tooling rather than to this slice.
