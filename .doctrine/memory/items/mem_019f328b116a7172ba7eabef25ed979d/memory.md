# isolation:worktree forks reach doctrine MCP with a clean binary — MCP-funnel viable for a workflow-spawned orchestrator

**An `isolation:'worktree'` fork CAN reach the doctrine MCP server** when
`DOCTRINE_BIN` is a real binary. Verified live 2026-07-05 (CHR-039): an isolated
fork agent's `mcp__doctrine__memory_retrieve` returned 5/335 cleanly. **There is no
isolation limit** on MCP reachability.

**Consequence (IDE-031).** A workflow-spawned confined orchestrator — itself running
in a fork — can drive the dispatch funnel via its **granted** `dispatch_*` MCP tools
(`dispatch_import`/`conclude_phase`/`reap`; the `dispatch-orchestrator` subagent type
already grants them). **MCP-funnel is viable; a CLI-only funnel is NOT forced.**

**Grant vs connection are separate layers.** A fork gets the tool *definitions*
(ToolSearch loads schemas) and, with a clean binary, a live *connection*. The
operator's "workers only see explicitly-granted MCP tools" rule governs
*visibility*, upstream-independent of connection. Grant the orchestrator/worker only
the narrow surface it needs (dispatch-worker grants just `worker_commit`).

⚠ Do NOT confuse a connection FAILURE with an isolation limit — if a fork reports
`MCP server not connected`, suspect a `DOCTRINE_BIN` shim breaking stdio first:
[[mem.fact.dispatch.mcp-stdio-shim-breaks-fork-servers]]. Related:
[[mem.fact.workflow.agent-worktree-fires-create-fork-hook]],
[[mem.fact.dispatch.confined-orchestrator-driveloop-realizable]]. Context: RFC-011;
findings `.doctrine/rfc/011/chr-039-findings.md`.
