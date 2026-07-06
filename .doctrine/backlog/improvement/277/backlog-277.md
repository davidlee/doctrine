# IMP-277: drive-slice claude arm never arms the worker spawn

Surfaced by SL-206 PHASE-16 live acceptance (3 drives, 2026-07-07). Deferred by
operator decision (accept the UX compromise now, harden later).

## The gap

`install/workflows/drive-slice.js` (the `/drive-slice` driver) **never calls
`doctrine dispatch arm-spawn`** — `grep arm-spawn` = 0 hits. On the claude arm the
Workflow script spawns the worker leaf directly (`agent({agentType:'dispatch-worker',
isolation:'worktree'})`). For that leaf's `worktree create-fork` hook to fork at the
coordination base (worker-marked + provisioned, so `worker_commit` resolves), the
**spawner's cwd must be the coord tree's arming dir** with
`.doctrine/state/dispatch/spawn/base` written — the cwd is the positional
discriminator (`arm-spawn --help`, design §5.3). Unarmed ⇒ benign Passthrough ⇒
detached@HEAD, unmarked ⇒ base guard refuses the worker (fork_tip null). This is
exactly the 3-drive failure mode.

## Verified-viable recipe (what the driver SHOULD do)

Capability is proven: `mem.fact.dispatch.armed-workflow-worker-mode-b-viable`
(SL-206 PHASE-08 P5 probe, `verified`) — an ARMED workflow dispatch-worker leaf
retains `worker_commit` and resolves to a provisioned `DispatchRecord`. Mechanism
web: `mem.fact.workflow.agent-worktree-fires-create-fork-hook` (payload cwd = driver
working dir), `mem.fact.claude.workflow-strips-agent-tool` (workflow leaves can't
nest-spawn, but KEEP MCP), `mem.fact.dispatch.confined-orchestrator-driveloop-realizable`
(arm-spawn --base requirement).

To realize (B) e2e for real multi-phase drives:
1. **Launch ritual** (currently un-encoded operator step) — set up the coord tree on
   `dispatch/<slice>`, `dispatch arm-spawn --base <coord-tip>`, park the drive
   session cwd at the printed arming dir, THEN launch the Workflow. Should be wrapped
   (a launcher, or documented in the driver header / a `/drive-slice` runbook).
2. **Per-phase re-arm** — after each `dispatch_conclude_phase` advances the coord tip,
   the interior O (dispatch-orchestrator, unjailed, has Bash) must run
   `arm-spawn --base <new-tip>` during prep, before the next leaf spawn. Without this
   the base file stays at phase-1's base and phase-2's worker forks at the wrong base.
   Add the arm-spawn step to the O's prep instructions in `hopPrompt`.

## Fallback / alternative (not chosen)

FD-1: drive via an **Agent-tool-spawned** orchestrator (retains the `Agent` tool →
arms + cd + spawns workers itself), instead of a Workflow. Cleaner spawn ergonomics
but abandons the "Workflow is sole spawn authority" seam posture and moots EX-4.
Recorded in `mem.fact.claude.workflow-strips-agent-tool`.

## Acceptance note

PHASE-16 was closed with the green e2e witness WAIVED (cost — a full drive is
~15min/~200k tok and the topology is now understood). (B) is capability-verified;
this item hardens the driver so the verified recipe is actually encoded rather than
left as an operator ritual.
