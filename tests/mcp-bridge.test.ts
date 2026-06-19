import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { spawn, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

const BIN_PATH = process.env.DOCTRINE_BIN || '/home/david/.cargo/bin/doctrine';

// Check binary exists
const binaryExists = fs.existsSync(BIN_PATH);

// ── Pure function implementations (kept in sync with extension) ──

interface JsonRpcResponse {
  jsonrpc: '2.0';
  id?: number;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

class TimeoutError extends Error {
  constructor(operation: string, ms: number) {
    super(`${operation} timed out after ${ms}ms`);
    this.name = 'TimeoutError';
  }
}

class ProcessDeadError extends Error {
  constructor(exitCode: number | null, stderr: string) {
    super(
      `doctrine process exited with code ${exitCode}${stderr ? '\nstderr:\n' + stderr : ''}`
    );
    this.name = 'ProcessDeadError';
  }
}

function buildRequest(id: number, method: string, params?: unknown): string {
  return JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n';
}

function parseResponse(line: string): JsonRpcResponse {
  return JSON.parse(line) as JsonRpcResponse;
}

function toolPiName(mcpName: string): string {
  return `doctrine_${mcpName}`;
}

function stripPiPrefix(piName: string): string {
  return piName.startsWith('doctrine_') ? piName.slice('doctrine_'.length) : piName;
}

interface McpContent {
  type: string;
  text?: string;
  data?: string;
}

function extractText(content: McpContent[]): string {
  return content
    .filter(
      (c): c is { type: 'text'; text: string } =>
        c.type === 'text' && typeof c.text === 'string'
    )
    .map((c) => c.text)
    .join('\n');
}

function formatMcpError(err: {
  code: number;
  message: string;
  data?: unknown;
}): string {
  const base = `MCP error ${err.code}: ${err.message}`;
  return err.data ? `${base}\ndata: ${JSON.stringify(err.data)}` : base;
}

function withTimeout<T>(
  promise: Promise<T>,
  ms: number,
  label?: string
): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new TimeoutError(label ?? 'operation', ms)),
      ms
    );
    promise.then(
      (v) => {
        clearTimeout(timer);
        resolve(v);
      },
      (e) => {
        clearTimeout(timer);
        reject(e);
      }
    );
  });
}

// ── Unit tests (no process spawn needed — always run) ──

describe('Unit: pure functions', () => {
  it('buildRequest produces valid JSON-RPC line', () => {
    const line = buildRequest(1, 'initialize', { protocolVersion: '2024-11-05' });
    expect(line.endsWith('\n')).toBe(true);
    const parsed = JSON.parse(line.trim());
    expect(parsed.jsonrpc).toBe('2.0');
    expect(parsed.id).toBe(1);
    expect(parsed.method).toBe('initialize');
    expect(parsed.params.protocolVersion).toBe('2024-11-05');
  });

  it('parseResponse parses valid JSON', () => {
    const line = JSON.stringify({ jsonrpc: '2.0', id: 1, result: { ok: true } });
    const resp = parseResponse(line);
    expect(resp.id).toBe(1);
    expect((resp.result as Record<string, unknown>).ok).toBe(true);
  });

  it('parseResponse throws on malformed JSON', () => {
    expect(() => parseResponse('not json')).toThrow();
  });

  it('toolPiName prefixes correctly', () => {
    expect(toolPiName('review_raise')).toBe('doctrine_review_raise');
    expect(toolPiName('review_list')).toBe('doctrine_review_list');
  });

  it('stripPiPrefix removes prefix', () => {
    expect(stripPiPrefix('doctrine_review_raise')).toBe('review_raise');
    expect(stripPiPrefix('review_raise')).toBe('review_raise');
  });

  it('toolPiName / stripPiPrefix round-trip', () => {
    const names = ['review_raise', 'review_list', 'review_show', 'review_dispose'];
    for (const name of names) {
      expect(stripPiPrefix(toolPiName(name))).toBe(name);
    }
  });

  it('extractText filters non-text, joins text', () => {
    const content: McpContent[] = [
      { type: 'text', text: 'Hello' },
      { type: 'image', data: 'base64...' },
      { type: 'text', text: 'World' },
      { type: 'resource' },
    ];
    expect(extractText(content)).toBe('Hello\nWorld');
  });

  it('extractText returns empty string for no text', () => {
    const content: McpContent[] = [{ type: 'image', data: 'base64...' }];
    expect(extractText(content)).toBe('');
  });

  it('extractText handles empty array', () => {
    expect(extractText([])).toBe('');
  });

  it('formatMcpError formats with code + message', () => {
    const err = { code: -32602, message: 'Invalid params' };
    expect(formatMcpError(err)).toBe('MCP error -32602: Invalid params');
  });

  it('formatMcpError includes data when present', () => {
    const err = { code: -32602, message: 'Invalid params', data: { field: 'name' } };
    const formatted = formatMcpError(err);
    expect(formatted).toContain('MCP error -32602: Invalid params');
    expect(formatted).toContain('data: {"field":"name"}');
  });

  it('withTimeout resolves before timeout', async () => {
    const result = await withTimeout(Promise.resolve(42), 1000);
    expect(result).toBe(42);
  });

  it('withTimeout rejects after timeout', async () => {
    const never = new Promise(() => {});
    await expect(withTimeout(never, 10, 'test')).rejects.toThrow(TimeoutError);
    await expect(withTimeout(never, 10, 'test')).rejects.toThrow(
      'test timed out after 10ms'
    );
  });

  it('withTimeout rejects with underlying error', async () => {
    const fail = Promise.reject(new Error('boom'));
    await expect(withTimeout(fail, 1000)).rejects.toThrow('boom');
  });

  it('TimeoutError name is "TimeoutError"', () => {
    const err = new TimeoutError('op', 100);
    expect(err.name).toBe('TimeoutError');
    expect(err.message).toBe('op timed out after 100ms');
  });

  it('ProcessDeadError includes exit code + stderr', () => {
    const err = new ProcessDeadError(1, 'some stderr output');
    expect(err.name).toBe('ProcessDeadError');
    expect(err.message).toContain('exited with code 1');
    expect(err.message).toContain('some stderr output');
  });

  it('ProcessDeadError with null exit code', () => {
    const err = new ProcessDeadError(null, '');
    expect(err.message).toContain('exited with code null');
    expect(err.message).not.toContain('stderr:');
  });
});

// ── Integration tests (spawn real doctrine — skip if binary missing) ──

(binaryExists ? describe : describe.skip)('Integration: doctrine MCP bridge', () => {
  let proc: ChildProcess | null = null;
  const pending = new Map<
    number,
    {
      resolve: (r: JsonRpcResponse) => void;
      reject: (e: Error) => void;
    }
  >();
  let requestId = 0;

  function startMCP(cwd: string): ChildProcess {
    const p = spawn(BIN_PATH, ['serve', '--mcp', '--path', cwd], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    const readline = require('readline').createInterface({
      input: p.stdout!,
    });

    readline.on('line', (line: string) => {
      if (!line.trim()) return;
      let resp: JsonRpcResponse;
      try {
        resp = JSON.parse(line) as JsonRpcResponse;
      } catch {
        return;
      }
      if (resp.id === undefined) return;
      const handler = pending.get(resp.id);
      if (handler) {
        pending.delete(resp.id);
        handler.resolve(resp);
      }
    });

    let stderr = '';
    p.stderr!.on('data', (chunk: Buffer) => {
      stderr += chunk.toString();
    });

    p.on('error', () => {
      const deadErr = new ProcessDeadError(null, stderr);
      for (const [, { reject }] of pending) reject(deadErr);
      pending.clear();
    });

    p.on('exit', (code) => {
      const deadErr = new ProcessDeadError(code, stderr);
      for (const [, { reject }] of pending) reject(deadErr);
      pending.clear();
    });

    return p;
  }

  async function sendRequest(
    method: string,
    params?: unknown,
    timeout?: number
  ): Promise<JsonRpcResponse> {
    if (!proc) throw new Error('Process not started');
    const id = ++requestId;
    const request = buildRequest(id, method, params);

    return new Promise<JsonRpcResponse>((resolve, reject) => {
      const timer = timeout
        ? setTimeout(() => {
            pending.delete(id);
            reject(new TimeoutError(method, timeout));
          }, timeout)
        : null;

      pending.set(id, {
        resolve: (resp: JsonRpcResponse) => {
          if (timer) clearTimeout(timer);
          if (resp.error)
            reject(new Error(formatMcpError(resp.error)));
          else resolve(resp);
        },
        reject: (err: Error) => {
          if (timer) clearTimeout(timer);
          reject(err);
        },
      });

      proc!.stdin!.write(request, (err) => {
        if (err) {
          if (timer) clearTimeout(timer);
          pending.delete(id);
          reject(new Error(`write failed: ${err.message}`));
        }
      });
    });
  }

  let testDir: string;

  beforeAll(() => {
    testDir = fs.mkdtempSync(path.join(os.tmpdir(), 'doctrine-test-'));
    fs.mkdirSync(path.join(testDir, '.doctrine'), { recursive: true });
  });

  afterAll(() => {
    if (proc && !proc.killed) {
      try {
        proc.stdin!.write(
          JSON.stringify({
            jsonrpc: '2.0',
            id: 999,
            method: 'shutdown',
          }) + '\n'
        );
      } catch {
        // pipe may be dead
      }
      proc.kill();
    }
    fs.rmSync(testDir, { recursive: true, force: true });
  });

  it('initialize + tools/list returns 10 tools', async () => {
    proc = startMCP(testDir);

    const initResp = await sendRequest(
      'initialize',
      {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
          name: 'doctrine-pi-mcp',
          version: '1.0.0',
        },
      },
      5000
    );
    const result = initResp.result as {
      capabilities?: { tools?: unknown };
    } | undefined;
    expect(result?.capabilities?.tools).toBeTruthy();

    // Send initialized notification
    proc.stdin!.write(
      JSON.stringify({
        jsonrpc: '2.0',
        method: 'notifications/initialized',
      }) + '\n'
    );

    const toolsResp = await sendRequest('tools/list', undefined, 5000);
    const toolsResult = toolsResp.result as {
      tools?: { name: string; description: string; inputSchema: unknown }[];
    } | undefined;

    expect(toolsResult).toBeDefined();
    expect(toolsResult!.tools).toBeDefined();
    const tools = toolsResult!.tools!;

    expect(tools.length).toBe(10);

    for (const tool of tools) {
      expect(tool.name).toMatch(/^review_/);
    }

    const names = tools.map((t) => t.name);
    expect(names).toContain('review_raise');
    expect(names).toContain('review_list');
    expect(names).toContain('review_show');
    expect(names).toContain('review_dispose');
  });

  it('review_list returns rows', async () => {
    const resp = await sendRequest(
      'tools/call',
      { name: 'review_list', arguments: {} },
      30000
    );
    const result = resp.result as {
      content: { type: string; text?: string }[];
    } | undefined;
    expect(result).toBeDefined();
    expect(result!.content).toBeDefined();
    expect(result!.content.length).toBeGreaterThan(0);

    const text = result!.content
      .filter((c) => c.type === 'text' && typeof c.text === 'string')
      .map((c) => c.text)
      .join('\n');
    expect(text.length).toBeGreaterThan(0);
  });

  it('call with bad params returns MCP error', async () => {
    await expect(
      sendRequest(
        'tools/call',
        { name: 'review_raise', arguments: {} },
        30000
      )
    ).rejects.toThrow('MCP error');
  });

  it('kill process mid-session yields ProcessDeadError on next call', async () => {
    proc!.kill();

    // Wait for exit handler to fire
    await new Promise((r) => setTimeout(r, 200));

    // After process is killed: either sendRequest immediately fails on write
    // (pipe destroyed before exit handler), or exit handler rejects pending.
    // Both are correct error handling paths — just need one to fire.
    let caught: Error | null = null;
    try {
      await sendRequest('tools/list', undefined, 5000);
    } catch (e) {
      caught = e as Error;
    }
    expect(caught).toBeTruthy();
    expect(
      caught!.message.includes('doctrine process exited') ||
        caught!.message.includes('write failed')
    ).toBe(true);
  });

  it('non-MCP binary times out on handshake', async () => {
    // Use 'sleep' which reads nothing from stdin and never responds
    const fakeProc = spawn('sleep', ['10'], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    const readline = require('readline').createInterface({
      input: fakeProc.stdout!,
    });

    const localPending = new Map<
      number,
      { resolve: (r: JsonRpcResponse) => void; reject: (e: Error) => void }
    >();
    let localId = 0;

    readline.on('line', (line: string) => {
      if (!line.trim()) return;
      let resp: JsonRpcResponse;
      try {
        resp = JSON.parse(line) as JsonRpcResponse;
      } catch {
        return;
      }
      if (resp.id === undefined) return;
      const handler = localPending.get(resp.id);
      if (handler) {
        localPending.delete(resp.id);
        handler.resolve(resp);
      }
    });

    const id = ++localId;
    const request = buildRequest(id, 'initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'test', version: '1.0.0' },
    });

    // Write the request — sleep won't respond
    try {
      fakeProc.stdin!.write(request);
    } catch {
      // pipe may close early
    }

    const timeoutPromise = withTimeout(
      new Promise<JsonRpcResponse>((resolve, reject) => {
        localPending.set(id, { resolve, reject });
        // Request already written above — just wait for response that never comes
      }),
      500,
      'initialize'
    );

    await expect(timeoutPromise).rejects.toThrow(TimeoutError);

    fakeProc.kill();
  });
});
