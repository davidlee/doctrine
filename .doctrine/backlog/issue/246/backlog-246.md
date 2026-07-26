# ISS-246: dispatch_reap over MCP returns opaque -32603, losing the gc not-landed diagnosis

Surfaced at SL-228 PHASE-06, alongside ISS-245.

## Symptom

```
dispatch_reap{slice: 228, name: "dispatch/agent-addd7999cd4e54cc9"}
  → MCP error -32603: Internal error
```

No reason, no detail, no remedy. The CLI arm of the same engine says everything
the caller needs:

```
doctrine worktree gc --fork dispatch/agent-addd7999cd4e54cc9 --dry-run
  → dispatch/agent-addd7999cd4e54cc9: not-landed — `--force` to reap, or
    `--superseded-head <SHA>` if spent-and-abandoned. If you squash-merged,
    re-land via `worktree land` (--no-ff).
```

## Cause

`dispatch_reap`'s own tool description states the contract: *"an unlanded fork is
a hard gc error (not-landed)"* — i.e. it is **by design** an `Err` rather than a
`Refused{reason, detail}` variant. The MCP transport then flattens any `Err` to
`-32603 Internal error` and drops the message, so the documented `not-landed`
signal never reaches the caller.

Contrast the sibling write verbs, which model refusals as *data*:
`dispatch_import` and `dispatch_conclude_phase` both return
`{"Refused": {reason, detail}}` with an enumerated reason set.

## Why it matters

The whole point of SL-228 is that a refusal's text **is** the recovery procedure
(FR-009 — `IllegalTransition`'s Display, `src/funnel_machine.rs:357-368`). An
orchestrator driving by verb output alone gets nothing actionable here and must
fall back to the CLI to find out what happened — which is the rescue behaviour
the slice exists to delete. It is a direct hazard for PHASE-07's memory-blind
benchmark: a fresh orchestrator has no memory telling it to re-run the CLI arm.

Cost when hit: 3 extra tool calls to recover the diagnosis.

## Fix

Give `dispatch_reap` the same shape as its siblings — fold `not-landed` (and the
other gc verdicts) into the `Refused{reason, detail}` payload with an enumerated
reason set, and carry the CLI's remedy sentence in `detail`. Reserve `Err` for
genuine internal faults.

Sibling check worth doing at the same time: audit every dispatch MCP tool for an
`Err` path that a caller is expected to act on.

## Relation

ISS-245 is the underlying reason the refusal fires at all; this item is the
independent diagnosability defect and stands even after ISS-245 is fixed.
