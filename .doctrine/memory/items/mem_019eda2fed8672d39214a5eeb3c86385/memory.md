# Pre-enumerated maps are suggestive, never exhaustive

A list of reference sites written by a human — a design's reference map, a
finding's "affected sites", your own sense of where a symbol appears — is a
**hypothesis**. The ground truth is a `rg`/`grep` sweep run to exhaustion and
actually read. Enumerate, then sweep, then reconcile the two.

Original evidence: `SL-082` § 3.2's map `K1`–`K11` missed `reconcile/SKILL.md:20`,
caught only by the sweep.

## It applies to the footprint of a *repair*, not just to a design's map

This is the broader form and the one that keeps biting. When you integrate a
review finding, you enumerate where the fix must land — and that enumeration
deserves exactly the same distrust as the enumeration that produced the finding.

`SL-253` `RV-354` `F-1`: the repair changed a function's return type. The
integration updated it in the kernel signature and in the prose narrowings list,
and **missed a third declaration** in a payload-side contract block, shipping a
document that contradicted the ruling it had just adopted. A refused-alternative
bullet elsewhere carried the same stale shape. `grep -- "-> RowVerdict"` returned
seven hits and would have caught both in seconds; reasoning about where the
signature "obviously" appeared did not.

That design had already recorded four enumerations of a related class, each
believed complete and each wrong. The repair's footprint made five — and it was
the first that was not about the subject matter at all.

## The rule

**Never conclude an enumeration from reasoning when a grep can decide it.** After
sweeping, read every hit and classify it — a raw count is not a result. Expect
live contracts, historical descriptions and refused alternatives to be mixed
together, and say which each hit is; that classification is the deliverable, and
it is what makes the sweep re-checkable by the next reader.

A negative grep is also untrustworthy without a positive control — see
[[mem.pattern.search.negative-grep-needs-a-positive-control]].