# IMP-359: arm-spawn accepts arming a phase past the worker-commit position

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine dispatch arm-spawn --slice N --phase PHASE-NN` arms a claude-arm
worker spawn without consulting that phase's funnel position. If the row has
already advanced past `imported` — the last position at which `worker_commit`
will accept a delta — the spawn is doomed, and **nothing says so until the
worker's final action**.

## Observed — SL-233 PHASE-14

The first worker landed 7 of 8 exit criteria; the funnel ran `import` then
`verify`, reaching `verified`. The orchestrator armed and spawned a second
worker to deliver the eighth criterion. Every preceding step passed silently:
`check prove`, `worktree marker --clear`, `check regression capture`,
`arm-spawn` ("armed SL-233 at base <B>"), and the `WorktreeCreate` hook forked
and provisioned normally.

The worker then did ~14 minutes of correct, green work and hit the wall at the
end:

```
{"Refused":{"reason":"already-verified","detail":"record-worker-commit refused at position `verified` — already-verified; expected: conclude"}}
```

The refusal is correct and well-worded. The defect is purely **where it fires**:
`worker_commit` is the last action of a worker's life, so a positional
precondition enforced there costs a whole worker run to discover, and the delta
has to be rescued through the fallback live-worktree import path plus a manual
coordination commit.

## Proposed

`arm-spawn` already takes both `--slice` and `--phase`, so it knows exactly
which funnel row it is arming against. It should refuse (or at minimum warn
loudly) when that row stands past `imported`. The information is available at
arming time and the failure is otherwise unrecoverable in place.

## Adjacent, not duplicate

IMP-233 asks `arm-spawn` to resolve the coord tree from `--slice` or refuse a
wrong-root arming write. Same command, independent guard — that one is about
*where* the arming write lands, this one about *whether the phase can still
accept a worker at all*.

## Secondary observation, worth its own consideration

The funnel models **one worker per phase** and has no first-class path for a
phase that lands partially complete. Related: `dispatch next` prescribed
`conclude` for PHASE-14 while its `VT-1` was failing, because `dispatch verify`
runs the test suite and does not consult the VT anchors. An orchestrator
obeying the oracle blindly would have concluded an incomplete phase.

Surfaced by SL-233 PHASE-14.
