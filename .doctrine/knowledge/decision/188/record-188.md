## Decision

`SL-252` takes `DEC-185`'s recorded fallback, S3: the fixture's readable set stays **directory-granular and wide**, but it is **declared rather than host-derived**, and the transcript **states the posture** so a reader knows what the verdicts are worth. No closure resolver, no narrowing to a payload toolset, no change to production.

This resolves `inq-1` — the unit of the readable set — as *declared roots*, and it is not the shape this slice set out to build.

## Why not the narrowing

Every narrowing shape examined turned out to be blocked by the same production defect, `ISS-344`. Readable inputs are resolved with `canonicalize` and then bound at the resolved path, source and destination both, so the declared name is destroyed. File-granular sets therefore cannot invoke multicall binaries at all, and any set narrow enough to matter empties the inner `PATH` — which breaks *provisioning*, since `clone_inside` runs all four git operations as bare `git`.

Repairing that means a revision against `SL-248` `PHASE-05` `EX-3`, a redesign of the inner-`PATH` derivation, and probably binding closure roots as well. That is a slice, not a phase, and `DEC-184` already fixed this slice's boundary at its own narrowing.

## What S3 still buys

`ISS-341`'s complaint is that *"the verdicts are not false — they are worth much less than they read, and nothing in the transcript says so."* S3 answers the second clause in full and the first in part:

- the readable set stops being derived from the host's `$PATH` by walking to top-level ancestors, so it is bounded by what the fixture says rather than by what the host happens to look like;
- `/run` and `/var` stop being roots, which is `ISS-340`;
- the posture is reported on the verdict-free `observations` channel, so a reader is told the set is wide.

## What it does not buy, stated plainly

`REV-051` ruled `REQ-459` criterion 2 **partial** because the fixture binds whole host top-level roots. S3 does not close that. The set is still wide; it is merely honest about being wide. The finding stays partial until `ISS-344` is repaired, and the reconciliation must say so rather than claim the shortfall discharged.

## The judgement behind it

The owner's stated reason for taking S3 is not that it is correct. It is that the capsule programme was conceived to shed complexity and increase guaranteed containment, and the evidence of this slice is that it has traded one kind of complexity for another: live design errors that surface as subtle, host-sensitive, expensive-to-diagnose defects, over a surface so large that engaging with the design at all is a major context investment. `SL-252` is being closed out cheaply and honestly so the programme can be revisited architecturally rather than patched further. See `RSK-231`.
