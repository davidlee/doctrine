# Confined orchestrator drive-loop realizable on claude arm

**A fully-confined subagent orchestrator (cwd = coordination worktree, jailed) CAN
drive nested `isolation:worktree` workers on Claude Code 2.1.198.** The SL-199
capstone (Mode B, off-main-thread orchestration) is feasible on the claude arm —
the earlier "F7 blocker / infeasible" framing was WRONG; it was two recipe defects,
not a harness ceiling. Verified live (SL-199 PHASE-05 F9/F10/F12).

**Two provisioning requirements for the confined per-phase Fork to land
`dispatch/<name>`@B with a jail record (so `worker_commit` resolves):**

1. **Worker isolation must be deterministic** — `isolation: worktree` on the
   `dispatch-worker` def FRONTMATTER, not a per-call Agent param
   ([[mem_019f3161168a79e3ac628ed1c2415b55]]). Per-call omission → worker runs
   in-place in the coord tree, no fork ([[mem_019f2d4d50427d61bc562373551d4df3]]).
2. **Arm with the base** — `doctrine dispatch arm-spawn --base <coord-tip-sha>
   --path .` (the `--base` is REQUIRED; `arm-spawn --path .` alone exits 2). Writes
   `.doctrine/state/dispatch/spawn/base`. The confined create-fork trigger is
   `cwd_is_coord_root ∧ coord_in_dispatch ∧ base` (`src/worktree/create.rs`); the
   discriminator is the orchestrator's cwd (always coord-root — it can't reach the
   arming dir because a confined subagent's cwd resets every Bash call,
   [[mem_019f2c5ba5bc70a38fe015a817b3e270]]). Absent base ⇒ confined Passthrough ⇒
   benign detached tree, no branch/record ⇒ `worker_commit` unknown-agent.

With both, the harness `WorktreeCreate` hook Forks off `base` regardless of how
isolation was requested — base-control lives in the hook, not the isolation
mechanism ([[mem_019ec6142d3b71008f2149a6d84ba981]]).

**Design impact:** SL-199 design §5.D steps 1–2 are stale — step 1 omits `--base`,
step 2 frames isolation as a call-param. Both need the amendment above; §6 probe
was primitive-correct. Full detail: `.doctrine/state/ex3-scaffold.md` (F1–F13) and
the SL-199 PHASE-05 replan brief.
