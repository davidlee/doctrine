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

**WITNESSED — an `mcp__*` write tool bypasses the wall (Claude Code 2.1.198, live
probe 2026-07-04, trust: high; was previously inferred, RSK-225 gate now discharged):**
a `general-purpose` subagent called `mcp__doctrine__memory_record` (a writable MCP
tool) under **both** wall verdicts and the write succeeded both times:
- **Reject arm** (cwd = primary tree): raw Bash and Write both denied
  (`worktree-jail: cwd-not-a-worktree: /workspace/doctrine`), yet `memory_record`
  wrote a new authored entity to `/workspace/doctrine/.doctrine/memory/items/…`.
- **Jail arm** (`isolation: worktree`, cwd = linked worktree): Bash writes to the
  primary tree and shared `.git` blocked (RO filesystem), yet `memory_record`'s
  returned `canonical_path` was in the **primary** tree (files verified present in
  primary, absent in the worktree) — the write escaped the jail cwd entirely.
The MCP server resolves paths against the primary repo root, not the subagent's cwd,
and the `PreToolUse` wall never sees the call. So MCP writes are an unmediated escape
for a broadly-granted worker (RSK-225) and simultaneously the enabler of RFC-005
Mode B. Any writable MCP tool a worker holds is un-walled — pin worker `tools:` to the
single gated commit tool.

**Arm fork (load-bearing, subprocess half confirmed):** the passthrough is a
**Claude-arm** property — the MCP server (`doctrine serve --mcp`, `.mcp.json`) is a
stdio child of the **top-level harness**, not of any subagent, so it sits outside
every subagent bwrap. On the **subprocess arm** (codex/pi), a **stdio** MCP server the
worker spawns is a *child of the jailed `pi` process* → inherits the `bwrap --ro-bind
/ /` namespace (`scripts/pi-spawn-confined.sh`) → **no** passthrough (confirmed by
bwrap child-namespace inheritance; current pi workers run `--no-extensions` with no
MCP at all, so nothing to bypass there today). A persistent (http/sse) MCP server
launched **outside** the jail would unify both arms as unconfined — a deployment
departure from stdio simplicity.

Used by: the RFC-005 subagent-orchestrator design (`.doctrine/rfc/005/
subagent-orchestrator-design.md`) — Mode B routes privileged writes through unconfined
MCP tools; IMP-253 (gated `worker_commit`). Version-sensitive: re-probe on harness
upgrades. See [[mem_019f18d2a9307cc38d5e4ba9749e6208]] (confine subagents via
PreToolUse+bwrap).
