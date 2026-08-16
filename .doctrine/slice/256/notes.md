# Notes SL-256: Recording an act emits a change row

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-16 · design (run `dr-01a0088b`, rev 5, stage `exploring`) · c136177ab

### Produced

- `DEC-237`, `DEC-238`, `DEC-239` — minted through the run's own checkpoint
  dispositions (`cp-1`…`cp-4` over `inq-1`…`inq-4`), so each is bound to the
  question it answers.
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

- `inq-5` — the one unresolved node. Does this owe a durable statement: a new
  `REQ` under SPEC-029, or a widening of `STD-003`? Live in the run.
- `QUE-219` — not this slice's to settle, but `DEC-239` now bears on it and
  says how. Relation carries the descriptor.
