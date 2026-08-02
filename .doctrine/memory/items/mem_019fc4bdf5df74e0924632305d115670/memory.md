# A refused write leaves no oversized artifact

Found in SL-241 PHASE-05 writing H7, the disk-cap row of the P-C3 hostile
matrix (`scripts/spike-capsule`).

The obvious observable for a "capsule exceeds its disk bound" row is an
oversized file. **It is unobtainable, and that is what the bound MEANS.**

`ulimit -f` (RLIMIT_FSIZE) refuses the write *at* the limit rather than
trimming a file it let through. So `dd`/`truncate` opens the file, seeks past
the limit, takes SIGXFSZ on the write, and leaves a **zero-length** file
behind. A clause counting files at or over the cap finds none — on every host,
forever. It can only ever red.

## What to do

**Assert the refused write's SIGNATURE, not its product.** The artifact the
process attempted is present and *empty* — that is the observable, and it is a
stronger one, because a non-empty file there would mean the write SUCCEEDED and
whatever refused the run was not the bound.

**Pair it with the clause that makes it mean something.** "Present and empty"
passes just as well against a process that never wrote the file at all. Pair it
with the cumulative footprint measured *under* the cap, which says two things at
once: the honest work is nowhere near the bound (so the overrun is attributable
to the hostile write), and the cumulative leg had nothing to say (so the leg
that fired was the per-file one).

## The generalisation

A limit enforced by REFUSAL is not observable through the state it prevented.
Look instead for:

- the attempt's residue (an empty file, a partial record, a rolled-back row),
- the enforcer's own report (an exit status, a signal, a log line), and
- a control proving everything else stayed under the limit.

The same shape catches its dual: **a clause that cannot fail is not a control.**
"No file over the bound reached the quarantine" reads like containment and is
vacuous when no such file can exist anywhere — it would pass on every run for a
reason unrelated to what it claims to check.

See [[mem.pattern.tests.invert-ordering-by-wrapping]] and
[[mem.pattern.tests.mutate-the-data-not-just-delete-it]] for the falsification
discipline that surfaces both.
