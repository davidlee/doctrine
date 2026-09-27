# REQ-484: `boot install` registers the doctrine MCP server by a narrow-path, edit-preserving write of `[mcp_servers.doctrine]` into the project `.codex/config.toml` — a portable `sh -c` entry carrying no host absolute path — owning only a shape doctrine has emitted (the wrapper line in the emitted-forms set, `env_vars` absent or exactly the emitted whitelist), refreshing a stale owned entry, preserving foreign keys and tables, and printing the TOML snippet rather than clobbering a file or entry it cannot own; the report states what was written, never that the harness has activated it.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
