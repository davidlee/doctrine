# IMP-156: Add --spawn-from <BACKLOG-ID> flag to doctrine slice new

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

See REQ-065 (pending).
