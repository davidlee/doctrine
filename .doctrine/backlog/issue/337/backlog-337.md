# ISS-337: Setup decoys stay inheritable across the arm

One member of the descriptor-isolation class is deliberately left open.

## The hazard

`trusted_side_setup` cannot hold the descriptor window across the arm's fork —
`fork_within_the_descriptor_window` takes the same lock (`F-16`), so holding it
would deadlock. Its decoy descriptor set is therefore **inheritable for the
arm's duration**.

## Why it is harmless today, and why that is not enough

It is harmless only because every descriptor-delta row currently runs **in a
child**. That is a property of today's row set, not an invariant of the seam. A
future in-process descriptor-delta row re-opens the hazard with nothing to stop
it.

The rest of the class was closed properly: the shipped repair is mutual exclusion
at the source — one seam returning the guard together with the decoy set, so a
caller cannot bind the descriptors without the lifetime that protects them.
`trusted_side_setup` is the one member that repair cannot reach.

Recorded here rather than fixed because the fix is a lock-structure change, not a
line, and `SL-248` had no row that needed it.

## References

`SL-248` `PHASE-10` phase-sheet finding `F-30` · `notes.md` § *Owed* item 127 ·
related findings `F-16`, `F-19`, `F-29` · `RV-352`
