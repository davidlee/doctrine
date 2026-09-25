# A state-dir scan must admit only canonical names and check the file's own id

Scanning a directory of per-entity state (`.doctrine/state/slice/NNN/…`) by
parsing each child name as a number has two traps. RV-392 caught both in
`design tree`'s no-slice run scan (SL-266 PHASE-03):

1. **Alias names.** `0233` parses to 233 too. If the scan then reconstructs the
   canonical path from the number, one entity is counted twice (F-2). Admit a
   child only when the single-sourced path helper names *that* directory:
   `design_snapshot_path(root, n).parent() == Some(child)`. That keeps the
   `{:03}` spelling in `state.rs` (STD-001).
2. **Misfiled content.** The directory name and the file's embedded id can
   disagree (for example, a copied snapshot). Judging by the directory while
   rendering the embedded id presents one entity under another's status (F-1).
   Check `parsed.id == dir_id` and skip with the mismatch as the cause
   (STD-003), rather than picking one.

The same holds for any read keyed by a path that also carries identity inside
the file.
