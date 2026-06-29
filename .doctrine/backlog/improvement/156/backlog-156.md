# IMP-156: Add --spawn-from <BACKLOG-ID> flag to doctrine slice new

This fulfills REQ-065: "Spawn a backlog item from a record without converting it into work."

REQ-065 requires a record to be able to spawn a backlog item, recording a
`spawns` relation on the record (inbound `spawned_by` derived).

Add `--spawn-from <RECORD-ID>` to `doctrine backlog new`:

```
doctrine backlog new improvement --spawn-from DEC-001 "Audit memory staleness"
```

This would:
1. Create the backlog item as usual
2. Run `link RECORD-ID spawns IMP-NNN` to record the spawn edge

The `spawns` relation already exists in RELATION_RULES
(`src/relation.rs`): record kinds (ASM/DEC/QUE/CON) → backlog kinds (ISS/IMP/CHR/RSK/IDE).
Inbound `spawned_by` is derived.

## Follow-up — generalise to create-time provenance authoring (SL-176)

SL-176 (Finish Axis B) lands the neutral **`references(originates_from)`** role
("I was born from the target") authored at the live/born end. Once it ships, this
item should grow a sibling create-time flag so provenance is set at creation, not a
second `link` step:

- `doctrine backlog new improvement --originates-from SL-NNN "…"` →
  `link IMP-NNN references --role originates_from SL-NNN` (backlog born from a slice's
  work — IMP-207's `spawned_from` case).
- `doctrine slice new --originates-from IMP-NNN` → the slice's origin edge (generalises
  the current `scoped_from` authoring; SL→SL splits too).

`--spawn-from` (record→backlog `spawns`) and `--originates-from` (work→origin role)
are distinct edges on distinct labels — both are create-time ergonomics for the same
"don't make me run a second `link`" want. Decide at build whether they share one flag
surface or stay separate. Blocked on SL-176 landing the role.
