# Review RV-364 — reconciliation of SL-256

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Surface reviewed.** The `edge` primary worktree at `c0c69f634`, not a dispatch
candidate branch — `SL-256` ran solo (`boundaries.toml`: three `provenance =
"solo"` rows), so there is no `review/*` evidence ref and no candidate
interaction branch to admit.

**Facet.** `reconciliation` — does the landed tree match `design.md` and the five
decisions the design run locked (`DEC-237`…`DEC-241`), and is every gap
dispositioned before the slice may advance?

### Lines of attack

1. **The vocabulary and the roster split (`sec-2`, `DEC-239`).** `READABLE` (23)
   / `EMITTABLE` (22) membership; the subset relation proved at compile time
   rather than asserted at runtime; `as_str` as the token's *only* source; and
   `LegacyAcceptanceAttested` renamed without the wire token, the rendered token
   or the stored payload shape moving. The invariant: **a snapshot written by an
   older binary must still parse**, because `ChangeEvent` deserialises strictly
   and one unrecognised `event` costs the whole snapshot (`R1`, `ISS-315`).
2. **The emit seam (`sec-3`, `DEC-238`, `DEC-241`).** One production route by
   which an act reaches the snapshot; `Pending` mandatory, never
   `Option<Pending>`; `admit_against` deleted rather than merely unused;
   admission strictly before storage; and the contractual `[ActRecorded,
   ReviewDisposed]` row order, which runs *opposite* to construction order and is
   therefore the trap an implementor falls into.
3. **Does the defect actually close?** `ISS-355` reported one of two silent
   paths. The invariant: every one of the four recording paths — bare
   `agent_declaration`, `checkpoint_act` without a disposition, a disposing act,
   and the run-level `acceptance` field — renders a row naming the act's subject
   and kind.
4. **Bounds and budgets the change consumes.** `DESIGN_EVENT_NAME_BYTES` (`R2`,
   which is how `REQ-437` is discharged) and `WIDEST_PAYLOAD_EVENT` (`R5`, a
   hardcoded exemplar whose compile-time assert cannot catch it moving). Probed
   past the design's own register for any *other* fixed budget this slice spends
   more of.
5. **`REQ-478`'s discharge.** Three acceptance criteria, a coverage cell bound to
   a runnable positive control, and the requirement's own lifecycle status.
6. **The conformance algebra.** Every `undeclared` and `undelivered` cell ruled
   on — scope creep, a stale design, or declared-and-explained — with the
   boundary registry itself checked for completeness and for foreign commits
   riding a phase range.
7. **Standing corpus state the notes deferred to this audit.** `memory validate`
   exiting 1, and `ISS-315` still live on the read path this slice touched.

**What is *not* probed, and why.** The absorption mechanism (`ISS-333` /
`ISS-346`, gated on `QUE-219`) and the seam's non-bypassability (`IMP-437`) are
both explicit Non-Goals with owners. `SL-251`'s three coordination sites are
discharged at *that* slice's reconcile, not here.

## Synthesis

### The closure story

`ISS-355` reported that a successful `agent_declaration` apply printed nothing
that distinguished *landed* from *silently discarded*. The slice's own scoping
found that report was one of three: `record_act` emitted a row only when the act
carried a review disposition, and the run-level `acceptance` path pushed a row of
its own vocabulary — so the change log already treated one recorded act as a
material change and two others as invisible, with no stated reason for the split.
That asymmetry, not the reported instance, is what the slice set out to delete.

**It is deleted, and the evidence is behavioural rather than structural.** All
four recording paths now render a row naming the act's subject and kind, each
pinned by its own e2e check that asserts event, subject *and* ordered terms
(`tests/e2e_design_state.rs:1243`, `:1265`, `:1293`, `:1328`) — three facts per
check, because each alone was already true of the snapshot before the slice. The
row order on the disposing arm is asserted as an ordered pair rather than as set
membership, which is the one check that catches the trap `sec-3` named: the
disposition row is constructed *before* the record is admitted, so construction
order runs opposite to the contractual vector order, and an implementor who lets
the vector follow construction emits the pair backwards.

The structural half is stronger than the tests. `admit_and_record` returns
`Pending`, not `Option<Pending>`, so there is no arm on which emission is
optional; `admit_against` is deleted rather than left unused, so there is no
second admission route; and admission strictly precedes storage, so a refused
record leaves the candidate snapshot untouched. Verified by enumeration:
`ChangeEvent::ActRecorded` is constructed at exactly one site in `src/`
(`run.rs:753`), and the storage sinks are reached from exactly one production
site each (`run.rs:706-707`, inside `ActRecord::insert`).

The gate is green (`doctrine check gate`, exit 0), all eleven `VT` rows across the
three phases pass, and `REQ-478`'s coverage cell verifies against a runnable
positive control — a `--matcher-pattern` naming one test's own pass line, so the
cell fails if that test was never compiled, which is what makes it evidence rather
than a claim.

### What the roster split bought, and what it cost

`DEC-239` widened scope mid-design, by explicit human decision, to retire
`AcceptanceAttested` — and the slice paid for that widening properly rather than
admitting it sideways. The `READABLE` (23) / `EMITTABLE` (22) split is the shape
of the underlying fact: **the vocabulary a run writes is not the vocabulary it
must read**, because `ChangeEvent` deserialises strictly and one unrecognised
`event` costs the whole snapshot rather than one row. The subset relation is
proved at compile time by a `const fn is_subset` walking tokens in the module's
existing slice-recursion idiom, not asserted at runtime — which upgrades a
planned test to a proof and, as `F-16` found during the design's own adversarial
pass, incidentally gives `EMITTABLE` the `src/`-side consumer it needs to survive
the crate's `deny(unused)` meeting a compilation-unit boundary. Confirmed by
enumeration: that assert is still `EMITTABLE`'s only reader outside
`tests/e2e_design_state.rs`, so it must not later be deleted as redundant, and the
comment above it says so.

The retirement itself is the part most likely to have gone wrong and did not.
`as_str` is now the token's only source (`#[serde(try_from/into)]` replacing
`rename_all`), so the Rust identifier moved to `LegacyAcceptanceAttested` while
the wire token, the rendered token and the stored run-wide, term-free payload
shape all stayed put. Pinned at the whole-file tier in `snapshot.rs`, with two
assertions rather than one, because `DEC-239` refused two different repairs: a
bare serde alias would rename the variant but not the row, and whole-row
normalisation at deserialise would manufacture a subject and an `act` term the
writer never stored — then persist the invention as history on the next write.
The census this compatibility argument protects is live and was verified here: 9
`acceptance_attested` rows across 7 of this repo's own design runs, and 8
`evidence_invalidated` rows on `SL-244`'s.

The cost is `F-5`, and it is worth naming plainly because the ledger is where it
becomes durable: single-sourcing the token took away serde's derived error
message, which listed the accepted vocabulary. The refusal now names only the
offending token. That message fires on a live path — `doctrine design show 244`
fails on it today — and the reader who sees it is the one person for whom the
accepted list is the useful half. `IMP-445` owns the repair. Strictness itself was
preserved exactly, and tested.

### Standing risks and consciously accepted tradeoffs

- **The emit seam is a convention, not a guarantee.** `CheckpointActGroup::record`
  and `AgentDeclarationGroup::record` stay `pub(crate)`, so the storage sinks
  remain reachable; `ActRecord::insert` stops *using* the direct route without
  closing it. Restricting visibility breaks 13 call sites across `fixture.rs` and
  `tests.rs`, both outside this slice's bounds. `IMP-437` owns it, the design
  states the boundary rather than implying a guarantee it does not deliver, and
  the code comment at `run.rs:719-727` says the same thing where an implementor
  will read it. Accepted.
- **An existing observable changed, by decision.** A review-disposing act emits
  two rows where it emitted one (`DEC-241`), and the acceptance path's row changed
  vocabulary — from a run-wide term-free `acceptance_attested` to a subjected
  `act_recorded`. Both were decided explicitly rather than absorbed; `DEC-241`'s
  reasoning is that they are different claims and that suppressing the recording
  row on that one arm would carve the original asymmetry back into the very seam
  `DEC-238` unified.
- **`ISS-315` is live and reproducible, and is not this slice's regression.**
  `doctrine design show 244` fails at snapshot line 4927 on
  `integrated_review_recorded` — a token with no `READABLE` member and no alias.
  Established with a positive control rather than by inspection: the pre-slice
  binary fails identically, and that same binary parses `SL-251`. The slice's read
  path is what makes the *other* retired tokens keep working; this one predates it.
- **`ENVELOPE_CHANGE_ROWS` is consumed harder** (`F-6`). No budget is breached and
  the elision discloses `omitted`/`total`, so nothing lies — but a resuming agent
  reaches the ten-row cap after fewer applies than before.
- **Corpus drift is real and stays visible** (`F-8`). `memory validate` exits 1
  with 15 findings. None belongs to this slice; pulling 14 unrelated
  re-attestations into this audit would be scope the slice never took.

### On the method that produced this

Worth recording because the audit's own yield is the evidence for it. The design
took four independent attacks before a line of production code was written —
`RV-359` (empty), `RV-360` (16 findings, external), a prototype that ran the type
model through a compiler in a confined fork, and an internal pass over that
prototype's repairs whose two findings the external reviewer re-derived rather
than accepted. This audit then found **no defect in the landed code**: nine
findings, of which one is a knowingly-traded diagnostic with an owner, two are
rulings that the conformance and corpus signals are behaving correctly, five are
canon-versus-reality corrections in the slice's own artefacts, and one is a
governance status field nobody moved. That distribution — all the residue in the
paperwork, none in the behaviour — is what a design that has stopped yielding to
reading looks like when it lands.

The one method failure is `F-8`'s: a ruling reached by pattern-matching a scope
prefix (`src/design_run/**`) instead of enumerating. It cost nothing here because
the conclusion happened to survive enumeration, which is precisely why it is worth
writing down.

## Reconciliation Brief

### Per-slice (direct edit)

- **`slice-256.toml` selector registry — `F-4`.** `doctrine slice selector rm` the
  two `design-target` selectors `src/design_run/render/mod.rs` and
  `src/design_run/render/change_row.rs`. This is the load-bearing change: `slice
  conformance` computes from the registry, so a prose-only edit leaves the
  `undelivered` cell red. Mirror it in `design.md`'s affected-surface table, where
  both files are currently listed as fenced-but-expected-untouched — the fence did
  its job and is now discharged, not deleted for being inconvenient.
- **`design.md:593`, `:626`, `:708` — `F-2`.** `cpa-design_accepted` →
  `cpa-design-accepted` and `act=design_accepted` → `act=design-accepted`. Act
  tokens are kebab-case, from `ActKind::as_str` (`attestation.rs:116`). The plan's
  half of this erratum was already corrected at PHASE-02 (`7c8095664`); this closes
  the pair.
- **`design.md:951` — `F-3`.** `--command --test` → `--command=--test` in `sec-4`'s
  coverage-cell recipe. Clap refuses the bare form (`unexpected argument '--test'
  found`); any `--command` value beginning with `-` needs the `=` form. The
  recorded cell already stores the intended argv, so this is the copy-pasteable
  recipe only.
- **`design.md` sec-4 census — `F-7`.** "7 rows across 6 live runs" → "9 rows
  across 7 live runs", matching the verified census and `change_log.rs:180`.
- **`design.md` risk register — `F-6`.** One clause beside `R5` naming
  `ENVELOPE_CHANGE_ROWS` (`render/mod.rs:78`) as a second fixed budget a new event
  spends — consumed, not breached — and pointing at `changes()`'s `omitted` /
  `total` disclosure (`render/envelope.rs:1073-1092`) as why no repair is owed.

**Off-surface, deliberately.** No `plan.toml` edit is in this brief. PHASE-02's
`VT-1` annotation already carries the corrected mandate in place, and `EN-/EX-/VT-`
ids are immutable-append — a divergence requiring a *changed* plan criterion would
be a design escalation, not a reconcile direct edit. None arose.

### Governance/spec (REV)

- **`REQ-478` status `pending → active` — `F-1`.** SPEC-029 `FR-009`. Evidence:
  all three acceptance criteria discharged by named e2e checks
  (`e2e_design_state.rs:1243`/`:1265`/`:1293`/`:1328` for criterion 1; `:1090`
  iterating `EMITTABLE` for criterion 2; `:1119` for criterion 3), and the
  coverage cell `verified` under `doctrine coverage verify 256` against a
  positive-control matcher anchored at `c0c69f634`.

### Not reconcile's — owners already exist

Recorded so `/reconcile` does not adopt them: `IMP-445` (`F-5`, the refusal's
missing token list), `IMP-282` (`F-9`, conformance noise from slice-own process
artefacts), `IMP-437` (the emit seam's non-bypassability), `ISS-315` (the live
unparseable `SL-244` snapshot, not this slice's regression), `ISS-448` / `IMP-273`
(the unpinned toolchain), `ISS-449` (`verify-vt` claiming "keyword present" on
rows it never checked), and `SL-251`'s three coordination sites — `design.md`
¶422-428, its ledger row at 2289, and `payload_contract.rs:501` — which are
discharged at *that* slice's reconcile.
