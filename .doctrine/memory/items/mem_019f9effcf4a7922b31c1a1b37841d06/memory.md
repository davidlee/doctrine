# A half-arm binds nothing — `arm-spawn --slice` without `--phase` costs a whole worker run

SL-228 PHASE-04 made the durable fork binding `(slice, phase)`, snapshotted into
the worker's `DispatchRecord` **one-shot at the fork point** so a stale arm can
never mis-bind the next spawn. Both halves are required:
`DispatchRecord::binding()` folds a half-bound record to `None`, and
`require_binding` turns that into the typed refusal
(`src/worktree/dispatch_record.rs`).

```
doctrine dispatch arm-spawn --base <B> --slice 228                     # ← binds NOTHING
doctrine dispatch arm-spawn --base <B> --slice 228 --phase PHASE-09    # correct
```

## The failure mode is end-loaded

Nothing refuses at arm time, at `cd`, or at spawn. The worker forks, works,
tests, and only its **final** `worker_commit` refuses:

```
{"Refused":{"reason":"unprovable-fork","detail":"dispatch/agent-<id>"}}
```

So the entire worker run is spent before the orchestrator's mistake surfaces.
The `detail` carries only the branch name — it names neither the cause (a
half-arm) nor a fixing verb, which is a live counter-example to the
"a refusal's text IS the recovery procedure" claim (SL-228 D10, ISS-250).

## There is no re-bind verb

The binding is snapshotted at fork time by design, and no verb rewrites it on an
existing fork. Two real options:

1. **Re-arm with both halves and re-spawn.** Clean — the fork-time snapshot
   property is preserved. Costs a second worker run.
2. **Fallback (A), the live-worktree import.** The delta is uncommitted in the
   worker's worktree, which persists after the Agent returns:
   `doctrine worktree import --base <B> --from-worktree <path> --slice <N>` —
   same `classify_import` scope belt, same in-process prove gate, non-committing.
   Then the orchestrator commits one on the coord branch and supplies the
   attribution itself: `dispatch record-boundary --slice <N> --phase PHASE-NN
   --code-start <B> --code-end <S>` (plus `slice record-delta`, ISS-241).

Option 2 is legitimate here and NOT "routing around a refusal": `unprovable-fork`
is a verdict on the orchestrator's provisioning, not on the worker's content.
The refusals you must never import around are the delta verdicts —
`forbidden-zone`, `commit-gate-red`, `undeclared-scope`. Cost of option 2: the
commit carries orchestrator authorship, and the phase attribution is explicit
rather than fork-time-snapshotted.

## Why an orchestrator walks into it

Handover packets and older notes carry the pre-PHASE-04 command shape
(`arm-spawn --base B --slice N`), which was complete when written. Read
`arm-spawn --help` rather than a handover's funnel recipe — the flag's own text
says it outright: *"Both halves are needed: a half-arm binds nothing."*

Observed driving SL-228 PHASE-09 (2026-07-27); the worker's delta was green and
was landed via option 2.
