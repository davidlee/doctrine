`install/dispatch-mechanics.md` documents **two** orchestration modes, and the
distinction is easy to miss because most prose says "the funnel" without saying
which.

- **Mode A** — the main-thread orchestrator: unconfined, runs raw git, applies
  the worker delta, commits, flips the phase and records the boundary as
  *separate acts*. It does **not** consult the funnel record. This is what the
  codex/pi subprocess arm runs in production.
- **Mode B** — the confined-orchestrator arm: cwd jailed to the coordination
  tree, `.git` read-only, drives the same fork→land pipeline entirely through the
  dispatch MCP tools, and the funnel record is its state.

**The trap.** `funnel_machine`'s `TABLE` admits only `Spawn` from
`current: None`, and the production `Transition::Spawn` writers are
`land_spawn_row` (via `create-fork`, the claude arm), `worker_commit`, and
`dispatch_import`'s heal-forward. All three need a **bound** fork — one forked
with `--slice N --phase PHASE-NN` under `<coord>/.worktrees/<name>`.
`scripts/pi-spawn-confined.sh` passes `--worker` alone, so pi-arm forks are
unbound, no Spawn row ever lands, and `dispatch next` would sit at `Spawn`
forever. That is not a bug: Mode A never asks. `dispatch-mechanics.md` names "a
fork with **no funnel row** (solo / pre-funnel / legacy)" as a supported case for
the reap oracle.

**Why it matters.** Reading `dispatch next`'s prescription as universal leads
straight to a false conclusion — an external reviewer on `RV-355` concluded the
`SL-254` collapse "cannot drive verify, conclude, or reap", which is true of Mode
B and false of the retained Mode A path. Before reasoning about funnel rows, fix
which mode you are in. `plugins/doctrine/skills/dispatch/SKILL.md` currently
says the funnel "is driven by `doctrine dispatch next` … identical on both arms",
which cannot be true of an arm that never lands a row — treat that line as
suspect until it is corrected.

Related: [[mem.signpost.doctrine.dispatch]].
