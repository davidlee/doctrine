# DEC-193: Capsule quarantine retention and reaping

**Proposed** 2026-08-11, filling the retention axis `DEC-133` left deliberately
unspecified. `DEC-192`'s amendment made it concrete for the first time: the
persistent quarantine repository *is* the forensic exhibit, so *when it is
reaped* is now an unset value rather than an open axis.

## The owner's default position

> With explicit user confirmation, at the end of the closure of a slice, with a
> backstop gc task to reap any stale leftovers.

Three parts, and the shape is right: the exhibit's useful life is the change it
evidences, so slice closure is the natural boundary; a human gate keeps deletion
of evidence deliberate; and a backstop stops orphans accumulating when the happy
path is not walked.

## The refinement this record proposes

**Reap the admitted, hold the refused.** For *admitted* work the objects are in
the canonical repository already — the quarantine is a redundant second copy and
reaping it loses nothing recoverable. For *refused*, *conflicted* or
failed-verification work the quarantine is the **only** copy, and that is
precisely the case `DEC-133`'s forensic exhibit exists to serve. The exhibit is
most valuable exactly where the work did not land.

Both classes are known at close, from the admission journal.

That in turn fixes the confirmation prompt. A per-slice confirmation on the clean
path is **ceremony**: there is nothing to inspect, the answer is always yes, and a
gate whose answer is always yes trains people to click through the one time it
matters. Confirmation earns its keep when it fires only over the held set —
refused, conflicted, failed verification — where a human might genuinely say
"keep that one".

So: auto-reap admitted quarantines at slice close; hold the rest; confirm over
the held set; gc backs the whole thing.

## Two values this leaves for whoever implements it

1. **The gc predicate.** "Stale" needs naming or it rots. *Capsule no longer
   exists* and *slice closed* are both computable and are the primary
   conditions; an age fallback catches the orphan where neither resolves —
   a quarantine whose capsule and slice cannot be determined at all.
2. **The disk high-water mark is per slice, not per capsule.** Persistent
   per-capsule quarantine plus reap-at-slice-close means every phase's
   quarantine coexists for the slice's lifetime. At doctrine's ~32 MiB
   incremental that is noise; under parallel dispatch it is N × repo, and it
   should be a deliberate figure in the volume sizing rather than a discovered
   one.

## What is not in question

The **pin** is durable and unaffected. The ref and sha `capsule-collect` prints
is the admission journal entry, and it survives every reaping policy — `DEC-133`
separates exactly these two lifetimes, and this record only sets the short one.

## Related

`DEC-133` (the axis this fills), `DEC-192` (which made it concrete), `EVD-016`,
`ASM-010` (why a disk image is not an alternative exhibit), `IMP-426`,
`RFC-025`.
