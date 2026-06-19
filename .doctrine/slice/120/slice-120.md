# Pi extension: bridge .mcp.json MCP servers as tools

## Context

IMP-117 requests a pi extension that reads `.mcp.json` — the standard MCP
server configuration file — and bridges each configured server's tools into
pi as callable tools the LLM can use.

A working MCP server already exists in this repo: `doctrine serve --mcp`
exposes 10 review verbs over JSON-RPC 2.0 on stdio. The `.mcp.json` at the
project root configures it. The bridge extension will make those tools
available inside a pi session without manual wiring — the LLM can call
`review_raise`, `review_dispose`, etc. directly.

The extension is generic: it reads any `.mcp.json`, spawns each configured
MCP server process, discovers its tool list via `tools/list`, and registers
each tool with pi's `registerTool()`. When the LLM calls one, the extension
translates it into a `tools/call` JSON-RPC request and returns the result.

## Scope & Objectives

1. **A pi extension** (TypeScript, single-file, project-local at
   `.pi/extensions/mcp-bridge.ts`) that on `session_start`:
   - Reads `.mcp.json` from the project root
   - For each entry in `mcpServers`: spawns the process, negotiates MCP
     handshake (`initialize` → `notifications/initialized`), discovers tools
     via `tools/list`
   - Registers each discovered tool with `pi.registerTool()`, namespaced
     under the server name: `mcp_<server>_<tool>` (e.g. `mcp_doctrine_review_raise`)
   - Manages server lifecycle: spawn once, keep alive for the session, teardown
     on `session_shutdown`

2. **Tool execution** — each registered tool, on call:
   - Translates pi tool params into a `tools/call` JSON-RPC request
   - Sends it to the MCP server process
   - Returns the MCP `content` array as the tool result

3. **Error handling** — transport failures, server crashes, and MCP-level
   errors map to thrown errors so the LLM sees the failure.

4. **Tested** — unit tests for `.mcp.json` parsing, namespace generation.
   Integration test with the doctrine MCP server (10 tools registered and
   callable).

## Non-Goals

- MCP transport other than stdio (no SSE, no WebSocket)
- Server process management beyond start/stop (no health checks, no auto-restart)
- Tool parameter schema translation beyond pass-through (MCP JSON Schema →
  TypeBox is lossy; pass the raw JSON Schema in tool description)
- CLI flags or configuration beyond `.mcp.json`
- Hot-reload when `.mcp.json` changes mid-session
- Overriding built-in pi tools
- State persistence across sessions (servers restart each session)

## Risks

- **Process lifecycle**: MCP servers are long-lived subprocesses. If a server
  hangs, the extension must not block pi. Mitigation: per-call timeout; abort
  via `ctx.signal`.
- **Schema mismatch**: MCP tools use JSON Schema; pi uses TypeBox. Direct
  mapping is lossy. Mitigation: pass JSON Schema as description text so the LLM
  can see constraints; accept loose params (`Type.Any()`).
- **Startup overhead**: spawning and initializing servers on every
  `session_start` adds latency. Acceptable for now (doctrine MCP starts fast);
  revisit if needed.
- **Project trust**: the extension runs in a trusted project context only (it
  spawns arbitrary processes from `.mcp.json`). Pi's trust gating handles this
  naturally — `.pi/extensions/` loads only after trust.

## Assumptions

- Node.js child_process stdio piped communication works in the bubblewrap jail
- The doctrine MCP server binary is available at the path in `.mcp.json`
  (`/home/david/.cargo/bin/doctrine`)
- Standard MCP 2024-11-05 protocol version

## Open Questions

- OQ-1: Should tool names use `mcp_<server>_<tool>` or a shorter namespace?
  Default: `mcp_<server>_<tool>` for clarity; revisit in design.

## Summary

A TypeScript pi extension that reads `.mcp.json`, spawns each configured MCP
server, discovers its tools, and registers them as pi tools — turning the
doctrine MCP server (and any future MCP server) into LLM-callable tools inside
a pi session.

## Closure

Verify: `.pi/extensions/mcp-bridge.ts` exists and passes lint; `tools/list`
responds with the same tools the MCP server advertises; calling a tool through
the bridge returns the same result as a direct MCP call. The 10 doctrine review
tools are registered and callable in a pi session.
