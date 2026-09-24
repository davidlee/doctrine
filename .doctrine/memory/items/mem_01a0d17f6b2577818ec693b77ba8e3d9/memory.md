pi extension API (0.87.x):

- `tool_result` is post-execution and its result may replace `content`; handlers are awaited and compose. There is **no** pre-execution context channel (`tool_call` can only block or mutate input).
- `ToolResultEvent` = `{ type, toolCallId, toolName, input, content, isError, details }`. It carries **no agent or session identity**, so Claude's `agent_id` main-thread gate cannot be reproduced on pi.
- Usable context is on `ExtensionContext`: `ctx.cwd`, `ctx.sessionManager.getSessionId()`, `ctx.signal`.
- Built-in tool inputs: `read`/`write`/`edit` carry `path`; `bash` carries `command` — normally **relative to pi's cwd**.
- `PI_SUBAGENT_CHILD=1` is set by the **pi-subagents package** runner (`src/runs/background/subagent-runner.js`), not by pi core. Depending on it makes an extension correct only under that one spawner.
- A subprocess from a `tool_result` handler runs on the event loop: use an async spawn, and bound it with `timeout` + `signal` or a hang holds the tool result and the turn.
