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

Codex does not read `.mcp.json`; it reads a TOML `mcp_servers` block in its own
config surface. So the doctrine MCP server (`doctrine serve --mcp`, server id
`doctrine-mcp`, 10 review tools) is still unreachable over MCP for codex-driven
sessions.

### Surface confirmed (2026-09-27)

- **Project-local path is `.codex/config.toml`**, and it is real: "User-level
  configuration lives in `~/.codex/config.toml`. You can also add project-scoped
  overrides in `.codex/config.toml` files … Codex loads project-scoped config
  files only when you trust the project"
  (<https://developers.openai.com/codex/config-reference>); "Codex walks from the
  project root to your current [dir]"
  (<https://developers.openai.com/codex/config-advanced>). Merge order is
  built-ins < `~/.codex/config.toml` < project `.codex/config.toml` < profile <
  `-c` flags < env.
- **Shape** (Codex config reference, `mcp_servers.<id>.*` keys):
  `command`, `args`, `env`, `cwd`, `enabled`, `url`, … — i.e. the target entry is
  `[mcp_servers.doctrine]\ncommand = "doctrine"\nargs = ["serve", "--mcp"]`.
- **`codex mcp add` cannot write the project surface.** It has no scope flag;
  the only documented destination is `~/.codex/config.toml` (verified against
  `codex mcp add --help`, codex 0.155.1: options are `-c/--config`, `--env`,
  `--url`, `--enable`, …; there is no `--scope`/`--local`). A manual
  `codex mcp add doctrine -- doctrine serve --mcp` therefore registers *user*-wide
  — the observation that prompted this note. **Install must merge the
  project-local TOML itself**; shelling `codex mcp add` is not an option.
- `-c mcp_servers.doctrine.command=…` (repeated per key) is the only
  CLI-side, file-free alternative; rejected as the install posture (it lives in
  the harness invocation, not in the repo install artefacts).

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

- OQ-1 **Answered →** project-local `.codex/config.toml`, mirroring the Claude
  arm's posture (see *Surface confirmed* above). No global write.
- OQ-4 Trust gate: codex reads project-scoped config only for *trusted*
  projects. Is that a documented install-time caveat (report it in the install
  summary) or a non-issue because a repo running `doctrine install` is already
  the user's own? Also confirm `mcp_servers` is *not* in the set of keys a
  project layer may not override (the reference names machine-local provider /
  auth / telemetry keys — `mcp_servers` does not appear among them, but verify
  against the live docs before relying on it).
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
- <https://developers.openai.com/codex/config-reference> — `mcp_servers.*` keys,
  project-config precedence, trust gate.
- <https://developers.openai.com/codex/config-advanced> — project config walk
  (project root → cwd).
