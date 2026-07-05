# Background workflow worker has no live escalation channel — human-in-loop is return-value only

**A subagent running inside a background Workflow cannot reach the user live.**
Verified 2026-07-05 (CHR-039), across probes:
- **No semantic ask** — the workflow agent has no `AskUserQuestion` tool, and cannot
  invoke skills, so `/consult` is impossible.
- **No per-action approval prompt** — an un-allowlisted doctrine MCP call executed
  without surfacing any prompt (tested with the main session in `acceptEdits`, not
  just YOLO). It just runs.

**Consequence (IDE-031 design constraint).** All human-in-loop must be **return-value
at the script boundary**: a worker halts, returns a structured "needs decision X",
and the orchestrator (or the main thread between `agent()` calls) relays it to the
user; the answer feeds the next or a resumed agent. There is NO automatic escalation
— architect it explicitly. Fits doctrine's report-and-halt dispatch model.

**Safety corollary.** A fork worker CAN reach doctrine MCP
([[mem.fact.workflow.isolated-fork-reaches-doctrine-mcp]]) and has unconstrained FS
writes in its fork — so it is contained **by policy, not by MCP-unreachability**.
The only real containment is the orchestrator-sole-writer model (ADR-006), the gated
`worker_commit`, the import belt, report-and-halt, and granting workers a NARROW MCP
surface. **The correct posture: a workflow spawns a confined orchestrator (the trust
boundary), never a bare worker.** Related:
[[mem.fact.workflow.agent-worktree-fires-create-fork-hook]],
[[mem.fact.dispatch.confined-orchestrator-driveloop-realizable]]. Context: RFC-011;
findings `.doctrine/rfc/011/chr-039-findings.md`.
