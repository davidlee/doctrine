# ISS-446: lifecycle_findings reports a clean corpus on a degraded read

`backlog::lifecycle_findings` (`src/backlog.rs:2720`) swallows two failed reads and
returns a clean bill of health from each:

- `let Ok(items) = read_all(root) else { return Vec::new() }` — an unreadable
  backlog corpus yields **no findings**, not an error.
- `crate::meta::read_metas(&root.join(".doctrine/slice"), …).unwrap_or_default()` —
  an unreadable slice directory yields an empty status map, so every slice a
  finding would consult reads as status-less.

Both are STD-003 (*no silent skip — a degraded read is disclosed*) and the failure
mode is the worst shape of it: this feeds `doctor`, so the degraded read renders as
*passing*, which is exactly the honest-record defect SL-238 was raised over one
surface over.

**Provenance.** Found by SL-238 PHASE-08's `VA-1` sweep, as a residual of the
`unwrap_or_default()` census. Not fixed there: PHASE-08 owns the dep/seq routing
seam, and the four laundered reads that slice enumerated (two in
`commands/dep_seq.rs`, two in `backlog.rs`'s duplicate prune leg) are all gone. This
is a *different* surface that the same grep surfaces, and folding it in would have
widened a phase mid-flight.

Note for whoever takes it: SL-238's own `notes.md` records the census as three
residual `unwrap_or_default()` calls that are "Option-chain defaults over no read at
all". That reading was right for `:2411` and `:3565` and wrong for this one — it is
over a read.
