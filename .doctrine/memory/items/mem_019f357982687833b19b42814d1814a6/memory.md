# A workflow-confined (isolation:worktree) agent gets a READ-ONLY shared .git in the jail — it cannot drive the dispatch funnel; the orchestrator role needs RW .git

Empirically verified SL-206 PHASE-05 (2026-07-06, Opus 4.8, this harness) driving
the shipped `/drive-slice` reference workflow live.

## The trap

`/drive-slice` spawns its per-phase orchestrator as
`agent(orchestratorPrompt, { isolation: 'worktree', agentType: 'dispatch-orchestrator' })`.
Under the bubblewrap jail, an `isolation:'worktree'` agent is confined exactly like
a dispatch WORKER: it is jailed to its own linked worktree, and the shared
`/workspace/<repo>/.git` is mounted READ-ONLY. So every funnel WRITE the orchestrator
must perform fails hard:

- `dispatch arm-spawn` → "Read-only file system" writing `.doctrine/state/dispatch/spawn`.
- `doctrine worktree fork` / `git update-ref` / `git commit` (index.lock) →
  "cannot lock ref … Read-only file system".

Consequence: the orchestrator cannot arm the base, cannot mint or commit a worker
fork, and cannot import/conclude. Phases never reach Completed. (Reads still resolve
— `dispatch_next_ready`/`_phase_receipt` are fine — so it is NOT a coord refusal and
NOT a funnel refusal; it is an environment/permission wall.)

## Why — the model inversion

The dispatch CLAUDE ARM that WORKS puts the orchestrator on the MAIN thread: it has
RW `.git`, operates in/against the coordination worktree, and spawns CONFINED
WORKERS (`isolation:'worktree'`). `/drive-slice` inverts this — it confines the
ORCHESTRATOR, the one role that fundamentally needs RW `.git` + coord write. The
worktree-free `dispatch_import`/`_conclude`/`_reap` MCP tools (object-db, coord
resolved server-side) would be reachable even from a confined cwd, but
`arm-spawn` (coord state write) and worker-fork minting (ref write) are NOT.

## Fix direction

The orchestrator agent must run UNCONFINED so it inherits main-tree RW `.git`,
reach the coord tree via absolute paths + server-side-resolved dispatch tools, and
confine only the WORKER it spawns. This is a DESIGN change to the driver's spawn
model — not a script-contract patch. (Distinct from, and stacked on top of, the
harness-contract defects: pure-literal `meta`, top-level `run` invocation, slice
validation — see the SL-206 PHASE-05 notes.)

## Same root as the worker cases

The RO shared `.git` for a jailed linked worktree is the SAME constraint behind
[[mem.pattern.dispatch.correct-worker-delta-in-place]] (worker can't self-`reset`/
re-commit) and [[mem.pattern.dispatch.worker-commit-stale-path-false-red]]. There
the RO-`.git` victim is the WORKER (correct — the orchestrator lands its bytes).
Here the victim is the ORCHESTRATOR — which is a design bug, because the
orchestrator must never be confined.
