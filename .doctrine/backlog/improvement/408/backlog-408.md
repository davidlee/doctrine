# IMP-408: CoverageStatus Blocked carries no cause

`RunOutcome::Unobtainable` fuses at least six distinct conditions — resolve
error, spawn failure, wall-clock timeout, empty or unreadable glob match set,
unparseable hand-edited regex, empty argv — into `CoverageStatus::Blocked`, and
`CoverageEntry` persists no reason, exit code, or captured output.

A `Blocked` cell therefore cannot be triaged without re-running it. The
information existed at the moment of failure and was discarded.

**The fix exists nearby**: `FailureSet::Unobtainable { why: String }`
(`src/regression.rs`) already carries its cause. This is the same "solved in
one subsystem, absent from its neighbour" pattern as ISS-325.

**Sequencing.** Schema change to a persisted type — governed by the SPEC-002
amendment CHR-058 produces, same as ISS-325.

Evidence: `.doctrine/rfc/027/proof-binding-study/conclusion.md` § R3.
