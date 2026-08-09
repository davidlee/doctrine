# Review RV-351 — reconciliation of SL-249

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode: conformance.** Post-implementation audit of `SL-249` (knowledge facet
write seam), eight phases, all `completed`, driven solo in the primary tree — not
dispatched, so the surface under review is `edge` itself at `05e0de495`, not a
candidate interaction branch.

**Entry state, observed not assumed.** `doctrine check gate` → exit 0.
`doctrine slice verify-vt 249` → 29 VT criteria across 8 phases, all `PASS`.
`git status --porcelain` → clean but for two paths foreign to this slice
(`.claude/settings.json`, `scripts/find-empty-reqs.sh`). Phase boundaries recorded
for all 8 phases; `PHASE-07`'s is `59f77ce10..8db126b9c`.

**The audit's unusual starting position.** `PHASE-07`'s `T10` wrote a reconcile
ledger into `notes.md` § *Open* before this audit opened — deliberately, because
`EX-11` required the phase to hand its departure forward rather than leave
reconcile to grep for it. So most of what an audit would normally *discover* is
already written down. That is a hazard, not a convenience: an audit that merely
transcribes a ledger the implementer wrote has verified nothing. The lines of
attack below are chosen to test that ledger rather than restate it.

### Lines of attack

1. **Does the ledger's own count survive re-derivation?** It asserts fourteen
   owed items against a brief that said twelve, and instructs reconcile to
   reconcile to fourteen. `R-inventory` — this slice's standing risk, five
   firings — says the number written down is the number most likely wrong. Count
   it independently before accepting it.
2. **The conformance algebra, read as the mechanical signal it is.** `undeclared`
   and `undelivered` are computed from recorded source-deltas and owe nothing to
   anyone's recollection. Where they disagree with the ledger, the algebra is the
   witness.
3. **Did the plan quietly outrank the design?** `DEC-182` moved the governance
   amendment from reconcile into `PHASE-07`. That is exactly the shape `EX-11`
   exists to catch. The test is not whether the move was right — it was ruled —
   but whether the design now *says* what happened.
4. **Where is the evidence thinnest?** A slice whose every VT passes can still
   rest on a control that was never run. `PHASE-01`'s `C2` is the known case;
   look for its siblings — tasks that structurally could not stage a red, and
   what stood in for one.
5. **What will nobody carry?** Items with no owning entity after close —
   contested premises, one-way doors, sole guards — are the ones that vanish. A
   note in a slice's `notes.md` is not an owner.

**Out of scope.** The landing point itself (`DEC-182`, settled), and the
`VH-1` acceptance (given). Neither is reopened here.

## Synthesis

### The closure story

`SL-249` set out to wire a facet write seam that already half-existed, and it
landed: eight phases, 29 `VT` criteria all passing, `doctrine check gate` exit 0,
and a governance amendment (`REV-050`) that moved the record-kind set from four
to seven across `SPEC-019` and `PRD-010` in both storage tiers. The slice is
closeable. Sixteen findings, none a blocker, all terminal.

What makes this audit worth reading is not the verdict but where the evidence
was thin, and the audit's own starting position made that harder rather than
easier. `PHASE-07`'s `T10` wrote a reconcile ledger into `notes.md` *before* this
audit opened, because `EX-11` required the phase to hand its departure forward
rather than leave reconcile to grep. That is good discipline and it is also a
trap: an audit that transcribes the implementer's ledger has verified nothing.
So the lines of attack were chosen to test the ledger, and the two things worth
reporting are what the test found.

**The ledger's count held.** It asserted fourteen owed items against a brief that
said twelve, and instructed reconcile to reconcile to fourteen. Re-derived
independently — six `design.md` edits, five scope/process notes, three backlog
carries — it is fourteen. `R-inventory`, this slice's standing risk, predicted
the written number would be wrong; here it was right.

**The ledger's content did not hold, in two places, and both are the shape the
ledger itself warned about.** First, `slice conformance`'s **undelivered** cell —
two `design-target` selectors naming homes the implementation did not use
(`src/catalog/scan.rs`, `src/design_run/admission.rs`) — is not carried anywhere
in the ledger. It discusses the undeclared cell at length and is silent on its
sibling. Second, the scope item that *is* carried says the undeclared cell holds
three files; it holds **eleven** source and test paths. So `R-inventory` fired a
sixth time, inside the very artefact written to hand the other five forward, and
it took the mechanical algebra rather than a careful reader to see it. That is
the single most useful thing this audit produced, and it is an argument for
running `slice conformance` early rather than treating it as a formality: it owes
nothing to anyone's recollection.

A third structural point, from `PHASE-07` itself: the phase shipped a canary
against stale record-kind enumerations and the same amendment left a stale
*facet-enum* enumeration the canary is constitutionally unable to see (`ISS-332`,
found by `VA-1`, fixed in place). The wrong inference is "widen the canary"; the
right one is that a `VA` criterion is not redundant with a `VT` criterion over the
same prose artefact. Both lessons now live in the memory corpus rather than in
this slice's prose, because they outlive the slice.

### Standing risks

- **`VT-3` is `EX-5`'s sole guard**, and its blast radius was *measured* at
  exactly one test rather than assumed. Waive or delete it and the acceptance
  digest silently stops covering the facet, with nothing else in the suite
  noticing. Before this audit the item had no owner at all — it sat in `notes.md`
  § *Open* and in none of the ledger's three carried groups. `F-13` gives it one.
- **`CHR-060`'s premise is contested and `F-7` is its only carrier.** The chore
  says rename `facet_write::FacetField` → `FacetValue`; this slice found that
  `facet_write`'s type is key + value, so it really *is* a field, and an unkeyed
  value type has the better claim to the name. Nobody reopens `CHR-060` until
  pickup. If reconcile writes only the type name and drops the argument, whoever
  picks it up executes a rename whose premise may be wrong.
- **`settle` is a one-way door per record.** `waived` and `invalidated` sit in
  both a settleable set and `WITHDRAWN_STATUSES`, so a settled `CON` or `ASM` can
  be neither re-settled nor settled to its sibling state. Intended, and it falls
  out of two correct rules meeting — which is exactly why no single rule's author
  had cause to write it down.
- **`R-inventory` is a practice problem, not a slice problem.** Six firings.
  Carried out of the slice by
  `mem.pattern.verification.re-derive-every-inventory-at-use`.

### Tradeoffs consciously accepted

- **`PHASE-01`'s `C2` control does not exist and cannot be reconstructed**
  (`F-9`, the only `tolerated`). The `EN-2` extraction's behaviour-preservation
  rests on inspection — one call site, an identical format string — not on a
  control that would have gone red. The control had to run *before* the
  extraction; the worker that would have run it died before writing Findings, so
  the fact is unrecoverable rather than merely unrecorded. Writing the test now
  would prove today's behaviour equals today's behaviour. Tolerated on two
  grounds: the equivalence argument is narrow and inspectable, and the systemic
  cause was fixed mid-slice — `LOOP.md` now requires a commit and a Findings line
  per task, which is why no later phase carries this shape.
- **Three tasks shipped without a stageable red** (`F-10`), each compensated by a
  positive control that was actually run. The shape is structural, not sloppy:
  under `warnings = "deny"` with `-D dead-code`, a phase whose enabling task must
  land before its test task cannot observe a red by the ordinary rhythm.
- **`PHASE-04` boxed `D4`'s payload** (`F-11`) because
  `clippy::large_enum_variant` under a zero-warnings gate left no choice.
  Substance intact, spelling different.
- **The design's literal `RawValue` spelling was unbuildable** (`F-7`) — it would
  hard-fail the layering gate — so `PHASE-06` shipped `WireFacetValue`. A
  departure taken because the alternative did not compile, which is the best kind
  of reason to have one.

### Arithmetic, stated rather than implied

Sixteen findings. Fourteen map to the ledger's fourteen owed items (the three
backlog carries collapse into one finding, `F-15`) once `F-3` **supersedes**
rather than restates the ledger's scope item; two are new (`F-2`, the undelivered
cell; `F-13`, `VT-3`'s ownerless singularity). Thirteen require a reconcile write
— twelve `verified` delegations plus the one `tolerated` residual, which is
stated rather than fixed. Two require nothing (`F-14`, `F-16`): both are already
carried durably by the memory corpus, and a slice's `notes.md` is not an owner
for a cross-slice pattern.

## Reconciliation Brief

**Read `F-3` before the scope items below** — it supersedes `notes.md`'s
"one file to three" wording rather than sitting beside it. Writing the three
would re-ship the defect this audit found.

### Governance/spec (REV) — *empty, and that is the finding*

No `REV` is owed. `SL-249`'s governance amendment already landed as `REV-050`
(`done` · `approved`) inside `PHASE-07`, per `DEC-182`. That is exactly why `F-1`
exists: the design still describes a reconcile-time landing that has already
happened. Do not mint a `REV` for anything below — a slice's `design.md` is
**not** a legal `revises` target (`ADR-013`: `{SPEC, PRD, REQ, ADR, POL, STD}`,
verified against `revision change add --help`, "Existing-target ops: the live
peer FK"; a design is not a peer entity). Every item below is a direct edit.

### Per-slice (direct edit) — `design.md` prose

- **`F-1` · `design.md` §3, §5.3, §6, §7, §10** — the amendment landed in
  `PHASE-07`, not at reconcile. Seven sites: l.**132** (§3, `ADR-013`), l.**731**
  (§5.3), l.**1103** and l.**1127–1128** (§6), l.**1245** (§7, `D8a`), l.**1600**
  and l.**1609** (§10). Correct the design; do not let the plan stand as the
  higher authority. `DEC-182` itself is settled and not reopened.
- **`F-4` · `design.md` §5.1** — `FacetField` → `FacetFieldRow`. The `ISS-329`
  ruling (option 1 + `CHR-060` for option 2) is settled; only the name is stale.
- **`F-5` · `design.md` §5.5** — narrow `I10`'s quantifier to what the generated
  matrix establishes (some submission at each kind, not every submission), per
  `DEC-183`. Code half stays with `ISS-327` / `ISS-328`.
- **`F-6` · `design.md` §5.2** — state the derivation as *status-seeded ∩ facet
  row*. Taken literally over `facet_fields` alone it yields five settleable
  states, not four. This **aligns §5.2 with `DEC-178`**, which already says
  "an exact correspondence" — it does not amend the decision.
- **`F-7` · `design.md` §5.2 — two halves, both owed.** (a) Record
  `WireFacetValue` and the layering constraint that forced it (`RawValue` lives
  in `knowledge`, command tier; `design_run` is `leaf, out=0`,
  `layering.toml:31`). (b) Record `CHR-060`'s **contested premise** — this brief
  item is its only carrier. Writing (a) alone discharges half the item.
- **`F-8` · `design.md` §10** — strike press item 2. It attributes to `DEC-178`
  an argument `DEC-178` does not make; the ordering argument was the design's own
  drafted `D6`. Verified against `knowledge inspect DEC-178`. `settle`'s
  justification does not depend on it.
- **`F-12` · `design.md`, `settle`'s contract** — state that `settle` is a
  one-way door per record.
- **`F-13` · `design.md`, sited where a `VT-3` waiver would be considered** —
  `VT-3` is `EX-5`'s sole guard; blast radius measured at exactly one test.

### Per-slice (direct edit) — selector registry, `slice-249.toml`

Conformance reads the **registry**, not the prose. A prose-only fix leaves the
cell red. Name the verb; cite §6 as its mirror.

- **`F-2`** — `doctrine slice selector rm 249 src/catalog/scan.rs
  src/design_run/admission.rs`. Both were candidate homes the implementation did
  not use: the tripwire landed in `src/doctor_checks.rs` (conformant, +257), the
  admission refusal in `Batch::validate` (`src/design_run/submission.rs:1054`).
  Mirror in `design.md` §6.
- **`F-3`** — `doctrine slice selector add 249 …` for the paths judged in scope:
  `src/commands/facet.rs` (the `KeyPosture` call site, `1683a5703`),
  `src/commands/doctor.rs` and `src/finding.rs` (check #12's registry and its
  `Finding` category, `dfd51354d`), `src/commands/guard.rs` (CLI-variant
  classification), and `src/input.rs` / `src/memory.rs` / `tests/e2e_mcp_server.rs`
  (`PHASE-08` `T1`'s shared body-flag helpers, `7a4e5bd07`, net −45 in
  `memory.rs`). Paths judged genuinely out of scope — `LOOP.md`, the
  `tests/fixtures/governance_kind_coverage/**` fixtures — stay undeclared with a
  sentence saying why. `src/design_run/*` already rides the `scope-relevant`
  selector. Mirror in `design.md` §6.

### Per-slice (direct edit) — process notes

- **`F-10`** — under `warnings = "deny"` with `-D dead-code`, a phase whose
  enabling task must land before its test task cannot stage a red and owes a
  positive control instead. Three instances this slice (`PHASE-08` T4/T6,
  `PHASE-06` `EX-4`'s substituted recipe, `PHASE-03` T3(b)); `PHASE-05`'s
  permanently-open control is the standing example of what happens unstated.
- **`F-11`** — one line: `D4`'s payload ships as `Option<Box<KnowledgeFacetEdit>>`,
  forced by `clippy::large_enum_variant`. Substance intact.
- **`F-9` (tolerated — state, do not fix)** — `PHASE-01`'s `C2` control does not
  exist and cannot be reconstructed. Record the residual and its two mitigating
  facts; do not write a test that would prove nothing.

### Carried to close (backlog, not a reconcile write)

- **`F-15`** — mint `IMP-403` leads 3–5. `CHR-056` stays open (not a blocker).
  `CHR-060` stays open **and inherits `F-7`'s contested premise** — whoever picks
  it up must read that argument first.

### No write owed

- **`F-14`** (`R-inventory`, six firings) and **`F-16`** (the canary's structural
  blindness to `ISS-332`) are `aligned`. Both are carried by the memory corpus
  (`mem.pattern.verification.re-derive-every-inventory-at-use`,
  `mem.pattern.verification.guard-blind-to-its-own-residue`, `05e0de495`).
  Explicitly **not** actioned: widening the canary to cover facet-enum lists —
  that chases one adjacent shape and leaves the class open.

## Reconciliation Outcome

Fourteen brief items, thirteen requiring a write. All written. **No `REV` was
minted** — the brief's governance/spec section is empty by construction:
`SL-249`'s amendment already landed as `REV-050` (`done` · `approved`) inside
`PHASE-07` per `DEC-182`, and a slice's `design.md` is not a legal `revises`
target (`ADR-013`). Every item below was a direct edit.

### Selector registry — the load-bearing surface (`slice-249.toml`)

Conformance reads the registry, not the prose, so these are the writes that
actually move the cells.

- **`F-2`** — `doctrine slice selector rm 249 src/catalog/scan.rs
  src/design_run/admission.rs`. Both were candidate homes the implementation did
  not use. The **shared `design-target` note** was rewritten in the same pass:
  it still enumerated `admission.rs` as the refusal's home and
  `catalog/scan.rs` as the tripwire's, so removing the rows while leaving the
  note would have left the registry asserting the same staleness one field
  over.
- **`F-3`** — `doctrine slice selector add 249 … --intent design-target` for all
  **eleven** paths the audit re-derived, superseding the `PHASE-07` ledger's
  "one file to three": `src/commands/facet.rs`, `src/commands/doctor.rs`,
  `src/commands/guard.rs`, `src/finding.rs`, `src/input.rs`, `src/memory.rs`,
  `src/design_run/ids.rs`, `src/design_run/run.rs`, `src/design_run/tests.rs`,
  `tests/e2e_mcp_server.rs`, `tests/governance_kind_coverage.rs`. Each carries
  its reason in the batch note.
- **Cells before → after:** `undelivered (2) → (0)`; `conformant (7) → (18)`;
  `undeclared (91) → (80)`, the residue being `.doctrine/**` authored entities
  plus `LOOP.md` and the five canary fixture files, which the brief judged out
  of scope.

> **A correction to the brief's mechanism, found while making the edit.** The
> brief recorded `src/design_run/*` as "already riding the `scope-relevant`
> selector" and therefore needing nothing. It does ride it, and that does **not**
> discharge the undeclared cell: `conformance::compute` is handed the
> `design-target` selectors alone (`src/slice.rs`'s conformance shell,
> `src/conformance.rs`), which is why the design-run trio sat in the cell
> despite `src/design_run/**` being declared — and why `undeclared_detail`'s own
> remediation line hardcodes `--intent design-target`. Adding the eleven as
> `scope-relevant` would have left the cell red, which is the brief's own
> warning one level deeper. All eleven were declared `design-target`.
> Recorded here rather than raised as a finding: it is a mechanism fact
> validated while locating the edit point, not new issue discovery.

### Direct edits — `design.md`

- **`F-1`** (seven sites, all seven written) — the amendment landed in
  `PHASE-07`, not at reconcile. § 3's `ADR-013` bullet; § 5.3's `D8a` carry;
  § 6's *"Open, and resolving at REV authorship"* heading and lead-in (now
  *"Open at drafting, resolved at REV authorship"*, with `inq-7` and `inq-9`
  each carrying their `REV-050` disposition); § 6's `ADR-013`-apply-path
  unknown (answered: `apply` auto-lands `status` rows only, `modify` rows
  surface for manual landing — and `D8a` did not ride the REV at all); § 7's
  `D8a`; § 10 press items 3 and 5. `DEC-182` is **not** reopened — only the
  design's account of it is corrected.
- **`F-4`** — § 5.1: the declaration struct is `FacetFieldRow`, with the
  `ISS-329` ruling and the `Row`-over-`Spec`/`Decl` reasoning recorded. Three
  spellings corrected (struct, `facet_fields` signature, § 5.2's `FacetEdit`).
- **`F-5`** — § 5.5: `I10`'s quantifier narrowed per `DEC-183` — refused at that
  kind, **or some** submission at that kind makes it effectful. The non-vacuity
  argument (three positive controls, four state-inert cells named in the test
  module) is kept, since the narrowing is what makes it load-bearing.
- **`F-6`** — § 5.2: the derivation stated as **status-seeded ∩ facet row**, with
  `decided_by`/`decided_on` named as the fifth state a literal facet-only read
  admits. Aligns § 5.2 with `DEC-178`'s "exact correspondence"; does not amend
  it. Verified against the shipped `derived_settleable` doc comment.
- **`F-7`** (both halves) — § 5.2: (a) the wire ships `WireFacetValue`, with the
  layering constraint that forced it (`RawValue` in `knowledge`, command tier;
  `design_run` declared `leaf`/`out=0` at `.doctrine/adr/001/layering.toml:31`).
  (b) **`CHR-060`'s contested premise**, in a block quote addressed to whoever
  picks it up: `facet_write`'s type is key + value (verified —
  `FacetField::Str { key, value }` / `Arr { key, values }`), so it really *is* a
  field and an unkeyed value type has the better claim to `FacetValue`. This
  brief item was its only carrier.
- **`F-8`** — § 10 press item 2 struck, with the misattribution to `DEC-178`
  stated and the two arguments that keep `settle` standing on its own recorded.
  Struck in place rather than deleted, so items 3–7 keep their numbers and the
  strike is legible.
- **`F-11`** — § 5.2, one note: the kind-dispatched payload ships as
  `Option<Box<KnowledgeFacetEdit>>`, forced by `clippy::large_enum_variant`
  under a zero-warnings gate. *Sited at the CLI surface rather than at `D4`: the
  finding cites `D4`, but this design's `D4` is "the wire's prose key is
  `body`" and `KnowledgeFacetEdit` appears nowhere in `design.md`. The fact is
  recorded where the reader meets the verb; no id is owed either way.*
- **`F-12`** — § 5.2, new subsection *"`settle` is a one-way door per record"*.
  Stated precisely rather than broadly: the door is one-way for `invalidated`
  (`ASM`) and `waived` (`CON`), which sit in both the settleable set and
  `WITHDRAWN_STATUSES` while `settle` refuses a withdrawn record. `validated`
  and `answered` are not withdrawn, so `ASM` may still travel
  `validated → invalidated` — into the same terminal place. With the
  state-to-itself refusal every kind's settle graph is acyclic and at most one
  step deep. The escape is `knowledge status` + `knowledge edit <kind>`
  (verified: `set_record_status` validates the token and carries no withdrawn
  guard), which is `I4`, not an oversight.
- **`F-13`** — § 9, new subsection *"Sole guards — read this before waiving a
  `VT`"*. `VT-3` is `EX-5`'s sole guard; blast radius **measured** at exactly one
  test (`skip_serializing_if` → `skip_serializing`). Sited in § 9 because that
  is the section read when deciding whether a criterion still earns its test —
  `plan.toml`'s criteria are immutable-append and not a reconcile surface.
- **`F-10`** — § 9, new subsection *"When a red cannot be staged"*. The
  structural shape (under `warnings = "deny"` with `-D dead-code`, an enabling
  task landing before its test task cannot stage a red and owes a positive
  control), its three instances, and `PHASE-05`'s permanently-open control as
  the standing example of leaving it unstated.
- **`F-9`** (`tolerated` — stated, not fixed) — § 9, new subsection *"The
  evidence this slice did not get"*. `PHASE-01`'s `EN-2` extraction rests on
  inspection; the control is unrecoverable, not merely unrecorded; the two
  mitigating facts (narrow inspectable equivalence; `LOOP.md` fixed mid-slice)
  are recorded as grounds, and the section says plainly that every `VT` passing
  does not make the gap smaller. **No test was written** — one now would prove
  today's behaviour equals today's behaviour.

### Direct edits — `slice-249.md`

Not in the brief, and owed by the same two findings: § *Affected surface* is the
scope card's own mirror of the registry and it carried `F-2`'s stale claim
verbatim.

- **`F-2`** — the `src/design_run/admission.rs` bullet ("where the correspondence
  refusal belongs") struck and corrected to `Batch::validate` in
  `src/design_run/submission.rs`.
- **`F-3`** — a reconcile note recording that eleven paths the list never
  anticipated were touched, pointing at `design.md` § 6 for the enumeration.

### Carried to close, not written here

- **`F-15`** — `IMP-403` leads 3–5 need minting; `CHR-056` and `CHR-060` stay
  open. `CHR-060` inherits `F-7`'s contested premise, now durably carried in
  `design.md` § 5.2 — the close checklist should point at it, since whoever
  picks up `CHR-060` must read it before acting on the rename.

### No write owed

- **`F-14`**, **`F-16`** — `aligned`, carried by the memory corpus
  (`05e0de495`). Nothing written; the closing story names them.

### Escalation

None. No brief item required a design-model change, and no item named a surface
reconcile does not write (no `plan.toml` criterion edit was asked for or made).

Reconcile pass complete — handoff to `/close`.
