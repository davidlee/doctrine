# IMP-366: Re-emit outstanding delegation assignments on read

SL-233 PHASE-10 emits an exported assignment on `design apply`'s stdout at the
moment the export is taken — the same place, and for the same reason, the lock
discloses what it rests on where the lock is taken. Nothing re-emits it.

So if that stdout is lost — a scrolled terminal, a crashed session, a delegate who
never received it — the assignment has to be reconstructed by hand from the
snapshot. **No information is lost**: the delegation stores the obligation as it
stood at export, so the id, the question, the exported revision and the
proposal-only contract are all still derivable. It is a missing convenience, not a
missing fact, and it was disclosed as a bound of the phase rather than discovered
afterwards.

The natural home is the turn envelope (DEC-064: every read is a rendering of one
envelope), which is why PHASE-10 did not do it. Adding a slot means touching
PHASE-04's cardinality caps and the eviction ladder, and an outstanding assignment
is not obviously a member of the *budgeted* projection — its question text must
render whole to be self-contained, and the budgeted rendering elides prose. Two
candidate shapes, and choosing between them is the work:

1. an envelope slot with its own `ENVELOPE_*` cap, elided like any other prose —
   which makes the re-emitted assignment potentially *not* self-contained, and
   therefore a different kind of artefact from the one export emitted;
2. an unbudgeted re-emission on the existing `--full` path (`Detail::Full` already
   returns the envelope unbounded), which keeps "an assignment renders whole" true
   everywhere and adds no cap.

(2) looks right and cheap. It should be settled with the projection-bounds sketch
open, not by whoever needs it first.

Originates from SL-233 PHASE-10 (sheet decision D3).
