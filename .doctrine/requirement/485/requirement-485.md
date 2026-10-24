# REQ-485: `serve --mcp` outside a doctrine project — no project root, or a root without `.doctrine/` — completes the MCP handshake rootless: `initialize` succeeds with instructions naming the cause, `tools/list` is empty, and `tools/call` returns a JSON-RPC error carrying the same cause; inside a doctrine project behaviour is unchanged.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
