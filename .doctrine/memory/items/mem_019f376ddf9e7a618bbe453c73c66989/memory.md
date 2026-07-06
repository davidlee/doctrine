# Armed workflow dispatch-worker leaf retains worker_commit and RESOLVES (mode-B viable)

A Claude Code **Workflow** `agent({agentType:'dispatch-worker', isolation:'worktree'})`
leaf — spawned with the session cwd parked at the coord tree's arming dir
(`dispatch arm-spawn --base <sha>` written) — **retains the `worker_commit` MCP
tool** and its armed fork **resolves to a provisioned `DispatchRecord`**. So the
**(B) worker self-commit** path is viable from the unjail Workflow arm, not just
the shipped Agent-tool claude arm. This closes design SL-206 §5 **OQ-3 / P5**
(the one open gate on (B): "does a workflow leaf retain the MCP tool?").

## Proof (SL-206 PHASE-08 P5 probe, 2026-07-06, Claude Code / Opus 4.8)

Workflow `wf_9829cbac-a41`, one `dispatch-worker` leaf armed at `base=ff8c61db`:

- `has_worker_commit_tool = **true**` — the leaf introspected its own toolset and
  `mcp__doctrine__worker_commit` was present (MCP tools survive the workflow strip;
  only `Agent`/`Grep`/`Glob` are removed — cf. [[mem_019f36028bca7411b33fde4981aaba85]] row 2).
- `worker_commit(...)` returned **`Refused reason=commit-gate-red`**, NOT
  `unknown-agent`. Reaching the `check commit` gate proves the server **resolved**
  the opaque `agent` (`wf_9829cbac-a41-1`) to a provisioned record and ran the belts.
  The RED was an ambient stale-baked-binary artifact
  ([[mem_019f376e2f6b7571af71290f8ea994c2]]), not a P5 capability gap; the clean-tip
  witness (non-null `fork_tip`) is deferred to SL-206 PHASE-16's green e2e.

## Why the contrast with the unknown-agent wall

[[mem_019f361983617d529a054e52617c5f55]] recorded `worker_commit → unknown-agent`
for a workflow leaf — but that probe (`wf_01ed2bf2-ff8`) was a **benign/unarmed**
`agent(isolation:worktree)` minted under `.worktrees/` (cwd ≠ arming dir ⇒
`create-fork` makes a detached tree at HEAD, **unmarked, unprovisioned**). The
**armed** dispatch-worker fork (cwd == arming dir ⇒ fork off the arming `base`,
**worker-marked + provisioned**) is exactly what makes `worker_commit` resolve. So
the wall is unarmed-fork-specific, not a workflow-arm property.

## For /drive-slice (SL-206 PHASE-14)

The driver authors **(B) target + (A) fallback** (design D9). (B) is now
de-risked: the jailed worker leaf self-commits via the server-side `worker_commit`
(unconfined MCP process has RW `.git`; the worker never touches `.git`). Fall to
(A) — disposing-O imports the uncommitted working-tree diff — only if a future
harness strips MCP from leaves.

Relates: [[mem_019f331005d776c1a65c65bfe59581bf]] (armed fork mints at base),
[[mem_019f361983617d529a054e52617c5f55]] (unarmed unknown-agent, the contrast),
[[mem_019f36028bca7411b33fde4981aaba85]] (workflow strips Agent, keeps MCP).
