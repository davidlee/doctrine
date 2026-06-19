# IMP-111: Codex MCP server registration during install (separate config surface from .mcp.json)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

CHR-013 wired the **Claude** arm of MCP server registration into install: a
project-root `.mcp.json` `mcpServers.doctrine = { command: <abs doctrine>, args:
["serve","--mcp"] }` entry, written via `plan_mcp` / `install_mcp` in
`src/boot.rs`, riding the existing `install_baseref` merge core. The
`Harness::Codex` arm of `install_refresh` deliberately carries
`RefreshOutcome::None` for MCP — codex was out of CHR-013's scope.

Codex does not read `.mcp.json`; it reads its own config surface (a TOML
`mcp_servers` block — confirm exact path/shape from codex docs, e.g.
`~/.codex/config.toml` global vs a project-local `.codex/` surface). So the
doctrine MCP server (`doctrine serve --mcp`, server id `doctrine-mcp`, 10 review
tools) is still unreachable over MCP for codex-driven sessions.

## Wanted

Register the doctrine MCP server with codex during install, mirroring the Claude
arm's posture: idempotent additive merge, no-clobber of a foreign/customised
entry, fail-soft on malformed config, absolute exec path stamped.

- Add a codex MCP planner/installer beside `plan_mcp`/`install_mcp` (or
  generalise the existing core if the merge shape is close enough — watch for a
  parallel implementation: TOML vs JSON differ, so a shared *core* may not pay).
- Flip the `Harness::Codex` arm of `install_refresh` from `None` to the codex MCP
  outcome; report it in `wire()` alongside the Claude line.

## Open questions

- OQ-1 Codex MCP config path: global (`~/.codex/config.toml`) vs project-local.
  Project-local is the install posture CHR-013 chose for Claude; confirm codex
  supports a project-scoped MCP config before committing to it.
- OQ-2 TOML merge: codex config is TOML, not JSON — the `serde_json::Value`
  narrow-path mutate in `plan_mcp` does not transfer. A `toml_edit`-based
  edit-preserving merge is the likely shape (don't clobber comments/other keys).
- OQ-3 Ownership predicate: same shape test as Claude (command file-name
  `doctrine` + args `["serve","--mcp"]`), adapted to codex's entry layout.

## Pointers

- `src/boot.rs` — `install_refresh` (the `Harness::Codex` arm to flip),
  `plan_mcp` / `install_mcp` / `is_doctrine_mcp_entry` / `desired_mcp_entry`
  (the Claude precedent, CHR-013), `MCP_REL` / `MCP_SERVER_KEY`.
- CHR-013 — the Claude arm this extends.
