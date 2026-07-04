# SL-182 PreToolUse jail only walls Bash/Edit/Write; other tools pass through

The dispatch confinement wall (`src/worktree/pretooluse.rs` `decide()`) matches
`tool_name`: `Bash` → nested-bwrap wrap, `Edit|Write` → `realpath ⊆ cwd` pathcheck,
**everything else → `PassThrough`**. So the jail governs only those three tools.

**Witnessed (Claude Code 2.1.198, live probe — trust: high):**
- `Agent` and `Read` tools **pass through even under the `Reject` verdict** — a
  subagent whose cwd is the primary tree (`agent_id` present + not a linked worktree
  → `Reject`) has Bash+Write denied (`worktree-jail: cwd-not-a-worktree`) yet still
  **spawns nested subagents and reads files**.
- **Nested subagents work to ≥ depth-3** (harness cap is depth-5; a subagent gets the
  `Agent` tool iff its def lists it — `dispatch-worker` omits it deliberately).
- `SubagentStart` payload carries `agent_type` + `agent_id` + `cwd` (but
  `parent_agent_id`/`depth` are null); **`PreToolUse` payload carries `agent_id` +
  `cwd` but NOT `agent_type`** — so the wall cannot tell an orchestrator subagent from
  a worker at deny-time. This is why role must be resolved at `SubagentStart`.
- `resolve_target` (`jail.rs`): no `agent_id` → `Orchestrator` (unconfined, main
  thread); `agent_id` + linked-wt cwd → `Jail(cwd)` (rw only cwd; shared `.git` RO);
  `agent_id` + non-wt cwd → `Reject`.

**INFERRED, NOT YET WITNESSED (trust: low — needs a probe before any design leans on
it):** that an `mcp__*` **write** tool bypasses the wall the same way `Agent`/`Read`
do. It follows from `decide()`'s `PassThrough` branch, but no probe has driven an MCP
*write* through `Jail`/`Reject`. See RSK on MCP worker permissions.

**Arm fork (load-bearing):** the passthrough is a **Claude-arm** property — the MCP
server is a separate process reached via the harness, unconfined relative to the
subagent's bwrap. On the **subprocess arm** (codex/pi), a **stdio** MCP server the
worker spawns is a *child of the jailed process* → inherits the jail (no passthrough).
Persistent (http/sse) MCP could unify both arms as unconfined, but that is a
deployment departure from stdio simplicity.

Used by: the RFC-005 subagent-orchestrator design (`.doctrine/rfc/005/
subagent-orchestrator-design.md`) — Mode B routes privileged writes through unconfined
MCP tools; IMP-253 (gated `worker_commit`). Version-sensitive: re-probe on harness
upgrades. See [[mem_019f18d2a9307cc38d5e4ba9749e6208]] (confine subagents via
PreToolUse+bwrap).
