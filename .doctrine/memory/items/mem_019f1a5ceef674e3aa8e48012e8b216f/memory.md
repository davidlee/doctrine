# Dispatch arming is single-slot — per-arming policy granularity

`dispatch arm-spawn` writes **one** `base` file into one arming dir
(`<coord>/.doctrine/state/dispatch/spawn/`); `dispatch-agent` issues **N parallel
spawns off one arming** ("arm once, then issue N spawns … all read the same B",
`.claude/skills/dispatch-agent/SKILL.md`). The harness-assigned worktree `name`
(`agent-<id>`) does not exist until the `create-fork` hook fires at spawn, so
**there is no pre-spawn key that distinguishes parallel siblings.**

**Consequence for per-worker state (e.g. jail policy):** the slot's natural
granularity is **per-arming**, not per-worker.
- **Serial drive** (one in-flight worker per arming): per-arming == per-worker.
  The single pre-declared intent binds unambiguously to the one new worktree.
- **Parallel fan-out**: the one declared intent is **shared by every worker in
  the batch** — intentional sharing, not a leak (one slot = one intent, no
  differing sibling to cross-contaminate). The orchestrator must declare a value
  valid for ALL members.

**Why no inversion fixes it on the claude arm:** the `Agent` call **blocks until
the worker completes**, so the orchestrator gets no turn between spawn and the
worker's first tool call in which to write name-keyed state; a worker writing its
own id would breach ADR-006 sole-writer. **Distinct concurrent per-worker
profiles need the pi/subprocess arm** (orchestrator runs `worktree fork --worker`
itself → knows the name pre-spawn) or a future per-spawn correlation token.

- Verified against `src/dispatch.rs` (`run_arm_spawn`), `src/worktree/create.rs`
  (Fork/Passthrough), `dispatch-agent/SKILL.md`, 2026-07-01. Origin: SL-182
  RV-200 F-1.
- See [[mem.fact.claude.worktree-remove-auto-teardown]].
