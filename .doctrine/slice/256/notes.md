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
fresh-as-of: 2026-08-16 · design (run `dr-01a0088b`, rev 25, stage `drafting`) · pending

### Produced

- `DEC-237`…`DEC-241` — minted through the run's own checkpoint dispositions (`cp-1`…`cp-6` over `inq-1`…`inq-6`), so each is bound
  to the question it answers.
- `REQ-478` (`FR-009` under SPEC-029) — the durable statement `DEC-240` rules
  this slice owes. Authored `pending`; its coverage cell is deferred to when the
  e2e checks exist.
- `ISS-367` — sequenced `after SL-256`.
- `research/research.md` + `raw/` — two-thread round, with a verification pass
  appended.

### Learned

- `DEC-238` carries the general statement (a derived row can only report changes
  in the *key* of the set it differences). `ISS-367` is its second instance.
- Candidates for `/record-memory` at close, not yet written: the key-vs-content
  rule above, and that `pi-scout` line anchors drift onto the doc comment or
  attribute above an item while its content stays accurate (observed across
  three cites this round — see `research.md` § Verification pass errata).

### Open

- **`graph-reviewed` is invalidated and must be re-attested by the user before
  `drafting` → `reviewing`.** Declaring `inq-6` mid-drafting moved the inquiry
  map, which invalidated both it and `blocking-set-declared`; the agent half has
  been redeclared over all six nodes, the user half has not. `governance-confirmed`
  and `sufficiency-accepted` are unaffected and still current. The lesson is
  cheap and worth keeping: adding a node after the exploring gate costs two acts
  to restore, one of them the user's.
- **`SL-251`'s last phase is in flight** and will likely land in a worktree for
  audit before this design finishes. That is better than the posture the scope
  assumes: `R4` and the Non-Goals argue disjointness from a file list, and we
  will instead be able to check against the landed capsule. Re-read `R4` and the
  `SL-251` coordination note at that point rather than before.
- Sections still to draft: the vocabulary (`ActRecorded` plus the readable/
  emittable split), the emit seam, and verification. Both of the first two want
  diagrams per the drafting obligation. `sec-1` is declared and revised;
  `doctrine design materialise` has **not** been run, so `design.md` does not
  yet exist on disk.
- `QUE-219` — not this slice's to settle, but `DEC-239` now bears on it and
  says how. Relation carries the descriptor.
- Coverage cell for `REQ-478` — deferred by design, not forgotten. Recipe and
  criterion mapping are in the slice's Verification & Closure Intent.
