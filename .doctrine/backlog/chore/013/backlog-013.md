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

## Resolution

Implemented the Claude arm in `src/boot.rs`, riding beside the existing
`install_baseref` / `install_claude_hook` merge cores (no parallel writer):

- `plan_mcp` (pure) + `install_mcp` (imperative read→plan→atomic-write) register
  `mcpServers.doctrine = { command: <abs exec>, args: ["serve","--mcp"] }` in the
  project-root `.mcp.json` (`MCP_REL`, `MCP_SERVER_KEY`). Tools surface as
  `mcp__doctrine__review_*`.
- `is_doctrine_mcp_entry` ownership: command file-name `doctrine` + args exactly
  `["serve","--mcp"]`. Absent → wire; ours+stale-path → refresh; ours+current →
  no-op; foreign/customised `doctrine` key, non-object `mcpServers`, or malformed
  JSON → leave untouched + print manual snippet (no clobber).
- Wired into `install_refresh`'s Claude arm via a new `RefreshReport.mcp` field;
  reported in `wire()`. Covers both `doctrine install` (forward-step boot wire)
  and `doctrine boot install`. Codex arm carries `None`.
- Decisions taken: OQ-1 `.mcp.json` (project-root, committed/team-shared — NOT
  the gitignored `settings.local.json`); OQ-2 project-local by construction
  (root-relative path); OQ-3 idempotent additive merge via the shared core;
  OQ-4 stamp the absolute exec path (refreshable), matching the hook precedent.
- Tests: 8 new in `boot::tests` (pure planner matrix + `is_doctrine_mcp_entry`)
  plus the `install_refresh` integration test extended to assert `.mcp.json`
  write / dry-run skip / idempotent re-run / codex skip.

## Follow-up (not in this chore)

- **Codex MCP registration** — deferred (user scoped this to Claude/`.mcp.json`).
  Codex reads a separate config surface (TOML `mcp_servers` block, not
  `.mcp.json`); wiring it is a separate item. The `Harness::Codex` arm currently
  carries `RefreshOutcome::None` for MCP.
- The forward-step *summary* line (`print_forward_summary`) still reads "wire
  @-import + session hooks"; the per-harness runtime output reports the MCP
  registration accurately. A summary-text refresh is cosmetic, left out.
