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
  ├─ session_start ─► spawn doctrine serve --mcp --path <cwd>
  │                    ├─ proc.on('error') / proc.on('exit') listeners
  │                    ├─ persistent line-reader on stdout
  │                    ├─ initialize → server_info, capabilities
  │                    ├─ tools/list → 10 tool defs
  │                    └─ registerTool("doctrine_review_raise", ...) × 10
  │
  ├─ tool_call: doctrine_review_raise
  │   └─ buildRequest(id, "tools/call", {name:"review_raise", arguments:{...}})
  │       └─ write to stdin, await response by id from persistent reader
  │           └─ validate response.id === id, check response.error
  │               └─ filter text content, return result
  │
  └─ session_shutdown ─► graceful shutdown → exit notification → kill fallback
```

Single file: `.pi/extensions/doctrine/mcp.ts`. Sibling to the `index.ts` that
SL-119 generates (boot refresh). The doctrine binary path (`BIN_PATH`) is baked
at install time via `current_exe()` — same mechanism as SL-119's
`plan_pi_extension` / `install_pi_extension`, extended to write a second file.

**Scope note:** The scope doc describes reading `.mcp.json`. The design diverges
deliberately — baking the binary path at install time is simpler, more reliable
in the bubblewrap jail, and avoids JSON parsing in TypeScript. The binary path
and `--mcp` args are known constants; `.mcp.json` is an authored config for
external consumers, not a runtime input for this extension.

No npm dependencies beyond the pi SDK (`@earendil-works/pi-coding-agent`,
`typebox`). Uses Node.js built-ins: `child_process`, `readline`.

---

## Design decisions

| Decision | Rationale |
|---|---|
| Pass-through params (`Type.Object({}, {additionalProperties: true})`) | No JSON Schema → TypeBox mapping. JSON Schema pasted into tool description. LLM constructs args from description. |
| `doctrine_<tool>` naming | Clear provenance, zero collision risk. |
| Spawn once per session | MCP is designed for persistent sessions. Avoids handshake per call. |
| Persistent stdout reader | One `readline` interface for the process lifetime. Lines dispatched by `id`; notifications ignored. Avoids buffer leakage from per-call interfaces (F-6). |
| Binary path baked at install (`BIN_PATH`) | Same `current_exe()` pattern as SL-119. No runtime path discovery. |
| `BIN_PATH` duplicated (not shared with `index.ts`) | Deliberate: both files are independent — either can be installed without the other. Accepted redundancy over hidden coupling. |
| Graceful MCP shutdown | Send `shutdown` request, await response (2s timeout), send `notifications/exit`, then `proc.kill()` fallback. Respects MCP lifecycle spec. |
| Stderr byte-bounded ring buffer | Piped, consumed silently. Last 16KB appended to error messages on failure. |
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
  id?: number;            // absent on notifications
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
};

type McpTool = {
  name: string;
  description: string;
  inputSchema: unknown;
};

type ToolsListResult = { tools: McpTool[] };
type McpContent = { type: string; text?: string; data?: string };
type McpToolCallResult = { content: McpContent[]; isError?: boolean };

class TimeoutError extends Error {
  constructor(operation: string, ms: number) {
    super(`${operation} timed out after ${ms}ms`);
    this.name = "TimeoutError";
  }
}

class ProcessDeadError extends Error {
  constructor(exitCode: number | null, stderr: string) {
    super(`doctrine process exited with code ${exitCode}${stderr ? "\nstderr:\n" + stderr : ""}`);
    this.name = "ProcessDeadError";
  }
}
```

## Pure functions

```typescript
/** Build a JSON-RPC request line (compact JSON + "\n"). */
function buildRequest(id: number, method: string, params?: unknown): string {
  return JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n";
}

/** Parse one JSON-RPC response line. Throws on malformed JSON. */
function parseResponse(line: string): JsonRpcResponse {
  return JSON.parse(line) as JsonRpcResponse;
}

/** "review_raise" → "doctrine_review_raise" */
function toolPiName(mcpName: string): string {
  return `doctrine_${mcpName}`;
}

/** "doctrine_review_raise" → "review_raise" */
function stripPiPrefix(piName: string): string {
  return piName.startsWith("doctrine_") ? piName.slice("doctrine_".length) : piName;
}

/** Extract text from MCP content items, filtering non-text. */
function extractText(content: McpContent[]): string {
  return content
    .filter((c): c is { type: "text"; text: string } => c.type === "text" && typeof c.text === "string")
    .map((c) => c.text)
    .join("\n");
}

/** Format an MCP error response for the LLM. */
function formatMcpError(err: { code: number; message: string; data?: unknown }): string {
  const base = `MCP error ${err.code}: ${err.message}`;
  return err.data ? `${base}\ndata: ${JSON.stringify(err.data)}` : base;
}

/** Race a promise against a timeout. */
function withTimeout<T>(promise: Promise<T>, ms: number, label?: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new TimeoutError(label ?? "operation", ms)), ms);
    promise.then(
      (v) => { clearTimeout(timer); resolve(v); },
      (e) => { clearTimeout(timer); reject(e); },
    );
  });
}
```

## Process lifecycle

```typescript
let proc: ChildProcess | null = null;
let procDead = false;
let procExitCode: number | null = null;
let stderrBytes = "";
let requestId = 0;
let started = false;

// Pending response handlers keyed by request id.
const pending = new Map<number, {
  resolve: (r: JsonRpcResponse) => void;
  reject: (e: Error) => void;
}>();

const MAX_STDERR_BYTES = 16384;

function spawnDoctrine(binPath: string, cwd: string): ChildProcess {
  const p = spawn(binPath, ["serve", "--mcp", "--path", cwd], {
    stdio: ["pipe", "pipe", "pipe"],
  });

  p.on("error", (err) => {
    procDead = true;
    // Reject all pending ops
    const deadErr = new ProcessDeadError(null, stderrBytes);
    for (const [, { reject }] of pending) reject(deadErr);
    pending.clear();
  });

  p.on("exit", (code) => {
    procDead = true;
    procExitCode = code;
    const deadErr = new ProcessDeadError(code, stderrBytes);
    for (const [, { reject }] of pending) reject(deadErr);
    pending.clear();
  });

  // Persistent stdout line reader — one readline for process lifetime
  const rl = readline.createInterface({ input: p.stdout! });
  rl.on("line", (line: string) => {
    if (!line.trim()) return;
    let resp: JsonRpcResponse;
    try {
      resp = parseResponse(line);
    } catch {
      return; // malformed line, skip
    }
    if (resp.id === undefined) {
      // Notification — ignore (MCP allows server→client notifications)
      return;
    }
    const handler = pending.get(resp.id);
    if (handler) {
      pending.delete(resp.id);
      handler.resolve(resp);
    }
  });

  // Stderr byte-bounded ring buffer
  p.stderr!.on("data", (chunk: Buffer) => {
    stderrBytes = (stderrBytes + chunk.toString()).slice(-MAX_STDERR_BYTES);
  });

  return p;
}

function assertProcAlive(): ChildProcess {
  if (!proc || procDead) {
    throw new ProcessDeadError(procExitCode, stderrBytes);
  }
  return proc;
}

async function sendRequest(
  method: string,
  params?: unknown,
  signal?: AbortSignal,
): Promise<JsonRpcResponse> {
  const p = assertProcAlive();
  const id = ++requestId;
  const request = buildRequest(id, method, params);

  return new Promise<JsonRpcResponse>((resolve, reject) => {
    const onAbort = () => {
      pending.delete(id);
      reject(new Error("aborted"));
    };
    signal?.addEventListener("abort", onAbort, { once: true });

    pending.set(id, {
      resolve: (resp: JsonRpcResponse) => {
        signal?.removeEventListener("abort", onAbort);
        if (resp.error) reject(new Error(formatMcpError(resp.error)));
        else resolve(resp);
      },
      reject: (err: Error) => {
        signal?.removeEventListener("abort", onAbort);
        reject(err);
      },
    });

    p.stdin!.write(request, (err) => {
      if (err) {
        pending.delete(id);
        reject(new Error(`write failed: ${err.message}`));
      }
    });
  });
}

// ── Handshake ──

async function initialize(signal?: AbortSignal): Promise<void> {
  // 1. Send initialize request
  const initResp = await withTimeout(
    sendRequest("initialize", {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo: { name: "doctrine-pi-mcp", version: "1.0.0" },
    }, signal),
    2000, "initialize",
  );
  // 2. Verify server capabilities
  const result = initResp.result as { capabilities?: { tools?: unknown } } | undefined;
  if (!result?.capabilities?.tools) {
    throw new Error("MCP server does not advertise tools capability");
  }
  // 3. Send initialized notification (fire-and-forget, no response expected)
  assertProcAlive().stdin!.write(
    buildRequest(0, "notifications/initialized"),
  );
}

// ── Tool discovery ──

async function discoverTools(signal?: AbortSignal): Promise<McpTool[]> {
  const resp = await withTimeout(
    sendRequest("tools/list", undefined, signal),
    5000, "tools/list",
  );
  const result = resp.result as ToolsListResult | undefined;
  return result?.tools ?? [];
}

// ── Tool call ──

async function callTool(
  toolName: string,
  args: unknown,
  signal?: AbortSignal,
): Promise<McpToolCallResult> {
  const resp = await withTimeout(
    sendRequest("tools/call", { name: toolName, arguments: args }, signal),
    30_000, `tools/call ${toolName}`,
  );
  return resp.result as McpToolCallResult;
}

// ── Graceful shutdown ──

async function shutdown(proc: ChildProcess, timeoutMs = 2000): Promise<void> {
  if (procDead) return;
  try {
    await withTimeout(sendRequest("shutdown"), timeoutMs, "shutdown");
  } catch {
    // timeout — force kill below
  }
  // Send exit notification (fire-and-forget)
  try {
    proc.stdin!.write(buildRequest(0, "notifications/exit"));
  } catch {
    // pipe already closed
  }
  if (!proc.killed) proc.kill();
}
```

---

## Tool registration

Each MCP tool becomes:

```typescript
pi.registerTool({
  name: toolPiName(tool.name),
  label: `Doctrine ${tool.name}`,
  description: `${tool.description}\n\nParameters (JSON Schema):\n${JSON.stringify(tool.inputSchema, null, 2)}`,
  parameters: Type.Object({}, { additionalProperties: true }),
  async execute(_toolCallId, params, signal, _onUpdate, _ctx) {
    const result = await callTool(tool.name, params, signal);
    return {
      content: [{ type: "text", text: extractText(result.content) }],
      details: {},
    };
  },
});
```

---

## session_start handler

```typescript
pi.on("session_start", async (_event, ctx) => {
  // Guard idempotency — session_start may fire more than once
  if (started) return;
  started = true;

  try {
    proc = spawnDoctrine(BIN_PATH, ctx.cwd);  // baked path + explicit --path
    await initialize();
    const tools = await discoverTools();

    for (const tool of tools) {
      pi.registerTool({
        name: toolPiName(tool.name),
        label: `Doctrine ${tool.name}`,
        description: `${tool.description}\n\nParameters (JSON Schema):\n${JSON.stringify(tool.inputSchema, null, 2)}`,
        parameters: Type.Object({}, { additionalProperties: true }),
        async execute(_toolCallId, params, signal) {
          const result = await callTool(tool.name, params, signal);
          return {
            content: [{ type: "text", text: extractText(result.content) }],
            details: {},
          };
        },
      });
    }
  } catch (err) {
    // Tools won't be registered — session continues without them
    if (ctx.hasUI) {
      ctx.ui.notify(`doctrine-mcp: ${(err as Error).message}`, "warning");
    }
  }
});
```

---

## session_shutdown handler

```typescript
pi.on("session_shutdown", async () => {
  if (proc) {
    await shutdown(proc);
    proc = null;
    procDead = false;
    started = false;
  }
});
```

---

## Error handling

| Scenario | Handling |
|---|---|
| Binary missing / spawn fails | `proc.on("error")` fires → rejects pending ops; `session_start` catch notifies |
| MCP handshake timeout (2s) | `withTimeout` rejects → catch notifies, no tools |
| `tools/list` timeout (5s) | Same; no tools registered |
| `tools/list` returns empty | No tools registered — no-op |
| Tool call timeout (30s) | `withTimeout` rejects → `TimeoutError` to LLM |
| MCP returns error response | `pending.set` handler rejects → `formatMcpError` message to LLM |
| Process crashes mid-session | `proc.on("exit")` fires → `ProcessDeadError` with exit code + stderr to pending ops |
| Response-id mismatch / notification | Persistent reader checks `resp.id`; notification (`id === undefined`) silently ignored; unmatched id drops (no handler to resolve) |
| Write to dead pipe | `stdin.write` callback returns error → reject |
| Concurrent tool calls | N/A — pi calls tools sequentially per turn |
| `session_start` fires twice | `started` guard → second invocation no-ops |
| `/reload` mid-session | Old `session_shutdown` graceful-shutdowns process; new `session_start` re-spawns |

---

## Stderr strategy

```typescript
const MAX_STDERR_BYTES = 16384;
let stderrBytes = "";

proc.stderr!.on("data", (chunk: Buffer) => {
  stderrBytes = (stderrBytes + chunk.toString()).slice(-MAX_STDERR_BYTES);
});
```

Truncated to last 16KB — large enough for a Rust backtrace, bounded memory.
Appended to `ProcessDeadError` messages on crash.

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
| Server crash mid-session | Kill process → next call gets `ProcessDeadError` with stderr |
| Notification between request/response | Persistent reader ignores id-less lines, delivers correct response |
| Handshake timeout (2s) | Non-MCP binary → timeout, tools absent, session ok |
| `session_shutdown` graceful | `shutdown` request sent, exit notification, process killed |
| `session_start` idempotent | Second call no-ops (started guard) |
| Pure functions | `buildRequest`, `parseResponse`, `toolPiName`, `stripPiPrefix`, `extractText`, `formatMcpError`, `withTimeout` — unit-testable via `vitest` |
| Integration | `vitest` spawns real `doctrine serve --mcp` |
| Extension file installed by `doctrine boot install` | Integration: file present, contains baked `BIN_PATH` |

---

## Risks

- **Runtime compatibility**: extension runs under Node.js/jiti (pi runtime);
  tests run under vitest (Node.js). APIs identical — no cross-runtime gap.
- **Process lifecycle**: if pi reloads extensions mid-session (`/reload`), the
  old `session_shutdown` gracefully shuts down the process, new `session_start`
  re-spawns. Expected and correct.
- **`ctx.ui.notify` availability**: RPC mode (`ctx.mode === "rpc"`) supports
  `notify`. Print mode (`-p`) does not — guard with `ctx.hasUI`.
- **`ctx.cwd` semantics**: `--path <ctx.cwd>` assumes pi is started from the
  project root where `.doctrine/` lives. Pi sessions started from subdirectories
  would pass a wrong `--path`. Mitigation: `doctrine serve` auto-detects from
  cwd; explicit `--path` is a robustness belt, not a strict gate.

## Dependencies on SL-119

SL-119 generates `.pi/extensions/doctrine/index.ts` with baked `BIN_PATH`.
SL-120 generates `.pi/extensions/doctrine/mcp.ts` — same directory, same
`install_refresh` call site, same `current_exe()` pattern. The two
extensions are independent files; either can exist without the other.

Deliberate `BIN_PATH` duplication: each file is self-contained so either can be
installed without the other. No import coupling.

## Assumptions

- `doctrine serve --mcp` binary is the same `current_exe()` used by
  `doctrine boot install` (standard install)
- `child_process.spawn` stdio pipes work in the bubblewrap jail
- MCP protocol version 2024-11-05
- `ctx.cwd` is the project root (where `.doctrine/` lives)

## Adversarial review findings

All resolved — integrated into design.

| # | Severity | Finding | Disposition |
|---|----------|---------|-------------|
| F-1 | blocker | Response-id not validated; notifications could break one-read-per-call | Fixed — persistent reader dispatches by id, ignores notifications |
| F-2 | blocker | EPIPE detection wrong in Node.js (write doesn't throw synchronously) | Fixed — `proc.on("error")` + `proc.on("exit")` listeners, `assertProcAlive()` guard, write callback error handling |
| F-3 | blocker | `withTimeout` undefined | Fixed — implemented + `TimeoutError` class |
| F-4 | major | Scope-design divergence on `.mcp.json` reading | Acknowledged — design note explains deliberate divergence |
| F-5 | major | File path diverges (doctrine-mcp.ts vs doctrine/mcp.ts) | Design path is correct; scope doc to be updated |
| F-6 | major | `readOneLine` creates fresh interface per call, risks buffer leakage | Fixed — persistent `readline` interface for process lifetime |
| F-7 | major | No idempotent re-registration guard | Fixed — `started` guard on `session_start` |
| F-8 | major | No graceful MCP shutdown | Fixed — `shutdown()` sends `shutdown` request, exit notification, kill fallback |
| F-9 | major | Pure functions declared but never integrated | Fixed — `buildRequest`, `parseResponse`, `toolPiName`, `extractText` integrated into code samples |
| F-10 | major | `BIN_PATH` duplication without acknowledgment | Documented — deliberate independence, accepted redundancy |
| F-11 | minor | `notifications/initialized` write-without-read hole | Fixed — explicit `assertProcAlive().stdin!.write(...)` shown |
| F-12 | minor | Non-text content produces "undefined" | Fixed — `extractText()` filters by `c.type === "text"` |
| F-13 | minor | `ctx.cwd` semantics undocumented | Documented under Risks |
| F-14 | minor | Design doesn't follow canonical template | Acknowledged — template divergence accepted for this design |
| F-15 | minor | No error listener on spawned process | Fixed — `proc.on("error", ...)` + `proc.on("exit", ...)` |
| F-16 | nit | Stderr ring buffer — 100 lines is a magic number | Fixed — byte-bounded 16KB ring buffer |
| F-17 | nit | Missing open questions section | Acknowledged — none remain after review integration |
| F-18 | nit | Underscore-prefix naming inconsistency | Fixed — consistent prefix usage in updated code |
