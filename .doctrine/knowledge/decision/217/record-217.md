## Decision

The **confined-orchestrator arm (Mode B)** retires as a running
configuration together with the in-session claude worker arm that `SL-254`
collapses. This is a *consequence* of two choices already settled, not a new
judgement.

## Why it follows rather than being chosen

`funnel_machine`'s `TABLE` admits only `Transition::Spawn` from `current: None`,
so a phase with no funnel row can reach no other position. Every production
`Spawn` writer needs a **bound** fork — one cut with `--worker --slice N --phase
PHASE-NN` and a `dir` under `<coord>/.worktrees/<name>` (`fork.rs:216-246`):

| `Spawn` writer | fate in `SL-254` |
|---|---|
| `land_spawn_row` (`dispatch.rs:6958`, via `create-fork`'s Fork arm) | deleted with the Fork arm (`D2`) |
| `worker_commit` | deleted (`DEC-204`) |
| `dispatch_import`'s heal-forward (`mcp_server/dispatch.rs:286-315`) | retained, but refuses without the durable binding |

The design's `OQ-1` settles that the collapsed arm's forks stay **unbound**, as
the pi arm's already are. Unbound forks and a live Mode B are therefore
**mutually exclusive**: after the collapse no production path lands a `Spawn`
row, so Mode B loses its entry point — not merely one tool's producer, which is
all `OQ-2` previously admitted. Mode B additionally arms `create-fork` through
`dispatch arm-spawn` (`dispatch-mechanics.md`, "The fork override"), which `D2`
deletes.

**Reversing this means reversing the unbound-fork settlement, not amending
prose.**

## What is *not* affected

**Mode A — the main-thread orchestrator — is untouched.** It applies the worker
delta, commits, flips the phase and records the boundary as *separate acts*, and
it **never consults the funnel record**; `install/dispatch-mechanics.md` names "a
fork with no funnel row" as an explicitly supported case for the reap oracle.
Mode A is what the codex/pi subprocess arm runs in production and is what the
collapsed arm lands on. A reading of `dispatch next`'s prescription as universal
is what produced `RV-355` `F-1`'s claim that the collapse "cannot drive verify,
conclude, or reap" — true of Mode B, false of the retained path
([[mem.fact.dispatch.mode-a-vs-mode-b-funnel]]).

## Consequences

- The MCP funnel tools and `funnel_machine` are **retained, not deleted** (design
  `D6`), now with no production producer for `Spawn` *or* `RecordWorkerCommit`.
  `funnel-machine.md` is a generated artefact pinned to the code table, which does
  not change, so it is not a `REV` target.
- `SPEC-021` `REQ-384`'s enumerated positions `spawned` and `worker-committed`
  lose their production writers, and `REQ-387`'s "whether committing directly or
  through mediation" loses the mediated leg's entry point. Both join the `REV`.
- `REQ-335`'s confined-orchestrator tier stays `pending` **as a contract**; what
  retires is its one partial implementation.
- `plugins/doctrine/skills/dispatch/SKILL.md`'s claim that the funnel "is driven
  by `doctrine dispatch next` … identical on both arms" is false of an arm that
  never lands a row, and is corrected.

Raised by `RV-355` `F-1`.