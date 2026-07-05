# STDIO-shim footgun: wrapping ${DOCTRINE_BIN} serve --mcp breaks fork-spawned MCP servers

**Never point `DOCTRINE_BIN` at a shell wrapper when doctrine MCP is in play.**
`.mcp.json` launches the server as `${DOCTRINE_BIN:-doctrine} serve --mcp` — a
**stdio JSON-RPC** server. A shim in `DOCTRINE_BIN` (even a transparent
`exec "$REAL" "$@"`) breaks the handshake for any **fork-spawned** server: the
protocol is intolerant of wrapper buffering/latency/stray bytes on stdout.

**Why it masquerades as an "isolation limit"** (the trap): the MAIN session's MCP
server is launched ONCE at session start. If the shim is set *after* start, main's
connection keeps working, and non-isolated workflow agents inherit main's live pipe
— so they pass. Only `isolation:'worktree'` forks, which spawn their OWN server via
the `.mcp.json` command in the fork env (now the shim), fail with
`MCP server "doctrine" is not connected`. The asymmetry reads exactly like "forks
can't reach MCP" when the real cause is the wrapper.

**Witnessed 2026-07-05 (CHR-039):** a `DOCTRINE_BIN` logging shim (for observing the
`WorktreeCreate` hook) silently invalidated a whole round of fork write-gate probing;
the granted `memory_retrieve` failed from a fork under the shim but succeeded once
reverted to a real binary. Cost a wrong "isolation limit" inference.

**Rule:** set `DOCTRINE_BIN` to a real executable (`~/.cargo/bin/doctrine` or a
`target/debug/doctrine`), never a script, whenever MCP forks may spawn. If you must
instrument the hook, accept that fork MCP is confounded while the shim is live.
Note: the MCP server DOES honor `DOCTRINE_BIN` (via `.mcp.json`) — so a real dev
binary can front it cleanly. Related:
[[mem.fact.workflow.isolated-fork-reaches-doctrine-mcp]]. Context: RFC-011; findings
`.doctrine/rfc/011/chr-039-findings.md`.
