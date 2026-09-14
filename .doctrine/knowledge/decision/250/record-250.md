Settles inq-18. ISS-361 stays open against the residual window rather than being closed by this slice: EVD-028 ruled out its reported mechanism, EVD-029 names the surviving suspect, and neither establishes that the single witness saw this window.


## Correction — SPEC-029 already governs this (added at `explore.scope`)

Recorded before `SPEC-029` (*Design run engine*) had been read, which the
`explore.scope` runbook step then caught. The ruling stands; two-thirds of its
reasoning was reinvented rather than retrieved, and one framing was wrong.

`SPEC-029` makes the mint-before-snapshot ordering **specified behaviour**, not
a defect. Its reserve-then-journal responsibility reads: *journal the checkpoint
intent, claim a canonical id, journal the id, materialise the record, apply
status and relations, then complete the snapshot — with every post-journal
recovery resuming the exact reserved id and no failure path deleting authored
knowledge to repair a runtime error.* And its watermark responsibility states
the late-check residual directly: the re-check happens *immediately before every
write it guards, where a divergence abandons the write without advancing the run
while journalled effects remain and stay recoverable.*

So:

- **The rejected alternative was not merely unattractive, it is prohibited.**
  "Unwind record materialisation on refusal" is the failure path deleting
  authored knowledge that `SPEC-029` forbids by name. The reasoning given here —
  that it would be a second writer of the authored tier — reaches the governed
  answer independently, but the authority is the spec, not the argument.
- **The residual does not need to be newly stated; it needs to be honoured.**
  This record framed naming the late-check window as something the slice would
  add. `SPEC-029` already names it, and `DEC-100` already records the analogous
  lost-update window as consciously tolerated and disclosed rather than designed
  away. Leg 1's work is therefore *make the code match the spec*, not *decide a
  new rule*.
- **What survives unchanged is the hoisting.** A check that could have run
  before the write and did not is a genuine defect against the spec's own
  ordering, and `recheck_watermark_before_write` is specified to be late — so
  the hoist/residual split this record chose is the right one, now for a
  governed reason.

`EVD-029` should be read with this: the ordering it reports is accurate, and
its characterisation of that ordering as a hole is too strong for the part the
spec prescribes.
