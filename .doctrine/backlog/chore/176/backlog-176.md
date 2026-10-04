# CHR-176: mcp-bridge test stale tool assertions

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`tests/mcp-bridge.test.ts` predates the current MCP tool set: a stale
`toBe(10)` tool count and an assertion that every tool name starts with
`review_`. SL-278 updated it but could not run it (no vitest in the jail).
Bring the assertions current and run it on a host with vitest.
