# IMP-117: Pi extension: bridge .mcp.json MCP servers as tools

Pi has no native MCP client. The project already runs a doctrine MCP server
(`.mcp.json`: `doctrine serve --mcp`). A pi extension that reads `.mcp.json`,
connects to listed servers, discovers their tools, and registers them via
`pi.registerTool()` would let subagents (and interactive sessions) use doctrine
verbs (`review new`, `review raise`, `review dispose`, etc.) without the
orchestrator hand-rolling raw files.

This is the missing piece that caused RV-102 to be created by direct file
write instead of through the review lifecycle CLI.

## Sketch

- read `.mcp.json` (cwd-walk upward like `AGENTS.md`)
- connect to each `mcpServers` entry via `@modelcontextprotocol/sdk`
- `tools/list` → `pi.registerTool()` mapping JSON Schema → TypeBox
- `tools/call` → forward to MCP server, return result
- handle lifetime: connect on `session_start`, disconnect on `session_shutdown`

## Riders

- MCP transport: stdio or streamable HTTP? Doctrine's `serve --mcp` uses stdio.
- Tool name disambiguation: prefix with server name (`doctrine__review_new`).
- Startup delay: MCP server launch + tool negotiation adds latency on session start.
