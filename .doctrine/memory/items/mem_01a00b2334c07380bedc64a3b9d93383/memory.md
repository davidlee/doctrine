# A removal claim must attribute every surviving occurrence

Found in `SL-238` PHASE-04, discharging a `VA` criterion that asked for a record
of *"the disappearance of the `dropped (dangling: …)` line form"*.

Grepping the string in the post-state returned **1, not 0**. Both instinctive
readings are wrong:

- *"still there — the deletion failed"* → it did not; and
- *"close enough, it's a comment or a test"* → it was neither.

The string had **three** producers in the pre-state, on **two different
surfaces**: two inside the renderer the phase deleted, and one inside an
unrelated write path (`backlog after --prune`) the phase never touched and a
later phase owns. So the only true claim is a **scoped** one — *the listing
footer's form is gone; the identically-worded `--prune` report line survives,
deliberately*.

## The method

Count producers in the **pre-state**, not just the post-state:

```sh
git show <base>:<file> | grep -n -F "<the string>"   # 3 hits, with line numbers
grep -n -F "<the string>" <file>                     # 1 hit
```

The pre-state hit list is what lets you attribute each survivor. A post-state
grep alone yields a number with nothing to subtract it from, so it cannot tell a
deliberate survivor from a missed deletion. The same sweep found a second
survivor of the same shape on the same untouched surface (the word `absent`),
which a one-string check would have missed entirely.

## Why it matters

An unscoped *"X is gone"* in an audit record is a false claim — and in a slice
whose whole purpose is removing a surface that stated something it had not
checked, it is the very defect class under repair. It also mis-primes the next
phase: a later sweep reads the deliberate survivor as a leftover and deletes
behaviour nobody decided to delete.

Related: [[mem.pattern.harness.grep-negative-needs-positive-control]] — the
mirror case, where a *zero* needs a control to be believed. Here a *non-zero*
needs a pre-state census to be interpreted. And
[[mem.pattern.review.sweep-defect-class-not-instance]] — sweep the class, not
the instances you were handed; the `VA` named one string, and the class had two.

Two siblings from the same seam:
[[mem.pattern.testing.grep-for-the-pin-before-characterising]] — the pre-state
census is the same move applied to *tests* rather than to output strings; and
[[mem.pattern.plan.verification-subject-must-outlive-the-phase]] — this
finding surfaced while discharging exactly such a `VA` into authored notes,
which is the only reason the scoping survived the phase.
