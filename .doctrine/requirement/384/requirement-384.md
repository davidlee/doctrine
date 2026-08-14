# REQ-384: Dispatch funnel position is persisted per-phase as authoritative run-state, advancing through explicit transitions that include verification as an authoritative, evidence-carrying step faithful to REQ-287 ordering (spawned, worker-committed, imported, verified, concluded, reaped); the record has a single-writer authority and crash-safe idempotent recovery (its concrete home and CAS/concurrency contract are slice design).

## Statement

> **AMENDED — NARROWED (SL-254, 2026-08-14).** The requirement stands and the
> funnel record is RETAINED; only the reachability of one position narrows. The
> `worker-committed` position and its `record-worker-commit` transition remain in
> the ordering and in `src/funnel_machine.rs`, but on the shipped path no
> producer lands them on their own: the worker CANNOT commit — it hands back an
> UNCOMMITTED working tree and the orchestrator imports the working-tree diff —
> and the `worker_commit` MCP tool that was their only standalone producer is
> deleted (`DEC-204`). The title is retained unchanged; the elaboration below
> states how the position is reached now.

The position is reached only as part of import's healing prefix. `dispatch_import`
folds the whole provable prefix `[Spawn, RecordWorkerCommit, Import]` in one
advance, so the record still carries a faithful, ordered history — but the
position never durably RESTS at `worker-committed`; it passes through it. The
`already-worker-committed` refusal token and the transition's replay identity are
retained for exactly that fold, and because the machine is the single authority
that may land a position (`funnel_machine::attempt`), retaining the row costs
nothing and preserves REQ-287's ordering as stated.

Unaffected and still required: per-phase persistence as authoritative run-state,
verification as an authoritative evidence-carrying step, single-writer authority,
and crash-safe idempotent recovery.

**Related, deliberately not asserted here.** SL-254's shipped path leaves forks
UNBOUND (`OQ-1` — no `DispatchRecord` is written) and the funnel row is named by
the orchestrator's explicit `PHASE-NN`. That changes who supplies the phase key,
not the record's contract, and `OQ-1` is unsettled.

## Rationale

Deleting a position because its producer went away would rewrite the ordering
REQ-287 fixes and would lose the distinction between "the fork tip existed" and
"the delta landed" — a distinction import's replay identity still depends on. The
cheaper and more honest move is to keep the position and record that it is now
transited rather than occupied.
