# IMP-422: Rename row 9's test to match its mechanism

`a_write_through_every_readable_mount_fails` claims more than the test does, and
the gap is deliberate rather than accidental.

## The gap

A conformance `Row` carries one `ArmShape`, so both arms run **one** payload.
Under the control arm the declared readable mounts are read-write — and on a real
host those are the operator's own `/nix` and `/bin`. A payload that *literally*
wrote through every readable mount would write **outside the fixture**. The first
spike did exactly that, leaving `/nix/va2-write` and `/bin/va2-write` on the
operator's filesystem where nothing reclaims them; they were swept by hand
(`SL-248` `notes.md` item 109, phase sheet `F-11`).

What ships instead is correct: the declared mounts are derived per entry the way
row 4 derives them and read with `[ -w ]` (`access(2)`/`W_OK`, which reports
`EROFS` for a read-only mount whatever the caller's identity, so it separates the
two attachments without touching them), plus **one** real write into this run's
own source export — `DEC-157`'s channel, beneath the fixture root.

So the mechanism is right and the name is wrong.

## Why `SL-248` kept the name

`PHASE-10` `VT-2` names this test, which makes the name a **verification floor**.
Renaming the test alone breaks that link. `SL-248` corrected the *design's*
wording to the mechanism and stated the discrepancy at the top of the test's own
doc comment, but left the name standing.

## What this item must do

Rename the test **and** carry the `VT-2` mandate that pins it, together, in one
change. Criteria ids are immutable and edits append rather than renumber, so this
needs the mandate handled deliberately — it is not a `sed`.

Suggested direction: name the property the test actually establishes — the
declared readable mounts are read-only *per entry*, established by a non-mutating
read, with the single real write proving the source export's channel is the
writable one.

Originates from `SL-248` `RV-352` reconcile, `notes.md` items 118 / 109
(phase sheet `PHASE-10` `F-20`). Owner's call at reconcile: correct the design
now, rename later, "so it doesn't lie".
