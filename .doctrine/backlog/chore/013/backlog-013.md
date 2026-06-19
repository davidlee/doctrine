# CHR-013: doctrine install does not register the MCP server with detected harnesses

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`doctrine serve --mcp` ships an MCP stdio server exposing the 10 review verbs as
JSON-RPC tools (`src/mcp_server/`, `src/commands/serve.rs`; server id
`doctrine-mcp`). But the installer never tells any harness about it. `doctrine
install`'s forward steps are: memory sync → boot wire (@-import + session hooks)
→ skills per agent (`src/install.rs::run_forward_steps`). None registers the MCP
server, and there is no `.mcp.json` / `mcpServers` entry shipped or generated.

Net: an installed project gets the skills/hooks but the review tools are never
reachable over MCP unless the user wires the server by hand.

## Wanted

A forward step (mirroring the existing boot-wire / skills-per-agent legs) that
registers `doctrine serve --mcp` with each detected harness:

- **Claude Code** — project-local `.mcp.json` (or `settings.json` `mcpServers`)
  entry: command `doctrine serve --mcp`. Match the merge/idempotency posture of
  the existing `install_claude_hook` path (wire / refresh / already-current /
  printed-fallback).
- **codex** — its MCP server config surface (confirm shape from codex docs;
  TOML `mcp_servers` block).

## Open questions

- OQ-1 `.mcp.json` vs `settings.json` `mcpServers` for Claude Code — which does
  the install flow already prefer for project-local config? Reuse that seam.
- OQ-2 Global vs project-local: the SubagentStart hook is project-local only
  (`if !args.global`). Should MCP registration follow the same constraint?
- OQ-3 Idempotency / re-install: must be additive no-op like the rest of the
  forward steps. Reuse the harness-merge helpers rather than a parallel writer.
- OQ-4 Does the absolute `doctrine` path need stamping (as the hooks do with
  `exec`), or is bare `doctrine serve --mcp` on PATH sufficient?

## Pointers

- `src/install.rs` — `run_forward_steps`, `print_forward_summary`,
  `install_claude_hook` call site.
- `src/boot.rs` — `resolve_harnesses`, `harness_label`, `wire`, `HookSpec`,
  `install_claude_hook` (the merge-into-settings precedent).
- `src/commands/serve.rs`, `src/mcp_server/mod.rs` (`ServerInfo` id
  `doctrine-mcp`), `src/mcp_server/tools.rs` (the 10 tools).
