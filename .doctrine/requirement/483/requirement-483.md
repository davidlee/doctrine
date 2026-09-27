# REQ-483: `boot install` registers the doctrine MCP server under `mcpServers.doctrine` in the project `.mcp.json` — the portable `${DOCTRINE_BIN:-doctrine}` invocation, or a legacy absolute-path form it owns — refreshing a stale owned entry and preserving every foreign key and server, and printing a manual-paste snippet rather than clobbering a file or entry it cannot interpret.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
