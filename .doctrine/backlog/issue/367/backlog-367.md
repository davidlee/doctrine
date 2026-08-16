# ISS-367: live_acts is blind to same-kind replacement, so ActInvalidated under-reports

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`live_acts` (`src/design_run/run.rs:1777`) returns
`BTreeSet<(ActKind, DesignId)>`, and `act_id` (`run.rs:671`) is
`prefix + act.as_str()` — purely kind-derived. So the tuple's two components are
the same fact twice: the id is a function of the kind.

`invalidation_rows` (`run.rs:1842`) differences that set before and after an
apply. A live act replaced by another live act of the same kind therefore
produces an **identical** tuple on both sides, an empty difference, and **no
`ActInvalidated` row**.

## Why it matters

`live_acts`' own doc claims otherwise (`run.rs:1771-1774`):

> An act carrying no covered map is `Coverage::Artefact`-bound: its own recorded
> content cannot move, so it is inert here and never dies of coverage. It still
> leaves the set by being *replaced*, which is a death worth a row —
> `CheckpointActGroup::record` retains-then-pushes by kind.

The last sentence is wider than the code delivers. Replacement removes the old
record from the group, but not from *this set*, because the surviving record
occupies the identical `(kind, id)` slot. The row the doc promises is not
emitted. So an artefact-bound act is claimed to die visibly on replacement and
in fact dies silently — the one path the doc singles out as covered is the one
that is not.

The derivation is still sound for the question it was built for: an act whose
covered map has moved leaves the set, because `live()` stops holding. Only the
replacement case is blind.

## Scope note

Found while designing `SL-256` (*Recording an act emits a change row*), which
establishes the general statement in `DEC-238`: **a derived row can only report
changes in the key of the set it differences.** `SL-256` fixes the *recorded*
half by emitting explicitly at the record seam; it does not touch the
*invalidated* half, so this asymmetry survives it — `ActRecorded` will fire on a
replacement while `ActInvalidated` still will not.

Deliberately not folded into `SL-256`: repairing it means either widening the
liveness tuple with something that changes on replacement (a fingerprint catches
unequal replacement only) or moving invalidation-on-replacement to the same
explicit seam — and the latter partly undoes the cannot-forget property that
made derivation right for content movement. That is a real design question, not
a mechanical fix.

## Candidate directions

1. Emit the `ActInvalidated` row for a replacement explicitly, from the same
   `admit_and_record` seam `SL-256` introduces — the replaced record is in hand
   at exactly that point.
2. Widen the liveness tuple with the record's content fingerprint. Catches
   unequal replacement; still silent on an identical re-record.
3. Decide the doc is wrong rather than the code, and narrow the claim at
   `run.rs:1771-1774` to what derivation can actually deliver.

Direction 1 is the most likely, and it is cheap **only while `SL-256` is in
flight** — after that the seam exists but nobody is standing at it.

## References

- `DEC-238` — a recording is an occurrence, emitted explicitly; the general
  statement about what derivation can and cannot see
- `SL-256` — the slice that found this
- `ISS-355` — the recorded-half defect
