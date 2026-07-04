# RSK-226: worker_commit cannot authenticate its MCP caller (no agent_id seam)

<!-- Backlog body — prose, never structurally parsed (storage rule). -->

## The risk

SL-198's `worker_commit` MCP tool commits a delta into a worktree named by the
worker-supplied `dir` argument, running in the unconfined doctrine MCP server. But the
MCP server entrypoint receives only `{ name, arguments }` (`src/mcp_server/tools.rs:395`)
— **no caller `agent_id`**. And a jailed worker's `Read` passes the SL-182 wall
(`mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only`), so no on-disk secret
(nonce/capability) is safe from a sibling worker either. Therefore **true
"the caller owns `dir`" authentication is not achievable in the current harness.**

## Residual accepted by SL-198 (design §10 X1)

`worker_commit` fences the **target** instead of the caller: `dir` must be a spawned
worker worktree carrying a trusted, immutable, per-worktree base record (snapshotted at
`create-fork`, ro to the worker), and the belts (`check commit`, scope vs design-target
selectors, single-non-merge `C^==B`) bound the rest. The residual: a **poisoned worker
can target a sibling's legitimately-spawned worktree** and land an **in-scope,
prove-passing** commit onto its branch — attribution confusion, caught by review/audit,
**not** a jail escape or privilege escalation. This is consistent with the locked RFC-005
threat model ("belts bound blast radius, not intent; intent-preservation stays with
review/audit/human").

## The follow-on work (why this is a risk, not just a note)

A real caller-binding needs a **harness `agent_id` → worktree channel to the MCP server**
(the server must learn *which* subagent is calling, unforgeably). That does not exist
today. Options to investigate: a harness feature request (pass caller identity to MCP
tools); a per-spawn capability delivered only via the worker prompt AND validated against
server-process memory (not disk); or accepting the residual permanently for single-owner
repos and tightening only where adversarial workers are in scope.

Relates to SL-198, RSK-225, SL-182,
[[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]].
