# SubagentStart fires for Agent-tool spawn from a child-session main thread

**Claim.** A `SubagentStart` command hook **fires** for a top-level `Agent`-tool
spawn *even when the spawning session itself has `CLAUDE_CODE_CHILD_SESSION=1`*.
The hook matched on `agent_type`, carried `agent_id` + `agent_type`, and its write
landed **before** the subagent's first `PreToolUse` (sync-blocking).

**Evidence (SL-206 P1b, 2026-07-06).** Session env `CLAUDE_CODE_CHILD_SESSION=1`.
Main thread `Agent`-tool-spawned a `dispatch-orchestrator` subagent (agent
`a418b827a5f402ec4`). A `SubagentStart(dispatch-orchestrator)` hook wrote the
orchestrator's `agent_id` to an allowlist; the `worktree pretooluse` hook then
saw it on the subagent's very first Bash (3× `NOMINATED-passthrough`, no denial).
Preserved: `.doctrine/slice/206/scratchpad`/`p1-evidence-nominate.log`.

**Correction to [[mem_019ec84b53...]].** That memory states SubagentStart
"silently no-fires when the spawning session is itself a nested/child Claude
session (`CLAUDE_CODE_CHILD_SESSION=1`)." Refuted as stated: a child session's
**own main thread** spawning via the `Agent` tool DOES get `SubagentStart`. The
true no-fire condition is narrower — likely only spawns nested *below* that (a
subagent spawning a subagent), which this POC did NOT exercise. Do not rely on
"child session ⇒ no SubagentStart" for the top-level orchestrator spawn.

**Also observed.** A `settings.local.json` hooks edit **hot-reloaded** — the new
`SubagentStart` matcher took effect without a session restart.

**Consequence for SL-206 A2.** The orchestrator-unjail nomination (SubagentStart →
allowlist → `pretooluse` PassThrough) is *more* robust than the direction doc's
§4.4 hedge assumed: the lie it hedged against did not materialise for the
orchestrator spawn. Strengthens A2; further weakens D' (workflow-form revival).

See [[mem_019ee3a08...]] (SubagentStart fires + is matchable), [[mem_019ec0a5...]]
(sync-blocking, not fail-closeable), and `.doctrine/slice/206/unjail-direction.md`
§4.3 / §6 P1 RESULT.
