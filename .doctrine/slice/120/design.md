# SL-120 Design: Doctrine MCP tools in pi

## Current behaviour

Pi sessions in the doctrine project have no access to doctrine's review tools.
The LLM cannot call `review_raise`, `review_dispose`, etc. — it must simulate
them with file reads and writes, or the human runs `doctrine review` manually.

`doctrine serve --mcp` already exposes 10 review verbs over JSON-RPC 2.0 on
stdio. It is configured in `.mcp.json` for external consumers, but pi has no
bridge to it.

## Target behaviour

A pi extension registers the 10 doctrine review tools so the LLM can call them
directly: `doctrine_review_raise`, `doctrine_review_show`, etc. The extension
spawns `doctrine serve --mcp` once per session, negotiates the MCP handshake,
discovers tools via `tools/list`, and registers each as a pi tool with
`pi.registerTool()`.

---

## Architecture

```
pi session
  │
  ├─ session_start ─► spawn doctrine serve --mcp
  │                    ├─ initialize → server_info, capabilities
  │                    ├─ tools/list → 10 tool defs
  │                    └─ registerTool("doctrine_review_raise", ...) × 10
  │
  ├─ tool_call: doctrine_review_raise
  │   └─ JSON-RPC tools/call {name:"review_raise", arguments:{...}}
  │       └─ stdin → doctrine process → stdout
  │           └─ MCP content[] → tool result
  │
  └─ session_shutdown ─► process.kill()
```

Single file: `.pi/extensions/doctrine/mcp.ts`. Sibling to the `index.ts` that
SL-119 generates (boot refresh). The doctrine binary path is baked at install
time — same mechanism as SL-119's `plan_pi_extension` / `install_pi_extension`,
extended to write a second file.

No npm dependencies beyond the pi SDK (`@earendil-works/pi-coding-agent`,
`typebox`). Uses Node.js built-ins: `child_process`, `readline`.

---

## Design decisions

| Decision | Rationale |
|---|---|
| Pass-through params (`Type.Any()`) | No JSON Schema → TypeBox mapping. JSON Schema pasted into tool description. LLM constructs args from description. |
| `doctrine_<tool>` naming | Clear provenance, zero collision risk, matches `.mcp.json` key. |
| Spawn once per session | MCP is designed for persistent sessions. Avoids handshake per call. |
| Binary path baked at install | Same `current_exe()` pattern as SL-119. No runtime path discovery. |
| Stderr ring buffer | Piped, consumed silently. Appended to error messages only on failure. |
| Sequential request ID | LLM calls tools one at a time per turn. Simple monotonic counter. |
| No auto-restart | If process dies, next call errors. Session restart re-spawns. |

---

## Key types

```typescript
// MCP JSON-RPC (subset)
type JsonRpcRequest = {
  jsonrpc: "2.0";
  id: number;
  method: string;
  params?: unknown;
};

type JsonRpcResponse = {
  jsonrpc: "2.0";
  id: number;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
};

type McpTool = {
  name: string;
  description: string;
  inputSchema: unknown;
};

type ToolsListResult = { tools: McpTool[] };
type McpContent = { type: "text"; text: string };
type McpToolCallResult = { content: McpContent[]; isError?: boolean };
```

## Pure functions

```typescript
// Build a JSON-RPC request line
function buildRequest(id: number, method: string, params?: unknown): string

// Parse one JSON-RPC response line
function parseResponse(line: string): JsonRpcResponse

// "review_raise" → "doctrine_review_raise"
function toolPiName(mcpName: string): string

// "doctrine_review_raise" → "review_raise"
function stripPiPrefix(piName: string): string
```

## Tool registration

Each MCP tool becomes:

```typescript
pi.registerTool({
  name: `doctrine_${tool.name}`,
  label: `Doctrine ${tool.name}`,
  description: `${tool.description}\n\nParameters (JSON Schema):\n${JSON.stringify(tool.inputSchema, null, 2)}`,
  parameters: Type.Object({}, { additionalProperties: true }),
  async execute(_toolCallId, params, signal, _onUpdate, _ctx) {
    const request = buildRequest(nextId(), "tools/call", {
      name: tool.name,
      arguments: params,
    });
    // write to process.stdin, read from process.stdout...
    const response = parseResponse(line);
    if (response.error) throw new Error(formatError(response.error));
    return response.result;
  },
});
```

---

## Process lifecycle

```typescript
let proc: ChildProcess | null = null;
let stderrBuf: string[] = [];
let requestId = 0;

function spawnDoctrine(binPath: string, cwd: string): ChildProcess {
  return spawn(binPath, ["serve", "--mcp", "--path", cwd], {
    stdio: ["pipe", "pipe", "pipe"],
  });
}

async function initialize(proc: ChildProcess, signal?: AbortSignal): Promise<void> {
  // 1. Send initialize
  // 2. Read response, verify capabilities.tools exists
  // 3. Send notifications/initialized
}

async function discoverTools(proc: ChildProcess, signal?: AbortSignal): Promise<McpTool[]> {
  // Send tools/list, parse response, return tools array
}

async function callTool(
  proc: ChildProcess,
  toolName: string,
  args: unknown,
  signal?: AbortSignal,
): Promise<McpToolCallResult> {
  const id = ++requestId;
  const request = JSON.stringify({
    jsonrpc: "2.0", id,
    method: "tools/call",
    params: { name: toolName, arguments: args },
  });
  proc.stdin!.write(request + "\n");
  // Await one JSON line from stdout (with signal/timeout)
  const line = await readOneLine(proc.stdout!, signal);
  const response: JsonRpcResponse = JSON.parse(line);
  if (response.error) throw new Error(`MCP error ${response.error.code}: ${response.error.message}`);
  return response.result as McpToolCallResult;
}

async function readOneLine(
  stream: Readable,
  signal?: AbortSignal,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const rl = readline.createInterface({ input: stream });
    const onAbort = () => { rl.close(); reject(new Error("aborted")); };
    signal?.addEventListener("abort", onAbort, { once: true });
    rl.once("line", (line) => {
      signal?.removeEventListener("abort", onAbort);
      rl.close();
      resolve(line);
    });
  });
}
```

---

## session_start handler

```typescript
pi.on("session_start", async (_event, ctx) => {
  try {
    proc = spawnDoctrine(BIN_PATH, ctx.cwd);  // baked path + explicit --path
    await withTimeout(initialize(proc), 2000);
    const tools = await withTimeout(discoverTools(proc), 5000);

    for (const tool of tools) {
      pi.registerTool({
        name: `doctrine_${tool.name}`,
        label: `Doctrine ${tool.name}`,
        description: `${tool.description}\n\nParameters (JSON Schema):\n${JSON.stringify(tool.inputSchema, null, 2)}`,
        parameters: Type.Object({}, { additionalProperties: true }),
        async execute(_toolCallId, params, signal) {
          const result = await withTimeout(
            callTool(proc!, tool.name, params, signal),
            30_000,
          );
          const text = result.content.map((c) => c.text).join("\n");
          return { content: [{ type: "text", text }], details: {} };
        },
      });
    }
  } catch (err) {
    // Tools won't be registered — session continues without them
    ctx.ui?.notify?.(`doctrine-mcp: ${(err as Error).message}`, "warning");
  }
});
```

---

## session_shutdown handler

```typescript
pi.on("session_shutdown", async () => {
  if (proc && !proc.killed) {
    proc.kill();
    proc = null;
  }
});
```

---

## Error handling

| Scenario | Handling |
|---|---|
| Binary missing / spawn fails | `session_start` catches, notifies, no tools registered |
| MCP handshake timeout (2s) | Throw; session continues without tools |
| `tools/list` timeout (5s) | Throw; session continues without tools |
| `tools/list` returns empty | No tools registered — no-op |
| Tool call timeout (30s) | `AbortSignal` via `ctx.signal`; error to LLM |
| MCP returns error response | Thrown as `Error` with code + message + stderr tail |
| Process crashes mid-session | Next tool call gets EPIPE → error with stderr buffer |
| Concurrent tool calls | N/A — pi calls tools sequentially per turn |

### Stderr strategy

```typescript
// On stderr data:
proc.stderr!.on("data", (chunk: Buffer) => {
  stderrBuf.push(chunk.toString());
  if (stderrBuf.length > 100) stderrBuf.shift(); // ring buffer, ~100 lines
});

// On error:
function stderrTail(): string {
  return stderrBuf.length ? "\nstderr:\n" + stderrBuf.join("") : "";
}
```

---

## Code impact summary

| Path | Change |
|---|---|
| `.pi/extensions/doctrine/mcp.ts` | New file — the extension |
| `src/boot.rs` | Extended `install_refresh` pi arm to also generate `mcp.ts` |
| `tests/mcp-bridge.test.ts` | New — unit + integration tests |

The `src/boot.rs` change mirrors SL-119's extension generation pattern:
`plan_mcp_extension(root, exec) -> ExtAction` and
`install_mcp_extension(root, exec, dry_run) -> ExtOutcome`. The generated file
bakes `current_exe()` as `BIN_PATH`.

---

## Verification

| What | How |
|---|---|
| Extension loads, 10 tools registered | `pi --list-tools` includes `doctrine_review_*` |
| Tool naming follows convention | `doctrine_review_new`, `doctrine_review_list`, etc. |
| `doctrine_review_list {}` returns rows | Integration: spawn doctrine, call tool, parse response |
| `doctrine_review_show {reference:"1"}` works | Params pass through correctly |
| Bad params → MCP error surfaced | Missing required field → error with MCP code |
| Server crash mid-session | Kill process, next call → error with stderr |
| Handshake timeout (2s) | Non-MCP binary → timeout, tools absent, session ok |
| `session_shutdown` cleanup | Process killed, no zombie |
| Idempotent re-registration | Second `session_start` no-ops |
| Pure functions | Unit tests via `vitest` |
| Integration | `vitest` spawns real `doctrine serve --mcp` |
| Extension file installed by `doctrine boot install` | Integration: file present, contains baked path |

---

## Risks

- **Runtime compatibility**: extension runs under Node.js/jiti (pi runtime);
  tests run under vitest (Node.js). APIs identical — no cross-runtime gap.
- **Process lifecycle**: if pi reloads extensions mid-session (`/reload`), the
  old `session_shutdown` kills the process, new `session_start` re-spawns.
  Expected and correct.
- **`ctx.ui.notify` availability**: RPC mode (`ctx.mode === "rpc"`) supports
  `notify`. Print mode (`-p`) does not — guard with `ctx.hasUI`.

## Dependencies on SL-119

SL-119 generates `.pi/extensions/doctrine/index.ts` with baked `BIN_PATH`.
SL-120 generates `.pi/extensions/doctrine/mcp.ts` — same directory, same
`install_refresh` call site, same `current_exe()` pattern. The two
extensions are independent files; either can exist without the other.

## Assumptions

- `doctrine serve --mcp` binary is the same `current_exe()` used by
  `doctrine boot install` (standard install)
- `child_process.spawn` stdio pipes work in the bubblewrap jail
- MCP protocol version 2024-11-05

## Self-review findings

All resolved — integrated into design.

| # | Severity | Finding | Disposition |
|---|----------|---------|-------------|
| F-1 | minor | `Type.Any()` existence unconfirmed in pi's bundled typebox | Fixed — use `Type.Object({}, { additionalProperties: true })` instead |
| F-2 | minor | `doctrine serve --mcp` should get explicit `--path` | Fixed — spawn with `--path <cwd>` |
