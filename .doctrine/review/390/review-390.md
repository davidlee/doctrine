# Review RV-390 — reconciliation of SL-264

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of `SL-264` against `design.md` sec-6 (`VT-1`..`VT-7`, `VA`) and
`plan.toml`, after `RV-389`'s code review (18 terminal). Surface reviewed: the
primary tree on `edge` (solo phases, no dispatch candidate), HEAD `83acf8da2`.

Lines of attack:

- **Registry truth.** `slice conformance` complete and attributable — every phase
  row holding only the phase's own patch; the post-phase `RV-389` repairs inside
  the declared selectors.
- **Criteria discharged as written.** Each `VT` has a test that says what the
  criterion says; the qualified discharges disclosed in `notes.md` are ruled on.
- **The `VA` substitution** (`RV-389` `F-11`): rule, or produce the real-run
  evidence.
- **Invalidation still fires** (`R1`): the narrowing loosens only what the design
  intends; a new blocker still reaches the user.
- **Governance mirrors.** The decisions the design amended (`DEC-121`, `DEC-126`,
  `DEC-300`, `DEC-301`) say what the code does; closure obligations are owned.
- `doctrine check gate` green.


## Synthesis

`SL-264` does what its design says. `doctrine check gate` is green end to end
(fmt, clippy at zero warnings, eslint, build, validate, and 123 test-result lines, none failing); the
`ISS-483` reserve red the phase notes carried no longer reproduces. Every design
criterion has a test that states it, and the one criterion that asked for a real
run now has one.

**The central claim, shown on real data (`F-7`).** A scratch copy of `SL-264`'s
own locked run, whose 7 nodes are all covered by both attested acts, was driven
through the four cases that matter: a non-blocking addition re-faces nothing; a
blocking addition re-faces `graph-reviewed` and only that; moving a node added
after the act re-faces nothing (`RV-386` `F-1`'s scoping); re-wording a covered
node re-faces `sufficiency-accepted`. So the loosening (`R1`) is bounded where the
design bounds it, and a new blocker still reaches the human (`R4`). The live run
was not touched.

**The registry needed repair, not the code.** PHASE-01 had no row, and the other
four solo rows were multi-commit ranges on shared `edge` that swept in `SL-265`'s
implementation and `SL-266`'s design commits — 12 foreign source paths read as
scope creep. Re-recorded from real commits (`F-1`, `F-2`); what is left is one
interleaved `SL-266` commit's `.doctrine/` files. The review repairs sit in no
phase row, so conformance cannot see them; the audit checked them by hand (`F-3`)
and found one undeclared path (`F-4`).

**Standing risks and tradeoffs accepted.**

- Conformance is blind to post-phase review repairs (`F-3`, tolerated). With
  review-before-audit now routine, this will recur on every slice — worth a
  registry mode for it, not a per-slice workaround.
- `SL-233`'s projection-bounds sketch no longer matches the `ChangeEvent` vocabulary
  it gave rise to (`F-12`, tolerated): a closed slice's revision-pinned record is
  left as history.
- A question re-word still emits no change row of its own (`F-15` → `ISS-488`).
  This slice made re-words load-bearing, so the gap is sharper than it was, but
  the vocabulary is `SL-233`'s.
- Three criteria are discharged in a qualified form, all disclosed as they landed
  and all adequate (`F-13`).

Two small code gaps were fixed in the audit (`0841788fb`): `VT-7`'s test now
asserts the effective set, and a duplicated fixture key is gone (`F-8`, `F-9`).

## Reconciliation Brief

### Per-slice (direct edit / selector registry)

- **`F-4`, selector registry** (load-bearing):
  `doctrine slice selector add 264 install/design-prompts/delegation.md --intent design-target`.
  Mirror in the sec-5 table (next item).
- **`F-5`, selector registry**: `doctrine slice selector rm 264 src/design_run/gate.rs
  src/design_run/attestation.rs src/design_run/run.rs src/design_run/tests.rs` —
  subsumed by `src/design_run/**`; clears the false undelivered cell.
- **`F-6`, `design.md` sec-5**: add to the code-impact table, one reason each:
  `src/commands/design.rs` (the `KeyContract` `home` construction site and the
  node-declaring payload fixtures); `src/design_run/refusal.rs`
  (`BlockingJudgementMissing`, `BlockingJudgementWithdrawn`, `RetiredAct`,
  `ProposalCannotClear`); `contract_check.rs` (the legacy enum-token exception);
  `delegation.rs` and `fixture.rs` (`InquiryNode::open` call sites, the frozen
  `LEGACY_SNAPSHOT`); `render/envelope.rs` (the per-home contract render);
  `install/design-prompts/delegation.md` (the proposal-null bullet, `RV-389` `F-16`).
  Correct the closing sentence so it no longer commits the selectors to the
  shorter list.
- **`F-11`, `design.md` sec-6 `VT-6`**: repair *"and no a submitted
  `blocking-set-declared` is refused"* to *"a submitted `blocking-set-declared` is
  refused (`RetiredAct`) and the stored set is unchanged (`F-16`)"*.
- **`F-7`, `design.md` sec-6 `VA`** (optional mirror): note it was discharged by the
  audit's replay on `SL-264`'s own run (`RV-390` `F-7`), not by the PHASE-05 e2e.

### Governance / knowledge (no REV reaches a DEC)

- **`F-10`, `DEC-126`**: amend the `initial-concerns-recorded` table row from
  *"reviewed graph + declared blocking set"* to *"reviewed graph with per-node
  blocking judgements (`DEC-302`)"*, via `doctrine knowledge edit`. The appended
  amendment section stays.

### Close-time bookkeeping

- **`F-14`**: resolve `IMP-469` and `ISS-481`, citing `SL-264`. On `RFC-031` T4,
  record the outcome, the `DEC-062` reading in `DEC-301`, and the fitness measure
  from `F-7` (2 nodes and 1 edge added after sufficiency acceptance, with no void).

### Not reconcile's

- `F-15` → `ISS-488`. `F-3`, `F-12` tolerated. `F-13` aligned. `F-1`, `F-2`, `F-8`,
  `F-9` fixed in the audit.
