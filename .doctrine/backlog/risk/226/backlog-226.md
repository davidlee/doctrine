# RSK-226: worker_commit cannot authenticate its MCP caller (no agent_id seam)

<!-- Backlog body — prose, never structurally parsed (storage rule). -->

## The risk

SL-198's `worker_commit` MCP tool commits a delta into a worktree, running in the
unconfined doctrine MCP server. But the MCP server entrypoint receives only
`{ name, arguments }` (`src/mcp_server/tools.rs:395`) — **no caller `agent_id`**. And a
jailed worker's `Read` passes the SL-182 wall
(`mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only`), so no on-disk secret
(nonce/capability) is safe from a sibling worker either. Therefore **true caller
authentication ("this caller owns this worktree") is not achievable in the current
harness.**

## Residual accepted by SL-198 (design §10 X1; owner ruling 2026-07-04)

`worker_commit` takes an **`agent` id (the worker's self-reported worktree name)**, never
a free `dir`, and **looks up** the worktree from the per-worktree registry
(`JAIL_SUBPATH/<agent>.toml`, present ⟺ legitimately spawned; `dir`, `base`, `branch` all
derive from that one key). The worker cannot name an arbitrary path — the identifier must
hit the registry — and the belts (`check commit`, scope vs design-target selectors,
single-non-merge `C^==B`) bound the rest. **Residual (accepted, small blast radius):** a
poisoned worker that knows/guesses a **sibling's registered name** can land an in-scope,
gate-passing commit onto that sibling's `dispatch/<name>` branch — attribution confusion,
caught by review/audit, **not** a jail escape or escalation; **the poisoner's own work is
not promoted** (the orchestrator imports the branch it spawned). Consistent with the
locked RFC-005 threat model ("belts bound blast radius, not intent; intent-preservation
stays with review/audit/human").

## The follow-on work (why this is a risk, not just a note)

A real caller-binding needs a **harness `agent_id` → worktree channel to the MCP server**
(the server must learn *which* subagent is calling, unforgeably). That does not exist
today. Options to investigate: a harness feature request (pass caller identity to MCP
tools); a per-spawn capability delivered only via the worker prompt AND validated against
server-process memory (not disk); or accepting the residual permanently for single-owner
repos and tightening only where adversarial workers are in scope.

Relates to SL-198, RSK-225, SL-182,
[[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]].
