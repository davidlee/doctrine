# Pi extension: doctrine MCP tools in pi

## Context

IMP-117 requests a pi extension that makes the doctrine MCP server's review
tools callable by the LLM inside a pi session.

`doctrine serve --mcp` already exposes 10 review verbs over JSON-RPC 2.0 on
stdio. The `.mcp.json` at the project root configures it. The extension spawns
the doctrine MCP server, discovers its tools, and registers them with
`pi.registerTool()` — the LLM can then call `review_raise`, `review_dispose`,
etc. directly.

This is **not** a generic MCP client. It is purpose-built for the doctrine
MCP server, configured via `.mcp.json`.

## Scope & Objectives

1. **A pi extension** (TypeScript, single-file, project-local at
   `.pi/extensions/doctrine-mcp.ts`) that on `session_start`:
   - Reads the `doctrine` entry from `.mcp.json`
   - Spawns `doctrine serve --mcp`, negotiates the MCP handshake
     (`initialize` → `notifications/initialized`), discovers tools via
     `tools/list`
   - Registers each discovered tool with `pi.registerTool()`, namespaced
     under the server key: `doctrine_review_raise`, `doctrine_review_show`,
     etc.
   - Manages server lifecycle: spawn once, keep alive for the session,
     teardown on `session_shutdown`

2. **Tool execution** — each registered tool, on call:
   - Translates pi tool params into a `tools/call` JSON-RPC request
   - Sends it to the doctrine MCP process over stdio
   - Returns the MCP `content` array as the tool result

3. **Error handling** — transport failures, server crashes, and MCP error
   responses map to thrown errors so the LLM sees the failure.

4. **Tested** — integration test: the 10 doctrine review tools register and
   are callable.

## Non-Goals

- Generic MCP client (arbitrary `.mcp.json` servers)
- MCP transport other than stdio
- Server process management beyond start/stop (no health checks, no restart)
- JSON Schema → TypeBox translation (pass as description; accept `Type.Any()`)
- Hot-reload when `.mcp.json` changes mid-session
- State persistence across sessions

## Risks

- **Process lifecycle**: if the MCP server hangs, the extension must not block
  pi. Mitigation: per-call timeout; abort via `ctx.signal`.
- **Schema mismatch**: MCP uses JSON Schema; pi uses TypeBox. Mitigation: pass
  JSON Schema as description text; accept loose params (`Type.Any()`).
- **Startup overhead**: spawning the server on every `session_start`. Doctrine
  starts fast; acceptable.
- **Project trust**: `.pi/extensions/` loads only after trust — natural gating.

## Assumptions

- Node.js `child_process` stdio works in the bubblewrap jail
- The doctrine binary at `/home/david/.cargo/bin/doctrine` is available
- MCP protocol version 2024-11-05

## Open Questions

- OQ-1: Should tool names be `doctrine_<tool>` or bare `review_*`?
  Default: `doctrine_<tool>` for clarity; revisit in design.

## Summary

A single-purpose pi extension that bridges the doctrine MCP server's 10 review
tools into pi, so the LLM can directly call doctrine's review verbs.

## Closure

Verify: `.pi/extensions/doctrine-mcp.ts` exists and passes lint; `tools/list`
returns 10 tools; calling `doctrine_review_list` through the bridge returns
valid review data. All 10 tools registered and callable.
