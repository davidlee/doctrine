# Notes SL-256: Recording an act emits a change row

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-16 · design (run `dr-01a0088b`, rev 6, stage `exploring`) · pending

### Produced

- `DEC-237`, `DEC-238`, `DEC-239`, `DEC-240` — minted through the run's own
  checkpoint dispositions (`cp-1`…`cp-5` over `inq-1`…`inq-5`), so each is bound
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

- The inquiry map is fully resolved (5/5). What the run now needs is not a
  question but two acts it cannot perform for itself: `governance-confirmed` and
  `graph-reviewed`, both the user's, plus the agent's `blocking-set-declared`.
  Those clear `exploring` → `drafting`.
- `QUE-219` — not this slice's to settle, but `DEC-239` now bears on it and
  says how. Relation carries the descriptor.
- Coverage cell for `REQ-478` — deferred by design, not forgotten. Recipe and
  criterion mapping are in the slice's Verification & Closure Intent.
