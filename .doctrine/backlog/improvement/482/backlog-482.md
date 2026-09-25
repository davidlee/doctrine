# IMP-482: Surface foreign-scoped commits inside a recorded phase boundary range

A recorded phase boundary can span another slice's commits and nothing signals it:
`dispatch record-delta` stamps `(start, end)` and `doctrine dispatch phase-receipt`
reports the pair, but no verb reports that the range contains commits whose
`(SL-NNN)` scope differs from the phase's slice. The reviewer must `git log` each
span and exclude foreign files by eye (obs `01a0d23b`).

Fix: a boundary/receipt field naming foreign-scoped commits in the range, or a
review-verb check. Related: `ISS-268`, `ISS-317`. See RFC-032 `research.md` F9.
