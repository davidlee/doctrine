# Notes SL-256: Recording an act emits a change row

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- `explore.triage`, run rev 11; refreshed at rev 25. An index into where each element lives, not a
     second copy of it — the scope doc is the authority for all five columns. -->

| element | where it lives | state |
|---|---|---|
| open questions | `slice-256.md` § Open Questions — `OQ-1`…`OQ-3` | all settled, each annotated with the DEC that settled it |
| in-run questions | design run `dr-01a0088b` — `inq-1`…`inq-6` | 6/6 resolved; `inq-6` raised mid-drafting |
| risks | `slice-256.md` § Risks & Assumptions — `R1`…`R6` | `R3` retired by research; `R6` added from the `explore.memory` retrieve |
| assumptions | same section — `A1` | discharged by `DEC-240` |
| shaping decisions | `DEC-237` (event shape + terms), `DEC-238` (emit seam), `DEC-239` (roster split), `DEC-240` (durable statement), `DEC-241` (review-disposing arm emits two rows) | all `accepted`, all bound to the node that asked |
| constraining governance | `research.md` § Thread 1 — binding: `STD-001`, `REQ-437`, `REQ-436`, `ADR-001`; checked-not-applicable: `STD-002`, `STD-003`, `POL-001`, `POL-002`, `ADR-019`, and an ADR sweep | confirmed by the canon pass; `STD-003` subsequently ruled on rather than merely noted |

The one live tension left for drafting is not a question but a sequencing fact:
`DEC-239` widened scope to retire a member whose legacy rows appear in `SL-251`'s
own design-run snapshot. Read-path only by design — the roster split exists to
keep exactly those rows parsing — but it wants re-checking at `SL-251`'s
integration rather than assuming.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-16 · design (run `dr-01a0088b`, rev 42, stage `reviewing`)

### Produced

- `DEC-237`…`DEC-241` — minted through the run's own checkpoint dispositions
  (`cp-1`…`cp-6` over `inq-1`…`inq-6`), so each is bound to the question it
  answers. `DEC-238` carries an appended correction (`RV-360` `F-1`).
- `REQ-478` (`FR-009` under SPEC-029) — authored `pending`; coverage cell
  deferred until the e2e checks exist.
- `ISS-367`, `IMP-437` — both sequenced `after SL-256`.
- `RV-360` — external adversarial pass over all four sections. **14 findings, all
  disposed.** Codex raised `F-1`…`F-7`, verified five of its own dispositions and
  contested `F-2` and `F-4`; a responder verification pass produced eight further
  defects which codex raised as `F-8`…`F-14`. `RV-359` was empty and is committed
  as-is.
- `DEC-239` carries an appended correction: the retirement's pinning mechanism
  moves from `#[serde(rename)]` to `#[serde(try_from/into)]` single-sourced
  through `as_str`. Substance unchanged; `STD-001` satisfied rather than excepted.
- `src/design_run/refusal.rs` added as an eighth selector — `ChangeEvent`'s
  `TryFrom` needs one new variant. Fence widening declared, not discovered.
- `design.md` `sec-1`…`sec-4` drafted, revised and materialised (`9fe802b8f`,
  `1e495e88c`). Scope and selectors reconciled in the same commit.
- `research/research.md` + `raw/` — two-thread round, verification pass appended.

### Learned

- `mem.fact.coverage.vt-needs-a-check-field` — recorded this session.
- Candidates for `/record-memory` at close, not yet written: `DEC-238`'s
  key-vs-content rule (a derived row can only report changes in the *key* of the
  set it differences); `pi-scout` line anchors drift onto the doc comment or
  attribute above an item while content stays accurate (`research.md`
  § Verification pass errata); adding an inquiry node after the exploring gate
  costs two acts to restore, one of them the user's; an argument from absence
  needs enumeration, not a generalisation (`RV-360` `F-7`).
- **HIGH — grep line numbers from the Bash tool are intermittently wrong.** The
  same `grep -n` returned `1081/1086/1091` early in a session and `1094/1099/1104`
  later for the same three call expressions in `src/design_run/tests.rs`; the Read
  tool agrees with the second. Multi-line `sed` ranges come back with lines elided,
  so content cannot be aligned to a range's numbers either. This corrupted a whole
  verification pass and was caught only because an external reviewer's anchors
  disagreed. Extends `mem.fact.rtk.output-filter-rewrites-identifiers` from
  identifiers to line numbers, and adds that it is **non-deterministic**. Ground
  truth is the Read tool or `awk '{print NR}'` cross-checked against it.
- An acceptance criterion can be a **relieving** clause rather than a positive
  proof obligation. `REQ-478`'s third criterion exempts retired vocabulary from
  the emission obligation; the design read it as demanding proof that emission
  cannot occur, imported a burden it could not discharge, and a reviewer correctly
  found the shortfall. Read the requirement's own rationale before restating a
  criterion in a design's words (`RV-360` `F-2`).
- `#[serde(try_from = "String", into = "String")]` is this repo's idiom for
  single-sourcing a token enum's wire spelling through one `as_str`
  (`ids.rs:131`, `attestation.rs:935`, `change_log.rs:387`), and `Refusal`'s
  `Display` doc names serde's `try_from` as the boundary it exists to cross. A
  serde attribute cannot take a `const`; this is the way round that (`RV-360`
  `F-4`).

### Open

- **Four sections outstanding review, and the run's `review_pass` is STALE** —
  every fingerprint moved again at rev 37/39/41. Policy is `human-only`, so the
  attestations and the review-pass disposition are the user's.
- `RV-360` is `await=raiser` with all 14 findings answered. `F-2` is answered by
  a **contest of the contest** — the ruling is that `REQ-478` criterion 3 never
  asked for the proof the reviewer required — so it is the one disposition most
  likely to come back.
- `IMP-437` — the emit seam is a convention, not a guarantee; needs its own
  scope because the fix reaches `fixture.rs` and `SL-251`'s `tests.rs`.
- `ISS-367` — `live_acts` blind to same-kind replacement. `sec-1`/`sec-3` state
  the boundary so the design promises no symmetry it does not deliver.
- **`SL-251` coordination: three sites, not two** — its `design.md` ¶ 422–428,
  its ledger row at 2289, and `payload_contract.rs:501`. Discharged at that
  slice's reconcile, not here.
- `QUE-219` — not this slice's to settle; `DEC-239` bears on it and the relation
  carries the descriptor.
- Coverage cell for `REQ-478` — deferred by design. The recipe was **wrong** and
  is corrected in `sec-4`; see `mem.fact.coverage.vt-needs-a-check-field`.
