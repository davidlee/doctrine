# Review RV-366 — reconciliation of SL-259

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Surface reviewed.** `edge` at `503c5a2a3` — the slice ran in the primary
worktree, not through `/dispatch`, so there is no candidate interaction branch
and the reviewed surface is the branch itself. Implementation range
`138c31e88..503c5a2a3` (18 files, +3114/−396).

**Subject.** SL-259 *Truthful apply* — one invariant with four legs, six phases,
eight originating items (`ISS-450`, `ISS-367`, `ISS-315`, `ISS-333`, `ISS-328`,
`ISS-327`, `ISS-290`, `ISS-361`).

### Lines of attack

1. **Leg by leg, against `design.md` rather than against the plan.** The plan is
   not higher authority than the design. For each of the four legs: did the code
   deliver the *rule* the design states, or the instance the issue reported?
   `sec-1`'s own framing — that instance-by-instance repair is how `ISS-315`
   recurred three times — is the standard this slice is held to.
2. **Authored truth vs the tree.** `sec-9` `R0` says every file:line and count in
   this design is a claim to re-verify, and three failed external checking during
   `RV-365`. The design was locked at revision 56 and the implementation moved
   under it six times. Every divergence between `design.md` and the landed code is
   a finding, whichever side is wrong.
3. **The self-referential test.** This slice is about an exit signal telling the
   truth. Its own closing artefacts — `design.md` `sec-6`'s "no witness", `sec-3`'s
   four cells, `sec-7`'s code-impact table, the selector registry `conformance`
   reads — must not themselves lie about what landed.
4. **Verification integrity.** Every `VT` passes and the gate is green; that is
   necessary, not sufficient. Probe whether each pin fires for the *reason* the
   criterion names (`sec-8` — "a refusal that fires for the wrong reason is a test
   passing for the wrong reason"), and whether the waived/struck criteria
   (PHASE-06 `EX-5`, PHASE-04's fourth cell) were struck on evidence.
5. **Refusal-surface blast radius.** Three phases changed what `apply` refuses.
   Each new refusal is a behavioural break for an existing caller and for pins
   written by earlier slices (`SL-233`, `SL-249`, `SL-251`). Who owns the
   reconciliation of each?
6. **The residuals that were kept open deliberately** — `ISS-361`, `IMP-446`,
   `IMP-447`, `IMP-448`, `DEC-252`. Is each open for a stated reason with a
   trigger, or is it drift wearing a tracking id?

### Invariants held

- **Leg 1** — error ⇒ revision, receipt, change log unmoved; authored tier
  unmoved except across `SPEC-029`'s named late-watermark window.
- **Leg 2** — every material change emits its row, at the seam that performs it.
- **Leg 3** — unknown or inert input ⇒ typed refusal naming what was expected;
  refusal is a property of the request, never of whether anything changed.
- **Leg 4** — degrade what is history, refuse what is state; `ChangeEvent`
  untouched, both const proofs intact.
- **Governance** — `ADR-001` layering (`payload_contract` is a leaf), `STD-001`
  single-source, `STD-003` disclosed degradation, `SPEC-029`'s reserve-then-journal.

## Synthesis

**SL-259 delivered its invariant.** All four legs are in the tree, all eighteen
`VT` criteria pass (`doctrine slice verify-vt 259`), `doctrine check gate` exits
0, and the three `VA` criteria that needed discharging at audit are discharged
below. Nine findings, none a blocker. Every one of them is about what the slice
**says** about itself, not about what it does — which is a defensible place for
a slice's defects to be, and an uncomfortable one for this slice in particular.

### The legs, against the design rather than the plan

**Leg 2 — truthful rows.** `declare_node`'s create branch no longer returns
early; one row-producing path runs over two priors, so a `needs` edge declared
at creation emits `needs_added` (`ISS-450`). Both retain-by-kind act stores now
return the act they displaced — `CheckpointActGroup::record` and
`AgentDeclarationGroup::record` — and the caller turns that into `ActInvalidated`
at the seam that knows a replacement happened (`ISS-367`). `PHASE-01` `VA-1`
**discharged**: the grep the criterion names returns nothing because the
mechanism changed shape (`retain` became `position`+`remove`), so I took the
positive control instead — those two `record` functions are the only
displacement-by-kind sites in `snapshot.rs`, and both return `displaced`.
`live_acts` and `invalidation_rows` now *document their own blindness* rather
than carrying the claim they could not keep, and `invalidation_rows` states why
no dedup is owed: the two cases are disjoint, because an act present on both
sides of the difference is exactly what the derivation cannot see.

**Leg 4 — degrade history, refuse state.** `StoredRow` / `RawRow` /
`Unreadable` land inside `ChangeLog`, the ordered try classifies the cause at
the point of failure, and `revision`/`index` stay required so a structurally
broken row still refuses. I probed the classifier rather than reading it:
`term_cause`'s final arm infers `TermTooLong` from key-and-kind-read plus a
string value, which is only sound if nothing else can fail there — and it is,
because `PayloadTermWire` carries no `deny_unknown_fields` and `admit`'s length
bound is the sole remaining predicate. `PHASE-02` `VA-2` **discharged live**:
`doctrine design show 244` reads (`ISS-315`). `VA-1` **holds** — see `F-8`; both
`const _: ()` proofs are intact and the two forced pin edits tighten rather than
weaken.

**Leg 3 — typed refusal.** The walk runs before deserialisation and reaches
what serde structurally cannot. Probed live against this slice's own run:

```
$ doctrine design apply 259 --input /tmp/p.json     # {"…","cursror":"inq-2"}
Error: unknown key `cursror` at `cursror`: ApplyRequest admits `run_uid`,
  `known_revision`, `submission_id`, `adopt_authored`, `traversal`, `stage`,
  `acceptance`, `declare`, `delegation`, `discharge`, `review_policy`,
  `checkpoint_act`, `agent_declaration`
  doctrine design contract --format prompt
```

That is `EX-4` discharged in one line — the key, its dotted path, the admitting
type named, the full admitted list, the contract's address — and the run's
revision is unmoved, which is leg 1 demonstrating itself on the way past. The
published contract is pinned to the render by a golden test
(`design.rs:2826`), so `unknown-keys: refused` cannot drift from the code that
enforces it. `refuse_unknown_keys` has exactly **one** production call site; I
checked for a second parse route into `ApplyRequest` and there is none.

**Leg 1 — error implies nothing landed.** `PHASE-06` `VA-1` asks for an
enumeration, not a verdict, so here it is. Reading `apply` from `:1753`: after
`refuse_unresumable_mints` the remaining fallible calls are `execute_mint`
(journal/claim/materialise — effects and `.context()`'d invariant assertions,
no predicate over the request), pass 2, `write_journal`, the pre-write hook and
`recheck_watermark_before_write`. Pass 2's two resolution-dependent predicates
were closed structurally rather than argued away: `resolution_of` is now the one
expression both passes' key sets are built through, and `widest_canonical_id` is
a `const` proof over the whole `KINDS` table (`gate.rs`'s `widest_condition`
idiom) replacing `sec-6`'s byte-counting-in-prose. `apply_record_effects`'
facet arrives pre-validated at admission. **No refusal predicate remains between
`execute_mint` and pass 2's completion that could have run earlier.**

One asymmetry I looked at and am content with: pass 1 always resolves the review
plan while pass 2 resolves it only when `opening_review`, so pass 1's key set is
a superset. `review_pass` is consumed by an `if let` at `run.rs:495` with no
refusal, so the asymmetry cannot produce a pass-2-only refusal. It can produce a
silent no-op — which is `F-7`.

### The standing risks

**R0 came true, and it came true about the fix.** `sec-9` `R0` said every claim
in this design is to be re-verified at implementation because three failed
external checking during `RV-365`. Five of this audit's nine findings (`F-1`,
`F-2`, `F-3`, `F-8`, and `F-9`'s cousin) are exactly that class, found the same
way — by reading the tree against the prose. The design predicted its own
failure mode and was right. What is worth recording is that the implementation
*caught* them: `PHASE-04` `F-1` found the cell count, `PHASE-06` `F-1` found the
witness, and `notes.md` carried both to audit rather than letting them close
silently. The process worked; the artefact lagged.

**The slice's headline claim is the one it got least wrong.** `sec-6` asserted a
negative — *the hoist has no witness* — and `PHASE-06` reproduced one. It would
be easy to read that as the design failing at its own thesis. It is closer to
the opposite: the ruling (`DEC-250`, hoist on principle) was made *without* a
witness and turned out to have one, so the decision was right for a reason its
author did not have. The correction owed is to the evidence sentence, not to the
ruling. `ISS-361` stays open against the residual late-watermark window, exactly
as `sec-6` and `EX-5` insist, and was not closed on a plausible story.

### Tradeoffs consciously accepted

- **Nine redundant selector rows stay** (`F-4`). `selector doctor` flags every
  `design-target` under `src/design_run/` as subsumed by the `scope-relevant`
  glob. Removing them to silence the advisory would delete the signal
  `conformance` reads. The noise is the cheaper side.
- **`DEC-252` gets a reconciliation line, not a `REV`** (`F-6`). Stated in full
  in that finding's response; the short form is that plan criteria are
  immutable-append, so a `REV` would resolve to no legal write, and the real gap
  is discoverability.
- **`IMP-447` keeps an uninhabited `UnknownKeys::SilentlyDropped` arm**
  (`PHASE-03` sheet `D5`), and `IMP-446`'s `Declaration` split stays deferred
  under its stated trigger — first live non-empty `delegation`. Both are
  trigger-bound rather than prose-noted, which is the standard `sec-2` set.
- **`IMP-448`** — `entity.rs:557` hand-spells `kinds::canonical_id`'s format —
  is pre-existing `STD-001` debt surfaced by `PHASE-06`'s const proof, correctly
  filed rather than absorbed.

### What this audit did not do

It did not re-review the code for quality — that is `/code-review`'s facet, and
the slice has not had an implementation-facet adversarial pass. Given
`RV-365`'s hit rate on the *design*, one is worth considering before `/close`;
the honest counter is that `notes.md` records the implementation probing itself
hard at every phase (`PHASE-05` `F-4`'s hole in its own `VT` matrix is the
strongest example), and the gate plus eighteen green `VT`s is real evidence. I
flag it as the User's call rather than deciding it here.

## Reconciliation Brief

Seven of the nine findings write. Two (`F-5`, `F-7`) are captured as backlog
items and need nothing from `/reconcile`.

### Per-slice (direct edit)

**`design.md` — four sections, all the same class: the prose lagged a delivery
that was itself correct. Qualify by axis; do not strike, and do not touch the
rulings.**

- **`sec-3` §*The state axis*** (`F-1`): "Four cells … `DEC-246` refuses all
  four" → **three**. `PHASE-01` made `lifecycle` honoured at creation, closing
  that cell before leg 3 reached it; `plan.toml` already carries the correction
  (`f3558f3dd`, `PHASE-04` `EN-2`/`EX-1`). `DEC-246`'s ruling is unchanged.
- **`sec-6` §*What is actually defective*** and **`sec-8` leg 1** (`F-2`): both
  state the hoist has no witness. True of the **pass-1/pass-2 axis**; false of
  the **mint-loop axis**, where `PHASE-06` `F-1` reproduced one (`execute_mint`
  step 1 carried `SL-249` `D8`'s retry guard, so a two-checkpoint payload whose
  second plan trips it materialises the first plan's record, then refuses). Name
  the axis in both places. Record that the repair is `refuse_unresumable_mints`
  over the whole batch, and that `PHASE-06` `F-3` closed the pass-1/pass-2 axis
  structurally (`resolution_of`, `widest_canonical_id`) rather than by the
  observation `sec-6` rested on. `EX-5`'s refusal to manufacture a pass-2
  predicate stands untouched.
- **`sec-5` §*Preserve, and where the opacity lives*** (`F-8`): "`ChangeEvent`
  is not touched … every existing pin holds unchanged" → the **type** is
  untouched (closed, `Copy`, `const fn as_str`, both `const _: ()` proofs
  intact, no `Opaque` variant — all verified at audit); its **impl** gained a
  fallible `shaped()` at `PHASE-05`, and **two** `snapshot.rs` pins were
  re-patterned to `[StoredRow::Read(row)]`, which tightens them. Say all three.
  `PHASE-02` `VA-1` is adjudicated **held**.
- **`sec-7` §*Code impact*** (`F-3`): add the eight unlisted rows —
  `src/design_run/contract_check.rs` (new, leg 3's walk landed as a leaf sibling
  under `ADR-001`, not inside `payload_contract`), `refusal.rs`, `mod.rs`,
  `ids.rs` (`SubjectState`), `gate.rs` (`join` widened, `STD-001`),
  `render/change_row.rs` (**where the `STD-003` disclosure actually sits** —
  `sec-7` currently attributes it to `render/envelope.rs`), `fixture.rs`, and
  `install/design-payload-contract.md` (a deliverable of `EX-3`). Widen the
  `src/commands/design.rs` row past "`apply`'s check ordering" to name
  `refuse_unresumable_mints`, `resolution_of`, `widest_canonical_id` and the
  rewritten module doc.

**Selector registry — the load-bearing half of the conformance repair** (`F-4`).
`design.md` `sec-7` is the mirror; `slice-259.toml` is what `doctrine slice
conformance` reads, so the prose edit above does **not** clear the cell.

```
doctrine slice selector add 259 --intent design-target \
  src/design_run/fixture.rs src/design_run/gate.rs src/design_run/ids.rs \
  src/design_run/render/change_row.rs src/design_run/tests.rs \
  tests/e2e_design_review.rs tests/e2e_design_state.rs
```

(Confirm the verb's exact flags with `doctrine slice selector add --help`.) Then
re-run `doctrine slice conformance 259` — the seven source rows should clear;
the eighteen `.doctrine/` rows will not, and must not be selector-declared
(`F-5` / `IMP-449`).

**`DEC-252` — settle and link** (`F-6`). Currently `status = "proposed"` with an
empty `[facet]` and no relation edges, while `DEC-243`..`DEC-251` are all
`accepted` and related. It is the only in-band record of why `SL-251`
`PHASE-07` `VT-3` now reads red.

- `doctrine knowledge settle DEC-252` — `proposed` → `accepted`, populating
  `context` / `choice` / `alternatives` / `rationale` / `consequences` from the
  prose the record already carries.
- `doctrine link DEC-252 references --role concerns SL-251 --descriptor "…"` and
  the same to `SL-259`, so an auditor re-running `SL-251`'s gate reaches the
  supersession in one hop.
- Replace the record's closing *Carried forward* paragraph with this audit's
  ruling: **a reconciliation line, not a `REV`** — `PHASE-NN`/`VT-n` ids are
  immutable-append so a `REV` would resolve to no legal write; no governance
  artefact changed; the gap is discoverability and the link closes it.

**`DEC-250` — repair the mangled array** (`F-9`). Join `consequences[2]` and
`consequences[3]`; they are one sentence split at its internal comma. Cosmetic;
do not chase the cause unless a second record shows the same shape while editing.

### Governance/spec (REV)

**None.** No ADR, policy, standard or spec requires a revision. Checked
explicitly: `SPEC-029`'s reserve-then-journal protocol and its late-watermark
prescription are what leg 1 was built *to*, and the hoist moved checks earlier
without adding a second writer of the authored tier, so nothing in the spec
became untrue. `STD-003` is discharged, not amended (`RawRow.why` carries the
*why* the standard asks for). `ADR-001`'s leaf rule was obeyed, and the place it
bit — the walk could not import `crate::knowledge` to adjudicate `MapKey::Extern`
— was answered inside the slice (`PHASE-03` `EX-7`/`EX-8`) rather than by
bending the layering. `DEC-225` survives `DEC-252` intact.

### Deliberately not in this brief

- **`SL-233` `EX-11(a)`** (`PHASE-02` `F5` in `notes.md`): `DEC-249` inverted its
  wire half from *refuse the file* to *retain the row*. A closed slice's plan
  criterion is immutable-append and is **off-surface** for `/reconcile` — there
  is no legal edit. The inversion is recorded in `notes.md` and in
  `mem.fact.design-run.change-log-degrades-state-refuses`; that is the whole of
  what is owed. Same reasoning as `F-6`'s ruling, one slice over.
- **`PHASE-01` `D5`'s refusal-surface change** (a node declared `resolved` with
  no disposition now refuses where it was silently seated `open`): this is leg 3
  working as specified — `DEC-245`, refuse input the engine will not act on —
  not drift. `notes.md` asked for a line at audit against leg 1; the line is
  that leg 1 is untouched, because a refusal that fires *before* anything lands
  is precisely what "error ⇒ nothing landed" promises.
- **`ISS-361`**, **`IMP-446`**, **`IMP-447`**, **`IMP-448`** — all open with
  stated reasons and, where applicable, triggers. No reconciliation owed.

## Reconciliation Outcome

Every brief item written. **No `REV` authored** — the brief's governance/spec
section is explicitly *None*, and reconcile confirmed rather than re-derived
that: no ADR, policy, standard or spec became untrue.

`SL-259`'s design run (`dr-01a08e32`) is `locked` and `materialised`, so these
edits land out of band and the run's section fingerprints now diverge from
`design.md`. That is the surface working as designed at this stage
(`mem.pattern.reconcile.edit-design-out-of-band`): the regress →
`adopt_authored` → re-lock loop is for a run still governing execution, and
would spend two user acts and a fresh adversarial attestation on a slice about
to go `done`.

### Direct edits applied

- **`design.md` `sec-3` §*The state axis*** (`F-1`): four cells *named*, **three
  landed**. States that `PHASE-01` closed the `lifecycle` cell from the other
  direction — one row-producing path over two priors makes a `lifecycle`
  declared at creation land and emit, so nothing inert remains to refuse — that
  re-refusing it would regress `PHASE-01`, and that `plan.toml` already carries
  the correction (`PHASE-04` `EN-2`/`EX-1`). `DEC-246`'s ruling untouched.
- **`design.md` `sec-6` §*What is actually defective*** (`F-2`): the "no
  witness" assertion qualified to the **pass-1/pass-2 axis**, plus a new
  paragraph naming the **mint-loop** axis where `PHASE-06` `F-1` reproduced one
  (`execute_mint` step 1 carrying `SL-249` `D8`'s retry guard; a two-checkpoint
  payload materialises the first plan's record, then refuses), the repair
  (`refuse_unresumable_mints` over the whole batch), and why no fixture could
  see it (`PHASE-06` `F-2`). A further paragraph records that `PHASE-06` `F-3`
  closed the pass axis **structurally** — `resolution_of` as the one expression
  both key sets are built through, `widest_canonical_id` as a `const` proof over
  `KINDS` — rather than by the observation the section rested on. `DEC-250`'s
  ruling and `EX-5` untouched.
- **`design.md` `sec-8` leg 1** (`F-2`): same axis-naming on "no such instance
  is reachable", plus a line pinning the mint-loop axis, which did have one.
- **`design.md` `sec-5` §*Preserve, and where the opacity lives*** (`F-8`): the
  claim split three ways — the **type** untouched (variant set closed, `Copy`,
  `const fn as_str`, both `const _: ()` proofs intact, no `Opaque`), its
  **impl** gained a fallible `shaped()` at `PHASE-05`, and **two** `snapshot.rs`
  pins re-patterned to `[StoredRow::Read(row)]`, which tighten them.
  `PHASE-02` `VA-1` recorded as **held** in substance.
- **`design.md` `sec-7` §*Code impact*** (`F-3`): eight rows added
  (`render/change_row.rs`, `contract_check.rs`, `refusal.rs`, `mod.rs`,
  `ids.rs`, `gate.rs`, `fixture.rs`, `install/design-payload-contract.md`), the
  `commands/design.rs` row widened to name `refuse_unresumable_mints`,
  `resolution_of`, `widest_canonical_id` and the rewritten module doc, and the
  **`STD-003` attribution corrected** from `render/envelope.rs` to
  `render/change_row.rs` (verified at the source: `render_unreadable`,
  `UNREADABLE_REASON_KEY`, and its own `const _: ()` width proof). A closing
  note records the correction and points at the selector registry as the
  load-bearing half.
- **Selector registry** (`F-4`): `doctrine slice selector add 259 --intent
  design-target` over the seven paths. `doctrine slice conformance 259` now
  reads **18 conformant, 0 undelivered**, and the residual 18 undeclared are
  exactly the `.doctrine/` lifecycle-artefact class `F-5` measured — as the
  brief predicted. The nine redundant `selector doctor` rows stay, per `F-4`'s
  stated tradeoff.
- **`DEC-252`** (`F-6`): `proposed` → `accepted`; `[facet]` populated from the
  prose it already carried; `references --role concerns` edges to **both**
  `SL-251` and `SL-259`, so an auditor re-running `SL-251`'s gate reaches the
  supersession in one hop. The *Carried forward* paragraph is replaced by the
  ruling itself — a reconciliation line, not a `REV`, on `F-6`'s three reasons.
  **One correction to the brief:** it named `doctrine knowledge settle`, whose
  `<STATE>` enum is `answered | validated | invalidated | waived`; a decision's
  `proposed → accepted` is `doctrine knowledge status`, with `knowledge edit
  decision` for the facet.
- **`DEC-250`** (`F-9`): `consequences[2]` and `[3]` rejoined into one sentence.

### Escalated

- **`ISS-453`** — `F-9`'s escalation trigger fired. `DEC-243` carries **four**
  fragments of the same shape, and both records were mangled by the same commit
  (`a3b565571`, the CLI amendment against `RV-365`; neither carried the shape at
  `a3b565571^`). The mechanism is now established rather than suspected:
  `src/knowledge.rs:2952`/`:2958` declare `--alternatives` and `--consequences`
  with `#[arg(value_delimiter = ',')]`, so any prose element with an internal
  comma is split at every comma and reported as success. Records written through
  the design run's JSON payload are unaffected (`DEC-251`'s comma-rich
  `alternatives` are intact) — it is CLI-path-only. `DEC-243` was repaired
  alongside `DEC-250`, by hand: the CLI cannot repair either, since re-sending
  the joined sentence re-splits it. The fix itself is out of `SL-259`'s scope
  and needs a design call (repeatable flag vs. typed refusal), so it is filed,
  not taken.

### Withdrawn / tolerated / follow-up

- `RV-366` `F-5` — follow-up, captured as **`IMP-449`** (conformance's undeclared
  cell is majority lifecycle noise; platform-level, reproduced on `SL-256`).
- `RV-366` `F-7` — follow-up, captured as **`ISS-452`** (opening a review pass
  writes no row; site outside all four legs, `SL-259` did not touch it).

Nothing else from the brief remains. Two surfaces the brief explicitly named as
**off-surface** were not written and are not owed: `SL-233` `EX-11(a)` and
`SL-251` `PHASE-07` `VT-3`, both closed slices' plan criteria, immutable-append.

Reconcile pass complete — handoff to `/close`.
