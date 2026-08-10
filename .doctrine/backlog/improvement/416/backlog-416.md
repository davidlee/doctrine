# IMP-416: Agent-execution timeout key

A second execution-timeout key, sized against an **agent's** tail rather than a
build's, with its own parse, default posture and enforcement.

## Why

`SL-248` `sec-5` derives `execution-timeout-seconds = 900` from the spike's Rust
*build* fixture — 352 s measured, re-bounded to 900 with headroom. That figure
bounds a build.

But `ADR-020` makes the capsule the dispatch authority boundary, `DEC-134` fixes
the headless worker, and the capsule mounts `/agent` as "the state a harness
accumulates across a run" — so what a capsule executes is an **agent**, and
`Execution.timeout` bounds it. An agent run is not a build. `SL-248`'s own
`PHASE-03` planner sub-agent took 650 s writing no code and running no builds,
and a worker phase on that slice ran about a session. At 900 an agent is
`SIGTERM`ed mid-phase.

## The ruling this implements

Owner's ruling, 2026-08-08: **`900` stands, scoped to build/verification
execution, and agent execution gets its own bound as separate work.**

The rationale is that the two workloads have very different characteristics, and
a bound slack enough for an agent means waiting two hours to discover a `cp`
typo. One key serving both is wrong in both directions — too tight for the
agent, too slack to fail a build fast. So this is a second key, not a bigger
number.

## Scope note

The design correction is separate and belongs to `SL-248`'s reconciliation:
`sec-5`'s justification for `900` should say it bounds a build/verification
contract, not any capsule execution. That edit is not this item.

This could not be done inside `SL-248`: `PHASE-03` `EX-19` fixes the key set, so
adding a key there would have broken an exit criterion as written.

## References

`SL-248` `notes.md` § *Owed* item 16(b) · `RV-352` · `ADR-020` · `DEC-134`
