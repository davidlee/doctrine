# DEC-213: Re-scope SL-254 to the arm collapse alone

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->


## Correction at reconcile (2026-08-14, `RV-356` `F-3`)

This record's second consequence says `DEC-204`'s hazard dissolves because "with
the incumbent import transport retained `classify_import` survives untouched and
**keeps the belt**". It keeps the **two hard-coded floors** (`.doctrine/**`,
`.claude/**`), which is what `INV-2` claims and what remains true. It does not keep
`[dispatch] worker-forbidden-writes`: `classify_import` never read that key, so the
matcher lost its only production reader (`worker_commit`) at `PHASE-06` and nothing
replaced it.

The narrowing decision itself is unaffected — the floors were the load-bearing half
and they are intact. What is affected is the claim that *nothing* was left
unenforced. Supplying an enforcing reader is `SL-255`'s (`IDE-051`); the full
correction is on `DEC-204`.
