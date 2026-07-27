## The shape

You add a guard: *every mutating operation refuses when observed state diverges
from what we last wrote.* Sound. Then it turns out one of those operations —
the recovery/re-adoption path — **exists precisely because state diverged**. The
guard now refuses the only verb that can clear the divergence. The system wedges
in exactly the state the guard was added to make safe.

Caught on RV-315 (SL-233) as F-20: an authored-file watermark check was written
as a universal entry rule, in front of an `adopt_authored` path whose whole
purpose is to admit foreign bytes. It regressed a finding (F-8) that was already
terminal.

## The fix, and the part that is easy to get wrong

"Re-adopt bypasses the guard" is **not** the fix — that lets a caller launder
arbitrary foreign state by naming the escape hatch. The exception must be a
*protocol* with enumerated obligations. For the watermark case:

- the caller declares the exact current fingerprint and it must match observed bytes;
- the complete structural map validates before anything is admitted;
- all affected content-bound evidence is invalidated, with **no clearance
  inherited across the crossing**;
- the baseline moves **only** after the candidate validates in full;
- an invalid or stale attempt moves neither clearance nor baseline.

Note the fingerprint is a *concurrency token, not proof of authorship*. It stops
a lost update; it does not establish who wrote the bytes or that anyone reviewed
them. Don't let the guard's presence imply an authority claim it cannot make.

## Second-order trap: the borrowed guarantee

The same review caught a related overclaim. `src/review.rs::with_turn` runs an
entry-CAS + pre-write-CAS pair and can honestly promise "nothing was written" on
refusal — because it holds a writer lock and hashes the very file it atomically
replaces. Borrowing that *shape* for a cross-file, multi-effect operation is
fine; borrowing the *promise* is not. If effects are ordered before the final
write (a journal, a reserved id, an authored record) and are never rolled back,
the honest guarantee is **"the run does not advance"**, not "nothing was
written".

And with no lock over a human editor, a pre-write comparison narrows the race
but cannot close it: an edit landing between the final comparison and the rename
is caught at the *next* entry. State that as **delayed detection, never silent
acceptance** rather than claiming same-invocation safety.

## Cue

When adding a guard, enumerate the operations it fronts and ask of each: *does
this one exist in order to cross the condition I am now refusing?* If yes, it
needs a specified admission protocol before the guard ships.
