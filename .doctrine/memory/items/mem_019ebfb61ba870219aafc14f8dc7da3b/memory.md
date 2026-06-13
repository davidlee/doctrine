# Claude Code dispatch worker identity via SubagentStart hook, not a SessionStart env-seam

Empirically established (claude-code **2.1.173**, in-jail) by a live hook+subagent
experiment for SL-056. These overturn three premises the SL-056 design had been
*guessing* about Claude's hooks API.

## What fires (and what does not)

- **`SessionStart` does NOT fire for `Agent` `isolation:worktree` subagents.** Only the
  top-level/main session gets it (incl. `source:"resume"`). A subagent **inherits the
  parent session's environment** (same `CLAUDE_CODE_SESSION_ID`), so any var a parent
  SessionStart hook injected via `CLAUDE_ENV_FILE` is *carried over but points at the
  main root* — useless as worker identity.
- **`SubagentStart` DOES fire** for each subagent, before it runs. Payload:
  `{session_id (=parent's), transcript_path, cwd (=the worktree path), agent_id,
  agent_type, hook_event_name}`. **But `CLAUDE_ENV_FILE` is UNSET for it** — a
  SubagentStart hook **cannot inject persistent env** into the subagent.
- ⇒ **The SessionStart + `CLAUDE_ENV_FILE` env-seam is NOT reachable for the Agent-tool
  subagent spawn model.** Claude worker identity cannot be env-based like codex/pi's
  `DOCTRINE_WORKER`. (Only a *different* spawn model — a fresh top-level `claude`
  session in the worktree — would fire SessionStart; rejected as API-billed.)

## The clean mechanism the experiment found

`SubagentStart` hands an orchestrator-configured hook, per subagent, before it runs:
**`cwd`** (which worktree to stamp), **`agent_type`** (a discriminator the orchestrator
controls via the dispatch `subagent_type`), and **`agent_id`** (a correlation handle).
So claude worker identity = **a SubagentStart hook stamps the disk marker into `cwd`
when `agent_type` is the dispatch-worker type** (and `cwd` is a linked worktree). This
discriminates dispatch from benign subagents (no over-broad branding) and needs **no
arm sentinel** — each SubagentStart fires for its own subagent independently.

- **Concurrency is race-free** (confirmed): two simultaneous `isolation:worktree`
  subagents produced two SubagentStart events, each with its own distinct
  `cwd`/`agent_id` — no shared slot, no cross-talk. So the SL-056 **serial-only +
  arm/sentinel/lease/single-slot apparatus is obviated**; concurrent claude dispatch is
  safe. This dissolves SL-056 charges γ/θ/κ/ο and reframes Charge C.

## Other corrected facts

- `Agent isolation:worktree` ⇒ a **real linked git worktree** at
  `.claude/worktrees/agent-<agentId>` (branch `worktree-agent-<agentId>`, git-dir
  `.git/worktrees/agent-<agentId>`). `is_linked_worktree` is a sound identity substrate.
- **The Agent tool returns a handle** — `agentId` — and the worktree name embeds it. The
  design's repeated "Agent returns no worktree handle" claim is **false** (though the id
  returns *after* the worker runs, so it's a post-hoc/gc correlation handle, not a
  pre-stamp one).
- `CLAUDE_CODE_CHILD_SESSION=1` is present on **both** orchestrator and subagent — **not
  a discriminator**. `WorktreeCreate` hook payload (separate from SubagentStart) carries
  `name`, not `agent_type`.

**Why:** SL-056 was burned across 7 inquisitions partly by *guessing* Claude's
capabilities. Verify hook behaviour with a live probe (temporary SessionStart/
SubagentStart logging hook + `CLAUDE_ENV_FILE` injection test + a spawned subagent that
reports its env), never infer from docs alone — they were silent/ambiguous here.

**How to apply:** For the SL-056 claude path, stamp the marker via a SubagentStart hook
keyed on `agent_type`; keep the disk marker as the harness-agnostic floor (codex/pi
still stamp via `fork --worker` + `DOCTRINE_WORKER`). Treat the SubagentStart payload
shape + its missing `CLAUDE_ENV_FILE` as version-fragile harness facts — spike-gate them
(cf. [[mem.pattern.parse.toml-error-classification-fragile]]). Relates to
[[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]] (the agnostic
floor still cannot rely on a free env seam; claude's is confirmed *absent* for subagents).
